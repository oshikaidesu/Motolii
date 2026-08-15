//! Blitzパネル1面ぶんの器。**見た目も意味も持たない。**
//!
//! ホストは eframe(egui)。Blitz は毎フレーム、eframe が持つ wgpu device 上の
//! 自前textureへ HTML/CSS を描き、egui はそれを画像として合成する
//! — `spikes/blitz-probe/src/bin/texture_host.rs`(P7)の実走体そのままの経路。
//! 描画は `browser_blitz::render::BlitzSurface`(実証済み)を使い回す。
//! **新しい描画方式はここで作らない。**
//!
//! HTML/CSS はどれも既存の移植実装(`timeline_blitz` / `inspector_blitz` /
//! `chrome_blitz` / `browser_blitz`)が出したものを使う。色も寸法もこのfileでは決めない。
//!
//! **Stage だけは別経路。** `re_view_spatial::SpatialStage` は元から egui の
//! ウィジェットなので、ホストの `egui::Ui` へ直接挿す(`EmbeddedSpatialStage::show_in`)。
//! 自前textureも2つ目の egui context も持たない。
//!
//! ## このfileが持たないもの
//! - 入力ルーティング(マウスを受けない。`Sense::hover()` で場所を取るだけ)
//! - Document編集・DomainIntent(表示する値は既存実装の投影とsampleのまま)
//! どちらも後続capsule。
//!
//! ## 実測済みの罠(このリポジトリで踏んだもの)
//! 1. textureのformatは `Rgba8Unorm`。`Rgba8UnormSrgb` にすると色が浮く
//!    (不透明パネルの値が45→117。`blitz_dump/gpu.rs:11-13`)
//! 2. `doc.resolve()` は**2回**呼ぶ。blitz-dom は z-index 付き要素の座標を
//!    `flush_styles_to_layout` で積むが、それは `resolve_layout` より前に走るので
//!    1レイアウト遅れる。1回だと z-index 付き要素が原点へ落ちる。
//!    `chrome_blitz` の modal scrim が `z-index:20`(`productStyles.ts:5`)を持つので実際に効く
//! 3. `blitz_net::Provider` は Tokio reactor を要求する。無いとpanicする
//!    (`browser_blitz/mod.rs` の罠2)。**このfileは runtime を張らない** — 下記参照
//!
//! ## reactor の契約
//! `HtmlDocument` の構築は呼び出し側(`app.rs`)の reactor の中で起きる前提で書く。
//! ここで `tokio::runtime::Runtime` を新設すると、パネルごとに worker thread が増え、
//! しかも作り直しのたびに runtime を drop することになる。
//! 実際の判定は `tokio::runtime::Handle::try_current()` で行い、
//! **reactorが無い場合は `net_provider` を渡さずに文書を組む**(panicさせない)。
//! Timeline / Inspector / chrome の4枚はHTML内に外部リソースを持たないので、
//! net provider が無くても絵は変わらない。外部リソースを持つのは Browser だけで、
//! そちらは `BrowserBlitzPanel` が自前で reactor を張る。

use std::path::PathBuf;

use blitz_dom::{DocumentConfig, StyleThreading};
use blitz_html::HtmlDocument;
use blitz_traits::shell::{ColorScheme, Viewport};

use crate::browser_blitz::render::BlitzSurface;
use crate::browser_blitz::{image_items, BrowserBlitzPanel, BrowserItem, DEFAULT_MAX_ITEMS};
use crate::chrome_blitz;
use crate::inspector_blitz;
use crate::timeline_blitz::{project_for_blitz, timeline_html};

/// 合成先textureのformat。`Rgba8UnormSrgb` にしないこと(罠1)。
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// textureを作り直す粒度(物理px)。splitterのドラッグ中はペインの大きさが毎フレーム
/// 変わるので、1px単位で `Renderer::new` をやり直すと描画資源の再確保が止まらない。
/// **絵を引き伸ばして誤魔化さない**ため、量子化するのは *textureの確保サイズ* だけで、
/// 合成は常に等倍(1 texel = 1 物理px)にする。
const TEXTURE_QUANTUM: u32 = 32;

