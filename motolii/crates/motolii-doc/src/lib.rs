// 家: 作品の意味を読む側 — 値・時刻・評価の型と、読み取りモデル(StoreView・Recording)。
// 書き込み(Document・Intent・Undo・保存)は motolii-edit(crates/motolii-edit)、書き出しは motolii-render::export と motolii-jobs。
pub mod core;
pub mod eval;
pub mod store;
pub mod vector;

/// 家の中の道は `crate::doc::…` のまま。crate が割れても文は変えない。
pub mod doc {
    pub use crate::{core, eval, store, vector};
}
