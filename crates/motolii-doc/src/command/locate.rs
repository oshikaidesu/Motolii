use std::collections::BTreeMap;

use crate::schema::{AudioComponent, ClipSource, ItemEnvelope, TrackItem};
use crate::{Document, LayerId};

use super::{CommandError, ParentLocator};

/// `item` subtreeのLayerId集合と`layer_names`のキーが一致することを要求する。
pub(super) fn ensure_layer_names_match_item(
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

pub(super) fn find_track_item_mut(doc: &mut Document, target: LayerId) -> Option<&mut TrackItem> {
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
