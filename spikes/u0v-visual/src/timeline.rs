//! wgpu でタイムライン overview を1枚テクスチャに描画(ListView 不使用)。

use crate::token_gen::{ResolvedToken, ThemeTokens};
use bytemuck::{Pod, Zeroable};
use motolii_gpu::GpuCtx;

const SHADER: &str = r#"
struct ColorData { rgba: vec4<f32>, }
@group(0) @binding(0) var<uniform> color: ColorData;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var positions = array<vec2<f32>, 6>(
        vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(-1.0, 1.0),
        vec2(-1.0, 1.0), vec2(1.0, -1.0), vec2(1.0, 1.0),
    );
    var out: VsOut;
    out.pos = vec4(positions[vi], 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(_in: VsOut) -> @location(0) vec4<f32> {
    return color.rgba;
}
"#;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ColorUniform {
    rgba: [f32; 4],
}

pub struct TimelineRenderer {
    width: u32,
    height: u32,
    clear: wgpu::Color,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
}

impl TimelineRenderer {
    pub fn new(gpu: &GpuCtx, width: u32, height: u32, theme: &ThemeTokens) -> Self {
        let clear = theme_color(theme, "color.surface.inset", 0.13, 0.13, 0.13);
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("u0v-timeline-shader"),
                source: wgpu::ShaderSource::Wgsl(SHADER.into()),
            });
        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("u0v-timeline-bgl"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });
        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("u0v-timeline-pl"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            });
        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("u0v-timeline-pipeline"),
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
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
            label: Some("u0v-timeline-color"),
            size: std::mem::size_of::<ColorUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            width,
            height,
            clear,
            pipeline,
            bind_group_layout,
            uniform_buffer,
        }
    }

    pub fn render(&self, gpu: &GpuCtx, theme: &ThemeTokens) -> wgpu::Texture {
        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("u0v-timeline"),
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("u0v-timeline-enc"),
            });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("u0v-timeline-clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        let clips = [
            ("color.item.video", 0.12, 0.08, 0.22, 0.14),
            ("color.item.shape", 0.36, 0.08, 0.18, 0.14),
            ("color.item.text", 0.58, 0.08, 0.16, 0.14),
            ("color.item.audio", 0.12, 0.28, 0.55, 0.18),
            ("color.item.group", 0.12, 0.52, 0.24, 0.14),
            ("color.item.mesh", 0.40, 0.52, 0.20, 0.14),
        ];
        for (token, x, y, w, h) in clips {
            draw_rect(
                gpu,
                &mut encoder,
                &view,
                &self.pipeline,
                &self.bind_group_layout,
                &self.uniform_buffer,
                self.width,
                self.height,
                x,
                y,
                w,
                h,
                theme_clip_color(theme, token),
            );
        }
        let playhead = theme_clip_color(theme, "color.playhead");
        draw_rect(
            gpu,
            &mut encoder,
            &view,
            &self.pipeline,
            &self.bind_group_layout,
            &self.uniform_buffer,
            self.width,
            self.height,
            0.47,
            0.0,
            0.004,
            1.0,
            playhead,
        );
        gpu.queue.submit(Some(encoder.finish()));
        texture
    }
}

fn theme_color(theme: &ThemeTokens, path: &str, r: f32, g: f32, b: f32) -> wgpu::Color {
    match theme.tokens.get(path) {
        Some(ResolvedToken::Color(c)) => wgpu::Color {
            r: c.r as f64 / 255.0,
            g: c.g as f64 / 255.0,
            b: c.b as f64 / 255.0,
            a: c.a as f64,
        },
        _ => wgpu::Color {
            r: r as f64,
            g: g as f64,
            b: b as f64,
            a: 1.0,
        },
    }
}

fn theme_clip_color(theme: &ThemeTokens, path: &str) -> [f32; 4] {
    match theme.tokens.get(path) {
        Some(ResolvedToken::Color(c)) => [
            c.r as f32 / 255.0,
            c.g as f32 / 255.0,
            c.b as f32 / 255.0,
            c.a,
        ],
        _ => [0.4, 0.4, 0.4, 1.0],
    }
}

fn draw_rect(
    gpu: &GpuCtx,
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    pipeline: &wgpu::RenderPipeline,
    bgl: &wgpu::BindGroupLayout,
    uniform_buffer: &wgpu::Buffer,
    tex_w: u32,
    tex_h: u32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    rgba: [f32; 4],
) {
    let uniform = ColorUniform { rgba };
    gpu.queue
        .write_buffer(uniform_buffer, 0, bytemuck::bytes_of(&uniform));
    let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: bgl,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });
    let px_x = (x * tex_w as f32) as u32;
    let px_y = (y * tex_h as f32) as u32;
    let px_w = (w * tex_w as f32).max(1.0) as u32;
    let px_h = (h * tex_h as f32).max(1.0) as u32;
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    pass.set_scissor_rect(px_x, px_y, px_w, px_h);
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.draw(0..6, 0..1);
}

pub fn render_timeline_for_theme(
    gpu: &GpuCtx,
    theme: &ThemeTokens,
) -> Result<wgpu::Texture, String> {
    let renderer = TimelineRenderer::new(gpu, 960, 280, theme);
    Ok(renderer.render(gpu, theme))
}
