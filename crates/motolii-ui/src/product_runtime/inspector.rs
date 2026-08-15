//! Inspector gestureとposition key。live curveが変わったら失敗閉じ。

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

use super::app::ProductApp;
use super::error::ProductRuntimeError;
use super::position::{position_gesture_baseline, resolve_position_gesture_command};

impl ProductApp {
    pub(super) fn inspector_render_request(
        &self,
        document: Arc<motolii_doc::Document>,
    ) -> RenderRequest {
        RenderRequest {
            document,
            data_tracks: Arc::clone(&self.render_request_template.data_tracks),
            evaluation_time: EvaluationTime::new(self.editor_playhead.current),
            desc: self.render_request_template.desc,
            quality: self.render_request_template.quality,
        }
    }

    pub(super) fn submit_inspector_baseline(
        &self,
    ) -> Result<RenderGeneration, ProductRuntimeError> {
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

    pub(super) fn submit_inspector_preview(
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

    pub(super) fn process_inspector_gestures(&mut self) -> Result<(), ProductRuntimeError> {
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

    pub(super) fn process_inspector_position_gestures(
        &mut self,
    ) -> Result<(), ProductRuntimeError> {
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

    pub(super) fn process_pending_position_commit(
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

    pub(super) fn process_inspector_position_key_intents(
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
}
