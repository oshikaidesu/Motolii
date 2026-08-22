//! 運転席 — 第6波 shell 結線(B31: Stage マーキー選択 → `Session` 反映)。
//! `marquee.rs` 冒頭 doc「shell 側の想定結線: `session.selected_layers =
//! apply_selection(...)`」を検分する。合成規則自体
//! (`apply_selection`/`release_message` 等)は `motolii-stage-pane` 側の純関数
//! 試験が持つ — ここで見るのは `Message::Marquee` → `Session` の配線と、
//! `select_single`/`select_all_layers` と同じ「単一なら focus も揃える」規約。

use motolii_shell::{stage, Message, Shell};
use motolii_store::LayerId;

#[test]
fn replace_selection_with_a_single_id_sets_both_selection_and_selected_layers() {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::Marquee(stage::marquee::SelectLayers {
        ids: vec![LayerId(1)],
        additive: false,
    }));
    assert_eq!(shell.session().selected_layers, vec![LayerId(1)]);
    assert_eq!(
        shell.session().selection,
        Some(LayerId(1)),
        "単一選択なのに focus(selection)が立っていない — gizmo が出ない"
    );
}

#[test]
fn replace_selection_with_multiple_ids_clears_the_single_focus() {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::Marquee(stage::marquee::SelectLayers {
        ids: vec![LayerId(1), LayerId(7)],
        additive: false,
    }));
    assert_eq!(shell.session().selected_layers, vec![LayerId(1), LayerId(7)]);
    assert_eq!(
        shell.session().selection,
        None,
        "複数選択なのに単一 focus が残っている(gizmo が誤って複数選択時にも出る)"
    );
}

#[test]
fn empty_non_additive_selection_deselects_everything() {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::Select(LayerId(1)));
    let _ = shell.update(Message::Marquee(stage::marquee::SelectLayers { ids: vec![], additive: false }));
    assert!(shell.session().selected_layers.is_empty(), "空マーキー/空クリックが選択解除になっていない");
    assert_eq!(shell.session().selection, None);
}

#[test]
fn additive_toggle_removes_an_already_selected_layer() {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::Marquee(stage::marquee::SelectLayers {
        ids: vec![LayerId(1)],
        additive: false,
    }));
    let _ = shell.update(Message::Marquee(stage::marquee::SelectLayers {
        ids: vec![LayerId(1)],
        additive: true,
    }));
    assert!(
        shell.session().selected_layers.is_empty(),
        "Shift+クリックで選択済みレイヤーを外すトグルが効いていない"
    );
}

#[test]
fn additive_toggle_adds_a_new_layer_while_keeping_existing_selection() {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::Marquee(stage::marquee::SelectLayers {
        ids: vec![LayerId(1)],
        additive: false,
    }));
    let _ = shell.update(Message::Marquee(stage::marquee::SelectLayers {
        ids: vec![LayerId(7)],
        additive: true,
    }));
    assert_eq!(shell.session().selected_layers, vec![LayerId(1), LayerId(7)]);
    assert_eq!(shell.session().selection, None, "追加で複数選択になったので focus は無いはず");
}
