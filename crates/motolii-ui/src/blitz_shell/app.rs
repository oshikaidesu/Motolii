//! Blitz パネルを 1 つの窓に合体させるための eframe アプリ。
//!
//! ドッキング（分割・タブ化・resize）は Blitz へ移植せず egui の責任とする裁定に従い、
//! ここでは `egui_tiles` の既定挙動をそのまま出す。操作感を「改善」しない。
//!
//! このファイルは器（どこに何の面が座るか）だけを決める。
//! ペインの中身・色・寸法は `super::pane::BlitzPane` が描く。
//! レイアウトの永続化もしない（毎回既定の並びで起動する）。
//!
//! **Document の座席はここに無い**（2026-08-18「ログと構造の強制」）。`ProjectSeat`
//! （`ProjectSession` の OS lock ＋ 唯一の writer を抱える Timeline エディタ）は
//! `super::intent::ShellGateway` の中に居て、app が触れるのは
//! [`BlitzShellApp::dispatch`] に `UiIntent` を渡す道だけである。つまりこのファイルの
//! 仕事は 3 つに減った:
//!
//! 1. **入力を intent に翻訳する**（Cmd+N → 訊く → `NewProject { path }`）
//! 2. **面を状態へ合わせる**（`resync_view` が Stage へ snapshot を配り直す）
//! 3. 器（どこに何の面が座るか）を決める
//!
//! Stage へ流れるのは相変わらず immutable snapshot だけで、**Timeline は live のとき
//! native エディタ**（`timeline_editor::TimelineEditor`）である。その中の編集と
//! Undo/Redo はまだ intent を通らない（`motolii-doc` の D2 journal が受けている。
//! shell 層の intent 化は wave E）。座席無しの起動は従来どおりスタート画面。

use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui_wgpu::RenderState;
use egui_tiles::{Container, Linear, LinearDir, Tile, TileId, Tiles, Tree, UiResponse};

use super::drive::{NativePrompts, ShellPrompts, ShellTranscript};
use super::intent::{IntentJournal, ProjectSeat, ShellGateway, UiIntent};
use super::pane::{BlitzPane, PaneKind};
use crate::browser_panel::BrowserRequest;
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
    /// 面が出した要求(いまは Browser のカードのダブルクリックだけ)。
    /// **ここでは実行しない** — 座席を触るのは描き終わってからの `app` である。
    /// 1フレームに1件で足りる(人の指は1本)。
    browser_request: Option<BrowserRequest>,
}

