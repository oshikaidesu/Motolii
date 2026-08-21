//! 運転席 — Timeline 第2波 T4(キーの時刻編集: ドラッグ・nudge・リタイム、正典
//! `next/reference/timeline-grammar.md` §3・§8.1・裁定146)の落ちるテスト先行
//! (発注書 ORACLE の a〜f)。`Shell::update` だけを叩く — `tests/suite/drive.rs`
//! の T2 clip drag 試験(`dragging_a_clip_body_moves_it_and_undoes_in_one_step`
//! 節)と同じ形。
//!
//! fixture(`Shell::new_fixture()`)の2つの track を使い分ける:
//! - **サビ歌詞**(既定選択、position に2キー 510/570)— 単一キーの move/Esc
//! - **タイトルロゴ**(`Message::Select` で選び直す、opacity に4キー
//!   0/20/70/90、clip 範囲 [0,90])— 複数選択の相対間隔維持・nudge・retime・
//!   同時刻衝突(ヘッドルームが要るので複数キーの試験はこちらに寄せてある)

use motolii_shell::timeline_pane::KeySelector;
use motolii_shell::{Message, Shell};
use motolii_store::{property, LayerId, PropertyId};

/// タイトルロゴ(fixture.rs: `LAYERS` の1番目 → `LayerId(1)`)を選択し直した
/// Shell。opacity に4キー(0/20/70/90、clip [0,90])を持つ — 複数選択の試験に
/// 要るヘッドルームがサビ歌詞(2キーだけ)より広い。
fn shell_with_logo_selected() -> Shell {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::Select(LayerId(1)));
    assert_eq!(shell.session().selection, Some(LayerId(1)), "タイトルロゴを選べていない");
    shell
}

fn opacity_frames(shell: &Shell) -> Vec<i64> {
    let rows = shell.timeline_property_rows();
    rows.iter()
        .find(|row| row.property.name() == property::OPACITY)
        .map(|row| row.keys.iter().map(|k| k.frame).collect())
        .unwrap_or_default()
}

