use crate::schema::TrackItem;
use crate::Document;

use super::asset::{apply_admit_asset, apply_remove_asset};
use super::clip::{
    apply_reparent_track_item, validate_clip_duration, validate_clip_in_payload, validate_clip_start,
};
use super::effect::{
    apply_copy_local_effect, apply_create_effect, apply_link_effect_use, apply_restore_effect_use,
    apply_undo_copy_local_effect, apply_undo_create_effect, apply_undo_link_effect_use,
    apply_unlink_effect_use,
};
use super::effect_instance::{
    apply_add_effect, apply_add_effect_definition, apply_delete_effect_definition,
    apply_remove_effect, apply_set_effect_enabled,
};
use super::envelope::{apply_envelope_set_property, write_source_param};
use super::locate::{
    find_audio_component_mut, find_envelope, find_envelope_mut, find_track_item_mut,
};
use super::position_key::{
    apply_add_position_key, apply_remove_position_key, apply_set_position_key_interp,
    apply_set_position_key_time, apply_set_position_key_value,
    apply_set_transform_param_key_time, undo_add_position_key, undo_remove_position_key,
};
use super::split::{apply_split_clip, apply_unsplit_clip};
use super::track_item::{apply_add_track_item, apply_remove_track_item};
use super::{Command, CommandError, ScalarPropertyId};

