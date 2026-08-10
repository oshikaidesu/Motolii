use std::num::NonZeroU64;

use bytemuck::{Pod, Zeroable};

use crate::{
    readback_buffer, readback_frame, required_limits, Frame, RenderError, HEIGHT,
    READBACK_BYTES_PER_ROW, WIDTH,
};

const MAX_REPLAY_FRAME: u32 = 15;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct FeedbackUniform {
    frame_index: u32,
    _padding: [u32; 3],
}

/// Replays a tiny host-owned feedback sequence from its transparent initial condition.
pub fn render_feedback_trail(target_frame: u32) -> Result<Frame, RenderError> {
    if target_frame > MAX_REPLAY_FRAME {
        return Err(RenderError::ReplayTooLong {
            target_frame,
            max_frame: MAX_REPLAY_FRAME,
        });
    }

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .map_err(|_| RenderError::AdapterUnavailable)?;
    required_limits(&adapter)?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("m5-feedback-trail-proof"),
        ..Default::default()
    }))
    .map_err(|error| RenderError::Device(error.to_string()))?;

    let textures = [
        feedback_texture(&device, "m5-feedback-history-a"),
        feedback_texture(&device, "m5-feedback-history-b"),
    ];
    let views = textures
        .each_ref()
        .map(|texture| texture.create_view(&wgpu::TextureViewDescriptor::default()));
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("m5-feedback-trail-shader"),
        source: wgpu::ShaderSource::Wgsl(FEEDBACK_SHADER.into()),
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("m5-feedback-trail-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: NonZeroU64::new(16),
                },
                count: None,
            },
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("m5-feedback-trail-pipeline-layout"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("m5-feedback-trail-pipeline"),
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

    let uniform_stride = device.limits().min_uniform_buffer_offset_alignment.max(16) as u64;
    let uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("m5-feedback-frame-uniforms"),
        size: uniform_stride * u64::from(target_frame + 1),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    for frame_index in 0..=target_frame {
        queue.write_buffer(
            &uniform,
            uniform_stride * u64::from(frame_index),
            bytemuck::bytes_of(&FeedbackUniform {
                frame_index,
                _padding: [0; 3],
            }),
        );
    }
    let bind_groups = views.each_ref().map(|view| {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("m5-feedback-history-bind-group"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &uniform,
                        offset: 0,
                        size: NonZeroU64::new(16),
                    }),
                },
            ],
        })
    });

    let buffer = readback_buffer(&device);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("m5-feedback-trail-encoder"),
    });
    for view in &views {
        let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("m5-feedback-explicit-initial-clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
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
    }
    for frame_index in 0..=target_frame {
        let input = (frame_index % 2) as usize;
        let output = 1 - input;
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("m5-feedback-trail-step"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &views[output],
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
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(
            0,
            &bind_groups[input],
            &[(uniform_stride * u64::from(frame_index)) as u32],
        );
        pass.draw(0..3, 0..1);
    }
    let final_texture = &textures[(1 - (target_frame % 2)) as usize];
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: final_texture,
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
    readback_frame(&device, &buffer)
}

fn feedback_texture(device: &wgpu::Device, label: &'static str) -> wgpu::Texture {
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
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

const FEEDBACK_SHADER: &str = r#"
struct FeedbackParams {
  frame_index: u32,
  _padding_0: u32,
  _padding_1: u32,
  _padding_2: u32,
};
@group(0) @binding(0) var history: texture_2d<f32>;
@group(0) @binding(1) var<uniform> params: FeedbackParams;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
  let positions = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
  return vec4<f32>(positions[index], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let previous = textureLoad(history, vec2<i32>(position.xy), 0) * 0.72;
  let center = vec2<f32>(5.0 + f32(params.frame_index) * 4.0, 16.0);
  let current_alpha = select(0.0, 1.0, distance(position.xy, center) <= 2.25);
  let current = vec4<f32>(0.15, 0.85, 0.55, 1.0) * current_alpha;
  return current + previous * (1.0 - current.a);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(frame: &Frame, x: u32, y: u32) -> [u8; 4] {
        frame.pixels[(y * frame.width + x) as usize].rgba
    }

    #[test]
    fn feedback_replay_is_deterministic_and_retains_alpha_trail_or_returns_typed_refusal() {
        match (render_feedback_trail(4), render_feedback_trail(4)) {
            (Ok(first), Ok(replay)) => {
                assert_eq!(first, replay, "fresh seek replay must reproduce the frame");
                assert_eq!(pixel(&first, 0, 0), [0, 0, 0, 0]);
                let oldest = pixel(&first, 5, 16);
                let current = pixel(&first, 21, 16);
                assert!(oldest[3] > 0 && oldest[3] < current[3]);
                assert_eq!(current[3], 255);
            }
            (Err(RenderError::AdapterUnavailable | RenderError::InsufficientLimits { .. }), _)
            | (_, Err(RenderError::AdapterUnavailable | RenderError::InsufficientLimits { .. })) => {
            }
            (Err(error), _) | (_, Err(error)) => panic!("unexpected GPU failure: {error}"),
        }
    }

    #[test]
    fn feedback_replay_is_explicitly_bounded() {
        assert!(matches!(
            render_feedback_trail(MAX_REPLAY_FRAME + 1),
            Err(RenderError::ReplayTooLong { .. })
        ));
    }
}
