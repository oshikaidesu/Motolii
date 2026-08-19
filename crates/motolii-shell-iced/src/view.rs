//! 絵。**モデルを書く道がここには無い**(`&Shell` しか受け取らない)。
//!
//! iced ではこれが型で保証される — egui shell の `show(&mut self, ui)` は
//! 描画の途中で製品状態を書けてしまい、それを禁じるのに走査フェンスが要った。
//!
//! ## 文言は egui 版と同じ
//!
//! 移行の途中で利用者の見る言葉が揺れないよう、スタート画面と status 帯の
//! 文字列は `blitz_shell/app.rs` からそのまま写している。違うのは**組み方**だけ:
//! egui は "New Project…   Cmd+N" を1つの label に詰めているが、ここは行(`row`)に
//! 割った。近道の表示は別の物なので別の widget にする方が iced では素直で、
//! 運転席の selector(`&str` は**完全一致**)も「押したいボタンの名前」だけを
//! 名指しできる。
//!
//! ## M-3 で帯に来た物
//!
//! Undo / Redo ボタン(編集面 = Timeline と一緒に来た)。`Cmd+Z` / `Shift+Cmd+Z`
//! と同じ `UiIntent::Undo` / `Redo` へ流れる — 経路も意味も1つ。
//!
//! ## Browser は別ファイル(browser-revise レーン、2026-08-19)
//!
//! Browser pane の絵は [`crate::browser_pane`] へ切り出した
//! (`inspector_pane.rs` と対称 — 他レーンとの衝突を減らす目的も兼ねる)。
//! ここに残るのは `browser_pane::browser_pane(shell)` の呼び出しだけ。

use iced::widget::canvas::Canvas;
use iced::widget::{button, column, container, row, rule, stack, text};
use iced::{Center, Element, Fill};

use crate::message::Message;
use crate::shell::Shell;
use crate::theme::style;
use crate::timeline::keys::interp_menu_items;
use crate::timeline::{TimelineMsg, TimelineProgram};
use crate::widgets::context_menu::{context_menu, MenuEvent};
use crate::window_input::window_input;

/// スタート画面の見出し。
pub const TITLE: &str = "Motolii";
/// 見出しの下の一行。
pub const TAGLINE: &str = "Make one 3\u{2013}5 minute music video.";
/// New ボタンの名前。**運転席がこの文字列で押す**(完全一致)。
pub const NEW_PROJECT: &str = "New Project\u{2026}";
/// New ボタンに添える近道。
pub const NEW_PROJECT_SHORTCUT: &str = "Cmd+N";
/// Open ボタンの名前。**運転席がこの文字列で押す**(完全一致)。
pub const OPEN_PROJECT: &str = "Open\u{2026}";
/// Open ボタンに添える近道。
pub const OPEN_PROJECT_SHORTCUT: &str = "Cmd+O";
/// スタート画面の末尾の一行。
pub const DROP_HINT: &str = "Then just drop video and audio into this window.";
/// Inspector pane の幅(既定配置の右)。
pub const INSPECTOR_PANE_W: f32 = 300.0;
// 上段(Browser|Stage|Inspector)対 下段 Timeline の高さ比は egui shell
// (`crates/motolii-ui/src/blitz_shell/app.rs::build_initial_tree`)の中央列を
// 正典にする: そこは Stage/Timeline を `insert_vertical_tile` へ明示 share を
// 与えず渡していて、`egui_tiles::Shares::get_share` は未設定を 1.0 として扱う
// ので実測は 1:1(50:50)。「だいたい6:4」という見積りではなく、egui 側に実在
// するこの数字を `seated()` の `Length::FillPortion(1)` 2本で採る。
/// 書き出しを始めるボタン。
pub const EXPORT: &str = "Export";
/// 走っている書き出しを止めるボタン。
pub const CANCEL_EXPORT: &str = "Cancel";
/// 帯の Undo ボタン(`Cmd+Z` と同じ入口 — 経路も意味も1つ)。
pub const UNDO: &str = "Undo";
/// 帯の Redo ボタン(`Shift+Cmd+Z` と同じ入口)。
pub const REDO: &str = "Redo";

/// 未保存の座席の名乗り方。egui shell の status 帯と**一字も違わない**。
pub fn unsaved_label(name: &str) -> String {
    format!("\u{25cf} {name} \u{2014} unsaved")
}

/// 走っている書き出しの名乗り方(進捗口が無い v0 の「まだ生きている」表示)。
pub fn exporting_label(seconds: u64) -> String {
    format!("Exporting\u{2026} {seconds}s")
}

