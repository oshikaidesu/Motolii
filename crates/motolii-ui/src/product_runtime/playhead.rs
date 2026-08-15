//! editor playheadのscrub値。layout epochが変わったらpress値へ戻す。

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PlayheadScrub {
    pub(super) initial: RationalTime,
    pub(super) layout_epoch: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct EditorPlayhead {
    pub(super) current: RationalTime,
    pub(super) scrub: Option<PlayheadScrub>,
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
    pub(super) fn begin(&mut self, layout_epoch: u64, time: RationalTime) -> bool {
        self.scrub = Some(PlayheadScrub {
            initial: self.current,
            layout_epoch,
        });
        self.set(time)
    }

    pub(super) fn update(&mut self, layout_epoch: u64, time: RationalTime) -> Option<bool> {
        (self.scrub?.layout_epoch == layout_epoch).then(|| self.set(time))
    }

    pub(super) fn finish(&mut self, layout_epoch: u64) -> bool {
        let Some(scrub) = self.scrub else {
            return false;
        };
        if scrub.layout_epoch != layout_epoch {
            return self.cancel();
        }
        self.scrub = None;
        false
    }

    pub(super) fn cancel(&mut self) -> bool {
        let Some(scrub) = self.scrub.take() else {
            return false;
        };
        self.set(scrub.initial)
    }

    pub(super) fn retire(&mut self) -> bool {
        self.scrub.take().is_some()
    }

    pub(super) fn set(&mut self, time: RationalTime) -> bool {
        if self.current == time {
            return false;
        }
        self.current = time;
        true
    }
}
