use std::time::Instant;

use re_renderer::view_builder::{OrthographicCameraMode, Projection, TargetConfiguration};
use re_renderer::{Color32, LineDrawableBuilder, PointCloudBuilder, Size, ViewBuilder};
use skia_safe::{AlphaType, Color, ColorType, ImageInfo, Paint, PaintStyle, Rect, surfaces};
use wgpu::util::DeviceExt;

const CHROMA_VIDEO_BYTES: &[u8] = include_bytes!("../fixtures/chroma-key.mp4");

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

/// Timelineは頂点quadではなくSkia rasterを1枚上げてblitする。
/// Stageのoverlayと同じ経路(CPU raster → write_texture → 全画面blit)である。
struct TimelineResources {
    surface_texture: wgpu::Texture,
    blit_pipeline: wgpu::RenderPipeline,
    blit_bind_group: wgpu::BindGroup,
    pixels: Vec<u8>,
    dirty: bool,
}

struct StageResources {
    preview: wgpu::Texture,
    overlay: wgpu::Texture,
    rerun: re_renderer::RenderContext,
    video: re_renderer::video::Video,
    video_started: Instant,
    video_duration: f64,
    chroma_pipeline: wgpu::RenderPipeline,
    chroma_bind_group_layout: wgpu::BindGroupLayout,
    chroma_sampler: wgpu::Sampler,
    chroma_bind_group: Option<wgpu::BindGroup>,
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
    timeline: Option<TimelineResources>,
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
            .then(|| {
                create_stage_resources(
                    &adapter,
                    &device,
                    &queue,
                    format,
                    config.width,
                    config.height,
                )
            })
            .transpose()?;
        let timeline = (scene == SceneKind::Timeline)
            .then(|| create_timeline_resources(&device, format, config.width, config.height));
        Ok(Self {
            _instance: instance,
            surface,
            _adapter: adapter,
            device,
            queue,
            config,
            pipeline,
            stage,
            timeline,
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
            self.stage = Some(
                create_stage_resources(
                    &self._adapter,
                    &self.device,
                    &self.queue,
                    self.config.format,
                    width,
                    height,
                )
                .expect("Rerun Stage resources were valid at startup"),
            );
        }
        if self.scene == SceneKind::Timeline {
            self.timeline = Some(create_timeline_resources(
                &self.device,
                self.config.format,
                width,
                height,
            ));
        }
    }

    pub(crate) fn set_timeline_state(&mut self, selected_object_index: i32, playhead: f64) {
        self.selected_object_index = selected_object_index.max(0);
        self.playhead = playhead.clamp(0.0, 1.0);
        if let Some(timeline) = &mut self.timeline {
            timeline.dirty = true;
        }
    }

    pub(crate) fn timeline_hit_test(&self, x: f64, y: f64) -> Option<(i32, f64)> {
        if self.scene != SceneKind::Timeline {
            return None;
        }
        crate::timeline_skia::hit_test(self.config.width, self.config.height, x, y)
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
        // Timelineは頂点quadを捨ててSkia rasterへ移した。Stage以外の残りだけがこの経路を使う。
        let vertices: Vec<Vertex> = Vec::new();
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
            render_rerun_stage(stage, &preview_view, self.config.width, self.config.height)?;
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
        } else if self.scene == SceneKind::Timeline {
            let timeline = self.timeline.as_mut().expect("timeline resources");
            if timeline.dirty {
                let raster_started = Instant::now();
                crate::timeline_skia::draw_timeline(
                    &mut timeline.pixels,
                    self.config.width,
                    self.config.height,
                    self.playhead,
                    self.selected_object_index,
                );
                self.queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &timeline.surface_texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &timeline.pixels,
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
                timeline.dirty = false;
                self.stats.overlay_uploads += 1;
                self.stats.overlay_last_us = raster_started.elapsed().as_micros() as u64;
            }
            let attachments = [Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear),
                    store: wgpu::StoreOp::Store,
                },
            })];
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Motolii Skia timeline blit"),
                color_attachments: &attachments,
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&timeline.blit_pipeline);
            pass.set_bind_group(0, &timeline.blit_bind_group, &[]);
            pass.draw(0..3, 0..1);
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
}

