use bytemuck::{Pod, Zeroable};
use thiserror::Error;

pub const WIDTH: u32 = 32;
pub const HEIGHT: u32 = 32;
pub const READBACK_BYTES_PER_ROW: u32 = 256;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialCase {
    pub base_color: [f32; 3],
    pub metallic: f32,
    pub roughness: f32,
    pub normal: [f32; 3],
    pub emissive: [f32; 3],
    pub environment: [f32; 3],
    pub unlit: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pixel {
    pub rgba: [u8; 4],
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("no compatible wgpu adapter is available")]
    AdapterUnavailable,
    #[error("adapter lacks the required offscreen limits: {reason}")]
    InsufficientLimits { reason: &'static str },
    #[error("wgpu device creation failed: {0}")]
    Device(String),
    #[error("shader or pipeline creation failed: {0}")]
    Pipeline(String),
    #[error("GPU readback failed: {0}")]
    Readback(String),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GpuLimits {
    pub max_texture_dimension_2d: u32,
    pub max_storage_buffer_binding_size: u64,
}

pub fn required_limits(adapter: &wgpu::Adapter) -> Result<GpuLimits, RenderError> {
    let limits = adapter.limits();
    if limits.max_texture_dimension_2d < WIDTH || limits.max_texture_dimension_2d < HEIGHT {
        return Err(RenderError::InsufficientLimits {
            reason: "max_texture_dimension_2d",
        });
    }
    if limits.max_storage_buffer_binding_size < 256 {
        return Err(RenderError::InsufficientLimits {
            reason: "max_storage_buffer_binding_size",
        });
    }
    Ok(GpuLimits {
        max_texture_dimension_2d: limits.max_texture_dimension_2d,
        max_storage_buffer_binding_size: limits.max_storage_buffer_binding_size,
    })
}

pub fn render_case(case: MaterialCase) -> Result<Pixel, RenderError> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .map_err(|_| RenderError::AdapterUnavailable)?;
    required_limits(&adapter)?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("m5-r0-headless-pbr"),
        ..Default::default()
    }))
    .map_err(|error| RenderError::Device(error.to_string()))?;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("m5-r0-material-shader"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("m5-r0-material-layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(std::num::NonZeroU64::new(64).unwrap()),
            },
            count: None,
        }],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("m5-r0-pipeline-layout"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("m5-r0-material-pipeline"),
        layout: Some(&pipeline_layout),
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
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        multiview_mask: None,
        cache: None,
    });

    let uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("m5-r0-material-uniform"),
        size: 64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let bytes = MaterialUniform::from_case(case);
    queue.write_buffer(&uniform, 0, bytemuck::bytes_of(&bytes));
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("m5-r0-material-bind-group"),
        layout: &layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform.as_entire_binding(),
        }],
    });

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("m5-r0-offscreen-target"),
        size: wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let buffer = readback_buffer(&device);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("m5-r0-render-encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("m5-r0-render-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(READBACK_BYTES_PER_ROW),
                rows_per_image: Some(HEIGHT),
            },
        },
        wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    let slice = buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |result| {
        if let Err(error) = result {
            eprintln!("readback map failed: {error}");
        }
    });
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })
        .map_err(|error| RenderError::Readback(error.to_string()))?;
    let mapped = slice.get_mapped_range();
    let pixel = Pixel {
        rgba: mapped[0..4]
            .try_into()
            .map_err(|_| RenderError::Readback("short mapped range".to_owned()))?,
    };
    drop(mapped);
    buffer.unmap();
    Ok(pixel)
}

fn readback_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("m5-r0-readback"),
        size: (READBACK_BYTES_PER_ROW * HEIGHT) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct MaterialUniform {
    base_color: [f32; 4],
    normal_metallic: [f32; 4],
    emissive_roughness: [f32; 4],
    environment_mode: [f32; 4],
}

impl MaterialUniform {
    fn from_case(case: MaterialCase) -> Self {
        Self {
            base_color: [
                case.base_color[0],
                case.base_color[1],
                case.base_color[2],
                1.0,
            ],
            normal_metallic: [
                case.normal[0],
                case.normal[1],
                case.normal[2],
                case.metallic,
            ],
            emissive_roughness: [
                case.emissive[0],
                case.emissive[1],
                case.emissive[2],
                case.roughness,
            ],
            environment_mode: [
                case.environment[0],
                case.environment[1],
                case.environment[2],
                if case.unlit { 1.0 } else { 0.0 },
            ],
        }
    }
}

