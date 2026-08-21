//! Document 本体 — 書き口1本 + undo/redo。

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

use crate::components::{
    descriptor_assets, descriptor_attrs, descriptor_composition, descriptor_effects,
    descriptor_markers, descriptor_masks, descriptor_meta, descriptor_present, descriptor_shapes,
    descriptor_slots, descriptor_text, descriptor_track, LayerPresent, TrackJson,
};
use crate::slot::PropertySource;
use crate::view::StoreView;
use crate::{LayerAttrsPatch, Slot, SlotId, StoreError, EDIT_TIMELINE};

/// layer の安定 ID。entity path はこれ1つから決まる。
///
/// `Serialize`/`Deserialize` は layer-meta 束で足した — `LayerAttrs.parent` /
/// `Matte.layer` が参照として持つため(裁定65 の「参照は `LayerId`」をそのまま辿れる)。
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct LayerId(pub u64);

impl LayerId {
    pub fn entity_path(self) -> EntityPath {
        EntityPath::from(format!("/layer/{}", self.0))
    }
}

/// property の名前。AE の property list の1行に相当する。
///
/// 構築時に component 識別子まで解決しておく。`ComponentIdentifier` は空文字を拒む
/// interned 型なので、**検証を境界で1回だけ**行い、以降は失敗し得ない形にする。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PropertyId {
    name: String,
    component: re_types_core::ComponentIdentifier,
}

impl PropertyId {
    pub fn new(name: &str) -> Result<Self, StoreError> {
        if crate::property::RESERVED.contains(&name) {
            return Err(StoreError::Property(format!(
                "`{name}` は layer 自身の component 名なので property に使えない"
            )));
        }
        let component = re_types_core::ComponentIdentifier::try_new(format!("Layer:{name}"))
            .map_err(|e| StoreError::Property(e.to_string()))?;
        Ok(Self {
            name: name.to_owned(),
            component,
        })
    }

    /// マスクの形状トラック。**id で名前が決まる**ので、並べ替えても消しても
    /// 別のマスクへ付き直さない。
    ///
    /// 標準 property と同じく予約語でも空でもないので、構築は失敗し得ない。
    pub fn mask_shape(mask: crate::MaskId) -> Self {
        Self::mask_property(mask, "shape")
    }

    /// マスクの不透明度トラック(1.0 基準)。
    pub fn mask_opacity(mask: crate::MaskId) -> Self {
        Self::mask_property(mask, "opacity")
    }

    fn mask_property(mask: crate::MaskId, attr: &str) -> Self {
        let name = format!("{}{mask}.{attr}", crate::property::MASK_PREFIX);
        Self::new(&name).expect("マスクの property 名は予約語でも空でもない")
    }

    /// effect インスタンスの param トラック(`effects/effect ef`、effect 発注単位)。
    /// **平坦な名前**(`effect.{id}.param.{name}`)にしてあるので、マスク/テキストと
    /// 同じく新しい機構を足さずに `KeyframeTrack` へ乗る(裁定72)。`name` は plugin が
    /// 決める param 名で、値は `motolii_eval::Value` の該当バリアント
    /// (`F64`/`Bool`/`Enum`/`Color`/`Vec2`/`LayerId`)。
    pub fn effect_param(effect: crate::EffectId, name: &str) -> Result<Self, StoreError> {
        let property_name = format!("{}{effect}.param.{name}", crate::property::EFFECT_PREFIX);
        Self::new(&property_name)
    }

    /// アニメーターの selector が動かす値(`text-range-selector s`/Start)。**平坦な
    /// 名前**(`text_range.{id}.selector.{attr}`)にしてあるので、マスク/effect と
    /// 同じく新しい機構を足さずに `KeyframeTrack` へ乗る。
    pub fn text_range_selector_start(range: crate::TextRangeId) -> Self {
        Self::text_range_selector_property(range, "start")
    }

    /// `text-range-selector e`(End)。
    pub fn text_range_selector_end(range: crate::TextRangeId) -> Self {
        Self::text_range_selector_property(range, "end")
    }

    /// `text-range-selector o`(Offset)。**カラオケワイプはこれを時間駆動するだけに
    /// 畳める**(offset に打った track を再生位置で動かすだけで実現できる、地図の
    /// note どおり)。
    pub fn text_range_selector_offset(range: crate::TextRangeId) -> Self {
        Self::text_range_selector_property(range, "offset")
    }

