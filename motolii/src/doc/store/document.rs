
mod apply;
mod group;
mod ids;
mod validate;

pub use ids::{LayerId, PropertyId};

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use re_chunk::{Chunk, RowId};
use re_entity_db::EntityDb;
use re_log_types::{
    AbsoluteTimeRange, EntityPath, StoreId, StoreKind, TimePoint, Timeline, TimelineName,
};
use re_types_core::{Component, SerializedComponentBatch};

use crate::doc::eval::Value;

#[cfg(test)]
use crate::doc::store::components::LayerPresent;
use crate::doc::store::components::TrackJson;
use crate::doc::store::slot::{PropertyLink, PropertySource};
use crate::doc::store::view::StoreView;
use crate::doc::store::{LayerAttrsPatch, Mask, Slot, SlotId, StoreError, EDIT_TIMELINE};

#[derive(Clone, Debug)]
pub enum Intent {
    AddLayer(LayerId),
    RemoveLayer(LayerId),
    SetTrack {
        layer: LayerId,
        property: PropertyId,
        track: crate::doc::eval::KeyframeTrack,
    },
    /// **動かない値**を置く。キーは作らない —— 利用者が ◇ を押すまで
    /// 時間の世界へ入れない(根底3)。
    SetConstant {
        layer: LayerId,
        property: PropertyId,
        value: crate::doc::eval::Value,
    },
    SetPropertySlot {
        layer: LayerId,
        property: PropertyId,
        slot: SlotId,
    },
    SetPropertyLink {
        layer: LayerId,
        property: PropertyId,
        link: PropertyLink,
    },
    SetPropertyModulators {
        layer: LayerId,
        property: PropertyId,
        modulators: Vec<PropertyLink>,
    },
    SetCameraPropertyModulators {
        property: PropertyId,
        modulators: Vec<PropertyLink>,
    },
    SetMeta {
        layer: LayerId,
        meta: crate::doc::store::LayerMeta,
    },
    SetSource {
        layer: LayerId,
        source: crate::doc::store::LayerSource,
    },
    SetOrder { layer: LayerId, order: i16 },
    SetMasks {
        layer: LayerId,
        masks: Vec<crate::doc::store::Mask>,
    },
    AddMask {
        layer: LayerId,
        mask: Mask,
        shape: crate::doc::eval::KeyframeTrack,
    },
    SetTiming {
        layer: LayerId,
        timing: crate::doc::store::LayerTiming,
    },
    SetAttrs {
        layer: LayerId,
        patch: LayerAttrsPatch,
    },
    SetEffects {
        layer: LayerId,
        effects: Vec<crate::doc::store::EffectInstance>,
    },
    SetShapes {
        layer: LayerId,
        shapes: Vec<crate::doc::store::ShapeNode>,
    },
    SetTextDocument {
        layer: LayerId,
        document: crate::doc::store::TextDocument,
    },
    SetComposition(crate::doc::store::Composition),
    SetMarkers { markers: Vec<crate::doc::store::Marker> },
    SetCameraTrack {
        property: PropertyId,
        track: crate::doc::eval::KeyframeTrack,
    },
    SetCameraPropertySlot {
        property: PropertyId,
        slot: SlotId,
    },
    SetSlots {
        slots: Vec<Slot>,
    },
    AdmitAsset {
        draft: crate::doc::store::AssetDraft,
    },
    RemoveAsset {
        asset: crate::doc::store::AssetId,
    },
    RelinkAsset {
        asset: crate::doc::store::AssetId,
        path_absolute: String,
        project_root: Option<String>,
    },
    Freeze {
        group: LayerId,
    },
    Unfreeze {
        group: LayerId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Revision {
    store: re_chunk_store::ChunkStoreGeneration,
    head: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayRevision {
    revision: Revision,
    transient_generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum TransientKey {
    Layer(LayerId, PropertyId),
    Camera(PropertyId),
}

#[derive(Default)]
pub(crate) struct TrackCache {
    revision: Option<Revision>,
    entries: HashMap<TransientKey, Option<PropertySource>>,
}

impl TrackCache {
    fn sync(&mut self, current: &Revision) {
        if self.revision.as_ref() != Some(current) {
            self.entries.clear();
            self.revision = Some(current.clone());
        }
    }

    pub(crate) fn get_or_try_insert_with(
        &mut self,
        current: &Revision,
        key: TransientKey,
        miss: impl FnOnce() -> Result<Option<PropertySource>, StoreError>,
    ) -> Result<Option<PropertySource>, StoreError> {
        self.sync(current);
        if let Some(cached) = self.entries.get(&key) {
            return Ok(cached.clone());
        }
        let value = miss()?;
        self.entries.insert(key, value.clone());
        Ok(value)
    }
}

pub struct Document {
    pub(crate) db: EntityDb,
    head: i64,
    tip: i64,
    floor: i64,
    transient: HashMap<TransientKey, Value>,
    transient_generation: u64,
    track_cache: RefCell<TrackCache>,
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Document {
    pub fn new() -> Self {
        Self::with_store_id(StoreId::random(StoreKind::Recording, "motolii"))
    }

    pub(crate) fn with_store_id(store_id: StoreId) -> Self {
        Self {
            db: EntityDb::new(store_id),
            head: 0,
            tip: 0,
            floor: 0,
            transient: HashMap::new(),
            transient_generation: 0,
            track_cache: RefCell::new(TrackCache::default()),
        }
    }

    pub(crate) fn composition_path() -> EntityPath {
        EntityPath::from("/composition")
    }

    fn timeline() -> Timeline {
        Timeline::new_sequence(EDIT_TIMELINE)
    }

    fn timeline_name() -> TimelineName {
        *Self::timeline().name()
    }

    pub fn view(&self) -> StoreView<'_> {
        StoreView::new(
            &self.db,
            self.head,
            &self.transient,
            self.revision(),
            &self.track_cache,
        )
    }

    pub fn edit_head(&self) -> i64 {
        self.head
    }

    pub(crate) fn rebuild_head_from_store(&mut self) {
        let head = self
            .db
            .time_range_for(&Self::timeline_name())
            .map(|range| range.max().as_i64())
            .unwrap_or(0);
        self.head = head;
        self.tip = head;
        self.floor = head;
    }

    pub fn mark_undo_floor(&mut self) {
        self.floor = self.head;
    }

    pub fn can_undo(&self) -> bool {
        self.head > self.floor
    }

    pub fn can_redo(&self) -> bool {
        self.head < self.tip
    }

    pub fn undo(&mut self) -> bool {
        if self.can_undo() {
            self.head -= 1;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if self.can_redo() {
            self.head += 1;
            true
        } else {
            false
        }
    }

    pub fn apply_all(
        &mut self,
        intents: impl IntoIterator<Item = Intent>,
    ) -> Result<(), StoreError> {
        let intents: Vec<Intent> = intents.into_iter().collect();
        if intents.is_empty() {
            return Ok(());
        }

        self.drop_redo_space();
        let original_head = self.head;
        let original_tip = self.tip;
        let at = self.head + 1;
        for intent in intents {
            if let Err(error) = self.write(intent, at) {
                self.discard_batch_at(at, original_head, original_tip);
                return Err(error);
            }
        }
        self.head = at;
        self.tip = at;
        Ok(())
    }

    fn discard_batch_at(&mut self, at: i64, original_head: i64, original_tip: i64) {
        self.db.drop_time_range(
            &Self::timeline_name(),
            AbsoluteTimeRange::new(at, at),
            re_chunk_store::ChunkDeletionReason::ExplicitDrop,
        );
        self.head = original_head;
        self.tip = original_tip;
    }

    pub fn apply(&mut self, intent: Intent) -> Result<(), StoreError> {
        self.apply_all([intent])
    }

    fn drop_redo_space(&mut self) {
        if self.head < self.tip {
            self.db.drop_time_range(
                &Self::timeline_name(),
                AbsoluteTimeRange::new(self.head + 1, self.tip),
                re_chunk_store::ChunkDeletionReason::ExplicitDrop,
            );
            self.tip = self.head;
        }
    }

    fn ingest(
        &mut self,
        path: EntityPath,
        batches: Vec<SerializedComponentBatch>,
        at: i64,
    ) -> Result<(), StoreError> {
        let chunk = Chunk::builder(path)
            .with_serialized_batches(
                RowId::new(),
                TimePoint::default().with(Self::timeline(), at),
                batches,
            )
            .build()
            .map_err(|e| StoreError::Chunk(e.to_string()))?;

        self.db
            .add_chunk(&Arc::new(chunk))
            .map_err(|e| StoreError::Ingest(e.to_string()))?;

        self.head = at;
        self.tip = at;
        Ok(())
    }

    pub(crate) fn copy_track_json(
        &mut self,
        path: EntityPath,
        component: re_types_core::ComponentIdentifier,
        archetype: &'static str,
        json: String,
        at: i64,
    ) -> Result<(), StoreError> {
        let descriptor = re_types_core::ComponentDescriptor {
            archetype: Some(archetype.into()),
            component,
            component_type: Some(TrackJson::name()),
        };
        let batch = SerializedComponentBatch {
            descriptor,
            array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                .map_err(|e| StoreError::Chunk(e.to_string()))?,
        };
        self.ingest(path, vec![batch], at)
    }

    pub fn revision(&self) -> Revision {
        Revision {
            store: self.db.generation(),
            head: self.head,
        }
    }

    pub fn display_revision(&self) -> DisplayRevision {
        DisplayRevision {
            revision: self.revision(),
            transient_generation: self.transient_generation,
        }
    }

    pub fn set_transient(&mut self, layer: LayerId, property: PropertyId, value: Value) {
        self.transient
            .insert(TransientKey::Layer(layer, property), value);
        self.bump_transient_generation();
    }

    pub fn set_camera_transient(&mut self, property: PropertyId, value: Value) {
        self.transient.insert(TransientKey::Camera(property), value);
        self.bump_transient_generation();
    }

    pub fn clear_transient(&mut self, layer: LayerId, property: &PropertyId) {
        if self
            .transient
            .remove(&TransientKey::Layer(layer, property.clone()))
            .is_some()
        {
            self.bump_transient_generation();
        }
    }

    pub fn clear_camera_transient(&mut self, property: &PropertyId) {
        if self
            .transient
            .remove(&TransientKey::Camera(property.clone()))
            .is_some()
        {
            self.bump_transient_generation();
        }
    }

    pub fn clear_all_transients(&mut self) {
        if !self.transient.is_empty() {
            self.transient.clear();
            self.bump_transient_generation();
        }
    }

    fn bump_transient_generation(&mut self) {
        self.transient_generation = self.transient_generation.wrapping_add(1);
    }

    pub fn store_bytes(&self) -> u64 {
        self.db.byte_size_of_physical_chunks()
    }

    pub fn store_chunks(&self) -> usize {
        self.db.num_physical_chunks()
    }
}
