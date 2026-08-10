use crate::{RenderError, HEIGHT, WIDTH};

const BYTES_PER_PIXEL: u32 = 8;
const BYTES_PER_ROW: u32 = WIDTH * BYTES_PER_PIXEL;
const FRAME_BYTES: u64 = (BYTES_PER_ROW * HEIGHT) as u64;

#[derive(Clone, Debug, PartialEq)]
pub struct GlowFrame {
    pub width: u32,
    pub height: u32,
    pub source: Vec<[f32; 4]>,
    pub bloom: Vec<[f32; 4]>,
    pub output: Vec<[f32; 4]>,
}

pub struct GlowFixture {
    device: wgpu::Device,
    queue: wgpu::Queue,
    source: wgpu::Texture,
    bloom: wgpu::Texture,
    output: wgpu::Texture,
    source_view: wgpu::TextureView,
    bloom_view: wgpu::TextureView,
    blur_ping_view: wgpu::TextureView,
    output_view: wgpu::TextureView,
    readback: wgpu::Buffer,
    source_pipeline: wgpu::RenderPipeline,
    bright_pipeline: wgpu::RenderPipeline,
    blur_horizontal_pipeline: wgpu::RenderPipeline,
    blur_vertical_pipeline: wgpu::RenderPipeline,
    composite_pipeline: wgpu::RenderPipeline,
    bright_bind_group: wgpu::BindGroup,
    blur_horizontal_bind_group: wgpu::BindGroup,
    blur_vertical_bind_group: wgpu::BindGroup,
    composite_bind_group: wgpu::BindGroup,
}

