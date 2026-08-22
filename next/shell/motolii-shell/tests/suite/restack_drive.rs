//! 運転席 — 第6波 shell 結線(B44: `timeline_pane::Message::RestackLayer` の
//! keymap 露出 + Edit メニュー末尾の補間動詞露出)。
//!
//! `RestackLayer` 自体の意味(`restack_layers` — block coalesce・0..n 振り
//! 直し・端 no-op)は `motolii-timeline-pane::write` 側の試験
//! (`write.rs` の `restack_*` テスト群)が既に持っている — この波で shell が
//! 足したのは **keymap の口**(Cmd+Alt+↑/↓・+Shift)と **Edit メニュー末尾の
//! `SetKeyInterp` 露出**だけなので、ここではその2点だけを検分する
//! (`timeline_pane::Message` は既に `Message::Timeline` の「other」経路で
//! `PaneState::update` へ委譲されるので、shell 側に重複ロジックを持たない)。

use motolii_shell::{menu, timeline, timeline_pane, Message};
use motolii_store::Interp;

#[test]
fn cmd_alt_up_resolves_to_restack_forward() {
    use iced::keyboard::{key::Named, Key, Modifiers};
    let key = Key::Named(Named::ArrowUp);
    let modifiers = Modifiers::COMMAND | Modifiers::ALT;
    let message = motolii_shell::resolve_navigation_key(&key, modifiers, false);
    assert!(matches!(
        message,
        Some(Message::Timeline(timeline_pane::Message::RestackLayer(
            timeline::StackDirection::Forward
        )))
    ));
}

#[test]
fn cmd_alt_down_resolves_to_restack_backward() {
    use iced::keyboard::{key::Named, Key, Modifiers};
    let key = Key::Named(Named::ArrowDown);
    let modifiers = Modifiers::COMMAND | Modifiers::ALT;
    let message = motolii_shell::resolve_navigation_key(&key, modifiers, false);
    assert!(matches!(
        message,
        Some(Message::Timeline(timeline_pane::Message::RestackLayer(
            timeline::StackDirection::Backward
        )))
    ));
}

#[test]
fn cmd_alt_shift_up_resolves_to_restack_to_front() {
    use iced::keyboard::{key::Named, Key, Modifiers};
    let key = Key::Named(Named::ArrowUp);
    let modifiers = Modifiers::COMMAND | Modifiers::ALT | Modifiers::SHIFT;
    let message = motolii_shell::resolve_navigation_key(&key, modifiers, false);
    assert!(matches!(
        message,
        Some(Message::Timeline(timeline_pane::Message::RestackLayer(
            timeline::StackDirection::ToFront
        )))
    ));
}

#[test]
fn cmd_alt_shift_down_resolves_to_restack_to_back() {
    use iced::keyboard::{key::Named, Key, Modifiers};
    let key = Key::Named(Named::ArrowDown);
    let modifiers = Modifiers::COMMAND | Modifiers::ALT | Modifiers::SHIFT;
    let message = motolii_shell::resolve_navigation_key(&key, modifiers, false);
    assert!(matches!(
        message,
        Some(Message::Timeline(timeline_pane::Message::RestackLayer(
            timeline::StackDirection::ToBack
        )))
    ));
}

#[test]
fn plain_arrow_up_down_is_untouched_by_the_restack_binding() {
    // 修飾無しの↑/↓は他の腕(タイムライン以外)にも取られていないはず —
    // NudgeKeyframe/StepPlayhead は左右矢印のみを使うので、上下矢印は
    // resolve_navigation_key のどの腕からも発火しない。
    use iced::keyboard::{key::Named, Key, Modifiers};
    assert!(motolii_shell::resolve_navigation_key(&Key::Named(Named::ArrowUp), Modifiers::default(), false)
        .is_none());
    assert!(
        motolii_shell::resolve_navigation_key(&Key::Named(Named::ArrowDown), Modifiers::default(), false)
            .is_none()
    );
}

#[test]
fn edit_menu_exposes_the_interpolation_verbs_at_the_end() {
    let edit_menu = menu::menus().into_iter().find(|m| m.label == "Edit").expect("Edit メニューが無い");
    let labels: Vec<&str> = edit_menu.items.iter().map(|item| item.label).collect();
    for expected in [
        "Interpolation: Hold",
        "Interpolation: Linear",
        "Interpolation: Easy Ease",
        "Interpolation: Easy Ease In",
        "Interpolation: Easy Ease Out",
    ] {
        assert!(labels.contains(&expected), "Edit メニューに {expected} が無い: {labels:?}");
    }
    // 「末尾」の検分: 最後の5項目が補間動詞であること。
    let tail: Vec<&str> = labels.iter().rev().take(5).rev().copied().collect();
    assert!(
        tail.iter().all(|label| label.starts_with("Interpolation:")),
        "補間動詞が Edit メニューの末尾に無い: {labels:?}"
    );
}

#[test]
fn edit_menu_hold_item_carries_the_real_hold_interp_message() {
    let edit_menu = menu::menus().into_iter().find(|m| m.label == "Edit").expect("Edit メニューが無い");
    let hold = edit_menu
        .items
        .into_iter()
        .find(|item| item.label == "Interpolation: Hold")
        .expect("Hold 項目が無い");
    assert!(matches!(
        hold.message,
        Message::Timeline(timeline_pane::Message::SetKeyInterp(Interp::Hold))
    ));
}
