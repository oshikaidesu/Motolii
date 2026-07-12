//! motolii-nodes: M1ノード層の最小実装。
//!
//! T7ではまず、プラグイン境界をノード経由でGPU実行できることと、
//! 空間パラメータを正準座標で扱うことをコード化する。

use std::sync::OnceLock;

use motolii_core::{premultiply_rgba_f32, FrameDesc, Quality, RationalTime};
use motolii_eval::{DataTracks, ParamSource, Value};
use motolii_gpu::{GpuCtx, PipelineCache};
use motolii_plugin::{
    CompositePlugin, FilterPlugin, NodeDesc, PluginError, PluginId, RenderCtx, ResolvedParams,
    TextureRef,
};
use wgpu::util::DeviceExt;

// 互換: 既存呼び出しは nodes 経由のまま。正本は motolii-core(M2E-14)。
pub use motolii_core::{
    CanonicalPoint, CanonicalSize, PixelPoint, PixelSize, ViewportTransform, ViewportTransformError,
};

#[derive(Debug, thiserror::Error)]
pub enum NodeError {
    #[error(transparent)]
    Plugin(#[from] PluginError),
    #[error(transparent)]
    Viewport(#[from] ViewportTransformError),
    #[error("{node} requires premultiplied {role} frame")]
    PremultipliedRequired {
        node: &'static str,
        role: &'static str,
    },
    #[error("{node} requires matching input/output dimensions")]
    DimensionMismatch { node: &'static str },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectOverlay {
    pub center: CanonicalPoint,
    pub size: CanonicalSize,
    /// 非線形sRGB・straight・0..1(M2E-13。合成前にpremulへ)
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CircleOverlay {
    pub center: CanonicalPoint,
    /// 正準空間の半径(高さ=1.0基準)
    pub radius: f64,
    /// 非線形sRGB・straight・0..1(M2E-13。合成前にpremulへ)
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineOverlay {
    pub start: CanonicalPoint,
    pub end: CanonicalPoint,
    /// 正準空間の線幅(高さ=1.0基準)
    pub width: f64,
    /// 非線形sRGB・straight・0..1(M2E-13。合成前にpremulへ)
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OverlayShape {
    Rect(RectOverlay),
    Circle(CircleOverlay),
    Line(LineOverlay),
}

/// キーフレーム/DataTrack駆動の矩形オーバーレイ。
/// フレーム時刻 `t` で評価すると解決済みの [`RectOverlay`] になる。
#[derive(Debug, Clone, PartialEq)]
pub struct ParamRectOverlay {
    pub center: ParamSource,
    pub size: ParamSource,
    pub color: ParamSource,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ParamOverlayError {
    #[error("overlay center must evaluate to Vec2")]
    CenterNotVec2,
    #[error("overlay size must evaluate to Vec2")]
    SizeNotVec2,
    #[error("overlay color must evaluate to Color")]
    ColorNotColor,
    #[error("overlay size must be > 0")]
    NonPositiveSize,
}

impl ParamRectOverlay {
    /// 定数オーバーレイから構築(CLI固定引数や既存テスト向け)。
    pub fn constant(rect: RectOverlay) -> Self {
        Self {
            center: ParamSource::Const(Value::Vec2([rect.center.x, rect.center.y])),
            size: ParamSource::Const(Value::Vec2([rect.size.width, rect.size.height])),
            color: ParamSource::Const(Value::Color(rect.color.map(|c| c as f64))),
        }
    }

    pub fn eval(
        &self,
        t: RationalTime,
        ctx: &DataTracks,
    ) -> Result<RectOverlay, ParamOverlayError> {
        let center = self
            .center
            .eval(t, ctx)
            .as_vec2()
            .ok_or(ParamOverlayError::CenterNotVec2)?;
        let size = self
            .size
            .eval(t, ctx)
            .as_vec2()
            .ok_or(ParamOverlayError::SizeNotVec2)?;
        let color = self
            .color
            .eval(t, ctx)
            .as_color()
            .ok_or(ParamOverlayError::ColorNotColor)?;
        if size[0] <= 0.0 || size[1] <= 0.0 {
            return Err(ParamOverlayError::NonPositiveSize);
        }
        Ok(RectOverlay {
            center: CanonicalPoint {
                x: center[0],
                y: center[1],
            },
            size: CanonicalSize {
                width: size[0],
                height: size[1],
            },
            color: color.map(|c| c.clamp(0.0, 1.0) as f32),
        })
    }
}

pub struct FilterNode {
    plugin: &'static dyn FilterPlugin,
    params: ResolvedParams,
}

impl FilterNode {
    pub fn new(plugin: &'static dyn FilterPlugin) -> Self {
        Self {
            plugin,
            params: default_params(plugin),
        }
    }

    pub fn set_param(&mut self, id: &'static str, value: Value) {
        self.params.insert(id, value);
    }

    pub fn render(
        &self,
        gpu: &GpuCtx,
        pipelines: &mut PipelineCache,
        ctx: &RenderCtx,
        input: TextureRef<'_>,
        output: TextureRef<'_>,
    ) -> Result<(), NodeError> {
        let _viewport = ViewportTransform::from_desc(&output.desc)?;
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("motolii-nodes-filter"),
            });
        self.plugin.render(
            gpu,
            pipelines,
            &mut encoder,
            ctx,
            &self.params,
            input,
            output,
        )?;
        gpu.queue.submit([encoder.finish()]);
        Ok(())
    }
}

fn default_params(plugin: &'static dyn FilterPlugin) -> ResolvedParams {
    let mut params = ResolvedParams::new();
    for p in &plugin.desc().params {
        params.insert(p.id, p.default.clone());
    }
    params
}

pub fn create_rgba_render_target(gpu: &GpuCtx, desc: FrameDesc, label: &str) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: desc.width,
            height: desc.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct OverlayUniform {
    shape_kind: u32,
    _pad0: [u32; 3],
    params0: [f32; 4],
    params1: [f32; 4],
    color: [f32; 4],
}

const OVERLAY_SHAPE_RECT: u32 = 0;
const OVERLAY_SHAPE_CIRCLE: u32 = 1;
const OVERLAY_SHAPE_LINE: u32 = 2;

pub struct OverlayNode {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    shape: OverlayShape,
}

impl OverlayNode {
    pub fn new(gpu: &GpuCtx) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("motolii-nodes-overlay-shapes"),
                source: wgpu::ShaderSource::Wgsl(include_str!("overlay_shapes.wgsl").into()),
            });
        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("motolii-nodes-overlay-bgl"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });
        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("motolii-nodes-overlay-layout"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            });
        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("motolii-nodes-overlay-pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });
        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("motolii-nodes-overlay-sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            pipeline,
            bind_group_layout,
            sampler,
            shape: OverlayShape::Rect(RectOverlay {
                center: CanonicalPoint::CENTER,
                size: CanonicalSize {
                    width: 0.0,
                    height: 0.0,
                },
                color: [0.0; 4],
            }),
        }
    }

    /// 単発レンダ向け。ループ内では `new` + `set_shape` でパイプラインを使い回すこと。
    pub fn with_shape(gpu: &GpuCtx, shape: OverlayShape) -> Self {
        let mut node = Self::new(gpu);
        node.shape = shape;
        node
    }

    /// 単発レンダ向け。ループ内では `new` + `set_rect` でパイプラインを使い回すこと。
    pub fn with_rect(gpu: &GpuCtx, rect: RectOverlay) -> Self {
        Self::with_shape(gpu, OverlayShape::Rect(rect))
    }

    pub fn with_circle(gpu: &GpuCtx, circle: CircleOverlay) -> Self {
        Self::with_shape(gpu, OverlayShape::Circle(circle))
    }

    pub fn with_line(gpu: &GpuCtx, line: LineOverlay) -> Self {
        Self::with_shape(gpu, OverlayShape::Line(line))
    }

    pub fn set_shape(&mut self, shape: OverlayShape) {
        self.shape = shape;
    }

    pub fn set_rect(&mut self, rect: RectOverlay) {
        self.shape = OverlayShape::Rect(rect);
    }

    pub fn set_circle(&mut self, circle: CircleOverlay) {
        self.shape = OverlayShape::Circle(circle);
    }

    pub fn set_line(&mut self, line: LineOverlay) {
        self.shape = OverlayShape::Line(line);
    }

    pub fn render(
        &self,
        gpu: &GpuCtx,
        input: TextureRef<'_>,
        output: TextureRef<'_>,
    ) -> Result<(), NodeError> {
        require_same_dimensions("overlay", input.desc, output.desc)?;
        if input.desc.premultiplied || output.desc.premultiplied {
            require_premultiplied("overlay", "input", input.desc)?;
            require_premultiplied("overlay", "output", output.desc)?;
        }
        let viewport = ViewportTransform::from_desc(&output.desc)?;
        let uniform = overlay_uniform(&viewport, self.shape, output.desc.premultiplied);
        let uniform_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("motolii-nodes-overlay-uniform"),
                contents: bytemuck::bytes_of(&uniform),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let input_view = input
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-nodes-overlay-bg"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("motolii-nodes-overlay"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("motolii-nodes-overlay-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                multiview_mask: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        gpu.queue.submit([encoder.finish()]);
        Ok(())
    }
}

fn overlay_uniform(
    viewport: &ViewportTransform,
    shape: OverlayShape,
    premultiplied: bool,
) -> OverlayUniform {
    let color = |straight: [f32; 4]| {
        if premultiplied {
            premultiply_rgba_f32(straight)
        } else {
            straight
        }
    };
    match shape {
        OverlayShape::Rect(rect) => {
            let center = viewport.point_to_px(rect.center);
            let size = viewport.size_to_px(rect.size);
            OverlayUniform {
                shape_kind: OVERLAY_SHAPE_RECT,
                _pad0: [0; 3],
                params0: [
                    (center.x - size.width * 0.5) as f32,
                    (center.y - size.height * 0.5) as f32,
                    0.0,
                    0.0,
                ],
                params1: [
                    (center.x + size.width * 0.5) as f32,
                    (center.y + size.height * 0.5) as f32,
                    0.0,
                    0.0,
                ],
                color: color(rect.color),
            }
        }
        OverlayShape::Circle(circle) => {
            let center = viewport.point_to_px(circle.center);
            let radius = (circle.radius * viewport.height_px() as f64) as f32;
            OverlayUniform {
                shape_kind: OVERLAY_SHAPE_CIRCLE,
                _pad0: [0; 3],
                params0: [center.x as f32, center.y as f32, radius, 0.0],
                params1: [0.0; 4],
                color: color(circle.color),
            }
        }
        OverlayShape::Line(line) => {
            let start = viewport.point_to_px(line.start);
            let end = viewport.point_to_px(line.end);
            let width = (line.width * viewport.height_px() as f64) as f32;
            OverlayUniform {
                shape_kind: OVERLAY_SHAPE_LINE,
                _pad0: [0; 3],
                params0: [start.x as f32, start.y as f32, width, 0.0],
                params1: [end.x as f32, end.y as f32, 0.0, 0.0],
                color: color(line.color),
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositeMode {
    Normal,
    Add,
    Multiply,
}

/// 合成時の色空間経路。`Quality.precise_color` から選ぶ(M2E-18)。
///
/// v1は両枝とも現行sRGB空間ブレンドWGSL(恒等)。将来 `LinearPrecise` だけ差し替える受け皿。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CompositeColorPath {
    /// `precise_color == false`: sRGB空間ブレンド等の近似を許容
    SrgbApprox = 0,
    /// `precise_color == true`: リニア精密合成の席(v1はSrgbApproxと同一WGSL)
    LinearPrecise = 1,
}

/// 不変な合成分岐プラン。隠れた可変状態なし(M2E-18)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompositeRenderPlan {
    pub color_path: CompositeColorPath,
}

/// `Quality` → 合成分岐プラン。純関数。
pub fn plan_composite_render(quality: Quality) -> CompositeRenderPlan {
    CompositeRenderPlan {
        color_path: select_composite_color_path(quality.precise_color),
    }
}

/// `Quality.precise_color` → 合成分岐。純関数(GPU不要)。
pub fn select_composite_color_path(precise_color: bool) -> CompositeColorPath {
    if precise_color {
        CompositeColorPath::LinearPrecise
    } else {
        CompositeColorPath::SrgbApprox
    }
}

/// 分岐点のWGSL選択。v1は両枝とも同一ソース(恒等実装)。
/// 将来 `LinearPrecise` だけ別ファイルへ差し替える。
pub fn composite_blend_shader_source(path: CompositeColorPath) -> &'static str {
    match path {
        CompositeColorPath::SrgbApprox | CompositeColorPath::LinearPrecise => {
            include_str!("composite_blend.wgsl")
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CompositeUniform {
    mode: u32,
    _pad: [u32; 3],
}

const COMPOSITE_MODE_NORMAL: u32 = 0;
const COMPOSITE_MODE_ADD: u32 = 1;
const COMPOSITE_MODE_MULTIPLY: u32 = 2;

pub struct CompositeNode {
    mode: CompositeMode,
    /// `CompositeColorPath::SrgbApprox` 用。コンストラクタで構築しレンダ時に再利用。
    pipeline_srgb: wgpu::RenderPipeline,
    /// `CompositeColorPath::LinearPrecise` 用。v1は同一WGSLでも別パイプライン実体。
    pipeline_linear: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl CompositeNode {
    pub fn new(gpu: &GpuCtx) -> Self {
        Self::with_mode(gpu, CompositeMode::Normal)
    }

    pub fn with_mode(gpu: &GpuCtx, mode: CompositeMode) -> Self {
        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("motolii-nodes-composite-bgl"),
                    entries: &[
                        texture_entry(0),
                        sampler_entry(1),
                        texture_entry(2),
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });
        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("motolii-nodes-composite-layout"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            });
        let pipeline_srgb = create_composite_pipeline(
            gpu,
            &pipeline_layout,
            CompositeColorPath::SrgbApprox,
            "motolii-nodes-composite-srgb",
        );
        let pipeline_linear = create_composite_pipeline(
            gpu,
            &pipeline_layout,
            CompositeColorPath::LinearPrecise,
            "motolii-nodes-composite-linear",
        );
        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("motolii-nodes-composite-sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            mode,
            pipeline_srgb,
            pipeline_linear,
            bind_group_layout,
            sampler,
        }
    }

    pub fn set_mode(&mut self, mode: CompositeMode) {
        self.mode = mode;
    }

    /// プランに対応する実パイプライン(ループ外構築済み)。
    pub fn pipeline_for_plan(&self, plan: CompositeRenderPlan) -> &wgpu::RenderPipeline {
        match plan.color_path {
            CompositeColorPath::SrgbApprox => &self.pipeline_srgb,
            CompositeColorPath::LinearPrecise => &self.pipeline_linear,
        }
    }

    pub fn render(
        &self,
        gpu: &GpuCtx,
        ctx: &RenderCtx,
        background: TextureRef<'_>,
        foreground: TextureRef<'_>,
        output: TextureRef<'_>,
    ) -> Result<(), NodeError> {
        self.render_with_encoder(gpu, None, ctx, background, foreground, output)
    }

    fn render_with_encoder(
        &self,
        gpu: &GpuCtx,
        encoder: Option<&mut wgpu::CommandEncoder>,
        ctx: &RenderCtx,
        background: TextureRef<'_>,
        foreground: TextureRef<'_>,
        output: TextureRef<'_>,
    ) -> Result<(), NodeError> {
        let plan = plan_composite_render(ctx.quality);
        let pipeline = self.pipeline_for_plan(plan);

        require_premultiplied("composite-normal", "background", background.desc)?;
        require_premultiplied("composite-normal", "foreground", foreground.desc)?;
        require_premultiplied("composite-normal", "output", output.desc)?;
        // 背景は正規化UVサンプル。出力と同一アスペクトかつ整数倍スケール差のみ許可。
        require_same_dimensions("composite-normal", foreground.desc, output.desc)?;
        require_compatible_background("composite-normal", background.desc, output.desc)?;
        let bg_view = background
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let fg_view = foreground
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let uniform = CompositeUniform {
            mode: composite_mode_to_u32(self.mode),
            _pad: [0; 3],
        };
        let uniform_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("motolii-nodes-composite-uniform"),
                contents: bytemuck::bytes_of(&uniform),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-nodes-composite-bg"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&bg_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&fg_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        if let Some(encoder) = encoder {
            Self::encode_pass(encoder, pipeline, &output_view, &bind_group);
        } else {
            let mut encoder = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("motolii-nodes-composite"),
                });
            Self::encode_pass(&mut encoder, pipeline, &output_view, &bind_group);
            gpu.queue.submit([encoder.finish()]);
        }
        Ok(())
    }

    fn encode_pass(
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &wgpu::RenderPipeline,
        output_view: &wgpu::TextureView,
        bind_group: &wgpu::BindGroup,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("motolii-nodes-composite-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            multiview_mask: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

fn create_composite_pipeline(
    gpu: &GpuCtx,
    pipeline_layout: &wgpu::PipelineLayout,
    path: CompositeColorPath,
    label: &str,
) -> wgpu::RenderPipeline {
    let shader = gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(composite_blend_shader_source(path).into()),
        });
    gpu.device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        })
}

