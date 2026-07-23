use std::{
    borrow::Cow,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use accesskit::{Action, Node, NodeId, Rect as AccessRect, Role, Tree, TreeId, TreeUpdate};
#[cfg(target_os = "macos")]
use block2::RcBlock;
#[cfg(target_os = "macos")]
use core_graphics::{
    event::{CGEvent, CGEventFlags, KeyCode},
    event_source::{CGEventSource, CGEventSourceStateID},
};
use g0_9_surface_host::{
    AcceptanceCounters, AccessibilityProjection, AxNodeId, AxRole, CompositionState,
    FocusCoordinator, FocusRole, LifecycleRecorder, SemanticCounts, ShortcutKey, ShortcutSink,
    SurfaceLayout, WebMessage, WebSource, LEFT_WEBVIEW_WIDTH, RIGHT_WEBVIEW_WIDTH,
};
#[cfg(target_os = "macos")]
use objc2::{rc::Retained, runtime::AnyObject};
#[cfg(target_os = "macos")]
use objc2_app_kit::{NSApplication, NSEvent, NSEventMask, NSEventModifierFlags};
#[cfg(target_os = "macos")]
use objc2_foundation::MainThreadMarker;
use serde_json::json;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    keyboard::{Key, ModifiersState, NamedKey},
    window::{Fullscreen, Window, WindowId},
};
use wry::{
    dpi::{LogicalPosition as WebPosition, LogicalSize as WebSize},
    Rect, WebView, WebViewBuilder,
};

const INITIAL_WIDTH: f64 = 1200.0;
const INITIAL_HEIGHT: f64 = 800.0;

const AX_ROOT: NodeId = NodeId(0);
const AX_STAGE: NodeId = NodeId(1);
const AX_STAGE_CANVAS: NodeId = NodeId(2);
const AX_TIMELINE: NodeId = NodeId(3);
const AX_TIMELINE_TRACKS: NodeId = NodeId(4);
const AX_TIMELINE_PLAYHEAD: NodeId = NodeId(5);
const AUTOMATED_FOCUS_STEPS: [FocusRole; 8] = [
    FocusRole::WebLeft,
    FocusRole::NativeTimeline,
    FocusRole::WebRight,
    FocusRole::NativeStage,
    FocusRole::WebRight,
    FocusRole::NativeTimeline,
    FocusRole::WebLeft,
    FocusRole::NativeStage,
];
const MAX_FOCUS_OBSERVATIONS: usize = 64;

fn ax_node_id(id: AxNodeId) -> NodeId {
    match id {
        AxNodeId::Root => AX_ROOT,
        AxNodeId::Stage => AX_STAGE,
        AxNodeId::StageCanvas => AX_STAGE_CANVAS,
        AxNodeId::Timeline => AX_TIMELINE,
        AxNodeId::TimelineTracks => AX_TIMELINE_TRACKS,
        AxNodeId::TimelinePlayhead => AX_TIMELINE_PLAYHEAD,
    }
}

/// クリック物理座標をStage/Timelineどちらの領域かに写像する純粋関数。
/// `SurfaceLayout::timeline_y`を境界として使い、`SurfaceLayout`自体が
/// 保証する非重複分割にそのまま従う。
fn native_click_role(cursor_y: f64, layout: SurfaceLayout) -> FocusRole {
    if cursor_y < f64::from(layout.timeline_y) {
        FocusRole::NativeStage
    } else {
        FocusRole::NativeTimeline
    }
}

fn ax_role(role: AxRole) -> Role {
    match role {
        AxRole::Window => Role::Window,
        AxRole::Pane => Role::Pane,
        AxRole::Generic => Role::GenericContainer,
    }
}

/// Documentが存在しないため、`SemanticCounts`は常に既定値のまま渡し、
/// ノード数がclip/key/selection数に連動しないようにする。
fn build_accessibility_tree(focus: FocusRole, layout: SurfaceLayout) -> TreeUpdate {
    let projection = AccessibilityProjection::project(SemanticCounts::default(), layout);
    let nodes = projection
        .nodes()
        .iter()
        .map(|spec| {
            let mut node = Node::new(ax_role(spec.role));
            node.set_label(spec.label);
            if !spec.children.is_empty() {
                node.set_children(
                    spec.children
                        .iter()
                        .copied()
                        .map(ax_node_id)
                        .collect::<Vec<_>>(),
                );
            }
            node.set_bounds(AccessRect {
                x0: spec.bounds.x0,
                y0: spec.bounds.y0,
                x1: spec.bounds.x1,
                y1: spec.bounds.y1,
            });
            if spec.focusable {
                node.add_action(Action::Focus);
            }
            (ax_node_id(spec.id), node)
        })
        .collect();

    TreeUpdate {
        nodes,
        tree: Some(Tree::new(AX_ROOT)),
        tree_id: TreeId::ROOT,
        focus: ax_node_id(projection.focused(focus)),
    }
}

