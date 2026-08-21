//! 運転席 — G1 グループ化動詞(裁定174「意図優先の原則」)。
//!
//! `Document::group_layers`/`ungroup_layers` 自体の oracle(1 undo・恒等 Group
//! の絵不変・M13・入れ子)は `motolii-store/tests/group_ungroup.rs` が既に
//! 固定済み — ここで見るのは **Shell 経由の配線**:
//! - ⌘G/⌘⇧G が `resolve_navigation_key` を通って正しい `Message` を出す
//!   (`shortcut_drive.rs` と同じ手口)
//! - `Shell::update` が `Session::selected_layers` を読み、選択規則(グループ化
//!   後は Group を選ぶ・解除後は旧子らを選ぶ)を実装している
//! - Ungroup の数値証明(子の world 位置保存)を `Shell::update` 経由(Inspector
//!   の Transform field 実口 — `inspector_drive.rs` と同じ書き方)で確かめる
//!   — H1 と同じ数字系(`transform_hierarchy.rs` 参照)。**書き口を新設しない**
//!   (Shell の唯一の書き口は `Message` — このファイルは既存の
//!   `Message::Inspector(FieldInput/FieldSubmit)` をそのまま使う)。

use iced::keyboard::{Key, Modifiers};
use motolii_shell::inspector_pane::{self, TransformField};
use motolii_shell::{resolve_navigation_key, Message, Shell};
use motolii_store::{LayerId, LayerSource, RationalTime};

fn shell() -> Shell {
    Shell::new().0
}

fn t0() -> RationalTime {
    RationalTime::ZERO
}

fn add_layer(shell: &mut Shell) -> LayerId {
    let _ = shell.update(Message::AddLayer);
    shell.session().selection.expect("AddLayer は選択する")
}

/// `field` を選択中の layer へ書く(実際の Inspector 打鍵と同じ2手:
/// FieldInput で下書き→FieldSubmit で `Intent::SetTrack` を1回確定)。
fn set_field(shell: &mut Shell, field: TransformField, text: &str) {
    let _ = shell.update(Message::Inspector(inspector_pane::Message::FieldInput(
        field,
        text.to_owned(),
    )));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::FieldSubmit(field)));
}

// ---------------------------------------------------------------------------
// (a) ⌘G/⌘⇧G のキー解決
// ---------------------------------------------------------------------------

#[test]
fn cmd_g_resolves_to_group_layers() {
    let key = Key::Character("g".into());
    assert!(
        matches!(resolve_navigation_key(&key, Modifiers::COMMAND, false), Some(Message::GroupLayers)),
        "Cmd+G が GroupLayers を出さない"
    );
}

#[test]
fn cmd_shift_g_resolves_to_ungroup_layers() {
    let key = Key::Character("g".into());
    assert!(
        matches!(
            resolve_navigation_key(&key, Modifiers::COMMAND | Modifiers::SHIFT, false),
            Some(Message::UngroupLayers)
        ),
        "Cmd+Shift+G が UngroupLayers を出さない"
    );
}

/// `shortcut_drive.rs` (b) と同型 — text 編集中は横取りしない。
#[test]
fn group_shortcuts_do_not_fire_while_a_text_field_has_already_captured_the_key() {
    let candidates: Vec<(Key, Modifiers)> = vec![
        (Key::Character("g".into()), Modifiers::COMMAND),
        (Key::Character("g".into()), Modifiers::COMMAND | Modifiers::SHIFT),
    ];
    for (key, modifiers) in &candidates {
        assert!(
            resolve_navigation_key(key, *modifiers, true).is_none(),
            "captured=true なのに {key:?}+{modifiers:?} が Message を出している"
        );
    }
    for (key, modifiers) in &candidates {
        assert!(
            resolve_navigation_key(key, *modifiers, false).is_some(),
            "captured=false なのに {key:?}+{modifiers:?} が発火しない"
        );
    }
}

// ---------------------------------------------------------------------------
// (b) Message::GroupLayers — 選択規則・1 undo
// ---------------------------------------------------------------------------

#[test]
fn group_layers_bundles_the_multi_selection_and_selects_the_new_group() {
    let mut shell = shell();
    let a = add_layer(&mut shell);
    let b = add_layer(&mut shell);
    let c = add_layer(&mut shell);
    assert_eq!(shell.layer_count(), 3);

    // present な layer は a/b/c の3本だけ — Select All で3本まとめて選ぶ
    // (`clipboard::select_all` は「見えている行だけ」= 今は present 全部)。
    let _ = shell.update(Message::SelectAllLayers);
    let mut selected = shell.session().selected_layers.clone();
    selected.sort();
    let mut expected = vec![a, b, c];
    expected.sort();
    assert_eq!(selected, expected, "Select All の前提が崩れている");

    let _ = shell.update(Message::GroupLayers);
    assert_eq!(shell.status(), None, "GroupLayers が拒否されている: {:?}", shell.status());
    assert_eq!(shell.layer_count(), 4, "Group 層が増えていない");

    let group = shell.session().selection.expect("グループ化後は Group を選ぶはず");
    assert_eq!(
        shell.store_view().meta(group).unwrap().unwrap().source,
        LayerSource::Group
    );
    for child in [a, b, c] {
        assert_eq!(
            shell.store_view().attrs(child).unwrap().unwrap().parent,
            Some(group),
            "子の parent が Group を向いていない"
        );
    }

    // 1 gesture = 1 undo。
    let _ = shell.update(Message::Undo);
    assert_eq!(
        shell.layer_count(),
        3,
        "Undo 1回で GroupLayers 前へ完全に戻らない = 1操作が複数 undo になっている"
    );
}

