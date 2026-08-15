//! Browser lifecycleとplace入力の排出。古いepochのcallbackは無視する。

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

use crate::canonical_drop::canonical_drop_from_ndc;
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

use super::app::ProductApp;
use super::error::ProductRuntimeError;
use super::place::{ClassifiedPlaceTerminal, PlacePreviewPhase};
use super::ProductEvent;

impl ProductApp {
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
        }
    }

    pub(super) fn handle_browser_lifecycle(
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

    pub(super) fn replace_browser(
        &mut self,
        instance_epoch: u64,
    ) -> Result<(), ProductRuntimeError> {
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
}

pub(super) struct OptionalNumber(Option<f64>);

impl std::fmt::Display for OptionalNumber {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(value) => write!(formatter, "{value:.6}"),
            None => formatter.write_str("outside"),
        }
    }
}

pub(super) fn build_browser_runtime(
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BrowserRecoveryDecision {
    Ignore,
    Replace { instance_epoch: u64 },
    Degrade,
}

#[derive(Debug)]
pub(super) struct BrowserLifecycleCoordinator {
    pub(super) next_instance_epoch: u64,
    pub(super) automatic_process_recovery_used: bool,
    pub(super) degraded: bool,
}

impl BrowserLifecycleCoordinator {
    pub(super) fn new(initial_instance_epoch: u64) -> Result<Self, ProductRuntimeError> {
        Ok(Self {
            next_instance_epoch: initial_instance_epoch
                .checked_add(1)
                .ok_or(ProductRuntimeError::BrowserInstanceEpochExhausted)?,
            automatic_process_recovery_used: false,
            degraded: false,
        })
    }

    pub(super) fn observe(
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
