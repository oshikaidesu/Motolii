//! Blitzパネル1面ぶんの器。**見た目も意味も持たない。**
//!
//! ホストは eframe(egui)。Blitz は毎フレーム、eframe が持つ wgpu device 上の
//! 自前textureへ HTML/CSS を描き、egui はそれを画像として合成する
//! — `spikes/blitz-probe/src/bin/texture_host.rs`(P7)の実走体そのままの経路。
//! 描画は `browser_blitz::render::BlitzSurface`(実証済み)を使い回す。
//! **新しい描画方式はここで作らない。**
//!
//! HTML/CSS はどれも既存の移植実装(`timeline_blitz` / `chrome_blitz`)が
//! 出したものを使う。色も寸法もこのfileでは決めない。
//!
//! **Stage は別経路。** `re_view_spatial::SpatialStage` は元から egui の
//! ウィジェットなので、ホストの `egui::Ui` へ直接挿す(`EmbeddedSpatialStage::show_in`)。
//! 自前textureも2つ目の egui context も持たない。
//!
//! **Browser / Inspector も egui native(2026-08-18 差し替え)。**
//! 見た目の正本は `docs/mocks-ui/public/browser-library.html` / `inspector-library.html`
//! で、描き手は `crate::browser_panel` / `crate::inspector_panel`。
//! Stage と同じくホストの `egui::Ui` へ直接描き、自前 texture を持たない。
//! 旧 `browser_blitz` / `inspector_blitz` は dump 器・oracle 源として残る
//! (この pane からは呼ばない)。Browser の縮小実体 cache
//! (`browser_blitz::thumbnail`)は native 側も同じものを食う。
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
//! Timeline / chrome の4枚はHTML内に外部リソースを持たないので、
//! net provider が無くても絵は変わらない(Browser / Inspector は native pane で
//! そもそも Blitz を通らない)。

use std::path::{Path, PathBuf};
use std::sync::Arc;

use blitz_dom::{DocumentConfig, StyleThreading};
use blitz_html::HtmlDocument;
use blitz_traits::shell::{ColorScheme, Viewport};

use crate::browser_blitz::render::BlitzSurface;
use crate::browser_panel::BrowserPanel;
use crate::chrome_blitz;
use crate::inspector_panel::{
    InspectorAction, InspectorEditParam, InspectorPanel, FIXTURE_TARGET_LAYER,
};
use crate::stage_frame_seat::StageFrameSeat;
use crate::timeline_blitz::{project_for_blitz, timeline_html};
use crate::timeline_rows::ParamRef;

/// 合成先textureのformat。`Rgba8UnormSrgb` にしないこと(罠1)。
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// textureを作り直す粒度(物理px)。splitterのドラッグ中はペインの大きさが毎フレーム
/// 変わるので、1px単位で `Renderer::new` をやり直すと描画資源の再確保が止まらない。
/// **絵を引き伸ばして誤魔化さない**ため、量子化するのは *textureの確保サイズ* だけで、
/// 合成は常に等倍(1 texel = 1 物理px)にする。
const TEXTURE_QUANTUM: u32 = 32;

/// 計測の出力間隔(フレーム)。
const TRACE_EVERY: u32 = 60;

fn trace_enabled() -> bool {
    std::env::var_os("MOTOLII_SHELL_TRACE").is_some()
}

/// Timelineの絵にするDocument。`blitz_dump/main.rs:126-131` と同じ。
/// `Document::new_current()` は空で、投影が bar も key も出さない。
/// Inspector の fixture read-model も同じ文書から組む(decoder P1 と同じ)。
const TIMELINE_DOC: &str = "docs/mocks-ui/fixtures/reference-document.json";

