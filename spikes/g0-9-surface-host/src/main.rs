use std::{
    borrow::Cow,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use g0_9_surface_host::{
    AcceptanceCounters, SurfaceLayout, LEFT_WEBVIEW_WIDTH, RIGHT_WEBVIEW_WIDTH,
};
use serde_json::json;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    window::{Window, WindowId},
};
use wry::{
    dpi::{LogicalPosition as WebPosition, LogicalSize as WebSize},
    Rect, WebView, WebViewBuilder,
};

const INITIAL_WIDTH: f64 = 1200.0;
const INITIAL_HEIGHT: f64 = 800.0;

#[derive(Debug)]
enum UserEvent {
    WebMessage(String),
}

struct GfxState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    stage_pipeline: wgpu::RenderPipeline,
    timeline_pipeline: wgpu::RenderPipeline,
}

enum RenderOutcome {
    Presented,
    Reconfigure,
    Skip,
    Validation,
}

impl GfxState {
    fn new(window: &Arc<Window>) -> Self {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(Arc::clone(window)).unwrap();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("surface adapter");
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("g0-9-surface-host"),
            ..Default::default()
        }))
        .expect("surface device");

        let size = window.inner_size();
        let capabilities = surface.get_capabilities(&adapter);
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

        let stage_pipeline = create_pipeline(&device, format, "stage", [0.08, 0.42, 0.92, 1.0]);
        let timeline_pipeline =
            create_pipeline(&device, format, "timeline", [0.95, 0.35, 0.12, 1.0]);

        Self {
            surface,
            device,
            queue,
            config,
            stage_pipeline,
            timeline_pipeline,
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

    fn render(&mut self, layout: SurfaceLayout) -> RenderOutcome {
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
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("g0-9-surface-frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("g0-9-stage-timeline-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.025,
                            g: 0.027,
                            b: 0.033,
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
            pass.set_pipeline(&self.stage_pipeline);
            pass.set_viewport(
                layout.native_x,
                0.0,
                layout.native_width,
                layout.stage_height,
                0.0,
                1.0,
            );
            pass.draw(0..3, 0..1);
            pass.set_pipeline(&self.timeline_pipeline);
            pass.set_viewport(
                layout.native_x,
                layout.timeline_y,
                layout.native_width,
                layout.timeline_height,
                0.0,
                1.0,
            );
            pass.draw(0..3, 0..1);
        }
        self.queue.submit([encoder.finish()]);
        frame.present();
        RenderOutcome::Presented
    }
}

