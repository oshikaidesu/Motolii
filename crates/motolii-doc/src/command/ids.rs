use serde::{Deserialize, Serialize};

use crate::asset::AssetId;
use crate::stable_id::{EffectDefinitionId, EffectId, KeyframeId};
use crate::track_id::TrackId;
use crate::LayerId;

/// `SetProperty`が書き込める閉じたプロパティ集合(envelope本体+effect params)。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScalarPropertyId {
    Position,
    Anchor,
    Scale,
    Rotation,
    Opacity,
    EffectParam(EffectId, String),
    /// `ClipSource::Plugin.params` の既存キー。値型は `DocParam` が正本。
    SourceParam(String),
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
    SourceParam(String),
    EffectList(EffectId),
    /// D1l: `DeleteEffectDefinition`/`AddEffectDefinition`(台帳の生存)。
    EffectDefinitionLifecycle(EffectDefinitionId),
    /// D1l: `CopyLocalEffect`/`UndoCopyLocalEffect`(1つのUseのdefinition_id付け替え)。
    EffectDefinitionLink(EffectId),
    AssetLifecycle(AssetId),
    AudioEnabled(usize),
    AudioGain(usize),
    ChildList,
    PositionKeyInterp(KeyframeId),
    PositionKeyValue(KeyframeId),
    PositionKeyTime(KeyframeId),
    ClipStart,
    ClipIn,
    ClipOut,
    Split,
    Reparent,
    ItemVisible,
    ItemSolo,
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
            ScalarPropertyId::SourceParam(name) => PropertyId::SourceParam(name),
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
    /// M2 Asset: `AdmitAsset` / `RemoveAsset`(inverse)共用。
    AssetLifecycle,
    SetAudioComponentEnabled,
    SetAudioComponentGain,
    AddTrackItem,
    RemoveTrackItem,
    AddPositionKey,
    SetPositionKeyInterp,
    SetPositionKeyValue,
    SetPositionKeyTime,
    RemovePositionKey,
    SetClipStart,
    TrimClipIn,
    TrimClipOut,
    SplitClip,
    ReparentClip,
    SetItemVisible,
    SetItemSolo,
}

/// S18: `gesture_id + command_kind + target_stable_id + property_id`。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MergeKey {
    pub gesture: GestureId,
    pub kind: CommandKind,
    pub target_stable_id: u64,
    pub property: PropertyId,
}
