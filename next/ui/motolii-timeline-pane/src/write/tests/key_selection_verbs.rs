//! キー選択動詞(第3切片 B15、map 484/509/510)+ 既存 Delete の複数対応確認
//! (発注書2「複数キーの一括 Delete」)。**未実行**(裁定189)。SP-2 分割で
//! 元は `key_selection_verb_tests`(`write.rs` 内の兄弟モジュール)だった
//! 物をそのまま移設(中身は無改変)。

use super::fixtures::*;
use crate::write::*;

/// map 484: 全キー選択解除 — Session だけが動き、Document は無傷。
#[test]
fn deselect_all_keys_clears_the_selection_and_anchor() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    let property = opacity();
    place(&mut doc, layer, 0);
    set_track(&mut doc, layer, &property, &[0, 100]);
    doc.mark_undo_floor();

    let mut session = Session::default();
    session.selected_keys = vec![selector(layer, &property, 0)];
    session.key_anchor = Some(selector(layer, &property, 0));
    let mut pane = PaneState::new();

    let reason = pane.update(Message::DeselectAllKeys, &mut doc, &mut session, no_mods());
    assert!(reason.is_none());
    assert!(session.selected_keys.is_empty(), "選択が残っている");
    assert!(session.key_anchor.is_none(), "anchor が残っている");
    assert!(!doc.can_undo(), "選択解除が Document を動かした");
}

/// map 509(§8.1 SelectAllKeysOfProperty): property の全キーが選択へ
/// 差し替わり、anchor は先頭キー。
#[test]
fn select_all_keys_of_property_selects_every_key_and_anchors_the_first() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    let property = opacity();
    place(&mut doc, layer, 0);
    set_track(&mut doc, layer, &property, &[0, 100, 250]);

    let mut session = Session::default();
    session.selection = Some(layer);
    // 差し替え(足し引きではない)ことを見るため、別の選択を先に置く。
    session.selected_keys = vec![selector(layer, &property, 100)];
    let mut pane = PaneState::new();

    let reason = pane.update(
        Message::SelectAllKeysOfProperty { layer, property: property.clone() },
        &mut doc,
        &mut session,
        no_mods(),
    );
    assert!(reason.is_none());
    assert_eq!(
        session.selected_keys,
        vec![
            selector(layer, &property, 0),
            selector(layer, &property, 100),
            selector(layer, &property, 250),
        ],
        "property の全キーが選択されていない"
    );
    assert_eq!(session.key_anchor, Some(selector(layer, &property, 0)), "anchor が先頭キーでない");
}

/// map 510: 表示中(選択 layer のキー持ち property 行)の全キーが
/// `key_order`(行順→時刻順)で選択される。
#[test]
fn select_all_visible_keys_spans_every_visible_property_row() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    let (opacity, position_x) = (opacity(), position_x());
    place(&mut doc, layer, 0);
    set_track(&mut doc, layer, &opacity, &[0, 100]);
    set_track(&mut doc, layer, &position_x, &[50]);

    let mut session = Session::default();
    session.selection = Some(layer);
    let mut pane = PaneState::new();

    let reason = pane.update(Message::SelectAllVisibleKeys, &mut doc, &mut session, no_mods());
    assert!(reason.is_none());
    assert_eq!(session.selected_keys.len(), 3, "表示中の全キー(2+1本)が選択されていない");
    for key in [
        selector(layer, &opacity, 0),
        selector(layer, &opacity, 100),
        selector(layer, &position_x, 50),
    ] {
        assert!(session.selected_keys.contains(&key), "{key:?} が選択に入っていない");
    }
    // anchor は key_order の先頭(行順→時刻順の1本目)。
    let fps = doc.view().composition().unwrap().unwrap().fps;
    let order = key_order(&property_rows(&doc.view(), &session, Some(fps)));
    assert_eq!(session.key_anchor.as_ref(), order.first(), "anchor が key_order の先頭でない");
}

/// **発注書2の確認**: 既存 `DeleteSelectedKeys` は複数キー(property 跨ぎ)を
/// 1回で消し、**1 undo** で全部戻る — 複数対応は既に実装済みであることの柵。
#[test]
fn delete_selected_keys_removes_keys_across_properties_in_one_undo() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    let (opacity, position_x) = (opacity(), position_x());
    place(&mut doc, layer, 0);
    set_track(&mut doc, layer, &opacity, &[0, 100]);
    set_track(&mut doc, layer, &position_x, &[50, 60]);
    doc.mark_undo_floor();

    let mut session = Session::default();
    session.selection = Some(layer);
    session.selected_keys =
        vec![selector(layer, &opacity, 100), selector(layer, &position_x, 50)];
    let mut pane = PaneState::new();

    let reason = pane.update(Message::DeleteSelectedKeys, &mut doc, &mut session, no_mods());
    assert!(reason.is_none());
    let frames = |doc: &Document, property: &PropertyId| -> Vec<i64> {
        interps_of(doc, layer, property).into_iter().map(|(frame, _)| frame).collect()
    };
    assert_eq!(frames(&doc, &opacity), vec![0], "opacity 側の選択キーが消えていない");
    assert_eq!(frames(&doc, &position_x), vec![60], "position.x 側の選択キーが消えていない");

    assert!(doc.undo(), "undo が効かない");
    assert_eq!(frames(&doc, &opacity), vec![0, 100]);
    assert_eq!(frames(&doc, &position_x), vec![50, 60]);
    assert!(!doc.can_undo(), "property 跨ぎの削除が複数 undo 段に割れている(1操作=1undo 違反)");
}
