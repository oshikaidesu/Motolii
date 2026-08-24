
use iced::Task;

use motolii_store::{Asset, AssetStatus, Composition, Document, Intent};
use motolii_shell_state::layout::WorkspaceSnapshot;

use crate::tokens::Tokens;
use crate::{
    file_dialogs, metrics, Message, RecentFiles, Session, Shell,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct FrontState {
    session: Session,
    panes: WorkspaceSnapshot,
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_shell_state::focus::PaneKind;
    use motolii_shell_state::layout::LayoutNode;

    fn front_state() -> FrontState {
        FrontState {
            session: Session { playhead: 27, ..Session::default() },
            panes: WorkspaceSnapshot {
                open: false,
                root: LayoutNode::Leaf { kind: PaneKind::Timeline, hidden: false },
            },
        }
    }

    #[test]
    fn front_state_round_trips_through_json() {
        let state = front_state();
        let json = serde_json::to_vec(&state).expect("front state serialize");
        let restored: FrontState = serde_json::from_slice(&json).expect("front state deserialize");
        assert_eq!(restored.session.playhead, 27);
        assert_eq!(restored.panes.open, false);
    }

    #[test]
    fn restoring_front_state_keeps_the_playhead() {
        let state = front_state();
        let json = serde_json::to_vec(&state).expect("front state serialize");
        let restored: FrontState = serde_json::from_slice(&json).expect("front state deserialize");
        assert_eq!(restored.session.playhead, state.session.playhead);
    }
}

/* motolii-component
id = "shell.project_session_reopen"
kind = "semantic"
weight = "core_edit"
maps = []
entry = ["write_front_state", "restore_front_state"]
meaning = ["LastProjectPathRead", "OpenPathChosen", "QuitConfirmed"]
evaluation = ["front_state_round_trips_through_json", "restoring_front_state_keeps_the_playhead"]
render = ["TimelinePane", "pane_grid::PaneGrid"]
observable = ["front_state_round_trips_through_json"]
*/

/* motolii-component
id = "shell.find_missing_footage"
kind = "semantic"
weight = "core_edit"
maps = []
entry = ["FindMissingFootageRequested", "relink_asset_path"]
meaning = ["RelinkAssetPathChosen", "RelinkAsset"]
evaluation = ["sweep_asset_status", "relinking_a_missing_asset_makes_it_present"]
render = ["AssetStatus", "AssetListItem"]
observable = ["RelinkAssetPathChosen"]
*/

/* motolii-component
id = "shell.collect_project_files"
kind = "semantic"
weight = "core_edit"
maps = []
entry = ["CollectFilesRequested", "collect_project_files"]
meaning = ["CollectFilesPathChosen", "RelinkAsset"]
evaluation = ["collect_project_files"]
render = ["status", "AssetStatus"]
observable = ["CollectFilesPathChosen"]
*/

impl Shell {

    /// `AdmitPaths`/`FlushDrops` の共有入口。ファイルは従来どおり、フォルダは
    /// [`file_dialogs::expand_import_paths`] で supported media へ展開してから
    /// 既存の `admit` へ渡す。展開の I/O 失敗は黙って空扱いにせず status 帯へ
    /// 出す(M13) — `admit` 自身の probe/fingerprint 拒否とは別の失敗境界。
    pub(crate) fn admit_import_paths(&mut self, paths: Vec<std::path::PathBuf>) {
        match file_dialogs::expand_import_paths(paths) {
            Ok(paths) => self.admit(paths),
            Err(error) => {
                self.status = Some(format!("素材フォルダを展開できない: {error}"));
            }
        }
    }

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
                self.recent_files.remember(path.clone());
                self.write_front_state();
                Self::write_recent_files(&self.recent_files);
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
                self.restore_front_state(&path);
                self.recent_files.remember(path.clone());
                Self::write_recent_files(&self.recent_files);
                // C-1 波C「再起動で続きが開く」: 明示 Open でも sidecar を
                // 更新する(次回起動時にこの project を再度開く)。
                Self::write_last_project_path(&path);
                // 素材の在り処を**この1回だけ**解決する(2026-08-23)。
                // `Asset::status` は保存されない(環境の事実であって作品の内容
                // ではない)ので、開いた直後は全件 `Unchecked`。`canonicalize` は
                // syscall なので毎フレームは不可 — 「開く」という離散イベントが
                // 素材数ぶんで頭打ちになる自然な境界。
                self.sweep_asset_status();
            }
            Err(error) => self.status = Some(format!("開けない: {error}")),
        }
    }

    /// 全素材の在り処を1回だけ解決して `asset_status` へ溜める。
    ///
    /// `Asset::status` は `#[serde(skip)]` なので store から読み直すと必ず
    /// `Unchecked` に戻る。Browser のバッジはこの map を重ねて出す
    /// (`browser_pane::model::assets_with_status`)。
    ///
    /// **呼ぶのは離散イベントのときだけ** — project を開いた直後と、素材を
    /// 取り込んだ直後。`canonicalize` は syscall で、素材数ぶん走る。
    pub(crate) fn sweep_asset_status(&mut self) {
        let root = self
            .current_path
            .as_ref()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf());
        let assets = self.doc.view().assets().unwrap_or_default();
        self.asset_status = assets
            .into_iter()
            .map(|asset| {
                let status = asset.resolve_status(root.as_deref());
                (asset.id, status)
            })
            .collect();
        let missing = self
            .asset_status
            .values()
            .filter(|status| matches!(status, AssetStatus::Missing))
            .count();
        let unreadable = self
            .asset_status
            .values()
            .filter(|status| matches!(status, AssetStatus::Unreadable { .. }))
            .count();
        if missing > 0 || unreadable > 0 {
            self.status = Some(format!(
                "不足素材: {missing}件、読み取り不可: {unreadable}件"
            ));
        }
    }

    /// 欠損素材を1件ずつ、選択した実体へ繋ぎ直す。ID/content hash は
    /// `Intent::RelinkAsset` が保持し、shell は path 選択と環境状態の再評価だけを担う。
    pub(crate) fn relink_asset_path(
        &mut self,
        asset: motolii_store::AssetId,
        path: std::path::PathBuf,
    ) {
        let path = match std::fs::canonicalize(&path) {
            Ok(path) if path.is_file() => path,
            Ok(_) => {
                self.status = Some(format!("素材ファイルではありません: {}", path.display()));
                return;
            }
            Err(error) => {
                self.status = Some(format!("素材を読み取れない: {error}"));
                return;
            }
        };
        let project_root = self
            .current_path
            .as_ref()
            .and_then(|project| project.parent())
            .and_then(|root| std::fs::canonicalize(root).ok());
        let result = self.doc.apply(Intent::RelinkAsset {
            asset,
            path_absolute: Asset::normalize_path(&path.to_string_lossy()),
            project_root: project_root
                .as_deref()
                .map(|root| Asset::normalize_path(&root.to_string_lossy())),
        });
        match result {
            Ok(()) => {
                self.sweep_asset_status();
                self.status = Some(format!("素材を繋ぎ直しました: {}", path.display()));
            }
            Err(error) => self.status = Some(format!("素材を繋ぎ直せない: {error}")),
        }
    }

    /// 現在の作品を package 用の Document へ複製し、存在する file-backed 素材だけを
    /// package 隣の `media/` へ集める。**現在の `self.doc` は書き換えない**ため、
    /// Collect Files は Save As ではなく配布用の副産物を作る操作になる。
    pub(crate) fn collect_project_files(&mut self, package_path: std::path::PathBuf) {
        let parent = package_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| std::path::Path::new("."));
        if let Err(error) = std::fs::create_dir_all(parent) {
            self.status = Some(format!("収集先を作れない: {error}"));
            return;
        }
        let package_root = std::fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
        let media_dir = package_root.join("media");
        if let Err(error) = std::fs::create_dir_all(&media_dir) {
            self.status = Some(format!("収集先 media を作れない: {error}"));
            return;
        }

        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let temporary = std::env::temp_dir().join(format!(
            "motolii-collect-{}-{stamp}.rrd",
            std::process::id()
        ));
        if let Err(error) = self.doc.save(&temporary) {
            self.status = Some(format!("収集用の複製を作れない: {error}"));
            return;
        }
        let mut package = match Document::load(&temporary) {
            Ok(document) => document,
            Err(error) => {
                let _ = std::fs::remove_file(&temporary);
                self.status = Some(format!("収集用の複製を読めない: {error}"));
                return;
            }
        };
        let source_root = self
            .current_path
            .as_ref()
            .and_then(|project| project.parent())
            .and_then(|root| std::fs::canonicalize(root).ok());
        let assets = package.view().assets().unwrap_or_default();
        let mut collected = 0usize;
        let mut skipped = 0usize;
        for asset in assets {
            let resolved = match asset.resolve_status(source_root.as_deref()) {
                motolii_store::AssetStatus::Present { resolved_path } => {
                    std::path::PathBuf::from(resolved_path)
                }
                _ => {
                    skipped += 1;
                    continue;
                }
            };
            let file_name = asset
                .file_name
                .clone()
                .unwrap_or_else(|| format!("asset-{}.bin", asset.id));
            let destination = media_dir.join(format!("{}-{file_name}", asset.id));
            if let Err(error) = std::fs::copy(&resolved, &destination) {
                skipped += 1;
                self.status = Some(format!("素材を収集できない: {error}"));
                continue;
            }
            if let Err(error) = package.apply(Intent::RelinkAsset {
                asset: asset.id,
                path_absolute: Asset::normalize_path(&destination.to_string_lossy()),
                project_root: Some(Asset::normalize_path(&package_root.to_string_lossy())),
            }) {
                let _ = std::fs::remove_file(&destination);
                skipped += 1;
                self.status = Some(format!("収集した素材を繋ぎ直せない: {error}"));
                continue;
            }
            collected += 1;
        }
        let result = package.save(&package_path);
        let _ = std::fs::remove_file(&temporary);
        match result {
            Ok(()) => {
                self.status = Some(format!(
                    "素材を収集しました: {collected}件{} — {}",
                    if skipped == 0 {
                        String::new()
                    } else {
                        format!(", 未収集 {skipped}件")
                    },
                    package_path.display()
                ));
            }
            Err(error) => self.status = Some(format!("収集した project を保存できない: {error}")),
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

    fn recent_files_sidecar_path() -> Option<std::path::PathBuf> {
        Some(Self::user_settings_dir()?.join("recent_projects.json"))
    }

    fn front_state_sidecar_path(project: &std::path::Path) -> std::path::PathBuf {
        project.with_extension("motolii-state.json")
    }

    /// Session と pane 木を project ごとの sidecar へ保存する。Document の
    /// flatten/save とは別の経路で、undo や作品の内容へ混ぜない。
    pub(crate) fn write_front_state(&self) {
        let Some(project) = self.current_path.as_deref() else { return };
        let Some(panes) = self.panes.snapshot() else { return };
        let state = FrontState { session: self.session.clone(), panes };
        let Ok(json) = serde_json::to_vec_pretty(&state) else { return };
        let _ = std::fs::write(Self::front_state_sidecar_path(project), json);
    }

    /// project sidecar を読み戻す。壊れた/古い sidecar は project 本体を
    /// 開くことを妨げず、Session と既定 pane のまま続ける。
    pub(crate) fn restore_front_state(&mut self, project: &std::path::Path) {
        let Ok(bytes) = std::fs::read(Self::front_state_sidecar_path(project)) else { return };
        let Ok(state) = serde_json::from_slice::<FrontState>(&bytes) else { return };
        let FrontState { session, panes } = state;
        self.session = session;
        if panes.open != self.browser.is_open() {
            self.browser.update(crate::browser_pane::Message::ToggleBrowserPanel);
        }
        let _ = self.panes.restore(&panes);
    }

    pub(crate) fn read_recent_files() -> Option<RecentFiles> {
        let path = Self::recent_files_sidecar_path()?;
        let bytes = std::fs::read(path).ok()?;
        let mut recent = serde_json::from_slice::<RecentFiles>(&bytes).ok()?;
        recent.remove_missing();
        Some(recent)
    }

    pub(crate) fn write_recent_files(recent: &RecentFiles) {
        let Some(path) = Self::recent_files_sidecar_path() else { return };
        let Some(dir) = path.parent() else { return };
        let _ = std::fs::create_dir_all(dir);
        let Ok(json) = serde_json::to_vec_pretty(recent) else { return };
        let _ = std::fs::write(path, json);
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

use motolii_store::StoreView;

impl Shell {
    pub fn layer_count(&self) -> usize {
        self.doc.view().layers().len()
    }

    /// `StoreView` をそのまま返す(読むだけ)。**運転席の検分器具用**
    /// (`layer_count`/`composition` と同じ「運転席が見るための口」の1つ) —
    /// G1(裁定174)の ungroup 数値証明(`tests/suite/group_drive.rs`)が
    /// `resolve()`/`local_transform()` を直接叩くのに使う。`view` という名前は
    /// 既に `Shell::view() -> Element` が使っているので `store_view` にした。
    pub fn store_view(&self) -> StoreView<'_> {
        self.doc.view()
    }

    pub(crate) fn comp_duration(&self) -> i64 {
        self.doc
            .view()
            .composition()
            .ok()
            .flatten()
            .map(|c| c.duration_frames)
            .unwrap_or(0)
    }

    pub fn can_undo(&self) -> bool {
        self.doc.can_undo()
    }

    /// 直近の Save As が書いた path(未保存・Save a Copy 直後は前回のまま —
    /// `Message::SaveACopyRequested` doc 参照)。**運転席が見るための口**。
    pub fn current_path(&self) -> Option<&std::path::Path> {
        self.current_path.as_deref()
    }

    /// 未保存の変更があるか。**運転席が見るための口**(`Shell::is_dirty` の
    /// 公開版 — MB-1、`saved_revision` フィールド doc 参照)。
    pub fn is_project_dirty(&self) -> bool {
        self.is_dirty()
    }

    /// `Message::Redo` の可否。**運転席が見るための口**(`can_undo` と同じ形)。
    /// drag-to-scrub がキャンセル時に redo 空間を汚していないかを運転席から
    /// 確かめるのに使う(`inspector_drive.rs`)。
    pub fn can_redo(&self) -> bool {
        self.doc.can_redo()
    }

    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    /// 今の comp 設定。**screenshot 器具**が Stage の letterbox を組むのに使う
    /// (`timeline_pane::TimelinePane::new` も同じ `composition()` 呼び出しをする)。
    pub fn composition(&self) -> Option<Composition> {
        self.doc.view().composition().ok().flatten()
    }

    /// 今の Session(選択・再生位置)。**読むだけ** — `Session` 自体のフィールドは
    /// pub だが、書ける口は `Message` 経由の `update()` だけ。
    pub fn session(&self) -> &Session {
        &self.session
    }

    /// Settings 窓の台帳の読み口(S2、裁定182/188)。旧 `settings_panel_open()`
    /// (screenshot 器具専用の bool)は廃止 — Settings は OS 窓になり、単窓
    /// オフスクリーン合成の screenshot 器具の**対象外**(`screenshot.rs` 冒頭
    /// doc の明示コメント参照)。この口は窓台帳の検分
    /// (`tests/suite/window_drive.rs`/`q0_fence.rs`)が使う。
    pub fn settings_window(&self) -> Option<iced::window::Id> {
        self.settings_window
    }

    /// Export 窓の台帳の読み口(B09、第6波)。`settings_window()` と同型 —
    /// 運転席(`tests/suite/export_drive.rs`)が open/close の状態遷移を読む。
    pub fn export_window(&self) -> Option<iced::window::Id> {
        self.export_window
    }



    /// `Shell::update` から委譲される領域別 dispatch(2026-08-23 SP-1 レーン、
    /// `docs/reviews/2026-08-23-shell-split-plan.md` の続き)。**中身は無改変** —
    /// 元の巨大な `update()` match の腕をそのままここへ移しただけ(裁定どおり
    /// 移送と委譲だけ、バグ修正・整形は混ぜない)。渡された `message` がこの
    /// 領域の variant でなければ `Err(message)` で突き返す — `crate::dispatch_message`
    /// の chain-of-responsibility が次の領域dispatchへ渡す。**新しい Message 枝は
    /// ここへ腕を1本足すだけで済み、`lib.rs` は触らない**(MC-1 と同じ効能)。
    pub(crate) fn dispatch_document_io(&mut self, message: Message) -> Result<Task<Message>, Message> {
        let mut task = Task::none();
        match message {
            Message::Undo => {
                if !self.doc.undo() {
                    self.status = Some("これ以上戻せない".to_owned());
                }
            }
            Message::Redo => {
                if !self.doc.redo() {
                    self.status = Some("これ以上進めない".to_owned());
                }
            }
            Message::AdmitPaths(paths) => self.admit_import_paths(paths),
            Message::DropReceived(path) => self.pending_drops.push(path),
            Message::FlushDrops => {
                if !self.pending_drops.is_empty() {
                    let paths = std::mem::take(&mut self.pending_drops);
                    self.admit_import_paths(paths);
                }
            }
            Message::TokensFileChanged => {
                self.tokens = Tokens::load();
                metrics::record_tokens_reload();
            }
            Message::MainWindowOpened(id) => self.main_window = Some(id),
            Message::SettingsWindowOpened(id) => self.settings_window = Some(id),
            Message::ExportWindowOpened(id) => self.export_window = Some(id),
            Message::WindowClosed(id) => {
                if self.main_window == Some(id) {
                    // main 閉=アプリ終了(probe 注意点1)。daemon は放って
                    // おくと窓ゼロで生き続け、winit shell が compositor を
                    // `None` 化(device 破棄)する — zero-copy presenter
                    // (裁定170/171)の単一 device 前提を守るため、窓ゼロ状態
                    // そのものを作らない。
                    task = iced::exit();
                } else if self.settings_window == Some(id) {
                    // OS の閉じるボタン経路(トグル経由の close は
                    // `toggle_settings_window` が先行抹消済み — その場合
                    // ここへ来る時点で台帳は既に None なので何もしない)。
                    self.settings_window = None;
                } else if self.export_window == Some(id) {
                    // Export 窓の OS 閉じるボタン経路(`toggle_export_window`
                    // と同じ扱い、Settings 窓と同型)。
                    self.export_window = None;
                }
            }
            Message::NewProjectRequested => task = self.confirm_then(Message::NewProjectConfirmed),
            Message::NewProjectConfirmed(confirmed) => {
                if confirmed {
                    self.reset_document();
                }
            }
            Message::SaveRequested => {
                if let Some(path) = self.current_path.clone() {
                    self.perform_save_as(path);
                } else {
                    task = Task::perform(self.dialogs.pick_save_path(), Message::SaveAsPathChosen);
                }
            }
            Message::SaveAsRequested => {
                task = Task::perform(self.dialogs.pick_save_path(), Message::SaveAsPathChosen);
            }
            Message::SaveAsPathChosen(Some(path)) => self.perform_save_as(path),
            Message::SaveAsPathChosen(None) => {}
            Message::SaveACopyRequested => {
                task = Task::perform(self.dialogs.pick_save_path(), Message::SaveACopyPathChosen);
            }
            Message::SaveACopyPathChosen(Some(path)) => self.perform_save_a_copy(path),
            Message::SaveACopyPathChosen(None) => {}
            Message::OpenRequested => task = self.confirm_then_pick_open(),
            Message::OpenRecentRequested => self.recent_menu_open = !self.recent_menu_open,
            Message::RecentFileSelected(index) => {
                self.recent_menu_open = false;
                self.pending_recent_path = self.recent_files.path(index).map(std::path::Path::to_path_buf);
                if self.pending_recent_path.is_some() {
                    task = self.confirm_then(Message::RecentFileConfirmed);
                }
            }
            Message::RecentFileConfirmed(confirmed) => {
                if confirmed {
                    if let Some(path) = self.pending_recent_path.take() {
                        self.perform_open(path);
                    }
                } else {
                    self.pending_recent_path = None;
                }
            }
            Message::OpenPathChosen(Some(path)) => self.perform_open(path),
            Message::OpenPathChosen(None) => {}
            Message::RecentFilesLoaded(recent) => {
                if let Some(recent) = recent {
                    self.recent_files = recent;
                }
            }
            Message::FindMissingFootageRequested => {
                self.sweep_asset_status();
                let missing: Vec<_> = self
                    .asset_status
                    .iter()
                    .filter(|(_, status)| matches!(status, AssetStatus::Missing))
                    .map(|(asset, _)| *asset)
                    .collect();
                let mut missing = missing;
                missing.sort_unstable();
                if let Some(asset) = missing.first().copied() {
                    self.pending_relink_asset = Some(asset);
                    self.status = Some(format!(
                        "不足素材 {}件。繋ぎ直すファイルを選択してください",
                        missing.len()
                    ));
                    task = Task::perform(
                        self.dialogs.pick_open_path(),
                        Message::RelinkAssetPathChosen,
                    );
                } else {
                    self.status = Some("不足素材はありません".to_owned());
                }
            }
            Message::RelinkAssetPathChosen(Some(path)) => {
                if let Some(asset) = self.pending_relink_asset.take() {
                    self.relink_asset_path(asset, path);
                }
            }
            Message::RelinkAssetPathChosen(None) => {
                self.pending_relink_asset = None;
            }
            Message::CollectFilesRequested => {
                task = Task::perform(self.dialogs.pick_save_path(), Message::CollectFilesPathChosen);
            }
            Message::CollectFilesPathChosen(Some(path)) => self.collect_project_files(path),
            Message::CollectFilesPathChosen(None) => {}
            Message::ImportMediaRequested => {
                task = Task::perform(self.dialogs.pick_import_paths(), Message::AdmitPaths);
            }
            Message::QuitRequested => task = self.confirm_then(Message::QuitConfirmed),
            Message::QuitConfirmed(confirmed) => {
                if confirmed {
                    self.write_front_state();
                    self.dialogs.quit();
                }
            }
            Message::WindowCloseRequested(id) => {
                if self.main_window == Some(id) {
                    task = self.confirm_then(Message::WindowCloseConfirmed);
                }
                // Settings/Export 窓は `exit_on_close_request: true`(既定)の
                // ままなので、この Message は main 以外の id では実際には
                // 届かない(防御的に無視するだけ)。
            }
            Message::WindowCloseConfirmed(confirmed) => {
                if confirmed {
                    self.write_front_state();
                    task = iced::exit();
                }
                // false: 何もしない。main 窓は `exit_on_close_request: false`
                // のおかげでまだ閉じていない(OS へ Close を送っていない)ので、
                // 見た目どおり編集を続けられる。
            }
            Message::LastProjectPathRead(Some(path)) => {
                match Document::load(&path) {
                    Ok(doc) => {
                        self.saved_revision = doc.revision();
                        self.last_auto_saved = self.saved_revision.clone();
                        self.doc = doc;
                        self.current_path = Some(path.clone());
                        self.session = Session::default();
                        self.restore_front_state(&path);
                        self.recent_files.remember(path.clone());
                        Self::write_recent_files(&self.recent_files);
                        self.sweep_asset_status();
                    }
                    // 拒否は必ず出す(M13)。既定 Document のまま起動を続ける
                    // ── 黙って上書きしない/黙って落とさない、どちらも避ける。
                    Err(error) => {
                        self.status = Some(format!("前回のプロジェクトを開けない: {error}"));
                    }
                }
                if let Some(autosave_path) = Self::recoverable_autosave(&path) {
                    self.pending_recovery = Some(autosave_path);
                    task = Task::perform(
                        self.dialogs.confirm_recover_autosave(),
                        Message::AutoSaveRecoveryConfirmed,
                    );
                }
            }
            Message::LastProjectPathRead(None) => {}
            Message::AutoSaveRecoveryConfirmed(confirmed) => {
                if let Some(autosave_path) = self.pending_recovery.take() {
                    if confirmed {
                        self.perform_recover_autosave(autosave_path);
                    }
                }
            }
            Message::AutoSaveTick => self.run_auto_save(),
            other => return Err(other),
        }
        Ok(task)
    }
}
