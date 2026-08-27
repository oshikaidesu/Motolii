//! layer 安定 ID(`LayerId`)と property 識別子(`PropertyId`)。
//! `document.rs` から移送(裁定220 SP-3、中身は変えていない)。

use re_log_types::EntityPath;
use serde::{Deserialize, Serialize};

use crate::StoreError;

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

    /// マスクの膨張(px、単一スカラー)。**Lottie 実在語彙**
    /// (`next/reference/lottie-coverage.tsv` 行197 `helpers mask x`(Expand)——
    /// 旧 verdict は「不採用」だったが B02 で撤回、`mask.rs` 冒頭「2026-08-22」節参照)。
    /// **キーフレーム可能**(AE 実機で Mask Opacity/Shape と同じくタイムラインで動く
    /// property)。正で外側へ広げ、負で内側へ縮める(Lottie/AE と同じ符号)。
    ///
    /// キーを打っていない = **既定 `0.0`(無効)** — `mask_opacity` の「既定 1.0」と
    /// 対になる「既定 0」。
    pub fn mask_expansion(mask: crate::MaskId) -> Self {
        Self::mask_property(mask, "expansion")
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

    /// effect の on/off(`effects/effect en` 相当)。**裁定213**で
    /// `EffectInstance::enabled` という静止 `bool` フィールドをやめ、他の
    /// animatable な param(`effect_param`)と同じ平坦 track へ寄せた ——
    /// `motolii_eval::Value::Bool` は補間が Hold なので、途中の値が存在しない
    /// on/off の意味とちょうど合う(`effect.rs` モジュール doc 参照)。
    ///
    /// キーを打っていない(track が無い)= **既定で有効**(`true`)——`mask_opacity`
    /// の「既定 1.0」と同じ判断(effect を追加した直後は何もしなくても効いて
    /// いるはず、という利用者の直感)。
    ///
    /// 標準 property と同じく予約語でも空でもないので、構築は失敗し得ない。
    pub fn effect_enabled(effect: crate::EffectId) -> Self {
        let name = format!("{}{effect}.enabled", crate::property::EFFECT_PREFIX);
        Self::new(&name).expect("effect の property 名は予約語でも空でもない")
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

/// **裸の文字列として符号化する**(`SlotId`/`crate::slot::translate_link` と同じ
/// 「透過ニュータイプ」の流儀)。`link` 発注単位(裁定206)が
/// [`crate::slot::PropertyLink::source_property`] を保存へ乗せる必要が出て初めて
/// 要った実装 — それまで `PropertyId` は一度も Document の JSON へ直接埋め込まれた
/// ことが無かった(常に `PropertyId::new(name)` で組み立て直す一時的な鍵としてのみ
/// 使われていた)。`component`(interned `ComponentIdentifier`)は運ばない —
/// 復元時に [`PropertyId::new`] を呼び直せば同じ値になるので、二重に持つ理由が無い。
impl Serialize for PropertyId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.name)
    }
}

/// 復元は [`PropertyId::new`] を呼び直すだけ — 予約語/空文字の柵も自動的に効く
/// (壊れた JSON から `masks`/`meta` 等の予約名を持つ `PropertyId` が復活しない)。
impl<'de> Deserialize<'de> for PropertyId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let name = <String as Deserialize>::deserialize(deserializer)?;
        PropertyId::new(&name).map_err(serde::de::Error::custom)
    }
}
