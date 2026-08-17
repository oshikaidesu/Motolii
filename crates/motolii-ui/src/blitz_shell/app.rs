//! Blitz パネルを 1 つの窓に合体させるための eframe アプリ。
//!
//! ドッキング（分割・タブ化・resize）は Blitz へ移植せず egui の責任とする裁定に従い、
//! ここでは `egui_tiles` の既定挙動をそのまま出す。操作感を「改善」しない。
//!
//! このファイルは器（どこに何の面が座るか）だけを決める。
//! ペインの中身・色・寸法は `super::pane::BlitzPane` が描く。
//! レイアウトの永続化もしない（毎回既定の並びで起動する）。
//!
//! Document の座席もここにある（`ProjectSeat`）。`--project` で開いた実プロジェクトの
//! `ProjectSession`（OS lock）と Timeline エディタ（唯一の writer を抱える）を app が
//! 抱え、Stage へは immutable snapshot だけを流す。**Timeline は live のとき
//! native エディタ**（`timeline_editor::TimelineEditor`）で、移動・トリム・選択・
//! Undo/Redo が writer を通る。座席無しの起動は従来どおり fixture 展示。

use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui_wgpu::RenderState;
use egui_tiles::{Container, Linear, LinearDir, Tile, TileId, Tiles, Tree, UiResponse};

use super::pane::{BlitzPane, PaneKind};
use crate::export_seat::{
    can_start_export, default_export_file_name, ExportFinish, ExportRequest, ExportRun,
};
use crate::timeline_editor::TimelineEditor;

/// `egui_tiles` のペイン描画をパネルへ委譲するだけの behavior。
///
/// `BlitzPane::show` が wgpu の `RenderState` を要求するので、
/// behavior が参照を持ち回る。live project があるときは Timeline pane だけ
/// Blitz ではなく native エディタが描く（Stage が egui 直描きなのと同じ形）。
struct BlitzShellBehavior<'a> {
    render_state: &'a RenderState,
    /// live の Timeline エディタ。`None` なら Timeline も fixture の Blitz 表示。
    editor: Option<&'a mut TimelineEditor>,
}

impl egui_tiles::Behavior<BlitzPane> for BlitzShellBehavior<'_> {
    fn pane_ui(&mut self, ui: &mut egui::Ui, _tile_id: TileId, pane: &mut BlitzPane) -> UiResponse {
        if pane.kind() == PaneKind::Timeline {
            if let Some(editor) = self.editor.as_deref_mut() {
                editor.show(ui);
                return UiResponse::None;
            }
        }
        // Stage が出すのは playhead 時刻の合成フレーム。時刻の正本はエディタ
        // （writer と同じ席）で、ここは読んで渡すだけ。
        if pane.kind() == PaneKind::Stage {
            if let Some(editor) = self.editor.as_deref() {
                pane.set_live_playhead(editor.playhead_seconds());
            }
        }
        pane.show(ui, self.render_state);
        // ペイン本体をドラッグ元にはしない（タブのドラッグだけで足りる）。
        UiResponse::None
    }

    fn tab_title_for_pane(&mut self, pane: &BlitzPane) -> egui::WidgetText {
        pane.title().into()
    }
}

/// live project の座席。開いた project の `ProjectSession`（OS 排他 lock）と、
/// Timeline エディタをひとまとめに持つ。**Document の唯一の writer はエディタの
/// 中に1つだけ座る**（single writer）。
///
/// pane 側へ出て行くのは `snapshot()` の immutable snapshot だけ。
/// 第二の Document 保持経路（cache / 読み直し）はここから先に作らない。
pub struct ProjectSeat {
    /// 開いている project のパス。**同じ project を開き直す時に lock を先に返す**
    /// 判断がこれを見る（`reseat_project`）。
    path: PathBuf,
    /// project identity の排他 lock。app が生きているあいだ握り続け
    /// （落とすと他プロセスが同じ project を開けてしまう）、保存（`save`）も
    /// この lock の中の `save_document` を通る（CLI `document_edit.rs` と同じ経路）。
    session: motolii_doc::ProjectSession,
    /// Timeline エディタ。編集は全部（キー入力も操作 API も）この中の writer を通る。
    editor: TimelineEditor,
    /// 最後に project ファイルへ書いた（開いた直後はファイルにある）内容の snapshot。
    /// dirty 判定はこれとの**内容比較**（`is_dirty`）。undo 台帳の深さや revision の
    /// 一致では判定しない — undo で保存内容へ戻れば clean、深さが偶然揃っても
    /// 内容が違えば dirty、が正しく出る。
    saved: Arc<motolii_doc::Document>,
    /// `is_dirty` の cache（判定した時の revision とその答え）。revision が
    /// 進んでいなければ Document 比較をやり直さない（status 帯が毎フレーム見るため）。
    dirty_cache: std::cell::Cell<Option<(u64, bool)>>,
}

