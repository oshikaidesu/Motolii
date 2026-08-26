//! Transport 帯(map 1041-1045 採用済・1138 Timecode)の落ちるテスト先行
//! (発注 2026-08-22「タイムラインに再生ボタンが無いのは謎」)。
//!
//! オラクル:
//! - (a) transport 帯が存在し、5ボタンが業界慣習順(S0: 先頭/戻/再生/進/末尾)
//!   で並ぶこと・各ボタンが既存の再生系 Message 名へ素直に写ること
//! - (b) Timecode 表示(1138)が playhead に追随すること(フレーム番号+秒)
//! - (c) transport の Message は pane-local → shell が畳む既存の型に従い、
//!   `PaneState::update` では no-op(実結線は shell の先取り — 5例外と同型)
//! - (d) 寸法は dimensions.json の `timeline_transport` 節(裁定178: Rust
//!   直書き禁止)・S1(踏面はグリフより大きく帯高いっぱい)

use motolii_shell_state::Session;
use motolii_store::{Composition, Document, Fps, Intent};
use motolii_timeline_pane::tokens::{Colors, Dimensions};
use motolii_timeline_pane::{transport_spec, transport_spec_with_frame_draft, Message, PaneState, TimelinePane};

fn fps30() -> Fps {
    Fps::try_new(30, 1).expect("30/1 は正の既約 fps")
}

// ---------------------------------------------------------------------------
// (a) 5ボタンの存在・順序(S0)・Message 写像
// ---------------------------------------------------------------------------

#[test]
fn s0_the_five_buttons_run_start_back_play_forward_end() {
    let spec = transport_spec(0, Some(fps30()), false, false);
    assert_eq!(spec.buttons.len(), 5, "transport は5ボタン(|◀ ◂ ▶ ▸ ▶|)");
    assert!(
        matches!(spec.buttons[0].message, Message::JumpPlayheadToStart),
        "先頭ボタンが JumpPlayheadToStart でない: {:?}",
        spec.buttons[0].message
    );
    assert!(
        matches!(spec.buttons[1].message, Message::StepPlayhead(-1)),
        "1コマ戻ボタンが StepPlayhead(-1) でない: {:?}",
        spec.buttons[1].message
    );
    assert!(
        matches!(spec.buttons[2].message, Message::TogglePlayback),
        "中央ボタンが TogglePlayback でない: {:?}",
        spec.buttons[2].message
    );
    assert!(
        matches!(spec.buttons[3].message, Message::StepPlayhead(1)),
        "1コマ進ボタンが StepPlayhead(1) でない: {:?}",
        spec.buttons[3].message
    );
    assert!(
        matches!(spec.buttons[4].message, Message::JumpPlayheadToEnd),
        "末尾ボタンが JumpPlayheadToEnd でない: {:?}",
        spec.buttons[4].message
    );
}

#[test]
fn transport_exposes_previous_and_next_property_key_buttons() {
    let spec = transport_spec(0, Some(fps30()), false, false);
    assert!(matches!(spec.keyframe_buttons[0].message, Message::JumpToPreviousKeyframe));
    assert!(matches!(spec.keyframe_buttons[1].message, Message::JumpToNextKeyframe));
}

/// Play ボタンは「次に何が起きるか」を示す(shell 旧 Play バーと同じ慣習):
/// 停止中=▶(押すと再生)、再生中=⏸(押すと停止)。再生中だけ状態 ink
/// (accent、rail の M/S/L active と同じ器)が立つ。絵は裁定186 で文字
/// グリフ→ SVG アイコン(`motolii-icons`)へ置換 — Message 写像は不変。
#[test]
fn the_play_button_face_flips_between_play_and_pause() {
    let paused = transport_spec(0, Some(fps30()), false, false);
    let playing = transport_spec(0, Some(fps30()), true, false);
    assert_ne!(
        paused.buttons[2].icon, playing.buttons[2].icon,
        "再生中と停止中で Play ボタンの顔が変わっていない"
    );
    assert!(!paused.buttons[2].active, "停止中に active が立っている");
    assert!(playing.buttons[2].active, "再生中に active が立っていない");
    // 他の4ボタンは状態を持たない(押し口であって器ではない)。
    for i in [0usize, 1, 3, 4] {
        assert!(!paused.buttons[i].active && !playing.buttons[i].active);
    }
}

// ---------------------------------------------------------------------------
// (b) Timecode(1138)は playhead に追随する — フレーム番号+秒
// ---------------------------------------------------------------------------

#[test]
fn the_timecode_follows_the_playhead() {
    let at_zero = transport_spec(0, Some(fps30()), false, false);
    assert_eq!(at_zero.frames, "0");
    assert_eq!(at_zero.seconds.as_deref(), Some("0.00"));

    let at_150 = transport_spec(150, Some(fps30()), false, false);
    assert_eq!(at_150.frames, "150", "フレーム番号が playhead に追随しない");
    assert_eq!(at_150.seconds.as_deref(), Some("5.00"), "秒表示が playhead に追随しない");
}

