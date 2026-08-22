//! 運転席 — 第5波 shell 結線(TL+ の B21 JKL シャトル+ B18 作業範囲/ループ)の
//! 落ちるテスト先行。`nav_drive.rs`/`playback_drive.rs` と同じ形: `Shell::update`
//! だけを叩き、状態は公開読み口(`session()`/`timeline_work_area()`/
//! `timeline_loop_enabled()`)で検分する。
//!
//! fixture(`Shell::new_fixture()`)実測値(`fixture.rs`): comp 30fps・尺 1800
//! フレーム(終端 1799)・playhead 既定 900。
//!
//! - シャトルの意味(1→2→4→8 の状態機械)は `timeline_pane::shuttle` の単体
//!   試験が持つ — ここで見るのは**結線**: `Message::Timeline(Shuttle(..))` →
//!   `Message::PlaybackTick` で playhead が `rate` フレーム動くこと。
//! - ループ折り返しの意味は `timeline_pane::work_area::advanced_playhead` の
//!   単体試験が持つ — ここで見るのは shell の tick(シャトル/実時間 transport
//!   の両方)が実際にそこを通ること。

use std::sync::Arc;

use motolii_audio::{DeviceWaitLatency, PlaybackClock, PlaybackCounters, PlaybackSession};
use motolii_core::{Fps, RationalTime};
use motolii_shell::timeline_pane::{LoopBandPart, Message as TlMessage, ShuttleCommand, WorkArea};
use motolii_shell::{Message, Shell};

fn fixture_fps() -> Fps {
    Fps::try_new(30, 1).expect("30fps")
}

/// フェイクの再生セッション(`playback_drive.rs` と同じ手 — 実 cpal は開かない)。
fn fake_session_at(frame: i64) -> (PlaybackSession, Arc<PlaybackCounters>) {
    let counters = Arc::new(PlaybackCounters::default());
    let wait = Arc::new(DeviceWaitLatency::default());
    let mut clock =
        PlaybackClock::new(Arc::clone(&counters), wait, 48_000).expect("48kHzは有効なsample_rate");
    let at = RationalTime::try_from_frame(frame, fixture_fps()).expect("fixtureのfpsは有効");
    clock.start(at);
    (PlaybackSession::for_simulation(clock), counters)
}

fn shuttle(shell: &mut Shell, command: ShuttleCommand) {
    let _ = shell.update(Message::Timeline(TlMessage::Shuttle(command)));
}

fn tick(shell: &mut Shell) {
    let _ = shell.update(Message::PlaybackTick);
}

/// playhead を `frame` に置いてから作業範囲 [in, out) を B/N の動詞で引く
/// (`SetWorkAreaIn`/`SetWorkAreaOut` — keymap B/N の実体)。
fn mark_work_area(shell: &mut Shell, in_frame: i64, out_frame: i64) {
    let _ = shell.update(Message::ScrubTo(in_frame));
    let _ = shell.update(Message::Timeline(TlMessage::SetWorkAreaIn));
    let _ = shell.update(Message::ScrubTo(out_frame));
    let _ = shell.update(Message::Timeline(TlMessage::SetWorkAreaOut));
    assert_eq!(
        shell.timeline_work_area(),
        Some(WorkArea { start: in_frame, end: out_frame }),
        "B/N(SetWorkAreaIn/Out)で作業範囲が立っていない"
    );
}

// ---------------------------------------------------------------------------
// シャトル → playhead が動く(結線一覧 3)
// ---------------------------------------------------------------------------

/// L(順)= 1 tick に 1 フレーム、連打で倍速(2x)。K = 停止。
#[test]
fn shuttle_forward_advances_the_playhead_by_rate_frames_per_tick() {
    let mut shell = Shell::new_fixture().0;
    assert_eq!(shell.session().playhead, 900);

    shuttle(&mut shell, ShuttleCommand::Forward); // 1x
    tick(&mut shell);
    assert_eq!(shell.session().playhead, 901, "L 1打で 1 frame/tick になっていない");

    shuttle(&mut shell, ShuttleCommand::Forward); // 連打 → 2x
    tick(&mut shell);
    assert_eq!(shell.session().playhead, 903, "L 連打で 2 frame/tick になっていない");

    shuttle(&mut shell, ShuttleCommand::Stop); // K
    tick(&mut shell);
    assert_eq!(shell.session().playhead, 903, "K の後も playhead が動いている");
}

