
mod resolve;

use std::cell::RefCell;
use std::collections::HashMap;

use motolii_core::RationalTime;
use motolii_eval::{KeyframeTrack, Value};
use re_chunk_store::LatestAtQuery;
use re_entity_db::EntityDb;
use re_log_types::{EntityPath, Timeline};

use crate::components::{
    descriptor_assets, descriptor_attrs, descriptor_composition, descriptor_effects,
    descriptor_markers, descriptor_masks, descriptor_meta, descriptor_present, descriptor_shapes,
    descriptor_slots, descriptor_text, descriptor_track, LayerPresent, TrackJson,
};
use crate::document::{TrackCache, TransientKey};
use crate::slot::{PropertyBase, PropertySource};
use crate::{
    Asset, AssetId, AssetTable, Composition, Document, EffectInstance, LayerAttrs, LayerId,
    LayerMeta, LayerSource, Marker, Mask, PropertyId, Revision, ShapeNode, Slot, SlotId,
    StoreError, TextDocument, EDIT_TIMELINE,
};

#[derive(Clone)]
pub struct StoreView<'a> {
    db: &'a EntityDb,
    at: i64,
    transient: &'a HashMap<TransientKey, Value>,
    revision: Revision,
    track_cache: &'a RefCell<TrackCache>,
}

const MAX_LINK_DEPTH: u32 = 64;

impl<'a> StoreView<'a> {
    pub(crate) fn new(
        db: &'a EntityDb,
        at: i64,
        transient: &'a HashMap<TransientKey, Value>,
        revision: Revision,
        track_cache: &'a RefCell<TrackCache>,
    ) -> Self {
        Self {
            db,
            at,
            transient,
            revision,
            track_cache,
        }
    }

    fn cache_key(path: &EntityPath, property: &PropertyId) -> Option<TransientKey> {
        if *path == Document::composition_path() {
            Some(TransientKey::Camera(property.clone()))
        } else {
            Some(TransientKey::Layer(layer_id_of(path)?, property.clone()))
        }
    }

    fn query(&self) -> LatestAtQuery {
        LatestAtQuery::new(*Timeline::new_sequence(EDIT_TIMELINE).name(), self.at)
    }

    pub fn layers(&self) -> Vec<LayerId> {
        let query = self.query();
        let mut out: Vec<LayerId> = self
            .db
            .sorted_entity_paths()
            .filter_map(|path| {
                let id = layer_id_of(path)?;
                let results = self
                    .db
                    .latest_at(&query, path, [descriptor_present().component]);
                let present = results
                    .component_batch::<LayerPresent>(descriptor_present().component)?
                    .first()
                    .copied()?;
                present.0.then_some(id)
            })
            .collect();
        out.sort();
        out
    }

    pub fn has_layer(&self, layer: LayerId) -> bool {
        self.layers().contains(&layer)
    }

    pub fn next_layer_id(&self) -> u64 {
        self.db
            .sorted_entity_paths()
            .filter_map(layer_id_of)
            .map(|id| id.0)
            .max()
            .map(|max| max + 1)
            .unwrap_or(1)
    }

    pub fn properties(&self, layer: LayerId) -> Vec<PropertyId> {
        let path = layer.entity_path();
        let engine = self.db.storage_engine();
        let Some(components) = engine.store().schema().all_components_for_entity(&path) else {
            return Vec::new();
        };
        let mut out: Vec<PropertyId> = components
            .iter()
            .filter_map(|component| {
                let name = component.as_str().strip_prefix("Layer:")?;
                if crate::property::RESERVED.contains(&name) {
                    return None;
                }
                PropertyId::new(name).ok()
            })
            .collect();
        out.sort();
        out
    }

