use std::{path::PathBuf, sync::Arc};

use g0_9_graph_view::{
    build_scene, hit_target, plot_contains, GraphSession, Primitive, Report, TextPrimitive, HEIGHT,
    WIDTH,
};
use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
    window::{Window, WindowId},
};

const AUTO_PRESENT_TARGET: u32 = 120;
const PRIMITIVE_CAPACITY: usize = 1_024;

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
    primitive_buffer: wgpu::Buffer,
    primitive_count: u32,
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
    fn new(window: &Arc<Window>, session: &GraphSession) -> Self {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(Arc::clone(window))
            .expect("graph view surface");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("graph view adapter");
        let adapter_info = adapter.get_info();
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("g0-9-graph-view"),
            ..Default::default()
        }))
        .expect("graph view device");
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
            label: Some("graph-view-screen"),
            size: std::mem::size_of::<ScreenUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let primitive_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("graph-view-primitives"),
            size: (std::mem::size_of::<Primitive>() * PRIMITIVE_CAPACITY) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("graph-view-bind-layout"),
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
            label: Some("graph-view-bind-group"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_buffer.as_entire_binding(),
            }],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("graph-view-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("graph.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("graph-view-pipeline-layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("graph-view-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Primitive>() as u64,
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
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 32,
                            shader_location: 2,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32,
                            offset: 48,
                            shader_location: 3,
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
            primitive_buffer,
            primitive_count: 0,
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
            viewport,
            atlas,
            text_renderer,
            texts: Vec::new(),
            adapter: adapter_info.name,
            backend: format!("{:?}", adapter_info.backend),
        };
        state.update_scene(session);
        state
    }

    fn configure(&mut self, width: u32, height: u32, session: &GraphSession) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.update_scene(session);
    }

    fn update_scene(&mut self, session: &GraphSession) {
        let scale_x = self.config.width as f32 / WIDTH;
        let scale_y = self.config.height as f32 / HEIGHT;
        let scene = build_scene(session);
        let scaled = scene
            .primitives
            .into_iter()
            .map(|mut primitive| {
                primitive.bounds[0] *= scale_x;
                primitive.bounds[1] *= scale_y;
                primitive.bounds[2] *= scale_x;
                primitive.bounds[3] *= scale_y;
                if primitive.shape == 3 {
                    primitive.extra[0] *= scale_x;
                    primitive.extra[1] *= scale_y;
                    primitive.extra[2] *= scale_x.min(scale_y);
                }
                primitive
            })
            .collect::<Vec<_>>();
        assert!(
            scaled.len() <= PRIMITIVE_CAPACITY,
            "fixture exceeded primitive capacity"
        );
        self.primitive_count = scaled.len() as u32;
        self.queue
            .write_buffer(&self.primitive_buffer, 0, bytemuck::cast_slice(&scaled));
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
            .expect("prepare graph view text");
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
                label: Some("graph-view-frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("graph-view-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.018,
                            g: 0.019,
                            b: 0.021,
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
            pass.set_vertex_buffer(0, self.primitive_buffer.slice(..));
            pass.draw(0..6, 0..self.primitive_count);
            self.text_renderer
                .render(&self.atlas, &self.viewport, &mut pass)
                .expect("render graph view text");
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
        Some(font_size * 1.6),
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
            bottom: (top + font_size * 1.6).ceil().min(surface_height as f32) as i32,
        },
        color: Color::rgba(text.color[0], text.color[1], text.color[2], text.color[3]),
    }
}

struct App {
    auto: bool,
    report_path: PathBuf,
    window: Option<Arc<Window>>,
    gfx: Option<GfxState>,
    session: GraphSession,
    cursor: [f32; 2],
    previous_cursor: [f32; 2],
    drag_token: Option<u64>,
    panning: bool,
    modifiers: ModifiersState,
    next_token: u64,
    present_count: u32,
}

impl App {
    fn new(auto: bool) -> Self {
        Self {
            auto,
            report_path: std::env::var_os("G0_9_GRAPH_VIEW_REPORT")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/tmp/motolii-g0-9-graph-view-report.json")),
            window: None,
            gfx: None,
            session: GraphSession::default(),
            cursor: [0.0; 2],
            previous_cursor: [0.0; 2],
            drag_token: None,
            panning: false,
            modifiers: ModifiersState::empty(),
            next_token: 1,
            present_count: 0,
        }
    }