/// J(逆)は鏡像 — 1 tick に 1 フレーム戻る。方向転換は等速へ戻る。
#[test]
fn shuttle_reverse_mirrors_forward_and_direction_change_resets_to_1x() {
    let mut shell = Shell::new_fixture().0;

    shuttle(&mut shell, ShuttleCommand::Reverse);
    tick(&mut shell);
    assert_eq!(shell.session().playhead, 899, "J 1打で -1 frame/tick になっていない");

    shuttle(&mut shell, ShuttleCommand::Reverse); // -2x
    tick(&mut shell);
    assert_eq!(shell.session().playhead, 897);

    shuttle(&mut shell, ShuttleCommand::Forward); // 方向転換 → +1x(慣性を持ち越さない)
    tick(&mut shell);
    assert_eq!(shell.session().playhead, 898, "方向転換が等速 1x へ戻っていない");
}

/// comp 端で clamp されたら自動停止(transport の「終端で自動 Pause」と同型)。
#[test]
fn shuttle_stops_itself_at_the_composition_edge() {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::ScrubTo(1799));

    shuttle(&mut shell, ShuttleCommand::Forward);
    tick(&mut shell);
    assert_eq!(shell.session().playhead, 1799, "終端を超えてしまっている");

    // 自動停止済み — 以後の tick でも動かない(走り続ける空タイマーにしない)。
    tick(&mut shell);
    assert_eq!(shell.session().playhead, 1799);
}

/// 再生と掴みは相互排他(拘束5): 実時間 transport 走行中にシャトルへ乗ると
/// transport 側は位置を freeze して停止する(2つの clock を併走させない)。
#[test]
fn entering_the_shuttle_stops_the_real_time_transport_first() {
    let mut shell = Shell::new_fixture().0;
    let (session, counters) = fake_session_at(900);
    shell.debug_start_playback_with_session(session);
    counters.advance_supplied_for_simulation(48_000); // +30 frames
    tick(&mut shell);
    assert_eq!(shell.session().playhead, 930);

    shuttle(&mut shell, ShuttleCommand::Forward);
    assert!(!shell.is_playing(), "シャトルへ乗ったのに transport が走り続けている");
    tick(&mut shell);
    assert_eq!(shell.session().playhead, 931, "シャトルが freeze 位置から進んでいない");
}

/// Space(TogglePlayback)はシャトル走行中なら「停止」— 実時間再生を重ねて
/// 起動しない(Play‖Pause の「再生中→停止」の読みの延長)。
#[test]
fn space_during_a_shuttle_stops_it_instead_of_starting_real_playback() {
    let mut shell = Shell::new_fixture().0;
    shuttle(&mut shell, ShuttleCommand::Forward);
    tick(&mut shell);
    assert_eq!(shell.session().playhead, 901);

    let _ = shell.update(Message::TogglePlayback);
    assert!(!shell.is_playing(), "シャトル中の Space が実デバイス再生を起動した");
    tick(&mut shell);
    assert_eq!(shell.session().playhead, 901, "Space の後もシャトルが走っている");
}

// ---------------------------------------------------------------------------
// ループ折り返し(結線一覧 4 — `advanced_playhead` の実効化)
// ---------------------------------------------------------------------------

/// シャトル(倍速 = 1 tick に複数フレーム)でも範囲長で正しく畳まれる。
#[test]
fn shuttle_playback_wraps_inside_the_work_area_when_looping() {
    let mut shell = Shell::new_fixture().0;
    mark_work_area(&mut shell, 890, 910);
    let _ = shell.update(Message::Timeline(TlMessage::ToggleLoop));
    assert!(shell.timeline_loop_enabled(), "ToggleLoop が効いていない");

    let _ = shell.update(Message::ScrubTo(908));
    shuttle(&mut shell, ShuttleCommand::Forward); // 1x
    shuttle(&mut shell, ShuttleCommand::Forward); // 2x
    shuttle(&mut shell, ShuttleCommand::Forward); // 4x
    tick(&mut shell);
    // advanced_playhead(908, +4, [890,910)) = 890 + (912-890) % 20 = 892。
    assert_eq!(shell.session().playhead, 892, "シャトルがループ帯で折り返していない");
}

