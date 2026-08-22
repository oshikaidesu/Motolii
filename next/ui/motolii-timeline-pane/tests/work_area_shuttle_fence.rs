//! B21(再生/スクラブ)+B18(作業範囲)第1切片の落ちるテスト先行
//! (発注 2026-08-22)。オラクル:
//! - (a) ループ境界の折り返し(`work_area::advanced_playhead` — 順/逆/倍速/
//!   範囲外素通し)
//! - (b) JKL 状態機械(`shuttle::ShuttleState` — 1x→倍速→上限・方向転換
//!   リセット・K 停止)
//! - (c) work area drag と clamp(新規/リサイズ/移動/最短1フレーム/
//!   Mark/Clear 動詞 — `PaneState::update` の Message 経由で検分)
//! - (d) 既存 transport_fence 10本を壊さない(同居ファイルで無改変維持 —
//!   spec の呼び口だけ `looping` 引数が増えた)
//!
//! 細粒度の純関数テストは各モジュール(`work_area.rs`/`shuttle.rs`/`nav.rs`)
//! 内 — ここは Message→状態の統合面だけを検分する(`transport_fence.rs` の
//! (c) と同じ型)。

use motolii_shell_state::Session;
use motolii_store::{
    Composition, Document, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming,
};
use motolii_timeline_pane::{
    transport_spec, LoopBandPart, Message, PaneState, ShuttleCommand, ShuttleState, WorkArea,
};

fn fps30() -> Fps {
    Fps::try_new(30, 1).expect("30/1 は正の既約 fps")
}

fn doc_with_comp() -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: fps30(),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .expect("comp 設定");
    doc
}

fn update(pane: &mut PaneState, doc: &mut Document, session: &mut Session, message: Message) -> Option<String> {
    pane.update(message, doc, session, iced::keyboard::Modifiers::default())
}

// ---------------------------------------------------------------------------
// (c) work area: 新規ドラッグ・リサイズ・移動・clamp(Message 経由)
// ---------------------------------------------------------------------------

/// 正典 §5「空白=新規(左右どちらから引いても同じ)」+「引いたら即 on」。
#[test]
fn dragging_a_new_band_creates_the_work_area_and_turns_the_loop_on() {
    let mut doc = doc_with_comp();
    let mut session = Session::default();
    let mut pane = PaneState::new();
    assert!(pane.work_area().is_none());
    assert!(!pane.loop_enabled());

    update(&mut pane, &mut doc, &mut session, Message::LoopBandGrabbed { part: LoopBandPart::Blank, at_frame: 100 });
    update(&mut pane, &mut doc, &mut session, Message::LoopDragMoved { at_frame: 40 });
    update(&mut pane, &mut doc, &mut session, Message::LoopDragReleased);

    assert_eq!(
        pane.work_area(),
        Some(WorkArea { start: 40, end: 100 }),
        "左へ引いても [40,100) になる"
    );
    assert!(pane.loop_enabled(), "引いたら即 on(正典 §5 — 別キーで有効化させない)");
}

/// 正典 §5「端=リサイズ(反対端は掴んだ瞬間の値で固定)」。
#[test]
fn resizing_an_edge_keeps_the_opposite_end_fixed() {
    let mut doc = doc_with_comp();
    let mut session = Session::default();
    let mut pane = PaneState::new();
    update(&mut pane, &mut doc, &mut session, Message::LoopBandGrabbed { part: LoopBandPart::Blank, at_frame: 100 });
    update(&mut pane, &mut doc, &mut session, Message::LoopDragMoved { at_frame: 200 });
    update(&mut pane, &mut doc, &mut session, Message::LoopDragReleased);
    assert_eq!(pane.work_area(), Some(WorkArea { start: 100, end: 200 }));

    // In 端を掴んで右へ 150 まで — Out(200)は固定のまま。
    update(&mut pane, &mut doc, &mut session, Message::LoopBandGrabbed { part: LoopBandPart::EdgeIn, at_frame: 100 });
    update(&mut pane, &mut doc, &mut session, Message::LoopDragMoved { at_frame: 150 });
    update(&mut pane, &mut doc, &mut session, Message::LoopDragReleased);
    assert_eq!(pane.work_area(), Some(WorkArea { start: 150, end: 200 }));

    // さらに In 端で Out を追い越して 250 へ — 畳まれず [200,250) に張り直る
    // (正典 §5「追い越しで区間が畳まれない」)。
    update(&mut pane, &mut doc, &mut session, Message::LoopBandGrabbed { part: LoopBandPart::EdgeIn, at_frame: 150 });
    update(&mut pane, &mut doc, &mut session, Message::LoopDragMoved { at_frame: 250 });
    update(&mut pane, &mut doc, &mut session, Message::LoopDragReleased);
    assert_eq!(pane.work_area(), Some(WorkArea { start: 200, end: 250 }));
}

