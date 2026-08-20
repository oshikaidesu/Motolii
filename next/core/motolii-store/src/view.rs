//! 読み口 — front が受け取る唯一の物。可変な口を1つも持たない。

use motolii_core::RationalTime;
use motolii_eval::{KeyframeTrack, Value};
use re_chunk_store::LatestAtQuery;
use re_entity_db::EntityDb;
use re_log_types::{EntityPath, Timeline};

use crate::components::{
    descriptor_attrs, descriptor_composition, descriptor_effects, descriptor_markers,
    descriptor_masks, descriptor_meta, descriptor_present, descriptor_shapes, descriptor_slots,
    descriptor_text, descriptor_track, LayerPresent, TrackJson,
};
use crate::slot::PropertySource;
use crate::{
    property, Composition, Document, EffectInstance, LayerAttrs, LayerId, LayerMeta,
    LayerPlacement, Marker, Mask, PropertyId, ResolvedLayer, ResolvedMask, Shape, Slot, SlotId,
    StoreError, TextDocument, EDIT_TIMELINE,
};

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

    /// 新規 layer の id 採番の**正本**。**墓標(tombstone)込みの最大 id + 1**を返す。
    ///
    /// [`Self::layers`] は present な layer だけを返すので、それを基に採番すると
    /// 「削除した最大 id の layer を再び置く」瞬間に墓標の id を再利用してしまう
    /// (2026-08-20 の敵対的レビュー)。id はトラック/マスクの entity path
    /// (`/layer/{id}`)そのものなので、再利用すると死んだ layer の component が
    /// 新しい layer へ「復活」して付き直る — かつ `Intent::SetMeta` は
    /// 新規配置専用の柵(裁定108(c))を持つため、墓標の id には既に `meta` component が
    /// 残っており、正当な新規配置がその柵に引っかかって `Err` になる。
    ///
    /// `sorted_entity_paths()` は tombstone を含め**一度でも書かれた entity path**を
    /// 全部返すので、present かどうかを見ずに id だけを拾う。
    pub fn next_layer_id(&self) -> u64 {
        self.db
            .sorted_entity_paths()
            .filter_map(layer_id_of)
            .map(|id| id.0)
            .max()
            .map(|max| max + 1)
            .unwrap_or(1)
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

    /// この entity(layer か `/composition`)が今の edit 時点で持つ、`TrackJson` で
    /// 符号化された component **全部**を、component 名を知らずに読む。
    ///
    /// **`persist.rs::flattened()` の核**(裁定108(a) の構造修正)。`meta`/`masks`/
    /// `attrs`/`effects`/`shapes`/`text`/`Composition:settings`/`Composition:markers`/
    /// 個々の property track を1つずつ名指しする代わりに、この口が
    /// `all_components_for_entity` へ**store に聞く**(裁定57)。新しい component を
    /// 足しても、ここにもコピー先にも1行も足さずに保存へ乗る。
    ///
    /// `Layer:present`(`LayerPresent`、bool)だけは対象外 — 別型で `TrackJson` として
    /// 読めないのと、存在は `Intent::AddLayer` が別途持つため。
    ///
    /// **`present` 以外は全部 `TrackJson` のはず、という前提を検査する**
    /// (2026-08-20 の敵対的レビュー修正)。以前は component が値を持っているのに
    /// `TrackJson` として型が合わない場合、`component_batch::<TrackJson>` が
    /// `None` を返すのを「値が無い」と同じ扱いで `filter_map` が黙って捨てていた —
    /// `Layer:present` 以外に別 `Loggable` 型の component が増えた日、その component は
    /// `flattened()`/`save()` から**エラーも出さず静かに消える**。「値が無い」
    /// (`component_batch_raw` も `None`)と「値はあるが `TrackJson` として読めない」
    /// (`component_batch_raw` は `Some` なのに型付き読みが `None`)を区別し、
    /// 後者だけ `Err` にする(裁定37 と同じ「無い」と「壊れている」の非同義)。
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
                // この edit 時点では値を持たない(別の時点でだけ書かれた component が
                // entity の schema に残っているだけ) — 「無い」なので飛ばしてよい。
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

    /// property の値の出処(`PropertySource::{Track,Slot}`、`slot` 発注単位)。
    /// **評価はしない** — 生の意味をそのまま返す。`Ok(None)` = **その property に
    /// 値が無い**。`Err` = **値はあるが読めない**。この2つを同義にしない — 同義にすると
    /// 壊れた Document が静かに既定値へ落ち、利用者には「値が勝手に戻った」としか
    /// 見えない(M13: 無反応ゼロ / 拒否は理由が分かる)。
    ///
    /// layer の property もカメラの property(裁定116)も同じ読み方(どの entity の
    /// component を latest-at で引くか)しか違わないので、経路を1本に保つ。
    fn source_at_path(
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

    /// property の keyframe track。**`PropertySource::Slot` を指している property は
    /// ここでは `None`** — この property 自身は track を持たない(値はスロット表の
    /// 側にある)。「track が無い」と「スロットへ委譲している」を区別したい場合は
    /// [`Self::property_source`] を使う。評価込みの値が欲しい場合は [`Self::value_at`]
    /// (スロット参照も解決する)。
    fn track_at_path(
        &self,
        path: &EntityPath,
        property: &PropertyId,
    ) -> Result<Option<KeyframeTrack>, StoreError> {
        Ok(match self.source_at_path(path, property)? {
            Some(PropertySource::Track(track)) => Some(track),
            Some(PropertySource::Slot(_)) | None => None,
        })
    }

    pub fn track(
        &self,
        layer: LayerId,
        property: &PropertyId,
    ) -> Result<Option<KeyframeTrack>, StoreError> {
        self.track_at_path(&layer.entity_path(), property)
    }

    /// カメラの property の keyframe track(`PropertyId::camera` で作った物のみ意味を
    /// 持つ)。無ければ `Ok(None)`(その property はまだキーを打っていない、またはスロット
    /// 参照へ委譲している)。
    pub fn camera_track(&self, property: &PropertyId) -> Result<Option<KeyframeTrack>, StoreError> {
        self.track_at_path(&Document::composition_path(), property)
    }

    /// この property の生の出処(`Track` か `Slot` か)。`track()`/`value_at()` は
    /// この2つを区別しない読み方(片方は「値なし」に潰す、片方は解決する)なので、
    /// Inspector 等が「この行はスロット参照だ」と表示したい時はここを使う。
    pub fn property_source(
        &self,
        layer: LayerId,
        property: &PropertyId,
    ) -> Result<Option<PropertySource>, StoreError> {
        self.source_at_path(&layer.entity_path(), property)
    }

    /// カメラの property 版(同上)。
    pub fn camera_property_source(
        &self,
        property: &PropertyId,
    ) -> Result<Option<PropertySource>, StoreError> {
        self.source_at_path(&Document::composition_path(), property)
    }

    /// comp の Slots 表(`composition/animation/slots`)。並びは編集順
    /// (`Intent::SetSlots` が渡した `Vec` の並びをそのまま保つ、mask/effect と同じ流儀)。
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

    /// `id` を持つスロットの track。**表に無い id は `Ok(None)`**(まだキーを打って
    /// いない property と同じ扱い、裁定20 の応用 — 参照先が無いだけで壊れてはいない)。
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
        match self.source_at_path(path, property)? {
            Some(PropertySource::Track(track)) => Ok(Some(track.eval(t))),
            // **スロット参照はここで解決する** — `value_at`/`camera_value_at` を
            // 呼ぶ側は「この property がスロットへ委譲しているか」を意識しなくてよい
            // (`properties/property sid` の意味そのもの、地図の note どおり)。
            Some(PropertySource::Slot(slot_id)) => {
                Ok(self.slot_track(&slot_id)?.map(|track| track.eval(t)))
            }
            None => Ok(None),
        }
    }

    /// comp 時刻の値。**補間の意味は `motolii-eval` が持つ**ので、ここは呼ぶだけ。
    /// スロット参照も含めて解決済みの値を返す。
    pub fn value_at(
        &self,
        layer: LayerId,
        property: &PropertyId,
        t: RationalTime,
    ) -> Result<Option<Value>, StoreError> {
        self.value_at_path(&layer.entity_path(), property, t)
    }

    /// カメラの property の comp 時刻の値。
    pub fn camera_value_at(
        &self,
        property: &PropertyId,
        t: RationalTime,
    ) -> Result<Option<Value>, StoreError> {
        self.value_at_path(&Document::composition_path(), property, t)
    }

    /// この comp 時刻でのカメラ(裁定113/115)。track が無い property は既定値になる
    /// (パン無し・zoom=1・roll=0)— 裁定20「キーを打っていない property は静止値」を
    /// カメラにもそのまま適用する。
    pub fn resolve_camera(&self, t: RationalTime) -> Result<motolii_core::ResolvedCamera, StoreError> {
        let center_property = PropertyId::camera(property::CAMERA_CENTER)?;
        let center = match self.camera_value_at(&center_property, t)? {
            Some(Value::Vec2(v)) => [v[0] as f32, v[1] as f32],
            Some(other) => {
                return Err(StoreError::Property(format!(
                    "{} に2成分でない値が入っている: {other:?}",
                    property::CAMERA_CENTER
                )))
            }
            None => [0.0, 0.0],
        };

        let zoom_property = PropertyId::camera(property::CAMERA_ZOOM)?;
        let zoom = match self.camera_value_at(&zoom_property, t)? {
            Some(Value::F64(v)) => v as f32,
            Some(other) => {
                return Err(StoreError::Property(format!(
                    "{} に数値でない値が入っている: {other:?}",
                    property::CAMERA_ZOOM
                )))
            }
            None => 1.0,
        };

        let roll_property = PropertyId::camera(property::CAMERA_ROLL)?;
        let roll_degrees = match self.camera_value_at(&roll_property, t)? {
            Some(Value::F64(v)) => v as f32,
            Some(other) => {
                return Err(StoreError::Property(format!(
                    "{} に数値でない値が入っている: {other:?}",
                    property::CAMERA_ROLL
                )))
            }
            None => 0.0,
        };

        Ok(motolii_core::ResolvedCamera {
            center,
            zoom,
            roll_degrees,
        })
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

    /// comp のマーカー一覧。**宣言順**(マスクと同じく暗黙の隣接参照を作らない、裁定66)。
    ///
    /// component が無い = マーカーが1枚も無いので空を返す(マスクの `masks()` と同じ扱い)。
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

    /// この layer のマスク一覧(キーを打たない部分だけ)。**スタックの順**。
    ///
    /// component が無い = **マスクが1枚も無い**なので空を返す。ここは `meta` と違って
    /// 「無い」と「空」が同じ意味である。読めた上で壊れている場合だけ `Err`(裁定37)。
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

    /// layer の非アニメーション属性(hidden/parent/blend mode/matte/name/auto-orient)。
    ///
    /// `Ok(None)` = **まだ一度も `SetAttrs` で書かれていない**。`meta` と同じく
    /// 「無い」と「空」を同義にしない(裁定37)— ただし読み手側(`resolve`)は
    /// `None` を「既定値」として扱ってよい(属性は元々省略可能なので、これは
    /// マスク一覧の「無い=0枚」と同じ形)。
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

    /// layer が持つ effect インスタンスの列。無ければ空(masks と同じ扱い)。
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

    /// shape-layer の図形列。無ければ空。
    pub fn shapes(&self, layer: LayerId) -> Result<Vec<Shape>, StoreError> {
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

    /// text-layer の中身(content・組版既定値・フォント参照)。`Ok(None)` = **まだ一度も
    /// `SetTextDocument` で書かれていない** — `meta`/`attrs` と同じく「無い」と「壊れている」
    /// を同義にしない(裁定37)。
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

    /// comp 時刻でのマスク。形状も不透明度も普通の property track から取る。
    fn resolved_masks(
        &self,
        layer: LayerId,
        t: RationalTime,
    ) -> Result<Vec<ResolvedMask>, StoreError> {
        let mut out = Vec::new();
        for mask in self.masks(layer)? {
            let shape_property = PropertyId::mask_shape(mask.id);
            let shape = match self.value_at(layer, &shape_property, t)? {
                Some(Value::Path(path)) => path,
                // **黙って飛ばさない**。形状の無いマスクは壊れた Document であって、
                // 既定値で描くと利用者には「マスクが勝手に消えた」としか見えない。
                Some(other) => {
                    return Err(StoreError::Property(format!(
                        "マスク {} の形状にパスでない値が入っている: {other:?}",
                        mask.id
                    )))
                }
                None => {
                    return Err(StoreError::Property(format!(
                        "マスク {} に形状が無い(`mask.{}.shape` が未設定)",
                        mask.id, mask.id
                    )))
                }
            };

            let opacity_property = PropertyId::mask_opacity(mask.id);
            // キーを打っていない property は静止値(裁定20)。既定は不透明。
            let opacity = match self.value_at(layer, &opacity_property, t)? {
                Some(Value::F64(v)) => v as f32,
                Some(other) => {
                    return Err(StoreError::Property(format!(
                        "マスク {} の不透明度に数値でない値が入っている: {other:?}",
                        mask.id
                    )))
                }
                None => 1.0,
            };

            out.push(ResolvedMask {
                mode: mask.mode,
                inverted: mask.inverted,
                opacity: opacity.clamp(0.0, 1.0),
                shape,
            });
        }
        Ok(out)
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

        // hidden は「今フレームは描かない」— present(削除)とは別物(裁定108(c) 系)。
        // 属性が一度も書かれていない layer は既定(非 hidden)として扱う。
        let attrs = self.attrs(layer)?.unwrap_or_default();
        if attrs.hidden {
            return Ok(None);
        }

        // 時間の判定は Document がする。engine は解決済みの素材フレームを受け取るだけ。
        let Some(composition) = self.composition()? else {
            return Ok(None);
        };
        let comp_frame = t
            .try_to_frame_floor(composition.fps)
            .map_err(|e| StoreError::Property(e.to_string()))?;
        let Some(mut source_frame) = meta.timing.source_frame(comp_frame) else {
            return Ok(None);
        };
        // `tm`(Time Remap、precomposition-layer)。track があれば**素材のフレーム番号を
        // 直接**上書きする — 通常の speed/trim による写像より優先する(裁定65 が
        // timing から追い出した分、property 側で戻す)。timing が「居る/居ない」を
        // 決める点は変わらないので、上の `covers` 判定はそのまま活きる。
        if let Some(remap) = self.value_at(layer, &PropertyId::new(property::TIME_REMAP)?, t)? {
            match remap {
                Value::F64(v) => source_frame = v.floor() as i64,
                other => {
                    return Err(StoreError::Property(format!(
                        "{} に数値でない値が入っている: {other:?}",
                        property::TIME_REMAP
                    )))
                }
            }
        }
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

        let vec2 = |name: &str, default: [f32; 2]| -> Result<[f32; 2], StoreError> {
            let property = PropertyId::new(name)?;
            match self.value_at(layer, &property, t)? {
                Some(Value::Vec2(v)) => Ok([v[0] as f32, v[1] as f32]),
                Some(other) => Err(StoreError::Property(format!(
                    "{name} に2成分でない値が入っている: {other:?}"
                ))),
                None => Ok(default),
            }
        };

        // 行列は `motolii-core` が組む。**適用順序の正本はそこ1箇所**(裁定58)。
        let transform = LayerPlacement::from_transform(
            vec2(property::ANCHOR, [0.0, 0.0])?,
            self.resolve_position(layer, t)?,
            vec2(property::SCALE, [1.0, 1.0])?,
            scalar(property::ROTATION, 0.0)?,
            scalar(property::SKEW, 0.0)?,
            scalar(property::SKEW_AXIS, 0.0)?,
        );

        Ok(Some(ResolvedLayer {
            placement: LayerPlacement {
                transform,
                opacity: scalar(property::OPACITY, 1.0)?.clamp(0.0, 1.0),
                order: meta.order,
                // 裁定113/116: 全員 z=0 既定。`position.x`/`position.y`(split-position)
                // の隣に同じ流儀で置いた `position.z`。
                z: scalar(property::POSITION_Z, 0.0)?,
            },
            declared_size: size,
            source: meta.source,
            source_frame,
            masks: self.resolved_masks(layer, t)?,
            blend_mode: attrs.blend_mode,
            matte: attrs.matte,
            pinned: attrs.pinned,
        }))
    }

    /// position の値。**`position`(Vec2 単一 track)を優先し、無ければ split(x/y 別
    /// track)を試す**(裁定61)。どちらも無ければ既定 `[0,0]`。
    ///
    /// split は「x か y のどちらかだけキーを打つ」も許す — 片方が無い場合はその成分だけ
    /// 0.0(AE で「そちらの軸は動かしていない」と同じ扱い)。
    fn resolve_position(&self, layer: LayerId, t: RationalTime) -> Result<[f32; 2], StoreError> {
        let position = PropertyId::new(property::POSITION)?;
        match self.value_at(layer, &position, t)? {
            Some(Value::Vec2(v)) => return Ok([v[0] as f32, v[1] as f32]),
            Some(other) => {
                return Err(StoreError::Property(format!(
                    "{} に2成分でない値が入っている: {other:?}",
                    property::POSITION
                )))
            }
            None => {}
        }

        let x = self.split_position_component(layer, property::POSITION_X, t)?;
        let y = self.split_position_component(layer, property::POSITION_Y, t)?;
        Ok([x.unwrap_or(0.0), y.unwrap_or(0.0)])
    }

    fn split_position_component(
        &self,
        layer: LayerId,
        name: &str,
        t: RationalTime,
    ) -> Result<Option<f32>, StoreError> {
        let property = PropertyId::new(name)?;
        match self.value_at(layer, &property, t)? {
            Some(Value::F64(v)) => Ok(Some(v as f32)),
            Some(other) => Err(StoreError::Property(format!(
                "{name} に数値でない値が入っている: {other:?}"
            ))),
            None => Ok(None),
        }
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
