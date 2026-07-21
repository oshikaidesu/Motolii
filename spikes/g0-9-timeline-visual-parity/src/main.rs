use std::{path::PathBuf, sync::Arc};

use g0_9_timeline_visual_parity::{
    build_scene, RectPrimitive, TextPrimitive, VisualParityReport, AUTO_PRESENT_TARGET,
    FIXTURE_HEIGHT, FIXTURE_WIDTH, OBJECTS,
};
use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ScreenUniform {
    size: [f32; 2],
    _padding: [f32; 2],
}

struct PreparedText {
    buffer: Buffer,
    left: f32,
    top: f32,
    bounds: TextBounds,
    color: Color,
}

struct GfxState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    screen_buffer: wgpu::Buffer,
    rect_buffer: wgpu::Buffer,
    rect_count: u32,
    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: Viewport,
    atlas: TextAtlas,
    text_renderer: TextRenderer,
    texts: Vec<PreparedText>,
    adapter: String,
    backend: String,
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
            label: Some("g0-9-timeline-visual-parity"),
            ..Default::default()
        }))
        .expect("surface device");
        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(capabilities.formats[0]);
        let size = window.inner_size();
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

        let screen_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("timeline-screen-uniform"),
            size: std::mem::size_of::<ScreenUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let rect_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("timeline-visual-primitives"),
            size: (std::mem::size_of::<RectPrimitive>() * 512) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("timeline-visual-bind-group-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("timeline-visual-bind-group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_buffer.as_entire_binding(),
            }],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("timeline-visual-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("timeline.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("timeline-visual-pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("timeline-visual-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<RectPrimitive>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32,
                            offset: 32,
                            shader_location: 2,
                        },
                    ],
                }],
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

        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let viewport = Viewport::new(&device, &cache);
        let mut atlas = TextAtlas::new(&device, &queue, &cache, format);
        let text_renderer =
            TextRenderer::new(&mut atlas, &device, wgpu::MultisampleState::default(), None);

        let mut state = Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            bind_group,
            screen_buffer,
            rect_buffer,
            rect_count: 0,
            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,
            texts: Vec::new(),
            adapter: adapter_info.name,
            backend: format!("{:?}", adapter_info.backend),
        };
        state.update_scene();
        state
    }

    fn configure(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.update_scene();
    }

    fn update_scene(&mut self) {
        let scale_x = self.config.width as f32 / FIXTURE_WIDTH;
        let scale_y = self.config.height as f32 / FIXTURE_HEIGHT;
        let scene = build_scene(FIXTURE_WIDTH, FIXTURE_HEIGHT);
        let scaled_rects = scene
            .rects
            .into_iter()
            .map(|mut primitive| {
                primitive.rect[0] *= scale_x;
                primitive.rect[1] *= scale_y;
                primitive.rect[2] *= scale_x;
                primitive.rect[3] *= scale_y;
                primitive
            })
            .collect::<Vec<_>>();
        assert!(
            scaled_rects.len() <= 512,
            "fixture exceeded static primitive capacity"
        );
        self.rect_count = scaled_rects.len() as u32;
        self.queue
            .write_buffer(&self.rect_buffer, 0, bytemuck::cast_slice(&scaled_rects));
        self.queue.write_buffer(
            &self.screen_buffer,
            0,
            bytemuck::bytes_of(&ScreenUniform {
                size: [self.config.width as f32, self.config.height as f32],
                _padding: [0.0; 2],
            }),
        );
        self.viewport.update(
            &self.queue,
            Resolution {
                width: self.config.width,
                height: self.config.height,
            },
        );
        self.texts = scene
            .texts
            .into_iter()
            .map(|text| {
                prepare_text(
                    &mut self.font_system,
                    text,
                    scale_x,
                    scale_y,
                    self.config.width,
                    self.config.height,
                )
            })
            .collect();
        let areas = self.texts.iter().map(|text| TextArea {
            buffer: &text.buffer,
            left: text.left,
            top: text.top,
            scale: 1.0,
            bounds: text.bounds,
            default_color: text.color,
            custom_glyphs: &[],
        });
        self.text_renderer
            .prepare(
                &self.device,
                &self.queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                areas,
                &mut self.swash_cache,
            )
            .expect("prepare timeline text");
    }

    fn render(&mut self) -> bool {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return false
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return false;
            }
            wgpu::CurrentSurfaceTexture::Validation => return false,
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("timeline-visual-frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("timeline-visual-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.018,
                            g: 0.020,
                            b: 0.022,
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
            pass.set_vertex_buffer(0, self.rect_buffer.slice(..));
            pass.draw(0..6, 0..self.rect_count);
            self.text_renderer
                .render(&self.atlas, &self.viewport, &mut pass)
                .expect("render timeline text");
        }
        self.queue.submit([encoder.finish()]);
        frame.present();
        true
    }
}