impl ProjectSeat {
    /// 実プロジェクトを開いて座席を作る。経路は既存の `open_project_resolved`
    /// （= `ProjectSession::open` ＋ plugin 解決。`document_export.rs:18` /
    /// `timeline_widget_lab.rs:120` と同じ慣行）。
    ///
    /// 開けなければ `Err` — **fixture へ黙って fallback しない**。呼び手（`main.rs`）が
    /// 起動失敗として扱う。
    pub fn open(path: &Path) -> Result<Self, String> {
        let catalog = Arc::new(
            motolii_plugin::reference::reference_catalog()
                .map_err(|error| format!("plugin catalog を作れない: {error}"))?,
        );
        let opened = motolii_doc::open_project_resolved(
            path,
            &motolii_doc::ResourceLimits::production(),
            &catalog,
        )
        .map_err(|error| format!("project {} を開けない: {error}", path.display()))?;
        let writer = motolii_doc::DocumentWriter::new(opened.recovered.document, catalog).map_err(
            |error| {
                format!(
                    "project {} の DocumentWriter を作れない: {error}",
                    path.display()
                )
            },
        )?;
        let editor =
            TimelineEditor::new(writer).with_project_root(path.parent().map(Path::to_path_buf));
        // 開いた直後の snapshot がそのまま「ファイルにある内容」= clean の基準。
        let saved = Arc::clone(editor.document());
        Ok(Self {
            path: path.to_path_buf(),
            session: opened.session,
            // project root = document path の親(CLI export と同じ規約)。
            // soundtrack 再生の asset path 解決に使う。
            editor,
            saved,
            dirty_cache: std::cell::Cell::new(None),
        })
    }

    /// writer snapshot を project ファイルへ書き戻す（Cmd+S の後ろ）。
    ///
    /// 経路は既存の `ProjectSession::save_document`（CLI `document_edit.rs` の
    /// `with_writer` が編集後に通すのと同じ）で、**新しい保存経路を作らない**。
    /// journal への常時追記はここではしない（明示保存のみ）。
    pub fn save(&mut self) -> Result<(), String> {
        let snapshot = Arc::clone(self.editor.document());
        self.session
            .save_document(&snapshot, &motolii_doc::SaveOptions::default())
            .map_err(|error| format!("save {} failed: {error}", self.path.display()))?;
        self.saved = snapshot;
        self.dirty_cache.set(Some((self.editor.revision(), false)));
        Ok(())
    }

    /// 未保存の編集があるか。**保存済み内容との比較**で答える。
    ///
    /// revision や undo 台帳の深さの一致は根拠にしない — undo で保存時の内容へ
    /// 戻れば clean に戻り、保存点より下へ undo してから別の編集で深さだけ揃っても
    /// dirty のまま、が正しく出る。比較は revision が進んだ時だけやり直す。
    pub fn is_dirty(&self) -> bool {
        let revision = self.editor.revision();
        if let Some((seen, dirty)) = self.dirty_cache.get() {
            if seen == revision {
                return dirty;
            }
        }
        let current = self.editor.document();
        let dirty = !Arc::ptr_eq(current, &self.saved) && **current != *self.saved;
        self.dirty_cache.set(Some((revision, dirty)));
        dirty
    }

    /// 開いている project のパス。
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// pane へ流す immutable snapshot（エディタが writer から取った cached をそのまま配る）。
    pub fn snapshot(&self) -> Arc<motolii_doc::Document> {
        Arc::clone(self.editor.document())
    }

    /// Timeline エディタ（読み）。revision / snapshot の照合に使う。
    pub fn editor(&self) -> &TimelineEditor {
        &self.editor
    }

    /// Timeline エディタ（書き）。pane の描画と統合テストの操作 API がここを通る。
    /// writer 自体は外へ出さない — 編集は必ずエディタの操作として通る。
    pub fn editor_mut(&mut self) -> &mut TimelineEditor {
        &mut self.editor
    }
}

/// 新規 project を1つ作る（空コンポ + V1 トラック1本）。
///
/// 意味は CLI の `new_document`（`crates/motolii-cli/src/document_debug.rs:12`）
/// そのもので、**作ったものはそのまま `ProjectSeat::open` で開ける**
/// （= `--project` 起動と同じ状態になる）。既にあるファイルは踏まない。
///
/// dialog はここに無い。呼び手（`BlitzShellApp`）が場所を決めてから呼ぶ。
pub fn create_project_file(path: &Path) -> Result<(), String> {
    if path.exists() {
        return Err(format!("project already exists: {}", path.display()));
    }
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut document = motolii_doc::Document::new_current();
    // 2026-08-17決定: 新規projectの出力解像度既定は1920x1080(CLI `new_document`と同値。
    // 以後の変更は`Command::SetCompositionResolution`だけが書く)。
    document
        .composition
        .set_resolution(Some((1920, 1080)))
        .map_err(|error| error.to_string())?;
    // 受け皿のトラックが無いと clip を1本も置けない（`prepare_place_asset_clip`）。
    let track = document
        .track_ids
        .allocate("V1")
        .map_err(|error| error.to_string())?;
    document.tracks.push(motolii_doc::Track {
        id: track,
        items: vec![],
    });
    let mut session =
        motolii_doc::ProjectSession::acquire(path, &motolii_doc::ResourceLimits::production())
            .map_err(|error| error.to_string())?;
    session
        .save_document(&document, &motolii_doc::SaveOptions::default())
        .map_err(|error| error.to_string())
    // session はここで落ちて lock を返す。開くのは呼び手（`reseat_project`）。
}

