//! motolii-export: M1の最小書き出しループ。
//!
//! 解析やCLIはまだ持たず、動画フレームをGPUでRGBA化し、motolii-renderの共通経路で
//! オーバーレイ合成して、motolii-media::Encoderへ流す。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use motolii_core::{ColorSpace, FrameDesc, PixelFormat, Quality, RationalTime, TimeMap};
use motolii_doc::{
    build_document_frame_graph, resolve_asset_path, AssetId, Document, EvaluationTime, GraphError,
    TrackItem,
};
use motolii_eval::DataTracks;
use motolii_gpu::{GpuCtx, RgbaDownloader, YuvToRgba};
use motolii_media::{probe, Encoder, FrameReader, MediaInfo};
use motolii_nodes::{ParamOverlayError, ParamRectOverlay};
use motolii_plugin::{reference::register_reference_plugins, PluginRegistry, TextureRef};
use motolii_render::{
    render_frame_with_background_texture, render_graph_cached, BackgroundTextureRequest,
    RenderGraphInputs, RenderSession, TextureId,
};

#[derive(Debug)]
pub struct ExportOverlayRequest<'a> {
    pub input_path: &'a Path,
    pub output_path: &'a Path,
    pub start_frame: i64,
    /// Noneなら入力ストリーム終端まで書き出す。
    pub frame_count: Option<usize>,
    pub overlay: ParamRectOverlay,
    /// ParamDriver等で事前構築したDataTrack集合。
    pub data_tracks: DataTracks,
    /// ソース時刻解決(F-4)。デフォルトは恒等。
    pub time_map: TimeMap,
    /// trueなら検証用のほぼロスレスH.264で書く。
    pub qp0: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportReport {
    pub frames_written: usize,
    pub desc: FrameDesc,
    pub fps: motolii_core::Fps,
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("invalid export request: {0}")]
    InvalidRequest(&'static str),
    #[error(transparent)]
    Media(#[from] motolii_media::MediaError),
    #[error(transparent)]
    Render(#[from] motolii_render::RenderError),
    #[error(transparent)]
    Gpu(#[from] motolii_gpu::GpuRuntimeError),
    #[error(transparent)]
    Overlay(#[from] ParamOverlayError),
    #[error(transparent)]
    Yuv(#[from] motolii_gpu::YuvError),
    #[error(transparent)]
    TimeMap(#[from] motolii_core::TimeMapError),
    #[error(transparent)]
    DocGraph(#[from] GraphError),
    #[error(transparent)]
    Plugin(#[from] motolii_plugin::PluginError),
    #[error(transparent)]
    RationalTime(#[from] motolii_core::RationalTimeError),
    #[error("document has no video source clip")]
    NoVideoSource,
    #[error("multiple video asset clips in one frame")]
    MultipleVideoSources,
    #[error("asset {0} path could not be resolved")]
    UnresolvedAsset(u64),
    #[error("mapped source frame index is negative: {0}")]
    NegativeSourceFrame(i64),
    #[error("video asset {asset} size {got_w}x{got_h} != export {want_w}x{want_h}")]
    VideoDimensionMismatch {
        asset: u64,
        got_w: u32,
        got_h: u32,
        want_w: u32,
        want_h: u32,
    },
}

/// 書き出し設定。Document≠ExportJob(M2E-11⑤)。
#[derive(Debug)]
pub struct ExportJob<'a> {
    pub doc: &'a Document,
    pub output_path: &'a Path,
    pub project_root: Option<&'a Path>,
    pub frame_count: Option<usize>,
    pub qp0: bool,
    pub data_tracks: DataTracks,
}

/// 書き出しループのGPUダウンロード待ち。高負荷下の正当な遅延を許容する。
pub const EXPORT_DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(300);

pub fn export_overlay_video(
    gpu: &GpuCtx,
    request: &ExportOverlayRequest<'_>,
) -> Result<ExportReport, ExportError> {
    if request.start_frame < 0 {
        return Err(ExportError::InvalidRequest("start_frame must be >= 0"));
    }
    request.time_map.validate()?;
    if !request.time_map.is_identity() {
        return Err(ExportError::InvalidRequest(
            "only identity TimeMap is accepted for export until M2; \
             non-identity maps do not affect decode and would silently mis-report source_time",
        ));
    }

    let info = probe(request.input_path)?;
    let mut reader = FrameReader::open(request.input_path, &info, request.start_frame)?;
    let desc = FrameDesc::packed(
        info.width,
        info.height,
        PixelFormat::Rgba8Unorm,
        ColorSpace::Srgb,
        true,
    );
    let mut yuv = YuvToRgba::new(gpu);
    // ステージングバッファを使い回すダウンローダ(performance-model原則3: 毎フレーム確保しない)。
    // 書き出し中は解像度が変わらないため、実質初回のみの確保になる。
    let mut downloader = RgbaDownloader::new();
    let mut encoder = Encoder::open(request.output_path, &desc, info.fps, request.qp0)?;
    let mut render_session = RenderSession::new(gpu);
    let mut frames_written = 0usize;
    let tracks = request.data_tracks.clone();
    let mut loop_error: Option<ExportError> = None;

    while request
        .frame_count
        .map(|limit| frames_written < limit)
        .unwrap_or(true)
    {
        let Some(frame) = (match reader.next_frame() {
            Ok(frame) => frame,
            Err(e) => {
                loop_error = Some(e.into());
                break;
            }
        }) else {
            break;
        };

        match (|| -> Result<(), ExportError> {
            let overlay = request.overlay.eval(frame.pts, &tracks)?;
            let background = yuv.convert(gpu, &frame)?;
            let rendered = render_frame_with_background_texture(
                gpu,
                &mut render_session,
                &BackgroundTextureRequest {
                    desc,
                    timeline_time: frame.pts,
                    time_map: request.time_map,
                    background: TextureRef {
                        texture: &background,
                        desc,
                    },
                    overlay,
                },
                Quality::FINAL,
            )?;
            let rgba = downloader.download(gpu, &rendered.texture, EXPORT_DOWNLOAD_TIMEOUT)?;
            encoder.write_frame(&rgba)?;
            Ok(())
        })() {
            Ok(()) => frames_written += 1,
            Err(e) => {
                loop_error = Some(e);
                break;
            }
        }
    }

    // エラー時もfinishを必ず呼び、moovを書いて部分書き出しを再生可能に残す。
    let finish_error = encoder.finish().err().map(ExportError::from);
    if let Some(e) = loop_error {
        return Err(e);
    }
    if let Some(e) = finish_error {
        return Err(e);
    }
    Ok(ExportReport {
        frames_written,
        desc,
        fps: info.fps,
    })
}

pub fn export_document_video(
    gpu: &GpuCtx,
    job: &ExportJob<'_>,
) -> Result<ExportReport, ExportError> {
    let mut registry = PluginRegistry::new();
    register_reference_plugins(&mut registry)?;
    // エンコーダ寸法のみ文書内の動画アセットから決める(デコード束縛ではない)。
    let desc = resolve_export_frame_desc(job.doc, job.project_root)?;
    let timeline_fps = job.doc.composition.fps;
    let mut yuv = YuvToRgba::new(gpu);
    let mut downloader = RgbaDownloader::new();
    let mut encoder = Encoder::open(job.output_path, &desc, timeline_fps, job.qp0)?;
    let mut render_session = RenderSession::new(gpu);
    let tracks = job.data_tracks.clone();
    // AssetId → 開いたままの FrameReader。順方向は next_frame、巻き戻し/切替時のみ reopen。
    let mut readers: HashMap<u64, CachedAssetReader> = HashMap::new();
    let mut frames_written = 0usize;
    let mut loop_error = None;
    while job.frame_count.map(|n| frames_written < n).unwrap_or(true) {
        let timeline_time = match RationalTime::try_from_frame(frames_written as i64, timeline_fps)
        {
            Ok(t) => t,
            Err(e) => {
                loop_error = Some(e.into());
                break;
            }
        };
        if job.frame_count.is_none() && timeline_time >= job.doc.composition.duration {
            break;
        }
        match (|| -> Result<(), ExportError> {
            let built = build_document_frame_graph(
                job.doc,
                EvaluationTime::new(timeline_time),
                desc,
                &tracks,
                &registry,
                job.project_root,
            )?;
            let active = active_video_slot(&built.video_slots)?;
            // 動画が無いフレームは空の video_sources でオーバーレイのみ続行。
            let background = if let Some((_slot_id, asset_id)) = active {
                let cached = ensure_asset_reader(job, asset_id, &mut readers)?;
                if cached.info.width != desc.width || cached.info.height != desc.height {
                    return Err(ExportError::VideoDimensionMismatch {
                        asset: asset_id.get(),
                        got_w: cached.info.width,
                        got_h: cached.info.height,
                        want_w: desc.width,
                        want_h: desc.height,
                    });
                }
                let source_frame = built.source_time.try_to_frame_floor(cached.info.fps)?;
                if source_frame < 0 {
                    return Err(ExportError::NegativeSourceFrame(source_frame));
                }
                let frame = cached.read_at(source_frame)?;
                Some(yuv.convert(gpu, &frame)?)
            } else {
                None
            };
            let video_inputs: Vec<(TextureId, TextureRef<'_>)> = match (active, background.as_ref())
            {
                (Some((slot_id, _)), Some(tex)) => {
                    vec![(slot_id, TextureRef { texture: tex, desc })]
                }
                _ => Vec::new(),
            };
            let rendered = render_graph_cached(
                gpu,
                &mut render_session,
                timeline_time,
                &built.graph,
                &RenderGraphInputs {
                    video_sources: &video_inputs,
                    source_time: Some(built.source_time),
                    plugins: Some(&registry),
                },
                Quality::FINAL,
            )?;
            encoder.write_frame(&downloader.download(
                gpu,
                &rendered.texture,
                EXPORT_DOWNLOAD_TIMEOUT,
            )?)?;
            Ok(())
        })() {
            Ok(()) => frames_written += 1,
            Err(e) => {
                loop_error = Some(e);
                break;
            }
        }
    }
    let finish_error = encoder.finish().err().map(ExportError::from);
    if let Some(e) = loop_error {
        return Err(e);
    }
    if let Some(e) = finish_error {
        return Err(e);
    }
    Ok(ExportReport {
        frames_written,
        desc,
        fps: timeline_fps,
    })
}

/// 同一フレームに複数の異なる動画アセットが同時に居るときだけ Err。
fn active_video_slot(
    slots: &[(TextureId, AssetId)],
) -> Result<Option<(TextureId, AssetId)>, ExportError> {
    let Some((first_tid, first_aid)) = slots.first().copied() else {
        return Ok(None);
    };
    for (_, aid) in slots.iter().skip(1) {
        if *aid != first_aid {
            return Err(ExportError::MultipleVideoSources);
        }
    }
    Ok(Some((first_tid, first_aid)))
}

/// 資産ごとの常駐デコーダ。順方向は ffmpeg を開き直さない。
struct CachedAssetReader {
    path: PathBuf,
    info: MediaInfo,
    reader: FrameReader,
}

impl CachedAssetReader {
    fn open(path: PathBuf, info: MediaInfo, start_frame: i64) -> Result<Self, ExportError> {
        let reader = FrameReader::open(&path, &info, start_frame)?;
        Ok(Self { path, info, reader })
    }

    fn read_at(&mut self, frame_index: i64) -> Result<motolii_core::CpuFrame, ExportError> {
        if frame_index < 0 {
            return Err(ExportError::NegativeSourceFrame(frame_index));
        }
        let next = self.reader.next_frame_index();
        if frame_index < next {
            // 巻き戻し: シーク付きで開き直す。
            self.reader = FrameReader::open(&self.path, &self.info, frame_index)?;
        } else {
            // 順方向: 目的フレーム直前まで読み捨て。
            while self.reader.next_frame_index() < frame_index {
                if self.reader.next_frame()?.is_none() {
                    return Err(ExportError::Media(motolii_media::MediaError::Ffmpeg(
                        format!("frame {frame_index} out of range"),
                    )));
                }
            }
        }
        self.reader.next_frame()?.ok_or_else(|| {
            ExportError::Media(motolii_media::MediaError::Ffmpeg(format!(
                "frame {frame_index} out of range"
            )))
        })
    }
}

fn ensure_asset_reader<'a>(
    job: &ExportJob<'_>,
    asset_id: AssetId,
    readers: &'a mut HashMap<u64, CachedAssetReader>,
) -> Result<&'a mut CachedAssetReader, ExportError> {
    use std::collections::hash_map::Entry;
    match readers.entry(asset_id.get()) {
        Entry::Occupied(e) => Ok(e.into_mut()),
        Entry::Vacant(e) => {
            let asset = job
                .doc
                .assets
                .get(asset_id)
                .ok_or(ExportError::UnresolvedAsset(asset_id.get()))?;
            let path = resolve_asset_path(asset, job.project_root)
                .ok_or(ExportError::UnresolvedAsset(asset_id.get()))?;
            let info = probe(&path)?;
            // 初回は先頭から。実際のフレーム位置は read_at が合わせる。
            let cached = CachedAssetReader::open(path, info, 0)?;
            Ok(e.insert(cached))
        }
    }
}

/// エンコーダ寸法用。文書に登場する動画アセットを走査する(アクティブ時刻は問わない)。
fn resolve_export_frame_desc(
    doc: &Document,
    project_root: Option<&Path>,
) -> Result<FrameDesc, ExportError> {
    let mut found = None;
    for track in &doc.tracks {
        collect_video_assets_from_items(&track.items, &mut found);
    }
    let Some(asset_id) = found else {
        return Err(ExportError::NoVideoSource);
    };
    let asset = doc
        .assets
        .get(asset_id)
        .ok_or(ExportError::UnresolvedAsset(asset_id.get()))?;
    let path = resolve_asset_path(asset, project_root)
        .ok_or(ExportError::UnresolvedAsset(asset_id.get()))?;
    let info = probe(&path)?;
    Ok(FrameDesc::packed(
        info.width,
        info.height,
        PixelFormat::Rgba8Unorm,
        ColorSpace::Srgb,
        true,
    ))
}

fn collect_video_assets_from_items(items: &[TrackItem], found: &mut Option<AssetId>) {
    for item in items {
        match item {
            TrackItem::Clip(clip) => {
                if let motolii_doc::ClipSource::Asset { asset } = clip.source {
                    if found.is_none() {
                        *found = Some(asset);
                    }
                }
            }
            TrackItem::Group(group) => {
                collect_video_assets_from_items(&group.children, found);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;

    use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat, RationalTime, TimeMap};
    use motolii_doc::{
        Asset, Clip, ClipSource, Composition, DocParam, Document, ItemEnvelope, Track, TrackItem,
        RECT_LAYER_SOURCE,
    };
    use motolii_eval::DataTracks;
    use motolii_media::Encoder;
    use motolii_nodes::{CanonicalPoint, CanonicalSize, ParamRectOverlay, RectOverlay};
    use motolii_testkit::{ffmpeg_or_skip, gpu_or_skip, tmp_dir};

    use super::*;

    #[test]
    fn export_rejects_non_identity_time_map() {
        let Some(gpu) = motolii_testkit::gpu_or_skip() else {
            return;
        };
        let err = export_overlay_video(
            &gpu,
            &ExportOverlayRequest {
                input_path: Path::new("missing.mp4"),
                output_path: Path::new("out.mp4"),
                start_frame: 0,
                frame_count: Some(1),
                overlay: ParamRectOverlay::constant(RectOverlay {
                    center: CanonicalPoint::CENTER,
                    size: CanonicalSize {
                        width: 0.5,
                        height: 0.5,
                    },
                    color: [1.0, 0.0, 0.0, 1.0],
                }),
                data_tracks: DataTracks::new(),
                time_map: TimeMap::constant_speed(
                    motolii_core::RationalTime::ZERO,
                    motolii_core::RationalTime::ZERO,
                    2,
                    1,
                )
                .unwrap(),
                qp0: true,
            },
        )
        .unwrap_err();
        assert!(matches!(err, ExportError::InvalidRequest(_)));
    }

    #[test]
    fn active_video_slot_none_when_empty() {
        assert!(matches!(active_video_slot(&[]), Ok(None)));
    }

    #[test]
    fn active_video_slot_rejects_simultaneous_different_assets() {
        let slots = [
            (TextureId(0), AssetId::from_raw(1)),
            (TextureId(1), AssetId::from_raw(2)),
        ];
        assert!(matches!(
            active_video_slot(&slots),
            Err(ExportError::MultipleVideoSources)
        ));
    }

    #[test]
    fn active_video_slot_allows_same_asset_twice() {
        let aid = AssetId::from_raw(7);
        let slots = [(TextureId(0), aid), (TextureId(1), aid)];
        assert_eq!(
            active_video_slot(&slots).unwrap(),
            Some((TextureId(0), aid))
        );
    }

    fn solid_video(path: &Path, w: u32, h: u32, frames: usize, fps: Fps, rgb: [u8; 3]) {
        let desc = FrameDesc::packed(w, h, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false);
        let mut enc = Encoder::open(path, &desc, fps, true).unwrap();
        for _ in 0..frames {
            let mut data = vec![0u8; desc.data_size()];
            for px in data.chunks_exact_mut(4) {
                px.copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
            }
            enc.write_frame(&data).unwrap();
        }
        enc.finish().unwrap();
    }

    #[test]
    fn export_continues_when_video_inactive_then_staggers_assets() {
        if !ffmpeg_or_skip() {
            return;
        }
        let Some(gpu) = gpu_or_skip() else { return };

        let dir = tmp_dir("export-stagger");
        let a_path = dir.join("a.mp4");
        let b_path = dir.join("b.mp4");
        let out = dir.join("out.mp4");
        let fps = Fps::try_new(12, 1).unwrap();
        solid_video(&a_path, 32, 24, 8, fps, [20, 20, 20]);
        solid_video(&b_path, 32, 24, 8, fps, [200, 200, 200]);

        let mut doc = Document::new_v1();
        doc.composition =
            Composition::try_new(32, 24, RationalTime::try_new(12, 12).unwrap(), fps).unwrap();
        let a_id = AssetId::from_raw(0);
        let b_id = AssetId::from_raw(1);
        for (id, name, file) in [(a_id, "a", "a.mp4"), (b_id, "b", "b.mp4")] {
            doc.assets
                .insert(Asset {
                    id,
                    name: name.into(),
                    asset_type: "video/mp4".into(),
                    content_hash: format!("sha256:{name}"),
                    path_absolute: None,
                    path_project_relative: None,
                    file_name: Some(file.into()),
                    size_bytes: None,
                    head_hash: None,
                    tail_hash: None,
                })
                .unwrap();
        }
        let overlay_layer = doc.layers.allocate("overlay").unwrap();
        let a_layer = doc.layers.allocate("va").unwrap();
        let b_layer = doc.layers.allocate("vb").unwrap();
        let track_id = doc.track_ids.allocate("V1").unwrap();

        // 0..4: 矩形のみ(video_slots 空)。4..8: asset A。8..12: asset B。
        let overlay = Clip {
            envelope: ItemEnvelope::new(overlay_layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(12, 12).unwrap(),
            time_map: TimeMap::identity(),
            source: ClipSource::Plugin {
                plugin_id: RECT_LAYER_SOURCE.into(),
                effect_version: 1,
                params: BTreeMap::from([
                    ("center".into(), DocParam::const_vec2([0.0, 0.0])),
                    ("size".into(), DocParam::const_vec2([0.4, 0.4])),
                    ("color".into(), DocParam::const_color([1.0, 0.0, 0.0, 1.0])),
                ]),
                extra: Default::default(),
            },
            path_ops: Vec::new(),
        };
        let clip_a = Clip {
            envelope: ItemEnvelope::new(a_layer),
            start: RationalTime::try_new(4, 12).unwrap(),
            duration: RationalTime::try_new(4, 12).unwrap(),
            time_map: TimeMap::identity(),
            source: ClipSource::Asset { asset: a_id },
            path_ops: Vec::new(),
        };
        let clip_b = Clip {
            envelope: ItemEnvelope::new(b_layer),
            start: RationalTime::try_new(8, 12).unwrap(),
            duration: RationalTime::try_new(4, 12).unwrap(),
            time_map: TimeMap::identity(),
            source: ClipSource::Asset { asset: b_id },
            path_ops: Vec::new(),
        };
        doc.tracks.push(Track {
            id: track_id,
            items: vec![
                TrackItem::Clip(overlay),
                TrackItem::Clip(clip_a),
                TrackItem::Clip(clip_b),
            ],
        });

        let report = export_document_video(
            &gpu,
            &ExportJob {
                doc: &doc,
                output_path: &out,
                project_root: Some(&dir),
                frame_count: Some(12),
                qp0: true,
                data_tracks: DataTracks::new(),
            },
        )
        .expect("staggered export");
        assert_eq!(report.frames_written, 12);
        std::fs::remove_dir_all(&dir).ok();
    }
}