/// Inspector が出した1つの要求を、エディタの適用口へ写す。
///
/// ここには判断が無い — `InspectorEditParam` を `ParamRef` へ写し替えるだけで、
/// 「何が書けるか」「1 gesture の切れ目はどこか」はエディタ側が持つ。
fn apply_inspector_action(
    editor: &mut crate::timeline_editor::TimelineEditor,
    layer: motolii_doc::LayerId,
    action: InspectorAction,
) {
    let param = |param: InspectorEditParam| match param {
        InspectorEditParam::Position => ParamRef::Position,
        InspectorEditParam::Opacity => ParamRef::Opacity,
    };
    match action {
        InspectorAction::BeginEdit { param: p } => editor.begin_param_edit(layer, param(p)),
        InspectorAction::SetComponent {
            param: p,
            component,
            value,
        } => editor.set_param_component(layer, param(p), component, value),
        InspectorAction::EndEdit => editor.end_param_edit(),
        InspectorAction::KeyAtPlayhead { param: p, .. } => {
            editor.key_param_at_playhead(layer, param(p), action.key_components())
        }
    }
}

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
            PaneKind::Browser => Content::NativeBrowser(None),
            PaneKind::Inspector => Content::NativeInspector(None),
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

    /// live Document(編集threadの `DocumentWriter` が出した snapshot)を映す面として作る。
    ///
    /// この座席が意味を持つのは **Timeline と Stage だけ**(他の面は本レーンでは
    /// fixture のまま)。渡された `Arc` はそのまま持つ — ここで読み直しも clone もしない
    /// (single writer: 読み手は immutable snapshot)。
    pub fn with_document(kind: PaneKind, document: Arc<motolii_doc::Document>) -> Self {
        let mut pane = Self::new(kind);
        match &mut pane.content {
            Content::Html(html) if kind == PaneKind::Timeline => {
                html.source_document = Some(document);
            }
            Content::Stage(stage) => stage.document = Some(document),
            // Browser / Inspector / chrome は Document を映さない(後続レーン)。
            _ => {}
        }
        pane
    }

    pub fn kind(&self) -> PaneKind {
        self.kind
    }

    /// 編集で進んだ新しい snapshot を流し込む(single writer の reader 側)。
    /// 意味を持つのは Stage だけで、幾何は次のフレームで組み直す。
    /// live の Timeline はエディタが描くので、ここでは何も持たせない。
    pub(crate) fn set_live_document(&mut self, document: Arc<motolii_doc::Document>) {
        if let Content::Stage(stage) = &mut self.content {
            stage.document = Some(document);
            // 大きさが同じでも geometry を再適用させる。
            stage.fitted = (0, 0);
        }
    }

    /// live の Inspector を描いて、出た編集要求をそのままエディタの適用口へ渡す。
    ///
    /// **Inspector は Document を書かない**(single writer)。ここは
    /// 「投影を座らせる → 描く → 出た要求を writer へ渡す」の3手だけで、
    /// 判断も第二の Document 保持も持たない。live の Timeline がエディタ自身に
    /// 描かせているのと同じ形である。
    pub(crate) fn show_live_inspector(
        &mut self,
        ui: &mut egui::Ui,
        editor: &mut crate::timeline_editor::TimelineEditor,
    ) {
        let Content::NativeInspector(slot) = &mut self.content else {
            return;
        };
        let panel = slot.get_or_insert_with(|| InspectorPanel::placeholder("No selection"));

        // 投影は snapshot から。`Arc` を1本借りるだけで読み直さない。
        let document = Arc::clone(editor.document());
        let selection: Vec<motolii_doc::LayerId> = editor.selected_layers().to_vec();
        match editor.playhead_time() {
            Some(at) => panel.seat_live(&document, &selection, at),
            // fps の格子へ載らない playhead。理由を出して触らせない。
            None => panel.seat_live(&document, &[], motolii_core::RationalTime::ZERO),
        }

        let actions = panel.show(ui);
        let Some(&layer) = selection.first().filter(|_| selection.len() == 1) else {
            return;
        };
        for action in actions {
            apply_inspector_action(editor, layer, action);
        }
    }

    /// playhead(秒)を Stage へ置く。**Stage が出すのはこの時刻の合成フレーム**で、
    /// 編集と同じく writer 側の値をそのまま読むだけ(ここで進めない)。
    /// Stage 以外の面には意味が無い。
    pub(crate) fn set_live_playhead(&mut self, seconds: f32) {
        if let Content::Stage(stage) = &mut self.content {
            stage.playhead = seconds;
        }
    }

    /// テストの検分用: この面に座らせた live Document。fixture 動線では `None`。
    #[cfg(test)]
    pub(crate) fn live_document(&self) -> Option<&Arc<motolii_doc::Document>> {
        match &self.content {
            Content::Html(html) => html.source_document.as_ref(),
            Content::Stage(stage) => stage.document.as_ref(),
            Content::NativeBrowser(_) | Content::NativeInspector(_) => None,
        }
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

        // Stage / Browser / Inspector は **ホストの egui へ直接描く**。
        // 以下の texture 経路には来ない。Stage は元から egui のウィジェット、
        // Browser / Inspector は 2026-08-18 に egui native へ差し替えた
        // (`crate::browser_panel` / `crate::inspector_panel`)。
        match &mut self.content {
            Content::Stage(_) | Content::NativeBrowser(_) | Content::NativeInspector(_) => {
                let trace_started = trace_enabled().then(std::time::Instant::now);
                match &mut self.content {
                    Content::Stage(pane) => pane.show(ui, render_state),
                    Content::NativeBrowser(panel) => {
                        // 初回表示時に作る(フォルダ走査と縮小実体の準備を、
                        // 窓を開く前に払わない)。
                        panel
                            .get_or_insert_with(BrowserPanel::default_shell)
                            .show(ui);
                    }
                    Content::NativeInspector(panel) => {
                        // 座席が無いとき(fixture 展示)の Inspector。live の Inspector は
                        // `show_live_inspector` が描く — 編集要求を writer へ渡すために
                        // エディタと同じ場所で回す必要がある。
                        let _ = panel
                            .get_or_insert_with(|| {
                                InspectorPanel::from_document_path(
                                    Path::new(TIMELINE_DOC),
                                    FIXTURE_TARGET_LAYER,
                                )
                            })
                            .show(ui);
                    }
                    Content::Html(_) => unreachable!("checked above"),
                }
                if let Some(started) = trace_started {
                    self.trace_frame(started.elapsed(), width, height);
                }
                return;
            }
            Content::Html(_) => {}
        }

        let (device, queue) = (&render_state.device, &render_state.queue);

        let (paint_width, paint_height) = (width, height);

        // 確保する texture の大きさ。Html系は切り上げて作り直しの頻度を下げ、
        // 使うのは左上の `paint_*` 分だけ(UVで切り出す)。
        let (texture_width, texture_height) = (
            ceil_to(paint_width, TEXTURE_QUANTUM),
            ceil_to(paint_height, TEXTURE_QUANTUM),
        );

        ensure_target(
            &mut self.target,
            render_state,
            texture_width,
            texture_height,
        );
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
            // native 系は上で返している。
            Content::Stage(_) | Content::NativeBrowser(_) | Content::NativeInspector(_) => {}
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
/// format は `FORMAT` 固定。Stage / Browser / Inspector はホストの egui へ
/// 直接描くので、この器を使うのは Blitz 面(Html)だけになった。
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
    /// Browser native pane(`crate::browser_panel`)。初回表示時に作る
    /// (フォルダ走査と縮小実体の準備を、窓を開く前に払わない)。
    NativeBrowser(Option<BrowserPanel>),
    /// Inspector native pane(`crate::inspector_panel`)。同上。
    NativeInspector(Option<InspectorPanel>),
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
    /// 幾何を積んだ時刻。**絵(合成フレーム)と同じ時刻**でなければ、枠だけが
    /// t=0 に残って絵とズレる。`None` はまだ一度も積んでいない。
    framed_at: Option<motolii_core::RationalTime>,
    /// 最後に Stage へ積んだ幾何。**同じものを積み直さない** — 積むたびに
    /// Rerun の recording へ chunk が増えるので、動かない layer を再生中に
    /// 毎フレーム積むと store が伸び続ける。
    applied_geometry: Option<crate::stage_app_geometry::AppStageGeometry>,
    /// live Document(writer の snapshot)。`None` なら fixture を読む(従来動線)。
    document: Option<Arc<motolii_doc::Document>>,
    /// playhead(秒)。live のとき app が毎フレーム置く。
    playhead: f32,
    /// playhead 時刻の合成フレームの席。**live のときだけ立つ。**
    frame_seat: Option<StageFrameSeat>,
    /// 席を立てられなかった(GPU/worker)。理由は1度だけ言い、以後は幾何だけで出す。
    frame_seat_failed: bool,
}