/// 座席を差し替える。**旧 session の lock をいつ返すかがここの全部である。**
///
/// - 別の project … 新しい席を**先に開いてから**旧席を落とす。開けなくても
///   いま編集している席を失わない
/// - 同じ project … 先に旧席を落として lock を返してから開き直す
///   （そうしないと自分の lock に自分がぶつかる）
///
/// 開けなければ `Err(理由)` で、app は落とさない。dialog はここに無い。
pub fn reseat_project(current: &mut Option<ProjectSeat>, path: &Path) -> Result<(), String> {
    if current
        .as_ref()
        .is_some_and(|seat| same_project(seat.path(), path))
    {
        // 旧 session を先に落として lock を返す。
        *current = None;
    }
    let seat = ProjectSeat::open(path)?;
    // ここで初めて旧席（別 project の場合）が落ちる。
    *current = Some(seat);
    Ok(())
}

/// 同じ project を指しているか。symlink や `./` の違いで別物に見えないよう、
/// 取れるなら canonical path で比べる。
fn same_project(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

/// OS から窓へ落ちてきた path 群を受ける。**返すのは一言の status。**
///
/// project が開いていないときは Document が無いので取り込めない。黙って
/// 捨てず、**先に project を作る／開く**と案内する（v0。ドロップから
/// 新規 project を作る意味はまだ決めていない）。
pub fn admit_dropped_paths(seat: Option<&mut ProjectSeat>, paths: &[PathBuf]) -> String {
    let Some(seat) = seat else {
        return "open a project first — Cmd+N to create one, Cmd+O to open one".to_owned();
    };
    if paths.is_empty() {
        return "nothing dropped".to_owned();
    }
    seat.editor_mut().import_dropped_media(paths).summary()
}

/// 未保存 guard の3択。dialog（`prompt_unsaved_choice`）が返すのはこれだけで、
/// 何が起きるかは `decide_unsaved` が決める。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsavedChoice {
    /// 保存してから続行。
    Save,
    /// 編集を捨てて続行。
    Discard,
    /// やめる（いまの座席に留まる）。
    Cancel,
}

/// 未保存 guard の判断結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsavedDecision {
    /// そのまま続行してよい（clean だった、または破棄を選んだ）。
    Proceed,
    /// 先に保存してから続行する。保存に失敗したら続行しない（呼び手の責任）。
    SaveThenProceed,
    /// 続行しない。
    Stay,
}

/// 未保存のまま座席を捨てる操作（Cmd+O / Cmd+N / 窓を閉じる）の判断。
///
/// **dialog はここに無い**（`blitz_shell_file_entry.rs` と同じ形 — テストも製品も
/// この関数を通り、製品は `choose` に `prompt_unsaved_choice` を差す）。
/// clean なら `choose` を呼びもせず続行する。
pub fn decide_unsaved(dirty: bool, choose: impl FnOnce() -> UnsavedChoice) -> UnsavedDecision {
    if !dirty {
        return UnsavedDecision::Proceed;
    }
    match choose() {
        UnsavedChoice::Save => UnsavedDecision::SaveThenProceed,
        UnsavedChoice::Discard => UnsavedDecision::Proceed,
        UnsavedChoice::Cancel => UnsavedDecision::Stay,
    }
}

/// 未保存の3択を人に訊く。**dialog はここだけ**で、判断は `decide_unsaved` にある
/// （`prompt_new_project_path` / `prompt_open_project_path` と同じ集約）。
/// 閉じられた・知らない答えは Cancel 扱い（勝手に捨てない側へ倒す）。
fn prompt_unsaved_choice(project: &Path) -> UnsavedChoice {
    let name = project
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| project.display().to_string());
    let result = rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Warning)
        .set_title("Unsaved changes")
        .set_description(format!("{name} has unsaved changes."))
        .set_buttons(rfd::MessageButtons::YesNoCancelCustom(
            "Save".to_owned(),
            "Discard".to_owned(),
            "Cancel".to_owned(),
        ))
        .show();
    match result {
        rfd::MessageDialogResult::Custom(label) if label == "Save" => UnsavedChoice::Save,
        rfd::MessageDialogResult::Custom(label) if label == "Discard" => UnsavedChoice::Discard,
        rfd::MessageDialogResult::Yes => UnsavedChoice::Save,
        rfd::MessageDialogResult::No => UnsavedChoice::Discard,
        _ => UnsavedChoice::Cancel,
    }
}

/// Blitz パネルを合体表示するアプリ本体。
pub struct BlitzShellApp {
    /// `blitz_net::Provider` は Tokio reactor を要求し、無いと panic する。
    /// reactor を保証するのはこのアプリの責任。`update()` の先頭で enter する。
    runtime: tokio::runtime::Runtime,
    /// wgpu バックエンド前提（eframe は `features = ["wgpu"]`、glow ではない）。
    render_state: RenderState,
    tree: Tree<BlitzPane>,
    /// live project の座席。`None` は fixture 展示（従来の開発動線）。
    project: Option<ProjectSeat>,
    /// Stage へ最後に配った snapshot の revision。エディタの編集で進んでいたら
    /// 次のフレームで新しい snapshot を配り直す（Stage の描画コードは触らない）。
    seated_revision: u64,
    /// 窓ぜんたいに関わる一言（ドロップ・New・Open の結果）。空なら帯を出さない。
    /// Timeline の中の出来事は従来どおりエディタ自身の status が言う。
    status: String,
    /// 実行中の書き出し（高々1件。実行中は Export ボタンが disabled = 二重起動防止）。
    /// UI thread は毎フレーム `try_finish` を poll するだけで、書き出しは別 thread。
    export: Option<ExportRun>,
}

