//! wraps: makepad_widgets の app host。面は `script_mod!` の宣言、再読込は makepad の
//! live reload(`--hot`)、窓の駆動は `--remote`。**ここに再読込機構を書き足さない。**
//! かつて `HotPanel` + `panel.splash` + 120ms タイマーで自前に作られており、makepad が
//! `Event::LiveEdit` で `script_mod!` を再実行すると面が空に戻る欠陥になっていた
//! (2026-08-27 撤去)。作り足すなら `wraps:` を `owns:` へ書き換えること — それが
//! 「上流に無い」という主張であり、`check.sh` の一覧に出て初めてレビューできる。

pub use makepad_widgets;

use makepad_widgets::*;
use motolii_engine::Engine;
use motolii_shell_state::Session;
use motolii_store::{Document, Intent, LayerAttrsPatch, LayerId, RationalTime};
use motolii_timeline_projection::{self as timeline_pane, stacking::restacked, StackDirection};
use std::time::Instant;

mod browser_surface;
mod theme_flat;
mod tokens;
mod chrome;
mod export_surface;
mod gesture_input;
mod inspector_surface;
mod settings_surface;
mod stage_chrome;
mod stage_import;
mod stage_surface;
mod timeline_surface;
use stage_surface::{SharedOsHandle, SharedSurfaceDesc, StagePresent, StageRoom, StageVerdict};
use timeline_surface::{
    LaneFlag, TimelineLane, TimelineModel, TimelinePropertyLane, TimelineSurface,
    TimelineSurfaceAction,
};

app_main!(App);

/// `browser_radio_groups` の `ids_array!` と同じ並び。索引が意味を運ぶので離さない。
/// `rail` の radio group。並びは `RAIL_HEADS` と対。`add_folder` は選択ではなく操作なので入れない。
macro_rules! browser_rail_ids {
    () => {
        ids_array!(
            browser_surface.browser_body.rail.all_media,
            browser_surface.browser_body.rail.video,
            browser_surface.browser_body.rail.images,
            browser_surface.browser_body.rail.audio,
            browser_surface.browser_body.rail.project,
            browser_surface.browser_body.rail.recent
        )
    };
}

const RAIL_ALL_MEDIA: usize = 0;




