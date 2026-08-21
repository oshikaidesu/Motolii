//! 運転席 — 実時間再生(A2、2026-08-21)の落ちるテスト先行(発注書 ORACLE
//! a〜e)。**デバイス抽象はフェイクで — A1と同じ手**: `PlaybackSession::
//! for_simulation`(実cpal/producerを一切開かない)で組んだセッションを
//! `Shell::debug_start_playback_with_session`で直接採用し、
//! `PlaybackCounters::advance_supplied_for_simulation`で手動供給を進めてから
//! `Shell::update(Message::PlaybackTick)`を叩く。
//!
//! **`Message::TogglePlayback`は「既に再生中→Pause」の向きだけ試験する**
//! (停止中→Playは`transport::open_real_playback`が実デバイスを開こうとする
//! 本番専用経路 — ORACLE「実デバイス経路はテスト不能でよい」の対象。
//! 停止中に`TogglePlayback`を叩くテストは意図的に置かない)。
//!
//! fixture(`Shell::new_fixture()`)実測値(`fixture.rs`): comp 30fps・尺1800
//! フレーム(終端フレーム=1799)・playhead既定900・15層(先頭行の掴みが可能)。

use std::sync::Arc;

use motolii_audio::{DeviceWaitLatency, PlaybackClock, PlaybackCounters, PlaybackSession};
use motolii_core::{Fps, RationalTime};
use motolii_shell::timeline_pane::{self, BarPart};
use motolii_shell::{Message, Shell};

fn fixture_fps() -> Fps {
    Fps::try_new(30, 1).expect("30fps")
}

/// フェイクの再生セッションを`frame`(comp フレーム)から組む。実cpal/producer
/// は一切開かない(`PlaybackSession::for_simulation`)。`Arc<PlaybackCounters>`
/// を呼び出し側へ返すので、テストは`advance_supplied_for_simulation`で直接
/// 供給を進められる(`motolii_audio::clock`のA1試験と同じ手)。
fn fake_session_at(frame: i64) -> (PlaybackSession, Arc<PlaybackCounters>) {
    let counters = Arc::new(PlaybackCounters::default());
    let wait = Arc::new(DeviceWaitLatency::default());
    let mut clock =
        PlaybackClock::new(Arc::clone(&counters), wait, 48_000).expect("48kHzは有効なsample_rate");
    let at = RationalTime::try_from_frame(frame, fixture_fps()).expect("fixtureのfpsは有効");
    clock.start(at);
    (PlaybackSession::for_simulation(clock), counters)
}

// ---------------------------------------------------------------------------
// (a) Play → tick で playhead が進む
// ---------------------------------------------------------------------------

#[test]
fn a_playback_tick_advances_playhead_from_clock_position() {
    let mut shell = Shell::new_fixture().0;
    assert_eq!(shell.session().playhead, 900, "fixtureの既定playheadが想定と違う");

    let (session, counters) = fake_session_at(900);
    shell.debug_start_playback_with_session(session);
    assert!(shell.is_playing(), "セッション採用直後はPlay中のはず");

    // +1.0秒(48,000サンプル@48kHz) = +30フレーム@30fps。
    counters.advance_supplied_for_simulation(48_000);
    let _ = shell.update(Message::PlaybackTick);

    assert_eq!(shell.session().playhead, 930, "tickでplayheadが進んでいない");
    assert!(shell.is_playing(), "tickだけでPlay状態が解除されてはいけない");
}

// ---------------------------------------------------------------------------
// (b) Pause で止まる
// ---------------------------------------------------------------------------

#[test]
fn b_toggle_playback_pauses_and_freezes_playhead() {
    let mut shell = Shell::new_fixture().0;
    let (session, counters) = fake_session_at(900);
    shell.debug_start_playback_with_session(session);

    counters.advance_supplied_for_simulation(48_000);
    let _ = shell.update(Message::PlaybackTick);
    assert_eq!(shell.session().playhead, 930);

    let _ = shell.update(Message::TogglePlayback);
    assert!(!shell.is_playing(), "TogglePlaybackでPauseできていない");
    assert_eq!(shell.session().playhead, 930, "Pause直前の位置が保存されていない");

    // Pause後にcountersがさらに進んでも(実デバイスの非同期停止競合を模す)、
    // セッション自体は既にstopしているのでtickはplayheadを動かさない。
    counters.advance_supplied_for_simulation(48_000);
    let _ = shell.update(Message::PlaybackTick);
    assert_eq!(shell.session().playhead, 930, "停止後もtickがplayheadを動かしている");
}

// ---------------------------------------------------------------------------
// (c) 再生中の scrub = seek
// ---------------------------------------------------------------------------

