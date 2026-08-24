//! Document 本体 — 書き口1本 + undo/redo。

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

use motolii_eval::Value;

#[cfg(test)]
use crate::components::LayerPresent;
use crate::components::TrackJson;
use crate::slot::{PropertyLink, PropertySource};
use crate::view::StoreView;
use crate::{LayerAttrsPatch, Mask, Slot, SlotId, StoreError, EDIT_TIMELINE};

/// 編集の意図。**Document を書き換える道はこれだけ**。
#[derive(Clone, Debug)]
pub enum Intent {
    AddLayer(LayerId),
    /// 墓標を立てるだけで、chunk は落とさない(落とすと undo で戻せない)。
    RemoveLayer(LayerId),
    SetTrack {
        layer: LayerId,
        property: PropertyId,
        track: motolii_eval::KeyframeTrack,
    },
    /// この property をスロット参照へ切り替える(`properties/property sid`、`slot`
    /// 発注単位)。**`SetTrack` と書く先は同じ component** — `PropertySource::slot()` を
    /// そこへ書くだけで、新しい component は増やさない(地図の note「第二の差し替え
    /// 機構を作らない」)。`SetTrack` を再び投げれば普通の track へ戻せる(同じ場所を
    /// 上書きするだけなので、専用の「解除」variant は要らない)。
    SetPropertySlot {
        layer: LayerId,
        property: PropertyId,
        slot: SlotId,
    },
    /// この property を**型付き link**(`docs/reviews/2026-08-22-persona-touchdesigner-round2.md`
    /// §1、裁定206。裁定213 で加算の特殊形として `PropertySource::link_only` へ
    /// 移った)へ切り替える。`SetTrack`/`SetPropertySlot` と全く同じ形 —
    /// `PropertySource::link_only()` を同じ component へ書くだけで、第二の差し替え
    /// 機構を増やさない。`SetTrack`/`SetPropertySlot` を再び投げれば普通の
    /// track/slot へ戻せる(同じ場所を上書きするだけ、専用の「解除」variant は
    /// 要らない)。
    ///
    /// **循環は書き込み時に拒否する**([`validate_no_link_cycle`]) — Blender の
    /// driver 依存グラフのような実行時検出・事後警告(§1.1/§1.5、循環実測
    /// [Blender Projects #64793](https://projects.blender.org/blender/blender/issues/64793))
    /// より強い保証。`motolii-eval` の「時刻t→値の純関数」契約
    /// (`motolii-eval/src/lib.rs:16`)を壊すと Preview=Export(裁定15)の保証が崩れるため、
    /// 妥協ではなく必須の柵。
    SetPropertyLink {
        layer: LayerId,
        property: PropertyId,
        link: PropertyLink,
    },
    /// この property の modulator 列を差し替える(裁定213「接続子は加算」)。
    /// **`base`(現在の `Track`/`Slot`)は読んで保つ** — `SetPropertyLink` が
    /// base ごと置き換えるのとは違い、これは「今の値の上に、これらを足す」を
    /// 書く口(`SetAttrs` が現在の `attrs` を読んでから部分更新するのと同じ
    /// 読み-書きの形)。空の `Vec` を渡せば modulator を全部外せる(専用の
    /// 「解除」variant は要らない、既存の流儀のまま)。
    ///
    /// **循環は modulator 1本ごとに [`validate_no_link_cycle`] で拒否する**
    /// (`SetPropertyLink` と同じ柵)。
    SetPropertyModulators {
        layer: LayerId,
        property: PropertyId,
        modulators: Vec<PropertyLink>,
    },
    /// カメラの property 版(同上、`SetCameraTrack`/`SetCameraPropertySlot` と
    /// entity を分けているのと同じ形)。
    SetCameraPropertyModulators {
        property: PropertyId,
        modulators: Vec<PropertyLink>,
    },
    /// 素材と重ね順の**新規配置専用**。
    ///
    /// **既に `meta` を持つ layer には使えない**(`write` が拒む、裁定108(c))。
    /// 呼び手が読まずに組んだ `LayerMeta` で丸ごと差し替えると、他のフィールド
    /// (代表例: `timing`)が黙って初期値へ戻る事故が起きていた。既存 layer の
    /// 素材・重ね順・配置を変えるのは [`Intent::SetSource`] / [`Intent::SetOrder`] /
    /// [`Intent::SetTiming`] の**フィールド単位の口**で、これらは内部で現在の
    /// `meta` を読んでから該当フィールドだけ書き換えるので、他のフィールドを
    /// 巻き込むことが構造的にできない。
    SetMeta {
        layer: LayerId,
        meta: crate::LayerMeta,
    },
    /// 既存 layer の素材を差し替える(`meta` を読んで `source` だけ書き換える)。
    SetSource {
        layer: LayerId,
        source: crate::LayerSource,
    },
    /// 既存 layer の重ね順を変える(`meta` を読んで `order` だけ書き換える)。
    SetOrder { layer: LayerId, order: i16 },
    /// マスクの並びと重ね方。**並べ替え・削除・モード変更はこれ1つ**
    /// (`LayerTiming` と同じ考え方で、操作ごとの専用 intent を足さない)。
    ///
    /// 形状と不透明度は `SetTrack` が書く。
    ///
    /// **新しく現れる mask id は拒む**(2026-08-22 発見の欠陥の恒久修正、
    /// `docs/reviews/2026-08-22-persona-motion-round2.md` §1 壁7): 以前はここで
    /// `masks` へ新しい id を足すだけで通ってしまい、対応する `mask.{id}.shape` の
    /// `SetTrack` を書き忘れても検査を素通りしていた——`resolved_masks` が実行時に
    /// 初めて `Err` を返す壊れた Document を構造的に作れていた。今は
    /// [`validate_masks_have_shapes`] が「現在の一覧に無い id は、この呼び出しの
    /// 時点で `mask.{id}.shape` が既に読めること」を要求する。**同じ `apply_all` の
    /// 中で先に `SetTrack`(shape)を書けば通る**(各 intent は同じ edit 刻みで
    /// 逐次 ingest されるので、後続 intent の検査は先行 intent の結果を見る) ——
    /// ただし1回で足りる [`Intent::AddMask`] の方を新規追加には使うこと。
    SetMasks {
        layer: LayerId,
        masks: Vec<crate::Mask>,
    },
    /// マスクを1枚**追加する専用の口**。`SetMasks`(一覧の並べ替え・削除・mode 変更)
    /// と `SetTrack`(shape)を呼び手が2つの intent に分けて書くと、片方だけ書いて
    /// 忘れる経路が構造的に残る(上記 `SetMasks` の doc 参照、壁7)。`AddMask` は
    /// 「一覧への追加」と「shape の初期値」を**同じ `write()` 呼び出しの中で1つの
    /// chunk として ingest する**——2つの intent の順序に頼らず、そもそも
    /// 「マスクだけあって shape が無い」中間状態が物理的に存在しない。
    ///
    /// 既存マスクの並べ替え・削除・mode 変更は引き続き `SetMasks` を使う
    /// (`AddMask` は新規追加専用 — 既に居る id を渡すと [`crate::mask::validate_unique_ids`]
    /// が拒む)。
    AddMask {
        layer: LayerId,
        mask: Mask,
        /// 初期 shape。既定矩形の1キー Hold でも、複数キーのアニメーションでもよい
        /// (`SetTrack` が受け取る形と同じ)。
        shape: motolii_eval::KeyframeTrack,
    },
    /// comp 上の配置と、素材のどこを使うか。move / trim / split はすべてこれ1つ。
    SetTiming {
        layer: LayerId,
        timing: crate::LayerTiming,
    },
    /// layer の非アニメーション属性(hidden / parent / blend mode / matte / name /
    /// auto-orient)の**部分更新**([`LayerAttrsPatch`] — フィールドごとに「触るか」を
    /// 持つ)。`meta` とは別 component なので `timing`/`source`/`order` を巻き込まない。
    ///
    /// **丸ごと差し替えではない**(2026-08-20 の敵対的レビュー修正) — `write` が現在の
    /// `attrs` を読んでから `patch` の `Some` なフィールドだけ重ねるので、
    /// 「読まずに組んだ値で他フィールドが黙って戻る」呼び出しが型的に書けない。
    ///
    /// `parent` に循環参照を作ろうとすると拒まれる(layer-meta 束の柵)。
    SetAttrs {
        layer: LayerId,
        patch: LayerAttrsPatch,
    },
    /// layer が持つ effect インスタンスの列(id・plugin id・順序のみ。param 値は
    /// `effect` 束が別途扱う)。丸ごと差し替え。
    SetEffects {
        layer: LayerId,
        effects: Vec<crate::EffectInstance>,
    },
    /// shape-layer の図形列。丸ごと差し替え。裁定173 H4: `Vec<crate::ShapeNode>` —
    /// 平坦な `Shape`(`ShapeNode::Leaf`)と入れ子グループ(`ShapeNode::Group`)を
    /// 混在させられる。旧 `Vec<Shape>` を渡していた呼び手は
    /// `shapes.into_iter().map(ShapeNode::Leaf).collect()` で移行する
    /// (`shapes: Vec<crate::Shape>` という**旧型のまま渡す口は無い** — 二重の
    /// SetShapes を持たせると shape schema の正本が2つになる)。
    SetShapes {
        layer: LayerId,
        shapes: Vec<crate::ShapeNode>,
    },
    /// text-layer の中身(content・組版既定値・フォント参照)。丸ごと差し替え —
    /// `SetShapes`/`SetEffects` と同じ形。`Layer:text` component はこの意味しか
    /// 持たない(他の関心事と同居しない)ので、`SetAttrs` のような部分更新は要らない。
    SetTextDocument {
        layer: LayerId,
        document: crate::TextDocument,
    },
    /// comp の設定(解像度・fps・尺)。**undo が効く**ので普通の編集と同じ経路。
    SetComposition(crate::Composition),
    /// comp のマーカー一覧。追加・削除・並べ替え・改名はすべてこれ1つ
    /// (`SetMasks`/`SetTiming` と同じ考え方)。
    SetMarkers { markers: Vec<crate::Marker> },
    /// カメラの property(`camera.center`/`camera.zoom`/`camera.roll`、裁定113/115)。
    /// **新しい機構ではない** — `SetTrack` と同じ形で、書く先が layer ではなく
    /// `/composition` entity なだけ(`PropertyId::camera` が経路を分ける)。
    SetCameraTrack {
        property: PropertyId,
        track: motolii_eval::KeyframeTrack,
    },
    /// カメラの property をスロット参照へ切り替える。[`Intent::SetPropertySlot`] の
    /// カメラ版(`SetCameraTrack`/`SetTrack` が entity を分けているのと同じ形)。
    SetCameraPropertySlot {
        property: PropertyId,
        slot: SlotId,
    },
    /// comp の Slots 表(`composition/animation/slots`)。追加・削除・並べ替え・
    /// 値の差し替えはすべてこれ1つ(`SetMasks`/`SetMarkers` と同じ考え方)。
    SetSlots {
        slots: Vec<Slot>,
    },
    /// 素材を台帳へ迎え入れる(裁定162: bin-first、取り込んだが未配置の素材)。
    /// **同一 `content_hash` の draft は台帳を増やさず既存 id を使い回す**
    /// (`crate::AssetTable::admit` の重複統合、旧台帳の意味そのまま)。呼び手が
    /// 結果の id を知りたければ `StoreView::assets()`/`asset()` を admit 後に読む —
    /// 他の Intent と同じく `apply` 自身は値を返さない(`AddLayer` が
    /// `StoreView::next_layer_id` を先に呼ぶのと対の形: こちらは id を先に決めず、
    /// 台帳自身に重複統合させてから読み手が引く)。
    AdmitAsset {
        draft: crate::AssetDraft,
    },
    /// 台帳から素材を取り除く。**この素材を指す layer が居るかは見ない**
    /// (`LayerSource::Media` との参照統合は非目標、裁定162 の第一波)。
    RemoveAsset {
        asset: crate::AssetId,
    },
    /// 欠損素材の実体だけを別 path へ繋ぎ直す。AssetId・content hash・表示名は
    /// 保持し、layer が指す台帳の身分を変えない。project root は相対 path を
    /// 再計算するための一時的な環境情報で、Document の保存内容には入らない。
    RelinkAsset {
        asset: crate::AssetId,
        path_absolute: String,
        project_root: Option<String>,
    },
    /// **freeze 意図動詞**(裁定119、G1 に続く「意図優先の原則」の実装束)。
    /// `group` を指す `LayerAttrs.frozen` を `true` にする。`group` は present な
    /// `LayerSource::Group` layer でなければならない(そうでなければ理由つき `Err`)。
    ///
    /// **専用 Intent にした**(G1 の `group_layers`/`ungroup_layers` と違い既存語彙の
    /// 合成では表さない) — freeze は `hidden`/`solo` のような汎用属性ではなく、
    /// 「この部分木を今後編集させない」という意味の重い宣言なので、汎用の
    /// `SetAttrs`/`LayerAttrsPatch` には乗せない(`LayerAttrs::frozen` の doc 参照)。
    ///
    /// **Document の意味は1bitも変わらない**(裁定119 OUTCOME) — frozen は
    /// 「以後の編集 Intent を拒む」というゲートの状態であって、絵そのものは
    /// このフラグの前後で同一(engine 側のキャッシュ/fingerprint は後続束)。
    ///
    /// 既に frozen な Group への `Freeze` は冪等(再度 `true` を書くだけ)。
    /// `locked` な Group への `Freeze` は他の層変更 Intent と同じく拒む
    /// (`check_not_locked`) — freeze も「この layer の状態を変える」書き込みである
    /// ことに変わりはない。祖先に凍結中の Group が居る場合も拒む
    /// (`check_not_frozen`) — 凍結中の部分木の中でさらに凍結状態を動かすのも
    /// 「中身への編集」の一種(先に外側を unfreeze すること)。
    Freeze {
        group: LayerId,
    },
    /// **unfreeze 意図動詞**。`group` の `LayerAttrs.frozen` を `false` に戻すだけ
    /// (裁定119「解凍 = flag を戻すだけで、何も失われない」)。検証・冪等性・
    /// locked/frozen 祖先の扱いは [`Intent::Freeze`] と対称。
    Unfreeze {
        group: LayerId,
    },
}