    pub(crate) fn track_json_components(
        &self,
        path: &EntityPath,
    ) -> Result<Vec<(re_types_core::ComponentIdentifier, String)>, StoreError> {
        let engine = self.db.storage_engine();
        let Some(components) = engine.store().schema().all_components_for_entity(path) else {
            return Ok(Vec::new());
        };
        let query = self.query();
        let present = descriptor_present().component;
        let mut out: Vec<(re_types_core::ComponentIdentifier, String)> = Vec::new();
        for component in components.iter().copied().filter(|component| *component != present) {
            let results = self.db.latest_at(&query, path, [component]);
            if results.component_batch_raw(component).is_none() {
                continue;
            }
            let Some(json) = results
                .component_batch::<TrackJson>(component)
                .and_then(|batch| batch.into_iter().next())
            else {
                return Err(StoreError::Property(format!(
                    "component `{}` は値を持っているが `TrackJson` として読めない — \
                     `Layer:present` 以外の component は flattened()/save() が\
                     機械的に全部運ぶ前提なので、型が違う component が増えたらここで\
                     気付く必要がある(黙って保存から消してはいけない)",
                    component.as_str()
                )));
            };
            out.push((component, json.0));
        }
        out.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
        Ok(out)
    }

    fn source_at_path(
        &self,
        path: &EntityPath,
        property: &PropertyId,
    ) -> Result<Option<PropertySource>, StoreError> {
        let Some(key) = Self::cache_key(path, property) else {
            return self.parse_source_at_path(path, property);
        };
        self.track_cache.borrow_mut().get_or_try_insert_with(
            &self.revision,
            key,
            || self.parse_source_at_path(path, property),
        )
    }

    fn parse_source_at_path(
        &self,
        path: &EntityPath,
        property: &PropertyId,
    ) -> Result<Option<PropertySource>, StoreError> {
        let descriptor = descriptor_track(property);
        let results = self
            .db
            .latest_at(&self.query(), path, [descriptor.component]);
        let Some(json) = results
            .component_batch::<TrackJson>(descriptor.component)
            .and_then(|batch| batch.into_iter().next())
        else {
            return Ok(None);
        };
        serde_json::from_str(&json.0)
            .map(Some)
            .map_err(StoreError::Encode)
    }

    fn track_at_path(
        &self,
        path: &EntityPath,
        property: &PropertyId,
    ) -> Result<Option<KeyframeTrack>, StoreError> {
        Ok(match self.source_at_path(path, property)? {
            Some(PropertySource {
                base: Some(PropertyBase::Track(track)),
                ..
            }) => Some(track),
            _ => None,
        })
    }

    pub fn track(
        &self,
        layer: LayerId,
        property: &PropertyId,
    ) -> Result<Option<KeyframeTrack>, StoreError> {
        self.track_at_path(&layer.entity_path(), property)
    }

    pub fn camera_track(&self, property: &PropertyId) -> Result<Option<KeyframeTrack>, StoreError> {
        self.track_at_path(&Document::composition_path(), property)
    }

    pub fn property_source(
        &self,
        layer: LayerId,
        property: &PropertyId,
    ) -> Result<Option<PropertySource>, StoreError> {
        self.source_at_path(&layer.entity_path(), property)
    }

    pub fn camera_property_source(
        &self,
        property: &PropertyId,
    ) -> Result<Option<PropertySource>, StoreError> {
        self.source_at_path(&Document::composition_path(), property)
    }

    pub fn slots(&self) -> Result<Vec<Slot>, StoreError> {
        let descriptor = descriptor_slots();
        let results = self
            .db
            .latest_at(&self.query(), &Document::composition_path(), [descriptor.component]);
        let Some(json) = results
            .component_batch::<TrackJson>(descriptor.component)
            .and_then(|batch| batch.into_iter().next())
        else {
            return Ok(Vec::new());
        };
        serde_json::from_str(&json.0).map_err(StoreError::Encode)
    }

    fn slot_track(&self, id: &SlotId) -> Result<Option<KeyframeTrack>, StoreError> {
        Ok(self
            .slots()?
            .into_iter()
            .find(|slot| &slot.id == id)
            .map(|slot| slot.track))
    }

    fn value_at_path(
        &self,
        path: &EntityPath,
        property: &PropertyId,
        t: RationalTime,
    ) -> Result<Option<Value>, StoreError> {
        self.value_at_path_resolving_links(path, property, t, 0)
    }