script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // Source: Motolii next/reference/mocks/*.html (visual/semantic contract only).
    // This is a Makepad proof surface, not a second product state owner.

    let IconButton = ButtonFlatterIcon{
        margin: 0
        width: mod.tokens.size.menu
        height: mod.tokens.size.bar
        icon_walk: Walk{width: mod.tokens.size.icon height: mod.tokens.size.icon}
        padding: 0
        draw_icon +: {color: mod.tokens.ink.body}
    }

    let IconFlatButton = ButtonFlatIcon{
        margin: 0
        width: mod.tokens.size.menu
        height: mod.tokens.size.bar
        icon_walk: Walk{width: mod.tokens.size.icon height: mod.tokens.size.icon}
        padding: 0
        draw_icon +: {color: mod.tokens.ink.body}
    }

    // 窓の大きな面の境も同じ接地の文法: 暗い溝 + 次の面の上縁の光(Live 実測)
    let SurfaceDivider = SolidView{
        width: Fill
        height: mod.tokens.space.s2
        flow: Down
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.face.desktop
        fill := View{width: Fill height: Fill}
        rim := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true new_batch: true draw_bg.color: mod.tokens.rule.rim}
    }

    // Timeline の面(TimeField / TimelineRow / RailToggle 等)はここから撤去した。
    // 描画は `timeline_surface.rs` の Rust 側へ移っており、この宣言群は誰も参照して
    // いなかった — 同じ意味の正本が2つあると、片方だけ直して食い違う。

    // Studio: kinds are Fill Views, then `Kind := Kind {}` on the Dock instance.
    let BrowserPane = View{
        width: Fill
        height: Fill
        browser_surface := BrowserSurface{}
    }

    let StagePane = View{
        width: Fill
        height: Fill
        stage_chrome := StageChrome{}
    }

    let InspectorPane = View{
        width: Fill
        height: Fill
        inspector_surface := InspectorSurface{}
    }

    let ExportPane = View{
        width: Fill
        height: Fill
        export_surface := ExportSurface{}
    }

    let SettingsPane = View{
        width: Fill
        height: Fill
        settings_surface := SettingsSurface{}
    }

    let ChromePane = View{
        width: Fill
        height: Fill
        chrome_gallery := ChromeGallery{}
    }

    let TimelinePane = View{
        width: Fill
        height: Fill
        flow: Down
        transport := SolidView{
            width: Fill
            height: mod.tokens.size.transport
            flow: Right
            align: Align{x: 0.5 y: 0.5}
            show_bg: true
            new_batch: true
            draw_bg.color: mod.tokens.face.panel
            play_toggle := ButtonFlatIcon{
                width: 26.0 * mod.tokens.scale
                height: mod.tokens.size.status
                icon_walk: Walk{width: mod.tokens.size.icon height: mod.tokens.size.icon}
                padding: Inset{left: 0 right: 0}
                draw_bg.color: #4f4f4f
                draw_bg.border_size: 0.0
                draw_icon +: {svg: crate_resource("self://resources/icons/play.svg") color: mod.tokens.accent.on}
            }
        }
        timeline_surface := TimelineSurface{
            width: Fill
            height: Fill
        }
    }


    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.inner_size: vec2(1440, 900)
                window.title: "Motolii Makepad Panel"
                body +: {
                    panel := SolidView{
        width: Fill
        height: Fill
        flow: Down
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.face.well

            chrome := SolidView{
                width: Fill
                height: mod.tokens.size.chrome
                flow: Right
                align: Align{y: 0.5}
                padding: Inset{left: mod.tokens.space.s5 right: mod.tokens.space.s5}
                show_bg: true
                new_batch: true
                draw_bg +: { color: mod.tokens.face.area }

                brand := SolidView{
                    width: Fit
                    height: mod.tokens.size.menu
                    flow: Right
                    align: Align{y: 0.5}
                    spacing: mod.tokens.space.s3
                    mark := Icon{width: mod.tokens.size.icon_lg height: mod.tokens.size.icon_lg icon_walk: Walk{width: mod.tokens.size.icon_lg height: mod.tokens.size.icon_lg} draw_icon +: {svg: crate_resource("self://resources/icons/motolii.svg") color: mod.tokens.ink.body}}
                    name := InkLabel{text: "MOTOLII" width: Fit padding: Inset{right: mod.tokens.space.s3} draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_bold{font_size: mod.tokens.text.lg}}
                }
                file := ButtonFlatter{text: "File" width: 42.0 * mod.tokens.scale height: mod.tokens.size.menu draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.md}}
                edit := ButtonFlatter{text: "Edit" width: 42.0 * mod.tokens.scale height: mod.tokens.size.menu draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.md}}
                layer := ButtonFlatter{text: "Layer" width: 50.0 * mod.tokens.scale height: mod.tokens.size.menu draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.md}}
                view := ButtonFlatter{text: "View" width: 42.0 * mod.tokens.scale height: mod.tokens.size.menu draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.md}}
                spacer := SolidView{width: Fill height: 1}
                project := InkLabel{
                    text: "Untitled / Motion Study"
                    width: Fit
                    draw_text.color: mod.tokens.ink.glyph
                    draw_text.text_style: theme.font_code{font_size: mod.tokens.text.md}
                }
                // アイコンの言っている事をそのまま行う2本。splash の `on_click` からは
                // 届かない — Dock に `script_call` の口が無いのでタブ選択も splitter も
                // script 側から呼べず、状態行へ文言を書くだけの「見た目のボタン」に
                // なっていた。押下は Rust 側(`App::handle_actions`)で受ける
                // (`toggle_playback` / `radio_button_set` と同じ経路)
                browser_toggle := IconButton{width: 26.0 * mod.tokens.scale draw_icon +: {svg: crate_resource("self://resources/icons/panels.svg")}}
                settings := IconButton{width: 26.0 * mod.tokens.scale draw_icon +: {svg: crate_resource("self://resources/icons/filter.svg")}}
            }

            chrome_surface_divider := SurfaceDivider{}

            dock := DockFlat{
                width: Fill
                height: Fill
                // 境界は「同値の面の間の暗線」ではなく「暗い溝 + 次のパネルの縁の光」
                // (Live 実測: 面 #3e3f3c → 溝 ~5pt #2b2b29 → 明縁 1-2px #adaeae → 面)。
                // 溝の深さと縁のハイライトが板を接地させる — 暗線だけだと浮いて見える
                splitter: Splitter{
                    draw_bg +: {
                        color_bg: mod.tokens.face.desktop
                        pixel: fn() {
                            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                            sdf.clear(self.color_bg)
                            // 明縁は「後ろ側」= 次のパネルの先頭端。is_vertical>0.5 は
                            // 縦棒(列を割る = 横方向の splitter)なので rect_size.x が
                            // 細い側 — 縁は右端(次パネルの左端)に立てる。
                            // else は横棒(行を割る)で rect_size.y が細い側 — 縁は下端。
                            // (以前はこの2本が入れ替わっており、縁が細い辺の外へ
                            // 落ちて可視スキャン線に一切かからなかった — それが
                            // 「override が効いていない」ように見えた本当の原因)
                            // SDF のアンチエイリアスは幅1の矩形だと両端のぼかしが
                            // 中央で重なり、塗りが半分以下しか乗らない(実測:
                            // #2a2a2a→#9c9c9c のはずが #525252 止まりだった)。
                            // 縁だけ aa を締めてハードエッジに近づける
                            sdf.aa = sdf.aa * 4.0
                            if self.is_vertical > 0.5 {
                                sdf.rect(self.rect_size.x - 1.0, 0.0, 1.0, self.rect_size.y)
                            } else {
                                sdf.rect(0.0, self.rect_size.y - 1.0, self.rect_size.x, 1.0)
                            }
                            // rule.rim #x9c9c9c。shader 本体は mod.* を解決できないので
                            // リテラルで持つ(値の正本は tokens 側)
                            sdf.fill(vec4(0.612, 0.612, 0.612, 1.0))
                            sdf.aa = sdf.aa * 0.25
                            // 掴みの棒は既定のまま(hover まで不可視)
                            if self.is_vertical > 0.5 {
                                sdf.box(self.splitter_pad, self.rect_size.y * 0.5 - self.bar_size * 0.5, self.rect_size.x - 2.0 * self.splitter_pad, self.bar_size, self.border_radius)
                            } else {
                                sdf.box(self.rect_size.x * 0.5 - self.bar_size * 0.5, self.splitter_pad, self.bar_size, self.rect_size.y - 2.0 * self.splitter_pad, self.border_radius)
                            }
                            return sdf.fill_keep(mix(self.color, mix(self.color_hover, self.color_drag, self.drag), self.hover))
                        }
                    }
                }
                // makepad の丸角オーバーレイは makepad の顔。Ableton は直角
                round_corner.border_radius: 0.0
                // 既定の 33pt は、この密度の中では帯だけが太い。タブは掴む所なので
                // 消さずに詰める(makepad 側の下限は 25pt)
                tab_bar: TabBarFlat{
                    height: mod.tokens.size.tab_bar
                    // 帯の面は完全に透明。浮くのはタブの札だけで、下の chrome は読めたまま。
                    // 既定は `color_2: #0000` へのグラデーションで、下の文字が中途半端に
                    // 抜ける — 一番読みにくい状態なので、不透明か透明かに振り切る。
                    // ドロップ判定は幾何(`is_over_tab_bar`)なので見えなくても効く
                    draw_bg.color: #x00000000
                    // 既定の tab は帯より高く(36 > 25)、下がはみ出て切れる。
                    // align.y は元から中央なので、直すのは箱の高さ
                    PermanentTab := TabFlat{
                        height: mod.tokens.size.tab_bar
                        padding: Inset{left: mod.tokens.space.s5 right: mod.tokens.space.s5}
                        draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}
                    }
                }

                root := DockSplitter{
                    axis: SplitterAxis.Vertical
                    align: SplitterAlign.FromB(mod.tokens.size.pane)
                    a: @top_split
                    b: @timeline_tabs
                }

                top_split := DockSplitter{
                    axis: SplitterAxis.Horizontal
                    align: SplitterAlign.FromA(mod.tokens.size.pane)
                    a: @browser_tabs
                    b: @center_split
                }

                center_split := DockSplitter{
                    axis: SplitterAxis.Horizontal
                    align: SplitterAlign.FromB(mod.tokens.size.pane)
                    a: @stage_tabs
                    b: @inspector_tabs
                }

                browser_tabs := DockTabs{
                    tabs: [@browser]
                    selected: 0
                    closable: false
                    float_tab_bar: false
                }

                stage_tabs := DockTabs{
                    tabs: [@stage]
                    selected: 0
                    closable: false
                    float_tab_bar: false
                }

                inspector_tabs := DockTabs{
                    tabs: [@inspector @export @settings @chrome_tab]
                    selected: 0
                    closable: false
                    float_tab_bar: false
                }

                timeline_tabs := DockTabs{
                    tabs: [@timeline]
                    selected: 0
                    closable: false
                    float_tab_bar: false
                }

                browser := DockTab{
                    name: "Browser"
                    template: @PermanentTab
                    kind: @BrowserPane
                }

                stage := DockTab{
                    name: "Stage"
                    template: @PermanentTab
                    kind: @StagePane
                }

                inspector := DockTab{
                    name: "Inspector"
                    template: @PermanentTab
                    kind: @InspectorPane
                }

                export := DockTab{
                    name: "Export"
                    template: @PermanentTab
                    kind: @ExportPane
                }

                settings := DockTab{
                    name: "Settings"
                    template: @PermanentTab
                    kind: @SettingsPane
                }

                chrome_tab := DockTab{
                    name: "Chrome"
                    template: @PermanentTab
                    kind: @ChromePane
                }

                timeline := DockTab{
                    name: "Timeline"
                    template: @PermanentTab
                    kind: @TimelinePane
                }

                BrowserPane := BrowserPane{}
                StagePane := StagePane{}
                InspectorPane := InspectorPane{}
                ExportPane := ExportPane{}
                SettingsPane := SettingsPane{}
                ChromePane := ChromePane{}
                TimelinePane := TimelinePane{}
            }

            status_surface_divider := SurfaceDivider{}

            status := InkLabel{
                text: "READY  ·  RERUN STAGE  ·  FRAME 900 / 1800"
                width: Fill
                height: mod.tokens.size.status
                padding: Inset{left: mod.tokens.space.s5}
                draw_text.color: mod.tokens.ink.faint
                draw_text.text_style: theme.font_code{font_size: mod.tokens.text.sm}
            }

                    }
                }
            }
        }
    }
}

