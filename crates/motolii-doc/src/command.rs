//! D2: コマンドシステム(apply/revert)。#103⑨: atomic command=property単位。
//!
//! **コマンドは決定済みの値を記録する**(実装ガード5)。「ドラッグ中」等の意図やデルタは
//! 持たず、apply/revertはold_value/new_valueの単純な書き込みで成立する(対称設計)。
//! 選択・hover・IME中間状態はこのenumに入れない(#103⑨、UI状態のまま)。
//!
//! **スコープ外(本PR)**: `ClipSource::Plugin`/`VectorContent`/`PathOp`配下のDocParam編集
//! コマンドはD1i-2(#100)と並走のため対象外。複製時のID再写像は`duplicate`が担当する。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::duplicate::{definition_semantic_body_eq, remint_order_keyframe_ids};
use crate::param::DocParam;
use crate::schema::{
    AudioComponent, BlendMode, ClipSource, ClippingMaskSettings, EffectDefinition, EffectInstance,
    EffectUse, ItemEnvelope, TrackItem,
};
use crate::stable_id::{EffectDefinitionId, EffectId, StableIdReservation};
use crate::track_id::TrackId;
use crate::validate::{self, stable_id_in_use};
use crate::{Document, LayerId, WRITER_VERSION};

/// `SetProperty`が書き込める閉じたプロパティ集合(envelope本体+effect params)。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScalarPropertyId {
    Position,
    Anchor,
    Scale,
    Rotation,
    Opacity,
    EffectParam(EffectId, String),
}

/// merge key(S18)の`property_id`成分。全コマンド種別を横断する。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PropertyId {
    Position,
    Anchor,
    Scale,
    Rotation,
    Opacity,
    Blend,
    ClippingMask,
    TransformParent,
    EffectEnabled(EffectId),
    EffectParam(EffectId, String),
    EffectList(EffectId),
    /// D1l: `DeleteEffectDefinition`/`AddEffectDefinition`(台帳の生存)。
    EffectDefinitionLifecycle(EffectDefinitionId),
    /// D1l: `CopyLocalEffect`/`UndoCopyLocalEffect`(1つのUseのdefinition_id付け替え)。
    EffectDefinitionLink(EffectId),
    AudioEnabled(usize),
    AudioGain(usize),
    ChildList,
}

impl From<ScalarPropertyId> for PropertyId {
    fn from(p: ScalarPropertyId) -> Self {
        match p {
            ScalarPropertyId::Position => PropertyId::Position,
            ScalarPropertyId::Anchor => PropertyId::Anchor,
            ScalarPropertyId::Scale => PropertyId::Scale,
            ScalarPropertyId::Rotation => PropertyId::Rotation,
            ScalarPropertyId::Opacity => PropertyId::Opacity,
            ScalarPropertyId::EffectParam(id, name) => PropertyId::EffectParam(id, name),
        }
    }
}

/// `AddTrackItem`/`RemoveTrackItem`の挿入先。トップレベルTrackか、Group内(ネスト)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParentLocator {
    Track(TrackId),
    Group(LayerId),
}

/// merge key(S18)の`gesture_id`成分。UI側のジェスチャ(ドラッグ等)単位で発行する
/// 実行時カウンタ — Document schemaには入れない(選択/操作状態はUI都合)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GestureId(u64);

impl GestureId {
    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

/// merge key(S18)の`command_kind`成分。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandKind {
    SetProperty,
    SetBlendMode,
    SetClippingMask,
    SetTransformParent,
    AddEffect,
    RemoveEffect,
    /// D1l v2: `CreateEffect` / `UndoCreateEffect`(inverse)共用。
    CreateEffect,
    /// D1l v2: `LinkEffectUse` / `UndoLinkEffectUse`(inverse)共用。
    LinkEffectUse,
    /// D1l v2: `UnlinkEffectUse` / `RestoreEffectUse`(inverse)共用。
    UnlinkEffectUse,
    SetEffectEnabled,
    /// D1l: `DeleteEffectDefinition` / `AddEffectDefinition`(inverse)共用。
    DeleteEffectDefinition,
    /// D1l v2: `CopyLocalEffect` / `UndoCopyLocalEffect`(inverse)共用。
    CopyLocalEffect,
    SetAudioComponentEnabled,
    SetAudioComponentGain,
    AddTrackItem,
    RemoveTrackItem,
}

/// S18: `gesture_id + command_kind + target_stable_id + property_id`。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MergeKey {
    pub gesture: GestureId,
    pub kind: CommandKind,
    pub target_stable_id: u64,
    pub property: PropertyId,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum CommandError {
    #[error("layer {0} not found")]
    LayerNotFound(u64),
    #[error("track {0} not found")]
    TrackNotFound(u64),
    #[error("group {0} not found (or is not a Group)")]
    GroupNotFound(u64),
    #[error("effect {effect} not found on layer {layer}")]
    EffectNotFound { effect: u64, layer: u64 },
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
    #[error(transparent)]
    StableIdAlloc(#[from] crate::stable_id::StableIdError),
}

/// atomic command(実装ガード5: 決定済みの値を記録)。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Command {
    SetProperty {
        target: LayerId,
        property: ScalarPropertyId,
        old_value: DocParam,
        new_value: DocParam,
    },
    SetBlendMode {
        target: LayerId,
        old: BlendMode,
        new: BlendMode,
    },
    SetClippingMask {
        target: LayerId,
        old: ClippingMaskSettings,
        new: ClippingMaskSettings,
    },
    SetTransformParent {
        target: LayerId,
        old: Option<LayerId>,
        new: Option<LayerId>,
    },
    AddEffect {
        target: LayerId,
        index: usize,
        effect: EffectInstance,
        /// `true` iff applyが台帳へ新規Definitionを挿入した(create)。
        /// Undo(inverse RemoveEffect)はこのときだけDefinitionも戻す。ユーザーUnlinkは`false`。
        introduced_definition: bool,
    },
    RemoveEffect {
        target: LayerId,
        index: usize,
        effect: EffectInstance,
        /// `AddEffect(introduced_definition=true)`のinverse専用。ユーザーUnlinkは常に`false`。
        introduced_definition: bool,
    },
    SetEffectEnabled {
        target: LayerId,
        effect: EffectId,
        old: bool,
        new: bool,
    },
    /// D1l/GAP-14§2.5: `use_count==0`のときのみ成立(採否は`apply`側)。
    DeleteEffectDefinition { definition: EffectDefinition },
    /// `DeleteEffectDefinition`のinverse専用(未参照時のみ台帳へ復元)。
    AddEffectDefinition { definition: EffectDefinition },
    /// D1l v2: 新Use+新Definitionを同時挿入(create)。
    CreateEffect {
        target: LayerId,
        index: usize,
        #[serde(rename = "use")]
        use_: EffectUse,
        definition: EffectDefinition,
        stable_id_reservation: StableIdReservation,
    },
    /// `CreateEffect`のinverse専用。
    UndoCreateEffect {
        target: LayerId,
        index: usize,
        #[serde(rename = "use")]
        use_: EffectUse,
        definition: EffectDefinition,
        stable_id_reservation: StableIdReservation,
    },
    /// D1l v2: 既存Definitionへ新Useを挿入(link)。
    LinkEffectUse {
        target: LayerId,
        index: usize,
        #[serde(rename = "use")]
        use_: EffectUse,
        stable_id_reservation: StableIdReservation,
    },
    /// `LinkEffectUse`のinverse専用。
    UndoLinkEffectUse {
        target: LayerId,
        index: usize,
        #[serde(rename = "use")]
        use_: EffectUse,
        stable_id_reservation: StableIdReservation,
    },
    /// D1l v2/GAP-14§2.2: 対象Useだけ除去。Definitionはorphan keep。
    UnlinkEffectUse {
        target: LayerId,
        index: usize,
        #[serde(rename = "use")]
        use_: EffectUse,
    },
    /// `UnlinkEffectUse`のinverse専用。
    RestoreEffectUse {
        target: LayerId,
        index: usize,
        #[serde(rename = "use")]
        use_: EffectUse,
    },
    /// D1l v2/GAP-14§2.3 Materialize: 採番済み完全payloadで当該Useだけ付け替える。
    CopyLocalEffect {
        use_id: EffectId,
        previous_definition_id: EffectDefinitionId,
        new_definition: EffectDefinition,
        stable_id_reservation: StableIdReservation,
    },
    /// `CopyLocalEffect`のinverse専用。
    UndoCopyLocalEffect {
        use_id: EffectId,
        previous_definition_id: EffectDefinitionId,
        new_definition: EffectDefinition,
        stable_id_reservation: StableIdReservation,
    },
    SetAudioComponentEnabled {
        target: LayerId,
        /// `ClipSource::Asset.audio` Vec内のindex(ordinalではない)。
        index: usize,
        old: bool,
        new: bool,
    },
    SetAudioComponentGain {
        target: LayerId,
        /// `ClipSource::Asset.audio` Vec内のindex(ordinalではない)。
        index: usize,
        old: DocParam,
        new: DocParam,
    },
    AddTrackItem {
        parent: ParentLocator,
        index: usize,
        item: TrackItem,
        /// subtreeの表示名。applyで台帳へ載せ、Removeのinverseで戻す。
        layer_names: BTreeMap<LayerId, String>,
    },
    RemoveTrackItem {
        parent: ParentLocator,
        index: usize,
        item: TrackItem,
        /// subtreeの表示名。台帳から外したあと、inverseのAddで復元する。
        layer_names: BTreeMap<LayerId, String>,
    },
}

