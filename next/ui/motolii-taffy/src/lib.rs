//! wraps: taffy 0.13(flex/grid 解決)を iced widget 境界へ
//!
//! 裁定183 案A(docs/reviews/2026-08-22-weblike-layout-survey.md)。
//! モック転写の「CSS 宣言 → 手作業の固定幅+Fill 翻訳」という手癖工程を、
//! CSS とほぼ同語彙のまま widget へ写せる形に置換する:
//!
//! ```ignore
//! use motolii_taffy::{style_from_css_decl, TaffyBox};
//!
//! let root = style_from_css_decl(
//!     "display:flex; justify-content:space-between; gap:4px; padding:8px",
//! )?;
//! let cell = style_from_css_decl("width:64px; height:26px")?;
//! let bar = TaffyBox::new(root)
//!     .push(cell.clone(), left_widget)
//!     .push(cell, right_widget);
//! ```
//!
//! 役割分担:
//! - **レイアウト解決 = taffy**(crates.io 0.13 — Bevy/Dioxus/Blitz が本番使用)。
//!   iced 素の Row/Column に無い語彙(space-between / space-around / wrap /
//!   grid-template の minmax・repeat)がそのまま動く。
//! - **CSS 値の解釈 = taffy の `parse` feature**(upstream の cssparser 実装を借用)。
//!   こちら側は宣言リストの分解と property → `taffy::Style` フィールドの束縛のみ。
//!   対応外の property・解釈できない値は**黙って無視せず `Err`**(fail-closed)。
//! - **widget 境界 = `TaffyBox`**: 子 widget の測定を taffy の measure 閉包から
//!   `Widget::layout` 呼び出しで行い、taffy が解いた矩形を iced の `layout::Node`
//!   へ無変換で写す(probe 実測: fork rev 73e686ee の trait と型がそのまま噛む)。
//!
//! この crate は「レイアウト解決の機構」であり寸法定数を一切持たない —
//! gap や padding の実値は呼び手が tokens(裁定117)から渡す。
//! iced fork 本体には一切手を入れていない(fork 非改変)。

mod css;
mod widget;

pub use css::{apply_css_decl, style_from_css_decl, CssDeclError};
pub use widget::TaffyBox;

// 呼び手が `taffy::Style` の値(`length(4.0)` 等)を自分で組む時に、taffy への
// direct dependency を増やさずに済むための再輸出。バージョンの正本はこの crate。
pub use taffy;
