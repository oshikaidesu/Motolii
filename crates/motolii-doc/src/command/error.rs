use thiserror::Error;

use crate::asset::AssetError;
use crate::doc_keyframe::DocKeyframeError;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum CommandError {
    #[error("command macro must contain at least one command")]
    EmptyMacro,
    #[error("layer {0} not found")]
    LayerNotFound(u64),
    #[error("track item {layer} is not a Clip")]
    TrackItemNotClip { layer: u64 },
    #[error("split time must be strictly inside clip {layer}")]
    SplitNotInterior { layer: u64 },
    #[error("invalid clip in-trim payload for layer {layer}")]
    InvalidClipTrim { layer: u64 },
    #[error("track {0} not found")]
    TrackNotFound(u64),
    #[error("group {0} not found (or is not a Group)")]
    GroupNotFound(u64),
    #[error("effect {effect} not found on layer {layer}")]
    EffectNotFound { effect: u64, layer: u64 },
    #[error("layer {layer} source is not ClipSource::Plugin")]
    SourceNotPlugin { layer: u64 },
    #[error("source param `{param}` not found on layer {layer}")]
    SourceParamNotFound { layer: u64, param: String },
    #[error("audio component index {index} not found on layer {layer}")]
    AudioComponentNotFound { layer: u64, index: usize },
    #[error("detach audio destination must be a different track/group lane than the source")]
    DetachSameLane,
    #[error("track item index {index} out of range (len={len})")]
    IndexOutOfRange { index: usize, len: usize },
    #[error(
        "removed track item does not match expected layer id (expected {expected}, found {found})"
    )]
    RemoveItemMismatch { expected: u64, found: u64 },
    #[error("removed effect does not match expected id (expected {expected}, found {found})")]
    RemoveEffectMismatch { expected: u64, found: u64 },
    #[error("removed effect definition payload does not match ledger (definition {id})")]
    RemoveEffectDefinitionMismatch { id: u64 },
    #[error("effect definition {id} not found")]
    EffectDefinitionNotFound { id: u64 },
    /// GAP-14§2.1: 参照中Definitionの削除はReject(Cascadeしない)。
    #[error("effect definition {id} is in use by effect use(s) {use_ids:?}")]
    DefinitionInUse { id: u64, use_ids: Vec<u64> },
    #[error("effect definition {id} already exists")]
    EffectDefinitionAlreadyExists { id: u64 },
    #[error("effect definition {id} payload does not match existing ledger entry")]
    EffectDefinitionMismatch { id: u64 },
    #[error("stable id {id} already exists in the shared document-local id space")]
    StableIdCollision { id: u64 },
    #[error(
        "undo copy-local rejected: definition {id} is still shared by other use(s) {use_ids:?}"
    )]
    UndoCopyLocalDefinitionInUse { id: u64, use_ids: Vec<u64> },
    #[error("copy-local effect use definition mismatch (expected {expected}, found {found})")]
    CopyLocalDefinitionMismatch { expected: u64, found: u64 },
    #[error("copy-local payload does not match source definition semantics")]
    CopyLocalPayloadMismatch,
    #[error("add position key payload mismatch (layer {layer}, key_id {key_id})")]
    AddPositionKeyPayloadMismatch { layer: u64, key_id: u64 },
    #[error("remove position key payload mismatch (layer {layer}, key_id {key_id})")]
    RemovePositionKeyPayloadMismatch { layer: u64, key_id: u64 },
    #[error("position interpolation source unsupported for layer {layer}")]
    PositionKeyInterpSourceUnsupported { layer: u64 },
    #[error("position interpolation value type mismatch for layer {layer}")]
    PositionKeyInterpValueTypeMismatch { layer: u64 },
    #[error("position key {key_id} not found on layer {layer}")]
    PositionKeyNotFound { layer: u64, key_id: u64 },
    #[error("position key interpolation invalid for key {key_id} on layer {layer}")]
    PositionKeyInterpInvalid {
        layer: u64,
        key_id: u64,
        source: DocKeyframeError,
    },
    #[error("position key interpolation payload mismatch for key {key_id} on layer {layer}")]
    PositionKeyInterpPayloadMismatch { layer: u64, key_id: u64 },
    #[error("position key value source unsupported for layer {layer}")]
    PositionKeyValueSourceUnsupported { layer: u64 },
    #[error("position key value type mismatch for layer {layer}")]
    PositionKeyValueTypeMismatch { layer: u64 },
    #[error("position key value not found for key {key_id} on layer {layer}")]
    PositionKeyValueNotFound { layer: u64, key_id: u64 },
    #[error("position key value for key {key_id} on layer {layer} must be finite")]
    PositionKeyValueNonFinite { layer: u64, key_id: u64 },
    #[error("position key value payload mismatch for key {key_id} on layer {layer}")]
    PositionKeyValuePayloadMismatch { layer: u64, key_id: u64 },
    #[error("position key time source unsupported for layer {layer}")]
    PositionKeyTimeSourceUnsupported { layer: u64 },
    #[error("position key time not found for key {key_id} on layer {layer}")]
    PositionKeyTimeNotFound { layer: u64, key_id: u64 },
    #[error("position key time payload mismatch for key {key_id} on layer {layer}")]
    PositionKeyTimePayloadMismatch { layer: u64, key_id: u64 },
    #[error("position key time {key_id} on layer {layer} would collide at destination")]
    PositionKeyTimeOccupied { layer: u64, key_id: u64 },
    #[error("position key time for key {key_id} on layer {layer} must be >= 0")]
    PositionKeyTimeNegative { layer: u64, key_id: u64 },
    #[error("transform param key time source unsupported for layer {layer}")]
    TransformParamKeyTimeSourceUnsupported { layer: u64 },
    #[error("transform param key time not found for key {key_id} on layer {layer}")]
    TransformParamKeyTimeNotFound { layer: u64, key_id: u64 },
    #[error("transform param key time payload mismatch for key {key_id} on layer {layer}")]
    TransformParamKeyTimePayloadMismatch { layer: u64, key_id: u64 },
    #[error("transform param key time {key_id} on layer {layer} would collide at destination")]
    TransformParamKeyTimeOccupied { layer: u64, key_id: u64 },
    #[error("transform param key time for key {key_id} on layer {layer} must be >= 0")]
    TransformParamKeyTimeNegative { layer: u64, key_id: u64 },
    #[error("effect use {use_id} not found in document")]
    EffectUseNotFound { use_id: u64 },
    #[error(
        "effect lifecycle v2 commands require a migrated v4 effect-definition document (version={version}, min_reader_version={min_reader_version})"
    )]
    EffectLifecycleRequiresV4Document {
        version: u32,
        min_reader_version: u32,
    },
    #[error("stable id reservation interval must be non-empty (before={before}, after={after})")]
    InvalidStableIdReservationInterval { before: u64, after: u64 },
    #[error(
        "stable id reservation introduced ids {introduced:?} do not match interval [{before}, {after})"
    )]
    StableIdReservationMismatch {
        before: u64,
        after: u64,
        introduced: Vec<u64>,
    },
    #[error(
        "stable id reservation counter mismatch (next={next}, before={before}, after={after})"
    )]
    StableIdReservationCounterMismatch { next: u64, before: u64, after: u64 },
    #[error("stable id {id} is outside reservation interval [{before}, {after})")]
    StableIdOutsideReservation { id: u64, before: u64, after: u64 },
    #[error("asset admission id {id} is ahead of live AssetTable next {next}")]
    AssetAdmissionCounterMismatch { id: u64, next: u64 },
    #[error("stale asset admission (prepared next={expected_next}, live next={actual_next})")]
    StaleAssetAdmission {
        expected_next: u64,
        actual_next: u64,
    },
    #[error("asset {id} is still referenced {use_count} time(s)")]
    AssetInUse { id: u64, use_count: usize },
    #[error("removed asset payload does not match live AssetTable entry {id}")]
    RemoveAssetPayloadMismatch { id: u64 },
    #[error(transparent)]
    Validate(#[from] crate::validate::DocumentError),
    #[error(transparent)]
    Plugin(#[from] crate::DocumentPluginError),
    #[error(
        "layer_names keys do not match track item subtree (item={item_layers:?}, names={named_layers:?})"
    )]
    LayerNamesMismatch {
        item_layers: Vec<u64>,
        named_layers: Vec<u64>,
    },
    #[error(transparent)]
    LayerIdAlloc(#[from] crate::LayerIdError),
    #[error("marker payload mismatch at index {index}")]
    MarkerMismatch { index: usize },
    #[error(transparent)]
    StableIdAlloc(#[from] crate::stable_id::StableIdError),
    #[error(transparent)]
    Asset(#[from] AssetError),
}
