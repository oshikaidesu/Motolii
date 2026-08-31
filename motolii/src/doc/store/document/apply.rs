
use re_types_core::SerializedComponentBatch;

use crate::doc::store::components::{
    descriptor_assets, descriptor_attrs, descriptor_composition, descriptor_effects,
    descriptor_markers, descriptor_masks, descriptor_meta, descriptor_present, descriptor_shapes,
    descriptor_slots, descriptor_text, descriptor_track, LayerPresent, TrackJson,
};
use crate::doc::store::slot::PropertySource;
use crate::doc::store::StoreError;

use super::validate::{
    check_not_frozen, check_not_locked, freeze_attrs_batch, is_frozen_or_within_frozen,
    validate_masks_have_shapes, validate_no_link_cycle, validate_no_parent_cycle,
};
use super::{Document, Intent};

impl Document {
    pub(crate) fn write(&mut self, intent: Intent, at: i64) -> Result<(), StoreError> {
        let batches = match intent {
            Intent::AddLayer(layer) => (layer.entity_path(), vec![serialize_present(true)?]),
            Intent::RemoveLayer(layer) => {
                check_not_locked(&self.view(), layer)?;
                check_not_frozen(&self.view(), layer)?;
                (layer.entity_path(), vec![serialize_present(false)?])
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
                check_not_frozen(&self.view(), layer)?;
                crate::doc::store::mask::validate_unique_ids(&masks)?;
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
                check_not_frozen(&self.view(), layer)?;
                let mut masks = self.view().masks(layer)?;
                masks.push(mask);
                crate::doc::store::mask::validate_unique_ids(&masks)?;
                let masks_json = serde_json::to_string(&masks)?;
                let shape_property = crate::doc::store::PropertyId::mask_shape(mask.id);
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
                check_not_frozen(&self.view(), layer)?;
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
                if current.locked {
                    let touches_other_than_locked = patch.hidden.is_some()
                        || patch.parent.is_some()
                        || patch.blend_mode.is_some()
                        || patch.matte.is_some()
                        || patch.name.is_some()
                        || patch.auto_orient.is_some()
                        || patch.pinned.is_some()
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
                check_not_frozen(&self.view(), layer)?;
                crate::doc::store::effect::validate_unique_ids(&effects)?;
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
                check_not_frozen(&self.view(), layer)?;
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
                check_not_frozen(&self.view(), layer)?;
                crate::doc::store::text::validate(&document)?;
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
                let json = serde_json::to_string(&PropertySource::constant(value))?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_track(&property),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetCameraTrack { property, track } => {
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
                let mut source = self
                    .view()
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
                let mut source = self
                    .view()
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
                crate::doc::store::slot::validate_unique_ids(&slots)?;
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