pub fn cpu_reference(case: MaterialCase) -> [u8; 4] {
    let normal = normalize(case.normal);
    let light = normalize([0.4, 0.5, 0.7]);
    let view = [0.0, 0.0, 1.0];
    let ndotl = dot(normal, light).max(0.0);
    let diffuse = scale(case.base_color, (1.0 - case.metallic) * ndotl);
    let reflected = reflect(scale(light, -1.0), normal);
    let specular = [dot(reflected, view)
        .max(0.0)
        .powf(8.0 + (1.0 - case.roughness) * 56.0); 3];
    let mut color = if case.unlit {
        case.base_color
    } else {
        add(
            add(scale(case.environment, 0.25 + ndotl * 0.25), diffuse),
            scale(specular, 0.08 + case.metallic * 0.22),
        )
    };
    color = add(color, case.emissive);
    [
        (color[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (color[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (color[2].clamp(0.0, 1.0) * 255.0).round() as u8,
        255,
    ]
}

fn normalize(value: [f32; 3]) -> [f32; 3] {
    let length = dot(value, value).sqrt().max(f32::EPSILON);
    scale(value, 1.0 / length)
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn scale(value: [f32; 3], factor: f32) -> [f32; 3] {
    [value[0] * factor, value[1] * factor, value[2] * factor]
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn reflect(value: [f32; 3], normal: [f32; 3]) -> [f32; 3] {
    add(value, scale(normal, -(2.0 * dot(normal, value))))
}

const SHADER: &str = r#"
struct Material {
  base_color: vec4<f32>,
  normal_metallic: vec4<f32>,
  emissive_roughness: vec4<f32>,
  environment_mode: vec4<f32>,
};
@group(0) @binding(0) var<uniform> material: Material;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
  let positions = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
  return vec4<f32>(positions[index], 0.0, 1.0);
}

fn dot3(a: vec3<f32>, b: vec3<f32>) -> f32 { return dot(a, b); }
fn reflect3(value: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
  return value + normal * (2.0 * dot3(normal, value) * -1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
  let normal = normalize(material.normal_metallic.xyz);
  let light = normalize(vec3<f32>(0.4, 0.5, 0.7));
  let view = vec3<f32>(0.0, 0.0, 1.0);
  let ndotl = max(dot3(normal, light), 0.0);
  let diffuse = material.base_color.rgb * (1.0 - material.normal_metallic.w) * ndotl;
  let reflected = reflect3(-light, normal);
  let specular = vec3<f32>(pow(max(dot3(reflected, view), 0.0), 8.0 + (1.0 - material.emissive_roughness.w) * 56.0));
  var color = material.base_color.rgb;
  if material.environment_mode.w < 0.5 {
    color = material.environment_mode.rgb * (0.25 + ndotl * 0.25)
      + diffuse
      + specular * (0.08 + material.normal_metallic.w * 0.22);
  }
  color = color + material.emissive_roughness.rgb;
  return vec4<f32>(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn cases() -> [MaterialCase; 4] {
        [
            MaterialCase {
                base_color: [0.7, 0.2, 0.1],
                metallic: 0.0,
                roughness: 0.6,
                normal: [0.0, 0.0, 1.0],
                emissive: [0.0, 0.0, 0.0],
                environment: [0.12, 0.15, 0.2],
                unlit: true,
            },
            MaterialCase {
                base_color: [0.7, 0.2, 0.1],
                metallic: 0.0,
                roughness: 0.6,
                normal: [0.0, 0.0, 1.0],
                emissive: [0.0, 0.0, 0.0],
                environment: [0.12, 0.15, 0.2],
                unlit: false,
            },
            MaterialCase {
                base_color: [0.7, 0.2, 0.1],
                metallic: 0.9,
                roughness: 0.2,
                normal: [0.0, 0.0, 1.0],
                emissive: [0.0, 0.0, 0.0],
                environment: [0.12, 0.15, 0.2],
                unlit: false,
            },
            MaterialCase {
                base_color: [0.2, 0.5, 0.8],
                metallic: 0.1,
                roughness: 0.4,
                normal: [0.35, -0.2, 0.92],
                emissive: [0.05, 0.02, 0.01],
                environment: [0.35, 0.2, 0.1],
                unlit: false,
            },
        ]
    }

    #[test]
    fn cpu_reference_covers_material_matrix() {
        let outputs: Vec<_> = cases().into_iter().map(cpu_reference).collect();
        assert_eq!(outputs.len(), 4);
        assert!(outputs.iter().all(|pixel| pixel[3] == 255));
        assert_ne!(outputs[0], outputs[1]);
        assert_ne!(outputs[1], outputs[2]);
        assert_ne!(outputs[2], outputs[3]);
    }

    #[test]
    fn gpu_matches_cpu_reference_or_returns_typed_refusal() {
        for case in cases() {
            match render_case(case) {
                Ok(pixel) => {
                    let expected = cpu_reference(case);
                    assert!(pixel
                        .rgba
                        .iter()
                        .zip(expected)
                        .all(|(actual, expected)| actual.abs_diff(expected) <= 2));
                }
                Err(RenderError::AdapterUnavailable | RenderError::InsufficientLimits { .. }) => {}
                Err(error) => panic!("unexpected GPU failure: {error}"),
            }
        }
    }
}
