//! motolii-ui: egui UI adapter層。
//!
//! toolkit APIはprivate module内に閉じ、domain/coreの公開契約へは出さない。

mod app;
mod command_registry;
mod diagnostic;
mod display_slot;
mod document_command_request;
mod document_edit_runtime;
mod domain_intent;
mod input_router;
mod interaction_state;
mod keymap;
mod keymap_codec;
mod layout;
mod layout_authority;
mod layout_runtime;
mod layout_runtime_adapter;
mod render_worker;
mod shell;
mod state_ownership;
mod static_preview;

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
    resolve_keymap, AsciiKey, AsciiKeyError, Binding, BuiltinKeymap, DeltaOperation,
    EffectiveTrigger, Gesture, KeyToken, KeymapDelta, KeymapDiagnostic, KeymapResolution, Modifier,
    ModifierError, Modifiers, PlatformBindingConstraints, PlatformCommandModifier, PointerButton,
};
pub use keymap_codec::{
    decode_keymap_json, encode_keymap_json, KeymapApplyError, KeymapCodecDiagnostic,
    KeymapCodecError, KeymapCodecLimits, LimitKind, LoadedKeymap, OpaqueOperationReason,
    KEYMAP_CODEC_VERSION,
};
pub use shell::{run_shell, ShellError};
pub use state_ownership::{UiStateLifetime, UiStateOwner};
pub use static_preview::StaticPreviewError;

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
