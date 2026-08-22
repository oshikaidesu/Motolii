//! oracle(発注書の落ちるテスト): バー項目 click → menu が開く → 項目 click →
//! `message` 発火。probe(`docs/reviews/2026-08-22-iced-aw-menu-probe.md` outcome 2)
//! が iced_aw のビルド不能で未達だった `iced_test::Simulator` 検分を、vendoring
//! 移植後のこの crate で達成する。
//!
//! 検分器具は shell の既存流儀(`tests/suite/iced_test_spike.rs`)と同じ:
//! text selector で find/click し、`into_messages` で publish を照合する。
//! overlay(開いた menu)へ selector が届くのは、fork の
//! `UserInterface::operate` が root の `overlay()` を辿り(runtime/src/
//! user_interface.rs)、vendored `MenuBarOverlay` が `operate()` を実装している
//! (probe で `overlay::menu::Menu` 案を落とした欠落が iced_aw 系には無い)ため。

use motolii_menubar::{menu_bar, Item, Menu};
use motolii_tokens_rs::Tokens;

#[derive(Debug, Clone, PartialEq)]
enum TestMessage {
    NewProject,
    SaveAs,
    Undo,
}

/// File/Edit の2本 — 実 shell の形(複数トップレベル)で組む。
fn two_menus() -> Vec<Menu<TestMessage>> {
    vec![
        Menu {
            label: "File",
            items: vec![
                Item {
                    label: "New Project",
                    shortcut: Some("Cmd+N"),
                    message: TestMessage::NewProject,
                },
                // shortcut 出典ゼロの形(`None`)も oracle に含める。
                Item {
                    label: "Save a Copy",
                    shortcut: None,
                    message: TestMessage::SaveAs,
                },
            ],
        },
        Menu {
            label: "Edit",
            items: vec![Item {
                label: "Undo",
                shortcut: Some("Cmd+Z"),
                message: TestMessage::Undo,
            }],
        },
    ]
}

/// **本命 oracle**: bar 項目 click で menu が開き(selector が overlay の項目へ
/// 届くようになり)、項目 click で `message` が publish される。
#[test]
fn clicking_a_bar_item_opens_the_menu_and_clicking_an_entry_publishes_its_message() {
    let tokens = Tokens::default();
    let mut ui = iced_test::simulator(menu_bar(two_menus(), tokens.dims, tokens.colors));

    ui.click("File").expect("バー項目 File が text selector で見つからない");
    ui.click("New Project")
        .expect("File menu が開いていない(overlay の項目へ selector が届かない)");

    let messages: Vec<_> = ui.into_messages().collect();
    assert_eq!(
        messages,
        vec![TestMessage::NewProject],
        "menu 項目 click が期待した message を publish していない"
    );
}

/// 閉じている間は menu 項目が widget 木に**存在しない**(Q0 と同型 — 開いて
/// いなければ selector から一切見えない)。開いた後にだけ見えることとの対で、
/// 「click が開閉を実際に切り替えている」ことの構造証明。
#[test]
fn menu_entries_are_absent_from_the_tree_until_the_bar_item_is_clicked() {
    let tokens = Tokens::default();
    let mut ui = iced_test::simulator(menu_bar(two_menus(), tokens.dims, tokens.colors));

    assert!(
        ui.find("New Project").is_err(),
        "閉じているのに menu 項目が selector から見えている"
    );

    ui.click("File").expect("バー項目 File が見つからない");
    assert!(
        ui.find("New Project").is_ok(),
        "File を click したのに menu 項目が見えるようにならない"
    );

    // 開いただけでは何も publish しない(silent open — バー項目は Message を
    // 運ばない、開閉は widget 内部状態)。
    let messages: Vec<_> = ui.into_messages().collect();
    assert!(
        messages.is_empty(),
        "menu を開いただけで message が飛んでいる: {messages:?}"
    );
}

/// 2本目のトップレベル(Edit)も同じ経路で動く — roots が1本きりの特殊解に
/// なっていないことの確認。
#[test]
fn the_second_top_level_menu_works_through_the_same_path() {
    let tokens = Tokens::default();
    let mut ui = iced_test::simulator(menu_bar(two_menus(), tokens.dims, tokens.colors));

    ui.click("Edit").expect("バー項目 Edit が見つからない");
    ui.click("Undo").expect("Edit menu の項目へ届かない");

    let messages: Vec<_> = ui.into_messages().collect();
    assert_eq!(messages, vec![TestMessage::Undo]);
}
