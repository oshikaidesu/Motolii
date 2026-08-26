use motolii_core::{RationalTime, TimeMap};

use crate::schema::TrackItem;
use crate::validate;
use crate::{Document, LayerId};

use super::locate::{
    envelope_of, find_envelope, find_item_location, find_items_vec, find_items_vec_mut,
    find_track_item_mut,
};
use super::{Command, CommandError, ParentLocator};

pub(super) fn validate_clip_start(
    doc: &Document,
    layer_id: u64,
    new_start: RationalTime,
    duration: RationalTime,
) -> Result<(), CommandError> {
    let end = new_start.try_add(duration).map_err(|_| {
        CommandError::Validate(validate::DocumentError::ClipIntervalOverflow { layer_id })
    })?;
    if end > doc.composition.duration {
        return Err(CommandError::Validate(
            validate::DocumentError::ClipPastComposition {
                layer_id,
                end,
                comp: doc.composition.duration,
            },
        ));
    }
    Ok(())
}

pub(super) fn clip_interval_overflow(layer_id: u64) -> CommandError {
    CommandError::Validate(validate::DocumentError::ClipIntervalOverflow { layer_id })
}

pub(super) fn validate_clip_duration(
    doc: &Document,
    layer_id: u64,
    start: RationalTime,
    new_duration: RationalTime,
) -> Result<(), CommandError> {
    let end = start
        .try_add(new_duration)
        .map_err(|_| clip_interval_overflow(layer_id))?;
    if new_duration <= RationalTime::ZERO {
        return Err(CommandError::Validate(
            validate::DocumentError::NonPositiveClipDuration { layer_id },
        ));
    }
    if end > doc.composition.duration {
        return Err(CommandError::Validate(
            validate::DocumentError::ClipPastComposition {
                layer_id,
                end,
                comp: doc.composition.duration,
            },
        ));
    }
    Ok(())
}

pub(super) fn validate_clip_in_payload(
    doc: &Document,
    layer_id: u64,
    old: (RationalTime, RationalTime, TimeMap),
    new: (RationalTime, RationalTime, TimeMap),
) -> Result<(), CommandError> {
    let (old_start, old_duration, old_time_map) = old;
    let (new_start, new_duration, new_time_map) = new;
    let old_end = old_start
        .try_add(old_duration)
        .map_err(|_| clip_interval_overflow(layer_id))?;
    let new_end = new_start
        .try_add(new_duration)
        .map_err(|_| clip_interval_overflow(layer_id))?;
    let delta = new_start
        .try_sub(old_start)
        .map_err(|_| clip_interval_overflow(layer_id))?;
    let expected_source_start = old_time_map
        .try_map(delta)
        .map_err(|_| clip_interval_overflow(layer_id))?;

    if new_duration <= RationalTime::ZERO {
        return Err(CommandError::Validate(
            validate::DocumentError::NonPositiveClipDuration { layer_id },
        ));
    }
    if old_duration <= RationalTime::ZERO
        || old_end != new_end
        || old_time_map.speed_num() != new_time_map.speed_num()
        || old_time_map.speed_den() != new_time_map.speed_den()
        || old_time_map.overrun_mode != new_time_map.overrun_mode
        || expected_source_start != new_time_map.source_start
    {
        return Err(CommandError::InvalidClipTrim { layer: layer_id });
    }
    if new_end > doc.composition.duration {
        return Err(CommandError::Validate(
            validate::DocumentError::ClipPastComposition {
                layer_id,
                end: new_end,
                comp: doc.composition.duration,
            },
        ));
    }
    Ok(())
}

