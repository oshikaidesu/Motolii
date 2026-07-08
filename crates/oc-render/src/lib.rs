//! oc-render: M1の最小フレーム評価入口。
//!
//! まず固定グラフ(SolidSource -> Overlay(rect) -> Composite(normal))だけを持ち、
//! 評価順・TimeMap・premultiplied alpha契約を1本の関数に束ねる。

use oc_core::{premultiply_rgba_f32, ColorSpace, FrameDesc, PixelFormat, RationalTime, TimeMap};
use oc_gpu::{upload_rgba, GpuCtx};
use oc_nodes::{create_rgba_render_target, CompositeNode, NodeError, OverlayNode, RectOverlay};
use oc_plugin::TextureRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolidSource {
    /// UI/APIから渡る色はstraight RGBAとして扱う。
    pub color: [f32; 4],
    pub time_map: TimeMap,
    /// RenderedFrame.source_timeへ反映する代表sourceかどうか。
    pub reports_source_time: bool,
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

#[derive(Debug, Clone, PartialEq)]
pub struct LinearRenderGraph {
    pub desc: FrameDesc,
    pub steps: Vec<RenderStep>,
    pub output: TextureId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderStep {
    SolidSource {
        output: TextureId,
        source: SolidSource,
    },
    OverlayRect {
        input: TextureId,
        output: TextureId,
        overlay: RectOverlay,
    },
    CompositeNormal {
        background: TextureId,
        foreground: TextureId,
        output: TextureId,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("render_frame requires an RGBA8 render target")]
    UnsupportedFrameDesc,
    #[error("render_frame output must be premultiplied")]
    OutputMustBePremultiplied,
    #[error("render graph has no texture for id {0}")]
    MissingTexture(usize),
    #[error("render graph has no source step")]
    MissingSource,
    #[error("render graph has no OverlayRect step")]
    MissingOverlay,
    #[error("render graph has no CompositeNormal step")]
    MissingCompositeNormal,
    #[error("render graph overlay rect count must be exactly 1, found {found}")]
    InvalidOverlayRectCount { found: usize },
    #[error("render graph composite normal count must be exactly 1, found {found}")]
    InvalidCompositeNormalCount { found: usize },
    #[error("overlay rect input id {input} must be produced by a transparent SolidSource (a=0.0)")]
    OverlayInputMustBeTransparentPrefill { input: usize },
    #[error("composite foreground id {foreground} must be produced by OverlayRect output")]
    CompositeForegroundMustComeFromOverlay { foreground: usize },
    #[error("composite background id {background} must be produced by SolidSource")]
    CompositeBackgroundMustComeFromSolid { background: usize },
    #[error("render graph output id {output} must be produced by CompositeNormal")]
    OutputMustBeProducedByCompositeNormal { output: usize },
    #[error("render graph has multiple reporting source steps")]
    MultipleReportingSources,
    #[error("render graph texture id {0} is not in compact order")]
    NonCompactTextureId(usize),
    #[error("render graph writes texture id {0} more than once")]
    DuplicateTextureWrite(usize),
    #[error(transparent)]
    Node(#[from] NodeError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TexProducer {
    Solid { transparent: bool },
    Overlay,
    Composite,
}

pub fn render_frame(
    gpu: &GpuCtx,
    request: &RenderFrameRequest,
) -> Result<RenderedFrame, RenderError> {
    render_graph(
        gpu,
        request.timeline_time,
        &linear_graph_from_request(request),
    )
}

pub fn render_graph(
    gpu: &GpuCtx,
    timeline_time: RationalTime,
    graph: &LinearRenderGraph,
) -> Result<RenderedFrame, RenderError> {
    validate_render_desc(graph.desc)?;
    let graph_plan = validate_linear_graph(graph, timeline_time)?;

    let desc = graph.desc;
    let mut textures: Vec<Option<wgpu::Texture>> =
        (0..graph_plan.texture_count).map(|_| None).collect();

    for step in &graph.steps {
        match *step {
            RenderStep::SolidSource { output, source } => {
                textures[output.0] = Some(upload_rgba(gpu, &desc, &solid_rgba(desc, source.color)));
            }
            RenderStep::OverlayRect {
                input,
                output,
                overlay,
            } => {
                let input_texture = texture_ref(&textures, input)?;
                let output_texture =
                    create_rgba_render_target(gpu, desc, "oc-render-graph-overlay");
                OverlayNode::new(gpu, overlay).render(
                    gpu,
                    TextureRef {
                        texture: input_texture,
                        desc,
                    },
                    TextureRef {
                        texture: &output_texture,
                        desc,
                    },
                )?;
                textures[output.0] = Some(output_texture);
            }
            RenderStep::CompositeNormal {
                background,
                foreground,
                output,
            } => {
                let background_texture = texture_ref(&textures, background)?;
                let foreground_texture = texture_ref(&textures, foreground)?;
                let output_texture = create_rgba_render_target(gpu, desc, "oc-render-graph-output");
                CompositeNode::new(gpu).render(
                    gpu,
                    TextureRef {
                        texture: background_texture,
                        desc,
                    },
                    TextureRef {
                        texture: foreground_texture,
                        desc,
                    },
                    TextureRef {
                        texture: &output_texture,
                        desc,
                    },
                )?;
                textures[output.0] = Some(output_texture);
            }
        }
    }

    let output_texture = textures
        .get_mut(graph.output.0)
        .and_then(Option::take)
        .ok_or(RenderError::MissingTexture(graph.output.0))?;

    Ok(RenderedFrame {
        texture: output_texture,
        desc,
        source_time: graph_plan.source_time,
    })
}

pub fn linear_graph_from_request(request: &RenderFrameRequest) -> LinearRenderGraph {
    LinearRenderGraph {
        desc: request.desc,
        steps: vec![
            RenderStep::SolidSource {
                output: TextureId(0),
                source: request.source,
            },
            RenderStep::SolidSource {
                output: TextureId(1),
                source: SolidSource {
                    color: [0.0, 0.0, 0.0, 0.0],
                    time_map: TimeMap::identity(),
                    reports_source_time: false,
                },
            },
            RenderStep::OverlayRect {
                input: TextureId(1),
                output: TextureId(2),
                overlay: request.overlay,
            },
            RenderStep::CompositeNormal {
                background: TextureId(0),
                foreground: TextureId(2),
                output: TextureId(3),
            },
        ],
        output: TextureId(3),
    }
}

#[derive(Debug, Clone, Copy)]
struct GraphPlan {
    texture_count: usize,
    source_time: RationalTime,
}

fn validate_linear_graph(
    graph: &LinearRenderGraph,
    timeline_time: RationalTime,
) -> Result<GraphPlan, RenderError> {
    let texture_count = texture_slot_count(graph)?;
    let mut written = vec![false; texture_count];
    let mut source_time = None;
    let mut producer: Vec<Option<TexProducer>> = vec![None; texture_count];
    let mut overlay_count = 0usize;
    let mut composite_count = 0usize;

    for step in &graph.steps {
        match *step {
            RenderStep::SolidSource { output, source } => {
                validate_output(output, &mut written)?;
                producer[output.0] = Some(TexProducer::Solid {
                    transparent: source.color[3] == 0.0,
                });
                if source.reports_source_time {
                    if source_time.is_some() {
                        return Err(RenderError::MultipleReportingSources);
                    }
                    source_time = Some(source.time_map.map(timeline_time));
                }
            }
            RenderStep::OverlayRect { input, output, .. } => {
                validate_input(input, &written)?;
                match producer.get(input.0).and_then(|p| *p) {
                    Some(TexProducer::Solid { transparent: true }) => {}
                    _ => {
                        return Err(RenderError::OverlayInputMustBeTransparentPrefill {
                            input: input.0,
                        })
                    }
                }
                validate_output(output, &mut written)?;
                producer[output.0] = Some(TexProducer::Overlay);
                overlay_count += 1;
            }
            RenderStep::CompositeNormal {
                background,
                foreground,
                output,
            } => {
                validate_input(background, &written)?;
                validate_input(foreground, &written)?;
                match producer.get(foreground.0).and_then(|p| *p) {
                    Some(TexProducer::Overlay) => {}
                    _ => {
                        return Err(RenderError::CompositeForegroundMustComeFromOverlay {
                            foreground: foreground.0,
                        })
                    }
                }
                match producer.get(background.0).and_then(|p| *p) {
                    Some(TexProducer::Solid { .. }) => {}
                    _ => {
                        return Err(RenderError::CompositeBackgroundMustComeFromSolid {
                            background: background.0,
                        })
                    }
                }
                validate_output(output, &mut written)?;
                producer[output.0] = Some(TexProducer::Composite);
                composite_count += 1;
            }
        }
    }

    validate_input(graph.output, &written)?;

    if overlay_count == 0 {
        return Err(RenderError::MissingOverlay);
    }
    if overlay_count != 1 {
        return Err(RenderError::InvalidOverlayRectCount {
            found: overlay_count,
        });
    }
    if composite_count == 0 {
        return Err(RenderError::MissingCompositeNormal);
    }
    if composite_count != 1 {
        return Err(RenderError::InvalidCompositeNormalCount {
            found: composite_count,
        });
    }

    match producer.get(graph.output.0).and_then(|p| *p) {
        Some(TexProducer::Composite) => Ok(()),
        _ => Err(RenderError::OutputMustBeProducedByCompositeNormal {
            output: graph.output.0,
        }),
    }?;

    Ok(GraphPlan {
        texture_count,
        source_time: source_time.ok_or(RenderError::MissingSource)?,
    })
}

fn texture_slot_count(graph: &LinearRenderGraph) -> Result<usize, RenderError> {
    let mut ids: Vec<_> = graph
        .steps
        .iter()
        .flat_map(|step| match *step {
            RenderStep::SolidSource { output, .. } => vec![output.0],
            RenderStep::OverlayRect { input, output, .. } => vec![input.0, output.0],
            RenderStep::CompositeNormal {
                background,
                foreground,
                output,
            } => vec![background.0, foreground.0, output.0],
        })
        .collect();
    ids.push(graph.output.0);
    ids.sort_unstable();
    ids.dedup();

    for (expected, actual) in ids.iter().copied().enumerate() {
        if expected != actual {
            return Err(RenderError::NonCompactTextureId(actual));
        }
    }
    Ok(ids.len())
}

fn validate_input(id: TextureId, written: &[bool]) -> Result<(), RenderError> {
    match written.get(id.0) {
        Some(true) => Ok(()),
        _ => Err(RenderError::MissingTexture(id.0)),
    }
}

fn validate_output(id: TextureId, written: &mut [bool]) -> Result<(), RenderError> {
    let Some(slot) = written.get_mut(id.0) else {
        return Err(RenderError::MissingTexture(id.0));
    };
    if *slot {
        return Err(RenderError::DuplicateTextureWrite(id.0));
    }
    *slot = true;
    Ok(())
}

fn texture_ref(
    textures: &[Option<wgpu::Texture>],
    id: TextureId,
) -> Result<&wgpu::Texture, RenderError> {
    textures
        .get(id.0)
        .and_then(Option::as_ref)
        .ok_or(RenderError::MissingTexture(id.0))
}

#[cfg(test)]
fn render_frame_direct(
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
        let request = centered_request();
        let desc = request.desc;

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

    #[test]
    fn graph_executor_matches_direct_fixed_path() {
        let Some(gpu) = gpu_or_skip() else { return };
        for request in [centered_request(), fractional_edge_request()] {
            let graph_rendered = render_graph(
                &gpu,
                request.timeline_time,
                &linear_graph_from_request(&request),
            )
            .unwrap();
            let direct_rendered = render_frame_direct(&gpu, &request).unwrap();

            let graph_actual = download_rgba(&gpu, &graph_rendered.texture);
            let direct_actual = download_rgba(&gpu, &direct_rendered.texture);
            assert_eq!(graph_rendered.source_time, direct_rendered.source_time);
            assert_rgba_close(
                "graph-matches-direct",
                RgbaImageDesc {
                    width: request.desc.width,
                    height: request.desc.height,
                },
                &graph_actual,
                &direct_actual,
                0,
            );
        }
    }

    #[test]
    fn graph_rejects_missing_dependency() {
        let Some(gpu) = gpu_or_skip() else { return };
        let desc = FrameDesc::packed(4, 4, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
        let graph = LinearRenderGraph {
            desc,
            steps: vec![RenderStep::OverlayRect {
                input: TextureId(1),
                output: TextureId(0),
                overlay: RectOverlay {
                    center: CanonicalPoint::CENTER,
                    size: CanonicalSize {
                        width: 0.5,
                        height: 0.5,
                    },
                    color: [1.0, 0.0, 0.0, 0.5],
                },
            }],
            output: TextureId(0),
        };

        let err = render_graph(&gpu, RationalTime::ZERO, &graph).unwrap_err();
        assert!(matches!(err, RenderError::MissingTexture(1)));
    }

    #[test]
    fn graph_rejects_multiple_reporting_sources() {
        let Some(gpu) = gpu_or_skip() else { return };
        let request = centered_request();
        let mut graph = linear_graph_from_request(&request);
        graph.steps.push(RenderStep::SolidSource {
            output: TextureId(4),
            source: SolidSource {
                color: [0.0, 0.0, 1.0, 0.5],
                time_map: TimeMap::identity(),
                reports_source_time: true,
            },
        });

        let err = render_graph(&gpu, request.timeline_time, &graph).unwrap_err();
        assert!(matches!(err, RenderError::MultipleReportingSources));
    }

    #[test]
    fn graph_rejects_non_compact_texture_ids() {
        let Some(gpu) = gpu_or_skip() else { return };
        let request = centered_request();
        let mut graph = linear_graph_from_request(&request);
        graph.output = TextureId(99);

        let err = render_graph(&gpu, request.timeline_time, &graph).unwrap_err();
        assert!(matches!(err, RenderError::NonCompactTextureId(99)));
    }

    #[test]
    fn graph_rejects_duplicate_writes() {
        let Some(gpu) = gpu_or_skip() else { return };
        let request = centered_request();
        let mut graph = linear_graph_from_request(&request);
        graph.steps.push(RenderStep::SolidSource {
            output: TextureId(0),
            source: SolidSource {
                color: [0.0, 0.0, 1.0, 0.5],
                time_map: TimeMap::identity(),
                reports_source_time: false,
            },
        });

        let err = render_graph(&gpu, request.timeline_time, &graph).unwrap_err();
        assert!(matches!(err, RenderError::DuplicateTextureWrite(0)));
    }

    #[test]
    fn graph_rejects_missing_reporting_source() {
        let Some(gpu) = gpu_or_skip() else { return };
        let mut request = centered_request();
        request.source.reports_source_time = false;
        let graph = linear_graph_from_request(&request);

        let err = render_graph(&gpu, request.timeline_time, &graph).unwrap_err();
        assert!(matches!(err, RenderError::MissingSource));
    }

    #[test]
    fn graph_rejects_overlay_input_not_transparent_prefill() {
        let Some(gpu) = gpu_or_skip() else { return };
        let request = centered_request();
        let mut graph = linear_graph_from_request(&request);

        // Overlay input is TextureId(1) in linear_graph_from_request().
        graph.steps[1] = RenderStep::SolidSource {
            output: TextureId(1),
            source: SolidSource {
                color: [0.0, 0.0, 0.0, 0.25], // should be a=0.0
                time_map: TimeMap::identity(),
                reports_source_time: false,
            },
        };

        let err = render_graph(&gpu, request.timeline_time, &graph).unwrap_err();
        assert!(matches!(
            err,
            RenderError::OverlayInputMustBeTransparentPrefill { input: 1 }
        ));
    }

    #[test]
    fn graph_rejects_composite_foreground_not_from_overlay() {
        let Some(gpu) = gpu_or_skip() else { return };
        let request = centered_request();
        let mut graph = linear_graph_from_request(&request);

        // CompositeNormal.foreground should be OverlayRect output (TextureId(2)).
        graph.steps[3] = RenderStep::CompositeNormal {
            background: TextureId(0),
            foreground: TextureId(0), // wrong
            output: TextureId(3),
        };

        let err = render_graph(&gpu, request.timeline_time, &graph).unwrap_err();
        assert!(matches!(
            err,
            RenderError::CompositeForegroundMustComeFromOverlay { foreground: 0 }
        ));
    }

    fn centered_request() -> RenderFrameRequest {
        RenderFrameRequest {
            desc: FrameDesc::packed(8, 4, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true),
            timeline_time: RationalTime::from_frame(6, Fps::new(30, 1)),
            source: SolidSource {
                color: [0.0, 1.0, 0.0, 0.5],
                time_map: TimeMap::constant_speed(
                    RationalTime::from_seconds(1),
                    RationalTime::ZERO,
                    2,
                    1,
                ),
                reports_source_time: true,
            },
            overlay: RectOverlay {
                center: CanonicalPoint::CENTER,
                size: CanonicalSize {
                    width: 0.5,
                    height: 0.5,
                },
                color: [1.0, 0.0, 0.0, 0.5],
            },
        }
    }

    fn fractional_edge_request() -> RenderFrameRequest {
        RenderFrameRequest {
            desc: FrameDesc::packed(13, 7, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true),
            timeline_time: RationalTime::from_frame(11, Fps::new(24, 1)),
            source: SolidSource {
                color: [0.2, 0.6, 1.0, 0.75],
                time_map: TimeMap::offset(RationalTime::from_seconds(3), RationalTime::ZERO),
                reports_source_time: true,
            },
            overlay: RectOverlay {
                center: CanonicalPoint { x: -0.43, y: 0.31 },
                size: CanonicalSize {
                    width: 0.71,
                    height: 0.38,
                },
                color: [1.0, 0.25, 0.0, 0.4],
            },
        }
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
