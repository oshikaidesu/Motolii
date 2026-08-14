//! Motoliiの窓やtoolkitから独立した入力契約。

mod command_registry;
mod domain_intent;
mod input_router;
mod keymap;
mod keymap_codec;
mod state_ownership;

pub use command_registry::{
    builtin_command_registry, CommandId, CommandIdError, CommandMetadata, CommandRegistry,
    CommandRegistryError,
};
pub use domain_intent::{DomainIntent, DomainIntentError};
pub use input_router::{
    ImeGateState, InputPhase, InputRouter, InputRouterError, NormalizedInput, RouterOutput,
    SafetyInterrupt,
};
pub use keymap::{
    product_action_host_kind, product_action_repeat_disposition, product_builtin_keymap,
    resolve_keymap, resolve_product_action, AsciiKey, AsciiKeyError, Binding, BuiltinKeymap,
    DeltaOperation, EffectiveTrigger, Gesture, KeyToken, KeymapDelta, KeymapDiagnostic,
    KeymapResolution, Modifier, ModifierError, Modifiers, PlatformBindingConstraints,
    PlatformCommandModifier, PointerButton, ProductAction, RepeatDisposition,
    PRODUCT_BUILTIN_KEYMAP_VERSION, PRODUCT_HOST_KIND_COPY, PRODUCT_HOST_KIND_CUT,
    PRODUCT_HOST_KIND_DUPLICATE, PRODUCT_HOST_KIND_GOTO_NEXT_KEY, PRODUCT_HOST_KIND_GOTO_NEXT_STEP,
    PRODUCT_HOST_KIND_GOTO_PREV_KEY, PRODUCT_HOST_KIND_GOTO_PREV_STEP, PRODUCT_HOST_KIND_MUTE,
    PRODUCT_HOST_KIND_PASTE, PRODUCT_HOST_KIND_SELECT_ALL, PRODUCT_HOST_KIND_SHUTTLE_FORWARD,
    PRODUCT_HOST_KIND_SHUTTLE_REVERSE, PRODUCT_HOST_KIND_SHUTTLE_STOP, PRODUCT_HOST_KIND_SOLO,
    PRODUCT_HOST_KIND_TOGGLE_PLAYBACK, PRODUCT_HOST_KIND_TRIM_CLIP_IN,
    PRODUCT_HOST_KIND_TRIM_CLIP_OUT, PRODUCT_KEYMAP_PROFILE_ID, PRODUCT_UNWIRED_DUPLICATE,
    PRODUCT_UNWIRED_MUTE, PRODUCT_UNWIRED_SOLO, PRODUCT_UNWIRED_SPLIT,
};
pub use keymap_codec::{
    decode_keymap_json, encode_keymap_json, KeymapApplyError, KeymapCodecDiagnostic,
    KeymapCodecError, KeymapCodecLimits, LimitKind, LoadedKeymap, OpaqueOperationReason,
    KEYMAP_CODEC_VERSION,
};
pub use state_ownership::{UiStateLifetime, UiStateOwner};
