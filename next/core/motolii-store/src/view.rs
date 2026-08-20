//! 読み口 — front が受け取る唯一の物。可変な口を1つも持たない。

use motolii_core::RationalTime;
use motolii_eval::{KeyframeTrack, Value};
use re_chunk_store::LatestAtQuery;
use re_entity_db::EntityDb;
use re_log_types::{EntityPath, Timeline};

use crate::components::{descriptor_meta, descriptor_present, descriptor_track, LayerPresent, TrackJson};
use crate::{property, LayerId, LayerMeta, PropertyId, ResolvedLayer, EDIT_TIMELINE};

/// ある edit 時点の Document の姿。**query の投影であって、独自の状態を持たない**。
#[derive(Clone, Copy)]
pub struct StoreView<'a> {
    db: &'a EntityDb,
    at: i64,
}

impl<'a> StoreView<'a> {
    pub(crate) fn new(db: &'a EntityDb, at: i64) -> Self {
        Self { db, at }
    }

    fn query(&self) -> LatestAtQuery {
        LatestAtQuery::new(*Timeline::new_sequence(EDIT_TIMELINE).name(), self.at)
    }

    /// この時点で存在する layer。削除は墓標なので、ここで false を弾く。
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

    /// property の keyframe track。**評価はしない** — 生の意味をそのまま返す。
    pub fn track(&self, layer: LayerId, property: &PropertyId) -> Option<KeyframeTrack> {
        let descriptor = descriptor_track(property);
        let path = layer.entity_path();
        let results = self
            .db
            .latest_at(&self.query(), &path, [descriptor.component]);
        let json = results
            .component_batch::<TrackJson>(descriptor.component)?
            .into_iter()
            .next()?;
        serde_json::from_str(&json.0).ok()
    }

    /// comp 時刻の値。**補間の意味は `motolii-eval` が持つ**ので、ここは呼ぶだけ。
    pub fn value_at(
        &self,
        layer: LayerId,
        property: &PropertyId,
        t: RationalTime,
    ) -> Option<Value> {
        self.track(layer, property).map(|track| track.eval(t))
    }

    pub fn meta(&self, layer: LayerId) -> Option<LayerMeta> {
        let descriptor = descriptor_meta();
        let path = layer.entity_path();
        let results = self
            .db
            .latest_at(&self.query(), &path, [descriptor.component]);
        let json = results
            .component_batch::<TrackJson>(descriptor.component)?
            .into_iter()
            .next()?;
        serde_json::from_str(&json.0).ok()
    }

    /// comp 時刻での layer の姿。**合成器へ渡す唯一の形**。
    ///
    /// track が無い property は既定値になる(位置 0、不透明度 1、大きさは素材のまま)。
    /// これは AE で「キーを打っていない property は静止値」と同じ扱いである。
    pub fn resolve(&self, layer: LayerId, t: RationalTime) -> Option<ResolvedLayer> {
        let meta = self.meta(layer)?;
        let size = meta.source.size();

        let scalar = |name: &str, default: f32| -> f32 {
            let Ok(property) = PropertyId::new(name) else {
                return default;
            };
            match self.value_at(layer, &property, t) {
                Some(Value::F64(v)) => v as f32,
                _ => default,
            }
        };

        Some(ResolvedLayer {
            top_left: [
                scalar(property::POSITION_X, 0.0),
                scalar(property::POSITION_Y, 0.0),
            ],
            size: [
                scalar(property::WIDTH, size[0]),
                scalar(property::HEIGHT, size[1]),
            ],
            opacity: scalar(property::OPACITY, 1.0).clamp(0.0, 1.0),
            order: meta.order,
            source: meta.source,
        })
    }

    /// この時刻に描くべき layer を**奥から手前の順**で返す。
    pub fn resolved_layers(&self, t: RationalTime) -> Vec<ResolvedLayer> {
        let mut out: Vec<ResolvedLayer> = self
            .layers()
            .into_iter()
            .filter_map(|layer| self.resolve(layer, t))
            .collect();
        out.sort_by_key(|layer| layer.order);
        out
    }
}

fn layer_id_of(path: &EntityPath) -> Option<LayerId> {
    let s = path.to_string();
    s.strip_prefix("/layer/")
        .and_then(|rest| rest.parse::<u64>().ok())
        .map(LayerId)
}
