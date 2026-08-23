//! マスク — layer 自身が持つパスで、その layer の中身を切り抜く。
//!
//! **matte とは別物**である(matte は他 layer の alpha で抜く、裁定66)。
//!
//! 意味は Lottie の `helpers/mask` から取る(発明しない)。ただし2点だけ寄せない:
//!
//! - **`None` を列挙へ混ぜない**。Lottie の `mask-mode` は `n`(無視)を持つが、
//!   「消す」ことは「重ね方」ではない。混ぜると、マスクが1枚居るのに効いていない
//!   という状態が保存でき、UI ではそれが2通りの消し方に見える
//! - **不透明度は比**(1.0 基準)。Lottie のパーセントは採らない(裁定58/65)
//!
//! 静止する部分(重ね方・反転・並び)だけがここにあり、**動く部分は property track**
//! である。形状は `mask.{id}.shape`、不透明度は `mask.{id}.opacity`、膨張は
//! `mask.{id}.expansion` という名前で普通の `KeyframeTrack` に載る — マスクのために
//! 新しい機構を1つも足していない。
//!
//! ## 2026-08-22: 膨張の追加(B02、Lottie `helpers/mask/x`(Expand))
//!
//! **旧い棄却理由**(このセクションより上、当初の版): 「膨張(`x` / Expand)は持たない。
//! パスのオフセット演算なので、要る日に effect で足す」。歴史として残すために文面は
//! 消さずこの節を足す形にした。
//!
//! **意味の正本は Lottie**(利用者裁定「意味は全て Lottie が持っている」)。
//! `next/reference/lottie-coverage.tsv` 行197(`helpers mask x Expand`)が実在する
//! Lottie 語彙として既に載っており、旧版の verdict は「不採用」だった
//! (地図ファイル自体は本発注の対象外なので verdict 欄は書き換えていない — RETURN 参照)。
//! `next/reference/normal-map.tsv` 行767(`Mask Expansion…`)は AE 側の呼称として同じ
//! 機能を指す。**フェザー(AE の Mask Feather、`normal-map.tsv` 行764/768/779/793)は
//! Lottie の `helpers/mask` に対応する `f` 相当のキーが無い** — `lottie-coverage.tsv`
//! を全文 grep しても `feather` は1件も出ない。Lottie に無い語彙を先に足すと発明になる
//! ため、**フェザーは今回は追加していない**(RETURN の「消化予定行 id」参照 — 767 のみ
//! 消化、768/779/793 は保留)。
//!
//! **なぜ今 expansion を入れるか**: 利用者裁定「普通の AE にする」(2026-08-17
//! `docs/decision-index.md` full-delegation 系列)のもとで動いている INS-mask レーンが、
//! 「store に型が無いので見送り」と報告した。**要る日が来た** — Lottie に実在する
//! 語彙であり、AE 実機でも Mask Opacity/Shape と同じくタイムラインでキーを打てる
//! 動く property なので、「effect で後付け」という旧い判断は「動く量は property track」
//! という本ファイルの法則自体と矛盾していた(先送りの理由が薄かった、というのが
//! 今回の評価)。
//!
//! **形**: [`crate::PropertyId::mask_expansion`](単一スカラー `Value::F64`、Lottie
//! `x` も AE も同じくスカラーで正で外側・負で内側)。**`Mask` struct にはフィールドを
//! 足していない** — shape/opacity と全く同じで、track の有無だけが「値を持つか」を
//! 表す(裁定20)。キーを打っていない場合の読み戻しは `0.0` = 無効、AE の新規マスクの
//! 既定と同型。
//!
//! **未完(次のレーンへ)**: [`ResolvedMask`] はまだ expansion を運ばない —
//! `crate::view::StoreView::resolved_masks` が読むのは今も shape/opacity だけなので、
//! `Intent::SetTrack` で書いた値は保存・undo・読み出し(`StoreView::value_at`)は効くが、
//! 描画(compositor)へはまだ渡らない。実際にマスクを広げる幾何演算(パスのオフセット)を
//! 持つのは compositor/engine 側の仕事で、この発注の範囲外(EXACT TARGET が
//! `mask.rs`/`attrs.rs`/`document.rs` のみだったため、`view.rs` は意図して触っていない)。

use serde::{Deserialize, Serialize};

use crate::StoreError;

/// layer の中でのマスクの安定 ID。**添字ではない。**
///
/// 形状トラックの名前がこの id から決まるので、並べ替えても1枚消しても
/// トラックが別のマスクへ付き直さない。添字で持つと、真ん中を消した瞬間に
/// 3枚目の形状が2枚目のマスクへ移る — 裁定65(`ind` によるレイヤ同一性)と
/// 裁定85(スパンに値を直書きしない)が捨てたのと同じ形である。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MaskId(pub u32);

impl std::fmt::Display for MaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 手前までのマスクの覆いと、このマスクの覆いをどう重ねるか。
///
/// Lottie の `constants/mask-mode` の7値から **`None`(`n`)を落とした6値**。
/// 畳み込みの順序は `Vec<Mask>` の並びが明示するので、暗黙の隣接参照は無い(裁定66)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaskMode {
    Add,
    Subtract,
    Intersect,
    Lighten,
    Darken,
    Difference,
}

