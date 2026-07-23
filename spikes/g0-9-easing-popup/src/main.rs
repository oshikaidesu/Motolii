use std::{path::PathBuf, sync::Arc};

use g0_9_easing_popup::{
    build_popup_scene, curve_point_from_graph, hit_action, hit_handle, hit_preset,
    hit_preset_index, place_popup, Bezier, Handle, LogicalRect, PhysicalRect, PopupAction,
    PopupSession, PopupVisualState, Primitive, SpikePresetStore, TextPrimitive, POPUP_HEIGHT,
    POPUP_WIDTH,
};
use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use serde::Deserialize;
use serde_json::json;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition},
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId, WindowLevel},
};
use wry::{dpi::LogicalSize as WebSize, Rect, WebView, WebViewBuilder};

const HOST_WIDTH: f64 = 860.0;
const HOST_HEIGHT: f64 = 560.0;

#[derive(Debug)]
enum UserEvent {
    WebMessage(String),
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AnchorWire {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenWire {
    kind: String,
    anchor: AnchorWire,
    layout_epoch: u64,
}

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

struct PopupGfx {
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

impl PopupGfx {
    fn new(
        window: &Arc<Window>,
        session: &PopupSession,
        store: &SpikePresetStore,
        visual: PopupVisualState,
    ) -> Self {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(Arc::clone(window))
            .expect("native popup surface");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("popup surface adapter");
        let adapter_info = adapter.get_info();
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("g0-9-easing-popup"),
            ..Default::default()
        }))
        .expect("popup device");
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
            label: Some("easing-popup-screen"),
            size: std::mem::size_of::<ScreenUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let primitive_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("easing-popup-primitives"),
            size: (std::mem::size_of::<Primitive>() * 1024) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("easing-popup-bind-group-layout"),
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
            label: Some("easing-popup-bind-group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_buffer.as_entire_binding(),
            }],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("easing-popup-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("popup.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("easing-popup-pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("easing-popup-pipeline"),
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
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let viewport = Viewport::new(&device, &cache);
        let mut atlas = TextAtlas::new(&device, &queue, &cache, format);
        let text_renderer =
            TextRenderer::new(&mut atlas, &device, wgpu::MultisampleState::default(), None);
        let mut gfx = Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            bind_group,
            screen_buffer,
            primitive_buffer,
            primitive_count: 0,
            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,
            texts: Vec::new(),
            adapter: adapter_info.name,
            backend: format!("{:?}", adapter_info.backend),
        };
        gfx.update_scene(session, store, visual);
        gfx
    }

    fn configure(
        &mut self,
        width: u32,
        height: u32,
        session: &PopupSession,
        store: &SpikePresetStore,
        visual: PopupVisualState,
    ) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.update_scene(session, store, visual);
    }

    fn update_scene(
        &mut self,
        session: &PopupSession,
        store: &SpikePresetStore,
        visual: PopupVisualState,
    ) {
        let scene = build_popup_scene(session, store, visual);
        let scale_x = self.config.width as f32 / POPUP_WIDTH as f32;
        let scale_y = self.config.height as f32 / POPUP_HEIGHT as f32;
        let scaled = scene
            .primitives
            .iter()
            .copied()
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
            scaled.len() <= 1024,
            "popup exceeded static primitive capacity"
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
            .expect("prepare popup text");
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
                label: Some("easing-popup-frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("easing-popup-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0103,
                            g: 0.0103,
                            b: 0.0103,
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
                .expect("render popup text");
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
    width: u32,
    height: u32,
) -> PreparedText {
    let size = text.size * scale_y;
    let mut buffer = Buffer::new(font_system, Metrics::new(size, size * 1.25));
    buffer.set_size(font_system, Some(text.width * scale_x), Some(size * 1.5));
    buffer.set_text(
        font_system,
        &text.text,
        &Attrs::new().family(if text.monospace {
            Family::Monospace
        } else {
            Family::SansSerif
        }),
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
                .min(width as f32) as i32,
            bottom: (top + size * 1.5).ceil().min(height as f32) as i32,
        },
        color: Color::rgba(text.color[0], text.color[1], text.color[2], text.color[3]),
    }
}

struct NativePopup {
    window: Arc<Window>,
    gfx: PopupGfx,
    session: PopupSession,
    cursor: [f32; 2],
    drag_token: Option<u64>,
    visual: PopupVisualState,
    received_focus: bool,
    needs_redraw: bool,
    present_count: u32,
}

