use std::{
    borrow::Cow,
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use g0_10_multi_surface_window::{LifecycleLedger, WindowRole};
use serde_json::json;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize},
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Fullscreen, Window, WindowId},
};

const EDITOR_SIZE: LogicalSize<f64> = LogicalSize::new(900.0, 680.0);
const PREVIEW_SIZE: LogicalSize<f64> = LogicalSize::new(720.0, 480.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AutoStage {
    InitialPresent,
    PreviewFault,
    EnterFullscreen,
    ExitFullscreen,
    ClosePreview,
    ReopenPreview,
    RefocusEditor,
    Complete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RenderOutcome {
    Presented,
    InjectedLost,
    Reconfigure,
    Skip,
    Validation,
}

struct SharedGpu {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl SharedGpu {
    fn new(window: &Arc<Window>) -> (Self, wgpu::Surface<'static>) {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(Arc::clone(window))
            .expect("editor surface");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("shared surface adapter");
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("g0-10-shared-device"),
            ..Default::default()
        }))
        .expect("shared surface device");
        (
            Self {
                instance,
                adapter,
                device,
                queue,
            },
            surface,
        )
    }
}

struct SurfaceWindow {
    role: WindowRole,
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    inject_loss_next: bool,
}

impl SurfaceWindow {
    fn new(
        role: WindowRole,
        window: Arc<Window>,
        surface: wgpu::Surface<'static>,
        gpu: &SharedGpu,
    ) -> Self {
        let capabilities = surface.get_capabilities(&gpu.adapter);
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
        surface.configure(&gpu.device, &config);
        let color = match role {
            WindowRole::Editor => [0.055, 0.06, 0.075, 1.0],
            WindowRole::DetachedPreview => [0.11, 0.085, 0.13, 1.0],
        };
        let pipeline = create_pipeline(&gpu.device, format, color);
        Self {
            role,
            window,
            surface,
            config,
            pipeline,
            inject_loss_next: false,
        }
    }

    fn configure(&mut self, gpu: &SharedGpu) {
        let size = self.window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&gpu.device, &self.config);
    }

    fn render(&mut self, gpu: &SharedGpu) -> RenderOutcome {
        if self.inject_loss_next {
            self.inject_loss_next = false;
            return RenderOutcome::InjectedLost;
        }
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
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
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("g0-10-frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("g0-10-surface-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
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
            pass.draw(0..3, 0..1);
        }
        gpu.queue.submit([encoder.finish()]);
        frame.present();
        RenderOutcome::Presented
    }
}

struct State {
    windows: HashMap<WindowId, SurfaceWindow>,
    gpu: Option<SharedGpu>,
    ledger: LifecycleLedger,
    report_path: PathBuf,
    auto: bool,
    auto_stage: AutoStage,
    fullscreen_resize_baseline: u32,
    fullscreen_transition_started: Option<Instant>,
    completed: bool,
}