/// Skia rasterを1枚受け取って全画面へblitするだけの資源。
///
/// textureは`Rgba8UnormSrgb`にする。Skiaが書くのはsRGBのbyteであり、
/// surfaceもsRGBなので、sampling時にsRGB→linear、書き出しでlinear→sRGBへ戻り往復が恒等になる。
/// `Rgba8Unorm`にすると1回分の変換が余計にかかって全体が白茶ける。
fn create_timeline_resources(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    width: u32,
    height: u32,
) -> TimelineResources {
    let surface_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Motolii Skia timeline raster"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Motolii Skia timeline blit shader"),
        source: wgpu::ShaderSource::Wgsl(
            r#"
            @group(0) @binding(0) var raster:texture_2d<f32>; @group(0) @binding(1) var samp:sampler;
            struct O { @builtin(position) p:vec4<f32>, @location(0) uv:vec2<f32> };
            @vertex fn vs(@builtin(vertex_index) i:u32)->O { var p=array<vec2<f32>,3>(vec2(-1.,-3.),vec2(3.,1.),vec2(-1.,1.)); var o:O; o.p=vec4(p[i],0.,1.); o.uv=vec2((p[i].x+1.)*.5,(1.-p[i].y)*.5); return o; }
            @fragment fn fs(i:O)->@location(0) vec4<f32> { return vec4(textureSample(raster,samp,i.uv).rgb,1.); }
        "#
            .into(),
        ),
    });
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Motolii Skia timeline blit layout"),
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
        ],
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });
    let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Motolii Skia timeline blit pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
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
    let blit_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &surface_texture.create_view(&Default::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    TimelineResources {
        surface_texture,
        blit_pipeline,
        blit_bind_group,
        pixels: vec![0; width as usize * height as usize * 4],
        dirty: true,
    }
}