/// 「見えている Document が変わったか」の印。
///
/// store の世代だけでは undo/redo を捉えられないので、edit 位置と一組にしてある。
///
/// **transient overlay(下記)は含まない** — `revision()` は履歴の意味だけを表す。
/// overlay の変化だけを理由に再描画したい呼び手は [`Document::display_revision`] を
/// 見ること。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Revision {
    store: re_chunk_store::ChunkStoreGeneration,
    head: i64,
}

/// 「見えている絵が変わったか」の印。**再描画専用** — undo/redo/保存の意味には一切
/// 関わらない。`revision()`(履歴)と transient overlay の世代を一組にしてあるので、
/// ドラッグ中に overlay だけが動いても front はここを見れば再描画できる。
///
/// front は `revision()` ではなくこちらを見ること(裁定: ドラッグ中の途中経過は
/// 履歴に入れないが、再描画は要る)。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayRevision {
    revision: Revision,
    transient_generation: u64,
}

/// transient overlay 1件の宛先。**layer property もカメラ property も同じ形**
/// (`Intent::SetTrack`/`Intent::SetCameraTrack` が entity を分けているのと同じ線引き、
/// `PropertyId::camera` の doc 参照)。
///
/// `PropertyId` 単体では「どの layer か」を持たない(`Layer:position` は全 layer で
/// 同じ component 識別子)ので、layer をまたいで同名 property を持つ Document で
/// overlay を安全に効かせるには scope が要る — これが無いと、layer A の `position` を
/// ドラッグ中に layer B の `position` を読んでも overlay 値が誤って返ってしまう
/// (`resolved_layers` は comp の全 layer を毎フレーム評価するので、この誤爆は
/// 理論上ではなく実際に毎フレーム起こる)。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum TransientKey {
    Layer(LayerId, PropertyId),
    Camera(PropertyId),
}

