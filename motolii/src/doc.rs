// 家: 意味と保存。編集状態(Document)と、その直列化。
// 書き込みは Intent 経由のみ。export は独立した工程ではなく doc の保存関数。
pub mod core;
pub mod eval;
pub mod export;
pub mod fixture;
pub mod store;