impl BlitzShellApp {
    /// `eframe::CreationContext` から作る（fixture 展示。従来どおり）。
    ///
    /// # Panics
    /// - wgpu の `RenderState` が取れない場合（glow バックエンドで起動された等）
    /// - Tokio runtime を作れない場合
    pub(crate) fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::with_seat(cc, None)
    }

    /// 座席ごと作る。`Some(seat)` なら Timeline / Stage は fixture ではなく
    /// その project の Document（writer の snapshot）を映す。
    ///
    /// # Panics
    /// `new` と同じ。
    pub(crate) fn with_seat(
        cc: &eframe::CreationContext<'_>,
        project: Option<ProjectSeat>,
    ) -> Self {
        let render_state = cc
            .wgpu_render_state
            .clone()
            .expect("BlitzShellApp は wgpu バックエンドを要求する（eframe features = [\"wgpu\"]）");

        // 記号(◆ ◇ ▶ ← ↔ →)が豆腐にならないよう、既定fontの後ろにHackを連ねる。
        // 新しいフォントは足していない。詳細は `egui_fonts`。
        crate::egui_fonts::install_symbol_fallback(&cc.egui_ctx);

        let runtime = tokio::runtime::Runtime::new()
            .expect("blitz_net::Provider 用の Tokio runtime を作れなかった");

        // snapshot はここで1度だけ取り、Stage が writer の出した `Arc` そのものを持つ。
        let snapshot = project.as_ref().map(ProjectSeat::snapshot);
        let seated_revision = project
            .as_ref()
            .map(|seat| seat.editor().revision())
            .unwrap_or(0);
        Self {
            runtime,
            render_state,
            tree: build_initial_tree(snapshot.as_ref()),
            project,
            seated_revision,
            status: String::new(),
            export: None,
        }
    }

    /// 座席の参照。
    pub fn project(&self) -> Option<&ProjectSeat> {
        self.project.as_ref()
    }

    /// 窓の一言（読み）。
    pub fn status(&self) -> &str {
        &self.status
    }

    /// 座席を差し替えて、Stage を新しい Document へ座らせ直す。
    ///
    /// 座り直しは既存の経路をそのまま使う: 開けたら `seat_stage_documents`
    /// （編集のたびに Stage へ配り直しているのと同じ関数）、席を失ったときだけ
    /// `build_initial_tree(None)` で fixture の並びへ戻す。
    fn reseat(&mut self, path: &Path) {
        match reseat_project(&mut self.project, path) {
            Ok(()) => {
                let seat = self.project.as_ref().expect("reseat_project seated one");
                let snapshot = seat.snapshot();
                self.seated_revision = seat.editor().revision();
                seat_stage_documents(&mut self.tree, &snapshot);
                self.status = format!("opened {}", path.display());
            }
            Err(error) => {
                // 同じ project を開き直そうとして落ちた時だけ席が空く。
                // 黙って fixture に見えないよう、並びも status も合わせる。
                if self.project.is_none() {
                    self.tree = build_initial_tree(None);
                    self.seated_revision = 0;
                }
                self.status = error;
            }
        }
    }

    /// 未保存のまま座席を捨てる操作（Cmd+O / Cmd+N / 窓を閉じる）の前に挟む。
    /// 続行してよければ `true`。判断は `decide_unsaved`、dialog は
    /// `prompt_unsaved_choice` に居る。「保存して続行」で保存に失敗したら
    /// **続行しない**（status に理由が出る）。
    fn clear_unsaved_or_stay(&mut self) -> bool {
        let Some(seat) = self.project.as_mut() else {
            return true;
        };
        let path = seat.path().to_path_buf();
        match decide_unsaved(seat.is_dirty(), || prompt_unsaved_choice(&path)) {
            UnsavedDecision::Proceed => true,
            UnsavedDecision::Stay => false,
            UnsavedDecision::SaveThenProceed => match seat.save() {
                Ok(()) => true,
                Err(error) => {
                    self.status = error;
                    false
                }
            },
        }
    }

    /// ドロップ・New・Open・Save を受ける。**描く前に1回だけ**通す。
    fn handle_file_entry(&mut self, ctx: &egui::Context) {
        // ---- New / Open / Save。dialog は main thread を止めて開く（eframe 慣行） ----
        let (new_project, open_project, save_project) = ctx.input_mut(|input| {
            (
                input.consume_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::COMMAND,
                    egui::Key::N,
                )),
                input.consume_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::COMMAND,
                    egui::Key::O,
                )),
                input.consume_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::COMMAND,
                    egui::Key::S,
                )),
            )
        });
        if save_project {
            match self.project.as_mut() {
                Some(seat) => match seat.save() {
                    Ok(()) => self.status = format!("saved {}", seat.path().display()),
                    Err(error) => self.status = error,
                },
                None => {
                    self.status =
                        "open a project first — Cmd+N to create one, Cmd+O to open one".to_owned();
                }
            }
        }
        // New / Open は座席を差し替える = いまの編集を捨てる。未保存なら先に訊く。
        if new_project && self.clear_unsaved_or_stay() {
            if let Some(path) = prompt_new_project_path() {
                match create_project_file(&path) {
                    Ok(()) => self.reseat(&path),
                    Err(error) => self.status = error,
                }
            }
        }
        if open_project && self.clear_unsaved_or_stay() {
            if let Some(path) = prompt_open_project_path() {
                self.reseat(&path);
            }
        }

        // ---- ドロップ。native では path が入っている ----
        let dropped: Vec<PathBuf> = ctx.input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .filter_map(|file| file.path.clone())
                .collect()
        });
        if !dropped.is_empty() {
            self.status = admit_dropped_paths(self.project.as_mut(), &dropped);
        }
    }

    /// Export ボタンの後ろ。保存先を訊いて（dialog はここで開く）、**現 writer
    /// snapshot** を別 thread の書き出しへ渡す（dirty でも書き出せる）。
    /// 判断（座席あり・実行中なし）は `can_start_export` — ボタンの enabled と
    /// 同じ関数なので、ここに来た時点で普通は真。dialog を閉じたら何もしない。
    fn begin_export(&mut self) {
        if !can_start_export(self.project.is_some(), self.export.is_some()) {
            return;
        }
        let seat = self
            .project
            .as_ref()
            .expect("can_start_export checked the seat");
        let Some(output_path) = prompt_export_path(seat.path()) else {
            return;
        };
        self.status = format!("exporting {}", output_path.display());
        self.export = Some(ExportRun::start(ExportRequest {
            document: seat.snapshot(),
            // project root = document path の親（CLI export と同じ規約）。
            project_root: seat.path().parent().map(Path::to_path_buf),
            output_path,
            // v0 は既定値のみ: composition 全長・通常品質。
            frame_count: None,
            qp0: false,
        }));
    }

    /// 実行中の書き出しの返事を受ける。**描く前に1回だけ**通す。
    /// 完了/キャンセル/失敗は status 帯の一言になり、run は捨てる。
    fn handle_export_finish(&mut self) {
        let Some(run) = self.export.as_mut() else {
            return;
        };
        let Some(finish) = run.try_finish() else {
            return;
        };
        self.status = match finish {
            ExportFinish::Done(report) => format!(
                "Exported {} ({}f)",
                run.output_path().display(),
                report.frames_written
            ),
            ExportFinish::Cancelled => {
                format!(
                    "export cancelled — {} was removed",
                    run.output_path().display()
                )
            }
            ExportFinish::Failed(reason) => format!("export failed: {reason}"),
        };
        self.export = None;
    }
}