struct App {
    proxy: EventLoopProxy<UserEvent>,
    host: Option<Arc<Window>>,
    webview: Option<WebView>,
    popup: Option<NativePopup>,
    layout_epoch: u64,
    revision: u64,
    current_curve: Bezier,
    next_drag_token: u64,
    open_count: u32,
    stale_open_rejected: u32,
    semantic_commit_count: u32,
    settings_write_count: u32,
    store: SpikePresetStore,
    store_path: PathBuf,
    report_path: PathBuf,
    last_placement: Option<g0_9_easing_popup::Placement>,
}

impl App {
    fn new(proxy: EventLoopProxy<UserEvent>) -> Self {
        let store_path = std::env::var_os("G0_9_EASING_STORE")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/tmp/motolii-g0-9-easing-popup-presets.json"));
        let store = SpikePresetStore::load(&store_path).unwrap_or_default();
        Self {
            proxy,
            host: None,
            webview: None,
            popup: None,
            layout_epoch: 1,
            revision: 1,
            current_curve: Bezier::SMOOTH,
            next_drag_token: 1,
            open_count: 0,
            stale_open_rejected: 0,
            semantic_commit_count: 0,
            settings_write_count: 0,
            store,
            store_path,
            report_path: std::env::var_os("G0_9_EASING_REPORT")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/tmp/motolii-g0-9-easing-popup-report.json")),
            last_placement: None,
        }
    }

    fn update_webview_bounds(&self) {
        let (Some(host), Some(webview)) = (&self.host, &self.webview) else {
            return;
        };
        let size = host.inner_size().to_logical::<f64>(host.scale_factor());
        let _ = webview.set_bounds(Rect {
            position: wry::dpi::LogicalPosition::new(0.0, 0.0).into(),
            size: WebSize::new(size.width, size.height).into(),
        });
        let _ = webview.evaluate_script(&format!("window.__layoutEpoch={};", self.layout_epoch));
    }

