use std::{
    path::PathBuf,
    process::Command,
    sync::Arc,
    time::{Duration, Instant},
};

use g0_9_surface_host::{SurfaceLayout, LEFT_WEBVIEW_WIDTH, RIGHT_WEBVIEW_WIDTH};
use g0_9_windowed_timeline::{
    build_vello_overlay_asset, make_key_instances, rss_from_ps_output, source_digest,
    summarize_samples, EvidenceCompleteness, FaceDescriptor, FixtureFont, RawReport, RendererMode,
    RendererModeError, ReportConditions, ResourceCreationCounters, ResourceCreationPhases, Rss,
    ScenarioDefinition, ScenarioFrame, DEFAULT_MEASURE_FRAMES, DEFAULT_MEASURE_SECONDS,
    DEFAULT_WARMUP_FRAMES, KEYFRAME_COUNT,
};
use vello::{AaConfig, AaSupport, RenderParams, Renderer as VelloRenderer, RendererOptions};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};
use wry::{
    dpi::{LogicalPosition as WebPosition, LogicalSize as WebSize},
    Rect, WebView, WebViewBuilder,
};

const INITIAL_WIDTH: f64 = 1440.0;
const INITIAL_HEIGHT: f64 = 900.0;
const OVERLAY_WIDTH: u32 = 240;
const OVERLAY_HEIGHT: u32 = 128;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ViewUniform {
    viewport_pan_zoom: [f32; 4],
    track_origin: [f32; 4],
}

struct GfxState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    view_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    vello_renderer: VelloRenderer,
    vello_overlay: g0_9_windowed_timeline::VelloOverlayAsset,
    overlay_texture: wgpu::Texture,
    overlay_view: wgpu::TextureView,
    overlay_pipeline: wgpu::RenderPipeline,
    overlay_bind_group: wgpu::BindGroup,
    egui: Option<EguiState>,
    pixels_per_point: f32,
    adapter: String,
    backend: String,
    creations: ResourceCreationCounters,
    scripted_selected_key_index: Option<u32>,
}

struct EguiState {
    context: egui::Context,
    renderer: egui_wgpu::Renderer,
}

enum RenderOutcome {
    Presented(Duration, Instant),
    Reconfigure,
    Skip,
    Validation,
    VelloFailure,
}

