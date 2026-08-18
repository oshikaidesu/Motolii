//! scrub_value — red 先行。
//!
//! 語彙(発注 capsule で固定): gesture は `Started` で開き、`Changed` が途中の
//! 値を運び、release / Enter が `Committed`、Esc が `Cancelled`(Q1)。
//! 値の復元は `Started` で値を控えた消費側の仕事なので、ここでは
//! **語彙の列そのもの**を検収する。

mod common;

use common::{left_pressed, left_released, point_and_move};
use motolii_shell_iced::widgets::scrub_value::{
    format_value, scrub_value, ScrubEvent, ScrubSpec,
};

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Scrub(ScrubEvent),
}

fn spec(value: f64) -> ScrubSpec {
    ScrubSpec {
        value,
        decimals: 2,
        min: None,
        max: None,
        step: 0.5,
        integer: false,
    }
}

fn sim(spec: ScrubSpec) -> iced_test::Simulator<'static, Msg> {
    iced_test::simulator(scrub_value(spec, Msg::Scrub))
}

fn events(ui: iced_test::Simulator<'_, Msg>) -> Vec<ScrubEvent> {
    ui.into_messages().map(|Msg::Scrub(event)| event).collect()
}

/// ドラッグ列 → `Started` → `Changed` → release で `Committed`。
#[test]
fn a_drag_changes_and_release_commits() {
    let mut ui = sim(spec(10.0));
    point_and_move(&mut ui, 100.0, 10.0);
    let _ = ui.simulate([left_pressed()]);
    point_and_move(&mut ui, 110.0, 10.0); // +10px × step 0.5 = 15.0
    point_and_move(&mut ui, 120.0, 10.0); // +20px × step 0.5 = 20.0
    let _ = ui.simulate([left_released()]);

    assert_eq!(
        events(ui),
        vec![
            ScrubEvent::Started,
            ScrubEvent::Changed(15.0),
            ScrubEvent::Changed(20.0),
            ScrubEvent::Committed(20.0),
        ],
    );
}

/// Esc = cancel(Q1)。以後の release / 移動は黙る。
#[test]
fn escape_during_a_drag_cancels_instead_of_committing() {
    let mut ui = sim(spec(10.0));
    point_and_move(&mut ui, 100.0, 10.0);
    let _ = ui.simulate([left_pressed()]);
    point_and_move(&mut ui, 110.0, 10.0);
    let _ = ui.tap_key(iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape));
    let _ = ui.simulate([left_released()]);
    point_and_move(&mut ui, 130.0, 10.0);

    assert_eq!(
        events(ui),
        vec![
            ScrubEvent::Started,
            ScrubEvent::Changed(15.0),
            ScrubEvent::Cancelled,
        ],
    );
}

/// min / max は `Changed` の中で clamp 済み。同じ値は二度言わない。
#[test]
fn changes_are_clamped_and_never_repeated() {
    let mut ui = sim(ScrubSpec {
        value: 10.0,
        decimals: 2,
        min: Some(8.0),
        max: Some(12.0),
        step: 0.5,
        integer: false,
    });
    point_and_move(&mut ui, 100.0, 10.0);
    let _ = ui.simulate([left_pressed()]);
    point_and_move(&mut ui, 110.0, 10.0); // 15.0 → clamp 12.0
    point_and_move(&mut ui, 130.0, 10.0); // まだ 12.0 — 二度言わない
    point_and_move(&mut ui, 90.0, 10.0); // 5.0 → clamp 8.0
    let _ = ui.simulate([left_released()]);

    assert_eq!(
        events(ui),
        vec![
            ScrubEvent::Started,
            ScrubEvent::Changed(12.0),
            ScrubEvent::Changed(8.0),
            ScrubEvent::Committed(8.0),
        ],
    );
}

/// integer = true は整数へ snap する。
#[test]
fn integer_specs_snap_to_whole_numbers() {
    let mut ui = sim(ScrubSpec {
        value: 10.0,
        decimals: 0,
        min: None,
        max: None,
        step: 0.2,
        integer: true,
    });
    point_and_move(&mut ui, 100.0, 10.0);
    let _ = ui.simulate([left_pressed()]);
    point_and_move(&mut ui, 103.0, 10.0); // 10.6 → 11
    point_and_move(&mut ui, 104.0, 10.0); // 10.8 → 11 — 二度言わない
    point_and_move(&mut ui, 110.0, 10.0); // 12.0
    let _ = ui.simulate([left_released()]);

    assert_eq!(
        events(ui),
        vec![
            ScrubEvent::Started,
            ScrubEvent::Changed(11.0),
            ScrubEvent::Changed(12.0),
            ScrubEvent::Committed(12.0),
        ],
    );
}

/// ダブルクリック → テキスト入力(打った字が前の値を置き換える)→ Enter = 確定。
#[test]
fn double_click_opens_text_entry_and_enter_commits() {
    let mut ui = sim(spec(10.0));
    point_and_move(&mut ui, 100.0, 10.0);
    let _ = ui.simulate([left_pressed(), left_released(), left_pressed(), left_released()]);
    let _ = ui.typewrite("42.5");
    let _ = ui.tap_key(iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter));

    assert_eq!(
        events(ui),
        vec![ScrubEvent::Started, ScrubEvent::Committed(42.5)],
    );
}

/// テキスト入力中の Esc も cancel(Q1)。
#[test]
fn escape_during_text_entry_cancels() {
    let mut ui = sim(spec(10.0));
    point_and_move(&mut ui, 100.0, 10.0);
    let _ = ui.simulate([left_pressed(), left_released(), left_pressed(), left_released()]);
    let _ = ui.typewrite("99");
    let _ = ui.tap_key(iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape));

    assert_eq!(events(ui), vec![ScrubEvent::Started, ScrubEvent::Cancelled]);
}

/// 動かさないクリック1回は gesture ではない — 何も言わない。
#[test]
fn a_plain_click_does_not_speak() {
    let mut ui = sim(spec(10.0));
    point_and_move(&mut ui, 100.0, 10.0);
    let _ = ui.simulate([left_pressed(), left_released()]);

    assert_eq!(events(ui), vec![]);
}

/// 表示は decimals 桁固定(発注 capsule)。
#[test]
fn the_display_is_fixed_to_the_requested_decimals() {
    assert_eq!(format_value(20.0, 2), "20.00");
    assert_eq!(format_value(12.0, 0), "12");
    assert_eq!(format_value(0.5, 3), "0.500");
}
