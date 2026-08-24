//! F 段階の transport frame input / property key navigation。

use super::fixtures::*;
use crate::write::*;

#[test]
fn committing_a_frame_input_moves_the_playhead() {
    let mut doc = doc_with_comp();
    let mut session = Session::default();
    session.playhead = 3;
    let mut pane = PaneState::new();

    pane.update(Message::FrameInput("42".into()), &mut doc, &mut session, no_mods());
    assert_eq!(session.playhead, 3, "入力途中で playhead を動かしている");
    assert_eq!(pane.frame_draft(), Some("42"));

    let reason = pane.update(Message::FrameCommit, &mut doc, &mut session, no_mods());
    assert!(reason.is_none());
    assert_eq!(session.playhead, 42);
    assert_eq!(pane.frame_draft(), None);
}

#[test]
fn invalid_frame_input_keeps_the_draft_and_reports_the_reason() {
    let mut doc = doc_with_comp();
    let mut session = Session::default();
    session.playhead = 3;
    let mut pane = PaneState::new();

    pane.update(Message::FrameInput("nope".into()), &mut doc, &mut session, no_mods());
    let reason = pane.update(Message::FrameCommit, &mut doc, &mut session, no_mods());
    assert_eq!(reason.as_deref(), Some("フレーム番号は整数で入力してください"));
    assert_eq!(session.playhead, 3);
    assert_eq!(pane.frame_draft(), Some("nope"));
}

#[test]
fn jumping_to_a_property_key_moves_only_the_playhead() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    let property = opacity();
    place(&mut doc, layer, 0);
    set_track(&mut doc, layer, &property, &[10, 40, 80]);
    doc.mark_undo_floor();
    let mut session = Session::default();
    session.selection = Some(layer);
    session.playhead = 20;
    let mut pane = PaneState::new();

    let reason = pane.update(Message::JumpToNextKeyframe, &mut doc, &mut session, no_mods());
    assert!(reason.is_none());
    assert_eq!(session.playhead, 40);

    let reason = pane.update(Message::JumpToPreviousKeyframe, &mut doc, &mut session, no_mods());
    assert!(reason.is_none());
    assert_eq!(session.playhead, 10);
    assert!(!doc.can_undo(), "playhead 移動が Document の undo を作っている");
}

#[test]
fn committing_graph_control_inputs_writes_bezier() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    let property = opacity();
    place(&mut doc, layer, 0);
    set_track(&mut doc, layer, &property, &[10, 40]);
    let mut session = Session::default();
    session.selected_keys = vec![selector(layer, &property, 10)];
    let mut pane = PaneState::new();

    pane.update(
        Message::GraphControlInput(crate::graph_editor::GraphControl::X1, "0.2".into()),
        &mut doc,
        &mut session,
        no_mods(),
    );
    pane.update(
        Message::GraphControlInput(crate::graph_editor::GraphControl::Y2, "0.8".into()),
        &mut doc,
        &mut session,
        no_mods(),
    );
    let reason = pane.update(Message::GraphCommit, &mut doc, &mut session, no_mods());
    assert!(reason.is_none());
    assert_eq!(
        interps_of(&doc, layer, &property)[0].1,
        motolii_store::Interp::Bezier {
            x1: 0.2,
            y1: 0.0,
            x2: 0.667,
            y2: 0.8,
        }
    );
    assert_eq!(pane.graph_editor_drafts(), [None, None, None, None]);
}
