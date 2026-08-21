//! 運転席 — 編集ショートカットの配線(S0 段差 群0、κ 台帳 FINDING 1)。
//!
//! `Message::Undo`/`Redo`/`CopyLayer`/`PasteLayer`/`CutLayer`/`DuplicateLayer`/
//! `SelectAllLayers`/`DeselectAllLayers` は実装・テスト済みなのに、UI 入口
//! (キー)が1本も無かった(header の Undo/Redo ボタンのみ)。`resolve_navigation_key`
//! (`nav_drive.rs` が直接叩いているのと同じ関数)へ Cmd+文字 の腕を足す。
//!
//! `nav_drive.rs`/`clipboard_drive.rs` と同じ流儀: (a)はキー解決→`Shell::update`
//! の両方を見る、(b)は `resolve_navigation_key` を直接叩いて captured ガードだけを見る
//! (`nav_drive.rs` (e) と同型)。

use iced::keyboard::{Key, Modifiers};
use motolii_shell::{resolve_navigation_key, Message, Shell};
use motolii_store::LayerId;

fn shell() -> Shell {
    Shell::new().0
}

// ---------------------------------------------------------------------------
// (a) 各ショートカット → キー解決が正しい Message を返し、Shell に効果が出る
// ---------------------------------------------------------------------------

#[test]
fn cmd_c_then_cmd_v_resolve_to_copy_and_paste_and_add_a_layer() {
    let copy = Key::Character("c".into());
    let paste = Key::Character("v".into());
    assert!(
        matches!(resolve_navigation_key(&copy, Modifiers::COMMAND, false), Some(Message::CopyLayer)),
        "Cmd+C が CopyLayer を出さない"
    );
    assert!(
        matches!(resolve_navigation_key(&paste, Modifiers::COMMAND, false), Some(Message::PasteLayer)),
        "Cmd+V が PasteLayer を出さない"
    );

    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);
    assert_eq!(shell.layer_count(), 1);

    let _ = shell.update(resolve_navigation_key(&copy, Modifiers::COMMAND, false).unwrap());
    let _ = shell.update(resolve_navigation_key(&paste, Modifiers::COMMAND, false).unwrap());
    assert_eq!(shell.layer_count(), 2, "Cmd+C→Cmd+V で layer が増えていない");
}

#[test]
fn cmd_x_resolves_to_cut_and_removes_the_layer() {
    let cut = Key::Character("x".into());
    assert!(
        matches!(resolve_navigation_key(&cut, Modifiers::COMMAND, false), Some(Message::CutLayer)),
        "Cmd+X が CutLayer を出さない"
    );

    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);
    assert_eq!(shell.layer_count(), 1);

    let _ = shell.update(resolve_navigation_key(&cut, Modifiers::COMMAND, false).unwrap());
    assert_eq!(shell.layer_count(), 0, "Cmd+X で layer が消えていない");
}

#[test]
fn cmd_d_resolves_to_duplicate_and_adds_a_layer() {
    let duplicate = Key::Character("d".into());
    assert!(
        matches!(
            resolve_navigation_key(&duplicate, Modifiers::COMMAND, false),
            Some(Message::DuplicateLayer)
        ),
        "Cmd+D が DuplicateLayer を出さない"
    );

    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);
    assert_eq!(shell.layer_count(), 1);

    let _ = shell.update(resolve_navigation_key(&duplicate, Modifiers::COMMAND, false).unwrap());
    assert_eq!(shell.layer_count(), 2, "Cmd+D で layer が増えていない");
}

#[test]
fn cmd_z_resolves_to_undo_and_reverts_the_last_operation() {
    let undo = Key::Character("z".into());
    assert!(
        matches!(resolve_navigation_key(&undo, Modifiers::COMMAND, false), Some(Message::Undo)),
        "Cmd+Z が Undo を出さない"
    );

    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);
    assert_eq!(shell.layer_count(), 1);

    let _ = shell.update(resolve_navigation_key(&undo, Modifiers::COMMAND, false).unwrap());
    assert_eq!(shell.layer_count(), 0, "Cmd+Z で AddLayer が undo されていない");
}