#[test]
fn c_scrub_while_playing_seeks_without_stopping() {
    let mut shell = Shell::new_fixture().0;
    let (session, counters) = fake_session_at(900);
    shell.debug_start_playback_with_session(session);

    counters.advance_supplied_for_simulation(48_000);
    let _ = shell.update(Message::PlaybackTick);
    assert_eq!(shell.session().playhead, 930);

    let _ = shell.update(Message::ScrubTo(500));
    assert_eq!(shell.session().playhead, 500, "scrubで即座にplayheadが跳んでいない");
    assert!(
        shell.is_playing(),
        "再生中のscrubでPlayが解除されてはいけない(seekは走行状態を変えない)"
    );

    // seek後もその位置から供給に応じて進む(seekはcountersを触らない原点の
    // 付け替えなので、供給の続きはseek先を基準に積算される)。
    counters.advance_supplied_for_simulation(48_000); // +30フレーム
    let _ = shell.update(Message::PlaybackTick);
    assert_eq!(shell.session().playhead, 530, "seek後の供給がseek先から積算されていない");
}

/// `timeline_pane::Message::ScrubTo`(Timeline側の空白部クリック)も同じ
/// `Shell::scrub_to`を通る(pane split survey §3.2 exception 1)ので同様に
/// seekになる。
#[test]
fn c_timeline_scrub_message_also_seeks_while_playing() {
    let mut shell = Shell::new_fixture().0;
    let (session, _counters) = fake_session_at(900);
    shell.debug_start_playback_with_session(session);

    let _ = shell.update(Message::Timeline(timeline_pane::Message::ScrubTo(200)));
    assert_eq!(shell.session().playhead, 200);
    assert!(shell.is_playing(), "Timeline経由のscrubでもPlayは解除されない");
}

// ---------------------------------------------------------------------------
// (d) ドラッグ中は Space が効かない(正典 §2 拘束5)
// ---------------------------------------------------------------------------

#[test]
fn d_space_is_ignored_while_dragging_a_clip() {
    let mut shell = Shell::new_fixture().0;
    let row = shell.timeline_rows()[0].clone();

    let _ = shell.update(Message::Timeline(timeline_pane::Message::BarGrabbed {
        layer: row.id,
        part: BarPart::Body,
        at_frame: row.start,
    }));

    let _ = shell.update(Message::TogglePlayback);
    assert!(
        !shell.is_playing(),
        "ドラッグ中にSpaceでPlayが始まってしまっている(正典§2拘束5違反)"
    );

    // 後始末(ドラッグ状態を残さない)。
    let _ = shell.update(Message::Timeline(timeline_pane::Message::DragCancelled));
}

/// ドラッグ中に押した Space は、再生中だったとしても Pause できない
/// (拘束5「再生と掴みは相互排他」— 逆方向も塞ぐ)。
#[test]
fn d_space_is_ignored_while_dragging_even_when_already_playing() {
    let mut shell = Shell::new_fixture().0;
    let (session, _counters) = fake_session_at(900);
    shell.debug_start_playback_with_session(session);
    assert!(shell.is_playing());

    let row = shell.timeline_rows()[0].clone();
    let _ = shell.update(Message::Timeline(timeline_pane::Message::BarGrabbed {
        layer: row.id,
        part: BarPart::Body,
        at_frame: row.start,
    }));

    let _ = shell.update(Message::TogglePlayback);
    assert!(
        shell.is_playing(),
        "ドラッグ中はSpaceが無効なので、再生中のままのはず(Pauseされてはいけない)"
    );

    let _ = shell.update(Message::Timeline(timeline_pane::Message::DragCancelled));
}

// ---------------------------------------------------------------------------
// (e) comp 終端で停止
// ---------------------------------------------------------------------------

#[test]
fn e_playback_stops_automatically_at_the_composition_end() {
    let mut shell = Shell::new_fixture().0;
    let (session, counters) = fake_session_at(1_798); // 終端(尺1800→最終フレーム1799)の直前

    shell.debug_start_playback_with_session(session);

    // +1.0秒(30フレーム相当@30fps)供給 — 終端を大きく超える。
    counters.advance_supplied_for_simulation(48_000);
    let _ = shell.update(Message::PlaybackTick);

    assert_eq!(
        shell.session().playhead,
        1_799,
        "終端(comp_end_frame)で頭打ちになっていない"
    );
    assert!(!shell.is_playing(), "終端到達で自動Pauseできていない");

    // 終端で止まった後はtickを送っても動かない(セッションが無い)。
    let _ = shell.update(Message::PlaybackTick);
    assert_eq!(shell.session().playhead, 1_799);
}