/// 窓ぜんたい。
///
/// 一番外は [`window_input`] — 近道キー・OS ドロップ・閉じる要求はここで
/// [`Message`] になる。中身は「座席の有無で変わる本体」と「status 帯」の2段。
pub fn view(shell: &Shell) -> Element<'_, Message> {
    let body: Element<'_, Message> = if shell.is_seated() {
        seated(shell)
    } else {
        container(start_screen()).center(Fill).into()
    };

    let mut page = column![container(body).width(Fill).height(Fill)]
        .width(Fill)
        .height(Fill);

    // 帯は「座席が居るあいだ」と「言われたことがある時」に出る。空の帯を常設しない
    // (egui 版と同じ判断)。
    if let Some(band) = status_band(shell) {
        page = page.push(band);
    }

    window_input(page).into()
}

/// 座席が無いときの画面。
fn start_screen<'a>() -> Element<'a, Message> {
    column![
        text(TITLE).size(34),
        text(TAGLINE).style(style::text_secondary),
        column![
            action_button(
                NEW_PROJECT,
                NEW_PROJECT_SHORTCUT,
                Message::NewProjectPressed
            ),
            action_button(
                OPEN_PROJECT,
                OPEN_PROJECT_SHORTCUT,
                Message::OpenProjectPressed
            ),
        ]
        .spacing(8)
        .align_x(Center),
        text(DROP_HINT).style(style::text_muted),
    ]
    .spacing(18)
    .align_x(Center)
    .into()
}

/// 名前と近道を1つのボタンに並べる。名前は独立した text なので、
/// 運転席は `"New Project…"` の完全一致で掴める。
/// 近道は添え物なので muted — 名前と同格の顔をさせない。
fn action_button<'a>(name: &'a str, shortcut: &'a str, message: Message) -> Element<'a, Message> {
    button(
        row![
            text(name).size(16),
            text(shortcut).size(16).style(style::text_muted)
        ]
        .spacing(12),
    )
    .style(style::action)
    .on_press(message)
    .into()
}

/// 座席が在るときの画面 — 既定配置(ui-interaction-language §3):
/// 上段は 左 Browser(M-4a)/ 中央 Stage 島(M-2)/ 右 Inspector(M-4b)、
/// 下段は Timeline(M-3, canvas)。分割線は 1px([`style::separator`])。
///
/// 中央上は [`crate::stage_island::stage_island`]: Rerun `SpatialStage` の
/// 合成絵が shader widget として立ち、入力はブリッジ(調停つき)を通る。
/// 上段:下段の高さ比は 1:1(egui shell の中央列の実測 — [`TOP_ROW_HEIGHT`] の
/// docコメント参照)。
fn seated(shell: &Shell) -> Element<'_, Message> {
    let top = row![
        crate::browser_pane::browser_pane(shell),
        rule::vertical(1).style(style::separator),
        container(crate::stage_island::stage_island(shell))
            .width(Fill)
            .height(Fill),
        rule::vertical(1).style(style::separator),
        container(crate::inspector_pane::inspector(
            shell.inspector(),
            shell.editor_status(),
        ))
        .width(INSPECTOR_PANE_W)
        .height(Fill),
    ]
    .height(iced::Length::FillPortion(1));

    column![
        top,
        rule::horizontal(1).style(style::separator),
        container(timeline_pane(shell))
            .id(TIMELINE_PANE_ID)
            .height(iced::Length::FillPortion(1)),
    ]
    .width(Fill)
    .height(Fill)
    .into()
}

/// Timeline pane を包む `container` の id。**運転席がここへ実座標を訊く**——
/// 4面合成(統合第2弾)で Timeline canvas が窓の左上 (0,0) には立たなくなった
/// ので、運転席は `Simulator::find(TIMELINE_PANE_ID)` で実 bounds を引いてから
/// canvas ローカル座標(`timeline::semantics::PaneGeometry`)へ足す
/// (`drive_timeline.rs::geometry_offset`)。
pub const TIMELINE_PANE_ID: iced::advanced::widget::Id =
    iced::advanced::widget::Id::new("motolii-timeline-pane");