impl State {
    fn new(auto: bool) -> Self {
        Self {
            windows: HashMap::new(),
            gpu: None,
            ledger: LifecycleLedger::default(),
            report_path: std::env::var_os("G0_10_REPORT")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    PathBuf::from("/tmp/motolii-g0-10-multi-surface-window-report.json")
                }),
            auto,
            auto_stage: AutoStage::InitialPresent,
            fullscreen_resize_baseline: 0,
            fullscreen_transition_started: None,
            completed: false,
        }
    }

    fn create_editor(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Motolii G0-10 Editor host")
                        .with_inner_size(EDITOR_SIZE)
                        .with_position(LogicalPosition::new(40.0, 80.0)),
                )
                .expect("editor window"),
        );
        let (gpu, surface) = SharedGpu::new(&window);
        let entry = SurfaceWindow::new(WindowRole::Editor, Arc::clone(&window), surface, &gpu);
        self.ledger
            .opened(WindowRole::Editor, window.scale_factor());
        self.windows.insert(window.id(), entry);
        self.gpu = Some(gpu);
    }

    fn create_preview(&mut self, event_loop: &ActiveEventLoop) {
        if self.window_id(WindowRole::DetachedPreview).is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Motolii G0-10 Detached Preview")
                        .with_inner_size(PREVIEW_SIZE)
                        .with_position(LogicalPosition::new(980.0, 120.0)),
                )
                .expect("preview window"),
        );
        let gpu = self.gpu.as_ref().expect("gpu after editor");
        let surface = gpu
            .instance
            .create_surface(Arc::clone(&window))
            .expect("preview surface");
        let entry = SurfaceWindow::new(
            WindowRole::DetachedPreview,
            Arc::clone(&window),
            surface,
            gpu,
        );
        self.ledger
            .opened(WindowRole::DetachedPreview, window.scale_factor());
        self.windows.insert(window.id(), entry);
        window.focus_window();
    }

    fn window_id(&self, role: WindowRole) -> Option<WindowId> {
        self.windows
            .iter()
            .find_map(|(id, entry)| (entry.role == role).then_some(*id))
    }

    fn window(&self, role: WindowRole) -> Option<&Arc<Window>> {
        self.windows
            .values()
            .find(|entry| entry.role == role)
            .map(|entry| &entry.window)
    }

    fn close_preview(&mut self) {
        if let Some(id) = self.window_id(WindowRole::DetachedPreview) {
            self.windows.remove(&id);
            self.ledger.close_preview();
        }
    }

    fn inject_preview_loss(&mut self) {
        if let Some(entry) = self
            .windows
            .values_mut()
            .find(|entry| entry.role == WindowRole::DetachedPreview)
        {
            entry.inject_loss_next = true;
            entry.window.request_redraw();
        }
    }

    fn auto_advance(&mut self, event_loop: &ActiveEventLoop) {
        if !self.auto || self.completed {
            return;
        }
        match self.auto_stage {
            AutoStage::InitialPresent => {
                if self.ledger.editor.present_count >= 2
                    && self.ledger.detached_preview.present_count >= 2
                {
                    self.inject_preview_loss();
                    self.auto_stage = AutoStage::PreviewFault;
                }
            }
            AutoStage::PreviewFault => {
                if self.ledger.detached_preview.injected_surface_lost_count == 1
                    && self.ledger.detached_preview.present_count >= 3
                {
                    if let Some(window) = self.window(WindowRole::DetachedPreview) {
                        window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                    }
                    self.fullscreen_resize_baseline =
                        self.ledger.detached_preview.resize_event_count;
                    self.fullscreen_transition_started = Some(Instant::now());
                    self.auto_stage = AutoStage::EnterFullscreen;
                }
            }
            AutoStage::EnterFullscreen => {
                if self
                    .window(WindowRole::DetachedPreview)
                    .is_some_and(|window| window.fullscreen().is_some())
                    && self.ledger.detached_preview.resize_event_count
                        > self.fullscreen_resize_baseline
                    && self.fullscreen_transition_elapsed()
                {
                    self.ledger
                        .record_fullscreen(WindowRole::DetachedPreview, true);
                    if let Some(window) = self.window(WindowRole::DetachedPreview) {
                        window.set_fullscreen(None);
                    }
                    self.fullscreen_resize_baseline =
                        self.ledger.detached_preview.resize_event_count;
                    self.fullscreen_transition_started = Some(Instant::now());
                    self.auto_stage = AutoStage::ExitFullscreen;
                }
            }
            AutoStage::ExitFullscreen => {
                if self
                    .window(WindowRole::DetachedPreview)
                    .is_some_and(|window| window.fullscreen().is_none())
                    && self.ledger.detached_preview.resize_event_count
                        > self.fullscreen_resize_baseline
                    && self.fullscreen_transition_elapsed()
                {
                    self.ledger
                        .record_fullscreen(WindowRole::DetachedPreview, false);
                    self.close_preview();
                    self.auto_stage = AutoStage::ClosePreview;
                }
            }
            AutoStage::ClosePreview => {
                if self.ledger.editor_presented_after_preview_close() {
                    self.create_preview(event_loop);
                    self.auto_stage = AutoStage::ReopenPreview;
                }
            }
            AutoStage::ReopenPreview => {
                if self.ledger.detached_preview.present_count >= 5 {
                    if let Some(window) = self.window(WindowRole::Editor) {
                        window.focus_window();
                    }
                    self.auto_stage = AutoStage::RefocusEditor;
                }
            }
            AutoStage::RefocusEditor => {
                if self.ledger.editor.focus_gained_count > 0 {
                    self.completed = self.ledger.host_state_preserved()
                        && self.ledger.fault_isolated_to_preview()
                        && self.ledger.editor_presented_after_preview_close()
                        && self.ledger.detached_preview.reopen_count == 1;
                    self.auto_stage = AutoStage::Complete;
                    self.publish_report();
                    event_loop.exit();
                }
            }
            AutoStage::Complete => {}
        }
    }

    fn publish_report(&self) {
        let report = json!({
            "status": if self.completed { "complete" } else { "running" },
            "auto_stage": format!("{:?}", self.auto_stage),
            "top_level_window_count": self.windows.len(),
            "surface_count_current": self.windows.len(),
            "surface_count_peak": self.ledger.surface_count_peak,
            "shared_device_count": usize::from(self.gpu.is_some()),
            "host_state": self.ledger.host_snapshot,
            "host_state_preserved": self.ledger.host_state_preserved(),
            "preview_open": self.ledger.preview_open,
            "editor_presented_after_preview_close": self.ledger.editor_presented_after_preview_close(),
            "fault_isolated_to_preview": self.ledger.fault_isolated_to_preview(),
            "editor": self.ledger.editor,
            "detached_preview": self.ledger.detached_preview,
        });
        let _ = std::fs::write(
            &self.report_path,
            serde_json::to_vec_pretty(&report).expect("serialize report"),
        );
    }

    fn request_all_redraws(&self) {
        for entry in self.windows.values() {
            entry.window.request_redraw();
        }
    }

    fn fullscreen_transition_elapsed(&self) -> bool {
        self.fullscreen_transition_started
            .is_some_and(|started| started.elapsed() >= Duration::from_millis(1500))
    }
}