/// 解析済み `PropertySource`(= `serde_json` で `TrackJson` を解いた結果)の
/// revision 鍵キャッシュ(裁定140)。
///
/// **`track()` コストの97%が serde_json 解析だった**(2026-08-21 計測、
/// `next/reference/KNOWN.md`)— `KeyframeTrack` まるごと1 component にした代償
/// (裁定11)が投影の読み側に出ていた。置き場は front ではなく**単一 writer(store)の
/// 読み口の裏**一択(R2 probe の目的そのもの、裁定140 の doc 参照)。
///
/// 無効化は [`Document::revision`] との比較で機械的に行う — **手動 invalidate 口は
/// 持たない**。`apply` の後に呼び手が何かを呼び忘れて古い値が残る事故を型で塞ぐ。
/// キー(`TransientKey`)は transient overlay と全く同じ「layer property か camera
/// property か」の scope なので、新しい鍵の形を増やさずに使い回す。
#[derive(Default)]
pub(crate) struct TrackCache {
    revision: Option<Revision>,
    entries: HashMap<TransientKey, Option<PropertySource>>,
}

impl TrackCache {
    /// `current` が前回と違えば中身を丸ごと捨てる。呼び手ごとに古さを判定させず、
    /// ここへ一括する。
    fn sync(&mut self, current: &Revision) {
        if self.revision.as_ref() != Some(current) {
            self.entries.clear();
            self.revision = Some(current.clone());
        }
    }