/// Browserペインだけの量子化粒度。こちらは文書自身の大きさも量子化される
/// (`BrowserBlitzPanel` は面の大きさを変えられず、作り直すしかないため)。
/// **切り下げる** — 面より大きい文書を組んで端を切り落とすと格子が欠けるので、
/// 少し小さい面を左上に等倍で置き、余った帯は余白として残す。
const BROWSER_QUANTUM: u32 = 64;

/// Browserの画像が届き終わったと見なすまでの、枚数が増えないフレーム数。
const BROWSER_SETTLE_FRAMES: u32 = 10;

/// 計測の出力間隔(フレーム)。
const TRACE_EVERY: u32 = 60;

fn trace_enabled() -> bool {
    std::env::var_os("MOTOLII_SHELL_TRACE").is_some()
}

/// Timelineの絵にするDocument。`blitz_dump/main.rs:126-131` と同じ。
/// `Document::new_current()` は空で、投影が bar も key も出さない。
const TIMELINE_DOC: &str = "docs/mocks-ui/fixtures/reference-document.json";

/// Browserが走査するフォルダ。`blitz_dump/main.rs:175` と同じ。
const BROWSER_DIR: &str = "docs/mocks";

/// 1ペインに出す面の種類。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PaneKind {
    Stage,
    Timeline,
    Inspector,
    Browser,
    ChromeExport,
    ChromeSettings,
    ChromePanels,
}

/// Blitzパネル1面。
pub struct BlitzPane {
    kind: PaneKind,
    /// 計測用。`MOTOLII_SHELL_TRACE=1` の時だけ動く。
    trace_total: std::time::Duration,
    trace_frames: u32,
    /// egui へ渡す合成先。大きさが変わった時だけ作り直す。
    target: Option<Target>,
    content: Content,
}

impl BlitzPane {
    pub fn new(kind: PaneKind) -> Self {
        let content = match kind {
            PaneKind::Stage => Content::Stage(StagePane::default()),
            PaneKind::Browser => Content::Browser(BrowserPane::default()),
            _ => Content::Html(HtmlPane::default()),
        };
        Self {
            kind,
            trace_total: std::time::Duration::ZERO,
            trace_frames: 0,
            target: None,
            content,
        }
    }

    pub fn kind(&self) -> PaneKind {
        self.kind
    }