/// 実時間 transport の tick もループ帯で折り返す — clock は線形に進み続けても
/// playhead は `start + (clock - start) % len` に畳まれる。
#[test]
fn real_time_playback_wraps_inside_the_work_area_when_looping() {
    let mut shell = Shell::new_fixture().0;
    mark_work_area(&mut shell, 890, 910);
    let _ = shell.update(Message::Timeline(TlMessage::ToggleLoop));

    let _ = shell.update(Message::ScrubTo(900));
    let (session, counters) = fake_session_at(900);
    shell.debug_start_playback_with_session(session);

    counters.advance_supplied_for_simulation(48_000); // +30 frames → clock=930
    tick(&mut shell);
    // 890 + (930 - 890) % 20 = 890。範囲の外へ出ない・自動 Pause もしない。
    assert_eq!(shell.session().playhead, 890, "transport 再生がループ帯で折り返していない");
    assert!(shell.is_playing(), "ループ折り返しが再生を止めてしまった");

    counters.advance_supplied_for_simulation(8_000); // +5 frames → clock=935
    tick(&mut shell);
    assert_eq!(shell.session().playhead, 895, "折り返し後の tick が安定していない");
}

/// ループ off・範囲の外は従来どおり(素通し+終端で自動 Pause)— 罠にしない。
#[test]
fn playback_outside_the_work_area_or_with_loop_off_passes_through() {
    let mut shell = Shell::new_fixture().0;
    mark_work_area(&mut shell, 100, 200);
    // ループ off のまま(既定)。playhead 900 は範囲の外。
    let _ = shell.update(Message::ScrubTo(900));
    let (session, counters) = fake_session_at(900);
    shell.debug_start_playback_with_session(session);
    counters.advance_supplied_for_simulation(48_000);
    tick(&mut shell);
    assert_eq!(shell.session().playhead, 930, "範囲外の再生が影響を受けている");
}

// ---------------------------------------------------------------------------
// Esc = ループ帯ドラッグのキャンセル(結線一覧 2)
// ---------------------------------------------------------------------------

/// ループ帯は live 更新なので、Esc は「掴んだ瞬間の値」への**復元**でなければ
/// ならない(捨てるだけでは戻らない — `cancel_loop_drag` doc)。新規ドラッグは
/// 引いた瞬間 loop on になるので、その on も戻る。
#[test]
fn escape_restores_the_work_area_grabbed_before_a_loop_band_drag() {
    let mut shell = Shell::new_fixture().0;
    assert_eq!(shell.timeline_work_area(), None);

    let _ = shell.update(Message::Timeline(TlMessage::LoopBandGrabbed {
        part: LoopBandPart::Blank,
        at_frame: 100,
    }));
    let _ = shell.update(Message::Timeline(TlMessage::LoopDragMoved { at_frame: 160 }));
    assert_eq!(
        shell.timeline_work_area(),
        Some(WorkArea { start: 100, end: 160 }),
        "ドラッグ中の live 更新が見えていない"
    );
    assert!(shell.timeline_loop_enabled(), "新規ドラッグは引いたら即 on のはず");

    let _ = shell.update(Message::EscapePressed);
    assert_eq!(shell.timeline_work_area(), None, "Esc が作業範囲を復元していない");
    assert!(!shell.timeline_loop_enabled(), "Esc が loop on/off を復元していない");
}

// ---------------------------------------------------------------------------
// 作業範囲の先頭/末尾へ(map 1064 — Shift+Home/End の実体)
// ---------------------------------------------------------------------------

#[test]
fn jump_to_work_area_start_and_end_land_on_the_half_open_bounds() {
    let mut shell = Shell::new_fixture().0;
    mark_work_area(&mut shell, 890, 910);

    let _ = shell.update(Message::ScrubTo(0));
    let _ = shell.update(Message::JumpToWorkAreaStart);
    assert_eq!(shell.session().playhead, 890, "作業範囲の先頭へ跳んでいない");

    let _ = shell.update(Message::JumpToWorkAreaEnd);
    assert_eq!(shell.session().playhead, 909, "末尾 = 半開の end-1 でない");
}

/// 範囲が無ければ no-op(`JumpClipEdge` と同じ「跳ぶ先が無ければ動かない」)。
#[test]
fn work_area_jumps_are_a_no_op_without_a_work_area() {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::ScrubTo(42));
    let _ = shell.update(Message::JumpToWorkAreaStart);
    assert_eq!(shell.session().playhead, 42);
    let _ = shell.update(Message::JumpToWorkAreaEnd);
    assert_eq!(shell.session().playhead, 42);
}

/// 帯が無い時の ToggleLoop は理由つき拒否(M13: 無反応ゼロ)— pane 側の意味を
/// shell の status 帯まで運べていることの結線検分。
#[test]
fn toggling_the_loop_without_a_work_area_reports_a_reason() {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::Timeline(TlMessage::ToggleLoop));
    assert!(shell.status().is_some(), "帯なし ToggleLoop の拒否理由が status 帯に出ていない");
    assert!(!shell.timeline_loop_enabled());
}
