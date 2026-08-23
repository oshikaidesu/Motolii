//! inline rename(第3切片、map B02 785・正典 §6)。**未実行**(裁定189)。
//! SP-2 分割で元は `rename_message_tests`(`write.rs` 内の兄弟モジュール)
//! だった物をそのまま移設(中身は無改変)。

use super::fixtures::*;
use crate::write::*;

fn name_of(doc: &Document, layer: LayerId) -> String {
    doc.view().attrs(layer).unwrap().unwrap_or_default().name
}

fn set_name(doc: &mut Document, layer: LayerId, name: &str) {
    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch { name: Some(name.to_owned()), ..Default::default() },
    })
    .expect("名前設定");
}

/// **オラクル(赤→緑)**: begin は現在名を下書きへ写し、commit が
/// `SetAttrs` を1回出す(**1 undo**)。
#[test]
fn rename_begin_seeds_the_draft_and_commit_writes_the_name_once() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place(&mut doc, layer, 0);
    set_name(&mut doc, layer, "before");
    doc.mark_undo_floor();

    let mut session = Session::default();
    session.selection = Some(layer);
    let mut pane = PaneState::new();

    assert!(pane.update(Message::RenameBegin(layer), &mut doc, &mut session, no_mods()).is_none());
    assert_eq!(pane.rename_draft(), Some((layer, "before")), "下書きが現在名から始まっていない");

    pane.update(Message::RenameEdited("after".into()), &mut doc, &mut session, no_mods());
    let reason = pane.update(Message::RenameCommit, &mut doc, &mut session, no_mods());
    assert!(reason.is_none(), "正常系の確定が拒否された: {reason:?}");
    assert!(pane.rename_draft().is_none(), "確定後も rename 状態が残っている");
    assert_eq!(name_of(&doc, layer), "after");

    assert!(doc.undo(), "undo が効かない");
    assert_eq!(name_of(&doc, layer), "before", "undo 1回で旧名へ戻らない");
    assert!(!doc.can_undo(), "改名が複数 undo 段に割れている(1操作=1undo 違反)");
}

/// 正典 §6「空名拒否」: 拒否は理由つきで、**下書きは捨てない**(編集継続 —
/// 入力を失わない)。Document は無傷。
#[test]
fn rename_commit_rejects_an_empty_name_and_keeps_editing() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place(&mut doc, layer, 0);
    set_name(&mut doc, layer, "before");
    doc.mark_undo_floor();

    let mut session = Session::default();
    let mut pane = PaneState::new();
    pane.update(Message::RenameBegin(layer), &mut doc, &mut session, no_mods());
    pane.update(Message::RenameEdited("   ".into()), &mut doc, &mut session, no_mods());

    let reason = pane.update(Message::RenameCommit, &mut doc, &mut session, no_mods());
    assert!(reason.is_some(), "空名が黙って通った/黙って捨てられた(M13 違反)");
    assert_eq!(pane.rename_draft(), Some((layer, "   ")), "拒否で下書きが失われた(編集継続でない)");
    assert_eq!(name_of(&doc, layer), "before");
    assert!(!doc.can_undo(), "拒否したのに Document が動いた");
}

/// 正典 §6「同名 no-op」: Intent を出さない(空 undo を作らない)。
#[test]
fn rename_commit_with_the_unchanged_name_is_a_noop() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place(&mut doc, layer, 0);
    set_name(&mut doc, layer, "same");
    doc.mark_undo_floor();

    let mut session = Session::default();
    let mut pane = PaneState::new();
    pane.update(Message::RenameBegin(layer), &mut doc, &mut session, no_mods());
    let reason = pane.update(Message::RenameCommit, &mut doc, &mut session, no_mods());
    assert!(reason.is_none());
    assert!(pane.rename_draft().is_none());
    assert!(!doc.can_undo(), "同名確定が空の undo エントリを作った");
}

/// ロック層は begin の時点で理由つき拒否(`start_drag` と同じ柵)。
#[test]
fn rename_begin_refuses_a_locked_layer() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place(&mut doc, layer, 0);
    lock(&mut doc, layer);

    let mut session = Session::default();
    let mut pane = PaneState::new();
    let reason = pane.update(Message::RenameBegin(layer), &mut doc, &mut session, no_mods());
    assert!(reason.is_some(), "ロック層の改名開始が黙って通った");
    assert!(pane.rename_draft().is_none());
}

/// Esc(裁定151「キャンセルの一般化」): `cancel_rename` は state を捨てる
/// だけで Document 無傷。2回目は「何も捨てなかった」。
#[test]
fn cancel_rename_drops_the_draft_without_touching_the_document() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place(&mut doc, layer, 0);
    set_name(&mut doc, layer, "before");
    doc.mark_undo_floor();

    let mut session = Session::default();
    let mut pane = PaneState::new();
    pane.update(Message::RenameBegin(layer), &mut doc, &mut session, no_mods());
    pane.update(Message::RenameEdited("half-typed".into()), &mut doc, &mut session, no_mods());

    assert!(pane.cancel_rename(), "捨てる物があるのに false");
    assert!(pane.rename_draft().is_none());
    assert!(!pane.cancel_rename(), "2回目のキャンセルが true を返した");
    assert_eq!(name_of(&doc, layer), "before");
    assert!(!doc.can_undo());
}
