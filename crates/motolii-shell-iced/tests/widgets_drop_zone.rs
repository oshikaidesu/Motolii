//! drop_zone — red 先行。
//!
//! 語彙(発注 capsule で固定): 面へ入って `HoverEnter`、出て `HoverLeave`。
//! 中での移動は黙る(enter / leave は縁でだけ言う)。受入可否(`accepting`)は
//! 絵の色の話で、語彙を変えない。

mod common;

use common::{cursor_left, point_and_move};
use motolii_shell_iced::widgets::drop_zone::{drop_zone, DropEvent};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Msg {
    Zone(DropEvent),
}

/// 200×100 の的。窓の左上に立つので、幾何はテストから読める。
fn zone(accepting: bool) -> iced::Element<'static, Msg> {
    drop_zone(
        iced::widget::container(iced::widget::text("target"))
            .width(200.0)
            .height(100.0)
            .into(),
        accepting,
        Msg::Zone,
    )
}

fn events(ui: iced_test::Simulator<'_, Msg>) -> Vec<DropEvent> {
    ui.into_messages().map(|Msg::Zone(event)| event).collect()
}

/// 入って1回・出て1回。中の移動は黙る。
#[test]
fn entering_and_leaving_speak_once_each() {
    let mut ui = iced_test::simulator(zone(true));
    point_and_move(&mut ui, 100.0, 50.0); // 入る
    point_and_move(&mut ui, 150.0, 50.0); // 中の移動 — 黙る
    point_and_move(&mut ui, 300.0, 50.0); // 出る

    assert_eq!(events(ui), vec![DropEvent::HoverEnter, DropEvent::HoverLeave]);
}

/// cursor が窓ごと出て行っても leave は言われる(hover が貼りつかない)。
#[test]
fn the_cursor_leaving_the_window_counts_as_leave() {
    let mut ui = iced_test::simulator(zone(true));
    point_and_move(&mut ui, 100.0, 50.0);
    let _ = ui.simulate([cursor_left()]);

    assert_eq!(events(ui), vec![DropEvent::HoverEnter, DropEvent::HoverLeave]);
}

/// 受け入れない面でも hover の語彙は同じ(可否は絵で言う)。
#[test]
fn a_rejecting_zone_still_reports_hover() {
    let mut ui = iced_test::simulator(zone(false));
    point_and_move(&mut ui, 100.0, 50.0);
    point_and_move(&mut ui, 300.0, 50.0);

    assert_eq!(events(ui), vec![DropEvent::HoverEnter, DropEvent::HoverLeave]);
}
