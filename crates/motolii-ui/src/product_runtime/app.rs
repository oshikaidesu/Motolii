//! ProductAppの所有とwindow lifecycle。Host席の組み立てだけを行う。

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
    DocumentEditRuntime, DocumentEditRuntimeError, PlaceRectangleRequest, PublishedDocument,
    SetPositionKeyInterpRequest, SetPositionKeyValueRequest,
};
use crate::host_pointer_capture::{HostPointerCancel, HostPointerCandidate};
use crate::inspector_host_runtime::{
    resolve_effect_param_preview_command, InspectorGestureTerminal, InspectorGestureTerminalCause,
    InspectorHostRuntime, InspectorHostRuntimeError, InspectorPositionAxis,
    InspectorPositionGestureStart, InspectorPositionGestureTerminal,
    InspectorPositionGestureTerminalCause,
};
use crate::layout_authority::LayoutAuthority;
use crate::native_host_layout::{
    key_tools_logical_rect, timeline_ruler_logical_rect, timeline_time_surface_logical_rect,
    LogicalRect, NativeHostLayout, PhysicalRect,
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
    builtin_command_registry, default_user_keymap_override_path, load_user_keymap_override,
    product_builtin_keymap, resolve_keymap, CommandIdError, CommandRegistry, CommandRegistryError,
    DomainIntent, EffectiveTrigger, ImeGateState, InputPhase, InputRouter, InputRouterError,
    KeyToken, KeymapResolution, ModifierError, Modifiers, NormalizedInput,
    PlatformBindingConstraints, PlatformCommandModifier, RouterOutput, SafetyInterrupt,
};

use super::browser::{build_browser_runtime, BrowserLifecycleCoordinator};
use super::easing::PositionActiveInterval;
use super::elapsed_ms;
use super::error::ProductRuntimeError;
use super::place::{
    ClassifiedPlaceTerminal, PlacePreviewPhase, PlaceTerminalAdmission, PlaceTerminalDelivery,
};
use super::place_overlay::rectangle_place_overlay;
use super::playback::{PlaybackLifecycle, PlaybackPreparation, ProductPlaybackError};
use super::playhead::EditorPlayhead;
use super::position::PositionGestureBaseline;
use super::projection::{
    PendingStageDrop, ProductStageProjection, ProductTimelineProjection,
};
use super::stage_transport::stage_transport_snapshot;
use super::surface::{ProductGpuParts, ProductSurface, ProductSurfaceError};
use super::ProductEvent;

pub(crate) struct ProductApp {
    // surface → WebView → Windowの順にdropし、AppKit backingを先に失わない。
    pub(super) gfx: Option<ProductSurface>,
    pub(super) browser: Option<BrowserHostRuntime>,
    pub(super) inspector: Option<InspectorHostRuntime>,
    pub(super) stage_chrome: Option<StageChromeHostRuntime>,
    pub(super) easing_popup: Option<(ProductEasingPopup, PositionActiveInterval, u64, u64)>,
    pub(super) timeline_tools: Option<TimelineToolsHostRuntime>,
    pub(super) window: Option<Arc<Window>>,
    pub(super) gpu: Arc<GpuCtx>,
    pub(super) parts: Option<ProductGpuParts>,
    pub(super) preview: Arc<StaticPreview>,
    pub(super) render_client: RenderWorkerClient,
    pub(super) render_request_template: RenderRequest,
    pub(super) stage_projection: ProductStageProjection,
    pub(super) timeline_projection: ProductTimelineProjection,
    pub(super) displayed_camera: motolii_core::CompCamera,
    pub(super) document_runtime: DocumentEditRuntime,
    pub(super) document_queue: DocumentEditQueue,
    pub(super) input_router: InputRouter,
    pub(super) command_keymap: KeymapResolution,
    pub(super) current_modifiers: Modifiers,
    pub(super) primary: Option<motolii_doc::LayerId>,
    pub(super) active_effect_use: Option<EffectId>,
    pub(super) projection_generation: u64,
    pub(super) current_document: Arc<motolii_doc::Document>,
    pub(super) proxy: EventLoopProxy<ProductEvent>,
    pub(super) layout_authority: LayoutAuthority,
    pub(super) layout: Option<NativeHostLayout>,
    pub(super) next_layout_epoch: u64,
    pub(super) browser_source: Option<BrowserPlaceIntent>,
    pub(super) browser_lifecycle: Option<BrowserLifecycleCoordinator>,
    pub(super) browser_focus_target: BrowserFocusTarget,
    pub(super) next_place_generation: u64,
    pub(super) active_place: Option<BrowserPlaceIntent>,
    pub(super) place_preview: PlacePreviewPhase,
    pub(super) terminal_admission: PlaceTerminalAdmission,
    pub(super) terminal_delivery: PlaceTerminalDelivery,
    pub(super) candidate_terminal: Option<ClassifiedPlaceTerminal>,
    pub(super) admitted_terminal: Option<ClassifiedPlaceTerminal>,
    pub(super) pending_stage_drop: Option<PendingStageDrop>,
    pub(super) surface_retry_at: Option<Instant>,
    pub(super) failure: Option<String>,
    pub(super) pending_inspector_commit: Option<InspectorGestureTerminal>,
    pub(super) pending_position_commit: Option<InspectorPositionGestureTerminal>,
    pub(super) position_gesture: Option<PositionGestureBaseline>,
    pub(super) timeline_move: Option<TimelineMoveGesture>,
    pub(super) timeline_trim: Option<TimelineTrimGesture>,
    pub(super) editor_playhead: EditorPlayhead,
    pub(super) playback_lifecycle: PlaybackLifecycle,
    pub(super) playback_preparation: Option<PlaybackPreparation>,
    pub(super) playback_session: Option<PlaybackSession>,
    pub(super) playback_caches: HashMap<(String, u32), Arc<PcmCache>>,
    pub(super) last_pointer_position: Option<[f64; 2]>,
}

impl ProductApp {
    pub(super) fn new(
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
        match gfx.render(layout, window, place_overlay.as_ref()) {
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
            Err(ProductSurfaceError::Fatal(reason)) => self.fail(event_loop, reason),
        }
    }
}

pub(super) fn product_command_keymap(
    registry: &CommandRegistry,
) -> Result<KeymapResolution, ProductRuntimeError> {
    let base = product_builtin_keymap();
    let delta = load_user_keymap_override(default_user_keymap_override_path().as_deref(), &base);
    let resolution = resolve_keymap(
        &base,
        &delta,
        &PlatformBindingConstraints::new(PlatformCommandModifier::Meta, Vec::new()),
        registry,
    );
    if resolution.diagnostics().is_empty() {
        Ok(resolution)
    } else {
        Err(ProductRuntimeError::CommandKeymap)
    }
}