    /// `text-range-selector a`(Max Amount)。重みの倍率。
    pub fn text_range_selector_max_amount(range: crate::TextRangeId) -> Self {
        Self::text_range_selector_property(range, "max_amount")
    }

    fn text_range_selector_property(range: crate::TextRangeId, attr: &str) -> Self {
        let name = format!(
            "{}{range}.selector.{attr}",
            crate::property::TEXT_RANGE_PREFIX
        );
        Self::new(&name).expect("text-range の property 名は予約語でも空でもない")
    }

    /// アニメーターが動かす property の束(`text-range a` Style、Lottie `text-style`
    /// 側の5フィールド)。**track の有無自体が「この animator がその属性を触るか」を
    /// 表す**(裁定20 の応用)。`text-style` が継承する `helpers/transform`
    /// (position/scale/rotation/opacity/skew)はこの切片では持たない — Rive の
    /// `text-modifier-group`(次切片)が正本を持つまでの持ち越し。
    pub fn text_range_fill_color(range: crate::TextRangeId) -> Self {
        Self::text_range_style_property(range, "fill_color")
    }

    /// `text-style sc`(Stroke Color)。
    pub fn text_range_stroke_color(range: crate::TextRangeId) -> Self {
        Self::text_range_style_property(range, "stroke_color")
    }

    /// `text-style sw`(Stroke Width)。
    pub fn text_range_stroke_width(range: crate::TextRangeId) -> Self {
        Self::text_range_style_property(range, "stroke_width")
    }

    /// `text-style ls`(Line Spacing)。**組版に触る**アニメーター(裁定76 — 送り幅/
    /// 行送りを動かすので2層分離では足りない)。
    pub fn text_range_line_spacing(range: crate::TextRangeId) -> Self {
        Self::text_range_style_property(range, "line_spacing")
    }

    /// `text-style t`(Letter Spacing)。トラッキング。**アニメーターだが送り幅を
    /// 動かす**(裁定76)。
    pub fn text_range_tracking(range: crate::TextRangeId) -> Self {
        Self::text_range_style_property(range, "tracking")
    }

    fn text_range_style_property(range: crate::TextRangeId, attr: &str) -> Self {
        let name = format!(
            "{}{range}.style.{attr}",
            crate::property::TEXT_RANGE_PREFIX
        );
        Self::new(&name).expect("text-range の property 名は予約語でも空でもない")
    }

    /// アニメーターがグリフに適用する変形(Rive `text-modifier-group`)。**アンカー**
    /// (`originX`/`originY`)。既定=字面中心(地図の note どおり、`text-alignment-options a`
    /// が同じ意味を Alignment 側に持つ)。
    pub fn text_range_origin(range: crate::TextRangeId) -> Self {
        Self::text_range_transform_property(range, "origin")
    }

    /// `text-modifier-group opacity`。適用先がグリフになるだけで、意味は普通の不透明度。
    pub fn text_range_opacity(range: crate::TextRangeId) -> Self {
        Self::text_range_transform_property(range, "opacity")
    }

    /// `text-modifier-group x`/`y`。Rive の `"group": "position"` が「保存が2成分に
    /// 割れているだけで意味は Vec2」と明示しており、裁定61(position は Vec2 単一
    /// property)と衝突しない。
    pub fn text_range_position(range: crate::TextRangeId) -> Self {
        Self::text_range_transform_property(range, "position")
    }

    /// `text-modifier-group rotation`。グリフ適用の回転(度)。
    pub fn text_range_rotation(range: crate::TextRangeId) -> Self {
        Self::text_range_transform_property(range, "rotation")
    }

    /// `text-modifier-group scaleX`/`scaleY`。`"group": "scale"` で Vec2(`position` と
    /// 同じ理由)。
    pub fn text_range_scale(range: crate::TextRangeId) -> Self {
        Self::text_range_transform_property(range, "scale")
    }

    fn text_range_transform_property(range: crate::TextRangeId, attr: &str) -> Self {
        let name = format!(
            "{}{range}.transform.{attr}",
            crate::property::TEXT_RANGE_PREFIX
        );
        Self::new(&name).expect("text-range の property 名は予約語でも空でもない")
    }

