//! Stage 表示枠。画素経路（Shared / import / render_into）は持たない。
use makepad_widgets::*;

// 正本: Ableton Live 12 Dark 実画面（2026-08-26 添付）からのサンプル値。記憶で埋めない。
//   バー #3d3d3d / 面 #4f4f4f / 縁1px #2d2d2d / 窪み #282828
//   明字 #dddddd / 墨 #ababab / 琥珀 #c49a38
// 形の言語: フラット暗面・角丸ゼロ・影なし。縁は 1px 暗線か明度差だけ。数値は窪み矩形。
script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    let IconButton = ButtonFlatterIcon{
        width: 24
        height: 22
        icon_walk: Walk{width: 13 height: 13}
        padding: Inset{left: 0 right: 0}
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {color: #xababab}
    }

    let IconFlatButton = ButtonFlatIcon{
        width: 24
        height: 22
        icon_walk: Walk{width: 13 height: 13}
        padding: Inset{left: 0 right: 0}
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {color: #xdddddd}
    }

    // 窪み矩形 — バー上に沈む数値欄。動作なし、見た目だけ
    let ValueWell = SolidView{
        width: Fit
        height: 16
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 5 right: 5}
        show_bg: true
        new_batch: true
        draw_bg.color: #x282828
    }

    mod.widgets.StageChromeBase = #(StageChrome::register_widget(vm))
    mod.widgets.StageChrome = set_type_default() do mod.widgets.StageChromeBase{
        width: Fill
        height: Fill
        flow: Down
        show_bg: true
        new_batch: true
        draw_bg.color: #x2d2d2d

        stage_head := SolidView{width: Fill height: 26 flow: Right spacing: 6 align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: #x3d3d3d
            stage_title := Label{text: "STAGE" width: 44 draw_text.color: #xdddddd draw_text.text_style: theme.font_bold{font_size: 8}}
            live_dot := SolidView{width: 5 height: 5 draw_bg.color: #xc49a38}
            live_source := Label{text: "RERUN" width: 42 draw_text.color: #xababab draw_text.text_style: theme.font_code{font_size: 8}}
            camera := IconFlatButton{width: 28 height: 20 draw_bg.color: #x282828 icon_walk: Walk{width: 13 height: 13} draw_icon +: {svg: crate_resource("self://resources/icons/camera.svg") color: #xc49a38} on_click: || { ui.stage_mode.set_text("CAMERA") }}
            user := IconButton{width: 28 height: 20 icon_walk: Walk{width: 13 height: 13} draw_icon +: {svg: crate_resource("self://resources/icons/user_view.svg")} on_click: || { ui.stage_mode.set_text("USER VIEW") }}
            stage_spacer := SolidView{width: Fill height: 1}
            tool_select := IconFlatButton{width: 30 height: 20 draw_bg.color: #x282828 icon_walk: Walk{width: 13 height: 13} draw_icon +: {svg: crate_resource("self://resources/icons/select.svg") color: #xc49a38}}
            tool_shape := IconButton{width: 30 height: 20 icon_walk: Walk{width: 13 height: 13} draw_icon +: {svg: crate_resource("self://resources/icons/shape.svg")}}
            tool_pen := IconButton{width: 30 height: 20 icon_walk: Walk{width: 13 height: 13} draw_icon +: {svg: crate_resource("self://resources/icons/pen.svg")}}
        }
        head_edge := SolidView{width: Fill height: 1 show_bg: true new_batch: true draw_bg.color: #x2d2d2d}
        stage_void := SolidView{width: Fill height: Fill flow: Down align: Align{x: 0.5 y: 0.5} padding: Inset{left: 8 right: 8 top: 8 bottom: 8} show_bg: true new_batch: true draw_bg.color: #x4f4f4f
            comp_frame := SolidView{width: 722 height: 407 padding: 1 show_bg: true new_batch: true draw_bg.color: #x2d2d2d
                comp := SolidView{width: 720 height: 405 flow: Overlay show_bg: true new_batch: true draw_bg.color: #x000000
                    // min_width/min_height: SharedPresentable textures have no
                    // vec_width_height(), so Image falls back to these (default 0
                    // = zero-sized quad = invisible stage).
                    stage_frame := Image{width: Fill height: Fill fit: ImageFit.Smallest}
                    stage_error := Label{width: Fill height: Fill align: Align{x: 0.5 y: 0.5} text: "" draw_text.color: #xc49a38 draw_text.text_style: theme.font_code{font_size: 10}}
                }
            }
        }
        band_edge := SolidView{width: Fill height: 1 show_bg: true new_batch: true draw_bg.color: #x2d2d2d}
        stage_band := SolidView{width: Fill height: 24 flow: Right spacing: 8 align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: #x3d3d3d
            mode_well := ValueWell{
                stage_mode := Label{text: "CAMERA" width: 48 draw_text.color: #xdddddd draw_text.text_style: theme.font_code{font_size: 8}}
            }
            resolution_well := ValueWell{
                resolution := Label{text: "1920 × 1080" width: 76 draw_text.color: #xdddddd draw_text.text_style: theme.font_code{font_size: 8}}
            }
            frame_rate_well := ValueWell{
                frame_rate := Label{text: "30 fps" width: 42 draw_text.color: #xdddddd draw_text.text_style: theme.font_code{font_size: 8}}
            }
            off_frame_dot := SolidView{width: 4 height: 4 draw_bg.color: #xc49a38}
            selection_state := Label{text: "CHORUS LYRICS · OFF FRAME" width: Fit draw_text.color: #xababab draw_text.text_style: theme.font_code{font_size: 8}}
            stage_band_spacer := SolidView{width: Fill height: 1}
            zoom_well := ValueWell{
                zoom := Label{text: "62%" width: 30 draw_text.color: #xdddddd draw_text.text_style: theme.font_code{font_size: 8}}
            }
            check := IconButton{width: 22 height: 18 icon_walk: Walk{width: 12 height: 12} draw_icon +: {svg: crate_resource("self://resources/icons/checker.svg")}}
            safe := IconButton{width: 22 height: 18 icon_walk: Walk{width: 12 height: 12} draw_icon +: {svg: crate_resource("self://resources/icons/safe.svg")}}
        }
    }
}

#[derive(Script, ScriptHook, WidgetRegister)]
pub struct StageChrome {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
}

impl WidgetNode for StageChrome {
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

impl Widget for StageChrome {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