/// Makepad view adapter. Writes go to Document / Session; pixels come from Engine.
/// widgets never keep a second Document.
struct BackendBridge {
    doc: Document,
    session: Session,
    engine: Engine,
    frame: Option<(u32, u32, Vec<u8>)>,
    status: Option<String>,
    playing: bool,
    present: StagePresent,
    stage_texture: Option<Texture>,
    stage_gpu: Option<wgpu::Texture>,
}

enum TimelineUpdate {
    None,
    Stage(String),
    ModelAndStage(String),
    Status(String),
}

/// A one-slot mailbox for expensive projections. Producers may run at pointer
/// frequency; the consumer runs at display frequency and only observes the
/// newest request. The payload can later become a `StageSurfaceSlot` without
/// changing Timeline or App event routing.
#[derive(Default)]
struct LatestFrameRequest {
    pending: bool,
}

impl LatestFrameRequest {
    fn request(&mut self) -> bool {
        std::mem::replace(&mut self.pending, true)
    }

    fn take(&mut self) -> bool {
        std::mem::take(&mut self.pending)
    }
}

impl BackendBridge {
    fn new_fixture() -> Self {
        let built = motolii_fixture::build();
        Self {
            session: Session {
                playhead: built.playhead,
                selection: Some(built.selected),
                selected_layers: vec![built.selected],
                ..Session::default()
            },
            doc: built.doc,
            engine: Engine::new().expect("GPU を用意できない"),
            frame: None,
            status: Some(built.status),
            playing: false,
            present: StagePresent::FallbackCpu,
            stage_texture: None,
            stage_gpu: None,
        }
    }

    fn display_name(name: &str) -> String {
        match name {
            "タイトルロゴ" => "Title Logo",
            "メインボーカル映像" => "Main Vocal",
            "Bロール_街並み" => "B-roll City",
            "1番Aメロ歌詞" => "Verse A Lyrics",
            "Bメロ歌詞" => "Pre-chorus Lyrics",
            "ダンスカット" => "Dance Cut",
            "サビ歌詞" => "Chorus Lyrics",
            "グリッチトランジション" => "Glitch Transition",
            "2番Aメロ歌詞" => "Verse 2 Lyrics",
            "波形ビジュアライザ" => "Wave Visualizer",
            "リリックモーション背景" => "Lyric Motion BG",
            "ラスサビ歌詞" => "Last Chorus Lyrics",
            "Bロール_夜景" => "B-roll Night",
            "エンドカード" => "End Card",
            "クレジット" => "Credits",
            other => other,
        }
        .to_owned()
    }

