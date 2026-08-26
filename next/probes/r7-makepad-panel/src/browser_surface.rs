//! Browser パネル枠。素材一覧の意味書きは波1。iced は引かない。
//! 出典: 利用者添付の Live 12 Dark 実画面
//!   `assets/image-cf39df4e-cc7d-4299-9900-56934306be7e.png`（1024×554 から画素サンプル）
//! 色（画像実測）: パネル面 #4f4f4f / 上バー #3d3d3d / 検索窪み #282828 /
//!   リスト見出し帯 #606060 / 行明字 #d8d8d8〜#e4e4e4 / 節見出し・案内字 #9d9d9d /
//!   選択 = 行全体ベタ #6b8d96 + 濃字 #133342（`chrome/parts/nav.rs` と同一サンプル）/
//!   rail・リスト境の縦線 #343434 / 継ぎ目 1px #2d2d2d / Favorites 赤 #f20813
//! 形（Live の言語）: 行は低く詰める（16）。選択は行全体のベタ塗り+濃字。
//!   区切りは 1px 暗線か面の明度差だけ（枠線で囲まない）。角丸ゼロ・影ゼロ。
//!   検索欄は窪み矩形（暗面に薄字プレースホルダ）。
//! hover/down は静止画に写らないため nav.rs と同じ実測面 #5c5c5c / #2d2d2d を使う。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // 素通しグリフ — 面を持たない。角丸・枠線なし
    let IconButton = ButtonFlatterIcon{
        width: 24
        height: 16
        icon_walk: Walk{width: 11 height: 11}
        padding: Inset{left: 0 right: 0}
        draw_icon +: {color: #xa0a0a0}
    }

    // tab — ベタ面のみで状態を語る。角丸・枠線なし
    let TabIcon = ButtonFlatIcon{
        width: Fill
        height: 22
        icon_walk: Walk{width: 12 height: 12}
        padding: Inset{left: 0 right: 0}
        draw_bg.color: #x3d3d3d
        draw_bg.color_hover: #x4f4f4f
        draw_bg.color_down: #x2d2d2d
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {color: #xa0a0a0}
    }

    // rail 行 — 低く詰める（16）。ベタ、hover は明度差だけ
    let RailRow = ButtonFlat{
        width: Fill
        height: 16
        icon_walk: Walk{width: 11 height: 11}
        align: Align{x: 0.0 y: 0.5}
        padding: Inset{left: 8 right: 8}
        draw_bg.color: #x4f4f4f
        draw_bg.color_hover: #x5c5c5c
        draw_bg.color_down: #x2d2d2d
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {color: #xa0a0a0}
        draw_text.color: #xe4e4e4
        draw_text.text_style: theme.font_regular{font_size: 8 line_spacing: 1.0 top_drop: 0.0}
    }

    // 選択済み rail 行 — ベタ塗り+濃字。`ButtonFlat.draw_bg.color` は uniform で
    // draw call 共有のため兄弟ごとに変えられない(実測 2026-08-27)。面は instance 色を
    // 持つ SolidView が塗り、当たりだけ透明 Button が受ける
    let RailRowOn = SolidView{
        width: Fill
        height: 16
        show_bg: true
        new_batch: true
        draw_bg.color: #x6b8d96
        label := ButtonFlatter{
            width: Fill
            height: Fill
            icon_walk: Walk{width: 11 height: 11}
            align: Align{x: 0.0 y: 0.5}
            padding: Inset{left: 8 right: 8}
            draw_icon +: {color: #x133342}
            draw_text.color: #x133342
            draw_text.text_style: theme.font_regular{font_size: 8 line_spacing: 1.0 top_drop: 0.0}
        }
    }

    // 選択済み tab — 同じ理由で同じ形
    let TabIconOn = SolidView{
        width: Fill
        height: 22
        show_bg: true
        new_batch: true
        draw_bg.color: #x6b8d96
        label := ButtonFlatterIcon{
            width: Fill
            height: Fill
            icon_walk: Walk{width: 12 height: 12}
            padding: Inset{left: 0 right: 0}
            draw_icon +: {color: #x133342}
        }
    }

    // 節見出し — Collections / Library / Places。薄字、上に群間の余白
    let RailCap = Label{
        width: Fill
        height: 18
        padding: Inset{left: 8 top: 7}
        draw_text.color: #x9d9d9d
        draw_text.text_style: theme.font_regular{font_size: 8 line_spacing: 1.0 top_drop: 0.0}
    }

    // 有効フィルタ chip — 選択の言語（ベタ #6b8d96 + 濃字 #133342、角丸なし）。
    // `chrome/parts/nav.rs` の ChromeChipOn と同一サンプル値。本 mod は chrome より先に
    // eval されるため参照でなく値で置く（未登録名の参照は葉が落ちる）
    let FilterChip = SolidView{
        width: Fit
        height: 16
        align: Align{y: 0.5}
        show_bg: true
        new_batch: true
        draw_bg.color: #x6b8d96
        label := ButtonFlatter{
            width: Fit
            height: Fit
            padding: Inset{left: 4 right: 4}
            draw_text.color: #x133342
            draw_text.text_style: theme.font_regular{font_size: 8 line_spacing: 1.0 top_drop: 0.0}
        }
    }

    // ファイル行 — Live 右リストの1行。低く詰める、ベタ、角丸なし
    let FileRow = ButtonFlat{
        width: Fill
        height: 16
        icon_walk: Walk{width: 11 height: 11}
        align: Align{x: 0.0 y: 0.5}
        padding: Inset{left: 6 right: 8}
        draw_bg.color: #x4f4f4f
        draw_bg.color_hover: #x5c5c5c
        draw_bg.color_down: #x2d2d2d
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {color: #xa0a0a0}
        draw_text.color: #xd8d8d8
        draw_text.text_style: theme.font_regular{font_size: 8 line_spacing: 1.0 top_drop: 0.0}
    }

    // rail とリストの境 — 画像実測 #343434 の縦 1px
    let PaneDivider = SolidView{
        width: 1
        height: Fill
        show_bg: true
        new_batch: true
        draw_bg.color: #x343434
    }

    // 継ぎ目 — 横 1px 暗線（枠線で囲まない）
    let SeamRule = SolidView{
        width: Fill
        height: 1
        show_bg: true
        new_batch: true
        draw_bg.color: #x2d2d2d
    }

    fn select_asset(name, file, kind){
        ui.browser_body.catalog.selection.selection_name.set_text(file)
        ui.browser_body.catalog.selection.selection_type.set_text(kind)
        name
    }

    mod.widgets.BrowserSurfaceBase = #(BrowserSurface::register_widget(vm))
    mod.widgets.BrowserSurface = set_type_default() do mod.widgets.BrowserSurfaceBase{
        width: Fill
        height: Fill
        flow: Down
        show_bg: true
        new_batch: true
        draw_bg.color: #x4f4f4f

        browser_head := SolidView{width: Fill height: 22 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: #x3d3d3d
            title := Label{text: "Browser" width: Fill draw_text.color: #xd0d0d0 draw_text.text_style: theme.font_regular{font_size: 10 line_spacing: 1.0 top_drop: 0.0}}
            local := Label{text: "LOCAL" width: Fit draw_text.color: #x757575 draw_text.text_style: theme.font_regular{font_size: 7 line_spacing: 1.0 top_drop: 0.0}}
        }
        head_rule := SeamRule{}
        browser_toolbar := SolidView{width: Fill height: 25 flow: Right spacing: 4 align: Align{y: 0.5} padding: Inset{left: 4 right: 4} show_bg: true new_batch: true draw_bg.color: #x4f4f4f
            back := IconButton{width: 18 draw_icon +: {svg: crate_resource("self://resources/icons/back.svg")}}
            forward := IconButton{width: 18 draw_icon +: {svg: crate_resource("self://resources/icons/forward.svg")}}
            search := SolidView{width: Fill height: 17 flow: Right spacing: 2 align: Align{y: 0.5} padding: Inset{left: 4 right: 4} show_bg: true new_batch: true draw_bg.color: #x282828
                search_glyph := IconButton{width: 15 draw_icon +: {svg: crate_resource("self://resources/icons/search.svg") color: #x909090}}
                search_hint := Label{text: "Search (Cmd + F)" width: Fill draw_text.color: #x909090 draw_text.text_style: theme.font_regular{font_size: 8 line_spacing: 1.0 top_drop: 0.0}}
            }
            filters := IconButton{width: 22 draw_icon +: {svg: crate_resource("self://resources/icons/filter.svg")}}
            tags := IconButton{width: 22 draw_icon +: {svg: crate_resource("self://resources/icons/tag.svg")}}
        }
        tabs := SolidView{width: Fill height: 22 flow: Right show_bg: true new_batch: true draw_bg.color: #x3d3d3d
            media := TabIconOn{label.draw_icon.svg: crate_resource("self://resources/icons/media.svg") label.on_click: || { ui.browser_body.catalog.catalog_head.set_text("All media"); ui.browser_body.catalog.catalog_status.set_text("") }}
            effects := TabIcon{draw_icon +: {svg: crate_resource("self://resources/icons/effects.svg")} on_click: || { ui.browser_body.catalog.catalog_head.set_text("All effects"); ui.browser_body.catalog.catalog_status.set_text("") }}
            create := TabIcon{draw_icon +: {svg: crate_resource("self://resources/icons/create.svg")} on_click: || { ui.browser_body.catalog.catalog_head.set_text("All create"); ui.browser_body.catalog.catalog_status.set_text("") }}
            panels := TabIcon{draw_icon +: {svg: crate_resource("self://resources/icons/panels.svg")} on_click: || { ui.browser_body.catalog.catalog_head.set_text("All panels"); ui.browser_body.catalog.catalog_status.set_text("") }}
        }

        browser_body := SolidView{width: Fill height: Fill flow: Right
            rail := SolidView{width: 112 height: Fill flow: Down padding: Inset{bottom: 2} show_bg: true new_batch: true draw_bg.color: #x4f4f4f
                collections := RailCap{text: "Collections"}
                favorite := RailRow{text: "Favorite" draw_icon +: {svg: crate_resource("self://resources/icons/star.svg") color: #xf20813}}
                broll := RailRow{text: "B-roll" draw_icon +: {svg: crate_resource("self://resources/icons/video.svg") color: #x4db7bd}}
                brand := RailRow{text: "Brand" draw_icon +: {svg: crate_resource("self://resources/icons/tag.svg") color: #xa676c5}}
                library := RailCap{text: "Library"}
                all_media := RailRowOn{label.text: "All media" label.draw_icon.svg: crate_resource("self://resources/icons/media.svg") label.on_click: || { ui.browser_body.catalog.catalog_status.set_text("") }}
                video := RailRow{text: "Video" draw_icon +: {svg: crate_resource("self://resources/icons/video.svg")}}
                images := RailRow{text: "Images" draw_icon +: {svg: crate_resource("self://resources/icons/image.svg")}}
                audio := RailRow{text: "Audio" draw_icon +: {svg: crate_resource("self://resources/icons/audio.svg")}}
                places := RailCap{text: "Places"}
                starter := RailRow{text: "Starter Media" draw_icon +: {svg: crate_resource("self://resources/icons/folder.svg")}}
                project_assets := RailRow{text: "Project assets" draw_icon +: {svg: crate_resource("self://resources/icons/project.svg")}}
                motion_assets := RailRow{text: "Motion assets" draw_icon +: {svg: crate_resource("self://resources/icons/folder.svg")}}
                add_folder := RailRow{text: "Add Folder..." draw_icon +: {svg: crate_resource("self://resources/icons/create.svg") color: #x909090} draw_text.color: #x909090}
            }
            browser_owner_divider := PaneDivider{}
            catalog := SolidView{width: Fill height: Fill flow: Down show_bg: true new_batch: true draw_bg.color: #x4f4f4f
                catalog_head_row := SolidView{width: Fill height: 18 flow: Right spacing: 4 align: Align{y: 0.5} padding: Inset{left: 6 right: 2} show_bg: true new_batch: true draw_bg.color: #x606060
                    catalog_head := Label{text: "All media" width: Fill draw_text.color: #xe4e4e4 draw_text.text_style: theme.font_regular{font_size: 8 line_spacing: 1.0 top_drop: 0.0}}
                    catalog_status := Label{text: "" width: Fit draw_text.color: #x9d9d9d draw_text.text_style: theme.font_regular{font_size: 8 line_spacing: 1.0 top_drop: 0.0}}
                    view_modes := View{width: Fit height: 16 flow: Right align: Align{y: 0.5}
                        mode_thumb := IconButton{width: 18 draw_icon +: {svg: crate_resource("self://resources/icons/media.svg")}}
                        mode_grid := IconButton{width: 18 draw_icon +: {svg: crate_resource("self://resources/icons/grid.svg") color: #xe4e4e4}}
                        mode_list := IconButton{width: 18 draw_icon +: {svg: crate_resource("self://resources/icons/list.svg")}}
                    }
                }
                filter_shelf := SolidView{width: Fill height: 20 flow: Right spacing: 2 align: Align{y: 0.5} padding: Inset{left: 4 right: 4} show_bg: true new_batch: true draw_bg.color: #x4f4f4f
                    filter_label := IconButton{width: 18 draw_icon +: {svg: crate_resource("self://resources/icons/filter.svg")}}
                    video_chip := FilterChip{label.text: "Video"}
                    broll_chip := FilterChip{label.text: "B-roll"}
                    clear_chip := IconButton{width: 18 draw_icon +: {svg: crate_resource("self://resources/icons/clear.svg")}}
                }
                result_list := SolidView{width: Fill height: Fill flow: Down padding: Inset{top: 1 bottom: 1} show_bg: true new_batch: true draw_bg.color: #x4f4f4f
                    clip := FileRow{text: "clip.mp4" draw_icon +: {svg: crate_resource("self://resources/icons/video.svg")} on_click: || select_asset("Starter Clip", "clip.mp4", "video · B-roll")}
                    mark := FileRow{text: "mark.svg" draw_icon +: {svg: crate_resource("self://resources/icons/motolii.svg")} on_click: || select_asset("Starter Mark", "mark.svg", "image · Brand")}
                    still := FileRow{text: "still.png" draw_icon +: {svg: crate_resource("self://resources/icons/image.svg")} on_click: || select_asset("Starter Still", "still.png", "image · B-roll")}
                    tone := FileRow{text: "tone.wav" draw_icon +: {svg: crate_resource("self://resources/icons/audio.svg")} on_click: || select_asset("Starter Tone", "tone.wav", "audio · WAV")}
                    project_clip := FileRow{text: "intro.mp4" draw_icon +: {svg: crate_resource("self://resources/icons/project.svg")} on_click: || select_asset("Project Intro", "intro.mp4", "video · Project assets")}
                    motion_clip := FileRow{text: "grain.mp4" draw_icon +: {svg: crate_resource("self://resources/icons/video.svg")} on_click: || select_asset("Motion Grain", "grain.mp4", "video · Motion assets")}
                }
                selection_rule := SeamRule{}
                selection := SolidView{width: Fill height: 20 flow: Right spacing: 4 align: Align{y: 0.5} padding: Inset{left: 6 right: 4} show_bg: true new_batch: true draw_bg.color: #x3d3d3d
                    selection_dot := SolidView{width: 5 height: 5 show_bg: true new_batch: true draw_bg.color: #x6b8d96}
                    selection_name := Label{text: "clip.mp4" width: Fit draw_text.color: #xe4e4e4 draw_text.text_style: theme.font_regular{font_size: 8 line_spacing: 1.0 top_drop: 0.0}}
                    selection_type := Label{text: "video · B-roll" width: Fill draw_text.color: #x9d9d9d draw_text.text_style: theme.font_regular{font_size: 8 line_spacing: 1.0 top_drop: 0.0}}
                    clear_selection := IconButton{width: 18 draw_icon +: {svg: crate_resource("self://resources/icons/clear.svg")}}
                }
            }
        }
    }
}

#[derive(Script, ScriptHook, WidgetRegister)]
pub struct BrowserSurface {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
}

impl WidgetNode for BrowserSurface {
    fn widget_uid(&self) -> WidgetUid {
        self.uid
    }

    fn children(&self, visit: &mut dyn FnMut(LiveId, WidgetRef)) {
        self.view.children(visit);
    }

    fn walk(&mut self, cx: &mut Cx) -> Walk {
        self.view.walk(cx)
    }

    fn area(&self) -> Area {
        self.view.area()
    }

    fn redraw(&mut self, cx: &mut Cx) {
        self.view.redraw(cx);
    }
}

impl Widget for BrowserSurface {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