fn position_frames(shell: &Shell) -> Vec<i64> {
    let rows = shell.timeline_property_rows();
    rows.iter()
        .find(|row| row.property.name() == property::POSITION)
        .map(|row| row.keys.iter().map(|k| k.frame).collect())
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// (a) 単一キーのドラッグ確定 → 時刻が動き Undo 1回
// ---------------------------------------------------------------------------

#[test]
fn a_dragging_a_single_key_moves_it_and_undoes_in_one_step() {
    let mut shell = Shell::new_fixture().0;
    let layer = shell.session().selection.expect("fixture は既定選択を持つ");
    let property = PropertyId::new(property::POSITION).expect("position は予約語ではない");
    assert_eq!(position_frames(&shell), vec![510, 570]);

    let key = KeySelector { layer, property, frame: 510 };
    let _ = shell.update(Message::TimelineKeyGrabbed { key, at_frame: 510, retime: false });
    let _ = shell.update(Message::TimelineKeyDragMoved { at_frame: 520, px_per_frame: 1.0 });
    let _ = shell.update(Message::TimelineKeyDragReleased);

    assert_eq!(
        position_frames(&shell),
        vec![520, 570],
        "本体ドラッグで510のキーが520へ動いていない(570は無関係のまま残るはず)"
    );
    assert_eq!(shell.status(), None, "動いただけなのに拒否理由が出ている");

    assert!(shell.can_undo());
    let _ = shell.update(Message::Undo);
    assert_eq!(position_frames(&shell), vec![510, 570], "Undo 1回で戻らない(1操作=1undo)");
}

// ---------------------------------------------------------------------------
// (b) ドラッグ中の Esc → 履歴完全無傷
// ---------------------------------------------------------------------------

#[test]
fn b_escape_during_a_key_drag_leaves_history_completely_untouched() {
    let mut shell = Shell::new_fixture().0;
    let layer = shell.session().selection.expect("fixture は既定選択を持つ");
    let property = PropertyId::new(property::POSITION).expect("position は予約語ではない");
    let can_undo_before = shell.can_undo();
    let can_redo_before = shell.can_redo();

    let key = KeySelector { layer, property, frame: 510 };
    let _ = shell.update(Message::TimelineKeyGrabbed { key, at_frame: 510, retime: false });
    let _ = shell.update(Message::TimelineKeyDragMoved { at_frame: 520, px_per_frame: 1.0 });
    let _ = shell.update(Message::EscapePressed);

    assert_eq!(shell.can_undo(), can_undo_before, "Esc がドラッグ以外の履歴まで動かしている");
    assert_eq!(shell.can_redo(), can_redo_before, "Esc が redo 空間を汚している");
    assert_eq!(position_frames(&shell), vec![510, 570], "Esc なのにキーが動いてしまっている");

    // 掴んだままのボタンで release が来ても、Esc で drag state は既に空 —
    // 何も起きない(二重確定の防止、clip drag の同名試験と同じ確認)。
    let _ = shell.update(Message::TimelineKeyDragReleased);
    assert_eq!(position_frames(&shell), vec![510, 570]);
    assert_eq!(shell.can_undo(), can_undo_before);
}

// ---------------------------------------------------------------------------
// (c) 複数選択の相対間隔維持
// ---------------------------------------------------------------------------

#[test]
fn c_dragging_multiple_selected_keys_preserves_their_relative_spacing() {
    let mut shell = shell_with_logo_selected();
    let layer = LayerId(1);
    let property = PropertyId::new(property::OPACITY).expect("opacity は予約語ではない");
    assert_eq!(opacity_frames(&shell), vec![0, 20, 70, 90]);

    let key0 = KeySelector { layer, property: property.clone(), frame: 0 };
    let key70 = KeySelector { layer, property: property.clone(), frame: 70 };
    let _ = shell.update(Message::TimelineKeySelect(
        motolii_shell::timeline_pane::KeySelectionOp::Single(key0.clone()),
    ));
    let _ = shell.update(Message::TimelineKeySelect(
        motolii_shell::timeline_pane::KeySelectionOp::Range(key70),
    ));

    // 掴むのは選択の一員(0) — delta=10(SNAP_PX=7pxより外へ逃がして候補への
    // 誤吸着を避ける、0/60/90が候補に載っているため)。
    let _ = shell.update(Message::TimelineKeyGrabbed { key: key0, at_frame: 0, retime: false });
    let _ = shell.update(Message::TimelineKeyDragMoved { at_frame: 10, px_per_frame: 1.0 });
    let _ = shell.update(Message::TimelineKeyDragReleased);

    assert_eq!(
        opacity_frames(&shell),
        vec![10, 30, 80, 90],
        "選択した0/20/70が同じ delta で動いていない(相対間隔=20/50 が崩れている、90は選択外なので不変のはず)"
    );
}

// ---------------------------------------------------------------------------
// (d) nudge 1/10 フレーム
// ---------------------------------------------------------------------------

#[test]
fn d_nudge_keyframe_moves_the_selected_key_by_one_frame() {
    let mut shell = shell_with_logo_selected();
    let layer = LayerId(1);
    let property = PropertyId::new(property::OPACITY).expect("opacity は予約語ではない");
    let key20 = KeySelector { layer, property, frame: 20 };

    let _ = shell.update(Message::TimelineKeySelect(
        motolii_shell::timeline_pane::KeySelectionOp::Single(key20),
    ));
    let _ = shell.update(Message::NudgeKeyframe(1));

    assert_eq!(opacity_frames(&shell), vec![0, 21, 70, 90], "Alt+→ 相当の1フレーム nudge が効いていない");
    assert!(shell.can_undo());
    let _ = shell.update(Message::Undo);
    assert_eq!(opacity_frames(&shell), vec![0, 20, 70, 90], "Undo 1回で戻らない");
}

#[test]
fn d_nudge_keyframe_moves_the_selected_key_by_ten_frames() {
    let mut shell = shell_with_logo_selected();
    let layer = LayerId(1);
    let property = PropertyId::new(property::OPACITY).expect("opacity は予約語ではない");
    let key70 = KeySelector { layer, property, frame: 70 };

    let _ = shell.update(Message::TimelineKeySelect(
        motolii_shell::timeline_pane::KeySelectionOp::Single(key70),
    ));
    let _ = shell.update(Message::NudgeKeyframe(-10));

    assert_eq!(opacity_frames(&shell), vec![0, 20, 60, 90], "Alt+Shift+← 相当の10フレーム nudge が効いていない");
}

// ---------------------------------------------------------------------------
// (e) リタイム: 3キー選択で端 Cmd ドラッグ → 中間キーが比例位置
// ---------------------------------------------------------------------------

/// 選択={20,70,90}(0は選択外)。端(90)を Cmd+ドラッグで anchor(20)側へ
/// 縮める — target=50 なので scale=(50-20)/(90-20)=30/70。中間キー(70)は
/// `20 + (70-20)*30/70 = 20 + 21.43 → round 21 = 41` へ移るはず(数値例は
/// RETURN 報告に転記)。
#[test]
fn e_retiming_a_selection_scales_the_middle_key_proportionally() {
    let mut shell = shell_with_logo_selected();
    let layer = LayerId(1);
    let property = PropertyId::new(property::OPACITY).expect("opacity は予約語ではない");
    let key20 = KeySelector { layer, property: property.clone(), frame: 20 };
    let key90 = KeySelector { layer, property: property.clone(), frame: 90 };

    let _ = shell.update(Message::TimelineKeySelect(
        motolii_shell::timeline_pane::KeySelectionOp::Single(key20),
    ));
    let _ = shell.update(Message::TimelineKeySelect(
        motolii_shell::timeline_pane::KeySelectionOp::Range(key90.clone()),
    ));

    let _ = shell.update(Message::TimelineKeyGrabbed { key: key90, at_frame: 90, retime: true });
    let _ = shell.update(Message::TimelineKeyDragMoved { at_frame: 50, px_per_frame: 1.0 });
    let _ = shell.update(Message::TimelineKeyDragReleased);

    assert_eq!(
        opacity_frames(&shell),
        vec![0, 20, 41, 50],
        "retime後の並び(anchor=20不動・edge=90→50・中間70→比例位置41・0は選択外で不変)が期待と違う"
    );

    assert!(shell.can_undo());
    let _ = shell.update(Message::Undo);
    assert_eq!(opacity_frames(&shell), vec![0, 20, 70, 90], "retime の Undo 1回で戻らない");
}

/// Cmd+drag が選択の端ではないキー、または選択1本だけでは retime が成立しない
/// (`Shell::start_timeline_key_drag` の二重の柵) — grab 自体が no-op になり、
/// 何も動かない。
#[test]
fn retime_does_not_start_when_grabbing_a_non_edge_key() {
    let mut shell = shell_with_logo_selected();
    let layer = LayerId(1);
    let property = PropertyId::new(property::OPACITY).expect("opacity は予約語ではない");
    let key20 = KeySelector { layer, property: property.clone(), frame: 20 };
    let key70 = KeySelector { layer, property: property.clone(), frame: 70 };
    let key90 = KeySelector { layer, property: property.clone(), frame: 90 };

    let _ = shell.update(Message::TimelineKeySelect(
        motolii_shell::timeline_pane::KeySelectionOp::Single(key20),
    ));
    let _ = shell.update(Message::TimelineKeySelect(
        motolii_shell::timeline_pane::KeySelectionOp::Range(key90),
    ));
    // 中間(70、端ではない)を retime:true で掴もうとしても不成立。
    let _ = shell.update(Message::TimelineKeyGrabbed { key: key70, at_frame: 70, retime: true });
    let _ = shell.update(Message::TimelineKeyDragMoved { at_frame: 50, px_per_frame: 1.0 });
    let _ = shell.update(Message::TimelineKeyDragReleased);

    assert_eq!(opacity_frames(&shell), vec![0, 20, 70, 90], "端でないキーの Cmd 掴みで retime が始まってしまっている");
}

// ---------------------------------------------------------------------------
// (f) 同時刻衝突: 2キーを同フレームへ向けて動かしても panic せず順序が保たれる
// ---------------------------------------------------------------------------

/// 選択={20,70}。端(70)を Cmd+ドラッグで anchor(20)ちょうどへ retime すると
/// scale=0 になり、**両キーとも frame 20 へ収束する**(anchor 自身の origin と
/// edge の着地が完全に一致する、最も単純な衝突の作り方)。panic せず、
/// track は1本に統合された上で undo で完全に戻ることを見る。
#[test]
fn f_retiming_two_keys_onto_the_same_frame_does_not_panic_and_collapses_deterministically() {
    let mut shell = shell_with_logo_selected();
    let layer = LayerId(1);
    let property = PropertyId::new(property::OPACITY).expect("opacity は予約語ではない");
    let key20 = KeySelector { layer, property: property.clone(), frame: 20 };
    let key70 = KeySelector { layer, property: property.clone(), frame: 70 };

    let _ = shell.update(Message::TimelineKeySelect(
        motolii_shell::timeline_pane::KeySelectionOp::Single(key20),
    ));
    let _ = shell.update(Message::TimelineKeySelect(
        motolii_shell::timeline_pane::KeySelectionOp::Range(key70.clone()),
    ));

    let _ = shell.update(Message::TimelineKeyGrabbed { key: key70, at_frame: 70, retime: true });
    // anchor(20)ちょうどへ — scale=0、選択2本とも frame 20 へ収束する。
    let _ = shell.update(Message::TimelineKeyDragMoved { at_frame: 20, px_per_frame: 1.0 });
    let _ = shell.update(Message::TimelineKeyDragReleased); // ここで panic しないことが本題。

    assert_eq!(
        opacity_frames(&shell),
        vec![0, 20, 90],
        "衝突した2本(20/70)が1本(20)に統合されていない、または0/90(選択外)が巻き込まれている"
    );

    assert!(shell.can_undo(), "衝突後の書き込みも1操作としてundoできるはず");
    let _ = shell.update(Message::Undo);
    assert_eq!(opacity_frames(&shell), vec![0, 20, 70, 90], "Undo 1回で衝突前の4本へ戻らない");
}

// ---------------------------------------------------------------------------
// 右クリック = Esc と同じ意味(裁定151「キャンセルの一般化」、clip drag と同型)
// ---------------------------------------------------------------------------

#[test]
fn right_click_during_a_key_drag_cancels_it_just_like_escape() {
    let mut shell = Shell::new_fixture().0;
    let layer = shell.session().selection.expect("fixture は既定選択を持つ");
    let property = PropertyId::new(property::POSITION).expect("position は予約語ではない");
    let can_undo_before = shell.can_undo();

    let key = KeySelector { layer, property, frame: 510 };
    let _ = shell.update(Message::TimelineKeyGrabbed { key, at_frame: 510, retime: false });
    let _ = shell.update(Message::TimelineKeyDragMoved { at_frame: 520, px_per_frame: 1.0 });
    let _ = shell.update(Message::TimelineKeyDragCancelled);

    assert_eq!(shell.can_undo(), can_undo_before);
    assert_eq!(position_frames(&shell), vec![510, 570]);
}

/// 掴んだだけで未移動なら release は no-op(正典 §2 と同じ判断をキーへ延長)。
#[test]
fn grabbing_a_key_without_moving_and_releasing_writes_nothing() {
    let mut shell = Shell::new_fixture().0;
    let layer = shell.session().selection.expect("fixture は既定選択を持つ");
    let property = PropertyId::new(property::POSITION).expect("position は予約語ではない");
    let can_undo_before = shell.can_undo();

    let key = KeySelector { layer, property, frame: 510 };
    let _ = shell.update(Message::TimelineKeyGrabbed { key, at_frame: 510, retime: false });
    let _ = shell.update(Message::TimelineKeyDragReleased);

    assert_eq!(position_frames(&shell), vec![510, 570]);
    assert_eq!(shell.can_undo(), can_undo_before, "動いていないのに undo できる操作が積まれている");
}

/// ロック中の layer はキーも掴めない(clip drag の同名試験・M13 と同じ判断)。
#[test]
fn grabbing_a_key_on_a_locked_layer_is_refused_with_a_reason() {
    let mut shell = Shell::new_fixture().0;
    let layer = shell.session().selection.expect("fixture は既定選択を持つ");
    let property = PropertyId::new(property::POSITION).expect("position は予約語ではない");
    let _ = shell.update(Message::LaneBarToggleLock(layer));

    let key = KeySelector { layer, property, frame: 510 };
    let _ = shell.update(Message::TimelineKeyGrabbed { key, at_frame: 510, retime: false });
    assert!(shell.status().is_some(), "ロック中のキー掴みが理由つきで拒否されていない(M13違反)");

    let _ = shell.update(Message::TimelineKeyDragMoved { at_frame: 520, px_per_frame: 1.0 });
    let _ = shell.update(Message::TimelineKeyDragReleased);
    assert_eq!(position_frames(&shell), vec![510, 570], "ロック中なのに動いてしまっている");
}