fn create_stage_resources(
    adapter: &wgpu::Adapter,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    surface_format: wgpu::TextureFormat,
    width: u32,
    height: u32,
) -> Result<StageResources, String> {
    let texture = |label, format, usage, view_formats: &[wgpu::TextureFormat]| {
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
            view_formats,
        })
    };
    let preview = texture(
        "Motolii product preview texture",
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        &[wgpu::TextureFormat::Rgba8UnormSrgb],
    );
    let overlay = texture(
        "Motolii cached Skia overlay",
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        &[],
    );
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
                resource: wgpu::BindingResource::TextureView(&preview.create_view(
                    &wgpu::TextureViewDescriptor {
                        format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
                        ..Default::default()
                    },
                )),
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
    let rerun = re_renderer::RenderContext::new(
        adapter,
        device.clone(),
        queue.clone(),
        wgpu::TextureFormat::Rgba8Unorm,
        re_renderer::RenderConfig::best_for_device_caps,
    )
    .map_err(|error| format!("create embedded Rerun renderer: {error}"))?;
    let video_description =
        re_video::VideoDataDescription::load_mp4(CHROMA_VIDEO_BYTES, "Motolii B002 chroma fixture")
            .map_err(|error| format!("load B002 chroma fixture: {error}"))?;
    let video_duration = video_description
        .duration()
        .map_or(2.0, |duration| duration.as_secs_f64());
    let video = re_renderer::video::Video::load(
        "Motolii B002 chroma fixture".into(),
        video_description,
        re_video::DecodeSettings::default(),
    );
    let chroma_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Motolii B002 chroma layout"),
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
            ],
        });
    let chroma_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Motolii B002 chroma shader"),
        source: wgpu::ShaderSource::Wgsl(
            r#"
            @group(0) @binding(0) var video:texture_2d<f32>;
            @group(0) @binding(1) var samp:sampler;
            struct O { @builtin(position) p:vec4<f32>, @location(0) uv:vec2<f32> };
            @vertex fn vs(@builtin(vertex_index) i:u32)->O {
                var p=array<vec2<f32>,3>(vec2(-1.,-3.),vec2(3.,1.),vec2(-1.,1.));
                var o:O; o.p=vec4(p[i],0.,1.); o.uv=vec2((p[i].x+1.)*.5,(1.-p[i].y)*.5); return o;
            }
            @fragment fn fs(i:O)->@location(0) vec4<f32> {
                let lo=vec2(.34,.20); let size=vec2(.52,.62); let uv=(i.uv-lo)/size;
                if (any(uv < vec2(0.)) || any(uv > vec2(1.))) { discard; }
                let c=textureSample(video,samp,uv);
                let green=c.g-max(c.r,c.b);
                let alpha=1.-smoothstep(.08,.24,green);
                let despilled=vec3(c.r,min(c.g,max(c.r,c.b)+.05),c.b);
                return vec4(despilled,alpha);
            }
            "#
            .into(),
        ),
    });
    let chroma_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Motolii B002 chroma pipeline layout"),
        bind_group_layouts: &[Some(&chroma_bind_group_layout)],
        immediate_size: 0,
    });
    let chroma_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Motolii B002 chroma pipeline"),
        layout: Some(&chroma_layout),
        vertex: wgpu::VertexState {
            module: &chroma_shader,
            entry_point: Some("vs"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &chroma_shader,
            entry_point: Some("fs"),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
    let chroma_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Motolii B002 chroma sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    Ok(StageResources {
        preview,
        overlay,
        rerun,
        video,
        video_started: Instant::now(),
        video_duration,
        chroma_pipeline,
        chroma_bind_group_layout,
        chroma_sampler,
        chroma_bind_group: None,
        composite_pipeline,
        composite_bind_group,
        pixels: vec![0; width as usize * height as usize * 4],
        dirty: true,
        gizmo: [width as f32 * 0.34, height as f32 * 0.32],
        drag_offset: [0.0; 2],
        dragging: false,
    })
}

fn render_rerun_stage(
    stage: &mut StageResources,
    target: &wgpu::TextureView,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let context = &mut stage.rerun;
    context.begin_frame();
    stage.video.begin_frame();

    let shape_commands = encode_rerun_stage_shapes(context, target, width, height)?;

    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Motolii embedded Rerun video overlay"),
        });
    let timescale = stage
        .video
        .data_descr()
        .timescale
        .unwrap_or(re_video::Timescale::NANOSECOND);
    let requested_seconds = stage.video_started.elapsed().as_secs_f64() % stage.video_duration;
    let video_frame = stage.video.frame_at(
        context,
        re_video::player::VideoPlayerStreamId(0),
        re_video::Time::from_secs(requested_seconds, timescale),
        &re_video::player::VideoSliceSource(CHROMA_VIDEO_BYTES),
    );
    if let Some(error) = video_frame.error {
        return Err(format!("decode B002 chroma fixture: {error}"));
    }
    if let Some(texture) = video_frame.output.and_then(|frame| frame.texture) {
        if stage.chroma_bind_group.is_none() {
            stage.chroma_bind_group = Some(context.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some("Motolii B002 chroma bind group"),
                    layout: &stage.chroma_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &texture.as_ref().default_view,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&stage.chroma_sampler),
                        },
                    ],
                },
            ));
        }
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Motolii B002 chroma pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&stage.chroma_pipeline);
        pass.set_bind_group(
            0,
            stage.chroma_bind_group.as_ref().expect("created above"),
            &[],
        );
        pass.draw(0..3, 0..1);
    }
    context.before_submit();
    context
        .queue
        .submit(shape_commands.into_iter().chain([encoder.finish()]));
    Ok(())
}

fn encode_rerun_stage_shapes(
    context: &mut re_renderer::RenderContext,
    target: &wgpu::TextureView,
    width: u32,
    height: u32,
) -> Result<[wgpu::CommandBuffer; 2], String> {
    let mut lines = LineDrawableBuilder::new(context);
    lines
        .batch("Motolii rectangle")
        .add_rectangle_outline_2d(
            [width as f32 * 0.20, height as f32 * 0.24].into(),
            [width as f32 * 0.42, 0.0].into(),
            [0.0, height as f32 * 0.46].into(),
        )
        .radius(Size::new_ui_points(4.0))
        .color(Color32::from_rgb(82, 214, 255));
    let lines = lines
        .into_draw_data()
        .map_err(|error| format!("build Rerun rectangle: {error}"))?;

    let mut points = PointCloudBuilder::new(context);
    points.batch("Motolii circle").add_points_2d(
        &[([width as f32 * 0.58, height as f32 * 0.48, 0.0]).into()],
        &[Size::new_scene_units(height as f32 * 0.16)],
        &[Color32::from_rgba_premultiplied(255, 82, 139, 210)],
        &[re_renderer::PickingLayerInstanceId::default()],
    );
    let points = points
        .into_draw_data()
        .map_err(|error| format!("build Rerun circle: {error}"))?;

    let mut view = ViewBuilder::new(
        context,
        TargetConfiguration {
            name: "Motolii RN embedded Rerun Stage".into(),
            resolution_in_pixel: [width, height],
            projection_from_view: Projection::Orthographic {
                camera_mode: OrthographicCameraMode::TopLeftCornerAndExtendZ,
                vertical_world_size: height as f32,
                far_plane_distance: 1_000.0,
            },
            pixels_per_point: 1.0,
            ..Default::default()
        },
        re_renderer::ViewBuilderId::new(0),
    )
    .map_err(|error| format!("build embedded Rerun view: {error}"))?;
    view.queue_draw(context, lines).queue_draw(context, points);
    let draw = view
        .draw(context, re_renderer::Rgba::from_rgb(0.035, 0.041, 0.050))
        .map_err(|error| format!("draw embedded Rerun view: {error}"))?;

    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Motolii embedded Rerun composite"),
        });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Motolii embedded Rerun Stage pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
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
        view.composite(context, &mut pass);
    }
    Ok([draw, encoder.finish()])
}