    fn timeline_model(&self) -> TimelineModel {
        let store = self.doc.view();
        let mut rows = timeline_pane::rows(&store, &self.session);
        // Timeline top means Stage front. Both are derived from the same
        // `LayerMeta.order`; no independent lane-order state exists here.
        rows.sort_by(|left, right| {
            let left_order = store
                .meta(left.id)
                .ok()
                .flatten()
                .map(|meta| meta.order)
                .unwrap_or(i16::MIN);
            let right_order = store
                .meta(right.id)
                .ok()
                .flatten()
                .map(|meta| meta.order)
                .unwrap_or(i16::MIN);
            right_order
                .cmp(&left_order)
                .then_with(|| right.id.cmp(&left.id))
        });
        drop(store);

        let lanes = rows
            .into_iter()
            .map(|row| TimelineLane {
                id: row.id.0,
                name: Self::display_name(&row.name),
                hidden: row.hidden,
                solo: row.solo,
                locked: row.locked,
                label_color: row
                    .label_color
                    .map(usize::from)
                    .unwrap_or_else(|| row.id.0.saturating_sub(1) as usize),
                start: row.start,
                duration: row.duration,
                selected: row.selected,
            })
            .collect();
        let property_lanes = timeline_pane::property_rows(
            &self.doc.view(),
            &self.session,
            self.doc.view().composition().ok().flatten().map(|c| c.fps),
        )
        .into_iter()
        .map(|row| TimelinePropertyLane {
            layer_id: row.layer.0,
            name: format!("> {}", row.property.name()),
            keys: row.keys.into_iter().map(|key| key.frame).collect(),
        })
        .collect();
        let composition = self.doc.view().composition().ok().flatten();
        let (duration_frames, fps_num, fps_den) = composition
            .map(|composition| {
                (
                    composition.duration_frames,
                    composition.fps.num(),
                    composition.fps.den(),
                )
            })
            .unwrap_or((1, 30, 1));

        TimelineModel {
            lanes,
            property_lanes,
            duration_frames,
            playhead: self.session.playhead,
            fps_num,
            fps_den,
        }
    }

    fn scrub_to(&mut self, frame: i64) {
        let started = Instant::now();
        self.session.playhead = frame.max(0);
        self.frame = None;
        log!(
            "PERF store_scrub frame={} elapsed_us={}",
            frame,
            started.elapsed().as_micros()
        );
    }

    fn restack_from_timeline(&mut self, layer_id: u64, target_from_front: usize) -> String {
        let store = self.doc.view();
        let Some(layer) = timeline_pane::rows(&store, &self.session)
            .into_iter()
            .find(|row| row.id.0 == layer_id)
            .map(|row| row.id)
        else {
            return format!("Timeline: layer {layer_id} no longer exists");
        };
        let layer_count = store.layers().len();
        drop(store);
        let target_from_front = target_from_front.min(layer_count.saturating_sub(1));
        let target_from_back = layer_count
            .saturating_sub(1)
            .saturating_sub(target_from_front);

        self.session.selection = Some(layer);
        self.session.selected_layers = vec![layer];

        let store = self.doc.view();
        let stack: Vec<(LayerId, i16)> = store
            .layers()
            .into_iter()
            .filter_map(|id| store.meta(id).ok().flatten().map(|meta| (id, meta.order)))
            .collect();
        drop(store);
        let changes = restacked(
            &stack,
            &[layer],
            StackDirection::ToIndexFromBack(target_from_back),
        );
        if !changes.is_empty() {
            let intents: Vec<Intent> = changes
                .into_iter()
                .map(|(layer, order)| Intent::SetOrder { layer, order })
                .collect();
            if let Err(error) = self.doc.apply_all(intents) {
                return format!("重なりを書けない: {error}");
            }
            self.frame = None;
        }
        self.status = Some(format!(
            "Timeline: layer {} moved to lane {} / Stage stack updated",
            layer_id,
            target_from_front + 1
        ));
        self.status.clone().expect("just set")
    }

    /// レールグリフ(M/S/L)のクリック。3属性とも `motolii_store::LayerAttrsPatch`
    /// (hidden/solo/locked)に本物の書き口 `Intent::SetAttrs` が既にあるので、
    /// M/S/L のどれも Document へ書く — `TimelineModel` だけをその場でいじる
    /// フェイクは要らない(発注の「本物の口が無ければ捏造しない」の逆側:
    /// 本物があるのに使わない方が不自然)。
    fn toggle_lane_flag_from_timeline(&mut self, layer_id: u64, flag: LaneFlag) -> String {
        let layer = LayerId(layer_id);
        let store = self.doc.view();
        let Some(attrs) = store.attrs(layer).ok().flatten() else {
            return format!("Timeline: layer {layer_id} no longer exists");
        };
        drop(store);

        let (label, next, mut patch) = match flag {
            LaneFlag::Hidden => ("HIDDEN", !attrs.hidden, LayerAttrsPatch::default()),
            LaneFlag::Solo => ("SOLO", !attrs.solo, LayerAttrsPatch::default()),
            LaneFlag::Locked => ("LOCKED", !attrs.locked, LayerAttrsPatch::default()),
        };
        match flag {
            LaneFlag::Hidden => patch.hidden = Some(next),
            LaneFlag::Solo => patch.solo = Some(next),
            LaneFlag::Locked => patch.locked = Some(next),
        }
        if let Err(error) = self.doc.apply_all([Intent::SetAttrs { layer, patch }]) {
            return format!("{label} を書けない: {error}");
        }
        self.frame = None;
        self.status = Some(format!(
            "Timeline: layer {layer_id} {label} {}",
            if next { "ON" } else { "OFF" }
        ));
        self.status.clone().expect("just set")
    }

    /// screenshot / export など明示 fallback 専用。通常の playhead / 再生からは呼ばない。
    fn frame_rgba(&mut self) -> Option<(u32, u32, &[u8])> {
        if self.frame.is_none() {
            let composition = self.doc.view().composition().ok().flatten()?;
            let t = RationalTime::try_from_frame(self.session.playhead, composition.fps).ok()?;
            let rgba = self.engine.render_frame(&self.doc.view(), t).ok()?;
            self.frame = Some((composition.width, composition.height, rgba));
        }
        self.frame
            .as_ref()
            .map(|(width, height, rgba)| (*width, *height, rgba.as_slice()))
    }

