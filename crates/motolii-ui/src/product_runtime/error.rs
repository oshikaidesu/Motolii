//! product runtimeの失敗。原因型を文字列へ潰さない。

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

use super::playback::ProductPlaybackError;

#[derive(Debug, thiserror::Error)]
pub(crate) enum ProductRuntimeError {
    #[error(transparent)]
    Gpu(#[from] motolii_gpu::GpuError),
    #[error(transparent)]
    Preview(#[from] crate::static_preview::StaticPreviewError),
    #[error(transparent)]
    RenderWorkerStart(#[from] crate::render_worker::RenderWorkerStartError),
    #[error(transparent)]
    RenderWorkerJoin(#[from] crate::render_worker::RenderJoinError),
    #[error(transparent)]
    RenderSubmit(#[from] crate::render_worker::RenderSubmitError),
    #[error(transparent)]
    RenderWorker(#[from] RenderWorkerError),
    #[error(transparent)]
    RepaintSignal(#[from] crate::render_worker::RepaintSignalRegistrationError),
    #[error(transparent)]
    Display(#[from] crate::display_slot::DisplaySlotError),
    #[error(transparent)]
    TimelineProjection(#[from] TimelineProjectionError),
    #[error(transparent)]
    EventLoop(#[from] winit::error::EventLoopError),
    #[error(transparent)]
    Os(#[from] winit::error::OsError),
    #[error(transparent)]
    Surface(#[from] wgpu::CreateSurfaceError),
    #[error(transparent)]
    Browser(#[from] BrowserHostRuntimeError),
    #[error(transparent)]
    Inspector(#[from] InspectorHostRuntimeError),
    #[error(transparent)]
    StageChrome(#[from] StageChromeHostRuntimeError),
    #[error(transparent)]
    StageEasingIntent(#[from] StageEasingIntentError),
    #[error(transparent)]
    StagePlaybackIntent(#[from] StagePlaybackIntentError),
    #[error(transparent)]
    Playback(#[from] ProductPlaybackError),
    #[error(transparent)]
    EasingPopup(#[from] ProductEasingPopupError),
    #[error(transparent)]
    TimelineTools(#[from] TimelineToolsHostRuntimeError),
    #[error(transparent)]
    Layout(#[from] crate::layout::LayoutError),
    #[error(transparent)]
    NativeHostLayout(#[from] crate::native_host_layout::NativeHostLayoutError),
    #[error("native product Surface is unsupported by the selected adapter")]
    SurfaceUnsupported,
    #[error("native product Host was initialized twice")]
    AlreadyInitialized,
    #[error("native product layout epoch is exhausted")]
    LayoutEpochExhausted,
    #[error("Browser instance epoch is exhausted")]
    BrowserInstanceEpochExhausted,
    #[error("Browser lifecycle coordinator is unavailable")]
    BrowserLifecycleUnavailable,
    #[error("Place terminal cannot be classified without a native Host layout")]
    PlaceTerminalLayoutUnavailable,
    #[error("Place admission rejected capture generation {0}")]
    PlaceAdmissionGenerationRejected(u64),
    #[error("Place capture generation is exhausted")]
    PlaceGenerationExhausted,
    #[error("Place Stage NDC could not be converted through the displayed camera")]
    PlaceCanonicalConversion,
    #[error(transparent)]
    DocumentEdit(#[from] DocumentEditRuntimeError),
    #[error(transparent)]
    DocumentDispatch(#[from] DocumentEditDispatchError),
    #[error(transparent)]
    InputRouter(#[from] InputRouterError),
    #[error(transparent)]
    CommandRegistry(#[from] CommandRegistryError),
    #[error(transparent)]
    CommandId(#[from] CommandIdError),
    #[error(transparent)]
    Modifier(#[from] ModifierError),
    #[error(transparent)]
    AsciiKey(#[from] crate::AsciiKeyError),
    #[error("product command keymap contains an invalid or conflicting binding")]
    CommandKeymap,
    #[error("native product runtime failed: {0}")]
    Runtime(String),
}
