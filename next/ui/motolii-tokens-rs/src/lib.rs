//! owns: デザイン token の読み口(寸法JSON+DTCG色の parse・watch・ui_scale 乗算点)。値の正本は JSON 側 — ここは複製しない。
//! デザイン値の外出し(裁定117)。
//!
//! **正本は2つ、どちらもここへコピーしない**:
//! - 寸法: `tokens/dimensions.json`(このファイルが機械可読正本。値は Ableton Live 12
//!   実測 — `docs/reviews/2026-08-19-ableton-density-measurements.md`)
//! - 色: `ui/motolii-tokens/sources/motolii-dark.json`(DTCG 形式。ここでも複製しない)
//!
//! debug ビルドはどちらも起動時にファイルから読み、[`watch_subscription`] が notify で
//! 変更を検知して再読込する。release は `include_str!` で埋め込んだ文字列を起動時に
//! 1回だけ parse する — **file I/O はゼロ**(iced の `Theme` は色・境界・影しか
//! 持てず寸法を Theme 化できないため、自前の [`Tokens`] を `State` に持つ形を採る)。
//!
//! raw 値の直書き禁止 — 全 pane はここ経由で寸法・色を読む。
//!
//! OWNS-JUSTIFICATION(A): 裁定117(デザイン値の外出し) — `iced::Theme` は
//! 色/境界/影しか持てず寸法をTheme化できないという上流の型の限界を具体的に
//! 確認した上で自前の`Tokens`を持つ(裁定215 棚卸し 2026-08-23 #31)。
//!
//! SP-8(裁定220): 1,514行だったこの1ファイルを責任ごとに割った(中身は移送のみ、
//! バグ修正・整形・リネームは混ぜていない)。公開型(`Colors`/`Dimensions`/`Ink`/
//! `TextWeight`)は名前も形も変えず、ここで再輸出して既存の参照経路
//! (`motolii_tokens_rs::Colors` 等)を保つ:
//! - [`mod dimensions`]: 寸法トークン(`Dimensions`)の struct・既定値・
//!   `parse`/`scaled` 等。
//! - [`mod colors`]: 色トークン(`Colors`)・色ロールの導出
//!   (`derive_state_colors` 等)・`iced::Theme` への変換。
//! - [`mod style`]: `TextWeight`/`Ink`(裁定137の文字の太さ3段・ink3段)。
//! - [`mod tokens`]: `Tokens`(起動時の読み込み一式)と `ui_scale` の書き戻し。
//! - [`mod watch`]: トークンファイルの変更監視 `Subscription` とデバウンス。

mod colors;
mod dimensions;
mod style;
mod tokens;
mod watch;

pub use colors::{theme_from_colors, Colors, LABEL_PALETTE_LEN};
pub use dimensions::Dimensions;
pub use style::{Ink, TextWeight};
pub use tokens::{replace_ui_scale, save_ui_scale, write_ui_scale_to_path, Tokens};
pub use watch::watch_subscription;
