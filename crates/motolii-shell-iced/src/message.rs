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

use crate::browser::{BrowserRail, BrowserViewMode};
use crate::inspector_pane::InspectorEvent;
use crate::timeline::TimelineMsg;

/// この窓で起きうることの全部(M-1 + M-2 Stage + M-3 Timeline + M-4a Browser +
/// M-4b Inspector)。
///
/// `Eq` を降ろして `PartialEq` だけにしてある — [`InspectorEvent`] がスクラブ値
/// (`f64`)を運び、Timeline の Message も秒(f32)を運ぶ。intent(`UiIntent`)側は
/// µs の整数なので `Eq` のまま。
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
    /// **Browser のカード選択はここへ繋がない**(外部候補の選択は Document 外)。
    LayerSelected(u64),
    /// Inspector pane で押された事実。intent 化は `Shell::update` の写像1箇所。
    Inspector(InspectorEvent),
    /// Browser の source rail(All media / Project / Recent)を選んだ。
    /// pane 内の表示切替で、Document には触れない。
    BrowserRailChosen(BrowserRail),
    /// Browser のカードの単クリック = **選択だけ**(Q1: click=選択)。
    /// 選ぶたびに clip が増えたら人は選べない — 要求は出ない。
    ///
    /// `UiIntent` に view 系の変種はまだ無い(`blitz_shell/intent.rs` の将来枠)
    /// ので、選択は pane の中の状態に留まり journal には載らない。**Inspector の
    /// 選択(`LayerSelected`)へも繋がない** — Browser の選択は pane-local のまま
    /// (interaction-language: 外部候補の選択は Document 外)。
    BrowserCardClicked(String),
    /// Browser のカードのダブルクリック = 「この実ファイルを playhead へ置いてくれ」。
    /// `UiIntent::AdmitPaths` になる — **OS ドロップと同じ1本の合流点**で、
    /// 入口が増えても intent は1種類のまま(egui 版 `BrowserRequest::PlaceFile` と同じ)。
    BrowserCardActivated(String),
    /// 掴んだファイルが窓の上に来た(`true`)/離れた(`false`)。
    /// **panel 内の受け皿表示だけ**を切り替える。取り込みそのものは従来どおり
    /// [`Message::FilesDropped`] = 殻の `AdmitPaths` で、ここでは奪わない。
    BrowserDropHover(bool),
    /// browser-library.html:25 検索欄への入力(browser-revise レーン)。
    /// egui `BrowserPanel::set_query` と同じ意味関数を通すだけで、pane 内の
    /// 表示切替に留まる(Document には触れない)。
    BrowserQueryChanged(String),
    /// browser-library.html:103 filter chip(Video/Image/Audio)を押した。
    /// 押し直しは解除(html:339-343 のトグルと同じ)。
    BrowserKindFilterChosen(&'static str),
    /// browser-library.html:344-350 `Clear`。kind chip と検索語を両方戻す。
    BrowserFiltersCleared,
    /// browser-library.html:26 `#filter-toggle`。filter shelf の開閉。
    BrowserFilterShelfToggled,
    /// browser-library.html:94-97 view mode(Thumbnails/Grid/List)を選んだ。
    BrowserViewChosen(BrowserViewMode),
    /// Timeline pane で起きたこと(M-3)。canvas が intent まで解決して運び、
    /// `Shell::update` が `UiIntent` へ写して dispatch する。
    Timeline(TimelineMsg),
    /// 波形の生成座席を1歩進める合図。**intent ではない** — decode thread からの
    /// 返事を受けるだけ(`ExportPolled` と同じ型の解決)。
    WaveformPolled,
    /// Stage 島の評価済みフレームの席(`stage_frame_seat`)を1歩進める合図。
    /// **intent ではない** — render worker thread からの返事を受け取り、
    /// widget 木を再描画させるためだけの合図(`ExportPolled` / `WaveformPolled`
    /// と同じ型の解決)。実際の `request`/`poll` は `stage_island::Embed` の
    /// `prepare` が持つ(ここは redraw の刻みを起こすだけ)。
    StagePolled,
}
