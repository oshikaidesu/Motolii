use std::collections::BTreeMap;

use crate::schema::{Group, ItemEnvelope, TrackItem};
use crate::{Document, LayerId};

use super::locate::{
    envelope_of, ensure_layer_names_match_item, find_item_location, find_items_vec,
    find_items_vec_mut, layer_names_for_item,
};
use super::{Command, CommandError, ParentLocator};

/// 空の Group を1つ作る準備をする。**中身を入れるのは呼び側**である。
///
/// グループ化は「空の Group を置く」+「選んだものを `ReparentClip` で入れる」の
/// 組み合わせで表す。**新しい意味の command を増やさない** — 逆操作は既にある
/// `RemoveTrackItem` / `ReparentClip` の逆で閉じており、Undo は 1 gesture で戻る。
///
/// LayerId は `reserve` のみ(台帳エントリは作らない)。エントリは戻り値の
/// `AddTrackItem.layer_names` の apply で載り、Undo の Remove で外れる —
/// 複製と同じ経路なので、`max_layers` に孤児が溜まらない。
pub fn prepare_add_group(
    doc: &mut Document,
    parent: ParentLocator,
    index: usize,
    name: &str,
) -> Result<Command, CommandError> {
    let len = find_items_vec(doc, parent)?.len();
    if index > len {
        return Err(CommandError::IndexOutOfRange { index, len });
    }
    let layer = doc.layers.reserve()?;
    let mut layer_names = BTreeMap::new();
    layer_names.insert(layer, name.to_owned());
    Ok(Command::AddTrackItem {
        parent,
        index,
        item: TrackItem::Group(Group {
            envelope: ItemEnvelope::new(layer),
            children: Vec::new(),
        }),
        layer_names,
    })
}

/// `target`が指すTrackItem(Clip/Group、子ごと)を外す準備をする。
///
/// **`prepare_duplicate_track_item`の裏返しである。** 複製が`AddTrackItem`で
/// 台帳へ載せるのと同じ経路を逆へ通すので、Undo(inverseの`AddTrackItem`)では
/// **同じLayerIdと表示名が`restore`で戻る** — idを振り直さない。
///
/// この関数はツリーも台帳も変更しない(適用は`apply_command`側)。
/// 同名の`Ok(None)`は無い — 「消す対象がある」なら必ず変化する。
pub fn prepare_remove_track_item(
    doc: &Document,
    target: LayerId,
) -> Result<Command, CommandError> {
    let (parent, index, item) =
        find_item_location(doc, target).ok_or(CommandError::LayerNotFound(target.get()))?;
    let layer_names = layer_names_for_item(doc, item)?;
    Ok(Command::RemoveTrackItem {
        parent,
        index,
        item: item.clone(),
        layer_names,
    })
}

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