impl Command {
    pub fn kind(&self) -> CommandKind {
        match self {
            Command::SetProperty { .. } => CommandKind::SetProperty,
            Command::SetBlendMode { .. } => CommandKind::SetBlendMode,
            Command::SetClippingMask { .. } => CommandKind::SetClippingMask,
            Command::SetTransformParent { .. } => CommandKind::SetTransformParent,
            Command::AddEffect { .. } => CommandKind::AddEffect,
            Command::RemoveEffect { .. } => CommandKind::RemoveEffect,
            Command::CreateEffect { .. } | Command::UndoCreateEffect { .. } => {
                CommandKind::CreateEffect
            }
            Command::LinkEffectUse { .. } | Command::UndoLinkEffectUse { .. } => {
                CommandKind::LinkEffectUse
            }
            Command::UnlinkEffectUse { .. } | Command::RestoreEffectUse { .. } => {
                CommandKind::UnlinkEffectUse
            }
            Command::SetEffectEnabled { .. } => CommandKind::SetEffectEnabled,
            Command::DeleteEffectDefinition { .. } | Command::AddEffectDefinition { .. } => {
                CommandKind::DeleteEffectDefinition
            }
            Command::CopyLocalEffect { .. } | Command::UndoCopyLocalEffect { .. } => {
                CommandKind::CopyLocalEffect
            }
            Command::SetAudioComponentEnabled { .. } => CommandKind::SetAudioComponentEnabled,
            Command::SetAudioComponentGain { .. } => CommandKind::SetAudioComponentGain,
            Command::AddTrackItem { .. } => CommandKind::AddTrackItem,
            Command::RemoveTrackItem { .. } => CommandKind::RemoveTrackItem,
        }
    }

    /// merge keyの`target_stable_id`(S18)。envelope系はLayerId、構造系は対象項目のLayerId。
    pub fn target_stable_id(&self) -> u64 {
        match self {
            Command::SetProperty { target, .. }
            | Command::SetBlendMode { target, .. }
            | Command::SetClippingMask { target, .. }
            | Command::SetTransformParent { target, .. }
            | Command::AddEffect { target, .. }
            | Command::RemoveEffect { target, .. }
            | Command::SetEffectEnabled { target, .. }
            | Command::SetAudioComponentEnabled { target, .. }
            | Command::SetAudioComponentGain { target, .. } => target.get(),
            Command::CreateEffect { target, .. }
            | Command::UndoCreateEffect { target, .. }
            | Command::LinkEffectUse { target, .. }
            | Command::UndoLinkEffectUse { target, .. }
            | Command::UnlinkEffectUse { target, .. }
            | Command::RestoreEffectUse { target, .. } => target.get(),
            Command::CopyLocalEffect { use_id, .. }
            | Command::UndoCopyLocalEffect { use_id, .. } => use_id.get(),
            Command::DeleteEffectDefinition { definition }
            | Command::AddEffectDefinition { definition } => definition.id.get(),
            Command::AddTrackItem { item, .. } | Command::RemoveTrackItem { item, .. } => {
                envelope_of(item).layer_id.get()
            }
        }
    }

    pub fn property(&self) -> PropertyId {
        match self {
            Command::SetProperty { property, .. } => property.clone().into(),
            Command::SetBlendMode { .. } => PropertyId::Blend,
            Command::SetClippingMask { .. } => PropertyId::ClippingMask,
            Command::SetTransformParent { .. } => PropertyId::TransformParent,
            Command::AddEffect { effect, .. } | Command::RemoveEffect { effect, .. } => {
                PropertyId::EffectList(effect.id)
            }
            Command::CreateEffect { use_, .. }
            | Command::UndoCreateEffect { use_, .. }
            | Command::LinkEffectUse { use_, .. }
            | Command::UndoLinkEffectUse { use_, .. }
            | Command::UnlinkEffectUse { use_, .. }
            | Command::RestoreEffectUse { use_, .. } => PropertyId::EffectList(use_.id),
            Command::SetEffectEnabled { effect, .. } => PropertyId::EffectEnabled(*effect),
            Command::DeleteEffectDefinition { definition }
            | Command::AddEffectDefinition { definition } => {
                PropertyId::EffectDefinitionLifecycle(definition.id)
            }
            Command::CopyLocalEffect { use_id, .. }
            | Command::UndoCopyLocalEffect { use_id, .. } => {
                PropertyId::EffectDefinitionLink(*use_id)
            }
            Command::SetAudioComponentEnabled { index, .. } => PropertyId::AudioEnabled(*index),
            Command::SetAudioComponentGain { index, .. } => PropertyId::AudioGain(*index),
            Command::AddTrackItem { .. } | Command::RemoveTrackItem { .. } => PropertyId::ChildList,
        }
    }

