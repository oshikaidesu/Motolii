//! oc-export: M1の最小書き出しループ。
//!
//! 解析やCLIはまだ持たず、動画フレームをGPUでRGBA化し、oc-renderの共通経路で
//! オーバーレイ合成して、oc-media::Encoderへ流す。

use std::path::Path;

use oc_core::{ColorSpace, FrameDesc, PixelFormat, Quality};
use oc_eval::DataTracks;
use oc_gpu::{GpuCtx, RgbaDownloader, YuvToRgba};
use oc_media::{probe, Encoder, FrameReader};
use oc_nodes::{ParamOverlayError, ParamRectOverlay};
use oc_plugin::TextureRef;
use oc_render::{render_frame_with_background_texture, BackgroundTextureRequest};

#[derive(Debug)]
pub struct ExportOverlayRequest<'a> {
    pub input_path: &'a Path,
    pub output_path: &'a Path,
    pub start_frame: i64,
    /// Noneなら入力ストリーム終端まで書き出す。
    pub frame_count: Option<usize>,
    pub overlay: ParamRectOverlay,
    /// trueなら検証用のほぼロスレスH.264で書く。
    pub qp0: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportReport {
    pub frames_written: usize,
    pub desc: FrameDesc,
    pub fps: oc_core::Fps,
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error(transparent)]
    Media(#[from] oc_media::MediaError),
    #[error(transparent)]
    Render(#[from] oc_render::RenderError),
    #[error(transparent)]
    Gpu(#[from] oc_gpu::GpuRuntimeError),
    #[error(transparent)]
    Overlay(#[from] ParamOverlayError),
}

pub fn export_overlay_video(
    gpu: &GpuCtx,
    request: &ExportOverlayRequest<'_>,
) -> Result<ExportReport, ExportError> {
    assert!(request.start_frame >= 0, "start_frame must be >= 0");

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
    let mut frames_written = 0usize;
    let tracks = DataTracks::new();

    while request
        .frame_count
        .map(|limit| frames_written < limit)
        .unwrap_or(true)
    {
        let Some(frame) = reader.next_frame()? else {
            break;
        };
        let overlay = request.overlay.eval(frame.pts, &tracks)?;
        let background = yuv.convert(gpu, &frame);
        let rendered = render_frame_with_background_texture(
            gpu,
            &BackgroundTextureRequest {
                desc,
                timeline_time: frame.pts,
                source_time: frame.pts,
                background: TextureRef {
                    texture: &background,
                    desc,
                },
                overlay,
            },
            Quality::FINAL,
        )?;
        let rgba = downloader.download(gpu, &rendered.texture)?;
        encoder.write_frame(&rgba)?;
        frames_written += 1;
    }

    encoder.finish()?;
    Ok(ExportReport {
        frames_written,
        desc,
        fps: info.fps,
    })
}
