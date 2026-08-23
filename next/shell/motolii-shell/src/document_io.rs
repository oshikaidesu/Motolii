
use iced::Task;

use motolii_store::{
    Composition, Document, Intent,
};

use crate::{
    file_dialogs, Message, Session, Shell,
};

impl Shell {

    /// 既定 comp だけを持つ、空の Document を組む(`new_with_dialogs`/
    /// `reset_document`(New Project、MB-1)が共有する)。空の Document には
    /// comp が無く Stage が何も出せない(M17 違反)ので、起動直後・New Project
    /// 直後のどちらも既定の comp を置く。**undo floor はここでは立てない**
    /// (呼び手が `saved_revision` を確定させたい時点を制御できるように —
    /// `new_with_dialogs`/`reset_document` の doc 参照)。
    pub(crate) fn default_document() -> Document {
        let mut doc = Document::new();
        let _ = doc.apply(Intent::SetComposition(Composition {
            width: 640,
            height: 360,
            fps: motolii_store::Fps::try_new(30, 1).expect("30fps"),
            duration_frames: 300,
            background: [0.0, 0.0, 0.0, 1.0],
        }));
        doc
    }

    // ---- File 束(MB-1、裁定176) ----

    /// 未保存の変更があるか。**`saved_revision` フィールドの doc が唯一の
    /// 判定根拠** — `Document::revision()`(履歴のみ、transient overlay は
    /// 含まない)を最後に保存した時点の値と比べるだけ。
    pub(crate) fn is_dirty(&self) -> bool {
        self.doc.revision() != self.saved_revision
    }

    /// dirty でなければ確認そのものを出さない `DialogFuture` を作る
    /// (`confirm_then`/`confirm_then_pick_open` 共通の下請け)。dirty で
    /// なければ [`file_dialogs::FileDialogs::confirm_discard`] を一切呼ばず
    /// `std::future::ready(true)` を返す ── `tests/suite/file_drive.rs` の
    /// 「dirty ではないのに確認ダイアログを呼んでいる」柵はこの分岐で保たれる
    /// (fake の呼び出し回数カウンタは `confirm_discard()` 自体を呼んだ時にしか
    /// 増えない)。
    pub(crate) fn confirm_discard_future(&self) -> file_dialogs::DialogFuture<bool> {
        if self.is_dirty() {
            self.dialogs.confirm_discard()
        } else {
            Box::pin(std::future::ready(true))
        }
    }

    /// New Project/Quit の dirty ガード(非同期版)。`wrap` で結果を包んだ
    /// `Message` を1つ運ぶ `Task` を返す ── ネイティブ dialog はモーダルなので
    /// 同期呼び出しは iced のイベントループを塞ぐ(`file_dialogs.rs` 冒頭 doc)。
    pub(crate) fn confirm_then(&self, wrap: fn(bool) -> Message) -> Task<Message> {
        Task::perform(self.confirm_discard_future(), wrap)
    }

    /// Open(id 1226)の dirty ガード+path 選択を1本の `Task` へ直列化する。
    /// dirty なら確認 → 確認できたら(または dirty でなければ即)path dialog を
    /// 開く、を1つの `async move { … }` に包む ── 確認をキャンセルしたら
    /// `pick_open_path` の future を一切 poll しない(await の早期 return)ので、
    /// OS dialog は実際には出ない。
    pub(crate) fn confirm_then_pick_open(&self) -> Task<Message> {
        let confirm = self.confirm_discard_future();
        let pick = self.dialogs.pick_open_path();
        Task::perform(
            async move {
                if !confirm.await {
                    return None;
                }
                pick.await
            },
            Message::OpenPathChosen,
        )
    }

    /// New Project(id 1221)本体。**Document を丸ごと差し替える**
    /// (`default_document` — 起動直後と同じ既定 comp)。`current_path`/
    /// `saved_revision` も新しい Document 基準へ揃えるので、直後は dirty では
    /// ない。`Session` も既定へ戻す(古い selection が存在しない layer を指す
    /// 事故を避ける — playhead/selection は前の project の物なので引き継がない)。
    pub(crate) fn reset_document(&mut self) {
        let mut doc = Self::default_document();
        doc.mark_undo_floor();
        self.saved_revision = doc.revision();
        self.last_auto_saved = self.saved_revision.clone();
        self.doc = doc;
        self.current_path = None;
        self.session = Session::default();
    }

