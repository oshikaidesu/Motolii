//! モデルと `update` — この殻の唯一の可変状態。
//!
//! 中身は `ShellGateway` 1つだけである。座席も transcript も journal も
//! ゲートウェイの中に在り、この型からは**読みしか出せない**。
//! 「journal を通らずに製品状態へ着く道」を新しい殻でも作らない、という
//! 2026-08-18 の構造の強制をそのまま引き継いでいる
//! (柵: `tests/intent_gateway_fence.rs`)。
//!
//! ## 判断は移していない
//!
//! 未保存 guard の3択は `motolii_ui::blitz_shell::decide_unsaved` を**そのまま**
//! 呼ぶ。Export を始めてよいかは `ShellGateway::can_start_export`。どちらも
//! egui shell が見ているのと同じ関数で、host を替えても答えが変わらない
//! (=「意味は不変のまま iced へ移る」)。

use std::path::{Path, PathBuf};

use motolii_ui::blitz_shell::{
    decide_unsaved, IntentEvent, ShellGateway, ShellPrompts, ShellTranscript, StatusEvent,
    UiIntent, UnsavedDecision,
};

use crate::browser::{BrowserCard, BrowserPane};
use crate::message::Message;

/// `update` 1件のあとで窓に残る唯一の要求。
///
/// iced では「窓を閉じる」も副作用なので、殻は決めるだけで実行しない
/// (実行は host = `main.rs` の `iced::exit()`)。テストは窓を持たないまま
/// 同じ判断を読める。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Outcome {
    /// このまま窓は開いている。
    #[default]
    Stay,
    /// 窓を閉じてよい(未保存の始末は済んでいる)。
    Close,
}

/// iced ホストの shell 状態。
pub struct Shell {
    /// 製品状態へ触れる唯一の口。**この crate は他の道を持たない。**
    gateway: ShellGateway,
    /// 人に訊く口。窓は `NativePrompts`、テスト・CLI 駆動は `ScriptedPrompts`。
    /// **egui shell と同じ trait の同じ実装**である(`crate::prompts` の注記)。
    prompts: Box<dyn ShellPrompts>,
    /// Browser pane の view 状態(rail・選択・受け皿表示)と登録 folder の走査。
    /// Document の写しは**持たない**(読みは都度 `gateway` の snapshot を渡す)。
    browser: BrowserPane,
}

impl Shell {
    /// 座席なしで始める(スタート画面)。Browser の登録 folder は既定
    /// (repo の starter media)。
    pub fn new(prompts: impl ShellPrompts + 'static) -> Self {
        Self::with_browser(prompts, BrowserPane::default_shell())
    }

    /// 登録 folder を差し替えて始める(運転席テスト用。窓の経路は同じ)。
    pub fn with_browser_root(prompts: impl ShellPrompts + 'static, root: PathBuf) -> Self {
        Self::with_browser(prompts, BrowserPane::with_root(root))
    }

    fn with_browser(prompts: impl ShellPrompts + 'static, browser: BrowserPane) -> Self {
        let mut shell = Self {
            gateway: ShellGateway::new(ShellTranscript::default()),
            prompts: Box::new(prompts),
            browser,
        };
        // 走査時に作れなかったサムネイルの理由は、最初から帯経路に居る(F-09)。
        shell.drain_browser_notices();
        shell
    }

