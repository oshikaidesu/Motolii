//! 読み口 — front が受け取る唯一の物。可変な口を1つも持たない。

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

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
use crate::slot::PropertySource;
use crate::{
    property, Asset, AssetId, AssetTable, Composition, Document, EffectInstance, LayerAttrs,
    LayerId, LayerMeta, LayerPlacement, LayerSource, Marker, Mask, PropertyId, ResolvedEffect,
    ResolvedLayer, ResolvedMask, Revision, ShapeNode, Slot, SlotId, StoreError, TextDocument,
    EDIT_TIMELINE,
};

/// ある edit 時点の Document の姿。**query の投影であって、独自の状態を持たない**。
///
/// `transient` は例外的に「独自の状態」に見えるが、これは Document が持つ overlay
/// への**借用**であって、`StoreView` 自身は何も所有しない(overlay の正本は
/// `Document::transient` のまま、ここはそれを読むだけ)。`track_cache` も同様 —
/// 解析済み track の**正本は `Document::track_cache`**、ここはその借用越しに
/// 読み書き(`RefCell`)するだけで、`StoreView` 自身が新しい状態を持つわけではない
/// (裁定140)。
///
/// `revision` を値で持つため(`Revision` は `ChunkStoreGeneration` を含み `Copy` では
/// ない)、以前の `Copy` 派生は落とした。呼び手は全員 `&StoreView<'_>` で受け取って
/// いるので(shell/engine/export/audio、2026-08-21 確認)実害は無い。
#[derive(Clone)]
pub struct StoreView<'a> {
    db: &'a EntityDb,
    at: i64,
    transient: &'a HashMap<TransientKey, Value>,
    revision: Revision,
    track_cache: &'a RefCell<TrackCache>,
}

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

    /// `path`/`property` を track キャッシュの鍵(`TransientKey`)へ写す。transient
    /// overlay と全く同じ scope の切り方(layer property か camera property か)を
    /// 使い回す — 鍵の形をもう1種類増やさない。
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
        // **解析済み track の revision 鍵キャッシュ**(裁定140)。`track()` コストの
        // 97%が serde_json 解析だった(2026-08-21 計測、KNOWN.md)ので、ここで
        // parse 済みの `PropertySource` を revision ごとに再利用する。無効化は
        // `TrackCache::sync` が revision 比較で機械的に行う — 手動 invalidate 口は
        // 無い。鍵を作れない path(layer でも camera でもない、現状の呼び手には
        // 存在しない)はキャッシュを経由せず素で読む。
        let Some(key) = Self::cache_key(path, property) else {
            return self.parse_source_at_path(path, property);
        };
        self.track_cache.borrow_mut().get_or_try_insert_with(
            &self.revision,
            key,
            || self.parse_source_at_path(path, property),
        )
    }

    /// `source_at_path` の素読み本体(キャッシュ未経由)。`TrackJson` を
    /// `serde_json` で `PropertySource` へ解く、この crate で唯一の parse 経路。
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
        // **overlay が最優先**(タスク#20 の恒久解)。track の評価より先に見る —
        // ドラッグ中は時刻に関わらずこの固定値を返す(overlay は「評価済みの値」を
        // 直接持つので、ここでは `track.eval(t)` を呼ばない)。`track()`/`camera_track()`
        // はこの overlay を一切見ない(裁定134 の線引きのまま — 生の意味だけを返す)。
        if let Some(value) = self.transient_value_at(path, property) {
            return Ok(Some(value));
        }
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

    /// `path` が指す entity(layer か `/composition`)に、`property` の overlay が
    /// 置かれていればその値を返す。**layer をまたいで誤爆しない**よう、`path` から
    /// layer/カメラの scope を復元してから overlay の key と突き合わせる
    /// (`TransientKey` の doc 参照)。
    fn transient_value_at(&self, path: &EntityPath, property: &PropertyId) -> Option<Value> {
        let key = if *path == Document::composition_path() {
            TransientKey::Camera(property.clone())
        } else {
            TransientKey::Layer(layer_id_of(path)?, property.clone())
        };
        self.transient.get(&key).cloned()
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

    /// 素材台帳(裁定162: bin-first — 取り込んだが未配置の素材。`Intent::AdmitAsset`/
    /// `Intent::RemoveAsset` の書き口はこの台帳を読んでから丸ごと書き戻す)。**まだ
    /// 一度も admit していない Document は空の台帳**(markers/slots と同じ「無い=空」
    /// の扱い、component が無ければ壊れているのではなく単に空)。
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

    /// 台帳の一覧。**`AssetId` 昇順**(`AssetTable` の内部が `BTreeMap`、旧台帳の
    /// 意味そのまま)。Browser 等の front はここを読む(EXACT TARGET #3、裁定162)。
    pub fn assets(&self) -> Result<Vec<Asset>, StoreError> {
        Ok(self.assets_table()?.iter().cloned().collect())
    }

    /// 単体引き。台帳に無い id は `Ok(None)`(`markers`/`masks` の「無い=空」と同じ
    /// 線引き — 該当 id が無いのは壊れた Document ではなく普通に有り得る)。
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

    /// `layer` の祖先鎖(`attrs.parent` を辿る)に、`frozen == true` な
    /// `LayerSource::Group` が居れば、そのうち最も近い物の id を返す(裁定119
    /// `docs/reviews/2026-08-20-group-layer-semantics-decision.md` §4)。
    ///
    /// **`layer` 自身の `frozen` 状態は見ない** — 凍結が拒むのは「中身」への編集で
    /// あって、凍結された Group 自身の attrs/timing/track への編集は含まない
    /// (`crate::document::check_not_frozen` の doc 参照)。
    ///
    /// `document::validate_no_parent_cycle`/`world_affine` と同じ `seen` 防御 —
    /// 書き込み時ガードで循環は作れないはずだが、壊れた Document を読んだ場合に
    /// 備えて無限ループしない形にしてある。
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

    /// shape-layer の図形列。無ければ空。裁定173 H4: `Vec<ShapeNode>` — 旧 `Vec<Shape>`
    /// の JSON も `ShapeNode::Leaf` の列として無改造で読める(`ShapeNode` は
    /// `#[serde(untagged)]`)。
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

    /// comp 時刻での effect スタック。**disabled な effect はここに現れない**
    /// (`resolve_with_solo` が hidden な layer を `None` で弾くのと同じ形 — 「切る」を
    /// フラグとして運ばず、入口で除く)。空スタックは `self.effects(layer)?` の1回の
    /// 読み出しだけで即 return し、`self.properties(layer)`(component 一覧の走査)
    /// までは踏まない — effect の無い layer の resolve コストを増やさないため。
    fn resolved_effects(
        &self,
        layer: LayerId,
        t: RationalTime,
    ) -> Result<Vec<ResolvedEffect>, StoreError> {
        let effects = self.effects(layer)?;
        if effects.is_empty() {
            return Ok(Vec::new());
        }

        // param 名の発見は既存の汎用列挙(裁定57「store に聞く」) — effect 専用の
        // 列挙 API を新設しない(縫い目調査 1a)。
        let properties = self.properties(layer);

        let mut out = Vec::with_capacity(effects.len());
        for effect in effects {
            if !effect.enabled {
                continue;
            }
            let prefix = format!("{}{}.param.", property::EFFECT_PREFIX, effect.id);
            let mut params = Vec::new();
            for candidate in &properties {
                let Some(param_name) = candidate.name().strip_prefix(prefix.as_str()) else {
                    continue;
                };
                // track の評価は既存の汎用経路(`value_at`)をそのまま再利用する —
                // 新しい評価器は書かない(EXACT TARGET #2)。track が実在すれば
                // `value_at` は必ず `Some` を返す(scalar/vec2 と同じ前提)。
                if let Some(value) = self.value_at(layer, candidate, t)? {
                    params.push((param_name.to_owned(), value));
                }
            }
            out.push(ResolvedEffect {
                plugin_id: effect.plugin_id,
                params,
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
        // `any_solo` は comp 全体を見ないと判定できないので、単発呼び出しではここで
        // 1回だけ走査する。`resolved_layers` は全 layer を回る側で既に同じ走査を
        // 1回すませているので、そちらは `resolve_with_solo` を直接呼んで
        // この再走査を踏まない(2026-08-20 の性能回帰: 層数 N に対し、ここで毎回
        // `any_solo` を呼ぶと `resolved_layers` 経由で N 回 × 全層走査 = O(N²) の
        // attrs 二重読みになっていた)。
        let any_solo = self.any_solo()?;
        // 単発呼び出しなので世界合成のメモ/循環ガードもこの1回限りの使い捨て
        // (裁定173 H1)。祖先を跨いで共有したいのは [`Self::resolved_layers`] が
        // 1回の document-wide resolve の中で複数の子から同じ祖先を引く場面 —
        // そちらは呼び出し側で1つの `memo` を作って全 layer 分使い回す。
        let present: HashSet<LayerId> = self.layers().into_iter().collect();
        let mut memo = HashMap::new();
        let mut visiting = HashSet::new();
        self.resolve_with_solo(layer, t, any_solo, &present, &mut memo, &mut visiting)
    }

    /// [`Self::resolve`] の本体。`any_solo` を呼び出し側から受け取ることで、
    /// 全層を回る [`Self::resolved_layers`] が層ごとに `any_solo` を再走査しなくて
    /// 済むようにする(1パスで導出した solo 判定を使い回す)。
    ///
    /// `present`/`memo`/`visiting` は世界合成(裁定173 H1、[`Self::world_affine`])の
    /// 入出力 — 呼び出し側([`Self::resolve`]/[`Self::resolved_layers`])が1回の
    /// resolve 呼び出し分だけ作り、複数 layer の resolve を跨いで使い回す
    /// (「同じフレームで親を二度解決しない」がここで成立する)。
    fn resolve_with_solo(
        &self,
        layer: LayerId,
        t: RationalTime,
        any_solo: bool,
        present: &HashSet<LayerId>,
        memo: &mut HashMap<LayerId, glam::Affine2>,
        visiting: &mut HashSet<LayerId>,
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

        // solo: comp のどこかに solo な layer が居るなら、solo でない layer は
        // hidden と同じ経路でここに落ちる。**hidden が勝つ** — 上の hidden 判定を
        // 通り抜けた(= 自分は hidden ではない)layer だけがここへ来るので、
        // 「隠した層を solo で復活させる」ことは構造的に起きない。solo な layer
        // 自身が同時に hidden なら、その layer は上で既に弾かれているが、
        // 「comp のどこかに solo な layer が居る」という判定(`any_solo`)には
        // hidden かどうかを問わず含める — solo フラグは hidden と独立な「意図」の
        // 表明であって、hidden がそれを見えなくするだけで無かったことにはしない
        // (裁定119 のグループ AND 導出と衝突しない、単層の bool のまま)。
        if any_solo && !attrs.solo {
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

        // **comp 空間へのアフィン**(裁定173 H1)。局所の行列そのものの意味の正本は
        // 今まで通り `motolii-core`(裁定58)— ここは「親の world アフィンを左から
        // 合成する」再帰の入口を呼ぶだけ([`Self::world_affine`] 参照)。parent が
        // 無い/tombstone/循環なら local のみへ縮退するので、parent を1つも使って
        // いない既存 Document はここまで含めて今まで通りの値を返す。
        let transform = self.world_affine(layer, t, present, memo, visiting)?;

        Ok(Some(ResolvedLayer {
            id: layer,
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
            effects: self.resolved_effects(layer, t)?,
            blend_mode: attrs.blend_mode,
            matte: attrs.matte,
            pinned: attrs.pinned,
        }))
    }

    /// **この layer 自身**の property track だけから local `Affine2` を組む(裁定58
    /// の正本 `LayerPlacement::from_transform` をそのまま呼ぶ)。祖先を一切見ない —
    /// [`Self::world_affine`] がこれを再帰の各段の部品として使う。
    fn local_placement_transform(
        &self,
        layer: LayerId,
        t: RationalTime,
    ) -> Result<glam::Affine2, StoreError> {
        let scalar = |name: &str, default: f32| -> Result<f32, StoreError> {
            let property = PropertyId::new(name)?;
            match self.value_at(layer, &property, t)? {
                Some(Value::F64(v)) => Ok(v as f32),
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
        Ok(LayerPlacement::from_transform(
            vec2(property::ANCHOR, [0.0, 0.0])?,
            self.resolve_position(layer, t)?,
            vec2(property::SCALE, [1.0, 1.0])?,
            scalar(property::ROTATION, 0.0)?,
            scalar(property::SKEW, 0.0)?,
            scalar(property::SKEW_AXIS, 0.0)?,
        ))
    }

    /// [`Self::local_placement_transform`] の外部公開口(裁定174 G1)。
    ///
    /// Ungroup が「Group の変換を子ローカルへ焼き込む」際に使う — 計算ロジックは
    /// ここで複製せず、H1 の正本をそのまま呼ぶだけ(`Document::ungroup_layers` が
    /// この口経由で group/子それぞれの local `Affine2` を読む、単一源)。
    pub fn local_transform(&self, layer: LayerId, t: RationalTime) -> Result<glam::Affine2, StoreError> {
        self.local_placement_transform(layer, t)
    }

    /// この layer の**world(comp 空間)アフィン** = 親の world アフィン × 自分の
    /// local アフィン(裁定173 H1、キーフレームは各ノードローカルのまま・**合成だけが
    /// 再帰**という利用者仮説の実装形)。旧世界 `crates/motolii-doc/src/
    /// spatial_resolve.rs::ensure_resolve_affine` の概念移植 — メモ化 `HashMap` +
    /// `visiting` の cycle ガードで「同じフレームで親を二度解決しない」を保証する。
    ///
    /// - **parent が無い**: local のみ(既存 Document は今まで通りの値)
    /// - **parent が tombstone(present ではない)/存在しない**: `present` でのフィルタで
    ///   `None` 扱いに縮退する — `next/ui/motolii-timeline-pane/src/projection.rs::rows`
    ///   (裁定173 H2)の `attrs.parent.filter(|p| present.contains(p))` と**同じ意味論**
    ///   (壊れた参照で resolve を落とさない、H2 到達性判定の踏襲)
    /// - **parent 鎖に循環がある**(書き込み時ガード `document::validate_no_parent_cycle`
    ///   を抜けた壊れた Document を読んだ場合の第二の柵、H-survey §2.2): 再訪を
    ///   `visiting` で検知し、その枝は local のみへ縮退する。**memo には書かない** —
    ///   循環中に観測される値は「本当の」world ではないので、後で(外側の呼び出しが)
    ///   書く値を上書きしない
    fn world_affine(
        &self,
        layer: LayerId,
        t: RationalTime,
        present: &HashSet<LayerId>,
        memo: &mut HashMap<LayerId, glam::Affine2>,
        visiting: &mut HashSet<LayerId>,
    ) -> Result<glam::Affine2, StoreError> {
        if let Some(world) = memo.get(&layer) {
            return Ok(*world);
        }
        // メモ化の呼び出し回数証明(裁定173 H1 oracle)が数える「本当にこの layer を
        // 解決した回数」— memo hit を素通りした、ここから先の1回だけを数える。
        #[cfg(test)]
        record_world_affine_compute();

        let local = self.local_placement_transform(layer, t)?;

        let parent = self
            .attrs(layer)?
            .unwrap_or_default()
            .parent
            .filter(|p| present.contains(p));
        let Some(parent) = parent else {
            memo.insert(layer, local);
            return Ok(local);
        };

        if !visiting.insert(layer) {
            // 防御的セカンドガード(H-survey §2.2)。壊れた Document でも無限再帰
            // しない — memo には書かず、ローカルのみへ縮退した値をこの枝だけに返す。
            return Ok(local);
        }
        let parent_world = self.world_affine(parent, t, present, memo, visiting)?;
        visiting.remove(&layer);

        let world = parent_world * local;
        memo.insert(layer, world);
        Ok(world)
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

    /// comp のどこかに `solo=true` な layer が居るか。**hidden は問わない**
    /// (`resolve` のコメント参照 — solo という意図の表明は hidden と独立)。
    fn any_solo(&self) -> Result<bool, StoreError> {
        for layer in self.layers() {
            if self.attrs(layer)?.unwrap_or_default().solo {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// この時刻に描くべき layer を**奥から手前の順**で返す。
    ///
    /// `any_solo` はここで**1回だけ**走査する。`resolve` を層ごとに呼ぶ素朴な実装だと、
    /// `resolve` が毎回 comp 全層を re-scan するので N layer で O(N²) の attrs 二重読みに
    /// なる(2026-08-20 の性能回帰の原因、r2 probe 実測で発覚)。resolve は既に
    /// layer ごとに自分の attrs を読んでいるので、その1パスから solo の有無だけ
    /// 先に導出して使い回す。
    ///
    /// **世界合成のメモ(裁定173 H1)もここで1回だけ作り、全 layer 分使い回す** —
    /// 兄弟が同じ祖先を parent に持つ場合、祖先の world アフィンは document-wide の
    /// この呼び出し1回につき1回しか解決されない(メモ化の呼び出し回数証明 oracle)。
    pub fn resolved_layers(&self, t: RationalTime) -> Result<Vec<ResolvedLayer>, StoreError> {
        let any_solo = self.any_solo()?;
        let layers = self.layers();
        let present: HashSet<LayerId> = layers.iter().copied().collect();
        let mut memo = HashMap::new();
        let mut visiting = HashSet::new();
        let mut out = Vec::new();
        for layer in layers {
            if let Some(resolved) =
                self.resolve_with_solo(layer, t, any_solo, &present, &mut memo, &mut visiting)?
            {
                out.push(resolved);
            }
        }
        out.sort_by_key(|layer| layer.placement.order);
        Ok(out)
    }
}

/// `world_affine` の呼び出し回数計測(裁定173 H1 oracle「メモ化の呼び出し回数証明」)。
/// `#[cfg(test)]` なのでテスト以外のビルドには存在しない — `StoreView` は `&self` の
/// 純粋な読み口という設計(モジュール doc)を壊さずに、白箱ユニットテスト
/// (このファイル末尾の `mod tests`)だけがこのスレッドローカルを覗く。
#[cfg(test)]
thread_local! {
    static WORLD_AFFINE_COMPUTE_COUNT: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn record_world_affine_compute() {
    WORLD_AFFINE_COMPUTE_COUNT.with(|c| c.set(c.get() + 1));
}

#[cfg(test)]
fn reset_world_affine_compute_count() {
    WORLD_AFFINE_COMPUTE_COUNT.with(|c| c.set(0));
}

#[cfg(test)]
fn world_affine_compute_count() -> u32 {
    WORLD_AFFINE_COMPUTE_COUNT.with(|c| c.get())
}

fn layer_id_of(path: &EntityPath) -> Option<LayerId> {
    let s = path.to_string();
    s.strip_prefix("/layer/")
        .and_then(|rest| rest.parse::<u64>().ok())
        .map(LayerId)
}

/// 裁定173 H1 の白箱ユニットテスト。`world_affine`/`resolve_with_solo` は
/// `pub(crate)` ですらない完全 private なので、この crate 自身の `#[cfg(test)]`
/// からしか叩けない(統合テスト `tests/*.rs` は別クレートなので届かない) —
/// 数値証明・serde 往復・tombstone 縮退は公開 API だけで書けるので
/// `tests/transform_hierarchy.rs` に置き、ここには**メモ化の呼び出し回数証明**
/// (private な計測フックが要る)だけを置く。
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Composition, Fps, Intent, LayerAttrsPatch, LayerMeta, LayerSource, LayerTiming};

    fn t(frame: i64) -> RationalTime {
        RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap()
    }

    fn place(doc: &mut Document, layer: LayerId, parent: Option<LayerId>) {
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Null,
                    order: layer.0 as i16,
                    timing: LayerTiming::place(0, None, 300),
                },
            },
        ])
        .unwrap();
        if let Some(parent) = parent {
            doc.apply(Intent::SetAttrs {
                layer,
                patch: LayerAttrsPatch {
                    parent: Some(Some(parent)),
                    ..Default::default()
                },
            })
            .unwrap();
        }
    }

    /// **メモ化の呼び出し回数証明**(裁定173 H1 oracle)。3階層(root A ← mid B ←
    /// leaf C1/C2 の2兄弟)で、B(と A)を2人の子から引いても `world_affine` の
    /// 本体計算(memo miss)は B/A それぞれちょうど1回しか起きない — 旧世界
    /// `spatial_resolve.rs::ensure_resolve_affine`(メモ化 `HashMap` + 事前解決パス)の
    /// 概念移植が効いていることの直接証拠。
    #[test]
    fn shared_ancestor_is_resolved_exactly_once_across_siblings() {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 300,
            background: Composition::default_background(),
        }))
        .unwrap();

        let (a, b, c1, c2) = (LayerId(1), LayerId(2), LayerId(3), LayerId(4));
        place(&mut doc, a, None);
        place(&mut doc, b, Some(a));
        place(&mut doc, c1, Some(b));
        place(&mut doc, c2, Some(b));

        let view = doc.view();
        let present: HashSet<LayerId> = view.layers().into_iter().collect();
        let mut memo = HashMap::new();
        let mut visiting = HashSet::new();
        reset_world_affine_compute_count();

        // C1 を解決: local(C1) + local(B) + local(A) の3回が「初めて」計算される。
        view.world_affine(c1, t(0), &present, &mut memo, &mut visiting)
            .unwrap();
        assert_eq!(
            world_affine_compute_count(),
            3,
            "root/mid/leaf1 の3層でちょうど3回のはず(まだ誰も共有していない)"
        );

        // C2 を解決: B と A は memo に既に居るので、C2 自身の local だけが増える。
        view.world_affine(c2, t(0), &present, &mut memo, &mut visiting)
            .unwrap();
        assert_eq!(
            world_affine_compute_count(),
            4,
            "B(と A)が2人目の子 C2 のために再計算されてしまっている(メモ化が効いていない)"
        );
    }
}
