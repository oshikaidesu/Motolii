//! Window/Help メニューの**項目定義**(発注 2026-08-22、bundle B25/B06)。
//!
//! 実 `Message` は shell(次波)が持つ。この crate は `Message` 型を知らない —
//! 各項目の message はジェネリック引数([`WindowMenuMessages`]/
//! [`HelpMenuMessages`])で呼び手から受け取る。既存の [`Item`]/[`Menu`] API
//! (`lib.rs`)にそのまま載せる — 新しい公開型は増やさない。
//!
//! ## 出典と抽出方針
//!
//! 正本は `next/reference/normal-map.tsv` の `bundle` 列。B25(パネル可視性/
//! フォーカス束、91行 — 採用済9/採用予定82)・B06(ヘルプ/診断束、19行 —
//! 採用予定19)の**採用予定**行だけを対象に、「Motolii に対応する機構が
//! 実在する行」だけを消化した。機構が無い行(存在しないパネル種別・
//! 未実装のログ/ライセンス/更新チェック基盤等)は見送り — 発注の RETURN で
//! 行idを報告する(この crate にはledgerファイルを増やさない、発注書の
//! 「README/ボード/裁定/map 不触」の対象外だが同種の理由で最小限に留める)。
//!
//! ### Window(B25) — 消化6行 / 見送り75行
//!
//! 実在する pane は4つ(`next/shell/motolii-shell/src/pane_layout.rs`
//! `PaneKind`): Browser(開閉可能 — `build_configuration(open: bool, …)`)・
//! Inspector/Stage/Timeline(常設 — フォーカス移動のみ意味を持つ)。
//!
//! | 順 | label | 出典行id(意味) | action |
//! |---|---|---|---|
//! | 1 | Browser | 1525 "Open or close Project panel" | open/close |
//! | 2 | Inspector | 801 "Active Window: Inspector" | focus |
//! | 3 | Stage | 1317 "Active Window: Timeline Viewer"(観測カメラ/render出力の表示先) | focus |
//! | 4 | Timeline | 1316 "Active Window: Timeline" | focus |
//! | 5 | Cycle Panel | 1503 "Cycle to previous or next panel in active frame" | cycle |
//! | 6 | Close Panel | 1499 "Close active panel or all viewers of type"(1500と重複統合) | close |
//!
//! 並びは S0(パネル4枚 → パネル管理2動詞)。shortcut は全項目 `None` —
//! 実装済み割当が無い(S6、飾り禁止)。
//!
//! ### Help(B06) — 消化3行 / 見送り12行(+重複統合4行)
//!
//! | 順 | label | 出典行id(重複統合) |
//! |---|---|---|
//! | 1 | Documentation | 582(+572/579/580) |
//! | 2 | Community Forum | 585(+573) |
//! | 3 | Send Feedback… | 588 |
//!
//! 見送り: 570(Crash Reporter)・571(Product Improvement Program)・
//! 575/589(Check for Updates/Updates…)・577/584/586(診断ログ基盤なし)・
//! 581(License基盤なし)・583(Effect Reference — 構造化リファレンス未執筆)・
//! 587(Scripting Help — scripting API 未実装)・590/591(Welcome Screen —
//! `next/GOALS.md` M1「未」、実装され次第この crate の項目化を再検討)。

use crate::{Item, Menu};

