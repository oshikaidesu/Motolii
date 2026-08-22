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
//! - **File** = MB-1(裁定176)の4動詞(New Project/Save As/Save a Copy/
//!   Quit)+ 第2波(2026-08-22、rfd 非同期化と同時発注)の Open(id 1226)・
//!   Import Media…(id 592「Import (media/file)」の第2の入口 ── 従来は
//!   OS drop のみだった)。
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
use motolii_store::Interp;

use crate::{timeline_pane, Message};

/// トップレベル4本の正本(左から順)。`menus()` と同じ出典 — q0_fence の
/// menubar 除外(bar click は「menu が開く」という widget 内部応答で、
/// `Message` を publish しない)がこの表を読んで bar 領域を特定する。
pub const TOP_LEVEL_LABELS: [&str; 4] = ["File", "Edit", "Layer", "View"];

/// header メニューバーの全定義。`Shell::header` が毎フレーム
/// `motolii_menubar::menu_bar(menus(), …)` へ渡す。
pub fn menus() -> Vec<Menu<Message>> {
    vec![
        // ---- File(MB-1、裁定176 — normal-map id 1221/1225/1227/1223 + 第2波
        // id 1226/592) ----
        Menu {
            label: "File",
            items: vec![
                Item {
                    label: "New Project",
                    shortcut: Some("Cmd+N"),
                    message: Message::NewProjectRequested,
                },
                // id 1226「Open Project」。entries(menu:shortcut:panel:pref)=
                // 2:0:0:0 — shortcut 出典ゼロなので発明しない(`KNOWN.md` の
                // Cmd+O 教訓どおり、既に Cmd+O は空いているが出典が無い限り
                // 埋めない)。
                Item { label: "Open…", shortcut: None, message: Message::OpenRequested },
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
                // 第6波(B09 書き出し束、map 538「Quick Export」消化)。
                // S6 併存 — Cmd+E は `resolve_navigation_key` に実装済み。
                Item {
                    label: "Export…",
                    shortcut: Some("Cmd+E"),
                    message: Message::Export(crate::export_pane::Message::ToggleExportDialog),
                },
                // id 592「Import (media/file)」の第2の入口(従来は OS drop
                // のみ)。entries は 4:1(shortcut 出典が1件あるが、実際の割当
                // キーの一次資料までは未確認 ── 誤った shortcut を発明する
                // より併記しない方を採る、飾り shortcut 禁止の規律)。
                Item {
                    label: "Import Media…",
                    shortcut: None,
                    message: Message::ImportMediaRequested,
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
                // 第6波(B15 補間束、`timeline::write` の `SetKeyInterp` 露出)。
                // `motolii_menubar::Item`/`Menu` は入れ子 submenu を持たない
                // (`motolii-menubar` crate doc「公開面は最小」)ので、「submenu
                // 級」= Edit メニュー末尾にひとまとまりの Item 群として置く。
                // 選択キーの track 全体へ一括適用(`write.rs::set_key_interp`
                // — 空選択・選択キー無しは黙って no-op、既存柵のまま)。
                // shortcut は未実装(飾り表記を書かない、S6 併存の規律どおり)。
                Item {
                    label: "Interpolation: Hold",
                    shortcut: None,
                    message: Message::Timeline(timeline_pane::Message::SetKeyInterp(Interp::Hold)),
                },
                Item {
                    label: "Interpolation: Linear",
                    shortcut: None,
                    message: Message::Timeline(timeline_pane::Message::SetKeyInterp(Interp::Linear)),
                },
                Item {
                    label: "Interpolation: Easy Ease",
                    shortcut: None,
                    message: Message::Timeline(timeline_pane::Message::SetKeyInterp(
                        timeline_pane::EASY_EASE,
                    )),
                },
                Item {
                    label: "Interpolation: Easy Ease In",
                    shortcut: None,
                    message: Message::Timeline(timeline_pane::Message::SetKeyInterp(
                        timeline_pane::EASY_EASE_IN,
                    )),
                },
                Item {
                    label: "Interpolation: Easy Ease Out",
                    shortcut: None,
                    message: Message::Timeline(timeline_pane::Message::SetKeyInterp(
                        timeline_pane::EASY_EASE_OUT,
                    )),
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
                // 第6波(B22 方眼シート束、`stage::sheets` 冒頭 doc「家(結線は
                // 次波)— 表示トグルの家は Viewer(状態帯 or View メニュー)」)。
                // 4項目とも `stage::SheetMessage::Toggle` — 状態
                // (`Shell::sheet_toggles`)はトグル済みかどうかを見せない
                // (`motolii_menubar::Item` に checkmark 面が無い、crate doc
                // 「公開面は最小」)ので、押すたびに反転するだけの動詞として
                // 露出する(Checkerboard と同格)。shortcut は未実装。
                Item {
                    label: "Grid",
                    shortcut: None,
                    message: Message::Sheet(crate::stage::SheetMessage::Toggle(
                        crate::stage::Sheet::Grid,
                    )),
                },
                Item {
                    label: "Thirds",
                    shortcut: None,
                    message: Message::Sheet(crate::stage::SheetMessage::Toggle(
                        crate::stage::Sheet::Thirds,
                    )),
                },
                Item {
                    label: "Golden Ratio",
                    shortcut: None,
                    message: Message::Sheet(crate::stage::SheetMessage::Toggle(
                        crate::stage::Sheet::GoldenRatio,
                    )),
                },
                Item {
                    label: "Safe Margins",
                    shortcut: None,
                    message: Message::Sheet(crate::stage::SheetMessage::Toggle(
                        crate::stage::Sheet::SafeMargins,
                    )),
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