impl GfxState {
    fn new(window: &Arc<Window>, renderer_mode: RendererMode) -> Self {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(Arc::clone(window))
            .expect("timeline surface");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("surface adapter");
        let adapter_info = adapter.get_info();
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("g0-9-windowed-timeline"),
            ..Default::default()
        }))
        .expect("surface device");

        let capabilities = surface.get_capabilities(&adapter);
        let size = window.inner_size();
        let format = capabilities.formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let mut creations = ResourceCreationCounters::default();
        let descriptor_text =
            std::env::var("G0_9_CJK_FACE").expect("G0_9_CJK_FACE exact face descriptor");
        let descriptor =
            FaceDescriptor::parse(&descriptor_text).expect("G0_9_CJK_FACE exact face descriptor");
        let fixture_font = FixtureFont::build(descriptor).expect("exact fixture font");
        let vello_overlay = build_vello_overlay_asset(&fixture_font).expect("exact Vello overlay");
        let vello_renderer = VelloRenderer::new(
            &device,
            RendererOptions {
                use_cpu: false,
                antialiasing_support: AaSupport::area_only(),
                num_init_threads: std::num::NonZeroUsize::new(1),
                pipeline_cache: None,
            },
        )
        .expect("Vello renderer");

        let overlay_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("timeline-vello-overlay-target"),
            size: wgpu::Extent3d {
                width: OVERLAY_WIDTH,
                height: OVERLAY_HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        creations.textures += 1;
        let overlay_view = overlay_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let overlay_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("timeline-vello-overlay-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let overlay_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("timeline-vello-overlay-bind-group-layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
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
        let overlay_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("timeline-vello-overlay-bind-group"),
            layout: &overlay_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&overlay_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&overlay_sampler),
                },
            ],
        });
        creations.bind_groups += 1;
        let overlay_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("timeline-vello-overlay-composite-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("overlay_composite.wgsl").into()),
        });
        let overlay_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("timeline-vello-overlay-pipeline-layout"),
                bind_group_layouts: &[Some(&overlay_bind_group_layout)],
                immediate_size: 0,
            });
        let overlay_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("timeline-vello-overlay-composite-pipeline"),
            layout: Some(&overlay_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &overlay_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &overlay_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
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
        creations.pipelines += 1;

        let keys = make_key_instances(KEYFRAME_COUNT);
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("timeline-100k-keys"),
            size: std::mem::size_of_val(keys.as_slice()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        creations.buffers += 1;
        queue.write_buffer(&instance_buffer, 0, bytemuck::cast_slice(&keys));

        let view_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("timeline-view-uniform"),
            size: std::mem::size_of::<ViewUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        creations.buffers += 1;

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("timeline-bind-group-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("timeline-bind-group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: view_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: instance_buffer.as_entire_binding(),
                },
            ],
        });
        creations.bind_groups += 1;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("timeline-100k-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("timeline.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("timeline-pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("timeline-100k-pipeline"),
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
                    format,
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
        creations.pipelines += 1;

        let egui = match renderer_mode {
            RendererMode::DirectVello => None,
            RendererMode::EguiVello => Some(EguiState {
                context: egui::Context::default(),
                renderer: egui_wgpu::Renderer::new(&device, format, Default::default()),
            }),
        };

        Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            bind_group,
            view_buffer,
            instance_buffer,
            vello_renderer,
            vello_overlay,
            overlay_texture,
            overlay_view,
            overlay_pipeline,
            overlay_bind_group,
            egui,
            pixels_per_point: window.scale_factor() as f32,
            adapter: adapter_info.name,
            backend: format!("{:?}", adapter_info.backend),
            creations,
            scripted_selected_key_index: None,
        }
    }

    fn configure(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    fn update_scale_factor(&mut self, scale_factor: f64) {
        self.pixels_per_point = scale_factor as f32;
    }

    fn render(&mut self, layout: SurfaceLayout, scenario: &ScenarioFrame) -> RenderOutcome {
        let started = Instant::now();
        debug_assert_eq!(self.overlay_texture.width(), OVERLAY_WIDTH);
        debug_assert_eq!(self.overlay_texture.height(), OVERLAY_HEIGHT);
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return RenderOutcome::Skip;
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                return RenderOutcome::Reconfigure;
            }
            wgpu::CurrentSurfaceTexture::Validation => return RenderOutcome::Validation,
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        if let Some(previous) = self
            .scripted_selected_key_index
            .replace(scenario.selected_key_index)
        {
            self.queue.write_buffer(
                &self.instance_buffer,
                u64::from(previous)
                    * std::mem::size_of::<g0_9_windowed_timeline::KeyInstance>() as u64,
                bytemuck::bytes_of(&scripted_key_instance(previous, previous % 10 == 0)),
            );
        }
        self.queue.write_buffer(
            &self.instance_buffer,
            u64::from(scenario.selected_key_index)
                * std::mem::size_of::<g0_9_windowed_timeline::KeyInstance>() as u64,
            bytemuck::bytes_of(&scripted_key_instance(scenario.selected_key_index, true)),
        );
        self.queue.write_buffer(
            &self.view_buffer,
            0,
            bytemuck::bytes_of(&ViewUniform {
                viewport_pan_zoom: [
                    self.config.width as f32,
                    self.config.height as f32,
                    scenario.pan_seconds as f32,
                    scenario.zoom_pixels_per_second as f32,
                ],
                track_origin: [
                    (self.config.height as f32 / 32.0).max(8.0),
                    layout.native_x,
                    0.0,
                    0.0,
                ],
            }),
        );
        let input_applied_at = Instant::now();

        if self
            .vello_renderer
            .render_to_texture(
                &self.device,
                &self.queue,
                self.vello_overlay.scene(),
                &self.overlay_view,
                &RenderParams {
                    base_color: vello::peniko::Color::TRANSPARENT,
                    width: OVERLAY_WIDTH,
                    height: OVERLAY_HEIGHT,
                    antialiasing_method: AaConfig::Area,
                },
            )
            .is_err()
        {
            return RenderOutcome::VelloFailure;
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("windowed-timeline-frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("windowed-timeline-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.035,
                            g: 0.038,
                            b: 0.047,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_viewport(
                layout.native_x,
                0.0,
                layout.native_width,
                self.config.height as f32,
                0.0,
                1.0,
            );
            pass.set_scissor_rect(
                layout.native_x.max(0.0) as u32,
                0,
                layout.native_width.max(1.0) as u32,
                self.config.height,
            );
            pass.draw(0..6, 0..KEYFRAME_COUNT as u32);
            pass.set_pipeline(&self.overlay_pipeline);
            pass.set_bind_group(0, &self.overlay_bind_group, &[]);
            pass.set_viewport(
                layout.native_x,
                0.0,
                OVERLAY_WIDTH as f32,
                OVERLAY_HEIGHT as f32,
                0.0,
                1.0,
            );
            pass.set_scissor_rect(
                layout.native_x.max(0.0) as u32,
                0,
                OVERLAY_WIDTH.min(layout.native_width.max(0.0) as u32),
                OVERLAY_HEIGHT.min(self.config.height),
            );
            pass.draw(0..6, 0..1);
        }
        let (egui_command_buffers, egui_texture_free) =
            self.render_egui(&mut encoder, &view, layout);
        self.queue
            .submit(egui_command_buffers.into_iter().chain([encoder.finish()]));
        if let Some(egui) = &mut self.egui {
            for texture_id in egui_texture_free {
                egui.renderer.free_texture(&texture_id);
            }
        }
        frame.present();
        RenderOutcome::Presented(started.elapsed(), input_applied_at)
    }

    fn render_egui(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        layout: SurfaceLayout,
    ) -> (Vec<wgpu::CommandBuffer>, Vec<egui::TextureId>) {
        let Some(egui) = &mut self.egui else {
            return (Vec::new(), Vec::new());
        };
        let pixels_per_point = self.pixels_per_point;
        let screen_rect = egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(
                self.config.width as f32 / pixels_per_point,
                self.config.height as f32 / pixels_per_point,
            ),
        );
        let raw_input = egui::RawInput {
            screen_rect: Some(screen_rect),
            ..Default::default()
        };
        let native_clip_rect = egui::Rect::from_min_size(
            egui::pos2(layout.native_x / pixels_per_point, 0.0),
            egui::vec2(
                layout.native_width / pixels_per_point,
                self.config.height as f32 / pixels_per_point,
            ),
        );
        let output = egui.context.run_ui(raw_input, |ui| {
            ui.painter()
                .rect_filled(native_clip_rect, 0.0, egui::Color32::TRANSPARENT);
        });
        let pixels_per_point = output.pixels_per_point;
        let paint_jobs = egui.context.tessellate(output.shapes, pixels_per_point);
        for (texture_id, image_delta) in &output.textures_delta.set {
            egui.renderer
                .update_texture(&self.device, &self.queue, *texture_id, image_delta);
        }
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point,
        };
        let command_buffers = egui.renderer.update_buffers(
            &self.device,
            &self.queue,
            encoder,
            &paint_jobs,
            &screen_descriptor,
        );
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("windowed-timeline-egui-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
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
            let mut render_pass = render_pass.forget_lifetime();
            egui.renderer
                .render(&mut render_pass, &paint_jobs, &screen_descriptor);
        }
        (command_buffers, output.textures_delta.free)
    }
}