/// 下段 Timeline pane(M-3)。canvas は自分の受け持つ矩形の左上を原点に取る
/// ので、運転席のポインタ座標(`timeline::semantics::PaneGeometry`)はその
/// 矩形のローカル座標のまま成立する(窓座標への変換は [`TIMELINE_PANE_ID`])。
///
/// **2026-08-19 M-6**: 右クリックで開いたキーの補間メニュー
/// (`TimelinePane::key_menu`)が在れば、canvas の上に重ねる
/// (`widgets::context_menu` — M-4w で用意した既存の overlay 部品を初めて使う)。
fn timeline_pane(shell: &Shell) -> Element<'_, Message> {
    let canvas: Element<'_, Message> = Canvas::new(TimelineProgram { shell })
        .width(Fill)
        .height(Fill)
        .into();
    let Some(menu) = shell.timeline_pane().key_menu else {
        return canvas;
    };
    let items = interp_menu_items(menu.param);
    let (layer, param, at_us) = (
        menu.layer,
        menu.param,
        motolii_ui::blitz_shell::seconds_to_us(menu.at_seconds),
    );
    let overlay = context_menu(items, menu.anchor, move |event| match event {
        MenuEvent::Chosen(id) => match crate::timeline::keys::interp_for_menu_id(id) {
            Some(interp) => Message::Timeline(TimelineMsg::KeyInterpChosen {
                layer,
                param,
                at_us,
                interp,
            }),
            None => Message::Timeline(TimelineMsg::KeyMenuDismissed),
        },
        MenuEvent::Dismissed => Message::Timeline(TimelineMsg::KeyMenuDismissed),
    });
    stack(vec![canvas, overlay]).width(Fill).height(Fill).into()
}

/// status 帯 — **信用の可視化と、窓が言ったことの最新1行**。
///
/// 並びは egui shell と同じ: 保存状態 → 書き出し面 → 最新の一言。
/// 言うことも座席も無ければ帯そのものを出さない。
fn status_band(shell: &Shell) -> Option<Element<'_, Message>> {
    let latest = shell.latest_report();
    if !shell.is_seated() && latest.is_none() {
        return None;
    }

    let mut band = row![].spacing(12).align_y(Center);

    // 保存状態(保存済みなら project 名、未保存なら ● 付き)。
    if let Some(name) = shell.project_name() {
        let label = if shell.is_dirty() {
            unsaved_label(&name)
        } else {
            name
        };
        band = band.push(text(label).size(13));
    }

    // Undo / Redo(M-3)。`Cmd+Z` / `Shift+Cmd+Z` と同じ入口へ流れる。
    // 台帳が空の側は押せない(触れそうで触れない物にしない — 灰色は「無効」の意味)。
    if shell.is_seated() {
        band = band.push(button(text(UNDO).size(13)).on_press_maybe(
            (shell.undo_len() > 0).then_some(Message::Timeline(TimelineMsg::UndoPressed)),
        ));
        band = band.push(button(text(REDO).size(13)).on_press_maybe(
            (shell.redo_len() > 0).then_some(Message::Timeline(TimelineMsg::RedoPressed)),
        ));
    }

    // 書き出し面。実行中は経過秒と Cancel、そうでなければ Export。
    // **実行中に Export は出さない** = 二重起動の口が無い。
    if let Some(seconds) = shell.export_elapsed_seconds() {
        band = band.push(text(exporting_label(seconds)).size(13));
        let mut cancel = button(text(CANCEL_EXPORT).size(13)).style(style::action);
        if !shell.export_cancel_requested() {
            cancel = cancel.on_press(Message::CancelExportPressed);
        }
        band = band.push(cancel);
    } else if shell.is_seated() {
        let mut export = button(text(EXPORT).size(13)).style(style::action);
        if shell.can_start_export() {
            export = export.on_press(Message::ExportPressed);
        }
        band = band.push(export);
    }

    if let Some(latest) = latest {
        band = band.push(text(latest).size(13).style(style::text_secondary));
    }

    // 近道キーの提示(2026-08-19 iced 近道キー移植レーン)。**実際に効く
    // キーだけ**(Q0)— 表は `crate::shortcuts` が正本で、ここは読むだけ。
    // Timeline が立っている(= 座席がある)ときだけ出す。
    let mut column = column![].push(rule::horizontal(1).style(style::separator));
    if shell.is_seated() {
        column = column.push(
            container(text(crate::shortcuts::legend_line()).size(11).style(style::text_muted))
                .padding([2, 8]),
        );
    }

    // 帯は panel 面 + 上辺の区切り線。土台との段差は明度差と 1px の線で作る
    // (UI視覚言語「面」: 階層は小さな明度差と境界線で作る)。
    Some(
        column
            .push(
                container(band)
                    .padding(8)
                    .width(Fill)
                    .style(style::status_band),
            )
            .into(),
    )
}