/// 非整数 fps(29.97 = 30000/1001)でも秒は有理 fps から正しく出る。
#[test]
fn the_seconds_use_the_rational_fps() {
    let ntsc = Fps::try_new(30000, 1001).expect("29.97 fps");
    let spec = transport_spec(300, Some(ntsc), false, false);
    assert_eq!(spec.seconds.as_deref(), Some("10.01"), "300f / 29.97fps ≈ 10.01s");
}

/// comp が無い(fps を引けない)時は秒を出さない — 黙って誤った値を描くより
/// 描かない方がまし(marker_frame の M13 と同じ姿勢)。フレーム番号は残る。
#[test]
fn without_a_composition_the_seconds_go_dark_but_frames_remain() {
    let spec = transport_spec(42, None, false, false);
    assert_eq!(spec.frames, "42");
    assert!(spec.seconds.is_none(), "fps 無しで秒をでっち上げている");
}

#[test]
fn the_frame_input_spec_shows_the_draft_until_commit() {
    let spec = transport_spec_with_frame_draft(42, Some(fps30()), false, false, Some("1"));
    assert_eq!(spec.frames, "1");
    assert_eq!(spec.seconds.as_deref(), Some("1.40"), "入力途中の秒表示まで先走っている");
}

/// transport 帯の存在(S6: 常設可視): `view_with_transport()` が pane の絵の
/// 上に帯を積んで widget 木を組み終えること(スモーク — 絵と意味の対応自体は
/// 上の spec 試験が正本)。既存 `view()` は無改変(shell の生座標 click 検分を
/// 壊さない — `view_with_transport` の doc 参照)。
#[test]
fn the_transport_band_composes_over_the_pane_view() {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: fps30(),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .expect("comp 設定");
    let store = doc.view();
    let session = Session::default();
    let pane = TimelinePane::new(
        &store,
        &session,
        Dimensions::default(),
        Colors::default(),
        iced::keyboard::Modifiers::default(),
    )
    .with_playing(true);
    let _band_and_body = pane.view_with_transport();
}

// ---------------------------------------------------------------------------
// (c) transport Message は PaneState::update では no-op(shell 先取りの型)
// ---------------------------------------------------------------------------

#[test]
fn transport_messages_are_a_no_op_in_the_pane_write_path() {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: fps30(),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .expect("comp 設定");
    let mut session = Session::default();
    session.playhead = 7;
    let mut pane = PaneState::new();

    for message in [
        Message::TogglePlayback,
        Message::StepPlayhead(-1),
        Message::StepPlayhead(1),
        Message::JumpPlayheadToStart,
        Message::JumpPlayheadToEnd,
    ] {
        let reason = pane.update(
            message,
            &mut doc,
            &mut session,
            iced::keyboard::Modifiers::default(),
        );
        assert!(reason.is_none(), "transport Message が拒否理由を返している");
    }
    // pane 側は playhead も Document も触らない(実結線は shell の先取り)。
    assert_eq!(session.playhead, 7, "pane 側で playhead を動かしてしまっている");
    assert_eq!(doc.view().layers().len(), 0);
}

// ---------------------------------------------------------------------------
// (d) 寸法は dimensions.json の timeline_transport 節(裁定178)+S1
// ---------------------------------------------------------------------------

/// 正本 JSON(tokens crate の dimensions.json)に timeline_transport 節が
/// 実在し、struct の既定値と一致する(既定値だけあって正本に無い、を防ぐ)。
#[test]
fn the_transport_tokens_live_in_the_dimensions_json() {
    let dims = Dimensions::load_from_path(&Dimensions::debug_source_path())
        .expect("正本 dimensions.json を読めない");
    let defaults = Dimensions::default();
    assert_eq!(dims.timeline_transport_height, defaults.timeline_transport_height);
    assert_eq!(dims.timeline_transport_button_width, defaults.timeline_transport_button_width);
    assert_eq!(dims.timeline_transport_gap, defaults.timeline_transport_gap);
}

/// S1: ボタン踏面(幅×帯高いっぱい)はグリフ(body_text 字寸)より大きい。
#[test]
fn s1_the_button_tread_is_larger_than_its_glyph() {
    let dims = Dimensions::default();
    assert!(
        dims.timeline_transport_button_width > dims.theme().text.body,
        "踏面幅がグリフ字寸以下: {} <= {}",
        dims.timeline_transport_button_width,
        dims.theme().text.body
    );
    assert!(
        dims.timeline_transport_height > dims.theme().text.body,
        "帯高(=踏面高)がグリフ字寸以下: {} <= {}",
        dims.timeline_transport_height,
        dims.theme().text.body
    );
}

/// ui_scale は transport 寸法にも掛かる(適用点1箇所の仲間に入っている)。
#[test]
fn the_transport_tokens_scale_with_ui_scale() {
    let dims = Dimensions::default();
    let scaled = dims.scaled(1.5);
    assert_eq!(scaled.timeline_transport_height, dims.timeline_transport_height * 1.5);
    assert_eq!(
        scaled.timeline_transport_button_width,
        dims.timeline_transport_button_width * 1.5
    );
    assert_eq!(scaled.timeline_transport_gap, dims.timeline_transport_gap * 1.5);
}
