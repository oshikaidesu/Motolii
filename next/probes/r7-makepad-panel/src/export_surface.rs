//! Export パネル枠。範囲・形式の意味は波1。iced は引かない。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.ExportSurfaceBase = #(ExportSurface::register_widget(vm))
    mod.widgets.ExportSurface = set_type_default() do mod.widgets.ExportSurfaceBase{
        width: Fill
        height: Fill
        flow: Down
        show_bg: true
        new_batch: true
        draw_bg.color: #x363636

        export_head := SolidView{width: Fill height: 26 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: #x2f2f2f
            title := Label{text: "Export" width: Fill draw_text.color: #xcfcfcf draw_text.text_style: theme.font_bold{font_size: 11}}
        }
        export_body := SolidView{width: Fill height: Fill flow: Down padding: 12 spacing: 6 show_bg: true new_batch: true draw_bg.color: #x363636
            area := Label{text: "range · format · audio · progress" width: Fill draw_text.color: #x757575 draw_text.text_style: theme.font_code{font_size: 9}}
        }
    }
}

#[derive(Script, ScriptHook, WidgetRegister)]
pub struct ExportSurface {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
}

impl WidgetNode for ExportSurface {
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

impl Widget for ExportSurface {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
