pub use makepad_widgets;

use makepad_widgets::*;
use motolii_engine::Engine;
use motolii_shell_state::Session;
use motolii_store::{Document, Intent, LayerId, RationalTime};
use motolii_timeline_pane::{self as timeline_pane, stacking::restacked, StackDirection};
use std::time::Instant;

mod browser_surface;
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
    TimelineLane, TimelineModel, TimelinePropertyLane, TimelineSurface, TimelineSurfaceAction,
};

app_main!(App);

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // Source: Motolii next/reference/mocks/*.html (visual/semantic contract only).
    // This is a Makepad proof surface, not a second product state owner.

    let IconButton = ButtonFlatterIcon{
        width: 24
        height: 22
        icon_walk: Walk{width: 13 height: 13}
        padding: Inset{left: 0 right: 0}
        draw_icon +: {color: #xb7b7b7}
    }

    let IconFlatButton = ButtonFlatIcon{
        width: 24
        height: 22
        icon_walk: Walk{width: 13 height: 13}
        padding: Inset{left: 0 right: 0}
        draw_icon +: {color: #xcfcfcf}
    }

    let RailToggle = ButtonFlat{
        width: 12
        height: 12
        margin: 0
        padding: 0
        spacing: 0
        align: Center
        label_walk: Walk{width: Fill height: Fill}
        draw_bg.color: #x4a4a4a
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 2.0
        draw_text.color: #x757575
        draw_text.text_style: theme.font_regular{font_size: 6 line_spacing: 1.0 top_drop: 0.0}
    }

    let TimelineLabel = Label{
        width: Fit
        height: Fit
        padding: 0
        draw_text.color: #xcfcfcf
        draw_text.text_style: theme.font_regular{font_size: 8.25 line_spacing: 1.0 top_drop: 0.0}
    }

    let TimelineKeyLabel = Label{
        width: Fill
        height: Fit
        padding: 0
        draw_text.color: #x757575
        draw_text.text_style: theme.font_regular{font_size: 7.5 line_spacing: 1.0 top_drop: 0.0}
    }

    // A line is reserved for a change in interaction owner or coordinate system.
    // Related controls use spacing and fill state instead of decorative outlines.
    let PaneDivider = SolidView{
        width: 1
        height: Fill
        show_bg: true
        new_batch: true
        draw_bg.color: #x1d1d1d
    }

    let SurfaceDivider = SolidView{
        width: Fill
        height: 1
        show_bg: true
        new_batch: true
        draw_bg.color: #x1d1d1d
    }

    let TimeField = SolidView{
        width: 810
        height: Fill
        flow: Overlay
        align: Align{y: 0.5}
        show_bg: true
        draw_bg.color: #x363636
        band_1 := SolidView{width: 67.5 height: Fill margin: Inset{left: 67.5} draw_bg.color: #xFFFFFF09}
        band_2 := SolidView{width: 67.5 height: Fill margin: Inset{left: 202.5} draw_bg.color: #xFFFFFF09}
        band_3 := SolidView{width: 67.5 height: Fill margin: Inset{left: 337.5} draw_bg.color: #xFFFFFF09}
        band_4 := SolidView{width: 67.5 height: Fill margin: Inset{left: 472.5} draw_bg.color: #xFFFFFF09}
        band_5 := SolidView{width: 67.5 height: Fill margin: Inset{left: 607.5} draw_bg.color: #xFFFFFF09}
        band_6 := SolidView{width: 67.5 height: Fill margin: Inset{left: 742.5} draw_bg.color: #xFFFFFF09}
        grid_01 := SolidView{width: 1 height: Fill margin: Inset{left: 13.5} draw_bg.color: #x0000002e}
        grid_02 := SolidView{width: 1 height: Fill margin: Inset{left: 27} draw_bg.color: #x0000002e}
        grid_03 := SolidView{width: 1 height: Fill margin: Inset{left: 40.5} draw_bg.color: #x0000002e}
        grid_04 := SolidView{width: 1 height: Fill margin: Inset{left: 54} draw_bg.color: #x0000002e}
        grid_05 := SolidView{width: 1 height: Fill margin: Inset{left: 67.5} draw_bg.color: #x0000004d}
        grid_06 := SolidView{width: 1 height: Fill margin: Inset{left: 81} draw_bg.color: #x0000002e}
        grid_07 := SolidView{width: 1 height: Fill margin: Inset{left: 94.5} draw_bg.color: #x0000002e}
        grid_08 := SolidView{width: 1 height: Fill margin: Inset{left: 108} draw_bg.color: #x0000002e}
        grid_09 := SolidView{width: 1 height: Fill margin: Inset{left: 121.5} draw_bg.color: #x0000002e}
        grid_10 := SolidView{width: 1 height: Fill margin: Inset{left: 135} draw_bg.color: #x0000004d}
        grid_11 := SolidView{width: 1 height: Fill margin: Inset{left: 148.5} draw_bg.color: #x0000002e}
        grid_12 := SolidView{width: 1 height: Fill margin: Inset{left: 162} draw_bg.color: #x0000002e}
        grid_13 := SolidView{width: 1 height: Fill margin: Inset{left: 175.5} draw_bg.color: #x0000002e}
        grid_14 := SolidView{width: 1 height: Fill margin: Inset{left: 189} draw_bg.color: #x0000002e}
        grid_15 := SolidView{width: 1 height: Fill margin: Inset{left: 202.5} draw_bg.color: #x0000004d}
        grid_16 := SolidView{width: 1 height: Fill margin: Inset{left: 216} draw_bg.color: #x0000002e}
        grid_17 := SolidView{width: 1 height: Fill margin: Inset{left: 229.5} draw_bg.color: #x0000002e}
        grid_18 := SolidView{width: 1 height: Fill margin: Inset{left: 243} draw_bg.color: #x0000002e}
        grid_19 := SolidView{width: 1 height: Fill margin: Inset{left: 256.5} draw_bg.color: #x0000002e}
        grid_20 := SolidView{width: 1 height: Fill margin: Inset{left: 270} draw_bg.color: #x0000004d}
        grid_21 := SolidView{width: 1 height: Fill margin: Inset{left: 283.5} draw_bg.color: #x0000002e}
        grid_22 := SolidView{width: 1 height: Fill margin: Inset{left: 297} draw_bg.color: #x0000002e}
        grid_23 := SolidView{width: 1 height: Fill margin: Inset{left: 310.5} draw_bg.color: #x0000002e}
        grid_24 := SolidView{width: 1 height: Fill margin: Inset{left: 324} draw_bg.color: #x0000002e}
        grid_25 := SolidView{width: 1 height: Fill margin: Inset{left: 337.5} draw_bg.color: #x0000004d}
        grid_26 := SolidView{width: 1 height: Fill margin: Inset{left: 351} draw_bg.color: #x0000002e}
        grid_27 := SolidView{width: 1 height: Fill margin: Inset{left: 364.5} draw_bg.color: #x0000002e}
        grid_28 := SolidView{width: 1 height: Fill margin: Inset{left: 378} draw_bg.color: #x0000002e}
        grid_29 := SolidView{width: 1 height: Fill margin: Inset{left: 391.5} draw_bg.color: #x0000002e}
        grid_30 := SolidView{width: 1 height: Fill margin: Inset{left: 405} draw_bg.color: #x0000004d}
        grid_31 := SolidView{width: 1 height: Fill margin: Inset{left: 418.5} draw_bg.color: #x0000002e}
        grid_32 := SolidView{width: 1 height: Fill margin: Inset{left: 432} draw_bg.color: #x0000002e}
        grid_33 := SolidView{width: 1 height: Fill margin: Inset{left: 445.5} draw_bg.color: #x0000002e}
        grid_34 := SolidView{width: 1 height: Fill margin: Inset{left: 459} draw_bg.color: #x0000002e}
        grid_35 := SolidView{width: 1 height: Fill margin: Inset{left: 472.5} draw_bg.color: #x0000004d}
        grid_36 := SolidView{width: 1 height: Fill margin: Inset{left: 486} draw_bg.color: #x0000002e}
        grid_37 := SolidView{width: 1 height: Fill margin: Inset{left: 499.5} draw_bg.color: #x0000002e}
        grid_38 := SolidView{width: 1 height: Fill margin: Inset{left: 513} draw_bg.color: #x0000002e}
        grid_39 := SolidView{width: 1 height: Fill margin: Inset{left: 526.5} draw_bg.color: #x0000002e}
        grid_40 := SolidView{width: 1 height: Fill margin: Inset{left: 540} draw_bg.color: #x0000004d}
        grid_41 := SolidView{width: 1 height: Fill margin: Inset{left: 553.5} draw_bg.color: #x0000002e}
        grid_42 := SolidView{width: 1 height: Fill margin: Inset{left: 567} draw_bg.color: #x0000002e}
        grid_43 := SolidView{width: 1 height: Fill margin: Inset{left: 580.5} draw_bg.color: #x0000002e}
        grid_44 := SolidView{width: 1 height: Fill margin: Inset{left: 594} draw_bg.color: #x0000002e}
        grid_45 := SolidView{width: 1 height: Fill margin: Inset{left: 607.5} draw_bg.color: #x0000004d}
        grid_46 := SolidView{width: 1 height: Fill margin: Inset{left: 621} draw_bg.color: #x0000002e}
        grid_47 := SolidView{width: 1 height: Fill margin: Inset{left: 634.5} draw_bg.color: #x0000002e}
        grid_48 := SolidView{width: 1 height: Fill margin: Inset{left: 648} draw_bg.color: #x0000002e}
        grid_49 := SolidView{width: 1 height: Fill margin: Inset{left: 661.5} draw_bg.color: #x0000002e}
        grid_50 := SolidView{width: 1 height: Fill margin: Inset{left: 675} draw_bg.color: #x0000004d}
        grid_51 := SolidView{width: 1 height: Fill margin: Inset{left: 688.5} draw_bg.color: #x0000002e}
        grid_52 := SolidView{width: 1 height: Fill margin: Inset{left: 702} draw_bg.color: #x0000002e}
        grid_53 := SolidView{width: 1 height: Fill margin: Inset{left: 715.5} draw_bg.color: #x0000002e}
        grid_54 := SolidView{width: 1 height: Fill margin: Inset{left: 729} draw_bg.color: #x0000002e}
        grid_55 := SolidView{width: 1 height: Fill margin: Inset{left: 742.5} draw_bg.color: #x0000004d}
        grid_56 := SolidView{width: 1 height: Fill margin: Inset{left: 756} draw_bg.color: #x0000002e}
        grid_57 := SolidView{width: 1 height: Fill margin: Inset{left: 769.5} draw_bg.color: #x0000002e}
        grid_58 := SolidView{width: 1 height: Fill margin: Inset{left: 783} draw_bg.color: #x0000002e}
        grid_59 := SolidView{width: 1 height: Fill margin: Inset{left: 796.5} draw_bg.color: #x0000002e}
    }

    let TimelineRow = SolidView{
        width: Fill
        height: 26
        flow: Overlay
        show_bg: true
        new_batch: true
        draw_bg.color: #x363636
        content := View{width: Fill height: Fill flow: Right}
        rail_divider := PaneDivider{margin: Inset{left: 149}}
        separator := SolidView{width: Fill height: 1 margin: Inset{top: 25} draw_bg.color: #x00000038}
    }

    let TimelineKeyRow = SolidView{
        width: Fill
        height: 18
        flow: Overlay
        show_bg: true
        new_batch: true
        draw_bg.color: #x363636
        content := View{width: Fill height: Fill flow: Right}
        rail_divider := PaneDivider{margin: Inset{left: 149}}
        separator := SolidView{width: Fill height: 1 margin: Inset{top: 17} draw_bg.color: #x00000038}
    }

    let ZebraTimeField = TimeField{
        draw_bg.color: #xFFFFFF0d
    }

    let SelectedTimeField = TimeField{
        draw_bg.color: #x565048
    }

    let KeyTimeField = TimeField{
        draw_bg.color: #x303030
    }

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
            height: 24
            flow: Right
            align: Align{x: 0.5 y: 0.5}
            show_bg: true
            new_batch: true
            draw_bg.color: #x3d3d3d
            play_toggle := ButtonFlatIcon{
                width: 26
                height: 20
                icon_walk: Walk{width: 12 height: 12}
                padding: Inset{left: 0 right: 0}
                draw_bg.color: #4f4f4f
                draw_bg.border_size: 0.0
                draw_icon +: {svg: crate_resource("self://resources/icons/play.svg") color: #xd8b574}
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
        draw_bg.color: #x282828

            chrome := SolidView{
                width: Fill
                height: 32
                flow: Right
                align: Align{y: 0.5}
                padding: Inset{left: 10 right: 10}
                show_bg: true
                new_batch: true
                draw_bg +: { color: #x242424 }

                brand := SolidView{
                    width: 74
                    height: 24
                    flow: Right
                    align: Align{y: 0.5}
                    spacing: 5
                    mark := Icon{width: 17 height: 17 icon_walk: Walk{width: 17 height: 17} draw_icon +: {svg: crate_resource("self://resources/icons/motolii.svg") color: #xd8d8d8}}
                    name := Label{text: "MOTOLII" width: Fill draw_text.color: #xd8d8d8 draw_text.text_style: theme.font_bold{font_size: 10}}
                }
                file := ButtonFlatter{text: "File" width: 42 height: 24 draw_text.color: #xb7b7b7 draw_text.text_style: theme.font_regular{font_size: 9}}
                edit := ButtonFlatter{text: "Edit" width: 42 height: 24 draw_text.color: #xb7b7b7 draw_text.text_style: theme.font_regular{font_size: 9}}
                layer := ButtonFlatter{text: "Layer" width: 50 height: 24 draw_text.color: #xb7b7b7 draw_text.text_style: theme.font_regular{font_size: 9}}
                view := ButtonFlatter{text: "View" width: 42 height: 24 draw_text.color: #xb7b7b7 draw_text.text_style: theme.font_regular{font_size: 9}}
                spacer := SolidView{width: Fill height: 1}
                project := Label{
                    text: "Untitled / Motion Study"
                    width: Fit
                    draw_text.color: #xa0a0a0
                    draw_text.text_style: theme.font_code{font_size: 9}
                }
                browser_toggle := IconButton{width: 26 draw_icon +: {svg: crate_resource("self://resources/icons/panels.svg")} on_click: || { ui.status.set_text("Browser panel") }}
                settings := IconButton{width: 26 draw_icon +: {svg: crate_resource("self://resources/icons/filter.svg")} on_click: || { ui.status.set_text("Settings") }}
            }

            chrome_surface_divider := SurfaceDivider{}

            dock := DockFlat{
                width: Fill
                height: Fill

                root := DockSplitter{
                    axis: SplitterAxis.Vertical
                    align: SplitterAlign.FromB(300.0)
                    a: @top_split
                    b: @timeline_tabs
                }

                top_split := DockSplitter{
                    axis: SplitterAxis.Horizontal
                    align: SplitterAlign.FromA(300.0)
                    a: @browser_tabs
                    b: @center_split
                }

                center_split := DockSplitter{
                    axis: SplitterAxis.Horizontal
                    align: SplitterAlign.FromB(300.0)
                    a: @stage_tabs
                    b: @inspector_tabs
                }

                browser_tabs := DockTabs{
                    tabs: [@browser]
                    selected: 0
                    closable: false
                }

                stage_tabs := DockTabs{
                    tabs: [@stage]
                    selected: 0
                    closable: false
                }

                inspector_tabs := DockTabs{
                    tabs: [@inspector @export @settings @chrome_tab]
                    selected: 3
                    closable: false
                }

                timeline_tabs := DockTabs{
                    tabs: [@timeline]
                    selected: 0
                    closable: false
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

            status := Label{
                text: "READY  ·  RERUN STAGE  ·  FRAME 900 / 1800"
                width: Fill
                height: 20
                padding: Inset{left: 10}
                draw_text.color: #x747879
                draw_text.text_style: theme.font_code{font_size: 8}
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
        }
    }
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    playback_timer: Timer,
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

    fn set_status(&self, cx: &mut Cx, status: &str) {
        self.ui
            .widget(cx, ids!(panel.status))
            .as_label()
            .set_text(cx, status);
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
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
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
        crate::makepad_widgets::script_mod(vm);
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