impl GlowFixture {
    pub fn new() -> Result<Self, RenderError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .map_err(|_| RenderError::AdapterUnavailable)?;
        crate::required_limits(&adapter)?;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("m5-r0-glow-fixture"),
            ..Default::default()
        }))
        .map_err(|error| RenderError::Device(error.to_string()))?;

        let texture = |label| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: WIDTH,
                    height: HEIGHT,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
        };
        let source = texture("m5-r0-glow-source");
        let bloom = texture("m5-r0-glow-host-transient-bloom");
        let blur_ping = texture("m5-r0-glow-host-transient-ping");
        let output = texture("m5-r0-glow-output");
        let source_view = source.create_view(&Default::default());
        let bloom_view = bloom.create_view(&Default::default());
        let blur_ping_view = blur_ping.create_view(&Default::default());
        let output_view = output.create_view(&Default::default());

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("m5-r0-glow-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let one_texture_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("m5-r0-glow-one-texture-layout"),
                entries: &[texture_entry(0)],
            });
        let two_texture_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("m5-r0-glow-two-texture-layout"),
                entries: &[texture_entry(0), texture_entry(1)],
            });
        let empty_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("m5-r0-glow-source-layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });
        let one_texture_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("m5-r0-glow-filter-layout"),
                bind_group_layouts: &[Some(&one_texture_layout)],
                immediate_size: 0,
            });
        let two_texture_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("m5-r0-glow-composite-layout"),
                bind_group_layouts: &[Some(&two_texture_layout)],
                immediate_size: 0,
            });
        let pipeline = |label, layout, entry_point| {
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
                        format: wgpu::TextureFormat::Rgba16Float,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let source_pipeline = pipeline(
            "m5-r0-glow-source-pipeline",
            &empty_pipeline_layout,
            "source_fs",
        );
        let bright_pipeline = pipeline(
            "m5-r0-glow-bright-pipeline",
            &one_texture_pipeline_layout,
            "bright_fs",
        );
        let blur_horizontal_pipeline = pipeline(
            "m5-r0-glow-blur-horizontal-pipeline",
            &one_texture_pipeline_layout,
            "blur_horizontal_fs",
        );
        let blur_vertical_pipeline = pipeline(
            "m5-r0-glow-blur-vertical-pipeline",
            &one_texture_pipeline_layout,
            "blur_vertical_fs",
        );
        let composite_pipeline = pipeline(
            "m5-r0-glow-composite-pipeline",
            &two_texture_pipeline_layout,
            "composite_fs",
        );
        let one_texture_bind_group = |label, view: &wgpu::TextureView| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &one_texture_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                }],
            })
        };
        let bright_bind_group = one_texture_bind_group("m5-r0-glow-bright-bind", &source_view);
        let blur_horizontal_bind_group =
            one_texture_bind_group("m5-r0-glow-blur-horizontal-bind", &bloom_view);
        let blur_vertical_bind_group =
            one_texture_bind_group("m5-r0-glow-blur-vertical-bind", &blur_ping_view);
        let composite_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("m5-r0-glow-composite-bind"),
            layout: &two_texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&source_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&bloom_view),
                },
            ],
        });
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("m5-r0-glow-readback"),
            size: FRAME_BYTES * 3,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Ok(Self {
            device,
            queue,
            source,
            bloom,
            output,
            source_view,
            bloom_view,
            blur_ping_view,
            output_view,
            readback,
            source_pipeline,
            bright_pipeline,
            blur_horizontal_pipeline,
            blur_vertical_pipeline,
            composite_pipeline,
            bright_bind_group,
            blur_horizontal_bind_group,
            blur_vertical_bind_group,
            composite_bind_group,
        })
    }

    pub fn render(&self) -> Result<GlowFrame, RenderError> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("m5-r0-glow-encoder"),
            });
        draw_pass(
            &mut encoder,
            "m5-r0-glow-source-pass",
            &self.source_view,
            &self.source_pipeline,
            None,
        );
        draw_pass(
            &mut encoder,
            "m5-r0-glow-bright-pass",
            &self.bloom_view,
            &self.bright_pipeline,
            Some(&self.bright_bind_group),
        );
        draw_pass(
            &mut encoder,
            "m5-r0-glow-blur-horizontal-pass",
            &self.blur_ping_view,
            &self.blur_horizontal_pipeline,
            Some(&self.blur_horizontal_bind_group),
        );
        draw_pass(
            &mut encoder,
            "m5-r0-glow-blur-vertical-pass",
            &self.bloom_view,
            &self.blur_vertical_pipeline,
            Some(&self.blur_vertical_bind_group),
        );
        draw_pass(
            &mut encoder,
            "m5-r0-glow-composite-pass",
            &self.output_view,
            &self.composite_pipeline,
            Some(&self.composite_bind_group),
        );
        for (texture, offset) in [
            (&self.source, 0),
            (&self.bloom, FRAME_BYTES),
            (&self.output, FRAME_BYTES * 2),
        ] {
            encoder.copy_texture_to_buffer(
                texture.as_image_copy(),
                wgpu::TexelCopyBufferInfo {
                    buffer: &self.readback,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset,
                        bytes_per_row: Some(BYTES_PER_ROW),
                        rows_per_image: Some(HEIGHT),
                    },
                },
                wgpu::Extent3d {
                    width: WIDTH,
                    height: HEIGHT,
                    depth_or_array_layers: 1,
                },
            );
        }
        self.queue.submit([encoder.finish()]);
        let slice = self.readback.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .map_err(|error| RenderError::Readback(error.to_string()))?;
        let mapped = slice.get_mapped_range();
        let source = decode_frame(&mapped[..FRAME_BYTES as usize]);
        let bloom = decode_frame(&mapped[FRAME_BYTES as usize..(FRAME_BYTES * 2) as usize]);
        let output = decode_frame(&mapped[(FRAME_BYTES * 2) as usize..]);
        drop(mapped);
        self.readback.unmap();
        Ok(GlowFrame {
            width: WIDTH,
            height: HEIGHT,
            source,
            bloom,
            output,
        })
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

fn draw_pass(
    encoder: &mut wgpu::CommandEncoder,
    label: &'static str,
    target: &wgpu::TextureView,
    pipeline: &wgpu::RenderPipeline,
    bind_group: Option<&wgpu::BindGroup>,
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
    if let Some(bind_group) = bind_group {
        pass.set_bind_group(0, bind_group, &[]);
    }
    pass.draw(0..3, 0..1);
}

fn decode_frame(bytes: &[u8]) -> Vec<[f32; 4]> {
    bytes
        .chunks_exact(8)
        .map(|pixel| {
            std::array::from_fn(|channel| {
                let offset = channel * 2;
                half_to_f32(u16::from_le_bytes([pixel[offset], pixel[offset + 1]]))
            })
        })
        .collect()
}

