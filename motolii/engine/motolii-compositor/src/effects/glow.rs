
pub(crate) const GLOW_INTERMEDIATE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

pub(crate) struct GlowPipelines {
    bright: wgpu::RenderPipeline,
    blur_horizontal: wgpu::RenderPipeline,
    blur_vertical: wgpu::RenderPipeline,
    composite: wgpu::RenderPipeline,
    one_texture_layout: wgpu::BindGroupLayout,
    two_texture_layout: wgpu::BindGroupLayout,
    params_layout: wgpu::BindGroupLayout,
}

impl GlowPipelines {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("motolii-compositor-glow-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let one_texture_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("motolii-compositor-glow-one-texture-layout"),
                entries: &[texture_entry(0)],
            });
        let two_texture_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("motolii-compositor-glow-two-texture-layout"),
                entries: &[texture_entry(0), texture_entry(1)],
            });
        let params_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("motolii-compositor-glow-params-layout"),
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

        let one_texture_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("motolii-compositor-glow-one-texture-pipeline-layout"),
                bind_group_layouts: &[Some(&one_texture_layout), Some(&params_layout)],
                immediate_size: 0,
            });
        let two_texture_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("motolii-compositor-glow-two-texture-pipeline-layout"),
                bind_group_layouts: &[Some(&two_texture_layout), Some(&params_layout)],
                immediate_size: 0,
            });

        let pipeline = |label: &'static str,
                         layout: &wgpu::PipelineLayout,
                         entry_point: &'static str| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(entry_point),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: GLOW_INTERMEDIATE_FORMAT,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                multiview_mask: None,
                cache: None,
            })
        };

        let bright = pipeline(
            "motolii-compositor-glow-bright-pipeline",
            &one_texture_pipeline_layout,
            "bright_fs",
        );
        let blur_horizontal = pipeline(
            "motolii-compositor-glow-blur-horizontal-pipeline",
            &one_texture_pipeline_layout,
            "blur_horizontal_fs",
        );
        let blur_vertical = pipeline(
            "motolii-compositor-glow-blur-vertical-pipeline",
            &one_texture_pipeline_layout,
            "blur_vertical_fs",
        );
        let composite = pipeline(
            "motolii-compositor-glow-composite-pipeline",
            &two_texture_pipeline_layout,
            "composite_fs",
        );

        Self {
            bright,
            blur_horizontal,
            blur_vertical,
            composite,
            one_texture_layout,
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
        source_view: &wgpu::TextureView,
        bloom_view: &wgpu::TextureView,
        blur_ping_view: &wgpu::TextureView,
        dst_view: &wgpu::TextureView,
        threshold: f32,
        intensity: f32,
        radius: f32,
    ) {
        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("motolii-compositor-glow-params"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut bytes = [0u8; 16];
        bytes[0..4].copy_from_slice(&threshold.to_le_bytes());
        bytes[4..8].copy_from_slice(&intensity.to_le_bytes());
        bytes[8..12].copy_from_slice(&radius.to_le_bytes());
        queue.write_buffer(&params_buffer, 0, &bytes);

        let params_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-compositor-glow-params-bind"),
            layout: &self.params_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buffer.as_entire_binding(),
            }],
        });

        let one_texture_bind_group = |label: &'static str, view: &wgpu::TextureView| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &self.one_texture_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                }],
            })
        };

        let bright_bind = one_texture_bind_group("motolii-compositor-glow-bright-bind", source_view);
        draw_pass(
            encoder,
            "motolii-compositor-glow-bright-pass",
            bloom_view,
            &self.bright,
            &bright_bind,
            &params_bind_group,
        );

        let blur_h_bind = one_texture_bind_group("motolii-compositor-glow-blur-h-bind", bloom_view);
        draw_pass(
            encoder,
            "motolii-compositor-glow-blur-horizontal-pass",
            blur_ping_view,
            &self.blur_horizontal,
            &blur_h_bind,
            &params_bind_group,
        );

        let blur_v_bind =
            one_texture_bind_group("motolii-compositor-glow-blur-v-bind", blur_ping_view);
        draw_pass(
            encoder,
            "motolii-compositor-glow-blur-vertical-pass",
            bloom_view,
            &self.blur_vertical,
            &blur_v_bind,
            &params_bind_group,
        );

        let composite_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-compositor-glow-composite-bind"),
            layout: &self.two_texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(source_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(bloom_view),
                },
            ],
        });
        draw_pass(
            encoder,
            "motolii-compositor-glow-composite-pass",
            dst_view,
            &self.composite,
            &composite_bind,
            &params_bind_group,
        );
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

#[allow(clippy::too_many_arguments)]
fn draw_pass(
    encoder: &mut wgpu::CommandEncoder,
    label: &'static str,
    target: &wgpu::TextureView,
    pipeline: &wgpu::RenderPipeline,
    bind_group: &wgpu::BindGroup,
    params_bind_group: &wgpu::BindGroup,
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target,
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
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, bind_group, &[]);
    pass.set_bind_group(1, params_bind_group, &[]);
    pass.draw(0..3, 0..1);
}

const SHADER: &str = r#"
struct GlowParams {
  threshold: f32,
  intensity: f32,
  radius: f32,
  _pad: f32,
};

@group(0) @binding(0) var input_a: texture_2d<f32>;
@group(0) @binding(1) var input_b: texture_2d<f32>;
@group(1) @binding(0) var<uniform> params: GlowParams;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
  let positions = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
  return vec4<f32>(positions[index], 0.0, 1.0);
}

@fragment
fn bright_fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
  let value = textureLoad(input_a, vec2<i32>(p.xy), 0);
  let luminance = dot(value.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
  let contribution = max(luminance - params.threshold, 0.0) / max(luminance, 0.000001);
  return vec4<f32>(value.rgb * contribution, value.a * contribution);
}

fn blur_at(p: vec2<i32>, direction: vec2<i32>) -> vec4<f32> {
  let size = vec2<i32>(textureDimensions(input_a));
  let lo = vec2<i32>(0);
  let hi = size - vec2<i32>(1);
  let step = max(i32(round(params.radius)), 1);
  let d1 = direction * step;
  let d2 = direction * step * 2;
  return textureLoad(input_a, clamp(p - d2, lo, hi), 0) * 0.0625
    + textureLoad(input_a, clamp(p - d1, lo, hi), 0) * 0.25
    + textureLoad(input_a, clamp(p, lo, hi), 0) * 0.375
    + textureLoad(input_a, clamp(p + d1, lo, hi), 0) * 0.25
    + textureLoad(input_a, clamp(p + d2, lo, hi), 0) * 0.0625;
}

@fragment
fn blur_horizontal_fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
  return blur_at(vec2<i32>(p.xy), vec2<i32>(1, 0));
}

@fragment
fn blur_vertical_fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
  return blur_at(vec2<i32>(p.xy), vec2<i32>(0, 1));
}

@fragment
fn composite_fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
  let source = textureLoad(input_a, vec2<i32>(p.xy), 0);
  let glow = textureLoad(input_b, vec2<i32>(p.xy), 0) * params.intensity;
  return clamp(
    vec4<f32>(source.rgb + glow.rgb, source.a + glow.a * (1.0 - source.a)),
    vec4<f32>(0.0),
    vec4<f32>(1.0)
  );
}
"#;