    pub fn merge_key(&self, gesture: GestureId) -> MergeKey {
        MergeKey {
            gesture,
            kind: self.kind(),
            target_stable_id: self.target_stable_id(),
            property: self.property(),
        }
    }

    /// 新規stable identityを導入するv2 lifecycle variantだけ`Some`を返す(D1l/journal追補§2.2)。
    pub fn stable_id_reservation(&self) -> Option<StableIdReservation> {
        match self {
            Command::CreateEffect {
                stable_id_reservation,
                ..
            }
            | Command::UndoCreateEffect {
                stable_id_reservation,
                ..
            }
            | Command::LinkEffectUse {
                stable_id_reservation,
                ..
            }
            | Command::UndoLinkEffectUse {
                stable_id_reservation,
                ..
            }
            | Command::CopyLocalEffect {
                stable_id_reservation,
                ..
            }
            | Command::UndoCopyLocalEffect {
                stable_id_reservation,
                ..
            } => Some(*stable_id_reservation),
            Command::SetProperty { .. }
            | Command::SetBlendMode { .. }
            | Command::SetClippingMask { .. }
            | Command::SetTransformParent { .. }
            | Command::AddEffect { .. }
            | Command::RemoveEffect { .. }
            | Command::SetEffectEnabled { .. }
            | Command::DeleteEffectDefinition { .. }
            | Command::AddEffectDefinition { .. }
            | Command::UnlinkEffectUse { .. }
            | Command::RestoreEffectUse { .. }
            | Command::SetAudioComponentEnabled { .. }
            | Command::SetAudioComponentGain { .. }
            | Command::AddTrackItem { .. }
            | Command::RemoveTrackItem { .. } => None,
        }
    }

