
pub(crate) const MATTE_TARGET_FORMAT: wgpu::TextureFormat = crate::render::compositor::blend::SEPARABLE_BLEND_TARGET_FORMAT;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatteMode {
    Alpha,
    InvertedAlpha,
    Luma,
    InvertedLuma,
}

pub(crate) fn matte_mode_index(mode: MatteMode) -> u32 {
    match mode {
        MatteMode::Alpha => 0,
        MatteMode::InvertedAlpha => 1,
        MatteMode::Luma => 2,
        MatteMode::InvertedLuma => 3,
    }
}

pub(crate) struct MattePipelines {
    pipeline: wgpu::RenderPipeline,
    two_texture_layout: wgpu::BindGroupLayout,
    params_layout: wgpu::BindGroupLayout,
}

impl MattePipelines {
    pub(crate) fn new(ctx: &re_renderer::RenderContext) -> Self {
        let device = &ctx.device;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("motolii-compositor-matte-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let two_texture_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("motolii-compositor-matte-two-texture-layout"),
                entries: &[texture_entry(0), texture_entry(1)],
            });
        let params_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("motolii-compositor-matte-params-layout"),
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("motolii-compositor-matte-pipeline-layout"),
            bind_group_layouts: &[Some(&two_texture_layout), Some(&params_layout)],
            immediate_size: 0,
        });

        let vs_handle = re_renderer::renderer::screen_triangle_vertex_shader(ctx);
        let shader_modules = ctx.gpu_resources.shader_modules.resources();
        let screen_triangle_vs = shader_modules.get(vs_handle).expect("上流の頂点シェーダ");

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("motolii-compositor-matte-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: screen_triangle_vs,
                entry_point: Some("main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("matte_fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: MATTE_TARGET_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            two_texture_layout,
            params_layout,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        layer_view: &wgpu::TextureView,
        matte_view: &wgpu::TextureView,
        out_view: &wgpu::TextureView,
        mode: u32,
    ) {
        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("motolii-compositor-matte-params"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut bytes = [0u8; 16];
        bytes[0..4].copy_from_slice(&mode.to_le_bytes());
        queue.write_buffer(&params_buffer, 0, &bytes);

        let params_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-compositor-matte-params-bind"),
            layout: &self.params_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buffer.as_entire_binding(),
            }],
        });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-compositor-matte-texture-bind"),
            layout: &self.two_texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(layer_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(matte_view),
                },
            ],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("motolii-compositor-matte-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: out_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &texture_bind_group, &[]);
        pass.set_bind_group(1, &params_bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

fn texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

const SHADER: &str = r#"
struct MatteParams {
  mode: u32,
  _pad0: u32,
  _pad1: u32,
  _pad2: u32,
};

@group(0) @binding(0) var layer_tex: texture_2d<f32>;
@group(0) @binding(1) var matte_tex: texture_2d<f32>;
@group(1) @binding(0) var<uniform> params: MatteParams;

const LUMA_WEIGHTS: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);

fn coverage(mode: u32, matte: vec4<f32>) -> f32 {
  if (mode == 0u) {
    return matte.a;
  }
  if (mode == 1u) {
    return 1.0 - matte.a;
  }
  if (mode == 2u) {
    return dot(matte.rgb, LUMA_WEIGHTS);
  }
  return 1.0 - dot(matte.rgb, LUMA_WEIGHTS);
}

@fragment
fn matte_fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
  let layer = textureLoad(layer_tex, vec2<i32>(p.xy), 0);
  let matte = textureLoad(matte_tex, vec2<i32>(p.xy), 0);

  let c = clamp(coverage(params.mode, matte), 0.0, 1.0);

  return clamp(layer * c, vec4<f32>(0.0), vec4<f32>(1.0));
}
"#;
