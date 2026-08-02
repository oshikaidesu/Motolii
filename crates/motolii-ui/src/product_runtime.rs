//! macOS通常project sessionのdirect native Surface Host。

use std::borrow::Cow;
use std::sync::Arc;
use std::time::{Duration, Instant};

use motolii_core::{CanonicalPoint, Quality, RationalTime};
use motolii_doc::{Command, EffectId, EvaluationTime, LayerId};
use motolii_eval::DataTracks;
use motolii_gpu::GpuCtx;
use winit::dpi::LogicalSize;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::window::Window;

use crate::app::canonical_drop_from_ndc;
use crate::browser_host::BrowserPlaceIntent;
use crate::browser_host_runtime::{
    BrowserFocusTarget, BrowserHostRuntime, BrowserHostRuntimeError, BrowserLifecycleEvent,
};
use crate::document_edit_runtime::{
    AddPositionKeyRequest, AttachEffectRequest, DocumentEditDispatchError, DocumentEditQueue,
    DocumentEditRuntime, DocumentEditRuntimeError, MoveClipRequest, PlaceRectangleRequest,
    PublishedDocument, SetPositionKeyInterpRequest, TrimClipEdge, TrimClipRequest,
};
use crate::easing_popup_runtime::{
    EasingPopupError, EasingPopupInput, EasingPopupOpen, EasingPopupOutcome, EasingPopupRuntime,
};
use crate::host_pointer_capture::{HostPointerCancel, HostPointerCandidate, HostPointerPress};
use crate::inspector_host_runtime::{
    resolve_effect_param_preview_command, InspectorGestureTerminal, InspectorGestureTerminalCause,
    InspectorHostRuntime, InspectorHostRuntimeError,
};
use crate::layout_authority::LayoutAuthority;
use crate::native_host_layout::{LogicalRect, NativeHostLayout, PhysicalRect};
use crate::native_timeline_renderer::{
    key_tools_logical_rect, timeline_ruler_logical_rect, timeline_time_surface_logical_rect,
    NativeTimelineRenderer, NativeTimelineRendererError, TimelineIntervalPreview,
    TimelinePrepareInput,
};
use crate::render_worker::{
    RenderGeneration, RenderRequest, RenderWorker, RenderWorkerClient, RenderWorkerError,
};
use crate::stage_chrome_host_runtime::{
    StageChromeHostRuntime, StageChromeHostRuntimeError, StageChromeIntentError,
};
use crate::static_preview::{bootstrap_frame_desc, prepare_in_setup_worker, StaticPreview};
use crate::timeline_projection::{
    project_position_interval, project_timeline, TimelineHit, TimelineMetrics, TimelineProjection,
    TimelineProjectionError, TimelineViewport,
};
use crate::timeline_tools_host_runtime::{TimelineToolsHostRuntime, TimelineToolsHostRuntimeError};
use crate::{
    builtin_command_registry, resolve_keymap, AsciiKey, Binding, BuiltinKeymap, CommandId,
    CommandIdError, CommandRegistry, CommandRegistryError, EffectiveTrigger, Gesture, InputPhase,
    InputRouter, InputRouterError, KeyToken, KeymapDelta, KeymapResolution, Modifier,
    ModifierError, Modifiers, NormalizedInput, PlatformBindingConstraints, PlatformCommandModifier,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum ProductEvent {
    Wake,
    BrowserLifecycle(BrowserLifecycleEvent),
}

pub(crate) fn run(document_runtime: DocumentEditRuntime) -> Result<(), ProductRuntimeError> {
    let startup = Instant::now();
    crate::ui_numeric_trace::emit(format_args!("kind=startup phase=begin elapsed_ms=0.000"));
    let document = document_runtime.snapshot();
    let gpu_started = Instant::now();
    let (gpu, parts) = GpuCtx::new_for_ui()?;
    crate::ui_numeric_trace::emit(format_args!(
        "kind=startup phase=gpu-ready phase_ms={:.3} elapsed_ms={:.3}",
        elapsed_ms(gpu_started),
        elapsed_ms(startup),
    ));
    let parts = ProductGpuParts {
        instance: parts.instance,
        adapter: parts.adapter,
        device: parts.device,
    };
    let gpu = Arc::new(gpu);
    let preview_started = Instant::now();
    let preview = Arc::new(prepare_in_setup_worker(
        Arc::clone(&gpu),
        Arc::clone(&document),
        bootstrap_frame_desc()?,
    )?);
    crate::ui_numeric_trace::emit(format_args!(
        "kind=startup phase=preview-ready phase_ms={:.3} elapsed_ms={:.3} width={} height={}",
        elapsed_ms(preview_started),
        elapsed_ms(startup),
        preview.slot().desc().width,
        preview.slot().desc().height,
    ));
    let mut render_worker = RenderWorker::spawn(Arc::clone(&gpu))?;
    let render_client = render_worker.client();
    let event_loop = EventLoop::<ProductEvent>::with_user_event().build()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let proxy = event_loop.create_proxy();
    let wake_proxy = proxy.clone();
    render_client.register_repaint_signal(Arc::new(move || {
        let _ = wake_proxy.send_event(ProductEvent::Wake);
    }))?;
    let render_request_template = RenderRequest {
        document,
        data_tracks: Arc::new(DataTracks::new()),
        evaluation_time: EvaluationTime::new(RationalTime::ZERO),
        desc: bootstrap_frame_desc()?,
        quality: Quality::DRAFT,
    };
    let mut app = ProductApp::new(
        gpu,
        parts,
        preview,
        document_runtime,
        render_client,
        render_request_template,
        proxy,
    )?;
    crate::ui_numeric_trace::emit(format_args!(
        "kind=startup phase=event-loop-ready elapsed_ms={:.3}",
        elapsed_ms(startup),
    ));
    let run_result = event_loop.run_app(&mut app);
    render_worker.close();
    let join_result = render_worker.join();
    crate::ui_numeric_trace::emit(format_args!(
        "kind=shutdown phase=event-loop-exit elapsed_ms={:.3}",
        elapsed_ms(startup),
    ));
    run_result?;
    join_result?;
    if let Some(error) = app.failure {
        return Err(ProductRuntimeError::Runtime(error));
    }
    Ok(())
}

// Linux/Windows CIでもmacOS製品runtime全体をcompileし、private境界の接続欠落を検出する。
#[cfg(not(target_os = "macos"))]
fn compile_product_runtime() {
    let _: fn(DocumentEditRuntime) -> Result<(), ProductRuntimeError> = run;
    let _ = BrowserLifecycleEvent::ProcessTerminated { instance_epoch: 0 };
}

#[cfg(not(target_os = "macos"))]
const _: fn() = compile_product_runtime;

pub(crate) struct ProductApp {
    easing_popup: Option<EasingPopupRuntime>,
    // surface → WebView → Windowの順にdropし、AppKit backingを先に失わない。
    gfx: Option<ProductSurface>,
    browser: Option<BrowserHostRuntime>,
    inspector: Option<InspectorHostRuntime>,
    stage_chrome: Option<StageChromeHostRuntime>,
    timeline_tools: Option<TimelineToolsHostRuntime>,
    window: Option<Arc<Window>>,
    gpu: Arc<GpuCtx>,
    parts: Option<ProductGpuParts>,
    preview: Arc<StaticPreview>,
    render_client: RenderWorkerClient,
    render_request_template: RenderRequest,
    stage_projection: ProductStageProjection,
    timeline_projection: ProductTimelineProjection,
    displayed_camera: motolii_core::CompCamera,
    document_runtime: DocumentEditRuntime,
    document_queue: DocumentEditQueue,
    input_router: InputRouter,
    command_keymap: KeymapResolution,
    primary: Option<motolii_doc::LayerId>,
    active_effect_use: Option<EffectId>,
    selected_position_key: Option<(LayerId, motolii_doc::KeyframeId)>,
    playhead: RationalTime,
    projection_generation: u64,
    current_document: Arc<motolii_doc::Document>,
    proxy: EventLoopProxy<ProductEvent>,
    layout_authority: LayoutAuthority,
    layout: Option<NativeHostLayout>,
    next_layout_epoch: u64,
    browser_source: Option<BrowserPlaceIntent>,
    browser_lifecycle: Option<BrowserLifecycleCoordinator>,
    browser_focus_target: BrowserFocusTarget,
    next_place_generation: u64,
    active_place: Option<BrowserPlaceIntent>,
    place_preview: PlacePreviewPhase,
    terminal_admission: PlaceTerminalAdmission,
    terminal_delivery: PlaceTerminalDelivery,
    candidate_terminal: Option<ClassifiedPlaceTerminal>,
    admitted_terminal: Option<ClassifiedPlaceTerminal>,
    pending_stage_drop: Option<PendingStageDrop>,
    surface_retry_at: Option<Instant>,
    failure: Option<String>,
    pending_inspector_commit: Option<InspectorGestureTerminal>,
    next_interval_generation: u64,
    active_interval: Option<ActiveIntervalGesture>,
    interval_preview: Option<TimelineIntervalPreview>,
    horizontal_start: RationalTime,
    vertical_offset: f64,
    cursor: [f64; 2],
}

#[derive(Debug, Clone)]
struct PendingStageDrop {
    source: BrowserPlaceIntent,
    generation: u64,
    layout_epoch: u64,
    ndc: [f64; 2],
}

const INTERVAL_HANDLE_LOGICAL_PX: f64 = 8.0;
const SNAP_DISTANCE_LOGICAL_PX: f64 = 8.0;
const TIMELINE_PIXELS_PER_SECOND: f64 = 80.0;
const TIMELINE_ROW_HEIGHT: f64 = 34.0;

fn line_delta_to_logical(delta: [f64; 2]) -> [f64; 2] {
    [
        if delta[0].is_finite() {
            delta[0] * TIMELINE_ROW_HEIGHT
        } else {
            0.0
        },
        if delta[1].is_finite() {
            delta[1] * TIMELINE_ROW_HEIGHT
        } else {
            0.0
        },
    ]
}

fn pixel_delta_to_logical(delta: [f64; 2], scale_factor: f64) -> [f64; 2] {
    let scale = if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    };
    [
        if delta[0].is_finite() {
            delta[0] / scale
        } else {
            0.0
        },
        if delta[1].is_finite() {
            delta[1] / scale
        } else {
            0.0
        },
    ]
}

fn clamp_horizontal_start_value(
    horizontal_start: f64,
    viewport_start: f64,
    duration: f64,
    visible_span: f64,
) -> f64 {
    let max_start = (viewport_start + duration - visible_span).max(viewport_start);
    horizontal_start.max(viewport_start).min(max_start)
}

