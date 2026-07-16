//! motolii-export: M1の最小書き出しループ + D3 Document直結書き出し + D6/AG-4音声mux。
//!
//! ProjectV1 経路の `ExportOverlayRequest` は M1 互換のため残す。
//! Document 書き出しは `ExportJob` → `build_document_frame_graph` → `render_graph_cached`
//! で直結し、ExportOverlayRequest ミラーを作らない(M2E-11⑤)。
//! D6/AG-4: 単一未加工Soundtrackはstream-copy経路、Clip audio等があれば
//! `AudioProgram::mix_audio` の正準PCMをAAC encodeしてmuxする。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use motolii_audio::{AudioProgram, CANONICAL_SAMPLE_RATE};
use motolii_core::{ColorSpace, FrameDesc, PixelFormat, Quality, RationalTime, TimeMap};
use motolii_doc::{
    build_document_frame_graph, resolve_asset_path, AssetId, ClipSource, Document, EvaluationTime,
    GraphError, PluginOpenWarning, TrackItem,
};
use motolii_eval::DataTracks;
use motolii_gpu::{GpuCtx, RgbaDownloader, YuvToRgba};
use motolii_media::{
    mux_mixed_pcm, mux_soundtrack, probe, write_f32le_wav_stereo_48k, Encoder, FrameReader,
    MediaInfo, MixedPcmMuxRequest, SoundtrackMuxRequest,
};
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
    #[error("video asset has no decodable frames")]
    EmptyVideoAsset,
    /// 実装ガード9 / D1f接続: 開くは警告、書き出しは拒否。
    #[error("export refused: document has degraded plugins (open-only warnings are not allowed on export)")]
    DegradedPlugins(Vec<PluginOpenWarning>),
    #[error(transparent)]
    Audio(#[from] motolii_audio::AudioError),
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

/// Document → render graph 直結の書き出し(D3) + 楽曲mux(D6)。
pub fn export_document_video(
    gpu: &GpuCtx,
    job: &ExportJob<'_>,
) -> Result<ExportReport, ExportError> {
    // 実装ガード9: 開く=警告、書き出す=拒否(D1fの接続点。未知pluginは発明しない)。
    // `doc.layer_source.rect` は known_plugin_info で現行versionのみ既知契約。
    // 未来版は FutureVersion 警告のままここに残り、書き出しを拒否する。
    let degraded = job.doc.plugin_open_warnings();
    if !degraded.is_empty() {
        return Err(ExportError::DegradedPlugins(degraded));
    }

    let mut registry = PluginRegistry::new();
    register_reference_plugins(&mut registry)?;
    let desc = resolve_export_frame_desc(job.doc, job.project_root)?;
    let timeline_fps = job.doc.composition.fps;
    let soundtrack = resolve_audio_export(job)?;

    // muxが要るときは映像のみを一時ファイルへ書き、成功後に最終出力へ合成する。
    let video_only_path = match &soundtrack {
        AudioExportPlan::None => None,
        AudioExportPlan::SoundtrackFast { .. } | AudioExportPlan::MixedPcm { .. } => {
            Some(job.output_path.with_extension("video-only.tmp.mp4"))
        }
    };
    let encode_path: &Path = video_only_path.as_deref().unwrap_or(job.output_path);

    let mut yuv = YuvToRgba::new(gpu);
    let mut downloader = RgbaDownloader::new();
    let mut encoder = Encoder::open(encode_path, &desc, timeline_fps, job.qp0)?;
    let mut render_session = RenderSession::new(gpu);
    let tracks = job.data_tracks.clone();
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
            // スロットごとに独立デコード。テクスチャ寿命をループ末まで延ばす。
            let mut backgrounds = Vec::with_capacity(built.video_slots.len());
            for slot in &built.video_slots {
                let cached = ensure_asset_reader(job, slot.asset, &mut readers)?;
                if cached.info.width != desc.width || cached.info.height != desc.height {
                    return Err(ExportError::VideoDimensionMismatch {
                        asset: slot.asset.get(),
                        got_w: cached.info.width,
                        got_h: cached.info.height,
                        want_w: desc.width,
                        want_h: desc.height,
                    });
                }
                let source_frame = freeze_source_frame(slot.source_time, &cached.info)?;
                let frame = cached.read_at(source_frame)?;
                backgrounds.push(yuv.convert(gpu, &frame)?);
            }
            let video_inputs: Vec<(TextureId, TextureRef<'_>)> = built
                .video_slots
                .iter()
                .zip(backgrounds.iter())
                .map(|(slot, tex)| (slot.texture_id, TextureRef { texture: tex, desc }))
                .collect();
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
        if !matches!(soundtrack, AudioExportPlan::None) {
            let _ = std::fs::remove_file(encode_path);
        }
        return Err(e);
    }
    if let Some(e) = finish_error {
        if !matches!(soundtrack, AudioExportPlan::None) {
            let _ = std::fs::remove_file(encode_path);
        }
        return Err(e);
    }

    match soundtrack {
        AudioExportPlan::None => {}
        AudioExportPlan::SoundtrackFast(audio) => {
            let mux_result = mux_soundtrack(&SoundtrackMuxRequest {
                video_path: encode_path,
                audio_path: &audio.path,
                output_path: job.output_path,
                start_offset: audio.start_offset,
                master_gain: audio.master_gain,
            });
            let _ = std::fs::remove_file(encode_path);
            mux_result?;
        }
        AudioExportPlan::MixedPcm { frame_count } => {
            let pcm_path = job.output_path.with_extension("mixed.tmp.wav");
            let mux_result = (|| -> Result<(), ExportError> {
                let mut caches = HashMap::new();
                let program = AudioProgram::from_document(job.doc, job.project_root, &mut caches)?;
                // chunk境界で結果が変わらないことを保証するため、固定chunkで連結する。
                let mut pcm = Vec::with_capacity(frame_count.saturating_mul(2));
                let mut start = 0u64;
                let total = frame_count as u64;
                const CHUNK: usize = 48_000; // 1秒
                while start < total {
                    let n = ((total - start) as usize).min(CHUNK);
                    let (chunk, _) = program.mix_audio(start, n, None)?;
                    pcm.extend_from_slice(&chunk);
                    start += n as u64;
                }
                write_f32le_wav_stereo_48k(&pcm_path, &pcm)?;
                mux_mixed_pcm(&MixedPcmMuxRequest {
                    video_path: encode_path,
                    pcm_wav_path: &pcm_path,
                    output_path: job.output_path,
                })?;
                Ok(())
            })();
            let _ = std::fs::remove_file(encode_path);
            let _ = std::fs::remove_file(&pcm_path);
            mux_result?;
        }
    }

    Ok(ExportReport {
        frames_written,
        desc,
        fps: timeline_fps,
    })
}

enum AudioExportPlan {
    None,
    /// 単一未加工Soundtrack → 既存 D6 stream-copy/AAC 経路。
    SoundtrackFast(ResolvedSoundtrack),
    /// Clip audio / 複数source / gain automation / retime → mixしてAAC。
    MixedPcm {
        frame_count: usize,
    },
}

struct ResolvedSoundtrack {
    path: PathBuf,
    start_offset: RationalTime,
    master_gain: f64,
}

/// AG-4: fast pathは「可聴sourceがSoundtrackのみ」かつClip側に加工音声が無いときだけ。
fn resolve_audio_export(job: &ExportJob<'_>) -> Result<AudioExportPlan, ExportError> {
    let has_clip_audio = document_has_enabled_clip_audio(job.doc);
    let has_clip_retime = document_has_non_identity_clip_audio_retime(job.doc);

    if has_clip_audio || has_clip_retime {
        let frames = composition_canonical_frames(job.doc.composition.duration)?;
        return Ok(AudioExportPlan::MixedPcm {
            frame_count: frames,
        });
    }

    let Some(st) = job.doc.soundtrack else {
        return Ok(AudioExportPlan::None);
    };
    let asset = job
        .doc
        .assets
        .get(st.asset)
        .ok_or(ExportError::UnresolvedAsset(st.asset.get()))?;
    let path = resolve_asset_path(asset, job.project_root)
        .ok_or(ExportError::UnresolvedAsset(st.asset.get()))?;
    Ok(AudioExportPlan::SoundtrackFast(ResolvedSoundtrack {
        path,
        start_offset: st.start_offset,
        master_gain: st.master_gain(),
    }))
}

fn composition_canonical_frames(duration: RationalTime) -> Result<usize, ExportError> {
    if duration <= RationalTime::ZERO {
        return Ok(0);
    }
    let num = duration.num().max(0) as u128;
    let den = duration.den().max(1) as u128;
    let frames = (num * u128::from(CANONICAL_SAMPLE_RATE)) / den;
    Ok(frames as usize)
}

fn document_has_enabled_clip_audio(doc: &Document) -> bool {
    fn walk(items: &[TrackItem]) -> bool {
        for item in items {
            match item {
                TrackItem::Clip(clip) => {
                    if let ClipSource::Asset { audio, .. } = &clip.source {
                        if audio.iter().any(|c| c.enabled) {
                            return true;
                        }
                    }
                }
                TrackItem::Group(g) => {
                    if walk(&g.children) {
                        return true;
                    }
                }
            }
        }
        false
    }
    doc.tracks.iter().any(|t| walk(&t.items))
}

fn document_has_non_identity_clip_audio_retime(doc: &Document) -> bool {
    fn walk(items: &[TrackItem]) -> bool {
        for item in items {
            match item {
                TrackItem::Clip(clip) => {
                    if let ClipSource::Asset { audio, .. } = &clip.source {
                        if audio.iter().any(|c| c.enabled)
                            && (clip.time_map.speed_num() != 1 || clip.time_map.speed_den() != 1)
                        {
                            return true;
                        }
                    }
                }
                TrackItem::Group(g) => {
                    if walk(&g.children) {
                        return true;
                    }
                }
            }
        }
        false
    }
    doc.tracks.iter().any(|t| walk(&t.items))
}

/// Freeze: 素材 available 範囲へクランプ。`nb_frames` が無ければ `duration` から導出。
/// どちらも無ければ呼び出し側の EOF 保持に委ねる(クランプしない)。
pub fn freeze_source_frame(
    source_time: RationalTime,
    info: &MediaInfo,
) -> Result<i64, ExportError> {
    let mut source_frame = source_time.try_to_frame_floor(info.fps)?;
    if source_frame < 0 {
        source_frame = 0;
    }
    if let Some(n) = info.nb_frames {
        if n > 0 && source_frame >= n {
            source_frame = n - 1;
        }
    } else if let Some(duration) = info.duration {
        let end_exclusive = duration.try_to_frame_floor(info.fps)?;
        if end_exclusive > 0 && source_frame >= end_exclusive {
            source_frame = end_exclusive - 1;
        }
    }
    Ok(source_frame)
}

struct CachedAssetReader {
    path: PathBuf,
    info: MediaInfo,
    reader: FrameReader,
    /// Freeze: EOF 到達後も最後に読めたフレームを保持する。
    last_frame: Option<motolii_core::CpuFrame>,
}

impl CachedAssetReader {
    fn open(path: PathBuf, info: MediaInfo, start_frame: i64) -> Result<Self, ExportError> {
        let reader = FrameReader::open(&path, &info, start_frame)?;
        Ok(Self {
            path,
            info,
            reader,
            last_frame: None,
        })
    }

    fn read_at(&mut self, frame_index: i64) -> Result<motolii_core::CpuFrame, ExportError> {
        if frame_index < 0 {
            return Err(ExportError::NegativeSourceFrame(frame_index));
        }
        let next = self.reader.next_frame_index();
        if frame_index < next {
            self.reader = FrameReader::open(&self.path, &self.info, frame_index)?;
            self.last_frame = None;
        }
        while self.reader.next_frame_index() < frame_index {
            match self.reader.next_frame()? {
                Some(f) => self.last_frame = Some(f),
                None => {
                    // EOF: Freeze = 最終フレーム保持
                    return self.last_frame.clone().ok_or(ExportError::EmptyVideoAsset);
                }
            }
        }
        match self.reader.next_frame()? {
            Some(f) => {
                self.last_frame = Some(f.clone());
                Ok(f)
            }
            None => self.last_frame.clone().ok_or(ExportError::EmptyVideoAsset),
        }
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
            let cached = CachedAssetReader::open(path, info, 0)?;
            Ok(e.insert(cached))
        }
    }
}

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
                // audio-only(video: None)は解像度候補にしない。ordinal≠0はvalidate拒否済みでも
                // export単体経路で黙ってv:0へ落とさないようスキップする(AG-1 review)。
                if let motolii_doc::ClipSource::Asset {
                    asset,
                    video: Some(video),
                    ..
                } = &clip.source
                {
                    if video.stream.ordinal == 0 && found.is_none() {
                        *found = Some(*asset);
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
#[allow(deprecated)]
mod tests {
    use std::path::Path;

    use motolii_core::TimeMap;
    use motolii_eval::DataTracks;
    use motolii_nodes::{CanonicalPoint, CanonicalSize, ParamRectOverlay, RectOverlay};

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
                time_map: TimeMap::constant_speed(motolii_core::RationalTime::ZERO, 2, 1).unwrap(),
                qp0: true,
            },
        )
        .unwrap_err();
        assert!(matches!(err, ExportError::InvalidRequest(_)));
    }

    #[test]
    fn collect_video_assets_skips_leading_audio_only_clip() {
        use motolii_doc::{
            AudioComponent, Clip, ClipSource, Document, ItemEnvelope, Track, TrackItem,
            VideoComponent,
        };

        let mut doc = Document::new_v1();
        let audio_asset = doc.assets.allocate("sfx", "audio/wav", "h-audio").unwrap();
        let video_asset = doc.assets.allocate("bg", "video/mp4", "h-video").unwrap();
        let audio_layer = doc.layers.allocate("audio").unwrap();
        let video_layer = doc.layers.allocate("video").unwrap();
        let track = doc.track_ids.allocate("V1").unwrap();
        doc.tracks.push(Track {
            id: track,
            items: vec![
                TrackItem::Clip(Clip {
                    envelope: ItemEnvelope::new(audio_layer),
                    start: RationalTime::ZERO,
                    duration: RationalTime::try_new(1, 1).unwrap(),
                    time_map: TimeMap::default(),
                    source: ClipSource::Asset {
                        asset: audio_asset,
                        video: None,
                        audio: vec![AudioComponent::ordinal(0)],
                    },
                }),
                TrackItem::Clip(Clip {
                    envelope: ItemEnvelope::new(video_layer),
                    start: RationalTime::ZERO,
                    duration: RationalTime::try_new(1, 1).unwrap(),
                    time_map: TimeMap::default(),
                    source: ClipSource::Asset {
                        asset: video_asset,
                        video: Some(VideoComponent::ordinal(0)),
                        audio: Vec::new(),
                    },
                }),
            ],
        });

        let mut found = None;
        collect_video_assets_from_items(&doc.tracks[0].items, &mut found);
        assert_eq!(
            found,
            Some(video_asset),
            "leading audio-only clip must not become the export resolution source"
        );

        // 順序を入れ替えても同じvideo assetを選ぶ。
        doc.tracks[0].items.swap(0, 1);
        let mut found_swapped = None;
        collect_video_assets_from_items(&doc.tracks[0].items, &mut found_swapped);
        assert_eq!(found_swapped, Some(video_asset));
    }

    #[test]
    fn freeze_source_frame_uses_duration_when_nb_frames_missing() {
        use motolii_core::Fps;
        use motolii_media::MediaInfo;

        let info = MediaInfo {
            width: 16,
            height: 8,
            fps: Fps::try_new(30, 1).unwrap(),
            duration: Some(RationalTime::try_new(1, 1).unwrap()),
            nb_frames: None,
            color_space: ColorSpace::Srgb,
            rotation: 0,
        };
        // 1s @ 30fps → end_exclusive=30、frame 40 は 29 へクランプ。
        let t = RationalTime::try_from_frame(40, info.fps).unwrap();
        assert_eq!(freeze_source_frame(t, &info).unwrap(), 29);
        let early = RationalTime::try_from_frame(5, info.fps).unwrap();
        assert_eq!(freeze_source_frame(early, &info).unwrap(), 5);
    }

    #[test]
    fn freeze_source_frame_prefers_nb_frames() {
        use motolii_core::Fps;
        use motolii_media::MediaInfo;

        let info = MediaInfo {
            width: 16,
            height: 8,
            fps: Fps::try_new(24, 1).unwrap(),
            duration: Some(RationalTime::try_new(10, 1).unwrap()),
            nb_frames: Some(10),
            color_space: ColorSpace::Srgb,
            rotation: 0,
        };
        let t = RationalTime::try_from_frame(100, info.fps).unwrap();
        assert_eq!(freeze_source_frame(t, &info).unwrap(), 9);
    }
}