impl CompositePlugin for CompositeNode {
    fn desc(&self) -> &NodeDesc {
        composite_normal_desc()
    }

    fn render(
        &self,
        gpu: &GpuCtx,
        _pipelines: &mut PipelineCache,
        encoder: &mut wgpu::CommandEncoder,
        ctx: &RenderCtx,
        _params: &ResolvedParams,
        inputs: &[TextureRef<'_>],
        output: TextureRef<'_>,
    ) -> Result<(), PluginError> {
        if inputs.len() != 2 {
            return Err(PluginError::Render(format!(
                "composite-normal expects 2 inputs, got {}",
                inputs.len()
            )));
        }
        self.render_with_encoder(gpu, Some(encoder), ctx, inputs[0], inputs[1], output)
            .map_err(|e| PluginError::Render(e.to_string()))
    }
}

fn composite_mode_to_u32(mode: CompositeMode) -> u32 {
    match mode {
        CompositeMode::Normal => COMPOSITE_MODE_NORMAL,
        CompositeMode::Add => COMPOSITE_MODE_ADD,
        CompositeMode::Multiply => COMPOSITE_MODE_MULTIPLY,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClippingMaskMode {
    Alpha,
    Luminance,
    InvertAlpha,
    InvertLuminance,
}
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct MaskUniform {
    mode: u32,
    _pad: [u32; 3],
}
pub struct MaskNode {
    mode: ClippingMaskMode,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
    sampler: wgpu::Sampler,
}
impl MaskNode {
    pub fn new(gpu: &GpuCtx) -> Self {
        Self::with_mode(gpu, ClippingMaskMode::Alpha)
    }
    pub fn with_mode(gpu: &GpuCtx, mode: ClippingMaskMode) -> Self {
        let bgl = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("mask-bgl"),
                entries: &[
                    texture_entry(0),
                    sampler_entry(1),
                    texture_entry(2),
                    sampler_entry(3),
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let pl = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("mask-pl"),
                bind_group_layouts: &[Some(&bgl)],
                immediate_size: 0,
            });
        let sh = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("mask-sh"),
                source: wgpu::ShaderSource::Wgsl(include_str!("mask_apply.wgsl").into()),
            });
        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("mask-pipe"),
                layout: Some(&pl),
                vertex: wgpu::VertexState {
                    module: &sh,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &sh,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });
        let uniform_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("mask-u"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("mask-s"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            mode,
            pipeline,
            bind_group_layout: bgl,
            uniform_buffer,
            sampler,
        }
    }
    pub fn set_mode(&mut self, mode: ClippingMaskMode) {
        self.mode = mode;
    }
    pub fn render(
        &self,
        gpu: &GpuCtx,
        content: TextureRef<'_>,
        mask: TextureRef<'_>,
        output: TextureRef<'_>,
    ) -> Result<(), NodeError> {
        require_premultiplied("MaskNode", "content", content.desc)?;
        require_premultiplied("MaskNode", "mask", mask.desc)?;
        require_premultiplied("MaskNode", "output", output.desc)?;
        let m = match self.mode {
            ClippingMaskMode::Alpha => 0,
            ClippingMaskMode::Luminance => 1,
            ClippingMaskMode::InvertAlpha => 2,
            ClippingMaskMode::InvertLuminance => 3,
        };
        gpu.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::bytes_of(&MaskUniform {
                mode: m,
                _pad: [0; 3],
            }),
        );
        let cv = content.texture.create_view(&Default::default());
        let mv = mask.texture.create_view(&Default::default());
        let ov = output.texture.create_view(&Default::default());
        let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("mask-bg"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&cv),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&mv),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
            ],
        });
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("mask"),
            });
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("mask-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &ov,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                multiview_mask: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bg, &[]);
            pass.draw(0..3, 0..1);
        }
        gpu.queue.submit([enc.finish()]);
        Ok(())
    }
}