    /// Save As(id 1225)。path 選択→保存(既存の汎用 persist 経路、
    /// `Document::save` = `flattened()` で履歴を畳んでから書く、`persist.rs`
    /// doc 参照)→成功したら `current_path`/`saved_revision` を更新して dirty を
    /// 解消する。キャンセル・書き込み失敗のどちらも `current_path` は不変。
    /// **`last_auto_saved` も同じ revision へ揃える** — 本体そのものが今
    /// この時点の内容で書けたので、次の tick が同じ revision のまま無駄な
    /// 自動保存を起こさないようにする(`AutoSaveConfig` doc の「dirty 判定」)。
    pub(crate) fn perform_save_as(&mut self, path: std::path::PathBuf) {
        match self.doc.save(&path) {
            Ok(()) => {
                self.current_path = Some(path);
                self.saved_revision = self.doc.revision();
                self.last_auto_saved = self.saved_revision.clone();
            }
            // 拒否は必ず出す。黙って消さない(M13 と同じ規律)。
            Err(error) => self.status = Some(format!("保存できない: {error}")),
        }
    }

    /// Save a Copy(id 1227)。Save As と同じ path 選択・同じ persist 経路だが、
    /// **`current_path`/`saved_revision` は据え置く**(`Message::
    /// SaveACopyRequested` doc「現 path 維持のまま別名へ書く」)——開いている
    /// project の身分(どの path と紐付いているか・dirty かどうか)は変わらない。
    pub(crate) fn perform_save_a_copy(&mut self, path: std::path::PathBuf) {
        if let Err(error) = self.doc.save(&path) {
            self.status = Some(format!("コピーを保存できない: {error}"));
        }
    }

    /// Open(id 1226)本体。既存の汎用 persist 経路(`Document::load`、
    /// `persist.rs` doc 参照 — 読込直後は `mark_undo_floor` 済みで戻せない)で
    /// 読み、成功したら Document を丸ごと差し替える(`reset_document` と同じ
    /// 「身分を新しい対象へ揃える」規律 ── `current_path`/`saved_revision`/
    /// `Session` の3点)。読み込みに失敗したら何も変えない(拒否は必ず出す、
    /// M13 規律)。
    pub(crate) fn perform_open(&mut self, path: std::path::PathBuf) {
        match Document::load(&path) {
            Ok(doc) => {
                self.saved_revision = doc.revision();
                self.last_auto_saved = self.saved_revision.clone();
                self.doc = doc;
                self.current_path = Some(path);
                self.session = Session::default();
            }
            Err(error) => self.status = Some(format!("開けない: {error}")),
        }
    }

    // ---- AUTOSAVE(SET+ B12 第2切片、shell 結線) ----

    /// `Message::AutoSaveTick` の受け口。**再生中・ドラッグ中はスキップ**
    /// (正典 §2 拘束5と同型 — `toggle_playback`/`apply_shuttle` と同じ
    /// `is_dragging()` 判定 + 実時間 transport/JKL シャトルのどちらかが
    /// 走っていれば見送る。ディスク I/O で1フレームでも巻き込むと再生の
    /// コマ落ちに直結するため、掴み・再生の最中に自動保存を割り込ませない)。
    /// tick そのものは `auto_save_enabled=false` の間は `subscription()` が
    /// 発行しないが、念のためここでも確認する(二重の柵、`is_dirty` 等の他の
    /// 判定と同じ「呼び口を絞るだけでなく受け口でも確認する」規律)。
    ///
    /// 実際の書き込みは `motolii_store::Document::auto_save` — dirty 判定
    /// (`self.doc.revision() == *since` なら `Ok(None)`)も保存先のローテーション
    /// (世代数超過分の削除)もそちら側の責務。ここは `current_path`/
    /// `last_auto_saved`/`auto_save_config` を渡すだけの glue。
    pub(crate) fn run_auto_save(&mut self) {
        if !self.auto_save_enabled {
            return;
        }
        if self.transport.is_running() || !self.shuttle.is_stopped() || self.is_dragging() {
            return;
        }
        match self.doc.auto_save(
            self.current_path.as_deref(),
            &self.last_auto_saved,
            &self.auto_save_config,
        ) {
            Ok(Some(path)) => {
                self.last_auto_saved = self.doc.revision();
                self.status = Some(format!("自動保存しました: {}", path.display()));
            }
            // `project_path` が無い(未保存の新規 project)か、前回の自動保存
            // から未編集(dirty でない)のどちらか — 黙って何もしない
            // (`Document::auto_save` doc「何もせず Ok(None)」、tick のたびに
            // status を出すと逆に無反応ゼロの趣旨に反する雑音になる)。
            Ok(None) => {}
            // 拒否は必ず出す(M13 と同じ規律)。
            Err(error) => self.status = Some(format!("自動保存できない: {error}")),
        }
    }

}

