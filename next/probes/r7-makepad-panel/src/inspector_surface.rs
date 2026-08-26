//! Inspector パネル枠。property / key の意味書きは波1。iced は引かない。
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

    mod.widgets.InspectorSurfaceBase = #(InspectorSurface::register_widget(vm))
    mod.widgets.InspectorSurface = set_type_default() do mod.widgets.InspectorSurfaceBase{
        width: Fill
        height: Fill
        flow: Down
        show_bg: true
        new_batch: true
        draw_bg.color: #x363636

        inspector_head := SolidView{width: Fill height: 26 flow: Right spacing: 6 align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: #x2f2f2f
            accent_dot := SolidView{width: 5 height: 5 draw_bg.color: #x8eb086 draw_bg.border_radius: 2.5}
            title := Label{text: "Inspector" width: Fill draw_text.color: #xb8b8b8 draw_text.text_style: theme.font_bold{font_size: 11}}
            context := Label{text: "Layer 7 · Solid" width: Fit draw_text.color: #x757575 draw_text.text_style: theme.font_code{font_size: 8}}
        }
        modes := SolidView{width: Fill height: 28 flow: Right show_bg: true new_batch: true draw_bg.color: #x363636
            effect := ButtonFlat{text: "Effect" width: Fill height: 28 icon_walk: Walk{width: 12 height: 12} draw_icon +: {svg: crate_resource("self://resources/icons/effects.svg") color: #xd8b574} draw_bg.color: #x242424 draw_bg.border_size: 0.0 draw_bg.border_radius: 0.0 draw_text.color: #xb8b8b8 draw_text.text_style: theme.font_bold{font_size: 8}}
            layer_mode := ButtonFlatter{text: "Custom" width: Fill height: 28 icon_walk: Walk{width: 12 height: 12} draw_icon +: {svg: crate_resource("self://resources/icons/shape.svg") color: #x757575} draw_text.color: #x757575 draw_text.text_style: theme.font_regular{font_size: 8}}
        }
        selection := SolidView{width: Fill height: 46 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: #x3e3e3e
            glyph := Icon{width: 27 height: 20 icon_walk: Walk{width: 16 height: 16} draw_icon +: {svg: crate_resource("self://resources/icons/shape.svg") color: #x242424} draw_bg +: {color: #xd8c97f}}
            selection_copy := SolidView{width: Fill height: 32 flow: Down padding: Inset{left: 8}
                selection_name := Label{text: "Chorus Lyrics" width: Fill height: 15 draw_text.color: #xb8b8b8 draw_text.text_style: theme.font_bold{font_size: 10}}
                selection_type := Label{text: "Solid · selected · off frame" width: Fill height: 13 draw_text.color: #xa88969 draw_text.text_style: theme.font_code{font_size: 8}}
            }
            mute := IconButton{width: 22 height: 21 icon_walk: Walk{width: 12 height: 12} draw_icon +: {svg: crate_resource("self://resources/icons/mute.svg") color: #x757575}}
            solo := IconFlatButton{width: 22 height: 21 draw_bg.color: #x443d2e draw_bg.border_size: 0.0 draw_bg.border_radius: 0.0 icon_walk: Walk{width: 12 height: 12} draw_icon +: {svg: crate_resource("self://resources/icons/solo.svg") color: #xd8b574}}
        }
        transform_section := SolidView{width: Fill height: 26 flow: Right margin: Inset{top: 4} align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: #x2f2f2f
            section := Label{text: "TRANSFORM" width: Fill draw_text.color: #x8c8c8c draw_text.text_style: theme.font_bold{font_size: 7.5}}
            section_count := Label{text: "1 PROP · 2 KEYS" width: Fit draw_text.color: #x757575 draw_text.text_style: theme.font_code{font_size: 8}}
        }
        columns := SolidView{width: Fill height: 21 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: #x303030
            c0 := Label{text: "PROPERTY" width: Fill draw_text.color: #x757575 draw_text.text_style: theme.font_bold{font_size: 7.5}}
            cx := Label{text: "X" width: 42 draw_text.color: #x757575 draw_text.text_style: theme.font_code{font_size: 8}}
            cy := Label{text: "Y" width: 42 draw_text.color: #x757575 draw_text.text_style: theme.font_code{font_size: 8}}
            cz := Label{text: "Z" width: 42 draw_text.color: #x757575 draw_text.text_style: theme.font_code{font_size: 8}}
            ck := Icon{width: 26 height: 18 icon_walk: Walk{width: 10 height: 10} draw_icon +: {svg: crate_resource("self://resources/icons/keyframe.svg") color: #xd8b574}}
        }
        property_rows := SolidView{width: Fill height: Fill flow: Down
            position_row := SolidView{width: Fill height: 25 flow: Right align: Align{y: 0.5} show_bg: true new_batch: true draw_bg.color: #x363636
                indicator := SolidView{width: 3 height: 8 draw_bg.color: #x78b5b0}
                label := Label{text: "Position" width: Fill padding: Inset{left: 8} draw_text.color: #xb8b8b8 draw_text.text_style: theme.font_regular{font_size: 10}}
                x := ButtonFlat{text: "960" width: 42 height: 21 draw_bg.color: #x242424 draw_bg.border_size: 0.0 draw_bg.border_radius: 0.0 draw_text.color: #xd8b574 draw_text.text_style: theme.font_code{font_size: 9}}
                y := ButtonFlat{text: "540" width: 42 height: 21 draw_bg.color: #x242424 draw_bg.border_size: 0.0 draw_bg.border_radius: 0.0 draw_text.color: #xd8b574 draw_text.text_style: theme.font_code{font_size: 9}}
                z := ButtonFlat{text: "0" width: 42 height: 21 draw_bg.color: #x242424 draw_bg.border_size: 0.0 draw_bg.border_radius: 0.0 draw_text.color: #x8c8c8c draw_text.text_style: theme.font_code{font_size: 9}}
                key := IconButton{width: 26 height: 21 icon_walk: Walk{width: 11 height: 11} draw_icon +: {svg: crate_resource("self://resources/icons/keyframe.svg") color: #xd8b574}}
            }
            scale_row := SolidView{width: Fill height: 25 flow: Right align: Align{y: 0.5} show_bg: true new_batch: true draw_bg.color: #x363636
                indicator := SolidView{width: 3 height: 8 draw_bg.color: #x8eb086}
                label := Label{text: "Scale" width: Fill padding: Inset{left: 8} draw_text.color: #xb8b8b8 draw_text.text_style: theme.font_regular{font_size: 10}}
                x := ButtonFlat{text: "1.000" width: 42 height: 21 draw_bg.color: #x242424 draw_bg.border_size: 0.0 draw_bg.border_radius: 0.0 draw_text.color: #xb8b8b8 draw_text.text_style: theme.font_code{font_size: 9}}
                y := ButtonFlat{text: "1.000" width: 42 height: 21 draw_bg.color: #x242424 draw_bg.border_size: 0.0 draw_bg.border_radius: 0.0 draw_text.color: #xb8b8b8 draw_text.text_style: theme.font_code{font_size: 9}}
                z := ButtonFlat{text: "1.000" width: 42 height: 21 draw_bg.color: #x242424 draw_bg.border_size: 0.0 draw_bg.border_radius: 0.0 draw_text.color: #x8c8c8c draw_text.text_style: theme.font_code{font_size: 9}}
                key := IconButton{width: 26 height: 21 icon_walk: Walk{width: 11 height: 11} draw_icon +: {svg: crate_resource("self://resources/icons/keyframe.svg") color: #x676767}}
            }
            rotation_row := SolidView{width: Fill height: 25 flow: Right align: Align{y: 0.5} show_bg: true new_batch: true draw_bg.color: #x363636
                indicator := SolidView{width: 3 height: 8 draw_bg.color: #xd8b574}
                label := Label{text: "Rotation" width: Fill padding: Inset{left: 8} draw_text.color: #xb8b8b8 draw_text.text_style: theme.font_regular{font_size: 10}}
                x := ButtonFlat{text: "0°" width: 42 height: 21 draw_bg.color: #x242424 draw_bg.border_size: 0.0 draw_bg.border_radius: 0.0 draw_text.color: #xb8b8b8 draw_text.text_style: theme.font_code{font_size: 9}}
                y := ButtonFlat{text: "0°" width: 42 height: 21 draw_bg.color: #x242424 draw_bg.border_size: 0.0 draw_bg.border_radius: 0.0 draw_text.color: #xb8b8b8 draw_text.text_style: theme.font_code{font_size: 9}}
                z := ButtonFlat{text: "0°" width: 42 height: 21 draw_bg.color: #x242424 draw_bg.border_size: 0.0 draw_bg.border_radius: 0.0 draw_text.color: #x8c8c8c draw_text.text_style: theme.font_code{font_size: 9}}
                key := IconButton{width: 26 height: 21 icon_walk: Walk{width: 11 height: 11} draw_icon +: {svg: crate_resource("self://resources/icons/keyframe.svg") color: #x676767}}
            }
            opacity_row := SolidView{width: Fill height: 25 flow: Right align: Align{y: 0.5} show_bg: true new_batch: true draw_bg.color: #x363636
                indicator := SolidView{width: 3 height: 8 draw_bg.color: #x8c8c8c}
                label := Label{text: "Opacity" width: Fill padding: Inset{left: 8} draw_text.color: #xb8b8b8 draw_text.text_style: theme.font_regular{font_size: 10}}
                value := ButtonFlat{text: "100%" width: 42 height: 21 draw_bg.color: #x242424 draw_bg.border_size: 0.0 draw_bg.border_radius: 0.0 draw_text.color: #xb8b8b8 draw_text.text_style: theme.font_code{font_size: 9}}
                key := IconButton{width: 26 height: 21 icon_walk: Walk{width: 11 height: 11} draw_icon +: {svg: crate_resource("self://resources/icons/keyframe.svg") color: #xaaa0d0}}
            }
            blend_row := SolidView{width: Fill height: 25 flow: Right align: Align{y: 0.5} show_bg: true new_batch: true draw_bg.color: #x363636
                indicator := SolidView{width: 3 height: 8 draw_bg.color: #xd8b574}
                label := Label{text: "Blend" width: Fill padding: Inset{left: 8} draw_text.color: #xb8b8b8 draw_text.text_style: theme.font_regular{font_size: 10}}
                value := ButtonFlat{text: "Normal" width: 84 height: 21 draw_bg.color: #x242424 draw_bg.border_size: 0.0 draw_bg.border_radius: 0.0 draw_text.color: #xb8b8b8 draw_text.text_style: theme.font_code{font_size: 9}}
                key := IconButton{width: 26 height: 21 icon_walk: Walk{width: 11 height: 11} draw_icon +: {svg: crate_resource("self://resources/icons/keyframe.svg") color: #x676767}}
            }
        }
    }
}

#[derive(Script, ScriptHook, WidgetRegister)]
pub struct InspectorSurface {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
}

impl WidgetNode for InspectorSurface {
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

impl Widget for InspectorSurface {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
