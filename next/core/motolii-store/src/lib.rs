//! wraps: re_entity_db::EntityDb — Document の実体。
//!
//! **undo / redo は `edit` timeline の latest-at 移動そのもの**であり、自前の履歴機構を
//! 持たない(rerun blueprint の undo と同じ機構。R0-2 で1000編集跨ぎを実測)。
//! 「新しい編集をする前に redo 空間を落とす」も rerun の規則をそのまま踏襲する。
//!
//! ここに書いてよいのは「store の口をどう開けるか」だけである。時刻→値の意味は
//! `motolii-eval`(移植した正本)が持ち、この crate は評価を呼ぶだけで再実装しない。
//!
//! 設計上の柵:
//! - 読み手が受け取るのは [`StoreView`] だけで、可変ハンドルは外へ出ない
//! - 書き口は [`Document::apply`] 1本だけ
//! - **削除も append**(tombstone)。`drop_entity_path` を使うと undo で戻せなくなる

mod components;
mod document;
mod view;

pub use document::{Document, Intent, LayerId, PropertyId};
pub use view::StoreView;

pub use motolii_core::RationalTime;
pub use motolii_eval::{Interp, Keyframe, KeyframeTrack, Value};

/// `edit` timeline の名前。undo/redo はこの軸の移動である。
pub const EDIT_TIMELINE: &str = "edit";

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("chunk の組み立てに失敗した: {0}")]
    Chunk(String),
    #[error("store への追加に失敗した: {0}")]
    Ingest(String),
    #[error("track の符号化に失敗した: {0}")]
    Encode(#[from] serde_json::Error),
    #[error("property 名が不正: {0}")]
    Property(String),
}
