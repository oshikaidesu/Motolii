use std::time::Instant;

use skia_safe::{AlphaType, Color, ColorType, ImageInfo, Paint, PaintStyle, Rect, surfaces};
use wgpu::util::DeviceExt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SceneKind {
    Stage,
    Timeline,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderStats {
    pub frame_count: u64,
    pub last_cpu_us: u64,
    pub max_cpu_us: u64,
    pub vertex_bytes: u64,
    pub overlay_uploads: u64,
    pub overlay_last_us: u64,
    pub pointer_downs: u64,
    pub pointer_moves: u64,
    pub pointer_ups: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PointerPhase {
    Down,
    Move,
    Up,
    Cancel,
}

struct StageResources {
    preview: wgpu::Texture,
    overlay: wgpu::Texture,
    preview_pipeline: wgpu::RenderPipeline,
    composite_pipeline: wgpu::RenderPipeline,
    composite_bind_group: wgpu::BindGroup,
    pixels: Vec<u8>,
    dirty: bool,
    gizmo: [f32; 2],
    drag_offset: [f32; 2],
    dragging: bool,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

pub(crate) struct RendererCore {
    _instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    _adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    stage: Option<StageResources>,
    scene: SceneKind,
    selected_object_index: i32,
    playhead: f64,
    frame: u64,
    stats: RenderStats,
}

impl RendererCore {
    pub(crate) fn new(
        instance: wgpu::Instance,
        surface: wgpu::Surface<'static>,
        width: u32,
        height: u32,
        scene: SceneKind,
    ) -> Result<Self, String> {
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .map_err(|error| format!("request adapter: {error}"))?;

        let adapter_limits = adapter.limits();
        let max_texture_dimension_2d = adapter_limits.max_texture_dimension_2d;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Motolii RN native-component probe"),
            required_features: wgpu::Features::empty(),
            required_limits: adapter_limits,
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
        }))
        .map_err(|error| format!("request device: {error}"))?;

        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(capabilities.formats[0]);
        let present_mode = capabilities
            .present_modes
            .iter()
            .copied()
            .find(|mode| *mode == wgpu::PresentMode::Fifo)
            .unwrap_or(capabilities.present_modes[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: width.clamp(1, max_texture_dimension_2d),
            height: height.clamp(1, max_texture_dimension_2d),
            present_mode,
            desired_maximum_frame_latency: 2,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Motolii timeline rectangle shader"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.position, 0.0, 1.0);
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
"#
                .into(),
            ),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Motolii rectangle pipeline layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Motolii rectangle pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 8,
                            shader_location: 1,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let stage = (scene == SceneKind::Stage)
            .then(|| create_stage_resources(&device, format, config.width, config.height));
        Ok(Self {
            _instance: instance,
            surface,
            _adapter: adapter,
            device,
            queue,
            config,
            pipeline,
            stage,
            scene,
            selected_object_index: 1,
            playhead: 0.54,
            frame: 0,
            stats: RenderStats::default(),
        })
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        let max_dimension = self.device.limits().max_texture_dimension_2d;
        let width = width.clamp(1, max_dimension);
        let height = height.clamp(1, max_dimension);
        if self.config.width == width && self.config.height == height {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        if self.scene == SceneKind::Stage {
            self.stage = Some(create_stage_resources(
                &self.device,
                self.config.format,
                width,
                height,
            ));
        }
    }

    pub(crate) fn set_timeline_state(&mut self, selected_object_index: i32, playhead: f64) {
        self.selected_object_index = selected_object_index.clamp(0, 499);
        self.playhead = playhead.clamp(0.0, 1.0);
    }

    pub(crate) fn timeline_hit_test(&self, x: f64, y: f64) -> Option<(i32, f64)> {
        if self.scene != SceneKind::Timeline || self.config.width == 0 || self.config.height == 0 {
            return None;
        }
        let time = (x / f64::from(self.config.width)).clamp(0.0, 0.999_999);
        let track = ((y / f64::from(self.config.height)).clamp(0.0, 0.999_999) * 20.0) as i32;
        let clip = (time * 25.0) as i32;
        Some((track * 25 + clip, time))
    }

    pub(crate) fn stats(&self) -> RenderStats {
        self.stats
    }

    pub(crate) fn stage_pointer(&mut self, phase: PointerPhase, x: f64, y: f64) {
        let Some(stage) = &mut self.stage else { return };
        let point = [x as f32, y as f32];
        match phase {
            PointerPhase::Down => {
                self.stats.pointer_downs += 1;
                let inside = point[0] >= stage.gizmo[0]
                    && point[0] <= stage.gizmo[0] + 360.0
                    && point[1] >= stage.gizmo[1]
                    && point[1] <= stage.gizmo[1] + 220.0;
                if inside {
                    stage.dragging = true;
                    stage.drag_offset = [point[0] - stage.gizmo[0], point[1] - stage.gizmo[1]];
                }
            }
            PointerPhase::Move => {
                self.stats.pointer_moves += 1;
                if stage.dragging {
                    stage.gizmo = [
                        point[0] - stage.drag_offset[0],
                        point[1] - stage.drag_offset[1],
                    ];
                    stage.dirty = true;
                }
            }
            PointerPhase::Up => {
                self.stats.pointer_ups += 1;
                stage.dragging = false;
                stage.dirty = true;
            }
            PointerPhase::Cancel => {
                stage.dragging = false;
                stage.dirty = true;
            }
        }
    }

    pub(crate) fn render(&mut self) -> Result<(), String> {
        let started = Instant::now();
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(output)
            | wgpu::CurrentSurfaceTexture::Suboptimal(output) => output,
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => return Err("surface lost".into()),
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err("surface validation failure".into());
            }
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let vertices = if self.scene == SceneKind::Timeline {
            self.timeline_vertices()
        } else {
            Vec::new()
        };
        let vertex_buffer = (!vertices.is_empty()).then(|| {
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Motolii timeline frame vertices"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                })
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Motolii native-component frame"),
            });

        let clear = wgpu::Color {
            r: 0.035,
            g: 0.041,
            b: 0.050,
            a: 1.0,
        };
        if self.scene == SceneKind::Stage {
            let stage = self.stage.as_mut().expect("stage resources");
            if stage.dirty {
                let overlay_started = Instant::now();
                draw_stage_overlay(
                    &mut stage.pixels,
                    self.config.width,
                    self.config.height,
                    stage.gizmo,
                    stage.dragging,
                );
                self.queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &stage.overlay,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &stage.pixels,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(self.config.width * 4),
                        rows_per_image: Some(self.config.height),
                    },
                    wgpu::Extent3d {
                        width: self.config.width,
                        height: self.config.height,
                        depth_or_array_layers: 1,
                    },
                );
                stage.dirty = false;
                self.stats.overlay_uploads += 1;
                self.stats.overlay_last_us = overlay_started.elapsed().as_micros() as u64;
            }
            let preview_view = stage.preview.create_view(&Default::default());
            {
                let attachments = [Some(wgpu::RenderPassColorAttachment {
                    view: &preview_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })];
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Motolii preview texture"),
                    color_attachments: &attachments,
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                pass.set_pipeline(&stage.preview_pipeline);
                pass.draw(0..3, 0..1);
            }
            {
                let attachments = [Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })];
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Motolii Skia overlay composite"),
                    color_attachments: &attachments,
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                pass.set_pipeline(&stage.composite_pipeline);
                pass.set_bind_group(0, &stage.composite_bind_group, &[]);
                pass.draw(0..3, 0..1);
            }
        } else {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Motolii native frame"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if let Some(buffer) = &vertex_buffer {
                pass.set_pipeline(&self.pipeline);
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..vertices.len() as u32, 0..1);
            }
        }
        self.queue.submit(Some(encoder.finish()));
        output.present();
        self.frame = self.frame.wrapping_add(1);
        let cpu_us = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
        self.stats.frame_count = self.stats.frame_count.wrapping_add(1);
        self.stats.last_cpu_us = cpu_us;
        self.stats.max_cpu_us = self.stats.max_cpu_us.max(cpu_us);
        self.stats.vertex_bytes = (vertices.len() * std::mem::size_of::<Vertex>()) as u64;
        Ok(())
    }

    fn timeline_vertices(&self) -> Vec<Vertex> {
        let mut vertices = Vec::with_capacity(9_100);
        let palette = [
            [0.35, 0.48, 0.78, 1.0],
            [0.55, 0.36, 0.69, 1.0],
            [0.34, 0.58, 0.48, 1.0],
            [0.65, 0.43, 0.27, 1.0],
        ];
        for track in 0..20 {
            let track_top = track as f32 / 20.0;
            let track_bottom = (track + 1) as f32 / 20.0;
            push_rect(
                &mut vertices,
                0.0,
                track_bottom - 0.002,
                1.0,
                track_bottom,
                [0.18, 0.20, 0.23, 1.0],
            );
            for clip in 0..25 {
                let index = track * 25 + clip;
                let left = clip as f32 / 25.0 + 0.0015;
                let right = (clip + 1) as f32 / 25.0 - 0.0015;
                let top = track_top + 0.006;
                let bottom = track_bottom - 0.006;
                let color = if index == self.selected_object_index as usize {
                    [0.88, 0.78, 0.34, 1.0]
                } else {
                    palette[index % palette.len()]
                };
                push_rect(&mut vertices, left, top, right, bottom, color);

                let inner_width = (right - left) * 0.16;
                for sample in 0..2 {
                    let wave_left = left + 0.004 + sample as f32 * inner_width * 1.2;
                    let amplitude = 0.18 + ((index + sample * 7) % 5) as f32 * 0.09;
                    let mid = (top + bottom) * 0.5;
                    let half = (bottom - top) * amplitude * 0.5;
                    push_rect(
                        &mut vertices,
                        wave_left,
                        mid - half,
                        (wave_left + inner_width).min(right - 0.003),
                        mid + half,
                        [0.92, 0.94, 0.92, 0.36],
                    );
                }
            }
        }
        let playhead = self.playhead as f32;
        push_rect(
            &mut vertices,
            (playhead - 0.0012).max(0.0),
            0.0,
            (playhead + 0.0012).min(1.0),
            1.0,
            [0.96, 0.88, 0.38, 0.95],
        );
        vertices
    }
}

