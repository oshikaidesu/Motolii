//! motolii-export: M1の最小書き出しループ。
//!
//! 解析やCLIはまだ持たず、動画フレームをGPUでRGBA化し、motolii-renderの共通経路で
//! オーバーレイ合成して、motolii-media::Encoderへ流す。

use std::path::Path;
use std::time::Duration;

use motolii_core::{ColorSpace, FrameDesc, PixelFormat, Quality, TimeMap};
use motolii_eval::DataTracks;
use motolii_gpu::{GpuCtx, RgbaDownloader, YuvToRgba};
use motolii_media::{probe, Encoder, FrameReader};
use motolii_nodes::{ParamOverlayError, ParamRectOverlay};
use motolii_plugin::TextureRef;
use motolii_render::{
    render_frame_with_background_texture, BackgroundTextureRequest, RenderSession,
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use motolii_core::TimeMap;
    use motolii_eval::DataTracks;
    use motolii_nodes::{CanonicalPoint, CanonicalSize, ParamRectOverlay, RectOverlay};

    use super::*;

    #[test]
    fn export_rejects_non_identity_time_map() {
        let Ok(gpu) = GpuCtx::new_headless() else {
            eprintln!("SKIP: no GPU adapter");
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