    fn refresh(&mut self) {
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.update_scene(&self.session);
        }
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn write_report(&self) {
        let gfx = self.gfx.as_ref().expect("gfx state");
        let scene = build_scene(&self.session);
        let report = Report {
            ticket: "G0-9-native-graph-view",
            status: "fixture-complete",
            adapter: gfx.adapter.clone(),
            backend: gfx.backend.clone(),
            channel_count: self.session.channels.len(),
            key_count: self
                .session
                .channels
                .iter()
                .map(|channel| channel.keys.len())
                .sum(),
            primitive_count: scene.primitives.len(),
            text_run_count: scene.texts.len(),
            semantic_commit_count: self.session.semantic_commit_count,
            selected_key_count: self.session.selection.len(),
            navigation_change_count: self.session.navigation_change_count,
            selection_change_count: self.session.selection_change_count,
            readback_count: self.session.readback_count,
            hot_drag_resource_creation_count: self.session.hot_resource_creation_count,
            present_count: self.present_count,
            pass: self.present_count >= AUTO_PRESENT_TARGET
                && self.session.readback_count == 0
                && self.session.hot_resource_creation_count == 0,
        };
        std::fs::write(
            &self.report_path,
            serde_json::to_vec_pretty(&report).expect("serialize report"),
        )
        .expect("write graph view report");
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
                        .with_title("Motolii native Blender-like Graph View")
                        .with_inner_size(LogicalSize::new(WIDTH as f64, HEIGHT as f64)),
                )
                .expect("graph view window"),
        );
        self.gfx = Some(GfxState::new(&window, &self.session));
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
            WindowEvent::CloseRequested => {
                self.write_report();
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(gfx) = self.gfx.as_mut() {
                    gfx.configure(size.width, size.height, &self.session);
                }
                window.request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let scale = window.scale_factor();
                self.previous_cursor = self.cursor;
                self.cursor = [(position.x / scale) as f32, (position.y / scale) as f32];
                let changed = if self.drag_token.is_some() {
                    self.session.update_drag_screen(self.cursor)
                } else if self.panning {
                    self.session.pan_by_view([
                        self.cursor[0] - self.previous_cursor[0],
                        self.cursor[1] - self.previous_cursor[1],
                    ])
                } else {
                    self.session.update_marquee(self.cursor)
                };
                if changed {
                    self.refresh();
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if let Some(target) = hit_target(&self.session, self.cursor) {
                    let token = self.next_token;
                    self.next_token += 1;
                    if self.session.begin_drag_at(
                        target,
                        token,
                        Some(self.cursor),
                        self.modifiers.shift_key(),
                    ) {
                        self.drag_token = Some(token);
                        self.refresh();
                    }
                } else if plot_contains(self.cursor)
                    && self
                        .session
                        .begin_marquee(self.cursor, self.modifiers.shift_key())
                {
                    self.refresh();
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                if let Some(token) = self.drag_token.take() {
                    self.session.release(token);
                    self.refresh();
                } else if self.session.release_marquee() {
                    self.refresh();
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Middle,
                ..
            } if plot_contains(self.cursor) => {
                self.panning = true;
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Middle,
                ..
            } => {
                self.panning = false;
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let amount = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y as f64 * 0.12,
                    MouseScrollDelta::PixelDelta(position) => position.y * 0.005,
                };
                if self.session.zoom_about_screen(self.cursor, amount.exp()) {
                    self.refresh();
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers.state(),
            WindowEvent::Focused(false) => {
                self.panning = false;
                if self.session.cancel() {
                    self.drag_token = None;
                    self.refresh();
                }
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                let changed = match event.physical_key {
                    PhysicalKey::Code(KeyCode::Escape) => {
                        self.drag_token = None;
                        self.panning = false;
                        self.session.cancel()
                    }
                    PhysicalKey::Code(KeyCode::Home) => {
                        self.session.fit_all();
                        true
                    }
                    PhysicalKey::Code(KeyCode::KeyF) => self.session.fit_selection(),
                    PhysicalKey::Code(KeyCode::KeyS) => {
                        self.session.toggle_snap();
                        true
                    }
                    _ => false,
                };
                if changed {
                    self.refresh();
                }
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
        .expect("run graph view spike");
}
