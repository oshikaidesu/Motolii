use super::Command;

impl Command {
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
            Command::AdmitAsset { mut asset } => {
                asset.normalize_self();
                Command::RemoveAsset { asset }
            }
            Command::RemoveAsset { mut asset } => {
                asset.normalize_self();
                Command::AdmitAsset { asset }
            }
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
            Command::AddPositionKey {
                target,
                old_value,
                new_value,
                added_key_id,
                stable_id_reservation,
            } => Command::UndoAddPositionKey {
                target,
                old_value,
                new_value,
                added_key_id,
                stable_id_reservation,
            },
            Command::UndoAddPositionKey {
                target,
                old_value,
                new_value,
                added_key_id,
                stable_id_reservation,
            } => Command::AddPositionKey {
                target,
                old_value,
                new_value,
                added_key_id,
                stable_id_reservation,
            },
            Command::SetPositionKeyInterp {
                target,
                key,
                old,
                new,
            } => Command::SetPositionKeyInterp {
                target,
                key,
                old: new,
                new: old,
            },
            Command::SetPositionKeyValue {
                target,
                key,
                old,
                new,
            } => Command::SetPositionKeyValue {
                target,
                key,
                old: new,
                new: old,
            },
            Command::SetPositionKeyTime {
                target,
                key,
                old,
                new,
            } => Command::SetPositionKeyTime {
                target,
                key,
                old: new,
                new: old,
            },
            Command::SetTransformParamKeyTime {
                target,
                property,
                key,
                old,
                new,
            } => Command::SetTransformParamKeyTime {
                target,
                property,
                key,
                old: new,
                new: old,
            },
            Command::RemovePositionKey {
                target,
                old_value,
                new_value,
                removed_key_id,
                stable_id_reservation,
            } => Command::UndoRemovePositionKey {
                target,
                old_value,
                new_value,
                removed_key_id,
                stable_id_reservation,
            },
            Command::UndoRemovePositionKey {
                target,
                old_value,
                new_value,
                removed_key_id,
                stable_id_reservation,
            } => Command::RemovePositionKey {
                target,
                old_value,
                new_value,
                removed_key_id,
                stable_id_reservation,
            },
            Command::SetClipStart { target, old, new } => Command::SetClipStart {
                target,
                old: new,
                new: old,
            },
            Command::TrimClipIn {
                target,
                old_start,
                old_duration,
                old_time_map,
                new_start,
                new_duration,
                new_time_map,
            } => Command::TrimClipIn {
                target,
                old_start: new_start,
                old_duration: new_duration,
                old_time_map: new_time_map,
                new_start: old_start,
                new_duration: old_duration,
                new_time_map: old_time_map,
            },
            Command::TrimClipOut {
                target,
                old_duration,
                new_duration,
            } => Command::TrimClipOut {
                target,
                old_duration: new_duration,
                new_duration: old_duration,
            },
            Command::SplitClip {
                target,
                old_duration,
                new_duration,
                right_parent,
                right_index,
                right_item,
                right_layer_names,
            } => Command::UnsplitClip {
                target,
                old_duration: new_duration,
                new_duration: old_duration,
                right_parent,
                right_index,
                right_item,
                right_layer_names,
            },
            Command::UnsplitClip {
                target,
                old_duration,
                new_duration,
                right_parent,
                right_index,
                right_item,
                right_layer_names,
            } => Command::SplitClip {
                target,
                old_duration: new_duration,
                new_duration: old_duration,
                right_parent,
                right_index,
                right_item,
                right_layer_names,
            },
            Command::ReparentClip {
                target,
                old_parent,
                old_index,
                new_parent,
                new_index,
                old_start,
                new_start,
            } => Command::ReparentClip {
                target,
                old_parent: new_parent,
                old_index: new_index,
                new_parent: old_parent,
                new_index: old_index,
                old_start: new_start,
                new_start: old_start,
            },
            Command::SetItemVisible { target, old, new } => Command::SetItemVisible {
                target,
                old: new,
                new: old,
            },
            Command::SetItemSolo { target, old, new } => Command::SetItemSolo {
                target,
                old: new,
                new: old,
            },
        }
    }
}