    fn open_popup(&mut self, event_loop: &ActiveEventLoop, request: OpenWire) {
        if request.kind != "open-easing" || request.layout_epoch != self.layout_epoch {
            self.stale_open_rejected += 1;
            self.write_report();
            return;
        }
        self.popup = None;
        let host = self.host.as_ref().expect("host exists");
        let scale = host.scale_factor();
        let host_origin = host.outer_position().unwrap_or(PhysicalPosition::new(0, 0));
        let monitor = host.current_monitor().or_else(|| host.primary_monitor());
        let (monitor_position, monitor_size) = monitor
            .map(|monitor| (monitor.position(), monitor.size()))
            .unwrap_or((PhysicalPosition::new(0, 0), host.inner_size()));
        let placement = place_popup(
            [host_origin.x, host_origin.y],
            LogicalRect {
                x: request.anchor.x,
                y: request.anchor.y,
                width: request.anchor.width,
                height: request.anchor.height,
            },
            scale,
            PhysicalRect {
                x: monitor_position.x,
                y: monitor_position.y,
                width: monitor_size.width,
                height: monitor_size.height,
            },
        )
        .expect("valid popup placement");
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Motolii · Interval Easing")
                        .with_inner_size(LogicalSize::new(POPUP_WIDTH, POPUP_HEIGHT))
                        .with_position(PhysicalPosition::new(placement.x, placement.y))
                        .with_decorations(false)
                        .with_resizable(false)
                        .with_window_level(WindowLevel::AlwaysOnTop),
                )
                .expect("native popup window"),
        );
        let session = PopupSession::new(self.current_curve, self.revision, self.layout_epoch);
        let visual = PopupVisualState {
            focused_handle: Some(Handle::Start),
            ..Default::default()
        };
        let mut gfx = PopupGfx::new(&window, &session, &self.store, visual);
        let initial_present_count = u32::from(gfx.render());
        window.focus_window();
        self.open_count += 1;
        self.last_placement = Some(placement);
        self.popup = Some(NativePopup {
            window,
            gfx,
            session,
            cursor: [0.0; 2],
            drag_token: None,
            visual,
            received_focus: false,
            needs_redraw: initial_present_count == 0,
            present_count: initial_present_count,
        });
        if let Some(popup) = self.popup.as_ref() {
            popup.window.request_redraw();
        }
        self.write_report();
    }

    fn refresh_popup(&mut self) {
        if let Some(popup) = self.popup.as_mut() {
            popup
                .gfx
                .update_scene(&popup.session, &self.store, popup.visual);
            if popup.gfx.render() {
                popup.present_count += 1;
                popup.needs_redraw = false;
            } else {
                popup.needs_redraw = true;
            }
            popup.window.request_redraw();
        }
        self.write_report();
    }

    fn save_current_preset(&mut self) {
        let Some(popup) = self.popup.as_mut() else {
            return;
        };
        let name = format!("My curve {}", self.store.presets.len() + 1);
        self.store.save_curve(name, popup.session.curve());
        if self.store.save(&self.store_path).is_ok() {
            popup.session.record_settings_write();
            self.settings_write_count += 1;
        }
        self.refresh_popup();
    }

    fn favorite_latest(&mut self) {
        let Some(id) = self.store.presets.last().map(|preset| preset.id.clone()) else {
            return;
        };
        if self.store.set_favorite(&id) && self.store.save(&self.store_path).is_ok() {
            if let Some(popup) = self.popup.as_mut() {
                popup.session.record_settings_write();
            }
            self.settings_write_count += 1;
        }
        self.refresh_popup();
    }

    fn write_report(&self) {
        let (present_count, readback_count, hot_creations, ax_count, adapter, backend) = self
            .popup
            .as_ref()
            .map_or((0, 0, 0, 0, String::new(), String::new()), |popup| {
                (
                    popup.present_count,
                    popup.session.readback_count,
                    popup.session.hot_resource_creation_count,
                    popup.session.bounded_accessibility().len(),
                    popup.gfx.adapter.clone(),
                    popup.gfx.backend.clone(),
                )
            });
        let report = json!({
            "ticket": "G0-9-native-easing-popup",
            "status": "isolated-spike",
            "popup_content_owner": "native-wgpu",
            "react_owner": "trigger-and-summary-only",
            "open_count": self.open_count,
            "stale_open_rejected": self.stale_open_rejected,
            "layout_epoch": self.layout_epoch,
            "revision": self.revision,
            "semantic_commit_count": self.semantic_commit_count,
            "settings_write_count": self.settings_write_count,
            "user_preset_count": self.store.presets.len(),
            "thumbnail_source": "typed-curve-projection",
            "present_count": present_count,
            "readback_count": readback_count,
            "hot_drag_resource_creation_count": hot_creations,
            "bounded_accessibility_node_count": ax_count,
            "adapter": adapter,
            "backend": backend,
            "placement": self.last_placement.map(|placement| json!({
                "x": placement.x,
                "y": placement.y,
                "width": placement.width,
                "height": placement.height,
                "vertical": format!("{:?}", placement.vertical),
            })),
        });
        let _ = std::fs::write(
            &self.report_path,
            serde_json::to_vec_pretty(&report).unwrap(),
        );
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.host.is_some() {
            return;
        }
        let host = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Motolii hybrid Easing popup spike")
                        .with_inner_size(LogicalSize::new(HOST_WIDTH, HOST_HEIGHT)),
                )
                .expect("host window"),
        );
        let size = host.inner_size().to_logical::<f64>(host.scale_factor());
        let proxy = self.proxy.clone();
        let webview = WebViewBuilder::new()
            .with_bounds(Rect {
                position: wry::dpi::LogicalPosition::new(0.0, 0.0).into(),
                size: WebSize::new(size.width, size.height).into(),
            })
            .with_accept_first_mouse(true)
            .with_html(HOST_HTML)
            .with_ipc_handler(move |request| {
                let _ = proxy.send_event(UserEvent::WebMessage(request.body().to_owned()));
            })
            .build_as_child(host.as_ref())
            .expect("React-like trigger WebView");
        self.host = Some(host);
        self.webview = Some(webview);
        self.update_webview_bounds();
        self.write_report();
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        let UserEvent::WebMessage(message) = event;
        if let Ok(request) = serde_json::from_str::<OpenWire>(&message) {
            self.open_popup(event_loop, request);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(popup) = self.popup.as_ref().filter(|popup| popup.needs_redraw) {
            popup.window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self
            .host
            .as_ref()
            .is_some_and(|host| host.id() == window_id)
        {
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                    self.layout_epoch += 1;
                    if let Some(popup) = self.popup.as_mut() {
                        popup.session.cancel();
                    }
                    self.popup = None;
                    self.update_webview_bounds();
                    self.write_report();
                }
                _ => {}
            }
            return;
        }
        if self
            .popup
            .as_ref()
            .is_none_or(|popup| popup.window.id() != window_id)
        {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                if let Some(popup) = self.popup.as_mut() {
                    popup.session.cancel();
                }
                self.popup = None;
                self.write_report();
            }
            WindowEvent::Focused(focused) => {
                if focused {
                    if let Some(popup) = self.popup.as_mut() {
                        popup.received_focus = true;
                    }
                } else if self
                    .popup
                    .as_ref()
                    .is_some_and(|popup| popup.received_focus)
                {
                    if let Some(popup) = self.popup.as_mut() {
                        popup.session.cancel();
                    }
                    self.popup = None;
                    self.write_report();
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(popup) = self.popup.as_mut() {
                    popup.gfx.configure(
                        size.width,
                        size.height,
                        &popup.session,
                        &self.store,
                        popup.visual,
                    );
                    popup.needs_redraw = true;
                    popup.window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(popup) = self.popup.as_mut() {
                    if popup.gfx.render() {
                        popup.present_count += 1;
                        popup.needs_redraw = false;
                    }
                }
                self.write_report();
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(popup) = self.popup.as_mut() {
                    let scale = popup.window.scale_factor() as f32;
                    popup.cursor = [position.x as f32 / scale, position.y as f32 / scale];
                    let next_preset = hit_preset_index(popup.cursor);
                    let next_handle = hit_handle(popup.session.curve(), popup.cursor);
                    let visual_changed = popup.visual.hovered_preset != next_preset
                        || popup.visual.hovered_handle != next_handle;
                    popup.visual.hovered_preset = next_preset;
                    popup.visual.hovered_handle = next_handle;
                    if popup.drag_token.is_some() {
                        popup
                            .session
                            .update_drag(curve_point_from_graph(popup.cursor));
                        popup
                            .gfx
                            .update_scene(&popup.session, &self.store, popup.visual);
                        popup.window.request_redraw();
                    } else if visual_changed {
                        popup
                            .gfx
                            .update_scene(&popup.session, &self.store, popup.visual);
                        popup.window.request_redraw();
                    }
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => match state {
                ElementState::Pressed => {
                    let (cursor, curve) = self
                        .popup
                        .as_ref()
                        .map(|popup| (popup.cursor, popup.session.curve()))
                        .unwrap();
                    if let Some(handle) = hit_handle(curve, cursor) {
                        let token = self.next_drag_token;
                        self.next_drag_token += 1;
                        if let Some(popup) = self.popup.as_mut() {
                            if popup.session.begin_drag(handle, token) {
                                popup.drag_token = Some(token);
                                popup.visual.focused_handle = Some(handle);
                            }
                        }
                        self.refresh_popup();
                    } else if let Some(curve) = hit_preset(cursor) {
                        if let Some(popup) = self.popup.as_mut() {
                            popup.session.apply_preset(curve);
                            self.current_curve = curve;
                            self.revision += 1;
                            self.semantic_commit_count += 1;
                            popup.session.rebase(self.revision, self.layout_epoch);
                        }
                        self.refresh_popup();
                    } else if let Some(action) = hit_action(cursor) {
                        match action {
                            PopupAction::SavePreset => self.save_current_preset(),
                            PopupAction::FavoriteLatest => self.favorite_latest(),
                            PopupAction::Close => {
                                if let Some(popup) = self.popup.as_mut() {
                                    popup.session.cancel();
                                }
                                self.popup = None;
                                self.write_report();
                            }
                        }
                    }
                }
                ElementState::Released => {
                    let token = self
                        .popup
                        .as_mut()
                        .and_then(|popup| popup.drag_token.take());
                    if let Some(token) = token {
                        let committed = self.popup.as_mut().and_then(|popup| {
                            popup
                                .session
                                .release(token, self.revision, self.layout_epoch)
                        });
                        if let Some(commit) = committed {
                            self.current_curve = commit.curve;
                            self.revision += 1;
                            self.semantic_commit_count += 1;
                            if let Some(popup) = self.popup.as_mut() {
                                popup.session.rebase(self.revision, self.layout_epoch);
                            }
                        }
                        self.refresh_popup();
                    }
                }
            },
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                match event.logical_key {
                    Key::Named(NamedKey::Escape) => {
                        if let Some(popup) = self.popup.as_mut() {
                            popup.session.cancel();
                        }
                        self.popup = None;
                        self.write_report();
                    }
                    Key::Named(NamedKey::Tab) => {
                        if let Some(popup) = self.popup.as_mut() {
                            popup.visual.focused_handle = Some(match popup.visual.focused_handle {
                                Some(Handle::Start) => Handle::End,
                                Some(Handle::End) | None => Handle::Start,
                            });
                        }
                        self.refresh_popup();
                    }
                    Key::Named(NamedKey::ArrowLeft)
                    | Key::Named(NamedKey::ArrowRight)
                    | Key::Named(NamedKey::ArrowUp)
                    | Key::Named(NamedKey::ArrowDown) => {
                        let Some(popup) = self.popup.as_mut() else {
                            return;
                        };
                        let curve = popup.session.curve();
                        let handle = popup.visual.focused_handle.unwrap_or(Handle::Start);
                        let mut point = match handle {
                            Handle::Start => [curve.x1, curve.y1],
                            Handle::End => [curve.x2, curve.y2],
                        };
                        match event.logical_key {
                            Key::Named(NamedKey::ArrowLeft) => point[0] -= 0.01,
                            Key::Named(NamedKey::ArrowRight) => point[0] += 0.01,
                            Key::Named(NamedKey::ArrowUp) => point[1] += 0.01,
                            Key::Named(NamedKey::ArrowDown) => point[1] -= 0.01,
                            _ => {}
                        }
                        let token = self.next_drag_token;
                        self.next_drag_token += 1;
                        popup.session.begin_drag(handle, token);
                        popup.session.update_drag(point);
                        if let Some(commit) =
                            popup
                                .session
                                .release(token, self.revision, self.layout_epoch)
                        {
                            self.current_curve = commit.curve;
                            self.revision += 1;
                            self.semantic_commit_count += 1;
                            popup.session.rebase(self.revision, self.layout_epoch);
                        }
                        self.refresh_popup();
                    }
                    Key::Character(value) if value.eq_ignore_ascii_case("s") => {
                        self.save_current_preset()
                    }
                    Key::Character(value) if value.eq_ignore_ascii_case("f") => {
                        self.favorite_latest()
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

const HOST_HTML: &str = r#"<!doctype html><html><head><meta charset="utf-8"><style>
:root{color-scheme:dark}*{box-sizing:border-box}html,body{margin:0;height:100%;background:#17191e;color:#e9ebef;font:13px -apple-system,BlinkMacSystemFont,sans-serif}
main{display:grid;grid-template-columns:220px 1fr 230px;height:100%}.panel{padding:18px;border-right:1px solid #343740;background:#22252c}.panel:last-child{border:0;border-left:1px solid #343740}.stage{display:grid;place-items:center;background:#111319}.frame{width:72%;aspect-ratio:16/9;border:1px solid #555b68;background:#202633;display:grid;place-items:center;color:#8d94a3}
h2{font-size:11px;letter-spacing:.12em;color:#a7adb9;margin:0 0 18px}.effect{padding:12px;border:1px solid #3a3e48;background:#292d35}.curve-row{display:flex;align-items:center;gap:8px;margin-top:18px}.graph{width:34px;height:28px;border:1px solid #636a78;background:#15171b;color:#f0a657;border-radius:3px;cursor:pointer;font-size:17px}.summary{font:11px ui-monospace,SFMono-Regular,monospace;color:#c8cbd2}.hint{margin-top:18px;color:#9096a2;line-height:1.5}
</style></head><body><main><section class="panel"><h2>BROWSER</h2><div class="effect">Echo Bloom<br><small>Pulse rings</small></div></section><section class="stage"><div class="frame">native Stage remains independent</div></section><section class="panel"><h2>INSPECTOR</h2><strong>Intensity</strong><div class="curve-row"><button id="graph" class="graph" aria-label="Open Interval Easing Editor">⌁</button><span class="summary">Smooth · 0.40 0.00 0.20 1.00</span></div><p class="hint">React owns this trigger and summary only.<br>The complete editor opens as a native wgpu popup.</p></section></main><script>
window.__layoutEpoch=1;document.querySelector('#graph').addEventListener('click',event=>{const r=event.currentTarget.getBoundingClientRect();window.ipc.postMessage(JSON.stringify({kind:'open-easing',anchor:{x:r.x,y:r.y,width:r.width,height:r.height},layoutEpoch:window.__layoutEpoch}))});
</script></body></html>"#;

fn main() {
    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .expect("event loop");
    event_loop.set_control_flow(ControlFlow::Wait);
    let proxy = event_loop.create_proxy();
    event_loop
        .run_app(&mut App::new(proxy))
        .expect("native Easing popup spike");
}
