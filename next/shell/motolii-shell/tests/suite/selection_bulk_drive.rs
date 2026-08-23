//! 運転席 — 選択の正本の一本化と一括動詞(C-2、軸台帳 A08)。
//!
//! 裁定218: テストは「検収条件そのもの」に絞る。ここに置くのはその2点だけ
//! ——他の動詞(Lock/Hide/Solo 一括版)は型で説明が付く(RETURN 参照: guard
//! 節が空選択を弾き、`apply_all` が1 undo を保証し、目標値の計算自体は
//! 既に `motolii-timeline-pane::rows::bulk_toggle_target` 側のテストが
//! 固定済み)ので、ここでは重ねない。
//!
//! `clipboard_drive.rs`/`group_drive.rs` と同じ流儀(窓を開けずに `Shell` を
//! 動かす)。

use motolii_shell::{Message, Shell};

fn shell() -> Shell {
    Shell::new().0
}

fn add_layer(shell: &mut Shell) {
    let _ = shell.update(Message::AddLayer);
}

/// 検収条件(id434): 複数選択して複製すると、選んだ枚数ぶん全部複製される
/// (旧実装は `session.selection` 単数しか読まず1枚しか複製できなかった)。
/// 1 apply_all = 1 undo であることも同じ操作で確かめる。
#[test]
fn select_all_then_duplicate_duplicates_every_selected_layer() {
    let mut shell = shell();
    add_layer(&mut shell);
    add_layer(&mut shell);
    add_layer(&mut shell);
    assert_eq!(shell.layer_count(), 3);

    let _ = shell.update(Message::SelectAllLayers);
    assert_eq!(shell.session().selected_layers.len(), 3, "Select All が3枚を選んでいない");

    let _ = shell.update(Message::DuplicateLayer);
    assert_eq!(shell.status(), None, "Duplicate が拒否されている: {:?}", shell.status());
    assert_eq!(shell.layer_count(), 6, "選択3枚のうち1枚しか複製されていない(旧: 単数のみの穴)");
    assert_eq!(shell.session().selected_layers.len(), 3, "複製後に増えた3枚を選び直していない");

    let _ = shell.update(Message::Undo);
    assert_eq!(shell.layer_count(), 3, "Undo 1回で複製3枚が戻らない = 1操作が複数 undo になっている");
}

/// 検収条件(id431): 複数選択して削除すると全部消える(専用動詞が無く
/// Cut 経由の単数削除しかできなかった穴)。同じ操作で1 undo も確かめる。
#[test]
fn select_all_then_delete_removes_every_selected_layer_in_one_undo_step() {
    let mut shell = shell();
    add_layer(&mut shell);
    add_layer(&mut shell);
    add_layer(&mut shell);
    assert_eq!(shell.layer_count(), 3);

    let _ = shell.update(Message::SelectAllLayers);
    let _ = shell.update(Message::DeleteSelectedLayers);
    assert_eq!(shell.status(), None, "Delete が拒否されている: {:?}", shell.status());
    assert_eq!(shell.layer_count(), 0, "選択した3枚のうち一部しか消えていない");
    assert!(shell.session().selected_layers.is_empty());
    assert!(shell.session().selection.is_none());

    let _ = shell.update(Message::Undo);
    assert_eq!(shell.layer_count(), 3, "Undo 1回で削除3枚が戻らない = 1操作が複数 undo になっている");
}
