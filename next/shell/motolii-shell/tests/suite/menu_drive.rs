//! MB-2(裁定179 D1 根治+181/187)の運転席検分 — header の箱ボタン列を
//! `motolii-menubar::menu_bar` へ差し替えた後の oracle。
//!
//! ORACLE(発注書):
//! - menubar 化後も既存 menu_drive 相当(全動詞が shortcut 併記で現れ、click で
//!   `Message` を publish)が通ること
//! - 旧箱ボタン(File/Edit/Undo/Redo/+ Layer/Browser/Settings の輪郭 button 列)が
//!   header から消えていること
//!
//! ## 検分の手口(MB-0 からの変更点)
//!
//! menubar の開閉状態は widget 内部にある(`motolii_menubar::menu_bar` doc
//! 「shell 側に Toggle 系 `Message` は要らない」)ので、旧 `ToggleEditMenu` →
//! `Shell::update` → `view()` 作り直しの型は使えない。代わりに
//! `next/ui/motolii-menubar/tests/menubar_oracle.rs` と同じ型 — **同一
//! `Simulator` インスタンスの中で** bar 項目 click → 項目 click を続けて行い、
//! `into_messages` で publish を照合する(overlay の項目へ selector が届くのは
//! fork の `UserInterface::operate` が root の `overlay()` を辿るため —
//! menubar_oracle 冒頭 doc 参照)。

use motolii_shell::{Message, Shell};

use crate::target_walk::collect_targets;

fn text_count(targets: &[iced_test::selector::Target], content: &str) -> usize {
    targets
        .iter()
        .filter(|t| {
            matches!(t, iced_test::selector::Target::Text { content: c, .. } if c == content)
        })
        .count()
}

/// **MB-2 の骨格**: header に4本のトップレベル(File/Edit/Layer/View)が常に
/// 見える — `menu.rs::TOP_LEVEL_LABELS` が正本(q0_fence の menubar 除外と
/// 同じ出典を読む)。
#[test]
fn the_header_shows_the_four_menubar_top_levels() {
    let shell = Shell::new().0;
    let targets = collect_targets(shell.view());
    assert_eq!(
        motolii_shell::menu::TOP_LEVEL_LABELS,
        ["File", "Edit", "Layer", "View"],
        "トップレベルの正本が発注書の4メニューと違う"
    );
    for label in motolii_shell::menu::TOP_LEVEL_LABELS {
        assert_eq!(
            text_count(&targets, label),
            1,
            "header にトップレベル {label:?} が1つだけ見えるはず"
        );
    }
}

/// **旧箱ボタンの廃止**(発注書 oracle 後半): File/Edit トリガー以外の
/// 旧文言 button(Undo/Redo/+ Layer/Browser/Settings)が閉じた既定状態の木に
/// 存在しない。"Undo"/"Redo" は Edit menu の中へ、"+ Layer" は Layer menu の
/// "New Layer" へ、Browser/Settings は右端の icon ボタン(文字なし・tooltip で
/// 語る、裁定187)へ引っ越した — 閉じている menubar の項目は木に現れない
/// (menubar_oracle「閉じている間は存在しない」)ので、既定状態でこれらの
/// 文字が1件でも見えたら旧箱ボタンの残骸。
#[test]
fn the_legacy_box_buttons_are_gone_from_the_header() {
    let shell = Shell::new().0;
    let targets = collect_targets(shell.view());
    for legacy in ["Undo", "Redo", "+ Layer"] {
        assert_eq!(
            text_count(&targets, legacy),
            0,
            "旧箱ボタンの文言 {legacy:?} がまだ木に見える(閉じた menubar の外に残骸がある)"
        );
    }
    // Browser/Settings は pane 題帯のラベルとして残る(pane_layout::title /
    // Settings パネル題帯)が、**header の文言ボタンとしては消えた** — 既定
    // 状態(Browser pane 閉・Settings パネル閉)の木では0件になる。
    for legacy in ["Browser", "Settings"] {
        assert_eq!(
            text_count(&targets, legacy),
            0,
            "header の旧 {legacy:?} 文言ボタンがまだ木に見える(icon+tooltip 化(裁定187)前の残骸)"
        );
    }
}

