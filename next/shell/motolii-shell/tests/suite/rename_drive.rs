//! 運転席 — 第6波 shell 結線(rename、`timeline::write` 冒頭 doc の統合
//! 手順「`with_rename` 配線・Esc へ `cancel_rename`・Enter(単一選択)=
//! RenameBegin」)。`RenameBegin`/`RenameEdited`/`RenameCommit`/`RenameCancel`
//! 自体の意味(空名拒否・同名 no-op・ロック拒否)は `motolii-timeline-pane`
//! 側の試験(`write.rs::rename_message_tests`)が持つ — ここで見るのは
//! supervisor が足した3点の配線だけ。

use motolii_shell::{timeline_pane, Message, Shell};
use motolii_store::LayerId;

#[test]
fn enter_key_resolves_to_rename_selected_layer() {
    use iced::keyboard::{key::Named, Key, Modifiers};
    let key = Key::Named(Named::Enter);
    let message = motolii_shell::resolve_navigation_key(&key, Modifiers::default(), false);
    assert!(
        matches!(message, Some(Message::RenameSelectedLayer)),
        "Enter が Message::RenameSelectedLayer を出さない: {message:?}"
    );
}

#[test]
fn enter_is_not_stolen_while_typing() {
    use iced::keyboard::{key::Named, Key, Modifiers};
    let key = Key::Named(Named::Enter);
    assert!(
        motolii_shell::resolve_navigation_key(&key, Modifiers::default(), true).is_none(),
        "captured=true(text 編集中)なのに Enter が発火している(rename 自身の on_submit と二重発火する)"
    );
}

#[test]
fn rename_selected_layer_with_a_single_selection_begins_rename_and_committing_writes_the_name() {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::Select(LayerId(1)));
    assert!(!shell.can_undo());

    let _ = shell.update(Message::RenameSelectedLayer);
    let _ = shell.update(Message::Timeline(timeline_pane::Message::RenameEdited(
        "新しい名前".to_owned(),
    )));
    let _ = shell.update(Message::Timeline(timeline_pane::Message::RenameCommit));

    let attrs = shell.store_view().attrs(LayerId(1)).ok().flatten().unwrap_or_default();
    assert_eq!(attrs.name, "新しい名前", "Enter → 打鍵 → Commit で名前が変わっていない");
    assert!(shell.can_undo(), "rename の確定が undo 履歴に乗っていない");
}

#[test]
fn rename_selected_layer_with_no_selection_is_a_noop() {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::DeselectAllLayers);
    assert_eq!(shell.session().selection, None);

    let _ = shell.update(Message::RenameSelectedLayer);
    // 選択が無いので RenameBegin は発行されない — Commit を送っても
    // 何も書き換わらないことで間接的に確かめる(直接の draft 読み口は無い)。
    let _ = shell.update(Message::Timeline(timeline_pane::Message::RenameCommit));
    assert!(!shell.can_undo(), "選択が無いのに rename が Document へ書き込んでいる");
}

#[test]
fn escape_cancels_an_in_progress_rename_without_touching_the_document() {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::Select(LayerId(1)));
    let _ = shell.update(Message::RenameSelectedLayer);
    let _ = shell.update(Message::Timeline(timeline_pane::Message::RenameEdited(
        "捨てられる下書き".to_owned(),
    )));

    let _ = shell.update(Message::EscapePressed);

    let attrs = shell.store_view().attrs(LayerId(1)).ok().flatten().unwrap_or_default();
    assert_ne!(attrs.name, "捨てられる下書き", "Esc が rename 下書きを確定させてしまっている");
    assert!(!shell.can_undo(), "Esc の取消が Document に触れている");

    // Esc 後に改めて Enter → 打鍵 → Commit すれば通常どおり書ける
    // (cancel_rename が状態を壊れたままにしていないことの確認)。
    let _ = shell.update(Message::RenameSelectedLayer);
    let _ = shell.update(Message::Timeline(timeline_pane::Message::RenameEdited(
        "改めての名前".to_owned(),
    )));
    let _ = shell.update(Message::Timeline(timeline_pane::Message::RenameCommit));
    let attrs = shell.store_view().attrs(LayerId(1)).ok().flatten().unwrap_or_default();
    assert_eq!(attrs.name, "改めての名前", "Esc 後に rename をやり直せない");
}

#[test]
fn build_timeline_pane_carries_the_rename_draft_for_the_rail_text_input() {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::Select(LayerId(1)));
    let _ = shell.update(Message::RenameSelectedLayer);
    // `.with_rename(...)` が型として組めること(rail.rs が読む `pane.rename`
    // の供給元)——描画までは screenshot 器具の領分なので、ここは
    // `TimelinePane` が組み上がることだけを確かめる。
    let _pane = shell.build_timeline_pane();
}
