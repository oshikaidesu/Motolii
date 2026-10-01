//! Read-side caches and projected preview values. No editing commands live here.
use super::{
    LayerAttrs, LayerId, LayerTiming, PropertyId, PropertySource, ShapeNode,
    StoreError, TextDocument,
};
use re_log_types::EntityPath;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Revision {
    pub store: re_chunk_store::ChunkStoreGeneration,
    pub head: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayRevision {
    pub revision: Revision,
    pub transient_generation: u64,
}

pub fn composition_path() -> EntityPath {
    EntityPath::from("/composition")
}

#[derive(Default)]
pub struct ReadOverlay {
    pub sources: HashMap<TransientKey, PropertySource>,
    pub timings: HashMap<LayerId, LayerTiming>,
    pub attrs: HashMap<LayerId, LayerAttrs>,
    pub shapes: HashMap<LayerId, Vec<ShapeNode>>,
    pub texts: HashMap<LayerId, TextDocument>,
    pub count: usize,
    pub generation: u64,
}
impl ReadOverlay {
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TransientKey {
    Layer(LayerId, PropertyId),
    Camera(PropertyId),
}

/// 層の attrs / meta の cache。毎回 latest_at + serde_json では、行を組む度に層²回の parse になる。
/// revision が動けば丸ごと捨てる(TrackCache と同じ流儀)。
#[derive(Default)]
pub struct RecordCache {
    revision: Option<Revision>,
    pub attrs: HashMap<LayerId, Option<super::LayerAttrs>>,
    pub meta: HashMap<LayerId, Option<super::LayerMeta>>,
    pub clipping: Option<HashMap<LayerId, Option<LayerId>>>,
    /// 書類の寸法・fps・尺。track で持つので時刻も鍵に要るが、**1 コマの中では不変**。
    /// 毎回引くと latest_at + JSON parse が 1 コマに千回単位で走る(2026-09-21 の標本で 16%)。
    pub composition: Option<(i64, Option<super::Composition>)>,
    /// 居る層の一覧。書類の entity path を全部舐めて latest_at するので、1 コマ 1 回に畳む。
    pub layers: Option<(i64, std::sync::Arc<Vec<LayerId>>)>,
}

impl RecordCache {
    pub fn sync(&mut self, current: &Revision) {
        if self.revision.as_ref() != Some(current) {
            self.attrs.clear();
            self.meta.clear();
            self.clipping = None;
            self.composition = None;
            self.layers = None;
            self.revision = Some(current.clone());
        }
    }
}

#[derive(Default)]
pub struct TrackCache {
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

    pub fn get_or_try_insert_with(
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
