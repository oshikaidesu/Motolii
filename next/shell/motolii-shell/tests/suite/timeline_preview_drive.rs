//! 運転席 — Timeline 第2波 T5(ドラッグ中のライブプレビュー描画、正典
//! `next/reference/timeline-grammar.md` §5.5「プレビューは毎フレーム」)の
//! 落ちるテスト先行。
//!
//! T4 までの finding: clip drag(`Shell::TimelineDragState`)も key drag
//! (`Shell::TimelineKeyDragState`)も **preview 値が canvas に届いておらず、
//! release まで絵が動かない**(§5.5 に反する無反応)。T5 はこれを
//! `Shell::build_timeline_pane()`(`view()` が実際に呼ぶのと同じ組み立て関数)
//! + `TimelinePane::rows()`/`property_rows()`(preview 適用後の投影を読む口)
//! で根治する。
//!
//! ここで見るのは「ドラッグ中に pane の投影が動いているか」だけ —
//! move/trim/retime の**計算そのもの**が正しいかは `drive.rs`
//! (T2 clip drag)・`timeline_key_gesture_drive.rs`(T4 key drag)が既に見ている
//! ので再確認しない(同じ数値例を再利用するだけ)。

use motolii_shell::timeline_pane::{BarPart, KeySelector};
use motolii_shell::{Message, Shell};
use motolii_store::{property, LayerId, PropertyId};

// ---------------------------------------------------------------------------
// クリップ(bar)の drag preview
// ---------------------------------------------------------------------------

/// `drive.rs::dragging_a_clip_body_moves_it_and_undoes_in_one_step` と同じ
/// セットアップ(comp 300frame、playhead=50 で AddLayer → `[50, 300)`)。
#[test]
fn clip_drag_preview_reaches_the_pane_before_release() {
    let mut shell = Shell::new().0;
    shell.update(Message::ScrubTo(50));
    shell.update(Message::AddLayer);
    let id = shell.timeline_rows()[0].id;
    assert_eq!(shell.timeline_rows()[0].start, 50);

    // 掴む前: pane の投影は Document の投影とまだ一致し、誰も dragging でない。
    let baseline = shell.build_timeline_pane();
    assert_eq!(baseline.rows()[0].start, 50);
    assert!(!baseline.rows()[0].dragging);

    shell.update(Message::TimelineBarGrabbed {
        layer: id,
        part: BarPart::Body,
        at_frame: 50,
    });
    shell.update(Message::TimelineDragMoved {
        at_frame: 30,
        px_per_frame: 1.0,
    });

    // **赤→緑の核心**: release 前、Document(`timeline_rows`)はまだ 50 のまま
    // だが、pane の投影(`build_timeline_pane().rows()`)はもう 30 を見せている。
    assert_eq!(shell.timeline_rows()[0].start, 50, "Document が release 前に動いてしまっている");
    let mid_drag = shell.build_timeline_pane();
    assert_eq!(
        mid_drag.rows()[0].start,
        30,
        "ドラッグ中の pane の投影に preview 値が乗っていない(§5.5 違反)"
    );
    assert!(
        mid_drag.rows()[0].dragging,
        "ドラッグ中の行に dragging フラグが立っていない(ACCENT 強調の出典)"
    );

    shell.update(Message::TimelineDragReleased);

    // release 後: Document と pane の投影が再び一致し、dragging は消える。
    assert_eq!(shell.timeline_rows()[0].start, 30);
    let after_release = shell.build_timeline_pane();
    assert_eq!(after_release.rows()[0].start, 30);
    assert!(!after_release.rows()[0].dragging, "release 後も dragging が残っている");
}

/// 右クリック cancel は Document 不接触のまま — pane の投影も掴む前の値へ戻る
/// (`TimelineDragState` doc comment の「捨てるだけで無傷」を pane 側からも見る)。
#[test]
fn clip_drag_preview_reverts_on_cancel() {
    let mut shell = Shell::new().0;
    shell.update(Message::ScrubTo(50));
    shell.update(Message::AddLayer);
    let id = shell.timeline_rows()[0].id;

    shell.update(Message::TimelineBarGrabbed {
        layer: id,
        part: BarPart::Body,
        at_frame: 50,
    });
    shell.update(Message::TimelineDragMoved {
        at_frame: 30,
        px_per_frame: 1.0,
    });
    assert_eq!(shell.build_timeline_pane().rows()[0].start, 30);

    shell.update(Message::TimelineDragCancelled);

    let after_cancel = shell.build_timeline_pane();
    assert_eq!(after_cancel.rows()[0].start, 50, "cancel 後の pane の投影が掴む前の値へ戻っていない");
    assert!(!after_cancel.rows()[0].dragging);
    assert_eq!(shell.timeline_rows()[0].start, 50, "cancel なのに Document が動いている");
}