fn scripted_key_instance(index: u32, selected: bool) -> g0_9_windowed_timeline::KeyInstance {
    g0_9_windowed_timeline::KeyInstance {
        time_seconds: (index % 10_000) as f32 * 0.01,
        track: (index % 32) as f32,
        selected: u32::from(selected),
        _padding: 0,
    }
}

struct State {
    window: Option<Arc<Window>>,
    left_webview: Option<WebView>,
    right_webview: Option<WebView>,
    gfx: Option<GfxState>,
    renderer_mode: RendererMode,
    layout: Option<SurfaceLayout>,
    warmup_target: u32,
    measured_target: u32,
    seconds_target: f64,
    warmup_frames: u32,
    measured_frames: u32,
    measurement_started: Option<Instant>,
    frame_samples_ms: Vec<f64>,
    present_interval_samples_ms: Vec<f64>,
    previous_present: Option<Instant>,
    input_samples_ms: Vec<f64>,
    acquire_count: u64,
    present_count: u64,
    skip_count: u64,
    reconfigure_count: u64,
    readback_count: u64,
    initialization_baseline: ResourceCreationCounters,
    warmup_resource_baseline: ResourceCreationCounters,
    measurement_resource_baseline: ResourceCreationCounters,
    report_path: PathBuf,
}