    fn toggle_playback(&mut self) -> bool {
        self.playing = !self.playing;
        self.playing
    }

    fn playback_tick(&mut self) -> bool {
        if !self.playing {
            return false;
        }
        let started = Instant::now();
        let duration = self
            .doc
            .view()
            .composition()
            .ok()
            .flatten()
            .map(|composition| composition.duration_frames)
            .unwrap_or(1)
            .max(1);
        self.session.playhead = (self.session.playhead + 1) % duration;
        self.frame = None;
        log!(
            "PERF store_playback frame={} elapsed_us={}",
            self.session.playhead,
            started.elapsed().as_micros()
        );
        true
    }

    fn apply_timeline_action(&mut self, action: &TimelineSurfaceAction) -> TimelineUpdate {
        match *action {
            TimelineSurfaceAction::None => TimelineUpdate::None,
            TimelineSurfaceAction::Scrub(frame) => {
                self.scrub_to(frame);
                TimelineUpdate::Stage(format!("SCRUB  ·  FRAME {frame}"))
            }
            TimelineSurfaceAction::Restack {
                layer_id,
                target_from_front,
            } => TimelineUpdate::ModelAndStage(
                self.restack_from_timeline(layer_id, target_from_front),
            ),
            TimelineSurfaceAction::ZoomChanged {
                start_frame,
                visible_frames,
            } => TimelineUpdate::Status(format!(
                "TIME ZOOM  ·  X ONLY  ·  START {start_frame}  ·  SPAN {visible_frames}F"
            )),
            TimelineSurfaceAction::ToggleLaneFlag { layer_id, flag } => TimelineUpdate::ModelAndStage(
                self.toggle_lane_flag_from_timeline(layer_id, flag),
            ),
        }
    }
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    playback_timer: Timer,
    /// Browser の選択。widget の animator ではなくここが正本。
    #[rust]
    browser_tab: usize,
    #[rust]
    browser_rail: usize,
    /// UI 全体の拡縮(%)。100 が等倍。
    #[rust(100)]
    ui_scale_percent: i32,
    /// いま明かしている浮くタブ行。開いた後の保持判定に使う。
    #[rust]
    revealed_bar: Option<LiveId>,
    /// Browser 面が畳まれているか。面の幅の正本は Dock の splitter align で、
    /// ここは「どちら向きに押すか」だけを持つ。
    #[rust]
    browser_collapsed: bool,
    /// 畳む直前の Browser 面の幅。戻す先が無いと畳みは片道になる。
    #[rust]
    browser_restore_width: f64,
    /// 状態行の文言。live edit は widget を宣言状態へ戻すので、ここが正本。
    #[rust]
    status_text: String,
    #[rust]
    stage_next_frame: NextFrame,
    /// 直前の室判定。変化したときだけ1行ログを出すため。
    #[rust]
    stage_verdict: Option<StageVerdict>,
    #[rust]
    stage_request: LatestFrameRequest,
    /// The existing product shell remains the sole Document/Engine owner. This
    /// probe only reads its compositor output for the Makepad Stage image.
    #[rust]
    backend: Option<BackendBridge>,
}

impl App {
    fn install_stage_frame(&mut self, cx: &mut Cx) {
        let verdict = self.try_present_shared(cx);
        self.set_stage_error(cx, &verdict.message());
        // 室が変わったときだけ1行。黒い Stage を見たらこの行だけ読めばよい。
        if self.stage_verdict != Some(verdict) {
            self.stage_verdict = Some(verdict);
            match verdict {
                StageVerdict::Shown => log!("STAGE room=- zero_copy=true shown"),
                StageVerdict::Stalled { room, reason } => {
                    log!(
                        "STAGE room={} owner={} reason={}",
                        room.tag(),
                        room.owner(),
                        reason
                    )
                }
            }
        }
    }

    fn try_present_shared(&mut self, cx: &mut Cx) -> StageVerdict {
        let Some(backend) = self.backend.as_mut() else {
            return StageVerdict::stalled(StageRoom::Seam, "backend is not up yet");
        };
        let Some(composition) = backend.doc.view().composition().ok().flatten() else {
            return StageVerdict::stalled(StageRoom::Host, "composition is unreadable");
        };
        let desc = SharedSurfaceDesc::from_comp(composition.width, composition.height);
        let recreate = backend.present.needs_recreate(desc) || backend.stage_gpu.is_none();
        if recreate {
            let (texture, handle) = cx.create_presentable_texture(
                desc.width,
                desc.height,
                SharedPresentablePixel::Rgba8Srgb,
            );
            let handle = match handle {
                makepad_widgets::SharedOsHandle::IoSurfaceId(id) => SharedOsHandle::IoSurfaceId(id),
                makepad_widgets::SharedOsHandle::DxgiSharedHandle(v) => {
                    SharedOsHandle::DxgiSharedHandle(v)
                }
                makepad_widgets::SharedOsHandle::DmaBufFd(fd) => SharedOsHandle::DmaBufFd(fd),
            };
            let Some(present) = StagePresent::shared(desc, handle) else {
                return StageVerdict::stalled(StageRoom::Leaf, "the shared surface handle is unusable");
            };
            let Some(gpu) =
                stage_import::import_presentable(backend.engine.gpu_device(), desc, handle)
            else {
                return StageVerdict::stalled(StageRoom::Seam, "cannot import the shared surface into wgpu");
            };
            backend.stage_texture = Some(texture);
            backend.stage_gpu = Some(gpu);
            backend.present = present;
        }
        let Some(gpu) = backend.stage_gpu.as_ref() else {
            return StageVerdict::stalled(StageRoom::Seam, "no shared surface is held");
        };
        let Ok(t) = RationalTime::try_from_frame(backend.session.playhead, composition.fps) else {
            return StageVerdict::stalled(StageRoom::Host, "playhead does not map to a time");
        };
        if backend
            .engine
            .render_frame_into(&backend.doc.view(), t, gpu)
            .is_err()
        {
            return StageVerdict::stalled(StageRoom::Host, "writing into the shared surface failed");
        }
        let present = backend.present;
        let Some(texture) = backend.stage_texture.clone() else {
            return StageVerdict::stalled(StageRoom::Seam, "no shared Texture is held");
        };
        let stage_image = self.stage_image(cx);
        if stage_image.is_empty() {
            return StageVerdict::stalled(StageRoom::Seam, "the Stage Image is not in the panel");
        }
        // 「表示側が答えた寸法」を持って帰る。書けたことは見えたことではない。
        let displayed = texture
            .get_format(cx)
            .vec_width_height()
            .map(|(width, height)| (width as u32, height as u32));
        stage_image.set_texture(cx, Some(texture));
        cx.redraw_all();
        // 「書けた」で終わらせない。出たかどうかは表示寸法が答える。
        stage_surface::check_shown(present, desc, displayed)
    }

