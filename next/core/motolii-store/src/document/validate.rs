//! `Document::write` (`apply.rs`) が使う書き込み前ガード群。`document.rs` から
//! 移送(裁定220 SP-3、中身は変えていない)。

use re_log_types::EntityPath;
use re_types_core::SerializedComponentBatch;

use crate::components::{descriptor_attrs, TrackJson};
use crate::slot::PropertyLink;
use crate::view::StoreView;
use crate::{LayerId, LayerSource, Mask, PropertyId, StoreError};

/// `parent` の親鎖を辿って `layer` 自身へ戻ってこないことを確かめる。
///
/// **循環参照は絶対に作れない**(layer-meta 束の柵)。作れると、親を辿って transform を
/// 合成する日(未実装、resolve はまだ parent を読んでいない)に無限ループになる。
/// `seen` は防御的な保険 — 既存の親鎖が(バグ等で)既に壊れて循環していても、
/// この呼び出しが無限に回らないようにする。
pub(super) fn validate_no_parent_cycle(
    view: &StoreView,
    layer: LayerId,
    new_parent: Option<LayerId>,
) -> Result<(), StoreError> {
    let mut current = new_parent;
    let mut seen = std::collections::HashSet::new();
    while let Some(candidate) = current {
        if candidate == layer {
            return Err(StoreError::Property(format!(
                "layer {} の parent を layer {} にすると循環参照になる(親鎖を辿ると \
                 自分自身へ戻ってくる)",
                layer.0,
                new_parent.expect("new_parent が None なら親鎖を辿らない").0
            )));
        }
        if !seen.insert(candidate) {
            break;
        }
        current = view
            .attrs(candidate)
            .ok()
            .flatten()
            .and_then(|attrs| attrs.parent);
    }
    Ok(())
}

/// `new_link` の参照鎖を辿って `(layer, property)` 自身へ戻ってこないことを確かめる。
/// [`validate_no_parent_cycle`] の `(LayerId, PropertyId)` 版 — 新しい検査手法では
/// なく同型の複製(裁定206・`docs/reviews/2026-08-22-persona-touchdesigner-round2.md`
/// §1.5 の設計どおり)。
///
/// **循環参照は絶対に作れない**(書き込み時拒否)。Blender の driver 依存グラフは
/// 循環を実行時に検出し「ランダムな点で切って」評価を続ける([Blender Projects
/// #64793](https://projects.blender.org/blender/blender/issues/64793) が報告する
/// 「ジャンプ・チラつき」の原因そのもの)——ここは書き込み時に拒むので、その弱点を
/// 生じさせない。`seen` は防御的な保険(`validate_no_parent_cycle` と同じ理由)。
/// **裁定213 で分岐に対応した** — 排他的な `Link` 単鎖だった頃は「次の1点」を
/// 辿るだけで足りたが、`modulators` は複数本になり得るので、これは DFS
/// (スタック + 訪問済み集合)になった。`new_link` 自身の参照鎖に加えて、
/// 参照先が既に持っている**全** modulator を枝として辿る——どの枝から辿っても
/// `start` へ戻れば拒む。
pub(super) fn validate_no_link_cycle(
    view: &StoreView,
    layer: LayerId,
    property: &PropertyId,
    new_link: &PropertyLink,
) -> Result<(), StoreError> {
    let start = (layer, property.clone());
    let mut seen: std::collections::HashSet<(LayerId, PropertyId)> = std::collections::HashSet::new();
    seen.insert(start.clone());
    let mut stack = vec![(new_link.source_layer, new_link.source_property.clone())];
    while let Some(candidate) = stack.pop() {
        if candidate == start {
            return Err(StoreError::Property(format!(
                "layer {} の property `{}` を layer {} の property `{}` へ link すると \
                 循環参照になる(参照鎖を辿ると自分自身へ戻ってくる)",
                layer.0,
                property.name(),
                candidate.0 .0,
                candidate.1.name()
            )));
        }
        if !seen.insert(candidate.clone()) {
            continue; // 既に見た枝(合流点)は辿り直さない。
        }
        if let Some(source) = view.property_source(candidate.0, &candidate.1).ok().flatten() {
            for modulator in &source.modulators {
                stack.push((modulator.source_layer, modulator.source_property.clone()));
            }
        }
    }
    Ok(())
}

/// **壁7の恒久修正**(2026-08-22、`docs/reviews/2026-08-22-persona-motion-round2.md`
/// §1): `SetMasks` の一覧に**現在の一覧に無い id**(=新規追加)が混ざっていたら、
/// その id の `mask.{id}.shape` が(この呼び出しの時点で)既に読めることを要求する。
///
/// 「読めること」だけを見る——形状の**型**(`Value::Path` かどうか)や実際の解決は
/// 見ない。型検査は既存どおり `StoreView::resolved_masks` が resolve 時に行う
/// (`mask.rs`/`view.rs` の既存の役割分担、裁定37「無い」と「壊れている」の非同義を
/// そのまま踏襲——ここは「無い」だけを拒む)。
///
/// 既存 mask(並べ替え・削除・mode 変更で id が変わらない物)はここに引っかからない
/// — shape は以前から存在するはずなので、再検査しても意味が無い。
pub(super) fn validate_masks_have_shapes(
    view: &StoreView,
    layer: LayerId,
    masks: &[Mask],
) -> Result<(), StoreError> {
    let existing_ids: std::collections::HashSet<crate::MaskId> =
        view.masks(layer)?.iter().map(|m| m.id).collect();
    for mask in masks {
        if existing_ids.contains(&mask.id) {
            continue;
        }
        let shape_property = crate::PropertyId::mask_shape(mask.id);
        if view.property_source(layer, &shape_property)?.is_none() {
            return Err(StoreError::Property(format!(
                "マスク {} を追加しようとしたが `mask.{}.shape` がまだ無い — 先に \
                 (同じ apply_all の中で)shape の SetTrack を書くこと。1回で束ねたい \
                 なら `Intent::AddMask` を使うこと",
                mask.id, mask.id
            )));
        }
    }
    Ok(())
}