#[derive(Debug)]
enum UserEvent {
    WebMessage(String),
    Accesskit(accesskit_winit::Event),
    NativeTab { backward: bool },
}

impl From<accesskit_winit::Event> for UserEvent {
    fn from(event: accesskit_winit::Event) -> Self {
        Self::Accesskit(event)
    }
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

struct FocusObservation {
    role: FocusRole,
    source: &'static str,
    responder_class: String,
    responder_family: &'static str,
    matches_role: bool,
}

struct AutomatedFocusCheck {
    step: usize,
    awaiting_result: bool,
    started: bool,
    next_action: Instant,
    deadline: Instant,
    pass: Option<bool>,
    error: Option<String>,
}

impl AutomatedFocusCheck {
    fn new(now: Instant) -> Self {
        Self {
            step: 0,
            awaiting_result: false,
            started: false,
            next_action: now,
            deadline: now + Duration::from_secs(15),
            pass: None,
            error: None,
        }
    }
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
    cursor_y: f64,
    native_drag_active: bool,
    resize_target: u32,
    resize_requests: u32,
    next_resize: Instant,
    report_path: PathBuf,
    accesskit: Option<accesskit_winit::Adapter>,
    focus: FocusCoordinator,
    shortcut_sink: ShortcutSink,
    composing_source: Option<WebSource>,
    shift_held: bool,
    minimize_lifecycle: LifecycleRecorder,
    fullscreen_lifecycle: LifecycleRecorder,
    occluded: bool,
    fullscreen_active: bool,
    left_ready: bool,
    right_ready: bool,
    native_focus_active: Arc<AtomicBool>,
    #[cfg(target_os = "macos")]
    native_tab_monitor: Option<Retained<AnyObject>>,
    focus_observations: Vec<FocusObservation>,
    automated_focus: Option<AutomatedFocusCheck>,
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
        let native_focus_active = Arc::new(AtomicBool::new(true));
        #[cfg(target_os = "macos")]
        let native_tab_monitor =
            install_native_tab_monitor(proxy.clone(), Arc::clone(&native_focus_active));
        let automated_focus = std::env::var_os("G0_9_AUTOMATE_FOCUS")
            .is_some()
            .then(|| AutomatedFocusCheck::new(Instant::now()));
        Self {
            proxy,
            window: None,
            left_webview: None,
            right_webview: None,
            gfx: None,
            layout: None,
            counters: AcceptanceCounters::default(),
            cursor_x: 0.0,
            cursor_y: 0.0,
            native_drag_active: false,
            resize_target,
            resize_requests: 0,
            next_resize: Instant::now() + Duration::from_millis(500),
            report_path,
            accesskit: None,
            focus: FocusCoordinator::new(0),
            shortcut_sink: ShortcutSink::default(),
            composing_source: None,
            shift_held: false,
            minimize_lifecycle: LifecycleRecorder::default(),
            fullscreen_lifecycle: LifecycleRecorder::default(),
            occluded: false,
            fullscreen_active: false,
            left_ready: false,
            right_ready: false,
            native_focus_active,
            #[cfg(target_os = "macos")]
            native_tab_monitor,
            focus_observations: Vec::new(),
            automated_focus,
        }
    }

    fn ime_owner(&self) -> Option<FocusRole> {
        self.composing_source.map(WebSource::focus_role)
    }

    /// `Window::focus_window`だけではWKWebView側がmacOSのfirst responderの
    /// ままになり実際には移動しないため、wryの`focus_parent`で明示的に
    /// resignさせる。
    fn apply_focus(&mut self, role: FocusRole, source: &'static str) {
        self.native_focus_active.store(
            matches!(role, FocusRole::NativeStage | FocusRole::NativeTimeline),
            Ordering::Release,
        );
        match role {
            FocusRole::WebLeft => {
                if let Some(webview) = &self.left_webview {
                    let _ = webview.focus();
                }
            }
            FocusRole::WebRight => {
                if let Some(webview) = &self.right_webview {
                    let _ = webview.focus();
                }
            }
            FocusRole::NativeStage | FocusRole::NativeTimeline => {
                if let Some(window) = &self.window {
                    window.focus_window();
                }
                if let Some(webview) = self.left_webview.as_ref().or(self.right_webview.as_ref()) {
                    let _ = webview.focus_parent();
                }
            }
        }
        if let (Some(layout), Some(adapter)) = (self.layout, &mut self.accesskit) {
            adapter.update_if_active(|| build_accessibility_tree(role, layout));
        }
        let (responder_class, responder_family) = actual_first_responder();
        let expected_family = focus_role_family(role);
        if self.focus_observations.len() == MAX_FOCUS_OBSERVATIONS {
            self.focus_observations.remove(0);
        }
        self.focus_observations.push(FocusObservation {
            role,
            source,
            responder_class,
            responder_family,
            matches_role: responder_family == expected_family,
        });
    }