fn prepare_text(
    font_system: &mut FontSystem,
    text: TextPrimitive,
    scale_x: f32,
    scale_y: f32,
    surface_width: u32,
    surface_height: u32,
) -> PreparedText {
    let font_size = text.size * scale_y;
    let mut buffer = Buffer::new(font_system, Metrics::new(font_size, font_size * 1.25));
    buffer.set_size(
        font_system,
        Some(text.width * scale_x),
        Some(text.height * scale_y),
    );
    let family = if text.monospace {
        Family::Monospace
    } else {
        Family::SansSerif
    };
    buffer.set_text(
        font_system,
        &text.text,
        &Attrs::new().family(family),
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(font_system, false);
    let left = text.left * scale_x;
    let top = text.top * scale_y;
    PreparedText {
        buffer,
        left,
        top,
        bounds: TextBounds {
            left: left.floor() as i32,
            top: top.floor() as i32,
            right: ((text.left + text.width) * scale_x)
                .ceil()
                .min(surface_width as f32) as i32,
            bottom: ((text.top + text.height) * scale_y)
                .ceil()
                .min(surface_height as f32) as i32,
        },
        color: Color::rgba(text.color[0], text.color[1], text.color[2], text.color[3]),
    }
}

struct App {
    auto: bool,
    report_path: PathBuf,
    window: Option<Arc<Window>>,
    gfx: Option<GfxState>,
    present_count: u32,
}

impl App {
    fn new(auto: bool) -> Self {
        Self {
            auto,
            report_path: std::env::var_os("G0_9_TIMELINE_VISUAL_REPORT")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    PathBuf::from("/tmp/motolii-g0-9-timeline-visual-parity-report.json")
                }),
            window: None,
            gfx: None,
            present_count: 0,
        }
    }

    fn write_report(&self) {
        let gfx = self.gfx.as_ref().expect("gfx state");
        let scene = build_scene(FIXTURE_WIDTH, FIXTURE_HEIGHT);
        let report = VisualParityReport {
            ticket: "G0-9-timeline-visual-parity",
            status: "complete",
            adapter: gfx.adapter.clone(),
            backend: gfx.backend.clone(),
            object_count: OBJECTS.len(),
            rect_primitive_count: scene.rects.len(),
            text_run_count: scene.texts.len(),
            present_count: self.present_count,
            readback_count: 0,
            semantic_state_owner_count: 1,
            pass: self.present_count >= AUTO_PRESENT_TARGET,
        };
        std::fs::write(
            &self.report_path,
            serde_json::to_vec_pretty(&report).expect("serialize report"),
        )
        .expect("write report");
        println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("print report")
        );
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Motolii native Timeline visual parity")
                        .with_inner_size(LogicalSize::new(
                            FIXTURE_WIDTH as f64,
                            FIXTURE_HEIGHT as f64,
                        )),
                )
                .expect("timeline window"),
        );
        self.gfx = Some(GfxState::new(&window));
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        if window.id() != window_id {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(gfx) = self.gfx.as_mut() {
                    gfx.configure(size.width, size.height);
                }
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                if self.gfx.as_mut().is_some_and(GfxState::render) {
                    self.present_count += 1;
                }
                if self.auto && self.present_count >= AUTO_PRESENT_TARGET {
                    self.write_report();
                    event_loop.exit();
                } else {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

fn main() {
    let auto = std::env::args().any(|arg| arg == "--auto");
    let event_loop = EventLoop::new().expect("event loop");
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop
        .run_app(&mut App::new(auto))
        .expect("run visual parity spike");
}