impl StagePane {
    /// 合成フレームを出す面か。**live Document が座っている時だけ。**
    /// fixture 展示(`--project` 無し)は幾何だけの従来表示のままにする。
    fn wants_evaluated_frame(&self) -> bool {
        self.document.is_some()
    }

    /// 幾何を積む時刻。**合成フレームと同じ格子**へ載せた playhead。
    ///
    /// live Document が座っていない fixture 展示は playhead を持たないので t=0。
    /// composition の fps / duration から格子が出せない時も t=0 へ落とす
    /// (枠を消すより、絵の無い時刻の枠を出すほうが害が小さい)。
    fn geometry_time(&self) -> motolii_core::RationalTime {
        self.document
            .as_deref()
            .and_then(|document| {
                crate::stage_frame_seat::snap_to_frame(
                    self.playhead,
                    document.composition.fps,
                    document.composition.duration,
                )
            })
            .unwrap_or(motolii_core::RationalTime::ZERO)
    }

    /// 席を立て、playhead 時刻を要求し、届いていれば受ける。
    /// 戻すものは無い — 出す1枚は `frame_seat.frame()` から取る。
    fn drive_frame_seat(
        &mut self,
        render_state: &eframe::egui_wgpu::RenderState,
        ctx: &egui::Context,
    ) {
        if !self.wants_evaluated_frame() {
            return;
        }
        if self.frame_seat.is_none() && !self.frame_seat_failed {
            // **ホストと同じ device** で評価する。完成 texture は Rerun の
            // texture pool へ実 handle のまま渡るので、別 device では import できない。
            let gpu = Arc::new(motolii_gpu::GpuCtx::from_device_queue(
                render_state.device.clone(),
                render_state.queue.clone(),
            ));
            match StageFrameSeat::spawn(gpu) {
                Ok(mut seat) => {
                    // 完成は別 thread で来る。入力が無くても塗り直させる。
                    let repaint = ctx.clone();
                    seat.on_frame_ready(move || repaint.request_repaint());
                    self.frame_seat = Some(seat);
                }
                Err(error) => {
                    self.frame_seat_failed = true;
                    eprintln!("blitz-pane: Stage の合成フレーム席を立てられない: {error}");
                }
            }
        }
        let Some(document) = self.document.clone() else {
            return;
        };
        let Some(seat) = self.frame_seat.as_mut() else {
            return;
        };
        seat.request(&document, self.playhead);
        seat.poll();
        if let Some(error) = seat.take_error() {
            eprintln!("blitz-pane: Stage の合成に失敗: {error}");
        }
    }

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