    /// `new`側を`Document`へ書き込む。
    pub fn apply(&self, doc: &mut Document) -> Result<(), CommandError> {
        match self {
            Command::SetProperty {
                target,
                property,
                new_value,
                ..
            } => match property {
                // D1l: EffectParamはEffectUseではなくEffectDefinition側を書き換える(共有Use全体へ反映)。
                ScalarPropertyId::EffectParam(effect_id, name) => {
                    let layer = target.get();
                    let definition_id = find_envelope(doc, *target)
                        .ok_or(CommandError::LayerNotFound(layer))?
                        .effects
                        .iter()
                        .find(|u| u.id == *effect_id)
                        .map(|u| u.definition_id)
                        .ok_or(CommandError::EffectNotFound {
                            effect: effect_id.get(),
                            layer,
                        })?;
                    let def = doc.effect_definition_mut(definition_id).ok_or(
                        CommandError::EffectDefinitionNotFound {
                            id: definition_id.get(),
                        },
                    )?;
                    def.params.insert(name.clone(), new_value.clone());
                    Ok(())
                }
                _ => {
                    let env = find_envelope_mut(doc, *target)?;
                    write_property(env, property, new_value.clone())
                }
            },
            Command::SetBlendMode { target, new, .. } => {
                find_envelope_mut(doc, *target)?.blend = *new;
                Ok(())
            }
            Command::SetClippingMask { target, new, .. } => {
                find_envelope_mut(doc, *target)?.clipping_mask = new.clone();
                Ok(())
            }
            Command::SetTransformParent { target, new, .. } => {
                find_envelope_mut(doc, *target)?.transform.parent = *new;
                Ok(())
            }
            Command::AddEffect {
                target,
                index,
                effect,
                introduced_definition,
            } => {
                let env =
                    find_envelope(doc, *target).ok_or(CommandError::LayerNotFound(target.get()))?;
                if *index > env.effects.len() {
                    return Err(CommandError::IndexOutOfRange {
                        index: *index,
                        len: env.effects.len(),
                    });
                }
                let (use_, def) = effect.clone().into_use_and_definition();
                if stable_id_in_use(doc, use_.id.get()) {
                    return Err(CommandError::StableIdCollision { id: use_.id.get() });
                }
                match doc.effect_definition(def.id) {
                    Some(existing) if existing == &def => {
                        if *introduced_definition {
                            return Err(CommandError::EffectDefinitionAlreadyExists {
                                id: def.id.get(),
                            });
                        }
                    }
                    Some(_) => {
                        return Err(CommandError::EffectDefinitionMismatch { id: def.id.get() })
                    }
                    None => {
                        if !*introduced_definition {
                            return Err(CommandError::EffectDefinitionNotFound {
                                id: def.id.get(),
                            });
                        }
                        if stable_id_in_use(doc, def.id.get()) {
                            return Err(CommandError::StableIdCollision { id: def.id.get() });
                        }
                        doc.effect_definitions.push(def);
                    }
                }
                find_envelope_mut(doc, *target)?
                    .effects
                    .insert(*index, use_);
                Ok(())
            }
            Command::RemoveEffect {
                target,
                index,
                effect,
                introduced_definition,
            } => {
                let layer = target.get();
                let env = find_envelope(doc, *target).ok_or(CommandError::LayerNotFound(layer))?;
                if *index >= env.effects.len() {
                    return Err(CommandError::IndexOutOfRange {
                        index: *index,
                        len: env.effects.len(),
                    });
                }
                let at_index = env.effects[*index].clone();
                if at_index.id != effect.id {
                    return Err(CommandError::RemoveEffectMismatch {
                        expected: effect.id.get(),
                        found: at_index.id.get(),
                    });
                }
                if at_index.definition_id != effect.definition_id {
                    return Err(CommandError::RemoveEffectDefinitionMismatch {
                        id: effect.definition_id.get(),
                    });
                }
                let (_, expected_def) = effect.clone().into_use_and_definition();
                let ledger_def = doc
                    .effect_definition(effect.definition_id)
                    .ok_or(CommandError::EffectDefinitionNotFound {
                        id: effect.definition_id.get(),
                    })?
                    .clone();
                if ledger_def != expected_def {
                    return Err(CommandError::RemoveEffectDefinitionMismatch {
                        id: effect.definition_id.get(),
                    });
                }
                if *introduced_definition {
                    let remaining = doc.effect_use_count(effect.definition_id);
                    if remaining != 1 {
                        return Err(CommandError::DefinitionInUse {
                            id: effect.definition_id.get(),
                            use_ids: doc
                                .effect_use_ids(effect.definition_id)
                                .into_iter()
                                .map(|id| id.get())
                                .collect(),
                        });
                    }
                    if doc
                        .effect_definitions
                        .iter()
                        .position(|d| d.id == effect.definition_id)
                        .is_none()
                    {
                        return Err(CommandError::EffectDefinitionNotFound {
                            id: effect.definition_id.get(),
                        });
                    }
                }
                find_envelope_mut(doc, *target)?.effects.remove(*index);
                if *introduced_definition {
                    let idx = doc
                        .effect_definitions
                        .iter()
                        .position(|d| d.id == effect.definition_id)
                        .ok_or(CommandError::EffectDefinitionNotFound {
                            id: effect.definition_id.get(),
                        })?;
                    doc.effect_definitions.remove(idx);
                }
                Ok(())
            }
            Command::SetEffectEnabled {
                target,
                effect,
                new,
                ..
            } => {
                let layer = target.get();
                let definition_id = find_envelope(doc, *target)
                    .ok_or(CommandError::LayerNotFound(layer))?
                    .effects
                    .iter()
                    .find(|u| u.id == *effect)
                    .map(|u| u.definition_id)
                    .ok_or(CommandError::EffectNotFound {
                        effect: effect.get(),
                        layer,
                    })?;
                let def = doc.effect_definition_mut(definition_id).ok_or(
                    CommandError::EffectDefinitionNotFound {
                        id: definition_id.get(),
                    },
                )?;
                def.enabled = *new;
                Ok(())
            }
            Command::DeleteEffectDefinition { definition } => {
                let use_ids: Vec<u64> = doc
                    .effect_use_ids(definition.id)
                    .into_iter()
                    .map(|id| id.get())
                    .collect();
                if !use_ids.is_empty() {
                    return Err(CommandError::DefinitionInUse {
                        id: definition.id.get(),
                        use_ids,
                    });
                }
                let existing = doc.effect_definition(definition.id).ok_or(
                    CommandError::EffectDefinitionNotFound {
                        id: definition.id.get(),
                    },
                )?;
                if existing != definition {
                    return Err(CommandError::EffectDefinitionMismatch {
                        id: definition.id.get(),
                    });
                }
                let idx = doc
                    .effect_definitions
                    .iter()
                    .position(|d| d.id == definition.id)
                    .ok_or(CommandError::EffectDefinitionNotFound {
                        id: definition.id.get(),
                    })?;
                doc.effect_definitions.remove(idx);
                Ok(())
            }
            Command::AddEffectDefinition { definition } => {
                if doc.effect_definition(definition.id).is_some() {
                    return Err(CommandError::EffectDefinitionAlreadyExists {
                        id: definition.id.get(),
                    });
                }
                if stable_id_in_use(doc, definition.id.get()) {
                    return Err(CommandError::StableIdCollision {
                        id: definition.id.get(),
                    });
                }
                doc.effect_definitions.push(definition.clone());
                Ok(())
            }
            Command::CreateEffect {
                target,
                index,
                use_,
                definition,
                stable_id_reservation,
            } => apply_create_effect(
                doc,
                *target,
                *index,
                use_.clone(),
                definition.clone(),
                *stable_id_reservation,
            ),
            Command::UndoCreateEffect {
                target,
                index,
                use_,
                definition,
                stable_id_reservation,
            } => apply_undo_create_effect(
                doc,
                *target,
                *index,
                use_.clone(),
                definition.clone(),
                *stable_id_reservation,
            ),
            Command::LinkEffectUse {
                target,
                index,
                use_,
                stable_id_reservation,
            } => apply_link_effect_use(doc, *target, *index, use_.clone(), *stable_id_reservation),
            Command::UndoLinkEffectUse {
                target,
                index,
                use_,
                stable_id_reservation,
            } => apply_undo_link_effect_use(
                doc,
                *target,
                *index,
                use_.clone(),
                *stable_id_reservation,
            ),
            Command::UnlinkEffectUse {
                target,
                index,
                use_,
            } => apply_unlink_effect_use(doc, *target, *index, use_.clone()),
            Command::RestoreEffectUse {
                target,
                index,
                use_,
            } => apply_restore_effect_use(doc, *target, *index, use_.clone()),
            Command::CopyLocalEffect {
                use_id,
                previous_definition_id,
                new_definition,
                stable_id_reservation,
            } => apply_copy_local_effect(
                doc,
                *use_id,
                *previous_definition_id,
                new_definition.clone(),
                *stable_id_reservation,
            ),
            Command::UndoCopyLocalEffect {
                use_id,
                previous_definition_id,
                new_definition,
                stable_id_reservation,
            } => apply_undo_copy_local_effect(
                doc,
                *use_id,
                *previous_definition_id,
                new_definition.clone(),
                *stable_id_reservation,
            ),
            Command::SetAudioComponentEnabled {
                target, index, new, ..
            } => {
                find_audio_component_mut(doc, *target, *index)?.enabled = *new;
                Ok(())
            }
            Command::SetAudioComponentGain {
                target, index, new, ..
            } => {
                find_audio_component_mut(doc, *target, *index)?.gain = new.clone();
                Ok(())
            }
            Command::AddTrackItem {
                parent,
                index,
                item,
                layer_names,
            } => {
                // 事前検査のみ — 失敗時はツリー・台帳とも未変更。
                ensure_layer_names_match_item(item, layer_names)?;
                let len = find_items_vec(doc, *parent)?.len();
                if *index > len {
                    return Err(CommandError::IndexOutOfRange { index: *index, len });
                }
                // 載せる予定のIDについて、restoreがExhaustedになるケースだけ事前拒否。
                for id in layer_names.keys() {
                    if !doc.layers.contains(*id) && id.get() == u64::MAX {
                        return Err(CommandError::LayerIdAlloc(crate::LayerIdError::Exhausted));
                    }
                }

                // ここから更新。事前検査済みなので台帳→ツリーの順で確定する。
                for (id, name) in layer_names {
                    if !doc.layers.contains(*id) {
                        doc.layers.restore(*id, name.clone())?;
                    }
                }
                find_items_vec_mut(doc, *parent)?.insert(*index, item.clone());
                Ok(())
            }
            Command::RemoveTrackItem {
                parent,
                index,
                item,
                layer_names,
            } => {
                // 事前検査のみ — 失敗時はツリー・台帳とも未変更。
                ensure_layer_names_match_item(item, layer_names)?;
                let items = find_items_vec(doc, *parent)?;
                if *index >= items.len() {
                    return Err(CommandError::IndexOutOfRange {
                        index: *index,
                        len: items.len(),
                    });
                }
                let found = envelope_of(&items[*index]).layer_id;
                let expected = envelope_of(item).layer_id;
                if found != expected {
                    return Err(CommandError::RemoveItemMismatch {
                        expected: expected.get(),
                        found: found.get(),
                    });
                }
                for id in layer_names.keys() {
                    if !doc.layers.contains(*id) {
                        return Err(CommandError::LayerNotFound(id.get()));
                    }
                }

                find_items_vec_mut(doc, *parent)?.remove(*index);
                for id in layer_names.keys() {
                    doc.layers.remove(*id)?;
                }
                Ok(())
            }
        }
    }

