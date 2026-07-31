//! macOS通常project sessionのdirect native Surface Host。

use std::borrow::Cow;
use std::sync::Arc;
use std::time::{Duration, Instant};

use motolii_core::{CanonicalPoint, Quality, RationalTime};
use motolii_doc::{EffectId, EvaluationTime};
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
    AttachEffectRequest, DocumentEditDispatchError, DocumentEditQueue, DocumentEditRuntime,
    DocumentEditRuntimeError, PlaceRectangleRequest, PublishedDocument,
};
use crate::host_pointer_capture::{HostPointerCancel, HostPointerCandidate};
use crate::inspector_host_runtime::{InspectorHostRuntime, InspectorHostRuntimeError};
use crate::layout_authority::LayoutAuthority;
use crate::native_host_layout::{LogicalRect, NativeHostLayout, PhysicalRect};
use crate::native_timeline_renderer::{
    key_tools_logical_rect, timeline_time_surface_logical_rect, NativeTimelineRenderer,
    NativeTimelineRendererError,
};
use crate::render_worker::{
    RenderGeneration, RenderRequest, RenderWorker, RenderWorkerClient, RenderWorkerError,
};
use crate::stage_chrome_host_runtime::{StageChromeHostRuntime, StageChromeHostRuntimeError};
use crate::static_preview::{bootstrap_frame_desc, prepare_in_setup_worker, StaticPreview};
use crate::timeline_projection::{
    project_timeline, TimelineHit, TimelineMetrics, TimelineProjection, TimelineProjectionError,
    TimelineViewport,
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
}

#[derive(Debug, Clone)]
struct PendingStageDrop {
    source: BrowserPlaceIntent,
    generation: u64,
    layout_epoch: u64,
    ndc: [f64; 2],
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
}

impl ProductTimelineProjection {
    fn from_document(document: &motolii_doc::Document) -> Result<Self, TimelineProjectionError> {
        let duration_seconds = document.composition.duration.as_seconds_f64();
        let projection = project_timeline(
            document,
            &TimelineMetrics {
                band_height: 1.0,
                units_per_second: duration_seconds.recip(),
                key_half_extent: 1.0,
            },
            &TimelineViewport {
                start: RationalTime::ZERO,
                end: document.composition.duration,
            },
        )?;
        let band_span = projection
            .bars()
            .iter()
            .map(|bar| bar.y_bottom)
            .fold(1.0, f64::max);
        Ok(Self {
            projection,
            band_span,
        })
    }

    fn hit_test(&self, position: [f64; 2], layout: NativeHostLayout) -> Option<TimelineHit> {
        let time_surface = timeline_time_surface_logical_rect(layout)?;
        if !time_surface.contains(position) {
            return None;
        }
        let x = (position[0] - time_surface.x) / time_surface.width;
        let y = ((position[1] - time_surface.y) / time_surface.height) * self.band_span;
        Some(self.projection.hit_test(x, y))
    }
}

#[derive(Debug, Default)]
struct PlaceTerminalDelivery {
    delivered_high_water: Option<u64>,
}