// ---------------------------------------------------------------------------
// キー(菱形)の drag preview
// ---------------------------------------------------------------------------

/// fixture 既定選択(サビ歌詞、position に2キー 510/570)。
/// `timeline_key_gesture_drive.rs::a_dragging_a_single_key_moves_it_and_undoes_in_one_step`
/// と同じ数値。
#[test]
fn key_drag_preview_reaches_the_pane_before_release() {
    let mut shell = Shell::new_fixture().0;
    let layer = shell.session().selection.expect("fixture は既定選択を持つ");
    let property = PropertyId::new(property::POSITION).expect("position は予約語ではない");
    let key = KeySelector { layer, property, frame: 510 };

    shell.update(Message::TimelineKeyGrabbed { key: key.clone(), at_frame: 510, retime: false });
    shell.update(Message::TimelineKeyDragMoved { at_frame: 520, px_per_frame: 1.0 });

    // Document はまだ 510 のまま、pane の投影はもう 520 を見せている。
    let doc_frames: Vec<i64> = shell
        .timeline_property_rows()
        .into_iter()
        .find(|row| row.property.name() == property::POSITION)
        .map(|row| row.keys.into_iter().map(|k| k.frame).collect())
        .unwrap_or_default();
    assert_eq!(doc_frames, vec![510, 570], "Document が release 前に動いてしまっている");

    let mid_drag = shell.build_timeline_pane();
    let pane_frames: Vec<i64> = mid_drag
        .property_rows()
        .iter()
        .find(|row| row.property.name() == property::POSITION)
        .map(|row| row.keys.iter().map(|k| k.frame).collect())
        .unwrap_or_default();
    assert_eq!(
        pane_frames,
        vec![520, 570],
        "ドラッグ中の pane の投影にキー preview 値が乗っていない(§5.5 違反)"
    );

    shell.update(Message::TimelineKeyDragReleased);

    let after_release = shell.build_timeline_pane();
    let pane_frames_after: Vec<i64> = after_release
        .property_rows()
        .iter()
        .find(|row| row.property.name() == property::POSITION)
        .map(|row| row.keys.iter().map(|k| k.frame).collect())
        .unwrap_or_default();
    assert_eq!(pane_frames_after, vec![520, 570]);
}

/// リタイム(裁定146): 選択キー全部が比例位置でプレビューされる(EXACT TARGET 4)
/// — `timeline_key_gesture_drive.rs::e_retiming_a_selection_scales_the_middle_key_proportionally`
/// と同じ数値例(anchor=20 不動・edge=90→50・中間70→比例位置41)。
#[test]
fn key_retime_preview_moves_every_selected_key_in_the_pane() {
    let mut shell = Shell::new_fixture().0;
    shell.update(Message::Select(LayerId(1)));
    let layer = LayerId(1);
    let property = PropertyId::new(property::OPACITY).expect("opacity は予約語ではない");
    let key20 = KeySelector { layer, property: property.clone(), frame: 20 };
    let key90 = KeySelector { layer, property: property.clone(), frame: 90 };

    shell.update(Message::TimelineKeySelect(
        motolii_shell::timeline_pane::KeySelectionOp::Single(key20),
    ));
    shell.update(Message::TimelineKeySelect(
        motolii_shell::timeline_pane::KeySelectionOp::Range(key90.clone()),
    ));

    shell.update(Message::TimelineKeyGrabbed { key: key90, at_frame: 90, retime: true });
    shell.update(Message::TimelineKeyDragMoved { at_frame: 50, px_per_frame: 1.0 });

    let mid_drag = shell.build_timeline_pane();
    let pane_frames: Vec<i64> = mid_drag
        .property_rows()
        .iter()
        .find(|row| row.property.name() == property::OPACITY)
        .map(|row| row.keys.iter().map(|k| k.frame).collect())
        .unwrap_or_default();
    assert_eq!(
        pane_frames,
        vec![0, 20, 41, 50],
        "retime 中、選択キー全部(中間キー含む)が比例位置で pane の投影に乗っていない"
    );

    shell.update(Message::TimelineKeyDragReleased);
    assert!(shell.can_undo());
}

/// 非ドラッグ中は `build_timeline_pane()` が `Shell::timeline_rows()`/
/// `timeline_property_rows()` と一致する(preview 経路が通常描画を汚さないこと
/// の柵)。
#[test]
fn pane_preview_is_a_passthrough_when_nothing_is_dragging() {
    let shell = Shell::new_fixture().0;
    let pane = shell.build_timeline_pane();

    assert_eq!(pane.rows().to_vec(), shell.timeline_rows());
    assert_eq!(pane.property_rows().to_vec(), shell.timeline_property_rows());
    assert!(pane.rows().iter().all(|row| !row.dragging));
}
