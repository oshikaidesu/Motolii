//! Time-Reverse Keyframes(map 518、`keys2::reversed_key_group` の結線)。
//! SP-2 分割で元は `reverse_selected_keys_tests`(`write.rs` 内の兄弟
//! モジュール)だった物をそのまま移設(中身は無改変)。

use super::fixtures::*;
use crate::write::*;

fn value_at(doc: &Document, layer: LayerId, property: &PropertyId, frame: i64) -> f64 {
    let fps = doc.view().composition().unwrap().unwrap().fps;
    let track = doc.view().track(layer, property).unwrap().unwrap();
    let key = track
        .keys()
        .iter()
        .find(|k| k.t.try_to_frame_round(fps) == Ok(frame))
        .unwrap_or_else(|| panic!("frame={frame} にキーが無い"));
    match key.value {
        motolii_store::Value::F64(v) => v,
        _ => panic!("F64 以外の value"),
    }
}

fn set_track_with_values(doc: &mut Document, layer: LayerId, property: &PropertyId, points: &[(i64, f64)]) {
    let mut track = KeyframeTrack::new();
    for &(frame, value) in points {
        track.insert(motolii_store::Keyframe {
            t: RationalTime::try_new(frame, 30).expect("frame は収まる"),
            value: motolii_store::Value::F64(value),
            interp: Interp::Linear,
            spatial: None,
        });
    }
    doc.apply(Intent::SetTrack { layer, property: property.clone(), track }).expect("track を書ける");
}

/// **オラクル(赤→緑)**: 選択キー集合が自分自身の `[min,max]` の中で鏡映
/// する——値はキーに付いたまま frame だけ入れ替わる。確定は1回の
/// `apply_all` = **1 undo**。
#[test]
fn reverses_the_selected_keys_around_their_own_span_in_one_undo() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    let property = opacity();
    place(&mut doc, layer, 0);
    set_track_with_values(&mut doc, layer, &property, &[(0, 0.0), (30, 0.5), (100, 1.0)]);
    doc.mark_undo_floor();

    let mut session = Session::default();
    session.selection = Some(layer);
    session.selected_keys =
        vec![selector(layer, &property, 0), selector(layer, &property, 30), selector(layer, &property, 100)];
    let mut pane = PaneState::new();

    let reason = pane.update(Message::ReverseSelectedKeys, &mut doc, &mut session, no_mods());
    assert!(reason.is_none(), "正常系で拒否理由が返った: {reason:?}");
    // min=0, max=100 の鏡映: 0↔100 が入れ替わり、30 は70(0+100-30)へ動く。
    assert_eq!(value_at(&doc, layer, &property, 100), 0.0, "元frame=0の値がframe=100へ");
    assert_eq!(value_at(&doc, layer, &property, 0), 1.0, "元frame=100の値がframe=0へ");
    assert_eq!(value_at(&doc, layer, &property, 70), 0.5, "中間キーは鏡映位置(70)へ");

    assert!(doc.undo(), "1回目の undo が効かない");
    assert_eq!(value_at(&doc, layer, &property, 0), 0.0, "undo 1回で確定前へ戻っていない(1操作=1undo 違反)");
    assert!(!doc.can_undo(), "余分な undo 段がある(1操作=1undo 違反)");
}

/// 単独キー1本は鏡映の中心が自分自身になるので no-op(frame は動かない)
/// ——それでも `commit_key_frames` は「動いていない」と見て intent を
/// 出さない(1操作の粒度は保つが、undo 段は増えない)。
#[test]
fn a_single_selected_key_is_left_untouched() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    let property = opacity();
    place(&mut doc, layer, 0);
    set_track_with_values(&mut doc, layer, &property, &[(42, 1.0)]);
    doc.mark_undo_floor();

    let mut session = Session::default();
    session.selection = Some(layer);
    session.selected_keys = vec![selector(layer, &property, 42)];
    let mut pane = PaneState::new();

    let reason = pane.update(Message::ReverseSelectedKeys, &mut doc, &mut session, no_mods());
    assert!(reason.is_none());
    assert_eq!(value_at(&doc, layer, &property, 42), 1.0, "単独キーは動かないはず");
}

/// 空選択は理由つき拒否(M13)— Document は無傷。
#[test]
fn reverse_selected_keys_with_no_selection_refuses_with_a_reason() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place(&mut doc, layer, 0);
    doc.mark_undo_floor();
    let mut session = Session::default();
    let mut pane = PaneState::new();

    let reason = pane.update(Message::ReverseSelectedKeys, &mut doc, &mut session, no_mods());
    assert!(reason.is_some(), "空選択が黙って飲み込まれた(M13 違反)");
    assert!(!doc.can_undo(), "拒否したのに Document が動いた");
}

/// ロック層は理由つき拒否 — track は不変。
#[test]
fn reverse_selected_keys_refuses_a_locked_layer() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    let property = opacity();
    place(&mut doc, layer, 0);
    set_track_with_values(&mut doc, layer, &property, &[(0, 0.0), (100, 1.0)]);
    lock(&mut doc, layer);
    doc.mark_undo_floor();

    let mut session = Session::default();
    session.selection = Some(layer);
    session.selected_keys = vec![selector(layer, &property, 0), selector(layer, &property, 100)];
    let mut pane = PaneState::new();

    let reason = pane.update(Message::ReverseSelectedKeys, &mut doc, &mut session, no_mods());
    assert!(reason.is_some(), "ロック層への時間反転が黙って通った");
    assert_eq!(value_at(&doc, layer, &property, 0), 0.0, "拒否したのに track が動いた");
}

/// 複数 property をまたぐ選択でも1回の undo で確定する——各 property は
/// それぞれ自分の `[min,max]` で独立に鏡映する。
#[test]
fn spans_multiple_properties_in_a_single_undo() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    let opacity_p = opacity();
    let position_p = position_x();
    place(&mut doc, layer, 0);
    set_track_with_values(&mut doc, layer, &opacity_p, &[(0, 0.0), (100, 1.0)]);
    set_track_with_values(&mut doc, layer, &position_p, &[(10, 5.0), (50, 25.0)]);
    doc.mark_undo_floor();

    let mut session = Session::default();
    session.selection = Some(layer);
    session.selected_keys = vec![
        selector(layer, &opacity_p, 0),
        selector(layer, &opacity_p, 100),
        selector(layer, &position_p, 10),
        selector(layer, &position_p, 50),
    ];
    let mut pane = PaneState::new();

    let reason = pane.update(Message::ReverseSelectedKeys, &mut doc, &mut session, no_mods());
    assert!(reason.is_none(), "{reason:?}");
    assert_eq!(value_at(&doc, layer, &opacity_p, 100), 0.0);
    assert_eq!(value_at(&doc, layer, &opacity_p, 0), 1.0);
    assert_eq!(value_at(&doc, layer, &position_p, 50), 5.0);
    assert_eq!(value_at(&doc, layer, &position_p, 10), 25.0);

    assert!(doc.undo(), "1回目の undo が効かない");
    assert_eq!(value_at(&doc, layer, &opacity_p, 0), 0.0, "undo 1回で両property戻っていない");
    assert_eq!(value_at(&doc, layer, &position_p, 10), 5.0, "undo 1回で両property戻っていない");
    assert!(!doc.can_undo(), "余分な undo 段がある(1操作=1undo 違反、複数property跨ぎ)");
}