    /// `key` が無ければ `miss` を呼んで解析し、結果をキャッシュしてから返す。
    /// RefCell の borrow を1回に畳むため get/put を分けない。
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
    /// 現在の edit 位置。0 = 空の Document。
    head: i64,
    /// 到達済みの最大 edit 位置。redo の上限。
    tip: i64,
    /// undo の底。**ここより前へは戻れない**。
    ///
    /// 起動直後に置いた既定の comp や、project を開いた直後の状態は「編集」ではないので
    /// 戻せてはいけない。戻せると Stage が理由もなく空になる(実際に起きた)。
    floor: i64,
    /// 非履歴の overlay(タスク#20 の恒久解)。**edit timeline には一切書かない** —
    /// undo/redo/保存/`revision()` の履歴意味に無関係。`StoreView::value_at` が
    /// track の評価より優先して読む(`track()` 自身は読まない、下記 doc 参照)。
    ///
    /// ドラッグ中の途中経過はここに置き、確定時は呼び手が通常の `Intent` を1発
    /// `apply` してから [`Self::clear_transient`] する — 1 gesture が自然に 1 undo に
    /// なる。キャンセルは `clear_transient` だけで履歴は無傷のまま。
    transient: HashMap<TransientKey, Value>,
    /// overlay の世代。[`Self::display_revision`] に混ぜて再描画のためだけに使う。
    /// `revision()` には混ぜない(履歴の意味を変えないため)。
    transient_generation: u64,
    /// 解析済み track の revision 鍵キャッシュ(裁定140)。`StoreView` は `&self` の
    /// 借用しか持たないので `RefCell` — 可変なのはキャッシュだけで、Document の
    /// 意味上の状態(履歴・overlay)は今までどおり `apply`/`set_transient` 経由でしか
    /// 動かない。
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

