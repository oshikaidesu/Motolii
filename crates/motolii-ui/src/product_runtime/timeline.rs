//! Timelineのplayhead / move / trim / 選択。generationが変わったら捨てる。

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
use super::publish::trace_document_publish;

impl ProductApp {
    pub(super) fn refresh_editor_playhead(&mut self) -> Result<(), ProductRuntimeError> {
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

    pub(super) fn cancel_editor_playhead(
        &mut self,
        reason: &'static str,
    ) -> Result<(), ProductRuntimeError> {
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

    pub(super) fn retire_editor_playhead(&mut self, reason: &'static str) {
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

    pub(super) fn cancel_timeline_move(&mut self, reason: &'static str) {
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

    pub(super) fn cancel_timeline_trim(&mut self, reason: &'static str) {
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

    pub(super) fn finish_timeline_move(
        &mut self,
        event_loop: &ActiveEventLoop,
        position: [f64; 2],
    ) {
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

    pub(super) fn finish_timeline_trim(
        &mut self,
        event_loop: &ActiveEventLoop,
        position: [f64; 2],
    ) {
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

    pub(super) fn set_idle_control_flow(&self, event_loop: &ActiveEventLoop) {
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

    pub(super) fn handle_timeline_click(
        &mut self,
        event_loop: &ActiveEventLoop,
        position: [f64; 2],
    ) {
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
}

pub(super) fn find_clip_start(
    document: &motolii_doc::Document,
    target: LayerId,
) -> Option<RationalTime> {
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

pub(super) fn find_clip_interval(
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

pub(super) fn timeline_trim_gesture_is_current(
    gesture: &TimelineTrimGesture,
    projection_generation: u64,
    interval: Option<(RationalTime, RationalTime)>,
) -> bool {
    gesture.generation() == projection_generation
        && interval == Some((gesture.initial_start(), gesture.initial_end()))
}