    fn value_at_path_resolving_links(
        &self,
        path: &EntityPath,
        property: &PropertyId,
        t: RationalTime,
        link_depth: u32,
    ) -> Result<Option<Value>, StoreError> {
        if let Some(value) = self.transient_value_at(path, property) {
            return Ok(Some(value));
        }
        let Some(source) = self.source_at_path(path, property)? else {
            return Ok(None);
        };

        let mut acc: Option<Value> = match source.base {
            Some(PropertyBase::Track(track)) => Some(track.eval(t)),
            Some(PropertyBase::Slot(slot_id)) => {
                self.slot_track(&slot_id)?.map(|track| track.eval(t))
            }
            None => None,
        };

        for modulator in &source.modulators {
            if link_depth >= MAX_LINK_DEPTH {
                return Err(StoreError::Property(format!(
                    "link/modulator の参照鎖が深すぎる({MAX_LINK_DEPTH}段以上) — \
                     書き込み時の循環拒否をすり抜けた壊れた Document の可能性がある"
                )));
            }
            let source_t = t.try_add(modulator.time_offset).map_err(|e| {
                StoreError::Property(format!("modulator の time_offset を適用できない: {e}"))
            })?;
            let source_value = self.value_at_path_resolving_links(
                &modulator.source_layer.entity_path(),
                &modulator.source_property,
                source_t,
                link_depth + 1,
            )?;
            let Some(source_value) = source_value else {
                continue;
            };
            let Some(contribution) =
                crate::slot::translate_link(&modulator.plugin_id, &modulator.params, source_value)
            else {
                continue; // 型不一致・未知の plugin_id は近似せず寄与ゼロ。
            };
            acc = Some(match acc {
                Some(current) => current.add(&contribution).unwrap_or(current),
                None => contribution,
            });
        }

        Ok(acc)
    }

    fn transient_value_at(&self, path: &EntityPath, property: &PropertyId) -> Option<Value> {
        let key = if *path == Document::composition_path() {
            TransientKey::Camera(property.clone())
        } else {
            TransientKey::Layer(layer_id_of(path)?, property.clone())
        };
        self.transient.get(&key).cloned()
    }

    pub fn value_at(
        &self,
        layer: LayerId,
        property: &PropertyId,
        t: RationalTime,
    ) -> Result<Option<Value>, StoreError> {
        self.value_at_path(&layer.entity_path(), property, t)
    }

    pub fn camera_value_at(
        &self,
        property: &PropertyId,
        t: RationalTime,
    ) -> Result<Option<Value>, StoreError> {
        self.value_at_path(&Document::composition_path(), property, t)
    }

    pub fn composition(&self) -> Result<Option<Composition>, StoreError> {
        let descriptor = descriptor_composition();
        let path = Document::composition_path();
        let results = self
            .db
            .latest_at(&self.query(), &path, [descriptor.component]);
        let Some(json) = results
            .component_batch::<TrackJson>(descriptor.component)
            .and_then(|batch| batch.into_iter().next())
        else {
            return Ok(None);
        };
        serde_json::from_str(&json.0)
            .map(Some)
            .map_err(StoreError::Encode)
    }

    pub fn markers(&self) -> Result<Vec<Marker>, StoreError> {
        let descriptor = descriptor_markers();
        let path = Document::composition_path();
        let results = self
            .db
            .latest_at(&self.query(), &path, [descriptor.component]);
        let Some(json) = results
            .component_batch::<TrackJson>(descriptor.component)
            .and_then(|batch| batch.into_iter().next())
        else {
            return Ok(Vec::new());
        };
        serde_json::from_str(&json.0).map_err(StoreError::Encode)
    }

    pub(crate) fn assets_table(&self) -> Result<AssetTable, StoreError> {
        let descriptor = descriptor_assets();
        let path = Document::composition_path();
        let results = self
            .db
            .latest_at(&self.query(), &path, [descriptor.component]);
        let Some(json) = results
            .component_batch::<TrackJson>(descriptor.component)
            .and_then(|batch| batch.into_iter().next())
        else {
            return Ok(AssetTable::new());
        };
        serde_json::from_str(&json.0).map_err(StoreError::Encode)
    }

    pub fn assets(&self) -> Result<Vec<Asset>, StoreError> {
        Ok(self.assets_table()?.iter().cloned().collect())
    }

    pub fn asset(&self, id: AssetId) -> Result<Option<Asset>, StoreError> {
        Ok(self.assets_table()?.get(id).cloned())
    }