    /// Message 1件を受ける。**ここが Message → `UiIntent` の唯一の写像**である。
    ///
    /// dialog が答えなければ何も起きない(intent も記録されない)。
    /// 「起こそうとした行動」だけが journal に載る、という規律は egui shell と同じ。
    pub fn update(&mut self, message: Message) -> Outcome {
        match message {
            Message::NewProjectPressed => {
                // 座席を差し替える = いまの編集を捨てる。未保存なら先に訊く。
                if !self.clear_unsaved_or_stay() {
                    return Outcome::Stay;
                }
                let Some(path) = self.prompts.new_project_path() else {
                    return Outcome::Stay;
                };
                let _ = self.gateway.dispatch(UiIntent::NewProject { path });
            }
            Message::OpenProjectPressed => {
                if !self.clear_unsaved_or_stay() {
                    return Outcome::Stay;
                }
                let Some(path) = self.prompts.open_project_path() else {
                    return Outcome::Stay;
                };
                let _ = self.gateway.dispatch(UiIntent::OpenProject { path });
            }
            Message::SavePressed => {
                // 訊くことは無い(保存先は座席が知っている)。座席が無い時に
                // 何を言うかもゲートウェイの側に既に在る。
                let _ = self.gateway.dispatch(UiIntent::SaveProject);
            }
            Message::ExportPressed => {
                // 判断(座席あり・実行中なし)はボタンの enabled と同じ関数を見る。
                if !self.gateway.can_start_export() {
                    return Outcome::Stay;
                }
                let Some(project) = self.project_path() else {
                    return Outcome::Stay;
                };
                // 訊いて断られたら何も記録しない(訊いただけの操作は intent ではない)。
                let Some(output) = self.prompts.export_path(&project) else {
                    return Outcome::Stay;
                };
                let _ = self.gateway.dispatch(UiIntent::BeginExport { output });
            }
            Message::CancelExportPressed => {
                let _ = self.gateway.dispatch(UiIntent::CancelExport);
            }
            Message::FilesDropped(paths) => {
                let _ = self.gateway.dispatch(UiIntent::AdmitPaths { paths });
            }
            Message::CloseRequested => {
                // 窓を閉じるのも「未保存のまま座席を捨てる」操作。
                return if self.clear_unsaved_or_stay() {
                    Outcome::Close
                } else {
                    Outcome::Stay
                };
            }
            Message::ExportPolled => {
                // **intent ではない** — 走っている thread からの返事を受けるだけ。
                self.gateway.poll_export();
            }
            Message::BrowserRailChosen(rail) => {
                // pane 内の表示切替。Document に触れないので intent にならない
                // (view 系 intent は `blitz_shell/intent.rs` の将来枠)。
                self.browser.set_rail(rail);
            }
            Message::BrowserCardClicked(id) => {
                // 単クリック = 選択だけ(Q1)。報酬は selection tray と枠の強調。
                self.browser.select(&id);
            }
            Message::BrowserCardActivated(id) => {
                // 置いたカードは選ばれてもいる(egui 版と同じ)。
                self.browser.select(&id);
                match self.browser_place_path(&id) {
                    Some(path) => {
                        // **OS ドロップと同じ1本の合流点。** 成立も失敗も帯が言う
                        // (`admit_dropped_paths` の一言がそのまま出る)。
                        let _ = self.gateway.dispatch(UiIntent::AdmitPaths { paths: vec![path] });
                    }
                    None => {
                        // 拒否の報酬(Q3): 黙って無視しない。存在しない path を
                        // intent にもしない(egui 版 `place_request` と同じ判断)。
                        self.gateway.transcript().report(format!(
                            "browser: cannot place {id} — the file is missing or was moved"
                        ));
                    }
                }
            }
            Message::BrowserDropHover(hovering) => {
                // panel 内の受け皿表示だけ。取り込みは `FilesDropped` のまま。
                self.browser.set_drop_hover(hovering);
            }
        }
        // Document が進んでいれば、新しく載った image asset の縮小実体を用意する
        // (既に在る分は fs metadata を見るだけ)。作れなかった理由は帯経路へ。
        self.sync_browser();
        Outcome::Stay
    }

    /// Browser の読み(rail・選択・受け皿表示・サムネイル)。
    pub fn browser(&self) -> &BrowserPane {
        &self.browser
    }

    /// いまの rail の card 一覧(Document snapshot を写した投影)。
    pub fn browser_cards(&self) -> Vec<BrowserCard> {
        let seat = self.gateway.project();
        let snapshot = seat.map(|seat| seat.snapshot());
        let root = self.browser_project_root();
        self.browser
            .cards(snapshot.as_deref(), root.as_deref())
    }

    /// card 1枚が指す実ファイル(配置要求の解決)。
    fn browser_place_path(&self, id: &str) -> Option<PathBuf> {
        let snapshot = self.gateway.project().map(|seat| seat.snapshot());
        let root = self.browser_project_root();
        self.browser
            .place_path(id, snapshot.as_deref(), root.as_deref())
    }

    /// project root = document path の親(CLI export と同じ規約)。
    fn browser_project_root(&self) -> Option<PathBuf> {
        self.gateway
            .project()
            .and_then(|seat| seat.path().parent().map(Path::to_path_buf))
    }

    /// Document の進みに Browser のサムネイルを追いつかせ、理由を帯へ写す。
    fn sync_browser(&mut self) {
        if let Some(seat) = self.gateway.project() {
            let snapshot = seat.snapshot();
            let revision = seat.editor().revision();
            let root = seat.path().parent().map(Path::to_path_buf);
            self.browser
                .ensure_asset_thumbnails(&snapshot, root.as_deref(), revision);
        }
        self.drain_browser_notices();
    }

    /// pane が返した理由を status 帯(= `--status-log`)へ写す。
    /// stderr に書かない(2026-08-18 外部診断 F-09 の流儀)。
    fn drain_browser_notices(&mut self) {
        for notice in self.browser.take_notices() {
            self.gateway.transcript().report(notice);
        }
    }