fn create_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    label: &'static str,
    color: [f32; 4],
) -> wgpu::RenderPipeline {
    let shader_source = format!(
        r#"
@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {{
    let x = f32(i32(index) - 1);
    let y = f32(i32(index & 1u) * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}}

@fragment
fn fs_main() -> @location(0) vec4<f32> {{
    return vec4<f32>({:.8}, {:.8}, {:.8}, {:.8});
}}
"#,
        color[0], color[1], color[2], color[3]
    );
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(Cow::Owned(shader_source)),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(format.into())],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

struct State {
    proxy: EventLoopProxy<UserEvent>,
    window: Option<Arc<Window>>,
    left_webview: Option<WebView>,
    right_webview: Option<WebView>,
    gfx: Option<GfxState>,
    layout: Option<SurfaceLayout>,
    counters: AcceptanceCounters,
    cursor_x: f64,
    native_drag_active: bool,
    resize_target: u32,
    resize_requests: u32,
    next_resize: Instant,
    report_path: PathBuf,
}

impl State {
    fn new(proxy: EventLoopProxy<UserEvent>) -> Self {
        let resize_target = std::env::var("G0_9_RESIZE_TARGET")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(100);
        let report_path = std::env::var_os("G0_9_REPORT")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/tmp/motolii-g0-9-surface-host-report.json"));
        Self {
            proxy,
            window: None,
            left_webview: None,
            right_webview: None,
            gfx: None,
            layout: None,
            counters: AcceptanceCounters::default(),
            cursor_x: 0.0,
            native_drag_active: false,
            resize_target,
            resize_requests: 0,
            next_resize: Instant::now() + Duration::from_millis(500),
            report_path,
        }
    }

    fn update_layout(&mut self, width: u32, height: u32) {
        let Some(window) = &self.window else {
            return;
        };
        let Some(layout) = SurfaceLayout::try_new(width, height, window.scale_factor()) else {
            self.layout = None;
            return;
        };
        let left = Rect {
            position: WebPosition::new(0.0, 0.0).into(),
            size: WebSize::new(LEFT_WEBVIEW_WIDTH, layout.logical_height).into(),
        };
        let right = Rect {
            position: WebPosition::new(layout.logical_width - RIGHT_WEBVIEW_WIDTH, 0.0).into(),
            size: WebSize::new(RIGHT_WEBVIEW_WIDTH, layout.logical_height).into(),
        };
        if let Some(webview) = &self.left_webview {
            webview.set_bounds(left).expect("left bounds");
        }
        if let Some(webview) = &self.right_webview {
            webview.set_bounds(right).expect("right bounds");
        }
        self.counters.layout_epoch += 1;
        self.layout = Some(layout);
    }

    fn consume_web_message(&mut self, message: &str) {
        match message {
            value if value.ends_with(":drag-start") => self.counters.web_drag_started += 1,
            value if value.ends_with(":drag-move") => self.counters.web_drag_moved += 1,
            value if value.ends_with(":drag-end") => self.counters.web_drag_ended += 1,
            value if value.ends_with(":input") => self.counters.web_input_events += 1,
            "left:focus-right" => {
                if let Some(webview) = &self.right_webview {
                    let _ = webview.focus();
                }
            }
            _ => {}
        }
        self.publish_report();
    }

    fn title(&self) -> String {
        let resize = if self.counters.resize_target_passes(self.resize_target) {
            "PASS".to_owned()
        } else {
            format!("{}/{}", self.counters.resize_events, self.resize_target)
        };
        let present = if self.counters.present_invariant_holds() {
            "PASS"
        } else {
            "FAIL"
        };
        let native_drag =
            if self.counters.native_drag_crossed_webview && self.counters.native_drag_released {
                "PASS"
            } else if self.native_drag_active {
                "ACTIVE"
            } else {
                "WAIT"
            };
        format!(
            "G0-9 wgpu29 | resize={resize} present={present} readback={} native-drag={native_drag} web={}/{}/{} input={}",
            self.counters.readback_count,
            self.counters.web_drag_started,
            self.counters.web_drag_moved,
            self.counters.web_drag_ended,
            self.counters.web_input_events,
        )
    }

    fn publish_report(&self) {
        let report = json!({
            "wgpu_major": 29,
            "surface_count": 1,
            "native_viewport_count": 2,
            "webview_count": 2,
            "acquire_count": self.counters.acquire_count,
            "present_count": self.counters.present_count,
            "readback_count": self.counters.readback_count,
            "present_invariant": self.counters.present_invariant_holds(),
            "resize_target": self.resize_target,
            "resize_events": self.counters.resize_events,
            "layout_epoch": self.counters.layout_epoch,
            "resize_pass": self.counters.resize_target_passes(self.resize_target),
            "native_drag_moves": self.counters.native_drag_moves,
            "native_drag_crossed_webview": self.counters.native_drag_crossed_webview,
            "native_drag_released": self.counters.native_drag_released,
            "web_drag_started": self.counters.web_drag_started,
            "web_drag_moved": self.counters.web_drag_moved,
            "web_drag_ended": self.counters.web_drag_ended,
            "web_input_events": self.counters.web_input_events,
        });
        let _ = std::fs::write(
            &self.report_path,
            serde_json::to_vec_pretty(&report).unwrap(),
        );
        if let Some(window) = &self.window {
            window.set_title(&self.title());
        }
    }
}

impl ApplicationHandler<UserEvent> for State {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = Window::default_attributes()
            .with_title("G0-9 wgpu29 starting")
            .with_inner_size(LogicalSize::new(INITIAL_WIDTH, INITIAL_HEIGHT));
        let window = Arc::new(event_loop.create_window(attributes).expect("host window"));
        let gfx = GfxState::new(&window);
        let initial = window.inner_size();
        let layout = SurfaceLayout::try_new(initial.width, initial.height, window.scale_factor())
            .expect("initial layout");
        let left_webview = make_webview(
            &window,
            self.proxy.clone(),
            "left",
            Rect {
                position: WebPosition::new(0.0, 0.0).into(),
                size: WebSize::new(LEFT_WEBVIEW_WIDTH, layout.logical_height).into(),
            },
            LEFT_HTML,
        );
        let right_webview = make_webview(
            &window,
            self.proxy.clone(),
            "right",
            Rect {
                position: WebPosition::new(layout.logical_width - RIGHT_WEBVIEW_WIDTH, 0.0).into(),
                size: WebSize::new(RIGHT_WEBVIEW_WIDTH, layout.logical_height).into(),
            },
            RIGHT_HTML,
        );

        self.window = Some(window);
        self.left_webview = Some(left_webview);
        self.right_webview = Some(right_webview);
        self.gfx = Some(gfx);
        self.layout = Some(layout);
        self.counters.layout_epoch = 1;
        self.publish_report();
        self.window.as_ref().unwrap().request_redraw();
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::WebMessage(message) => self.consume_web_message(&message),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    self.counters.resize_events += 1;
                    if let Some(gfx) = &mut self.gfx {
                        gfx.configure(size.width, size.height);
                    }
                    self.update_layout(size.width, size.height);
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                self.publish_report();
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(window) = self.window.clone() {
                    let size = window.inner_size();
                    self.update_layout(size.width, size.height);
                    window.request_redraw();
                }
                self.publish_report();
            }
            WindowEvent::RedrawRequested => {
                let Some(layout) = self.layout else {
                    return;
                };
                match self.gfx.as_mut().unwrap().render(layout) {
                    RenderOutcome::Presented => {
                        self.counters.acquire_count += 1;
                        self.counters.present_count += 1;
                    }
                    RenderOutcome::Reconfigure => {
                        if let Some(window) = &self.window {
                            let size = window.inner_size();
                            self.gfx
                                .as_mut()
                                .unwrap()
                                .configure(size.width, size.height);
                            window.request_redraw();
                        }
                    }
                    RenderOutcome::Skip => {}
                    RenderOutcome::Validation => event_loop.exit(),
                }
                self.publish_report();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_x = position.x;
                if self.native_drag_active {
                    self.counters.native_drag_moves += 1;
                    if self
                        .layout
                        .is_some_and(|layout| layout.cursor_is_over_webview(position.x))
                    {
                        self.counters.native_drag_crossed_webview = true;
                    }
                    self.publish_report();
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => match state {
                ElementState::Pressed => {
                    if self
                        .layout
                        .is_some_and(|layout| !layout.cursor_is_over_webview(self.cursor_x))
                    {
                        self.native_drag_active = true;
                        self.counters.native_drag_crossed_webview = false;
                        self.counters.native_drag_released = false;
                    }
                }
                ElementState::Released if self.native_drag_active => {
                    self.native_drag_active = false;
                    self.counters.native_drag_released = true;
                    self.publish_report();
                }
                ElementState::Released => {}
            },
            WindowEvent::CursorLeft { .. } if self.native_drag_active => {
                self.publish_report();
            }
            WindowEvent::CloseRequested => {
                self.publish_report();
                event_loop.exit();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        if self.resize_requests < self.resize_target && now >= self.next_resize {
            let request = self.resize_requests;
            let width = 1100.0 + f64::from(request % 7) * 17.0;
            let height = 720.0 + f64::from(request % 5) * 13.0;
            if let Some(window) = &self.window {
                let _ = window.request_inner_size(LogicalSize::new(width, height));
            }
            self.resize_requests += 1;
            self.next_resize = now + Duration::from_millis(25);
            event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_resize));
        } else if self.resize_requests >= self.resize_target {
            event_loop.set_control_flow(ControlFlow::Wait);
        } else {
            event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_resize));
        }
    }
}

