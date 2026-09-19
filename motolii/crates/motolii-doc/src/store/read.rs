//! Read-side caches and projected preview values. No editing commands live here.
use super::{
    LayerAttrs, LayerId, LayerTiming, PropertyId, PropertySource, ShapeNode,
    StoreError, TextDocument,
};
use re_log_types::EntityPath;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Revision {
    pub(crate) store: re_chunk_store::ChunkStoreGeneration,
    pub(crate) head: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayRevision {
    pub(crate) revision: Revision,
    pub(crate) transient_generation: u64,
}

pub(crate) fn composition_path() -> EntityPath {
    EntityPath::from("/composition")
}

#[derive(Default)]
pub(crate) struct ReadOverlay {
    pub(crate) sources: HashMap<TransientKey, PropertySource>,
    pub(crate) timings: HashMap<LayerId, LayerTiming>,
    pub(crate) attrs: HashMap<LayerId, LayerAttrs>,
    pub(crate) shapes: HashMap<LayerId, Vec<ShapeNode>>,
    pub(crate) texts: HashMap<LayerId, TextDocument>,
    pub(crate) count: usize,
    pub(crate) generation: u64,
}
impl ReadOverlay {
    pub(crate) fn is_empty(&self) -> bool {
        self.count == 0
    }
    pub(crate) fn clear(&mut self) {
        *self = Self::default();
    }
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
    pub(crate) clipping: Option<HashMap<LayerId, Option<LayerId>>>,
}

impl RecordCache {
    pub(crate) fn sync(&mut self, current: &Revision) {
        if self.revision.as_ref() != Some(current) {
            self.attrs.clear();
            self.meta.clear();
            self.clipping = None;
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
