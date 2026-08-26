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
        draw_icon +: {color: mod.tokens.ink.glyph}
    }

    // tab — 「N のうち1つ」は makepad の radio。選択は `active`(instance)。
    // 面の色は uniform(draw call 共有)なので個体ごとに効かない。よって面は
    // instance の hover/down/active から shader で作る(実測 2026-08-27)。
    let TabIcon = RadioButtonTabFlat{
        width: Fill
        height: mod.tokens.size.bar
        icon_walk: Walk{width: mod.tokens.size.icon height: mod.tokens.size.icon}
        label_walk: Walk{width: 0 height: 0}
        padding: Inset{left: 0 right: 0}
        align: Align{x: 0.5 y: 0.5}
        // 反応は即時。ふんわり遷移は「押した感じ」を殺す(利用者裁定 2026-08-27)
        animator.hover.off.from.all: Forward{duration: 0.0}
        animator.hover.on.from.all: Forward{duration: 0.0}
        animator.hover.down.from.all: Forward{duration: 0.0}
        animator.active.off.from.all: Forward{duration: 0.0}
        animator.active.on.from.all: Forward{duration: 0.0}
        draw_bg +: {
            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let sunk = max(self.active, self.down)
                let face = mod.tokens.face.bar.mix(#x4a4a4a, self.hover).mix(#x333333, sunk)
                sdf.rect(0.0, 0.0, self.rect_size.x, self.rect_size.y)
                sdf.fill(face)
                // 押し込みは縁で語る: 上が暗く、下が明るい。枠線では囲まない
                sdf.rect(0.0, 0.0, self.rect_size.x, 1.0)
                sdf.fill(face.mix(#x272727, sunk))
                sdf.rect(0.0, self.rect_size.y - 1.0, self.rect_size.x, 1.0)
                sdf.fill(face.mix(#x5a5a5a, sunk))
                return sdf.result
            }
        }
        draw_icon +: {color: mod.tokens.ink.glyph}
    }

    // rail 行 — 低く詰める(16)。選択は押し込まれた面で語る(文字色は反転しない)
    let RailRow = RadioButtonTabFlat{
        width: Fill
        height: mod.tokens.size.row
        icon_walk: Walk{width: mod.tokens.size.icon height: mod.tokens.size.icon}
        label_walk: Walk{width: Fill height: Fit margin: Inset{left: mod.tokens.space.s4}}
        align: Align{x: 0.0 y: 0.5}
        padding: Inset{left: mod.tokens.space.s5 right: mod.tokens.space.s5}
        // 反応は即時。ふんわり遷移は「押した感じ」を殺す(利用者裁定 2026-08-27)
        animator.hover.off.from.all: Forward{duration: 0.0}
        animator.hover.on.from.all: Forward{duration: 0.0}
        animator.hover.down.from.all: Forward{duration: 0.0}
        animator.active.off.from.all: Forward{duration: 0.0}
        animator.active.on.from.all: Forward{duration: 0.0}
        draw_bg +: {
            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let sunk = max(self.active, self.down)
                let face = mod.tokens.face.panel.mix(#x5c5c5c, self.hover).mix(#x444444, sunk)
                sdf.rect(0.0, 0.0, self.rect_size.x, self.rect_size.y)
                sdf.fill(face)
                // 押し込みは縁で語る: 上が暗く、下が明るい。枠線では囲まない
                sdf.rect(0.0, 0.0, self.rect_size.x, 1.0)
                sdf.fill(face.mix(#x313131, sunk))
                sdf.rect(0.0, self.rect_size.y - 1.0, self.rect_size.x, 1.0)
                sdf.fill(face.mix(#x616161, sunk))
                return sdf.result
            }
        }
        draw_icon +: {color: mod.tokens.ink.glyph}
        draw_text.color: mod.tokens.ink.strong
        draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.base line_spacing: 1.0 top_drop: 0.0}
    }

    // 節見出し — Collections / Library / Places。薄字、上に群間の余白
    let RailCap = Label{
        width: Fill
        height: mod.tokens.size.cap
        padding: Inset{left: mod.tokens.space.s5 top: mod.tokens.space.s3}
        draw_text.color: mod.tokens.ink.muted
        draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm line_spacing: 1.0 top_drop: 0.0}
    }

    // 群の境は線で引く(利用者裁定 2026-08-27)。面の明度差が既に境になっている
    // tab strip 直下だけは引かない — 二重の境は境でなくなる
    let RailRule = SolidView{
        width: Fill
        height: mod.tokens.rule.size
        margin: Inset{top: mod.tokens.space.s3}
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.rule.owner
    }

    // 有効フィルタ chip — 選択の言語（ベタ #6b8d96 + 濃字 #133342、角丸なし）。
    // `chrome/parts/nav.rs` の ChromeChipOn と同一サンプル値。本 mod は chrome より先に
    // eval されるため参照でなく値で置く（未登録名の参照は葉が落ちる）
    let FilterChip = SolidView{
        width: Fit
        height: mod.tokens.size.chip
        align: Align{y: 0.5}
        show_bg: true
        new_batch: true
        draw_bg.color: #x6b8d96
        label := ButtonFlatter{
            width: Fit
            height: Fit
            padding: Inset{left: 4 right: 4}
            draw_text.color: #x133342
            draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm line_spacing: 1.0 top_drop: 0.0}
        }
    }

    // ファイル行 — Live 右リストの1行。低く詰める、ベタ、角丸なし
    let FileRow = ButtonFlat{
        width: Fill
        height: mod.tokens.size.row
        icon_walk: Walk{width: mod.tokens.size.icon height: mod.tokens.size.icon}
        spacing: mod.tokens.space.s4
        align: Align{x: 0.0 y: 0.5}
        padding: Inset{left: mod.tokens.space.s5 right: mod.tokens.space.s5}
        draw_bg.color: mod.tokens.face.panel
        draw_bg.color_hover: #x5c5c5c
        draw_bg.color_down: #x2d2d2d
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {color: mod.tokens.ink.glyph}
        draw_text.color: mod.tokens.ink.body
        draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.base line_spacing: 1.0 top_drop: 0.0}
    }

    // rail とリストの境 — 画像実測 #343434 の縦 1px
    let PaneDivider = SolidView{
        width: mod.tokens.rule.size
        height: Fill
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.rule.pane
    }

    // 継ぎ目 — 横 1px 暗線（枠線で囲まない）
    let SeamRule = SolidView{
        width: Fill
        height: mod.tokens.rule.size
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.rule.seam
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
        draw_bg.color: mod.tokens.face.panel

        browser_head := SolidView{width: Fill height: 22 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.bar
            title := Label{text: "Browser" width: Fill draw_text.color: #xd0d0d0 draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg line_spacing: 1.0 top_drop: 0.0}}
            local := Label{text: "LOCAL" width: Fit draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs line_spacing: 1.0 top_drop: 0.0}}
        }
        head_rule := SeamRule{}
        browser_toolbar := SolidView{width: Fill height: mod.tokens.size.toolbar flow: Right spacing: mod.tokens.space.s2 align: Align{y: 0.5} padding: Inset{left: 4 right: 4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            back := IconButton{width: 18 draw_icon +: {svg: crate_resource("self://resources/icons/back.svg")}}
            forward := IconButton{width: 18 draw_icon +: {svg: crate_resource("self://resources/icons/forward.svg")}}
            search := SolidView{width: Fill height: mod.tokens.size.field flow: Right spacing: mod.tokens.space.s1 align: Align{y: 0.5} padding: Inset{left: 4 right: 4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.sunk
                search_glyph := IconButton{width: 15 draw_icon +: {svg: crate_resource("self://resources/icons/search.svg") color: #x909090}}
                search_hint := Label{text: "Search (Cmd + F)" width: Fill draw_text.color: #x909090 draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm line_spacing: 1.0 top_drop: 0.0}}
            }
            filters := IconButton{width: 22 draw_icon +: {svg: crate_resource("self://resources/icons/filter.svg")}}
            tags := IconButton{width: 22 draw_icon +: {svg: crate_resource("self://resources/icons/tag.svg")}}
        }
        tabs := SolidView{width: Fill height: 22 flow: Right show_bg: true new_batch: true draw_bg.color: mod.tokens.face.bar
            media := TabIcon{draw_icon +: {svg: crate_resource("self://resources/icons/media.svg")}}
            effects := TabIcon{draw_icon +: {svg: crate_resource("self://resources/icons/effects.svg")}}
            create := TabIcon{draw_icon +: {svg: crate_resource("self://resources/icons/create.svg")}}
            panels := TabIcon{draw_icon +: {svg: crate_resource("self://resources/icons/panels.svg")}}
        }

        browser_body := SolidView{width: Fill height: Fill flow: Right
            rail := SolidView{width: mod.tokens.size.rail height: Fill flow: Down padding: Inset{bottom: 2} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
                collections := RailCap{text: "Collections"}
                favorite := RailRow{text: "Favorite" draw_icon +: {svg: crate_resource("self://resources/icons/star.svg") color: #xf20813}}
                broll := RailRow{text: "B-roll" draw_icon +: {svg: crate_resource("self://resources/icons/video.svg") color: #x4db7bd}}
                brand := RailRow{text: "Brand" draw_icon +: {svg: crate_resource("self://resources/icons/tag.svg") color: #xa676c5}}
                library_rule := RailRule{}
                library := RailCap{text: "Library"}
                all_media := RailRow{text: "All media" draw_icon +: {svg: crate_resource("self://resources/icons/media.svg")}}
                video := RailRow{text: "Video" draw_icon +: {svg: crate_resource("self://resources/icons/video.svg")}}
                images := RailRow{text: "Images" draw_icon +: {svg: crate_resource("self://resources/icons/image.svg")}}
                audio := RailRow{text: "Audio" draw_icon +: {svg: crate_resource("self://resources/icons/audio.svg")}}
                places_rule := RailRule{}
                places := RailCap{text: "Places"}
                starter := RailRow{text: "Starter Media" draw_icon +: {svg: crate_resource("self://resources/icons/folder.svg")}}
                project_assets := RailRow{text: "Project assets" draw_icon +: {svg: crate_resource("self://resources/icons/project.svg")}}
                motion_assets := RailRow{text: "Motion assets" draw_icon +: {svg: crate_resource("self://resources/icons/folder.svg")}}
                add_folder := RailRow{text: "Add Folder..." draw_icon +: {svg: crate_resource("self://resources/icons/create.svg") color: #x909090} draw_text.color: #x909090}
            }
            browser_owner_divider := PaneDivider{}
            catalog := SolidView{width: Fill height: Fill flow: Down show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
                catalog_head_row := SolidView{width: Fill height: mod.tokens.size.row_tight flow: Right spacing: mod.tokens.space.s2 align: Align{y: 0.5} padding: Inset{left: 6 right: 2} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.head
                    catalog_head := Label{text: "All media" width: Fill draw_text.color: mod.tokens.ink.strong draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm line_spacing: 1.0 top_drop: 0.0}}
                    catalog_status := Label{text: "" width: Fit draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm line_spacing: 1.0 top_drop: 0.0}}
                    view_modes := View{width: Fit height: 16 flow: Right align: Align{y: 0.5}
                        mode_thumb := IconButton{width: 18 draw_icon +: {svg: crate_resource("self://resources/icons/media.svg")}}
                        mode_grid := IconButton{width: 18 draw_icon +: {svg: crate_resource("self://resources/icons/grid.svg") color: mod.tokens.ink.strong}}
                        mode_list := IconButton{width: 18 draw_icon +: {svg: crate_resource("self://resources/icons/list.svg")}}
                    }
                }
                filter_shelf := SolidView{width: Fill height: mod.tokens.size.cap flow: Right spacing: mod.tokens.space.s1 align: Align{y: 0.5} padding: Inset{left: 4 right: 4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
                    filter_label := IconButton{width: 18 draw_icon +: {svg: crate_resource("self://resources/icons/filter.svg")}}
                    video_chip := FilterChip{label.text: "Video"}
                    broll_chip := FilterChip{label.text: "B-roll"}
                    clear_chip := IconButton{width: 18 draw_icon +: {svg: crate_resource("self://resources/icons/clear.svg")}}
                }
                result_list := SolidView{width: Fill height: Fill flow: Down padding: Inset{top: 5 bottom: 5} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
                    clip := FileRow{text: "clip.mp4" draw_icon +: {svg: crate_resource("self://resources/icons/video.svg")} on_click: || select_asset("Starter Clip", "clip.mp4", "video · B-roll")}
                    mark := FileRow{text: "mark.svg" draw_icon +: {svg: crate_resource("self://resources/icons/motolii.svg")} on_click: || select_asset("Starter Mark", "mark.svg", "image · Brand")}
                    still := FileRow{text: "still.png" draw_icon +: {svg: crate_resource("self://resources/icons/image.svg")} on_click: || select_asset("Starter Still", "still.png", "image · B-roll")}
                    tone := FileRow{text: "tone.wav" draw_icon +: {svg: crate_resource("self://resources/icons/audio.svg")} on_click: || select_asset("Starter Tone", "tone.wav", "audio · WAV")}
                    project_clip := FileRow{text: "intro.mp4" draw_icon +: {svg: crate_resource("self://resources/icons/project.svg")} on_click: || select_asset("Project Intro", "intro.mp4", "video · Project assets")}
                    motion_clip := FileRow{text: "grain.mp4" draw_icon +: {svg: crate_resource("self://resources/icons/video.svg")} on_click: || select_asset("Motion Grain", "grain.mp4", "video · Motion assets")}
                }
                selection_rule := SeamRule{}
                selection := SolidView{width: Fill height: mod.tokens.size.cap flow: Right spacing: mod.tokens.space.s2 align: Align{y: 0.5} padding: Inset{left: 6 right: 4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.bar
                    selection_dot := SolidView{width: 5 height: 5 show_bg: true new_batch: true draw_bg.color: #x6b8d96}
                    selection_name := Label{text: "clip.mp4" width: Fit draw_text.color: mod.tokens.ink.strong draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm line_spacing: 1.0 top_drop: 0.0}}
                    selection_type := Label{text: "video · B-roll" width: Fill draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm line_spacing: 1.0 top_drop: 0.0}}
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
