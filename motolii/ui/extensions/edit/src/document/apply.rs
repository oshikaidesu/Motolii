#[allow(unused_imports)]
use crate::document::{Animate, Document, Intent};
use re_types_core::SerializedComponentBatch;

use motolii_doc::store::components::{
    descriptor_assets, descriptor_attrs, descriptor_composition, descriptor_effects,
    descriptor_notebook, descriptor_markers, descriptor_masks, descriptor_meta, descriptor_present, descriptor_shapes,
    descriptor_slots, descriptor_text, descriptor_track, LayerPresent, TrackJson,
};
use motolii_doc::store::slot::PropertySource;
use motolii_doc::store::StoreError;

use crate::document::validate::{
    check_not_frozen, check_not_frozen_inside, check_not_locked, freeze_attrs_batch, is_frozen_or_within_frozen, property_is_inside,
    validate_masks_have_shapes, validate_no_link_cycle, validate_no_parent_cycle,
};
use motolii_doc::store::{};

impl Document {
    pub(crate) fn write(&mut self, intent: Intent, at: i64) -> Result<(), StoreError> {
        let batches = match intent {
            Intent::AddLayer(layer) => (layer.entity_path(), vec![serialize_present(true)?]),
            Intent::RemoveLayer(layer) => {
                let view = self.view();
                let present = view.layers();
                let mut removed = std::collections::HashSet::from([layer]);
                loop {
                    let before = removed.len();
                    for &candidate in &present {
                        if view.attrs(candidate)?.and_then(|attrs| attrs.parent)
                            .is_some_and(|parent| removed.contains(&parent)) {
                            removed.insert(candidate);
                        }
                    }
                    if removed.len() == before { break; }
                }
                let targets: Vec<_> = present.into_iter().filter(|id| removed.contains(id)).collect();
                for &target in &targets {
                    check_not_locked(&view, target)?;
                    check_not_frozen(&view, target)?;
                }
                for target in targets {
                    self.ingest(target.entity_path(), vec![serialize_present(false)?], at)?;
                }
                return Ok(());
            }
            Intent::SetComposition(composition) => {
                let json = serde_json::to_string(&composition)?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_composition(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetNotebook { notebook } => {
                notebook.validate()?;
                let json = serde_json::to_string(&notebook)?;
                (Self::composition_path(), vec![SerializedComponentBatch {
                    descriptor: descriptor_notebook(),
                    array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)]).map_err(|e| StoreError::Chunk(e.to_string()))?,
                }])
            }
            Intent::SetMarkers { markers } => {
                let json = serde_json::to_string(&markers)?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_markers(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetMasks { layer, masks } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen_inside(&self.view(), layer, "its masks")?;
                motolii_doc::store::mask::validate_unique_ids(&masks)?;
                validate_masks_have_shapes(&self.view(), layer, &masks)?;
                let json = serde_json::to_string(&masks)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_masks(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::AddMask { layer, mask, shape } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen_inside(&self.view(), layer, "its masks")?;
                let mut masks = self.view().masks(layer)?;
                masks.push(mask);
                motolii_doc::store::mask::validate_unique_ids(&masks)?;
                let masks_json = serde_json::to_string(&masks)?;
                let shape_property = motolii_doc::store::PropertyId::mask_shape(mask.id);
                let shape_json = serde_json::to_string(&PropertySource::track(shape))?;
                (
                    layer.entity_path(),
                    vec![
                        SerializedComponentBatch {
                            descriptor: descriptor_masks(),
                            array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(
                                masks_json,
                            )])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                        },
                        SerializedComponentBatch {
                            descriptor: descriptor_track(&shape_property),
                            array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(
                                shape_json,
                            )])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                        },
                    ],
                )
            }
            Intent::SetTiming { layer, timing } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen(&self.view(), layer)?;
                let current = self.view().meta(layer)?;
                let Some(mut meta) = current else {
                    return Err(StoreError::Property(format!(
                        "layer {} に素材が置かれていないので配置を決められない",
                        layer.0
                    )));
                };
                meta.timing = timing;
                let json = serde_json::to_string(&meta)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_meta(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetMeta { layer, meta } => {
                if self.view().meta(layer)?.is_some() {
                    return Err(StoreError::Property(format!(
                        "layer {} は既に meta を持つ。SetMeta は新規配置専用 — 既存 layer の \
                         素材/重ね順/配置を変えるには SetSource/SetOrder/SetTiming を使うこと",
                        layer.0
                    )));
                }
                let json = serde_json::to_string(&meta)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_meta(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetSource { layer, source } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen_inside(&self.view(), layer, "its source")?;
                let current = self.view().meta(layer)?;
                let Some(mut meta) = current else {
                    return Err(StoreError::Property(format!(
                        "layer {} に meta が無い(先に SetMeta で配置すること)",
                        layer.0
                    )));
                };
                meta.source = source;
                let json = serde_json::to_string(&meta)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_meta(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetOrder { layer, order } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen(&self.view(), layer)?;
                let current = self.view().meta(layer)?;
                let Some(mut meta) = current else {
                    return Err(StoreError::Property(format!(
                        "layer {} に meta が無い(先に SetMeta で配置すること)",
                        layer.0
                    )));
                };
                meta.order = order;
                let json = serde_json::to_string(&meta)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_meta(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetAttrs { layer, patch } => {
                check_not_frozen(&self.view(), layer)?;
                if let Some(Some(new_parent)) = patch.parent {
                    if is_frozen_or_within_frozen(&self.view(), new_parent)? {
                        return Err(StoreError::Property(format!(
                            "layer {} の parent を layer {} にはできない — \
                             凍結中(frozen)のグループか、その部分木の中にある \
                             (先に unfreeze すること)",
                            layer.0, new_parent.0
                        )));
                    }
                }
                let current = self.view().attrs(layer)?.unwrap_or_default();
                // 環境層は空(照明)であって時間の姿ではないので、ゴーストを持たない。
                // 環境にした瞬間に既存のゴーストも落とす(旨みがない: 裁定 2026-09-07)。
                let mut patch = patch;
                if patch.environment == Some(true) {
                    patch.ghost = Some(None);
                } else if patch.ghost.is_some_and(|g| g.is_some()) && current.environment {
                    return Err(StoreError::Property(format!(
                        "layer {} は環境層なのでゴーストを持てない",
                        layer.0
                    )));
                }
                if current.locked {
                    let touches_other_than_locked = patch.hidden.is_some()
                        || patch.parent.is_some()
                        || patch.blend_mode.is_some()
                        || patch.matte.is_some()
                        || patch.clip_to_below.is_some()
                        || patch.name.is_some()
                        || patch.auto_orient.is_some()
                        || patch.projection.is_some()
                        || patch.solo.is_some()
                        || patch.label_color.is_some();
                    if touches_other_than_locked {
                        return Err(StoreError::Property(format!(
                            "layer {} は locked なので attrs を変更できない(先に \
                             locked を外すこと)",
                            layer.0
                        )));
                    }
                }
                if let Some(new_parent) = patch.parent {
                    validate_no_parent_cycle(&self.view(), layer, new_parent)?;
                }
                let attrs = patch.apply_to(current);
                let json = serde_json::to_string(&attrs)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_attrs(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetEffects { layer, effects } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen_inside(&self.view(), layer, "its effects")?;
                motolii_doc::store::effect::validate_unique_ids(&effects)?;
                let json = serde_json::to_string(&effects)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_effects(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetShapes { layer, shapes } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen_inside(&self.view(), layer, "its shapes")?;
                let json = serde_json::to_string(&shapes)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_shapes(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetTextDocument { layer, document } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen_inside(&self.view(), layer, "its text")?;
                motolii_doc::store::text::validate(&document)?;
                let json = serde_json::to_string(&document)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_text(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetTrack {
                layer,
                property,
                track,
            } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen(&self.view(), layer)?;
                if property_is_inside(property.name()) { check_not_frozen_inside(&self.view(), layer, "its effects")?; }
                track
                    .validate()
                    .map_err(|error| StoreError::Property(error.to_string()))?;
                let json = serde_json::to_string(&PropertySource::track(track))?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_track(&property),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetConstant {
                layer,
                property,
                value,
            } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen(&self.view(), layer)?;
                if property_is_inside(property.name()) { check_not_frozen_inside(&self.view(), layer, "its effects")?; }
                (layer.entity_path(), vec![constant_batch(&property, value)?])
            }
            Intent::SetCameraConstant { property, value } => {
                (Self::composition_path(), vec![constant_batch(&property, value)?])
            }
            Intent::SetCameraTrack { property, track } => {
                track
                    .validate()
                    .map_err(|error| StoreError::Property(error.to_string()))?;
                let json = serde_json::to_string(&PropertySource::track(track))?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_track(&property),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetPropertySlot {
                layer,
                property,
                slot,
            } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen(&self.view(), layer)?;
                let json = serde_json::to_string(&PropertySource::slot(slot))?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_track(&property),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetPropertyLink {
                layer,
                property,
                link,
            } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen(&self.view(), layer)?;
                validate_no_link_cycle(&self.view(), layer, &property, &link)?;
                let json = serde_json::to_string(&PropertySource::link_only(link))?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_track(&property),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetCameraPropertySlot { property, slot } => {
                let json = serde_json::to_string(&PropertySource::slot(slot))?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_track(&property),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetPropertyModulators {
                layer,
                property,
                modulators,
            } => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen(&self.view(), layer)?;
                for link in &modulators {
                    validate_no_link_cycle(&self.view(), layer, &property, link)?;
                }
                let mut source =
                    self.view()
                        .property_source(layer, &property)?
                        .unwrap_or(PropertySource {
                            base: None,
                            modulators: Vec::new(),
                        });
                source.modulators = modulators;
                let json = serde_json::to_string(&source)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_track(&property),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetCameraPropertyModulators {
                property,
                modulators,
            } => {
                let mut source =
                    self.view()
                        .camera_property_source(&property)?
                        .unwrap_or(PropertySource {
                            base: None,
                            modulators: Vec::new(),
                        });
                source.modulators = modulators;
                let json = serde_json::to_string(&source)?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_track(&property),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetSlots { slots } => {
                motolii_doc::store::slot::validate_unique_ids(&slots)?;
                let json = serde_json::to_string(&slots)?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_slots(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::AdmitAsset { draft } => {
                let mut table = self.view().assets_table()?;
                table
                    .admit(draft)
                    .map_err(|e| StoreError::Property(e.to_string()))?;
                let json = serde_json::to_string(&table)?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_assets(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::RemoveAsset { asset } => {
                let mut table = self.view().assets_table()?;
                table
                    .remove(asset)
                    .map_err(|e| StoreError::Property(e.to_string()))?;
                let json = serde_json::to_string(&table)?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_assets(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::RelinkAsset {
                asset,
                path_absolute,
                project_root,
            } => {
                let mut table = self.view().assets_table()?;
                let path = std::path::Path::new(&path_absolute);
                table
                    .relink(
                        asset,
                        path,
                        project_root.as_deref().map(std::path::Path::new),
                    )
                    .map_err(|e| StoreError::Property(e.to_string()))?;
                let json = serde_json::to_string(&table)?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_assets(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::Freeze { group } => {
                check_not_locked(&self.view(), group)?;
                check_not_frozen(&self.view(), group)?;
                freeze_attrs_batch(&self.view(), group, true)?
            }
            Intent::Unfreeze { group } => {
                check_not_locked(&self.view(), group)?;
                check_not_frozen(&self.view(), group)?;
                freeze_attrs_batch(&self.view(), group, false)?
            }
        };

        let (path, batches) = batches;
        self.ingest(path, batches, at)
    }
}

fn serialize_present(present: bool) -> Result<SerializedComponentBatch, StoreError> {
    Ok(SerializedComponentBatch {
        descriptor: descriptor_present(),
        array: <LayerPresent as re_types_core::Loggable>::to_arrow([LayerPresent(present)])
            .map_err(|e| StoreError::Chunk(e.to_string()))?,
    })
}

/// 動かない値 1 つ分の書き込み。**型のある口が在ればそちらへ**、無ければ従来の文字列へ。
/// 型を増やす時にここは変わらない(増えるのは `value_components` の対応表だけ)。
fn constant_batch(
    property: &motolii_doc::store::PropertyId,
    value: motolii_doc::eval::Value,
) -> Result<SerializedComponentBatch, StoreError> {
    if let Some(typed) = motolii_doc::store::value_components::constant_batch(property, &value) {
        return typed;
    }
    let json = serde_json::to_string(&PropertySource::constant(value))?;
    Ok(SerializedComponentBatch {
        descriptor: descriptor_track(property),
        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
            .map_err(|e| StoreError::Chunk(e.to_string()))?,
    })
}

#[cfg(test)]
mod typed_constant_tests {
    use super::*;
    use motolii_doc::eval::Value;
    use motolii_doc::store::{property, LayerId, PropertyId, RationalTime};

    /// 動かない値は型のまま置かれ、そのまま読み戻る(文字列を経由しない)。
    #[test]
    fn a_constant_round_trips_without_json() {
        let mut doc = Document::new();
        let layer = LayerId(7);
        let opacity = PropertyId::new(property::OPACITY).unwrap();
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetConstant { layer, property: opacity.clone(), value: Value::F64(0.25) },
        ])
        .unwrap();
        assert_eq!(doc.view().value_at(layer, &opacity, RationalTime::ZERO).unwrap(), Some(Value::F64(0.25)));

        // 型が変わる値へ上書きしても、新しい型が古い型を覆う。
        doc.apply(Intent::SetConstant { layer, property: opacity.clone(), value: Value::Bool(true) }).unwrap();
        assert_eq!(doc.view().value_at(layer, &opacity, RationalTime::ZERO).unwrap(), Some(Value::Bool(true)));

        // undo は edit 軸の巻き戻しなので、型付きでもそのまま効く。
        doc.undo();
        assert_eq!(doc.view().value_at(layer, &opacity, RationalTime::ZERO).unwrap(), Some(Value::F64(0.25)));
    }

    /// 文字列のままの値(古い書類・まだ型の無い Vec2)も読める。
    #[test]
    fn a_vec2_still_travels_as_text() {
        let mut doc = Document::new();
        let layer = LayerId(7);
        let position = PropertyId::new(property::POSITION).unwrap();
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetConstant { layer, property: position.clone(), value: Value::Vec2([12.5, -3.0]) },
        ])
        .unwrap();
        assert_eq!(
            doc.view().value_at(layer, &position, RationalTime::ZERO).unwrap(),
            Some(Value::Vec2([12.5, -3.0]))
        );
    }
}
