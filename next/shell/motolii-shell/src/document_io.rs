
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
                self.current_path = Some(path.clone());
                self.saved_revision = self.doc.revision();
                self.last_auto_saved = self.saved_revision.clone();
                // 保存成功の合図(C-1 波C「保存成功の合図が無い」、P3 手順81/
                // Q3「沈黙禁止」)。失敗時は下の Err 枝が既に理由を書く —
                // 成功時だけ無言だった非対称を消す。裁定185(説明は下部バー)
                // どおり status 帯を使う、新しいダイアログ/トーストは作らない。
                self.status = Some(format!("保存しました: {}", path.display()));
                // C-1 波C「再起動で続きが開く」: 次回起動が黙って読み返せる
                // ように、成功した path を sidecar へ書く(ベストエフォート ──
                // 書けなくても保存自体は成功しているので失敗させない)。
                Self::write_last_project_path(&path);
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
        match self.doc.save(&path) {
            // 保存成功の合図(`perform_save_as` と同じ規律、Q3)。
            // `current_path`/`saved_revision`/sidecar は据え置く ── 開いている
            // project の身分はコピー保存では変わらない(doc 冒頭参照)。
            Ok(()) => self.status = Some(format!("コピーを保存しました: {}", path.display())),
            Err(error) => self.status = Some(format!("コピーを保存できない: {error}")),
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
                self.current_path = Some(path.clone());
                self.session = Session::default();
                // C-1 波C「再起動で続きが開く」: 明示 Open でも sidecar を
                // 更新する(次回起動時にこの project を再度開く)。
                Self::write_last_project_path(&path);
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

    // ---- User Settings 相当の sidecar(C-1 波C「再起動で続きが開く」) ----
    //
    // A06「置き場所が未設計」の答え: **Document には入れない**(裁定46/107、
    // 発注書の第一候補どおり)。`next/` に `dirs`/`ProjectDirs` 等の User
    // Settings 層がまだ無い(KNOWN.md 実測 grep 0件)ので、新しい依存を足す
    // 判断はこのレーンの裁量を超える ── 代わりに OS 標準のユーザー設定
    // ディレクトリを `std::env` だけで組み、1行(path 文字列)だけを書く
    // 最小の sidecar にした。**中身は path だけ**で Session(選択/playhead/
    // pane レイアウト)は一切入れていない ── A06 が「Document に入れるべき
    // でない」と別問題扱いした Session 永続化そのものは、置き場(User
    // Settings 層)自体を新設する設計判断が要るため、このレーンでは着手
    // せず未着手のまま残す(RETURN 参照)。

    /// OS ごとのユーザー設定ディレクトリ(`~/Library/Application Support/
    /// Motolii` / `$XDG_CONFIG_HOME or ~/.config/motolii` / `%APPDATA%\Motolii`)。
    /// 環境変数が引けなければ `None`(sidecar 機能を無効化するだけ ── 書けない/
    /// 読めない環境でも起動自体は既定 Document のまま続く)。
    fn user_settings_dir() -> Option<std::path::PathBuf> {
        #[cfg(target_os = "macos")]
        {
            let home = std::env::var_os("HOME")?;
            Some(std::path::PathBuf::from(home).join("Library/Application Support/Motolii"))
        }
        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var_os("APPDATA")?;
            Some(std::path::PathBuf::from(appdata).join("Motolii"))
        }
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
                return Some(std::path::PathBuf::from(xdg).join("motolii"));
            }
            let home = std::env::var_os("HOME")?;
            Some(std::path::PathBuf::from(home).join(".config/motolii"))
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows", unix)))]
        {
            None
        }
    }

    fn last_project_sidecar_path() -> Option<std::path::PathBuf> {
        Some(Self::user_settings_dir()?.join("last_project.txt"))
    }

    /// 直近に保存/開いた project の path を sidecar へ記録する
    /// (`perform_save_as`/`perform_open` から呼ぶ)。**ベストエフォート** ──
    /// 書けなくても呼び手の保存/読込自体は成功しているので、ここは何も
    /// 返さず失敗を握りつぶす(sidecar が書けないことは M13 の「拒否」では
    /// ない ── 明示保存の結果は変わらない、次回の自動再オープンが効かなく
    /// なるだけ)。
    pub(crate) fn write_last_project_path(path: &std::path::Path) {
        let Some(sidecar) = Self::last_project_sidecar_path() else {
            return;
        };
        if let Some(dir) = sidecar.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(&sidecar, path.to_string_lossy().as_bytes());
    }

    /// sidecar から前回の project path を読む。**存在確認込み**(path が
    /// 記録されていても、削除/移動されていれば `None` ── 呼び手
    /// (`Message::LastProjectPathRead`)はそのまま既定 Document で起動を
    /// 続ける)。`Shell::boot` の Task からのみ呼ばれる(`new`/
    /// `new_with_dialogs` からは呼ばない ── 試験がホームディレクトリの
    /// 実ファイルに左右されないようにするための境界、`Shell::boot` doc
    /// 参照)。
    pub(crate) fn read_last_project_path() -> Option<std::path::PathBuf> {
        let sidecar = Self::last_project_sidecar_path()?;
        let content = std::fs::read_to_string(sidecar).ok()?;
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return None;
        }
        let path = std::path::PathBuf::from(trimmed);
        path.exists().then_some(path)
    }

    // ---- クラッシュ復帰(C-1 波C「autosave が書くだけで読み返されない」) ----

    /// autosave 世代のうち、本体ファイルより新しい物があれば返す(mtime 比較)。
    /// **世代ファイルの命名規約(`{stem}.autosave-{seq}{ext}`)は
    /// `persist.rs` 内部で非公開** — ここでは同じ規約を再実装せず、
    /// `Document::auto_save_dir`(公開)が指すディレクトリの中で
    /// **最新更新の1ファイル**を素直に拾う(`.`始まりの tmp 残骸は除く、
    /// `persist.rs::save_atomic` の隠しファイル規約と同じ判定)。
    pub(crate) fn recoverable_autosave(project_path: &std::path::Path) -> Option<std::path::PathBuf> {
        let project_mtime = std::fs::metadata(project_path).ok()?.modified().ok()?;
        let dir = Document::auto_save_dir(project_path);
        let entries = std::fs::read_dir(&dir).ok()?;
        let mut latest: Option<(std::path::PathBuf, std::time::SystemTime)> = None;
        for entry in entries.flatten() {
            let path = entry.path();
            let is_hidden = path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with('.'))
                .unwrap_or(true);
            if is_hidden {
                continue;
            }
            let Ok(meta) = entry.metadata() else { continue };
            let Ok(mtime) = meta.modified() else { continue };
            if latest.as_ref().map(|(_, t)| mtime > *t).unwrap_or(true) {
                latest = Some((path, mtime));
            }
        }
        let (autosave_path, autosave_mtime) = latest?;
        (autosave_mtime > project_mtime).then_some(autosave_path)
    }

    /// 利用者が復元を承諾した後の本体(`Message::AutoSaveRecoveryConfirmed(true)`)。
    /// **黙って上書きしない**(4製品先例どおり)── `self.doc` だけを autosave
    /// の内容へ差し替え、`current_path`/`saved_revision` は元のまま据え置く。
    /// `saved_revision` を更新しないので `is_dirty()` は即座に真になり(未保存●
    /// が点く)、明示 Save(Cmd+S/File>Save)を押すまで本体ファイルは一切
    /// 書き換わらない ── 復元は「読み込むだけ」、確定は利用者の意思。
    pub(crate) fn perform_recover_autosave(&mut self, autosave_path: std::path::PathBuf) {
        match Document::load(&autosave_path) {
            Ok(doc) => {
                self.doc = doc;
                self.session = Session::default();
                self.status =
                    Some("自動保存から復元しました(保存すると確定します)".to_owned());
            }
            Err(error) => self.status = Some(format!("自動保存を復元できない: {error}")),
        }
    }

}