    /// `text-variation-modifier axisValue`(Δ)。裁定76 の3層のうち「再シェープする層」
    /// の唯一の住人 — [`crate::TextRange::variation_axes`] が持つタグに対応する動く量。
    /// スパン側の絶対値([`Self::text_style_axis`])とは別の track(層が違う、二重帳簿
    /// ではない、地図の note どおり)。
    pub fn text_range_variation_axis(range: crate::TextRangeId, tag: &str) -> Self {
        let name = format!(
            "{}{range}.variation.{tag}",
            crate::property::TEXT_RANGE_PREFIX
        );
        Self::new(&name).expect("text-range の property 名は予約語でも空でもない")
    }

    /// `text-style-axis axisValue`。スタイル表の行が持つ可変フォント軸の**絶対値**。
    /// **裁定92 の唯一の例外** — 他のスタイル属性は v1 で静止だが、軸値はシェーピングの
    /// 入力そのものなので P6「軸だけはスタイル層でアニメ可」に当たる(裁定93)。
    pub fn text_style_axis(style: crate::TextStyleId, tag: &str) -> Self {
        let name = format!("{}{style}.axis.{tag}", crate::property::TEXT_STYLE_PREFIX);
        Self::new(&name).expect("text-style の property 名は予約語でも空でもない")
    }