    /// tab に出す名前。**画面の題ではなく面の名前**なので短くする。
    pub fn title(&self) -> &'static str {
        match self.kind {
            PaneKind::Stage => "Stage",
            PaneKind::Timeline => "Timeline",
            PaneKind::Inspector => "Inspector",
            PaneKind::Browser => "Browser",
            PaneKind::ChromeExport => "Export",
            PaneKind::ChromeSettings => "Settings",
            PaneKind::ChromePanels => "Panels",
        }
    }

    /// `ui` の available rect いっぱいにこのペインの絵を描く。
    ///
    /// textureの作り直し・egui側への再登録・Blitz側のviewport更新はすべてここが持つ。
    /// マウスは受けない(`Sense::hover()`)。入力ルーティングは後続capsule。
    pub fn show(&mut self, ui: &mut egui::Ui, render_state: &eframe::egui_wgpu::RenderState) {
        let available = ui.available_size_before_wrap();
        let points_per_pixel = ui.ctx().pixels_per_point();
        // 面は物理pxで確保する。egui の point ではなく物理pxに合わせておけば、
        // 合成は等倍のまま(拡大でぼやけない)。
        // 文書のほうは **CSS px**(= 物理px / pixels_per_point)で組み、
        // `Viewport::hidpi_scale` と `BlitzSurface::set_scale` の両方へ同じ倍率を渡す。
        // 片方だけだとレイアウトと描画の縮尺がずれ、両方1.0だと Retina で全部が半分に見える。
        let width = (available.x * points_per_pixel).floor().max(0.0) as u32;
        let height = (available.y * points_per_pixel).floor().max(0.0) as u32;
        // 0サイズのtexture生成はpanicするので、その前に返す。
        if width == 0 || height == 0 {
            return;
        }

        // Stage だけは **ホストの egui へ直接描く**。以下の texture 経路には来ない。
        // `SpatialStage` は元から egui のウィジェットなので、間に自前の
        // texture と2つ目の egui を挟む理由が(ホストが egui になった今は)無い。
        if matches!(self.content, Content::Stage(_)) {
            let trace_started = trace_enabled().then(std::time::Instant::now);
            if let Content::Stage(pane) = &mut self.content {
                pane.show(ui, render_state);
            }
            if let Some(started) = trace_started {
                self.trace_frame(started.elapsed(), width, height);
            }
            return;
        }

        let (device, queue) = (&render_state.device, &render_state.queue);

        // 描く面の大きさ(物理px)。Html系は要求どおり、Browserは切り下げ。
        let (paint_width, paint_height) = match &self.content {
            // Stage は上で返しているのでここへは来ない。
            Content::Html(_) | Content::Stage(_) => (width, height),
            Content::Browser(_) => (
                floor_to(width, BROWSER_QUANTUM),
                floor_to(height, BROWSER_QUANTUM),
            ),
        };
        if paint_width == 0 || paint_height == 0 {
            return;
        }

        // 確保する texture の大きさ。Html系は切り上げて作り直しの頻度を下げ、
        // 使うのは左上の `paint_*` 分だけ(UVで切り出す)。
        let (texture_width, texture_height) = match &self.content {
            Content::Html(_) => (
                ceil_to(paint_width, TEXTURE_QUANTUM),
                ceil_to(paint_height, TEXTURE_QUANTUM),
            ),
            Content::Stage(_) | Content::Browser(_) => (paint_width, paint_height),
        };

        ensure_target(&mut self.target, render_state, texture_width, texture_height);
        // 直前に作った(か、そのまま生きている)ので必ず在る。
        // 万一無ければ今フレームは描かない — 毎フレームの経路でpanicさせない。
        let Some(target) = self.target.as_ref() else {
            return;
        };
        // `self.target` の借りは描き終わりで切る。合成で要るのは id だけなので先に写す。
        let texture_id = target.id;

        // 遅さの出所を測る窓。`MOTOLII_SHELL_TRACE=1` の時だけ働く。
        // 「重いのはどのペインか」を推測せずに言うために置いている。
        let trace_started = trace_enabled().then(std::time::Instant::now);

        match &mut self.content {
            Content::Html(pane) => {
                pane.render(
                    self.kind,
                    device,
                    queue,
                    &render_state.adapter,
                    &target.view,
                    texture_width,
                    texture_height,
                    paint_width,
                    paint_height,
                    points_per_pixel as f64,
                );
            }
            Content::Browser(pane) => {
                pane.render(
                    device,
                    queue,
                    &render_state.adapter,
                    &target.view,
                    paint_width,
                    paint_height,
                    points_per_pixel as f64,
                );
            }
            // Stage は上で返している。
            Content::Stage(_) => {}
        }

        if let Some(started) = trace_started {
            self.trace_frame(started.elapsed(), paint_width, paint_height);
        }

        // ---- egui が Blitz の texture を合成する ----
        // 場所だけ取る。`Sense::hover()` なのでクリックもドラッグも受けない。
        let (rect, _response) = ui.allocate_exact_size(available, egui::Sense::hover());
        let size = egui::vec2(
            paint_width as f32 / points_per_pixel,
            paint_height as f32 / points_per_pixel,
        );
        // 左上に等倍で置く。**引き伸ばさない** — 余りが出た場合は余白として残す。
        let draw = egui::Rect::from_min_size(rect.min, size);
        let uv = egui::Rect::from_min_max(
            egui::pos2(0.0, 0.0),
            egui::pos2(
                paint_width as f32 / texture_width as f32,
                paint_height as f32 / texture_height as f32,
            ),
        );
        ui.painter()
            .image(texture_id, draw, uv, egui::Color32::WHITE);
    }

    /// 1フレームぶんの実測を積み、`TRACE_EVERY` ごとに吐く。
    fn trace_frame(&mut self, elapsed: std::time::Duration, width: u32, height: u32) {
        self.trace_total += elapsed;
        self.trace_frames += 1;
        if self.trace_frames < TRACE_EVERY {
            return;
        }
        eprintln!(
            "shell-trace {:>16}: {:.2} ms/frame ({}x{})",
            self.title(),
            self.trace_total.as_secs_f64() * 1000.0 / TRACE_EVERY as f64,
            width,
            height
        );
        self.trace_total = std::time::Duration::ZERO;
        self.trace_frames = 0;
    }
}