impl State {
    fn new() -> Self {
        Self {
            window: None,
            left_webview: None,
            right_webview: None,
            gfx: None,
            renderer_mode: renderer_mode_from_env(),
            layout: None,
            warmup_target: env_u32("G0_9_TIMELINE_WARMUP", DEFAULT_WARMUP_FRAMES),
            measured_target: env_u32("G0_9_TIMELINE_FRAMES", DEFAULT_MEASURE_FRAMES),
            seconds_target: env_f64("G0_9_TIMELINE_SECONDS", DEFAULT_MEASURE_SECONDS),
            warmup_frames: 0,
            measured_frames: 0,
            measurement_started: None,
            frame_samples_ms: Vec::new(),
            present_interval_samples_ms: Vec::new(),
            previous_present: None,
            input_samples_ms: Vec::new(),
            acquire_count: 0,
            present_count: 0,
            skip_count: 0,
            reconfigure_count: 0,
            readback_count: 0,
            initialization_baseline: ResourceCreationCounters::default(),
            warmup_resource_baseline: ResourceCreationCounters::default(),
            measurement_resource_baseline: ResourceCreationCounters::default(),
            report_path: std::env::var_os("G0_9_TIMELINE_REPORT")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/tmp/motolii-g0-9-windowed-timeline.json")),
        }
    }

    fn measured_seconds(&self) -> f64 {
        self.measurement_started
            .map(|started| started.elapsed().as_secs_f64())
            .unwrap_or(0.0)
    }

    fn update_layout(&mut self) {
        let Some(window) = &self.window else {
            return;
        };
        let size = window.inner_size();
        let Some(layout) = SurfaceLayout::try_new(size.width, size.height, window.scale_factor())
        else {
            self.layout = None;
            return;
        };
        if let Some(webview) = &self.left_webview {
            webview
                .set_bounds(Rect {
                    position: WebPosition::new(0.0, 0.0).into(),
                    size: WebSize::new(LEFT_WEBVIEW_WIDTH, layout.logical_height).into(),
                })
                .expect("left bounds");
        }
        if let Some(webview) = &self.right_webview {
            webview
                .set_bounds(Rect {
                    position: WebPosition::new(layout.logical_width - RIGHT_WEBVIEW_WIDTH, 0.0)
                        .into(),
                    size: WebSize::new(RIGHT_WEBVIEW_WIDTH, layout.logical_height).into(),
                })
                .expect("right bounds");
        }
        self.layout = Some(layout);
    }