#[test]
fn group_layers_on_an_empty_selection_is_a_no_op() {
    let mut shell = shell();
    let _ = add_layer(&mut shell);
    let _ = shell.update(Message::DeselectAllLayers);
    assert!(shell.session().selected_layers.is_empty());

    let can_undo_before = shell.can_undo();
    let _ = shell.update(Message::GroupLayers);
    assert_eq!(shell.layer_count(), 1, "空選択なのに Group が生まれている");
    assert_eq!(
        shell.can_undo(),
        can_undo_before,
        "空選択なのに undo 刻みが積まれている"
    );
}

// ---------------------------------------------------------------------------
// (c) Message::UngroupLayers — 選択規則・世界位置保存
// ---------------------------------------------------------------------------

#[test]
fn ungroup_layers_restores_the_old_children_as_the_selection() {
    let mut shell = shell();
    let a = add_layer(&mut shell);
    let b = add_layer(&mut shell);
    let _ = shell.update(Message::SelectAllLayers);
    let _ = shell.update(Message::GroupLayers);
    let group = shell.session().selection.expect("グループ化後は Group を選ぶ");

    // Ungroup 対象は今選ばれている Group 自身(`select_single` が
    // `selected_layers` も同期させている)。
    let _ = shell.update(Message::UngroupLayers);

    assert_eq!(shell.status(), None, "UngroupLayers が拒否されている: {:?}", shell.status());
    assert!(!shell.store_view().has_layer(group), "Group が tombstone になっていない");

    let mut selected = shell.session().selected_layers.clone();
    selected.sort();
    let mut expected = vec![a, b];
    expected.sort();
    assert_eq!(selected, expected, "解除後に旧子らが選ばれていない");
    assert!(shell.session().selection.is_none(), "複数の旧子は単一focusにならないはず");
}

/// 単一の子を持つ Group を解除した場合は `select_single` と同型(単一 focus
/// も入る)。
#[test]
fn ungrouping_a_single_child_group_restores_a_single_focus_selection() {
    let mut shell = shell();
    let a = add_layer(&mut shell);
    let _ = shell.update(Message::GroupLayers);
    let group = shell.session().selection.unwrap();

    let _ = shell.update(Message::UngroupLayers);
    assert_eq!(shell.session().selection, Some(a));
    assert_eq!(shell.session().selected_layers, vec![a]);
    assert!(!shell.store_view().has_layer(group));
}

/// **数値証明(H1 と同じ数字系)**: Group に position/rotation を与えてから
/// Ungroup しても、子の world 位置は変わらない(`Document::ungroup_layers` の
/// 焼き込みが `Shell::update` 経由でも効くことの直接証拠) — 手計算は
/// `motolii-store/tests/group_ungroup.rs::ungroup_preserves_the_childs_world_position_with_rotation`
/// と同型(子 position(10,5)・Group position(100,0)・rotation 90° →
/// world(95,10))。
#[test]
fn ungroup_preserves_the_childs_world_position_through_shell_update() {
    let mut shell = shell();
    let a = add_layer(&mut shell);
    set_field(&mut shell, TransformField::PositionX, "10");
    set_field(&mut shell, TransformField::PositionY, "5");

    let _ = shell.update(Message::GroupLayers);
    shell.session().selection.expect("グループ化後は Group を選ぶ");
    set_field(&mut shell, TransformField::PositionX, "100");
    set_field(&mut shell, TransformField::PositionY, "0");
    set_field(&mut shell, TransformField::Rotation, "90");

    let before = shell
        .store_view()
        .resolve(a, t0())
        .unwrap()
        .unwrap()
        .placement
        .transform;
    assert!((before.translation.x - 95.0).abs() < 1e-2, "{before:?}");
    assert!((before.translation.y - 10.0).abs() < 1e-2, "{before:?}");

    let _ = shell.update(Message::UngroupLayers);
    assert_eq!(shell.status(), None, "UngroupLayers が拒否されている: {:?}", shell.status());

    let after = shell
        .store_view()
        .resolve(a, t0())
        .unwrap()
        .unwrap()
        .placement
        .transform;
    assert!(
        (before.translation.x - after.translation.x).abs() < 1e-2
            && (before.translation.y - after.translation.y).abs() < 1e-2,
        "Ungroup で子の world 位置が変わった: before={before:?} after={after:?}"
    );
}

#[test]
fn ungroup_layers_on_an_empty_selection_is_a_no_op() {
    let mut shell = shell();
    let can_undo_before = shell.can_undo();
    let _ = shell.update(Message::UngroupLayers);
    assert_eq!(shell.can_undo(), can_undo_before, "空選択なのに undo 刻みが積まれている");
}

/// Group でない layer を選んだまま Ungroup しても無視される(`Document::
/// ungroup_layers` が黙って飛ばす、`Shell` は拒否 status も出さない)。
#[test]
fn ungrouping_a_selection_with_no_group_in_it_is_a_no_op() {
    let mut shell = shell();
    let a = add_layer(&mut shell);
    let can_undo_before = shell.can_undo();

    let _ = shell.update(Message::UngroupLayers);
    assert_eq!(shell.status(), None);
    assert_eq!(shell.can_undo(), can_undo_before, "Group でない選択で undo 刻みが積まれた");
    assert!(shell.store_view().has_layer(a), "Group でない layer が消されている");
}
