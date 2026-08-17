use crate::stable_id::StableIdReservation;

use super::locate::envelope_of;
use super::{Command, CommandKind, GestureId, MergeKey, PropertyId};

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
            Command::AdmitAsset { .. } | Command::RemoveAsset { .. } => CommandKind::AssetLifecycle,
            Command::SetAudioComponentEnabled { .. } => CommandKind::SetAudioComponentEnabled,
            Command::SetAudioComponentGain { .. } => CommandKind::SetAudioComponentGain,
            Command::AddTrackItem { .. } => CommandKind::AddTrackItem,
            Command::RemoveTrackItem { .. } => CommandKind::RemoveTrackItem,
            Command::AddPositionKey { .. } | Command::UndoAddPositionKey { .. } => {
                CommandKind::AddPositionKey
            }
            Command::SetPositionKeyInterp { .. } => CommandKind::SetPositionKeyInterp,
            Command::SetPositionKeyValue { .. } => CommandKind::SetPositionKeyValue,
            Command::SetPositionKeyTime { .. } => CommandKind::SetPositionKeyTime,
            // transform param key族(add/remove/value)は`SetProperty` kindで揃っている。
            // 時刻移動だけ別kindにするとmerge key族が割れるので同じkindへ載せる。
            Command::SetTransformParamKeyTime { .. } => CommandKind::SetProperty,
            Command::RemovePositionKey { .. } | Command::UndoRemovePositionKey { .. } => {
                CommandKind::RemovePositionKey
            }
            Command::SetClipStart { .. } => CommandKind::SetClipStart,
            Command::TrimClipIn { .. } => CommandKind::TrimClipIn,
            Command::TrimClipOut { .. } => CommandKind::TrimClipOut,
            Command::SplitClip { .. } | Command::UnsplitClip { .. } => CommandKind::SplitClip,
            Command::ReparentClip { .. } => CommandKind::ReparentClip,
            Command::SetItemVisible { .. } => CommandKind::SetItemVisible,
            Command::SetItemSolo { .. } => CommandKind::SetItemSolo,
            Command::SetItemLock { .. } => CommandKind::SetItemLock,
            Command::SetItemColor { .. } => CommandKind::SetItemColor,
            Command::SetLayerName { .. } => CommandKind::SetLayerName,
            Command::AddLocator { .. } => CommandKind::AddLocator,
            Command::RemoveLocator { .. } => CommandKind::RemoveLocator,
            Command::SetLocatorTime { .. } => CommandKind::SetLocatorTime,
            Command::SetLocatorText { .. } => CommandKind::SetLocatorText,
            Command::SetSoundtrack { .. } => CommandKind::SetSoundtrack,
            Command::SetCompositionResolution { .. } => CommandKind::SetCompositionResolution,
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
            | Command::SetAudioComponentGain { target, .. }
            | Command::SetPositionKeyInterp { target, .. }
            | Command::SetPositionKeyValue { target, .. }
            | Command::SetPositionKeyTime { target, .. }
            | Command::SetTransformParamKeyTime { target, .. }
            | Command::SetClipStart { target, .. }
            | Command::TrimClipIn { target, .. }
            | Command::TrimClipOut { target, .. }
            | Command::SplitClip { target, .. }
            | Command::UnsplitClip { target, .. }
            | Command::ReparentClip { target, .. }
            | Command::SetItemVisible { target, .. }
            | Command::SetItemLock { target, .. }
            | Command::SetItemColor { target, .. }
            | Command::SetLayerName { target, .. }
            | Command::SetItemSolo { target, .. } => target.get(),
            Command::AddPositionKey { added_key_id, .. }
            | Command::UndoAddPositionKey { added_key_id, .. } => added_key_id.get(),
            Command::RemovePositionKey { removed_key_id, .. }
            | Command::UndoRemovePositionKey { removed_key_id, .. } => removed_key_id.get(),
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
            Command::AdmitAsset { asset } | Command::RemoveAsset { asset } => asset.id.get(),
            Command::AddTrackItem { item, .. } | Command::RemoveTrackItem { item, .. } => {
                envelope_of(item).layer_id.get()
            }
            // **メモは layer に属さない。** index をそのまま宛先にする
            Command::AddLocator { index, .. }
            | Command::RemoveLocator { index, .. }
            | Command::SetLocatorTime { index, .. }
            | Command::SetLocatorText { index, .. } => *index as u64,
            // **Documentに1本のsingleton。** 対象stable idを持たないので0固定
            // (kind+propertyだけで merge 対象が決まる)。
            Command::SetSoundtrack { .. } | Command::SetCompositionResolution { .. } => 0,
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
            Command::AdmitAsset { asset } | Command::RemoveAsset { asset } => {
                PropertyId::AssetLifecycle(asset.id)
            }
            Command::AddPositionKey { .. } | Command::UndoAddPositionKey { .. } => {
                PropertyId::Position
            }
            Command::RemovePositionKey { .. } | Command::UndoRemovePositionKey { .. } => {
                PropertyId::Position
            }
            Command::SetPositionKeyInterp { key, .. } => PropertyId::PositionKeyInterp(*key),
            Command::SetPositionKeyValue { key, .. } => PropertyId::PositionKeyValue(*key),
            Command::SetPositionKeyTime { key, .. } => PropertyId::PositionKeyTime(*key),
            Command::SetTransformParamKeyTime { key, .. } => {
                PropertyId::TransformParamKeyTime(*key)
            }
            Command::SetAudioComponentEnabled { index, .. } => PropertyId::AudioEnabled(*index),
            Command::SetAudioComponentGain { index, .. } => PropertyId::AudioGain(*index),
            Command::AddTrackItem { .. } | Command::RemoveTrackItem { .. } => PropertyId::ChildList,
            Command::SetClipStart { .. } => PropertyId::ClipStart,
            Command::TrimClipIn { .. } => PropertyId::ClipIn,
            Command::TrimClipOut { .. } => PropertyId::ClipOut,
            Command::SplitClip { .. } | Command::UnsplitClip { .. } => PropertyId::Split,
            Command::ReparentClip { .. } => PropertyId::Reparent,
            Command::SetItemVisible { .. } => PropertyId::ItemVisible,
            Command::SetItemSolo { .. } => PropertyId::ItemSolo,
            Command::SetItemLock { .. } => PropertyId::ItemLock,
            Command::SetItemColor { .. } => PropertyId::ItemColor,
            Command::SetLayerName { .. } => PropertyId::LayerName,
            Command::AddLocator { .. }
            | Command::RemoveLocator { .. }
            | Command::SetLocatorTime { .. }
            | Command::SetLocatorText { .. } => PropertyId::Locator,
            Command::SetSoundtrack { .. } => PropertyId::Soundtrack,
            Command::SetCompositionResolution { .. } => PropertyId::CompositionResolution,
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
            Command::AddPositionKey {
                stable_id_reservation,
                ..
            }
            | Command::UndoAddPositionKey {
                stable_id_reservation,
                ..
            }
            | Command::RemovePositionKey {
                stable_id_reservation,
                ..
            }
            | Command::UndoRemovePositionKey {
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
            | Command::AdmitAsset { .. }
            | Command::RemoveAsset { .. }
            | Command::SetAudioComponentEnabled { .. }
            | Command::SetAudioComponentGain { .. }
            | Command::AddTrackItem { .. }
            | Command::RemoveTrackItem { .. }
            | Command::SetPositionKeyInterp { .. }
            | Command::SetPositionKeyValue { .. }
            | Command::SetPositionKeyTime { .. }
            | Command::SetTransformParamKeyTime { .. }
            | Command::SetClipStart { .. }
            | Command::TrimClipIn { .. }
            | Command::TrimClipOut { .. }
            | Command::SplitClip { .. }
            | Command::UnsplitClip { .. }
            | Command::ReparentClip { .. }
            | Command::SetItemVisible { .. }
            | Command::SetItemLock { .. }
            | Command::SetItemColor { .. }
            | Command::SetLayerName { .. }
            | Command::AddLocator { .. }
            | Command::RemoveLocator { .. }
            | Command::SetLocatorTime { .. }
            | Command::SetLocatorText { .. }
            | Command::SetSoundtrack { .. }
            | Command::SetCompositionResolution { .. }
            | Command::SetItemSolo { .. } => None,
        }
    }
}
