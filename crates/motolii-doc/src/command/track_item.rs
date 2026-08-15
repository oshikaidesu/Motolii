use std::collections::BTreeMap;

use crate::schema::TrackItem;
use crate::{Document, LayerId};

use super::locate::{
    envelope_of, ensure_layer_names_match_item, find_items_vec, find_items_vec_mut,
};
use super::{CommandError, ParentLocator};

pub(super) fn apply_add_track_item(
    doc: &mut Document,
    parent: ParentLocator,
    index: usize,
    item: &TrackItem,
    layer_names: &BTreeMap<LayerId, String>,
) -> Result<(), CommandError> {
    // 事前検査のみ — 失敗時はツリー・台帳とも未変更。
    ensure_layer_names_match_item(item, layer_names)?;
    let len = find_items_vec(doc, parent)?.len();
    if index > len {
        return Err(CommandError::IndexOutOfRange { index, len });
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
    find_items_vec_mut(doc, parent)?.insert(index, item.clone());
    Ok(())
}

pub(super) fn apply_remove_track_item(
    doc: &mut Document,
    parent: ParentLocator,
    index: usize,
    item: &TrackItem,
    layer_names: &BTreeMap<LayerId, String>,
) -> Result<(), CommandError> {
    // 事前検査のみ — 失敗時はツリー・台帳とも未変更。
    ensure_layer_names_match_item(item, layer_names)?;
    let items = find_items_vec(doc, parent)?;
    if index >= items.len() {
        return Err(CommandError::IndexOutOfRange {
            index,
            len: items.len(),
        });
    }
    let found = envelope_of(&items[index]).layer_id;
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

    find_items_vec_mut(doc, parent)?.remove(index);
    for id in layer_names.keys() {
        doc.layers.remove(*id)?;
    }
    Ok(())
}
