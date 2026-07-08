//! oc-nodes: M1ノード層の最小実装。
//!
//! T7ではまず、プラグイン境界をノード経由でGPU実行できることと、
//! 空間パラメータを正準座標で扱うことをコード化する。

use std::sync::OnceLock;

use oc_core::{premultiply_rgba_f32, FrameDesc, RationalTime};
use oc_eval::{DataTracks, ParamSource, Value};
use oc_gpu::GpuCtx;
use oc_plugin::{
    CompositePlugin, FilterPlugin, NodeDesc, PluginError, PluginId, ResolvedParams, TextureRef,
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
        t: RationalTime,
        input: TextureRef<'_>,
        output: TextureRef<'_>,
    ) -> Result<(), NodeError> {
        let _viewport = ViewportTransform::from_desc(&output.desc);
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("oc-nodes-filter"),
            });
        self.plugin
            .render(gpu, &mut encoder, t, &self.params, input, output)?;
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
struct RectUniform {
    min_px: [f32; 2],
    max_px: [f32; 2],
    color: [f32; 4],
}

pub struct OverlayNode {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    rect: RectOverlay,
}

impl OverlayNode {
    pub fn new(gpu: &GpuCtx, rect: RectOverlay) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("oc-nodes-overlay-rect"),
                source: wgpu::ShaderSource::Wgsl(include_str!("overlay_rect.wgsl").into()),
            });
        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("oc-nodes-overlay-bgl"),
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
                label: Some("oc-nodes-overlay-layout"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            });
        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("oc-nodes-overlay-pipeline"),
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
            label: Some("oc-nodes-overlay-sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            pipeline,
            bind_group_layout,
            sampler,
            rect,
        }
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
        let center = viewport.point_to_px(self.rect.center);
        let size = viewport.size_to_px(self.rect.size);
        let uniform = RectUniform {
            min_px: [
                (center.x - size.width * 0.5) as f32,
                (center.y - size.height * 0.5) as f32,
            ],
            max_px: [
                (center.x + size.width * 0.5) as f32,
                (center.y + size.height * 0.5) as f32,
            ],
            color: if output.desc.premultiplied {
                premultiply_rgba_f32(self.rect.color)
            } else {
                self.rect.color
            },
        };
        let uniform_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("oc-nodes-overlay-uniform"),
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
            label: Some("oc-nodes-overlay-bg"),
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
                label: Some("oc-nodes-overlay"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("oc-nodes-overlay-pass"),
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

pub struct CompositeNode {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl CompositeNode {
    pub fn new(gpu: &GpuCtx) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("oc-nodes-composite-normal"),
                source: wgpu::ShaderSource::Wgsl(include_str!("composite_normal.wgsl").into()),
            });
        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("oc-nodes-composite-bgl"),
                    entries: &[texture_entry(0), sampler_entry(1), texture_entry(2)],
                });
        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("oc-nodes-composite-layout"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            });
        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("oc-nodes-composite-pipeline"),
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
            label: Some("oc-nodes-composite-sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            pipeline,
            bind_group_layout,
            sampler,
        }
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
        require_same_dimensions("composite-normal", background.desc, foreground.desc)?;
        require_same_dimensions("composite-normal", background.desc, output.desc)?;
        let bg_view = background
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let fg_view = foreground
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("oc-nodes-composite-bg"),
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
            ],
        });

        if let Some(encoder) = encoder {
            self.encode_pass(encoder, &output_view, &bind_group);
        } else {
            let mut encoder = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("oc-nodes-composite"),
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
            label: Some("oc-nodes-composite-pass"),
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
        encoder: &mut wgpu::CommandEncoder,
        t: RationalTime,
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
        self.render_with_encoder(gpu, Some(encoder), t, inputs[0], inputs[1], output)
            .map_err(|e| PluginError::Render(e.to_string()))
    }
}

fn composite_normal_desc() -> &'static NodeDesc {
    static DESC: OnceLock<NodeDesc> = OnceLock::new();
    DESC.get_or_init(|| NodeDesc {
        id: PluginId("core.composite.normal"),
        display_name: "Normal",
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
