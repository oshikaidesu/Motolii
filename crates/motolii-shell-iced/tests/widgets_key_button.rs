//! key_button — red 先行。
//!
//! 3状態の語彙は 2026-08-13 裁定
//! (`docs/reviews/2026-08-13-inspector-key-add-ux-decision.md`):
//! unkeyed = 灰 outline / animated = accent outline / current = accent fill。

use motolii_shell_iced::widgets::key_button::{key_button, look, KeyState};
use motolii_shell_iced::widgets::palette::PALETTE;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Msg {
    AddKey,
}

/// 3状態 → 絵の写像が裁定の語彙どおりである。
#[test]
fn the_three_states_follow_the_2026_08_13_vocabulary() {
    let unkeyed = look(KeyState::Unkeyed);
    let animated = look(KeyState::Animated);
    let current = look(KeyState::Current);

    // unkeyed = 灰 outline: 中抜きの菱形で、accent ではない灰。
    assert_eq!(unkeyed.glyph, "\u{25c7}");
    assert_ne!(unkeyed.color, PALETTE.accent);
    // animated = accent outline: 中抜きのまま accent。
    assert_eq!(animated.glyph, "\u{25c7}");
    assert_eq!(animated.color, PALETTE.accent);
    // current = accent fill: 塗りの菱形で accent。
    assert_eq!(current.glyph, "\u{25c6}");
    assert_eq!(current.color, PALETTE.accent);
    // unkeyed と animated は glyph が同じでも色で区別できる。
    assert_ne!(unkeyed.color, animated.color);
}

/// 押せば `on_press` がそのまま1回出る。
#[test]
fn pressing_the_button_emits_on_press() {
    let mut ui = iced_test::simulator(key_button(KeyState::Unkeyed, Msg::AddKey));
    ui.click("\u{25c7}").expect("押せる菱形が立っている");

    let messages: Vec<Msg> = ui.into_messages().collect();
    assert_eq!(messages, vec![Msg::AddKey]);
}

/// current は塗りの菱形(◆)で立つ — 絵まで見て言う。
#[test]
fn a_current_key_button_shows_a_filled_diamond() {
    let mut ui = iced_test::simulator(key_button(KeyState::Current, Msg::AddKey));
    assert!(
        ui.find("\u{25c6}").is_ok(),
        "current の key button は ◆ で立っていなければならない"
    );
}
