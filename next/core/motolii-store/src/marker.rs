//! marker — コンポジションの名前つきロケータ。
//!
//! 意味は Lottie の `helpers/marker`(`cm`/`tm`/`dr`)から取る(発明しない)。
//!
//! **マスクとの違い**: マスクは他の property track から `mask.{id}.…` という名前で
//! 参照される(裁定65 の同一性の話)。マーカーはどの property からも参照されないので、
//! `MaskId` に相当する安定 id は要らない — 並びは `Vec<Marker>` の宣言順そのもの。
//!
//! **`dr`(尺)は out point ではなく duration**(裁定64 と同じ理由: 絶対終端より
//! 長さの方が編集で壊れにくい)。`time` と同じ `RationalTime` で持つ — 尺 0 は
//! 「範囲ではなく単発のロケータ」を表す。

use serde::{Deserialize, Serialize};

use motolii_core::RationalTime;

/// マーカー1枚。**動く部分を持たない** — キーフレーム化される対象ではないので
/// property track ではなく、comp 自身の component として丸ごと保存する。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Marker {
    /// Lottie の `cm`(Comment)。ロケータの名前。
    pub name: String,
    /// Lottie の `tm`(Time)。ロケータの時刻。
    pub time: RationalTime,
    /// Lottie の `dr`(Duration)。範囲ロケータの尺。0 = 単発のロケータ。
    pub duration: RationalTime,
}
