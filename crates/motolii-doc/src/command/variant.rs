use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use motolii_core::{RationalTime, TimeMap};

use crate::asset::Asset;
use crate::param::DocParam;
use crate::schema::{
    BlendMode, ClippingMaskSettings, EffectDefinition, EffectInstance, EffectUse, TrackItem,
};
use crate::stable_id::{EffectDefinitionId, EffectId, KeyframeId, StableIdReservation};
use crate::{Document, LayerId};

use super::{CommandError, ParentLocator, ScalarPropertyId};

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
    /// 新規admitと`RemoveAsset`のinverse/Redo/replayで同じ完全payloadを使う。
    AdmitAsset { asset: Asset },
    /// 参照0かつlive payload一致時だけ台帳から除去する。
    RemoveAsset { asset: Asset },
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
    AddPositionKey {
        target: LayerId,
        old_value: DocParam,
        new_value: DocParam,
        added_key_id: KeyframeId,
        stable_id_reservation: StableIdReservation,
    },
    UndoAddPositionKey {
        target: LayerId,
        old_value: DocParam,
        new_value: DocParam,
        added_key_id: KeyframeId,
        stable_id_reservation: StableIdReservation,
    },
    SetPositionKeyInterp {
        target: LayerId,
        key: KeyframeId,
        old: motolii_eval::Interp,
        new: motolii_eval::Interp,
    },
    SetPositionKeyValue {
        target: LayerId,
        key: KeyframeId,
        old: [f64; 2],
        new: [f64; 2],
    },
    SetPositionKeyTime {
        target: LayerId,
        key: KeyframeId,
        old: RationalTime,
        new: RationalTime,
    },
    /// Scale / Rotation / Opacity の既存keyの時刻だけを移す。`SetPositionKeyTime`の鏡映で、
    /// `property`セレクタの範囲は`prepare_add_transform_param_key`族と同じ。
    /// Positionは`SetPositionKeyTime`が担当し続ける(こちらでは受け付けない)。
    SetTransformParamKeyTime {
        target: LayerId,
        property: ScalarPropertyId,
        key: KeyframeId,
        old: RationalTime,
        new: RationalTime,
    },
    RemovePositionKey {
        target: LayerId,
        old_value: DocParam,
        new_value: DocParam,
        removed_key_id: KeyframeId,
        stable_id_reservation: StableIdReservation,
    },
    UndoRemovePositionKey {
        target: LayerId,
        old_value: DocParam,
        new_value: DocParam,
        removed_key_id: KeyframeId,
        stable_id_reservation: StableIdReservation,
    },
    SetClipStart {
        target: LayerId,
        old: RationalTime,
        new: RationalTime,
    },
    TrimClipIn {
        target: LayerId,
        old_start: RationalTime,
        old_duration: RationalTime,
        old_time_map: TimeMap,
        new_start: RationalTime,
        new_duration: RationalTime,
        new_time_map: TimeMap,
    },
    TrimClipOut {
        target: LayerId,
        old_duration: RationalTime,
        new_duration: RationalTime,
    },
    /// 1 Clip の in/out 窓を playhead で二つに割る。右片は新 LayerId。ソースは共有。
    /// host kind は既存の `split`。skia はまだ撃たない。
    SplitClip {
        target: LayerId,
        old_duration: RationalTime,
        new_duration: RationalTime,
        right_parent: ParentLocator,
        right_index: usize,
        right_item: TrackItem,
        right_layer_names: BTreeMap<LayerId, String>,
    },
    UnsplitClip {
        target: LayerId,
        old_duration: RationalTime,
        new_duration: RationalTime,
        right_parent: ParentLocator,
        right_index: usize,
        right_item: TrackItem,
        right_layer_names: BTreeMap<LayerId, String>,
    },
    /// TrackItem の親/index を変える。host kind `reparent_clip` の viseme。
    /// `new_index` は old から外したあとの挿入位置。`SetClipStart` は同レーン用のまま触らない。
    ReparentClip {
        target: LayerId,
        old_parent: ParentLocator,
        old_index: usize,
        new_parent: ParentLocator,
        new_index: usize,
        old_start: Option<RationalTime>,
        new_start: Option<RationalTime>,
    },
    SetItemVisible {
        target: LayerId,
        old: bool,
        new: bool,
    },
    SetItemSolo {
        target: LayerId,
        old: bool,
        new: bool,
    },
}

/// fresh admitだけが使う非永続prepared値。Undo/Redo/replayの復元とは分離する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedAssetAdmission {
    pub(super) expected_next: u64,
    pub(super) asset: Asset,
}

impl PreparedAssetAdmission {
    pub fn asset(&self) -> &Asset {
        &self.asset
    }

    pub fn into_command_if_current(self, doc: &Document) -> Result<Command, CommandError> {
        let actual_next = doc.assets.peek_next();
        if actual_next != self.expected_next {
            return Err(CommandError::StaleAssetAdmission {
                expected_next: self.expected_next,
                actual_next,
            });
        }
        Ok(Command::AdmitAsset { asset: self.asset })
    }
}
