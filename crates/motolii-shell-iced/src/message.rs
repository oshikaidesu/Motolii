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

use crate::inspector_pane::InspectorEvent;

/// この窓で起きうることの全部(M-1 + M-2 Stage + M-4a Browser + M-4b Inspector)。
///
/// `Eq` を降ろしたのは [`InspectorEvent`] がスクラブ値(`f64`)を運ぶため。
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
    /// Stage 島からの報告(初期化・texture import・描画の失敗)。**intent ではない**
    /// — 利用者の操作でも文書の変化でもなく、窓が言うべき一言である。
    /// `Shell::update` が transcript(帯 / `--status-log`)へ写す。
    /// journal には載らない(replay は Stage の故障まで再現しない)。
    StageReported(Vec<String>),
    /// layer を1つ選んだ。`UiIntent::SelectLayer` になる。
    ///
    /// M-4b 時点でこれを出す widget はまだ無い(選択の面は Stage = M-2、
    /// Timeline = M-3 が持ってくる)。運転席(テスト)と後続 M がこの1点へ
    /// 合流する — 入口が増えても intent は1種類のまま(AdmitPaths と同じ型)。
    LayerSelected(u64),
    /// Inspector pane で押された事実。intent 化は `Shell::update` の写像1箇所。
    Inspector(InspectorEvent),
}
