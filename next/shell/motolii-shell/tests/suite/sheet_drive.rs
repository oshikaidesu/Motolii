//! 運転席 — 第6波 shell 結線(B22: 方眼シート束の View メニュー結線)。
//! `stage::sheets` 冒頭 doc「家(結線は次波)— トグル状態は shell が Session
//! 状態として持つ」を検分する: `Message::Sheet` が
//! `Shell::sheet_toggles()` を反転し、View メニューに4項目とも露出している。

use motolii_shell::{menu, stage, Message, Shell};

#[test]
fn sheet_toggles_default_to_all_off() {
    let shell = Shell::new_fixture().0;
    assert_eq!(shell.sheet_toggles(), stage::SheetToggles::default(), "既定は全部 off のはず(mock 初期値)");
}

#[test]
fn sheet_toggle_message_flips_grid_and_flips_back() {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::Sheet(stage::SheetMessage::Toggle(stage::Sheet::Grid)));
    assert!(shell.sheet_toggles().grid, "Grid トグルが反映されていない");

    let _ = shell.update(Message::Sheet(stage::SheetMessage::Toggle(stage::Sheet::Grid)));
    assert!(!shell.sheet_toggles().grid, "もう一度押しても戻らない(反転になっていない)");
}

#[test]
fn sheet_toggle_message_covers_all_four_sheets_independently() {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::Sheet(stage::SheetMessage::Toggle(stage::Sheet::Thirds)));
    let _ = shell.update(Message::Sheet(stage::SheetMessage::Toggle(stage::Sheet::GoldenRatio)));
    let _ = shell.update(Message::Sheet(stage::SheetMessage::Toggle(stage::Sheet::SafeMargins)));

    let toggles = shell.sheet_toggles();
    assert!(toggles.thirds, "Thirds が反映されていない");
    assert!(toggles.golden_ratio, "GoldenRatio が反映されていない");
    assert!(toggles.safe_margins, "SafeMargins が反映されていない");
    assert!(!toggles.grid, "Grid に触っていないのに立っている(独立トグルでない)");
}

#[test]
fn view_menu_exposes_all_four_sheet_toggles_with_no_invented_shortcut() {
    let view_menu = menu::menus()
        .into_iter()
        .find(|m| m.label == "View")
        .expect("View メニューが無い");

    let labels: Vec<&str> = view_menu.items.iter().map(|item| item.label).collect();
    for expected in ["Grid", "Thirds", "Golden Ratio", "Safe Margins"] {
        assert!(labels.contains(&expected), "View メニューに {expected} が無い: {labels:?}");
    }
    // shortcut 出典ゼロ(発明しない、S6 併存の規律 — menu.rs 冒頭 doc)。
    for item in &view_menu.items {
        if ["Grid", "Thirds", "Golden Ratio", "Safe Margins"].contains(&item.label) {
            assert!(item.shortcut.is_none(), "{} に無い shortcut を発明している", item.label);
        }
    }
}
