//! motolii-render: M1の最小フレーム評価入口。
//!
//! まず固定グラフ(SolidSource -> Overlay(rect) -> Composite(normal))だけを持ち、
//! 評価順・TimeMap・premultiplied alpha契約を1本の関数に束ねる。

use motolii_core::{
    premultiply_rgba_f32, ColorSpace, CompCamera, FrameDesc, PixelFormat, Quality, RationalTime,
    TimeMap, TimeMapError,
};
use motolii_gpu::{upload_rgba, GpuCtx, PipelineCache};
use motolii_nodes::{
    create_rgba_render_target, CompositeNode, NodeError, OverlayNode, RectOverlay,
};
use motolii_plugin::{
    LayerSourceContext, PluginError, PluginId, PluginRegistry, ResolvedParams, TextureRef,
};

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

#[derive(Debug, Clone, Copy)]
pub struct BackgroundTextureRequest<'a> {
    pub desc: FrameDesc,
    pub timeline_time: RationalTime,
    /// ソース時刻は必ずTimeMap経由(F-4)。恒等写像でもこの口を通す。
    pub time_map: TimeMap,
    /// 既にGPU上にある背景RGBAテクスチャ。動画フレームはmotolii-gpuのYUV→RGBA変換後に渡す。
    pub background: TextureRef<'a>,
    /// Overlay色はstraight RGBAとして受け取り、OverlayNodeがpremul化する。
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

#[derive(Debug, Clone, PartialEq)]
pub enum RenderStep {
    /// 動画レイヤー等、呼び出し側が供給するGPUテクスチャ（VideoSourceNode / T8-R4）。
    VideoSource { output: TextureId },
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
    /// PluginRegistry経由の一般ステップ(所見1)。種別はレジストリlookupで決まる。
    Plugin {
        id: PluginId,
        params: ResolvedParams,
        inputs: Vec<TextureId>,
        output: TextureId,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("render_frame requires an RGBA8 render target")]
    UnsupportedFrameDesc,
    #[error("render_frame output must be premultiplied")]
    OutputMustBePremultiplied,
    #[error("render graph external texture id {0} was not provided")]
    MissingVideoSource(usize),
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
    #[error("composite background id {background} must be produced by SolidSource or VideoSource")]
    CompositeBackgroundMustComeFromSolid { background: usize },
    #[error("render graph output id {output} must be produced by CompositeNormal")]
    OutputMustBeProducedByCompositeNormal { output: usize },
    #[error("render graph has multiple reporting source steps")]
    MultipleReportingSources,
    #[error("render graph texture id {0} is not in compact order")]
    NonCompactTextureId(usize),
    #[error("render graph writes texture id {0} more than once")]
    DuplicateTextureWrite(usize),
    #[error("render graph texture id {0} is written but never read (and is not graph output)")]
    UnusedTextureWrite(usize),
    #[error("render graph Plugin step requires RenderGraphInputs.plugins")]
    MissingPluginRegistry,
    #[error("unknown render plugin id: {0}")]
    UnknownPlugin(String),
    #[error("plugin {id} expects {expected} inputs, got {got}")]
    PluginInputCount {
        id: &'static str,
        expected: String,
        got: usize,
    },
    #[error(transparent)]
    Plugin(#[from] PluginError),
    #[error(transparent)]
    Node(#[from] NodeError),
    #[error(transparent)]
    TimeMap(#[from] TimeMapError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TexProducer {
    Solid { transparent: bool },
    VideoSource,
    Overlay,
    Composite,
    Plugin,
}

/// グラフ実行時に呼び出し側から注入するテクスチャとメタデータ。
#[derive(Debug, Clone, Copy, Default)]
pub struct RenderGraphInputs<'a> {
    pub video_sources: &'a [(TextureId, TextureRef<'a>)],
    /// 明示時はグラフ内のreporting SolidSourceを必須にしない。
    pub source_time: Option<RationalTime>,
    /// `RenderStep::Plugin` があるとき必須。レジストリ経由ディスパッチ(所見1)。
    pub plugins: Option<&'a PluginRegistry>,
}

/// シェーダ/パイプラインと中間テクスチャをフレーム間で使い回すセッション。
pub struct RenderSession {
    overlay: OverlayNode,
    composite: CompositeNode,
    pipelines: PipelineCache,
    transparent: Option<(FrameDesc, wgpu::Texture)>,
    /// 単色Solidの再利用(毎フレームuploadしない)。
    solid: Option<(FrameDesc, [f32; 4], wgpu::Texture)>,
    /// 中間RTピンポン2枚(performance-model §3 / M1-T8)。レイヤー数・TextureId数に依存しない。
    ping: Option<PingPongPool>,
}

struct PingPongPool {
    desc: FrameDesc,
    buffers: [wgpu::Texture; 2],
    next: usize,
    /// プールを作り直した回数(テスト用)。
    generations: u64,
}

impl RenderSession {
    pub fn new(gpu: &GpuCtx) -> Self {
        Self {
            overlay: OverlayNode::new(gpu),
            composite: CompositeNode::new(gpu),
            pipelines: PipelineCache::new(),
            transparent: None,
            solid: None,
            ping: None,
        }
    }

    pub fn pipeline_cache(&self) -> &PipelineCache {
        &self.pipelines
    }