/// 書き出し先を訊く。**dialog はここだけ**で、判定できる列は `export_seat` にある
/// （`prompt_new_project_path` / `prompt_open_project_path` と同じ集約）。
fn prompt_export_path(project: &Path) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Export video")
        .add_filter("MP4 video", &["mp4"])
        .set_file_name(default_export_file_name(project))
        .save_file()
}

/// New の場所を訊く。**dialog はここだけ**で、判定できる列は外にある。
fn prompt_new_project_path() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("New Motolii project")
        .add_filter("Motolii project", &["json"])
        .set_file_name("untitled.json")
        .save_file()
}

/// Open するファイルを訊く。同上。
fn prompt_open_project_path() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Open Motolii project")
        .add_filter("Motolii project", &["json"])
        .pick_file()
}

impl eframe::App for BlitzShellApp {
    /// eframe 0.35 の `App` は `update(ctx, frame)` ではなく `ui(&mut Ui, ..)` を要求する。
    /// 渡される `Ui` は余白も背景も持たないので、`CentralPanel` は自分で被せる
    /// (`eframe-0.35 src/epi.rs:165-176`)。
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // パネルが内部で blitz_net::Provider を起こしても panic しないよう、
        // フレーム全体を reactor の中で回す。
        let _guard = self.runtime.enter();

        // ファイルの入口（ドロップ / New / Open / Save）は描く前に通す。
        let ctx = ui.ctx().clone();
        self.handle_file_entry(&ctx);

