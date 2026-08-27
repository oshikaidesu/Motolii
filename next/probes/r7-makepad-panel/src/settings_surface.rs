//! Settings パネル枠。project / session / appearance の意味は波1。iced は引かない。
//! 見た目の正本: 利用者添付の Ableton Live Dark 実画面（2026-08-26 差し替え裁定）。
//! 色は画像から実測サンプル（記憶で埋めない）:
//!   面 #4f4f4f / 窪み欄 #141414 / 区切り 1px #1e1e1e / 頭帯 #646464 / 頭字 #292929 /
//!   節帯 #383838 / 明字 #cccccc / 薄字 #919191 / 橙点 #e89b3f。
//! 形: 枠線・角丸・影なし。欄は窪み（暗面）で示す。行 + ラベル + 窪み値欄の縦積み。
//! 行の値はすべて見た目用ダミー（出典なし）。Document / Session を読まない。
//! Chrome* 部品は参照しない（main.rs の読み込み順で本 mod が chrome より先）。
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
        draw_bg.color: mod.tokens.face.panel

        // 頭帯 — Live device title bar 調（#646464 帯 + 暗字 + 橙の角形活性点）
        settings_head := SolidView{width: Fill height: 26 flow: Right spacing: 6 align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: #x646464
            power_dot := SolidView{width: 7 height: 7 show_bg: true draw_bg.color: #xe89b3f}
            title := Label{text: "Settings" width: Fill draw_text.color: #x292929 draw_text.text_style: theme.font_bold{font_size: 11 line_spacing: 1.0 top_drop: 0.0}}
        }
        head_rule := SolidView{width: Fill height: 1 show_bg: true draw_bg.color: #x1e1e1e}

        // 以下の欄値はダミー（出典なし）。意味書きは波1
        project_band := SolidView{width: Fill height: 18 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: #x383838
            project_head := Label{text: "PROJECT" width: Fill draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_bold{font_size: 8 line_spacing: 1.0 top_drop: 0.0}}
        }
        frame_rate_row := SolidView{width: Fill height: 24 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := Label{text: "Frame Rate" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: 10 line_spacing: 1.0 top_drop: 0.0}}
            well := SolidView{width: 104 height: 18 flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: 6 right: 6} show_bg: true new_batch: true draw_bg.color: #x141414
                value := Label{text: "60 fps" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: 9 line_spacing: 1.0 top_drop: 0.0}}
            }
        }
        frame_rate_rule := SolidView{width: Fill height: 1 show_bg: true draw_bg.color: #x1e1e1e}
        resolution_row := SolidView{width: Fill height: 24 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := Label{text: "Resolution" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: 10 line_spacing: 1.0 top_drop: 0.0}}
            well := SolidView{width: 104 height: 18 flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: 6 right: 6} show_bg: true new_batch: true draw_bg.color: #x141414
                value := Label{text: "1920 × 1080" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: 9 line_spacing: 1.0 top_drop: 0.0}}
            }
        }
        resolution_rule := SolidView{width: Fill height: 1 show_bg: true draw_bg.color: #x1e1e1e}
        duration_row := SolidView{width: Fill height: 24 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := Label{text: "Duration" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: 10 line_spacing: 1.0 top_drop: 0.0}}
            well := SolidView{width: 104 height: 18 flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: 6 right: 6} show_bg: true new_batch: true draw_bg.color: #x141414
                value := Label{text: "300 F" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: 9 line_spacing: 1.0 top_drop: 0.0}}
            }
        }
        duration_rule := SolidView{width: Fill height: 1 show_bg: true draw_bg.color: #x1e1e1e}

        session_band := SolidView{width: Fill height: 18 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: #x383838
            session_head := Label{text: "SESSION" width: Fill draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_bold{font_size: 8 line_spacing: 1.0 top_drop: 0.0}}
        }
        autosave_row := SolidView{width: Fill height: 24 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := Label{text: "Autosave" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: 10 line_spacing: 1.0 top_drop: 0.0}}
            well := SolidView{width: 104 height: 18 flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: 6 right: 6} show_bg: true new_batch: true draw_bg.color: #x141414
                value := Label{text: "On" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: 9 line_spacing: 1.0 top_drop: 0.0}}
            }
        }
        autosave_rule := SolidView{width: Fill height: 1 show_bg: true draw_bg.color: #x1e1e1e}
        undo_row := SolidView{width: Fill height: 24 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := Label{text: "Undo Depth" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: 10 line_spacing: 1.0 top_drop: 0.0}}
            well := SolidView{width: 104 height: 18 flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: 6 right: 6} show_bg: true new_batch: true draw_bg.color: #x141414
                value := Label{text: "200" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: 9 line_spacing: 1.0 top_drop: 0.0}}
            }
        }
        undo_rule := SolidView{width: Fill height: 1 show_bg: true draw_bg.color: #x1e1e1e}

        appearance_band := SolidView{width: Fill height: 18 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: #x383838
            appearance_head := Label{text: "APPEARANCE" width: Fill draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_bold{font_size: 8 line_spacing: 1.0 top_drop: 0.0}}
        }
        theme_row := SolidView{width: Fill height: 24 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := Label{text: "Theme" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: 10 line_spacing: 1.0 top_drop: 0.0}}
            well := SolidView{width: 104 height: 18 flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: 6 right: 6} show_bg: true new_batch: true draw_bg.color: #x141414
                value := Label{text: "Live Dark" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: 9 line_spacing: 1.0 top_drop: 0.0}}
            }
        }
        theme_rule := SolidView{width: Fill height: 1 show_bg: true draw_bg.color: #x1e1e1e}
        scale_row := SolidView{width: Fill height: 24 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := Label{text: "UI Scale" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: 10 line_spacing: 1.0 top_drop: 0.0}}
            well := SolidView{width: 104 height: 18 flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: 6 right: 6} show_bg: true new_batch: true draw_bg.color: #x141414
                value := Label{text: "100 %" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: 9 line_spacing: 1.0 top_drop: 0.0}}
            }
        }
        scale_rule := SolidView{width: Fill height: 1 show_bg: true draw_bg.color: #x1e1e1e}

        // 残余は素の面（#4f4f4f）。枠・影・角丸は置かない
        settings_fill := View{width: Fill height: Fill}
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
