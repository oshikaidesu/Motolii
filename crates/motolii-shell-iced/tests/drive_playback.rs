//! 再生機構の運転席(2026-08-19 iced 再生機構移植レーン)— 窓を開かずに
//! Space/L・transport の2ボタン・tick の配線を確かめる。
//!
//! ## 審判の分担
//!
//! - **playhead が時間で進む**ことの決定的な証明(`dt` を直接動かす、実
//!   sleep は使わない)は `motolii-ui` 側のユニットテスト
//!   (`crates/motolii-ui/src/timeline_editor/playback.rs`)が持つ。
//! - ここは「iced の Message / keyboard / canvas 配線が、その証明済みの
//!   `TimelineEditor` メソッドへ実際に届くか」だけを見る。`playback_tick_
//!   advances_the_playhead_over_real_time` の1本だけは実配線の証拠として
//!   短い real sleep を使う(30ms、composition 全長 16s に対して十分短い)。
//! - **停止時に Stage が正しい絵になる**ことの pixel oracle は
//!   `playback_stage_pixels.rs`。

mod common;

use std::path::PathBuf;

use common::drain;
use iced::keyboard::key::Named;
use motolii_shell_iced::timeline::semantics::{play_pause_button_rect, to_start_button_rect, PaneGeometry};
use motolii_shell_iced::{view, Message, Outcome, ScriptedPrompts, Shell};
use motolii_ui::blitz_shell::UiIntent;
use motolii_ui::timeline_editor::lab_fixture;

/// lab の fixture(soundtrack 無し)を一時 project に保存する。
/// `drive_timeline.rs::fixture_project` と同じ組み方(この file は自己完結
/// させる — 既存 test file を並走レーンと取り合わない)。
fn fixture_project(tag: &str) -> PathBuf {
    let (document, _names) = lab_fixture();
    let dir = motolii_testkit::tmp_dir(&format!("iced_playback_{tag}"));
    let path = dir.join("project.json");
    let mut session =
        motolii_doc::ProjectSession::acquire(&path, &motolii_doc::ResourceLimits::production())
            .expect("acquire temp project");
    session
        .save_document(&document, &motolii_doc::SaveOptions::default())
        .expect("save temp project");
    drop(session);
    path
}

/// Open ボタンで fixture project へ座る(dialog は台本が答える)。
fn seated_shell(path: &PathBuf) -> Shell {
    let mut shell = Shell::new(ScriptedPrompts {
        open_project_path: Some(path.clone()),
        ..ScriptedPrompts::default()
    });
    let pressed = common::press(
        iced_test::simulator(view(&shell)),
        motolii_shell_iced::view::OPEN_PROJECT,
    );
    drain(&mut shell, pressed);
    assert!(shell.is_seated(), "fixture project へ座れていない");
    shell
}

/// キーボードだけの1フレーム。
fn key_step(shell: &mut Shell, events: impl IntoIterator<Item = iced::event::Event>) -> Outcome {
    let messages = common::feed(iced_test::simulator(view(shell)), events);
    drain(shell, messages)
}

/// 修飾なしの1キー(押して離す)。
fn tap_named(named: Named) -> Vec<iced::event::Event> {
    iced_test::simulator::tap_key(iced::keyboard::Key::Named(named), None).collect()
}

/// 修飾なしの1文字キー。
fn tap_char(c: char) -> Vec<iced::event::Event> {
    iced_test::simulator::tap_key(iced::keyboard::Key::Character(c.to_string().into()), None)
        .collect()
}

/// Timeline pane container の実 bounds(窓座標)。`drive_timeline.rs` と同じ
/// 理由(4面合成で canvas は窓の左上には立たない)。
fn timeline_bounds(shell: &Shell) -> iced::Rectangle {
    let mut probe = iced_test::simulator(view(shell));
    let target: iced_test::selector::Target = probe
        .find(motolii_shell_iced::view::TIMELINE_PANE_ID)
        .expect("Timeline pane container が座席上に立っていない");
    target.bounds()
}

fn pane_geometry(shell: &Shell) -> PaneGeometry {
    let bounds = timeline_bounds(shell);
    PaneGeometry {
        width: bounds.width,
        height: bounds.height,
        wave_h: 0.0, // fixture に soundtrack は無い
    }
}

/// pane ローカル座標 `at` を1回だけ押す(押す→離すの往復はしない — hit_test は
/// 押下だけで発火するので `pointer_step` と同じ最小形)。
fn click_at(shell: &mut Shell, at: iced::Point) -> Outcome {
    let origin = timeline_bounds(shell);
    let at = iced::Point::new(at.x + origin.x, at.y + origin.y);
    let mut ui = iced_test::simulator(view(shell));
    ui.point_at(at);
    let events = vec![
        iced::Event::Mouse(iced::mouse::Event::CursorMoved { position: at }),
        iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
    ];
    let _ = ui.simulate(events);
    let messages: Vec<Message> = ui.into_messages().collect();
    drain(shell, messages)
}

// ---------------------------------------------------------------------------
// Space / L — 実 keyboard 経路(window_input.rs の新アーム → shortcuts.rs)
// ---------------------------------------------------------------------------