    fn publish_report(&self) {
        let Some(gfx) = &self.gfx else {
            return;
        };
        let Some(frame_timing) = summarize_samples(&self.frame_samples_ms) else {
            return;
        };
        let Some(present_timing) = summarize_samples(&self.present_interval_samples_ms) else {
            return;
        };
        let Some(input_timing) = summarize_samples(&self.input_samples_ms) else {
            return;
        };
        let rss = collect_rss();
        let complete = self.measured_frames >= self.measured_target
            && self.measured_seconds() >= self.seconds_target
            && self.acquire_count != 0
            && self.acquire_count == self.present_count
            && self.readback_count == 0
            && self.input_samples_ms.len() == self.measured_frames as usize
            && matches!(rss, Rss::Available { .. });
        let report = RawReport {
            renderer: self.renderer_mode,
            scenario_digest: ScenarioDefinition::fixed()
                .digests()
                .expect("fixed scenario")
                .scenario_sha256,
            input_digest: ScenarioDefinition::fixed()
                .digests()
                .expect("fixed scenario")
                .input_sequence_sha256,
            source_digest: source_digest(),
            font_digest: gfx.vello_overlay.metadata.font_sha256.clone(),
            glyph_digest: gfx.vello_overlay.metadata.glyph_digest.clone(),
            conditions: ReportConditions {
                device: format!("{}|{}", gfx.adapter, gfx.backend),
                surface: format!("{:?}|fifo|1", gfx.config.format),
                window: format!(
                    "{}x{}@{}",
                    gfx.config.width, gfx.config.height, gfx.pixels_per_point
                ),
                webview: "2-opaque-offline-child".to_owned(),
                fixture: "g0-9-windowed-timeline.v1|1000-clips|100000-keys".to_owned(),
                target: format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
            },
            measured_frames: self.measured_frames,
            measured_seconds: self.measured_seconds(),
            acquire_count: self.acquire_count,
            present_count: self.present_count,
            skip_count: self.skip_count,
            reconfigure_count: self.reconfigure_count,
            readback_count: self.readback_count,
            frame_timing,
            present_timing,
            input_timing,
            rss,
            resource_creations: ResourceCreationPhases {
                initialization: self.initialization_baseline,
                warmup: self
                    .warmup_resource_baseline
                    .delta(self.initialization_baseline),
                measured: gfx.creations.delta(self.measurement_resource_baseline),
            },
            completeness: if complete {
                EvidenceCompleteness::Complete
            } else {
                EvidenceCompleteness::Incomplete {
                    reason:
                        "windowed measurement did not reach the required complete evidence state"
                            .to_owned(),
                }
            },
        };
        report.validate().expect("strict raw report");
        std::fs::write(
            &self.report_path,
            serde_json::to_vec_pretty(&report).expect("serialize raw report"),
        )
        .expect("write raw report");
    }
}