    /// 同一性を指定して作る。**読込のためだけの口** — file が持つ id をそのまま使う。
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

    /// comp 設定の置き場。layer(`/layer/{id}`)と混ざらない固定の path。
    pub(crate) fn composition_path() -> EntityPath {
        EntityPath::from("/composition")
    }

    fn timeline() -> Timeline {
        Timeline::new_sequence(EDIT_TIMELINE)
    }

    fn timeline_name() -> TimelineName {
        *Self::timeline().name()
    }

    /// 読み手が受け取る唯一の物。可変ハンドルは外へ出さない。
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

    /// 読み込んだ store から edit 位置を復元する。
    ///
    /// 保存は履歴を畳むので刻みは1つだが、**それを決め打ちにしない** — store に聞く。
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

    /// 今の状態を **undo の底**にする。
    ///
    /// 「新規作成した」「project を開いた」の直後に呼ぶ。ここより前は編集ではないので
    /// 戻せない。呼ばないと、起動時に置いた既定値を利用者が undo で消せてしまう。
    pub fn mark_undo_floor(&mut self) {
        self.floor = self.head;
    }

    pub fn can_undo(&self) -> bool {
        self.head > self.floor
    }

    pub fn can_redo(&self) -> bool {
        self.head < self.tip
    }

    /// **時間を戻すだけ**。store からは何も失われない。
    pub fn undo(&mut self) -> bool {
        if self.can_undo() {
            self.head -= 1;
            true
        } else {
            false
        }
    }