/// 正典 §5「中=平行移動」+ comp 端 clamp(長さ保存)。
#[test]
fn moving_the_band_preserves_length_and_clamps_at_the_walls() {
    let mut doc = doc_with_comp();
    let mut session = Session::default();
    let mut pane = PaneState::new();
    update(&mut pane, &mut doc, &mut session, Message::LoopBandGrabbed { part: LoopBandPart::Blank, at_frame: 100 });
    update(&mut pane, &mut doc, &mut session, Message::LoopDragMoved { at_frame: 160 });
    update(&mut pane, &mut doc, &mut session, Message::LoopDragReleased);

    update(&mut pane, &mut doc, &mut session, Message::LoopBandGrabbed { part: LoopBandPart::Body, at_frame: 120 });
    update(&mut pane, &mut doc, &mut session, Message::LoopDragMoved { at_frame: 900 });
    update(&mut pane, &mut doc, &mut session, Message::LoopDragReleased);
    assert_eq!(
        pane.work_area(),
        Some(WorkArea { start: 240, end: 300 }),
        "右壁で止まり長さ 60 を保つ"
    );
}

/// 最短1フレーム保証(正典 §5): 引いただけで動かさなくても 1 フレームの帯。
#[test]
fn an_unmoved_new_drag_still_leaves_a_one_frame_band() {
    let mut doc = doc_with_comp();
    let mut session = Session::default();
    let mut pane = PaneState::new();
    update(&mut pane, &mut doc, &mut session, Message::LoopBandGrabbed { part: LoopBandPart::Blank, at_frame: 50 });
    update(&mut pane, &mut doc, &mut session, Message::LoopDragReleased);
    assert_eq!(pane.work_area(), Some(WorkArea { start: 50, end: 51 }));
}

/// キャンセルの一般化(裁定151): 右クリック相当の `LoopDragCancelled` で
/// 掴んだ瞬間の範囲と on/off へ復元する。
#[test]
fn cancelling_a_loop_drag_restores_the_grab_time_state() {
    let mut doc = doc_with_comp();
    let mut session = Session::default();
    let mut pane = PaneState::new();
    // 新規ドラッグをキャンセル → 帯なし・off へ戻る(即 on も巻き戻す)。
    update(&mut pane, &mut doc, &mut session, Message::LoopBandGrabbed { part: LoopBandPart::Blank, at_frame: 100 });
    update(&mut pane, &mut doc, &mut session, Message::LoopDragMoved { at_frame: 200 });
    update(&mut pane, &mut doc, &mut session, Message::LoopDragCancelled);
    assert_eq!(pane.work_area(), None, "新規ドラッグのキャンセルで帯が残っている");
    assert!(!pane.loop_enabled(), "即 on まで巻き戻っていない");

    // 既存帯のリサイズをキャンセル → 元の範囲へ戻る。
    update(&mut pane, &mut doc, &mut session, Message::LoopBandGrabbed { part: LoopBandPart::Blank, at_frame: 100 });
    update(&mut pane, &mut doc, &mut session, Message::LoopDragMoved { at_frame: 160 });
    update(&mut pane, &mut doc, &mut session, Message::LoopDragReleased);
    let before = pane.work_area();
    update(&mut pane, &mut doc, &mut session, Message::LoopBandGrabbed { part: LoopBandPart::EdgeOut, at_frame: 160 });
    update(&mut pane, &mut doc, &mut session, Message::LoopDragMoved { at_frame: 300 });
    update(&mut pane, &mut doc, &mut session, Message::LoopDragCancelled);
    assert_eq!(pane.work_area(), before, "リサイズのキャンセルが復元しない");
}