/// locked な layer への層変更 Intent を拒む(`SetAttrs` は別扱い — 上の
/// `Intent::SetAttrs` 腕を参照。`locked` 自身の解除/再ロックだけ通す規則は
/// `SetAttrs` にしか無い、他の Intent には「触ってよい locked 以外のフィールド」が
/// 無いので単純に全拒否でよい)。
pub(super) fn check_not_locked(view: &StoreView, layer: LayerId) -> Result<(), StoreError> {
    if view.attrs(layer)?.unwrap_or_default().locked {
        return Err(StoreError::Property(format!(
            "layer {} は locked なので編集できない(先に SetAttrs で locked を外すこと)",
            layer.0
        )));
    }
    Ok(())
}

/// 凍結中(`frozen`)の部分木への編集を理由つきで拒む(裁定119
/// `docs/reviews/2026-08-20-group-layer-semantics-decision.md` §4「凍結中の中身への
/// 編集は理由つき拒否。黙って自動解凍しない」)。
///
/// `layer` の祖先鎖に `frozen == true` な `LayerSource::Group` が居れば `Err`。
/// **`layer` 自身の frozen 状態は見ない** — 凍結された Group 自身への編集(位置を
/// 動かす・改名する等)は「中身」ではないので、ここでは拒まない
/// (`StoreView::frozen_ancestor` の doc 参照。この非対称性が
/// `Intent::RemoveLayer(frozen_group)` を許しつつ `Intent::RemoveLayer(child)` を
/// 拒む仕組みそのもの — 凍結グループ自体の削除は tombstone なので可逆、grouping
/// 束のテスト内コメント参照)。
pub(super) fn check_not_frozen(view: &StoreView, layer: LayerId) -> Result<(), StoreError> {
    if let Some(group) = view.frozen_ancestor(layer)? {
        return Err(StoreError::Property(format!(
            "layer {} は凍結中(frozen)のグループ(layer {})の部分木にあるので編集できない \
             (先にそのグループを unfreeze すること)",
            layer.0, group.0
        )));
    }
    Ok(())
}

/// `candidate` が(それ自身が)frozen な `LayerSource::Group` か、または frozen な
/// Group の部分木の中に居るか。`check_not_frozen` は `layer` 自身の frozen 状態を
/// 見ないので、「凍結中の Group そのものを新しい親にする」経路(= 部分木へ新しい
/// 子を迎え入れる編集)はここでしか捕まえられない(`Intent::SetAttrs` の `parent`
/// 検証が呼ぶ)。
pub(super) fn is_frozen_or_within_frozen(view: &StoreView, candidate: LayerId) -> Result<bool, StoreError> {
    let self_frozen = view
        .meta(candidate)?
        .map(|meta| meta.source == LayerSource::Group)
        .unwrap_or(false)
        && view.attrs(candidate)?.unwrap_or_default().frozen;
    Ok(self_frozen || view.frozen_ancestor(candidate)?.is_some())
}

/// [`Intent::Freeze`]/[`Intent::Unfreeze`] の共通実装。`group` が present な
/// `LayerSource::Group` layer であることを確かめてから、`attrs.frozen` だけを
/// 書き換えて返す(read-modify-write、`Intent::SetAttrs` と同じ物理形 —
/// `descriptor_attrs()` の同じ component へ書くので、専用 component は増やさない)。
///
/// **locked/frozen な祖先の柵はここでは掛けない** — 呼び出し側([`Document::write`]
/// の `Intent::Freeze`/`Intent::Unfreeze` 腕)が `check_not_locked`/`check_not_frozen`
/// を先に呼んでから、この関数へは検証済みの `group` だけを渡す(他の read-modify-write
/// 腕と同じ役割分担)。
pub(super) fn freeze_attrs_batch(
    view: &StoreView,
    group: LayerId,
    frozen: bool,
) -> Result<(EntityPath, Vec<SerializedComponentBatch>), StoreError> {
    if !view.has_layer(group) {
        return Err(StoreError::Property(format!(
            "layer {} は存在しない(present ではない)ので freeze/unfreeze できない",
            group.0
        )));
    }
    let is_group = view
        .meta(group)?
        .map(|meta| meta.source == LayerSource::Group)
        .unwrap_or(false);
    if !is_group {
        return Err(StoreError::Property(format!(
            "layer {} は LayerSource::Group ではないので freeze/unfreeze できない",
            group.0
        )));
    }
    let mut attrs = view.attrs(group)?.unwrap_or_default();
    attrs.frozen = frozen;
    let json = serde_json::to_string(&attrs)?;
    Ok((
        group.entity_path(),
        vec![SerializedComponentBatch {
            descriptor: descriptor_attrs(),
            array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                .map_err(|e| StoreError::Chunk(e.to_string()))?,
        }],
    ))
}