/// 既定は `Add`(裁定160 発注γ MK2) — AE で新規マスクを描いた時の既定 mode と同型
/// (`crate::attrs::BlendMode` の `Default` が `Normal` を既定にするのと同じ形)。
/// `#[serde(default)]`(下記 [`Mask::mode`])が読み戻しに使う。
impl Default for MaskMode {
    fn default() -> Self {
        Self::Add
    }
}

impl MaskMode {
    /// [`crate::PropertyId::mask_mode`] の `Value::Enum` 表現(裁定214 —
    /// マスク合成に直接効くので時間軸に乗る)。**補間は Hold**(裁定213)。添字は
    /// 保存形の一部になるので、既存の並び([`crate::BlendMode::to_enum_value`]と
    /// 同じ流儀)を変えてはいけない。
    pub fn to_enum_value(self) -> i64 {
        match self {
            MaskMode::Add => 0,
            MaskMode::Subtract => 1,
            MaskMode::Intersect => 2,
            MaskMode::Lighten => 3,
            MaskMode::Darken => 4,
            MaskMode::Difference => 5,
        }
    }

    /// [`Self::to_enum_value`] の逆写像。未知の値は `None`(壊れた track を近似しない)。
    pub fn from_enum_value(v: i64) -> Option<Self> {
        match v {
            0 => Some(MaskMode::Add),
            1 => Some(MaskMode::Subtract),
            2 => Some(MaskMode::Intersect),
            3 => Some(MaskMode::Lighten),
            4 => Some(MaskMode::Darken),
            5 => Some(MaskMode::Difference),
            _ => None,
        }
    }
}

/// **裁定214**: Mask Mode / Mask Inverted も出力に直接効くので時間軸に乗る
/// (A03棚卸し行 — マスク合成に直接効く)。id ごとの track なので
/// [`crate::property::MASK_PREFIX`] 経由の平坦名(`mask.{id}.mode`/
/// `mask.{id}.inverted`、`mask_shape`/`mask_opacity`/`mask_expansion` と同じ形)。
/// **配線済み**(`crate::view::StoreView::resolved_masks` が `value_at` 経由で読む、
/// track 無しは静的 [`Mask::mode`]/[`Mask::inverted`] が既定値、裁定20)。
impl crate::PropertyId {
    /// マスクの重ね方(`Value::Enum`、[`MaskMode::to_enum_value`])。
    pub fn mask_mode(mask: MaskId) -> Self {
        Self::mask_attr_property(mask, "mode")
    }

    /// マスクの反転(`Value::Bool`)。
    pub fn mask_inverted(mask: MaskId) -> Self {
        Self::mask_attr_property(mask, "inverted")
    }

    fn mask_attr_property(mask: MaskId, attr: &str) -> Self {
        let name = format!("{}{mask}.{attr}", crate::property::MASK_PREFIX);
        Self::new(&name).expect("マスクの property 名は予約語でも空でもない")
    }
}

/// マスク1枚のうち、**キーを打たない部分**。
///
/// 形状・不透明度・膨張はここに入れない — 動くので property track が持つ
/// ([`crate::PropertyId::mask_shape`] / [`crate::PropertyId::mask_opacity`] /
/// [`crate::PropertyId::mask_expansion`]、2026-08-22 追加分は本ファイル冒頭の節参照)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mask {
    pub id: MaskId,
    /// `#[serde(default)]`: 旧ドキュメントの JSON にこのキーが無いことがある
    /// (裁定160 発注γ MK2 で `mode` を足す前の保存)。無いと `Add` に読み戻す
    /// (`label_color` と同じ後方互換の形、`crate::attrs::LayerAttrs` 参照)。
    #[serde(default)]
    pub mode: MaskMode,
    /// 覆いの内外を入れ替える。**`Subtract` とは別物** — Subtract は手前までの
    /// 覆いから引く操作で、反転はこのマスク自身の覆いを裏返す。
    pub inverted: bool,
}

/// ある comp 時刻に解決済みのマスク。**描く側が要るのはこれだけ**。
///
/// **expansion をまだ持たない**(2026-08-22 時点)。`PropertyId::mask_expansion` の
/// track は書ける・読める・保存できるが、`StoreView::resolved_masks` がまだこれを
/// 引いていないので、ここには現れない。実際にマスクを広げる幾何演算を持つのは
/// compositor/engine 側の仕事で、そちらが実装される回にこの struct へフィールドを足し、
/// `resolved_masks` を対応させる(本ファイル冒頭の節参照)。
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedMask {
    pub mode: MaskMode,
    pub inverted: bool,
    /// 0.0–1.0。比であってパーセントではない。
    pub opacity: f32,
    pub shape: motolii_eval::Path,
}

/// 同じ id のマスクが2枚あると、形状トラックの持ち主が決まらない。
///
/// マスク一覧を書く道は [`crate::Intent::SetMasks`] 1本しかないので、検査もそこ1箇所でよい。
pub(crate) fn validate_unique_ids(masks: &[Mask]) -> Result<(), StoreError> {
    for (i, mask) in masks.iter().enumerate() {
        if masks[..i].iter().any(|other| other.id == mask.id) {
            return Err(StoreError::Property(format!(
                "マスク id {} が2枚ある。形状トラック `mask.{}.shape` がどちらの物か決まらない",
                mask.id, mask.id
            )));
        }
    }
    Ok(())
}
