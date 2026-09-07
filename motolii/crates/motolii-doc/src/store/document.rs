mod apply;
mod edit;
mod group;
mod projection;
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
    SetOrder {
        layer: LayerId,
        order: i16,
    },
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
    SetNotebook { notebook: crate::doc::store::Notebook },
    SetMarkers {
        markers: Vec<crate::doc::store::Marker>,
    },
    /// カメラの属性へ**素の値**を置く。層側の `SetConstant` と同じ意味で、
    /// 置き場が composition なだけ。
    SetCameraConstant {
        property: PropertyId,
        value: Value,
    },
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

/// 層の attrs / meta の cache。毎回 latest_at + serde_json では、行を組む度に層²回の parse になる。
/// revision が動けば丸ごと捨てる(TrackCache と同じ流儀)。
#[derive(Default)]
pub(crate) struct RecordCache {
    revision: Option<Revision>,
    pub(crate) attrs: HashMap<LayerId, Option<super::LayerAttrs>>,
    pub(crate) meta: HashMap<LayerId, Option<super::LayerMeta>>,
}

impl RecordCache {
    pub(crate) fn sync(&mut self, current: &Revision) {
        if self.revision.as_ref() != Some(current) {
            self.attrs.clear();
            self.meta.clear();
            self.revision = Some(current.clone());
        }
    }
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
    preview_edits: Vec<Intent>,
    preview_owner: u64,
    track_cache: RefCell<TrackCache>,
    record_cache: RefCell<RecordCache>,
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
            preview_edits: Vec::new(),
            preview_owner: 0,
            track_cache: RefCell::new(TrackCache::default()),
            record_cache: RefCell::new(RecordCache::default()),
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
            &self.preview_edits,
            self.revision(),
            &self.track_cache,
            &self.record_cache,
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

    /// 戻れる段数と進める段数。履歴を一覧にする側はこれだけ読む。
    pub fn history_depth(&self) -> (usize, usize) {
        (
            (self.head - self.floor).max(0) as usize,
            (self.tip - self.head).max(0) as usize,
        )
    }

    pub fn can_redo(&self) -> bool {
        self.head < self.tip
    }

    pub fn undo(&mut self) -> bool {
        if self.can_undo() {
            self.head -= 1;
            self.clear_all_transients();
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if self.can_redo() {
            self.head += 1;
            self.clear_all_transients();
            true
        } else {
            false
        }
    }

    pub fn apply_all(
        &mut self,
        intents: impl IntoIterator<Item = Intent>,
    ) -> Result<(), StoreError> {
        self.apply_then(intents, |_| Ok(Vec::new()))
    }

    /// 1 手で 2 段。先の intents を書き、**その結果の view** で次の intents を決めて、
    /// 同じ履歴の段に書く。「作ってから並べる」のように後段が前段の結果を見ないと
    /// 決まらない編集を Undo 一発にする。どちらかが失敗すれば両方とも残らない。
    pub fn apply_then(
        &mut self,
        first: impl IntoIterator<Item = Intent>,
        then: impl FnOnce(&Self) -> Result<Vec<Intent>, StoreError>,
    ) -> Result<(), StoreError> {
        let intents: Vec<Intent> = first.into_iter().collect();
        if intents.is_empty() {
            return Ok(());
        }

        let original_head = self.head;
        let original_tip = self.tip;
        // Rerun's chunk-sharing snapshot keeps the redo branch intact until the edit succeeds.
        let original_db = if self.can_redo() {
            let staged = self
                .db
                .clone_with_new_id(self.db.store_id().clone())
                .map_err(|error| StoreError::Ingest(error.to_string()))?;
            Some(std::mem::replace(&mut self.db, staged))
        } else {
            None
        };
        let transient = std::mem::take(&mut self.transient);
        let preview = std::mem::take(&mut self.preview_edits);
        self.drop_redo_space();
        let at = self.head + 1;
        if let Err(error) = self.write_staged(at, intents, then) {
            {
                if let Some(original) = original_db {
                    self.db = original;
                    self.head = original_head;
                    self.tip = original_tip;
                } else {
                    self.discard_batch_at(at, original_head, original_tip);
                }
                *self.track_cache.get_mut() = TrackCache::default();
                *self.record_cache.get_mut() = RecordCache::default();
                self.transient = transient;
                self.preview_edits = preview;
                return Err(error);
            }
        }
        self.head = at;
        self.tip = at;
        self.clear_all_transients();
        Ok(())
    }

    /// 段の中身。先を書き、head を仮に進めて `then` に見せ、返った物も同じ段へ書く。
    fn write_staged(
        &mut self,
        at: i64,
        first: Vec<Intent>,
        then: impl FnOnce(&Self) -> Result<Vec<Intent>, StoreError>,
    ) -> Result<(), StoreError> {
        for intent in first {
            self.write(intent, at)?;
        }
        let head = self.head;
        self.head = at;
        let more = then(self);
        self.head = head;
        for intent in more? {
            self.write(intent, at)?;
        }
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

    /// 属性へ値を置く。**キーが生まれる規則はここ1つ**。
    ///
    /// 時間の世界がまだ開いていなければ素の値のまま。開いていれば、
    /// **いま居る時刻**へ打つ。利用者が ◇ を押すまでキーは生まれない。
    pub fn place(
        &self,
        layer: LayerId,
        property: &PropertyId,
        value: Value,
        t: crate::doc::store::RationalTime,
    ) -> Intent {
        match self.view().track(layer, property).ok().flatten() {
            None => Intent::SetConstant {
                layer,
                property: property.clone(),
                value,
            },
            Some(mut track) => {
                track.insert(crate::doc::store::Keyframe {
                    t,
                    value,
                    interp: crate::doc::store::Interp::Linear,
                    spatial: None,
                });
                Intent::SetTrack {
                    layer,
                    property: property.clone(),
                    track,
                }
            }
        }
    }

    /// カメラ側の同じ規則。層を持たないので口が別なだけで、意味は同じ。
    pub fn place_camera(
        &self,
        property: &PropertyId,
        value: Value,
        t: crate::doc::store::RationalTime,
    ) -> Intent {
        match self.view().camera_track(property).ok().flatten() {
            None => Intent::SetCameraConstant {
                property: property.clone(),
                value,
            },
            Some(mut track) => {
                track.insert(crate::doc::store::Keyframe {
                    t,
                    value,
                    interp: crate::doc::store::Interp::Linear,
                    spatial: None,
                });
                Intent::SetCameraTrack {
                    property: property.clone(),
                    track,
                }
            }
        }
    }

    pub fn identity(&self) -> String {
        format!("{:?}", self.db.store_id())
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
        self.transient.clear();
        self.preview_edits.clear();
        self.preview_owner = self.preview_owner.wrapping_add(1).max(1);
        self.bump_transient_generation();
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
