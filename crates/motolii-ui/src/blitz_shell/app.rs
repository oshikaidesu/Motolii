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
    /// project identity の排他 lock。読みはしないが、app が生きているあいだ握り続ける
    /// （落とすと他プロセスが同じ project を開けてしまう）。
    _session: motolii_doc::ProjectSession,
    /// Timeline エディタ。編集は全部（キー入力も操作 API も）この中の writer を通る。
    editor: TimelineEditor,
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
        Ok(Self {
            path: path.to_path_buf(),
            _session: opened.session,
            // project root = document path の親(CLI export と同じ規約)。
            // soundtrack 再生の asset path 解決に使う。
            editor: TimelineEditor::new(writer)
                .with_project_root(path.parent().map(Path::to_path_buf)),
        })
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

    /// ドロップ・New・Open を受ける。**描く前に1回だけ**通す。
    fn handle_file_entry(&mut self, ctx: &egui::Context) {
        // ---- New / Open。dialog は main thread を止めて開く（eframe 慣行） ----
        let (new_project, open_project) = ctx.input_mut(|input| {
            (
                input.consume_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::COMMAND,
                    egui::Key::N,
                )),
                input.consume_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::COMMAND,
                    egui::Key::O,
                )),
            )
        });
        if new_project {
            if let Some(path) = prompt_new_project_path() {
                match create_project_file(&path) {
                    Ok(()) => self.reseat(&path),
                    Err(error) => self.status = error,
                }
            }
        }
        if open_project {
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

        // ファイルの入口（ドロップ / New / Open）は描く前に通す。
        let ctx = ui.ctx().clone();
        self.handle_file_entry(&ctx);

        // 窓の一言。何か言うことがある時だけ帯が出る（常設の面を増やさない）。
        if !self.status.is_empty() {
            egui::Panel::bottom("blitz_shell_status").show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(STATUS_PAD_X);
                    ui.label(
                        egui::RichText::new(&self.status)
                            .size(STATUS_FONT_SIZE)
                            .color(crate::timeline_editor::ACCENT),
                    );
                });
            });
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
