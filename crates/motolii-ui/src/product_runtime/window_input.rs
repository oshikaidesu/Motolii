//! 主windowのpointer / key / IME。gesture開始だけをここで武装する。

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
use super::projection::ProductTimelineHit;
use super::timeline::{find_clip_interval, find_clip_start};

impl ProductApp {
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
}
