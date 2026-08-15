use std::collections::BTreeMap;

use motolii_core::{RationalTime, TimeMap};

use crate::duplicate::{duplicate_track_item, DuplicateError};
use crate::schema::TrackItem;
use crate::{Document, LayerId};

use super::clip::{clip_interval_overflow, validate_clip_duration};
use super::locate::{
    envelope_of, ensure_layer_names_match_item, find_item_location, find_items_vec,
    find_items_vec_mut, find_track_item_mut,
};
use super::{Command, CommandError, ParentLocator};

pub fn prepare_split_clip(
    doc: &mut Document,
    target: LayerId,
    at: RationalTime,
) -> Result<Option<Command>, CommandError> {
    let layer = target.get();
    let (start, old_duration, end) = {
        let (_, _, item) =
            find_item_location(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
        let TrackItem::Clip(clip) = item else {
            return Err(CommandError::TrackItemNotClip { layer });
        };
        let end = clip
            .start
            .try_add(clip.duration)
            .map_err(|_| clip_interval_overflow(layer))?;
        (clip.start, clip.duration, end)
    };
    if at <= start || at >= end {
        return Err(CommandError::SplitNotInterior { layer });
    }
    let new_duration = at
        .try_sub(start)
        .map_err(|_| clip_interval_overflow(layer))?;
    validate_clip_duration(doc, layer, start, new_duration)?;

    let add = duplicate_track_item(doc, target).map_err(|error| match error {
        DuplicateError::Command(command) => command,
        DuplicateError::LayerId(error) => CommandError::LayerIdAlloc(error),
        DuplicateError::StableId(error) => CommandError::StableIdAlloc(error),
    })?;
    let Command::AddTrackItem {
        parent: right_parent,
        index: right_index,
        mut item,
        layer_names,
    } = add
    else {
        return Err(CommandError::TrackItemNotClip { layer });
    };
    let TrackItem::Clip(right) = &mut item else {
        return Err(CommandError::TrackItemNotClip { layer });
    };
    let delta = at
        .try_sub(start)
        .map_err(|_| clip_interval_overflow(layer))?;
    let new_source_start = right
        .time_map
        .try_map(delta)
        .map_err(|_| clip_interval_overflow(layer))?;
    right.time_map = TimeMap::try_new(
        new_source_start,
        right.time_map.speed_num(),
        right.time_map.speed_den(),
        right.time_map.overrun_mode,
    )
    .map_err(|_| clip_interval_overflow(layer))?;
    right.start = at;
    right.duration = end.try_sub(at).map_err(|_| clip_interval_overflow(layer))?;
    Ok(Some(Command::SplitClip {
        target,
        old_duration,
        new_duration,
        right_parent,
        right_index,
        right_item: item,
        right_layer_names: layer_names,
    }))
}

pub(super) fn apply_split_clip(
    doc: &mut Document,
    target: LayerId,
    old_duration: RationalTime,
    new_duration: RationalTime,
    right_parent: ParentLocator,
    right_index: usize,
    right_item: &TrackItem,
    right_layer_names: &BTreeMap<LayerId, String>,
) -> Result<(), CommandError> {
    let layer = target.get();
    ensure_layer_names_match_item(right_item, right_layer_names)?;
    let len = find_items_vec(doc, right_parent)?.len();
    if right_index > len {
        return Err(CommandError::IndexOutOfRange {
            index: right_index,
            len,
        });
    }
    for id in right_layer_names.keys() {
        if !doc.layers.contains(*id) && id.get() == u64::MAX {
            return Err(CommandError::LayerIdAlloc(crate::LayerIdError::Exhausted));
        }
    }
    let start = {
        let item = find_track_item_mut(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
        let TrackItem::Clip(clip) = item else {
            return Err(CommandError::TrackItemNotClip { layer });
        };
        if clip.duration != old_duration {
            return Err(CommandError::InvalidClipTrim { layer });
        }
        clip.start
    };
    validate_clip_duration(doc, layer, start, new_duration)?;
    for (id, name) in right_layer_names {
        if !doc.layers.contains(*id) {
            doc.layers.restore(*id, name.clone())?;
        }
    }
    let item = find_track_item_mut(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    let TrackItem::Clip(clip) = item else {
        return Err(CommandError::TrackItemNotClip { layer });
    };
    clip.duration = new_duration;
    find_items_vec_mut(doc, right_parent)?.insert(right_index, right_item.clone());
    Ok(())
}

pub(super) fn apply_unsplit_clip(
    doc: &mut Document,
    target: LayerId,
    old_duration: RationalTime,
    new_duration: RationalTime,
    right_parent: ParentLocator,
    right_index: usize,
    right_item: &TrackItem,
    right_layer_names: &BTreeMap<LayerId, String>,
) -> Result<(), CommandError> {
    let layer = target.get();
    ensure_layer_names_match_item(right_item, right_layer_names)?;
    let items = find_items_vec(doc, right_parent)?;
    if right_index >= items.len() {
        return Err(CommandError::IndexOutOfRange {
            index: right_index,
            len: items.len(),
        });
    }
    let found = envelope_of(&items[right_index]).layer_id;
    let expected = envelope_of(right_item).layer_id;
    if found != expected {
        return Err(CommandError::RemoveItemMismatch {
            expected: expected.get(),
            found: found.get(),
        });
    }
    for id in right_layer_names.keys() {
        if !doc.layers.contains(*id) {
            return Err(CommandError::LayerNotFound(id.get()));
        }
    }
    let start = {
        let item = find_track_item_mut(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
        let TrackItem::Clip(clip) = item else {
            return Err(CommandError::TrackItemNotClip { layer });
        };
        if clip.duration != old_duration {
            return Err(CommandError::InvalidClipTrim { layer });
        }
        clip.start
    };
    validate_clip_duration(doc, layer, start, new_duration)?;
    find_items_vec_mut(doc, right_parent)?.remove(right_index);
    for id in right_layer_names.keys() {
        doc.layers.remove(*id)?;
    }
    let item = find_track_item_mut(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    let TrackItem::Clip(clip) = item else {
        return Err(CommandError::TrackItemNotClip { layer });
    };
    clip.duration = new_duration;
    Ok(())
}