/// Edit/Layer メニューの**項目定義**(発注 2026-08-22、bundle B33/B34)。
///
/// ## 出典と抽出方針
///
/// 正本は `next/reference/normal-map.tsv` の `bundle` 列。B33(クリップボード/
/// 履歴束、27行)・B34(グループ化束、7行)の**採用予定**行から、メニュー項目
/// として表現でき、かつ shell に実装済みの `Message` へ写せる行を freq降順で
/// 抽出する方針だったが、**新規追加はゼロ**だった — B33/B34 の 採用済 行
/// (Cut/Copy/Paste/Duplicate/Undo/Redo/Group/Ungroup、下記表)は既に
/// shell `menu.rs`(MB-0/G1)が結線済みで、採用予定 側の残り行は全て
/// 「実装が無い」か「採用済 行の重複記載」で見送りになったため。
///
/// この関数群の主目的は shell `menu.rs` に散っていた Edit/Layer の並び・
/// ラベル・shortcut表記を **この crate へ正本化する**こと(発注書 §2)。
/// 既存 [`Item`]/[`Menu`] API のみ使用、新しい公開型は増やさない。
///
/// ### Edit — 消化8行(既存 shell 定義をそのまま移送)
///
/// | 順 | label | shortcut | 出典行id | bundle |
/// |---|---|---|---|---|
/// | 1 | Undo | Cmd+Z | 437(437/467で shortcut 確認) | B33 |
/// | 2 | Redo | Cmd+Shift+Z | 435(467で shortcut 確認) | B33 |
/// | 3 | Cut | Cmd+X | 432 | B33 |
/// | 4 | Copy | Cmd+C | 429 | B33 |
/// | 5 | Paste | Cmd+V | 430 | B33 |
/// | 6 | Duplicate | Cmd+D | 434 | B33 |
/// | 7 | Select All | Cmd+A | 436 | B31(B33外 — 既存 shell 定義の移送) |
/// | 8 | Deselect All | Cmd+Shift+A | 433 | B31(同上) |
///
/// 並びは shell 既存定義の S0 慣習順(取り消し系 → クリップボード系 →
/// 選択系)をそのまま維持 — 移送であって並び替えではない。
///
/// #### B33 採用予定行の消化/見送り(今回の抽出対象・新規追加ゼロ)
///
/// | id | canonical | 判定 | 理由 |
/// |---|---|---|---|
/// | 438 | Edit Original | 見送り | 元アプリ起動連携の機構なし |
/// | 439 | Find | 見送り | プロジェクト内検索の機構なし |
/// | 440 | Paste Attributes | 見送り | プロパティ単体貼り付けの機構なし(Paste はレイヤー/クリップ全体のみ) |
/// | 448 | Copy with Property Links | 見送り | リンク付きコピー(expression/parented複製)の機構なし |
/// | 449 | Copy/Cut/Paste = Ctrl+C/X/V | 消化済に統合 | 429/430/432(採用済)の shortcut 確認記載の重複、新規項目化なし |
/// | 453 | Duplicate selected items | 消化済に統合 | 434(採用済 Duplicate)の重複記載 |
/// | 458 | History(操作履歴) | 見送り | 履歴パネル/可視化UIの機構なし |
/// | 459 | History(アンドゥ履歴サブメニュー) | 見送り | サブメニュー機構なし(同上) |
/// | 460 | Paste layers at current time | 見送り | 貼り付け位置指定オプションの分岐 `Message` 不在 |
/// | 461 | Paste Value(Color) | 見送り | プロパティ単体の値貼り付け機構なし |
/// | 462 | Reveal in Finder | 見送り | OS ファイルシステム連携の機構なし |
/// | 463 | Reveal in Finder(重複) | 見送り | 462と同一、機構なし |
/// | 464 | Revival Undo(Option+Cmd+Z) | 見送り | 既存 Redo(Cmd+Shift+Z)と機能重複だが shortcut 相異 — 未実装 keybind の追加になるため不採用(S6) |
/// | 465 | Show Duplicate Frames | 見送り | フレーム重複検出表示の機構なし |
/// | 466 | Undo(アンドゥ履歴を消去) | 見送り | 履歴クリア(Undo単体とは別操作)の機構なし |
/// | 831 | Edit in Adobe Audition | 見送り | サードパーティアプリ連携、対象外 |
///
/// ### Layer — 消化5行(既存 shell 定義をそのまま移送)
///
/// | 順 | label | shortcut | 出典 | bundle |
/// |---|---|---|---|---|
/// | 1 | New Layer | None | shortcut/normal-map出典ゼロ(旧 "+Layer" 箱ボタン) | — |
/// | 2 | Group | Cmd+G | 455/456/457(G1・裁定174) | B34 |
/// | 3 | Ungroup | Cmd+Shift+G | 468/469/470(同上) | B34 |
/// | 4 | Freeze | None | 裁定119(意図動詞、normal-map出典ゼロ) | — |
/// | 5 | Unfreeze | None | 同上 | — |
///
/// #### B34 採用予定行の消化/見送り(今回の抽出対象・新規追加ゼロ)
///
/// | id | canonical | 判定 | 理由 |
/// |---|---|---|---|
/// | 1334 | Group clips = Ctrl+G | 消化済に統合 | 既存 Group(455ほか採用済・Cmd+G)と同一動作の重複記載。理由欄も他NLE用語("compound clip")混同の疑いを指摘 — 新規項目化せず既存 Group で充足 |
///
/// shortcut 併記は S6(実装済みのみ)。`Freeze`/`Unfreeze`/`New Layer` は
/// normal-map 出典を持たない Motolii 固有動詞 — 存在しない shortcut を
/// 発明しない(既存 shell コメントの規律をそのまま継承)。
pub struct EditMenuMessages<M> {
    /// 取り消し(normal-map 437/467)。
    pub undo: M,
    /// やり直し(normal-map 435/467)。
    pub redo: M,
    /// 切り取り(normal-map 432)。
    pub cut: M,
    /// コピー(normal-map 429)。
    pub copy: M,
    /// 貼り付け(normal-map 430)。
    pub paste: M,
    /// 複製(normal-map 434)。
    pub duplicate: M,
    /// 全選択(normal-map 436、bundle B31)。
    pub select_all: M,
    /// 選択解除(normal-map 433、bundle B31)。
    pub deselect_all: M,
}