    pub fn pipeline_cache_mut(&mut self) -> &mut PipelineCache {
        &mut self.pipelines
    }

    /// ピンポン中間バッファ枚数(未使用なら0、使用中は常に2)。
    pub fn ping_pong_len(&self) -> usize {
        self.ping.as_ref().map(|_| 2).unwrap_or(0)
    }

    pub fn ping_pong_generations(&self) -> u64 {
        self.ping.as_ref().map(|p| p.generations).unwrap_or(0)
    }

    fn transparent_texture(&mut self, gpu: &GpuCtx, desc: FrameDesc) -> &wgpu::Texture {
        if self.transparent.as_ref().map(|(d, _)| *d) != Some(desc) {
            let tex = upload_rgba(gpu, &desc, &solid_rgba(desc, [0.0, 0.0, 0.0, 0.0]));
            self.transparent = Some((desc, tex));
        }
        &self.transparent.as_ref().unwrap().1
    }

    fn solid_texture(&mut self, gpu: &GpuCtx, desc: FrameDesc, color: [f32; 4]) -> &wgpu::Texture {
        let hit = self
            .solid
            .as_ref()
            .is_some_and(|(d, c, _)| *d == desc && *c == color);
        if !hit {
            let tex = upload_rgba(gpu, &desc, &solid_rgba(desc, color));
            self.solid = Some((desc, color, tex));
        }
        &self.solid.as_ref().unwrap().2
    }

    /// 中間レンダターゲットをピンポン2枚から取得(交互)。
    fn acquire_ping(&mut self, gpu: &GpuCtx, desc: FrameDesc) -> wgpu::Texture {
        if self.ping.as_ref().map(|p| p.desc) != Some(desc) {
            let a = create_rgba_render_target(gpu, desc, "motolii-render-ping-a");
            let b = create_rgba_render_target(gpu, desc, "motolii-render-ping-b");
            let generations = self.ping.as_ref().map(|p| p.generations + 1).unwrap_or(1);
            self.ping = Some(PingPongPool {
                desc,
                buffers: [a, b],
                next: 0,
                generations,
            });
        }
        let pool = self.ping.as_mut().unwrap();
        let tex = pool.buffers[pool.next].clone();
        pool.next = 1 - pool.next;
        tex
    }
}

pub fn render_frame(
    gpu: &GpuCtx,
    request: &RenderFrameRequest,
    quality: Quality,
) -> Result<RenderedFrame, RenderError> {
    let mut session = RenderSession::new(gpu);
    render_graph_cached(
        gpu,
        &mut session,
        request.timeline_time,
        &linear_graph_from_request(request),
        &RenderGraphInputs::default(),
        quality,
    )
}

pub fn render_frame_with_background_texture(
    gpu: &GpuCtx,
    session: &mut RenderSession,
    request: &BackgroundTextureRequest<'_>,
    quality: Quality,
) -> Result<RenderedFrame, RenderError> {
    validate_render_desc(request.desc)?;
    let desc = quality.render_desc(request.desc);
    validate_background_desc(desc, request.background.desc)?;
    // 外部背景経路も render_graph 一本化。オーバーレイ形状だけ毎フレーム差し替える。
    let graph = linear_graph_with_video_source(request.desc, request.overlay);
    let source_time = request.time_map.try_map(request.timeline_time)?;
    render_graph_cached(
        gpu,
        session,
        request.timeline_time,
        &graph,
        &RenderGraphInputs {
            video_sources: &[(TextureId(0), request.background)],
            source_time: Some(source_time),
            plugins: None,
        },
        quality,
    )
}

pub fn render_graph(
    gpu: &GpuCtx,
    timeline_time: RationalTime,
    graph: &LinearRenderGraph,
    quality: Quality,
) -> Result<RenderedFrame, RenderError> {
    let mut session = RenderSession::new(gpu);
    render_graph_cached(
        gpu,
        &mut session,
        timeline_time,
        graph,
        &RenderGraphInputs::default(),
        quality,
    )
}