    /// **時間を進めるだけ**。
    pub fn redo(&mut self) -> bool {
        if self.can_redo() {
            self.head += 1;
            true
        } else {
            false
        }
    }

    /// **1操作 = 1 undo**。複数の intent をまとめて1つの edit 刻みへ書く。
    ///
    /// 「layer を置く」のような操作は本来 `AddLayer` + `SetMeta` + `SetTrack` の
    /// 複数 intent だが、利用者から見れば1操作である。個別に `apply` すると
    /// **1操作を戻すのに Undo を何回も押すことになる**(ui-quality-bar Q2)。
    ///
    /// ドラッグは対象外 — 途中経過は pane が持ち、**確定の1件だけが intent** なので
    /// もともと1 undo になる。ここが要るのは「本質的に複数 intent な1操作」だけ。
    ///
    /// **原子性**(2026-08-20 の敵対的レビュー修正): バッチ内の intent が1つでも `Err`
    /// を返したら、**バッチ全体を無かったことにする** — `view()` が呼ぶ前と一致する。
    /// 修正前は `write` が intent ごとに即 `ingest`(store へ追記 + `head`/`tip` 前進)
    /// していたので、`apply_all([正当, 不正])` は「正当な分だけ store に確定し、
    /// `head` も進んだまま `Err` を返す」という部分コミットになっていた。
    ///
    /// **実装**: バッチ内の全 intent は同じ edit 刻み `at` へ書く。失敗したら、
    /// その `at` に書いた分をこの batch の途中経過ごと畳んで(`drop_time_range`)
    /// `head`/`tip` をバッチ前の値へ戻す。**undo 履歴には何も残らない** —
    /// この batch は最初から `head` を進めていないので、undo/redo の意味
    /// (裁定2「undo は時間の移動」/ 裁定47「undo floor」/ 裁定48「redo は時間を
    /// 進めるだけ」)は一切変わらない。
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

    /// 失敗した batch がこの edit 刻みに残した分を消し、`head`/`tip` を batch 前へ戻す。
    /// **`drop_redo_space` と同じ変種**(上流が undo/redo スタック操作用に用意した
    /// `ExplicitDrop`)を使う — この batch は一度も「確定した編集」になっていないので、
    /// undo で戻すべき履歴ではなく、単に「無かったことにする」対象である。
    fn discard_batch_at(&mut self, at: i64, original_head: i64, original_tip: i64) {
        self.db.drop_time_range(
            &Self::timeline_name(),
            AbsoluteTimeRange::new(at, at),
            re_chunk_store::ChunkDeletionReason::ExplicitDrop,
        );
        self.head = original_head;
        self.tip = original_tip;
    }

