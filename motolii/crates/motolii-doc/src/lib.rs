// 家: 意味と保存。編集状態(Document)と、その直列化。
// 書き込みは Intent 経由のみ。export は独立した工程ではなく doc の保存関数。
pub mod core;
pub mod eval;
pub mod fixture;
pub mod store;
pub mod vector;

/// 家の中の道は `crate::doc::…` のまま。crate が割れても文は変えない。
pub mod doc {
    pub use crate::{core, eval, fixture, store, vector};
}
