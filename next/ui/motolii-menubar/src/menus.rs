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
