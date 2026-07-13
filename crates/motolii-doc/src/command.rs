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

use crate::param::DocParam;
use crate::schema::{BlendMode, ClippingMaskSettings, EffectInstance, ItemEnvelope, TrackItem};
use crate::stable_id::EffectId;
use crate::track_id::TrackId;
use crate::{Document, LayerId};

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
    SetEffectEnabled,
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
    #[error("track item index {index} out of range (len={len})")]
    IndexOutOfRange { index: usize, len: usize },
    #[error(
        "removed track item does not match expected layer id (expected {expected}, found {found})"
    )]
    RemoveItemMismatch { expected: u64, found: u64 },
    #[error("removed effect does not match expected id (expected {expected}, found {found})")]
    RemoveEffectMismatch { expected: u64, found: u64 },
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
    },
    RemoveEffect {
        target: LayerId,
        index: usize,
        effect: EffectInstance,
    },
    SetEffectEnabled {
        target: LayerId,
        effect: EffectId,
        old: bool,
        new: bool,
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
            Command::SetEffectEnabled { .. } => CommandKind::SetEffectEnabled,
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
            | Command::SetEffectEnabled { target, .. } => target.get(),
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
            Command::SetEffectEnabled { effect, .. } => PropertyId::EffectEnabled(*effect),
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

    /// `new`側を`Document`へ書き込む。
    pub fn apply(&self, doc: &mut Document) -> Result<(), CommandError> {
        match self {
            Command::SetProperty {
                target,
                property,
                new_value,
                ..
            } => {
                let env = find_envelope_mut(doc, *target)?;
                write_property(env, property, new_value.clone())
            }
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
            } => {
                // 事前検査: 対象envelopeが存在する。index > lenは拒否(==lenは末尾追加)。
                let env =
                    find_envelope(doc, *target).ok_or(CommandError::LayerNotFound(target.get()))?;
                if *index > env.effects.len() {
                    return Err(CommandError::IndexOutOfRange {
                        index: *index,
                        len: env.effects.len(),
                    });
                }
                find_envelope_mut(doc, *target)?
                    .effects
                    .insert(*index, effect.clone());
                Ok(())
            }
            Command::RemoveEffect {
                target,
                index,
                effect,
            } => {
                let env = find_envelope_mut(doc, *target)?;
                if *index >= env.effects.len() {
                    return Err(CommandError::IndexOutOfRange {
                        index: *index,
                        len: env.effects.len(),
                    });
                }
                let found = env.effects[*index].id;
                if found != effect.id {
                    return Err(CommandError::RemoveEffectMismatch {
                        expected: effect.id.get(),
                        found: found.get(),
                    });
                }
                env.effects.remove(*index);
                Ok(())
            }
            Command::SetEffectEnabled {
                target,
                effect,
                new,
                ..
            } => {
                let layer = target.get();
                let env = find_envelope_mut(doc, *target)?;
                let e = env.effects.iter_mut().find(|e| e.id == *effect).ok_or(
                    CommandError::EffectNotFound {
                        effect: effect.get(),
                        layer,
                    },
                )?;
                e.enabled = *new;
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
            } => Command::RemoveEffect {
                target,
                index,
                effect,
            },
            Command::RemoveEffect {
                target,
                index,
                effect,
            } => Command::AddEffect {
                target,
                index,
                effect,
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
        ScalarPropertyId::EffectParam(effect_id, name) => {
            let layer = env.layer_id.get();
            let e = env.effects.iter_mut().find(|e| e.id == *effect_id).ok_or(
                CommandError::EffectNotFound {
                    effect: effect_id.get(),
                    layer,
                },
            )?;
            e.params.insert(name.clone(), value);
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
fn find_items_vec(doc: &Document, parent: ParentLocator) -> Result<&[TrackItem], CommandError> {
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
