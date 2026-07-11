//! motolii-nodes: M1ノード層の最小実装。
//!
//! T7ではまず、プラグイン境界をノード経由でGPU実行できることと、
//! 空間パラメータを正準座標で扱うことをコード化する。

use std::sync::OnceLock;

use motolii_core::{premultiply_rgba_f32, FrameDesc, RationalTime};
use motolii_eval::{DataTracks, ParamSource, Value};
use motolii_gpu::{GpuCtx, PipelineCache};
use motolii_plugin::{
    CompositePlugin, FilterPlugin, NodeDesc, PluginError, PluginId, RenderCtx, ResolvedParams,
    TextureRef,
};
use wgpu::util::DeviceExt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanonicalPoint {
    pub x: f64,
    pub y: f64,
}

impl CanonicalPoint {
    pub const CENTER: Self = Self { x: 0.0, y: 0.0 };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanonicalSize {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PixelPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PixelSize {
    pub width: f64,
    pub height: f64,
}

/// 正準空間(原点中央・Y-up・高さ=1.0)からピクセル空間(Y-down)への変換。
///
/// px変換はレンダ直前のこの型に集約し、ノードパラメータにはpx値を持たせない。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportTransform {
    width_px: u32,
    height_px: u32,
}

impl ViewportTransform {
    pub fn new(width_px: u32, height_px: u32) -> Self {
        assert!(width_px > 0 && height_px > 0, "viewport must be non-zero");
        Self {
            width_px,
            height_px,
        }
    }

    pub fn from_desc(desc: &FrameDesc) -> Self {
        Self::new(desc.width, desc.height)
    }

    pub fn point_to_px(self, p: CanonicalPoint) -> PixelPoint {
        let h = self.height_px as f64;
        PixelPoint {
            x: self.width_px as f64 * 0.5 + p.x * h,
            y: self.height_px as f64 * 0.5 - p.y * h,
        }
    }

    pub fn size_to_px(self, s: CanonicalSize) -> PixelSize {
        let h = self.height_px as f64;
        PixelSize {
            width: s.width * h,
            height: s.height * h,
        }
    }

    pub fn height_px(self) -> u32 {
        self.height_px
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NodeError {
    #[error(transparent)]
    Plugin(#[from] PluginError),
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
    /// straight RGBA, 0..1
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CircleOverlay {
    pub center: CanonicalPoint,
    /// 正準空間の半径(高さ=1.0基準)
    pub radius: f64,
    /// straight RGBA, 0..1
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineOverlay {
    pub start: CanonicalPoint,
    pub end: CanonicalPoint,
    /// 正準空間の線幅(高さ=1.0基準)
    pub width: f64,
    /// straight RGBA, 0..1
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
        let _viewport = ViewportTransform::from_desc(&output.desc);
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
        let viewport = ViewportTransform::from_desc(&output.desc);
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
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl CompositeNode {
    pub fn new(gpu: &GpuCtx) -> Self {
        Self::with_mode(gpu, CompositeMode::Normal)
    }

    pub fn with_mode(gpu: &GpuCtx, mode: CompositeMode) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("motolii-nodes-composite-blend"),
                source: wgpu::ShaderSource::Wgsl(include_str!("composite_blend.wgsl").into()),
            });
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
        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("motolii-nodes-composite-pipeline"),
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
            label: Some("motolii-nodes-composite-sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            mode,
            pipeline,
            bind_group_layout,
            sampler,
        }
    }

    pub fn set_mode(&mut self, mode: CompositeMode) {
        self.mode = mode;
    }

    pub fn render(
        &self,
        gpu: &GpuCtx,
        background: TextureRef<'_>,
        foreground: TextureRef<'_>,
        output: TextureRef<'_>,
    ) -> Result<(), NodeError> {
        self.render_with_encoder(
            gpu,
            None,
            RationalTime::ZERO,
            background,
            foreground,
            output,
        )
    }

    fn render_with_encoder(
        &self,
        gpu: &GpuCtx,
        encoder: Option<&mut wgpu::CommandEncoder>,
        _t: RationalTime,
        background: TextureRef<'_>,
        foreground: TextureRef<'_>,
        output: TextureRef<'_>,
    ) -> Result<(), NodeError> {
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
            self.encode_pass(encoder, &output_view, &bind_group);
        } else {
            let mut encoder = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("motolii-nodes-composite"),
                });
            self.encode_pass(&mut encoder, &output_view, &bind_group);
            gpu.queue.submit([encoder.finish()]);
        }
        Ok(())
    }

    fn encode_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
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
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
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
        self.render_with_encoder(gpu, Some(encoder), ctx.t, inputs[0], inputs[1], output)
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
mod tests {
    use super::*;

    #[test]
    fn canonical_center_maps_to_pixel_center() {
        let tx = ViewportTransform::new(1920, 1080);
        assert_eq!(
            tx.point_to_px(CanonicalPoint::CENTER),
            PixelPoint { x: 960.0, y: 540.0 }
        );
    }

    #[test]
    fn canonical_uses_height_as_unit_and_y_up() {
        let tx = ViewportTransform::new(1920, 1080);
        assert_eq!(
            tx.point_to_px(CanonicalPoint { x: 0.5, y: 0.25 }),
            PixelPoint {
                x: 1500.0,
                y: 270.0
            }
        );
        assert_eq!(
            tx.size_to_px(CanonicalSize {
                width: 0.25,
                height: 0.5
            }),
            PixelSize {
                width: 270.0,
                height: 540.0
            }
        );
    }
}
