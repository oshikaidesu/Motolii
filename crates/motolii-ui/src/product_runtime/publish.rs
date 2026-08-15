//! PublishedDocumentの採用とStage投影。RN snapshot口はここへ再実装しない。

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
use super::projection::ProductTimelineProjection;
use super::stage_transport::publish_stage_transport_snapshot_with_state;

impl ProductApp {
    pub(super) fn submit_stage_projection(&self) -> Result<RenderGeneration, ProductRuntimeError> {
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

    pub(super) fn process_pending_inspector_commit(
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

    pub(super) fn handle_history_trigger(
        &mut self,
        event_loop: &ActiveEventLoop,
        trigger: EffectiveTrigger,
    ) {
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

    pub(super) fn adopt_full_publish(
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
        // RN 製品の snapshot 口は rn_product_host。ここは winit seat の PublishedDocument 採用。
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

    pub(super) fn publish_stage_transport(&self) -> Result<(), ProductRuntimeError> {
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

    pub(super) fn adopt_history_publish(
        &mut self,
        event_loop: &ActiveEventLoop,
        published: PublishedDocument,
    ) {
        self.adopt_full_publish(event_loop, published, "history");
    }

    pub(super) fn reconcile_active_effect_use(&mut self, published: &PublishedDocument) {
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

    pub(super) fn publish_timeline_tools(&self) -> Result<(), ProductRuntimeError> {
        if let Some(timeline_tools) = &self.timeline_tools {
            timeline_tools.publish(
                self.timeline_projection.projection.bars().len(),
                self.timeline_projection.projection.keys().len(),
            )?;
        }
        Ok(())
    }

    pub(super) fn drain_stage_projection(&mut self) -> Result<(), ProductRuntimeError> {
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

    pub(super) fn process_attach_effect(
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
}

pub(super) fn active_effect_candidate(
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

pub(super) fn trace_document_publish(route: &str, published: &PublishedDocument) {
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

pub(super) fn trace_timeline_projection(
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
