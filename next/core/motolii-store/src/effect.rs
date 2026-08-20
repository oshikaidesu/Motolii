//! layer が持つ effect の**列**(`layers/visual-layer/ef`、layer-meta 束)。
//!
//! ここが持つのは「どの effect を・どの順で」だけ。**`en`(Enabled)・`ty` の細部
//! (プラグイン設定の param map・Bool/Enum 型の値)は `effect` 発注単位(10行、未着手)
//! の仕事**であって、ここでは作らない — layer-meta の地図の行(`ef Effects`)は
//! 「layer が effect の列を持つ形」だけを指しており、`effects/effect/*` の3行と
//! `effect-values/*` の7行は別行のまま残る。
//!
//! マスクと同じ形にしてある: `id` は安定 ID(添字ではない、裁定65/85 と同型)。
//! param の値そのものは(effect 束が実装する時に)`mask.{id}.…` と同じ平坦な
//! `PropertyId` 命名(`effect.{id}.{param}`)で普通の `KeyframeTrack` に乗る想定
//! (裁定72)なので、id の安定性は今のうちから要る。

use serde::{Deserialize, Serialize};

use crate::StoreError;

/// layer の中での effect インスタンスの安定 ID。マスクの [`crate::MaskId`] と同型の理由 —
/// 添字だと真ん中を消した時に後続の param track が別の effect へ付き直る。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EffectId(pub u32);

impl std::fmt::Display for EffectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 1つの effect インスタンス。**param の値は持たない**(上記どおり `effect` 束の仕事)。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectInstance {
    pub id: EffectId,
    /// `effects/effect/ty` に相当する識別。**閉じた int registry ではなく plugin id
    /// 文字列**(裁定70 — D6「拡張の口が trait 1本」。int registry だと
    /// first/third-party が同じ口にならない)。
    pub plugin_id: String,
}

/// 同じ id の effect が2枚あると、将来の param track(`effect.{id}.…`)の持ち主が
/// 決まらない。[`crate::mask::validate_unique_ids`] と同型の検査。
pub(crate) fn validate_unique_ids(effects: &[EffectInstance]) -> Result<(), StoreError> {
    for (i, effect) in effects.iter().enumerate() {
        if effects[..i].iter().any(|other| other.id == effect.id) {
            return Err(StoreError::Property(format!(
                "effect id {} が2枚ある",
                effect.id
            )));
        }
    }
    Ok(())
}