impl egui_tiles::Behavior<BlitzPane> for BlitzShellBehavior<'_> {
    fn pane_ui(&mut self, ui: &mut egui::Ui, _tile_id: TileId, pane: &mut BlitzPane) -> UiResponse {
        if pane.kind() == PaneKind::Timeline {
            if let Some(editor) = self.editor.as_deref_mut() {
                editor.show(ui);
                return UiResponse::None;
            }
        }
        // Inspector も live のときは fixture ではなく**選択を映す**。編集要求は
        // その場でエディタの適用口へ渡す — Document を書くのは相変わらず
        // エディタが抱える1つの writer だけである(single writer)。
        if pane.kind() == PaneKind::Inspector {
            if let Some(editor) = self.editor.as_deref_mut() {
                pane.show_live_inspector(ui, editor);
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
        if let Some(request) = pane.show(ui, self.render_state) {
            self.browser_request = Some(request);
        }
        // ペイン本体をドラッグ元にはしない（タブのドラッグだけで足りる）。
        UiResponse::None
    }

    fn tab_title_for_pane(&mut self, pane: &BlitzPane) -> egui::WidgetText {
        pane.title().into()
    }
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

/// Blitz パネルを合体表示するアプリ本体。
pub struct BlitzShellApp {
    /// `blitz_net::Provider` は Tokio reactor を要求し、無いと panic する。
    /// reactor を保証するのはこのアプリの責任。`update()` の先頭で enter する。
    runtime: tokio::runtime::Runtime,
    /// wgpu バックエンド前提（eframe は `features = ["wgpu"]`、glow ではない）。
    render_state: RenderState,
    tree: Tree<BlitzPane>,
    /// **製品状態への唯一の口**（`super::intent`）。live project の座席と実行中の
    /// 書き出しはこの中に居て、app は `UiIntent` を渡す以外に動かせない。
    /// 記録（原因のログ）と実行が1点に集まっているのがこの型の全部である。
    gateway: ShellGateway,
    /// Stage へ最後に配った座席（パスと revision）。ここと gateway の現状がずれた
    /// フレームで、新しい snapshot を Stage へ配り直す（`resync_view`）。
    seated: Option<PathBuf>,
    seated_revision: u64,
    /// 窓ぜんたいに関わる一言（ドロップ・New・Open の結果）の**唯一の言い場所**。
    /// 帯は `latest()` を映し、言われた全文はそのまま残る（`--status-log` へも出る）。
    /// pane の失敗も同じ台帳へ来る（clone を配ってある）ので、stderr へ消えない。
    /// Timeline の中の出来事は従来どおりエディタ自身の status が言う。
    /// gateway が持っているのと**同じ台帳**（`Arc` 共有）。
    transcript: ShellTranscript,
    /// 人に訊く4本（New / Open / Export / 未保存）。窓は `NativePrompts`（rfd）で、
    /// テスト・CLI 駆動は台本が答える。**app は rfd を直接呼ばない。**
    ///
    /// 訊くのは intent の**外**で、決まった答え（path・3択）だけが intent の中へ
    /// 入る。だから replay は dialog を二度と開かない。
    prompts: Box<dyn ShellPrompts>,
    /// fixture 展示モード（開発動線・screenshot テスト用、`--fixture`）。
    /// **既定は false** — 座席なしの起動は展示ではなくスタート画面
    /// (New / Open) を出す。展示が製品状態に見える混乱をUXチェック第1号で確認した。
    fixture: bool,
    /// Browser が見るフォルダ。`None` は既定(`docs/mocks`)。座席を失って並びを
    /// 組み直すとき(`resync_view` の失席経路)も同じ根を渡し直すために持っている。
    browser_root: Option<PathBuf>,
}

impl BlitzShellApp {
    /// `eframe::CreationContext` から作る（fixture 展示。従来どおり）。
    ///
    /// # Panics
    /// - wgpu の `RenderState` が取れない場合（glow バックエンドで起動された等）
    /// - Tokio runtime を作れない場合
    pub(crate) fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::with_seat(cc, None, true)
    }

    /// 座席ごと作る。`Some(seat)` なら Timeline / Stage は fixture ではなく
    /// その project の Document（writer の snapshot）を映す。
    ///
    /// `CreationContext` から取り出せるものを取り出して [`Self::with_deps`] へ渡す
    /// **薄皮**で、判断はここに無い。訊き手は窓の `NativePrompts`（= 現行の rfd）。
    ///
    /// # Panics
    /// `new` と同じ。
    pub(crate) fn with_seat(
        cc: &eframe::CreationContext<'_>,
        project: Option<ProjectSeat>,
        fixture: bool,
    ) -> Self {
        let render_state = cc
            .wgpu_render_state
            .clone()
            .expect("BlitzShellApp は wgpu バックエンドを要求する（eframe features = [\"wgpu\"]）");
        Self::with_deps(
            &cc.egui_ctx,
            render_state,
            Box::new(NativePrompts),
            project,
            fixture,
            // 窓は従来どおり `docs/mocks` を見る(Browser の根はこのレーンの主題ではない)。
            None,
        )
    }

    /// 依存を全部**外から**渡して作る構築 seam。窓（`CreationContext`）でも
    /// headless の運転席（`drive::DrivenShell`）でも同じ shell が立つ。
    ///
    /// ここで足しているのは器だけ（reactor・帯の台帳・面の並び）で、
    /// 製品の挙動は `with_seat` 経由と1つも変わらない。
    ///
    /// # Panics
    /// Tokio runtime を作れない場合。
    ///
    /// `browser_root` は Browser が見るフォルダ。`None` は製品の既定(`docs/mocks`)で、
    /// 運転席だけが実 media の入った folder を座らせる(窓を開かずにカードを触るため)。
    pub(crate) fn with_deps(
        egui_ctx: &egui::Context,
        render_state: RenderState,
        prompts: Box<dyn ShellPrompts>,
        project: Option<ProjectSeat>,
        fixture: bool,
        browser_root: Option<PathBuf>,
    ) -> Self {
        // 記号(◆ ◇ ▶ ← ↔ →)が豆腐にならないよう、既定fontの後ろにHackを連ねる。
        // 新しいフォントは足していない。詳細は `egui_fonts`。
        crate::egui_fonts::install_symbol_fallback(egui_ctx);

        let runtime = tokio::runtime::Runtime::new()
            .expect("blitz_net::Provider 用の Tokio runtime を作れなかった");

        // snapshot はここで1度だけ取り、Stage が writer の出した `Arc` そのものを持つ。
        let snapshot = project.as_ref().map(ProjectSeat::snapshot);
        let seated = project.as_ref().map(|seat| seat.path().to_path_buf());
        let seated_revision = project
            .as_ref()
            .map(|seat| seat.editor().revision())
            .unwrap_or(0);
        // 帯の台帳は面より先に作る。pane は clone を持って同じ場所へ言う。
        let transcript = ShellTranscript::default();
        // 製品状態は最初からゲートウェイの中。`--project` で開いた座席も
        // 「どうやって着いたか」を journal の第1行に持つ（`ShellGateway::seated`）。
        let gateway = match project {
            Some(seat) => ShellGateway::seated(transcript.clone(), seat),
            None => ShellGateway::new(transcript.clone()),
        };
        Self {
            runtime,
            render_state,
            tree: build_initial_tree(snapshot.as_ref(), &transcript, browser_root.clone()),
            gateway,
            seated,
            seated_revision,
            transcript,
            prompts,
            fixture,
            browser_root,
        }
    }

    /// スタート画面を出すべきか。座席が無く、fixture 展示も明示されていない時。
    fn shows_welcome(&self) -> bool {
        !self.gateway.is_seated() && !self.fixture
    }

    /// **利用者の操作をゲートウェイへ流す唯一の口。**
    ///
    /// intent は記録されてから実行され、実行のあとで面（Stage の snapshot・
    /// fixture への戻り）を現状へ合わせる。返すのは「意図どおり進んだか」で、
    /// 進まなかった理由は帯が言っている。
    fn dispatch(&mut self, intent: UiIntent) -> bool {
        let proceeded = self.gateway.dispatch(intent);
        self.resync_view();
        proceeded
    }

    /// 面をゲートウェイの現状へ合わせる。**判断はここに無い** — 座席が変わったか
    /// （path）、Document が進んだか（revision）を見て、Stage へ配り直すだけ。
    ///
    /// 座席を失ったときだけ `build_initial_tree(None)` で並びを組み直す
    /// （黙って fixture に見えないため。旧 `reseat` の失敗経路と同じ）。
    fn resync_view(&mut self) {
        match self.gateway.project() {
            Some(seat) => {
                let path = seat.path().to_path_buf();
                let revision = seat.editor().revision();
                if self.seated.as_deref() != Some(path.as_path()) || revision != self.seated_revision
                {
                    let snapshot = seat.snapshot();
                    seat_stage_documents(&mut self.tree, &snapshot);
                    self.seated = Some(path);
                    self.seated_revision = revision;
                }
            }
            None => {
                if self.seated.is_some() {
                    self.tree =
                        build_initial_tree(None, &self.transcript, self.browser_root.clone());
                    self.seated = None;
                    self.seated_revision = 0;
                }
            }
        }
    }

    /// New の実体（Cmd+N とスタート画面のボタンが共用）。
    /// **訊くのはここ、記録と実行はゲートウェイ**。
    fn request_new_project(&mut self) {
        if !self.clear_unsaved_or_stay() {
            return;
        }
        if let Some(path) = self.prompts.new_project_path() {
            self.dispatch(UiIntent::NewProject { path });
        }
    }

    /// Open の実体（Cmd+O とスタート画面のボタンが共用）。
    fn request_open_project(&mut self) {
        if !self.clear_unsaved_or_stay() {
            return;
        }
        if let Some(path) = self.prompts.open_project_path() {
            self.dispatch(UiIntent::OpenProject { path });
        }
    }

    /// 座席の参照。
    pub fn project(&self) -> Option<&ProjectSeat> {
        self.gateway.project()
    }

    /// 窓の一言の台帳（読み）。帯が映すのは `latest()`、`--status-log` は全文を流す。
    pub(crate) fn transcript(&self) -> &ShellTranscript {
        &self.transcript
    }

    /// 原因のログ（読み）。`--intent-log` は全文を流し、replay oracle はここを読む。
    pub(crate) fn intent_journal(&self) -> &IntentJournal {
        self.gateway.journal()
    }

    /// 未保存のまま座席を捨てる操作（Cmd+O / Cmd+N / 窓を閉じる）の前に挟む。
    /// 続行してよければ `true`。判断は `decide_unsaved`、訊き手は
    /// `prompts.unsaved_choice` に居る。「保存して続行」で保存に失敗したら
    /// **続行しない**（帯に理由が出る）。
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
            UnsavedDecision::SaveThenProceed => self.dispatch(UiIntent::SaveProject),
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
            self.dispatch(UiIntent::SaveProject);
        }
        // New / Open は座席を差し替える = いまの編集を捨てる。未保存なら先に訊く
        // （その判断ごと `request_*` が持つ。スタート画面のボタンと同じ入口）。
        if new_project {
            self.request_new_project();
        }
        if open_project {
            self.request_open_project();
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
            // OS ドロップと Browser のダブルクリックは**同じ intent**へ合流する。
            self.dispatch(UiIntent::AdmitPaths { paths: dropped });
        }
    }

    /// Export ボタンの後ろ。保存先を訊いて（訊き手は `prompts`）、決まった保存先を
    /// intent に入れて渡す。判断（座席あり・実行中なし）は `can_start_export` —
    /// ボタンの enabled と同じ関数で、実体はゲートウェイの中にもう一度ある。
    /// 訊いて断られたら何も記録しない（**訊いただけの操作は intent ではない**）。
    fn begin_export(&mut self) {
        if !self.gateway.can_start_export() {
            return;
        }
        let project = self
            .gateway
            .project()
            .expect("can_start_export checked the seat")
            .path()
            .to_path_buf();
        let Some(output) = self.prompts.export_path(&project) else {
            return;
        };
        self.dispatch(UiIntent::BeginExport { output });
    }

    /// 1フレーム描く。
    ///
    /// eframe 0.35 の `App` は `update(ctx, frame)` ではなく `ui(&mut Ui, ..)` を要求し、
    /// 渡される `Ui` は余白も背景も持たないので `CentralPanel` は自分で被せる
    /// (`eframe-0.35 src/epi.rs:165-176`)。**`eframe::Frame` は使わない**ので取らない —
    /// 窓を持たない運転席（`drive::DrivenShell`）からも同じ1フレームを回せる。
    /// 窓を開く側の `eframe::App` は `runner.rs` の `Harness` が持つ。
    pub(crate) fn ui(&mut self, ui: &mut egui::Ui) {
        // パネルが内部で blitz_net::Provider を起こしても panic しないよう、
        // フレーム全体を reactor の中で回す。
        let _guard = self.runtime.enter();

        // ファイルの入口（ドロップ / New / Open / Save）は描く前に通す。
        let ctx = ui.ctx().clone();
        self.handle_file_entry(&ctx);

        // 書き出し thread の返事も描く前に受ける。**これは intent ではない** —
        // 利用者の操作ではなく、世界の側からの返事だから。走っているあいだは入力が
        // 無くても回し続ける（経過秒の更新と完了の受け取りのため）。
        self.gateway.poll_export();
        if self.gateway.export().is_some() {
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
        //
        // 押された結果は**その場で実行しない**。描画の中は「押された」を拾うだけで、
        // dialog と intent の dispatch は描き終わってから通す（New / Open と同じ形）。
        let mut want_export = false;
        let mut want_cancel = false;
        // 帯が映すのは台帳の**最新の1行**。言われた全文は transcript に残る。
        let latest = self.transcript.latest();
        let seated = self.gateway.is_seated();
        let can_export = self.gateway.can_start_export();
        if seated || latest.is_some() {
            egui::Panel::bottom("blitz_shell_status").show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(STATUS_PAD_X);
                    if let Some(seat) = self.gateway.project_mut() {
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
                    if let Some(run) = self.gateway.export() {
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
                            want_cancel = true;
                        }
                    } else if seated {
                        ui.separator();
                        let export = ui.add_enabled(
                            can_export,
                            egui::Button::new(egui::RichText::new("Export").size(STATUS_FONT_SIZE)),
                        );
                        if export.clicked() {
                            // dialog（保存先）は描画の外で開く（New / Open と同じ形）。
                            want_export = true;
                        }
                    }
                    if let Some(latest) = latest.as_deref() {
                        if seated {
                            ui.separator();
                        }
                        ui.label(
                            egui::RichText::new(latest)
                                .size(STATUS_FONT_SIZE)
                                .color(crate::timeline_editor::ACCENT),
                        );
                    }
                });
            });
        }
        if want_cancel {
            self.dispatch(UiIntent::CancelExport);
        }
        if want_export {
            self.begin_export();
        }

        // 座席が無い既定起動は panel 群でなく**スタート画面**。fixture 展示は
        // `--fixture` の明示だけ(展示が製品状態に見える混乱を UX チェック第1号で確認)。
        if self.shows_welcome() {
            let mut new_clicked = false;
            let mut open_clicked = false;
            egui::CentralPanel::default().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() * 0.32);
                    ui.label(
                        egui::RichText::new("Motolii")
                            .size(34.0)
                            .strong(),
                    );
                    ui.add_space(6.0);
                    ui.label("Make one 3\u{2013}5 minute music video.");
                    ui.add_space(24.0);
                    new_clicked = ui
                        .add(egui::Button::new(
                            egui::RichText::new("New Project\u{2026}   Cmd+N").size(16.0),
                        ))
                        .clicked();
                    ui.add_space(8.0);
                    open_clicked = ui
                        .add(egui::Button::new(
                            egui::RichText::new("Open\u{2026}   Cmd+O").size(16.0),
                        ))
                        .clicked();
                    ui.add_space(18.0);
                    ui.label(
                        egui::RichText::new("Then just drop video and audio into this window.")
                            .weak(),
                    );
                });
            });
            if new_clicked {
                self.request_new_project();
            }
            if open_clicked {
                self.request_open_project();
            }
            return;
        }

        // Timeline / Inspector の中の編集は**まだ intent を通らない**（wave E）。
        // 通っているのは `motolii-doc` 側の D2 Command journal だけである。
        let mut behavior = BlitzShellBehavior {
            render_state: &self.render_state,
            editor: self.gateway.project_mut().map(ProjectSeat::editor_mut),
            browser_request: None,
        };

        egui::CentralPanel::default().show(ui, |ui| {
            self.tree.ui(&mut behavior, ui);
        });
        let browser_request = behavior.browser_request.take();
        drop(behavior);

        // Browser のカードのダブルクリック。**ドロップと同じ1本の経路へ合流させる** —
        // 新しい import 経路は作らない(2026-08-18 の実機一撃で「押しても何も起きない」
        // と分かった所。原因は判断が無いことではなく、要求がどこにも流れていなかったこと)。
        // 合流点は今や `UiIntent::AdmitPaths` そのもので、入口が増えても intent は
        // 1種類のまま。成立も失敗も帯(= transcript)が一言で言う。
        if let Some(BrowserRequest::PlaceFile(path)) = browser_request {
            self.dispatch(UiIntent::AdmitPaths { paths: vec![path] });
        }

        // 掴んだファイルが窓の上に来ているあいだ、受け取れることを見せる。
        paint_drop_hint(&ctx, self.gateway.is_seated());

        // エディタ（か Undo/Redo）が Document を進めていたら、同じ新 snapshot を
        // Stage へ配り直す。**intent を通らない編集はここでしか拾えない**（wave E で
        // 通るようになったら、この呼び出しは dispatch 側の resync に吸収される）。
        self.resync_view();
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
///
/// `transcript` は帯の台帳。**面の失敗もここへ言う**ので、全ペインに同じ clone を配る
/// （`pane.rs` の stderr 専用失敗を全廃した先がこれ。`tests/shell_error_fence.rs`）。
fn build_initial_tree(
    document: Option<&Arc<motolii_doc::Document>>,
    transcript: &ShellTranscript,
    browser_root: Option<PathBuf>,
) -> Tree<BlitzPane> {
    let mut tiles = Tiles::default();

    let plain = |kind: PaneKind| {
        BlitzPane::new(kind)
            .with_browser_root(browser_root.clone())
            .reporting_to(transcript)
    };
    let seated = |kind: PaneKind| match document {
        Some(doc) => BlitzPane::with_document(kind, Arc::clone(doc)).reporting_to(transcript),
        None => plain(kind),
    };

    // 左: Browser。
    let browser = tiles.insert_pane(plain(PaneKind::Browser));

    // 中央: 上が Stage（live Document が座る）、下が Timeline
    // （live ならエディタが behavior 側で描くので、pane は席だけ）。
    let stage = tiles.insert_pane(seated(PaneKind::Stage));
    let timeline = tiles.insert_pane(plain(PaneKind::Timeline));
    let center = tiles.insert_vertical_tile(vec![stage, timeline]);

    // 右: Inspector。
    let inspector = tiles.insert_pane(plain(PaneKind::Inspector));

    // chrome の 3 枚はタブとして 1 ペインにまとめる。
    // 注意: これらは本来モーダル／拡張パネルであって常設面ではない。
    // ここに席があるのは「main の画面を見る」ための便宜であり、
    // 常設パネルという UI 決定ではない。
    let chrome_export = tiles.insert_pane(plain(PaneKind::ChromeExport));
    let chrome_settings = tiles.insert_pane(plain(PaneKind::ChromeSettings));
    let chrome_panels = tiles.insert_pane(plain(PaneKind::ChromePanels));
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

        let tree = build_initial_tree(Some(&snapshot), &ShellTranscript::default(), None);
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
        let mut tree = build_initial_tree(Some(&first), &ShellTranscript::default(), None);

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
        let tree = build_initial_tree(None, &ShellTranscript::default(), None);
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
