//! Stage playbackのlifecycleとtransport snapshot。sessionは一つだけ。

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
use super::playhead::EditorPlayhead;
use super::ProductEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PlaybackLifecycle {
    pub(super) state: StagePlaybackState,
    pub(super) generation: u64,
    pub(super) session_active: bool,
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
    pub(super) fn state(self) -> StagePlaybackState {
        self.state
    }

    pub(super) fn begin_preparing(&mut self) -> Result<u64, ProductPlaybackError> {
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

    pub(super) fn accepts_preparation(self, generation: u64) -> bool {
        self.state == StagePlaybackState::Preparing && self.generation == generation
    }

    pub(super) fn activate(&mut self, generation: u64) -> Result<(), ProductPlaybackError> {
        if !self.accepts_preparation(generation) || self.session_active {
            return Err(ProductPlaybackError::LifecycleConflict { state: self.state });
        }
        self.state = StagePlaybackState::Playing;
        self.session_active = true;
        Ok(())
    }

    pub(super) fn cancel_preparing(&mut self) -> Result<(), ProductPlaybackError> {
        if self.state != StagePlaybackState::Preparing || self.session_active {
            return Err(ProductPlaybackError::LifecycleConflict { state: self.state });
        }
        self.invalidate()
    }

    pub(super) fn invalidate(&mut self) -> Result<(), ProductPlaybackError> {
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or(ProductPlaybackError::GenerationExhausted)?;
        self.state = StagePlaybackState::Idle;
        self.session_active = false;
        Ok(())
    }
}

pub(super) struct PlaybackPreparation {
    pub(super) generation: u64,
    pub(super) start_frame: u64,
    pub(super) receiver: mpsc::Receiver<PlaybackPreparationResult>,
}

pub(super) struct PlaybackPreparationResult {
    pub(super) generation: u64,
    pub(super) caches: HashMap<(String, u32), Arc<PcmCache>>,
    pub(super) program: Result<Arc<AudioProgram>, motolii_audio::AudioError>,
}

impl ProductApp {
    pub(super) fn process_stage_playback_intents(
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

    pub(super) fn toggle_playback(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), ProductRuntimeError> {
        match self.playback_lifecycle.state() {
            StagePlaybackState::Idle => self.begin_playback_preparation(event_loop),
            StagePlaybackState::Preparing => self.cancel_playback_preparation(event_loop),
            StagePlaybackState::Playing => self.pause_playback(event_loop),
        }
    }

    pub(super) fn begin_playback_preparation(
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

    pub(super) fn cancel_playback_preparation(
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

    pub(super) fn process_playback_preparation(&mut self) -> Result<(), ProductRuntimeError> {
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
            Some(self.gpu.as_ref()),
        )
        .map_err(ProductPlaybackError::from)?;
        self.playback_lifecycle.activate(received.generation)?;
        self.playback_session = Some(session);
        self.publish_stage_transport()?;
        self.submit_stage_projection()?;
        self.request_redraw();
        Ok(())
    }

    pub(super) fn pause_playback(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), ProductRuntimeError> {
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

    pub(super) fn process_playback_clock(
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

    pub(super) fn adopt_playback_frame_plan(
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

    pub(super) fn retire_playback_for_document_change(
        &mut self,
    ) -> Result<(), ProductRuntimeError> {
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

    pub(super) fn stop_playback_for_scrub(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), ProductRuntimeError> {
        match self.playback_lifecycle.state() {
            StagePlaybackState::Idle => Ok(()),
            StagePlaybackState::Preparing => self.cancel_playback_preparation(event_loop),
            StagePlaybackState::Playing => self.pause_playback(event_loop),
        }
    }
}

pub(super) fn canonical_playback_start_frame(
    current: RationalTime,
) -> Result<u64, ProductPlaybackError> {
    if current < RationalTime::ZERO {
        return Err(ProductPlaybackError::NegativePlayhead);
    }
    let canonical_fps = Fps::try_new(CANONICAL_SAMPLE_RATE as i64, 1)?;
    let frame = current.try_to_frame_floor(canonical_fps)?;
    u64::try_from(frame).map_err(|_| ProductPlaybackError::StartFrameOverflow)
}

pub(super) fn clamp_playback_time(
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