        // 合成フレームの席は Stage を借りる前に回す(借りが割れないように)。
        let ctx = ui.ctx().clone();
        self.drive_frame_seat(render_state, &ctx);

        let evaluated_frame = self.frame_seat.as_ref().and_then(StageFrameSeat::frame);
        // 幾何を積む時刻も Stage を借りる前に決める(同じ理由)。
        let framed_at = self.geometry_time();
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
        //
        // 時刻は絵と同じ格子へ載せる。`stage_frame_seat` が合成フレームを
        // `snap_to_frame` の時刻で作るので、枠も同じ関数を通す
        // (別の丸めをすると1フレームぶん枠だけ先走る)。
        let resized = self.fitted != (width, height);
        if resized || self.framed_at != Some(framed_at) {
            // live(`--project`)なら writer の snapshot をそのまま読む(single writer の
            // reader 側)。座っていなければ従来どおり fixture。
            let fixture;
            let document: &motolii_doc::Document = match &self.document {
                Some(live) => live,
                None => {
                    fixture = stage_document();
                    &fixture
                }
            };
            let geometry = crate::stage_app_geometry::app_stage_geometry(document, framed_at);
            if resized || self.applied_geometry.as_ref() != Some(&geometry) {
                stage.apply_host_stage_geometry(&geometry, width, height);
                self.applied_geometry = Some(geometry);
            }
            // カメラの掛け直しは面の大きさが変わった時だけ。playhead が動くたびに
            // 視点を戻すと、再生中にカメラが跳ねる。
            if resized {
                stage.fit_view(width, height);
            }
            self.fitted = (width, height);
            self.framed_at = Some(framed_at);
        }