/// 合成先texture1枚と、そのegui側の登録。
///
/// format は `FORMAT` 固定。Stage はホストの egui へ直接描くので、
/// この器を使うのは Blitz 面(Html / Browser)だけになった。
struct Target {
    #[allow(dead_code)]
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    id: egui::TextureId,
    width: u32,
    height: u32,
}

/// 大きさが変わっていたら作り直す。**egui側の登録も同時に更新する。**
///
/// `TextureId` は使い回す(`update_egui_texture_from_wgpu_texture`)。
/// `register_native_texture` を呼び直すとidが増え続け、古いbind groupが残る。
fn ensure_target(
    slot: &mut Option<Target>,
    render_state: &eframe::egui_wgpu::RenderState,
    width: u32,
    height: u32,
) {
    let current = slot
        .as_ref()
        .is_some_and(|target| target.width == width && target.height == height);
    if current {
        return;
    }

    let device = &render_state.device;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("motolii-blitz-pane"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        // egui へ渡すので TEXTURE_BINDING も要る(texture_host.rs:108-109)。
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    // 等倍で合成するので `Nearest`。P7 はこの組み合わせで「劣化なし」と判定している。
    let id = match slot.as_ref().map(|target| target.id) {
        Some(id) => {
            render_state
                .renderer
                .write()
                .update_egui_texture_from_wgpu_texture(
                    device,
                    &view,
                    wgpu::FilterMode::Nearest,
                    id,
                );
            id
        }
        None => render_state.renderer.write().register_native_texture(
            device,
            &view,
            wgpu::FilterMode::Nearest,
        ),
    };

    *slot = Some(Target {
        texture,
        view,
        id,
        width,
        height,
    });
}

enum Content {
    Html(HtmlPane),
    Browser(BrowserPane),
    /// Rerun Spatial Viewer の Stage。**Blitzではない。**
    /// Motolii は Viewer の wrapper であって、`re_renderer` で直接シーンを組まない
    /// (2026-08-11裁定)。ここは `rerun_stage::EmbeddedSpatialStage` を呼ぶだけ。
    /// これだけは合成先textureを持たず、ホストの egui へ直接描く。
    Stage(StagePane),
}

/// Rerun Stage 1面。**ホストの egui へ直接描く。**
///
/// `re_view_spatial::SpatialStage::show` は元から `&mut egui::Ui` を取るウィジェットで、
/// Rerun 自身も eframe のホスト egui へそのまま挿している。ホストが egui になった今、
/// 間に自前 texture と2つ目の egui context を挟む理由は無い
/// (元の `EmbeddedSpatialStage::render` は RN の native surface へ埋めるための足場)。
/// なので **`Target` を確保しない / `register_native_texture` もしない**。
#[derive(Default)]
struct StagePane {
    stage: Option<crate::rerun_stage::EmbeddedSpatialStage>,
    /// 幾何を積んだ面の大きさ(物理px)。変わったら `fit_view` をやり直す。
    fitted: (u32, u32),
}

impl StagePane {
    fn show(&mut self, ui: &mut egui::Ui, render_state: &eframe::egui_wgpu::RenderState) {
        if self.stage.is_none() {
            match crate::rerun_stage::EmbeddedSpatialStage::new_in_host(render_state) {
                Ok(stage) => self.stage = Some(stage),
                Err(error) => {
                    eprintln!("blitz-pane: Rerun Stage を作れない: {error}");
                    return;
                }
            }
        }
        let Some(stage) = self.stage.as_mut() else {
            return;
        };

        // ペインの大きさは `ui` から取る(自前textureが無いので他に出所が無い)。
        // 投影は物理px で行う — 変更前に texture の大きさを渡していたのと同じ量。
        let available = ui.available_size_before_wrap();
        let points_per_pixel = ui.ctx().pixels_per_point();
        let width = (available.x * points_per_pixel).floor().max(0.0) as u32;
        let height = (available.y * points_per_pixel).floor().max(0.0) as u32;
        if width == 0 || height == 0 {
            return;
        }

        // 幾何は既存の投影から来る。**ここで作らない。**
        // `app_stage_geometry` は `rn_product_host` 側にあり、C5(RN退役)で行き先が変わる。
        if self.fitted != (width, height) {
            let document = stage_document();
            let geometry = crate::rn_product_host::stage_projection::app_stage_geometry(
                &document,
                motolii_core::RationalTime::ZERO,
            );
            stage.apply_host_stage_geometry(&geometry, width, height);
            stage.fit_view(width, height);
            self.fitted = (width, height);
        }

        if let Err(error) = stage.show_in(ui, render_state) {
            eprintln!("blitz-pane: Rerun Stage の描画に失敗: {error}");
        }
    }
}

/// Stage が映すDocument。Timelineと同じものを読む。
fn stage_document() -> motolii_doc::Document {
    let path = PathBuf::from(TIMELINE_DOC);
    motolii_doc::load_document(&path).unwrap_or_else(|error| {
        eprintln!("blitz-pane: {} を読めないので空Documentで描く: {error}", path.display());
        motolii_doc::Document::new_current()
    })
}

/// HTML文書1枚で足りる面(Timeline / Inspector / chrome 3枚)。
#[derive(Default)]
struct HtmlPane {
    document: Option<HtmlDocument>,
    surface: Option<BlitzSurface>,
    /// 文書を組んだ面の大きさ(CSS px)。
    document_size: (u32, u32),
    /// 文書を組んだときの倍率。変わったら組み直す(CSS px の意味が変わるため)。
    document_scale: f64,
    /// `BlitzSurface` を作った texture の大きさ。
    surface_size: (u32, u32),
    /// Timelineの投影元。読み直さないように1度だけ持つ。
    source_document: Option<motolii_doc::Document>,
    /// 現在の文書とsurfaceの組で既に1回描いたか。**変わるまで描き直さない。**
    painted: bool,
}

impl HtmlPane {
    #[allow(clippy::too_many_arguments)]
    fn render(
        &mut self,
        kind: PaneKind,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        adapter: &wgpu::Adapter,
        target: &wgpu::TextureView,
        texture_width: u32,
        texture_height: u32,
        width: u32,
        height: u32,
        scale: f64,
    ) {
        // 文書は CSS px で組む。texture は物理px のまま。
        let css = (
            ((width as f64) / scale).round().max(1.0) as u32,
            ((height as f64) / scale).round().max(1.0) as u32,
        );
        if self.document.is_none() || self.document_size != css || self.document_scale != scale {
            let html = self.html(kind, css.0, css.1);
            self.document = Some(build_document(&html, width, height, scale));
            self.document_size = css;
            self.document_scale = scale;
            self.painted = false;
        }
        // **寸法が変わっただけなら作り直さない。** `BlitzSurface::resize` の doc の
        // とおり、`Renderer` は `RenderSize` を毎回受け取って自分で追随する。
        // 作り直すと画像キャッシュとグリフatlasを毎回捨てることになる。
        match self.surface.as_mut() {
            Some(surface) if self.surface_size != (texture_width, texture_height) => {
                surface.resize(texture_width, texture_height);
                self.surface_size = (texture_width, texture_height);
                self.painted = false;
            }
            Some(_) => {}
            None => {
                self.surface = Some(BlitzSurface::new(
                    device,
                    adapter,
                    queue,
                    FORMAT,
                    texture_width,
                    texture_height,
                ));
                self.surface_size = (texture_width, texture_height);
                self.painted = false;
            }
        }
        // 文書の `hidpi_scale` と同じ倍率を描画側にも渡す。片方だけだと縮尺がずれる。
        if let Some(surface) = self.surface.as_mut() {
            surface.set_scale(scale);
        }

        // **変わっていないフレームは描き直さない。**
        //
        // 以前はここが毎フレーム `paint_scene` を呼んでいた。`BlitzSurface::render` は
        // 毎回 `Scene::new` からDOM全体を起こし直す(差分もダーティ判定も無い)ので、
        // 同じ絵を作り直すために面の数だけ費用を払っていた
        // (合体シェルの実測: Timeline 42 / Inspector 49 / Export 28 ms/frame、debug)。
        //
        // texture はこのペインが所有していて他に書く者がいない。文書もsurfaceも
        // 作り直していないなら、前フレームの中身がそのまま生きている。
        //
        // 描き足りない事故が起きうるのは、blitz-paint が未取得リソースのあるフレームで
        // 何も描かずに戻る場合だけ。ここに来る4枚(Timeline / Inspector / chrome)は
        // **HTML内に外部リソースを持たない**(file冒頭の契約)ので、その事故は起きない。
        // 外部画像を持つ Browser は `BrowserPane` 側が別に面倒を見る。
        if self.painted {
            return;
        }
        if let (Some(document), Some(surface)) = (self.document.as_mut(), self.surface.as_mut()) {
            surface.render(document, device, queue, target);
            self.painted = true;
        }
    }

    /// 面の大きさ(**CSS px**)に合わせてHTMLを組む。出所はすべて既存の移植実装。
    fn html(&mut self, kind: PaneKind, width: u32, height: u32) -> String {
        let (w, h) = (width as f64, height as f64);
        let html = match kind {
            PaneKind::Timeline => {
                let document = self.timeline_document();
                let projection = project_for_blitz(document).ok();
                timeline_html(
                    document,
                    projection.as_ref(),
                    None,
                    motolii_core::RationalTime::ZERO,
                    w,
                    h,
                )
            }
            PaneKind::Inspector => inspector_blitz::inspector_html(&inspector_blitz::SAMPLE, w, h),
            PaneKind::ChromeExport => chrome_blitz::export_html(&chrome_blitz::EXPORT_SAMPLE, w, h),
            PaneKind::ChromeSettings => chrome_blitz::settings_html(w, h),
            PaneKind::ChromePanels => chrome_blitz::panels_html(w, h),
            // Browser は `BrowserPane`、Stage は `StagePane` が持つ。ここへは来ない。
            PaneKind::Browser | PaneKind::Stage => String::new(),
        };
        html
    }

    /// Timelineの投影元。読めなければ空Documentへ落とす(`blitz_dump/main.rs:134-147` と同じ)。
    fn timeline_document(&mut self) -> &motolii_doc::Document {
        self.source_document.get_or_insert_with(|| {
            let path = PathBuf::from(TIMELINE_DOC);
            match motolii_doc::load_document(&path) {
                Ok(document) => document,
                Err(error) => {
                    eprintln!(
                        "blitz-pane: {} を読めないので空Documentで描く: {error}",
                        path.display()
                    );
                    motolii_doc::Document::new_current()
                }
            }
        })
    }
}

/// Browser面。**これだけは自前のsurfaceを持つ**(`BrowserBlitzPanel` が
/// 文書・reactor・描画をひとまとめに抱える)ので、他の面と扱いが違う:
/// - `BlitzSurface` をこちらで作らない
/// - `resolve` も `BrowserBlitzPanel::render` の中で行われる(2回resolveは効かない。
///   Browserのmarkupは z-index を使わないので絵は変わらない)
/// - 面の大きさを後から変えられないので、変わったらパネルごと作り直す。
///   走査結果(`items`)は保持して再走査を避ける。
#[derive(Default)]
struct BrowserPane {
    panel: Option<BrowserBlitzPanel>,
    items: Option<Vec<BrowserItem>>,
    /// パネルを作った面の大きさ(CSS px)。
    size: (u32, u32),
    /// パネルを作ったときの倍率。変わったら作り直す(CSS px の意味が変わるため)。
    scale: f64,
    /// 画像が届き終わるまでの落ち着き待ち。**Browserだけは1回描いて終われない** —
    /// `blitz-net` の fetch が非同期で、1フレーム目ではサムネイルがまだ来ていない
    /// (`blitz_dump/main.rs:202-204` と同じ理由)。枚数が増えなくなったら描くのを止める。
    settled: u32,
    last_images: usize,
}

impl BrowserPane {
    fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        adapter: &wgpu::Adapter,
        target: &wgpu::TextureView,
        width: u32,
        height: u32,
        scale: f64,
    ) {
        // `BrowserBlitzPanel` は `width`/`height` を **CSS px** で取る
        // (ポインタ判定も同じ空間)。texture は倍率を掛けた物理px で作られる。
        let css = (
            ((width as f64) / scale).round().max(1.0) as u32,
            ((height as f64) / scale).round().max(1.0) as u32,
        );
        // **寸法が変わっただけならパネルを作り直さない**(`BrowserBlitzPanel::resize`)。
        // 作り直すと tokio runtime ごと立て直して画像を全部取り直すため、
        // ドラッグ中にサムネイルが消える。
        if self.panel.is_some() && (self.size != css || self.scale != scale) {
            if let Some(panel) = self.panel.as_mut() {
                panel.resize(css.0, css.1, scale);
            }
            self.size = css;
            self.scale = scale;
            // 文書を組み直すので、落ち着き待ちからやり直す。
            self.settled = 0;
            self.last_images = usize::MAX;
        }
        if self.panel.is_none() {
            let items = self
                .items
                .get_or_insert_with(|| image_items(&PathBuf::from(BROWSER_DIR), DEFAULT_MAX_ITEMS))
                .clone();
            let title = format!("Browser — {BROWSER_DIR}");
            // 作り直しは `BROWSER_QUANTUM` の段でしか起きない(splitterのドラッグ中に
            // 毎フレーム走らせない)。失敗したら今フレームは何も描かない。
            match BrowserBlitzPanel::new(
                device, adapter, queue, FORMAT, css.0, css.1, scale, title, items,
            ) {
                Ok(panel) => {
                    self.panel = Some(panel);
                    self.size = css;
                    self.scale = scale;
                    self.settled = 0;
                    self.last_images = usize::MAX;
                }
                Err(error) => {
                    eprintln!("blitz-pane: Browserパネルを作れない: {error}");
                    self.panel = None;
                    return;
                }
            }
        }
        if let Some(panel) = self.panel.as_mut() {
            // 落ち着いたら描き直さない。同じ絵を作り直すために毎フレーム
            // DOM全体をシーンへ起こし直す必要は無い(実測107 ms/frame、debug)。
            if self.settled >= BROWSER_SETTLE_FRAMES {
                return;
            }
            panel.render(device, queue, target);
            let images = panel.cached_image_count();
            self.settled = if images == self.last_images {
                self.settled + 1
            } else {
                0
            };
            self.last_images = images;
        }
    }
}