/// Edit メニュー本体を組む。並び・ラベルの正本はこの関数
/// (モジュール冒頭 doc の表と一致させること)。shell `menu.rs` の
/// `menus()` 内 Edit 定義はこの関数呼び出しへ差し替える(RETURN 参照)。
pub fn edit_menu<M>(items: EditMenuMessages<M>) -> Menu<M> {
    Menu {
        label: "Edit",
        items: vec![
            Item { label: "Undo", shortcut: Some("Cmd+Z"), message: items.undo },
            Item { label: "Redo", shortcut: Some("Cmd+Shift+Z"), message: items.redo },
            Item { label: "Cut", shortcut: Some("Cmd+X"), message: items.cut },
            Item { label: "Copy", shortcut: Some("Cmd+C"), message: items.copy },
            Item { label: "Paste", shortcut: Some("Cmd+V"), message: items.paste },
            Item { label: "Duplicate", shortcut: Some("Cmd+D"), message: items.duplicate },
            Item { label: "Select All", shortcut: Some("Cmd+A"), message: items.select_all },
            Item {
                label: "Deselect All",
                shortcut: Some("Cmd+Shift+A"),
                message: items.deselect_all,
            },
        ],
    }
}

/// [`layer_menu`] が必要とする message 一式。
pub struct LayerMenuMessages<M> {
    /// 新規レイヤー追加(normal-map出典ゼロ、旧 "+Layer" 箱ボタン)。
    pub new_layer: M,
    /// グループ化(normal-map 455/456/457、bundle B34・G1/裁定174)。
    pub group: M,
    /// グループ解除(normal-map 468/469/470、bundle B34・同上)。
    pub ungroup: M,
    /// 選択グループを凍結(裁定119、normal-map出典ゼロ)。
    pub freeze: M,
    /// 選択グループの凍結解除(同上)。
    pub unfreeze: M,
}

/// Layer メニュー本体を組む。shell `menu.rs` の `menus()` 内 Layer 定義は
/// この関数呼び出しへ差し替える(RETURN 参照)。
pub fn layer_menu<M>(items: LayerMenuMessages<M>) -> Menu<M> {
    Menu {
        label: "Layer",
        items: vec![
            Item { label: "New Layer", shortcut: None, message: items.new_layer },
            Item { label: "Group", shortcut: Some("Cmd+G"), message: items.group },
            Item { label: "Ungroup", shortcut: Some("Cmd+Shift+G"), message: items.ungroup },
            Item { label: "Freeze", shortcut: None, message: items.freeze },
            Item { label: "Unfreeze", shortcut: None, message: items.unfreeze },
        ],
    }
}

/// [`window_menu`] が必要とする message 一式。呼び手(shell)が各操作の
/// 具体 `Message` を埋める。
pub struct WindowMenuMessages<M> {
    /// Browser パネルの開閉トグル(normal-map 1525)。
    pub toggle_browser: M,
    /// Inspector パネルへフォーカス(normal-map 801)。
    pub focus_inspector: M,
    /// Stage パネルへフォーカス(normal-map 1317)。
    pub focus_stage: M,
    /// Timeline パネルへフォーカス(normal-map 1316)。
    pub focus_timeline: M,
    /// アクティブフレーム内で次のパネルへ巡回(normal-map 1503)。
    pub cycle_panel: M,
    /// アクティブパネルを閉じる(normal-map 1499/1500)。
    pub close_panel: M,
}

/// Window メニュー本体を組む。並び・ラベルの正本はこの関数(モジュール冒頭
/// doc の表と一致させること)。
pub fn window_menu<M>(items: WindowMenuMessages<M>) -> Menu<M> {
    Menu {
        label: "Window",
        items: vec![
            Item { label: "Browser", shortcut: None, message: items.toggle_browser },
            Item { label: "Inspector", shortcut: None, message: items.focus_inspector },
            Item { label: "Stage", shortcut: None, message: items.focus_stage },
            Item { label: "Timeline", shortcut: None, message: items.focus_timeline },
            Item { label: "Cycle Panel", shortcut: None, message: items.cycle_panel },
            Item { label: "Close Panel", shortcut: None, message: items.close_panel },
        ],
    }
}