#[test]
fn space_toggles_play_through_the_real_keyboard_path() {
    let path = fixture_project("space");
    let mut shell = seated_shell(&path);
    assert!(!shell.timeline_playing(), "座った直後は再生していない");

    key_step(&mut shell, tap_named(Named::Space));
    assert!(shell.timeline_playing(), "Space で再生が始まらない");

    key_step(&mut shell, tap_named(Named::Space));
    assert!(!shell.timeline_playing(), "Space の2打目で止まらない");
}

#[test]
fn l_toggles_loop_through_the_real_keyboard_path() {
    let path = fixture_project("loop");
    let mut shell = seated_shell(&path);
    assert!(!shell.timeline_loop_on(), "fixture の既定は loop off");

    key_step(&mut shell, tap_char('l'));
    assert!(shell.timeline_loop_on(), "L で loop が入らない");

    key_step(&mut shell, tap_char('l'));
    assert!(!shell.timeline_loop_on(), "L の2打目で切れない");
}

// ---------------------------------------------------------------------------
// transport の2ボタン — 実 canvas hit-test 経路(semantics.rs → canvas.rs)
// ---------------------------------------------------------------------------

#[test]
fn clicking_the_play_pause_button_toggles_playing() {
    let path = fixture_project("click_play");
    let mut shell = seated_shell(&path);
    let geometry = pane_geometry(&shell);
    let (x0, x1, y0, y1) = play_pause_button_rect(&geometry);
    let at = iced::Point::new((x0 + x1) * 0.5, (y0 + y1) * 0.5);

    assert!(!shell.timeline_playing());
    click_at(&mut shell, at);
    assert!(shell.timeline_playing(), "play ボタンを押しても再生が始まらない");

    click_at(&mut shell, at);
    assert!(!shell.timeline_playing(), "もう一度押しても一時停止しない");
}

#[test]
fn clicking_to_start_dispatches_the_set_playhead_intent() {
    let path = fixture_project("click_to_start");
    let mut shell = seated_shell(&path);
    // 先に進めておく(壁時計・soundtrack 無し fixture)。
    shell.update(Message::TogglePlayPressed);
    shell.update(Message::PlaybackTick);
    std::thread::sleep(std::time::Duration::from_millis(30));
    shell.update(Message::PlaybackTick);
    assert!(
        shell.timeline_playhead() > 0.0,
        "先に進めておく前提が崩れている(0のまま)"
    );

    let before = shell.intent_count();
    let geometry = pane_geometry(&shell);
    let (x0, x1, y0, y1) = to_start_button_rect(&geometry);
    let at = iced::Point::new((x0 + x1) * 0.5, (y0 + y1) * 0.5);
    click_at(&mut shell, at);

    assert_eq!(
        shell.timeline_playhead(),
        0.0,
        "to_start ボタンで playhead が 0 へ戻らない"
    );
    assert_eq!(
        shell.intent_count(),
        before + 1,
        "to_start は SetPlayhead intent を1件積むはず"
    );
    let last = shell.intents().pop().expect("直前に1件積んだはず");
    assert_eq!(
        last.intent,
        UiIntent::SetPlayhead { at_us: 0 },
        "to_start は既存の SetPlayhead を再利用するはず(新しい intent を作らない)"
    );
}

// ---------------------------------------------------------------------------
// tick / toggle は intent ではない(replay oracle が揺れないことの証拠)
// ---------------------------------------------------------------------------

#[test]
fn play_pause_tick_and_loop_are_not_logged_as_intents() {
    let path = fixture_project("no_intent");
    let mut shell = seated_shell(&path);
    // seated_shell 自体が OpenProject を1件積んでいる。以降それが増えないことを見る。
    let before = shell.intent_count();

    shell.update(Message::TogglePlayPressed); // play
    shell.update(Message::PlaybackTick);
    shell.update(Message::ToggleLoopPressed); // loop on
    shell.update(Message::ToggleLoopPressed); // loop off
    shell.update(Message::TogglePlayPressed); // pause

    assert_eq!(
        shell.intent_count(),
        before,
        "再生 start/stop・tick・loop の toggle が journal に載っている\
         (zoom/pan と同じ scope のはず — module doc 参照)"
    );
}

// ---------------------------------------------------------------------------
// PlaybackTick の配線: 実 wallclock で playhead が進む(短い real sleep)
// ---------------------------------------------------------------------------

#[test]
fn playback_tick_advances_the_playhead_over_real_time() {
    let path = fixture_project("tick_wallclock");
    let mut shell = seated_shell(&path);
    assert_eq!(shell.timeline_playhead(), 0.0);

    shell.update(Message::TogglePlayPressed);
    shell.update(Message::PlaybackTick);
    assert_eq!(
        shell.timeline_playhead(),
        0.0,
        "toggle 直後の1本目の tick は dt=0 のはず(last_playback_tick を \
         toggle でリセットしている)"
    );

    std::thread::sleep(std::time::Duration::from_millis(30));
    shell.update(Message::PlaybackTick);
    assert!(
        shell.timeline_playhead() > 0.0,
        "実時間が経ったのに playhead が進んでいない"
    );
    assert!(
        shell.timeline_playing(),
        "composition 全長(16s)より十分短いので、まだ再生中のはず"
    );
}