    pub fn meta(&self, layer: LayerId) -> Result<Option<LayerMeta>, StoreError> {
        let descriptor = descriptor_meta();
        let path = layer.entity_path();
        let results = self
            .db
            .latest_at(&self.query(), &path, [descriptor.component]);
        let Some(json) = results
            .component_batch::<TrackJson>(descriptor.component)
            .and_then(|batch| batch.into_iter().next())
        else {
            return Ok(None);
        };
        serde_json::from_str(&json.0)
            .map(Some)
            .map_err(StoreError::Encode)
    }

    pub fn masks(&self, layer: LayerId) -> Result<Vec<Mask>, StoreError> {
        let descriptor = descriptor_masks();
        let path = layer.entity_path();
        let results = self
            .db
            .latest_at(&self.query(), &path, [descriptor.component]);
        let Some(json) = results
            .component_batch::<TrackJson>(descriptor.component)
            .and_then(|batch| batch.into_iter().next())
        else {
            return Ok(Vec::new());
        };
        serde_json::from_str(&json.0).map_err(StoreError::Encode)
    }

    pub fn attrs(&self, layer: LayerId) -> Result<Option<LayerAttrs>, StoreError> {
        let descriptor = descriptor_attrs();
        let path = layer.entity_path();
        let results = self
            .db
            .latest_at(&self.query(), &path, [descriptor.component]);
        let Some(json) = results
            .component_batch::<TrackJson>(descriptor.component)
            .and_then(|batch| batch.into_iter().next())
        else {
            return Ok(None);
        };
        serde_json::from_str(&json.0)
            .map(Some)
            .map_err(StoreError::Encode)
    }

    pub fn frozen_ancestor(&self, layer: LayerId) -> Result<Option<LayerId>, StoreError> {
        let mut current = self.attrs(layer)?.and_then(|attrs| attrs.parent);
        let mut seen = std::collections::HashSet::new();
        while let Some(ancestor) = current {
            if !seen.insert(ancestor) {
                break;
            }
            let is_group = self
                .meta(ancestor)?
                .map(|meta| meta.source == LayerSource::Group)
                .unwrap_or(false);
            if is_group && self.attrs(ancestor)?.unwrap_or_default().frozen {
                return Ok(Some(ancestor));
            }
            current = self.attrs(ancestor)?.and_then(|attrs| attrs.parent);
        }
        Ok(None)
    }

    pub fn effects(&self, layer: LayerId) -> Result<Vec<EffectInstance>, StoreError> {
        let descriptor = descriptor_effects();
        let path = layer.entity_path();
        let results = self
            .db
            .latest_at(&self.query(), &path, [descriptor.component]);
        let Some(json) = results
            .component_batch::<TrackJson>(descriptor.component)
            .and_then(|batch| batch.into_iter().next())
        else {
            return Ok(Vec::new());
        };
        serde_json::from_str(&json.0).map_err(StoreError::Encode)
    }

    pub fn shapes(&self, layer: LayerId) -> Result<Vec<ShapeNode>, StoreError> {
        let descriptor = descriptor_shapes();
        let path = layer.entity_path();
        let results = self
            .db
            .latest_at(&self.query(), &path, [descriptor.component]);
        let Some(json) = results
            .component_batch::<TrackJson>(descriptor.component)
            .and_then(|batch| batch.into_iter().next())
        else {
            return Ok(Vec::new());
        };
        serde_json::from_str(&json.0).map_err(StoreError::Encode)
    }

    pub fn text_document(&self, layer: LayerId) -> Result<Option<TextDocument>, StoreError> {
        let descriptor = descriptor_text();
        let path = layer.entity_path();
        let results = self
            .db
            .latest_at(&self.query(), &path, [descriptor.component]);
        let Some(json) = results
            .component_batch::<TrackJson>(descriptor.component)
            .and_then(|batch| batch.into_iter().next())
        else {
            return Ok(None);
        };
        serde_json::from_str(&json.0)
            .map(Some)
            .map_err(StoreError::Encode)
    }
}

fn layer_id_of(path: &EntityPath) -> Option<LayerId> {
    let s = path.to_string();
    s.strip_prefix("/layer/")
        .and_then(|rest| rest.parse::<u64>().ok())
        .map(LayerId)
}
