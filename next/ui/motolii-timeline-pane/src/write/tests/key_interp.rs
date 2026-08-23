//! 選択キーの補間切替(第3切片 B15、map 495/512/513/514・485〜490)。
//! **未実行**(裁定189 — supervisor が波末一括)。SP-2 分割で元は
//! `key_interp_tests`(`write.rs` 内の兄弟モジュール)だった物をそのまま
//! 移設(中身は無改変)。

use super::fixtures::*;
use crate::write::*;

/// **オラクル(赤→緑)**: 選択された2キーだけ Hold になり、非選択キーは
/// Linear のまま。時刻は不変。確定は1回の `apply_all` = **1 undo**
/// (undo 1回で全キー Linear へ戻り、floor に着く)。
#[test]
fn set_key_interp_changes_only_selected_keys_in_one_undo() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    let property = opacity();
    place(&mut doc, layer, 0);
    set_track(&mut doc, layer, &property, &[0, 100, 250]);
    doc.mark_undo_floor();

    let mut session = Session::default();
    session.selection = Some(layer);
    session.selected_keys = vec![selector(layer, &property, 0), selector(layer, &property, 100)];
    let mut pane = PaneState::new();

    let reason = pane.update(Message::SetKeyInterp(Interp::Hold), &mut doc, &mut session, no_mods());
    assert!(reason.is_none(), "正常系で拒否理由が返った: {reason:?}");
    assert_eq!(
        interps_of(&doc, layer, &property),
        vec![(0, Interp::Hold), (100, Interp::Hold), (250, Interp::Linear)],
        "選択キーだけが Hold になっていない(時刻も不変のはず)"
    );
    // 選択は生きたまま(frame が動かないので selector はそのまま有効)。
    assert_eq!(session.selected_keys.len(), 2);

    assert!(doc.undo(), "1回目の undo が効かない");
    assert_eq!(
        interps_of(&doc, layer, &property),
        vec![(0, Interp::Linear), (100, Interp::Linear), (250, Interp::Linear)],
        "undo 1回で確定前へ戻っていない(1操作=1undo 違反)"
    );
    assert!(!doc.can_undo(), "余分な undo 段がある(1操作=1undo 違反)");
}

/// 空選択は理由つき拒否(M13)— Document は無傷。
#[test]
fn set_key_interp_with_no_selection_refuses_with_a_reason() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place(&mut doc, layer, 0);
    doc.mark_undo_floor();
    let mut session = Session::default();
    let mut pane = PaneState::new();

    let reason = pane.update(Message::SetKeyInterp(Interp::Hold), &mut doc, &mut session, no_mods());
    assert!(reason.is_some(), "空選択が黙って飲み込まれた(M13 違反)");
    assert!(!doc.can_undo(), "拒否したのに Document が動いた");
}

/// ロック層は理由つき拒否 — track は不変。
#[test]
fn set_key_interp_refuses_a_locked_layer() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    let property = opacity();
    place(&mut doc, layer, 0);
    set_track(&mut doc, layer, &property, &[0, 100]);
    lock(&mut doc, layer);
    doc.mark_undo_floor();

    let mut session = Session::default();
    session.selection = Some(layer);
    session.selected_keys = vec![selector(layer, &property, 0)];
    let mut pane = PaneState::new();

    let reason = pane.update(Message::SetKeyInterp(Interp::Hold), &mut doc, &mut session, no_mods());
    assert!(reason.is_some(), "ロック層への補間変更が黙って通った");
    assert_eq!(
        interps_of(&doc, layer, &property),
        vec![(0, Interp::Linear), (100, Interp::Linear)],
        "拒否したのに track が動いた"
    );
}

/// Easy Ease プリセット(map 485〜490)は妥当な Bezier(x1/x2 ∈ [0,1] —
/// `TrackError::InvalidBezier` の柵の内側)で、store を実際に往復できる。
#[test]
fn easy_ease_presets_are_valid_beziers_that_round_trip_through_the_store() {
    for preset in [EASY_EASE, EASY_EASE_IN, EASY_EASE_OUT] {
        let Interp::Bezier { x1, x2, .. } = preset else {
            panic!("Easy Ease プリセットが Bezier でない: {preset:?}");
        };
        assert!((0.0..=1.0).contains(&x1) && (0.0..=1.0).contains(&x2));
    }

    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    let property = opacity();
    place(&mut doc, layer, 0);
    set_track(&mut doc, layer, &property, &[0, 100]);
    let mut session = Session::default();
    session.selection = Some(layer);
    session.selected_keys = vec![selector(layer, &property, 0)];
    let mut pane = PaneState::new();

    let reason = pane.update(Message::SetKeyInterp(EASY_EASE), &mut doc, &mut session, no_mods());
    assert!(reason.is_none(), "Easy Ease の適用が拒否された: {reason:?}");
    assert_eq!(interps_of(&doc, layer, &property)[0], (0, EASY_EASE));
}