/// HTML1枚から `HtmlDocument` を作る。設定は `blitz_dump/main.rs:289-308` と同じ
/// (`StyleThreading::Sequential` / Dark / **2回resolve**)。
fn build_document(html: &str, width: u32, height: u32, scale: f64) -> HtmlDocument {
    // 罠3。reactor が無い場所で `Provider::shared` を呼ぶとpanicする。
    // ここで runtime を張らないのは file 冒頭の契約どおり。
    let has_reactor = tokio::runtime::Handle::try_current().is_ok();
    let mut document = HtmlDocument::from_html(
        html,
        DocumentConfig {
            style_threading: StyleThreading::Sequential,
            net_provider: if has_reactor {
                Some(blitz_net::Provider::shared(None))
            } else {
                None
            },
            ..Default::default()
        },
    );
    document.set_viewport(Viewport {
        window_size: (width, height),
        hidpi_scale: scale as f32,
        zoom: 1.0,
        color_scheme: ColorScheme::Dark,
    });
    // 2回呼ぶ(罠2)。z-index 付き要素の座標が1レイアウト遅れる。
    document.resolve(0.0);
    document.resolve(0.0);
    document
}

fn ceil_to(value: u32, quantum: u32) -> u32 {
    value.div_ceil(quantum) * quantum
}

fn floor_to(value: u32, quantum: u32) -> u32 {
    (value / quantum) * quantum
}