    /// native領域(Stage/Timeline)への実クリックはOSがそのままfirst
    /// responderを渡すが、host側のFocusCoordinator/AX/reportは自動では
    /// 追従しないため、`FocusIn`と同じ同期経路でここから明示的に揃える。
    fn sync_native_click_focus(&mut self, layout: SurfaceLayout) {
        let role = native_click_role(self.cursor_y, layout);
        if role != self.focus.current() {
            self.focus.sync_current(role);
            self.apply_focus(role, "native-click");
            self.publish_report();
        }
    }

    /// `ready`未受信のWebViewへ先に配信すると未定義値を読んでしまい、
    /// 後続の`request_focus`が失敗するため、受信済みのものだけへ配る。
    fn push_epoch_to_ready_webviews(&self) {
        let script = format!("window.__motoliiEpoch = {};", self.focus.epoch());
        if self.left_ready {
            if let Some(webview) = &self.left_webview {
                let _ = webview.evaluate_script(&script);
            }
        }
        if self.right_ready {
            if let Some(webview) = &self.right_webview {
                let _ = webview.evaluate_script(&script);
            }
        }
    }

    /// レイアウトepochと連動させることで、resize/scale変更前に発行された
    /// Web側のfocus要求を確実に無効化する。
    fn set_layout_epoch(&mut self, epoch: u64) {
        self.counters.layout_epoch = epoch;
        self.focus.set_epoch(epoch);
        self.push_epoch_to_ready_webviews();
    }

