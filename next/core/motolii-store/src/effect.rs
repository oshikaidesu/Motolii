//! layer が持つ effect の**列**(`layers/visual-layer/ef`、layer-meta 束 + `effect`
//! 発注単位)。
//!
//! **`effects/effect/*` の3行(`ef`/`en`/`ty`)と `effect-values/*` の7行、計10行が
//! ここで閉じる**:
//! - `ty`(Type)は [`EffectInstance::plugin_id`] — plugin id 文字列(裁定70。閉じた
//!   int registry にしない、D6「拡張の口が trait 1本」)
//! - `en`(Enabled)は [`EffectInstance::enabled`] — 消さずに切る。int-boolean ではなく
//!   `bool`(裁定78 と同じ理由で `Value::Bool` に寄せる場所を間違えない — これは
//!   `mask.inverted` と同じ**静止設定**であって animatable param ではないので、
//!   `Value`/`KeyframeTrack` には乗せない構造にした)
//! - `ef`(Effect Values、param 列)は **専用フィールドを持たない** —
//!   [`crate::PropertyId::effect_param`] が作る平坦 track(`effect.{id}.param.{name}`)の
//!   **有無そのものが「この param を触っているか」**(裁定20 の応用、mask/text-range と
//!   同じ形)。param の型語彙(scalar/Bool/Enum/color/Vec2/LayerId、effect-values の
//!   catalog)は `motolii_eval::Value` の `F64`/`Bool`/`Enum`/`Color`/`Vec2`/`LayerId`
//!   にそのまま対応する(裁定72「全 param が animatable なので既存の `KeyframeTrack` +
//!   `PropertyId` にそのまま乗る = 新機構ゼロ」)
//!
//! マスクと同じ形にしてある: `id` は安定 ID(添字ではない、裁定65/85 と同型)。

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

/// 1つの effect インスタンス。**param の値は持たない**(上記どおり `effect.{id}.param.*`
/// という平坦 track が持つ)。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectInstance {
    pub id: EffectId,
    /// `effects/effect/ty` に相当する識別。**閉じた int registry ではなく plugin id
    /// 文字列**(裁定70 — D6「拡張の口が trait 1本」。int registry だと
    /// first/third-party が同じ口にならない)。
    pub plugin_id: String,
    /// `effects/effect/en`(Enabled)。**消さずに切る** — マスクを1枚も外さず
    /// 一時的に効果を止める操作は、`SetEffects` で列から取り除く(=削除)とは別物。
    /// int-boolean ではなく `bool`(Lottie は `en` を int-boolean で持つが、ここは
    /// 静止設定なので `Value::Bool`(裁定78)の対象にはしない — animatable param とは
    /// 別の層にある、上記モジュール doc 参照)。
    pub enabled: bool,
}

/// ある comp 時刻に解決済みの effect インスタンス。**描く側が要るのはこれだけ**
/// (`ResolvedMask` と同じ形)。
///
/// `enabled` でない effect は**ここに現れない** — `id` フィールドも持たない。
/// これは `attrs.hidden` な layer が `resolve()` の外(`None`)へ落ちて `ResolvedLayer`
/// 自体を作らないのと同じ型: 「切る」は運ぶ情報ではなく、resolve の入口で弾く
/// 判定でしかない。順序は [`crate::StoreView::effects`] が返す宣言順のまま
/// (裁定66、マスクと同じ)。
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedEffect {
    /// `effects/effect/ty` に相当する識別。plugin id 文字列(裁定70)。
    pub plugin_id: String,
    /// この時刻で評価済みの param 値。名前つき(param 名 → 値)、param の宣言順ではなく
    /// track が実在する param だけを名前のアルファベット順で並べる
    /// (`StoreView::properties` が返す順をそのまま使う、裁定57「store に聞く」)。
    ///
    /// track の無い param(触っていない param)は**ここに現れない** — store は
    /// plugin の param カタログを知らない(裁定70)ので、既定値を埋める仕事は
    /// ここではなく呼び手(plugin 定義を知っている層)の役目。
    pub params: Vec<(String, crate::Value)>,
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