/// CU-201M-S: Clip `start`変更commandを構築する。成功・失敗とも live Document 不変。
pub fn prepare_set_clip_start(
    doc: &Document,
    target: LayerId,
    new: RationalTime,
) -> Result<Option<Command>, CommandError> {
    let layer = target.get();
    let (_, _, item) = find_item_location(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    let TrackItem::Clip(clip) = item else {
        return Err(CommandError::TrackItemNotClip { layer });
    };
    let old = clip.start;
    if new == old {
        return Ok(None);
    }
    validate_clip_start(doc, layer, new, clip.duration)?;
    Ok(Some(Command::SetClipStart { target, old, new }))
}

/// CU-201T-S: 左edgeを変更し、旧右端と残存source写像を保つcommandを構築する。
pub fn prepare_trim_clip_in(
    doc: &Document,
    target: LayerId,
    new_start: RationalTime,
) -> Result<Option<Command>, CommandError> {
    let layer = target.get();
    let (_, _, item) = find_item_location(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    let TrackItem::Clip(clip) = item else {
        return Err(CommandError::TrackItemNotClip { layer });
    };
    if new_start == clip.start {
        return Ok(None);
    }

    let old_start = clip.start;
    let old_duration = clip.duration;
    let old_time_map = clip.time_map;
    let old_end = old_start
        .try_add(old_duration)
        .map_err(|_| clip_interval_overflow(layer))?;
    let new_duration = old_end
        .try_sub(new_start)
        .map_err(|_| clip_interval_overflow(layer))?;
    let delta = new_start
        .try_sub(old_start)
        .map_err(|_| clip_interval_overflow(layer))?;
    let new_source_start = old_time_map
        .try_map(delta)
        .map_err(|_| clip_interval_overflow(layer))?;
    let new_time_map = TimeMap::try_new(
        new_source_start,
        old_time_map.speed_num(),
        old_time_map.speed_den(),
        old_time_map.overrun_mode,
    )
    .map_err(|_| clip_interval_overflow(layer))?;

    validate_clip_in_payload(
        doc,
        layer,
        (old_start, old_duration, old_time_map),
        (new_start, new_duration, new_time_map),
    )?;
    Ok(Some(Command::TrimClipIn {
        target,
        old_start,
        old_duration,
        old_time_map,
        new_start,
        new_duration,
        new_time_map,
    }))
}

/// CU-201T-S: 右edgeを変更し、`start`とTimeMapを保つcommandを構築する。
pub fn prepare_trim_clip_out(
    doc: &Document,
    target: LayerId,
    new_end: RationalTime,
) -> Result<Option<Command>, CommandError> {
    let layer = target.get();
    let (_, _, item) = find_item_location(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    let TrackItem::Clip(clip) = item else {
        return Err(CommandError::TrackItemNotClip { layer });
    };
    let old_end = clip
        .start
        .try_add(clip.duration)
        .map_err(|_| clip_interval_overflow(layer))?;
    if new_end == old_end {
        return Ok(None);
    }
    let new_duration = new_end
        .try_sub(clip.start)
        .map_err(|_| clip_interval_overflow(layer))?;
    validate_clip_duration(doc, layer, clip.start, new_duration)?;
    Ok(Some(Command::TrimClipOut {
        target,
        old_duration: clip.duration,
        new_duration,
    }))
}

/// dest の `(parent, index)` は「外したあとの挿入位置」。同じ親なら dest の現在indexでよい。
pub fn prepare_reparent_clip(
    doc: &Document,
    target: LayerId,
    new_parent: ParentLocator,
    new_index: usize,
    new_start: Option<RationalTime>,
) -> Result<Option<Command>, CommandError> {
    let layer = target.get();
    let (old_parent, old_index, item) =
        find_item_location(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    let old_start = match item {
        TrackItem::Clip(clip) => Some(clip.start),
        TrackItem::Group(_) => {
            if new_start.is_some() {
                return Err(CommandError::TrackItemNotClip { layer });
            }
            None
        }
    };
    let resolved_start = match (old_start, new_start) {
        (Some(_old), Some(new)) => Some(new),
        (Some(old), None) => Some(old),
        (None, None) => None,
        (None, Some(_)) => return Err(CommandError::TrackItemNotClip { layer }),
    };

    if old_parent == new_parent {
        let len_after = find_items_vec(doc, old_parent)?.len() - 1;
        if new_index > len_after {
            return Err(CommandError::IndexOutOfRange {
                index: new_index,
                len: len_after,
            });
        }
    } else {
        let dest_len = find_items_vec(doc, new_parent)?.len();
        if new_index > dest_len {
            return Err(CommandError::IndexOutOfRange {
                index: new_index,
                len: dest_len,
            });
        }
    }

    if old_parent == new_parent && old_index == new_index && old_start == resolved_start {
        return Ok(None);
    }
    if let (TrackItem::Clip(clip), Some(start)) = (item, resolved_start) {
        if Some(start) != old_start {
            validate_clip_start(doc, layer, start, clip.duration)?;
        }
    }
    Ok(Some(Command::ReparentClip {
        target,
        old_parent,
        old_index,
        new_parent,
        new_index,
        old_start,
        new_start: resolved_start,
    }))
}

pub fn prepare_set_item_visible(
    doc: &Document,
    target: LayerId,
    new: bool,
) -> Result<Option<Command>, CommandError> {
    let layer = target.get();
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    if env.visible == new {
        return Ok(None);
    }
    Ok(Some(Command::SetItemVisible {
        target,
        old: env.visible,
        new,
    }))
}

pub fn prepare_set_item_solo(
    doc: &Document,
    target: LayerId,
    new: bool,
) -> Result<Option<Command>, CommandError> {
    let layer = target.get();
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    if env.solo == new {
        return Ok(None);
    }
    Ok(Some(Command::SetItemSolo {
        target,
        old: env.solo,
        new,
    }))
}

/// 編集禁止フラグ。**評価・描画に影響しない**(B④)ので、UI 側の可否だけが変わる。
pub fn prepare_set_item_lock(
    doc: &Document,
    target: LayerId,
    new: bool,
) -> Result<Option<Command>, CommandError> {
    let layer = target.get();
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    if env.lock == new {
        return Ok(None);
    }
    Ok(Some(Command::SetItemLock {
        target,
        old: env.lock,
        new,
    }))
}

/// 行の色。**`None` は「選んでいない」**で、既定色を導くのは UI 側。
pub fn prepare_set_item_color(
    doc: &Document,
    target: LayerId,
    new: Option<u32>,
) -> Result<Option<Command>, CommandError> {
    let layer = target.get();
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    if env.color == new {
        return Ok(None);
    }
    Ok(Some(Command::SetItemColor {
        target,
        old: env.color,
        new,
    }))
}

pub(super) fn apply_reparent_track_item(
    doc: &mut Document,
    target: LayerId,
    old_parent: ParentLocator,
    old_index: usize,
    new_parent: ParentLocator,
    new_index: usize,
    new_start: Option<RationalTime>,
) -> Result<(), CommandError> {
    let layer = target.get();
    let old_items = find_items_vec(doc, old_parent)?;
    if old_index >= old_items.len() {
        return Err(CommandError::IndexOutOfRange {
            index: old_index,
            len: old_items.len(),
        });
    }
    if envelope_of(&old_items[old_index]).layer_id != target {
        return Err(CommandError::RemoveItemMismatch {
            expected: layer,
            found: envelope_of(&old_items[old_index]).layer_id.get(),
        });
    }
    if old_parent == new_parent {
        let len_after = old_items.len() - 1;
        if new_index > len_after {
            return Err(CommandError::IndexOutOfRange {
                index: new_index,
                len: len_after,
            });
        }
    } else {
        let dest_len = find_items_vec(doc, new_parent)?.len();
        if new_index > dest_len {
            return Err(CommandError::IndexOutOfRange {
                index: new_index,
                len: dest_len,
            });
        }
    }
    if let Some(start) = new_start {
        let duration = match &old_items[old_index] {
            TrackItem::Clip(clip) => clip.duration,
            TrackItem::Group(_) => return Err(CommandError::TrackItemNotClip { layer }),
        };
        validate_clip_start(doc, layer, start, duration)?;
    }

    if old_parent == new_parent {
        let items = find_items_vec_mut(doc, old_parent)?;
        let item = items.remove(old_index);
        items.insert(new_index, item);
    } else {
        let item = find_items_vec_mut(doc, old_parent)?.remove(old_index);
        find_items_vec_mut(doc, new_parent)?.insert(new_index, item);
    }
    if let Some(start) = new_start {
        let item = find_track_item_mut(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
        let TrackItem::Clip(clip) = item else {
            return Err(CommandError::TrackItemNotClip { layer });
        };
        clip.start = start;
    }
    Ok(())
}