impl Command {
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
                ScalarPropertyId::SourceParam(name) => {
                    write_source_param(doc, *target, name, new_value.clone())
                }
                _ => apply_envelope_set_property(doc, *target, property, new_value),
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
            } => apply_add_effect(doc, *target, *index, effect, *introduced_definition),
            Command::RemoveEffect {
                target,
                index,
                effect,
                introduced_definition,
            } => apply_remove_effect(doc, *target, *index, effect, *introduced_definition),
            Command::SetEffectEnabled {
                target,
                effect,
                new,
                ..
            } => apply_set_effect_enabled(doc, *target, *effect, *new),
            Command::DeleteEffectDefinition { definition } => {
                apply_delete_effect_definition(doc, definition)
            }
            Command::AddEffectDefinition { definition } => {
                apply_add_effect_definition(doc, definition)
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
            Command::AdmitAsset { asset } => apply_admit_asset(doc, asset),
            Command::RemoveAsset { asset } => apply_remove_asset(doc, asset),
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
            } => apply_add_track_item(doc, *parent, *index, item, layer_names),
            Command::RemoveTrackItem {
                parent,
                index,
                item,
                layer_names,
            } => apply_remove_track_item(doc, *parent, *index, item, layer_names),
            Command::AddPositionKey {
                target,
                old_value,
                new_value,
                added_key_id,
                stable_id_reservation,
            } => apply_add_position_key(
                doc,
                *target,
                old_value.clone(),
                new_value.clone(),
                *added_key_id,
                *stable_id_reservation,
            ),
            Command::UndoAddPositionKey {
                target,
                old_value,
                new_value,
                added_key_id,
                stable_id_reservation,
            } => undo_add_position_key(
                doc,
                *target,
                old_value.clone(),
                new_value.clone(),
                *added_key_id,
                *stable_id_reservation,
            ),
            Command::SetPositionKeyInterp {
                target,
                key,
                old,
                new,
            } => apply_set_position_key_interp(doc, *target, *key, old, new),
            Command::SetPositionKeyValue {
                target,
                key,
                old,
                new,
            } => apply_set_position_key_value(doc, *target, *key, *old, *new),
            Command::SetPositionKeyTime {
                target,
                key,
                old,
                new,
            } => apply_set_position_key_time(doc, *target, *key, *old, *new),
            Command::SetTransformParamKeyTime {
                target,
                property,
                key,
                old,
                new,
            } => apply_set_transform_param_key_time(doc, *target, property, *key, *old, *new),
            Command::RemovePositionKey {
                target,
                old_value,
                new_value,
                removed_key_id,
                stable_id_reservation,
            } => apply_remove_position_key(
                doc,
                *target,
                old_value.clone(),
                new_value.clone(),
                *removed_key_id,
                *stable_id_reservation,
            ),
            Command::UndoRemovePositionKey {
                target,
                old_value,
                new_value,
                removed_key_id,
                stable_id_reservation,
            } => undo_remove_position_key(
                doc,
                *target,
                old_value.clone(),
                new_value.clone(),
                *removed_key_id,
                *stable_id_reservation,
            ),
            Command::SetClipStart { target, new, .. } => {
                let layer = target.get();
                let duration = {
                    let item = find_track_item_mut(doc, *target)
                        .ok_or(CommandError::LayerNotFound(layer))?;
                    let TrackItem::Clip(clip) = item else {
                        return Err(CommandError::TrackItemNotClip { layer });
                    };
                    clip.duration
                };
                validate_clip_start(doc, layer, *new, duration)?;
                let item =
                    find_track_item_mut(doc, *target).ok_or(CommandError::LayerNotFound(layer))?;
                let TrackItem::Clip(clip) = item else {
                    return Err(CommandError::TrackItemNotClip { layer });
                };
                clip.start = *new;
                Ok(())
            }
            Command::TrimClipIn {
                target,
                old_start,
                old_duration,
                old_time_map,
                new_start,
                new_duration,
                new_time_map,
            } => {
                let layer = target.get();
                let item =
                    find_track_item_mut(doc, *target).ok_or(CommandError::LayerNotFound(layer))?;
                if !matches!(item, TrackItem::Clip(_)) {
                    return Err(CommandError::TrackItemNotClip { layer });
                }
                validate_clip_in_payload(
                    doc,
                    layer,
                    (*old_start, *old_duration, *old_time_map),
                    (*new_start, *new_duration, *new_time_map),
                )?;
                let item =
                    find_track_item_mut(doc, *target).ok_or(CommandError::LayerNotFound(layer))?;
                let TrackItem::Clip(clip) = item else {
                    return Err(CommandError::TrackItemNotClip { layer });
                };
                clip.start = *new_start;
                clip.duration = *new_duration;
                clip.time_map = *new_time_map;
                Ok(())
            }
            Command::TrimClipOut {
                target,
                new_duration,
                ..
            } => {
                let layer = target.get();
                let start = {
                    let item = find_track_item_mut(doc, *target)
                        .ok_or(CommandError::LayerNotFound(layer))?;
                    let TrackItem::Clip(clip) = item else {
                        return Err(CommandError::TrackItemNotClip { layer });
                    };
                    clip.start
                };
                validate_clip_duration(doc, layer, start, *new_duration)?;
                let item =
                    find_track_item_mut(doc, *target).ok_or(CommandError::LayerNotFound(layer))?;
                let TrackItem::Clip(clip) = item else {
                    return Err(CommandError::TrackItemNotClip { layer });
                };
                clip.duration = *new_duration;
                Ok(())
            }
            Command::SplitClip {
                target,
                old_duration,
                new_duration,
                right_parent,
                right_index,
                right_item,
                right_layer_names,
            } => apply_split_clip(
                doc,
                *target,
                *old_duration,
                *new_duration,
                *right_parent,
                *right_index,
                right_item,
                right_layer_names,
            ),
            Command::UnsplitClip {
                target,
                old_duration,
                new_duration,
                right_parent,
                right_index,
                right_item,
                right_layer_names,
            } => apply_unsplit_clip(
                doc,
                *target,
                *old_duration,
                *new_duration,
                *right_parent,
                *right_index,
                right_item,
                right_layer_names,
            ),
            Command::ReparentClip {
                target,
                old_parent,
                old_index,
                new_parent,
                new_index,
                new_start,
                ..
            } => apply_reparent_track_item(
                doc,
                *target,
                *old_parent,
                *old_index,
                *new_parent,
                *new_index,
                *new_start,
            ),
            Command::SetItemVisible { target, new, .. } => {
                find_envelope_mut(doc, *target)?.visible = *new;
                Ok(())
            }
            Command::SetItemSolo { target, new, .. } => {
                find_envelope_mut(doc, *target)?.solo = *new;
                Ok(())
            }
            Command::SetItemLock { target, new, .. } => {
                find_envelope_mut(doc, *target)?.lock = *new;
                Ok(())
            }
        }
    }
}