    fn dock(&self, cx: &mut Cx) -> DockRef {
        self.ui.widget(cx, ids!(panel.dock)).as_dock()
    }

    fn stage_image(&self, cx: &mut Cx) -> ImageRef {
        self.dock(cx)
            .item(id!(stage))
            .child_by_path(ids!(stage_frame))
            .as_image()
    }

    fn set_stage_error(&self, cx: &mut Cx, text: &str) {
        self.dock(cx)
            .item(id!(stage))
            .child_by_path(ids!(stage_error))
            .as_label()
            .set_text(cx, text);
        cx.redraw_all();
    }

    fn request_stage_frame(&mut self, cx: &mut Cx) {
        if !self.stage_request.request() {
            self.stage_next_frame = cx.new_next_frame();
        }
    }

    fn install_timeline_model(&mut self, cx: &mut Cx) {
        let started = Instant::now();
        let Some(model) = self.backend.as_ref().map(BackendBridge::timeline_model) else {
            return;
        };
        let timeline = self.timeline_ref(cx);
        let timeline_found = !timeline.is_empty();
        if let Some(mut timeline) = timeline.borrow_mut::<TimelineSurface>() {
            timeline.set_model(cx, model);
        }
        log!(
            "PERF timeline_projection elapsed_us={} timeline_found={}",
            started.elapsed().as_micros(),
            timeline_found,
        );
    }

    fn browser(&self, cx: &mut Cx) -> WidgetRef {
        self.dock(cx).item(id!(browser))
    }

    /// Browser の「N のうち1つ」は makepad の radio group が持つ。排他と選択移動は
    /// `RadioButtonSet::selected` の担当で、色をこちらで塗り替えない
    /// (`active` は instance、`draw_bg.color*` は group 共有の uniform)。
    fn browser_radio_groups(&mut self, cx: &mut Cx, actions: &Actions) {
        let browser = self.browser(cx);
        browser
            .radio_button_set(
                cx,
                ids_array!(
                    browser_surface.tabs.media,
                    browser_surface.tabs.effects,
                    browser_surface.tabs.create,
                    browser_surface.tabs.panels
                ),
            )
            .selected(cx, actions)
            .map(|index| self.browser_tab = index);
        if let Some(index) = browser
            .radio_button_set(cx, browser_rail_ids!())
            .selected(cx, actions)
        {
            self.browser_rail = index;
        }
    }


    /// 選択は App が持つ。widget は投影であって正本ではない — `script_mod!` の
    /// 再実行(hot reload)は animator を宣言状態へ戻すので、そのたび投影し直す。
    fn apply_browser_selection(&self, cx: &mut Cx) {
        let browser = self.browser(cx);
        let tabs = browser.radio_button_set(
            cx,
            ids_array!(
                browser_surface.tabs.media,
                browser_surface.tabs.effects,
                browser_surface.tabs.create,
                browser_surface.tabs.panels
            ),
        );
        for (index, item) in tabs.iter().enumerate() {
            item.set_active(cx, index == self.browser_tab, Animate::No);
        }
        let rail = browser.radio_button_set(cx, browser_rail_ids!());
        for (index, item) in rail.iter().enumerate() {
            item.set_active(cx, index == self.browser_rail, Animate::No);
        }
    }

    /// UI 全体の拡縮。寸法トークンが1箇所に集まっているので、倍率もここ1つで済む。
    ///
    /// 窓の `dpi_override` でも同じ絵は作れるが、実行時に差し替えると `--remote` の
    /// grab が Metal のアサーションで落ちる(drawable と grab テクスチャの寸法不一致、
    /// 実測 2026-08-27)。検証手段を壊さない方を採る。
    /// 浮くタブ行を明かす判断。**機構は Dock、判断はここ**(fork 差分に製品の判断を
    /// 入れない — gesture fork と同じ切り方)。
    ///
    /// 開く引き金はセルの左上の隅だけにする。帯の全幅を引き金にすると、上端の操作へ
    /// 手を伸ばしただけで開いてしまう。開いた後は帯の全幅で保持する — でないと
    /// タブへ向かって右へ動いた瞬間に閉じる。
    fn reveal_tab_bars_under(&mut self, cx: &mut Cx, abs: Vec2d) {
        let dock = self.dock(cx);
        let bar = 25.0 * tokens::ui_scale();
        let corner = 140.0 * tokens::ui_scale();
        for (id, cell) in dock.floating_tab_bar_cells() {
            let open_zone = Rect {
                pos: cell.pos,
                size: dvec2(corner.min(cell.size.x), bar),
            };
            // 帯はセルの**上**に生えるので、保持ゾーンは境界をまたぐ
            let hold_zone = Rect {
                pos: dvec2(cell.pos.x, cell.pos.y - bar),
                size: dvec2(cell.size.x, bar * 2.0),
            };
            let shown = if self.revealed_bar == Some(id) {
                hold_zone.contains(abs)
            } else {
                open_zone.contains(abs)
            };
            dock.set_tab_bar_revealed(cx, id, shown);
            if shown {
                self.revealed_bar = Some(id);
            } else if self.revealed_bar == Some(id) {
                self.revealed_bar = None;
            }
        }
    }