    /// 対称な逆コマンド。`apply(&inverse())`が`revert`になる(実装ガード5の対称設計)。
    pub fn inverse(&self) -> Command {
        match self.clone() {
            Command::SetProperty {
                target,
                property,
                old_value,
                new_value,
            } => Command::SetProperty {
                target,
                property,
                old_value: new_value,
                new_value: old_value,
            },
            Command::SetBlendMode { target, old, new } => Command::SetBlendMode {
                target,
                old: new,
                new: old,
            },
            Command::SetClippingMask { target, old, new } => Command::SetClippingMask {
                target,
                old: new,
                new: old,
            },
            Command::SetTransformParent { target, old, new } => Command::SetTransformParent {
                target,
                old: new,
                new: old,
            },
            Command::AddEffect {
                target,
                index,
                effect,
                introduced_definition,
            } => Command::RemoveEffect {
                target,
                index,
                effect,
                introduced_definition,
            },
            Command::RemoveEffect {
                target,
                index,
                effect,
                introduced_definition,
            } => Command::AddEffect {
                target,
                index,
                effect,
                introduced_definition,
            },
            Command::SetEffectEnabled {
                target,
                effect,
                old,
                new,
            } => Command::SetEffectEnabled {
                target,
                effect,
                old: new,
                new: old,
            },
            Command::DeleteEffectDefinition { definition } => {
                Command::AddEffectDefinition { definition }
            }
            Command::AddEffectDefinition { definition } => {
                Command::DeleteEffectDefinition { definition }
            }
            Command::CreateEffect {
                target,
                index,
                use_,
                definition,
                stable_id_reservation,
            } => Command::UndoCreateEffect {
                target,
                index,
                use_,
                definition,
                stable_id_reservation,
            },
            Command::UndoCreateEffect {
                target,
                index,
                use_,
                definition,
                stable_id_reservation,
            } => Command::CreateEffect {
                target,
                index,
                use_,
                definition,
                stable_id_reservation,
            },
            Command::LinkEffectUse {
                target,
                index,
                use_,
                stable_id_reservation,
            } => Command::UndoLinkEffectUse {
                target,
                index,
                use_,
                stable_id_reservation,
            },
            Command::UndoLinkEffectUse {
                target,
                index,
                use_,
                stable_id_reservation,
            } => Command::LinkEffectUse {
                target,
                index,
                use_,
                stable_id_reservation,
            },
            Command::UnlinkEffectUse {
                target,
                index,
                use_,
            } => Command::RestoreEffectUse {
                target,
                index,
                use_,
            },
            Command::RestoreEffectUse {
                target,
                index,
                use_,
            } => Command::UnlinkEffectUse {
                target,
                index,
                use_,
            },
            Command::CopyLocalEffect {
                use_id,
                previous_definition_id,
                new_definition,
                stable_id_reservation,
            } => Command::UndoCopyLocalEffect {
                use_id,
                previous_definition_id,
                new_definition,
                stable_id_reservation,
            },
            Command::UndoCopyLocalEffect {
                use_id,
                previous_definition_id,
                new_definition,
                stable_id_reservation,
            } => Command::CopyLocalEffect {
                use_id,
                previous_definition_id,
                new_definition,
                stable_id_reservation,
            },
            Command::SetAudioComponentEnabled {
                target,
                index,
                old,
                new,
            } => Command::SetAudioComponentEnabled {
                target,
                index,
                old: new,
                new: old,
            },
            Command::SetAudioComponentGain {
                target,
                index,
                old,
                new,
            } => Command::SetAudioComponentGain {
                target,
                index,
                old: new,
                new: old,
            },
            Command::AddTrackItem {
                parent,
                index,
                item,
                layer_names,
            } => Command::RemoveTrackItem {
                parent,
                index,
                item,
                layer_names,
            },
            Command::RemoveTrackItem {
                parent,
                index,
                item,
                layer_names,
            } => Command::AddTrackItem {
                parent,
                index,
                item,
                layer_names,
            },
        }
    }
}

/// `item` subtreeのLayerId集合と`layer_names`のキーが一致することを要求する。
fn ensure_layer_names_match_item(
    item: &TrackItem,
    layer_names: &BTreeMap<LayerId, String>,
) -> Result<(), CommandError> {
    let mut ids = Vec::new();
    collect_layer_ids(item, &mut ids);
    if ids.len() != layer_names.len() || ids.iter().any(|id| !layer_names.contains_key(id)) {
        return Err(CommandError::LayerNamesMismatch {
            item_layers: ids.iter().map(|id| id.get()).collect(),
            named_layers: layer_names.keys().map(|id| id.get()).collect(),
        });
    }
    Ok(())
}

/// TrackItem subtreeのLayerIdを深さ優先で集める。
pub fn collect_layer_ids(item: &TrackItem, out: &mut Vec<LayerId>) {
    out.push(envelope_of(item).layer_id);
    if let TrackItem::Group(g) = item {
        for child in &g.children {
            collect_layer_ids(child, out);
        }
    }
}

/// Document台帳からsubtreeの表示名を拾う。RemoveTrackItem構築用。
pub fn layer_names_for_item(
    doc: &Document,
    item: &TrackItem,
) -> Result<BTreeMap<LayerId, String>, CommandError> {
    let mut ids = Vec::new();
    collect_layer_ids(item, &mut ids);
    let mut names = BTreeMap::new();
    for id in ids {
        let name = doc
            .layers
            .display_name(id)
            .ok_or(CommandError::LayerNotFound(id.get()))?
            .to_string();
        names.insert(id, name);
    }
    Ok(names)
}

/// `ScalarPropertyId::EffectParam`は`Command::apply`側で`Document.effect_definitions`を
/// 直接書き換える(D1l: paramsはUseではなくDefinitionが持つ)。ここには到達しない防御的分岐。
fn write_property(
    env: &mut ItemEnvelope,
    property: &ScalarPropertyId,
    value: DocParam,
) -> Result<(), CommandError> {
    match property {
        ScalarPropertyId::Position => env.transform.position = value,
        ScalarPropertyId::Anchor => env.transform.anchor = value,
        ScalarPropertyId::Scale => env.transform.scale = value,
        ScalarPropertyId::Rotation => env.transform.rotation = value,
        ScalarPropertyId::Opacity => env.opacity = value,
        ScalarPropertyId::EffectParam(effect_id, _) => {
            return Err(CommandError::EffectNotFound {
                effect: effect_id.get(),
                layer: env.layer_id.get(),
            });
        }
    }
    Ok(())
}

pub(crate) fn envelope_of(item: &TrackItem) -> &ItemEnvelope {
    match item {
        TrackItem::Clip(c) => &c.envelope,
        TrackItem::Group(g) => &g.envelope,
    }
}

pub(crate) fn envelope_of_mut(item: &mut TrackItem) -> &mut ItemEnvelope {
    match item {
        TrackItem::Clip(c) => &mut c.envelope,
        TrackItem::Group(g) => &mut g.envelope,
    }
}

fn find_envelope_mut_in_items(
    items: &mut [TrackItem],
    target: LayerId,
) -> Option<&mut ItemEnvelope> {
    for item in items.iter_mut() {
        if envelope_of(item).layer_id == target {
            return Some(envelope_of_mut(item));
        }
        if let TrackItem::Group(g) = item {
            if let Some(found) = find_envelope_mut_in_items(&mut g.children, target) {
                return Some(found);
            }
        }
    }
    None
}

pub(crate) fn find_envelope_mut(
    doc: &mut Document,
    target: LayerId,
) -> Result<&mut ItemEnvelope, CommandError> {
    for track in &mut doc.tracks {
        if let Some(found) = find_envelope_mut_in_items(&mut track.items, target) {
            return Ok(found);
        }
    }
    Err(CommandError::LayerNotFound(target.get()))
}

