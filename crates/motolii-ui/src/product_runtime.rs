//! macOS通常project sessionのdirect native Surface Host。

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

use motolii_audio::{AudioProgram, PcmCache, CANONICAL_SAMPLE_RATE};
use motolii_core::{CanonicalPoint, Fps, FpsError, Quality, RationalTime, RationalTimeError};
use motolii_doc::{
    Command, DocParam, DocValue, EffectId, EvaluationTime, KeyframeId, LayerId, TrackItem,
};
use motolii_eval::DataTracks;
use motolii_gpu::GpuCtx;
use motolii_transport::{FramePlan, PlaybackSession, PlaybackSessionError, TransportError};
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
    DocumentEditRuntime, DocumentEditRuntimeError, PlaceMediaRequest, PlaceRectangleRequest,
    PublishedDocument, SetPositionKeyInterpRequest, SetPositionKeyValueRequest,
};
use crate::host_file_drop::{
    HostFileDropEvent, HostFileDropTerminal, PlatformFileDrop, PlatformFileDropError,
};
use crate::host_pointer_capture::{HostPointerCancel, HostPointerCandidate};
use crate::inspector_host_runtime::{
    resolve_effect_param_preview_command, InspectorGestureTerminal, InspectorGestureTerminalCause,
    InspectorHostRuntime, InspectorHostRuntimeError, InspectorPositionAxis,
    InspectorPositionGestureStart, InspectorPositionGestureTerminal,
    InspectorPositionGestureTerminalCause,
};
use crate::layout_authority::LayoutAuthority;
use crate::native_host_layout::{LogicalRect, NativeHostLayout, PhysicalRect};
use crate::native_timeline_renderer::{
    key_tools_logical_rect, timeline_ruler_logical_rect, timeline_time_surface_logical_rect,
    NativeTimelineRenderState, NativeTimelineRenderer, NativeTimelineRendererError,
};
use crate::product_easing_popup::{
    PopupTerminal, ProductEasingPopup, ProductEasingPopupError, ProductEasingPopupOpen,
};
use crate::render_worker::{
    RenderGeneration, RenderRequest, RenderWorker, RenderWorkerClient, RenderWorkerError,
};
use crate::stage_chrome_host_runtime::{
    StageChromeHostRuntime, StageChromeHostRuntimeError, StageEasingIntent, StageEasingIntentError,
    StagePlaybackIntentError, StagePlaybackState, StagePlaybackToggle, StageTransportSnapshot,
};
use crate::static_preview::{bootstrap_frame_desc, prepare_in_setup_worker, StaticPreview};
use crate::timeline_move_gesture::{TimelineMoveGesture, TimelineMoveRequest};
use crate::timeline_projection::{
    project_timeline, TimelineHit, TimelineMetrics, TimelineProjection, TimelineProjectionError,
    TimelineViewport,
};
use crate::timeline_tools_host_runtime::{TimelineToolsHostRuntime, TimelineToolsHostRuntimeError};
use crate::timeline_trim_gesture::{TimelineTrimEdge, TimelineTrimGesture};
use crate::{
    builtin_command_registry, resolve_keymap, AsciiKey, Binding, BuiltinKeymap, CommandId,
    CommandIdError, CommandRegistry, CommandRegistryError, DomainIntent, EffectiveTrigger, Gesture,
    ImeGateState, InputPhase, InputRouter, InputRouterError, KeyToken, KeymapDelta,
    KeymapResolution, Modifier, ModifierError, Modifiers, NormalizedInput,
    PlatformBindingConstraints, PlatformCommandModifier, RouterOutput, SafetyInterrupt,
};