impl ApplicationHandler for State {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.windows.is_empty() {
            self.create_editor(event_loop);
            self.create_preview(event_loop);
            self.publish_report();
            self.request_all_redraws();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(role) = self.windows.get(&window_id).map(|entry| entry.role) else {
            return;
        };
        match event {
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    if let (Some(gpu), Some(entry)) =
                        (self.gpu.as_ref(), self.windows.get_mut(&window_id))
                    {
                        entry.configure(gpu);
                        self.ledger.record_resize(role, entry.window.scale_factor());
                        entry.window.request_redraw();
                    }
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.ledger.record_scale_factor(role, scale_factor);
                if let (Some(gpu), Some(entry)) =
                    (self.gpu.as_ref(), self.windows.get_mut(&window_id))
                {
                    entry.configure(gpu);
                    entry.window.request_redraw();
                }
            }
            WindowEvent::Focused(focused) => self.ledger.record_focus(role, focused),
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed && !event.repeat =>
            {
                match event.logical_key {
                    Key::Character(ref key) if key.eq_ignore_ascii_case("l") => {
                        self.inject_preview_loss()
                    }
                    Key::Character(ref key) if key.eq_ignore_ascii_case("p") => {
                        if self.ledger.preview_open {
                            self.close_preview();
                        } else {
                            self.create_preview(event_loop);
                        }
                    }
                    Key::Character(ref key) if key.eq_ignore_ascii_case("f") => {
                        if let Some(window) = self.window(WindowRole::DetachedPreview) {
                            let entering = window.fullscreen().is_none();
                            window.set_fullscreen(entering.then_some(Fullscreen::Borderless(None)));
                            self.ledger
                                .record_fullscreen(WindowRole::DetachedPreview, entering);
                        }
                    }
                    Key::Named(NamedKey::Escape) => event_loop.exit(),
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested => {
                let outcome = {
                    let gpu = self.gpu.as_ref().expect("gpu after resume");
                    self.windows
                        .get_mut(&window_id)
                        .expect("known window")
                        .render(gpu)
                };
                match outcome {
                    RenderOutcome::Presented => self.ledger.record_present(role),
                    RenderOutcome::InjectedLost => {
                        let gpu = self.gpu.as_ref().expect("gpu after resume");
                        self.windows
                            .get_mut(&window_id)
                            .expect("known window")
                            .configure(gpu);
                        self.ledger.record_injected_loss(role);
                        self.windows
                            .get(&window_id)
                            .expect("known window")
                            .window
                            .request_redraw();
                    }
                    RenderOutcome::Reconfigure => {
                        let gpu = self.gpu.as_ref().expect("gpu after resume");
                        self.windows
                            .get_mut(&window_id)
                            .expect("known window")
                            .configure(gpu);
                        self.ledger.record_actual_reconfigure(role);
                        self.windows
                            .get(&window_id)
                            .expect("known window")
                            .window
                            .request_redraw();
                    }
                    RenderOutcome::Skip => {}
                    RenderOutcome::Validation => event_loop.exit(),
                }
            }
            WindowEvent::CloseRequested => {
                if role == WindowRole::DetachedPreview {
                    self.close_preview();
                } else {
                    event_loop.exit();
                }
            }
            _ => {}
        }
        self.publish_report();
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.auto {
            self.request_all_redraws();
            self.auto_advance(event_loop);
            self.publish_report();
        }
    }
}

fn create_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    color: [f32; 4],
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("g0-10-solid-color"),
        source: wgpu::ShaderSource::Wgsl(Cow::Owned(format!(
            r#"
@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {{
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );
    return vec4<f32>(positions[index], 0.0, 1.0);
}}

@fragment
fn fs_main() -> @location(0) vec4<f32> {{
    return vec4<f32>({:.8}, {:.8}, {:.8}, {:.8});
}}
"#,
            color[0], color[1], color[2], color[3]
        ))),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("g0-10-solid-layout"),
        bind_group_layouts: &[],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("g0-10-solid-pipeline"),
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
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

fn main() {
    let auto = std::env::args().any(|arg| arg == "--auto");
    let event_loop = EventLoop::new().expect("event loop");
    let mut state = State::new(auto);
    event_loop.run_app(&mut state).expect("g0-10 event loop");
}
