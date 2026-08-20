//! 読み口 — front が受け取る唯一の物。可変な口を1つも持たない。

use motolii_core::RationalTime;
use motolii_eval::{KeyframeTrack, Value};
use re_chunk_store::LatestAtQuery;
use re_entity_db::EntityDb;
use re_log_types::{EntityPath, Timeline};

use crate::components::{descriptor_composition, descriptor_meta, descriptor_present, descriptor_track, LayerPresent, TrackJson};
use crate::{property, Composition, Document, LayerId, LayerMeta, LayerPlacement, PropertyId, ResolvedLayer, StoreError, EDIT_TIMELINE};

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

    /// この layer が持っている property の名前。
    ///
    /// **store に聞く**(`all_components_for_entity`)。Document 側に「property の一覧」を
    /// 別に持つと、実体とずれた台帳がもう1つ生まれる。
    /// Inspector が行を並べる時もここを使う。
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
                // layer 自身の component は property ではない。
                if crate::property::RESERVED.contains(&name) {
                    return None;
                }
                PropertyId::new(name).ok()
            })
            .collect();
        out.sort();
        out
    }

    /// property の keyframe track。**評価はしない** — 生の意味をそのまま返す。
    /// `Ok(None)` = **その property に track が無い**。
    /// `Err` = **track はあるが読めない**。この2つを同義にしない — 同義にすると
    /// 壊れた Document が静かに既定値へ落ち、利用者には「値が勝手に戻った」としか
    /// 見えない(M13: 無反応ゼロ / 拒否は理由が分かる)。
    pub fn track(
        &self,
        layer: LayerId,
        property: &PropertyId,
    ) -> Result<Option<KeyframeTrack>, StoreError> {
        let descriptor = descriptor_track(property);
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

    /// comp 時刻の値。**補間の意味は `motolii-eval` が持つ**ので、ここは呼ぶだけ。
    pub fn value_at(
        &self,
        layer: LayerId,
        property: &PropertyId,
        t: RationalTime,
    ) -> Result<Option<Value>, StoreError> {
        Ok(self.track(layer, property)?.map(|track| track.eval(t)))
    }

    /// comp の設定。**preview も export もここから取る** — 引数で渡さない。
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

    /// comp 時刻での layer の姿。**合成器へ渡す唯一の形**。
    ///
    /// track が無い property は既定値になる(位置 0、不透明度 1、大きさは素材のまま)。
    /// これは AE で「キーを打っていない property は静止値」と同じ扱いである。
    /// `Ok(None)` = **この時刻にこの layer は居ない**(配置の外)。
    pub fn resolve(
        &self,
        layer: LayerId,
        t: RationalTime,
    ) -> Result<Option<ResolvedLayer>, StoreError> {
        let Some(meta) = self.meta(layer)? else {
            return Ok(None);
        };

        // 時間の判定は Document がする。engine は解決済みの素材フレームを受け取るだけ。
        let Some(composition) = self.composition()? else {
            return Ok(None);
        };
        let comp_frame = t
            .try_to_frame_floor(composition.fps)
            .map_err(|e| StoreError::Property(e.to_string()))?;
        let Some(source_frame) = meta.timing.source_frame(comp_frame) else {
            return Ok(None);
        };
        // 実素材の大きさは probe しないと分からない。ここでは 0 を置き、engine が
        // 「track が無く declared も無い」場合だけ素材の実寸で埋める。
        let size = meta.source.declared_size().unwrap_or([0.0, 0.0]);

        // 標準 property は必ず構築できる(予約語でない・空でない)ので、
        // ここで失敗したら**コードの誤り**であって Document の内容ではない。
        let scalar = |name: &str, default: f32| -> Result<f32, StoreError> {
            let property = PropertyId::new(name)?;
            match self.value_at(layer, &property, t)? {
                Some(Value::F64(v)) => Ok(v as f32),
                // track はあるが型が違う。既定値へ落とすと「打ったキーが効かない」に見える。
                Some(other) => Err(StoreError::Property(format!(
                    "{name} に数値でない値が入っている: {other:?}"
                ))),
                None => Ok(default),
            }
        };

        Ok(Some(ResolvedLayer {
            placement: LayerPlacement {
                top_left: [
                    scalar(property::POSITION_X, 0.0)?,
                    scalar(property::POSITION_Y, 0.0)?,
                ],
                size: [
                    scalar(property::WIDTH, size[0])?,
                    scalar(property::HEIGHT, size[1])?,
                ],
                opacity: scalar(property::OPACITY, 1.0)?.clamp(0.0, 1.0),
                order: meta.order,
            },
            source: meta.source,
            source_frame,
        }))
    }

    /// この時刻に描くべき layer を**奥から手前の順**で返す。
    pub fn resolved_layers(&self, t: RationalTime) -> Result<Vec<ResolvedLayer>, StoreError> {
        let mut out = Vec::new();
        for layer in self.layers() {
            if let Some(resolved) = self.resolve(layer, t)? {
                out.push(resolved);
            }
        }
        out.sort_by_key(|layer| layer.placement.order);
        Ok(out)
    }
}

fn layer_id_of(path: &EntityPath) -> Option<LayerId> {
    let s = path.to_string();
    s.strip_prefix("/layer/")
        .and_then(|rest| rest.parse::<u64>().ok())
        .map(LayerId)
}
