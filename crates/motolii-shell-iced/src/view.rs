//! 絵。**モデルを書く道がここには無い**(`&Shell` しか受け取らない)。
//!
//! iced ではこれが型で保証される — egui shell の `show(&mut self, ui)` は
//! 描画の途中で製品状態を書けてしまい、それを禁じるのに走査フェンスが要った。
//!
//! ## 文言は egui 版と同じ
//!
//! 移行の途中で利用者の見る言葉が揺れないよう、スタート画面の5つの文字列は
//! `blitz_shell/app.rs` の `shows_welcome()` 側からそのまま写している。
//! 違うのは**組み方**だけ: egui は "New Project…   Cmd+N" を1つの label に
//! 詰めているが、ここは行(`row`)に割った。近道の表示は別の物なので別の
//! widget にする方が iced では素直で、運転席の selector(`&str` は**完全一致**)も
//! 「押したいボタンの名前」だけを名指しできる。

use iced::widget::{button, column, container, row, text};
use iced::{Center, Element, Fill};

use crate::message::Message;
use crate::shell::Shell;

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
/// 座った後に出る、いまの M-0 の正直な中身。
///
/// **編集面のふりをした空箱を置かない**(2026-08-12 の Q0「触れそうで触れない物は
/// 不合格」)。M-1 が Timeline / Browser / Inspector を持ってくるまで、ここは
/// 「何がまだ無いか」を言うだけの一行である。
pub const SEATED_PLACEHOLDER: &str = "Project is open. The editing surface arrives in M-1.";

/// 窓ぜんたい。
pub fn view(shell: &Shell) -> Element<'_, Message> {
    let body = if shell.is_seated() {
        seated()
    } else {
        start_screen()
    };

    let mut page = column![container(body).center(Fill)].width(Fill).height(Fill);

    // 帯は「言われたことがある時だけ」出る。空の帯を常設しない
    // (egui 版と同じ判断: `latest()` が `None` なら帯を出さない)。
    if let Some(latest) = shell.latest_report() {
        page = page.push(status_band(latest));
    }

    page.into()
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

/// 座席が在るときの画面(M-0 は一行だけ)。
fn seated<'a>() -> Element<'a, Message> {
    text(SEATED_PLACEHOLDER).into()
}

/// status 帯 — **窓が言ったことの最新1行**。全文は transcript に残っている。
fn status_band<'a>(latest: String) -> Element<'a, Message> {
    container(text(latest)).padding(8).width(Fill).into()
}
