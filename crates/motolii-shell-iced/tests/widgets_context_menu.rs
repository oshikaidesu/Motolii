//! context_menu — red 先行。
//!
//! 語彙(発注 capsule で固定): 有効な項目で `Chosen(id)`、外クリックか Esc で
//! `Dismissed`。無効な項目は黙る(Q0 の「文脈 disabled」— 拒否は絵と cursor が言う)。

mod common;

use common::{left_pressed, left_released, point_and_move};
use motolii_shell_iced::widgets::context_menu::{
    context_menu, item_position, MenuEvent, MenuItem,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Msg {
    Menu(MenuEvent),
}

const AT: iced::Point = iced::Point::new(200.0, 150.0);

fn items() -> Vec<MenuItem> {
    vec![
        MenuItem {
            id: 1,
            label: "Cut".to_owned(),
            enabled: true,
        },
        MenuItem {
            id: 2,
            label: "Copy".to_owned(),
            enabled: true,
        },
        MenuItem {
            id: 7,
            label: "Paste".to_owned(),
            enabled: false,
        },
    ]
}

fn sim() -> iced_test::Simulator<'static, Msg> {
    iced_test::simulator(context_menu(items(), AT, Msg::Menu))
}

fn events(ui: iced_test::Simulator<'_, Msg>) -> Vec<MenuEvent> {
    ui.into_messages().map(|Msg::Menu(event)| event).collect()
}

/// 有効な項目を押すと、その id で `Chosen` が1回出る。
#[test]
fn choosing_an_enabled_item_says_chosen_with_its_id() {
    let mut ui = sim();
    let target = item_position(AT, 1); // "Copy" (id = 2)
    point_and_move(&mut ui, target.x, target.y);
    let _ = ui.simulate([left_pressed(), left_released()]);

    assert_eq!(events(ui), vec![MenuEvent::Chosen(2)]);
}

/// 無効な項目は選べない — event は出ず、メニューを消す口実にもならない。
#[test]
fn a_disabled_item_stays_silent() {
    let mut ui = sim();
    let target = item_position(AT, 2); // "Paste"(enabled: false)
    point_and_move(&mut ui, target.x, target.y);
    let _ = ui.simulate([left_pressed(), left_released()]);

    assert_eq!(events(ui), vec![]);
}

/// メニューの外を押すと `Dismissed`。
#[test]
fn pressing_outside_dismisses() {
    let mut ui = sim();
    point_and_move(&mut ui, 600.0, 500.0);
    let _ = ui.simulate([left_pressed(), left_released()]);

    assert_eq!(events(ui), vec![MenuEvent::Dismissed]);
}

/// Esc でも `Dismissed`。
#[test]
fn escape_dismisses() {
    let mut ui = sim();
    let _ = ui.tap_key(iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape));

    assert_eq!(events(ui), vec![MenuEvent::Dismissed]);
}