#[derive(Debug, Clone)]
pub(crate) enum ProductEvent {
    Wake,
    BrowserLifecycle(BrowserLifecycleEvent),
    FileDrop(HostFileDropEvent),
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
    easing_popup: Option<(ProductEasingPopup, PositionActiveInterval, u64, u64)>,
    timeline_tools: Option<TimelineToolsHostRuntime>,
    file_drop: Option<PlatformFileDrop>,
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
    current_modifiers: Modifiers,
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
    pending_inspector_commit: Option<InspectorGestureTerminal>,
    pending_position_commit: Option<InspectorPositionGestureTerminal>,
    position_gesture: Option<PositionGestureBaseline>,
    timeline_move: Option<TimelineMoveGesture>,
    timeline_trim: Option<TimelineTrimGesture>,
    editor_playhead: EditorPlayhead,
    playback_lifecycle: PlaybackLifecycle,
    playback_preparation: Option<PlaybackPreparation>,
    playback_session: Option<PlaybackSession>,
    playback_caches: HashMap<(String, u32), Arc<PcmCache>>,
    last_pointer_position: Option<[f64; 2]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PlayheadScrub {
    initial: RationalTime,
    layout_epoch: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EditorPlayhead {
    current: RationalTime,
    scrub: Option<PlayheadScrub>,
}

#[derive(Debug, Clone, PartialEq)]
struct PositionGestureBaseline {
    session: u64,
    target: LayerId,
    playhead: RationalTime,
    key: KeyframeId,
    value: [f64; 2],
    position: DocParam,
}

impl Default for EditorPlayhead {
    fn default() -> Self {
        Self {
            current: RationalTime::ZERO,
            scrub: None,
        }
    }
}

impl EditorPlayhead {
    fn begin(&mut self, layout_epoch: u64, time: RationalTime) -> bool {
        self.scrub = Some(PlayheadScrub {
            initial: self.current,
            layout_epoch,
        });
        self.set(time)
    }

    fn update(&mut self, layout_epoch: u64, time: RationalTime) -> Option<bool> {
        (self.scrub?.layout_epoch == layout_epoch).then(|| self.set(time))
    }

    fn finish(&mut self, layout_epoch: u64) -> bool {
        let Some(scrub) = self.scrub else {
            return false;
        };
        if scrub.layout_epoch != layout_epoch {
            return self.cancel();
        }
        self.scrub = None;
        false
    }

    fn cancel(&mut self) -> bool {
        let Some(scrub) = self.scrub.take() else {
            return false;
        };
        self.set(scrub.initial)
    }

    fn retire(&mut self) -> bool {
        self.scrub.take().is_some()
    }

    fn set(&mut self, time: RationalTime) -> bool {
        if self.current == time {
            return false;
        }
        self.current = time;
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PlaybackLifecycle {
    state: StagePlaybackState,
    generation: u64,
    session_active: bool,
}

impl Default for PlaybackLifecycle {
    fn default() -> Self {
        Self {
            state: StagePlaybackState::Idle,
            generation: 0,
            session_active: false,
        }
    }
}

impl PlaybackLifecycle {
    fn state(self) -> StagePlaybackState {
        self.state
    }

    fn begin_preparing(&mut self) -> Result<u64, ProductPlaybackError> {
        if self.state != StagePlaybackState::Idle || self.session_active {
            return Err(ProductPlaybackError::LifecycleConflict { state: self.state });
        }
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or(ProductPlaybackError::GenerationExhausted)?;
        self.state = StagePlaybackState::Preparing;
        Ok(self.generation)
    }

    fn accepts_preparation(self, generation: u64) -> bool {
        self.state == StagePlaybackState::Preparing && self.generation == generation
    }

    fn activate(&mut self, generation: u64) -> Result<(), ProductPlaybackError> {
        if !self.accepts_preparation(generation) || self.session_active {
            return Err(ProductPlaybackError::LifecycleConflict { state: self.state });
        }
        self.state = StagePlaybackState::Playing;
        self.session_active = true;
        Ok(())
    }

    fn cancel_preparing(&mut self) -> Result<(), ProductPlaybackError> {
        if self.state != StagePlaybackState::Preparing || self.session_active {
            return Err(ProductPlaybackError::LifecycleConflict { state: self.state });
        }
        self.invalidate()
    }

    fn invalidate(&mut self) -> Result<(), ProductPlaybackError> {
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or(ProductPlaybackError::GenerationExhausted)?;
        self.state = StagePlaybackState::Idle;
        self.session_active = false;
        Ok(())
    }
}

struct PlaybackPreparation {
    generation: u64,
    start_frame: u64,
    receiver: mpsc::Receiver<PlaybackPreparationResult>,
}

struct PlaybackPreparationResult {
    generation: u64,
    caches: HashMap<(String, u32), Arc<PcmCache>>,
    program: Result<Arc<AudioProgram>, motolii_audio::AudioError>,
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
    composition_duration: RationalTime,
    preview: Option<TimelineProjection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProductTimelineHit {
    Key { layer: LayerId, key: KeyframeId },
    Left { layer: LayerId },
    Right { layer: LayerId },
    Body { layer: LayerId },
    None,
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
            composition_duration: document.composition.duration,
            preview: None,
        })
    }

    fn render_projection(&self) -> &TimelineProjection {
        self.preview.as_ref().unwrap_or(&self.projection)
    }

    fn set_move_preview(&mut self, request: Option<TimelineMoveRequest>) {
        self.preview = request.and_then(|request| {
            self.projection.preview_move(
                request.layer,
                request.new_start,
                self.composition_duration,
            )
        });
    }

    fn clear_move_preview(&mut self) {
        self.preview = None;
    }

    fn set_trim_preview(
        &mut self,
        document: Option<&motolii_doc::Document>,
    ) -> Result<(), TimelineProjectionError> {
        self.preview = document
            .map(Self::from_document)
            .transpose()?
            .map(|projection| projection.projection);
        Ok(())
    }

    fn hit_test(&self, position: [f64; 2], layout: NativeHostLayout) -> Option<ProductTimelineHit> {
        self.hit_test_pair(position, layout)
            .map(|(_, private_hit)| private_hit)
    }

    fn hit_test_pair(
        &self,
        position: [f64; 2],
        layout: NativeHostLayout,
    ) -> Option<(TimelineHit, ProductTimelineHit)> {
        let time_surface = timeline_time_surface_logical_rect(layout)?;
        if !position.iter().all(|value| value.is_finite())
            || !time_surface.x.is_finite()
            || !time_surface.y.is_finite()
            || !time_surface.width.is_finite()
            || time_surface.width <= 0.0
            || !time_surface.height.is_finite()
            || time_surface.height <= 0.0
            || !self.band_span.is_finite()
            || self.band_span <= 0.0
            || !time_surface.contains(position)
        {
            return None;
        }
        let x = (position[0] - time_surface.x) / time_surface.width;
        let y = ((position[1] - time_surface.y) / time_surface.height) * self.band_span;
        let public_hit = self.render_projection().hit_test(x, y);
        let private_hit = match public_hit {
            TimelineHit::Key { layer, key } => ProductTimelineHit::Key { layer, key },
            TimelineHit::None => ProductTimelineHit::None,
            TimelineHit::Bar { layer } => {
                let Some(bar) = self
                    .render_projection()
                    .bars()
                    .iter()
                    .find(|bar| bar.layer == layer)
                else {
                    return Some((public_hit, ProductTimelineHit::Body { layer }));
                };
                let bar_width = (bar.x_end - bar.x_start) * time_surface.width;
                let bar_height = time_surface.height / self.band_span;
                if !bar.x_start.is_finite()
                    || !bar.x_end.is_finite()
                    || !bar_width.is_finite()
                    || bar_width < 25.0
                    || !bar_height.is_finite()
                    || bar_height < 16.0
                {
                    ProductTimelineHit::Body { layer }
                } else {
                    let bar_left = time_surface.x + bar.x_start * time_surface.width;
                    let local_x = position[0] - bar_left;
                    let edge_width = 15.0_f64.min(bar_width / 4.0);
                    if !bar_left.is_finite() || !local_x.is_finite() {
                        ProductTimelineHit::Body { layer }
                    } else if local_x <= edge_width {
                        ProductTimelineHit::Left { layer }
                    } else if local_x >= bar_width - edge_width {
                        ProductTimelineHit::Right { layer }
                    } else {
                        ProductTimelineHit::Body { layer }
                    }
                }
            }
        };
        Some((public_hit, private_hit))
    }

    fn time_at(&self, position: [f64; 2], layout: NativeHostLayout) -> Option<RationalTime> {
        let time_surface = timeline_time_surface_logical_rect(layout)?;
        if !position.iter().all(|value| value.is_finite()) || !time_surface.contains(position) {
            return None;
        }
        let normalized = (position[0] - time_surface.x) / time_surface.width;
        if !normalized.is_finite() {
            return None;
        }
        let fraction = RationalTime::try_from_decimal_str(&format!("{normalized:.9}")).ok()?;
        self.composition_duration.try_mul(fraction).ok()
    }

    fn ruler_time_at(
        &self,
        position: [f64; 2],
        layout: NativeHostLayout,
        require_ruler_hit: bool,
    ) -> Option<RationalTime> {
        let ruler = timeline_ruler_logical_rect(layout)?;
        if !position.iter().all(|value| value.is_finite())
            || !ruler.x.is_finite()
            || !ruler.y.is_finite()
            || !ruler.width.is_finite()
            || ruler.width <= 0.0
            || !ruler.height.is_finite()
            || ruler.height <= 0.0
        {
            return None;
        }
        let right = ruler.x + ruler.width;
        let bottom = ruler.y + ruler.height;
        if !right.is_finite()
            || !bottom.is_finite()
            || (require_ruler_hit
                && (position[0] < ruler.x
                    || position[0] > right
                    || position[1] < ruler.y
                    || position[1] >= bottom))
        {
            return None;
        }
        let normalized = ((position[0] - ruler.x) / ruler.width).clamp(0.0, 1.0);
        let fraction = RationalTime::try_from_decimal_str(&format!("{normalized:.9}")).ok()?;
        self.composition_duration.try_mul(fraction).ok()
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
            easing_popup: None,
            timeline_tools: None,
            file_drop: None,
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
            current_modifiers: Modifiers::default(),
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
            pending_inspector_commit: None,
            pending_position_commit: None,
            position_gesture: None,
            timeline_move: None,
            timeline_trim: None,
            editor_playhead: EditorPlayhead::default(),
            playback_lifecycle: PlaybackLifecycle::default(),
            playback_preparation: None,
            playback_session: None,
            playback_caches: HashMap::new(),
            last_pointer_position: None,
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
        let file_drop = PlatformFileDrop::new(&window, self.proxy.clone())?;
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
            self.editor_playhead.current,
        )?;
        crate::ui_numeric_trace::emit(format_args!(
            "kind=startup phase=inspector-created phase_ms={:.3}",
            elapsed_ms(inspector_started),
        ));
        let stage_chrome_started = Instant::now();
        let stage_wake_proxy = self.proxy.clone();
        let stage_chrome = StageChromeHostRuntime::new(
            &window,
            &stage_transport_snapshot(
                &self.current_document,
                self.primary,
                self.editor_playhead.current,
            ),
            Arc::new(move || {
                let _ = stage_wake_proxy.send_event(ProductEvent::Wake);
            }),
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
        self.inspector = Some(inspector);
        self.stage_chrome = Some(stage_chrome);
        self.timeline_tools = Some(timeline_tools);
        self.file_drop = Some(file_drop);
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
        self.cancel_editor_playhead("layout-changed")?;
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

    fn process_stage_playback_intents(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), ProductRuntimeError> {
        let intent = {
            let Some(stage_chrome) = &self.stage_chrome else {
                return Ok(());
            };
            stage_chrome.take_playback_intent()?
        };
        if matches!(intent, Some(StagePlaybackToggle)) {
            self.toggle_playback(event_loop)?;
        }
        Ok(())
    }

    fn toggle_playback(&mut self, event_loop: &ActiveEventLoop) -> Result<(), ProductRuntimeError> {
        match self.playback_lifecycle.state() {
            StagePlaybackState::Idle => self.begin_playback_preparation(event_loop),
            StagePlaybackState::Preparing => self.cancel_playback_preparation(event_loop),
            StagePlaybackState::Playing => self.pause_playback(event_loop),
        }
    }

    fn begin_playback_preparation(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), ProductRuntimeError> {
        let start_frame = canonical_playback_start_frame(self.editor_playhead.current)?;
        let generation = self.playback_lifecycle.begin_preparing()?;
        let (sender, receiver) = mpsc::sync_channel(1);
        let document = Arc::clone(&self.current_document);
        let project_root = self.document_runtime.project_root();
        let mut caches = std::mem::take(&mut self.playback_caches);
        let proxy = self.proxy.clone();
        let worker = std::thread::Builder::new()
            .name("motolii-audio-program".to_owned())
            .spawn(move || {
                let program =
                    AudioProgram::from_document(&document, project_root.as_deref(), &mut caches)
                        .map(Arc::new);
                let _ = sender.send(PlaybackPreparationResult {
                    generation,
                    caches,
                    program,
                });
                let _ = proxy.send_event(ProductEvent::Wake);
            });
        if let Err(error) = worker {
            let _ = self.playback_lifecycle.invalidate();
            return Err(ProductPlaybackError::PreparationSpawn(error).into());
        }
        self.playback_preparation = Some(PlaybackPreparation {
            generation,
            start_frame,
            receiver,
        });
        self.publish_stage_transport()?;
        self.request_redraw();
        self.set_idle_control_flow(event_loop);
        Ok(())
    }

    fn cancel_playback_preparation(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), ProductRuntimeError> {
        self.playback_preparation.take();
        self.playback_lifecycle.cancel_preparing()?;
        self.publish_stage_transport()?;
        self.request_redraw();
        self.set_idle_control_flow(event_loop);
        Ok(())
    }

    fn process_playback_preparation(&mut self) -> Result<(), ProductRuntimeError> {
        let Some(preparation) = self.playback_preparation.as_ref() else {
            return Ok(());
        };
        let received = match preparation.receiver.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => return Ok(()),
            Err(mpsc::TryRecvError::Disconnected) => {
                return Err(
                    ProductPlaybackError::PreparationDisconnected(preparation.generation).into(),
                )
            }
        };
        let preparation = self.playback_preparation.take().ok_or(
            ProductPlaybackError::PreparationDisconnected(received.generation),
        )?;
        if !self
            .playback_lifecycle
            .accepts_preparation(received.generation)
            || preparation.generation != received.generation
        {
            return Ok(());
        }
        self.playback_caches = received.caches;
        let program =
            received
                .program
                .map_err(|source| ProductPlaybackError::PreparationFailed {
                    generation: received.generation,
                    source,
                })?;
        if self.playback_session.is_some() {
            return Err(ProductPlaybackError::LifecycleConflict {
                state: self.playback_lifecycle.state(),
            }
            .into());
        }
        let session = PlaybackSession::open_default(
            program,
            preparation.start_frame,
            self.current_document.composition.fps,
            Quality::DRAFT,
            Some(&self.gpu),
        )
        .map_err(ProductPlaybackError::from)?;
        self.playback_lifecycle.activate(received.generation)?;
        self.playback_session = Some(session);
        self.publish_stage_transport()?;
        self.submit_stage_projection()?;
        self.request_redraw();
        Ok(())
    }

    fn pause_playback(&mut self, event_loop: &ActiveEventLoop) -> Result<(), ProductRuntimeError> {
        let time = self
            .playback_session
            .as_ref()
            .ok_or(ProductPlaybackError::LifecycleConflict {
                state: self.playback_lifecycle.state(),
            })?
            .transport()
            .perceptual_time()
            .map_err(ProductPlaybackError::from)?;
        let time = clamp_playback_time(time, self.current_document.composition.duration)?;
        self.playback_session.take();
        self.playback_lifecycle.invalidate()?;
        self.editor_playhead.set(time);
        self.publish_stage_transport()?;
        self.submit_stage_projection()?;
        self.request_redraw();
        self.set_idle_control_flow(event_loop);
        Ok(())
    }

    fn process_playback_clock(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), ProductRuntimeError> {
        if self.playback_lifecycle.state() != StagePlaybackState::Playing {
            return Ok(());
        }
        event_loop.set_control_flow(ControlFlow::Poll);
        let plan = self
            .playback_session
            .as_mut()
            .ok_or(ProductPlaybackError::LifecycleConflict {
                state: self.playback_lifecycle.state(),
            })?
            .transport_mut()
            .next_frame_plan()
            .map_err(ProductPlaybackError::from)?;
        self.adopt_playback_frame_plan(event_loop, plan)
    }

    fn adopt_playback_frame_plan(
        &mut self,
        event_loop: &ActiveEventLoop,
        plan: FramePlan,
    ) -> Result<(), ProductRuntimeError> {
        let duration = self.current_document.composition.duration;
        if duration < RationalTime::ZERO {
            return Err(ProductPlaybackError::NegativeCompositionDuration.into());
        }
        if plan.timeline_time >= duration {
            self.editor_playhead.set(duration);
            self.playback_session.take();
            self.playback_lifecycle.invalidate()?;
            self.publish_stage_transport()?;
            self.submit_stage_projection()?;
            self.request_redraw();
            self.set_idle_control_flow(event_loop);
            return Ok(());
        }
        if plan.timeline_time < RationalTime::ZERO {
            return Err(ProductPlaybackError::NegativePlayhead.into());
        }
        if self.editor_playhead.set(plan.timeline_time) {
            self.publish_stage_transport()?;
            self.submit_stage_projection()?;
            self.request_redraw();
        }
        Ok(())
    }

    fn retire_playback_for_document_change(&mut self) -> Result<(), ProductRuntimeError> {
        match self.playback_lifecycle.state() {
            StagePlaybackState::Idle => {
                if self.playback_preparation.is_some() || self.playback_session.is_some() {
                    return Err(ProductPlaybackError::LifecycleConflict {
                        state: StagePlaybackState::Idle,
                    }
                    .into());
                }
            }
            StagePlaybackState::Preparing => {
                self.playback_preparation.take();
                self.playback_lifecycle.cancel_preparing()?;
            }
            StagePlaybackState::Playing => {
                let time = self
                    .playback_session
                    .as_ref()
                    .ok_or(ProductPlaybackError::LifecycleConflict {
                        state: StagePlaybackState::Playing,
                    })?
                    .transport()
                    .perceptual_time()
                    .map_err(ProductPlaybackError::from)?;
                let time = clamp_playback_time(time, self.current_document.composition.duration)?;
                self.playback_session.take();
                self.playback_lifecycle.invalidate()?;
                self.editor_playhead.set(time);
            }
        }
        Ok(())
    }

    fn stop_playback_for_scrub(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), ProductRuntimeError> {
        match self.playback_lifecycle.state() {
            StagePlaybackState::Idle => Ok(()),
            StagePlaybackState::Preparing => self.cancel_playback_preparation(event_loop),
            StagePlaybackState::Playing => self.pause_playback(event_loop),
        }
    }

    pub(crate) fn poll_browser(&mut self, event_loop: &ActiveEventLoop) {
        if self
            .surface_retry_at
            .is_some_and(|retry_at| Instant::now() >= retry_at)
        {
            self.surface_retry_at = None;
            self.request_redraw();
        }
        if let Err(error) = self.process_stage_playback_intents(event_loop) {
            return self.fail(event_loop, error);
        }
        if let Err(error) = self.process_playback_clock(event_loop) {
            return self.fail(event_loop, error);
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
        self.process_stage_easing_intents(event_loop);
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
                    self.adopt_full_publish(event_loop, published, "place");
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
                if let Err(error) = self.process_stage_playback_intents(event_loop) {
                    self.fail(event_loop, error);
                    return;
                }
                if let Err(error) = self.process_playback_preparation() {
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
                if let Err(error) = self.process_pending_position_commit(event_loop) {
                    self.fail(event_loop, error);
                    return;
                }
                if let Err(error) = self.process_inspector_position_gestures() {
                    self.fail(event_loop, error);
                    return;
                }
                if let Err(error) = self.process_inspector_position_key_intents(event_loop) {
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
            ProductEvent::FileDrop(event) => self.process_file_drop(event_loop, event),
        }
    }

    fn process_file_drop(&mut self, event_loop: &ActiveEventLoop, event: HostFileDropEvent) {
        if event.terminal != HostFileDropTerminal::Perform {
            trace_media_drop_reject("terminal");
            return;
        }
        if self.active_place.is_some() {
            trace_media_drop_reject("internal-place-active");
            return;
        }
        let (ndc, position) =
            match canonical_media_drop_position(self.layout, self.displayed_camera, event.position)
            {
                Ok(position) => position,
                Err(reason) => {
                    trace_media_drop_reject(reason);
                    return;
                }
            };
        let prepared = match crate::media_drop::prepare_media_drop(&event.paths) {
            Ok(prepared) => prepared,
            Err(error) => {
                trace_media_drop_reject(error.reason());
                return;
            }
        };
        let duration = prepared.duration;
        self.document_queue.push_place_media(PlaceMediaRequest {
            position,
            playhead: RationalTime::ZERO,
            asset: prepared.asset,
            duration,
        });
        match self.document_runtime.process_next(
            &mut self.document_queue,
            self.primary,
            self.projection_generation,
        ) {
            Ok(Some(published)) => {
                crate::ui_numeric_trace::emit(format_args!(
                    "kind=media-drop-accept ndc_x={:.6} ndc_y={:.6} canonical_x={:.6} \
                     canonical_y={:.6} duration_num={} duration_den={}",
                    ndc[0],
                    ndc[1],
                    position[0],
                    position[1],
                    duration.num(),
                    duration.den(),
                ));
                self.adopt_full_publish(event_loop, published, "media-drop");
            }
            Ok(None) => trace_media_drop_reject("document-no-op"),
            Err(error @ DocumentEditRuntimeError::AddClipFailed(_)) => {
                trace_media_drop_reject("add-clip-failed");
                self.fail(event_loop, error);
            }
            Err(error) => {
                trace_media_drop_reject("document-edit");
                self.fail(event_loop, error);
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

    pub(crate) fn handle_window_cursor_moved(&mut self, position: [f64; 2]) {
        let Some(window) = &self.window else {
            return;
        };
        let scale = window.scale_factor();
        if !scale.is_finite() || scale <= 0.0 {
            return;
        }
        let logical = [position[0] / scale, position[1] / scale];
        self.last_pointer_position = Some(logical);
        if self.editor_playhead.scrub.is_some() {
            let Some(layout) = self.layout else {
                let _ = self.cancel_editor_playhead("layout-unavailable");
                return;
            };
            let Some(time) = self
                .timeline_projection
                .ruler_time_at(logical, layout, false)
            else {
                let _ = self.cancel_editor_playhead("invalid-ruler-mapping");
                return;
            };
            match self.editor_playhead.update(layout.epoch, time) {
                Some(changed) => {
                    if changed {
                        let _ = self.refresh_editor_playhead();
                    }
                }
                None => {
                    let _ = self.cancel_editor_playhead("layout-epoch-changed");
                }
            }
            return;
        }
        if let (Some(gesture), Some(layout)) = (self.timeline_trim.as_ref(), self.layout) {
            let Some(pointer_time) = self.timeline_projection.time_at(logical, layout) else {
                let _ = self.timeline_projection.set_trim_preview(None);
                self.request_redraw();
                return;
            };
            match gesture.preview(pointer_time) {
                Ok(request) => match self.document_runtime.preview_trim(request) {
                    Ok(Some(document)) => {
                        if self
                            .timeline_projection
                            .set_trim_preview(Some(&document))
                            .is_err()
                        {
                            self.cancel_timeline_trim("projection-failed");
                            return;
                        }
                        self.request_redraw();
                    }
                    Ok(None) | Err(DocumentEditRuntimeError::Command(_)) => {
                        let _ = self.timeline_projection.set_trim_preview(None);
                        self.request_redraw();
                    }
                    Err(_) => self.cancel_timeline_trim("preview-failed"),
                },
                Err(_) => self.cancel_timeline_trim("time-overflow"),
            }
            return;
        }
        let (Some(gesture), Some(layout)) = (self.timeline_move.as_ref(), self.layout) else {
            return;
        };
        let Some(pointer_time) = self.timeline_projection.time_at(logical, layout) else {
            self.timeline_projection.clear_move_preview();
            self.request_redraw();
            return;
        };
        match gesture.preview(pointer_time) {
            Ok(new_start) => {
                self.timeline_projection
                    .set_move_preview(Some(TimelineMoveRequest {
                        layer: gesture.layer(),
                        new_start,
                    }));
                self.request_redraw();
            }
            Err(_) => {
                self.cancel_timeline_move("time-overflow");
            }
        }
    }

    pub(crate) fn handle_window_pointer_phase(
        &mut self,
        event_loop: &ActiveEventLoop,
        phase: InputPhase,
    ) {
        let Some(position) = self.last_pointer_position else {
            return;
        };
        match phase {
            InputPhase::Press => {
                if self.timeline_move.is_some()
                    || self.timeline_trim.is_some()
                    || self.editor_playhead.scrub.is_some()
                {
                    return;
                }
                let Some(layout) = self.layout else {
                    return;
                };
                if let Some(time) = self
                    .timeline_projection
                    .ruler_time_at(position, layout, true)
                {
                    if let Err(error) = self.stop_playback_for_scrub(event_loop) {
                        self.fail(event_loop, error);
                        return;
                    }
                    if let Err(error) = self
                        .input_router
                        .route(NormalizedInput::Phase(InputPhase::DragStart))
                    {
                        self.fail(event_loop, error);
                        return;
                    }
                    let changed = self.editor_playhead.begin(layout.epoch, time);
                    crate::ui_numeric_trace::emit(format_args!(
                        "kind=timeline-playhead state=begin layout_epoch={}",
                        layout.epoch,
                    ));
                    if changed {
                        if let Err(error) = self.refresh_editor_playhead() {
                            self.fail(event_loop, error);
                        }
                    } else {
                        self.request_redraw();
                    }
                    return;
                }
                let Some(hit) = self.timeline_projection.hit_test(position, layout) else {
                    return;
                };
                let Some(pointer_time) = self.timeline_projection.time_at(position, layout) else {
                    return;
                };
                match hit {
                    ProductTimelineHit::Body { layer } => {
                        let Some(initial_start) = find_clip_start(&self.current_document, layer)
                        else {
                            return;
                        };
                        self.timeline_move = Some(TimelineMoveGesture::begin(
                            layer,
                            pointer_time,
                            initial_start,
                            self.projection_generation,
                        ));
                        crate::ui_numeric_trace::emit(format_args!(
                            "kind=timeline-move state=begin layer={} generation={}",
                            layer.get(),
                            self.projection_generation,
                        ));
                    }
                    ProductTimelineHit::Left { layer } | ProductTimelineHit::Right { layer } => {
                        let Some((initial_start, initial_end)) =
                            find_clip_interval(&self.current_document, layer)
                        else {
                            return;
                        };
                        let edge = match hit {
                            ProductTimelineHit::Left { .. } => TimelineTrimEdge::Left,
                            ProductTimelineHit::Right { .. } => TimelineTrimEdge::Right,
                            _ => unreachable!(),
                        };
                        self.timeline_trim = Some(TimelineTrimGesture::begin(
                            layer,
                            edge,
                            pointer_time,
                            initial_start,
                            initial_end,
                            self.projection_generation,
                        ));
                        crate::ui_numeric_trace::emit(format_args!(
                            "kind=timeline-trim state=begin layer={} edge={:?} generation={}",
                            layer.get(),
                            edge,
                            self.projection_generation,
                        ));
                    }
                    ProductTimelineHit::Key { .. } | ProductTimelineHit::None => {}
                }
                if self.timeline_move.is_some() || self.timeline_trim.is_some() {
                    if let Err(error) = self
                        .input_router
                        .route(NormalizedInput::Phase(InputPhase::DragStart))
                    {
                        self.cancel_timeline_move("input-router-error");
                        self.cancel_timeline_trim("input-router-error");
                        self.fail(event_loop, error);
                    }
                }
            }
            InputPhase::Release => {
                if self.editor_playhead.scrub.is_some() {
                    let Some(layout) = self.layout else {
                        if let Err(error) = self.cancel_editor_playhead("layout-unavailable") {
                            self.fail(event_loop, error);
                        }
                        return;
                    };
                    if let Err(error) = self
                        .input_router
                        .route(NormalizedInput::Phase(InputPhase::DragEnd))
                    {
                        self.fail(event_loop, error);
                        return;
                    }
                    if self.editor_playhead.finish(layout.epoch) {
                        if let Err(error) = self.refresh_editor_playhead() {
                            self.fail(event_loop, error);
                        }
                    }
                    self.request_redraw();
                } else if self.timeline_trim.is_some() {
                    self.finish_timeline_trim(event_loop, position);
                } else {
                    self.finish_timeline_move(event_loop, position);
                }
            }
            _ => {}
        }
    }

    pub(crate) fn handle_window_modifiers(&mut self, modifiers: Modifiers) {
        self.current_modifiers = modifiers;
    }

    pub(crate) fn handle_window_ime(&mut self, state: ImeGateState) {
        self.input_router.set_ime_gate(state);
    }

    pub(crate) fn handle_window_key(&mut self, event_loop: &ActiveEventLoop, key: KeyToken) {
        let trigger = EffectiveTrigger::Keyboard {
            key,
            modifiers: self.current_modifiers.clone(),
            phase: InputPhase::Press,
        };
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
        match output {
            RouterOutput::Intent {
                intent: DomainIntent::CancelInFlightGesture,
                ..
            } => {
                self.cancel_timeline_move("escape");
                self.cancel_timeline_trim("escape");
                if let Err(error) = self.cancel_editor_playhead("escape") {
                    self.fail(event_loop, error);
                }
            }
            RouterOutput::CancelCommandIgnored { .. } | RouterOutput::ShortcutSuppressed { .. } => {
            }
            _ => {}
        }
    }

    pub(crate) fn handle_window_safety_interrupt(
        &mut self,
        event_loop: &ActiveEventLoop,
        source: SafetyInterrupt,
    ) {
        let output = match self
            .input_router
            .route(NormalizedInput::SafetyInterrupt(source))
        {
            Ok(output) => output,
            Err(error) => return self.fail(event_loop, error),
        };
        if matches!(output, RouterOutput::SafetyCancel { .. }) {
            self.cancel_timeline_move("window-focus-or-capture-loss");
            self.cancel_timeline_trim("window-focus-or-capture-loss");
            if let Err(error) = self.cancel_editor_playhead("window-focus-or-capture-loss") {
                self.fail(event_loop, error);
            }
        }
    }

    fn refresh_editor_playhead(&mut self) -> Result<(), ProductRuntimeError> {
        let had_position_gesture =
            self.position_gesture.take().is_some() || self.pending_position_commit.take().is_some();
        self.publish_stage_transport()?;
        if let Some(inspector) = &self.inspector {
            inspector.publish(
                &self.current_document,
                self.primary,
                self.active_effect_use,
                self.editor_playhead.current,
            )?;
        }
        if had_position_gesture {
            self.submit_inspector_baseline()?;
        }
        self.submit_stage_projection()?;
        self.request_redraw();
        Ok(())
    }

    fn cancel_editor_playhead(&mut self, reason: &'static str) -> Result<(), ProductRuntimeError> {
        if self.editor_playhead.scrub.is_none() {
            return Ok(());
        }
        self.input_router
            .route(NormalizedInput::Phase(InputPhase::Cancel))?;
        let changed = self.editor_playhead.cancel();
        if changed {
            self.refresh_editor_playhead()?;
        } else {
            self.request_redraw();
        }
        crate::ui_numeric_trace::emit(format_args!(
            "kind=timeline-playhead state=cancel reason={}",
            reason,
        ));
        Ok(())
    }

    fn retire_editor_playhead(&mut self, reason: &'static str) {
        if self.editor_playhead.retire() {
            let _ = self
                .input_router
                .route(NormalizedInput::Phase(InputPhase::Cancel));
            crate::ui_numeric_trace::emit(format_args!(
                "kind=timeline-playhead state=retire reason={}",
                reason,
            ));
        }
    }

    fn cancel_timeline_move(&mut self, reason: &'static str) {
        if let Some(gesture) = self.timeline_move.take() {
            let _ = self
                .input_router
                .route(NormalizedInput::Phase(InputPhase::Cancel));
            crate::ui_numeric_trace::emit(format_args!(
                "kind=timeline-move state=cancel layer={} generation={} reason={}",
                gesture.layer().get(),
                gesture.generation(),
                reason,
            ));
            self.timeline_projection.clear_move_preview();
            self.request_redraw();
        }
    }

    fn cancel_timeline_trim(&mut self, reason: &'static str) {
        if let Some(gesture) = self.timeline_trim.take() {
            let _ = self
                .input_router
                .route(NormalizedInput::Phase(InputPhase::Cancel));
            crate::ui_numeric_trace::emit(format_args!(
                "kind=timeline-trim state=cancel layer={} edge={:?} generation={} reason={}",
                gesture.layer().get(),
                gesture.edge(),
                gesture.generation(),
                reason,
            ));
            let _ = self.timeline_projection.set_trim_preview(None);
            self.request_redraw();
        }
    }

    fn finish_timeline_move(&mut self, event_loop: &ActiveEventLoop, position: [f64; 2]) {
        let Some(gesture) = self.timeline_move.take() else {
            return;
        };
        if let Err(error) = self
            .input_router
            .route(NormalizedInput::Phase(InputPhase::DragEnd))
        {
            return self.fail(event_loop, error);
        }
        self.timeline_projection.clear_move_preview();
        let Some(layout) = self.layout else {
            return;
        };
        let Some(pointer_time) = self.timeline_projection.time_at(position, layout) else {
            self.request_redraw();
            return;
        };
        if gesture.generation() != self.projection_generation
            || find_clip_start(&self.current_document, gesture.layer())
                != Some(gesture.initial_start())
        {
            self.request_redraw();
            return;
        }
        let request = match gesture.release(pointer_time) {
            Ok(Some(request)) => request,
            Ok(None) => {
                self.request_redraw();
                return;
            }
            Err(_) => {
                self.request_redraw();
                return;
            }
        };
        self.document_queue.push_move_clip(request);
        match self.document_runtime.process_next(
            &mut self.document_queue,
            self.primary,
            self.projection_generation,
        ) {
            Ok(Some(published)) => self.adopt_full_publish(event_loop, published, "timeline-move"),
            Ok(None) => self.request_redraw(),
            Err(DocumentEditRuntimeError::Command(_)) => self.request_redraw(),
            Err(error) => self.fail(event_loop, error),
        }
    }

    fn finish_timeline_trim(&mut self, event_loop: &ActiveEventLoop, position: [f64; 2]) {
        let Some(gesture) = self.timeline_trim.take() else {
            return;
        };
        if let Err(error) = self
            .input_router
            .route(NormalizedInput::Phase(InputPhase::DragEnd))
        {
            return self.fail(event_loop, error);
        }
        let _ = self.timeline_projection.set_trim_preview(None);
        let Some(layout) = self.layout else {
            return;
        };
        let Some(pointer_time) = self.timeline_projection.time_at(position, layout) else {
            self.request_redraw();
            return;
        };
        if !timeline_trim_gesture_is_current(
            &gesture,
            self.projection_generation,
            find_clip_interval(&self.current_document, gesture.layer()),
        ) {
            self.request_redraw();
            return;
        }
        let request = match gesture.release(pointer_time) {
            Ok(Some(request)) => request,
            Ok(None) | Err(_) => {
                self.request_redraw();
                return;
            }
        };
        self.document_queue.push_trim_clip(request);
        match self.document_runtime.process_next(
            &mut self.document_queue,
            self.primary,
            self.projection_generation,
        ) {
            Ok(Some(published)) => self.adopt_full_publish(event_loop, published, "timeline-trim"),
            Ok(None) | Err(DocumentEditRuntimeError::Command(_)) => self.request_redraw(),
            Err(error) => self.fail(event_loop, error),
        }
    }

    fn set_idle_control_flow(&self, event_loop: &ActiveEventLoop) {
        if self.playback_lifecycle.state() == StagePlaybackState::Playing {
            event_loop.set_control_flow(ControlFlow::Poll);
            return;
        }
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
        let hit_pair = self.timeline_projection.hit_test_pair(position, layout);
        crate::ui_numeric_trace::emit(format_args!(
            "kind=timeline-hit layout_epoch={} logical_x={:.3} logical_y={:.3} hit={:?}",
            layout.epoch,
            position[0],
            position[1],
            hit_pair.as_ref().map(|(public_hit, _)| public_hit),
        ));
        let Some((public_hit, _)) = hit_pair else {
            return;
        };
        self.browser_focus_target = BrowserFocusTarget::Parent;
        if let Some(browser) = &self.browser {
            if let Err(error) = browser.ensure_focus(self.browser_focus_target) {
                return self.fail(event_loop, error);
            }
        }
        match public_hit {
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
                if let Err(error) = self.publish_stage_transport() {
                    return self.fail(event_loop, error);
                }
                if let Some(inspector) = &self.inspector {
                    if let Err(error) = inspector.publish(
                        &self.current_document,
                        self.primary,
                        self.active_effect_use,
                        self.editor_playhead.current,
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

    fn inspector_render_request(&self, document: Arc<motolii_doc::Document>) -> RenderRequest {
        RenderRequest {
            document,
            data_tracks: Arc::clone(&self.render_request_template.data_tracks),
            evaluation_time: EvaluationTime::new(self.editor_playhead.current),
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

    fn process_inspector_position_gestures(&mut self) -> Result<(), ProductRuntimeError> {
        if self.pending_position_commit.is_some() {
            return Ok(());
        }
        let Some(inspector) = &self.inspector else {
            return Ok(());
        };
        let document = Arc::clone(&self.current_document);
        if let Some(start) = inspector
            .take_position_start()
            .map_err(InspectorHostRuntimeError::from)
            .map_err(ProductRuntimeError::from)?
        {
            self.position_gesture = position_gesture_baseline(
                &document,
                self.primary,
                self.editor_playhead.current,
                start,
            );
        }

        let mut needs_baseline = false;
        let mut pending_commit = None;
        loop {
            let Some(terminal) = inspector
                .take_position_terminal()
                .map_err(InspectorHostRuntimeError::from)
                .map_err(ProductRuntimeError::from)?
            else {
                break;
            };
            match terminal.cause {
                InspectorPositionGestureTerminalCause::Cancel => {
                    self.position_gesture = None;
                    needs_baseline = true;
                }
                InspectorPositionGestureTerminalCause::Commit(value) => {
                    let Some(baseline) = self.position_gesture.as_ref() else {
                        needs_baseline = true;
                        continue;
                    };
                    if baseline.session != terminal.session {
                        self.position_gesture = None;
                        needs_baseline = true;
                        continue;
                    }
                    match resolve_position_gesture_command(
                        &document,
                        self.primary,
                        self.editor_playhead.current,
                        baseline,
                        terminal.axis,
                        value,
                    ) {
                        Some(command) => {
                            pending_commit = Some((terminal, command));
                            break;
                        }
                        None => {
                            self.position_gesture = None;
                            needs_baseline = true;
                        }
                    }
                }
            }
        }

        if let Some((terminal, command)) = pending_commit {
            self.pending_position_commit = Some(terminal);
            if needs_baseline {
                self.submit_inspector_baseline()?;
            }
            self.submit_inspector_preview(Arc::clone(&document), command)?;
            return Ok(());
        }

        let update = inspector
            .take_latest_position_update()
            .map_err(InspectorHostRuntimeError::from)
            .map_err(ProductRuntimeError::from)?;
        let Some(update) = update else {
            if needs_baseline {
                self.submit_inspector_baseline()?;
            }
            return Ok(());
        };
        let preview = self
            .position_gesture
            .as_ref()
            .and_then(|baseline| {
                (baseline.session == update.session).then(|| {
                    resolve_position_gesture_command(
                        &document,
                        self.primary,
                        self.editor_playhead.current,
                        baseline,
                        update.axis,
                        update.value,
                    )
                })
            })
            .flatten();
        if preview.is_none() {
            self.position_gesture = None;
            self.submit_inspector_baseline()?;
            return Ok(());
        }
        self.submit_inspector_preview(document, preview.expect("checked preview"))?;
        Ok(())
    }

    fn process_pending_position_commit(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), ProductRuntimeError> {
        let Some(terminal) = self.pending_position_commit.take() else {
            return Ok(());
        };
        let InspectorPositionGestureTerminalCause::Commit(value) = terminal.cause else {
            self.position_gesture = None;
            self.submit_inspector_baseline()?;
            return Ok(());
        };
        let document = Arc::clone(&self.current_document);
        let Some(baseline) = self.position_gesture.as_ref() else {
            self.submit_inspector_baseline()?;
            return Ok(());
        };
        let Some(command) = resolve_position_gesture_command(
            &document,
            self.primary,
            self.editor_playhead.current,
            baseline,
            terminal.axis,
            value,
        ) else {
            self.position_gesture = None;
            self.submit_inspector_baseline()?;
            return Ok(());
        };
        let Command::SetPositionKeyValue {
            target,
            key,
            old,
            new,
        } = command
        else {
            self.position_gesture = None;
            self.submit_inspector_baseline()?;
            return Ok(());
        };
        self.document_queue
            .push_set_position_key_value(SetPositionKeyValueRequest {
                target,
                key,
                old,
                new,
            });
        match self.document_runtime.process_next(
            &mut self.document_queue,
            self.primary,
            self.projection_generation,
        )? {
            Some(published) => {
                self.position_gesture = None;
                self.adopt_full_publish(event_loop, published, "inspector-position-key-value");
            }
            None => {
                self.position_gesture = None;
                self.submit_inspector_baseline()?;
            }
        }
        Ok(())
    }

    fn process_inspector_position_key_intents(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), ProductRuntimeError> {
        loop {
            let sequence = {
                let Some(inspector) = &self.inspector else {
                    return Ok(());
                };
                inspector
                    .take_add_position_key_intent()
                    .map_err(InspectorHostRuntimeError::from)
                    .map_err(ProductRuntimeError::from)?
            };
            let Some(sequence) = sequence else {
                return Ok(());
            };
            let Some(target) = self.primary else {
                continue;
            };
            let request = AddPositionKeyRequest {
                target,
                time: self.editor_playhead.current,
            };
            crate::ui_numeric_trace::emit(format_args!(
                "kind=inspector-position-key sequence={} target={:?} time={:?}",
                sequence, request.target, request.time,
            ));
            self.document_queue.push_add_position_key(request);
            if let Some(published) = self.document_runtime.process_next(
                &mut self.document_queue,
                self.primary,
                self.projection_generation,
            )? {
                self.adopt_full_publish(event_loop, published, "inspector-add-position-key");
            }
        }
    }

    fn process_stage_easing_intents(&mut self, event_loop: &ActiveEventLoop) {
        let intent = {
            let Some(stage_chrome) = &mut self.stage_chrome else {
                return;
            };
            if let Err(error) = stage_chrome.sync_easing_layout() {
                crate::ui_numeric_trace::emit(format_args!(
                    "kind=easing-popup event=layout-sync-rejected reason={error}",
                ));
                return;
            }
            match stage_chrome.take_easing_intent() {
                Ok(intent) => intent,
                Err(error) => {
                    crate::ui_numeric_trace::emit(format_args!(
                        "kind=easing-popup event=inbound-rejected reason={error}",
                    ));
                    return;
                }
            }
        };
        let Some(StageEasingIntent {
            anchor,
            layout_epoch,
        }) = intent
        else {
            return;
        };
        let current_interval = position_active_interval(
            &self.current_document,
            self.primary,
            self.editor_playhead.current,
        );
        let interval = match admit_easing_open(
            self.easing_popup.is_some(),
            self.layout.map(|layout| layout.epoch),
            layout_epoch,
            current_interval,
        ) {
            Ok(interval) => interval,
            Err(reason) => {
                crate::ui_numeric_trace::emit(format_args!(
                    "kind=easing-popup event=open-rejected reason={reason:?} layout_epoch={layout_epoch}",
                ));
                return;
            }
        };
        let (window, gfx, transport) = match (&self.window, &self.gfx, self.layout) {
            (Some(window), Some(gfx), Some(layout)) => (window, gfx, layout.stage_transport),
            _ => {
                crate::ui_numeric_trace::emit(format_args!(
                    "kind=easing-popup event=open-rejected reason=host-unavailable layout_epoch={layout_epoch}",
                ));
                return;
            }
        };
        match ProductEasingPopup::open(
            event_loop,
            ProductEasingPopupOpen {
                host: window,
                instance: &gfx.instance,
                adapter: &gfx.adapter,
                gpu: Arc::clone(&self.gpu),
                transport,
                anchor,
                interp: interval.left_interp,
            },
        ) {
            Ok(popup) => {
                crate::ui_numeric_trace::emit(format_args!(
                    "kind=easing-popup event=open layout_epoch={layout_epoch} generation={}",
                    self.projection_generation,
                ));
                self.easing_popup =
                    Some((popup, interval, self.projection_generation, layout_epoch));
            }
            Err(error) => {
                crate::ui_numeric_trace::emit(format_args!(
                    "kind=easing-popup event=open-rejected reason={error}",
                ));
            }
        }
    }

    pub(crate) fn handle_easing_popup_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let terminal = {
            let Some((popup, _, _, _)) = &mut self.easing_popup else {
                return;
            };
            if popup.window_id() != window_id {
                return;
            }
            popup.handle_event(&event)
        };
        let terminal = match terminal {
            Ok(terminal) => terminal,
            Err(error) => {
                self.easing_popup = None;
                crate::ui_numeric_trace::emit(format_args!(
                    "kind=easing-popup event=closed reason=render-error error={error}",
                ));
                return;
            }
        };
        let Some(terminal) = terminal else {
            return;
        };
        let Some((_, expected_interval, expected_generation, expected_layout_epoch)) =
            self.easing_popup.take()
        else {
            crate::ui_numeric_trace::emit(format_args!(
                "kind=easing-popup event=terminal-rejected reason=closed",
            ));
            return;
        };
        let PopupTerminal::Commit(interp) = terminal else {
            crate::ui_numeric_trace::emit(format_args!(
                "kind=easing-popup event=cancel layout_epoch={expected_layout_epoch}",
            ));
            return;
        };
        let current_interval = position_active_interval(
            &self.current_document,
            self.primary,
            self.editor_playhead.current,
        );
        let request = match admit_easing_terminal(
            self.projection_generation,
            self.layout.map(|layout| layout.epoch),
            current_interval.as_ref(),
            expected_generation,
            expected_layout_epoch,
            &expected_interval,
            interp,
        ) {
            Ok(request) => request,
            Err(reason) => {
                crate::ui_numeric_trace::emit(format_args!(
                    "kind=easing-popup event=terminal-rejected reason={reason:?} layout_epoch={expected_layout_epoch}",
                ));
                return;
            }
        };
        self.document_queue.push_set_position_key_interp(request);
        match self.document_runtime.process_next(
            &mut self.document_queue,
            self.primary,
            self.projection_generation,
        ) {
            Ok(Some(published)) => {
                crate::ui_numeric_trace::emit(format_args!(
                    "kind=easing-popup event=commit layout_epoch={expected_layout_epoch}",
                ));
                self.adopt_full_publish(event_loop, published, "stage-easing");
            }
            Ok(None) => {
                crate::ui_numeric_trace::emit(format_args!(
                    "kind=easing-popup event=terminal-rejected reason=d2-noop layout_epoch={expected_layout_epoch}",
                ));
            }
            Err(error) => self.fail(event_loop, error),
        }
    }

    pub(crate) fn easing_popup_window_id(&self) -> Option<winit::window::WindowId> {
        self.easing_popup
            .as_ref()
            .map(|(popup, _, _, _)| popup.window_id())
    }

    pub(crate) fn primary_window_id(&self) -> Option<winit::window::WindowId> {
        self.window.as_ref().map(|window| window.id())
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
        if !matches!(
            published.kind,
            crate::document_edit_runtime::DocumentEditActionKind::ReplacePrimary
                | crate::document_edit_runtime::DocumentEditActionKind::ClearPrimary
        ) {
            if let Err(error) = self.retire_playback_for_document_change() {
                return self.fail(event_loop, error);
            }
        }
        self.cancel_timeline_move("published-generation-changed");
        self.cancel_timeline_trim("published-generation-changed");
        self.position_gesture = None;
        self.pending_position_commit = None;
        self.retire_editor_playhead("published-generation-changed");
        trace_document_publish(route, &published);
        self.reconcile_active_effect_use(&published);
        self.current_document = published.snapshot;
        self.primary = published.primary;
        self.projection_generation = published.projection_generation;
        if let Err(error) = self.publish_stage_transport() {
            return self.fail(event_loop, error);
        }
        if let Some(inspector) = &self.inspector {
            if let Err(error) = inspector.publish(
                &self.current_document,
                self.primary,
                self.active_effect_use,
                self.editor_playhead.current,
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

    fn publish_stage_transport(&self) -> Result<(), ProductRuntimeError> {
        if let Some(stage_chrome) = &self.stage_chrome {
            publish_stage_transport_snapshot_with_state(
                &self.current_document,
                self.primary,
                self.editor_playhead.current,
                self.playback_lifecycle.state(),
                |snapshot| stage_chrome.publish(snapshot),
            )?;
        }
        Ok(())
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
            NativeTimelineRenderState {
                primary: self.primary,
                playhead: self.editor_playhead.current,
            },
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

fn find_clip_start(document: &motolii_doc::Document, target: LayerId) -> Option<RationalTime> {
    fn find(items: &[TrackItem], target: LayerId) -> Option<RationalTime> {
        for item in items {
            match item {
                TrackItem::Clip(clip) if clip.envelope.layer_id == target => {
                    return Some(clip.start);
                }
                TrackItem::Group(group) => {
                    if let Some(start) = find(&group.children, target) {
                        return Some(start);
                    }
                }
                TrackItem::Clip(_) => {}
            }
        }
        None
    }

    document
        .tracks
        .iter()
        .find_map(|track| find(&track.items, target))
}

fn find_clip_interval(
    document: &motolii_doc::Document,
    target: LayerId,
) -> Option<(RationalTime, RationalTime)> {
    fn find(items: &[TrackItem], target: LayerId) -> Option<(RationalTime, RationalTime)> {
        for item in items {
            match item {
                TrackItem::Clip(clip) if clip.envelope.layer_id == target => {
                    return Some((clip.start, clip.start.try_add(clip.duration).ok()?));
                }
                TrackItem::Group(group) => {
                    if let Some(interval) = find(&group.children, target) {
                        return Some(interval);
                    }
                }
                TrackItem::Clip(_) => {}
            }
        }
        None
    }

    document
        .tracks
        .iter()
        .find_map(|track| find(&track.items, target))
}

fn timeline_trim_gesture_is_current(
    gesture: &TimelineTrimGesture,
    projection_generation: u64,
    interval: Option<(RationalTime, RationalTime)>,
) -> bool {
    gesture.generation() == projection_generation
        && interval == Some((gesture.initial_start(), gesture.initial_end()))
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
    let base = product_builtin_keymap()?;
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

fn product_builtin_keymap() -> Result<BuiltinKeymap, ProductRuntimeError> {
    let primary = Modifiers::try_new([Modifier::Primary])?;
    let primary_shift = Modifiers::try_new([Modifier::Primary, Modifier::Shift])?;
    let z = KeyToken::Ascii(AsciiKey::try_new('z')?);
    Ok(BuiltinKeymap::new(
        2,
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
            Binding {
                gesture: Gesture::Keyboard {
                    key: KeyToken::Escape,
                    modifiers: Modifiers::default(),
                    phase: InputPhase::Press,
                },
                command: CommandId::try_new("motolii.gesture.cancel")?,
            },
        ],
    ))
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

#[derive(Debug, Clone, PartialEq)]
struct PositionActiveInterval {
    layer: LayerId,
    left_id: KeyframeId,
    left_t: RationalTime,
    right_id: KeyframeId,
    right_t: RationalTime,
    left_interp: motolii_eval::Interp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EasingOpenReject {
    PopupActive,
    LayoutEpochMismatch,
    NoActiveInterval,
}

fn admit_easing_open(
    popup_active: bool,
    current_layout_epoch: Option<u64>,
    intent_layout_epoch: u64,
    interval: Option<PositionActiveInterval>,
) -> Result<PositionActiveInterval, EasingOpenReject> {
    if popup_active {
        return Err(EasingOpenReject::PopupActive);
    }
    if current_layout_epoch != Some(intent_layout_epoch) {
        return Err(EasingOpenReject::LayoutEpochMismatch);
    }
    interval.ok_or(EasingOpenReject::NoActiveInterval)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EasingTerminalReject {
    GenerationMismatch,
    LayoutEpochMismatch,
    IntervalMismatch,
    SameValue,
}

fn admit_easing_terminal(
    current_generation: u64,
    current_layout_epoch: Option<u64>,
    current_interval: Option<&PositionActiveInterval>,
    expected_generation: u64,
    expected_layout_epoch: u64,
    expected_interval: &PositionActiveInterval,
    interp: motolii_eval::Interp,
) -> Result<SetPositionKeyInterpRequest, EasingTerminalReject> {
    if current_generation != expected_generation {
        return Err(EasingTerminalReject::GenerationMismatch);
    }
    if current_layout_epoch != Some(expected_layout_epoch) {
        return Err(EasingTerminalReject::LayoutEpochMismatch);
    }
    if current_interval != Some(expected_interval) {
        return Err(EasingTerminalReject::IntervalMismatch);
    }
    if expected_interval.left_interp == interp {
        return Err(EasingTerminalReject::SameValue);
    }
    Ok(SetPositionKeyInterpRequest {
        target: expected_interval.layer,
        key: expected_interval.left_id,
        interp,
    })
}

fn stage_transport_snapshot(
    document: &motolii_doc::Document,
    primary: Option<LayerId>,
    playhead: RationalTime,
) -> StageTransportSnapshot {
    stage_transport_snapshot_with_state(document, primary, playhead, StagePlaybackState::Idle)
}

fn stage_transport_snapshot_with_state(
    document: &motolii_doc::Document,
    primary: Option<LayerId>,
    playhead: RationalTime,
    playback_state: StagePlaybackState,
) -> StageTransportSnapshot {
    let object_name = position_active_interval(document, primary, playhead).and_then(|interval| {
        document
            .layers
            .display_name(interval.layer)
            .filter(|name| !name.is_empty())
            .map(str::to_owned)
    });
    if playback_state == StagePlaybackState::Idle {
        StageTransportSnapshot::with_position_active_interval(object_name)
    } else {
        StageTransportSnapshot::with_position_active_interval_and_state(object_name, playback_state)
    }
}

fn canonical_playback_start_frame(current: RationalTime) -> Result<u64, ProductPlaybackError> {
    if current < RationalTime::ZERO {
        return Err(ProductPlaybackError::NegativePlayhead);
    }
    let canonical_fps = Fps::try_new(CANONICAL_SAMPLE_RATE as i64, 1)?;
    let frame = current.try_to_frame_floor(canonical_fps)?;
    u64::try_from(frame).map_err(|_| ProductPlaybackError::StartFrameOverflow)
}

fn clamp_playback_time(
    time: RationalTime,
    duration: RationalTime,
) -> Result<RationalTime, ProductPlaybackError> {
    if duration < RationalTime::ZERO {
        return Err(ProductPlaybackError::NegativeCompositionDuration);
    }
    if time < RationalTime::ZERO {
        return Err(ProductPlaybackError::NegativePlayhead);
    }
    Ok(time.min(duration))
}

#[allow(dead_code)]
fn publish_stage_transport_snapshot<E>(
    document: &motolii_doc::Document,
    primary: Option<LayerId>,
    playhead: RationalTime,
    publish: impl FnOnce(&StageTransportSnapshot) -> Result<(), E>,
) -> Result<(), E> {
    publish_stage_transport_snapshot_with_state(
        document,
        primary,
        playhead,
        StagePlaybackState::Idle,
        publish,
    )
}

fn publish_stage_transport_snapshot_with_state<E>(
    document: &motolii_doc::Document,
    primary: Option<LayerId>,
    playhead: RationalTime,
    playback_state: StagePlaybackState,
    publish: impl FnOnce(&StageTransportSnapshot) -> Result<(), E>,
) -> Result<(), E> {
    let snapshot = stage_transport_snapshot_with_state(document, primary, playhead, playback_state);
    publish(&snapshot)
}

fn position_gesture_baseline(
    document: &motolii_doc::Document,
    primary: Option<LayerId>,
    playhead: RationalTime,
    start: InspectorPositionGestureStart,
) -> Option<PositionGestureBaseline> {
    let target = primary?;
    let (key, value, position) = position_key_value_at(document, Some(target), playhead)?;
    if axis_value(value, start.axis) != start.value {
        return None;
    }
    Some(PositionGestureBaseline {
        session: start.session,
        target,
        playhead,
        key,
        value,
        position,
    })
}

fn resolve_position_gesture_command(
    document: &motolii_doc::Document,
    primary: Option<LayerId>,
    playhead: RationalTime,
    baseline: &PositionGestureBaseline,
    axis: InspectorPositionAxis,
    value: f64,
) -> Option<Command> {
    if !value.is_finite() || primary != Some(baseline.target) || playhead != baseline.playhead {
        return None;
    }
    let (key, current, position) =
        position_key_value_at(document, Some(baseline.target), baseline.playhead)?;
    if key != baseline.key || current != baseline.value || position != baseline.position {
        return None;
    }
    let new = match axis {
        InspectorPositionAxis::X => [value, baseline.value[1]],
        InspectorPositionAxis::Y => [baseline.value[0], value],
    };
    (new != baseline.value).then_some(Command::SetPositionKeyValue {
        target: baseline.target,
        key: baseline.key,
        old: baseline.value,
        new,
    })
}

fn axis_value(value: [f64; 2], axis: InspectorPositionAxis) -> f64 {
    match axis {
        InspectorPositionAxis::X => value[0],
        InspectorPositionAxis::Y => value[1],
    }
}

fn position_key_value_at(
    document: &motolii_doc::Document,
    primary: Option<LayerId>,
    playhead: RationalTime,
) -> Option<(KeyframeId, [f64; 2], DocParam)> {
    fn find_envelope(items: &[TrackItem], target: LayerId) -> Option<&motolii_doc::ItemEnvelope> {
        for item in items {
            match item {
                TrackItem::Clip(clip) if clip.envelope.layer_id == target => {
                    return Some(&clip.envelope);
                }
                TrackItem::Group(group) if group.envelope.layer_id == target => {
                    return Some(&group.envelope);
                }
                TrackItem::Group(group) => {
                    if let Some(envelope) = find_envelope(&group.children, target) {
                        return Some(envelope);
                    }
                }
                TrackItem::Clip(_) => {}
            }
        }
        None
    }

    let target = primary?;
    let envelope = document
        .tracks
        .iter()
        .find_map(|track| find_envelope(&track.items, target))?;
    let DocParam::Keyframes(track) = &envelope.transform.position else {
        return None;
    };
    let keys = track.keys();
    if keys.is_empty()
        || track.validate().is_err()
        || keys.iter().any(|key| {
            !matches!(key.value, DocValue::Vec2(value) if value.iter().all(|value| value.is_finite()))
        })
    {
        return None;
    }
    let key = keys.iter().find(|key| key.t == playhead)?;
    let DocValue::Vec2(value) = key.value else {
        return None;
    };
    Some((key.id, value, envelope.transform.position.clone()))
}

fn position_active_interval(
    document: &motolii_doc::Document,
    primary: Option<LayerId>,
    playhead: RationalTime,
) -> Option<PositionActiveInterval> {
    fn find_envelope(items: &[TrackItem], target: LayerId) -> Option<&motolii_doc::ItemEnvelope> {
        for item in items {
            match item {
                TrackItem::Clip(clip) if clip.envelope.layer_id == target => {
                    return Some(&clip.envelope);
                }
                TrackItem::Group(group) if group.envelope.layer_id == target => {
                    return Some(&group.envelope);
                }
                TrackItem::Group(group) => {
                    if let Some(envelope) = find_envelope(&group.children, target) {
                        return Some(envelope);
                    }
                }
                TrackItem::Clip(_) => {}
            }
        }
        None
    }

    let layer = primary?;
    let envelope = document
        .tracks
        .iter()
        .find_map(|track| find_envelope(&track.items, layer))?;
    let DocParam::Keyframes(track) = &envelope.transform.position else {
        return None;
    };
    let keys = track.keys();
    if keys.len() < 2
        || track.validate().is_err()
        || keys
            .iter()
            .any(|key| !matches!(key.value, DocValue::Vec2(_)))
    {
        return None;
    }
    keys.windows(2).find_map(|pair| {
        let [left, right] = pair else {
            return None;
        };
        (left.t < playhead && playhead < right.t).then_some(PositionActiveInterval {
            layer,
            left_id: left.id,
            left_t: left.t,
            right_id: right.id,
            right_t: right.t,
            left_interp: left.interp,
        })
    })
}

struct ProductSurface {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
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
            instance: parts.instance,
            adapter: parts.adapter,
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
        timeline_state: NativeTimelineRenderState,
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
            timeline_projection.render_projection(),
            timeline_state,
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

fn canonical_media_drop_position(
    layout: Option<NativeHostLayout>,
    camera: motolii_core::CompCamera,
    position: [f64; 2],
) -> Result<([f64; 2], [f64; 2]), &'static str> {
    let layout = layout.ok_or("layout-unavailable")?;
    let ndc = layout.stage_ndc(position).ok_or("outside-stage")?;
    let canonical = canonical_drop_from_ndc(camera, ndc).ok_or("canonical-conversion")?;
    Ok((ndc, canonical))
}

fn trace_media_drop_reject(reason: &str) {
    crate::ui_numeric_trace::emit(format_args!("kind=media-drop-reject reason={reason}"));
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

pub(crate) fn create_preview_pipeline(
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
pub(crate) enum ProductPlaybackError {
    #[error("playback lifecycle conflict in {state:?} state")]
    LifecycleConflict { state: StagePlaybackState },
    #[error("playback preparation generation is exhausted")]
    GenerationExhausted,
    #[error("playhead time is negative")]
    NegativePlayhead,
    #[error("playback start frame overflows the canonical sample index")]
    StartFrameOverflow,
    #[error("composition duration is negative")]
    NegativeCompositionDuration,
    #[error("playback preparation worker failed to start")]
    PreparationSpawn(#[source] std::io::Error),
    #[error("playback preparation result for generation {0} disconnected")]
    PreparationDisconnected(u64),
    #[error("playback preparation failed for generation {generation}: {source}")]
    PreparationFailed {
        generation: u64,
        #[source]
        source: motolii_audio::AudioError,
    },
    #[error(transparent)]
    Audio(#[from] motolii_audio::AudioError),
    #[error(transparent)]
    Session(#[from] PlaybackSessionError),
    #[error(transparent)]
    Transport(#[from] TransportError),
    #[error(transparent)]
    Fps(#[from] FpsError),
    #[error(transparent)]
    Time(#[from] RationalTimeError),
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
    StageEasingIntent(#[from] StageEasingIntentError),
    #[error(transparent)]
    StagePlaybackIntent(#[from] StagePlaybackIntentError),
    #[error(transparent)]
    Playback(#[from] ProductPlaybackError),
    #[error(transparent)]
    EasingPopup(#[from] ProductEasingPopupError),
    #[error(transparent)]
    TimelineTools(#[from] TimelineToolsHostRuntimeError),
    #[error(transparent)]
    NativeTimeline(#[from] NativeTimelineRendererError),
    #[error(transparent)]
    FileDrop(#[from] PlatformFileDropError),
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
    use motolii_audio::{DeviceWaitLatency, PlaybackCounters};
    use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat};
    use motolii_doc::{DocKeyframe, DocKeyframeTrack};
    use motolii_transport::Transport;

    #[test]
    fn playback_lifecycle_cancels_stale_preparation_and_allows_one_session() {
        let mut lifecycle = PlaybackLifecycle::default();
        let stale_generation = lifecycle.begin_preparing().unwrap();
        assert!(lifecycle.accepts_preparation(stale_generation));
        lifecycle.cancel_preparing().unwrap();
        assert!(!lifecycle.accepts_preparation(stale_generation));

        let active_generation = lifecycle.begin_preparing().unwrap();
        lifecycle.activate(active_generation).unwrap();
        assert_eq!(lifecycle.state(), StagePlaybackState::Playing);
        assert!(lifecycle.activate(active_generation).is_err());
        lifecycle.invalidate().unwrap();
        assert_eq!(lifecycle.state(), StagePlaybackState::Idle);
        assert!(!lifecycle.session_active);
    }

    #[test]
    fn canonical_playback_start_frame_uses_exact_zero_and_sample_rate_conversion() {
        assert_eq!(
            canonical_playback_start_frame(RationalTime::ZERO).unwrap(),
            0
        );
        assert_eq!(
            canonical_playback_start_frame(RationalTime::try_new(1, 24).unwrap()).unwrap(),
            2_000
        );
        assert_eq!(
            canonical_playback_start_frame(RationalTime::try_new(1, 48_000).unwrap()).unwrap(),
            1
        );
        assert!(matches!(
            canonical_playback_start_frame(RationalTime::try_new(-1, 48_000).unwrap()),
            Err(ProductPlaybackError::NegativePlayhead)
        ));
    }

    #[test]
    fn transport_reports_absolute_time_and_repeats_without_counter_advance() {
        for sample_rate in [48_000_u32, 44_100_u32] {
            let counters = Arc::new(PlaybackCounters::default());
            counters.advance_supplied_for_simulation(u64::from(sample_rate) * 2);
            let mut transport = Transport::new(
                counters,
                Arc::new(DeviceWaitLatency::default()),
                Fps::try_new(30, 1).unwrap(),
                sample_rate,
                RationalTime::try_new(1, 1).unwrap(),
                Quality::DRAFT,
                false,
            )
            .unwrap();
            let first = transport.next_frame_plan().unwrap();
            let repeated = transport.next_frame_plan().unwrap();

            assert_eq!(first.timeline_time, RationalTime::try_new(3, 1).unwrap());
            assert_eq!(repeated.timeline_time, first.timeline_time);
        }
    }

    fn position_keyframe_document() -> (motolii_doc::Document, LayerId, [KeyframeId; 2]) {
        let mut document = crate::static_preview::bootstrap_document().unwrap();
        let layer = match &document.tracks[0].items[0] {
            TrackItem::Clip(clip) => clip.envelope.layer_id,
            TrackItem::Group(_) => unreachable!("bootstrap fixture is a clip"),
        };
        let ids = [KeyframeId::from_raw(101), KeyframeId::from_raw(102)];
        let mut track = DocKeyframeTrack::new();
        track.insert(DocKeyframe {
            id: ids[0],
            t: RationalTime::ZERO,
            value: DocValue::Vec2([0.0, 0.0]),
            interp: motolii_eval::Interp::Linear,
        });
        track.insert(DocKeyframe {
            id: ids[1],
            t: RationalTime::try_new(2, 1).unwrap(),
            value: DocValue::Vec2([1.0, 1.0]),
            interp: motolii_eval::Interp::Hold,
        });
        match &mut document.tracks[0].items[0] {
            TrackItem::Clip(clip) => {
                clip.envelope.transform.position = DocParam::Keyframes(track);
            }
            TrackItem::Group(_) => unreachable!("bootstrap fixture is a clip"),
        }
        (document, layer, ids)
    }

    #[test]
    fn position_active_interval_returns_exact_strict_interior_identity_without_document_write() {
        let (document, layer, ids) = position_keyframe_document();
        let before = serde_json::to_vec(&document).unwrap();
        let playhead = RationalTime::try_new(1, 1).unwrap();

        assert_eq!(
            position_active_interval(&document, Some(layer), playhead),
            Some(PositionActiveInterval {
                layer,
                left_id: ids[0],
                left_t: RationalTime::ZERO,
                right_id: ids[1],
                right_t: RationalTime::try_new(2, 1).unwrap(),
                left_interp: motolii_eval::Interp::Linear,
            })
        );
        assert_eq!(serde_json::to_vec(&document).unwrap(), before);
    }

    #[test]
    fn position_key_value_gesture_requires_exact_key_and_unchanged_live_curve() {
        let (document, layer, ids) = position_keyframe_document();
        let before = serde_json::to_vec(&document).unwrap();
        let playhead = RationalTime::ZERO;
        let baseline = position_gesture_baseline(
            &document,
            Some(layer),
            playhead,
            InspectorPositionGestureStart {
                session: 1,
                sequence: 1,
                axis: InspectorPositionAxis::X,
                value: 0.0,
            },
        )
        .expect("exact current Vec2 key must admit Position editing");
        assert_eq!(baseline.key, ids[0]);
        assert_eq!(baseline.value, [0.0, 0.0]);
        assert_eq!(
            resolve_position_gesture_command(
                &document,
                Some(layer),
                playhead,
                &baseline,
                InspectorPositionAxis::X,
                0.25,
            ),
            Some(Command::SetPositionKeyValue {
                target: layer,
                key: ids[0],
                old: [0.0, 0.0],
                new: [0.25, 0.0],
            })
        );
        for (primary, time, axis, value) in [
            (
                Some(layer),
                RationalTime::try_new(1, 1).unwrap(),
                InspectorPositionAxis::X,
                0.25,
            ),
            (None, playhead, InspectorPositionAxis::X, 0.25),
            (Some(layer), playhead, InspectorPositionAxis::X, 0.0),
            (Some(layer), playhead, InspectorPositionAxis::X, f64::NAN),
        ] {
            assert!(resolve_position_gesture_command(
                &document, primary, time, &baseline, axis, value,
            )
            .is_none());
        }
        let mut changed_curve = document.clone();
        match &mut changed_curve.tracks[0].items[0] {
            TrackItem::Clip(clip) => {
                let DocParam::Keyframes(track) = &mut clip.envelope.transform.position else {
                    unreachable!();
                };
                let mut replacement = track.get_by_id(ids[1]).unwrap().clone();
                replacement.value = DocValue::Vec2([2.0, 2.0]);
                track.insert(replacement);
            }
            TrackItem::Group(_) => unreachable!("bootstrap fixture is a clip"),
        }
        assert!(resolve_position_gesture_command(
            &changed_curve,
            Some(layer),
            playhead,
            &baseline,
            InspectorPositionAxis::X,
            0.25,
        )
        .is_none());
        assert_eq!(serde_json::to_vec(&document).unwrap(), before);
    }

    #[test]
    fn easing_admission_rejects_no_interval_stale_duplicate_identity_and_same_value_before_queue() {
        let (document, layer, ids) = position_keyframe_document();
        let interval =
            position_active_interval(&document, Some(layer), RationalTime::try_new(1, 1).unwrap())
                .unwrap();
        assert_eq!(
            admit_easing_open(false, Some(7), 7, None),
            Err(EasingOpenReject::NoActiveInterval),
        );
        assert_eq!(
            admit_easing_open(true, Some(7), 7, Some(interval.clone())),
            Err(EasingOpenReject::PopupActive),
        );
        assert_eq!(
            admit_easing_open(false, Some(8), 7, Some(interval.clone())),
            Err(EasingOpenReject::LayoutEpochMismatch),
        );

        let replacement = motolii_eval::Interp::Bezier {
            x1: 0.4,
            y1: 0.0,
            x2: 0.2,
            y2: 1.0,
        };
        let accepted =
            admit_easing_terminal(11, Some(7), Some(&interval), 11, 7, &interval, replacement)
                .unwrap();
        assert_eq!(
            accepted,
            SetPositionKeyInterpRequest {
                target: layer,
                key: ids[0],
                interp: replacement,
            },
        );

        let mut mismatched_interval = interval.clone();
        mismatched_interval.left_id = KeyframeId::from_raw(999);
        let rejected = [
            admit_easing_terminal(12, Some(7), Some(&interval), 11, 7, &interval, replacement),
            admit_easing_terminal(11, Some(8), Some(&interval), 11, 7, &interval, replacement),
            admit_easing_terminal(
                11,
                Some(7),
                Some(&mismatched_interval),
                11,
                7,
                &interval,
                replacement,
            ),
            admit_easing_terminal(
                11,
                Some(7),
                Some(&interval),
                11,
                7,
                &interval,
                interval.left_interp,
            ),
        ];
        assert_eq!(rejected[0], Err(EasingTerminalReject::GenerationMismatch),);
        assert_eq!(rejected[1], Err(EasingTerminalReject::LayoutEpochMismatch),);
        assert_eq!(rejected[2], Err(EasingTerminalReject::IntervalMismatch));
        assert_eq!(rejected[3], Err(EasingTerminalReject::SameValue));
        let mut queued = Vec::new();
        for request in rejected.into_iter().flatten() {
            queued.push(request);
        }
        assert!(queued.is_empty());
    }

    #[test]
    fn easing_popup_static_gpu_proof_retains_only_product_owned_gpu_parts() {
        let product = include_str!("product_runtime.rs");
        let popup = include_str!("product_easing_popup.rs");

        assert!(product.contains("instance: wgpu::Instance,"));
        assert!(product.contains("adapter: wgpu::Adapter,"));
        assert!(product.contains("&gfx.instance,"));
        assert!(product.contains("&gfx.adapter,"));
        assert!(product.contains("Arc::clone(&self.gpu),"));
        assert!(popup.contains("instance.create_surface(Arc::clone(&window))"));
        assert!(popup.contains("Renderer::new(&gpu.device"));
        assert!(popup.contains("&self.gpu.queue"));
        assert!(!popup.contains("request_device"));
        assert!(!popup.contains("EventLoop::new"));
    }

    #[test]
    fn position_active_interval_rejects_missing_primary_endpoints_and_non_vec2_position() {
        let (mut document, layer, _) = position_keyframe_document();

        assert_eq!(
            position_active_interval(&document, None, RationalTime::ZERO),
            None
        );
        assert_eq!(
            position_active_interval(&document, Some(layer), RationalTime::ZERO),
            None
        );
        assert_eq!(
            position_active_interval(&document, Some(layer), RationalTime::try_new(2, 1).unwrap(),),
            None
        );

        match &mut document.tracks[0].items[0] {
            TrackItem::Clip(clip) => {
                clip.envelope.transform.position = DocParam::const_f64(1.0);
            }
            TrackItem::Group(_) => unreachable!("bootstrap fixture is a clip"),
        }
        let before_unsupported = serde_json::to_vec(&document).unwrap();
        assert_eq!(
            position_active_interval(&document, Some(layer), RationalTime::try_new(1, 1).unwrap()),
            None
        );
        assert_eq!(serde_json::to_vec(&document).unwrap(), before_unsupported);
    }

    #[test]
    fn position_active_interval_fails_closed_for_every_non_position_or_incomplete_track_shape() {
        let (document, layer, ids) = position_keyframe_document();
        let interior = RationalTime::try_new(1, 1).unwrap();
        for playhead in [
            RationalTime::try_new(-1, 1).unwrap(),
            RationalTime::try_new(3, 1).unwrap(),
        ] {
            let before = serde_json::to_vec(&document).unwrap();
            assert_eq!(
                position_active_interval(&document, Some(layer), playhead),
                None
            );
            assert_eq!(serde_json::to_vec(&document).unwrap(), before);
        }

        let zero_keys = DocKeyframeTrack::new();
        let mut one_key = DocKeyframeTrack::new();
        one_key.insert(DocKeyframe {
            id: ids[0],
            t: RationalTime::ZERO,
            value: DocValue::Vec2([0.0, 0.0]),
            interp: motolii_eval::Interp::Linear,
        });
        let mut non_vec2 = DocKeyframeTrack::new();
        non_vec2.insert(DocKeyframe {
            id: ids[0],
            t: RationalTime::ZERO,
            value: DocValue::F64(0.0),
            interp: motolii_eval::Interp::Linear,
        });
        non_vec2.insert(DocKeyframe {
            id: ids[1],
            t: RationalTime::try_new(2, 1).unwrap(),
            value: DocValue::F64(1.0),
            interp: motolii_eval::Interp::Linear,
        });
        let variants = [
            DocParam::Keyframes(zero_keys),
            DocParam::Keyframes(one_key),
            DocParam::Keyframes(non_vec2),
            DocParam::Vec2Axes {
                x: Box::new(DocParam::const_f64(0.0)),
                y: Box::new(DocParam::const_f64(0.0)),
            },
            DocParam::Data {
                track: motolii_eval::DataTrackId("position".to_owned()),
                fallback: DocValue::Vec2([0.0, 0.0]),
            },
            DocParam::LookAt {
                target: layer,
                axis: motolii_doc::LookAtAxis::PlusY,
            },
            DocParam::Follow {
                target: layer,
                offset: [0.0, 0.0],
            },
        ];
        for param in variants {
            let mut case = document.clone();
            match &mut case.tracks[0].items[0] {
                TrackItem::Clip(clip) => clip.envelope.transform.position = param,
                TrackItem::Group(_) => unreachable!("bootstrap fixture is a clip"),
            }
            let before = serde_json::to_vec(&case).unwrap();
            assert_eq!(position_active_interval(&case, Some(layer), interior), None);
            assert_eq!(serde_json::to_vec(&case).unwrap(), before);
        }

        let before = serde_json::to_vec(&document).unwrap();
        assert_eq!(
            position_active_interval(&document, Some(LayerId::from_raw(999)), interior),
            None
        );
        assert_eq!(serde_json::to_vec(&document).unwrap(), before);
    }

    #[test]
    fn stage_transport_snapshot_depends_only_on_document_primary_and_playhead() {
        let (document, layer, _) = position_keyframe_document();
        let interior = RationalTime::try_new(1, 1).unwrap();
        let outside = RationalTime::try_new(3, 1).unwrap();
        let active =
            serde_json::to_value(stage_transport_snapshot(&document, Some(layer), interior))
                .unwrap();
        assert_eq!(
            active["activeInterval"],
            serde_json::json!({ "objectName": "static-preview", "channel": "Position" })
        );
        assert_eq!(
            serde_json::to_value(stage_transport_snapshot(&document, None, interior)).unwrap()
                ["activeInterval"],
            serde_json::Value::Null,
        );
        assert_eq!(
            serde_json::to_value(stage_transport_snapshot(&document, Some(layer), outside))
                .unwrap()["activeInterval"],
            serde_json::Value::Null,
        );
        let mut replaced = document.clone();
        match &mut replaced.tracks[0].items[0] {
            TrackItem::Clip(clip) => {
                clip.envelope.transform.position = DocParam::const_vec2([0.0, 0.0])
            }
            TrackItem::Group(_) => unreachable!("bootstrap fixture is a clip"),
        }
        assert_eq!(
            serde_json::to_value(stage_transport_snapshot(&replaced, Some(layer), interior))
                .unwrap()["activeInterval"],
            serde_json::Value::Null,
        );

        let missing_name_layer = LayerId::from_raw(999);
        let mut missing_name = document.clone();
        match &mut missing_name.tracks[0].items[0] {
            TrackItem::Clip(clip) => clip.envelope.layer_id = missing_name_layer,
            TrackItem::Group(_) => unreachable!("bootstrap fixture is a clip"),
        }
        assert_eq!(
            serde_json::to_value(stage_transport_snapshot(
                &missing_name,
                Some(missing_name_layer),
                interior,
            ))
            .unwrap()["activeInterval"],
            serde_json::Value::Null,
        );

        let mut empty_name = document.clone();
        let empty_name_layer = empty_name.layers.allocate("").unwrap();
        match &mut empty_name.tracks[0].items[0] {
            TrackItem::Clip(clip) => clip.envelope.layer_id = empty_name_layer,
            TrackItem::Group(_) => unreachable!("bootstrap fixture is a clip"),
        }
        assert_eq!(
            serde_json::to_value(stage_transport_snapshot(
                &empty_name,
                Some(empty_name_layer),
                interior,
            ))
            .unwrap()["activeInterval"],
            serde_json::Value::Null,
        );
    }

    #[test]
    fn stage_transport_publish_delivers_one_exact_snapshot_without_document_write() {
        let (document, layer, _) = position_keyframe_document();
        let before = serde_json::to_vec(&document).unwrap();
        let mut published = Vec::new();
        publish_stage_transport_snapshot(
            &document,
            Some(layer),
            RationalTime::try_new(1, 1).unwrap(),
            |snapshot| {
                published.push(snapshot.clone());
                Ok::<_, ()>(())
            },
        )
        .unwrap();
        assert_eq!(published.len(), 1);
        assert_eq!(
            serde_json::to_value(&published[0]).unwrap()["activeInterval"],
            serde_json::json!({ "objectName": "static-preview", "channel": "Position" })
        );
        assert_eq!(serde_json::to_vec(&document).unwrap(), before);

        published.clear();
        publish_stage_transport_snapshot(&document, None, RationalTime::ZERO, |snapshot| {
            published.push(snapshot.clone());
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(published.len(), 1);
        assert_eq!(
            serde_json::to_value(&published[0]).unwrap()["activeInterval"],
            serde_json::Value::Null,
        );
        assert_eq!(serde_json::to_vec(&document).unwrap(), before);
    }

    #[test]
    fn stage_transport_production_lifecycle_has_only_the_admitted_publish_paths() {
        let source = include_str!("product_runtime.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        let method = |name: &str| {
            let start = production.find(name).unwrap();
            let tail = &production[start..];
            let end = ["\n    fn ", "\n    pub(crate) fn "]
                .into_iter()
                .filter_map(|marker| tail[1..].find(marker).map(|end| end + 1))
                .min()
                .unwrap_or(tail.len());
            &tail[..end]
        };
        let free_fn = |name: &str| {
            let start = production.find(name).unwrap();
            let tail = &production[start..];
            &tail[..tail[1..]
                .find("\nfn ")
                .map(|end| end + 1)
                .unwrap_or(tail.len())]
        };

        assert!(production.contains("StageChromeHostRuntime::new(\n            &window,\n            &stage_transport_snapshot("));
        assert_eq!(
            production.matches("self.publish_stage_transport()").count(),
            9
        );
        assert!(method("fn refresh_editor_playhead").contains("self.publish_stage_transport()?"));
        assert!(method("fn publish_stage_transport")
            .contains("publish_stage_transport_snapshot_with_state("));
        assert!(!method("fn update_layout").contains("publish_stage_transport"));
        assert!(!method("fn update_layout").contains("refresh_editor_playhead"));
        let cancel = method("fn cancel_editor_playhead");
        assert!(cancel.contains(
            "if self.editor_playhead.scrub.is_none() {\n            return Ok(());\n        }"
        ));
        assert!(cancel.contains("if changed {\n            self.refresh_editor_playhead()?;"));
        let snapshot = free_fn("fn stage_transport_snapshot(");
        let publish = free_fn("fn publish_stage_transport_snapshot<");
        for body in [snapshot, publish] {
            for forbidden in [
                "journal",
                "history",
                "undo",
                "document_queue",
                "projection_generation",
            ] {
                assert!(!body.contains(forbidden));
            }
        }
        assert!(snapshot.contains("document: &motolii_doc::Document"));
        assert!(publish.contains("document: &motolii_doc::Document"));
    }

    #[test]
    fn inspector_position_key_wake_route_resolves_current_primary_and_playhead_before_browser_poll()
    {
        let source = include_str!("product_runtime.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        let method = |name: &str| {
            let start = production.find(name).unwrap();
            let tail = &production[start..];
            let end = ["\n    fn ", "\n    pub(crate) fn "]
                .into_iter()
                .filter_map(|marker| tail[1..].find(marker).map(|end| end + 1))
                .min()
                .unwrap_or(tail.len());
            &tail[..end]
        };

        let wake = method("pub(crate) fn handle_product_event");
        let position_route = method("fn process_inspector_position_key_intents");
        assert!(
            wake.find("self.process_inspector_position_key_intents(event_loop)")
                < wake.find("self.poll_browser(event_loop)")
        );
        for required in [
            "let Some(target) = self.primary else",
            "time: self.editor_playhead.current",
            "self.document_queue.push_add_position_key(request)",
            "self.document_runtime.process_next(",
            "self.adopt_full_publish(event_loop, published, \"inspector-add-position-key\")",
        ] {
            assert!(position_route.contains(required), "{required}");
        }
        for forbidden in ["RationalTime::ZERO", "mock", "postMessage", "Interp"] {
            assert!(!position_route.contains(forbidden), "{forbidden}");
        }
    }

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
    fn media_drop_outside_stage_is_rejected_before_document_dispatch() {
        let layout = test_layout(1);
        let camera =
            motolii_core::CompCamera::try_new(CanonicalPoint { x: 0.0, y: 0.0 }, 0.0, 1.0, 16, 9)
                .unwrap();
        assert_eq!(
            canonical_media_drop_position(Some(layout), camera, [-1.0, -1.0]),
            Err("outside-stage")
        );
    }

    #[test]
    fn media_drop_add_clip_failure_has_its_own_reject_reason() {
        let source = include_str!("product_runtime.rs");
        let route = source
            .split("fn process_file_drop(")
            .nth(1)
            .unwrap()
            .split("fn handle_browser_lifecycle(")
            .next()
            .unwrap();
        assert!(route.contains("DocumentEditRuntimeError::AddClipFailed"));
        assert!(route.contains("trace_media_drop_reject(\"add-clip-failed\")"));
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
    fn editor_playhead_starts_at_zero_and_retains_release_value() {
        let mut playhead = EditorPlayhead::default();
        let interior = RationalTime::try_new(3, 2).unwrap();

        assert_eq!(playhead.current, RationalTime::ZERO);
        assert!(playhead.begin(9, interior));
        assert!(!playhead.finish(9));
        assert_eq!(playhead.current, interior);
        assert!(playhead.scrub.is_none());
    }

    #[test]
    fn editor_playhead_cancel_and_layout_change_restore_press_value() {
        let mut playhead = EditorPlayhead::default();
        let press = RationalTime::try_new(1, 2).unwrap();
        let moved = RationalTime::try_new(3, 2).unwrap();

        assert!(playhead.begin(7, press));
        assert_eq!(playhead.update(7, moved), Some(true));
        assert!(playhead.cancel());
        assert_eq!(playhead.current, RationalTime::ZERO);
        assert!(playhead.begin(8, press));
        assert_eq!(playhead.update(9, moved), None);
        assert!(playhead.cancel());
        assert_eq!(playhead.current, RationalTime::ZERO);
    }

    #[test]
    fn editor_playhead_publish_retirement_preserves_current_value() {
        let mut playhead = EditorPlayhead::default();
        let press = RationalTime::try_new(1, 2).unwrap();
        let current = RationalTime::try_new(3, 2).unwrap();

        assert!(playhead.begin(7, press));
        assert_eq!(playhead.update(7, current), Some(true));
        assert!(playhead.retire());
        assert_eq!(playhead.current, current);
        assert!(playhead.scrub.is_none());
    }

    #[test]
    fn playhead_scrub_arms_existing_escape_and_safety_cancel_lifecycle() {
        let registry = builtin_command_registry().unwrap();
        let cancel = CommandId::try_new("motolii.gesture.cancel").unwrap();
        let mut router = InputRouter::new(registry);

        router
            .route(NormalizedInput::Phase(InputPhase::DragStart))
            .unwrap();
        assert!(matches!(
            router
                .route(NormalizedInput::Command {
                    phase: InputPhase::Press,
                    id: cancel.clone(),
                })
                .unwrap(),
            RouterOutput::Intent {
                intent: DomainIntent::CancelInFlightGesture,
                ..
            }
        ));
        router
            .route(NormalizedInput::Phase(InputPhase::DragStart))
            .unwrap();
        assert!(matches!(
            router
                .route(NormalizedInput::SafetyInterrupt(
                    SafetyInterrupt::WindowFocusLost
                ))
                .unwrap(),
            RouterOutput::SafetyCancel {
                intent: DomainIntent::CancelInFlightGesture,
                ..
            }
        ));
    }

    #[test]
    fn ruler_mapping_is_closed_clamped_and_excludes_non_ruler_inputs() {
        let document = crate::static_preview::bootstrap_document().unwrap();
        let projection = ProductTimelineProjection::from_document(&document).unwrap();
        let layout = test_layout(9);
        let ruler = timeline_ruler_logical_rect(layout).unwrap();
        let duration = document.composition.duration;
        let y = ruler.y + ruler.height / 2.0;

        assert_eq!(
            projection.ruler_time_at([ruler.x, y], layout, true),
            Some(RationalTime::ZERO)
        );
        assert_eq!(
            projection.ruler_time_at([ruler.x + ruler.width / 2.0, y], layout, true),
            Some(
                duration
                    .try_mul(RationalTime::try_new(1, 2).unwrap())
                    .unwrap()
            )
        );
        assert_eq!(
            projection.ruler_time_at([ruler.x + ruler.width, y], layout, true),
            Some(duration)
        );
        assert_eq!(
            projection.ruler_time_at([ruler.x - 1.0, y], layout, true),
            None
        );
        assert_eq!(
            projection.ruler_time_at([ruler.x + 1.0, ruler.y - 1.0], layout, true),
            None
        );
        assert_eq!(
            projection.ruler_time_at([ruler.x + 1.0, ruler.y + ruler.height + 1.0], layout, true),
            None
        );
        assert_eq!(projection.ruler_time_at([f64::NAN, y], layout, true), None);
        let mut missing_timeline = layout;
        missing_timeline.timeline = None;
        assert_eq!(
            projection.ruler_time_at([ruler.x, y], missing_timeline, true),
            None
        );
        assert_eq!(
            projection.ruler_time_at([ruler.x - 1.0, y], layout, false),
            Some(RationalTime::ZERO)
        );
        assert_eq!(
            projection.ruler_time_at([ruler.x + ruler.width + 1.0, y], layout, false),
            Some(duration)
        );
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
            Some(ProductTimelineHit::Body {
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
    fn timeline_private_hit_refines_only_admitted_bar_edges() {
        let document = crate::static_preview::bootstrap_document().unwrap();
        let projection = ProductTimelineProjection::from_document(&document).unwrap();
        let layer = projection.projection.bars()[0].layer;
        let layout = test_layout(9);
        let surface = timeline_time_surface_logical_rect(layout).unwrap();
        let y = surface.y + surface.height / 2.0;

        assert_eq!(
            projection.hit_test([surface.x + 15.0, y], layout),
            Some(ProductTimelineHit::Left { layer })
        );
        assert_eq!(
            projection.hit_test([surface.x + 15.001, y], layout),
            Some(ProductTimelineHit::Body { layer })
        );
        assert_eq!(
            projection.hit_test([surface.x + surface.width - 15.0, y], layout),
            Some(ProductTimelineHit::Right { layer })
        );

        let timeline = layout.timeline.unwrap();
        let horizontal_chrome = timeline.width - surface.width;
        let mut narrow = layout;
        narrow.timeline.as_mut().unwrap().width = horizontal_chrome + 24.999;
        let narrow_surface = timeline_time_surface_logical_rect(narrow).unwrap();
        assert_eq!(
            projection.hit_test(
                [
                    narrow_surface.x + 1.0,
                    narrow_surface.y + narrow_surface.height / 2.0
                ],
                narrow,
            ),
            Some(ProductTimelineHit::Body { layer })
        );
        narrow.timeline.as_mut().unwrap().width = horizontal_chrome + 25.0;
        let cutoff_surface = timeline_time_surface_logical_rect(narrow).unwrap();
        assert_eq!(
            projection.hit_test(
                [
                    cutoff_surface.x + 1.0,
                    cutoff_surface.y + cutoff_surface.height / 2.0
                ],
                narrow,
            ),
            Some(ProductTimelineHit::Left { layer })
        );

        let vertical_chrome = timeline.height - surface.height;
        let mut short = layout;
        short.timeline.as_mut().unwrap().height = vertical_chrome + 15.999;
        let short_surface = timeline_time_surface_logical_rect(short).unwrap();
        assert_eq!(
            projection.hit_test(
                [
                    short_surface.x + 1.0,
                    short_surface.y + short_surface.height / 2.0
                ],
                short,
            ),
            Some(ProductTimelineHit::Body { layer })
        );
        short.timeline.as_mut().unwrap().height = vertical_chrome + 16.0;
        let height_cutoff_surface = timeline_time_surface_logical_rect(short).unwrap();
        assert_eq!(
            projection.hit_test(
                [
                    height_cutoff_surface.x + 1.0,
                    height_cutoff_surface.y + height_cutoff_surface.height / 2.0,
                ],
                short,
            ),
            Some(ProductTimelineHit::Left { layer })
        );
    }

    #[test]
    fn timeline_trim_rejects_stale_generation_changed_interval_and_target_loss() {
        let interval = (
            RationalTime::try_new(2, 10).unwrap(),
            RationalTime::try_new(8, 10).unwrap(),
        );
        let gesture = TimelineTrimGesture::begin(
            LayerId::from_raw(7),
            TimelineTrimEdge::Left,
            RationalTime::try_new(3, 10).unwrap(),
            interval.0,
            interval.1,
            4,
        );

        assert!(timeline_trim_gesture_is_current(
            &gesture,
            4,
            Some(interval)
        ));
        assert!(!timeline_trim_gesture_is_current(
            &gesture,
            5,
            Some(interval)
        ));
        assert!(!timeline_trim_gesture_is_current(
            &gesture,
            4,
            Some((interval.0, RationalTime::try_new(9, 10).unwrap())),
        ));
        assert!(!timeline_trim_gesture_is_current(&gesture, 4, None));
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
    fn product_shortcuts_resolve_to_stable_command_ids() {
        let registry = builtin_command_registry().unwrap();
        let base = product_builtin_keymap().unwrap();
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
        let cancel = EffectiveTrigger::Keyboard {
            key: KeyToken::Escape,
            modifiers: Modifiers::default(),
            phase: InputPhase::Press,
        };
        let modified_cancel = EffectiveTrigger::Keyboard {
            key: KeyToken::Escape,
            modifiers: Modifiers::try_new([Modifier::Shift]).unwrap(),
            phase: InputPhase::Press,
        };

        assert_eq!(base.version, 2);
        assert_eq!(
            keymap.get(&undo).map(CommandId::as_str),
            Some("motolii.edit.undo")
        );
        assert_eq!(
            keymap.get(&redo).map(CommandId::as_str),
            Some("motolii.edit.redo")
        );
        assert_eq!(
            keymap.get(&cancel).map(CommandId::as_str),
            Some("motolii.gesture.cancel")
        );
        assert_eq!(keymap.get(&modified_cancel), None);
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
