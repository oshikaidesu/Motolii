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

use iced::widget::canvas::Canvas;
use iced::widget::{button, column, container, row, text};
use iced::{Center, Element, Fill};

use crate::message::Message;
use crate::shell::Shell;
use crate::timeline::{TimelineMsg, TimelineProgram};
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
    let mut page = if shell.is_seated() {
        // Timeline pane が面の全部(M-3)。Stage / Browser / Inspector は M-2 / M-4。
        // 触れない枠(Inbox・M/S レール・帯高 resize)は**置かない** — Q0。
        column![seated(shell)].width(Fill).height(Fill)
    } else {
        column![container(start_screen()).center(Fill)]
            .width(Fill)
            .height(Fill)
    };

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
        text(TAGLINE),
        column![
            action_button(NEW_PROJECT, NEW_PROJECT_SHORTCUT, Message::NewProjectPressed),
            action_button(
                OPEN_PROJECT,
                OPEN_PROJECT_SHORTCUT,
                Message::OpenProjectPressed
            ),
        ]
        .spacing(8)
        .align_x(Center),
        text(DROP_HINT),
    ]
    .spacing(18)
    .align_x(Center)
    .into()
}

/// 名前と近道を1つのボタンに並べる。名前は独立した text なので、
/// 運転席は `"New Project…"` の完全一致で掴める。
fn action_button<'a>(name: &'a str, shortcut: &'a str, message: Message) -> Element<'a, Message> {
    button(row![text(name).size(16), text(shortcut).size(16)].spacing(12))
        .on_press(message)
        .into()
}

/// 座席が在るときの画面 — Timeline pane(iced canvas)。
///
/// canvas は窓の左上 (0,0) から立つ(帯は下)。運転席がポインタ座標を
/// `timeline::semantics::PaneGeometry` の同じ式で計算できるのはこの配置による。
fn seated(shell: &Shell) -> Element<'_, Message> {
    Canvas::new(TimelineProgram { shell })
        .width(Fill)
        .height(Fill)
        .into()
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
        band = band.push(
            button(text(UNDO).size(13)).on_press_maybe(
                (shell.undo_len() > 0).then_some(Message::Timeline(TimelineMsg::UndoPressed)),
            ),
        );
        band = band.push(
            button(text(REDO).size(13)).on_press_maybe(
                (shell.redo_len() > 0).then_some(Message::Timeline(TimelineMsg::RedoPressed)),
            ),
        );
    }

    // 書き出し面。実行中は経過秒と Cancel、そうでなければ Export。
    // **実行中に Export は出さない** = 二重起動の口が無い。
    if let Some(seconds) = shell.export_elapsed_seconds() {
        band = band.push(text(exporting_label(seconds)).size(13));
        let mut cancel = button(text(CANCEL_EXPORT).size(13));
        if !shell.export_cancel_requested() {
            cancel = cancel.on_press(Message::CancelExportPressed);
        }
        band = band.push(cancel);
    } else if shell.is_seated() {
        let mut export = button(text(EXPORT).size(13));
        if shell.can_start_export() {
            export = export.on_press(Message::ExportPressed);
        }
        band = band.push(export);
    }

    if let Some(latest) = latest {
        band = band.push(text(latest).size(13));
    }

    Some(container(band).padding(8).width(Fill).into())
}
