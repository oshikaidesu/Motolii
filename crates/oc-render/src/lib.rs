//! oc-render: M1の最小フレーム評価入口。
//!
//! まず固定グラフ(SolidSource -> Overlay(rect) -> Composite(normal))だけを持ち、
//! 評価順・TimeMap・premultiplied alpha契約を1本の関数に束ねる。

use oc_core::{premultiply_rgba_f32, ColorSpace, FrameDesc, PixelFormat, RationalTime, TimeMap};
use oc_gpu::{upload_rgba, GpuCtx};
use oc_nodes::{create_rgba_render_target, CompositeNode, NodeError, OverlayNode, RectOverlay};
use oc_plugin::TextureRef;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolidSource {
    /// UI/APIから渡る色はstraight RGBAとして扱う。
    pub color: [f32; 4],
    pub time_map: TimeMap,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderFrameRequest {
    pub desc: FrameDesc,
    pub timeline_time: RationalTime,
    pub source: SolidSource,
    /// Overlay色もstraight RGBAとして受け取り、OverlayNodeがpremul化する。
    pub overlay: RectOverlay,
}

#[derive(Debug)]
pub struct RenderedFrame {
    pub texture: wgpu::Texture,
    pub desc: FrameDesc,
    pub source_time: RationalTime,
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("render_frame requires an RGBA8 render target")]
    UnsupportedFrameDesc,
    #[error("render_frame output must be premultiplied")]
    OutputMustBePremultiplied,
    #[error(transparent)]
    Node(#[from] NodeError),
}

pub fn render_frame(
    gpu: &GpuCtx,
    request: &RenderFrameRequest,
) -> Result<RenderedFrame, RenderError> {
    validate_render_desc(request.desc)?;

    let source_time = request.source.time_map.map(request.timeline_time);
    let desc = request.desc;

    let background = upload_rgba(gpu, &desc, &solid_rgba(desc, request.source.color));
    let transparent = upload_rgba(gpu, &desc, &solid_rgba(desc, [0.0, 0.0, 0.0, 0.0]));
    let foreground = create_rgba_render_target(gpu, desc, "oc-render-foreground");
    let output = create_rgba_render_target(gpu, desc, "oc-render-output");

    OverlayNode::new(gpu, request.overlay).render(
        gpu,
        TextureRef {
            texture: &transparent,
            desc,
        },
        TextureRef {
            texture: &foreground,
            desc,
        },
    )?;

    CompositeNode::new(gpu).render(
        gpu,
        TextureRef {
            texture: &background,
            desc,
        },
        TextureRef {
            texture: &foreground,
            desc,
        },
        TextureRef {
            texture: &output,
            desc,
        },
    )?;

    Ok(RenderedFrame {
        texture: output,
        desc,
        source_time,
    })
}

fn validate_render_desc(desc: FrameDesc) -> Result<(), RenderError> {
    if desc.format != PixelFormat::Rgba8Unorm || desc.color_space != ColorSpace::Srgb {
        return Err(RenderError::UnsupportedFrameDesc);
    }
    if !desc.premultiplied {
        return Err(RenderError::OutputMustBePremultiplied);
    }
    Ok(())
}

fn solid_rgba(desc: FrameDesc, straight_color: [f32; 4]) -> Vec<u8> {
    let color = premultiply_rgba_f32(straight_color).map(to_u8);
    let mut data = vec![0u8; desc.data_size()];
    for px in data.chunks_exact_mut(4) {
        px.copy_from_slice(&color);
    }
    data
}

fn to_u8(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use oc_core::{Fps, TimeMap};
    use oc_gpu::{download_rgba, GpuCtx};
    use oc_nodes::{CanonicalPoint, CanonicalSize};
    use oc_testkit::{assert_rgba_close, RgbaImageDesc};

    fn gpu_or_skip() -> Option<GpuCtx> {
        match GpuCtx::new_headless() {
            Ok(g) => Some(g),
            Err(e) => {
                eprintln!("SKIP: no GPU adapter: {e}");
                None
            }
        }
    }

    #[test]
    fn render_frame_runs_fixed_overlay_composite_graph() {
        let Some(gpu) = gpu_or_skip() else { return };
        let desc = FrameDesc::packed(8, 4, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
        let request = RenderFrameRequest {
            desc,
            timeline_time: RationalTime::from_frame(6, Fps::new(30, 1)),
            source: SolidSource {
                color: [0.0, 1.0, 0.0, 0.5],
                time_map: TimeMap::constant_speed(
                    RationalTime::from_seconds(1),
                    RationalTime::ZERO,
                    2,
                    1,
                ),
            },
            overlay: RectOverlay {
                center: CanonicalPoint::CENTER,
                size: CanonicalSize {
                    width: 0.5,
                    height: 0.5,
                },
                color: [1.0, 0.0, 0.0, 0.5],
            },
        };

        let rendered = render_frame(&gpu, &request).unwrap();
        assert_eq!(rendered.source_time, RationalTime::new(7, 5));

        let actual = download_rgba(&gpu, &rendered.texture);
        let expected = expected_fixed_graph(desc);
        assert_rgba_close(
            "render-frame-overlay-composite",
            RgbaImageDesc {
                width: desc.width,
                height: desc.height,
            },
            &actual,
            &expected,
            1,
        );
    }

    fn expected_fixed_graph(desc: FrameDesc) -> Vec<u8> {
        let bg = [0u8, 128, 0, 128];
        let fg = [128u8, 0, 0, 128];
        let over = premul_over_u8(bg, fg);
        let mut out = vec![0u8; desc.data_size()];
        for y in 0..desc.height {
            for x in 0..desc.width {
                let inside = (3..5).contains(&x) && (1..3).contains(&y);
                let i = ((y * desc.width + x) * 4) as usize;
                out[i..i + 4].copy_from_slice(if inside { &over } else { &bg });
            }
        }
        out
    }

    fn premul_over_u8(bg: [u8; 4], fg: [u8; 4]) -> [u8; 4] {
        let bg = bg.map(|v| v as f64 / 255.0);
        let fg = fg.map(|v| v as f64 / 255.0);
        let inv_a = 1.0 - fg[3];
        [
            to_u8_f64(fg[0] + bg[0] * inv_a),
            to_u8_f64(fg[1] + bg[1] * inv_a),
            to_u8_f64(fg[2] + bg[2] * inv_a),
            to_u8_f64(fg[3] + bg[3] * inv_a),
        ]
    }

    fn to_u8_f64(v: f64) -> u8 {
        (v.clamp(0.0, 1.0) * 255.0).round() as u8
    }
}