        if let Err(error) = stage.show_in(ui, render_state, evaluated_frame) {
            eprintln!("blitz-pane: Rerun Stage の描画に失敗: {error}");
        }
    }
}

/// Stage が映すDocument。Timelineと同じものを読む。
fn stage_document() -> motolii_doc::Document {
    let path = PathBuf::from(TIMELINE_DOC);
    motolii_doc::load_document(&path).unwrap_or_else(|error| {
        eprintln!(
            "blitz-pane: {} を読めないので空Documentで描く: {error}",
            path.display()
        );
        motolii_doc::Document::new_current()
    })
}

/// HTML文書1枚で足りる面(Timeline / chrome 3枚)。
#[derive(Default)]
struct HtmlPane {
    document: Option<HtmlDocument>,
    surface: Option<BlitzSurface>,
    /// 最後に文書へ渡した viewport。寸法変更は文書の再 parse ではなくここを更新する。
    viewport: Option<(u32, u32, f64)>,
    /// `BlitzSurface` を作った texture の大きさ。
    surface_size: (u32, u32),
    /// Timelineの投影元。live(`--project`)なら構築時に writer の snapshot が座り、
    /// fixture 動線なら初回描画で1度だけ読んでここに持つ。
    source_document: Option<Arc<motolii_doc::Document>>,
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
        self.ensure_document(kind, width, height, scale);
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
        // 何も描かずに戻る場合だけ。ここに来る4枚(Timeline / chrome 3枚)は
        // **HTML内に外部リソースを持たない**(file冒頭の契約)ので、その事故は起きない。
        // (外部画像を持つ Browser は native pane で、この経路を通らない。)
        if self.painted {
            return;
        }
        if let (Some(document), Some(surface)) = (self.document.as_mut(), self.surface.as_mut()) {
            surface.render(document, device, queue, target);
            self.painted = true;
        }
    }

    fn ensure_document(&mut self, kind: PaneKind, width: u32, height: u32, scale: f64) {
        let viewport = (width, height, scale);
        if self.document.is_none() {
            let html = self.html(kind);
            let mut document = build_document(&html, width, height, scale);
            // Timeline の clip と key は DOM に無い(`timeline_blitz/surface.rs`)。
            // ここで挿さないと、行と目盛だけの Timeline になる。
            if kind == PaneKind::Timeline {
                let source = self.timeline_document().clone();
                let projection = project_for_blitz(&source).ok();
                if !crate::timeline_blitz::attach_surface(
                    &mut document,
                    &source,
                    projection.as_ref(),
                    None,
                ) {
                    eprintln!("blitz-pane: #tl-surface が見つからず Timeline の面を挿せない");
                }
            }
            self.document = Some(document);
            self.viewport = Some(viewport);
            self.painted = false;
        } else if self.viewport != Some(viewport) {
            let document = self.document.as_mut().expect("checked above");
            document.set_viewport(blitz_viewport(width, height, scale));
            // z-index 付き要素の座標が1レイアウト遅れるため、初回構築時と同じく2回解く。
            document.resolve(0.0);
            document.resolve(0.0);
            self.viewport = Some(viewport);
            self.painted = false;
        }
    }

    /// 面の構造を一度だけ組む。寸法は Document の viewport が持つ。
    fn html(&mut self, kind: PaneKind) -> String {
        let html = match kind {
            PaneKind::Timeline => {
                let document = self.timeline_document();
                let projection = project_for_blitz(document).ok();
                timeline_html(
                    document,
                    projection.as_ref(),
                    None,
                    motolii_core::RationalTime::ZERO,
                )
            }
            PaneKind::ChromeExport => chrome_blitz::export_html(&chrome_blitz::EXPORT_SAMPLE),
            PaneKind::ChromeSettings => chrome_blitz::settings_html(),
            PaneKind::ChromePanels => chrome_blitz::panels_html(),
            // Browser / Inspector は native pane、Stage は `StagePane`。ここへは来ない。
            PaneKind::Browser | PaneKind::Inspector | PaneKind::Stage => String::new(),
        };
        html
    }

    /// Timelineの投影元。live が座っていればそれ。無ければ fixture を読み、
    /// 読めなければ空Documentへ落とす(`blitz_dump/main.rs:134-147` と同じ)。
    /// fixture の fallback はここ(表示専用の動線)だけで、`--project` の失敗は
    /// ここへ来る前に起動失敗になっている(`app.rs` の `ProjectSeat::open`)。
    fn timeline_document(&mut self) -> &Arc<motolii_doc::Document> {
        self.source_document.get_or_insert_with(|| {
            let path = PathBuf::from(TIMELINE_DOC);
            Arc::new(match motolii_doc::load_document(&path) {
                Ok(document) => document,
                Err(error) => {
                    eprintln!(
                        "blitz-pane: {} を読めないので空Documentで描く: {error}",
                        path.display()
                    );
                    motolii_doc::Document::new_current()
                }
            })
        })
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
    document.set_viewport(blitz_viewport(width, height, scale));
    // 2回呼ぶ(罠2)。z-index 付き要素の座標が1レイアウト遅れる。
    document.resolve(0.0);
    document.resolve(0.0);
    document
}