#[test]
fn cmd_shift_z_resolves_to_redo_and_reapplies_the_undone_operation() {
    let redo = Key::Character("z".into());
    assert!(
        matches!(
            resolve_navigation_key(&redo, Modifiers::COMMAND | Modifiers::SHIFT, false),
            Some(Message::Redo)
        ),
        "Cmd+Shift+Z が Redo を出さない"
    );

    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);
    let _ = shell.update(Message::Undo);
    assert_eq!(shell.layer_count(), 0);

    let _ = shell.update(
        resolve_navigation_key(&redo, Modifiers::COMMAND | Modifiers::SHIFT, false).unwrap(),
    );
    assert_eq!(shell.layer_count(), 1, "Cmd+Shift+Z で AddLayer が redo されていない");
}

#[test]
fn cmd_a_resolves_to_select_all_and_selects_every_layer() {
    let select_all = Key::Character("a".into());
    assert!(
        matches!(
            resolve_navigation_key(&select_all, Modifiers::COMMAND, false),
            Some(Message::SelectAllLayers)
        ),
        "Cmd+A が SelectAllLayers を出さない"
    );

    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);
    let _ = shell.update(Message::AddLayer);
    assert_eq!(shell.layer_count(), 2);

    let _ = shell.update(resolve_navigation_key(&select_all, Modifiers::COMMAND, false).unwrap());
    let mut selected = shell.session().selected_layers.clone();
    selected.sort();
    assert_eq!(selected, vec![LayerId(1), LayerId(2)], "Cmd+A で全 layer が選ばれていない");
}

#[test]
fn cmd_shift_a_resolves_to_deselect_all_and_clears_the_selection() {
    let deselect_all = Key::Character("a".into());
    assert!(
        matches!(
            resolve_navigation_key(&deselect_all, Modifiers::COMMAND | Modifiers::SHIFT, false),
            Some(Message::DeselectAllLayers)
        ),
        "Cmd+Shift+A が DeselectAllLayers を出さない"
    );

    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);
    let _ = shell.update(Message::SelectAllLayers);
    assert!(!shell.session().selected_layers.is_empty());

    let _ = shell.update(
        resolve_navigation_key(&deselect_all, Modifiers::COMMAND | Modifiers::SHIFT, false)
            .unwrap(),
    );
    assert!(shell.session().selected_layers.is_empty(), "Cmd+Shift+A で選択が残っている");
    assert!(shell.session().selection.is_none(), "Cmd+Shift+A で単一 focus が残っている");
}

// ---------------------------------------------------------------------------
// (b) captured ガード: text 入力中(name 欄編集中相当)は横取りしない
// ---------------------------------------------------------------------------

/// `nav_drive.rs` (e) と同型 — `captured=true`(他の widget、典型的には name 欄の
/// text_input が既にそのキーを消費した状態)を渡すと、新設した8腕のどれも一切
/// `Message` を出さない(テキスト編集が優先される、Q9 の構造保証)。
#[test]
fn b_edit_shortcuts_do_not_fire_while_a_text_field_has_already_captured_the_key() {
    let candidates: Vec<(Key, Modifiers)> = vec![
        (Key::Character("z".into()), Modifiers::COMMAND),
        (Key::Character("z".into()), Modifiers::COMMAND | Modifiers::SHIFT),
        (Key::Character("c".into()), Modifiers::COMMAND),
        (Key::Character("v".into()), Modifiers::COMMAND),
        (Key::Character("x".into()), Modifiers::COMMAND),
        (Key::Character("d".into()), Modifiers::COMMAND),
        (Key::Character("a".into()), Modifiers::COMMAND),
        (Key::Character("a".into()), Modifiers::COMMAND | Modifiers::SHIFT),
    ];

    for (key, modifiers) in &candidates {
        assert!(
            resolve_navigation_key(key, *modifiers, true).is_none(),
            "captured=true(text 編集中相当)なのに {key:?}+{modifiers:?} が Message を出している"
        );
    }

    // 対照実験: captured=false なら同じキーが普通に発火する。
    for (key, modifiers) in &candidates {
        assert!(
            resolve_navigation_key(key, *modifiers, false).is_some(),
            "captured=false なのに {key:?}+{modifiers:?} が発火しない"
        );
    }
}