// ---------------------------------------------------------------------------
// (c) Mark/Clear 動詞(map 719/720/721・725/726・296/297・724/727)
// ---------------------------------------------------------------------------

#[test]
fn mark_in_and_out_set_the_work_area_at_the_playhead() {
    let mut doc = doc_with_comp();
    let mut session = Session::default();
    let mut pane = PaneState::new();

    session.playhead = 80;
    update(&mut pane, &mut doc, &mut session, Message::SetWorkAreaIn);
    assert_eq!(pane.work_area(), Some(WorkArea { start: 80, end: 300 }), "In 単独= ここから最後まで");

    session.playhead = 200;
    update(&mut pane, &mut doc, &mut session, Message::SetWorkAreaOut);
    assert_eq!(pane.work_area(), Some(WorkArea { start: 80, end: 200 }), "Out は In を保つ");
}

#[test]
fn clear_verbs_open_one_side_or_remove_the_band() {
    let mut doc = doc_with_comp();
    let mut session = Session::default();
    let mut pane = PaneState::new();
    session.playhead = 80;
    update(&mut pane, &mut doc, &mut session, Message::SetWorkAreaIn);
    session.playhead = 200;
    update(&mut pane, &mut doc, &mut session, Message::SetWorkAreaOut);

    update(&mut pane, &mut doc, &mut session, Message::ClearWorkAreaIn);
    assert_eq!(pane.work_area(), Some(WorkArea { start: 0, end: 200 }), "Clear In(719)= 先頭へ開く");
    update(&mut pane, &mut doc, &mut session, Message::ClearWorkAreaOut);
    assert_eq!(pane.work_area(), Some(WorkArea { start: 0, end: 300 }), "Clear Out(721)= 終端へ開く");
    update(&mut pane, &mut doc, &mut session, Message::ClearWorkArea);
    assert_eq!(pane.work_area(), None, "Clear In and Out(720)= 帯ごと消す");
}

/// Mark Clip / Mark Selection(724/727): 選択 clip の範囲を作業範囲へ。
/// 選択なしは理由つき拒否(M13: 無反応ゼロ)。
#[test]
fn mark_selection_takes_the_selected_clip_span_and_refuses_without_one() {
    let mut doc = doc_with_comp();
    let mut session = Session::default();
    let mut pane = PaneState::new();

    let refused = update(&mut pane, &mut doc, &mut session, Message::SetWorkAreaToSelection);
    assert!(refused.is_some(), "選択なしで黙って何もしないのは M13 違反");
    assert!(pane.work_area().is_none());

    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::Solid { rgba: [255, 0, 0, 255], width: 64, height: 64 },
                order: 1,
                timing: LayerTiming { start: 60, duration: 90, source_in: 0, ..Default::default() },
            },
        },
    ])
    .expect("layer 配置");
    session.selection = Some(layer);
    session.selected_layers = vec![layer];

    let refused = update(&mut pane, &mut doc, &mut session, Message::SetWorkAreaToSelection);
    assert!(refused.is_none());
    assert_eq!(pane.work_area(), Some(WorkArea { start: 60, end: 150 }), "clip の [start, start+duration)");
}

// ---------------------------------------------------------------------------
// (a) ループ on/off と transport の顔(map 1082/1083)
// ---------------------------------------------------------------------------

