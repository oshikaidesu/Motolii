use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use g0_9_surface_host::{SurfaceLayout, LEFT_WEBVIEW_WIDTH, RIGHT_WEBVIEW_WIDTH};
use g0_9_windowed_timeline::{
    acceptance_passes, build_vello_overlay_asset, make_key_instances, summarize_samples,
    AcceptanceInput, FaceDescriptor, FixtureFont, ResourceCreationCounters, DEFAULT_MEASURE_FRAMES,
    DEFAULT_MEASURE_SECONDS, DEFAULT_WARMUP_FRAMES, KEYFRAME_COUNT,
};
use serde::Serialize;
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
    vello_renderer: VelloRenderer,
    vello_overlay: g0_9_windowed_timeline::VelloOverlayAsset,
    overlay_texture: wgpu::Texture,
    overlay_view: wgpu::TextureView,
    overlay_pipeline: wgpu::RenderPipeline,
    overlay_bind_group: wgpu::BindGroup,
    adapter: String,
    backend: String,
    creations: ResourceCreationCounters,
}

enum RenderOutcome {
    Presented(Duration),
    Reconfigure,
    Skip,
    Validation,
    VelloFailure,
}

impl GfxState {
    fn new(window: &Arc<Window>) -> Self {
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

        Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            bind_group,
            view_buffer,
            vello_renderer,
            vello_overlay,
            overlay_texture,
            overlay_view,
            overlay_pipeline,
            overlay_bind_group,
            adapter: adapter_info.name,
            backend: format!("{:?}", adapter_info.backend),
            creations,
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

    fn render(&mut self, layout: SurfaceLayout, frame_index: u64) -> RenderOutcome {
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
        let phase = frame_index as f32 * 0.0125;
        let pixels_per_second = 18.0 + phase.sin().abs() * 72.0;
        let visible_seconds = layout.native_width / pixels_per_second;
        let pan_seconds =
            (phase * 0.37).sin().mul_add(0.5, 0.5) * (100.0 - visible_seconds).max(0.0);
        self.queue.write_buffer(
            &self.view_buffer,
            0,
            bytemuck::bytes_of(&ViewUniform {
                viewport_pan_zoom: [
                    self.config.width as f32,
                    self.config.height as f32,
                    pan_seconds,
                    pixels_per_second,
                ],
                track_origin: [
                    (self.config.height as f32 / 32.0).max(8.0),
                    layout.native_x,
                    0.0,
                    0.0,
                ],
            }),
        );

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
        self.queue.submit([encoder.finish()]);
        frame.present();
        RenderOutcome::Presented(started.elapsed())
    }
}

#[derive(Serialize)]
struct Report<'a> {
    ticket: &'static str,
    status: &'static str,
    adapter: &'a str,
    backend: &'a str,
    surface_count: u32,
    native_viewport_count: u32,
    webview_count: u32,
    keyframes: usize,
    selected_keyframes: usize,
    warmup_frames: u32,
    measured_frames: u32,
    measured_seconds: f64,
    target_frames: u32,
    target_seconds: f64,
    acquire_count: u64,
    present_count: u64,
    surface_texture_view_count: u64,
    readback_count: u64,
    initialization_resource_creations: ResourceCreationCounters,
    frame_resource_creations: ResourceCreationCounters,
    median_frame_ms: f64,
    p95_frame_ms: f64,
    max_frame_ms: f64,
    median_present_interval_ms: f64,
    p95_present_interval_ms: f64,
    max_present_interval_ms: f64,
    throughput_fps: f64,
    deadline_miss_16_667_count: usize,
    pass: bool,
    measurement: &'static str,
    limitations: [&'static str; 3],
}

struct State {
    window: Option<Arc<Window>>,
    left_webview: Option<WebView>,
    right_webview: Option<WebView>,
    gfx: Option<GfxState>,
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
    acquire_count: u64,
    present_count: u64,
    readback_count: u64,
    initialization_baseline: ResourceCreationCounters,
    report_path: PathBuf,
}

impl State {
    fn new() -> Self {
        Self {
            window: None,
            left_webview: None,
            right_webview: None,
            gfx: None,
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
            acquire_count: 0,
            present_count: 0,
            readback_count: 0,
            initialization_baseline: ResourceCreationCounters::default(),
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

    fn publish_report(&self, status: &'static str) {
        let Some(gfx) = &self.gfx else {
            return;
        };
        let (median_frame_ms, p95_frame_ms, max_frame_ms) =
            summarize_samples(&self.frame_samples_ms)
                .map(|summary| {
                    (
                        summary.median_frame_ms,
                        summary.p95_frame_ms,
                        summary.max_frame_ms,
                    )
                })
                .unwrap_or((0.0, 0.0, 0.0));
        let (median_present_interval_ms, p95_present_interval_ms, max_present_interval_ms) =
            summarize_samples(&self.present_interval_samples_ms)
                .map(|summary| {
                    (
                        summary.median_frame_ms,
                        summary.p95_frame_ms,
                        summary.max_frame_ms,
                    )
                })
                .unwrap_or((0.0, 0.0, 0.0));
        let measured_seconds = self.measured_seconds();
        let frame_creations = gfx.creations.delta(self.initialization_baseline);
        let pass = acceptance_passes(AcceptanceInput {
            measured_frames: self.measured_frames,
            target_frames: self.measured_target,
            measured_seconds,
            target_seconds: self.seconds_target,
            acquire_count: self.acquire_count,
            present_count: self.present_count,
            readback_count: self.readback_count,
            frame_creations,
        });
        let report = Report {
            ticket: "G0-9-windowed-timeline",
            status,
            adapter: &gfx.adapter,
            backend: &gfx.backend,
            surface_count: 1,
            native_viewport_count: 1,
            webview_count: 2,
            keyframes: KEYFRAME_COUNT,
            selected_keyframes: 10_000,
            warmup_frames: self.warmup_frames,
            measured_frames: self.measured_frames,
            measured_seconds,
            target_frames: self.measured_target,
            target_seconds: self.seconds_target,
            acquire_count: self.acquire_count,
            present_count: self.present_count,
            surface_texture_view_count: self.acquire_count,
            readback_count: self.readback_count,
            initialization_resource_creations: self.initialization_baseline,
            frame_resource_creations: frame_creations,
            median_frame_ms,
            p95_frame_ms,
            max_frame_ms,
            median_present_interval_ms,
            p95_present_interval_ms,
            max_present_interval_ms,
            throughput_fps: if measured_seconds > 0.0 {
                f64::from(self.measured_frames) / measured_seconds
            } else {
                0.0
            },
            deadline_miss_16_667_count: self
                .present_interval_samples_ms
                .iter()
                .filter(|sample| **sample > 16.667)
                .count(),
            pass,
            measurement: "windowed wgpu Surface/Fifo acquire-to-present CPU wall time; 100,000 direct instances plus one fixed Vello text/path overlay every frame on the same device/queue; no device.poll wait and no readback",
            limitations: [
                "the fixed Vello overlay is not a typed renderer comparison or an egui branch",
                "GPU timestamp queries and input latency are not included",
                "macOS WKWebView child composition only; Windows WebView2 is untested",
            ],
        };
        std::fs::write(
            &self.report_path,
            serde_json::to_vec_pretty(&report).expect("serialize report"),
        )
        .expect("write report");
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
        let gfx = GfxState::new(&window);
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
                match self.gfx.as_mut().unwrap().render(layout, frame_index) {
                    RenderOutcome::Presented(elapsed) => {
                        self.acquire_count += 1;
                        self.present_count += 1;
                        if self.warmup_frames < self.warmup_target {
                            self.warmup_frames += 1;
                        } else {
                            let presented_at = Instant::now();
                            self.measurement_started.get_or_insert(presented_at);
                            if let Some(previous) = self.previous_present.replace(presented_at) {
                                self.present_interval_samples_ms.push(
                                    presented_at.duration_since(previous).as_secs_f64() * 1000.0,
                                );
                            }
                            self.measured_frames += 1;
                            self.frame_samples_ms.push(elapsed.as_secs_f64() * 1000.0);
                        }
                    }
                    RenderOutcome::Reconfigure => {
                        if let Some(window) = &self.window {
                            let size = window.inner_size();
                            self.gfx
                                .as_mut()
                                .unwrap()
                                .configure(size.width, size.height);
                        }
                    }
                    RenderOutcome::Skip => {}
                    RenderOutcome::Validation => {
                        self.publish_report("validation-error");
                        event_loop.exit();
                        return;
                    }
                    RenderOutcome::VelloFailure => {
                        self.publish_report("vello-render-error");
                        event_loop.exit();
                        return;
                    }
                }
                let done = self.measured_frames >= self.measured_target
                    && self.measured_seconds() >= self.seconds_target;
                if done {
                    self.publish_report("complete");
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
                self.publish_report("closed-before-complete");
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