fn blitz_viewport(width: u32, height: u32, scale: f64) -> Viewport {
    Viewport {
        window_size: (width, height),
        hidpi_scale: scale as f32,
        zoom: 1.0,
        color_scheme: ColorScheme::Dark,
    }
}

fn ceil_to(value: u32, quantum: u32) -> u32 {
    value.div_ceil(quantum) * quantum
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Browser / Inspector は Blitz texture 面ではなく native pane に座る
    /// (2026-08-18 差し替えの見張り)。
    #[test]
    fn browser_and_inspector_panes_are_native() {
        assert!(matches!(
            BlitzPane::new(PaneKind::Browser).content,
            Content::NativeBrowser(_)
        ));
        assert!(matches!(
            BlitzPane::new(PaneKind::Inspector).content,
            Content::NativeInspector(_)
        ));
        // Timeline / chrome は従来どおり Blitz(Html)面。
        for kind in [
            PaneKind::Timeline,
            PaneKind::ChromeExport,
            PaneKind::ChromeSettings,
            PaneKind::ChromePanels,
        ] {
            assert!(matches!(BlitzPane::new(kind).content, Content::Html(_)));
        }
    }

    /// **合成フレームの経路は live project の時だけ**動く。
    /// `--project` 無しの fixture 展示は幾何だけの従来表示のまま
    /// （席を立てず、render worker も GpuCtx も作らない）。
    #[test]
    fn the_evaluated_frame_path_only_runs_for_a_live_project() {
        let fixture = BlitzPane::new(PaneKind::Stage);
        let Content::Stage(stage) = &fixture.content else {
            panic!("Stage pane");
        };
        assert!(!stage.wants_evaluated_frame());
        assert!(stage.frame_seat.is_none());

        let live = BlitzPane::with_document(
            PaneKind::Stage,
            Arc::new(motolii_doc::Document::new_current()),
        );
        let Content::Stage(stage) = &live.content else {
            panic!("Stage pane");
        };
        assert!(stage.wants_evaluated_frame());
        assert!(
            stage.frame_seat.is_none(),
            "席は最初の描画で立てる（窓を開く前に GPU を掴まない）"
        );
    }

    /// playhead は Stage にだけ座る（他の面はこの値を知らない）。
    #[test]
    fn the_playhead_seats_only_on_the_stage() {
        let mut stage = BlitzPane::new(PaneKind::Stage);
        stage.set_live_playhead(2.5);
        let Content::Stage(pane) = &stage.content else {
            panic!("Stage pane");
        };
        assert_eq!(pane.playhead, 2.5);

        // 他の面は同じ入口を素通りする（panic も状態も持たない）。
        for kind in [PaneKind::Timeline, PaneKind::Browser, PaneKind::Inspector] {
            let mut pane = BlitzPane::new(kind);
            pane.set_live_playhead(2.5);
        }
    }

    /// **枠は絵と同じ時刻に立つ。** playhead が動けば layer 枠を積み直す時刻も動く。
    /// ここが t=0 固定だと、絵だけ playhead に追従して枠が置き去りになる。
    #[test]
    fn the_stage_geometry_time_follows_the_playhead() {
        let mut document = motolii_doc::Document::new_current();
        document.composition.duration = motolii_core::RationalTime::from_seconds(2);
        let fps = document.composition.fps;
        let mut stage = BlitzPane::with_document(PaneKind::Stage, Arc::new(document));

        stage.set_live_playhead(1.0);
        let Content::Stage(pane) = &stage.content else {
            panic!("Stage pane");
        };
        let at_one = pane.geometry_time();
        assert_eq!(
            at_one,
            motolii_core::RationalTime::from_seconds(1),
            "playhead 1秒の枠が t=1 に立っていない"
        );

        stage.set_live_playhead(0.0);
        let Content::Stage(pane) = &stage.content else {
            panic!("Stage pane");
        };
        assert_eq!(pane.geometry_time(), motolii_core::RationalTime::ZERO);

        // 合成フレームと同じ格子(`stage_frame_seat::snap_to_frame`)へ載る。
        let one_frame = 1.0 / (fps.num() as f32 / fps.den() as f32);
        stage.set_live_playhead(one_frame * 0.4);
        let Content::Stage(pane) = &stage.content else {
            panic!("Stage pane");
        };
        assert_eq!(
            pane.geometry_time(),
            motolii_core::RationalTime::ZERO,
            "フレーム格子へ丸めていない"
        );
    }

    /// fixture 展示(live Document 無し)は playhead を持たないので t=0 のまま。
    #[test]
    fn the_fixture_stage_stays_at_time_zero() {
        let mut stage = BlitzPane::new(PaneKind::Stage);
        stage.set_live_playhead(1.5);
        let Content::Stage(pane) = &stage.content else {
            panic!("Stage pane");
        };
        assert_eq!(pane.geometry_time(), motolii_core::RationalTime::ZERO);
    }

    #[test]
    fn resizing_html_panes_reuses_the_document() {
        for kind in [
            PaneKind::Timeline,
            PaneKind::ChromeExport,
            PaneKind::ChromeSettings,
            PaneKind::ChromePanels,
        ] {
            let mut pane = HtmlPane::default();
            pane.ensure_document(kind, 640, 480, 1.0);
            let before = pane.document.as_ref().expect("initial document") as *const HtmlDocument;

            pane.ensure_document(kind, 960, 640, 1.0);

            assert_eq!(
                before,
                pane.document.as_ref().expect("resized document") as *const HtmlDocument,
                "{kind:?} recreated its HTML document during resize"
            );
            assert_eq!(pane.viewport, Some((960, 640, 1.0)));
        }
    }
}
