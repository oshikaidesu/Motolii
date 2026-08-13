//! motolii-ui: egui UI adapter層。
//!
//! toolkit APIはprivate module内に閉じ、domain/coreの公開契約へは出さない。

mod app;
mod browser_host;
mod browser_host_runtime;
mod command_registry;
mod diagnostic;
mod diagnostic_projection;
mod display_slot;
mod document_command_request;
mod document_edit_runtime;
mod domain_intent;
mod host_pointer_capture;
mod input_router;
mod inspector_host_runtime;
mod interaction_state;
mod keymap;
mod keymap_codec;
mod layout;
mod layout_authority;
mod layout_geometry;
mod layout_runtime;
mod layout_runtime_adapter;
mod media_library;
mod native_host_layout;
mod parameter_control;
mod product_easing_popup;
mod product_runtime;
mod product_runtime_adapter;
mod rn_product_host;

#[doc(hidden)]
pub use rn_product_host::{
    host_commit_stage_transform_for_app, host_create_for_test, host_destroy_for_test,
    host_destroy_stage_for_test, host_dispatch_intent_for_test,
    host_preview_stage_transform_for_app, host_read_snapshot_for_test,
    host_register_stage_for_test, host_render_frame_for_app, AppStageFrame, AppStageGeometry,
    AppStageGeometryLayer, AppStageTransformEdit, AppStageTransformError, AppStageTransformPreview,
    HostRenderFrameResult, RnHostError, RnHostReasonCode, RnHostTestIntent, RnHostTestResponse,
    RnProductSnapshotForTest,
};
// RN app staticlibが同一crate graph内からRust経由で呼ぶための再export。
// extern importで参照するとarchiveから当該objectが引かれずリンクに失敗する。
#[cfg(target_os = "macos")]
#[doc(hidden)]
pub use rn_product_host::{
    motolii_rn_host_create, motolii_rn_host_dispatch_intent_json, motolii_rn_host_projection_stamp,
    motolii_rn_host_read_snapshot_json, motolii_rn_stage_register,
};
mod render_worker;
mod shell;
mod stage_chrome_host_runtime;
mod stage_geometry_projection;
mod stage_hit_test;
mod stage_overlay_gpu;
pub mod stage_overlay_raster;
mod state_ownership;
mod static_preview;
mod timeline_move_gesture;
mod timeline_projection;
#[cfg(target_os = "macos")]
mod timeline_skia_raster;
mod timeline_tools_host_runtime;
mod timeline_trim_gesture;
mod ui_numeric_trace;
mod user_keymap_prefs;

pub use command_registry::{
    builtin_command_registry, CommandId, CommandIdError, CommandMetadata, CommandRegistry,
    CommandRegistryError,
};
pub use diagnostic::{
    adapt_command_error, adapt_document_command_request_error, adapt_input_router_error,
    DiagnosticActionKind, DiagnosticEnvelope, DiagnosticFact, DiagnosticReasonCode,
    DiagnosticRecoverability, DiagnosticSubject, UnsupportedDiagnosticSource,
};
pub use document_command_request::{DocumentCommandRequest, DocumentCommandRequestError};
pub use domain_intent::{DomainIntent, DomainIntentError};
pub use input_router::{
    ImeGateState, InputPhase, InputRouter, InputRouterError, NormalizedInput, RouterOutput,
    SafetyInterrupt,
};
pub use interaction_state::{
    InteractionState, InteractionStateMachine, InteractionTransitionError,
};
pub use keymap::{
    product_action_host_kind, product_builtin_keymap, resolve_keymap, resolve_product_action,
    AsciiKey, AsciiKeyError, Binding, BuiltinKeymap, DeltaOperation, EffectiveTrigger, Gesture,
    KeyToken, KeymapDelta, KeymapDiagnostic, KeymapResolution, Modifier, ModifierError, Modifiers,
    PlatformBindingConstraints, PlatformCommandModifier, PointerButton, ProductAction,
    PRODUCT_BUILTIN_KEYMAP_VERSION, PRODUCT_HOST_KIND_DUPLICATE, PRODUCT_HOST_KIND_MUTE,
    PRODUCT_HOST_KIND_SHUTTLE_FORWARD, PRODUCT_HOST_KIND_SHUTTLE_REVERSE,
    PRODUCT_HOST_KIND_SHUTTLE_STOP, PRODUCT_HOST_KIND_SOLO, PRODUCT_HOST_KIND_TOGGLE_PLAYBACK,
    PRODUCT_HOST_KIND_TRIM_CLIP_IN, PRODUCT_HOST_KIND_TRIM_CLIP_OUT, PRODUCT_KEYMAP_PROFILE_ID,
    PRODUCT_UNWIRED_DUPLICATE, PRODUCT_UNWIRED_MUTE, PRODUCT_UNWIRED_SOLO, PRODUCT_UNWIRED_SPLIT,
};
pub use keymap_codec::{
    decode_keymap_json, encode_keymap_json, KeymapApplyError, KeymapCodecDiagnostic,
    KeymapCodecError, KeymapCodecLimits, LimitKind, LoadedKeymap, OpaqueOperationReason,
    KEYMAP_CODEC_VERSION,
};
pub use parameter_control::{
    map_parameter_control, HostParameterControl, ParameterControlError, ParameterControlSpec,
};
pub use shell::{run_shell, run_shell_with_project, ShellError};
pub use stage_geometry_projection::{
    project_stage_geometry, StageGeometryError, StageGeometryProjection, StageGeometryUnavailable,
    StageLayerGeometry, StageLayerProjection, StageLocalRect,
};
pub use state_ownership::{UiStateLifetime, UiStateOwner};
pub use static_preview::StaticPreviewError;
pub use timeline_projection::{
    project_timeline, TimelineBar, TimelineHit, TimelineKey, TimelineMetrics, TimelineProjection,
    TimelineProjectionError, TimelineUnsupported, TimelineViewport,
};
pub use user_keymap_prefs::{
    default_user_keymap_override_path, load_user_keymap_override, USER_KEYMAP_FILE_NAME,
};

/// 製品 UI クレートの識別子。依存方向 CI の許可リストと一致させる。
pub const CRATE_ID: &str = "motolii-ui";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiCrateInfo {
    pub crate_id: &'static str,
    pub toolkit_linked: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum UiError {
    #[error("egui runtime is not linked into {crate_id}")]
    ToolkitNotLinked { crate_id: &'static str },
}

/// U0a骨格: workspace上でegui依存が解決できることを返す。
pub fn crate_info() -> Result<UiCrateInfo, UiError> {
    let toolkit_linked = shell::toolkit_linked();
    if !toolkit_linked {
        return Err(UiError::ToolkitNotLinked { crate_id: CRATE_ID });
    }
    Ok(UiCrateInfo {
        crate_id: CRATE_ID,
        toolkit_linked,
    })
}
