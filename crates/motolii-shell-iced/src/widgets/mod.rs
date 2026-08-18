//! 再利用の葉 widget — M-4w。**panel への組み込みはここでやらない**(消費側レーンが別走中)。
//!
//! 4部品とも:
//! - イベント語彙・状態語彙は発注 capsule で固定(逸脱すると統合が壊れる)
//! - 色は [`palette`] だけを見る(theme レーンの成果へ 1 import 差し替えで追従)
//! - Q3: hover / press / drag 中の即時視覚応答。Q0: 死に見た目なし。
//!   Q1: release = 確定・Esc = cancel 復元(scrub)

pub mod context_menu;
pub mod drop_zone;
pub mod key_button;
pub mod palette;
pub mod scrub_value;

pub use context_menu::{context_menu, MenuEvent, MenuItem};
pub use drop_zone::{drop_zone, DropEvent};
pub use key_button::{key_button, KeyState};
pub use palette::{Palette, PALETTE};
pub use scrub_value::{scrub_value, ScrubEvent, ScrubSpec};
