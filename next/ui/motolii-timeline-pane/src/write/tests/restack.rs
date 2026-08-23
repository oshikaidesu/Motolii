//! Stage 重なり並べ替え(第3切片、map B44 184/292/293・§8.1)。
//! **未実行**(裁定189)。SP-2 分割で元は `restack_message_tests`
//! (`write.rs` 内の兄弟モジュール)だった物をそのまま移設(中身は無改変)。

use super::fixtures::*;
use crate::write::*;

fn orders(doc: &Document) -> Vec<(LayerId, i16)> {
    let store = doc.view();
    let mut out: Vec<(LayerId, i16)> = store
        .layers()
        .into_iter()
        .filter_map(|id| store.meta(id).ok().flatten().map(|meta| (id, meta.order)))
        .collect();
    out.sort_by_key(|&(id, order)| (order, id));
    out
}

/// **オラクル(赤→緑)**: Forward で選択 layer が直上の layer を1枚跨ぎ、
/// 確定は1回の `apply_all` = **1 undo**。
#[test]
fn restack_forward_moves_the_selection_in_front_in_one_undo() {
    let mut doc = doc_with_comp();
    let (back, front) = (LayerId(1), LayerId(2));
    place(&mut doc, back, 0);
    place(&mut doc, front, 1);
    doc.mark_undo_floor();

    let mut session = Session::default();
    session.selection = Some(back);
    let mut pane = PaneState::new();

    let reason = pane.update(Message::RestackLayer(StackDirection::Forward), &mut doc, &mut session, no_mods());
    assert!(reason.is_none(), "正常系で拒否理由が返った: {reason:?}");
    let sequence: Vec<LayerId> = orders(&doc).into_iter().map(|(id, _)| id).collect();
    assert_eq!(sequence, vec![front, back], "背面の layer が前面へ出ていない");

    assert!(doc.undo(), "undo が効かない");
    let sequence: Vec<LayerId> = orders(&doc).into_iter().map(|(id, _)| id).collect();
    assert_eq!(sequence, vec![back, front], "undo 1回で元の重なりへ戻らない");
    assert!(!doc.can_undo(), "並べ替えが複数 undo 段に割れている(1操作=1undo 違反)");
}

/// 空選択は理由つき拒否(M13)。
#[test]
fn restack_with_no_selection_refuses_with_a_reason() {
    let mut doc = doc_with_comp();
    place(&mut doc, LayerId(1), 0);
    doc.mark_undo_floor();
    let mut session = Session::default();
    let mut pane = PaneState::new();

    let reason = pane.update(Message::RestackLayer(StackDirection::ToFront), &mut doc, &mut session, no_mods());
    assert!(reason.is_some(), "空選択が黙って飲み込まれた(M13 違反)");
    assert!(!doc.can_undo());
}

/// ロック層は理由つき拒否。
#[test]
fn restack_refuses_a_locked_layer() {
    let mut doc = doc_with_comp();
    let (a, b) = (LayerId(1), LayerId(2));
    place(&mut doc, a, 0);
    place(&mut doc, b, 1);
    lock(&mut doc, a);
    doc.mark_undo_floor();

    let mut session = Session::default();
    session.selection = Some(a);
    let mut pane = PaneState::new();

    let reason = pane.update(Message::RestackLayer(StackDirection::Forward), &mut doc, &mut session, no_mods());
    assert!(reason.is_some(), "ロック層の並べ替えが黙って通った");
    assert!(!doc.can_undo(), "拒否したのに Document が動いた");
}

/// 端で動けない移動は**黙ってスキップ**(失敗ではない — §2 split の
/// 「端ちょうどはスキップ」と同格)で、undo エントリも作らない。
#[test]
fn restack_at_the_edge_is_a_silent_noop_without_an_undo_entry() {
    let mut doc = doc_with_comp();
    let (back, front) = (LayerId(1), LayerId(2));
    place(&mut doc, back, 0);
    place(&mut doc, front, 1);
    doc.mark_undo_floor();

    let mut session = Session::default();
    session.selection = Some(front);
    let mut pane = PaneState::new();

    let reason = pane.update(Message::RestackLayer(StackDirection::Forward), &mut doc, &mut session, no_mods());
    assert!(reason.is_none(), "端の no-op が拒否扱いになっている");
    assert!(!doc.can_undo(), "no-op なのに空の undo エントリができた");
}
