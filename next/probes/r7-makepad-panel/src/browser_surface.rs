//! Browser パネル枠。素材一覧の意味書きは波1。iced は引かない。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

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

    let PaneDivider = SolidView{
        width: 1
        height: Fill
        show_bg: true
        new_batch: true
        draw_bg.color: #x1d1d1d
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
        draw_bg.color: #x363636

        browser_head := SolidView{width: Fill height: 26 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: #x2f2f2f
            title := Label{text: "Browser" width: Fill draw_text.color: #xcfcfcf draw_text.text_style: theme.font_bold{font_size: 11}}
            local := Label{text: "LOCAL" width: Fit draw_text.color: #x757575 draw_text.text_style: theme.font_bold{font_size: 8}}
        }
        browser_toolbar := SolidView{width: Fill height: 30 flow: Right spacing: 2 align: Align{y: 0.5} padding: Inset{left: 4 right: 4} show_bg: true new_batch: true draw_bg.color: #x363636
            back := IconButton{width: 20 draw_icon +: {svg: crate_resource("self://resources/icons/back.svg")}}
            forward := IconButton{width: 20 draw_icon +: {svg: crate_resource("self://resources/icons/forward.svg")}}
            search := IconFlatButton{width: Fill height: 21 draw_bg.color: #x3e3e3e draw_bg.border_size: 1.0 draw_icon +: {svg: crate_resource("self://resources/icons/search.svg") color: #x85898a}}
            filters := IconButton{width: 28 height: 21 draw_icon +: {svg: crate_resource("self://resources/icons/filter.svg")}}
            tags := IconButton{width: 28 height: 21 draw_icon +: {svg: crate_resource("self://resources/icons/tag.svg")}}
        }
        tabs := SolidView{width: Fill height: 26 flow: Right show_bg: true new_batch: true draw_bg.color: #x363636
            media := IconFlatButton{width: Fill height: 26 draw_bg.color: #x282828 draw_bg.color_hover: #x3e3e3e draw_bg.border_size: 0.0 draw_icon +: {svg: crate_resource("self://resources/icons/media.svg")} on_click: || { ui.browser_body.catalog.catalog_head.set_text("All media"); ui.browser_body.catalog.catalog_status.set_text("") }}
            effects := IconButton{width: Fill height: 26 draw_icon +: {svg: crate_resource("self://resources/icons/effects.svg")} on_click: || { ui.browser_body.catalog.catalog_head.set_text("All effects"); ui.browser_body.catalog.catalog_status.set_text("") }}
            create := IconButton{width: Fill height: 26 draw_icon +: {svg: crate_resource("self://resources/icons/create.svg")} on_click: || { ui.browser_body.catalog.catalog_head.set_text("All create"); ui.browser_body.catalog.catalog_status.set_text("") }}
            panels := IconButton{width: Fill height: 26 draw_icon +: {svg: crate_resource("self://resources/icons/panels.svg")} on_click: || { ui.browser_body.catalog.catalog_head.set_text("All panels"); ui.browser_body.catalog.catalog_status.set_text("") }}
        }

        browser_body := SolidView{width: Fill height: Fill flow: Right
            rail := SolidView{width: 112 height: Fill flow: Down padding: Inset{top: 2 bottom: 6} show_bg: true new_batch: true draw_bg.color: #x363636
                collections := Label{text: "COLLECTIONS" width: Fill height: 16 padding: Inset{left: 8} draw_text.color: #x757575 draw_text.text_style: theme.font_bold{font_size: 7.5}}
                favorite := ButtonFlatter{text: "Favorite" width: Fill height: 19 icon_walk: Walk{width: 12 height: 12} draw_icon +: {svg: crate_resource("self://resources/icons/star.svg") color: #xc89a40} draw_text.color: #xc89a40 draw_text.text_style: theme.font_regular{font_size: 8.5}}
                broll := ButtonFlatter{text: "B-roll" width: Fill height: 19 icon_walk: Walk{width: 12 height: 12} draw_icon +: {svg: crate_resource("self://resources/icons/video.svg") color: #x4db7bd} draw_text.color: #x4db7bd draw_text.text_style: theme.font_regular{font_size: 8.5}}
                brand := ButtonFlatter{text: "Brand" width: Fill height: 19 icon_walk: Walk{width: 12 height: 12} draw_icon +: {svg: crate_resource("self://resources/icons/tag.svg") color: #xa676c5} draw_text.color: #xa676c5 draw_text.text_style: theme.font_regular{font_size: 8.5}}
                library := Label{text: "LIBRARY" width: Fill height: 16 padding: Inset{left: 8} draw_text.color: #x757575 draw_text.text_style: theme.font_bold{font_size: 7.5}}
                all_media := ButtonFlat{text: "All media" width: Fill height: 19 icon_walk: Walk{width: 12 height: 12} draw_icon +: {svg: crate_resource("self://resources/icons/media.svg")} draw_bg.color: #x4a4a4a draw_bg.border_size: 0.0 draw_bg.border_radius: 0.0 draw_text.color: #xcfcfcf draw_text.text_style: theme.font_regular{font_size: 8.5} on_click: || { ui.browser_body.catalog.catalog_status.set_text("") }}
                video := ButtonFlatter{text: "Video" width: Fill height: 19 icon_walk: Walk{width: 12 height: 12} draw_icon +: {svg: crate_resource("self://resources/icons/video.svg")} draw_text.color: #xb7b7b7 draw_text.text_style: theme.font_regular{font_size: 8.5}}
                images := ButtonFlatter{text: "Images" width: Fill height: 19 icon_walk: Walk{width: 12 height: 12} draw_icon +: {svg: crate_resource("self://resources/icons/image.svg")} draw_text.color: #xb7b7b7 draw_text.text_style: theme.font_regular{font_size: 8.5}}
                audio := ButtonFlatter{text: "Audio" width: Fill height: 19 icon_walk: Walk{width: 12 height: 12} draw_icon +: {svg: crate_resource("self://resources/icons/audio.svg")} draw_text.color: #xb7b7b7 draw_text.text_style: theme.font_regular{font_size: 8.5}}
                places := Label{text: "PLACES" width: Fill height: 16 padding: Inset{left: 8} draw_text.color: #x757575 draw_text.text_style: theme.font_bold{font_size: 7.5}}
                starter := ButtonFlatter{text: "Starter Media" width: Fill height: 19 icon_walk: Walk{width: 12 height: 12} draw_icon +: {svg: crate_resource("self://resources/icons/folder.svg")} draw_text.color: #xb7b7b7 draw_text.text_style: theme.font_regular{font_size: 8.5}}
                project_assets := ButtonFlatter{text: "Project assets" width: Fill height: 19 icon_walk: Walk{width: 12 height: 12} draw_icon +: {svg: crate_resource("self://resources/icons/project.svg")} draw_text.color: #xb7b7b7 draw_text.text_style: theme.font_regular{font_size: 8.5}}
                motion_assets := ButtonFlatter{text: "Motion assets" width: Fill height: 19 icon_walk: Walk{width: 12 height: 12} draw_icon +: {svg: crate_resource("self://resources/icons/folder.svg")} draw_text.color: #xb7b7b7 draw_text.text_style: theme.font_regular{font_size: 8.5}}
                add_folder := ButtonFlatter{text: "Add folder" width: Fill height: 19 icon_walk: Walk{width: 12 height: 12} draw_icon +: {svg: crate_resource("self://resources/icons/create.svg")} draw_text.color: #x767a7c draw_text.text_style: theme.font_regular{font_size: 8.5}}
            }
            browser_owner_divider := PaneDivider{}
            catalog := SolidView{width: Fill height: Fill flow: Down show_bg: true new_batch: true draw_bg.color: #x363636
                catalog_head_row := SolidView{width: Fill height: 31 flow: Right align: Align{y: 0.5} padding: Inset{left: 6 right: 6} show_bg: true new_batch: true draw_bg.color: #x363636
                    catalog_copy := SolidView{width: Fill height: 25 flow: Down
                        catalog_head := Label{text: "All media" width: Fill height: 14 draw_text.color: #xcfcfcf draw_text.text_style: theme.font_bold{font_size: 9}}
                        catalog_status := Label{text: "" width: Fill height: 11 draw_text.color: #x808487 draw_text.text_style: theme.font_regular{font_size: 8}}
                    }
                    view_modes := SolidView{width: 70 height: 22 flow: Right align: Align{y: 0.5}
                        mode_thumb := IconButton{width: 22 height: 21 draw_icon +: {svg: crate_resource("self://resources/icons/media.svg")}}
                        mode_grid := IconFlatButton{width: 22 height: 21 draw_bg.color: #x38362c draw_bg.border_size: 0.0 draw_bg.border_radius: 0.0 draw_icon +: {svg: crate_resource("self://resources/icons/grid.svg") color: #xe5cf9b}}
                        mode_list := IconButton{width: 22 height: 21 draw_icon +: {svg: crate_resource("self://resources/icons/list.svg")}}
                    }
                }
                filter_shelf := SolidView{width: Fill height: 24 flow: Right spacing: 2 align: Align{y: 0.5} padding: Inset{left: 4 right: 4} show_bg: true new_batch: true draw_bg.color: #x363636
                    filter_label := IconButton{width: 20 height: 20 draw_icon +: {svg: crate_resource("self://resources/icons/filter.svg") color: #x757575}}
                    video_chip := ButtonFlat{text: "Video" width: 42 height: 17 draw_bg.color: #x464646 draw_bg.border_size: 0.0 draw_bg.border_radius: 8.0 draw_text.color: #xc9ccca draw_text.text_style: theme.font_regular{font_size: 8}}
                    broll_chip := ButtonFlat{text: "B-roll" width: 46 height: 17 draw_bg.color: #x464646 draw_bg.border_size: 0.0 draw_bg.border_radius: 8.0 draw_text.color: #xc9ccca draw_text.text_style: theme.font_regular{font_size: 8}}
                    clear_chip := IconButton{width: 20 height: 17 draw_icon +: {svg: crate_resource("self://resources/icons/clear.svg")}}
                }
                result_grid := SolidView{width: Fill height: Fill flow: Down padding: Inset{left: 1 right: 1 bottom: 3} show_bg: true new_batch: true draw_bg.color: #x363636
                    cards_a := SolidView{width: Fill height: 76 flow: Right
                        clip_card := SolidView{width: Fill height: 76 flow: Down padding: Inset{left: 2 right: 2 top: 2 bottom: 2}
                            clip := IconFlatButton{width: Fill height: 48 icon_walk: Walk{width: 18 height: 18} draw_bg.color: #x5d7899 draw_bg.border_size: 1.0 draw_icon +: {svg: crate_resource("self://resources/icons/video.svg")} on_click: || select_asset("Starter Clip", "clip.mp4", "video · B-roll")}
                            clip_name := Label{text: "clip.mp4" width: Fill height: 12 draw_text.color: #xcfcfcf draw_text.text_style: theme.font_regular{font_size: 8}}
                            clip_meta := Label{text: "video · B-roll" width: Fill height: 10 draw_text.color: #x9ea2a3 draw_text.text_style: theme.font_code{font_size: 7.5}}
                        }
                        mark_card := SolidView{width: Fill height: 76 flow: Down padding: Inset{left: 2 right: 2 top: 2 bottom: 2}
                            mark := IconFlatButton{width: Fill height: 48 icon_walk: Walk{width: 18 height: 18} draw_bg.color: #x746398 draw_bg.border_size: 1.0 draw_icon +: {svg: crate_resource("self://resources/icons/motolii.svg")} on_click: || select_asset("Starter Mark", "mark.svg", "image · Brand")}
                            mark_name := Label{text: "mark.svg" width: Fill height: 12 draw_text.color: #xcfcfcf draw_text.text_style: theme.font_regular{font_size: 8}}
                            mark_meta := Label{text: "image · Brand" width: Fill height: 10 draw_text.color: #x9ea2a3 draw_text.text_style: theme.font_code{font_size: 7.5}}
                        }
                    }
                    cards_b := SolidView{width: Fill height: 76 flow: Right
                        still_card := SolidView{width: Fill height: 76 flow: Down padding: Inset{left: 2 right: 2 top: 2 bottom: 2}
                            still := IconFlatButton{width: Fill height: 48 icon_walk: Walk{width: 18 height: 18} draw_bg.color: #x88704e draw_bg.border_size: 1.0 draw_icon +: {svg: crate_resource("self://resources/icons/image.svg")} on_click: || select_asset("Starter Still", "still.png", "image · B-roll")}
                            still_name := Label{text: "still.png" width: Fill height: 12 draw_text.color: #xcfcfcf draw_text.text_style: theme.font_regular{font_size: 8}}
                            still_meta := Label{text: "image · B-roll" width: Fill height: 10 draw_text.color: #x9ea2a3 draw_text.text_style: theme.font_code{font_size: 7.5}}
                        }
                        tone_card := SolidView{width: Fill height: 76 flow: Down padding: Inset{left: 2 right: 2 top: 2 bottom: 2}
                            tone := IconFlatButton{width: Fill height: 48 icon_walk: Walk{width: 18 height: 18} draw_bg.color: #x557f6d draw_bg.border_size: 1.0 draw_icon +: {svg: crate_resource("self://resources/icons/audio.svg")} on_click: || select_asset("Starter Tone", "tone.wav", "audio · WAV")}
                            tone_name := Label{text: "tone.wav" width: Fill height: 12 draw_text.color: #xcfcfcf draw_text.text_style: theme.font_regular{font_size: 8}}
                            tone_meta := Label{text: "audio · WAV" width: Fill height: 10 draw_text.color: #x9ea2a3 draw_text.text_style: theme.font_code{font_size: 7.5}}
                        }
                    }
                    cards_c := SolidView{width: Fill height: 76 flow: Right
                        project_card := SolidView{width: Fill height: 76 flow: Down padding: Inset{left: 2 right: 2 top: 2 bottom: 2}
                            project_clip := IconFlatButton{width: Fill height: 48 icon_walk: Walk{width: 18 height: 18} draw_bg.color: #x875459 draw_bg.border_size: 1.0 draw_icon +: {svg: crate_resource("self://resources/icons/project.svg")} on_click: || select_asset("Project Intro", "intro.mp4", "video · Project assets")}
                            project_name := Label{text: "intro.mp4" width: Fill height: 12 draw_text.color: #xcfcfcf draw_text.text_style: theme.font_regular{font_size: 8}}
                            project_meta := Label{text: "video · Project assets" width: Fill height: 10 draw_text.color: #x9ea2a3 draw_text.text_style: theme.font_code{font_size: 7.5}}
                        }
                        motion_card := SolidView{width: Fill height: 76 flow: Down padding: Inset{left: 2 right: 2 top: 2 bottom: 2}
                            motion_clip := IconFlatButton{width: Fill height: 48 icon_walk: Walk{width: 18 height: 18} draw_bg.color: #x546a75 draw_bg.border_size: 1.0 draw_icon +: {svg: crate_resource("self://resources/icons/video.svg")} on_click: || select_asset("Motion Grain", "grain.mp4", "video · Motion assets")}
                            motion_name := Label{text: "grain.mp4" width: Fill height: 12 draw_text.color: #xcfcfcf draw_text.text_style: theme.font_regular{font_size: 8}}
                            motion_meta := Label{text: "video · Motion assets" width: Fill height: 10 draw_text.color: #x9ea2a3 draw_text.text_style: theme.font_code{font_size: 7.5}}
                        }
                    }
                }
                selection := SolidView{width: Fill height: 27 flow: Right align: Align{y: 0.5} padding: Inset{left: 6 right: 6} show_bg: true new_batch: true draw_bg.color: #x282828
                    selection_dot := SolidView{width: 5 height: 5 draw_bg.color: #xd8b574}
                    selection_name := Label{text: "clip.mp4" width: Fit draw_text.color: #xcfcfcf draw_text.text_style: theme.font_regular{font_size: 8}}
                    selection_type := Label{text: "video · B-roll" width: Fill height: 12 draw_text.color: #x83888a draw_text.text_style: theme.font_code{font_size: 8}}
                    clear_selection := IconButton{width: 20 height: 18 draw_icon +: {svg: crate_resource("self://resources/icons/clear.svg") color: #xc7b66c}}
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