fn create_stage_resources(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    width: u32,
    height: u32,
) -> StageResources {
    let texture = |label, format, usage| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        })
    };
    let preview = texture(
        "Motolii product preview texture",
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
    );
    let overlay = texture(
        "Motolii cached Skia overlay",
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
    );
    let fullscreen = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Motolii preview shader"),
        source: wgpu::ShaderSource::Wgsl(r#"
            struct O { @builtin(position) p:vec4<f32>, @location(0) uv:vec2<f32> };
            @vertex fn vs(@builtin(vertex_index) i:u32)->O { var p=array<vec2<f32>,3>(vec2(-1.,-3.),vec2(3.,1.),vec2(-1.,1.)); var o:O; o.p=vec4(p[i],0.,1.); o.uv=vec2((p[i].x+1.)*.5,(1.-p[i].y)*.5); return o; }
            @fragment fn preview_fs(i:O)->@location(0) vec4<f32> { let q=i.uv*2.-1.; let glow=.11/max(length(q-vec2(.18,-.12)),.12); let checker=select(.025,.055,((u32(i.uv.x*24.)+u32(i.uv.y*14.))&1u)==1u); return vec4(.035+checker+glow*.22,.045+checker+glow*.10,.065+checker+glow*.28,1.); }
        "#.into()),
    });
    let preview_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        immediate_size: 0,
    });
    let preview_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Motolii preview pipeline"),
        layout: Some(&preview_layout),
        vertex: wgpu::VertexState {
            module: &fullscreen,
            entry_point: Some("vs"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &fullscreen,
            entry_point: Some("preview_fs"),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    });
    let texture_entry = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            multisampled: false,
            view_dimension: wgpu::TextureViewDimension::D2,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
        },
        count: None,
    };
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            texture_entry(0),
            texture_entry(1),
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let composite_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Motolii Skia composite shader"),
        source: wgpu::ShaderSource::Wgsl(r#"
            @group(0) @binding(0) var preview:texture_2d<f32>; @group(0) @binding(1) var overlay:texture_2d<f32>; @group(0) @binding(2) var samp:sampler;
            struct O { @builtin(position) p:vec4<f32>, @location(0) uv:vec2<f32> };
            @vertex fn vs(@builtin(vertex_index) i:u32)->O { var p=array<vec2<f32>,3>(vec2(-1.,-3.),vec2(3.,1.),vec2(-1.,1.)); var o:O; o.p=vec4(p[i],0.,1.); o.uv=vec2((p[i].x+1.)*.5,(1.-p[i].y)*.5); return o; }
            @fragment fn fs(i:O)->@location(0) vec4<f32> { let b=textureSample(preview,samp,i.uv); let o=textureSample(overlay,samp,i.uv); return vec4(o.rgb+b.rgb*(1.-o.a),1.); }
        "#.into()),
    });
    let composite_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });
    let composite_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Motolii Skia composite pipeline"),
        layout: Some(&composite_layout),
        vertex: wgpu::VertexState {
            module: &composite_shader,
            entry_point: Some("vs"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &composite_shader,
            entry_point: Some("fs"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    });
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
    let composite_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &preview.create_view(&Default::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(
                    &overlay.create_view(&Default::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    StageResources {
        preview,
        overlay,
        preview_pipeline,
        composite_pipeline,
        composite_bind_group,
        pixels: vec![0; width as usize * height as usize * 4],
        dirty: true,
        gizmo: [width as f32 * 0.34, height as f32 * 0.32],
        drag_offset: [0.0; 2],
        dragging: false,
    }
}

fn draw_stage_overlay(bytes: &mut [u8], width: u32, height: u32, gizmo: [f32; 2], dragging: bool) {
    let info = ImageInfo::new(
        (width as i32, height as i32),
        ColorType::RGBA8888,
        AlphaType::Premul,
        None,
    );
    let mut surface =
        surfaces::wrap_pixels(&info, bytes, Some(width as usize * 4), None).expect("Skia pixels");
    let canvas = surface.canvas();
    canvas.clear(Color::TRANSPARENT);
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(PaintStyle::Stroke);
    paint.set_stroke_width(1.0);
    for x in (0..width).step_by(128) {
        paint.set_color(if x % 512 == 0 {
            Color::from_argb(90, 110, 180, 235)
        } else {
            Color::from_argb(34, 110, 180, 235)
        });
        canvas.draw_line((x as f32, 0.0), (x as f32, height as f32), &paint);
    }
    for y in (0..height).step_by(128) {
        paint.set_color(if y % 512 == 0 {
            Color::from_argb(90, 110, 180, 235)
        } else {
            Color::from_argb(34, 110, 180, 235)
        });
        canvas.draw_line((0.0, y as f32), (width as f32, y as f32), &paint);
    }
    let rect = Rect::from_xywh(gizmo[0], gizmo[1], 360.0, 220.0);
    paint.set_color(if dragging {
        Color::from_argb(250, 255, 95, 115)
    } else {
        Color::from_argb(245, 255, 207, 75)
    });
    paint.set_stroke_width(3.0);
    canvas.draw_rect(rect, &paint);
    paint.set_style(PaintStyle::Fill);
    for &(x, y) in &[
        (rect.left, rect.top),
        (rect.center_x(), rect.top),
        (rect.right, rect.top),
        (rect.left, rect.center_y()),
        (rect.right, rect.center_y()),
        (rect.left, rect.bottom),
        (rect.center_x(), rect.bottom),
        (rect.right, rect.bottom),
    ] {
        canvas.draw_circle((x, y), 8.0, &paint);
    }
    paint.set_style(PaintStyle::Stroke);
    paint.set_color(Color::from_argb(235, 75, 220, 245));
    paint.set_stroke_width(2.0);
    canvas.draw_line(
        (rect.center_x(), 0.0),
        (rect.center_x(), height as f32),
        &paint,
    );
    canvas.draw_line(
        (0.0, rect.center_y()),
        (width as f32, rect.center_y()),
        &paint,
    );
}

fn push_rect(
    vertices: &mut Vec<Vertex>,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    color: [f32; 4],
) {
    let l = left * 2.0 - 1.0;
    let r = right * 2.0 - 1.0;
    let t = 1.0 - top * 2.0;
    let b = 1.0 - bottom * 2.0;
    vertices.extend_from_slice(&[
        Vertex {
            position: [l, t],
            color,
        },
        Vertex {
            position: [l, b],
            color,
        },
        Vertex {
            position: [r, b],
            color,
        },
        Vertex {
            position: [l, t],
            color,
        },
        Vertex {
            position: [r, b],
            color,
        },
        Vertex {
            position: [r, t],
            color,
        },
    ]);
}