    fn set_ui_scale(&mut self, cx: &mut Cx, percent: i32) {
        let percent = tokens::set_ui_scale_percent(percent);
        if percent == self.ui_scale_percent {
            return;
        }
        self.ui_scale_percent = percent;
        // トークンは `script_mod!` の式へ焼き込まれている。焼き直しは live edit の仕事
        // (makepad が iOS の safe-area inset に使っているのと同じ経路)。
        cx.request_live_edit();
        self.set_status(cx, &format!("UI SCALE  ·  {percent}%"));
    }

    fn set_status(&mut self, cx: &mut Cx, status: &str) {
        self.status_text = status.to_string();
        self.project_status(cx);
    }

    fn project_status(&self, cx: &mut Cx) {
        if self.status_text.is_empty() {
            return;
        }
        self.ui
            .widget(cx, ids!(panel.status))
            .as_label()
            .set_text(cx, &self.status_text);
    }

    fn timeline_ref(&self, cx: &mut Cx) -> WidgetRef {
        self.dock(cx)
            .item(id!(timeline))
            .child_by_path(ids!(timeline_surface))
    }

    fn timeline_uid(&self, cx: &mut Cx) -> Option<WidgetUid> {
        let timeline = self.timeline_ref(cx);
        (!timeline.is_empty()).then(|| timeline.widget_uid())
    }

    fn play_uid(&self, cx: &mut Cx) -> Option<WidgetUid> {
        let play = self
            .dock(cx)
            .item(id!(timeline))
            .child_by_path(ids!(play_toggle));
        (!play.is_empty()).then(|| play.widget_uid())
    }

    /// パネル切替(panels.svg)。Browser 面を畳む/戻す。
    ///
    /// 面の幅の正本は Dock の splitter align 1つで、こちらは畳む前の幅だけ覚える。
    /// `FromA(0.0)` は掴み棒だけを残して A 側を潰す — 帯ごと消さないので、
    /// 畳んだ後もドラッグで戻せる(押した物が消えて戻せない、を作らない)。
    fn toggle_browser_panel(&mut self, cx: &mut Cx) {
        let dock = self.dock(cx);
        if self.browser_collapsed {
            // 畳む前が既に 0 幅だったなら宣言の既定へ戻す(そうしないと戻らない)
            let width = if self.browser_restore_width > 1.0 {
                self.browser_restore_width
            } else {
                300.0 * tokens::ui_scale()
            };
            dock.set_splitter_align(cx, id!(top_split), SplitterAlign::FromA(width), true);
            self.browser_collapsed = false;
            self.set_status(cx, "BROWSER  ·  SHOWN");
        } else {
            self.browser_restore_width = dock.splitter_position(id!(top_split)).unwrap_or(0.0);
            dock.set_splitter_align(cx, id!(top_split), SplitterAlign::FromA(0.0), true);
            self.browser_collapsed = true;
            self.set_status(cx, "BROWSER  ·  HIDDEN");
        }
    }

    /// 設定(filter.svg)。`SettingsPane` は inspector 側のタブとして既に居るので、
    /// 開くとは「そのタブを選ぶ」こと。新しい面を作らない。
    fn open_settings(&mut self, cx: &mut Cx) {
        self.dock(cx).select_tab(cx, id!(settings));
        self.set_status(cx, "SETTINGS");
    }

    /// いま文字を打っている最中か。
    ///
    /// 欄を名指ししない — 面は他レーンが増やし続けるので、木を辿って
    /// 「キーフォーカスを持つ `TextInput` が居るか」だけを聞く。名指しにすると
    /// 欄が増えるたびにここが古くなり、Space が再生へ抜ける穴が戻る。
    fn text_entry_has_focus(&self, cx: &Cx) -> bool {
        fn focused_text_input(cx: &Cx, node: &WidgetRef) -> bool {
            if node.borrow::<TextInput>().is_some() && node.key_focus(cx) {
                return true;
            }
            let mut found = false;
            node.children(&mut |_id, child| {
                if !found {
                    found = focused_text_input(cx, &child);
                }
            });
            found
        }
        focused_text_input(cx, &self.ui)
    }

