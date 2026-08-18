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

use crate::browser::BrowserRail;

/// この窓で起きうることの全部(M-1 + Browser pane = M-4a の範囲)。
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Browser の source rail(All media / Project / Recent)を選んだ。
    /// pane 内の表示切替で、Document には触れない。
    BrowserRailChosen(BrowserRail),
    /// Browser のカードの単クリック = **選択だけ**(Q1: click=選択)。
    /// 選ぶたびに clip が増えたら人は選べない — 要求は出ない。
    ///
    /// `UiIntent` に view 系の変種はまだ無い(`blitz_shell/intent.rs` の将来枠)
    /// ので、選択は pane の中の状態に留まり journal には載らない。
    BrowserCardClicked(String),
    /// Browser のカードのダブルクリック = 「この実ファイルを playhead へ置いてくれ」。
    /// `UiIntent::AdmitPaths` になる — **OS ドロップと同じ1本の合流点**で、
    /// 入口が増えても intent は1種類のまま(egui 版 `BrowserRequest::PlaceFile` と同じ)。
    BrowserCardActivated(String),
    /// 掴んだファイルが窓の上に来た(`true`)/離れた(`false`)。
    /// **panel 内の受け皿表示だけ**を切り替える。取り込みそのものは従来どおり
    /// [`Message::FilesDropped`] = 殻の `AdmitPaths` で、ここでは奪わない。
    BrowserDropHover(bool),
}