impl PlaceTerminalDelivery {
    fn deliver(&mut self, terminal: &ClassifiedPlaceTerminal) -> Option<PendingStageDrop> {
        if terminal.cause != PlaceTerminalCause::NoNonCommitCause
            || self
                .delivered_high_water
                .is_some_and(|high_water| terminal.generation <= high_water)
        {
            return None;
        }
        let (Some(layout_epoch), Some(ndc)) = (terminal.layout_epoch, terminal.stage_ndc) else {
            return None;
        };
        self.delivered_high_water = Some(terminal.generation);
        Some(PendingStageDrop {
            source: terminal.source.clone(),
            generation: terminal.generation,
            layout_epoch,
            ndc,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlaceTerminalCause {
    Escape,
    OutsideStage,
    CaptureLoss,
    NoNonCommitCause,
}

#[derive(Debug, Clone, PartialEq)]
struct ClassifiedPlaceTerminal {
    source: BrowserPlaceIntent,
    generation: u64,
    cause: PlaceTerminalCause,
    layout_epoch: Option<u64>,
    stage_ndc: Option<[f64; 2]>,
}

impl ClassifiedPlaceTerminal {
    fn released(
        source: BrowserPlaceIntent,
        generation: u64,
        position: [f64; 2],
        layout: NativeHostLayout,
    ) -> Self {
        let stage_ndc = layout.stage_ndc(position);
        Self {
            source,
            generation,
            cause: if stage_ndc.is_some() {
                PlaceTerminalCause::NoNonCommitCause
            } else {
                PlaceTerminalCause::OutsideStage
            },
            layout_epoch: Some(layout.epoch),
            stage_ndc,
        }
    }

    fn cancelled(source: BrowserPlaceIntent, generation: u64, reason: HostPointerCancel) -> Self {
        Self {
            source,
            generation,
            cause: match reason {
                HostPointerCancel::Escape => PlaceTerminalCause::Escape,
                HostPointerCancel::CaptureLost => PlaceTerminalCause::CaptureLoss,
            },
            layout_epoch: None,
            stage_ndc: None,
        }
    }
}

#[derive(Debug, Default)]
struct PlaceTerminalAdmission {
    active_generation: Option<u64>,
    retired_high_water: Option<u64>,
}

impl PlaceTerminalAdmission {
    fn begin(&mut self, generation: u64) -> bool {
        if self.active_generation.is_some()
            || self
                .retired_high_water
                .is_some_and(|high_water| generation <= high_water)
        {
            return false;
        }
        self.active_generation = Some(generation);
        true
    }

    fn admit(&mut self, terminal: &ClassifiedPlaceTerminal) -> bool {
        if self.active_generation != Some(terminal.generation) {
            return false;
        }
        self.active_generation = None;
        self.retired_high_water = Some(
            self.retired_high_water
                .map_or(terminal.generation, |high_water| {
                    high_water.max(terminal.generation)
                }),
        );
        terminal.cause == PlaceTerminalCause::NoNonCommitCause
    }

    fn retire_active(&mut self) {
        if let Some(generation) = self.active_generation.take() {
            self.retired_high_water = Some(
                self.retired_high_water
                    .map_or(generation, |high_water| high_water.max(generation)),
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct PlacePreviewProgress {
    source: BrowserPlaceIntent,
    generation: u64,
    layout_epoch: u64,
    stage_ndc: Option<[f64; 2]>,
}

#[derive(Debug, Default)]
struct PlacePreviewPhase {
    latest: Option<PlacePreviewProgress>,
}

impl PlacePreviewPhase {
    fn deliver(
        &mut self,
        source: &BrowserPlaceIntent,
        generation: u64,
        position: [f64; 2],
        layout: NativeHostLayout,
    ) {
        self.latest = Some(PlacePreviewProgress {
            source: source.clone(),
            generation,
            layout_epoch: layout.epoch,
            stage_ndc: layout.stage_ndc(position),
        });
    }

    fn clear(&mut self) {
        self.latest = None;
    }
}

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
        })
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
        let stage_chrome = StageChromeHostRuntime::new(&window)?;
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
        Ok(())
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
        if self.active_place.is_none() {
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
        if self.active_place.is_none() {
            self.poll_host_input(event_loop);
            self.set_idle_control_flow(event_loop);
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
                        )
                        {
                            return self.fail(event_loop, error);
                        }
                    }
                    self.timeline_projection =
                        match ProductTimelineProjection::from_document(&self.current_document) {
                            Ok(projection) => projection,
                            Err(error) => return self.fail(event_loop, error),
                        };
                    trace_timeline_projection(
                        "place",
                        self.projection_generation,
                        &self.timeline_projection,
                    );
                    if let Err(error) = self.publish_timeline_tools() {
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
        if self.active_place.is_some() {
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

    fn handle_timeline_click(&mut self, event_loop: &ActiveEventLoop, position: [f64; 2]) {
        let Some(layout) = self.layout else {
            return;
        };
        let hit = self.timeline_projection.hit_test(position, layout);
        crate::ui_numeric_trace::emit(format_args!(
            "kind=timeline-hit layout_epoch={} logical_x={:.3} logical_y={:.3} hit={:?}",
            layout.epoch, position[0], position[1], hit,
        ));
        let Some(hit) = hit else {
            return;
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
                    ) {
                        return self.fail(event_loop, error);
                    }
                }
                self.request_redraw();
            }
            Ok(None) => {}
            Err(error) => self.fail(event_loop, error),
        }
    }

    fn submit_stage_projection(&self) -> Result<RenderGeneration, ProductRuntimeError> {
        let generation = self.render_client.submit(RenderRequest {
            document: Arc::clone(&self.current_document),
            data_tracks: Arc::clone(&self.render_request_template.data_tracks),
            evaluation_time: self.render_request_template.evaluation_time,
            desc: self.render_request_template.desc,
            quality: self.render_request_template.quality,
        })?;
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
            ) {
                return self.fail(event_loop, error);
            }
        }
        self.timeline_projection =
            match ProductTimelineProjection::from_document(&self.current_document) {
                Ok(projection) => projection,
                Err(error) => return self.fail(event_loop, error),
            };
        trace_timeline_projection(route, self.projection_generation, &self.timeline_projection);
        if let Err(error) = self.publish_timeline_tools() {
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
        let (Some(gfx), Some(layout), Some(window)) = (&mut self.gfx, self.layout, &self.window)
        else {
            return;
        };
        match gfx.render(
            layout,
            window,
            &self.current_document,
            &self.timeline_projection,
            self.primary,
            place_overlay.as_ref(),
        ) {
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

struct ProductSurface {
    surface: wgpu::Surface<'static>,
    gpu: Arc<GpuCtx>,
    config: wgpu::SurfaceConfiguration,
    preview_pipeline: wgpu::RenderPipeline,
    preview_bind_group: wgpu::BindGroup,
    native_timeline_renderer: NativeTimelineRenderer,
    last_timeline_scene_trace: Option<(u64, usize, usize, usize, usize)>,
    place_overlay_pipeline: wgpu::RenderPipeline,
    place_overlay_vertices: wgpu::Buffer,
    occluded: bool,
}

struct ProductGpuParts {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
}

impl ProductSurface {
    fn new(
        window: &Arc<Window>,
        parts: ProductGpuParts,
        gpu: &Arc<GpuCtx>,
        preview: &StaticPreview,
    ) -> Result<Self, ProductRuntimeError> {
        let surface = parts.instance.create_surface(Arc::clone(window))?;
        if !parts.adapter.is_surface_supported(&surface) {
            return Err(ProductRuntimeError::SurfaceUnsupported);
        }
        let size = window.inner_size();
        let capabilities = surface.get_capabilities(&parts.adapter);
        let format = capabilities
            .formats
            .first()
            .copied()
            .ok_or(ProductRuntimeError::SurfaceUnsupported)?;
        let alpha_mode = capabilities
            .alpha_modes
            .first()
            .copied()
            .ok_or(ProductRuntimeError::SurfaceUnsupported)?;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats: vec![],
        };
        surface.configure(&parts.device, &config);
        let (preview_pipeline, preview_bind_group) =
            create_preview_pipeline(&parts.device, format, preview.slot().view());
        let native_timeline_renderer = NativeTimelineRenderer::new(
            &parts.device,
            &gpu.queue,
            format,
            size.width,
            size.height,
        )?;
        let place_overlay_pipeline = create_place_overlay_pipeline(&parts.device, format);
        let place_overlay_vertices = parts.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("motolii-product-place-overlay-vertices"),
            size: 48,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Ok(Self {
            surface,
            gpu: Arc::clone(gpu),
            config,
            preview_pipeline,
            preview_bind_group,
            native_timeline_renderer,
            last_timeline_scene_trace: None,
            place_overlay_pipeline,
            place_overlay_vertices,
            occluded: false,
        })
    }

    fn configure(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.native_timeline_renderer
            .resize(&self.gpu.device, width, height);
        self.reconfigure();
    }

    fn reconfigure(&self) {
        self.surface.configure(&self.gpu.device, &self.config);
    }

    fn render(
        &mut self,
        layout: NativeHostLayout,
        window: &Window,
        document: &motolii_doc::Document,
        timeline_projection: &ProductTimelineProjection,
        primary: Option<motolii_doc::LayerId>,
        place_overlay: Option<&RectanglePlaceOverlay>,
    ) -> Result<(), ProductSurfaceError> {
        if self.occluded || self.config.width == 0 || self.config.height == 0 {
            return Err(ProductSurfaceError::Skip);
        }
        let timeline_stats = self.native_timeline_renderer.prepare(
            &self.gpu.device,
            &self.gpu.queue,
            layout,
            document,
            &timeline_projection.projection,
            primary,
        )?;
        let trace_key = (
            layout.epoch,
            timeline_stats.rows,
            timeline_stats.bars,
            timeline_stats.keys,
            timeline_stats.text_runs,
        );
        if self.last_timeline_scene_trace != Some(trace_key) {
            if let Some(timeline) = layout.timeline_physical {
                crate::ui_numeric_trace::emit(format_args!(
                    "kind=timeline-scene layout_epoch={} rows={} bars={} keys={} text_runs={} \
                     physical_x={} physical_y={} physical_width={} physical_height={}",
                    layout.epoch,
                    timeline_stats.rows,
                    timeline_stats.bars,
                    timeline_stats.keys,
                    timeline_stats.text_runs,
                    timeline.x,
                    timeline.y,
                    timeline.width,
                    timeline.height,
                ));
                self.last_timeline_scene_trace = Some(trace_key);
            }
        }
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Timeout => {
                return Err(ProductSurfaceError::Retry);
            }
            wgpu::CurrentSurfaceTexture::Occluded => {
                return Err(ProductSurfaceError::Retry);
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                return Err(ProductSurfaceError::Recover);
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(ProductSurfaceError::Fatal(
                    "native product Surface validation failed".to_owned(),
                ));
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("motolii-product-native-frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("motolii-product-stage-timeline"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.018,
                            g: 0.020,
                            b: 0.024,
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
            draw_rect(
                &mut pass,
                layout.stage_physical,
                &self.preview_pipeline,
                Some(&self.preview_bind_group),
            );
            if let Some(place_overlay) = place_overlay {
                let bytes = place_overlay.vertex_bytes();
                self.gpu
                    .queue
                    .write_buffer(&self.place_overlay_vertices, 0, &bytes);
                pass.set_pipeline(&self.place_overlay_pipeline);
                pass.set_vertex_buffer(0, self.place_overlay_vertices.slice(..));
                pass.set_viewport(
                    layout.stage_physical.x as f32,
                    layout.stage_physical.y as f32,
                    layout.stage_physical.width as f32,
                    layout.stage_physical.height as f32,
                    0.0,
                    1.0,
                );
                pass.set_scissor_rect(
                    layout.stage_physical.x,
                    layout.stage_physical.y,
                    layout.stage_physical.width,
                    layout.stage_physical.height,
                );
                pass.draw(0..6, 0..1);
            }
            self.native_timeline_renderer.composite(&mut pass);
        }
        self.gpu.queue.submit([encoder.finish()]);
        window.pre_present_notify();
        frame.present();
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
struct RectanglePlaceOverlay {
    vertices: [[f32; 2]; 6],
}

impl RectanglePlaceOverlay {
    fn vertex_bytes(&self) -> [u8; 48] {
        let mut bytes = [0_u8; 48];
        for (index, component) in self.vertices.iter().flatten().enumerate() {
            let start = index * 4;
            bytes[start..start + 4].copy_from_slice(&component.to_ne_bytes());
        }
        bytes
    }
}

fn rectangle_place_overlay(
    camera: motolii_core::CompCamera,
    ndc: [f64; 2],
) -> Option<RectanglePlaceOverlay> {
    let center = canonical_drop_from_ndc(camera, ndc)?;
    let corners = [
        CanonicalPoint {
            x: center[0] - 0.1,
            y: center[1] - 0.1,
        },
        CanonicalPoint {
            x: center[0] + 0.1,
            y: center[1] - 0.1,
        },
        CanonicalPoint {
            x: center[0] + 0.1,
            y: center[1] + 0.1,
        },
        CanonicalPoint {
            x: center[0] - 0.1,
            y: center[1] + 0.1,
        },
    ];
    let mut projected = [[0.0_f32; 2]; 4];
    for (target, corner) in projected.iter_mut().zip(corners) {
        let (x, y) = camera.world_to_ndc(corner).ok()?;
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        *target = [x as f32, y as f32];
    }
    Some(RectanglePlaceOverlay {
        vertices: [
            projected[0],
            projected[1],
            projected[2],
            projected[0],
            projected[2],
            projected[3],
        ],
    })
}

fn draw_rect<'a>(
    pass: &mut wgpu::RenderPass<'a>,
    rect: PhysicalRect,
    pipeline: &'a wgpu::RenderPipeline,
    bind_group: Option<&'a wgpu::BindGroup>,
) {
    if rect.width == 0 || rect.height == 0 {
        return;
    }
    pass.set_pipeline(pipeline);
    if let Some(bind_group) = bind_group {
        pass.set_bind_group(0, bind_group, &[]);
    }
    pass.set_viewport(
        rect.x as f32,
        rect.y as f32,
        rect.width as f32,
        rect.height as f32,
        0.0,
        1.0,
    );
    pass.set_scissor_rect(rect.x, rect.y, rect.width, rect.height);
    pass.draw(0..3, 0..1);
}

fn create_preview_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    view: &wgpu::TextureView,
) -> (wgpu::RenderPipeline, wgpu::BindGroup) {
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("motolii-product-preview-sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("motolii-product-preview-layout"),
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
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("motolii-product-preview-bind-group"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    (
        create_pipeline(device, format, Some(&layout), PREVIEW_SHADER),
        bind_group,
    )
}

fn create_place_overlay_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("motolii-product-place-overlay-shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(PLACE_OVERLAY_SHADER)),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("motolii-product-place-overlay-layout"),
        bind_group_layouts: &[],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("motolii-product-place-overlay-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: 8,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                }],
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
    })
}

fn create_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    bind_group_layout: Option<&wgpu::BindGroupLayout>,
    source: &'static str,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("motolii-product-native-shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
    });
    let layouts: Vec<_> = bind_group_layout.into_iter().map(Some).collect();
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("motolii-product-native-pipeline-layout"),
        bind_group_layouts: &layouts,
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("motolii-product-native-pipeline"),
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

const PREVIEW_SHADER: &str = r#"
struct VertexOut { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> }
@vertex fn vs_main(@builtin(vertex_index) index: u32) -> VertexOut {
    var positions = array<vec2<f32>, 3>(vec2(-1.0,-1.0), vec2(3.0,-1.0), vec2(-1.0,3.0));
    var uvs = array<vec2<f32>, 3>(vec2(0.0,1.0), vec2(2.0,1.0), vec2(0.0,-1.0));
    var out: VertexOut; out.position = vec4(positions[index],0.0,1.0); out.uv = uvs[index]; return out;
}
@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@fragment fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(source_texture, source_sampler, in.uv);
}
"#;
const PLACE_OVERLAY_SHADER: &str = r#"
struct VertexOut { @builtin(position) position: vec4<f32> }
@vertex fn vs_main(@location(0) position: vec2<f32>) -> VertexOut {
    var out: VertexOut; out.position = vec4(position, 0.0, 1.0); return out;
}
@fragment fn fs_main() -> @location(0) vec4<f32> {
    return vec4(0.8, 0.58431375, 0.5294118, 0.42);
}
"#;

#[derive(Debug, thiserror::Error)]
enum ProductSurfaceError {
    #[error("native product Surface must be reconfigured")]
    Recover,
    #[error("native product Surface frame must be retried")]
    Retry,
    #[error("native product Surface frame is skipped")]
    Skip,
    #[error("native product Surface failed: {0}")]
    Fatal(String),
    #[error(transparent)]
    NativeTimeline(#[from] NativeTimelineRendererError),
}

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
mod tests {
    use super::*;
    use motolii_core::{ColorSpace, FrameDesc, PixelFormat};

    fn test_layout(epoch: u64) -> NativeHostLayout {
        test_layout_with(epoch, crate::layout::PanelLayout::built_in())
    }

    fn test_layout_with(epoch: u64, authority: crate::layout::PanelLayout) -> NativeHostLayout {
        let frame =
            FrameDesc::try_packed(1920, 1080, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true)
                .unwrap();
        NativeHostLayout::try_new(epoch, 1000, 800, 1.0, frame, &authority)
            .unwrap()
            .unwrap()
    }

    fn test_source() -> BrowserPlaceIntent {
        BrowserPlaceIntent {
            scope_ref: "builtin-stable".to_owned(),
            item_id: "rectangle".to_owned(),
        }
    }

    #[test]
    fn stage_projection_accepts_only_the_latest_new_generation() {
        let one = RenderGeneration::new(1).unwrap();
        let two = RenderGeneration::new(2).unwrap();
        let mut projection = ProductStageProjection::default();

        assert!(!projection.accepts(one, Some(two)));
        assert!(projection.accepts(two, Some(two)));
        projection.commit(two);
        assert!(!projection.accepts(two, Some(two)));
        assert!(!projection.accepts(one, Some(one)));
    }

    #[test]
    fn timeline_projection_uses_the_document_envelope_without_owned_range_state() {
        let document = crate::static_preview::bootstrap_document().unwrap();
        let projection = ProductTimelineProjection::from_document(&document).unwrap();
        let bar = projection.projection.bars().first().unwrap();

        assert_eq!(projection.projection.bars().len(), 1);
        assert_eq!(bar.x_start, 0.0);
        assert_eq!(bar.x_end, 1.0);
        assert_eq!(projection.band_span, 1.0);
    }

    #[test]
    fn timeline_time_surface_reuses_the_typed_projection_hit_and_excludes_chrome() {
        let document = crate::static_preview::bootstrap_document().unwrap();
        let projection = ProductTimelineProjection::from_document(&document).unwrap();
        let expected_layer = projection.projection.bars()[0].layer;
        let layout = test_layout(9);
        let timeline = layout.timeline.unwrap();
        let time_surface = timeline_time_surface_logical_rect(layout).unwrap();
        let center = [
            time_surface.x + time_surface.width / 2.0,
            time_surface.y + time_surface.height / 2.0,
        ];

        assert_eq!(
            projection.hit_test(center, layout),
            Some(TimelineHit::Bar {
                layer: expected_layer
            })
        );
        assert_eq!(
            projection.hit_test([timeline.x + 100.0, time_surface.y + 10.0], layout),
            None
        );
        assert_eq!(
            projection.hit_test([timeline.x + 220.0, time_surface.y + 10.0], layout),
            None
        );
        assert_eq!(
            projection.hit_test([time_surface.x + 10.0, timeline.y + 15.0], layout),
            None
        );
        assert_eq!(
            projection.hit_test([time_surface.x + 10.0, timeline.y + 40.0], layout),
            None
        );
        assert_eq!(
            projection.hit_test([layout.stage.x, layout.stage.y], layout),
            None
        );
    }

    #[test]
    fn hidden_timeline_has_no_selection_hit() {
        let document = crate::static_preview::bootstrap_document().unwrap();
        let projection = ProductTimelineProjection::from_document(&document).unwrap();
        let mut authority = crate::layout::PanelLayout::built_in();
        authority
            .apply(
                crate::layout::LayoutAction::Hide(crate::layout::PanelRole::Timeline),
                crate::layout::LayoutConstraints {
                    viewport_width: 1_000.0,
                    stage_min_width: 320.0,
                },
            )
            .unwrap();
        let layout = test_layout_with(10, authority);

        assert_eq!(projection.hit_test([500.0, 700.0], layout), None);
    }

    #[test]
    fn product_history_shortcuts_resolve_to_stable_command_ids() {
        let registry = builtin_command_registry().unwrap();
        let keymap = product_command_keymap(&registry).unwrap();
        let z = KeyToken::Ascii(AsciiKey::try_new('z').unwrap());
        let undo = EffectiveTrigger::Keyboard {
            key: z,
            modifiers: Modifiers::try_new([Modifier::Meta]).unwrap(),
            phase: InputPhase::Press,
        };
        let redo = EffectiveTrigger::Keyboard {
            key: z,
            modifiers: Modifiers::try_new([Modifier::Meta, Modifier::Shift]).unwrap(),
            phase: InputPhase::Press,
        };

        assert_eq!(
            keymap.get(&undo).map(CommandId::as_str),
            Some("motolii.edit.undo")
        );
        assert_eq!(
            keymap.get(&redo).map(CommandId::as_str),
            Some("motolii.edit.redo")
        );
        assert!(keymap.diagnostics().is_empty());
    }

    #[test]
    fn active_effect_candidate_prefers_attach_and_clears_on_primary_change() {
        let primary = motolii_doc::LayerId::from_raw(7);
        let other_primary = motolii_doc::LayerId::from_raw(8);
        let previous = EffectId::from_raw(10);
        let attached = EffectId::from_raw(11);

        assert_eq!(
            active_effect_candidate(
                Some(primary),
                Some(previous),
                Some(primary),
                Some(attached),
                |candidate_primary, candidate| {
                    candidate_primary == primary && candidate == attached
                },
            ),
            Some(attached)
        );
        assert_eq!(
            active_effect_candidate(
                Some(primary),
                Some(previous),
                Some(other_primary),
                Some(attached),
                |_, _| true,
            ),
            None
        );
    }

    #[test]
    fn active_effect_candidate_does_not_resurrect_after_disappearance() {
        let primary = motolii_doc::LayerId::from_raw(7);
        let effect = EffectId::from_raw(10);

        let after_removal =
            active_effect_candidate(Some(primary), Some(effect), Some(primary), None, |_, _| {
                false
            });
        assert_eq!(after_removal, None);
        assert_eq!(
            active_effect_candidate(
                Some(primary),
                after_removal,
                Some(primary),
                None,
                |candidate_primary, candidate| {
                    candidate_primary == primary && candidate == effect
                },
            ),
            None
        );
    }

    #[test]
    fn moved_progress_creates_a_nonterminal_preview_phase() {
        let layout = test_layout(9);
        let center = [
            layout.stage.x + layout.stage.width / 2.0,
            layout.stage.y + layout.stage.height / 2.0,
        ];
        let source = test_source();
        let mut phase = PlacePreviewPhase::default();

        phase.deliver(&source, 4, center, layout);

        assert_eq!(
            phase.latest,
            Some(PlacePreviewProgress {
                source,
                generation: 4,
                layout_epoch: 9,
                stage_ndc: Some([0.0, 0.0]),
            })
        );
    }

    #[test]
    fn outside_progress_updates_preview_without_becoming_a_terminal() {
        let layout = test_layout(9);
        let source = test_source();
        let mut phase = PlacePreviewPhase::default();
        phase.deliver(&source, 4, [10.0, 10.0], layout);

        assert_eq!(
            phase.latest,
            Some(PlacePreviewProgress {
                source,
                generation: 4,
                layout_epoch: 9,
                stage_ndc: None,
            })
        );
        phase.clear();
        assert_eq!(phase.latest, None);
    }

    #[test]
    fn release_inside_stage_has_no_noncommit_cause() {
        let layout = test_layout(9);
        let position = [
            layout.stage.x + layout.stage.width / 2.0,
            layout.stage.y + layout.stage.height / 2.0,
        ];

        assert_eq!(
            ClassifiedPlaceTerminal::released(test_source(), 4, position, layout),
            ClassifiedPlaceTerminal {
                source: test_source(),
                generation: 4,
                cause: PlaceTerminalCause::NoNonCommitCause,
                layout_epoch: Some(9),
                stage_ndc: Some([0.0, 0.0]),
            }
        );
    }

    #[test]
    fn release_outside_stage_has_outside_cause() {
        let layout = test_layout(9);

        assert_eq!(
            ClassifiedPlaceTerminal::released(test_source(), 4, [10.0, 10.0], layout),
            ClassifiedPlaceTerminal {
                source: test_source(),
                generation: 4,
                cause: PlaceTerminalCause::OutsideStage,
                layout_epoch: Some(9),
                stage_ndc: None,
            }
        );
    }

    #[test]
    fn cancellation_reason_maps_exhaustively_to_noncommit_cause() {
        for (reason, cause) in [
            (HostPointerCancel::Escape, PlaceTerminalCause::Escape),
            (
                HostPointerCancel::CaptureLost,
                PlaceTerminalCause::CaptureLoss,
            ),
        ] {
            assert_eq!(
                ClassifiedPlaceTerminal::cancelled(test_source(), 4, reason),
                ClassifiedPlaceTerminal {
                    source: test_source(),
                    generation: 4,
                    cause,
                    layout_epoch: None,
                    stage_ndc: None,
                }
            );
        }
    }

    #[test]
    fn admission_accepts_at_most_one_matching_commit_candidate() {
        let layout = test_layout(9);
        let position = [
            layout.stage.x + layout.stage.width / 2.0,
            layout.stage.y + layout.stage.height / 2.0,
        ];
        let terminal = ClassifiedPlaceTerminal::released(test_source(), 4, position, layout);
        let mut admission = PlaceTerminalAdmission::default();

        assert!(admission.begin(4));
        assert!(admission.admit(&terminal));
        assert!(!admission.admit(&terminal));
        assert!(!admission.begin(4));
    }

    #[test]
    fn noncommit_terminal_retires_generation_without_admission() {
        let terminal =
            ClassifiedPlaceTerminal::cancelled(test_source(), 4, HostPointerCancel::CaptureLost);
        let mut admission = PlaceTerminalAdmission::default();

        assert!(admission.begin(4));
        assert!(!admission.admit(&terminal));
        assert!(!admission.admit(&terminal));
        assert!(!admission.begin(4));
    }

    #[test]
    fn stale_terminal_does_not_retire_the_current_drag() {
        let layout = test_layout(9);
        let position = [
            layout.stage.x + layout.stage.width / 2.0,
            layout.stage.y + layout.stage.height / 2.0,
        ];
        let stale = ClassifiedPlaceTerminal::released(test_source(), 4, position, layout);
        let current = ClassifiedPlaceTerminal::released(test_source(), 5, position, layout);
        let mut admission = PlaceTerminalAdmission::default();

        assert!(admission.begin(4));
        assert!(admission.admit(&stale));
        assert!(admission.begin(5));
        assert!(!admission.admit(&stale));
        assert!(admission.admit(&current));
    }

    #[test]
    fn retained_high_water_rejects_replay_after_detail_eviction() {
        let layout = test_layout(9);
        let position = [
            layout.stage.x + layout.stage.width / 2.0,
            layout.stage.y + layout.stage.height / 2.0,
        ];
        let terminal = ClassifiedPlaceTerminal::released(test_source(), 8, position, layout);
        let mut admission = PlaceTerminalAdmission::default();

        assert!(admission.begin(8));
        assert!(admission.admit(&terminal));
        assert!(!admission.begin(7));
        assert!(!admission.begin(8));
        assert!(admission.begin(9));
    }

    #[test]
    fn admitted_terminal_delivers_once_to_the_single_pending_boundary() {
        let layout = test_layout(9);
        let position = [
            layout.stage.x + layout.stage.width / 2.0,
            layout.stage.y + layout.stage.height / 2.0,
        ];
        let terminal = ClassifiedPlaceTerminal::released(test_source(), 4, position, layout);
        let mut delivery = PlaceTerminalDelivery::default();

        let delivered = delivery.deliver(&terminal).unwrap();
        assert_eq!(delivered.generation, 4);
        assert_eq!(delivered.layout_epoch, 9);
        assert_eq!(delivered.ndc, [0.0, 0.0]);
        assert!(delivery.deliver(&terminal).is_none());
    }

    #[test]
    fn unadmitted_causes_cannot_enter_the_delivery_boundary() {
        let mut delivery = PlaceTerminalDelivery::default();
        for reason in [HostPointerCancel::Escape, HostPointerCancel::CaptureLost] {
            let terminal = ClassifiedPlaceTerminal::cancelled(test_source(), 4, reason);
            assert!(delivery.deliver(&terminal).is_none());
        }
    }

    #[test]
    fn lifecycle_replaces_with_new_epochs_and_ignores_old_callbacks() {
        let mut lifecycle = BrowserLifecycleCoordinator::new(7).unwrap();

        assert_eq!(
            lifecycle
                .observe(
                    7,
                    BrowserLifecycleEvent::ReloadStarted { instance_epoch: 7 }
                )
                .unwrap(),
            BrowserRecoveryDecision::Replace { instance_epoch: 8 }
        );
        assert_eq!(
            lifecycle
                .observe(
                    8,
                    BrowserLifecycleEvent::ReloadStarted { instance_epoch: 7 }
                )
                .unwrap(),
            BrowserRecoveryDecision::Ignore
        );
        assert_eq!(
            lifecycle
                .observe(
                    8,
                    BrowserLifecycleEvent::ReloadStarted { instance_epoch: 8 }
                )
                .unwrap(),
            BrowserRecoveryDecision::Replace { instance_epoch: 9 }
        );
    }

    #[test]
    fn automatic_process_recovery_is_bounded_to_one_replacement() {
        let mut lifecycle = BrowserLifecycleCoordinator::new(10).unwrap();

        assert_eq!(
            lifecycle
                .observe(
                    10,
                    BrowserLifecycleEvent::ProcessTerminated { instance_epoch: 10 }
                )
                .unwrap(),
            BrowserRecoveryDecision::Replace { instance_epoch: 11 }
        );
        assert_eq!(
            lifecycle
                .observe(
                    11,
                    BrowserLifecycleEvent::ProcessTerminated { instance_epoch: 11 }
                )
                .unwrap(),
            BrowserRecoveryDecision::Degrade
        );
        assert_eq!(
            lifecycle
                .observe(
                    11,
                    BrowserLifecycleEvent::ReloadStarted { instance_epoch: 11 }
                )
                .unwrap(),
            BrowserRecoveryDecision::Ignore
        );
    }

    #[test]
    fn lifecycle_epoch_exhaustion_is_typed() {
        assert!(matches!(
            BrowserLifecycleCoordinator::new(u64::MAX),
            Err(ProductRuntimeError::BrowserInstanceEpochExhausted)
        ));
    }
}