    /// カメラの property(`Composition.camera` の center/zoom/roll、裁定113/115)。
    ///
    /// **layer とは別の entity(`/composition`)へ書く**ので、component 識別子の
    /// 名前空間も分ける(`Composition:{name}` — layer 側の `Layer:{name}` と
    /// 衝突しない)。`RESERVED` は layer 自身の component 名(`meta`/`present`/…)を
    /// 弾く仕組みなので、別名前空間のここには適用しない(そもそも衝突しない)。
    pub fn camera(name: &str) -> Result<Self, StoreError> {
        let component = re_types_core::ComponentIdentifier::try_new(format!("Composition:{name}"))
            .map_err(|e| StoreError::Property(e.to_string()))?;
        Ok(Self {
            name: name.to_owned(),
            component,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn component(&self) -> re_types_core::ComponentIdentifier {
        self.component
    }
}

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
    /// 発注単位)。**`SetTrack` と書く先は同じ component** — `PropertySource::Slot` を
    /// そこへ書くだけで、新しい component は増やさない(地図の note「第二の差し替え
    /// 機構を作らない」)。`SetTrack` を再び投げれば普通の track へ戻せる(同じ場所を
    /// 上書きするだけなので、専用の「解除」variant は要らない)。
    SetPropertySlot {
        layer: LayerId,
        property: PropertyId,
        slot: SlotId,
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
    /// マスクの並びと重ね方。**追加・削除・並べ替え・モード変更はすべてこれ1つ**
    /// (`LayerTiming` と同じ考え方で、操作ごとの専用 intent を足さない)。
    ///
    /// 形状と不透明度は `SetTrack` が書く。「マスクを1枚足す」は
    /// `SetMasks` + `SetTrack` を `apply_all` で束ねた **1操作 = 1 undo** になる。
    SetMasks {
        layer: LayerId,
        masks: Vec<crate::Mask>,
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

    /// 唯一の意味づけ書き口。`pub(crate)` なのは `persist.rs` が `AddLayer` を
    /// 同じ edit 刻みで書くため(履歴を畳む = 1 tick にまとめる、裁定56)。
    pub(crate) fn write(&mut self, intent: Intent, at: i64) -> Result<(), StoreError> {
        let batches = match intent {
            Intent::AddLayer(layer) => (layer.entity_path(), vec![serialize_present(true)?]),
            Intent::RemoveLayer(layer) => {
                // 削除は最も破壊的な編集(元に戻すには undo するしかない)。locked は
                // 他の層変更 Intent と同じく理由つき Err で拒む(supervisor 裁定、
                // AE と同じ意味論)。解除→削除の2手は常に可能 — `check_not_locked` は
                // `locked` 自身の解除/再ロックだけを別扱いする `SetAttrs` を経由しない。
                check_not_locked(&self.view(), layer)?;
                (layer.entity_path(), vec![serialize_present(false)?])
            }
            Intent::SetComposition(composition) => {
                let json = serde_json::to_string(&composition)?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_composition(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetMarkers { markers } => {
                let json = serde_json::to_string(&markers)?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_markers(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetMasks { layer, masks } => {
                check_not_locked(&self.view(), layer)?;
                crate::mask::validate_unique_ids(&masks)?;
                let json = serde_json::to_string(&masks)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_masks(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetTiming { layer, timing } => {
                check_not_locked(&self.view(), layer)?;
                // meta の一部なので、読んで差し替えて書き戻す。
                // **専用の component を足さない** — 増やすと読み口も増える。
                let current = self.view().meta(layer)?;
                let Some(mut meta) = current else {
                    return Err(StoreError::Property(format!(
                        "layer {} に素材が置かれていないので配置を決められない",
                        layer.0
                    )));
                };
                meta.timing = timing;
                let json = serde_json::to_string(&meta)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_meta(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetMeta { layer, meta } => {
                // **新規配置専用**(裁定108(c))。既に meta があるのに丸ごと差し替えを
                // 許すと、呼び手が読まずに組んだ値で timing/source/order のどれかが
                // 黙って戻る事故が構造的に作れてしまう。既存 layer は
                // SetSource/SetOrder/SetTiming のフィールド単位の口を使うこと。
                if self.view().meta(layer)?.is_some() {
                    return Err(StoreError::Property(format!(
                        "layer {} は既に meta を持つ。SetMeta は新規配置専用 — 既存 layer の \
                         素材/重ね順/配置を変えるには SetSource/SetOrder/SetTiming を使うこと",
                        layer.0
                    )));
                }
                let json = serde_json::to_string(&meta)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_meta(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetSource { layer, source } => {
                check_not_locked(&self.view(), layer)?;
                let current = self.view().meta(layer)?;
                let Some(mut meta) = current else {
                    return Err(StoreError::Property(format!(
                        "layer {} に meta が無い(先に SetMeta で配置すること)",
                        layer.0
                    )));
                };
                meta.source = source;
                let json = serde_json::to_string(&meta)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_meta(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetOrder { layer, order } => {
                check_not_locked(&self.view(), layer)?;
                let current = self.view().meta(layer)?;
                let Some(mut meta) = current else {
                    return Err(StoreError::Property(format!(
                        "layer {} に meta が無い(先に SetMeta で配置すること)",
                        layer.0
                    )));
                };
                meta.order = order;
                let json = serde_json::to_string(&meta)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_meta(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetAttrs { layer, patch } => {
                // read-modify-write — `attrs` が無い layer への初回書き込みは
                // `LayerAttrs::default()` を土台にする(`meta` と違い、属性は元々
                // 省略可能なので「まだ無い」ことがエラーではない)。
                let current = self.view().attrs(layer)?.unwrap_or_default();
                // locked は `locked` 自身の解除(または再ロック)だけ常に通す —
                // 他のどれか1つでも `Some` なら「locked 以外のフィールド」を触ろうと
                // しているので拒む。自分をロックしたら二度と触れなくなる詰みを
                // 作らないよう、`locked` を触るだけの patch は現在の locked 状態に
                // 関わらず素通しする。
                if current.locked {
                    let touches_other_than_locked = patch.hidden.is_some()
                        || patch.parent.is_some()
                        || patch.blend_mode.is_some()
                        || patch.matte.is_some()
                        || patch.name.is_some()
                        || patch.auto_orient.is_some()
                        || patch.pinned.is_some()
                        || patch.solo.is_some()
                        || patch.label_color.is_some();
                    if touches_other_than_locked {
                        return Err(StoreError::Property(format!(
                            "layer {} は locked なので attrs を変更できない(先に \
                             locked を外すこと)",
                            layer.0
                        )));
                    }
                }
                if let Some(new_parent) = patch.parent {
                    validate_no_parent_cycle(&self.view(), layer, new_parent)?;
                }
                let attrs = patch.apply_to(current);
                let json = serde_json::to_string(&attrs)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_attrs(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetEffects { layer, effects } => {
                check_not_locked(&self.view(), layer)?;
                crate::effect::validate_unique_ids(&effects)?;
                let json = serde_json::to_string(&effects)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_effects(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetShapes { layer, shapes } => {
                check_not_locked(&self.view(), layer)?;
                let json = serde_json::to_string(&shapes)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_shapes(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetTextDocument { layer, document } => {
                check_not_locked(&self.view(), layer)?;
                crate::text::validate(&document)?;
                let json = serde_json::to_string(&document)?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_text(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetTrack {
                layer,
                property,
                track,
            } => {
                check_not_locked(&self.view(), layer)?;
                // **`PropertySource::Track` でラップして書く**(`slot` 発注単位)。
                // untagged なので wire 形は今までの `KeyframeTrack` の JSON と同じ —
                // 既存の呼び手・読み手(`view.track()`)は何も変わらない。
                let json = serde_json::to_string(&PropertySource::Track(track))?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_track(&property),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetCameraTrack { property, track } => {
                let json = serde_json::to_string(&PropertySource::Track(track))?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_track(&property),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetPropertySlot {
                layer,
                property,
                slot,
            } => {
                check_not_locked(&self.view(), layer)?;
                let json = serde_json::to_string(&PropertySource::Slot(slot))?;
                (
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_track(&property),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetCameraPropertySlot { property, slot } => {
                let json = serde_json::to_string(&PropertySource::Slot(slot))?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_track(&property),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetSlots { slots } => {
                crate::slot::validate_unique_ids(&slots)?;
                let json = serde_json::to_string(&slots)?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_slots(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::AdmitAsset { draft } => {
                // read-modify-write — `SetTiming`/`SetSource` と同じ形(裁定162):
                // 現在の台帳を読み、`admit` の重複統合を経てから丸ごと書き戻す。
                let mut table = self.view().assets_table()?;
                table
                    .admit(draft)
                    .map_err(|e| StoreError::Property(e.to_string()))?;
                let json = serde_json::to_string(&table)?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_assets(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::RemoveAsset { asset } => {
                let mut table = self.view().assets_table()?;
                table
                    .remove(asset)
                    .map_err(|e| StoreError::Property(e.to_string()))?;
                let json = serde_json::to_string(&table)?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_assets(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
        };

        let (path, batches) = batches;
        self.ingest(path, batches, at)
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

/// `parent` の親鎖を辿って `layer` 自身へ戻ってこないことを確かめる。
///
/// **循環参照は絶対に作れない**(layer-meta 束の柵)。作れると、親を辿って transform を
/// 合成する日(未実装、resolve はまだ parent を読んでいない)に無限ループになる。
/// `seen` は防御的な保険 — 既存の親鎖が(バグ等で)既に壊れて循環していても、
/// この呼び出しが無限に回らないようにする。
fn validate_no_parent_cycle(
    view: &StoreView,
    layer: LayerId,
    new_parent: Option<LayerId>,
) -> Result<(), StoreError> {
    let mut current = new_parent;
    let mut seen = std::collections::HashSet::new();
    while let Some(candidate) = current {
        if candidate == layer {
            return Err(StoreError::Property(format!(
                "layer {} の parent を layer {} にすると循環参照になる(親鎖を辿ると \
                 自分自身へ戻ってくる)",
                layer.0,
                new_parent.expect("new_parent が None なら親鎖を辿らない").0
            )));
        }
        if !seen.insert(candidate) {
            break;
        }
        current = view
            .attrs(candidate)
            .ok()
            .flatten()
            .and_then(|attrs| attrs.parent);
    }
    Ok(())
}

/// locked な layer への層変更 Intent を拒む(`SetAttrs` は別扱い — 上の
/// `Intent::SetAttrs` 腕を参照。`locked` 自身の解除/再ロックだけ通す規則は
/// `SetAttrs` にしか無い、他の Intent には「触ってよい locked 以外のフィールド」が
/// 無いので単純に全拒否でよい)。
fn check_not_locked(view: &StoreView, layer: LayerId) -> Result<(), StoreError> {
    if view.attrs(layer)?.unwrap_or_default().locked {
        return Err(StoreError::Property(format!(
            "layer {} は locked なので編集できない(先に SetAttrs で locked を外すこと)",
            layer.0
        )));
    }
    Ok(())
}

fn serialize_present(present: bool) -> Result<SerializedComponentBatch, StoreError> {
    Ok(SerializedComponentBatch {
        descriptor: descriptor_present(),
        array: <LayerPresent as re_types_core::Loggable>::to_arrow([LayerPresent(present)])
            .map_err(|e| StoreError::Chunk(e.to_string()))?,
    })
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
