//! 利用者が起こしたことの型。
//!
//! **`UiIntent` と1対1ではない。** `Message` は「押された」までを運び、
//! 「では何をするか」(dialog の答えを含めた `UiIntent`)は
//! [`Shell::update`](crate::Shell::update) が決める。iced の Message は
//! *入力の事実*、`UiIntent` は *起こす副作用* — 層が違うので、
//! 前者から後者への写像は `update` の中に1箇所だけ在る。
//!
//! この分け方は egui shell とも揃っている: あちらでも「ボタンが押された」は
//! `bool` で、`ShellGateway::dispatch` へ渡すのは dialog の答えが入った
//! `UiIntent` である。

/// スタート画面で起きうることの全部(M-0 の範囲)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message {
    /// New Project ボタン。dialog が答えたら `UiIntent::NewProject` になる。
    NewProjectPressed,
    /// Open ボタン。dialog が答えたら `UiIntent::OpenProject` になる。
    OpenProjectPressed,
}