/// `target`のAsset Clipから`audio[index]`を返す。
pub(crate) fn find_audio_component_mut(
    doc: &mut Document,
    target: LayerId,
    index: usize,
) -> Result<&mut AudioComponent, CommandError> {
    let layer = target.get();
    let item = find_track_item_mut(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    let TrackItem::Clip(clip) = item else {
        return Err(CommandError::AudioComponentNotFound { layer, index });
    };
    let ClipSource::Asset { audio, .. } = &mut clip.source else {
        return Err(CommandError::AudioComponentNotFound { layer, index });
    };
    audio
        .get_mut(index)
        .ok_or(CommandError::AudioComponentNotFound { layer, index })
}

fn find_track_item_mut(doc: &mut Document, target: LayerId) -> Option<&mut TrackItem> {
    fn find_in_items(items: &mut [TrackItem], target: LayerId) -> Option<&mut TrackItem> {
        for item in items {
            if envelope_of(item).layer_id == target {
                return Some(item);
            }
            if let TrackItem::Group(group) = item {
                if let Some(found) = find_in_items(&mut group.children, target) {
                    return Some(found);
                }
            }
        }
        None
    }

    doc.tracks
        .iter_mut()
        .find_map(|track| find_in_items(&mut track.items, target))
}

fn find_group_children_mut(
    items: &mut [TrackItem],
    target: LayerId,
) -> Option<&mut Vec<TrackItem>> {
    for item in items.iter_mut() {
        if let TrackItem::Group(g) = item {
            if g.envelope.layer_id == target {
                return Some(&mut g.children);
            }
            if let Some(found) = find_group_children_mut(&mut g.children, target) {
                return Some(found);
            }
        }
    }
    None
}

pub(crate) fn find_items_vec_mut(
    doc: &mut Document,
    parent: ParentLocator,
) -> Result<&mut Vec<TrackItem>, CommandError> {
    match parent {
        ParentLocator::Track(tid) => doc
            .tracks
            .iter_mut()
            .find(|t| t.id == tid)
            .map(|t| &mut t.items)
            .ok_or(CommandError::TrackNotFound(tid.get())),
        ParentLocator::Group(layer) => {
            for track in &mut doc.tracks {
                if let Some(found) = find_group_children_mut(&mut track.items, layer) {
                    return Ok(found);
                }
            }
            Err(CommandError::GroupNotFound(layer.get()))
        }
    }
}

fn find_group_children(items: &[TrackItem], target: LayerId) -> Option<&[TrackItem]> {
    for item in items {
        if let TrackItem::Group(g) = item {
            if g.envelope.layer_id == target {
                return Some(g.children.as_slice());
            }
            if let Some(found) = find_group_children(&g.children, target) {
                return Some(found);
            }
        }
    }
    None
}

/// 事前検査用の読み取り専用ロケータ。
pub(crate) fn find_items_vec(
    doc: &Document,
    parent: ParentLocator,
) -> Result<&[TrackItem], CommandError> {
    match parent {
        ParentLocator::Track(tid) => doc
            .tracks
            .iter()
            .find(|t| t.id == tid)
            .map(|t| t.items.as_slice())
            .ok_or(CommandError::TrackNotFound(tid.get())),
        ParentLocator::Group(layer) => {
            for track in &doc.tracks {
                if let Some(found) = find_group_children(&track.items, layer) {
                    return Ok(found);
                }
            }
            Err(CommandError::GroupNotFound(layer.get()))
        }
    }
}

/// 読み取り専用ロケータ(コマンド構築側が現在値を読むためのヘルパ)。
pub fn find_envelope(doc: &Document, target: LayerId) -> Option<&ItemEnvelope> {
    fn find_in_items(items: &[TrackItem], target: LayerId) -> Option<&ItemEnvelope> {
        for item in items {
            if envelope_of(item).layer_id == target {
                return Some(envelope_of(item));
            }
            if let TrackItem::Group(g) = item {
                if let Some(found) = find_in_items(&g.children, target) {
                    return Some(found);
                }
            }
        }
        None
    }
    doc.tracks
        .iter()
        .find_map(|t| find_in_items(&t.items, target))
}

/// 読み取り専用: `target`にある`TrackItem`とその親ロケータ・indexを返す(削除/複製の下準備用)。
pub fn find_item_location(
    doc: &Document,
    target: LayerId,
) -> Option<(ParentLocator, usize, &TrackItem)> {
    for track in &doc.tracks {
        if let Some((idx, item)) = track
            .items
            .iter()
            .enumerate()
            .find(|(_, it)| envelope_of(it).layer_id == target)
        {
            return Some((ParentLocator::Track(track.id), idx, item));
        }
        if let Some(found) = find_in_groups(&track.items, target) {
            return Some(found);
        }
    }
    None
}

fn find_in_groups(
    items: &[TrackItem],
    target: LayerId,
) -> Option<(ParentLocator, usize, &TrackItem)> {
    for item in items {
        if let TrackItem::Group(g) = item {
            if let Some((idx, child)) = g
                .children
                .iter()
                .enumerate()
                .find(|(_, it)| envelope_of(it).layer_id == target)
            {
                return Some((ParentLocator::Group(g.envelope.layer_id), idx, child));
            }
            if let Some(found) = find_in_groups(&g.children, target) {
                return Some(found);
            }
        }
    }
    None
}

struct ReservationCommit {
    advance_to: Option<u64>,
}

pub(crate) fn guard_effect_lifecycle_document(doc: &Document) -> Result<(), CommandError> {
    let required = validate::MIN_READER_VERSION_FOR_COMP_CAMERA;
    if doc.version != WRITER_VERSION || doc.min_reader_version != required {
        return Err(CommandError::EffectLifecycleRequiresV4Document {
            version: doc.version,
            min_reader_version: doc.min_reader_version,
        });
    }
    doc.validate().map_err(CommandError::Validate)?;
    Ok(())
}

fn swap_if_valid(doc: &mut Document, next: Document) -> Result<(), CommandError> {
    next.validate().map_err(CommandError::Validate)?;
    *doc = next;
    Ok(())
}

fn validate_reservation_shape(
    reservation: StableIdReservation,
    introduced: &[u64],
) -> Result<(), CommandError> {
    let before = reservation.before();
    let after = reservation.after();
    if before >= after {
        return Err(CommandError::InvalidStableIdReservationInterval { before, after });
    }
    let span = after
        .checked_sub(before)
        .ok_or(CommandError::InvalidStableIdReservationInterval { before, after })?;
    let introduced_len =
        u64::try_from(introduced.len()).map_err(|_| CommandError::StableIdReservationMismatch {
            before,
            after,
            introduced: introduced.to_vec(),
        })?;
    if introduced_len != span {
        return Err(CommandError::StableIdReservationMismatch {
            before,
            after,
            introduced: introduced.to_vec(),
        });
    }
    for (offset, &id) in introduced.iter().enumerate() {
        let offset =
            u64::try_from(offset).map_err(|_| CommandError::StableIdReservationMismatch {
                before,
                after,
                introduced: introduced.to_vec(),
            })?;
        let expected =
            before
                .checked_add(offset)
                .ok_or(CommandError::StableIdReservationMismatch {
                    before,
                    after,
                    introduced: introduced.to_vec(),
                })?;
        if id != expected {
            return Err(CommandError::StableIdReservationMismatch {
                before,
                after,
                introduced: introduced.to_vec(),
            });
        }
    }
    Ok(())
}

fn validate_reservation_for_apply(
    doc: &Document,
    reservation: StableIdReservation,
    introduced: &[u64],
) -> Result<ReservationCommit, CommandError> {
    validate_reservation_shape(reservation, introduced)?;
    let before = reservation.before();
    let after = reservation.after();
    for &id in introduced {
        if stable_id_in_use(doc, id) {
            return Err(CommandError::StableIdCollision { id });
        }
    }
    let next = doc.next_stable_id.peek_next();
    let advance_to = if next == before {
        Some(after)
    } else if next >= after {
        None
    } else {
        return Err(CommandError::StableIdReservationCounterMismatch {
            next,
            before,
            after,
        });
    };
    Ok(ReservationCommit { advance_to })
}

fn validate_reservation_for_undo(
    doc: &Document,
    reservation: StableIdReservation,
    introduced: &[u64],
) -> Result<(), CommandError> {
    validate_reservation_shape(reservation, introduced)?;
    let before = reservation.before();
    let after = reservation.after();
    let next = doc.next_stable_id.peek_next();
    if next < after {
        return Err(CommandError::StableIdReservationCounterMismatch {
            next,
            before,
            after,
        });
    }
    Ok(())
}

pub(crate) fn validate_reservation_closure(
    reservation: StableIdReservation,
    introduced: &[u64],
) -> Result<(), CommandError> {
    validate_reservation_shape(reservation, introduced)
}

pub(crate) fn introduced_ids_create(use_: &EffectUse, definition: &EffectDefinition) -> Vec<u64> {
    let mut ids = vec![use_.id.get(), definition.id.get()];
    ids.extend(
        remint_order_keyframe_ids(definition)
            .into_iter()
            .map(|id| id.get()),
    );
    ids
}

pub(crate) fn introduced_ids_link(use_: &EffectUse) -> Vec<u64> {
    vec![use_.id.get()]
}

pub(crate) fn introduced_ids_copy_local(new_definition: &EffectDefinition) -> Vec<u64> {
    let mut ids = vec![new_definition.id.get()];
    ids.extend(
        remint_order_keyframe_ids(new_definition)
            .into_iter()
            .map(|id| id.get()),
    );
    ids
}

fn apply_reservation_commit(doc: &mut Document, commit: ReservationCommit) {
    if let Some(after) = commit.advance_to {
        doc.next_stable_id.commit_validated_reservation(after);
    }
}

pub(crate) fn find_use_location(doc: &Document, use_id: EffectId) -> Option<(LayerId, usize)> {
    fn walk(items: &[TrackItem], use_id: EffectId) -> Option<(LayerId, usize)> {
        for item in items {
            let env = envelope_of(item);
            if let Some(index) = env.effects.iter().position(|u| u.id == use_id) {
                return Some((env.layer_id, index));
            }
            if let TrackItem::Group(g) = item {
                if let Some(found) = walk(&g.children, use_id) {
                    return Some(found);
                }
            }
        }
        None
    }
    doc.tracks
        .iter()
        .find_map(|track| walk(&track.items, use_id))
}

fn apply_create_effect(
    doc: &mut Document,
    target: LayerId,
    index: usize,
    use_: EffectUse,
    definition: EffectDefinition,
    reservation: StableIdReservation,
) -> Result<(), CommandError> {
    guard_effect_lifecycle_document(doc)?;
    if use_.definition_id != definition.id {
        return Err(CommandError::EffectDefinitionMismatch {
            id: definition.id.get(),
        });
    }
    let introduced = introduced_ids_create(&use_, &definition);
    let commit = validate_reservation_for_apply(doc, reservation, &introduced)?;
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(target.get()))?;
    if index > env.effects.len() {
        return Err(CommandError::IndexOutOfRange {
            index,
            len: env.effects.len(),
        });
    }
    if doc.effect_definition(definition.id).is_some() {
        return Err(CommandError::EffectDefinitionAlreadyExists {
            id: definition.id.get(),
        });
    }

    let mut next = doc.clone();
    {
        let env = find_envelope_mut(&mut next, target)?;
        env.effects.insert(index, use_);
    }
    next.effect_definitions.push(definition);
    apply_reservation_commit(&mut next, commit);
    swap_if_valid(doc, next)
}