impl ApplicationHandler for State {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("G0-9 windowed 100k Timeline")
                        .with_inner_size(LogicalSize::new(INITIAL_WIDTH, INITIAL_HEIGHT)),
                )
                .expect("timeline window"),
        );
        let gfx = GfxState::new(&window, self.renderer_mode);
        let layout = SurfaceLayout::try_new(
            window.inner_size().width,
            window.inner_size().height,
            window.scale_factor(),
        )
        .expect("initial layout");
        let left_webview = make_webview(
            &window,
            Rect {
                position: WebPosition::new(0.0, 0.0).into(),
                size: WebSize::new(LEFT_WEBVIEW_WIDTH, layout.logical_height).into(),
            },
            LEFT_HTML,
        );
        let right_webview = make_webview(
            &window,
            Rect {
                position: WebPosition::new(layout.logical_width - RIGHT_WEBVIEW_WIDTH, 0.0).into(),
                size: WebSize::new(RIGHT_WEBVIEW_WIDTH, layout.logical_height).into(),
            },
            RIGHT_HTML,
        );
        self.initialization_baseline = gfx.creations;
        self.warmup_resource_baseline = gfx.creations;
        self.measurement_resource_baseline = gfx.creations;
        self.window = Some(window);
        self.left_webview = Some(left_webview);
        self.right_webview = Some(right_webview);
        self.gfx = Some(gfx);
        self.layout = Some(layout);
        self.window.as_ref().unwrap().request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(size) => {
                if let Some(gfx) = &mut self.gfx {
                    gfx.configure(size.width, size.height);
                }
                self.update_layout();
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                if let (Some(gfx), Some(window)) = (&mut self.gfx, &self.window) {
                    gfx.update_scale_factor(window.scale_factor());
                }
                self.update_layout();
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                let Some(layout) = self.layout else {
                    return;
                };
                let frame_index = u64::from(self.warmup_frames) + u64::from(self.measured_frames);
                let scenario = ScenarioDefinition::fixed()
                    .at(frame_index)
                    .expect("fixed scenario frame");
                match self.gfx.as_mut().unwrap().render(layout, &scenario) {
                    RenderOutcome::Presented(elapsed, input_applied_at) => {
                        self.acquire_count += 1;
                        self.present_count += 1;
                        if self.warmup_frames < self.warmup_target {
                            self.warmup_frames += 1;
                            self.warmup_resource_baseline = self.gfx.as_ref().unwrap().creations;
                            if self.warmup_frames == self.warmup_target {
                                self.measurement_resource_baseline =
                                    self.gfx.as_ref().unwrap().creations;
                            }
                        } else {
                            let presented_at = Instant::now();
                            self.measurement_started.get_or_insert(presented_at);
                            if let Some(previous) = self.previous_present.replace(presented_at) {
                                self.present_interval_samples_ms.push(
                                    presented_at.duration_since(previous).as_secs_f64() * 1000.0,
                                );
                            }
                            self.input_samples_ms.push(
                                presented_at.duration_since(input_applied_at).as_secs_f64()
                                    * 1000.0,
                            );
                            self.measured_frames += 1;
                            self.frame_samples_ms.push(elapsed.as_secs_f64() * 1000.0);
                        }
                    }
                    RenderOutcome::Reconfigure => {
                        self.reconfigure_count += 1;
                        if let Some(window) = &self.window {
                            let size = window.inner_size();
                            self.gfx
                                .as_mut()
                                .unwrap()
                                .configure(size.width, size.height);
                        }
                    }
                    RenderOutcome::Skip => {
                        self.skip_count += 1;
                    }
                    RenderOutcome::Validation => {
                        self.publish_report();
                        event_loop.exit();
                        return;
                    }
                    RenderOutcome::VelloFailure => {
                        self.publish_report();
                        event_loop.exit();
                        return;
                    }
                }
                let done = self.measured_frames >= self.measured_target
                    && self.measured_seconds() >= self.seconds_target;
                if done {
                    self.publish_report();
                    event_loop.exit();
                } else if let Some(window) = &self.window {
                    window.set_title(&format!(
                        "G0-9 Timeline | warmup {}/{} | measured {} | {:.1}/{:.1}s",
                        self.warmup_frames,
                        self.warmup_target,
                        self.measured_frames,
                        self.measured_seconds(),
                        self.seconds_target,
                    ));
                    window.request_redraw();
                }
            }
            WindowEvent::CloseRequested => {
                self.publish_report();
                event_loop.exit();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Wait);
    }
}

fn make_webview(window: &Window, bounds: Rect, html: &'static str) -> WebView {
    WebViewBuilder::new()
        .with_bounds(bounds)
        .with_accept_first_mouse(true)
        .with_html(html)
        .build_as_child(window)
        .expect("opaque child webview")
}

