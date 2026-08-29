//! 読み口 — front が受け取る唯一の物。可変な口を1つも持たない。

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

/// [`StoreView::value_at_path_resolving_links`] の防御的な深さ上限。
/// `Intent::SetPropertyLink` の書き込み時循環拒否をすり抜けた壊れた Document
/// (手で書き換えた保存ファイル等)を読んでも無限再帰にならないための保険——
/// `world_affine`/`frozen_ancestor` の `seen`/`visiting` と同じ役割だが、link は
/// 呼び出し頻度が高い `value_at_path` の内側に居るのでハッシュ集合を毎回確保する
/// コストを避け、単純な深さカウンタにしてある(正常な Document ではこの分岐に
/// 一度も到達しない)。
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

    /// property の keyframe track。**`PropertySource::{Slot,Link}` を指している
    /// property はここでは `None`** — この property 自身は track を持たない(値は
    /// スロット表の側、または別 property の値にある)。「track が無い」と「委譲して
    /// いる」を区別したい場合は [`Self::property_source`] を使う。評価込みの値が
    /// 欲しい場合は [`Self::value_at`](スロット/link 参照も解決する)。
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
            // base が Slot・base 無し(modulator だけ)・そもそも source が無い、
            // のどれも「この property 自身の track」は持たない(裁定213 でも
            // この非対称は変わらない——評価済みの値が欲しければ `value_at`)。
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
        self.value_at_path_resolving_links(path, property, t, 0)
    }

    /// [`Self::value_at_path`] の本体。`link_depth` は modulator の参照鎖を辿った
    /// 深さ——`Intent::SetPropertyLink`/`Intent::SetPropertyModulators` は書き込み時に
    /// 循環を拒む(`document::validate_no_link_cycle`)ので正常な Document ではここが
    /// 伸び続けることは無いはずだが、`world_affine`/`frozen_ancestor` と同じ「壊れた
    /// Document を読んだ場合に備えた第二の防御」をここにも掛ける(手で書き換えた
    /// 保存ファイル等、書き込み経路を通らずに循環が紛れ込む可能性を潰す)。
    ///
    /// **裁定213**: 値 = `base` の評価値 + `modulators` の寄与の和。`Value::add`
    /// (`motolii-eval`)が「変調できる型」の境界を持っている——`Bool`/`Enum`/
    /// `LayerId` は常に `None` を返すので、そこでは**単一 source が勝つ**(最初に
    /// 確定した値のまま、以降の modulator は無視される)。型不一致・`Path` の
    /// 頂点数不一致も同じ理由で無視(近似しない、`translate_link` と同じ規約)。
    fn value_at_path_resolving_links(
        &self,
        path: &EntityPath,
        property: &PropertyId,
        t: RationalTime,
        link_depth: u32,
    ) -> Result<Option<Value>, StoreError> {
        // **overlay が最優先**(タスク#20 の恒久解)。track の評価より先に見る —
        // ドラッグ中は時刻に関わらずこの固定値を返す(overlay は「評価済みの値」を
        // 直接持つので、ここでは `track.eval(t)` を呼ばない)。`track()`/`camera_track()`
        // はこの overlay を一切見ない(裁定134 の線引きのまま — 生の意味だけを返す)。
        if let Some(value) = self.transient_value_at(path, property) {
            return Ok(Some(value));
        }
        let Some(source) = self.source_at_path(path, property)? else {
            return Ok(None);
        };

        // base の評価値(無ければ `None` — modulator の和だけが値になる)。
        let mut acc: Option<Value> = match source.base {
            Some(PropertyBase::Track(track)) => Some(track.eval(t)),
            // **スロット参照はここで解決する** — `value_at`/`camera_value_at` を
            // 呼ぶ側は「この property がスロットへ委譲しているか」を意識しなくて
            // よい(`properties/property sid` の意味そのもの、地図の note どおり)。
            Some(PropertyBase::Slot(slot_id)) => {
                self.slot_track(&slot_id)?.map(|track| track.eval(t))
            }
            None => None,
        };

        // **modulator の寄与を加算する**(裁定206 の型付き link 機構をそのまま
        // 「和の1項」として再利用)。読むのは (a) `t + time_offset`(Document 由来の
        // 静的値) (b) 参照先 property を**同じ経路**で再帰的に解決した値 (c)
        // `params`(Document 由来の animatable 値)だけ——壁時計・ライブ音声・乱数・
        // OS入力を一切読まないので、`motolii-eval` の「時刻t→値の純関数」契約
        // (`motolii-eval/src/lib.rs:16`)にそのまま収まる。
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
            // 参照先に値が無ければ、この modulator は寄与しない(裁定20の応用 —
            // ぶら下がった参照と同じ「無いだけ」の扱い)。
            let Some(source_value) = source_value else {
                continue;
            };
            let Some(contribution) =
                crate::slot::translate_link(&modulator.plugin_id, &modulator.params, source_value)
            else {
                continue; // 型不一致・未知の plugin_id は近似せず寄与ゼロ。
            };
            acc = Some(match acc {
                // **加算できなければ単一 source が勝つ**(`Value::add` が `None` を
                // 返す=Hold型・型不一致・Path条件不成立)——先に確定していた値を
                // そのまま保つ(base があれば base、無ければ先に確定した modulator)。
                Some(current) => current.add(&contribution).unwrap_or(current),
                None => contribution,
            });
        }

        Ok(acc)
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
}

fn layer_id_of(path: &EntityPath) -> Option<LayerId> {
    let s = path.to_string();
    s.strip_prefix("/layer/")
        .and_then(|rest| rest.parse::<u64>().ok())
        .map(LayerId)
}
