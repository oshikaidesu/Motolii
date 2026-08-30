
pub(crate) const SEPARABLE_BLEND_TARGET_FORMAT: wgpu::TextureFormat =
    wgpu::TextureFormat::Rgba8UnormSrgb;

pub(crate) struct SeparableBlendPipelines {
    pipeline: wgpu::RenderPipeline,
    two_texture_layout: wgpu::BindGroupLayout,
    params_layout: wgpu::BindGroupLayout,
}

impl SeparableBlendPipelines {
    pub(crate) fn new(ctx: &re_renderer::RenderContext) -> Self {
        let device = &ctx.device;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("motolii-compositor-blend-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let two_texture_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("motolii-compositor-blend-two-texture-layout"),
                entries: &[texture_entry(0), texture_entry(1)],
            });
        let params_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("motolii-compositor-blend-params-layout"),
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
            label: Some("motolii-compositor-blend-pipeline-layout"),
            bind_group_layouts: &[Some(&two_texture_layout), Some(&params_layout)],
            immediate_size: 0,
        });

        let vs_handle = re_renderer::renderer::screen_triangle_vertex_shader(ctx);
        let shader_modules = ctx.gpu_resources.shader_modules.resources();
        let screen_triangle_vs = shader_modules.get(vs_handle).expect("上流の頂点シェーダ");

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("motolii-compositor-blend-pipeline"),
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
                entry_point: Some("blend_fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: SEPARABLE_BLEND_TARGET_FORMAT,
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
        dst_view: &wgpu::TextureView,
        src_view: &wgpu::TextureView,
        out_view: &wgpu::TextureView,
        mode: u32,
    ) {
        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("motolii-compositor-blend-params"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut bytes = [0u8; 16];
        bytes[0..4].copy_from_slice(&mode.to_le_bytes());
        queue.write_buffer(&params_buffer, 0, &bytes);

        let params_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-compositor-blend-params-bind"),
            layout: &self.params_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buffer.as_entire_binding(),
            }],
        });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-compositor-blend-texture-bind"),
            layout: &self.two_texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(dst_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(src_view),
                },
            ],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("motolii-compositor-blend-pass"),
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
struct BlendParams {
  mode: u32,
  _pad0: u32,
  _pad1: u32,
  _pad2: u32,
};

@group(0) @binding(0) var dst_tex: texture_2d<f32>;
@group(0) @binding(1) var src_tex: texture_2d<f32>;
@group(1) @binding(0) var<uniform> params: BlendParams;

fn blend_channel(mode: u32, cb: f32, cs: f32) -> f32 {
  if (mode == 0u) {
    return cs * cb;
  }
  if (mode == 1u) {
    return cs + cb - cs * cb;
  }
  if (mode == 2u) {
    if (cb <= 0.5) {
      return 2.0 * cs * cb;
    }
    return 1.0 - 2.0 * (1.0 - cs) * (1.0 - cb);
  }
  if (mode == 3u) {
    return min(cb, cs);
  }
  if (mode == 4u) {
    return max(cb, cs);
  }
  if (mode == 5u) {
    if (cb <= 0.0) {
      return 0.0;
    }
    if (cs >= 1.0) {
      return 1.0;
    }
    return min(1.0, cb / (1.0 - cs));
  }
  if (mode == 6u) {
    if (cb >= 1.0) {
      return 1.0;
    }
    if (cs <= 0.0) {
      return 0.0;
    }
    return 1.0 - min(1.0, (1.0 - cb) / cs);
  }
  if (mode == 7u) {
    if (cs <= 0.5) {
      return 2.0 * cs * cb;
    }
    return 1.0 - 2.0 * (1.0 - cs) * (1.0 - cb);
  }
  if (mode == 8u) {
    if (cs <= 0.5) {
      return cb - (1.0 - 2.0 * cs) * cb * (1.0 - cb);
    }
    var d: f32;
    if (cb <= 0.25) {
      d = ((16.0 * cb - 12.0) * cb + 4.0) * cb;
    } else {
      d = sqrt(cb);
    }
    return cb + (2.0 * cs - 1.0) * (d - cb);
  }
  if (mode == 9u) {
    return abs(cb - cs);
  }
  return cs + cb - 2.0 * cs * cb;
}

fn lum(c: vec3<f32>) -> f32 {
  return dot(c, vec3<f32>(0.3, 0.59, 0.11));
}

fn clip_color(c_in: vec3<f32>) -> vec3<f32> {
  let l = lum(c_in);
  let n = min(min(c_in.r, c_in.g), c_in.b);
  let x = max(max(c_in.r, c_in.g), c_in.b);
  var c = c_in;
  if (n < 0.0) {
    c = l + (c - l) * (l / (l - n));
  }
  if (x > 1.0) {
    c = l + (c - l) * ((1.0 - l) / (x - l));
  }
  return c;
}

fn set_lum(c: vec3<f32>, l: f32) -> vec3<f32> {
  let d = l - lum(c);
  return clip_color(c + vec3<f32>(d, d, d));
}

fn sat(c: vec3<f32>) -> f32 {
  return max(max(c.r, c.g), c.b) - min(min(c.r, c.g), c.b);
}

fn set_sat(c: vec3<f32>, s: f32) -> vec3<f32> {
  let cmax = max(max(c.r, c.g), c.b);
  let cmin = min(min(c.r, c.g), c.b);
  if (cmax > cmin) {
    return (c - vec3<f32>(cmin)) * (s / (cmax - cmin));
  }
  return vec3<f32>(0.0, 0.0, 0.0);
}

fn nonseparable_blend(mode: u32, cb: vec3<f32>, cs: vec3<f32>) -> vec3<f32> {
  if (mode == 11u) {
    return set_lum(set_sat(cs, sat(cb)), lum(cb));
  }
  if (mode == 12u) {
    return set_lum(set_sat(cb, sat(cs)), lum(cb));
  }
  if (mode == 13u) {
    return set_lum(cs, lum(cb));
  }
  return set_lum(cb, lum(cs));
}

@fragment
fn blend_fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
  let dst = textureLoad(dst_tex, vec2<i32>(p.xy), 0);
  let src = textureLoad(src_tex, vec2<i32>(p.xy), 0);

  let alpha_b = dst.a;
  let alpha_s = src.a;

  var cb = vec3<f32>(0.0);
  if (alpha_b > 0.0) {
    cb = dst.rgb / alpha_b;
  }
  var cs = vec3<f32>(0.0);
  if (alpha_s > 0.0) {
    cs = src.rgb / alpha_s;
  }

  var b: vec3<f32>;
  if (params.mode < 11u) {
    b = vec3<f32>(
      blend_channel(params.mode, cb.r, cs.r),
      blend_channel(params.mode, cb.g, cs.g),
      blend_channel(params.mode, cb.b, cs.b),
    );
  } else {
    b = nonseparable_blend(params.mode, cb, cs);
  }

  let out_rgb = alpha_s * (1.0 - alpha_b) * cs + alpha_s * alpha_b * b + (1.0 - alpha_s) * alpha_b * cb;
  let out_a = alpha_s + alpha_b * (1.0 - alpha_s);

  return clamp(vec4<f32>(out_rgb, out_a), vec4<f32>(0.0), vec4<f32>(1.0));
}
"#;