fn half_to_f32(bits: u16) -> f32 {
    let sign = ((bits >> 15) as u32) << 31;
    let exponent = ((bits >> 10) & 0x1f) as u32;
    let fraction = (bits & 0x03ff) as u32;
    let value = match exponent {
        0 if fraction == 0 => sign,
        0 => {
            let shift = fraction.leading_zeros() - 21;
            let normalized = fraction << (shift + 1);
            sign | ((127 - 15 - shift) << 23) | ((normalized & 0x03ff) << 13)
        }
        31 => sign | 0x7f80_0000 | (fraction << 13),
        _ => sign | ((exponent + 127 - 15) << 23) | (fraction << 13),
    };
    f32::from_bits(value)
}

const SHADER: &str = r#"
@group(0) @binding(0) var input_a: texture_2d<f32>;
@group(0) @binding(1) var input_b: texture_2d<f32>;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
  let positions = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
  return vec4<f32>(positions[index], 0.0, 1.0);
}

@fragment
fn source_fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
  let q = p.xy / vec2<f32>(32.0, 32.0);
  let inside = q.x >= 0.375 && q.x < 0.625 && q.y >= 0.375 && q.y < 0.625;
  return select(vec4<f32>(0.0), vec4<f32>(4.0, 2.0, 0.5, 1.0), inside);
}

@fragment
fn bright_fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
  let value = textureLoad(input_a, vec2<i32>(p.xy), 0);
  let luminance = dot(value.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
  let contribution = max(luminance - 1.0, 0.0) / max(luminance, 0.000001);
  return vec4<f32>(value.rgb * contribution, value.a * contribution);
}

fn blur_at(p: vec2<i32>, direction: vec2<i32>) -> vec4<f32> {
  let size = vec2<i32>(textureDimensions(input_a));
  let lo = vec2<i32>(0);
  let hi = size - vec2<i32>(1);
  return textureLoad(input_a, clamp(p - direction * 2, lo, hi), 0) * 0.0625
    + textureLoad(input_a, clamp(p - direction, lo, hi), 0) * 0.25
    + textureLoad(input_a, clamp(p, lo, hi), 0) * 0.375
    + textureLoad(input_a, clamp(p + direction, lo, hi), 0) * 0.25
    + textureLoad(input_a, clamp(p + direction * 2, lo, hi), 0) * 0.0625;
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
  let glow = textureLoad(input_b, vec2<i32>(p.xy), 0) * 0.75;
  return vec4<f32>(source.rgb + glow.rgb, source.a + glow.a * (1.0 - source.a));
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glow_keeps_hdr_bright_pass_and_adds_it_back() {
        match GlowFixture::new().and_then(|fixture| fixture.render()) {
            Ok(frame) => {
                assert!(frame.bloom.iter().any(|pixel| pixel[0] > 1.0));
                assert!(frame.output.iter().any(|pixel| pixel[0] > 4.0));
            }
            Err(RenderError::AdapterUnavailable | RenderError::InsufficientLimits { .. }) => {}
            Err(error) => panic!("unexpected GPU failure: {error}"),
        }
    }

    #[test]
    fn glow_expands_alpha_only_within_the_declared_two_pixel_radius() {
        match GlowFixture::new().and_then(|fixture| fixture.render()) {
            Ok(frame) => {
                let mut expanded = 0;
                for y in 0..HEIGHT as usize {
                    for x in 0..WIDTH as usize {
                        let index = y * WIDTH as usize + x;
                        let output = frame.output[index];
                        if frame.source[index][3] == 0.0 && output[3] > 0.0 {
                            expanded += 1;
                            assert!((10..=21).contains(&x));
                            assert!((10..=21).contains(&y));
                        }
                        if output[3] == 0.0 {
                            assert_eq!(output, [0.0; 4]);
                        }
                    }
                }
                assert!(expanded > 0);
            }
            Err(RenderError::AdapterUnavailable | RenderError::InsufficientLimits { .. }) => {}
            Err(error) => panic!("unexpected GPU failure: {error}"),
        }
    }

    #[test]
    fn host_owned_textures_are_reused_for_repeat_evaluation() {
        match GlowFixture::new() {
            Ok(fixture) => {
                let first = fixture.render().unwrap();
                let second = fixture.render().unwrap();
                assert_eq!(first, second);
            }
            Err(RenderError::AdapterUnavailable | RenderError::InsufficientLimits { .. }) => {}
            Err(error) => panic!("unexpected GPU failure: {error}"),
        }
    }
}
