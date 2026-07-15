//! wgpu 1枚面へのタイムライン自前描画(インスタンス矩形)。

use std::num::NonZeroU64;

use bytemuck::{Pod, Zeroable};
use motolii_gpu::GpuCtx;

use crate::data::{Clip, Keyframe, TimelineModel, ViewState};

const VIEWPORT_W: f32 = 1920.0;
const VIEWPORT_H: f32 = 512.0;
const TRACK_HEIGHT_PX: f32 = 24.0;
const BASE_PX_PER_SEC: f32 = 12.0;
const KEYFRAME_SIZE_PX: f32 = 7.0;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct RectInstance {
    center: [f32; 2],
    half_size: [f32; 2],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ViewUniform {
    viewport: [f32; 2],
}

pub struct TimelineRenderer {
    pipeline: wgpu::RenderPipeline,
    view_bind_group: wgpu::BindGroup,
    instance_bind_group_layout: wgpu::BindGroupLayout,
    view_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instance_capacity: usize,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
}

pub struct FrameStats {
    pub visible_clips: usize,
    pub visible_keyframes: usize,
    pub cpu_cull_upload_us: u64,
}

impl TimelineRenderer {
    pub fn new(gpu: &GpuCtx) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("timeline-bench-shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("timeline.wgsl").into()),
            });

        let view_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("timeline-view"),
            size: std::mem::size_of::<ViewUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let view_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("timeline-view-layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: NonZeroU64::new(
                                std::mem::size_of::<ViewUniform>() as u64,
                            ),
                        },
                        count: None,
                    }],
                });

        let view_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("timeline-view-bind"),
            layout: &view_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: view_buffer.as_entire_binding(),
            }],
        });

        let instance_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("timeline-instance-layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("timeline-pipeline-layout"),
                bind_group_layouts: &[
                    Some(&view_bind_group_layout),
                    Some(&instance_bind_group_layout),
                ],
                immediate_size: 0,
            });

        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("timeline-pipeline"),
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
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

        let instance_capacity = 128_000;
        let instance_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("timeline-instances"),
            size: (instance_capacity * std::mem::size_of::<RectInstance>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("timeline-target"),
            size: wgpu::Extent3d {
                width: VIEWPORT_W as u32,
                height: VIEWPORT_H as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&Default::default());

        Self {
            pipeline,
            view_bind_group,
            instance_bind_group_layout,
            view_buffer,
            instance_buffer,
            instance_capacity,
            texture,
            texture_view,
        }
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn draw_frame(
        &mut self,
        gpu: &GpuCtx,
        model: &TimelineModel,
        view: ViewState,
    ) -> FrameStats {
        let t0 = std::time::Instant::now();
        let (instances, visible_clips, visible_keyframes) = build_instances(model, view);

        assert!(
            instances.len() <= self.instance_capacity,
            "instance overflow: {}",
            instances.len()
        );

        gpu.queue.write_buffer(
            &self.view_buffer,
            0,
            bytemuck::bytes_of(&ViewUniform {
                viewport: [VIEWPORT_W, VIEWPORT_H],
            }),
        );
        gpu.queue
            .write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));

        let instance_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("timeline-instance-bind"),
            layout: &self.instance_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.instance_buffer.as_entire_binding(),
            }],
        });

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("timeline-encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("timeline-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.11,
                            g: 0.11,
                            b: 0.14,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.view_bind_group, &[]);
            pass.set_bind_group(1, &instance_bind_group, &[]);
            pass.draw(0..6, 0..instances.len() as u32);
        }

        gpu.queue.submit(Some(encoder.finish()));
        gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok();

        FrameStats {
            visible_clips,
            visible_keyframes,
            cpu_cull_upload_us: t0.elapsed().as_micros() as u64,
        }
    }
}