fn clamp_vertical_offset_value(
    vertical_offset: f64,
    total_height: f64,
    surface_height: f64,
) -> f64 {
    let max_offset = (total_height - surface_height).max(0.0);
    if vertical_offset.is_finite() {
        vertical_offset.clamp(0.0, max_offset)
    } else {
        0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IntervalGestureKind {
    Move,
    TrimIn,
    TrimOut,
}

#[derive(Debug, Clone, Copy)]
struct IntervalPressTarget {
    layer: LayerId,
    kind: IntervalGestureKind,
    start: RationalTime,
    end: RationalTime,
}

#[derive(Debug, Clone, Copy)]
struct ActiveIntervalGesture {
    generation: u64,
    target: IntervalPressTarget,
    grab_time: RationalTime,
    layout_epoch: u64,
    projection_generation: u64,
}

#[derive(Debug, Default)]
struct ProductStageProjection {
    last_displayed_generation: Option<RenderGeneration>,
}

impl ProductStageProjection {
    fn accepts(
        &self,
        result_generation: RenderGeneration,
        latest_accepted_generation: Option<RenderGeneration>,
    ) -> bool {
        Some(result_generation) == latest_accepted_generation
            && self
                .last_displayed_generation
                .is_none_or(|displayed| result_generation > displayed)
    }

    fn commit(&mut self, generation: RenderGeneration) {
        self.last_displayed_generation = Some(generation);
    }
}

#[derive(Debug, Clone)]
struct ProductTimelineProjection {
    projection: TimelineProjection,
    band_span: f64,
    viewport: TimelineViewport,
    horizontal_start: RationalTime,
    vertical_offset: f64,
}

impl ProductTimelineProjection {
    fn from_document(document: &motolii_doc::Document) -> Result<Self, TimelineProjectionError> {
        let duration_seconds = document.composition.duration.as_seconds_f64();
        let viewport = TimelineViewport {
            start: RationalTime::ZERO,
            end: document.composition.duration,
        };
        let projection = project_timeline(
            document,
            &TimelineMetrics {
                band_height: 1.0,
                units_per_second: duration_seconds.recip(),
                key_half_extent: 1.0,
            },
            &viewport,
        )?;
        let band_span = projection
            .bars()
            .iter()
            .map(|bar| bar.y_bottom)
            .fold(1.0, f64::max);
        Ok(Self {
            projection,
            band_span,
            viewport,
            horizontal_start: RationalTime::ZERO,
            vertical_offset: 0.0,
        })
    }

    fn set_horizontal_start(&mut self, horizontal_start: RationalTime) {
        self.horizontal_start = horizontal_start;
    }

    fn set_vertical_offset(&mut self, vertical_offset: f64) {
        self.vertical_offset = vertical_offset;
    }

    fn visible_span_seconds(&self, layout: NativeHostLayout) -> Option<f64> {
        let surface = timeline_time_surface_logical_rect(layout)?;
        let span = surface.width / TIMELINE_PIXELS_PER_SECOND;
        (span > 0.0 && span.is_finite()).then_some(span)
    }

    fn rational_from_seconds(seconds: f64) -> Option<RationalTime> {
        if !seconds.is_finite() {
            return None;
        }
        RationalTime::try_from_decimal_str(&format!("{seconds:.9}")).ok()
    }

    fn hit_test(&self, position: [f64; 2], layout: NativeHostLayout) -> Option<TimelineHit> {
        let time_surface = timeline_time_surface_logical_rect(layout)?;
        if !time_surface.contains(position) {
            return None;
        }
        let x_local = position[0] - time_surface.x;
        let y_local = ((position[1] - time_surface.y) + self.vertical_offset) / TIMELINE_ROW_HEIGHT;
        let visible_seconds = self.visible_span_seconds(layout)?;
        let viewport_seconds = self
            .viewport
            .end
            .try_sub(self.viewport.start)
            .ok()?
            .as_seconds_f64();
        if viewport_seconds <= 0.0 {
            return None;
        }
        let local_seconds = (x_local / TIMELINE_PIXELS_PER_SECOND).clamp(0.0, visible_seconds);
        let x = (self.horizontal_start.as_seconds_f64() + local_seconds) / viewport_seconds;
        if !x.is_finite() {
            return None;
        }
        Some(self.projection.hit_test(x, y_local))
    }

    fn time_at(&self, position: [f64; 2], layout: NativeHostLayout) -> Option<RationalTime> {
        let surface = timeline_time_surface_logical_rect(layout)?;
        self.time_at_surface(position, surface)
    }

    fn ruler_time_at(&self, position: [f64; 2], layout: NativeHostLayout) -> Option<RationalTime> {
        let surface = timeline_ruler_logical_rect(layout)?;
        self.time_at_surface(position, surface)
    }

    fn time_at_surface(
        &self,
        position: [f64; 2],
        surface: crate::native_host_layout::LogicalRect,
    ) -> Option<RationalTime> {
        if !surface.contains(position) || !surface.width.is_finite() || surface.width <= 0.0 {
            return None;
        }
        let span = self.visible_span_seconds_from_surface(surface)?;
        let normalized = (position[0] - surface.x) / surface.width;
        let offset_seconds = (span * normalized).max(0.0).min(span);
        let offset = Self::rational_from_seconds(offset_seconds)?;
        let mut time = self.horizontal_start.try_add(offset).ok()?;
        if time > self.viewport.end {
            time = self.viewport.end;
        }
        if time < self.viewport.start {
            time = self.viewport.start;
        }
        Some(time)
    }

    fn visible_span_seconds_from_surface(
        &self,
        surface: crate::native_host_layout::LogicalRect,
    ) -> Option<f64> {
        let span = surface.width / TIMELINE_PIXELS_PER_SECOND;
        (span > 0.0 && span.is_finite()).then_some(span)
    }

    fn logical_x(&self, time: RationalTime, layout: NativeHostLayout) -> Option<f64> {
        self.unclamped_logical_x(time, layout)
    }

    fn unclamped_logical_x(&self, time: RationalTime, layout: NativeHostLayout) -> Option<f64> {
        let surface = timeline_time_surface_logical_rect(layout)?;
        let span = self.visible_span_seconds(layout)?;
        let viewport_seconds = self
            .viewport
            .end
            .try_sub(self.viewport.start)
            .ok()?
            .as_seconds_f64();
        if viewport_seconds <= 0.0 || !span.is_finite() || span <= 0.0 {
            return None;
        }
        let local_seconds = time.as_seconds_f64() - self.horizontal_start.as_seconds_f64();
        let x = surface.x + local_seconds * TIMELINE_PIXELS_PER_SECOND;
        x.is_finite().then_some(x)
    }

    fn interval_press_target(
        &self,
        position: [f64; 2],
        layout: NativeHostLayout,
    ) -> Option<IntervalPressTarget> {
        let TimelineHit::Bar { layer } = self.hit_test(position, layout)? else {
            return None;
        };
        let bar = self
            .projection
            .bars()
            .iter()
            .find(|bar| bar.layer == layer)?;
        let start_x = self.unclamped_logical_x(bar.start, layout)?;
        let end_x = self.unclamped_logical_x(bar.end, layout)?;
        let start_distance = (position[0] - start_x).abs();
        let end_distance = (position[0] - end_x).abs();
        let kind = if start_distance <= INTERVAL_HANDLE_LOGICAL_PX && start_distance <= end_distance
        {
            IntervalGestureKind::TrimIn
        } else if end_distance <= INTERVAL_HANDLE_LOGICAL_PX {
            IntervalGestureKind::TrimOut
        } else {
            IntervalGestureKind::Move
        };
        Some(IntervalPressTarget {
            layer,
            kind,
            start: bar.start,
            end: bar.end,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IntervalEdge {
    In,
    Out,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SnapTargetKind {
    OtherClip,
    Frame,
}

include!("product_interaction_runtime.rs");

impl ProductApp {
    fn new(
        gpu: Arc<GpuCtx>,
        parts: ProductGpuParts,
        preview: Arc<StaticPreview>,
        document_runtime: DocumentEditRuntime,
        render_client: RenderWorkerClient,
        render_request_template: RenderRequest,
        proxy: EventLoopProxy<ProductEvent>,
    ) -> Result<Self, ProductRuntimeError> {
        let displayed_camera = preview.camera();
        let current_document = document_runtime.snapshot();
        let timeline_projection = ProductTimelineProjection::from_document(&current_document)?;
        let command_registry = builtin_command_registry()?;
        let command_keymap = product_command_keymap(&command_registry)?;
        Ok(Self {
            easing_popup: None,
            gfx: None,
            browser: None,
            inspector: None,
            stage_chrome: None,
            timeline_tools: None,
            window: None,
            gpu,
            parts: Some(parts),
            preview,
            render_client,
            render_request_template,
            stage_projection: ProductStageProjection::default(),
            timeline_projection,
            displayed_camera,
            current_document,
            document_runtime,
            document_queue: DocumentEditQueue::default(),
            input_router: InputRouter::new(command_registry),
            command_keymap,
            primary: None,
            active_effect_use: None,
            selected_position_key: None,
            playhead: RationalTime::ZERO,
            projection_generation: 0,
            proxy,
            layout_authority: LayoutAuthority::built_in()?,
            layout: None,
            next_layout_epoch: 1,
            browser_source: None,
            browser_lifecycle: None,
            browser_focus_target: BrowserFocusTarget::Browser,
            next_place_generation: 1,
            active_place: None,
            place_preview: PlacePreviewPhase::default(),
            terminal_admission: PlaceTerminalAdmission::default(),
            terminal_delivery: PlaceTerminalDelivery::default(),
            candidate_terminal: None,
            admitted_terminal: None,
            pending_stage_drop: None,
            surface_retry_at: None,
            failure: None,
            pending_inspector_commit: None,
            next_interval_generation: 1,
            active_interval: None,
            interval_preview: None,
            horizontal_start: RationalTime::ZERO,
            vertical_offset: 0.0,
            cursor: [0.0, 0.0],
        })
    }

    pub(crate) fn take_pending_inspector_commit(&mut self) -> Option<InspectorGestureTerminal> {
        let taken = self.pending_inspector_commit.take();
        if taken.is_some() {
            let _ = self.proxy.send_event(ProductEvent::Wake);
        }
        taken
    }

    pub(crate) fn initialize(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), ProductRuntimeError> {
        let initialize_started = Instant::now();
        let window = Arc::new(
            event_loop.create_window(
                Window::default_attributes()
                    .with_title("Motolii")
                    .with_inner_size(LogicalSize::new(1200.0, 800.0))
                    .with_visible(false),
            )?,
        );
        crate::ui_numeric_trace::emit(format_args!(
            "kind=startup phase=window-created phase_ms={:.3}",
            elapsed_ms(initialize_started),
        ));
        let browser_started = Instant::now();
        let initial_instance_epoch = BrowserHostRuntime::fresh_instance_epoch()?;
        let browser_source = BrowserHostRuntime::built_in_rectangle_source(initial_instance_epoch);
        let browser = build_browser_runtime(
            &window,
            initial_instance_epoch,
            browser_source.clone(),
            self.proxy.clone(),
        )?;
        crate::ui_numeric_trace::emit(format_args!(
            "kind=startup phase=browser-created phase_ms={:.3} instance_epoch={}",
            elapsed_ms(browser_started),
            initial_instance_epoch,
        ));
        let inspector_started = Instant::now();
        let inspector = InspectorHostRuntime::new(
            &window,
            &self.current_document,
            self.primary,
            self.active_effect_use,
        )?;
        crate::ui_numeric_trace::emit(format_args!(
            "kind=startup phase=inspector-created phase_ms={:.3}",
            elapsed_ms(inspector_started),
        ));
        let stage_chrome_started = Instant::now();
        let stage_chrome = StageChromeHostRuntime::new(
            &window,
            &self.current_document,
            self.selected_position_key,
            self.projection_generation,
            self.playhead,
        )?;
        crate::ui_numeric_trace::emit(format_args!(
            "kind=startup phase=stage-chrome-created phase_ms={:.3}",
            elapsed_ms(stage_chrome_started),
        ));
        let timeline_tools_started = Instant::now();
        let timeline_tools = TimelineToolsHostRuntime::new(
            &window,
            self.timeline_projection.projection.bars().len(),
            self.timeline_projection.projection.keys().len(),
        )?;
        crate::ui_numeric_trace::emit(format_args!(
            "kind=startup phase=timeline-tools-created phase_ms={:.3}",
            elapsed_ms(timeline_tools_started),
        ));
        // WebViewの初期navigationをhidden中に開始し、白い未初期化面を見せない。
        // Metal sublayerはon-screenのcontent viewへ結び付けてからSurfaceを作る。
        window.set_visible(true);
        let parts = self
            .parts
            .take()
            .ok_or(ProductRuntimeError::AlreadyInitialized)?;
        let gfx = ProductSurface::new(&window, parts, &self.gpu, &self.preview)?;
        crate::ui_numeric_trace::emit(format_args!(
            "kind=startup phase=surface-created elapsed_ms={:.3}",
            elapsed_ms(initialize_started),
        ));
        self.browser_lifecycle = Some(BrowserLifecycleCoordinator::new(initial_instance_epoch)?);
        self.browser_source = Some(browser_source);
        self.window = Some(window);
        self.browser = Some(browser);
        let wake_proxy = self.proxy.clone();
        inspector
            .register_wake(Arc::new(move || {
                let _ = wake_proxy.send_event(ProductEvent::Wake);
            }))
            .map_err(InspectorHostRuntimeError::from)
            .map_err(ProductRuntimeError::from)?;
        let wake_proxy = self.proxy.clone();
        stage_chrome
            .register_wake(Arc::new(move || {
                let _ = wake_proxy.send_event(ProductEvent::Wake);
            }))
            .map_err(StageChromeHostRuntimeError::from)
            .map_err(ProductRuntimeError::from)?;
        self.inspector = Some(inspector);
        self.stage_chrome = Some(stage_chrome);
        self.timeline_tools = Some(timeline_tools);
        self.gfx = Some(gfx);
        self.update_layout()?;
        crate::ui_numeric_trace::emit(format_args!(
            "kind=startup phase=initialized elapsed_ms={:.3}",
            elapsed_ms(initialize_started),
        ));
        if let Some(window) = &self.window {
            window.request_redraw();
        }
        Ok(())
    }

    pub(crate) fn update_layout(&mut self) -> Result<(), ProductRuntimeError> {
        let Some(window) = &self.window else {
            return Ok(());
        };
        let size = window.inner_size();
        let Some(layout) = NativeHostLayout::try_new(
            self.next_layout_epoch,
            size.width,
            size.height,
            window.scale_factor(),
            self.preview.slot().desc(),
            self.layout_authority.intent(),
        )?
        else {
            self.layout = None;
            return Ok(());
        };
        self.next_layout_epoch = self
            .next_layout_epoch
            .checked_add(1)
            .ok_or(ProductRuntimeError::LayoutEpochExhausted)?;
        if let Some(browser) = &self.browser {
            browser.set_bounds(layout.epoch, layout.browser)?;
        }
        if let Some(inspector) = &mut self.inspector {
            inspector.set_bounds(layout.epoch, layout.inspector)?;
        }
        if let Some(stage_chrome) = &mut self.stage_chrome {
            stage_chrome.set_bounds(layout.epoch, layout.stage_header, layout.stage_transport)?;
        }
        if let Some(timeline_tools) = &mut self.timeline_tools {
            timeline_tools.set_bounds(layout.epoch, key_tools_logical_rect(layout))?;
        }
        let hidden_rect = LogicalRect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        };
        let browser = layout.browser.unwrap_or(hidden_rect);
        let inspector = layout.inspector.unwrap_or(hidden_rect);
        let timeline = layout.timeline.unwrap_or(hidden_rect);
        crate::ui_numeric_trace::emit(format_args!(
            "kind=layout epoch={} physical_width={} physical_height={} scale_factor={:.3} \
             browser_visible={} browser_x={:.3} browser_y={:.3} browser_width={:.3} browser_height={:.3} \
             stage_header_x={:.3} stage_header_y={:.3} stage_header_width={:.3} stage_header_height={:.3} \
             stage_viewport_x={:.3} stage_viewport_y={:.3} stage_viewport_width={:.3} stage_viewport_height={:.3} \
             stage_x={:.3} stage_y={:.3} stage_width={:.3} stage_height={:.3} \
             stage_transport_x={:.3} stage_transport_y={:.3} stage_transport_width={:.3} stage_transport_height={:.3} \
             inspector_visible={} inspector_x={:.3} inspector_y={:.3} inspector_width={:.3} inspector_height={:.3} \
             timeline_visible={} timeline_x={:.3} timeline_y={:.3} timeline_width={:.3} timeline_height={:.3}",
            layout.epoch,
            size.width,
            size.height,
            window.scale_factor(),
            layout.browser.is_some(),
            browser.x,
            browser.y,
            browser.width,
            browser.height,
            layout.stage_header.x,
            layout.stage_header.y,
            layout.stage_header.width,
            layout.stage_header.height,
            layout.stage_viewport.x,
            layout.stage_viewport.y,
            layout.stage_viewport.width,
            layout.stage_viewport.height,
            layout.stage.x,
            layout.stage.y,
            layout.stage.width,
            layout.stage.height,
            layout.stage_transport.x,
            layout.stage_transport.y,
            layout.stage_transport.width,
            layout.stage_transport.height,
            layout.inspector.is_some(),
            inspector.x,
            inspector.y,
            inspector.width,
            inspector.height,
            layout.timeline.is_some(),
            timeline.x,
            timeline.y,
            timeline.width,
            timeline.height,
        ));
        self.layout = Some(layout);
        self.synchronize_timeline_viewport(Some(layout));
        Ok(())
    }

    fn synchronize_timeline_viewport(&mut self, layout: Option<NativeHostLayout>) {
        let Some(layout) = layout else {
            return;
        };
        self.horizontal_start = self.clamp_horizontal_start(layout, self.horizontal_start);
        self.vertical_offset = self.clamp_vertical_offset(layout, self.vertical_offset);
        self.timeline_projection
            .set_horizontal_start(self.horizontal_start);
        self.timeline_projection
            .set_vertical_offset(self.vertical_offset);
        if let Some(gfx) = &mut self.gfx {
            gfx.native_timeline_renderer
                .set_timeline_viewport(self.horizontal_start, self.vertical_offset);
        }
    }

    fn clamp_horizontal_start(
        &self,
        layout: NativeHostLayout,
        horizontal_start: RationalTime,
    ) -> RationalTime {
        let Some(visible_span) = self.timeline_projection.visible_span_seconds(layout) else {
            return self.timeline_projection.viewport.start;
        };
        let duration = self
            .timeline_projection
            .viewport
            .end
            .try_sub(self.timeline_projection.viewport.start)
            .ok()
            .map(|time| time.as_seconds_f64());
        let Some(duration) = duration else {
            return self.timeline_projection.viewport.start;
        };
        let viewport_start = self.timeline_projection.viewport.start.as_seconds_f64();
        let next = clamp_horizontal_start_value(
            horizontal_start.as_seconds_f64(),
            viewport_start,
            duration,
            visible_span,
        );
        ProductTimelineProjection::rational_from_seconds(next)
            .unwrap_or(self.timeline_projection.viewport.start)
    }

    fn clamp_vertical_offset(&self, layout: NativeHostLayout, vertical_offset: f64) -> f64 {
        let Some(timeline_surface) = timeline_time_surface_logical_rect(layout) else {
            return 0.0;
        };
        if !timeline_surface.height.is_finite() || timeline_surface.height <= 0.0 {
            return 0.0;
        }
        let total_height = (self.timeline_projection.band_span * TIMELINE_ROW_HEIGHT).max(1.0);
        clamp_vertical_offset_value(vertical_offset, total_height, timeline_surface.height)
    }

    pub(crate) fn set_cursor(&mut self, cursor: [f64; 2]) {
        self.cursor = [
            if cursor[0].is_finite() {
                cursor[0]
            } else {
                0.0
            },
            if cursor[1].is_finite() {
                cursor[1]
            } else {
                0.0
            },
        ];
    }

    pub(crate) fn scroll_timeline_line_delta(&mut self, delta: [f64; 2]) {
        self.scroll_timeline_by_pixel_delta(line_delta_to_logical(delta));
    }

    pub(crate) fn scroll_timeline_pixel_delta(&mut self, delta: [f64; 2], scale_factor: f64) {
        self.scroll_timeline_by_pixel_delta(pixel_delta_to_logical(delta, scale_factor));
    }

    pub(crate) fn window_scale_factor(&self) -> Option<f64> {
        self.window.as_ref().map(|window| window.scale_factor())
    }

    fn scroll_timeline_by_pixel_delta(&mut self, delta: [f64; 2]) {
        let Some(layout) = self.layout else {
            return;
        };
        let dx = if delta[0].is_finite() { delta[0] } else { 0.0 };
        let dy = if delta[1].is_finite() { delta[1] } else { 0.0 };
        let mut horizontal = self.horizontal_start;
        let Some(dx_time) = ProductTimelineProjection::rational_from_seconds(
            (dx / TIMELINE_PIXELS_PER_SECOND).abs(),
        ) else {
            self.request_redraw();
            return;
        };
        if dx >= 0.0 {
            horizontal = horizontal.try_sub(dx_time).unwrap_or(horizontal);
        } else {
            horizontal = horizontal.try_add(dx_time).unwrap_or(horizontal);
        }
        let Some(surface) = timeline_time_surface_logical_rect(layout) else {
            return;
        };
        if !surface.contains(self.cursor) {
            return;
        }
        self.horizontal_start = self.clamp_horizontal_start(layout, horizontal);
        self.vertical_offset -= dy;
        self.vertical_offset = self.clamp_vertical_offset(layout, self.vertical_offset);
        self.synchronize_timeline_viewport(Some(layout));
        self.request_redraw();
    }

    pub(crate) fn poll_browser(&mut self, event_loop: &ActiveEventLoop) {
        if self
            .surface_retry_at
            .is_some_and(|retry_at| Instant::now() >= retry_at)
        {
            self.surface_retry_at = None;
            self.request_redraw();
        }
        let attach_effect = {
            let Some(browser) = &self.browser else {
                return;
            };
            browser.take_attach_effect_intent()
        };
        match attach_effect {
            Ok(Some(intent)) => {
                if let Err(error) = self.process_attach_effect(event_loop, intent.item_id) {
                    return self.fail(event_loop, error);
                }
            }
            Ok(None) => {}
            Err(error) => return self.fail(event_loop, error),
        }
        let Some(browser) = &self.browser else {
            return;
        };
        if let Err(error) = browser.ensure_focus(self.browser_focus_target) {
            return self.fail(event_loop, error);
        }
        if self.active_place.is_none() && self.active_interval.is_none() {
            let generation = self.next_place_generation;
            let Some(next_generation) = generation.checked_add(1) else {
                return self.fail(event_loop, ProductRuntimeError::PlaceGenerationExhausted);
            };
            match browser.take_place_intent(generation) {
                Ok(Some((intent, generation))) => {
                    self.next_place_generation = next_generation;
                    self.browser_focus_target = BrowserFocusTarget::Parent;
                    if let Err(error) = browser.ensure_focus(self.browser_focus_target) {
                        return self.fail(event_loop, error);
                    }
                    if !self.terminal_admission.begin(generation) {
                        return self.fail(
                            event_loop,
                            ProductRuntimeError::PlaceAdmissionGenerationRejected(generation),
                        );
                    }
                    self.place_preview.clear();
                    self.candidate_terminal = None;
                    self.admitted_terminal = None;
                    self.pending_stage_drop = None;
                    self.active_place = Some(intent);
                    crate::ui_numeric_trace::emit(format_args!(
                        "kind=browser-intent generation={} state=armed",
                        generation,
                    ));
                }
                Ok(None) => {}
                Err(error) => return self.fail(event_loop, error),
            }
        }
        if self.active_place.is_none() && self.active_interval.is_none() {
            self.poll_host_input(event_loop);
            if self.active_interval.is_some() {
                event_loop.set_control_flow(ControlFlow::Poll);
            } else {
                self.set_idle_control_flow(event_loop);
            }
            return;
        }
        if self.active_interval.is_some() {
            self.poll_interval_gesture(event_loop);
            return;
        }
        match browser.poll_pointer_candidate() {
            Ok(Some(HostPointerCandidate::Moved {
                generation,
                position,
            })) => {
                if let (Some(source), Some(layout)) = (&self.active_place, self.layout) {
                    let stage_ndc = layout.stage_ndc(position);
                    let changed = self.place_preview.latest.as_ref().is_none_or(|latest| {
                        latest.generation != generation
                            || latest.layout_epoch != layout.epoch
                            || latest.stage_ndc != stage_ndc
                    });
                    if changed {
                        crate::ui_numeric_trace::emit(format_args!(
                            "kind=place-move generation={} layout_epoch={} logical_x={:.3} \
                         logical_y={:.3} stage_x={:.3} stage_y={:.3} stage_width={:.3} \
                         stage_height={:.3} ndc_x={} ndc_y={}",
                            generation,
                            layout.epoch,
                            position[0],
                            position[1],
                            layout.stage.x,
                            layout.stage.y,
                            layout.stage.width,
                            layout.stage.height,
                            OptionalNumber(stage_ndc.map(|ndc| ndc[0])),
                            OptionalNumber(stage_ndc.map(|ndc| ndc[1])),
                        ));
                    }
                    self.place_preview
                        .deliver(source, generation, position, layout);
                    self.request_redraw();
                }
                event_loop.set_control_flow(ControlFlow::Poll);
            }
            Ok(Some(HostPointerCandidate::Released {
                generation,
                position,
            })) => {
                self.place_preview.clear();
                let Some(source) = self.active_place.take() else {
                    self.set_idle_control_flow(event_loop);
                    return;
                };
                let Some(layout) = self.layout else {
                    return self.fail(
                        event_loop,
                        ProductRuntimeError::PlaceTerminalLayoutUnavailable,
                    );
                };
                let terminal =
                    ClassifiedPlaceTerminal::released(source, generation, position, layout);
                crate::ui_numeric_trace::emit(format_args!(
                    "kind=place-release generation={} layout_epoch={} logical_x={:.3} \
                     logical_y={:.3} stage_x={:.3} stage_y={:.3} stage_width={:.3} \
                     stage_height={:.3} ndc_x={} ndc_y={} cause={:?}",
                    generation,
                    layout.epoch,
                    position[0],
                    position[1],
                    layout.stage.x,
                    layout.stage.y,
                    layout.stage.width,
                    layout.stage.height,
                    OptionalNumber(terminal.stage_ndc.map(|ndc| ndc[0])),
                    OptionalNumber(terminal.stage_ndc.map(|ndc| ndc[1])),
                    terminal.cause,
                ));
                if self.terminal_admission.admit(&terminal) {
                    self.pending_stage_drop = self.terminal_delivery.deliver(&terminal);
                    self.admitted_terminal = Some(terminal.clone());
                }
                self.candidate_terminal = Some(terminal);
                self.set_idle_control_flow(event_loop);
            }
            Ok(Some(HostPointerCandidate::Cancelled { generation, reason })) => {
                self.place_preview.clear();
                if let Some(source) = self.active_place.take() {
                    let terminal = ClassifiedPlaceTerminal::cancelled(source, generation, reason);
                    if self.terminal_admission.admit(&terminal) {
                        self.admitted_terminal = Some(terminal.clone());
                    }
                    self.candidate_terminal = Some(terminal);
                }
                self.set_idle_control_flow(event_loop);
            }
            Ok(None) => match browser.pointer_capture_is_active() {
                Ok(true) => event_loop.set_control_flow(ControlFlow::Poll),
                Ok(false) => self.set_idle_control_flow(event_loop),
                Err(error) => self.fail(event_loop, error),
            },
            Err(error) => self.fail(event_loop, error),
        }
        if let Some(drop) = &self.pending_stage_drop {
            let _ = (&drop.source, drop.generation, drop.layout_epoch, drop.ndc);
        }
        if let Some(terminal) = &self.candidate_terminal {
            let _ = (
                &terminal.source,
                terminal.generation,
                terminal.cause,
                terminal.layout_epoch,
                terminal.stage_ndc,
            );
        }
        if let Some(terminal) = &self.admitted_terminal {
            let _ = (
                &terminal.source,
                terminal.generation,
                terminal.layout_epoch,
                terminal.stage_ndc,
            );
        }
        if let Some(drop) = self.pending_stage_drop.take() {
            let Some(position) = canonical_drop_from_ndc(self.displayed_camera, drop.ndc) else {
                return self.fail(event_loop, ProductRuntimeError::PlaceCanonicalConversion);
            };
            crate::ui_numeric_trace::emit(format_args!(
                "kind=place-command generation={} layout_epoch={} ndc_x={:.6} ndc_y={:.6} \
                 canonical_x={:.6} canonical_y={:.6}",
                drop.generation,
                drop.layout_epoch,
                drop.ndc[0],
                drop.ndc[1],
                position[0],
                position[1],
            ));
            self.document_queue
                .push_place_rectangle(PlaceRectangleRequest {
                    position,
                    playhead: RationalTime::ZERO,
                });
            match self.document_runtime.process_next(
                &mut self.document_queue,
                self.primary,
                self.projection_generation,
            ) {
                Ok(Some(published)) => {
                    trace_document_publish("place", &published);
                    self.reconcile_active_effect_use(&published);
                    self.current_document = published.snapshot;
                    self.primary = published.primary;
                    self.projection_generation = published.projection_generation;
                    if let Some(inspector) = &self.inspector {
                        if let Err(error) = inspector.publish(
                            &self.current_document,
                            self.primary,
                            self.active_effect_use,
                            self.projection_generation,
                        ) {
                            return self.fail(event_loop, error);
                        }
                    }
                    self.timeline_projection =
                        match ProductTimelineProjection::from_document(&self.current_document) {
                            Ok(projection) => projection,
                            Err(error) => return self.fail(event_loop, error),
                        };
                    self.synchronize_timeline_viewport(self.layout);
                    trace_timeline_projection(
                        "place",
                        self.projection_generation,
                        &self.timeline_projection,
                    );
                    if let Err(error) = self.publish_timeline_tools() {
                        return self.fail(event_loop, error);
                    }
                    if let Err(error) = self.publish_stage_chrome(false) {
                        return self.fail(event_loop, error);
                    }
                    if let Err(error) = self.submit_stage_projection() {
                        return self.fail(event_loop, error);
                    }
                    self.request_redraw();
                }
                Ok(None) => {}
                Err(error) => self.fail(event_loop, error),
            }
        }
    }

    pub(crate) fn handle_product_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        event: ProductEvent,
    ) {
        match event {
            ProductEvent::Wake => {
                if let Err(error) = self.process_add_position_key_intents(event_loop) {
                    self.fail(event_loop, error);
                    return;
                }
                if let Err(error) = self.process_stage_chrome_intents(event_loop) {
                    self.fail(event_loop, error);
                    return;
                }
                if let Err(error) = self.process_pending_inspector_commit(event_loop) {
                    self.fail(event_loop, error);
                    return;
                }
                if let Err(error) = self.process_inspector_gestures() {
                    self.fail(event_loop, error);
                    return;
                }
                if let Err(error) = self.drain_stage_projection() {
                    self.fail(event_loop, error);
                    return;
                }
                // AppKit local monitorはwinit event配送後にclickを確定するため、
                // wakeの同じturnでHost inboxまで排出して次の入力を待たせない。
                self.poll_browser(event_loop);
                self.request_redraw();
            }
            ProductEvent::BrowserLifecycle(event) => {
                if let Err(error) = self.handle_browser_lifecycle(event) {
                    self.fail(event_loop, error);
                }
            }
        }
    }

    fn handle_browser_lifecycle(
        &mut self,
        event: BrowserLifecycleEvent,
    ) -> Result<(), ProductRuntimeError> {
        let Some(active_epoch) = self
            .browser
            .as_ref()
            .map(BrowserHostRuntime::instance_epoch)
            .transpose()?
        else {
            return Ok(());
        };
        let decision = self
            .browser_lifecycle
            .as_mut()
            .ok_or(ProductRuntimeError::BrowserLifecycleUnavailable)?
            .observe(active_epoch, event)?;
        crate::ui_numeric_trace::emit(format_args!(
            "kind=browser-lifecycle active_epoch={} event_epoch={} event={:?} decision={:?}",
            active_epoch,
            event.instance_epoch(),
            event,
            decision,
        ));
        match decision {
            BrowserRecoveryDecision::Ignore => Ok(()),
            BrowserRecoveryDecision::Replace { instance_epoch } => {
                self.replace_browser(instance_epoch)
            }
            BrowserRecoveryDecision::Degrade => {
                self.active_place = None;
                self.active_interval = None;
                self.interval_preview = None;
                self.place_preview.clear();
                self.terminal_admission.retire_active();
                self.candidate_terminal = None;
                self.admitted_terminal = None;
                self.browser.take();
                Ok(())
            }
        }
    }

    fn replace_browser(&mut self, instance_epoch: u64) -> Result<(), ProductRuntimeError> {
        let window = self
            .window
            .as_ref()
            .ok_or(ProductRuntimeError::BrowserLifecycleUnavailable)?;
        let source = self
            .browser_source
            .clone()
            .ok_or(ProductRuntimeError::BrowserLifecycleUnavailable)?;
        self.active_place = None;
        self.active_interval = None;
        self.interval_preview = None;
        self.place_preview.clear();
        self.terminal_admission.retire_active();
        self.candidate_terminal = None;
        self.admitted_terminal = None;
        self.browser.take();
        let browser = build_browser_runtime(window, instance_epoch, source, self.proxy.clone())?;
        if let Some(layout) = self.layout {
            browser.set_bounds(layout.epoch, layout.browser)?;
        }
        self.browser = Some(browser);
        self.request_redraw();
        Ok(())
    }

    fn set_idle_control_flow(&self, event_loop: &ActiveEventLoop) {
        match self.surface_retry_at {
            Some(retry_at) => event_loop.set_control_flow(ControlFlow::WaitUntil(retry_at)),
            None => event_loop.set_control_flow(ControlFlow::Wait),
        }
    }

    pub(crate) fn poll_host_input(&mut self, event_loop: &ActiveEventLoop) {
        if self.active_place.is_some() || self.active_interval.is_some() {
            return;
        }
        let command = {
            let Some(browser) = &self.browser else {
                return;
            };
            browser.poll_host_command()
        };
        match command {
            Ok(Some(trigger)) => self.handle_history_trigger(event_loop, trigger),
            Ok(None) => {}
            Err(error) => return self.fail(event_loop, error),
        }
        loop {
            let press = {
                let Some(browser) = &self.browser else {
                    return;
                };
                browser.poll_host_press()
            };
            match press {
                Ok(Some(press)) => {
                    if self.begin_interval_gesture(event_loop, press) {
                        return;
                    }
                }
                Ok(None) => break,
                Err(error) => {
                    self.fail(event_loop, error);
                    return;
                }
            }
        }
        loop {
            let click = {
                let Some(browser) = &self.browser else {
                    return;
                };
                browser.poll_host_click()
            };
            match click {
                Ok(Some(click)) => self.handle_timeline_click(event_loop, click.position),
                Ok(None) => break,
                Err(error) => {
                    self.fail(event_loop, error);
                    break;
                }
            }
        }
    }

    fn begin_interval_gesture(
        &mut self,
        event_loop: &ActiveEventLoop,
        press: HostPointerPress,
    ) -> bool {
        let Some(layout) = self.layout else {
            return false;
        };
        let Some(target) = self
            .timeline_projection
            .interval_press_target(press.position, layout)
        else {
            return false;
        };
        let Some(grab_time) = self.timeline_projection.time_at(press.position, layout) else {
            return false;
        };
        self.handle_timeline_click(event_loop, press.position);
        if self.failure.is_some() || self.primary != Some(target.layer) {
            return false;
        }
        let generation = self.next_interval_generation;
        let Some(next_generation) = generation.checked_add(1) else {
            self.fail(event_loop, ProductRuntimeError::IntervalGenerationExhausted);
            return false;
        };
        let armed = match &self.browser {
            Some(browser) => browser.arm_pointer_capture(generation),
            None => return false,
        };
        match armed {
            Ok(true) => {
                self.next_interval_generation = next_generation;
                self.active_interval = Some(ActiveIntervalGesture {
                    generation,
                    target,
                    grab_time,
                    layout_epoch: layout.epoch,
                    projection_generation: self.projection_generation,
                });
                self.interval_preview = Some(TimelineIntervalPreview {
                    layer: target.layer,
                    start: target.start,
                    end: target.end,
                });
                self.request_redraw();
                true
            }
            Ok(false) => false,
            Err(error) => {
                self.fail(event_loop, error);
                false
            }
        }
    }

    fn poll_interval_gesture(&mut self, event_loop: &ActiveEventLoop) {
        let candidate = match &self.browser {
            Some(browser) => browser.poll_pointer_candidate(),
            None => return,
        };
        match candidate {
            Ok(Some(HostPointerCandidate::Moved {
                generation,
                position,
            })) => {
                let Some(active) = self.active_interval else {
                    return;
                };
                let Some(layout) = self.layout else {
                    self.cancel_interval_gesture(event_loop);
                    return;
                };
                if generation != active.generation
                    || layout.epoch != active.layout_epoch
                    || self.projection_generation != active.projection_generation
                {
                    self.cancel_interval_gesture(event_loop);
                    return;
                }
                self.interval_preview = self
                    .timeline_projection
                    .time_at(position, layout)
                    .and_then(|pointer_time| {
                        interval_preview_candidate(
                            active,
                            pointer_time,
                            layout,
                            &self.timeline_projection,
                            &self.current_document,
                        )
                    });
                self.request_redraw();
                event_loop.set_control_flow(ControlFlow::Poll);
            }
            Ok(Some(HostPointerCandidate::Released {
                generation,
                position,
            })) => {
                let Some(active) = self.active_interval.take() else {
                    self.interval_preview = None;
                    self.set_idle_control_flow(event_loop);
                    return;
                };
                self.interval_preview = None;
                let Some(layout) = self.layout else {
                    self.set_idle_control_flow(event_loop);
                    return;
                };
                if generation != active.generation
                    || layout.epoch != active.layout_epoch
                    || self.projection_generation != active.projection_generation
                {
                    self.request_redraw();
                    self.set_idle_control_flow(event_loop);
                    return;
                }
                let Some(preview) =
                    self.timeline_projection
                        .time_at(position, layout)
                        .and_then(|pointer_time| {
                            interval_preview_candidate(
                                active,
                                pointer_time,
                                layout,
                                &self.timeline_projection,
                                &self.current_document,
                            )
                        })
                else {
                    self.request_redraw();
                    self.set_idle_control_flow(event_loop);
                    return;
                };
                self.commit_interval_gesture(event_loop, active, preview);
                self.set_idle_control_flow(event_loop);
            }
            Ok(Some(HostPointerCandidate::Cancelled { generation, .. })) => {
                if self
                    .active_interval
                    .is_some_and(|active| active.generation == generation)
                {
                    self.cancel_interval_gesture(event_loop);
                }
                self.set_idle_control_flow(event_loop);
            }
            Ok(None) => match &self.browser {
                Some(browser) => match browser.pointer_capture_is_active() {
                    Ok(true) => event_loop.set_control_flow(ControlFlow::Poll),
                    Ok(false) => {
                        self.cancel_interval_gesture(event_loop);
                        self.set_idle_control_flow(event_loop);
                    }
                    Err(error) => self.fail(event_loop, error),
                },
                None => self.cancel_interval_gesture(event_loop),
            },
            Err(error) => self.fail(event_loop, error),
        }
    }

    fn cancel_interval_gesture(&mut self, _event_loop: &ActiveEventLoop) {
        clear_interval_gesture(&mut self.active_interval, &mut self.interval_preview);
        self.request_redraw();
    }

    fn commit_interval_gesture(
        &mut self,
        event_loop: &ActiveEventLoop,
        active: ActiveIntervalGesture,
        preview: TimelineIntervalPreview,
    ) {
        match active.target.kind {
            IntervalGestureKind::Move => self.document_queue.push_move_clip(MoveClipRequest {
                layer_id: active.target.layer,
                start: preview.start,
            }),
            IntervalGestureKind::TrimIn => self.document_queue.push_trim_clip(TrimClipRequest {
                layer_id: active.target.layer,
                edge: TrimClipEdge::In,
                time: preview.start,
            }),
            IntervalGestureKind::TrimOut => self.document_queue.push_trim_clip(TrimClipRequest {
                layer_id: active.target.layer,
                edge: TrimClipEdge::Out,
                time: preview.end,
            }),
        }
        match self.document_runtime.process_next(
            &mut self.document_queue,
            self.primary,
            self.projection_generation,
        ) {
            Ok(Some(published)) => {
                self.adopt_full_publish(event_loop, published, "timeline-interval")
            }
            Ok(None) => self.request_redraw(),
            Err(DocumentEditRuntimeError::Command(error)) => {
                crate::ui_numeric_trace::emit(format_args!(
                    "kind=timeline-interval state=rejected error={error}"
                ));
                self.request_redraw();
            }
            Err(error) => self.fail(event_loop, error),
        }
    }

    fn handle_timeline_click(&mut self, event_loop: &ActiveEventLoop, position: [f64; 2]) {
        let Some(layout) = self.layout else {
            return;
        };
        if let Some(playhead) = self.timeline_projection.ruler_time_at(position, layout) {
            self.playhead = playhead;
            crate::ui_numeric_trace::emit(format_args!(
                "kind=playhead route=ruler-click layout_epoch={} time={:?}",
                layout.epoch, self.playhead,
            ));
            if let Err(error) = self.publish_stage_chrome(false) {
                return self.fail(event_loop, error);
            }
            if let Err(error) = self.submit_stage_projection() {
                return self.fail(event_loop, error);
            }
            self.request_redraw();
            return;
        }
        let hit = self.timeline_projection.hit_test(position, layout);
        crate::ui_numeric_trace::emit(format_args!(
            "kind=timeline-hit layout_epoch={} logical_x={:.3} logical_y={:.3} hit={:?}",
            layout.epoch, position[0], position[1], hit,
        ));
        let Some(hit) = hit else {
            return;
        };
        self.selected_position_key = match hit {
            TimelineHit::Key { layer, key } => Some((layer, key)),
            TimelineHit::Bar { .. } | TimelineHit::None => None,
        };
        self.browser_focus_target = BrowserFocusTarget::Parent;
        if let Some(browser) = &self.browser {
            if let Err(error) = browser.ensure_focus(self.browser_focus_target) {
                return self.fail(event_loop, error);
            }
        }
        match hit {
            TimelineHit::Key { layer, .. } | TimelineHit::Bar { layer } => {
                self.document_queue.push_replace_primary(layer);
            }
            TimelineHit::None => self.document_queue.push_clear_primary(),
        }
        match self.document_runtime.process_next(
            &mut self.document_queue,
            self.primary,
            self.projection_generation,
        ) {
            Ok(Some(published)) => {
                trace_document_publish("timeline-selection", &published);
                self.reconcile_active_effect_use(&published);
                self.current_document = published.snapshot;
                self.primary = published.primary;
                self.projection_generation = published.projection_generation;
                if let Some(inspector) = &self.inspector {
                    if let Err(error) = inspector.publish(
                        &self.current_document,
                        self.primary,
                        self.active_effect_use,
                        self.projection_generation,
                    ) {
                        return self.fail(event_loop, error);
                    }
                }
                if let Err(error) = self.publish_stage_chrome(false) {
                    return self.fail(event_loop, error);
                }
                self.request_redraw();
            }
            Ok(None) => {
                if let Err(error) = self.publish_stage_chrome(false) {
                    self.fail(event_loop, error);
                }
            }
            Err(error) => self.fail(event_loop, error),
        }
    }

    fn inspector_render_request(&self, document: Arc<motolii_doc::Document>) -> RenderRequest {
        RenderRequest {
            document,
            data_tracks: Arc::clone(&self.render_request_template.data_tracks),
            evaluation_time: EvaluationTime::new(self.playhead),
            desc: self.render_request_template.desc,
            quality: self.render_request_template.quality,
        }
    }

    fn submit_inspector_baseline(&self) -> Result<RenderGeneration, ProductRuntimeError> {
        let generation = self
            .render_client
            .submit(self.inspector_render_request(Arc::clone(&self.current_document)))?;
        crate::ui_numeric_trace::emit(format_args!(
            "kind=inspector-submit route=baseline generation={} projection_generation={}",
            generation.get(),
            self.projection_generation,
        ));
        Ok(generation)
    }

    fn submit_inspector_preview(
        &self,
        document: Arc<motolii_doc::Document>,
        preview_command: Command,
    ) -> Result<RenderGeneration, ProductRuntimeError> {
        let generation = self
            .render_client
            .submit_preview(self.inspector_render_request(document), preview_command)?;
        crate::ui_numeric_trace::emit(format_args!(
            "kind=inspector-submit route=preview generation={} projection_generation={}",
            generation.get(),
            self.projection_generation,
        ));
        Ok(generation)
    }

    fn process_inspector_gestures(&mut self) -> Result<(), ProductRuntimeError> {
        if self.pending_inspector_commit.is_some() {
            return Ok(());
        }
        let Some(inspector) = &self.inspector else {
            return Ok(());
        };
        let document = Arc::clone(&self.current_document);
        let mut needs_baseline = false;
        let mut pending_commit = None;
        loop {
            let Some(terminal) = inspector
                .take_terminal()
                .map_err(InspectorHostRuntimeError::from)
                .map_err(ProductRuntimeError::from)?
            else {
                break;
            };
            match terminal.cause {
                InspectorGestureTerminalCause::Cancel => {
                    needs_baseline = true;
                }
                InspectorGestureTerminalCause::Commit(value) => {
                    match resolve_effect_param_preview_command(&document, &terminal.identity, value)
                    {
                        Some(command) => {
                            pending_commit = Some((terminal, command));
                            break;
                        }
                        None => needs_baseline = true,
                    }
                }
            }
        }
        if let Some((terminal, command)) = pending_commit {
            self.pending_inspector_commit = Some(terminal);
            if needs_baseline {
                self.submit_inspector_baseline()?;
            }
            self.submit_inspector_preview(Arc::clone(&document), command)?;
            return Ok(());
        }
        let update = inspector
            .take_latest_update()
            .map_err(InspectorHostRuntimeError::from)
            .map_err(ProductRuntimeError::from)?;
        let had_update = update.is_some();
        let preview = update.as_ref().and_then(|update| {
            resolve_effect_param_preview_command(&document, &update.identity, update.value)
        });
        if needs_baseline || (had_update && preview.is_none()) {
            self.submit_inspector_baseline()?;
        }
        let Some(command) = preview else {
            return Ok(());
        };
        self.submit_inspector_preview(document, command)?;
        Ok(())
    }

    fn submit_stage_projection(&self) -> Result<RenderGeneration, ProductRuntimeError> {
        let generation = self
            .render_client
            .submit(self.inspector_render_request(Arc::clone(&self.current_document)))?;
        crate::ui_numeric_trace::emit(format_args!(
            "kind=stage-submit generation={} projection_generation={} width={} height={} quality={:?}",
            generation.get(),
            self.projection_generation,
            self.render_request_template.desc.width,
            self.render_request_template.desc.height,
            self.render_request_template.quality,
        ));
        Ok(generation)
    }

    fn process_pending_inspector_commit(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), ProductRuntimeError> {
        let Some(terminal) = self.take_pending_inspector_commit() else {
            return Ok(());
        };
        crate::ui_numeric_trace::emit(format_args!(
            "kind=inspector-commit session={} sequence={} cause={:?}",
            terminal.session, terminal.sequence, terminal.cause,
        ));
        match terminal.into_set_effect_param_request() {
            Some(request) => {
                self.document_queue.push_set_effect_param(request);
                match self.document_runtime.process_next(
                    &mut self.document_queue,
                    self.primary,
                    self.projection_generation,
                )? {
                    Some(published) => {
                        self.adopt_full_publish(event_loop, published, "inspector-opacity");
                    }
                    None => {
                        self.submit_inspector_baseline()?;
                    }
                }
            }
            None => {
                self.submit_inspector_baseline()?;
            }
        }
        Ok(())
    }

    fn handle_history_trigger(&mut self, event_loop: &ActiveEventLoop, trigger: EffectiveTrigger) {
        let Some(command) = self.command_keymap.get(&trigger).cloned() else {
            return;
        };
        let output = match self.input_router.route(NormalizedInput::Command {
            phase: InputPhase::Press,
            id: command,
        }) {
            Ok(output) => output,
            Err(error) => return self.fail(event_loop, error),
        };
        if let Err(error) = self.document_queue.push_prepared(output, None) {
            return self.fail(event_loop, error);
        }
        match self.document_runtime.process_next(
            &mut self.document_queue,
            self.primary,
            self.projection_generation,
        ) {
            Ok(Some(published)) => self.adopt_history_publish(event_loop, published),
            Ok(None) => {}
            Err(
                DocumentEditRuntimeError::NothingToUndo | DocumentEditRuntimeError::NothingToRedo,
            ) => {}
            Err(error) => self.fail(event_loop, error),
        }
    }

    fn adopt_full_publish(
        &mut self,
        event_loop: &ActiveEventLoop,
        published: PublishedDocument,
        route: &'static str,
    ) {
        trace_document_publish(route, &published);
        self.reconcile_active_effect_use(&published);
        self.current_document = published.snapshot;
        self.primary = published.primary;
        self.projection_generation = published.projection_generation;
        if let Some(inspector) = &self.inspector {
            if let Err(error) = inspector.publish(
                &self.current_document,
                self.primary,
                self.active_effect_use,
                self.projection_generation,
            ) {
                return self.fail(event_loop, error);
            }
        }
        self.timeline_projection =
            match ProductTimelineProjection::from_document(&self.current_document) {
                Ok(projection) => projection,
                Err(error) => return self.fail(event_loop, error),
            };
        self.synchronize_timeline_viewport(self.layout);
        trace_timeline_projection(route, self.projection_generation, &self.timeline_projection);
        if let Err(error) = self.publish_timeline_tools() {
            return self.fail(event_loop, error);
        }
        if let Err(error) = self.publish_stage_chrome(false) {
            return self.fail(event_loop, error);
        }
        if let Err(error) = self.submit_stage_projection() {
            return self.fail(event_loop, error);
        }
        self.request_redraw();
    }

    fn adopt_history_publish(
        &mut self,
        event_loop: &ActiveEventLoop,
        published: PublishedDocument,
    ) {
        self.adopt_full_publish(event_loop, published, "history");
    }

    fn reconcile_active_effect_use(&mut self, published: &PublishedDocument) {
        self.active_effect_use = active_effect_candidate(
            self.primary,
            self.active_effect_use,
            published.primary,
            published.created_effect_use,
            |primary, effect| {
                published
                    .snapshot
                    .find_effect_use(primary, effect)
                    .is_some()
            },
        );
    }

    fn publish_timeline_tools(&self) -> Result<(), ProductRuntimeError> {
        if let Some(timeline_tools) = &self.timeline_tools {
            timeline_tools.publish(
                self.timeline_projection.projection.bars().len(),
                self.timeline_projection.projection.keys().len(),
            )?;
        }
        Ok(())
    }

    fn drain_stage_projection(&mut self) -> Result<(), ProductRuntimeError> {
        let Some(result) = self.render_client.try_take_latest() else {
            return Ok(());
        };
        if !self.stage_projection.accepts(
            result.generation,
            self.render_client.latest_accepted_generation(),
        ) {
            crate::ui_numeric_trace::emit(format_args!(
                "kind=stage-result generation={} state=discarded latest_generation={}",
                result.generation.get(),
                self.render_client
                    .latest_accepted_generation()
                    .map_or(0, RenderGeneration::get),
            ));
            return Ok(());
        }
        let rendered = result.result?;
        self.preview.slot().copy(&self.gpu, &rendered.frame)?;
        self.displayed_camera = rendered.camera;
        self.stage_projection.commit(result.generation);
        crate::ui_numeric_trace::emit(format_args!(
            "kind=stage-result generation={} state=displayed width={} height={}",
            result.generation.get(),
            self.preview.slot().desc().width,
            self.preview.slot().desc().height,
        ));
        Ok(())
    }

    fn publish_stage_chrome(&self, easing_pressed: bool) -> Result<(), ProductRuntimeError> {
        if let Some(stage_chrome) = &self.stage_chrome {
            stage_chrome.publish(
                &self.current_document,
                self.selected_position_key,
                self.projection_generation,
                easing_pressed,
                self.playhead,
            )?;
        }
        Ok(())
    }

    fn process_stage_chrome_intents(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), ProductRuntimeError> {
        loop {
            let request = match &self.stage_chrome {
                Some(stage_chrome) => stage_chrome.take_open_easing()?,
                None => None,
            };
            let Some(request) = request else {
                break;
            };
            crate::ui_numeric_trace::emit(format_args!(
                "kind=easing-popup state=requested layer={} left_key={} right_key={} projection_generation={}",
                request.layer_id.get(),
                request.left_key_id.get(),
                request.right_key_id.get(),
                request.projection_generation,
            ));
            let Some(layout) = self.layout else {
                continue;
            };
            if request.layout_epoch != layout.epoch
                || request.projection_generation != self.projection_generation
            {
                crate::ui_numeric_trace::emit(format_args!(
                    "kind=easing-popup state=stale-open-rejected"
                ));
                continue;
            }
            let Some(interval) = project_position_interval(
                &self.current_document,
                request.layer_id,
                request.left_key_id,
            )
            .filter(|interval| interval.right_key == request.right_key_id) else {
                crate::ui_numeric_trace::emit(format_args!(
                    "kind=easing-popup state=interval-open-rejected"
                ));
                continue;
            };
            let host = self
                .window
                .as_ref()
                .ok_or(ProductRuntimeError::AlreadyInitialized)?;
            let gfx = self
                .gfx
                .as_ref()
                .ok_or(ProductRuntimeError::AlreadyInitialized)?;
            let anchor = crate::easing_popup_model::LogicalRect {
                x: layout.stage_transport.x + request.anchor.x,
                y: layout.stage_transport.y + request.anchor.y,
                width: request.anchor.width,
                height: request.anchor.height,
            };
            self.easing_popup = Some(EasingPopupRuntime::open(EasingPopupOpen {
                event_loop,
                host,
                instance: &gfx.instance,
                adapter: &gfx.adapter,
                gpu: Arc::clone(&self.gpu),
                target: request,
                anchor,
                interp: interval.interp,
            })?);
            self.publish_stage_chrome(true)?;
        }
        Ok(())
    }

    fn process_add_position_key_intents(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), ProductRuntimeError> {
        loop {
            let intent = match &self.inspector {
                Some(inspector) => inspector
                    .take_add_position_key()
                    .map_err(InspectorHostRuntimeError::from)?,
                None => None,
            };
            let Some(intent) = intent else {
                break;
            };
            if self.primary != Some(intent.layer_id)
                || self.projection_generation != intent.projection_generation
            {
                continue;
            }
            self.document_queue
                .push_add_position_key(AddPositionKeyRequest {
                    layer_id: intent.layer_id,
                    playhead: self.playhead,
                });
            match self.document_runtime.process_next(
                &mut self.document_queue,
                self.primary,
                self.projection_generation,
            ) {
                Ok(Some(published)) => {
                    self.adopt_full_publish(event_loop, published, "add-position-key")
                }
                Ok(None) => {}
                Err(error) => return Err(error.into()),
            }
        }
        Ok(())
    }

    pub(crate) fn handle_auxiliary_window_input(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        input: EasingPopupInput,
    ) -> bool {
        if self
            .easing_popup
            .as_ref()
            .is_none_or(|popup| popup.window_id() != window_id)
        {
            return false;
        }
        let outcome = self
            .easing_popup
            .as_mut()
            .expect("matched popup exists")
            .handle_input(input);
        match outcome {
            Ok(EasingPopupOutcome::Handled) => {}
            Ok(EasingPopupOutcome::Closed) => {
                self.easing_popup = None;
                if let Err(error) = self.publish_stage_chrome(false) {
                    self.fail(event_loop, error);
                }
            }
            Ok(EasingPopupOutcome::Commit(interp)) => {
                let target = self
                    .easing_popup
                    .take()
                    .expect("matched popup exists")
                    .target();
                self.document_queue
                    .push_set_position_key_interp(SetPositionKeyInterpRequest {
                        layer_id: target.layer_id,
                        left_key_id: target.left_key_id,
                        interp,
                    });
                match self.document_runtime.process_next(
                    &mut self.document_queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => {
                        self.adopt_full_publish(event_loop, published, "easing-popup")
                    }
                    Ok(None) => {
                        if let Err(error) = self.publish_stage_chrome(false) {
                            self.fail(event_loop, error);
                        }
                    }
                    Err(error) => self.fail(event_loop, error),
                }
            }
            Err(error) => {
                self.easing_popup = None;
                self.fail(event_loop, error);
            }
        }
        true
    }

    pub(crate) fn owns_auxiliary_window(&self, window_id: winit::window::WindowId) -> bool {
        self.easing_popup
            .as_ref()
            .is_some_and(|popup| popup.window_id() == window_id)
    }

    fn process_attach_effect(
        &mut self,
        event_loop: &ActiveEventLoop,
        plugin_id: String,
    ) -> Result<(), ProductRuntimeError> {
        self.document_queue
            .push_attach_effect(AttachEffectRequest { plugin_id });
        if let Some(published) = self.document_runtime.process_next(
            &mut self.document_queue,
            self.primary,
            self.projection_generation,
        )? {
            self.adopt_full_publish(event_loop, published, "attach-effect");
        }
        Ok(())
    }

    pub(crate) fn fail(&mut self, event_loop: &ActiveEventLoop, error: impl std::fmt::Display) {
        crate::ui_numeric_trace::emit(format_args!("kind=failure message={error}"));
        self.failure = Some(error.to_string());
        event_loop.exit();
    }

    pub(crate) fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    pub(crate) fn resize(&mut self, event_loop: &ActiveEventLoop, width: u32, height: u32) {
        crate::ui_numeric_trace::emit(format_args!(
            "kind=window event=resize physical_width={} physical_height={}",
            width, height,
        ));
        if let Some(gfx) = &mut self.gfx {
            gfx.configure(width, height);
        }
        if let Err(error) = self.update_layout() {
            self.fail(event_loop, error);
            return;
        }
        if width > 0 && height > 0 {
            self.render(event_loop);
        }
    }

    pub(crate) fn scale_factor_changed(&mut self, event_loop: &ActiveEventLoop) {
        let Some(window) = &self.window else {
            return;
        };
        let size = window.inner_size();
        crate::ui_numeric_trace::emit(format_args!(
            "kind=window event=scale-factor-changed scale_factor={:.3} physical_width={} physical_height={}",
            window.scale_factor(),
            size.width,
            size.height,
        ));
        self.resize(event_loop, size.width, size.height);
    }

    pub(crate) fn set_occluded(&mut self, occluded: bool) {
        crate::ui_numeric_trace::emit(format_args!(
            "kind=window event=occlusion occluded={occluded}"
        ));
        if let Some(gfx) = &mut self.gfx {
            gfx.occluded = occluded;
        }
        if !occluded {
            self.surface_retry_at = None;
            self.request_redraw();
        }
    }

    pub(crate) fn render(&mut self, event_loop: &ActiveEventLoop) {
        let place_overlay = self
            .place_preview
            .latest
            .as_ref()
            .and_then(|preview| preview.stage_ndc)
            .and_then(|ndc| rectangle_place_overlay(self.displayed_camera, ndc));
        let Some(layout) = self.layout else {
            return;
        };
        self.synchronize_timeline_viewport(Some(layout));
        let (Some(gfx), Some(window)) = (&mut self.gfx, self.window.as_ref()) else {
            return;
        };
        match gfx.render(ProductSurfaceFrame {
            layout,
            window,
            document: &self.current_document,
            timeline_projection: &self.timeline_projection,
            primary: self.primary,
            playhead: self.playhead,
            interval_preview: self.interval_preview,
            place_overlay: place_overlay.as_ref(),
        }) {
            Ok(()) => {}
            Err(ProductSurfaceError::Recover) => {
                crate::ui_numeric_trace::emit(format_args!(
                    "kind=surface state=recover layout_epoch={}",
                    layout.epoch,
                ));
                gfx.reconfigure();
                window.request_redraw();
            }
            Err(ProductSurfaceError::Retry) => {
                crate::ui_numeric_trace::emit(format_args!(
                    "kind=surface state=retry layout_epoch={} retry_ms=50",
                    layout.epoch,
                ));
                let retry_at = Instant::now() + Duration::from_millis(50);
                self.surface_retry_at = Some(retry_at);
                event_loop.set_control_flow(ControlFlow::WaitUntil(retry_at));
            }
            Err(ProductSurfaceError::Skip) => {}
            Err(ProductSurfaceError::NativeTimeline(error)) => self.fail(event_loop, error),
            Err(ProductSurfaceError::Fatal(reason)) => self.fail(event_loop, reason),
        }
    }
}

fn clear_interval_gesture(
    active: &mut Option<ActiveIntervalGesture>,
    preview: &mut Option<TimelineIntervalPreview>,
) {
    *active = None;
    *preview = None;
}

fn elapsed_ms(started_at: Instant) -> f64 {
    started_at.elapsed().as_secs_f64() * 1_000.0
}

fn active_effect_candidate(
    previous_primary: Option<motolii_doc::LayerId>,
    previous_active: Option<EffectId>,
    published_primary: Option<motolii_doc::LayerId>,
    created_effect_use: Option<EffectId>,
    contains: impl FnOnce(motolii_doc::LayerId, EffectId) -> bool,
) -> Option<EffectId> {
    if previous_primary != published_primary {
        return None;
    }
    let primary = published_primary?;
    let candidate = created_effect_use.or(previous_active)?;
    contains(primary, candidate).then_some(candidate)
}

fn trace_document_publish(route: &str, published: &PublishedDocument) {
    crate::ui_numeric_trace::emit(format_args!(
        "kind=document-publish route={} action={:?} revision={} projection_generation={} \
         primary_present={} track_count={}",
        route,
        published.kind,
        published.revision,
        published.projection_generation,
        published.primary.is_some(),
        published.snapshot.tracks.len(),
    ));
}

fn trace_timeline_projection(
    route: &str,
    projection_generation: u64,
    projection: &ProductTimelineProjection,
) {
    crate::ui_numeric_trace::emit(format_args!(
        "kind=timeline-projection route={} projection_generation={} bars={} keys={} unsupported={}",
        route,
        projection_generation,
        projection.projection.bars().len(),
        projection.projection.keys().len(),
        projection.projection.unsupported().len(),
    ));
}

struct OptionalNumber(Option<f64>);

impl std::fmt::Display for OptionalNumber {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(value) => write!(formatter, "{value:.6}"),
            None => formatter.write_str("outside"),
        }
    }
}

fn build_browser_runtime(
    window: &Window,
    instance_epoch: u64,
    source: BrowserPlaceIntent,
    proxy: EventLoopProxy<ProductEvent>,
) -> Result<BrowserHostRuntime, BrowserHostRuntimeError> {
    let wake_proxy = proxy.clone();
    let lifecycle_proxy = proxy;
    BrowserHostRuntime::new(
        window,
        instance_epoch,
        source,
        Arc::new(move || {
            let _ = wake_proxy.send_event(ProductEvent::Wake);
        }),
        Arc::new(move |event| {
            let _ = lifecycle_proxy.send_event(ProductEvent::BrowserLifecycle(event));
        }),
    )
}

fn product_command_keymap(
    registry: &CommandRegistry,
) -> Result<KeymapResolution, ProductRuntimeError> {
    let primary = Modifiers::try_new([Modifier::Primary])?;
    let primary_shift = Modifiers::try_new([Modifier::Primary, Modifier::Shift])?;
    let z = KeyToken::Ascii(AsciiKey::try_new('z')?);
    let base = BuiltinKeymap::new(
        1,
        vec![
            Binding {
                gesture: Gesture::Keyboard {
                    key: z,
                    modifiers: primary,
                    phase: InputPhase::Press,
                },
                command: CommandId::try_new("motolii.edit.undo")?,
            },
            Binding {
                gesture: Gesture::Keyboard {
                    key: z,
                    modifiers: primary_shift,
                    phase: InputPhase::Press,
                },
                command: CommandId::try_new("motolii.edit.redo")?,
            },
        ],
    );
    let resolution = resolve_keymap(
        &base,
        &KeymapDelta::default(),
        &PlatformBindingConstraints::new(PlatformCommandModifier::Meta, Vec::new()),
        registry,
    );
    if resolution.diagnostics().is_empty() {
        Ok(resolution)
    } else {
        Err(ProductRuntimeError::CommandKeymap)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BrowserRecoveryDecision {
    Ignore,
    Replace { instance_epoch: u64 },
    Degrade,
}

#[derive(Debug)]
struct BrowserLifecycleCoordinator {
    next_instance_epoch: u64,
    automatic_process_recovery_used: bool,
    degraded: bool,
}

impl BrowserLifecycleCoordinator {
    fn new(initial_instance_epoch: u64) -> Result<Self, ProductRuntimeError> {
        Ok(Self {
            next_instance_epoch: initial_instance_epoch
                .checked_add(1)
                .ok_or(ProductRuntimeError::BrowserInstanceEpochExhausted)?,
            automatic_process_recovery_used: false,
            degraded: false,
        })
    }

    fn observe(
        &mut self,
        active_epoch: u64,
        event: BrowserLifecycleEvent,
    ) -> Result<BrowserRecoveryDecision, ProductRuntimeError> {
        if self.degraded || event.instance_epoch() != active_epoch {
            return Ok(BrowserRecoveryDecision::Ignore);
        }
        if matches!(event, BrowserLifecycleEvent::ProcessTerminated { .. })
            && self.automatic_process_recovery_used
        {
            self.degraded = true;
            return Ok(BrowserRecoveryDecision::Degrade);
        }
        if matches!(event, BrowserLifecycleEvent::ProcessTerminated { .. }) {
            self.automatic_process_recovery_used = true;
        }
        let instance_epoch = self.next_instance_epoch;
        self.next_instance_epoch = self
            .next_instance_epoch
            .checked_add(1)
            .ok_or(ProductRuntimeError::BrowserInstanceEpochExhausted)?;
        Ok(BrowserRecoveryDecision::Replace { instance_epoch })
    }
}

include!("product_surface_runtime.rs");

#[derive(Debug, thiserror::Error)]
pub(crate) enum ProductRuntimeError {
    #[error(transparent)]
    Gpu(#[from] motolii_gpu::GpuError),
    #[error(transparent)]
    Preview(#[from] crate::static_preview::StaticPreviewError),
    #[error(transparent)]
    RenderWorkerStart(#[from] crate::render_worker::RenderWorkerStartError),
    #[error(transparent)]
    RenderWorkerJoin(#[from] crate::render_worker::RenderJoinError),
    #[error(transparent)]
    RenderSubmit(#[from] crate::render_worker::RenderSubmitError),
    #[error(transparent)]
    RenderWorker(#[from] RenderWorkerError),
    #[error(transparent)]
    RepaintSignal(#[from] crate::render_worker::RepaintSignalRegistrationError),
    #[error(transparent)]
    Display(#[from] crate::display_slot::DisplaySlotError),
    #[error(transparent)]
    TimelineProjection(#[from] TimelineProjectionError),
    #[error(transparent)]
    EventLoop(#[from] winit::error::EventLoopError),
    #[error(transparent)]
    Os(#[from] winit::error::OsError),
    #[error(transparent)]
    Surface(#[from] wgpu::CreateSurfaceError),
    #[error(transparent)]
    Browser(#[from] BrowserHostRuntimeError),
    #[error(transparent)]
    Inspector(#[from] InspectorHostRuntimeError),
    #[error(transparent)]
    StageChrome(#[from] StageChromeHostRuntimeError),
    #[error(transparent)]
    StageChromeIntent(#[from] StageChromeIntentError),
    #[error(transparent)]
    EasingPopup(#[from] EasingPopupError),
    #[error(transparent)]
    TimelineTools(#[from] TimelineToolsHostRuntimeError),
    #[error(transparent)]
    NativeTimeline(#[from] NativeTimelineRendererError),
    #[error(transparent)]
    Layout(#[from] crate::layout::LayoutError),
    #[error(transparent)]
    NativeHostLayout(#[from] crate::native_host_layout::NativeHostLayoutError),
    #[error("native product Surface is unsupported by the selected adapter")]
    SurfaceUnsupported,
    #[error("native product Host was initialized twice")]
    AlreadyInitialized,
    #[error("native product layout epoch is exhausted")]
    LayoutEpochExhausted,
    #[error("Browser instance epoch is exhausted")]
    BrowserInstanceEpochExhausted,
    #[error("Browser lifecycle coordinator is unavailable")]
    BrowserLifecycleUnavailable,
    #[error("Place terminal cannot be classified without a native Host layout")]
    PlaceTerminalLayoutUnavailable,
    #[error("Place admission rejected capture generation {0}")]
    PlaceAdmissionGenerationRejected(u64),
    #[error("Place capture generation is exhausted")]
    PlaceGenerationExhausted,
    #[error("Timeline interval capture generation is exhausted")]
    IntervalGenerationExhausted,
    #[error("Place Stage NDC could not be converted through the displayed camera")]
    PlaceCanonicalConversion,
    #[error(transparent)]
    DocumentEdit(#[from] DocumentEditRuntimeError),
    #[error(transparent)]
    DocumentDispatch(#[from] DocumentEditDispatchError),
    #[error(transparent)]
    InputRouter(#[from] InputRouterError),
    #[error(transparent)]
    CommandRegistry(#[from] CommandRegistryError),
    #[error(transparent)]
    CommandId(#[from] CommandIdError),
    #[error(transparent)]
    Modifier(#[from] ModifierError),
    #[error(transparent)]
    AsciiKey(#[from] crate::AsciiKeyError),
    #[error("product command keymap contains an invalid or conflicting binding")]
    CommandKeymap,
    #[error("native product runtime failed: {0}")]
    Runtime(String),
}

#[cfg(test)]
#[path = "product_runtime_tests.rs"]
mod tests;

#[cfg(test)]
mod projection_unit_tests {
    use super::*;
    use crate::layout::PanelLayout;
    use crate::native_host_layout::NativeHostLayout;

    fn test_layout() -> NativeHostLayout {
        let frame = motolii_core::FrameDesc::try_packed(
            1920,
            1080,
            motolii_core::PixelFormat::Rgba8Unorm,
            motolii_core::ColorSpace::Srgb,
            true,
        )
        .unwrap();
        let authority = PanelLayout::built_in();
        NativeHostLayout::try_new(1, 1_000, 800, 1.0, frame, &authority)
            .unwrap()
            .unwrap()
    }

    #[test]
    fn projection_hit_and_time_conversions_use_fixed_34x80_grid() {
        let document = crate::static_preview::bootstrap_document().unwrap();
        let mut projection = ProductTimelineProjection::from_document(&document).unwrap();
        projection.set_horizontal_start(RationalTime::try_new(2, 1).unwrap());
        let layout = test_layout();
        let surface =
            crate::native_timeline_renderer::timeline_time_surface_logical_rect(layout).unwrap();
        assert_eq!(
            projection
                .logical_x(RationalTime::try_new(3, 1).unwrap(), layout)
                .unwrap(),
            surface.x + 80.0
        );
        assert_eq!(
            projection
                .time_at([surface.x + 80.0, surface.y + 2.0], layout,)
                .unwrap(),
            RationalTime::try_new(3, 1).unwrap(),
        );
    }

    #[test]
    fn rational_from_seconds_filters_non_finite_values() {
        assert!(ProductTimelineProjection::rational_from_seconds(f64::NAN).is_none());
        assert!(ProductTimelineProjection::rational_from_seconds(f64::INFINITY).is_none());
    }

    #[test]
    fn scroll_delta_normalization_applies_each_unit_conversion_once() {
        assert_eq!(line_delta_to_logical([1.0, -2.0]), [34.0, -68.0]);
        assert_eq!(pixel_delta_to_logical([68.0, -34.0], 2.0), [34.0, -17.0]);
        assert_eq!(
            pixel_delta_to_logical([f64::NAN, f64::INFINITY], 2.0),
            [0.0, 0.0]
        );
    }

    #[test]
    fn viewport_clamps_survive_resize_and_document_publish_bounds() {
        assert_eq!(clamp_horizontal_start_value(99.0, 0.0, 10.0, 4.0), 6.0);
        assert_eq!(clamp_horizontal_start_value(-1.0, 0.0, 10.0, 4.0), 0.0);
        assert_eq!(clamp_vertical_offset_value(999.0, 850.0, 400.0), 450.0);
        assert_eq!(clamp_vertical_offset_value(-1.0, 850.0, 400.0), 0.0);
        assert_eq!(clamp_vertical_offset_value(f64::NAN, 850.0, 400.0), 0.0);
    }

    #[test]
    fn panned_hit_identity_and_offscreen_endpoints_remain_distinct() {
        let document = crate::static_preview::bootstrap_document().unwrap();
        let mut projection = ProductTimelineProjection::from_document(&document).unwrap();
        let layer = projection.projection.bars()[0].layer;
        projection.set_horizontal_start(RationalTime::try_new(8, 1).unwrap());
        let layout = test_layout();
        let surface =
            crate::native_timeline_renderer::timeline_time_surface_logical_rect(layout).unwrap();
        assert_eq!(
            projection.hit_test([surface.x + 1.0, surface.y + 17.0], layout),
            Some(TimelineHit::Bar { layer })
        );
        let end = projection.projection.bars()[0].end;
        let endpoint_projection = ProductTimelineProjection::from_document(&document).unwrap();
        let raw_end = endpoint_projection
            .unclamped_logical_x(end, layout)
            .unwrap();
        let endpoint_x = endpoint_projection.logical_x(end, layout).unwrap();
        assert!(raw_end > surface.x + surface.width);
        assert_eq!(endpoint_x, raw_end);
    }
}
