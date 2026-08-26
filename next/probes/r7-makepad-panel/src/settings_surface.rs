//! Settings パネル枠。project / session / appearance の意味は波1。iced は引かない。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.SettingsSurfaceBase = #(SettingsSurface::register_widget(vm))
    mod.widgets.SettingsSurface = set_type_default() do mod.widgets.SettingsSurfaceBase{
        width: Fill
        height: Fill
        flow: Down
        show_bg: true
        new_batch: true
        draw_bg.color: #x363636

        settings_head := SolidView{width: Fill height: 26 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: #x2f2f2f
            title := Label{text: "Settings" width: Fill draw_text.color: #xcfcfcf draw_text.text_style: theme.font_bold{font_size: 11}}
        }
        settings_body := SolidView{width: Fill height: Fill flow: Down padding: 12 spacing: 6 show_bg: true new_batch: true draw_bg.color: #x363636
            area := Label{text: "project · session · appearance · input · chrome" width: Fill draw_text.color: #x757575 draw_text.text_style: theme.font_code{font_size: 9}}
        }
    }
}

#[derive(Script, ScriptHook, WidgetRegister)]
pub struct SettingsSurface {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
}

impl WidgetNode for SettingsSurface {
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

impl Widget for SettingsSurface {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