fn build_instances(model: &TimelineModel, view: ViewState) -> (Vec<RectInstance>, usize, usize) {
    let px_per_sec = BASE_PX_PER_SEC * view.zoom;
    let track_h = TRACK_HEIGHT_PX * view.zoom;
    let margin_sec = 8.0 / px_per_sec;
    let time_min = view.pan_sec - margin_sec;
    let time_max = view.pan_sec + VIEWPORT_W / px_per_sec + margin_sec;
    let track_min = view.pan_track - 1.0;
    let track_max = view.pan_track + VIEWPORT_H / track_h + 1.0;

    let mut out = Vec::with_capacity(model.clips.len() + model.keyframes.len() + 64);

    // ビートグリッド風の縦線(薄い)
    let beat_sec = 0.5_f32;
    let first_line = (time_min / beat_sec).floor() as i32;
    let last_line = (time_max / beat_sec).ceil() as i32;
    for i in first_line..=last_line {
        let sec = i as f32 * beat_sec;
        let x = (sec - view.pan_sec) * px_per_sec;
        if !(0.0..VIEWPORT_W).contains(&x) {
            continue;
        }
        let major = i.rem_euclid(4) == 0;
        out.push(RectInstance {
            center: [x, VIEWPORT_H * 0.5],
            half_size: [0.5, VIEWPORT_H * 0.5],
            color: if major {
                [0.28, 0.28, 0.34, 0.55]
            } else {
                [0.22, 0.22, 0.28, 0.35]
            },
        });
    }

    // トラック区切り
    let first_track = track_min.floor() as i32;
    let last_track = track_max.ceil() as i32;
    for track in first_track..=last_track {
        if track < 0 || track as u32 >= model.track_count {
            continue;
        }
        let y = (track as f32 - view.pan_track) * track_h + track_h * 0.5;
        if !(0.0..VIEWPORT_H).contains(&y) {
            continue;
        }
        out.push(RectInstance {
            center: [VIEWPORT_W * 0.5, y],
            half_size: [VIEWPORT_W * 0.5, 0.5],
            color: [0.18, 0.18, 0.22, 0.9],
        });
    }

    let mut visible_clips = 0_usize;
    let mut visible_keyframes = 0_usize;

    for clip in &model.clips {
        if let Some(inst) = clip_instance(clip, view, px_per_sec, track_h, time_min, time_max, track_min, track_max)
        {
            out.push(inst);
            visible_clips += 1;
        }
    }

    for key in &model.keyframes {
        if let Some(inst) =
            keyframe_instance(model, key, view, px_per_sec, track_h, time_min, time_max, track_min, track_max)
        {
            out.push(inst);
            visible_keyframes += 1;
        }
    }

    (out, visible_clips, visible_keyframes)
}

fn clip_instance(
    clip: &Clip,
    view: ViewState,
    px_per_sec: f32,
    track_h: f32,
    time_min: f32,
    time_max: f32,
    track_min: f32,
    track_max: f32,
) -> Option<RectInstance> {
    let end = clip.start_sec + clip.duration_sec;
    if end < time_min || clip.start_sec > time_max {
        return None;
    }
    if clip.track as f32 + 1.0 < track_min || clip.track as f32 > track_max {
        return None;
    }

    let x0 = (clip.start_sec - view.pan_sec) * px_per_sec;
    let x1 = (end - view.pan_sec) * px_per_sec;
    let y = (clip.track as f32 - view.pan_track) * track_h;
    let (r, g, b) = hsv_to_rgb(clip.hue, 0.55, 0.72);
    Some(RectInstance {
        center: [(x0 + x1) * 0.5, y + track_h * 0.5],
        half_size: [((x1 - x0) * 0.5).max(1.0), track_h * 0.42],
        color: [r, g, b, 1.0],
    })
}

fn keyframe_instance(
    model: &TimelineModel,
    key: &Keyframe,
    view: ViewState,
    px_per_sec: f32,
    track_h: f32,
    time_min: f32,
    time_max: f32,
    track_min: f32,
    track_max: f32,
) -> Option<RectInstance> {
    let clip = model.clips.get(key.clip_index as usize)?;
    let abs_sec = clip.start_sec + key.time_in_clip_sec;
    if abs_sec < time_min || abs_sec > time_max {
        return None;
    }
    if clip.track as f32 + 1.0 < track_min || clip.track as f32 > track_max {
        return None;
    }

    let x = (abs_sec - view.pan_sec) * px_per_sec;
    let y = (clip.track as f32 - view.pan_track) * track_h + track_h * 0.5;
    let half = KEYFRAME_SIZE_PX * 0.5;
    Some(RectInstance {
        center: [x, y],
        half_size: [half, half],
        color: [1.0, 0.92, 0.35, 0.95],
    })
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match (h * 6.0) as u32 % 6 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (r + m, g + m, b + m)
}
