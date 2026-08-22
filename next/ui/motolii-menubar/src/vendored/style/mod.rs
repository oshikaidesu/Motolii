//! iced_aw の `src/style.rs` のうち、menu が要る2モジュール(`status`/`menu_bar`)
//! だけを抜き出した縮約(upstream の他 widget の style は持ち込まない)。
//! `pub use status::{Status, StyleFn}` は upstream `style.rs` の同名 re-export の写し —
//! vendored 側の `use crate::style::{Status, menu_bar::*}` がそのまま解決されるための口。

pub mod menu_bar;
pub mod status;

pub use status::{Status, StyleFn};
