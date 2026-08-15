//! Stage easing popupのopen / terminal。interval identityが一致する時だけqueueする。

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
use super::position::position_active_interval;

impl ProductApp {
    pub(super) fn process_stage_easing_intents(&mut self, event_loop: &ActiveEventLoop) {
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
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PositionActiveInterval {
    pub(super) layer: LayerId,
    pub(super) left_id: KeyframeId,
    pub(super) left_t: RationalTime,
    pub(super) right_id: KeyframeId,
    pub(super) right_t: RationalTime,
    pub(super) left_interp: motolii_eval::Interp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EasingOpenReject {
    PopupActive,
    LayoutEpochMismatch,
    NoActiveInterval,
}

pub(super) fn admit_easing_open(
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
pub(super) enum EasingTerminalReject {
    GenerationMismatch,
    LayoutEpochMismatch,
    IntervalMismatch,
    SameValue,
}

pub(super) fn admit_easing_terminal(
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