        // 書き出し thread の返事も描く前に受ける。走っているあいだは入力が
        // 無くても回し続ける（経過秒の更新と完了の受け取りのため）。
        self.handle_export_finish();
        if self.export.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        }

        // 窓を閉じるのも「未保存のまま座席を捨てる」操作。egui の慣行どおり
        // close_requested を見て、留まるなら CancelClose を返す。判断と保存は
        // Cmd+O / Cmd+N と同じ `clear_unsaved_or_stay`。
        if ctx.input(|input| input.viewport().close_requested()) && !self.clear_unsaved_or_stay() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }

        // 窓の帯。project が居るあいだは常設で、**信用の可視化**を持つ:
        // Undo/Redo ボタン（Cmd+Z / Shift+Cmd+Z と同じ入口）と、保存状態
        // （保存済みなら project 名、未保存なら ● 付き）。一言（ドロップ・New・
        // Open・Save の結果）は従来どおり同じ帯の右に出る。座席が無いときは
        // 従来どおり、何か言うことがある時だけ帯が出る。
        let mut want_export = false;
        if self.project.is_some() || !self.status.is_empty() {
            egui::Panel::bottom("blitz_shell_status").show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(STATUS_PAD_X);
                    if let Some(seat) = self.project.as_mut() {
                        // 効かない時は disabled（押せない見た目 = 台帳が空）。
                        let undo = ui.add_enabled(
                            seat.editor().undo_len() > 0,
                            egui::Button::new(egui::RichText::new("Undo").size(STATUS_FONT_SIZE)),
                        );
                        let redo = ui.add_enabled(
                            seat.editor().redo_len() > 0,
                            egui::Button::new(egui::RichText::new("Redo").size(STATUS_FONT_SIZE)),
                        );
                        if undo.clicked() {
                            seat.editor_mut().undo_gesture();
                        }
                        if redo.clicked() {
                            seat.editor_mut().redo_gesture();
                        }
                        ui.separator();
                        let name = seat
                            .path()
                            .file_name()
                            .map(|name| name.to_string_lossy().into_owned())
                            .unwrap_or_else(|| seat.path().display().to_string());
                        if seat.is_dirty() {
                            ui.label(
                                egui::RichText::new(format!("● {name} — unsaved"))
                                    .size(STATUS_FONT_SIZE)
                                    .color(crate::timeline_editor::ACCENT),
                            );
                        } else {
                            ui.label(egui::RichText::new(name).size(STATUS_FONT_SIZE));
                        }
                    }
                    // 書き出し面。全ペルソナが最後に当たる面なので、信用の可視化と
                    // 同じ帯に常設する（Blitz chrome の Export fixture はマウスを
                    // 受けないため、押せる面はここに置く）。実行中は indeterminate
                    // スピナー＋経過秒＋Cancel（export 側に進捗 callback の口が
                    // 無い v0 の形）。二重起動はボタン自体が消えることで防ぐ。
                    if let Some(run) = self.export.as_ref() {
                        ui.separator();
                        ui.add(egui::Spinner::new().size(STATUS_FONT_SIZE + 2.0));
                        ui.label(
                            egui::RichText::new(format!("Exporting… {}s", run.elapsed_seconds()))
                                .size(STATUS_FONT_SIZE),
                        );
                        let cancel = ui.add_enabled(
                            !run.cancel_requested(),
                            egui::Button::new(egui::RichText::new("Cancel").size(STATUS_FONT_SIZE)),
                        );
                        if cancel.clicked() {
                            run.cancel();
                        }
                    } else if self.project.is_some() {
                        ui.separator();
                        let export = ui.add_enabled(
                            can_start_export(self.project.is_some(), self.export.is_some()),
                            egui::Button::new(egui::RichText::new("Export").size(STATUS_FONT_SIZE)),
                        );
                        if export.clicked() {
                            // dialog（保存先）は描画の外で開く（New / Open と同じ形）。
                            want_export = true;
                        }
                    }
                    if !self.status.is_empty() {
                        if self.project.is_some() {
                            ui.separator();
                        }
                        ui.label(
                            egui::RichText::new(&self.status)
                                .size(STATUS_FONT_SIZE)
                                .color(crate::timeline_editor::ACCENT),
                        );
                    }
                });
            });
        }
        if want_export {
            self.begin_export();
        }

        let mut behavior = BlitzShellBehavior {
            render_state: &self.render_state,
            editor: self.project.as_mut().map(ProjectSeat::editor_mut),
        };

        egui::CentralPanel::default().show(ui, |ui| {
            self.tree.ui(&mut behavior, ui);
        });
        drop(behavior);

        // 掴んだファイルが窓の上に来ているあいだ、受け取れることを見せる。
        paint_drop_hint(&ctx, self.project.is_some());

        // エディタ（か Undo/Redo）が Document を進めていたら、同じ新 snapshot を
        // Stage へ配り直す。Stage の描画コードは変えない — 渡す `Arc` の更新だけ。
        if let Some(seat) = self.project.as_ref() {
            let revision = seat.editor().revision();
            if revision != self.seated_revision {
                let snapshot = seat.snapshot();
                seat_stage_documents(&mut self.tree, &snapshot);
                self.seated_revision = revision;
            }
        }
    }
}

/// browser-library.css:47 の toolbar と同じ余白・字送りを status 帯にも使う
/// （新しい寸法をここで決めない）。
const STATUS_PAD_X: f32 = 5.0;
const STATUS_FONT_SIZE: f32 = 9.0;

