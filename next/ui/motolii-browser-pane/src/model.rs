//! 一覧 projection(裁定162 切片 B1)+ rail/filter(切片 B2)— `StoreView::assets()`
//! から Browser が描く行を組み、rail scope + 検索文字列で絞る純関数。**IO も
//! 評価もしない** — `StoreView` が既に読んだ台帳をそのまま並べ替える/絞るだけの
//! 読み専用の投影(`timeline_pane::rows`/`projection.rs` と同じ形)。
//!
//! 移植元(意味の正本)は旧 `crates/motolii-shell-iced/src/browser.rs` の
//! `BrowserCard` 投影だが、この切片(B1+B2)の範囲は一覧+rail/filter まで —
//! 視覚(B3、`browser-library.html` 構造のトンマナ読み替え・Shell::view への
//! 組み込み)・サムネ(B4/B5)はまだこの crate に無い(crate 冒頭 doc 参照)。
//!
//! rail scope は mock(`browser-library.html` `.librarySidebar` `LIBRARY` 節)の
//! **種別のみ**(第一波、裁定162 付随裁定: MEDIA 種別のみ)。`COLLECTIONS`
//! (色ドット bin)・`PLACES`(Starter/Project/Motion フォルダ)は
//! `browser-semantics.html` 救出台帳で「予約地」(タグ束・filesystem 走査裁定
//! 待ち)と明記済み — この切片では実装しない。
//!
//! SP-6(裁定220 レーン、`browser-pane` を割る)による分割: 元 `model.rs`
//! (1,742行)を責任ごとに4分割(`projection`=素材の投影+欠落バッジの
//! 状態運搬・`rail`=filter rail の scope・`organize`=並べ替え/表示形式・
//! `tabs`=タブ+preview-local カタログ)。**中身は移送のみ**、この
//! ファイル自身は各モジュールの再輸出だけを持つ(`model::X` という既存の
//! 呼び出し経路を壊さないため)。

pub mod organize;
pub mod projection;
pub mod rail;
pub mod tabs;
#[cfg(test)]
mod test_support;

pub use organize::*;
pub use projection::*;
pub use rail::*;
pub use tabs::*;
