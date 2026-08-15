//! Browser placeのpreview / admission / delivery。commitは一度だけ。

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

use super::projection::PendingStageDrop;

#[derive(Debug, Default)]
pub(super) struct PlaceTerminalDelivery {
    pub(super) delivered_high_water: Option<u64>,
}

impl PlaceTerminalDelivery {
    pub(super) fn deliver(
        &mut self,
        terminal: &ClassifiedPlaceTerminal,
    ) -> Option<PendingStageDrop> {
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
pub(super) enum PlaceTerminalCause {
    Escape,
    OutsideStage,
    CaptureLoss,
    NoNonCommitCause,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ClassifiedPlaceTerminal {
    pub(super) source: BrowserPlaceIntent,
    pub(super) generation: u64,
    pub(super) cause: PlaceTerminalCause,
    pub(super) layout_epoch: Option<u64>,
    pub(super) stage_ndc: Option<[f64; 2]>,
}

impl ClassifiedPlaceTerminal {
    pub(super) fn released(
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

    pub(super) fn cancelled(
        source: BrowserPlaceIntent,
        generation: u64,
        reason: HostPointerCancel,
    ) -> Self {
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
pub(super) struct PlaceTerminalAdmission {
    pub(super) active_generation: Option<u64>,
    pub(super) retired_high_water: Option<u64>,
}

impl PlaceTerminalAdmission {
    pub(super) fn begin(&mut self, generation: u64) -> bool {
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

    pub(super) fn admit(&mut self, terminal: &ClassifiedPlaceTerminal) -> bool {
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

    pub(super) fn retire_active(&mut self) {
        if let Some(generation) = self.active_generation.take() {
            self.retired_high_water = Some(
                self.retired_high_water
                    .map_or(generation, |high_water| high_water.max(generation)),
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PlacePreviewProgress {
    pub(super) source: BrowserPlaceIntent,
    pub(super) generation: u64,
    pub(super) layout_epoch: u64,
    pub(super) stage_ndc: Option<[f64; 2]>,
}

#[derive(Debug, Default)]
pub(super) struct PlacePreviewPhase {
    pub(super) latest: Option<PlacePreviewProgress>,
}

impl PlacePreviewPhase {
    pub(super) fn deliver(
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

    pub(super) fn clear(&mut self) {
        self.latest = None;
    }
}