fn apply_undo_create_effect(
    doc: &mut Document,
    target: LayerId,
    index: usize,
    use_: EffectUse,
    definition: EffectDefinition,
    reservation: StableIdReservation,
) -> Result<(), CommandError> {
    guard_effect_lifecycle_document(doc)?;
    let introduced = introduced_ids_create(&use_, &definition);
    validate_reservation_for_undo(doc, reservation, &introduced)?;
    let layer = target.get();
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    if index >= env.effects.len() {
        return Err(CommandError::IndexOutOfRange {
            index,
            len: env.effects.len(),
        });
    }
    let at_index = &env.effects[index];
    if at_index.id != use_.id || at_index.definition_id != use_.definition_id {
        return Err(CommandError::RemoveEffectMismatch {
            expected: use_.id.get(),
            found: at_index.id.get(),
        });
    }
    let ledger_def =
        doc.effect_definition(definition.id)
            .ok_or(CommandError::EffectDefinitionNotFound {
                id: definition.id.get(),
            })?;
    if ledger_def != &definition {
        return Err(CommandError::RemoveEffectDefinitionMismatch {
            id: definition.id.get(),
        });
    }
    let remaining = doc.effect_use_count(definition.id);
    if remaining != 1 {
        return Err(CommandError::DefinitionInUse {
            id: definition.id.get(),
            use_ids: doc
                .effect_use_ids(definition.id)
                .into_iter()
                .map(|id| id.get())
                .collect(),
        });
    }

    let mut next = doc.clone();
    find_envelope_mut(&mut next, target)?.effects.remove(index);
    next.effect_definitions.retain(|d| d.id != definition.id);
    swap_if_valid(doc, next)
}

fn apply_link_effect_use(
    doc: &mut Document,
    target: LayerId,
    index: usize,
    use_: EffectUse,
    reservation: StableIdReservation,
) -> Result<(), CommandError> {
    guard_effect_lifecycle_document(doc)?;
    let introduced = introduced_ids_link(&use_);
    let commit = validate_reservation_for_apply(doc, reservation, &introduced)?;
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(target.get()))?;
    if index > env.effects.len() {
        return Err(CommandError::IndexOutOfRange {
            index,
            len: env.effects.len(),
        });
    }
    let existing = doc.effect_definition(use_.definition_id).ok_or(
        CommandError::EffectDefinitionNotFound {
            id: use_.definition_id.get(),
        },
    )?;

    let mut next = doc.clone();
    let _ = existing;
    find_envelope_mut(&mut next, target)?
        .effects
        .insert(index, use_);
    apply_reservation_commit(&mut next, commit);
    swap_if_valid(doc, next)
}