    /// 唯一の書き口。
    ///
    /// undo 後に新しい編集をしたら redo 空間を落とす — rerun blueprint と同じ規則
    /// (`re_viewer_context/src/undo.rs`: "When editing, we first drop all data after
    /// the current time.")。
    pub fn apply(&mut self, intent: Intent) -> Result<(), StoreError> {
        self.apply_all([intent])
    }

    /// undo 後に新しい編集をしたら redo 空間を落とす — rerun blueprint と同じ規則。
    fn drop_redo_space(&mut self) {
        if self.head < self.tip {
            self.db.drop_time_range(
                &Self::timeline_name(),
                AbsoluteTimeRange::new(self.head + 1, self.tip),
                // 上流が undo/redo スタック操作のために用意している変種をそのまま使う。
                re_chunk_store::ChunkDeletionReason::ExplicitDrop,
            );
            self.tip = self.head;
        }
    }

    /// 唯一の物理書き口(`Intent` の意味づけを終えた [`SerializedComponentBatch`] を
    /// 1つの chunk として store へ足す)。`write` の末尾と [`Self::copy_track_json`]
    /// (`persist.rs` の `flattened()` 専用)が両方ここへ落ちる — **書き口が2系統に
    /// 分岐しても、物理的な追記経路は1本のまま**。
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

    /// `flattened()` 専用: component の意味を知らずに、読んだ JSON をそのまま
    /// 別の component へコピーする。**store に聞く**(裁定57/108(a))形の核 —
    /// `persist.rs` が `meta`/`masks`/`attrs`/… を名前で列挙しなくてよいのは、
    /// この口が「どんな component 名でも」コピーできるため。新しい component を
    /// 足しても `flattened()` を直さなくてよい。
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

    /// 変化検出。front がこれを見れば「前回と同じか」が分かるので、
    /// **前回の値を自分で持つ必要が無い**。二重帳簿の入口を1つ塞ぐための口である。
    ///
    /// **上流の `EntityDb::generation` だけでは足りない**(2026-08-20 の敵対的レビュー):
    /// `undo`/`redo` は `head` を動かすだけで store に触らないので generation が変わらず、
    /// **undo しても front が再描画しない**。それでは front が `last_edit_head` を自分で
    /// 持つことになり、塞ぐと言った入口が逆に開く。よって **(store の世代, edit 位置)** を
    /// 一組で返す。
    pub fn revision(&self) -> Revision {
        Revision {
            store: self.db.generation(),
            head: self.head,
        }
    }

    /// 再描画専用の変化検出。[`Self::revision`] に transient overlay の世代を
    /// 混ぜたもの — overlay だけが動いた(ドラッグ中)場合も front はここを見れば
    /// 再描画できる。`revision()` 自体は overlay で動かない(履歴の意味を保つため)。
    pub fn display_revision(&self) -> DisplayRevision {
        DisplayRevision {
            revision: self.revision(),
            transient_generation: self.transient_generation,
        }
    }

    /// layer property の overlay を置く(無ければ足す、あれば置き換える)。**edit
    /// timeline には一切触れない** — undo/redo/保存/`revision()` の履歴意味に無関係。
    /// [`StoreView::value_at`] が track の評価より優先して読む。`StoreView::track` は
    /// 読まない(生の意味だけを返す線引きは変えない)。
    ///
    /// ドラッグ中は mouse-move ごとにここを呼ぶだけでよい(history には一切触れない
    /// ので、以前のような「undo してから apply し直す」squash は不要になる)。
    pub fn set_transient(&mut self, layer: LayerId, property: PropertyId, value: Value) {
        self.transient
            .insert(TransientKey::Layer(layer, property), value);
        self.bump_transient_generation();
    }

