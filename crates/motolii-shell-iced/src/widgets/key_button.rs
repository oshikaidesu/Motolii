//! key ボタン — 2026-08-13 裁定(`docs/reviews/2026-08-13-inspector-key-add-ux-decision.md`)
//! の3状態語彙: **unkeyed = 灰 outline / animated = accent outline / current = accent fill**。
//!
//! 状態の正本は呼び出し側の accepted snapshot(裁定「local optimistic key state を
//! 持たない」)。この widget は見た目と press だけを持つ。押した結果どうなるかは
//! 消費側が既存 add-key intent へ写す。

use iced::widget::{button, text};
use iced::{Background, Border, Element};

use crate::theme::{self, Tokens};

/// ボタンの一辺(正方形)。
pub const KEY_BUTTON_SIZE: f32 = 18.0;

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
/// token は [`Tokens::DARK`](crate::theme::Tokens::DARK) — Light がまだ無いので
/// 組み込み既定と同じ(`Tokens::resolve` と揃った時に差し替わる)。
pub fn look(state: KeyState) -> KeyLook {
    let t = &Tokens::DARK;
    match state {
        KeyState::Unkeyed => KeyLook {
            glyph: "\u{25c7}",
            color: t.text_secondary,
        },
        KeyState::Animated => KeyLook {
            glyph: "\u{25c7}",
            color: t.action_active,
        },
        KeyState::Current => KeyLook {
            glyph: "\u{25c6}",
            color: t.action_active,
        },
    }
}

/// key ボタンを1つ組む。押せば `on_press` がそのまま出る。
pub fn key_button<'a, M>(state: KeyState, on_press: M) -> iced::Element<'a, M>
where
    M: Clone + 'a,
{
    let look = look(state);
    let diamond: Element<'a, M> = text(look.glyph)
        .size(FONT_SIZE)
        .shaping(text::Shaping::Advanced)
        .center()
        .width(iced::Fill)
        .height(iced::Fill)
        .into();

    button(diamond)
        .width(KEY_BUTTON_SIZE)
        .height(KEY_BUTTON_SIZE)
        .padding(0)
        .on_press(on_press)
        .style(move |theme, status| {
            let t = Tokens::resolve(theme);
            // Q3 接触の報酬: hover / press で地が段階的に応える。
            let fill = match status {
                button::Status::Hovered => theme::mix(t.action_active, 16.0, t.surface_raised),
                button::Status::Pressed => theme::mix(t.action_active, 26.0, t.surface_raised),
                button::Status::Active | button::Status::Disabled => t.surface_raised,
            };
            button::Style {
                background: Some(Background::Color(fill)),
                text_color: look.color,
                border: Border {
                    color: t.border_default,
                    width: 1.0,
                    radius: 3.0.into(),
                },
                shadow: iced::Shadow::default(),
                snap: true,
            }
        })
        .into()
}

/// 菱形の字の大きさ。
const FONT_SIZE: f32 = 11.0;
