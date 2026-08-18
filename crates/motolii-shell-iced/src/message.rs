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
//!
//! ## 1つだけ intent にならない物が居る
//!
//! [`Message::ExportPolled`] は**利用者の操作ではない** — 走っている書き出し
//! thread からの返事を受け取る合図である。だから journal には載らない。
//! egui shell が毎フレーム `poll_export()` を呼んでいるのと同じ物で、
//! iced では「いつ呼ぶか」が Message として明示されるだけ。

use std::path::PathBuf;

use crate::timeline::TimelineMsg;

/// この窓で起きうることの全部(M-1 の殻 + M-3 の Timeline)。
///
/// `Eq` を降ろして `PartialEq` だけにしてある — Timeline の Message は
/// 秒(f32)を運ぶ。intent(`UiIntent`)側は µs の整数なので `Eq` のまま。
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    /// New Project ボタン / `Cmd+N`。dialog が答えたら `UiIntent::NewProject` になる。
    NewProjectPressed,
    /// Open ボタン / `Cmd+O`。dialog が答えたら `UiIntent::OpenProject` になる。
    OpenProjectPressed,
    /// `Cmd+S`。そのまま `UiIntent::SaveProject`(訊くことが無い)。
    SavePressed,
    /// status 帯の Export。dialog が答えたら `UiIntent::BeginExport` になる。
    ExportPressed,
    /// status 帯の Cancel。そのまま `UiIntent::CancelExport`。
    CancelExportPressed,
    /// OS ドロップ。**1フレームぶんまとめて**運ぶので `UiIntent::AdmitPaths` と
    /// 粒度が揃う(winit はファイル1つにつき1事象を出す)。
    FilesDropped(Vec<PathBuf>),
    /// 窓を閉じたい。未保存なら3択を挟んでから
    /// [`Outcome::Close`](crate::Outcome::Close) になる。
    CloseRequested,
    /// 走っている書き出しの返事を受ける合図。**intent ではない。**
    ExportPolled,
    /// Timeline pane で起きたこと(M-3)。canvas が intent まで解決して運び、
    /// `Shell::update` が `UiIntent` へ写して dispatch する。
    Timeline(TimelineMsg),
    /// 波形の生成座席を1歩進める合図。**intent ではない** — decode thread からの
    /// 返事を受けるだけ(`ExportPolled` と同じ型の解決)。
    WaveformPolled,
}