/// [`help_menu`] が必要とする message 一式。
pub struct HelpMenuMessages<M> {
    /// ドキュメントを開く(normal-map 582、572/579/580重複統合)。
    pub open_documentation: M,
    /// コミュニティ/フォーラムを開く(normal-map 585、573重複統合)。
    pub open_community_forum: M,
    /// フィードバック送信(normal-map 588)。
    pub send_feedback: M,
}

/// Help メニュー本体を組む。
pub fn help_menu<M>(items: HelpMenuMessages<M>) -> Menu<M> {
    Menu {
        label: "Help",
        items: vec![
            Item { label: "Documentation", shortcut: None, message: items.open_documentation },
            Item {
                label: "Community Forum",
                shortcut: None,
                message: items.open_community_forum,
            },
            Item { label: "Send Feedback…", shortcut: None, message: items.send_feedback },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct FakeMessage(&'static str);

    fn edit_messages() -> EditMenuMessages<FakeMessage> {
        EditMenuMessages {
            undo: FakeMessage("undo"),
            redo: FakeMessage("redo"),
            cut: FakeMessage("cut"),
            copy: FakeMessage("copy"),
            paste: FakeMessage("paste"),
            duplicate: FakeMessage("duplicate"),
            select_all: FakeMessage("select_all"),
            deselect_all: FakeMessage("deselect_all"),
        }
    }

    fn layer_messages() -> LayerMenuMessages<FakeMessage> {
        LayerMenuMessages {
            new_layer: FakeMessage("new_layer"),
            group: FakeMessage("group"),
            ungroup: FakeMessage("ungroup"),
            freeze: FakeMessage("freeze"),
            unfreeze: FakeMessage("unfreeze"),
        }
    }

    fn window_messages() -> WindowMenuMessages<FakeMessage> {
        WindowMenuMessages {
            toggle_browser: FakeMessage("toggle_browser"),
            focus_inspector: FakeMessage("focus_inspector"),
            focus_stage: FakeMessage("focus_stage"),
            focus_timeline: FakeMessage("focus_timeline"),
            cycle_panel: FakeMessage("cycle_panel"),
            close_panel: FakeMessage("close_panel"),
        }
    }

    fn help_messages() -> HelpMenuMessages<FakeMessage> {
        HelpMenuMessages {
            open_documentation: FakeMessage("open_documentation"),
            open_community_forum: FakeMessage("open_community_forum"),
            send_feedback: FakeMessage("send_feedback"),
        }
    }

    /// Edit は8項目・並び固定(S0慣習順: 取り消し系 → クリップボード系 →
    /// 選択系)— shell `menu.rs` 既存定義からの移送を固定する。
    #[test]
    fn edit_menu_has_eight_items_in_declared_order() {
        let menu = edit_menu(edit_messages());
        assert_eq!(menu.label, "Edit");
        let labels: Vec<&str> = menu.items.iter().map(|item| item.label).collect();
        assert_eq!(
            labels,
            vec![
                "Undo",
                "Redo",
                "Cut",
                "Copy",
                "Paste",
                "Duplicate",
                "Select All",
                "Deselect All",
            ]
        );
    }

    /// Layer は5項目・並び固定(New Layer → Group/Ungroup → Freeze/Unfreeze)
    /// — shell `menu.rs` 既存定義からの移送を固定する。
    #[test]
    fn layer_menu_has_five_items_in_declared_order() {
        let menu = layer_menu(layer_messages());
        assert_eq!(menu.label, "Layer");
        let labels: Vec<&str> = menu.items.iter().map(|item| item.label).collect();
        assert_eq!(labels, vec!["New Layer", "Group", "Ungroup", "Freeze", "Unfreeze"]);
    }

    #[test]
    fn edit_menu_items_carry_the_expected_message() {
        let menu = edit_menu(edit_messages());
        let messages: Vec<&FakeMessage> = menu.items.iter().map(|item| &item.message).collect();
        assert_eq!(
            messages,
            vec![
                &FakeMessage("undo"),
                &FakeMessage("redo"),
                &FakeMessage("cut"),
                &FakeMessage("copy"),
                &FakeMessage("paste"),
                &FakeMessage("duplicate"),
                &FakeMessage("select_all"),
                &FakeMessage("deselect_all"),
            ]
        );
    }

    #[test]
    fn layer_menu_items_carry_the_expected_message() {
        let menu = layer_menu(layer_messages());
        let messages: Vec<&FakeMessage> = menu.items.iter().map(|item| &item.message).collect();
        assert_eq!(
            messages,
            vec![
                &FakeMessage("new_layer"),
                &FakeMessage("group"),
                &FakeMessage("ungroup"),
                &FakeMessage("freeze"),
                &FakeMessage("unfreeze"),
            ]
        );
    }

    /// shortcut 併記は実装済み割当だけ(S6) — Edit は8項目全てに shell
    /// 既存 shortcut がある。Layer は Group/Ungroup のみ shortcut を持ち、
    /// New Layer/Freeze/Unfreeze は出典ゼロにつき `None`(飾り禁止)。
    #[test]
    fn edit_menu_shortcuts_match_shell_existing_assignments() {
        let menu = edit_menu(edit_messages());
        let shortcuts: Vec<Option<&str>> =
            menu.items.iter().map(|item| item.shortcut).collect();
        assert_eq!(
            shortcuts,
            vec![
                Some("Cmd+Z"),
                Some("Cmd+Shift+Z"),
                Some("Cmd+X"),
                Some("Cmd+C"),
                Some("Cmd+V"),
                Some("Cmd+D"),
                Some("Cmd+A"),
                Some("Cmd+Shift+A"),
            ]
        );
    }

    #[test]
    fn layer_menu_shortcuts_are_none_except_group_and_ungroup() {
        let menu = layer_menu(layer_messages());
        let shortcuts: Vec<Option<&str>> =
            menu.items.iter().map(|item| item.shortcut).collect();
        assert_eq!(
            shortcuts,
            vec![None, Some("Cmd+G"), Some("Cmd+Shift+G"), None, None]
        );
    }

    /// 項目数と並び(S0慣習順: パネル4枚 → パネル管理2動詞)。
    #[test]
    fn window_menu_has_six_items_in_declared_order() {
        let menu = window_menu(window_messages());
        assert_eq!(menu.label, "Window");
        let labels: Vec<&str> = menu.items.iter().map(|item| item.label).collect();
        assert_eq!(
            labels,
            vec!["Browser", "Inspector", "Stage", "Timeline", "Cycle Panel", "Close Panel"]
        );
    }

    /// Help は3項目・並び固定。
    #[test]
    fn help_menu_has_three_items_in_declared_order() {
        let menu = help_menu(help_messages());
        assert_eq!(menu.label, "Help");
        let labels: Vec<&str> = menu.items.iter().map(|item| item.label).collect();
        assert_eq!(labels, vec!["Documentation", "Community Forum", "Send Feedback…"]);
    }

    /// 全項目が呼び手の渡した message をそのまま運ぶ(取り違え防止 — 型上
    /// `Option` でないことは既に `Item::message: M` の署名が保証するので、
    /// ここでは値の対応関係を確認する)。
    #[test]
    fn window_menu_items_carry_the_expected_message() {
        let menu = window_menu(window_messages());
        let messages: Vec<&FakeMessage> = menu.items.iter().map(|item| &item.message).collect();
        assert_eq!(
            messages,
            vec![
                &FakeMessage("toggle_browser"),
                &FakeMessage("focus_inspector"),
                &FakeMessage("focus_stage"),
                &FakeMessage("focus_timeline"),
                &FakeMessage("cycle_panel"),
                &FakeMessage("close_panel"),
            ]
        );
    }

    #[test]
    fn help_menu_items_carry_the_expected_message() {
        let menu = help_menu(help_messages());
        let messages: Vec<&FakeMessage> = menu.items.iter().map(|item| &item.message).collect();
        assert_eq!(
            messages,
            vec![
                &FakeMessage("open_documentation"),
                &FakeMessage("open_community_forum"),
                &FakeMessage("send_feedback"),
            ]
        );
    }

    /// shortcut 併記は実装済み割当だけ(S6) — 今回は実装済み割当がゼロなので
    /// 全項目 `None`。将来 shortcut を足す時はこのテストを更新すること
    /// (飾り shortcut を書いたら壊れるようにする、という意味の柵ではなく
    /// 現状を固定するテスト)。
    #[test]
    fn no_item_claims_a_shortcut_yet() {
        let window = window_menu(window_messages());
        let help = help_menu(help_messages());
        assert!(window.items.iter().all(|item| item.shortcut.is_none()));
        assert!(help.items.iter().all(|item| item.shortcut.is_none()));
    }
}