/// **ORACLE 本体**: Edit クリック→項目(Undo とその Cmd+Z 表記)が現れる→
/// Undo 項目クリックで `Message::Undo` が publish。同一 Simulator 内で
/// 開閉を跨ぐ(冒頭 doc の手口)。
#[test]
fn clicking_edit_reveals_items_and_clicking_undo_publishes_undo() {
    let mut shell = Shell::new().0;
    let _ = shell.update(Message::AddLayer);

    let mut ui = iced_test::simulator(shell.view());
    // 閉じている間は項目が木に存在しない(Q0 と同型 — 旧 header の Undo 箱
    // ボタンも消えたので、"Undo" は開くまで0件)。
    assert!(
        ui.find("Undo").is_err(),
        "閉じているのに Undo 項目(または旧箱ボタンの残骸)が見えている"
    );

    ui.click("Edit").expect("バー項目 Edit が見つからない");
    ui.find("Cmd+Z")
        .expect("Undo 項目の shortcut 表記(Cmd+Z)が見えない — S6 併記が崩れている");
    ui.click("Undo").expect("Edit menu の Undo 項目へ届かない");

    let messages: Vec<Message> = ui.into_messages().collect();
    assert_eq!(
        messages.len(),
        1,
        "Edit を開いて Undo を押す操作が期待どおり1件の Message を出さない: {messages:?}"
    );
    assert!(
        matches!(messages[0], Message::Undo),
        "Undo 項目クリックが Message::Undo 以外を出している: {messages:?}"
    );
}

/// 全メニュー・全項目の対応表(発注書「メニュー→Message 対応表」の実行可能な
/// 正本)。各項目について
/// - shortcut 併記(あれば)が menu を開いた木に現れる(S6 併存表 — 実装済み
///   ショートカットのみ・飾り shortcut 禁止)
/// - 項目 click で対応する既存 `Message` が publish される
/// を1本で見る。項目ごとに fresh な `Simulator` を作る(1クリックで menu が
/// 閉じるため)。
#[test]
fn every_menu_item_appears_with_its_shortcut_and_publishes_its_message() {
    // (トップレベル, 項目, shortcut 表記, publish される Message の判定)
    let table: Vec<(&str, &str, Option<&str>, fn(&Message) -> bool)> = vec![
        // ---- File(MB-1 の4動詞そのまま) ----
        ("File", "New Project", Some("Cmd+N"), |m| matches!(m, Message::NewProjectRequested)),
        ("File", "Save As…", Some("Cmd+Shift+S"), |m| matches!(m, Message::SaveAsRequested)),
        // Save a Copy は normal-map の shortcut 出典ゼロ — 飾り shortcut 禁止。
        ("File", "Save a Copy…", None, |m| matches!(m, Message::SaveACopyRequested)),
        ("File", "Quit", Some("Cmd+Q"), |m| matches!(m, Message::QuitRequested)),
        // ---- Edit(既存 menu.rs の8動詞 — Undo/Redo 含む) ----
        ("Edit", "Undo", Some("Cmd+Z"), |m| matches!(m, Message::Undo)),
        ("Edit", "Redo", Some("Cmd+Shift+Z"), |m| matches!(m, Message::Redo)),
        ("Edit", "Cut", Some("Cmd+X"), |m| matches!(m, Message::CutLayer)),
        ("Edit", "Copy", Some("Cmd+C"), |m| matches!(m, Message::CopyLayer)),
        ("Edit", "Paste", Some("Cmd+V"), |m| matches!(m, Message::PasteLayer)),
        ("Edit", "Duplicate", Some("Cmd+D"), |m| matches!(m, Message::DuplicateLayer)),
        ("Edit", "Select All", Some("Cmd+A"), |m| matches!(m, Message::SelectAllLayers)),
        ("Edit", "Deselect All", Some("Cmd+Shift+A"), |m| matches!(m, Message::DeselectAllLayers)),
        // ---- Layer(既存 Message のみ) ----
        // 旧 "+ Layer" 箱ボタンの動詞(`Message::AddLayer`)。shortcut 出典
        // ゼロ(keymap に AddLayer の腕は無い)なので併記しない。
        ("Layer", "New Layer", None, |m| matches!(m, Message::AddLayer)),
        ("Layer", "Group", Some("Cmd+G"), |m| matches!(m, Message::GroupLayers)),
        ("Layer", "Ungroup", Some("Cmd+Shift+G"), |m| matches!(m, Message::UngroupLayers)),
        // Freeze/Unfreeze(裁定119 の意図動詞、store 実装済み・UI 初露出)。
        // shortcut 未実装 — 飾り表記は書かない。
        ("Layer", "Freeze", None, |m| matches!(m, Message::FreezeGroups)),
        ("Layer", "Unfreeze", None, |m| matches!(m, Message::UnfreezeGroups)),
        // ---- View(既存 Message のみ — ui_scale は Settings に残す) ----
        ("View", "Checkerboard", None, |m| {
            matches!(m, Message::Stage(motolii_shell::stage::Message::ToggleCheckerboard))
        }),
        ("View", "Reset View", Some("Shift+F"), |m| {
            matches!(m, Message::Stage(motolii_shell::stage::Message::ResetToRenderCamera))
        }),
    ];

    for (menu, item, shortcut, is_expected) in table {
        let shell = Shell::new().0;
        let mut ui = iced_test::simulator(shell.view());
        ui.click(menu)
            .unwrap_or_else(|e| panic!("バー項目 {menu:?} が見つからない: {e:?}"));
        if let Some(shortcut) = shortcut {
            ui.find(shortcut).unwrap_or_else(|e| {
                panic!("{menu:?} > {item:?} の shortcut 表記 {shortcut:?} が見えない: {e:?}")
            });
        }
        ui.click(item)
            .unwrap_or_else(|e| panic!("{menu:?} menu の項目 {item:?} へ届かない: {e:?}"));

        let messages: Vec<Message> = ui.into_messages().collect();
        assert_eq!(
            messages.len(),
            1,
            "{menu:?} > {item:?} click が期待どおり1件の Message を出さない: {messages:?}"
        );
        assert!(
            is_expected(&messages[0]),
            "{menu:?} > {item:?} click が期待した Message を出していない: {messages:?}"
        );
    }
}