#[cfg(test)]
mod chroma_tests {
    use super::{CHROMA_VIDEO_BYTES, encode_rerun_stage_shapes};

    const WIDTH: u32 = 320;
    const HEIGHT: u32 = 192;

    #[test]
    fn b002_fixture_seek_selects_the_same_sample_twice() {
        let video = re_video::VideoDataDescription::load_mp4(
            CHROMA_VIDEO_BYTES,
            "Motolii B002 chroma fixture",
        )
        .unwrap();
        assert_eq!(video.num_samples(), 60);
        let timescale = video.timescale.unwrap();
        let seek = re_video::Time::from_secs(1.25, timescale);
        let first = video
            .latest_sample_index_at_presentation_timestamp(seek)
            .unwrap();
        let second = video
            .latest_sample_index_at_presentation_timestamp(seek)
            .unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn rerun_stage_shapes_are_identical_across_two_output_targets() {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .expect("Rerun Stage parity probe requires a GPU adapter");
        let adapter_limits = adapter.limits();
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Motolii Rerun Stage parity probe"),
            required_features: wgpu::Features::empty(),
            required_limits: adapter_limits,
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
        }))
        .expect("create parity probe device");
        let mut context = re_renderer::RenderContext::new(
            &adapter,
            device.clone(),
            queue.clone(),
            wgpu::TextureFormat::Rgba8Unorm,
            re_renderer::RenderConfig::best_for_device_caps,
        )
        .expect("create Rerun renderer");

        let first = parity_target(&device, "Motolii Rerun Stage first output target");
        let second = parity_target(&device, "Motolii Rerun Stage second output target");
        for target in [&first, &second] {
            context.begin_frame();
            let view = target.create_view(&wgpu::TextureViewDescriptor::default());
            let commands = encode_rerun_stage_shapes(&mut context, &view, WIDTH, HEIGHT)
                .expect("encode shared Rerun Stage evaluation");
            context.before_submit();
            context.queue.submit(commands);
        }

        let first_rgba = read_back(&device, &queue, &first);
        let second_rgba = read_back(&device, &queue, &second);
        assert_eq!(first_rgba, second_rgba);
        assert!(
            first_rgba
                .chunks_exact(4)
                .any(|pixel| pixel[0] > 40 || pixel[1] > 40 || pixel[2] > 40),
            "the parity oracle must observe rendered shapes, not only the clear color"
        );
    }

    fn parity_target(device: &wgpu::Device, label: &'static str) -> wgpu::Texture {
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    }

    fn read_back(device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture) -> Vec<u8> {
        let unpadded = WIDTH * 4;
        let padded = unpadded.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Motolii Rerun Stage parity readback"),
            size: (padded * HEIGHT) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Motolii Rerun Stage parity copy"),
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
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
            result.expect("map parity readback")
        });
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .expect("wait for parity readback");
        let mapped = slice.get_mapped_range();
        let mut rgba = Vec::with_capacity((unpadded * HEIGHT) as usize);
        for row in 0..HEIGHT {
            let start = (row * padded) as usize;
            rgba.extend_from_slice(&mapped[start..start + unpadded as usize]);
        }
        rgba
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
