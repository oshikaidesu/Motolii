//! header メニューバーの**意味定義**(MB-2、裁定179 D1 根治+181/187)。
//!
//! ## MB-2 で widget と意味を分離した
//!
//! MB-0/MB-1 はこのファイルが widget(トリガー button+縦積み dropdown)まで
//! 自作していたが、`motolii-menubar` crate(iced_aw menu の vendoring 移植・
//! Simulator oracle 済み)が着地したので、このファイルは**メニューの中身
//! (`Menu`/`Item` の列)だけ**を持つ形へ畳んだ — 開閉状態は widget 内部
//! (`motolii_menubar::menu_bar` doc)、見た目は menubar crate の「枠の文法」
//! (裁定179: バー項目=常時輪郭なし・hover で面)。旧実装の
//! `ToggleEditMenu`/`ToggleFileMenu`(表示専用 view flag)は widget 内部状態に
//! 置き換わったため `Message` ごと廃止した(意味の家は1つ)。
//!
//! ## 全項目が「既存 `Message` の露出」(発注書 — 新しい意味は作らない)
//!
//! - **File** = MB-1(裁定176)の4動詞そのまま(New Project/Save As/
//!   Save a Copy/Quit)。
//! - **Edit** = MB-0+Edit の8動詞そのまま(Undo/Redo/Cut/Copy/Paste/
//!   Duplicate/Select All/Deselect All)。旧 header の Undo/Redo 箱ボタンは
//!   廃止 — 入口はメニューと shortcut の2本(S6 併存)。
//! - **Layer** = 旧 "+ Layer" 箱ボタン(`Message::AddLayer`)+G1 の
//!   Group/Ungroup(裁定174 — MB-0 当時は Layer トップレベルが無く Edit に
//!   同居させていた分の引っ越し)+**Freeze/Unfreeze(裁定119 の意図動詞、
//!   store 実装済み・UI 初露出 — `Message::FreezeGroups`/`UnfreezeGroups`)**。
//! - **View** = 市松トグル(`stage::Message::ToggleCheckerboard`)と観測視点
//!   リセット(`stage::Message::ResetToRenderCamera`、Shift+F 相当)。ui_scale
//!   は Settings パネルに残す(発注書 — View に入れない)。
//!
//! ## S6 併存(shortcut 表記は keymap の実態と一致)
//!
//! shortcut 併記は `resolve_navigation_key`(+ Shift+F の生 event 腕)に
//! **実装済みの割当だけ**を書く — 未実装の飾り shortcut 禁止(発注書)。
//! 出典ゼロの項目(Save a Copy/New Layer/Freeze/Unfreeze/Checkerboard)は
//! `shortcut: None`(存在しない shortcut を発明しない — MB-1 からの規律)。

use motolii_menubar::{Item, Menu};

use crate::Message;

/// トップレベル4本の正本(左から順)。`menus()` と同じ出典 — q0_fence の
/// menubar 除外(bar click は「menu が開く」という widget 内部応答で、
/// `Message` を publish しない)がこの表を読んで bar 領域を特定する。
pub const TOP_LEVEL_LABELS: [&str; 4] = ["File", "Edit", "Layer", "View"];

/// header メニューバーの全定義。`Shell::header` が毎フレーム
/// `motolii_menubar::menu_bar(menus(), …)` へ渡す。
pub fn menus() -> Vec<Menu<Message>> {
    vec![
        // ---- File(MB-1、裁定176 — normal-map id 1221/1225/1227/1223) ----
        Menu {
            label: "File",
            items: vec![
                Item {
                    label: "New Project",
                    shortcut: Some("Cmd+N"),
                    message: Message::NewProjectRequested,
                },
                Item {
                    label: "Save As…",
                    shortcut: Some("Cmd+Shift+S"),
                    message: Message::SaveAsRequested,
                },
                // normal-map entries 2:0:0:0 — shortcut 出典ゼロ(MB-1)。
                Item {
                    label: "Save a Copy…",
                    shortcut: None,
                    message: Message::SaveACopyRequested,
                },
                Item { label: "Quit", shortcut: Some("Cmd+Q"), message: Message::QuitRequested },
            ],
        },
        // ---- Edit(MB-0+Edit の8動詞、normal-map id 437/435/432/429/430/
        // 434/436/433) ----
        Menu {
            label: "Edit",
            items: vec![
                Item { label: "Undo", shortcut: Some("Cmd+Z"), message: Message::Undo },
                Item { label: "Redo", shortcut: Some("Cmd+Shift+Z"), message: Message::Redo },
                Item { label: "Cut", shortcut: Some("Cmd+X"), message: Message::CutLayer },
                Item { label: "Copy", shortcut: Some("Cmd+C"), message: Message::CopyLayer },
                Item { label: "Paste", shortcut: Some("Cmd+V"), message: Message::PasteLayer },
                Item {
                    label: "Duplicate",
                    shortcut: Some("Cmd+D"),
                    message: Message::DuplicateLayer,
                },
                Item {
                    label: "Select All",
                    shortcut: Some("Cmd+A"),
                    message: Message::SelectAllLayers,
                },
                Item {
                    label: "Deselect All",
                    shortcut: Some("Cmd+Shift+A"),
                    message: Message::DeselectAllLayers,
                },
            ],
        },
        // ---- Layer(MB-2 新設トップレベル — 中身は全て既存 Message) ----
        Menu {
            label: "Layer",
            items: vec![
                // 旧 "+ Layer" 箱ボタンの動詞。keymap に AddLayer の割当は
                // 無い(shortcut 出典ゼロ — 発明しない)。
                Item { label: "New Layer", shortcut: None, message: Message::AddLayer },
                // G1(裁定174)。MB-0 当時「独立した Layer トップレベルは
                // まだ無い」と注記して Edit に同居させていた分の引っ越し。
                Item { label: "Group", shortcut: Some("Cmd+G"), message: Message::GroupLayers },
                Item {
                    label: "Ungroup",
                    shortcut: Some("Cmd+Shift+G"),
                    message: Message::UngroupLayers,
                },
                // 裁定119(freeze 意図動詞)の UI 初露出。shortcut 未実装 —
                // 飾り表記は書かない。凍結ゲートの拒否理由は既存 status 経路
                // (`Shell::set_selected_groups_frozen`)で出る。
                Item { label: "Freeze", shortcut: None, message: Message::FreezeGroups },
                Item { label: "Unfreeze", shortcut: None, message: Message::UnfreezeGroups },
            ],
        },
        // ---- View(視界状態の動詞 — Document に乗らない表示専用) ----
        Menu {
            label: "View",
            items: vec![
                // 市松トグル(裁定141/163 — 発火元は Stage 下縁状態帯が正、
                // ここは第2の発見用入口。shortcut は未実装)。
                Item {
                    label: "Checkerboard",
                    shortcut: None,
                    message: Message::Stage(crate::stage::Message::ToggleCheckerboard),
                },
                // 観測視点リセット(裁定157)。Shift+F は `inspector_pointer_event`
                // の生 event 腕に実装済み — 実態と一致する表記のみ(S6)。
                Item {
                    label: "Reset View",
                    shortcut: Some("Shift+F"),
                    message: Message::Stage(crate::stage::Message::ResetToRenderCamera),
                },
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `TOP_LEVEL_LABELS`(q0_fence が読む正本)と `menus()` の実体がずれない。
    #[test]
    fn top_level_labels_match_the_menu_definitions() {
        let labels: Vec<&str> = menus().into_iter().map(|menu| menu.label).collect();
        assert_eq!(labels, TOP_LEVEL_LABELS);
    }
}