pub fn render_graph_cached(
    gpu: &GpuCtx,
    session: &mut RenderSession,
    timeline_time: RationalTime,
    graph: &LinearRenderGraph,
    inputs: &RenderGraphInputs<'_>,
    quality: Quality,
) -> Result<RenderedFrame, RenderError> {
    validate_render_desc(graph.desc)?;
    let graph_plan = validate_linear_graph(graph, timeline_time, inputs)?;

    // Quality.resolution_scaleのみ実効。正準座標はViewportTransform経由なので半解像度でも見た目比率は保つ。
    let desc = quality.render_desc(graph.desc);
    let mut textures: Vec<Option<wgpu::Texture>> =
        (0..graph_plan.texture_count).map(|_| None).collect();

    for step in &graph.steps {
        match step {
            RenderStep::VideoSource { output } => {
                let (_, tex) = inputs
                    .video_sources
                    .iter()
                    .find(|(id, _)| *id == *output)
                    .ok_or(RenderError::MissingVideoSource(output.0))?;
                // ハンドル(Arc)の複製でスロットに載せ、以降は通常テクスチャと同じ経路にする。
                textures[output.0] = Some(tex.texture.clone());
            }
            RenderStep::SolidSource { output, source } => {
                let texture = if source.color == [0.0, 0.0, 0.0, 0.0] {
                    session.transparent_texture(gpu, desc).clone()
                } else {
                    session.solid_texture(gpu, desc, source.color).clone()
                };
                textures[output.0] = Some(texture);
            }
            RenderStep::OverlayRect {
                input,
                output,
                overlay,
            } => {
                let input_texture = texture_ref(&textures, desc, *input)?;
                let output_texture = session.acquire_ping(gpu, desc);
                session.overlay.set_rect(*overlay);
                session.overlay.render(
                    gpu,
                    input_texture,
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
                let background_texture = texture_ref(&textures, desc, *background)?;
                let foreground_texture = texture_ref(&textures, desc, *foreground)?;
                let output_texture = session.acquire_ping(gpu, desc);
                session.composite.render(
                    gpu,
                    background_texture,
                    foreground_texture,
                    TextureRef {
                        texture: &output_texture,
                        desc,
                    },
                )?;
                textures[output.0] = Some(output_texture);
            }
            RenderStep::Plugin {
                id,
                params,
                inputs: plugin_inputs,
                output,
            } => {
                let registry = inputs.plugins.ok_or(RenderError::MissingPluginRegistry)?;
                let output_texture = session.acquire_ping(gpu, desc);
                let out_ref = TextureRef {
                    texture: &output_texture,
                    desc,
                };
                let mut encoder =
                    gpu.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("motolii-render-plugin"),
                        });
                dispatch_plugin(
                    registry,
                    id,
                    params,
                    plugin_inputs,
                    gpu,
                    &mut session.pipelines,
                    &mut encoder,
                    timeline_time,
                    &textures,
                    desc,
                    out_ref,
                )?;
                gpu.queue.submit([encoder.finish()]);
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

pub fn linear_graph_with_video_source(desc: FrameDesc, overlay: RectOverlay) -> LinearRenderGraph {
    LinearRenderGraph {
        desc,
        steps: vec![
            RenderStep::VideoSource {
                output: TextureId(0),
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
                overlay,
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
    inputs: &RenderGraphInputs<'_>,
) -> Result<GraphPlan, RenderError> {
    let texture_count = texture_slot_count(graph)?;
    let mut written = vec![false; texture_count];
    let mut read = vec![false; texture_count];
    let mut source_time = None;
    let mut producer: Vec<Option<TexProducer>> = vec![None; texture_count];
    let mut overlay_count = 0usize;
    let mut composite_count = 0usize;
    let mut has_plugin = false;

    for step in &graph.steps {
        match step {
            RenderStep::VideoSource { output } => {
                if !inputs.video_sources.iter().any(|(id, _)| *id == *output) {
                    return Err(RenderError::MissingVideoSource(output.0));
                }
                validate_output(*output, &mut written)?;
                producer[output.0] = Some(TexProducer::VideoSource);
            }
            RenderStep::SolidSource { output, source } => {
                validate_output(*output, &mut written)?;
                producer[output.0] = Some(TexProducer::Solid {
                    transparent: source.color[3] == 0.0,
                });
                if source.reports_source_time {
                    if source_time.is_some() {
                        return Err(RenderError::MultipleReportingSources);
                    }
                    source_time = Some(source.time_map.try_map(timeline_time)?);
                }
            }
            RenderStep::OverlayRect { input, output, .. } => {
                mark_read(*input, &mut read)?;
                validate_input(*input, &written)?;
                match producer.get(input.0).and_then(|p| *p) {
                    Some(TexProducer::Solid { transparent: true }) => {}
                    _ => {
                        return Err(RenderError::OverlayInputMustBeTransparentPrefill {
                            input: input.0,
                        })
                    }
                }
                validate_output(*output, &mut written)?;
                producer[output.0] = Some(TexProducer::Overlay);
                overlay_count += 1;
            }
            RenderStep::CompositeNormal {
                background,
                foreground,
                output,
            } => {
                mark_read(*background, &mut read)?;
                mark_read(*foreground, &mut read)?;
                validate_input(*background, &written)?;
                validate_input(*foreground, &written)?;
                match producer.get(foreground.0).and_then(|p| *p) {
                    Some(TexProducer::Overlay) => {}
                    _ => {
                        return Err(RenderError::CompositeForegroundMustComeFromOverlay {
                            foreground: foreground.0,
                        })
                    }
                }
                match producer.get(background.0).and_then(|p| *p) {
                    Some(TexProducer::Solid { .. }) | Some(TexProducer::VideoSource) => {}
                    _ => {
                        return Err(RenderError::CompositeBackgroundMustComeFromSolid {
                            background: background.0,
                        })
                    }
                }
                validate_output(*output, &mut written)?;
                producer[output.0] = Some(TexProducer::Composite);
                composite_count += 1;
            }
            RenderStep::Plugin {
                inputs: plugin_inputs,
                output,
                ..
            } => {
                if inputs.plugins.is_none() {
                    return Err(RenderError::MissingPluginRegistry);
                }
                for input in plugin_inputs {
                    mark_read(*input, &mut read)?;
                    validate_input(*input, &written)?;
                }
                validate_output(*output, &mut written)?;
                producer[output.0] = Some(TexProducer::Plugin);
                has_plugin = true;
            }
        }
    }

    mark_read(graph.output, &mut read)?;
    validate_input(graph.output, &written)?;

    for (id, was_written) in written.iter().enumerate() {
        if *was_written && !read[id] {
            return Err(RenderError::UnusedTextureWrite(id));
        }
    }

    if has_plugin {
        // Plugin経路は固定デモグラフ制約を外す。未使用書き込み検査で誤配線は既に弾く。
        if producer.get(graph.output.0).and_then(|p| *p).is_none() {
            return Err(RenderError::MissingTexture(graph.output.0));
        }
    } else {
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
    }

    Ok(GraphPlan {
        texture_count,
        source_time: inputs
            .source_time
            .or(source_time)
            .ok_or(RenderError::MissingSource)?,
    })
}

fn texture_slot_count(graph: &LinearRenderGraph) -> Result<usize, RenderError> {
    let mut ids: Vec<_> = graph
        .steps
        .iter()
        .flat_map(|step| match step {
            RenderStep::VideoSource { output } => vec![output.0],
            RenderStep::SolidSource { output, .. } => vec![output.0],
            RenderStep::OverlayRect { input, output, .. } => vec![input.0, output.0],
            RenderStep::CompositeNormal {
                background,
                foreground,
                output,
            } => vec![background.0, foreground.0, output.0],
            RenderStep::Plugin { inputs, output, .. } => {
                let mut v: Vec<_> = inputs.iter().map(|id| id.0).collect();
                v.push(output.0);
                v
            }
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

fn dispatch_plugin(
    registry: &PluginRegistry,
    id: &PluginId,
    params: &ResolvedParams,
    plugin_inputs: &[TextureId],
    gpu: &GpuCtx,
    pipelines: &mut PipelineCache,
    encoder: &mut wgpu::CommandEncoder,
    timeline_time: RationalTime,
    textures: &[Option<wgpu::Texture>],
    desc: FrameDesc,
    output: TextureRef<'_>,
) -> Result<(), RenderError> {
    if let Some(filter) = registry.filter(id) {
        let expected = filter.desc().min_inputs..=filter.desc().max_inputs;
        if !expected.contains(&plugin_inputs.len()) {
            return Err(RenderError::PluginInputCount {
                id: id.0,
                expected: format!(
                    "{}..={}",
                    filter.desc().min_inputs,
                    filter.desc().max_inputs
                ),
                got: plugin_inputs.len(),
            });
        }
        // Filter契約は入力テクスチャ1枚。descが0を許しても [0] でpanicしない。
        let Some(&input_id) = plugin_inputs.first() else {
            return Err(RenderError::PluginInputCount {
                id: id.0,
                expected: ">=1 (filter needs a bound input)".into(),
                got: 0,
            });
        };
        let input = texture_ref(textures, desc, input_id)?;
        filter.render(
            gpu,
            pipelines,
            encoder,
            timeline_time,
            params,
            input,
            output,
        )?;
        return Ok(());
    }

    if let Some(composite) = registry.composite(id) {
        let expected = composite.desc().min_inputs..=composite.desc().max_inputs;
        if !expected.contains(&plugin_inputs.len()) {
            return Err(RenderError::PluginInputCount {
                id: id.0,
                expected: format!(
                    "{}..={}",
                    composite.desc().min_inputs,
                    composite.desc().max_inputs
                ),
                got: plugin_inputs.len(),
            });
        }
        let input_refs: Result<Vec<_>, _> = plugin_inputs
            .iter()
            .map(|input| texture_ref(textures, desc, *input))
            .collect();
        composite.render(
            gpu,
            pipelines,
            encoder,
            timeline_time,
            params,
            &input_refs?,
            output,
        )?;
        return Ok(());
    }

    if let Some(layer) = registry.layer_source(id) {
        if !plugin_inputs.is_empty() {
            return Err(RenderError::PluginInputCount {
                id: id.0,
                expected: "0..=0".into(),
                got: plugin_inputs.len(),
            });
        }
        layer.render(
            gpu,
            pipelines,
            encoder,
            timeline_time,
            params,
            LayerSourceContext {
                camera: CompCamera::DEFAULT,
            },
            output,
        )?;
        return Ok(());
    }

    Err(RenderError::UnknownPlugin(id.0.to_string()))
}

fn validate_input(id: TextureId, written: &[bool]) -> Result<(), RenderError> {
    match written.get(id.0) {
        Some(true) => Ok(()),
        _ => Err(RenderError::MissingTexture(id.0)),
    }
}

fn mark_read(id: TextureId, read: &mut [bool]) -> Result<(), RenderError> {
    let Some(slot) = read.get_mut(id.0) else {
        return Err(RenderError::MissingTexture(id.0));
    };
    *slot = true;
    Ok(())
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

fn texture_ref<'a>(
    textures: &'a [Option<wgpu::Texture>],
    desc: FrameDesc,
    id: TextureId,
) -> Result<TextureRef<'a>, RenderError> {
    let texture = textures
        .get(id.0)
        .and_then(Option::as_ref)
        .ok_or(RenderError::MissingTexture(id.0))?;
    Ok(TextureRef { texture, desc })
}

#[cfg(test)]
fn render_frame_direct(
    gpu: &GpuCtx,
    request: &RenderFrameRequest,
    quality: Quality,
) -> Result<RenderedFrame, RenderError> {
    validate_render_desc(request.desc)?;

    let source_time = request.source.time_map.try_map(request.timeline_time)?;
    let desc = quality.render_desc(request.desc);

    let background = upload_rgba(gpu, &desc, &solid_rgba(desc, request.source.color));
    let transparent = upload_rgba(gpu, &desc, &solid_rgba(desc, [0.0, 0.0, 0.0, 0.0]));
    let foreground = create_rgba_render_target(gpu, desc, "motolii-render-foreground");
    let output = create_rgba_render_target(gpu, desc, "motolii-render-output");

    OverlayNode::with_rect(gpu, request.overlay).render(
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

fn validate_background_desc(output: FrameDesc, background: FrameDesc) -> Result<(), RenderError> {
    if background.width != output.width
        || background.height != output.height
        || background.format != PixelFormat::Rgba8Unorm
        || background.color_space != ColorSpace::Srgb
        || !background.premultiplied
    {
        return Err(RenderError::UnsupportedFrameDesc);
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
    use motolii_core::{Fps, Quality, TimeMap};
    use motolii_eval::Value;
    use motolii_gpu::download_rgba;
    use motolii_nodes::{CanonicalPoint, CanonicalSize};
    use motolii_plugin::reference::register_reference_plugins;
    use motolii_testkit::{assert_rgba_close, gpu_or_skip, RgbaImageDesc};

    #[test]
    fn render_frame_runs_fixed_overlay_composite_graph() {
        let Some(gpu) = gpu_or_skip() else { return };
        let request = centered_request();
        let desc = request.desc;

        let rendered = render_frame(&gpu, &request, Quality::FINAL).unwrap();
        assert_eq!(rendered.source_time, RationalTime::new(7, 5));
        assert_eq!(rendered.desc, desc);

        let actual = download_rgba(&gpu, &rendered.texture).unwrap();
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
    fn final_quality_matches_previous_unscaled_golden() {
        let Some(gpu) = gpu_or_skip() else { return };
        let request = centered_request();
        let rendered = render_frame(&gpu, &request, Quality::FINAL).unwrap();
        let actual = download_rgba(&gpu, &rendered.texture).unwrap();
        assert_rgba_close(
            "final-quality-unscaled-golden",
            RgbaImageDesc {
                width: request.desc.width,
                height: request.desc.height,
            },
            &actual,
            &expected_fixed_graph(request.desc),
            1,
        );
    }

    #[test]
    fn draft_quality_renders_half_resolution_without_crashing() {
        let Some(gpu) = gpu_or_skip() else { return };
        let request = centered_request();
        let rendered = render_frame(&gpu, &request, Quality::DRAFT).unwrap();
        assert_eq!(rendered.desc.width, request.desc.width / 2);
        assert_eq!(rendered.desc.height, request.desc.height / 2);
        assert_eq!(rendered.source_time, RationalTime::new(7, 5));

        let actual = download_rgba(&gpu, &rendered.texture).unwrap();
        assert_eq!(
            actual.len(),
            (rendered.desc.width * rendered.desc.height * 4) as usize
        );
        // Draftは厳密一致不要。「何かピクセルが出る」ことのみ保証。
        assert!(actual.iter().any(|&v| v != 0));
    }

    #[test]
    fn graph_executor_matches_direct_fixed_path() {
        let Some(gpu) = gpu_or_skip() else { return };
        for request in [centered_request(), fractional_edge_request()] {
            for quality in [Quality::FINAL, Quality::DRAFT] {
                let graph_rendered = render_graph(
                    &gpu,
                    request.timeline_time,
                    &linear_graph_from_request(&request),
                    quality,
                )
                .unwrap();
                let direct_rendered = render_frame_direct(&gpu, &request, quality).unwrap();

                let graph_actual = download_rgba(&gpu, &graph_rendered.texture).unwrap();
                let direct_actual = download_rgba(&gpu, &direct_rendered.texture).unwrap();
                assert_eq!(graph_rendered.source_time, direct_rendered.source_time);
                assert_eq!(graph_rendered.desc, direct_rendered.desc);
                assert_rgba_close(
                    "graph-matches-direct",
                    RgbaImageDesc {
                        width: graph_rendered.desc.width,
                        height: graph_rendered.desc.height,
                    },
                    &graph_actual,
                    &direct_actual,
                    0,
                );
            }
        }
    }

    #[test]
    fn tint_filter_uses_pipeline_cache_without_recompile() {
        let Some(gpu) = gpu_or_skip() else { return };
        let desc = FrameDesc::packed(16, 8, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
        let mut registry = PluginRegistry::new();
        register_reference_plugins(&mut registry).unwrap();

        let mut params = ResolvedParams::new();
        // 白(1,1,1,1) × tint(0.5, 0, 0, 1) → 暗い赤
        params.insert("color", Value::Color([0.5, 0.0, 0.0, 1.0]));

        let graph = LinearRenderGraph {
            desc,
            steps: vec![
                RenderStep::SolidSource {
                    output: TextureId(0),
                    source: SolidSource {
                        color: [1.0, 1.0, 1.0, 1.0],
                        time_map: TimeMap::identity(),
                        reports_source_time: true,
                    },
                },
                RenderStep::Plugin {
                    id: PluginId("core.filter.tint"),
                    params: params.clone(),
                    inputs: vec![TextureId(0)],
                    output: TextureId(1),
                },
            ],
            output: TextureId(1),
        };

        let mut session = RenderSession::new(&gpu);
        let inputs = RenderGraphInputs {
            plugins: Some(&registry),
            ..RenderGraphInputs::default()
        };

        let rendered = render_graph_cached(
            &gpu,
            &mut session,
            RationalTime::ZERO,
            &graph,
            &inputs,
            Quality::FINAL,
        )
        .unwrap();
        assert_eq!(session.pipeline_cache().misses(), 1);
        assert_eq!(session.pipeline_cache().hits(), 0);

        let _ = render_graph_cached(
            &gpu,
            &mut session,
            RationalTime::ZERO,
            &graph,
            &inputs,
            Quality::FINAL,
        )
        .unwrap();
        assert_eq!(session.pipeline_cache().misses(), 1);
        assert_eq!(session.pipeline_cache().hits(), 1);

        let actual = download_rgba(&gpu, &rendered.texture).unwrap();
        let mut expected = vec![0u8; desc.data_size()];
        for px in expected.chunks_exact_mut(4) {
            px.copy_from_slice(&[128, 0, 0, 255]);
        }
        assert_rgba_close(
            "plugin-tint-pipeline-cache",
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
    fn session_reuses_two_ping_pong_targets_across_frames() {
        let Some(gpu) = gpu_or_skip() else { return };
        let request = centered_request();
        let graph = linear_graph_from_request(&request);
        let mut session = RenderSession::new(&gpu);

        assert_eq!(session.ping_pong_len(), 0);
        for _ in 0..5 {
            let rendered = render_graph_cached(
                &gpu,
                &mut session,
                request.timeline_time,
                &graph,
                &RenderGraphInputs::default(),
                Quality::FINAL,
            )
            .unwrap();
            assert_eq!(rendered.desc, request.desc);
            assert_eq!(session.ping_pong_len(), 2);
            assert_eq!(session.ping_pong_generations(), 1);
        }
    }

    #[test]
    fn filter_dispatch_empty_inputs_returns_error_not_panic() {
        let Some(gpu) = gpu_or_skip() else { return };
        use motolii_gpu::PipelineCache;
        use motolii_plugin::{FilterPlugin, NodeDesc, PluginError};
        use std::sync::OnceLock;

        struct ZeroMinFilter;
        impl FilterPlugin for ZeroMinFilter {
            fn desc(&self) -> &NodeDesc {
                static DESC: OnceLock<NodeDesc> = OnceLock::new();
                DESC.get_or_init(|| NodeDesc {
                    id: PluginId("test.filter.zero_min"),
                    version: 1,
                    display_name: "ZeroMin",
                    category: "Utility",
                    tags: &[],
                    params: vec![],
                    min_inputs: 0,
                    max_inputs: 0,
                })
            }

            fn render(
                &self,
                _gpu: &GpuCtx,
                _pipelines: &mut PipelineCache,
                _encoder: &mut wgpu::CommandEncoder,
                _t: RationalTime,
                _params: &ResolvedParams,
                _input: TextureRef<'_>,
                _output: TextureRef<'_>,
            ) -> Result<(), PluginError> {
                Ok(())
            }
        }
        static ZERO: ZeroMinFilter = ZeroMinFilter;

        let mut registry = PluginRegistry::new();
        registry.register_filter(&ZERO).unwrap();

        let desc = FrameDesc::packed(8, 4, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
        let graph = LinearRenderGraph {
            desc,
            steps: vec![RenderStep::Plugin {
                id: PluginId("test.filter.zero_min"),
                params: ResolvedParams::new(),
                inputs: vec![],
                output: TextureId(0),
            }],
            output: TextureId(0),
        };

        let err = render_graph_cached(
            &gpu,
            &mut RenderSession::new(&gpu),
            RationalTime::ZERO,
            &graph,
            &RenderGraphInputs {
                plugins: Some(&registry),
                source_time: Some(RationalTime::ZERO),
                ..RenderGraphInputs::default()
            },
            Quality::FINAL,
        )
        .unwrap_err();
        assert!(
            matches!(err, RenderError::PluginInputCount { got: 0, .. }),
            "expected PluginInputCount, got {err:?}"
        );
    }

    #[test]
    fn plugin_graph_rejects_unused_texture_write() {
        let Some(gpu) = gpu_or_skip() else { return };
        let desc = FrameDesc::packed(8, 4, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
        let mut registry = PluginRegistry::new();
        register_reference_plugins(&mut registry).unwrap();

        let graph = LinearRenderGraph {
            desc,
            steps: vec![
                RenderStep::SolidSource {
                    output: TextureId(0),
                    source: SolidSource {
                        color: [1.0, 0.0, 0.0, 1.0],
                        time_map: TimeMap::identity(),
                        reports_source_time: true,
                    },
                },
                RenderStep::Plugin {
                    id: PluginId("core.filter.clear"),
                    params: ResolvedParams::new(),
                    inputs: vec![TextureId(0)],
                    output: TextureId(1),
                },
            ],
            // plugin出力を捨てて入力側を返す誤配線
            output: TextureId(0),
        };

        let err = render_graph_cached(
            &gpu,
            &mut RenderSession::new(&gpu),
            RationalTime::ZERO,
            &graph,
            &RenderGraphInputs {
                plugins: Some(&registry),
                ..RenderGraphInputs::default()
            },
            Quality::FINAL,
        )
        .unwrap_err();
        assert!(matches!(err, RenderError::UnusedTextureWrite(1)));
    }

    #[test]
    fn plugin_filter_dispatches_via_registry_golden() {
        let Some(gpu) = gpu_or_skip() else { return };
        let desc = FrameDesc::packed(16, 8, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
        let mut registry = PluginRegistry::new();
        register_reference_plugins(&mut registry).unwrap();

        let mut params = ResolvedParams::new();
        params.insert("color", Value::Color([0.0, 1.0, 0.0, 1.0]));

        let graph = LinearRenderGraph {
            desc,
            steps: vec![
                RenderStep::SolidSource {
                    output: TextureId(0),
                    source: SolidSource {
                        color: [1.0, 0.0, 0.0, 1.0],
                        time_map: TimeMap::identity(),
                        reports_source_time: true,
                    },
                },
                RenderStep::Plugin {
                    id: PluginId("core.filter.clear"),
                    params,
                    inputs: vec![TextureId(0)],
                    output: TextureId(1),
                },
            ],
            output: TextureId(1),
        };

        let mut session = RenderSession::new(&gpu);
        let rendered = render_graph_cached(
            &gpu,
            &mut session,
            RationalTime::ZERO,
            &graph,
            &RenderGraphInputs {
                plugins: Some(&registry),
                ..RenderGraphInputs::default()
            },
            Quality::FINAL,
        )
        .unwrap();

        let actual = download_rgba(&gpu, &rendered.texture).unwrap();
        let mut expected = vec![0u8; desc.data_size()];
        for px in expected.chunks_exact_mut(4) {
            px.copy_from_slice(&[0, 255, 0, 255]);
        }
        assert_rgba_close(
            "plugin-filter-registry-clear-green",
            RgbaImageDesc {
                width: desc.width,
                height: desc.height,
            },
            &actual,
            &expected,
            0,
        );
    }

    #[test]
    fn plugin_step_without_registry_errors() {
        let Some(gpu) = gpu_or_skip() else { return };
        let desc = FrameDesc::packed(8, 4, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
        let graph = LinearRenderGraph {
            desc,
            steps: vec![
                RenderStep::SolidSource {
                    output: TextureId(0),
                    source: SolidSource {
                        color: [0.0, 0.0, 0.0, 1.0],
                        time_map: TimeMap::identity(),
                        reports_source_time: true,
                    },
                },
                RenderStep::Plugin {
                    id: PluginId("core.filter.clear"),
                    params: ResolvedParams::new(),
                    inputs: vec![TextureId(0)],
                    output: TextureId(1),
                },
            ],
            output: TextureId(1),
        };
        let err = render_graph(&gpu, RationalTime::ZERO, &graph, Quality::FINAL).unwrap_err();
        assert!(matches!(err, RenderError::MissingPluginRegistry));
    }

    #[test]
    fn render_frame_accepts_external_background_texture() {
        let Some(gpu) = gpu_or_skip() else { return };
        let request = centered_request();
        let desc = request.desc;
        let background = upload_rgba(&gpu, &desc, &solid_rgba(desc, request.source.color));

        let mut session = RenderSession::new(&gpu);
        let time_map = TimeMap::offset(RationalTime::from_seconds(42), RationalTime::ZERO);
        let external = render_frame_with_background_texture(
            &gpu,
            &mut session,
            &BackgroundTextureRequest {
                desc,
                timeline_time: request.timeline_time,
                // offset: timeline 0 → source 42s(F-4製品経路)
                time_map,
                background: TextureRef {
                    texture: &background,
                    desc,
                },
                overlay: request.overlay,
            },
            Quality::FINAL,
        )
        .unwrap();

        let fixed = render_frame_direct(&gpu, &request, Quality::FINAL).unwrap();
        let external_actual = download_rgba(&gpu, &external.texture).unwrap();
        let fixed_actual = download_rgba(&gpu, &fixed.texture).unwrap();

        assert_eq!(
            external.source_time,
            time_map.try_map(request.timeline_time).unwrap()
        );
        assert_rgba_close(
            "external-background-matches-fixed",
            RgbaImageDesc {
                width: desc.width,
                height: desc.height,
            },
            &external_actual,
            &fixed_actual,
            0,
        );
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

        let err = render_graph(&gpu, RationalTime::ZERO, &graph, Quality::FINAL).unwrap_err();
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

        let err = render_graph(&gpu, request.timeline_time, &graph, Quality::FINAL).unwrap_err();
        assert!(matches!(err, RenderError::MultipleReportingSources));
    }

    #[test]
    fn graph_rejects_non_compact_texture_ids() {
        let Some(gpu) = gpu_or_skip() else { return };
        let request = centered_request();
        let mut graph = linear_graph_from_request(&request);
        graph.output = TextureId(99);

        let err = render_graph(&gpu, request.timeline_time, &graph, Quality::FINAL).unwrap_err();
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

        let err = render_graph(&gpu, request.timeline_time, &graph, Quality::FINAL).unwrap_err();
        assert!(matches!(err, RenderError::DuplicateTextureWrite(0)));
    }

    #[test]
    fn graph_rejects_missing_reporting_source() {
        let Some(gpu) = gpu_or_skip() else { return };
        let mut request = centered_request();
        request.source.reports_source_time = false;
        let graph = linear_graph_from_request(&request);

        let err = render_graph(&gpu, request.timeline_time, &graph, Quality::FINAL).unwrap_err();
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

        let err = render_graph(&gpu, request.timeline_time, &graph, Quality::FINAL).unwrap_err();
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

        let err = render_graph(&gpu, request.timeline_time, &graph, Quality::FINAL).unwrap_err();
        assert!(matches!(
            err,
            RenderError::CompositeForegroundMustComeFromOverlay { foreground: 0 }
        ));
    }

    #[test]
    fn plugin_composite_dispatches_via_registry_golden() {
        // FG-C1: Compositeもレジストリ経由でグラフから呼ばれること。
        let Some(gpu) = gpu_or_skip() else { return };
        let desc = FrameDesc::packed(4, 3, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);

        // CompositeNodeはGPUリソースを持つため静的参照にできず、テストでleakして登録する。
        let composite: &'static CompositeNode = Box::leak(Box::new(CompositeNode::new(&gpu)));
        let mut registry = PluginRegistry::new();
        registry.register_composite(composite).unwrap();

        let bg_px = [0u8, 128, 0, 128];
        let fg_px = [128u8, 0, 0, 128];
        let background = upload_rgba(&gpu, &desc, &tiled(desc, bg_px));
        let foreground = upload_rgba(&gpu, &desc, &tiled(desc, fg_px));

        let graph = LinearRenderGraph {
            desc,
            steps: vec![
                RenderStep::VideoSource {
                    output: TextureId(0),
                },
                RenderStep::VideoSource {
                    output: TextureId(1),
                },
                RenderStep::Plugin {
                    id: PluginId("core.composite.normal"),
                    params: ResolvedParams::new(),
                    inputs: vec![TextureId(0), TextureId(1)],
                    output: TextureId(2),
                },
            ],
            output: TextureId(2),
        };

        let rendered = render_graph_cached(
            &gpu,
            &mut RenderSession::new(&gpu),
            RationalTime::ZERO,
            &graph,
            &RenderGraphInputs {
                video_sources: &[
                    (
                        TextureId(0),
                        TextureRef {
                            texture: &background,
                            desc,
                        },
                    ),
                    (
                        TextureId(1),
                        TextureRef {
                            texture: &foreground,
                            desc,
                        },
                    ),
                ],
                source_time: Some(RationalTime::ZERO),
                plugins: Some(&registry),
            },
            Quality::FINAL,
        )
        .unwrap();

        let actual = download_rgba(&gpu, &rendered.texture).unwrap();
        let expected = tiled(desc, premul_over_u8(bg_px, fg_px));
        assert_rgba_close(
            "plugin-composite-registry-premul-over",
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
    fn draft_and_final_share_canonical_overlay_centroid() {
        // FG-C5: Draft半解像度でも正準空間上のオーバーレイ重心がFinalと一致する。
        let Some(gpu) = gpu_or_skip() else { return };
        let request = RenderFrameRequest {
            desc: FrameDesc::packed(32, 16, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true),
            timeline_time: RationalTime::ZERO,
            source: SolidSource {
                color: [0.0, 0.0, 0.0, 1.0],
                time_map: TimeMap::identity(),
                reports_source_time: true,
            },
            overlay: RectOverlay {
                center: CanonicalPoint { x: 0.25, y: -0.125 },
                size: CanonicalSize {
                    width: 0.5,
                    height: 0.5,
                },
                color: [1.0, 0.0, 0.0, 1.0],
            },
        };

        let final_frame = render_frame(&gpu, &request, Quality::FINAL).unwrap();
        let draft_frame = render_frame(&gpu, &request, Quality::DRAFT).unwrap();
        let final_rgba = download_rgba(&gpu, &final_frame.texture).unwrap();
        let draft_rgba = download_rgba(&gpu, &draft_frame.texture).unwrap();

        let (fx, fy) = opaque_centroid_canonical(&final_rgba, final_frame.desc);
        let (dx, dy) = opaque_centroid_canonical(&draft_rgba, draft_frame.desc);
        assert!(
            (fx - dx).abs() < 0.05 && (fy - dy).abs() < 0.05,
            "canonical centroid mismatch: final=({fx},{fy}) draft=({dx},{dy})"
        );
    }

    fn opaque_centroid_canonical(rgba: &[u8], desc: FrameDesc) -> (f64, f64) {
        let w = desc.width as f64;
        let h = desc.height as f64;
        let mut sx = 0.0;
        let mut sy = 0.0;
        let mut n = 0.0;
        for y in 0..desc.height {
            for x in 0..desc.width {
                let i = ((y * desc.width + x) * 4) as usize;
                if rgba[i + 3] > 200 && rgba[i] > 200 {
                    // ピクセル中心 → 正準(原点中央・Y-up・高さ=1)
                    let cx = (x as f64 + 0.5) / h - (w / h) * 0.5;
                    let cy = 0.5 - (y as f64 + 0.5) / h;
                    sx += cx;
                    sy += cy;
                    n += 1.0;
                }
            }
        }
        assert!(n > 0.0, "no opaque overlay pixels");
        (sx / n, sy / n)
    }

    fn tiled(desc: FrameDesc, px: [u8; 4]) -> Vec<u8> {
        let mut out = vec![0u8; desc.data_size()];
        for p in out.chunks_exact_mut(4) {
            p.copy_from_slice(&px);
        }
        out
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
                )
                .unwrap(),
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
