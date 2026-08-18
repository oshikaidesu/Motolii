//! key ボタン — 2026-08-13 裁定(`docs/reviews/2026-08-13-inspector-key-add-ux-decision.md`)
//! の3状態語彙: **unkeyed = 灰 outline / animated = accent outline / current = accent fill**。
//!
//! 状態の正本は呼び出し側の accepted snapshot(裁定「local optimistic key state を
//! 持たない」)。この widget は見た目と press だけを持つ。

use crate::widgets::palette;

/// key ボタンの3状態。**この enum が公開契約**(消費側 capsule と同文)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    /// key が1つも無い — 灰 outline(◇)。
    Unkeyed,
    /// 他時刻に key がある — accent outline(◇)。
    Animated,
    /// 現在 playhead に key がある — accent fill(◆)。
    Current,
}

/// 状態ごとの絵の語彙。draw とテストが同じ表を見る。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyLook {
    /// 菱形の字(outline = ◇ / fill = ◆)。
    pub glyph: &'static str,
    /// 菱形の色。
    pub color: iced::Color,
}

/// 3状態 → 絵。2026-08-13 裁定の写しであり、ここ以外で状態を色に写さない。
pub fn look(state: KeyState) -> KeyLook {
    let _ = (state, palette::PALETTE);
    todo!("red 先行 — 実装は次コミット")
}

/// key ボタンを1つ組む。押せば `on_press` がそのまま出る。
pub fn key_button<'a, M>(state: KeyState, on_press: M) -> iced::Element<'a, M>
where
    M: Clone + 'a,
{
    let _ = (state, on_press);
    todo!("red 先行 — 実装は次コミット")
}