/// 「L でループ on/off(帯は消えない)」(正典 §5)+ 帯なしトグルは理由つき拒否。
#[test]
fn toggle_loop_flips_without_erasing_the_band_and_refuses_bandless() {
    let mut doc = doc_with_comp();
    let mut session = Session::default();
    let mut pane = PaneState::new();

    let refused = update(&mut pane, &mut doc, &mut session, Message::ToggleLoop);
    assert!(refused.is_some(), "帯なしトグルが黙って飲み込まれている(M13)");

    update(&mut pane, &mut doc, &mut session, Message::LoopBandGrabbed { part: LoopBandPart::Blank, at_frame: 100 });
    update(&mut pane, &mut doc, &mut session, Message::LoopDragMoved { at_frame: 160 });
    update(&mut pane, &mut doc, &mut session, Message::LoopDragReleased);
    assert!(pane.loop_enabled());

    update(&mut pane, &mut doc, &mut session, Message::ToggleLoop);
    assert!(!pane.loop_enabled(), "off へ倒れていない");
    assert_eq!(
        pane.work_area(),
        Some(WorkArea { start: 100, end: 160 }),
        "off で帯が消えた — 引き直さず戻せない(正典 §5 違反)"
    );
    update(&mut pane, &mut doc, &mut session, Message::ToggleLoop);
    assert!(pane.loop_enabled(), "戻しの on が効かない");
}

/// transport 帯のループトグルは状態の器(裁定179): `active` が on/off に追随し、
/// 押し口は `Message::ToggleLoop`。
#[test]
fn the_transport_loop_button_reflects_the_loop_state() {
    let off = transport_spec(0, Some(fps30()), false, false);
    let on = transport_spec(0, Some(fps30()), false, true);
    assert!(!off.loop_button.active);
    assert!(on.loop_button.active);
    assert!(
        matches!(off.loop_button.message, Message::ToggleLoop),
        "ループボタンが ToggleLoop でない: {:?}",
        off.loop_button.message
    );
    // 5ボタン(S0 順)は無改変 — ループは別の束。
    assert_eq!(off.buttons.len(), 5);
}

// ---------------------------------------------------------------------------
// (b) JKL シャトル(統合面: Message は pane では no-op = shell 先取りの型)
// ---------------------------------------------------------------------------

/// JKL の意味は状態機械([`ShuttleState`])が正本 — 発注オラクル(b)の写し。
/// 細粒度(上限・方向転換)は `shuttle.rs` の unit tests。
#[test]
fn the_jkl_state_machine_walks_stop_forward_double_reverse_stop() {
    let s = ShuttleState::stopped();
    let s = s.apply(ShuttleCommand::Forward); // L
    assert_eq!(s.rate, 1);
    let s = s.apply(ShuttleCommand::Forward); // L 連打 = 倍速
    assert_eq!(s.rate, 2);
    let s = s.apply(ShuttleCommand::Reverse); // J = 方向転換で等速へ
    assert_eq!(s.rate, -1);
    let s = s.apply(ShuttleCommand::Stop); // K
    assert!(s.is_stopped());
}

/// Shuttle Message は transport 4腕と同じ「shell が先取りする例外」— pane の
/// 書き口では no-op(playhead も Document も触らない)。
#[test]
fn shuttle_messages_are_a_no_op_in_the_pane_write_path() {
    let mut doc = doc_with_comp();
    let mut session = Session::default();
    session.playhead = 7;
    let mut pane = PaneState::new();
    for command in [ShuttleCommand::Reverse, ShuttleCommand::Stop, ShuttleCommand::Forward] {
        let reason = update(&mut pane, &mut doc, &mut session, Message::Shuttle(command));
        assert!(reason.is_none());
    }
    assert_eq!(session.playhead, 7, "pane 側で playhead を動かしてしまっている");
}

// ---------------------------------------------------------------------------
// 拘束5(再生と掴みは相互排他): ループ帯ドラッグも「掴み」に数える
// ---------------------------------------------------------------------------

#[test]
fn a_loop_drag_counts_as_dragging_for_the_playback_exclusion() {
    let mut doc = doc_with_comp();
    let mut session = Session::default();
    let mut pane = PaneState::new();
    assert!(!pane.is_dragging());
    update(&mut pane, &mut doc, &mut session, Message::LoopBandGrabbed { part: LoopBandPart::Blank, at_frame: 10 });
    assert!(pane.is_dragging(), "ループ帯の掴み中に Space が効いてしまう(拘束5)");
    update(&mut pane, &mut doc, &mut session, Message::LoopDragReleased);
    assert!(!pane.is_dragging());
}