fn collect_rss() -> Rss {
    let pid = std::process::id().to_string();
    match Command::new("/bin/ps")
        .args(["-o", "rss=", "-p", &pid])
        .output()
    {
        Ok(output) if output.status.success() => match String::from_utf8(output.stdout) {
            Ok(stdout) => match rss_from_ps_output(&stdout) {
                Ok(rss) => rss,
                Err(error) => Rss::Unavailable {
                    reason: format!("/bin/ps rss output rejected: {error}"),
                },
            },
            Err(error) => Rss::Unavailable {
                reason: format!("/bin/ps rss output was not UTF-8: {error}"),
            },
        },
        Ok(output) => Rss::Unavailable {
            reason: format!("/bin/ps exited with {status}", status = output.status),
        },
        Err(error) => Rss::Unavailable {
            reason: format!("/bin/ps could not run: {error}"),
        },
    }
}

fn env_u32(name: &str, default: u32) -> u32 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn env_f64(name: &str, default: f64) -> f64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|value: &f64| value.is_finite() && *value >= 0.0)
        .unwrap_or(default)
}

fn renderer_mode_from_env() -> RendererMode {
    match std::env::var("G0_9_RENDERER_MODE") {
        Ok(value) => parse_renderer_mode(Some(&value))
            .expect("G0_9_RENDERER_MODE must be direct_vello or egui_vello"),
        Err(std::env::VarError::NotPresent) => {
            parse_renderer_mode(None).expect("the default renderer mode must be valid")
        }
        Err(std::env::VarError::NotUnicode(_)) => {
            panic!("G0_9_RENDERER_MODE must be valid Unicode")
        }
    }
}

fn parse_renderer_mode(value: Option<&str>) -> Result<RendererMode, RendererModeError> {
    value.unwrap_or("direct_vello").parse()
}