    fn toggle_playback(&mut self, cx: &mut Cx) {
        let playing = self
            .backend
            .as_mut()
            .map(BackendBridge::toggle_playback)
            .unwrap_or(false);
        self.set_status(
            cx,
            if playing {
                "PLAYING  ·  SPACE TO PAUSE"
            } else {
                "PAUSED"
            },
        );
        self.install_timeline_model(cx);
        self.request_stage_frame(cx);
    }

}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        self.backend = Some(BackendBridge::new_fixture());
        self.playback_timer = cx.start_interval(1.0 / 60.0);
        self.install_timeline_model(cx);
        self.request_stage_frame(cx);
        self.browser_rail = RAIL_ALL_MEDIA;
        self.apply_browser_selection(cx);
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        self.browser_radio_groups(cx, actions);
        if self
            .ui
            .widget(cx, ids!(panel.chrome.browser_toggle))
            .as_button()
            .clicked(actions)
        {
            self.toggle_browser_panel(cx);
        }
        if self
            .ui
            .widget(cx, ids!(panel.chrome.settings))
            .as_button()
            .clicked(actions)
        {
            self.open_settings(cx);
        }
        if let Some(uid) = self.play_uid(cx) {
            if actions
                .filter_widget_actions(uid)
                .any(|action| matches!(action.cast(), ButtonAction::Clicked { .. }))
            {
                self.toggle_playback(cx);
            }
        }
        let Some(uid) = self.timeline_uid(cx) else {
            return;
        };
        let timeline_actions: Vec<TimelineSurfaceAction> =
            actions.filter_widget_actions_cast(uid).collect();
        for action in timeline_actions {
            let update = self
                .backend
                .as_mut()
                .map(|backend| backend.apply_timeline_action(&action))
                .unwrap_or(TimelineUpdate::None);
            match update {
                TimelineUpdate::None => {}
                TimelineUpdate::Stage(status) => {
                    self.request_stage_frame(cx);
                    self.set_status(cx, &status);
                }
                TimelineUpdate::ModelAndStage(status) => {
                    self.install_timeline_model(cx);
                    self.request_stage_frame(cx);
                    self.set_status(cx, &status);
                }
                TimelineUpdate::Status(status) => {
                    self.set_status(cx, &status);
                }
            }
        }
    }

    fn handle_timer(&mut self, cx: &mut Cx, event: &TimerEvent) {
        if self.playback_timer.is_timer(event).is_some()
            && self
                .backend
                .as_mut()
                .map(BackendBridge::playback_tick)
                .unwrap_or(false)
        {
            self.install_timeline_model(cx);
            self.request_stage_frame(cx);
        }
    }

    fn handle_key_down(&mut self, cx: &mut Cx, event: &KeyEvent) {
        // `MatchEvent` は `ui.handle_event` より先に来る。フォーカスのある欄より先に
        // ここが Space を食べると、名前を打っている最中に再生が始まる。
        // 文字を打っている間は窓のショートカットを名乗らない — 手前の欄が持ち主。
        if self.text_entry_has_focus(cx) {
            return;
        }
        if event.modifiers.logo || event.modifiers.control {
            let step = if event.modifiers.shift { 10 } else { 1 };
            match event.key_code {
                KeyCode::Equals => {
                    self.set_ui_scale(cx, self.ui_scale_percent + step);
                    return;
                }
                KeyCode::Minus => {
                    self.set_ui_scale(cx, self.ui_scale_percent - step);
                    return;
                }
                KeyCode::Key0 => {
                    self.set_ui_scale(cx, 100);
                    return;
                }
                _ => {}
            }
        }
        if event.key_code == KeyCode::Space && !event.is_repeat {
            self.toggle_playback(cx);
        }
    }

    fn handle_next_frame(&mut self, cx: &mut Cx, event: &NextFrameEvent) {
        if event.set.contains(&self.stage_next_frame) && self.stage_request.take() {
            self.install_stage_frame(cx);
        }
    }
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        // Ableton の identity は palette ではなく形の文法にある(18テーマ同梱 =
        // 色を全部差し替えても Ableton に見える、が証拠)。テーマ横断で不変なのは:
        // 矩形のみ・角丸ゼロ・ベベルゼロ・影ゼロ、分離は 1px の暗線と明度段。
        // makepad の既定(corner_radius 2.5 / beveling 0.75)はその全部に反するので、
        // widget が theme.* を読む**前**に根を書き換える。現場の数百箇所を触らない。
        crate::makepad_widgets::theme_mod(vm);
        crate::theme_flat::script_mod(vm);
        crate::makepad_widgets::widgets_mod(vm);
        // 目盛りは誰よりも先。surface はこれを引く。
        crate::tokens::script_mod(vm);
        // Widget modules register before the UI modules that import them (DSL 正史)。
        // chrome (parts / gallery 含む) が先、surface 群が後。
        crate::chrome::script_mod(vm);
        crate::browser_surface::script_mod(vm);
        crate::stage_chrome::script_mod(vm);
        crate::inspector_surface::script_mod(vm);
        crate::export_surface::script_mod(vm);
        crate::settings_surface::script_mod(vm);
        crate::timeline_surface::script_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        // hot reload は `script_mod!` を再実行して widget を宣言状態へ戻す。
        // 選択は App が持っているので、投影し直すのはこちらの責任。
        if let Event::MouseMove(move_event) = event {
            let abs = move_event.abs;
            self.reveal_tab_bars_under(cx, abs);
        }
        // 窓の外へ出たら浮きタブを畳む。MouseMove は窓内でしか来ないので、
        // 出しっぱなしで固まるのはここを聞いていない時だけ
        if matches!(event, Event::MouseLeave(_)) {
            self.reveal_tab_bars_under(cx, dvec2(-1.0e6, -1.0e6));
        }
        if matches!(event, Event::LiveEdit) {
            self.apply_browser_selection(cx);
            self.project_status(cx);
        }
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

#[cfg(test)]
mod backend_tests {
    use super::*;

    #[test]
    fn stage_present_starts_as_named_cpu_fallback() {
        let backend = BackendBridge::new_fixture();
        assert_eq!(backend.present, StagePresent::FallbackCpu);
        assert!(!backend.present.is_zero_copy());
    }

    #[test]
    fn scrub_action_reaches_session() {
        let mut backend = BackendBridge::new_fixture();
        let update = backend.apply_timeline_action(&TimelineSurfaceAction::Scrub(600));

        assert!(matches!(update, TimelineUpdate::Stage(_)));
        assert_eq!(backend.session.playhead, 600);
    }

    #[test]
    fn lane_restack_action_changes_document_derived_stage_order() {
        let mut backend = BackendBridge::new_fixture();
        let layer_id = backend
            .timeline_model()
            .lanes
            .last()
            .expect("fixture lane")
            .id;
        let update = backend.apply_timeline_action(&TimelineSurfaceAction::Restack {
            layer_id,
            target_from_front: 0,
        });

        assert!(matches!(update, TimelineUpdate::ModelAndStage(_)));
        assert_eq!(backend.timeline_model().lanes[0].id, layer_id);
    }

    #[test]
    fn stage_requests_coalesce_to_one_latest_delivery() {
        let mut requests = LatestFrameRequest::default();
        assert!(!requests.request(), "first request schedules a consumer");
        assert!(
            requests.request(),
            "later requests reuse the pending consumer"
        );
        assert!(requests.take());
        assert!(
            !requests.take(),
            "one delivery consumes every coalesced request"
        );
    }
}