    /// native winit経路とWebView内`keydown`中継の両方が同じ`ShortcutSink`を
    /// 通ることで、composition中の抑制が発生元によらず一貫する。
    fn observe_shortcut(&mut self, key: ShortcutKey) {
        if self.shortcut_sink.observe(key) {
            self.publish_report();
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
        self.layout = Some(layout);
        self.set_layout_epoch(self.counters.layout_epoch + 1);
        let focus = self.focus.current();
        if let Some(adapter) = &mut self.accesskit {
            adapter.update_if_active(|| build_accessibility_tree(focus, layout));
        }
    }

    /// パース失敗やstale epochのfocus要求は、状態・カウンタ・reportの
    /// 変更前に必ずreturnする。
    fn consume_web_message(&mut self, raw: &str) {
        let Ok((source, message)) = g0_9_surface_host::parse_web_message(raw) else {
            return;
        };
        match message {
            WebMessage::DragStart => self.counters.web_drag_started += 1,
            WebMessage::DragMove => self.counters.web_drag_moved += 1,
            WebMessage::DragEnd => self.counters.web_drag_ended += 1,
            WebMessage::Input => self.counters.web_input_events += 1,
            WebMessage::Ready => {
                match source {
                    WebSource::Left => self.left_ready = true,
                    WebSource::Right => self.right_ready = true,
                }
                self.push_epoch_to_ready_webviews();
            }
            // `request_focus`を経ずにWebView内クリック等で実際に起きた
            // フォーカス移動なので、epochを消費しない同期として扱いつつ、
            // OSアクセシビリティ側のfocusedノードも同じ投影で追従させる。
            WebMessage::FocusIn => {
                // native移譲後に遅延したIPCが届いてもcoordinatorをWebへ
                // 巻き戻さないよう、実first-responderとの一致を先に要求する。
                if actual_first_responder().1 != "web" {
                    return;
                }
                let role = source.focus_role();
                self.native_focus_active.store(false, Ordering::Release);
                self.focus.sync_current(role);
                if let (Some(layout), Some(adapter)) = (self.layout, &mut self.accesskit) {
                    adapter.update_if_active(|| build_accessibility_tree(role, layout));
                }
            }
            // Tab/Shift+Tabはブラウザ既定の移動やhost側の修飾キー推測では
            // なく、WebView側スクリプトが中継した実イベントに従う。
            WebMessage::TabForward => {
                let role = self.focus.tab_forward();
                self.apply_focus(role, "web-dom");
            }
            WebMessage::TabBackward => {
                let role = self.focus.tab_backward();
                self.apply_focus(role, "web-dom");
            }
            WebMessage::ShortcutEnter => self.observe_shortcut(ShortcutKey::Enter),
            WebMessage::ShortcutEscape => self.observe_shortcut(ShortcutKey::Escape),
            WebMessage::ShortcutSpace => self.observe_shortcut(ShortcutKey::Space),
            WebMessage::CompositionStart => {
                self.composing_source = Some(source);
                self.shortcut_sink
                    .set_composition(CompositionState::Composing);
            }
            WebMessage::CompositionUpdate => self.shortcut_sink.record_composition_update(),
            WebMessage::CompositionEnd => {
                self.composing_source = None;
                self.shortcut_sink.set_composition(CompositionState::Idle);
            }
            WebMessage::FocusRequest { target, epoch } => {
                match self.focus.request_focus(target, epoch) {
                    Ok(role) => self.apply_focus(role, "web-focus-request"),
                    Err(_) => return,
                }
            }
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
            "G0-9 wgpu29 | resize={resize} present={present} readback={} native-drag={native_drag} web={}/{}/{} input={} focus={} ime={} shortcuts={}/{}/{} composition-updates={} fullscreen={}",
            self.counters.readback_count,
            self.counters.web_drag_started,
            self.counters.web_drag_moved,
            self.counters.web_drag_ended,
            self.counters.web_input_events,
            focus_role_wire(self.focus.current()),
            self.ime_owner().map(focus_role_wire).unwrap_or("none"),
            self.shortcut_sink.counters().enter,
            self.shortcut_sink.counters().escape,
            self.shortcut_sink.counters().space,
            self.shortcut_sink.composition_updates(),
            self.fullscreen_active,
        )
    }

    fn publish_report(&self) {
        let minimize = self.minimize_lifecycle.observation();
        let fullscreen = self.fullscreen_lifecycle.observation();
        let focus_observations = self
            .focus_observations
            .iter()
            .map(|observation| {
                json!({
                    "role": focus_role_wire(observation.role),
                    "source": observation.source,
                    "responder_class": observation.responder_class,
                    "responder_family": observation.responder_family,
                    "matches_role": observation.matches_role,
                })
            })
            .collect::<Vec<_>>();
        let automated_focus_pass = self.automated_focus.as_ref().and_then(|check| check.pass);
        let automated_focus_error = self
            .automated_focus
            .as_ref()
            .and_then(|check| check.error.as_deref());
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
            "focus_current": focus_role_wire(self.focus.current()),
            "focus_epoch": self.focus.epoch(),
            "ime_owner": self.ime_owner().map(focus_role_wire),
            "shortcut_enter": self.shortcut_sink.counters().enter,
            "shortcut_escape": self.shortcut_sink.counters().escape,
            "shortcut_space": self.shortcut_sink.counters().space,
            "composition_updates": self.shortcut_sink.composition_updates(),
            "minimized": self.occluded,
            "fullscreen_active": self.fullscreen_active,
            "minimize_lifecycle_requested_focus": minimize.requested_focus.map(focus_role_wire),
            "minimize_lifecycle_requested_ime_owner": minimize
                .requested_ime_owner
                .map(focus_role_wire),
            "minimize_lifecycle_restored_focus": minimize.restored_focus.map(focus_role_wire),
            "minimize_lifecycle_restored_ime_owner": minimize
                .restored_ime_owner
                .map(focus_role_wire),
            "fullscreen_lifecycle_requested_focus": fullscreen
                .requested_focus
                .map(focus_role_wire),
            "fullscreen_lifecycle_requested_ime_owner": fullscreen
                .requested_ime_owner
                .map(focus_role_wire),
            "fullscreen_lifecycle_restored_focus": fullscreen
                .restored_focus
                .map(focus_role_wire),
            "fullscreen_lifecycle_restored_ime_owner": fullscreen
                .restored_ime_owner
                .map(focus_role_wire),
            "focus_observations": focus_observations,
            "automated_focus_pass": automated_focus_pass,
            "automated_focus_error": automated_focus_error,
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

fn focus_role_wire(role: FocusRole) -> &'static str {
    match role {
        FocusRole::NativeStage => "native-stage",
        FocusRole::WebLeft => "web-left",
        FocusRole::NativeTimeline => "native-timeline",
        FocusRole::WebRight => "web-right",
    }
}

impl ApplicationHandler<UserEvent> for State {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = Window::default_attributes()
            .with_title("G0-9 wgpu29 starting")
            .with_inner_size(LogicalSize::new(INITIAL_WIDTH, INITIAL_HEIGHT))
            .with_visible(false);
        let window = Arc::new(event_loop.create_window(attributes).expect("host window"));
        let accesskit = accesskit_winit::Adapter::with_event_loop_proxy(
            event_loop,
            &window,
            self.proxy.clone(),
        );
        window.set_visible(true);
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
            &left_html(),
        );
        let right_webview = make_webview(
            &window,
            self.proxy.clone(),
            "right",
            Rect {
                position: WebPosition::new(layout.logical_width - RIGHT_WEBVIEW_WIDTH, 0.0).into(),
                size: WebSize::new(RIGHT_WEBVIEW_WIDTH, layout.logical_height).into(),
            },
            &right_html(),
        );

        self.window = Some(window);
        self.left_webview = Some(left_webview);
        self.right_webview = Some(right_webview);
        self.gfx = Some(gfx);
        self.layout = Some(layout);
        self.accesskit = Some(accesskit);
        // ここではepoch 1を設定するのみで、各WebViewへの配信は`ready`受信後。
        self.set_layout_epoch(1);
        self.publish_report();
        self.window.as_ref().unwrap().request_redraw();
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::WebMessage(message) => self.consume_web_message(&message),
            UserEvent::NativeTab { backward } => {
                let role = if backward {
                    self.focus.tab_backward()
                } else {
                    self.focus.tab_forward()
                };
                self.apply_focus(role, "native-monitor");
                self.publish_report();
            }
            UserEvent::Accesskit(event) => match event.window_event {
                accesskit_winit::WindowEvent::InitialTreeRequested => {
                    let focus = self.focus.current();
                    if let (Some(layout), Some(adapter)) = (self.layout, &mut self.accesskit) {
                        adapter.update_if_active(|| build_accessibility_tree(focus, layout));
                    }
                }
                accesskit_winit::WindowEvent::ActionRequested(request) => {
                    if request.action == accesskit::Action::Focus {
                        let target = match request.target_node {
                            AX_STAGE => Some(FocusRole::NativeStage),
                            AX_TIMELINE => Some(FocusRole::NativeTimeline),
                            _ => None,
                        };
                        if let Some(target) = target {
                            let epoch = self.focus.epoch();
                            if let Ok(role) = self.focus.request_focus(target, epoch) {
                                self.apply_focus(role, "accesskit");
                                self.publish_report();
                            }
                        }
                    }
                }
                accesskit_winit::WindowEvent::AccessibilityDeactivated => {}
            },
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(window) = self.window.clone() {
            if let Some(adapter) = &mut self.accesskit {
                adapter.process_event(&window, &event);
            }
        }
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
                // フルスクリーン切替はresizeを伴うため、`Occluded`ではなく
                // ここでF11開始分の遷移を確定させる。
                if let Some(window) = &self.window {
                    let now_fullscreen = window.fullscreen().is_some();
                    if now_fullscreen != self.fullscreen_active {
                        let was_fullscreen = self.fullscreen_active;
                        self.fullscreen_active = now_fullscreen;
                        if was_fullscreen && !now_fullscreen {
                            self.fullscreen_lifecycle
                                .record_restored(self.focus.current(), self.ime_owner());
                        }
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
                self.cursor_y = position.y;
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
                    if let Some(layout) = self.layout {
                        if !layout.cursor_is_over_webview(self.cursor_x) {
                            self.native_drag_active = true;
                            self.counters.native_drag_crossed_webview = false;
                            self.counters.native_drag_released = false;
                            self.sync_native_click_focus(layout);
                        }
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
            WindowEvent::ModifiersChanged(modifiers) => {
                self.shift_held = modifiers.state().contains(ModifiersState::SHIFT);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed && !event.repeat {
                    match event.logical_key {
                        #[cfg(not(target_os = "macos"))]
                        Key::Named(NamedKey::Tab) => {
                            let role = if self.shift_held {
                                self.focus.tab_backward()
                            } else {
                                self.focus.tab_forward()
                            };
                            self.apply_focus(role, "winit");
                            self.publish_report();
                        }
                        Key::Named(NamedKey::Enter) => self.observe_shortcut(ShortcutKey::Enter),
                        Key::Named(NamedKey::Escape) => self.observe_shortcut(ShortcutKey::Escape),
                        Key::Named(NamedKey::Space) => self.observe_shortcut(ShortcutKey::Space),
                        // CU-0G04的な注入ではなくwinit標準APIのみで遷移を
                        // 起こす。F11はフルスクリーンなので`Resized`側で、
                        // F9は実際に確定した`Occluded`側で観測を確定する。
                        Key::Named(NamedKey::F11) => {
                            if let Some(window) = self.window.clone() {
                                let entering = window.fullscreen().is_none();
                                if entering {
                                    self.fullscreen_lifecycle.record_pre_transition(
                                        self.focus.current(),
                                        self.ime_owner(),
                                    );
                                }
                                let target = if entering {
                                    Some(Fullscreen::Borderless(None))
                                } else {
                                    None
                                };
                                window.set_fullscreen(target);
                                self.publish_report();
                            }
                        }
                        Key::Named(NamedKey::F9) => {
                            if let Some(window) = self.window.clone() {
                                // pre_transitionの記録は`Occluded`側で
                                // `is_minimized()`により実際の最小化が
                                // 確認できてから行う。
                                window.set_minimized(true);
                                self.publish_report();
                            }
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::Occluded(occluded) => {
                // `Occluded(true)`は他ウィンドウに覆われた場合など最小化以外
                // でも起きるため、`is_minimized()`で実際の最小化を確認できた
                // ときだけ最小化として記録する。
                let confirmed_minimized = occluded
                    && self
                        .window
                        .as_ref()
                        .and_then(|window| window.is_minimized())
                        .unwrap_or(false);
                if confirmed_minimized && !self.occluded {
                    self.minimize_lifecycle
                        .record_pre_transition(self.focus.current(), self.ime_owner());
                    self.occluded = true;
                } else if !occluded && self.occluded {
                    self.minimize_lifecycle
                        .record_restored(self.focus.current(), self.ime_owner());
                    self.occluded = false;
                }
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
        self.advance_automated_focus(event_loop, now);
    }
}

impl State {
    fn fail_automated_focus(&mut self, event_loop: &ActiveEventLoop, error: String) {
        if let Some(check) = &mut self.automated_focus {
            check.pass = Some(false);
            check.error = Some(error);
        }
        self.publish_report();
        event_loop.exit();
    }

    fn advance_automated_focus(&mut self, event_loop: &ActiveEventLoop, now: Instant) {
        let Some(check) = self.automated_focus.as_ref() else {
            return;
        };
        if check.pass.is_some() {
            return;
        }
        if now >= check.deadline {
            self.fail_automated_focus(event_loop, "focus E2E timeout".to_owned());
            return;
        }
        if self.resize_requests < self.resize_target || !self.left_ready || !self.right_ready {
            event_loop.set_control_flow(ControlFlow::WaitUntil(now + Duration::from_millis(25)));
            return;
        }
        if !check.started {
            self.focus.sync_current(FocusRole::NativeStage);
            self.apply_focus(FocusRole::NativeStage, "automated-start");
            let check = self.automated_focus.as_mut().unwrap();
            check.started = true;
            check.next_action = now + Duration::from_millis(150);
            let next_action = check.next_action;
            self.publish_report();
            event_loop.set_control_flow(ControlFlow::WaitUntil(next_action));
            return;
        }

        let next_action = check.next_action;
        if now < next_action {
            event_loop.set_control_flow(ControlFlow::WaitUntil(next_action));
            return;
        }

        if check.awaiting_result {
            let step = check.step;
            let expected_role = AUTOMATED_FOCUS_STEPS[step];
            let expected_source = if step % 2 == 0 {
                "native-monitor"
            } else {
                "web-dom"
            };
            let observed = self.focus_observations.last();
            let valid = self.focus.current() == expected_role
                && observed.is_some_and(|observation| {
                    observation.role == expected_role
                        && observation.source == expected_source
                        && observation.matches_role
                });
            if !valid {
                let actual_source = observed
                    .map(|observation| observation.source)
                    .unwrap_or("none");
                self.fail_automated_focus(
                    event_loop,
                    format!(
                        "step {step}: expected {}/{expected_source}, got {}/{}",
                        focus_role_wire(expected_role),
                        focus_role_wire(self.focus.current()),
                        actual_source
                    ),
                );
                return;
            }

            let check = self.automated_focus.as_mut().unwrap();
            check.step += 1;
            check.awaiting_result = false;
            if check.step == AUTOMATED_FOCUS_STEPS.len() {
                check.pass = Some(true);
                self.publish_report();
                event_loop.exit();
                return;
            }
            check.next_action = now + Duration::from_millis(100);
            event_loop.set_control_flow(ControlFlow::WaitUntil(check.next_action));
            return;
        }

        let backward = check.step >= 4;
        if let Err(error) = post_tab_to_current_process(backward) {
            self.fail_automated_focus(event_loop, error.to_owned());
            return;
        }
        let check = self.automated_focus.as_mut().unwrap();
        check.awaiting_result = true;
        check.next_action = now + Duration::from_millis(350);
        event_loop.set_control_flow(ControlFlow::WaitUntil(check.next_action));
    }
}

fn focus_role_family(role: FocusRole) -> &'static str {
    match role {
        FocusRole::NativeStage | FocusRole::NativeTimeline => "native",
        FocusRole::WebLeft | FocusRole::WebRight => "web",
    }
}

#[cfg(target_os = "macos")]
fn actual_first_responder() -> (String, &'static str) {
    let Some(mtm) = MainThreadMarker::new() else {
        return ("not-main-thread".to_owned(), "unknown");
    };
    let app = NSApplication::sharedApplication(mtm);
    let Some(window) = app.keyWindow() else {
        return ("no-key-window".to_owned(), "unknown");
    };
    let Some(responder) = window.firstResponder() else {
        return ("none".to_owned(), "unknown");
    };
    let class_name = responder.class().name().to_owned();
    let family = if class_name.contains("WK") || class_name.contains("Web") {
        "web"
    } else {
        "native"
    };
    (class_name, family)
}

#[cfg(not(target_os = "macos"))]
fn actual_first_responder() -> (String, &'static str) {
    ("unsupported".to_owned(), "unknown")
}

#[cfg(target_os = "macos")]
fn post_tab_to_current_process(backward: bool) -> Result<(), &'static str> {
    let source = CGEventSource::new(CGEventSourceStateID::Private)
        .map_err(|_| "CGEventSource creation failed")?;
    let down = CGEvent::new_keyboard_event(source.clone(), KeyCode::TAB, true)
        .map_err(|_| "Tab key-down creation failed")?;
    let up = CGEvent::new_keyboard_event(source, KeyCode::TAB, false)
        .map_err(|_| "Tab key-up creation failed")?;
    if backward {
        down.set_flags(CGEventFlags::CGEventFlagShift);
        up.set_flags(CGEventFlags::CGEventFlagShift);
    }
    let pid = std::process::id() as i32;
    down.post_to_pid(pid);
    up.post_to_pid(pid);
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn post_tab_to_current_process(_backward: bool) -> Result<(), &'static str> {
    Err("macOSだけがfocus E2Eを実行できる")
}

#[cfg(target_os = "macos")]
fn install_native_tab_monitor(
    proxy: EventLoopProxy<UserEvent>,
    native_focus_active: Arc<AtomicBool>,
) -> Option<Retained<AnyObject>> {
    let handler = RcBlock::new(move |event: std::ptr::NonNull<NSEvent>| {
        let event = unsafe { event.as_ref() };
        let direction = unsafe {
            native_tab_direction(event.keyCode(), event.isARepeat(), event.modifierFlags())
        };
        if native_focus_active.load(Ordering::Acquire) && direction.is_some() {
            let backward = direction.unwrap();
            let _ = proxy.send_event(UserEvent::NativeTab { backward });
            std::ptr::null_mut()
        } else {
            event as *const NSEvent as *mut NSEvent
        }
    });
    unsafe { NSEvent::addLocalMonitorForEventsMatchingMask_handler(NSEventMask::KeyDown, &handler) }
}

#[cfg(target_os = "macos")]
fn native_tab_direction(
    key_code: u16,
    is_repeat: bool,
    modifiers: NSEventModifierFlags,
) -> Option<bool> {
    let disallowed = NSEventModifierFlags::NSEventModifierFlagControl
        | NSEventModifierFlags::NSEventModifierFlagOption
        | NSEventModifierFlags::NSEventModifierFlagCommand
        | NSEventModifierFlags::NSEventModifierFlagNumericPad
        | NSEventModifierFlags::NSEventModifierFlagHelp
        | NSEventModifierFlags::NSEventModifierFlagFunction;
    if key_code != 48 || is_repeat || modifiers.intersects(disallowed) {
        return None;
    }
    Some(modifiers.contains(NSEventModifierFlags::NSEventModifierFlagShift))
}

#[cfg(target_os = "macos")]
impl Drop for State {
    fn drop(&mut self) {
        if let Some(monitor) = self.native_tab_monitor.take() {
            unsafe { NSEvent::removeMonitor(&monitor) };
        }
    }
}

fn make_webview(
    window: &Window,
    proxy: EventLoopProxy<UserEvent>,
    role: &'static str,
    bounds: Rect,
    html: &str,
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

/// ブラウザ既定のtab順序やキー処理に暗黙に消費させず、host側の単一
/// FocusCoordinator/ShortcutSinkへ型付きIPCで中継する。両WebViewの
/// 編集可能inputを含むdocument全体でcomposition/focusinを観測する。
const RELAY_SCRIPT: &str = r#"
window.ipc.postMessage('ready');
document.addEventListener('focusin', () => window.ipc.postMessage('focus-in'));
document.addEventListener('compositionstart', () => window.ipc.postMessage('composition-start'));
document.addEventListener('compositionupdate', () => window.ipc.postMessage('composition-update'));
document.addEventListener('compositionend', () => window.ipc.postMessage('composition-end'));
document.addEventListener('keydown', (event) => {
  if (event.key === 'Tab') {
    if (event.metaKey || event.ctrlKey || event.altKey) { return; }
    event.preventDefault();
    window.ipc.postMessage(event.shiftKey ? 'tab-backward' : 'tab-forward');
    return;
  }
  if (event.isComposing || event.keyCode === 229) { return; }
  if (event.key === 'Enter') { window.ipc.postMessage('shortcut-enter'); return; }
  if (event.key === 'Escape') { window.ipc.postMessage('shortcut-escape'); return; }
  if (event.key === ' ') { window.ipc.postMessage('shortcut-space'); }
}, true);
"#;

const LEFT_HTML_BODY: &str = r#"<!doctype html><html><head><meta charset="utf-8"><style>
html,body{margin:0;height:100%;background:#25272e;color:#f5f6f8;font:15px -apple-system,sans-serif}
main{padding:18px}input,button{font:inherit;margin:4px 0;width:100%;box-sizing:border-box}
#drag{margin-top:18px;padding:28px 8px;border:2px solid #6fa8ff;border-radius:8px;text-align:center;touch-action:none}
</style></head><body><main><h2>Browser WebView</h2><label>Asset search (IME-capable)<input aria-label="Asset search" value="cloud"></label>
<button onclick="window.ipc.postMessage('focus-request:web-right:'+(window.__motoliiEpoch||0))">Focus Inspector</button>
<button onclick="window.ipc.postMessage('focus-request:native-timeline:'+(window.__motoliiEpoch||0))">Request Timeline Focus</button>
<div id="drag" role="button" tabindex="0" aria-label="Drag asset to native Stage">Drag asset → Stage</div>
<p id="status">opaque child view</p></main><script>
const drag=document.querySelector('#drag');let moves=0;
drag.addEventListener('pointerdown',e=>{moves=0;drag.setPointerCapture(e.pointerId);window.ipc.postMessage('drag-start')});
drag.addEventListener('pointermove',e=>{if(e.buttons&&moves++<4)window.ipc.postMessage('drag-move')});
drag.addEventListener('pointerup',()=>window.ipc.postMessage('drag-end'));
const input=document.querySelector('input');
input.addEventListener('input',()=>window.ipc.postMessage('input'));
</script>"#;

const RIGHT_HTML_BODY: &str = r#"<!doctype html><html><head><meta charset="utf-8"><style>
html,body{margin:0;height:100%;background:#292b32;color:#f5f6f8;font:15px -apple-system,sans-serif}
main{padding:18px}input,button{font:inherit;margin:4px 0;width:100%;box-sizing:border-box}
</style></head><body><main><h2>Inspector WebView</h2><label>Opacity<input aria-label="Opacity" value="100%"></label>
<button onclick="window.ipc.postMessage('input')">Apply</button><p>same React-kit boundary</p></main>"#;

fn left_html() -> String {
    format!("{LEFT_HTML_BODY}<script>{RELAY_SCRIPT}</script></body></html>")
}

fn right_html() -> String {
    format!("{RIGHT_HTML_BODY}<script>{RELAY_SCRIPT}</script></body></html>")
}

fn main() {
    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .expect("event loop");
    let proxy = event_loop.create_proxy();
    let mut state = State::new(proxy);
    event_loop.run_app(&mut state).expect("surface host");
    if state
        .automated_focus
        .as_ref()
        .is_some_and(|check| check.pass != Some(true))
    {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_click_above_timeline_boundary_resolves_to_stage() {
        let layout = SurfaceLayout::try_new(2400, 1600, 2.0).unwrap();
        assert_eq!(native_click_role(0.0, layout), FocusRole::NativeStage);
        assert_eq!(
            native_click_role(f64::from(layout.timeline_y) - 1.0, layout),
            FocusRole::NativeStage
        );
    }

    #[test]
    fn native_click_on_or_below_timeline_boundary_resolves_to_timeline() {
        let layout = SurfaceLayout::try_new(2400, 1600, 2.0).unwrap();
        assert_eq!(
            native_click_role(f64::from(layout.timeline_y), layout),
            FocusRole::NativeTimeline
        );
        assert_eq!(
            native_click_role(f64::from(layout.timeline_y) + 50.0, layout),
            FocusRole::NativeTimeline
        );
    }

    #[test]
    fn ax_node_id_and_ax_role_cover_every_projection_node() {
        let layout = SurfaceLayout::try_new(2400, 1600, 2.0).unwrap();
        let projection = AccessibilityProjection::project(SemanticCounts::default(), layout);
        for node in projection.nodes() {
            // AxNodeId/AxRoleの各バリアントが未知値としてpanicせず既知の
            // accesskit型へ写像できることを確認する。
            let _ = ax_node_id(node.id);
            let _ = ax_role(node.role);
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn native_tab_monitor_accepts_plain_and_shift_tab() {
        assert_eq!(
            native_tab_direction(48, false, NSEventModifierFlags::empty()),
            Some(false)
        );
        assert_eq!(
            native_tab_direction(48, false, NSEventModifierFlags::NSEventModifierFlagShift),
            Some(true)
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn native_tab_monitor_rejects_repeat_and_other_keys() {
        assert_eq!(
            native_tab_direction(48, true, NSEventModifierFlags::empty()),
            None
        );
        assert_eq!(
            native_tab_direction(36, false, NSEventModifierFlags::empty()),
            None
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn native_tab_monitor_does_not_steal_modified_tab() {
        for modifier in [
            NSEventModifierFlags::NSEventModifierFlagCommand,
            NSEventModifierFlags::NSEventModifierFlagControl,
            NSEventModifierFlags::NSEventModifierFlagOption,
        ] {
            assert_eq!(native_tab_direction(48, false, modifier), None);
        }
    }

    #[test]
    fn web_tab_relay_does_not_steal_non_shift_modifiers() {
        assert!(RELAY_SCRIPT.contains("event.metaKey || event.ctrlKey || event.altKey"));
    }
}