/// header 右端の icon ボタン(裁定187 icon+tooltip ペア第1号): Browser トグルと
/// Settings。文言 text を持たない(上の legacy テスト)ので、Message 経路で
/// 実配線を見る — collect_targets が拾う候補の click 総当たりで
/// `Browser(ToggleBrowserPanel)` と `Settings(ToggleSettingsPanel)` の両方が
/// header 帯(最上段)から出ることを確かめる。
#[test]
fn the_header_icon_buttons_publish_browser_and_settings_toggles() {
    let shell = Shell::new().0;
    let dims = shell.dims();
    let targets = collect_targets(shell.view());

    // header 帯の縦中心は "File" ラベル(menubar 最左)から読む — 窓の root
    // padding を数値でここへ再発明しない。
    let file_bounds = targets
        .iter()
        .find_map(|target| match target {
            iced_test::selector::Target::Text { content, .. } if content == "File" => {
                target.visible_bounds()
            }
            _ => None,
        })
        .expect("menubar の File ラベルが見つからない");
    let band_center_y = file_bounds.y + file_bounds.height / 2.0;

    let mut saw_browser_toggle = false;
    let mut saw_settings_toggle = false;
    for target in &targets {
        let Some(bounds) = target.visible_bounds() else { continue };
        // header 帯(File ラベルと同じ行)の候補だけを見る。
        let center_y = bounds.y + bounds.height / 2.0;
        if (center_y - band_center_y).abs() > dims.panel_header_height / 2.0 {
            continue;
        }
        let point =
            iced::Point::new(bounds.x + bounds.width / 2.0, bounds.y + bounds.height / 2.0);
        let probe = Shell::new().0;
        let mut ui = iced_test::simulator(probe.view());
        ui.point_at(point);
        let _ = ui.simulate([
            iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
            iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)),
        ]);
        for message in ui.into_messages() {
            match message {
                Message::Browser(motolii_shell::browser_pane::Message::ToggleBrowserPanel) => {
                    saw_browser_toggle = true;
                }
                Message::Settings(
                    motolii_shell::settings_pane::Message::ToggleSettingsPanel,
                ) => {
                    saw_settings_toggle = true;
                }
                _ => {}
            }
        }
    }
    assert!(
        saw_browser_toggle,
        "header の Browser icon ボタン(Icon::GridView)が ToggleBrowserPanel を出さない"
    );
    assert!(
        saw_settings_toggle,
        "header の Settings icon ボタン(Icon::Settings)が ToggleSettingsPanel を出さない"
    );
}
