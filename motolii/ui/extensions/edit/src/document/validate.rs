
#[allow(unused_imports)]
use crate::document::{Animate, Document, Intent};
use re_log_types::EntityPath;
use re_types_core::SerializedComponentBatch;

use motolii_doc::store::components::{descriptor_attrs, TrackJson};
use motolii_doc::store::slot::PropertyLink;
use motolii_doc::store::view::StoreView;
use motolii_doc::store::{LayerId, Mask, PropertyId, StoreError};

pub(super) fn validate_no_parent_cycle(
    view: &StoreView,
    layer: LayerId,
    new_parent: Option<LayerId>,
) -> Result<(), StoreError> {
    if let Some(parent) = new_parent {
        if !view.has_layer(parent) {
            return Err(StoreError::Property(format!("Parent layer {} does not exist", parent.0)));
        }
    }
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

pub(super) fn validate_masks_have_shapes(
    view: &StoreView,
    layer: LayerId,
    masks: &[Mask],
) -> Result<(), StoreError> {
    let existing_ids: std::collections::HashSet<motolii_doc::store::MaskId> =
        view.masks(layer)?.iter().map(|m| m.id).collect();
    for mask in masks {
        if existing_ids.contains(&mask.id) {
            continue;
        }
        let shape_property = motolii_doc::store::PropertyId::mask_shape(mask.id);
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

pub(super) fn check_not_locked(view: &StoreView, layer: LayerId) -> Result<(), StoreError> {
    if view.attrs(layer)?.unwrap_or_default().locked {
        return Err(StoreError::Property(format!(
            "layer {} は locked なので編集できない(先に SetAttrs で locked を外すこと)",
            layer.0
        )));
    }
    Ok(())
}

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
    // 旗だけ残す。評価も編集拒否もしない。古い undo が Freeze / Unfreeze をまだ持つ。
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