fn apply_undo_link_effect_use(
    doc: &mut Document,
    target: LayerId,
    index: usize,
    use_: EffectUse,
    reservation: StableIdReservation,
) -> Result<(), CommandError> {
    guard_effect_lifecycle_document(doc)?;
    let introduced = introduced_ids_link(&use_);
    validate_reservation_for_undo(doc, reservation, &introduced)?;
    let layer = target.get();
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    if index >= env.effects.len() {
        return Err(CommandError::IndexOutOfRange {
            index,
            len: env.effects.len(),
        });
    }
    let at_index = &env.effects[index];
    if at_index.id != use_.id || at_index.definition_id != use_.definition_id {
        return Err(CommandError::RemoveEffectMismatch {
            expected: use_.id.get(),
            found: at_index.id.get(),
        });
    }
    if doc.effect_definition(use_.definition_id).is_none() {
        return Err(CommandError::EffectDefinitionNotFound {
            id: use_.definition_id.get(),
        });
    }

    let mut next = doc.clone();
    find_envelope_mut(&mut next, target)?.effects.remove(index);
    swap_if_valid(doc, next)
}

fn apply_unlink_effect_use(
    doc: &mut Document,
    target: LayerId,
    index: usize,
    use_: EffectUse,
) -> Result<(), CommandError> {
    guard_effect_lifecycle_document(doc)?;
    let layer = target.get();
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    if index >= env.effects.len() {
        return Err(CommandError::IndexOutOfRange {
            index,
            len: env.effects.len(),
        });
    }
    let at_index = &env.effects[index];
    if at_index.id != use_.id || at_index.definition_id != use_.definition_id {
        return Err(CommandError::RemoveEffectMismatch {
            expected: use_.id.get(),
            found: at_index.id.get(),
        });
    }
    if doc.effect_definition(use_.definition_id).is_none() {
        return Err(CommandError::EffectDefinitionNotFound {
            id: use_.definition_id.get(),
        });
    }

    let mut next = doc.clone();
    find_envelope_mut(&mut next, target)?.effects.remove(index);
    swap_if_valid(doc, next)
}

fn apply_restore_effect_use(
    doc: &mut Document,
    target: LayerId,
    index: usize,
    use_: EffectUse,
) -> Result<(), CommandError> {
    guard_effect_lifecycle_document(doc)?;
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(target.get()))?;
    if index > env.effects.len() {
        return Err(CommandError::IndexOutOfRange {
            index,
            len: env.effects.len(),
        });
    }
    if stable_id_in_use(doc, use_.id.get()) {
        return Err(CommandError::StableIdCollision { id: use_.id.get() });
    }
    if doc.effect_definition(use_.definition_id).is_none() {
        return Err(CommandError::EffectDefinitionNotFound {
            id: use_.definition_id.get(),
        });
    }

    let mut next = doc.clone();
    find_envelope_mut(&mut next, target)?
        .effects
        .insert(index, use_);
    swap_if_valid(doc, next)
}

fn apply_copy_local_effect(
    doc: &mut Document,
    use_id: EffectId,
    previous_definition_id: EffectDefinitionId,
    new_definition: EffectDefinition,
    reservation: StableIdReservation,
) -> Result<(), CommandError> {
    guard_effect_lifecycle_document(doc)?;
    let introduced = introduced_ids_copy_local(&new_definition);
    let commit = validate_reservation_for_apply(doc, reservation, &introduced)?;
    let (target, index) =
        find_use_location(doc, use_id).ok_or(CommandError::EffectUseNotFound {
            use_id: use_id.get(),
        })?;
    {
        let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(target.get()))?;
        let use_ = &env.effects[index];
        if use_.id != use_id {
            return Err(CommandError::EffectUseNotFound {
                use_id: use_id.get(),
            });
        }
        if use_.definition_id != previous_definition_id {
            return Err(CommandError::CopyLocalDefinitionMismatch {
                expected: previous_definition_id.get(),
                found: use_.definition_id.get(),
            });
        }
    }
    let source = doc.effect_definition(previous_definition_id).ok_or(
        CommandError::EffectDefinitionNotFound {
            id: previous_definition_id.get(),
        },
    )?;
    if !definition_semantic_body_eq(source, &new_definition) {
        return Err(CommandError::CopyLocalPayloadMismatch);
    }
    if doc.effect_definition(new_definition.id).is_some() {
        return Err(CommandError::EffectDefinitionAlreadyExists {
            id: new_definition.id.get(),
        });
    }

    let mut next = doc.clone();
    {
        let env = find_envelope_mut(&mut next, target)?;
        env.effects[index].definition_id = new_definition.id;
    }
    next.effect_definitions.push(new_definition);
    apply_reservation_commit(&mut next, commit);
    swap_if_valid(doc, next)
}

fn apply_undo_copy_local_effect(
    doc: &mut Document,
    use_id: EffectId,
    previous_definition_id: EffectDefinitionId,
    new_definition: EffectDefinition,
    reservation: StableIdReservation,
) -> Result<(), CommandError> {
    guard_effect_lifecycle_document(doc)?;
    let introduced = introduced_ids_copy_local(&new_definition);
    validate_reservation_for_undo(doc, reservation, &introduced)?;
    let (target, index) =
        find_use_location(doc, use_id).ok_or(CommandError::EffectUseNotFound {
            use_id: use_id.get(),
        })?;
    {
        let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(target.get()))?;
        let use_ = &env.effects[index];
        if use_.definition_id != new_definition.id {
            return Err(CommandError::CopyLocalDefinitionMismatch {
                expected: new_definition.id.get(),
                found: use_.definition_id.get(),
            });
        }
    }
    if doc.effect_definition(previous_definition_id).is_none() {
        return Err(CommandError::EffectDefinitionNotFound {
            id: previous_definition_id.get(),
        });
    }
    let ledger =
        doc.effect_definition(new_definition.id)
            .ok_or(CommandError::EffectDefinitionNotFound {
                id: new_definition.id.get(),
            })?;
    if ledger != &new_definition {
        return Err(CommandError::EffectDefinitionMismatch {
            id: new_definition.id.get(),
        });
    }
    let shared_use_ids: Vec<u64> = doc
        .effect_use_ids(new_definition.id)
        .into_iter()
        .map(|id| id.get())
        .filter(|id| *id != use_id.get())
        .collect();
    if !shared_use_ids.is_empty() {
        return Err(CommandError::UndoCopyLocalDefinitionInUse {
            id: new_definition.id.get(),
            use_ids: shared_use_ids,
        });
    }

    let mut next = doc.clone();
    {
        let env = find_envelope_mut(&mut next, target)?;
        env.effects[index].definition_id = previous_definition_id;
    }
    next.effect_definitions
        .retain(|d| d.id != new_definition.id);
    swap_if_valid(doc, next)
}
