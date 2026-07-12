//! motolii-export: M1の最小書き出しループ。
//!
//! 解析やCLIはまだ持たず、動画フレームをGPUでRGBA化し、motolii-renderの共通経路で
//! オーバーレイ合成して、motolii-media::Encoderへ流す。

use std::path::{Path, PathBuf};
use std::time::Duration;

use motolii_core::{ColorSpace, FrameDesc, PixelFormat, Quality, RationalTime, TimeMap};
use motolii_doc::{
    build_document_frame_graph, resolve_asset_path, Document, EvaluationTime, GraphError,
};
use motolii_eval::DataTracks;
use motolii_gpu::{GpuCtx, RgbaDownloader, YuvToRgba};
use motolii_media::{probe, read_frame_at, Encoder, FrameReader};
use motolii_nodes::{ParamOverlayError, ParamRectOverlay};
use motolii_plugin::{reference::register_reference_plugins, PluginRegistry, TextureRef};
use motolii_render::{
    render_frame_with_background_texture, render_graph_cached, BackgroundTextureRequest,
    RenderGraphInputs, RenderSession,
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
    let (video_path, asset_id) = find_primary_video(job.doc, job.project_root)?;
    let info = probe(&video_path)?;
    let desc = FrameDesc::packed(
        info.width,
        info.height,
        PixelFormat::Rgba8Unorm,
        ColorSpace::Srgb,
        true,
    );
    // 書き出しループはタイムライン尺主導。デコード位置はグラフの source_time(TimeMap済み)。
    let timeline_fps = job.doc.composition.fps;
    let mut yuv = YuvToRgba::new(gpu);
    let mut downloader = RgbaDownloader::new();
    let mut encoder = Encoder::open(job.output_path, &desc, timeline_fps, job.qp0)?;
    let mut render_session = RenderSession::new(gpu);
    let tracks = job.data_tracks.clone();
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
            if built.video_slots.is_empty() {
                return Err(ExportError::NoVideoSource);
            }
            for (_, aid) in &built.video_slots {
                if aid.get() != asset_id {
                    return Err(ExportError::MultipleVideoSources);
                }
            }
            let source_frame = built.source_time.try_to_frame_floor(info.fps)?;
            if source_frame < 0 {
                return Err(ExportError::NegativeSourceFrame(source_frame));
            }
            let frame = read_frame_at(&video_path, &info, source_frame)?;
            let background = yuv.convert(gpu, &frame)?;
            let video_inputs: Vec<_> = built
                .video_slots
                .iter()
                .map(|(tid, _)| {
                    (
                        *tid,
                        TextureRef {
                            texture: &background,
                            desc,
                        },
                    )
                })
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

fn find_primary_video(
    doc: &Document,
    project_root: Option<&Path>,
) -> Result<(PathBuf, u64), ExportError> {
    for track in &doc.tracks {
        for item in &track.items {
            if let motolii_doc::TrackItem::Clip(clip) = item {
                if let motolii_doc::ClipSource::Asset { asset } = clip.source {
                    let a = doc
                        .assets
                        .get(asset)
                        .ok_or(ExportError::UnresolvedAsset(asset.get()))?;
                    let path = resolve_asset_path(a, project_root)
                        .ok_or(ExportError::UnresolvedAsset(asset.get()))?;
                    return Ok((path, asset.get()));
                }
            }
        }
    }
    Err(ExportError::NoVideoSource)
}

#[cfg(test)]
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
}
