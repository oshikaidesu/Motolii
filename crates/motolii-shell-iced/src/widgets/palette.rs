//! 仮パレット — theme レーンが別走しているあいだ、葉 widget が色を引く**唯一の口**。
//!
//! ここの値は `crates/motolii-ui/src/inspector_panel/theme.rs`(=
//! `docs/mocks-ui/public/inspector-library.css` の fallback 実値)からの写しで、
//! **この file で新しい色を1つも決めない**。統合 wave で theme レーンの成果に
//! 差し替える時は、この module の import 1本を差し替えれば全部品が追従する
//! (発注 capsule の指定)。
//!
//! - `bg_panel` = surface-panel `#1a1a1a`
//! - `bg_control` = surface-raised `#222222`
//! - `text_primary` = text-primary `#f0f0f0`
//! - `text_secondary` = text-secondary `#c6c6c6`
//! - `accent` = action-active `#d8b574`
//! - `outline` = border-strong `#686868`

use iced::Color;

/// 葉 widget が参照する色の束。field 名は発注 capsule で固定
/// (`bg_panel` / `bg_control` / `text_primary` / `text_secondary` / `accent` / `outline`)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    /// panel・overlay の地。
    pub bg_panel: Color,
    /// 触れる面(入力欄・ボタン)の地。
    pub bg_control: Color,
    /// 主要な字。
    pub text_primary: Color,
    /// 二次的な字(無効・補助)。
    pub text_secondary: Color,
    /// 選択・活性・key の色。
    pub accent: Color,
    /// 枠線。
    pub outline: Color,
}

const fn rgb(v: u32) -> Color {
    Color {
        r: ((v >> 16) & 0xff) as f32 / 255.0,
        g: ((v >> 8) & 0xff) as f32 / 255.0,
        b: (v & 0xff) as f32 / 255.0,
        a: 1.0,
    }
}

/// いまの仮パレット。全部品はここだけを見る。
pub const PALETTE: Palette = Palette {
    bg_panel: rgb(0x1a1a1a),
    bg_control: rgb(0x222222),
    text_primary: rgb(0xf0f0f0),
    text_secondary: rgb(0xc6c6c6),
    accent: rgb(0xd8b574),
    outline: rgb(0x686868),
};

/// CSS `color-mix(in srgb, A p%, B (100-p)%)` の写し(egui 版 `theme::mix` と同じ)。
/// hover / press の段階はここで**パレットの2色から**導く — 新しい色定数を作らない。
pub(crate) fn mix(a: Color, pct_a: f32, b: Color) -> Color {
    let t = (pct_a.clamp(0.0, 100.0)) / 100.0;
    let ch = |x: f32, y: f32| x * t + y * (1.0 - t);
    Color {
        r: ch(a.r, b.r),
        g: ch(a.g, b.g),
        b: ch(a.b, b.b),
        a: 1.0,
    }
}