fn composite_normal_desc() -> &'static NodeDesc {
    static DESC: OnceLock<NodeDesc> = OnceLock::new();
    DESC.get_or_init(|| NodeDesc {
        id: PluginId("core.composite.normal"),
        version: 1,
        display_name: "Normal",
        category: "Composite",
        tags: &["blend", "over", "premultiplied"],
        params: Vec::new(),
        min_inputs: 2,
        max_inputs: 2,
    })
}

fn require_premultiplied(
    node: &'static str,
    role: &'static str,
    desc: FrameDesc,
) -> Result<(), NodeError> {
    if desc.premultiplied {
        Ok(())
    } else {
        Err(NodeError::PremultipliedRequired { node, role })
    }
}

fn require_same_dimensions(
    node: &'static str,
    a: FrameDesc,
    b: FrameDesc,
) -> Result<(), NodeError> {
    if a.width == b.width && a.height == b.height {
        Ok(())
    } else {
        Err(NodeError::DimensionMismatch { node })
    }
}

fn require_compatible_background(
    node: &'static str,
    background: FrameDesc,
    output: FrameDesc,
) -> Result<(), NodeError> {
    if background.same_aspect_integer_scale(output) {
        Ok(())
    } else {
        Err(NodeError::DimensionMismatch { node })
    }
}

fn texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn sampler_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn select_composite_color_path_follows_precise_color_flag() {
        assert_eq!(
            select_composite_color_path(false),
            CompositeColorPath::SrgbApprox
        );
        assert_eq!(
            select_composite_color_path(true),
            CompositeColorPath::LinearPrecise
        );
        assert_eq!(
            plan_composite_render(Quality::DRAFT),
            CompositeRenderPlan {
                color_path: CompositeColorPath::SrgbApprox
            }
        );
        assert_eq!(
            plan_composite_render(Quality::FINAL),
            CompositeRenderPlan {
                color_path: CompositeColorPath::LinearPrecise
            }
        );
    }

    #[test]
    fn composite_blend_shader_source_is_identity_in_v1() {
        let approx = composite_blend_shader_source(CompositeColorPath::SrgbApprox);
        let precise = composite_blend_shader_source(CompositeColorPath::LinearPrecise);
        assert_eq!(approx, precise, "v1恒等: 両枝とも現行sRGBブレンドWGSL");
        assert!(approx.contains("fs_main"));
    }
}