/// ファイルを掴んだまま窓の上に来ているあいだの見せ方。
///
/// egui 慣行どおり `hovered_files` を見て前面レイヤに1枚だけ被せる。色は
/// Browser パネルの token をそのまま使い、新しい色を決めない。
/// 座席が無いときは「取り込めない」ことが分かる言葉にする。
fn paint_drop_hint(ctx: &egui::Context, seated: bool) {
    let hovering = ctx.input(|input| input.raw.hovered_files.len());
    if hovering == 0 {
        return;
    }
    let screen = ctx.content_rect();
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("blitz_shell_drop_hint"),
    ));
    // 面は暗く落として、縁だけ accent で囲む（Browser の focus 枠と同じ考え）。
    painter.rect_filled(
        screen,
        egui::CornerRadius::ZERO,
        egui::Color32::from_black_alpha(96),
    );
    painter.rect_stroke(
        screen.shrink(4.0),
        egui::CornerRadius::ZERO,
        egui::Stroke::new(2.0, crate::timeline_editor::ACCENT),
        egui::StrokeKind::Inside,
    );
    let message = if seated {
        format!(
            "drop {hovering} file{} to place at the playhead",
            if hovering == 1 { "" } else { "s" }
        )
    } else {
        "open a project first — Cmd+N to create one, Cmd+O to open one".to_owned()
    };
    painter.text(
        screen.center(),
        egui::Align2::CENTER_CENTER,
        message,
        egui::FontId::proportional(14.0),
        crate::timeline_editor::ACCENT,
    );
}

/// tree の中の Stage pane 全部へ、新しい snapshot を配り直す。
///
/// single writer の reader 側: ここで配るのは writer が出した `Arc` そのもので、
/// 読み直しも clone 編集もしない。
fn seat_stage_documents(tree: &mut Tree<BlitzPane>, document: &Arc<motolii_doc::Document>) {
    for (_, tile) in tree.tiles.iter_mut() {
        if let Tile::Pane(pane) = tile {
            if pane.kind() == PaneKind::Stage {
                pane.set_live_document(Arc::clone(document));
            }
        }
    }
}

