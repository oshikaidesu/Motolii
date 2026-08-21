//! 運転席 — Inspector の Speed 欄(SP1 第一波、map id=963「Time Stretch…」・
//! id=269「Reset Clip(speed)」消化、supervisor 決定1-7)。
//!
//! 見るのは発注書の ORACLE (c):
//! - Speed 欄へ 200 入力確定 → `LayerTiming.speed` が2倍・duration半分・
//!   start不変・undo 1発で戻る
//! - ロック layer は拒否+status(M13、move/trim と同じ形)
//! - 100% reset の no-op(Undo を積まない)

use motolii_shell::inspector_pane;
use motolii_shell::timeline_pane;
use motolii_shell::{Message, Shell};

fn shell() -> Shell {
    Shell::new().0
}

/// **本命**: 200% 確定 → speed 2倍・duration 半分・start 不変、Undo 1回で
/// 元(100%・duration 300)へ戻る(1 gesture = 1 undo)。
#[test]
fn submitting_200_percent_doubles_speed_and_halves_duration_with_one_undo() {
    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);

    let before = shell.timeline_rows()[0].clone();
    assert_eq!(before.start, 0, "AddLayer 直後の start が想定と違う(fixture 前提が崩れている)");
    assert_eq!(before.duration, 300, "AddLayer 直後の duration が想定と違う");
    assert_eq!(
        shell.inspector_selection().unwrap().attrs.speed_percent,
        100.0,
        "新規 layer の既定 speed が 100% でない"
    );

    let _ = shell.update(Message::Inspector(inspector_pane::Message::SpeedInput("200".to_owned())));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::SpeedSubmit));

    assert_eq!(
        shell.inspector_selection().unwrap().attrs.speed_percent,
        200.0,
        "200% へ確定していない"
    );
    let after = shell.timeline_rows()[0].clone();
    assert_eq!(after.start, 0, "start が動いている(source 窓の意味が壊れる、決定4)");
    assert_eq!(after.duration, 150, "duration が半分になっていない(決定4)");
    assert!(shell.can_undo());

    // **1 gesture = 1 undo**: 1回の Undo で speed も duration も同時に戻る。
    let _ = shell.update(Message::Undo);
    assert_eq!(
        shell.inspector_selection().unwrap().attrs.speed_percent,
        100.0,
        "Undo 1回で100%へ戻らない(1 gesture = 1 undo 違反)"
    );
    assert_eq!(shell.timeline_rows()[0].duration, 300, "Undo 1回で duration が元へ戻らない");
    assert_eq!(shell.timeline_rows()[0].start, 0);
}

/// Reset ボタン(id=269 消化): 200% から押すと 100%・duration も元へ戻る。
#[test]
fn reset_after_changing_speed_restores_normal_speed_and_duration() {
    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);
    let _ = shell.update(Message::Inspector(inspector_pane::Message::SpeedInput("200".to_owned())));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::SpeedSubmit));
    assert_eq!(shell.timeline_rows()[0].duration, 150);

    let _ = shell.update(Message::Inspector(inspector_pane::Message::ResetSpeed));

    assert_eq!(shell.inspector_selection().unwrap().attrs.speed_percent, 100.0);
    assert_eq!(
        shell.timeline_rows()[0].duration,
        300,
        "reset 後の duration が元(source 窓)へ戻っていない"
    );
}

/// **決定7**: 既に100%の時に Reset を押しても no-op — 別の undo エントリを
/// 積まない。積んでいれば、この1回の Undo では AddLayer 自体が消えず layer が
/// 残ってしまう。
#[test]
fn reset_at_100_percent_is_a_no_op_and_does_not_push_undo() {
    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);
    assert_eq!(shell.inspector_selection().unwrap().attrs.speed_percent, 100.0);

    let _ = shell.update(Message::Inspector(inspector_pane::Message::ResetSpeed));
    assert_eq!(shell.inspector_selection().unwrap().attrs.speed_percent, 100.0);

    let _ = shell.update(Message::Undo);
    assert_eq!(
        shell.layer_count(),
        0,
        "reset(no-op のはず)が Undo エントリを積んでいる — 1回の Undo で AddLayer が消えない"
    );
}

/// **M13**: ロック layer は Speed 編集も拒否し、理由を status 帯へ出す
/// (`Document::apply` の `check_not_locked` — move/trim と同じ拒否の型)。
#[test]
fn locked_layer_rejects_the_speed_edit_and_reports_status() {
    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);
    let id = shell.timeline_rows()[0].id;
    let _ = shell.update(Message::Timeline(timeline_pane::Message::ToggleLock(id)));

    let _ = shell.update(Message::Inspector(inspector_pane::Message::SpeedInput("200".to_owned())));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::SpeedSubmit));

    assert_eq!(
        shell.inspector_selection().unwrap().attrs.speed_percent,
        100.0,
        "locked なのに speed が変わった"
    );
    let status = shell.status();
    assert!(status.is_some(), "locked 拒否が status へ出ていない(M13 違反)");
    assert!(
        status.unwrap().contains("locked"),
        "拒否理由が locked に触れていない: {status:?}"
    );
}

/// 0%/負の入力は拒否(供給側 決定3)— speed は変わらず status に理由が出る。
#[test]
fn zero_or_negative_percent_is_rejected_with_status() {
    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);

    let _ = shell.update(Message::Inspector(inspector_pane::Message::SpeedInput("0".to_owned())));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::SpeedSubmit));

    assert_eq!(
        shell.inspector_selection().unwrap().attrs.speed_percent,
        100.0,
        "0% を受理してしまっている"
    );
    assert!(shell.status().is_some(), "0% 拒否が status に出ていない(M13 違反)");
}