const LEFT_HTML: &str = r#"<!doctype html><html><head><meta charset="utf-8"><style>
html,body{margin:0;height:100%;background:#22252b;color:#eef1f5;font:14px -apple-system,sans-serif}
main{padding:18px}input,button{box-sizing:border-box;width:100%;font:inherit;margin:6px 0;padding:7px}
</style></head><body><main><h2>Browser</h2><input aria-label="Search assets" value="100k fixture"><button>Rectangle</button><p>Opaque child WebView</p></main></body></html>"#;

const RIGHT_HTML: &str = r#"<!doctype html><html><head><meta charset="utf-8"><style>
html,body{margin:0;height:100%;background:#272a31;color:#eef1f5;font:14px -apple-system,sans-serif}
main{padding:18px}input{box-sizing:border-box;width:100%;font:inherit;margin:6px 0;padding:7px}
</style></head><body><main><h2>Inspector</h2><label>Position<input aria-label="Position" value="0, 0"></label><p>Offline HTML / opaque</p></main></body></html>"#;

fn main() {
    let event_loop = EventLoop::new().expect("event loop");
    let mut state = State::new();
    event_loop.run_app(&mut state).expect("windowed timeline");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renderer_mode_parser_accepts_only_canonical_modes() {
        assert_eq!(parse_renderer_mode(None), Ok(RendererMode::DirectVello));
        assert_eq!(
            parse_renderer_mode(Some("direct_vello")),
            Ok(RendererMode::DirectVello)
        );
        assert_eq!(
            parse_renderer_mode(Some("egui_vello")),
            Ok(RendererMode::EguiVello)
        );
    }

    #[test]
    fn renderer_mode_parser_rejects_empty_unknown_and_alias_modes() {
        for mode in ["", "direct", "egui", "vello", "DIRECT_VELLO"] {
            assert!(parse_renderer_mode(Some(mode)).is_err(), "{mode}");
        }
    }

    #[test]
    fn source_preserves_mode_gated_egui_lifecycle_and_submission_order() {
        let implementation = include_str!("main.rs")
            .split("\n#[cfg(test)]")
            .next()
            .expect("implementation source");
        assert!(implementation.contains("RendererMode::DirectVello => None"));
        assert!(implementation.contains("RendererMode::EguiVello => Some(EguiState"));
        assert!(implementation.contains("egui.context.run_ui(raw_input, |ui|"));
        assert!(!implementation.contains("egui.context.run(raw_input"));
        assert!(implementation.contains("let pixels_per_point = output.pixels_per_point;"));
        assert!(implementation
            .contains(".submit(egui_command_buffers.into_iter().chain([encoder.finish()]));"));
    }

    #[test]
    fn source_preserves_egui_texture_lifecycle_order() {
        let implementation = include_str!("main.rs")
            .split("\n#[cfg(test)]")
            .next()
            .expect("implementation source");
        let render_egui = implementation
            .split("fn render_egui(")
            .nth(1)
            .and_then(|tail| tail.split("\n}\n\nfn scripted_key_instance").next())
            .expect("egui render helper source section");
        let frame_caller = implementation
            .split("fn render(&mut self")
            .nth(1)
            .and_then(|tail| tail.split("\n    fn render_egui(").next())
            .expect("frame caller source section");
        let set = render_egui
            .find("output.textures_delta.set")
            .expect("texture set source");
        let update_texture = render_egui
            .find(".update_texture(")
            .expect("texture update source");
        let update_buffers = render_egui
            .find(".update_buffers(")
            .expect("buffer update source");
        let render = render_egui.find(".render(").expect("egui render source");
        let free_return = render_egui
            .find("output.textures_delta.free")
            .expect("texture free return source");
        let submit = frame_caller
            .find(".submit(egui_command_buffers.into_iter().chain([encoder.finish()]));")
            .expect("frame submission source");
        let free_texture = frame_caller
            .find(".free_texture(")
            .expect("texture release source");

        assert!(
            render_egui.contains("(command_buffers, output.textures_delta.free)"),
            "egui helper must return the current frame free IDs to its caller",
        );
        assert!(
            !render_egui.contains(".free_texture("),
            "egui helper must not release textures before frame submission",
        );
        assert!(
            set < update_texture
                && update_texture < update_buffers
                && update_buffers < render
                && render < free_return,
            "egui helper must return frees after set → update_texture → update_buffers → render",
        );
        assert!(
            submit < free_texture,
            "egui texture lifecycle must free_texture only after the current frame is submitted",
        );
    }

    #[test]
    fn source_keeps_the_100k_vello_workload_outside_the_mode_match() {
        let implementation = include_str!("main.rs")
            .split("\n#[cfg(test)]")
            .next()
            .expect("implementation source");
        let key_upload = implementation
            .find("make_key_instances(KEYFRAME_COUNT)")
            .expect("100k key upload source");
        let mode_match = implementation
            .find("let egui = match renderer_mode")
            .expect("renderer mode match source");
        let render = implementation
            .find("fn render(&mut self")
            .expect("shared render source");

        assert_eq!(
            implementation
                .matches("make_key_instances(KEYFRAME_COUNT)")
                .count(),
            1,
            "100k key upload must have one source occurrence",
        );
        assert_eq!(
            implementation
                .matches("pass.draw(0..6, 0..KEYFRAME_COUNT as u32);")
                .count(),
            1,
            "100k draw must have one source occurrence",
        );
        assert_eq!(
            implementation.matches(".render_to_texture(").count(),
            1,
            "Vello workload must have one source occurrence",
        );
        assert!(
            key_upload < mode_match && mode_match < render,
            "the renderer mode match must follow common 100k setup and precede the shared render path",
        );
    }

    #[test]
    fn render_hot_loop_has_no_spike_owned_resource_creation_or_readback() {
        let implementation = include_str!("main.rs")
            .split("\n#[cfg(test)]")
            .next()
            .expect("implementation source");
        let render = implementation
            .split("fn render(&mut self")
            .nth(1)
            .and_then(|tail| tail.split("\n    fn render_egui(").next())
            .expect("render source section");
        for forbidden in [
            "create_buffer(",
            "create_bind_group(",
            "create_render_pipeline(",
            "create_texture(",
            "copy_texture",
            "map_async",
            "PollType::wait",
        ] {
            assert!(
                !render.contains(forbidden),
                "render hot loop contains forbidden call: {forbidden}",
            );
        }
    }
}