fn make_webview(
    window: &Window,
    proxy: EventLoopProxy<UserEvent>,
    role: &'static str,
    bounds: Rect,
    html: &'static str,
) -> WebView {
    WebViewBuilder::new()
        .with_bounds(bounds)
        .with_accept_first_mouse(true)
        .with_html(html)
        .with_ipc_handler(move |request| {
            let _ = proxy.send_event(UserEvent::WebMessage(format!("{role}:{}", request.body())));
        })
        .build_as_child(window)
        .expect("opaque child webview")
}

const LEFT_HTML: &str = r#"<!doctype html><html><head><meta charset="utf-8"><style>
html,body{margin:0;height:100%;background:#25272e;color:#f5f6f8;font:15px -apple-system,sans-serif}
main{padding:18px}input,button{font:inherit;margin:4px 0;width:100%;box-sizing:border-box}
#drag{margin-top:18px;padding:28px 8px;border:2px solid #6fa8ff;border-radius:8px;text-align:center;touch-action:none}
</style></head><body><main><h2>Browser WebView</h2><label>Asset search<input aria-label="Asset search" value="cloud"></label>
<button onclick="window.ipc.postMessage('focus-right')">Focus Inspector</button>
<div id="drag" role="button" tabindex="0" aria-label="Drag asset to native Stage">Drag asset → Stage</div>
<p id="status">opaque child view</p></main><script>
const drag=document.querySelector('#drag');let moves=0;
drag.addEventListener('pointerdown',e=>{moves=0;drag.setPointerCapture(e.pointerId);window.ipc.postMessage('drag-start')});
drag.addEventListener('pointermove',e=>{if(e.buttons&&moves++<4)window.ipc.postMessage('drag-move')});
drag.addEventListener('pointerup',()=>window.ipc.postMessage('drag-end'));
document.querySelector('input').addEventListener('input',()=>window.ipc.postMessage('input'));
</script></body></html>"#;

const RIGHT_HTML: &str = r#"<!doctype html><html><head><meta charset="utf-8"><style>
html,body{margin:0;height:100%;background:#292b32;color:#f5f6f8;font:15px -apple-system,sans-serif}
main{padding:18px}input,button{font:inherit;margin:4px 0;width:100%;box-sizing:border-box}
</style></head><body><main><h2>Inspector WebView</h2><label>Opacity<input aria-label="Opacity" value="100%"></label>
<button onclick="window.ipc.postMessage('input')">Apply</button><p>same React-kit boundary</p></main></body></html>"#;

fn main() {
    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .expect("event loop");
    let proxy = event_loop.create_proxy();
    let mut state = State::new(proxy);
    event_loop.run_app(&mut state).expect("surface host");
}