    /// カメラ property 版(`Intent::SetCameraTrack`/`Intent::SetTrack` が entity を
    /// 分けているのと同じ形)。
    pub fn set_camera_transient(&mut self, property: PropertyId, value: Value) {
        self.transient.insert(TransientKey::Camera(property), value);
        self.bump_transient_generation();
    }

    /// この layer property の overlay を外す。**キャンセルはこれだけでよい**(履歴は
    /// 最初から無傷)。存在しない宛先を指定しても何も起きない(黙って no-op)。
    pub fn clear_transient(&mut self, layer: LayerId, property: &PropertyId) {
        if self
            .transient
            .remove(&TransientKey::Layer(layer, property.clone()))
            .is_some()
        {
            self.bump_transient_generation();
        }
    }

    /// カメラ property 版(同上)。
    pub fn clear_camera_transient(&mut self, property: &PropertyId) {
        if self
            .transient
            .remove(&TransientKey::Camera(property.clone()))
            .is_some()
        {
            self.bump_transient_generation();
        }
    }

    /// 今持っている overlay を全部外す。**確定/キャンセルの両方で最後に呼んでよい
    /// 保険口** — 個別の宛先を覚えていなくても、ジェスチャの終わりにこれ1つで
    /// overlay を必ず空にできる。
    pub fn clear_all_transients(&mut self) {
        if !self.transient.is_empty() {
            self.transient.clear();
            self.bump_transient_generation();
        }
    }

    fn bump_transient_generation(&mut self) {
        self.transient_generation = self.transient_generation.wrapping_add(1);
    }

    /// 実測用。製品経路ではない。
    pub fn store_bytes(&self) -> u64 {
        self.db.byte_size_of_physical_chunks()
    }

    /// 実測用。製品経路ではない。
    pub fn store_chunks(&self) -> usize {
        self.db.num_physical_chunks()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// defect 4 の再現(2026-08-20 の敵対的レビュー):
    /// `StoreView::track_json_components`(→ `flattened()`/`save()` の核)は
    /// `Layer:present` だけを別扱いし、それ以外は全部 `TrackJson` として読める前提で
    /// 書かれていた。**別 `Loggable` 型の component が増えると、黙って保存から消えていた**
    /// — `component_batch::<TrackJson>` が型不一致で `None` を返すのを「値が無い」と
    /// 同じ扱いで `filter_map` が飲み込んでいたため。
    ///
    /// この試験は `Document::ingest`(private、同一モジュール内なので白箱で叩ける)を
    /// 直接使い、`present` ではない component 名に `LayerPresent`(bool)という
    /// 別 `Loggable` 型の値を直に置く — 「将来 present 以外にも別型の component が
    /// 増えた日」の模倣。**今は黙って消えず `Err` になる**ことを固定する。
    #[test]
    fn a_non_track_json_component_other_than_present_is_reported_not_silently_dropped() {
        let mut doc = Document::new();
        let layer = LayerId(99);
        doc.apply(Intent::AddLayer(layer)).unwrap();

        let bogus = re_types_core::ComponentDescriptor {
            archetype: Some("motolii.archetypes.Layer".into()),
            component: "Layer:bogus".into(),
            component_type: Some(<LayerPresent as Component>::name()),
        };
        let batch = SerializedComponentBatch {
            descriptor: bogus,
            array: <LayerPresent as re_types_core::Loggable>::to_arrow([LayerPresent(true)])
                .unwrap(),
        };
        let at = doc.head + 1;
        doc.ingest(layer.entity_path(), vec![batch], at).unwrap();

        let result = doc.flattened();
        assert!(
            result.is_err(),
            "TrackJson でない component(present 以外)を静かに落としてしまっている \
             (flattened()/save() から黙って消える)"
        );
    }
}