/// 既定のレイアウトを組む。
///
/// 面の並びは `docs/ui-interaction-language.md` と `productStyles.ts` の
/// `workspace` / `centerColumn` を写したもので、新しい配置思想は足していない。
///
/// ```text
/// 横 ─┬─ Browser
///     ├─ 中央列（縦）─┬─ Stage
///     │               └─ Timeline
///     └─ 右列（縦）─┬─ Inspector
///                   └─ chrome タブ（Export / Settings / Panels）
/// ```
///
/// Stage だけ Blitz ではなく **Rerun Spatial Viewer** が描く。
/// Motolii はその wrapper であって `re_renderer` で直接シーンを組まない（2026-08-11裁定）。
///
/// `document` は live project の snapshot（`ProjectSeat::snapshot`）。座るのは
/// **Stage だけ**で、live の Timeline は pane ではなくエディタが描く
/// （`BlitzShellBehavior::pane_ui`）。Browser / Inspector / chrome は fixture のまま
/// （本レーンの範囲）。`None` なら全面が従来どおり fixture。
fn build_initial_tree(document: Option<&Arc<motolii_doc::Document>>) -> Tree<BlitzPane> {
    let mut tiles = Tiles::default();

    let seated = |kind: PaneKind| match document {
        Some(doc) => BlitzPane::with_document(kind, Arc::clone(doc)),
        None => BlitzPane::new(kind),
    };

    // 左: Browser。
    let browser = tiles.insert_pane(BlitzPane::new(PaneKind::Browser));

    // 中央: 上が Stage（live Document が座る）、下が Timeline
    // （live ならエディタが behavior 側で描くので、pane は席だけ）。
    let stage = tiles.insert_pane(seated(PaneKind::Stage));
    let timeline = tiles.insert_pane(BlitzPane::new(PaneKind::Timeline));
    let center = tiles.insert_vertical_tile(vec![stage, timeline]);

    // 右: Inspector。
    let inspector = tiles.insert_pane(BlitzPane::new(PaneKind::Inspector));

    // chrome の 3 枚はタブとして 1 ペインにまとめる。
    // 注意: これらは本来モーダル／拡張パネルであって常設面ではない。
    // ここに席があるのは「main の画面を見る」ための便宜であり、
    // 常設パネルという UI 決定ではない。
    let chrome_export = tiles.insert_pane(BlitzPane::new(PaneKind::ChromeExport));
    let chrome_settings = tiles.insert_pane(BlitzPane::new(PaneKind::ChromeSettings));
    let chrome_panels = tiles.insert_pane(BlitzPane::new(PaneKind::ChromePanels));
    let chrome = tiles.insert_tab_tile(vec![chrome_export, chrome_settings, chrome_panels]);

    // 右列は Inspector（上）と chrome タブ（下）。
    let right = tiles.insert_vertical_tile(vec![inspector, chrome]);

    // 3 列を横に並べる。`centerColumn` が flex:1 で左右が固定幅相当なので、
    // 中央の share を大きく取る。
    let mut root_linear = Linear::new(LinearDir::Horizontal, vec![browser, center, right]);
    root_linear.shares.set_share(browser, 0.22);
    root_linear.shares.set_share(center, 0.53);
    root_linear.shares.set_share(right, 0.25);
    let root = tiles.insert_new(Tile::Container(Container::Linear(root_linear)));

    Tree::new("blitz_shell_tree", root, tiles)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;

    /// 一時projectを1つ作る。`examples/create_timeline_lab_project.rs` と同じ経路
    /// (`ProjectSession::acquire` → `save_document`)。fixture と取り違えていないことを
    /// 後で判定できるよう、duration に目印の値を入れておく。
    fn create_project(tag: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("motolii-blitz-shell-{tag}-{nanos}"));
        std::fs::create_dir_all(&dir).expect("temp project dir");
        let path = dir.join("project.json");
        let mut doc = motolii_doc::Document::new_current();
        doc.composition.duration = marker_duration();
        let mut session =
            motolii_doc::ProjectSession::acquire(&path, &motolii_doc::ResourceLimits::production())
                .expect("acquire temp project");
        session
            .save_document(&doc, &motolii_doc::SaveOptions::default())
            .expect("save temp project");
        // lock を返す。`ProjectSeat::open` が取り直す。
        drop(session);
        path
    }

    /// fixture(`reference-document.json`)には出てこない、目印にする duration。
    fn marker_duration() -> motolii_core::RationalTime {
        motolii_core::RationalTime::try_new(7, 1).expect("marker duration")
    }

    /// live では Stage に snapshot が座り、Timeline は pane でなくエディタが持つ
    /// （編集レーンで Timeline の表示主体が HtmlPane からエディタへ移った）。
    #[test]
    fn opening_a_project_seats_its_document_into_stage_and_the_editor() {
        let path = create_project("seat");
        let seat = ProjectSeat::open(&path).expect("open temp project");
        let snapshot = seat.snapshot();
        assert_eq!(
            snapshot.composition.duration,
            marker_duration(),
            "seat must serve the opened project, not the fixture"
        );
        assert!(
            Arc::ptr_eq(seat.editor().document(), &snapshot),
            "the editor serves the writer snapshot itself, not a re-load"
        );

        let tree = build_initial_tree(Some(&snapshot));
        let mut timeline = 0;
        let mut stage = 0;
        for (_, tile) in tree.tiles.iter() {
            let Tile::Pane(pane) = tile else { continue };
            match pane.kind() {
                PaneKind::Timeline => {
                    timeline += 1;
                    assert!(
                        pane.live_document().is_none(),
                        "live の Timeline はエディタが描く。pane に第二の Document を持たせない"
                    );
                }
                PaneKind::Stage => {
                    stage += 1;
                    let doc = pane
                        .live_document()
                        .expect("Stage must receive the live Document");
                    assert!(
                        Arc::ptr_eq(doc, &snapshot),
                        "Stage must hold the writer snapshot itself, not a re-load"
                    );
                }
                other => assert!(
                    pane.live_document().is_none(),
                    "{other:?} stays on fixtures in this lane"
                ),
            }
        }
        assert_eq!((timeline, stage), (1, 1), "default layout has one of each");
    }

    /// エディタの編集が進んだら、Stage pane へ**同じ新 snapshot**が配り直される
    /// （`BlitzShellApp::ui` の revision 照合が呼ぶのはこの関数）。
    #[test]
    fn an_edit_reseats_the_stage_snapshot() {
        // clip を持つ lab fixture を実プロジェクトとして保存して開く。
        let (document, names) = crate::timeline_editor::lab_fixture();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("motolii-blitz-shell-reseat-{nanos}"));
        std::fs::create_dir_all(&dir).expect("temp project dir");
        let path = dir.join("project.json");
        let mut session =
            motolii_doc::ProjectSession::acquire(&path, &motolii_doc::ResourceLimits::production())
                .expect("acquire temp project");
        session
            .save_document(&document, &motolii_doc::SaveOptions::default())
            .expect("save temp project");
        drop(session);

        let mut seat = ProjectSeat::open(&path).expect("open temp project");
        let first = seat.snapshot();
        let mut tree = build_initial_tree(Some(&first));

        // エディタの操作 API で move を1回通す(writer 経由の実編集)。
        let layer = *names
            .iter()
            .find(|(_, name)| name.as_str() == "Background")
            .map(|(layer, _)| layer)
            .expect("fixture layer");
        let revision_before = seat.editor().revision();
        let editor = seat.editor_mut();
        editor.begin_clip_move(layer, 1.0);
        editor.drag_to(2.0);
        editor.release();
        assert!(
            seat.editor().revision() > revision_before,
            "the move must advance the writer revision"
        );

        let second = seat.snapshot();
        assert!(
            !Arc::ptr_eq(&first, &second),
            "the edit produces a new snapshot Arc"
        );

        seat_stage_documents(&mut tree, &second);
        let mut stage = 0;
        for (_, tile) in tree.tiles.iter() {
            let Tile::Pane(pane) = tile else { continue };
            if pane.kind() == PaneKind::Stage {
                stage += 1;
                let doc = pane.live_document().expect("Stage stays seated");
                assert!(
                    Arc::ptr_eq(doc, &second),
                    "Stage must be handed the new snapshot Arc itself"
                );
            }
        }
        assert_eq!(stage, 1, "default layout has one Stage");
    }

    #[test]
    fn fixture_tree_has_no_live_document() {
        let tree = build_initial_tree(None);
        for (_, tile) in tree.tiles.iter() {
            if let Tile::Pane(pane) = tile {
                assert!(
                    pane.live_document().is_none(),
                    "{:?} must stay on the fixture without --project",
                    pane.kind()
                );
            }
        }
    }

    #[test]
    fn opening_a_missing_project_fails_loudly() {
        let missing = std::env::temp_dir().join("motolii-blitz-shell-missing/nope/project.json");
        assert!(
            ProjectSeat::open(&missing).is_err(),
            "an unopenable project must be a startup error, not a silent fixture fallback"
        );
    }
}