    /// 未保存のまま座席を捨てる操作(New / Open / 窓を閉じる)の前に挟む。
    /// 続行してよければ `true`。判断は `decide_unsaved`、訊き手は
    /// `prompts.unsaved_choice` に居る。「保存して続行」で保存に失敗したら
    /// **続行しない**(帯に理由が出る)。
    ///
    /// 「保存して続行」の保存も利用者が決めた行動なので、`UiIntent::SaveProject`
    /// として記録される — 記録を見た側は New / Open の直前に保存が入ったことを
    /// 読み取れるし、replay も同じ順で保存する。
    fn clear_unsaved_or_stay(&mut self) -> bool {
        let Some(seat) = self.gateway.project() else {
            return true;
        };
        let path = seat.path().to_path_buf();
        let dirty = seat.is_dirty();
        let prompts = &mut self.prompts;
        match decide_unsaved(dirty, || prompts.unsaved_choice(&path)) {
            UnsavedDecision::Proceed => true,
            UnsavedDecision::Stay => false,
            UnsavedDecision::SaveThenProceed => self.gateway.dispatch(UiIntent::SaveProject),
        }
    }

    /// live project が座っているか。スタート画面を出すかどうかがこれで決まる。
    pub fn is_seated(&self) -> bool {
        self.gateway.is_seated()
    }

    /// 座っている project のパス。
    pub fn project_path(&self) -> Option<PathBuf> {
        self.gateway
            .project()
            .map(|seat| seat.path().to_path_buf())
    }

    /// 帯が名乗る project 名(パスが名前を持たなければパスそのもの)。
    pub fn project_name(&self) -> Option<String> {
        self.gateway.project().map(|seat| project_name(seat.path()))
    }

    /// 未保存の編集があるか。座席が無ければ「捨てる物が無い」= `false`。
    pub fn is_dirty(&self) -> bool {
        self.gateway.project().is_some_and(|seat| seat.is_dirty())
    }

    /// Export を始められるか。ボタンの enabled と `BeginExport` の門は同じ関数。
    pub fn can_start_export(&self) -> bool {
        self.gateway.can_start_export()
    }

    /// 書き出しが走っているか(帯が Exporting… と Cancel を出すかどうか)。
    pub fn export_running(&self) -> bool {
        self.gateway.export().is_some()
    }

    /// 走っている書き出しの経過秒。走っていなければ `None`。
    pub fn export_elapsed_seconds(&self) -> Option<u64> {
        self.gateway
            .export()
            .map(motolii_ui::export_seat::ExportRun::elapsed_seconds)
    }

    /// 既にキャンセルを頼んだか(Cancel の二度押しを止める)。
    pub fn export_cancel_requested(&self) -> bool {
        self.gateway
            .export()
            .is_some_and(motolii_ui::export_seat::ExportRun::cancel_requested)
    }

    /// status 帯が映す最新の一言。何も言われていなければ帯を出さない。
    pub fn latest_report(&self) -> Option<String> {
        self.gateway.transcript().latest()
    }

    /// 言われた全文(**結果**のログ)。replay の照合に使う。
    pub fn reports(&self) -> Vec<String> {
        self.gateway
            .transcript()
            .entries()
            .into_iter()
            .map(|event| event.text)
            .collect()
    }

    /// 原因のログ全文(順のまま)。`--intent-log` と replay がこれを読む。
    pub fn intents(&self) -> Vec<IntentEvent> {
        self.gateway.journal().entries()
    }

    /// journal に溜まっている行数。
    pub fn intent_count(&self) -> usize {
        self.gateway.journal().len()
    }

    /// 座席の writer 世代(replay の審判用)。座席が無ければ 0。
    pub fn revision(&self) -> u64 {
        self.gateway.revision()
    }

    /// 座席の Document に立っている track item の総数(replay の審判用)。
    pub fn track_item_count(&self) -> usize {
        self.gateway.track_item_count()
    }

    /// 既に `count` 行流した側が、その後に増えた分だけ受け取る
    /// ([`IntentLog`](crate::IntentLog) 用)。
    pub(crate) fn intents_since(&self, count: usize) -> Vec<IntentEvent> {
        self.gateway.journal().since(count)
    }

    /// 同じく結果のログの続き([`StatusLog`](crate::StatusLog) 用)。
    pub(crate) fn statuses_since(&self, count: usize) -> Vec<StatusEvent> {
        self.gateway.transcript().since(count)
    }

    /// transcript に溜まっている行数。
    pub(crate) fn status_count(&self) -> usize {
        self.gateway.transcript().len()
    }
}

/// 帯が名乗る名前。egui shell の status 帯と同じ決め方。
fn project_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}
