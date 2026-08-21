//! マスク — layer 自身が持つパスで、その layer の中身を切り抜く。
//!
//! **matte とは別物**である(matte は他 layer の alpha で抜く、裁定66)。
//!
//! 意味は Lottie の `helpers/mask` から取る(発明しない)。ただし3点だけ寄せない:
//!
//! - **`None` を列挙へ混ぜない**。Lottie の `mask-mode` は `n`(無視)を持つが、
//!   「消す」ことは「重ね方」ではない。混ぜると、マスクが1枚居るのに効いていない
//!   という状態が保存でき、UI ではそれが2通りの消し方に見える
//! - **不透明度は比**(1.0 基準)。Lottie のパーセントは採らない(裁定58/65)
//! - **膨張(`x` / Expand)は持たない**。パスのオフセット演算なので、要る日に effect で足す
//!
//! 静止する部分(重ね方・反転・並び)だけがここにあり、**動く部分は property track**
//! である。形状は `mask.{id}.shape`、不透明度は `mask.{id}.opacity` という名前で
//! 普通の `KeyframeTrack` に載る — マスクのために新しい機構を1つも足していない。

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

/// マスク1枚のうち、**キーを打たない部分**。
///
/// 形状と不透明度はここに入れない — 動くので property track が持つ
/// ([`crate::PropertyId::mask_shape`] / [`crate::PropertyId::mask_opacity`])。
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
