//! motolii-ui: egui UI adapter層。
//!
//! toolkit APIはprivate module内に閉じ、domain/coreの公開契約へは出さない。

mod canonical_drop;
/// 移植済みBlitzパネルを1つの窓へ合体させる殻(eframe host + egui_tiles)。bin から使うので pub。
pub mod blitz_shell;
mod blitz_css;
mod blitz_ui;
/// C6: フォルダ参照でメディア入口を開くBrowserパネル(Blitz)。
pub mod browser_blitz;
mod browser_host;
// C8: chrome と extension panel の見た目のBlitz(HTML/CSS)写し。bin から使うので pub。
pub mod chrome_blitz;
mod diagnostic;
mod diagnostic_projection;
mod display_slot;
mod egui_fonts;
pub mod dock;
mod document_command_request;
mod host_pointer_capture;
// C7: Inspector の見た目のBlitz(HTML/CSS)写し。bin から使うので pub。
pub mod inspector_blitz;
mod interaction_state;
mod layout;
mod layout_authority;
mod layout_geometry;
mod layout_runtime;
mod layout_runtime_adapter;
mod media_library;
mod native_host_layout;
mod palette_settings;
mod parameter_control;
mod product_easing_popup;
mod preview_pipeline;
/// Rerun Spatial Viewer の Stage。Motolii はこのViewerの wrapper であり、
/// `re_renderer` で直接シーンを組まない(2026-08-11裁定)。bin から使うので pub。
pub mod rerun_stage;

mod render_worker;
mod shell;
mod stage_app_geometry;
mod stage_geometry_projection;
mod stage_hit_test;
mod stage_overlay_gpu;
pub mod stage_overlay_raster;
mod static_preview;
pub mod timeline_blitz;
mod timeline_move_gesture;
mod timeline_projection;
#[cfg(target_os = "macos")]
mod timeline_skia_raster;
mod timeline_trim_gesture;
mod timeline_viewport_state;
mod ui_numeric_trace;
mod user_keymap_prefs;

pub use diagnostic::{
    adapt_command_error, adapt_document_command_request_error, adapt_input_router_error,
    DiagnosticActionKind, DiagnosticEnvelope, DiagnosticFact, DiagnosticReasonCode,
    DiagnosticRecoverability, DiagnosticSubject, UnsupportedDiagnosticSource,
};
pub use document_command_request::{DocumentCommandRequest, DocumentCommandRequestError};
pub use interaction_state::{
    InteractionState, InteractionStateMachine, InteractionTransitionError,
};
pub use motolii_input::{
    builtin_command_registry, decode_keymap_json, encode_keymap_json, product_action_host_kind,
    product_action_repeat_disposition, product_builtin_keymap, resolve_keymap,
    resolve_product_action, AsciiKey, AsciiKeyError, Binding, BuiltinKeymap, CommandId,
    CommandIdError, CommandMetadata, CommandRegistry, CommandRegistryError, DeltaOperation,
    DomainIntent, DomainIntentError, EffectiveTrigger, Gesture, ImeGateState, InputPhase,
    InputRouter, InputRouterError, KeyToken, KeymapApplyError, KeymapCodecDiagnostic,
    KeymapCodecError, KeymapCodecLimits, KeymapDelta, KeymapDiagnostic, KeymapResolution,
    LimitKind, LoadedKeymap, Modifier, ModifierError, Modifiers, NormalizedInput,
    OpaqueOperationReason, PlatformBindingConstraints, PlatformCommandModifier, PointerButton,
    ProductAction, RepeatDisposition, RouterOutput, SafetyInterrupt, UiStateLifetime, UiStateOwner,
    KEYMAP_CODEC_VERSION, PRODUCT_BUILTIN_KEYMAP_VERSION, PRODUCT_HOST_KIND_COPY,
    PRODUCT_HOST_KIND_CUT, PRODUCT_HOST_KIND_DUPLICATE, PRODUCT_HOST_KIND_GOTO_NEXT_KEY,
    PRODUCT_HOST_KIND_GOTO_NEXT_STEP, PRODUCT_HOST_KIND_GOTO_PREV_KEY,
    PRODUCT_HOST_KIND_GOTO_PREV_STEP, PRODUCT_HOST_KIND_MUTE, PRODUCT_HOST_KIND_PASTE,
    PRODUCT_HOST_KIND_SELECT_ALL, PRODUCT_HOST_KIND_SHUTTLE_FORWARD,
    PRODUCT_HOST_KIND_SHUTTLE_REVERSE, PRODUCT_HOST_KIND_SHUTTLE_STOP, PRODUCT_HOST_KIND_SOLO,
    PRODUCT_HOST_KIND_TOGGLE_PLAYBACK, PRODUCT_HOST_KIND_TRIM_CLIP_IN,
    PRODUCT_HOST_KIND_TRIM_CLIP_OUT, PRODUCT_KEYMAP_PROFILE_ID, PRODUCT_UNWIRED_DUPLICATE,
    PRODUCT_UNWIRED_MUTE, PRODUCT_UNWIRED_SOLO, PRODUCT_UNWIRED_SPLIT,
};
pub use palette_settings::{
    default_user_palette_settings_path, Palette, PaletteColor, PaletteColorId, PaletteId,
    PaletteSettings, PaletteSettingsError, Rgba, PALETTE_SETTINGS_VERSION, USER_PALETTE_FILE_NAME,
};
pub use parameter_control::{
    map_parameter_control, HostParameterControl, ParameterControlError, ParameterControlSpec,
};
pub use shell::ShellError;
pub use stage_geometry_projection::{
    project_stage_geometry, StageGeometryError, StageGeometryProjection, StageGeometryUnavailable,
    StageLayerGeometry, StageLayerProjection, StageLocalRect,
};
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
