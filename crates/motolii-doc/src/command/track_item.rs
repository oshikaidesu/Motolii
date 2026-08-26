use std::collections::BTreeMap;

use motolii_core::RationalTime;

use motolii_core::TimeMap;

use crate::schema::{
    AudioComponent, Clip, ClipSource, Group, ItemEnvelope, TrackItem, VideoComponent,
};
use crate::{AssetId, Document, LayerId};

use super::locate::{
    ensure_layer_names_match_item, envelope_of, find_item_location, find_items_vec,
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

/// 素材clipを最初のTrackの末尾へ置く準備をする(Place意味のCLI/GUI共通口)。
///
/// - 尺は`min(素材の長さ, composition end - start)`。**素材より長いclipは作らない** —
///   CU-101は「startからcomposition endまで」だったが、実走(2026-08-18観察(1))で
///   4sの素材が10sのclipになり終端がフリーズフレームの尾になると分かったので置き換えた。
///   `Asset.duration`を持たない素材(生成系・stream・旧文書)は従来どおりcomposition endまで
/// - componentはasset_typeの大分類から: `video/*`と`image/*`→video ordinal 0、
///   `audio/*`→audio ordinal 0。stream実在の細部はexport/mix時の検証に任せる。
///   静止画が絵の列に入るのは、mediaが**1フレームのvideo stream**として読むから
///   (2026-08-18: 利用者の初回タッチで閉まっていた扉。尺が無い素材なので、
///   上の「長さ不明→composition end まで」がそのまま静止画の尺の意味になる)
/// - `LayerId`はreserveのみ。表示名はasset名で`layer_names`に載せる(`prepare_add_group`と同型)
pub fn prepare_place_asset_clip(
    doc: &mut Document,
    asset_id: AssetId,
    start: RationalTime,
) -> Result<Command, CommandError> {
    let asset = doc.assets.get(asset_id).ok_or(CommandError::Validate(
        crate::validate::DocumentError::UnknownAssetId { id: asset_id.get() },
    ))?;
    let (video, audio) =
        if asset.asset_type.starts_with("video/") || asset.asset_type.starts_with("image/") {
            (Some(VideoComponent::ordinal(0)), Vec::new())
        } else if asset.asset_type.starts_with("audio/") {
            (None, vec![AudioComponent::ordinal(0)])
        } else {
            return Err(CommandError::UnsupportedPlacementAssetType {
                asset_type: asset.asset_type.clone(),
            });
        };
    let name = asset.name.clone();
    // 素材の長さ。非正値は「測れていない」と同じ扱いにする(壊れたヒントで
    // 置けなくなるより、従来のcomposition end挙動へ落ちる方が人にとって普通)。
    let source_duration = asset.duration.filter(|d| *d > RationalTime::ZERO);

    // composition end - start(正確な有理数演算、i128で桁溢れ検査)。
    let end = doc.composition.duration;
    let num =
        (end.num() as i128) * (start.den() as i128) - (start.num() as i128) * (end.den() as i128);
    let den = (end.den() as i128) * (start.den() as i128);
    if num <= 0 {
        return Err(CommandError::PlacementOutsideComposition);
    }
    let remaining = i64::try_from(num)
        .ok()
        .zip(i64::try_from(den).ok())
        .and_then(|(num, den)| RationalTime::try_new(num, den).ok())
        .ok_or(CommandError::PlacementTimeOverflow)?;
    // 素材の長さを知っているなら、そこで切る。知らないなら残り全部。
    let duration = match source_duration {
        Some(source) => source.min(remaining),
        None => remaining,
    };

    let track = doc
        .tracks
        .first()
        .ok_or(CommandError::NoTrackForPlacement)?;
    let parent = ParentLocator::Track(track.id);
    let index = track.items.len();

    let layer = doc.layers.reserve()?;
    let mut layer_names = BTreeMap::new();
    layer_names.insert(layer, name);
    Ok(Command::AddTrackItem {
        parent,
        index,
        item: TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start,
            duration,
            time_map: TimeMap::identity(),
            source: ClipSource::Asset {
                asset: asset_id,
                video,
                audio,
            },
        }),
        layer_names,
    })
}

/// メモを置く準備をする。**末尾に足す** — 保持順が index の意味なので、
/// 時刻で並べ替えない(並べ替えると既存 command の宛先が動く)。
pub fn prepare_add_locator(doc: &Document, t: RationalTime, text: &str) -> Command {
    Command::AddLocator {
        index: doc.locators.len(),
        locator: crate::schema::Locator {
            t,
            text: text.to_owned(),
        },
    }
}

/// メモを外す準備をする。payload も載せる(inverse がそのまま戻せる)。
pub fn prepare_remove_locator(doc: &Document, index: usize) -> Result<Command, CommandError> {
    let locator = doc
        .locators
        .get(index)
        .ok_or(CommandError::IndexOutOfRange {
            index,
            len: doc.locators.len(),
        })?;
    Ok(Command::RemoveLocator {
        index,
        locator: locator.clone(),
    })
}

/// メモの時刻。same-value は `None`。
pub fn prepare_set_locator_time(
    doc: &Document,
    index: usize,
    new: RationalTime,
) -> Result<Option<Command>, CommandError> {
    let locator = doc
        .locators
        .get(index)
        .ok_or(CommandError::IndexOutOfRange {
            index,
            len: doc.locators.len(),
        })?;
    if locator.t == new {
        return Ok(None);
    }
    Ok(Some(Command::SetLocatorTime {
        index,
        old: locator.t,
        new,
    }))
}

/// メモの文。same-value は `None`。
pub fn prepare_set_locator_text(
    doc: &Document,
    index: usize,
    new: &str,
) -> Result<Option<Command>, CommandError> {
    let locator = doc
        .locators
        .get(index)
        .ok_or(CommandError::IndexOutOfRange {
            index,
            len: doc.locators.len(),
        })?;
    if locator.text == new {
        return Ok(None);
    }
    Ok(Some(Command::SetLocatorText {
        index,
        old: locator.text.clone(),
        new: new.to_owned(),
    }))
}

/// 表示名を差し替える準備をする。same-value は `None`。
///
/// **名前は識別子ではない。** 参照は全部 `LayerId` なので、変えても
/// `transform.parent` / `LookAt` / journal の指し先は動かない。
pub fn prepare_set_layer_name(
    doc: &Document,
    target: LayerId,
    new: &str,
) -> Result<Option<Command>, CommandError> {
    let old = doc
        .layers
        .display_name(target)
        .ok_or(CommandError::LayerNotFound(target.get()))?;
    if old == new {
        return Ok(None);
    }
    Ok(Some(Command::SetLayerName {
        target,
        old: old.to_owned(),
        new: new.to_owned(),
    }))
}

/// `target`が指すTrackItem(Clip/Group、子ごと)を外す準備をする。
///
/// **`prepare_duplicate_track_item`の裏返しである。** 複製が`AddTrackItem`で
/// 台帳へ載せるのと同じ経路を逆へ通すので、Undo(inverseの`AddTrackItem`)では
/// **同じLayerIdと表示名が`restore`で戻る** — idを振り直さない。
///
/// この関数はツリーも台帳も変更しない(適用は`apply_command`側)。
/// 同名の`Ok(None)`は無い — 「消す対象がある」なら必ず変化する。
pub fn prepare_remove_track_item(doc: &Document, target: LayerId) -> Result<Command, CommandError> {
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
