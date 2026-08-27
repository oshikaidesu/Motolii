//! Export パネル枠。範囲・形式の意味は波1。iced は引かない。
//! 見た目の正本: 利用者添付の Ableton Live Dark 実画面（2026-08-26 差し替え裁定）。
//! 色は画像から実測サンプル（記憶で埋めない）:
//!   面 #4f4f4f / 窪み欄 #141414 / 区切り 1px #1e1e1e / 頭帯 #646464 / 頭字 #292929 /
//!   明字 #cccccc / 薄字 #919191 / 橙点 #e89b3f / 進捗塗り(青緑) #8fc8db / 状態帯 #2b2b2b。
//! 形: 枠線・角丸・影なし。欄は窪み（暗面）で示す。進捗は細い溝 + 明るい塗り。
//! 空域は「Drop Audio Effects Here」調（面のまま中央に薄字だけ）。
//! 進捗読取・状態帯の幾何は chrome/parts の ChromeProgressReadout / ChromeStatus を
//! 手本に写した（main.rs の読み込み順で本 mod が chrome より先のため直接参照しない）。
//! 行の値・進捗値はすべて見た目用ダミー（出典なし）。Document / Session を読まない。
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
        draw_bg.color: mod.tokens.face.panel

        // 頭帯 — Live device title bar 調（#646464 帯 + 暗字 + 橙の角形活性点）
        export_head := SolidView{width: Fill height: 26 flow: Right spacing: 6 align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: #x646464
            power_dot := SolidView{width: 7 height: 7 show_bg: true draw_bg.color: #xe89b3f}
            title := Label{text: "Export" width: Fill draw_text.color: #x292929 draw_text.text_style: theme.font_bold{font_size: 11 line_spacing: 1.0 top_drop: 0.0}}
        }
        head_rule := SolidView{width: Fill height: 1 show_bg: true draw_bg.color: #x1e1e1e}

        // 以下の欄値はダミー（出典なし）。意味書きは波1
        range_row := SolidView{width: Fill height: 24 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := Label{text: "Range" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: 10 line_spacing: 1.0 top_drop: 0.0}}
            well := SolidView{width: 104 height: 18 flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: 6 right: 6} show_bg: true new_batch: true draw_bg.color: #x141414
                value := Label{text: "0 – 300 F" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: 9 line_spacing: 1.0 top_drop: 0.0}}
            }
        }
        range_rule := SolidView{width: Fill height: 1 show_bg: true draw_bg.color: #x1e1e1e}
        format_row := SolidView{width: Fill height: 24 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := Label{text: "Format" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: 10 line_spacing: 1.0 top_drop: 0.0}}
            well := SolidView{width: 104 height: 18 flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: 6 right: 6} show_bg: true new_batch: true draw_bg.color: #x141414
                value := Label{text: "MP4 · H.264" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: 9 line_spacing: 1.0 top_drop: 0.0}}
            }
        }
        format_rule := SolidView{width: Fill height: 1 show_bg: true draw_bg.color: #x1e1e1e}
        audio_row := SolidView{width: Fill height: 24 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := Label{text: "Audio" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: 10 line_spacing: 1.0 top_drop: 0.0}}
            well := SolidView{width: 104 height: 18 flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: 6 right: 6} show_bg: true new_batch: true draw_bg.color: #x141414
                value := Label{text: "AAC · 48 kHz" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: 9 line_spacing: 1.0 top_drop: 0.0}}
            }
        }
        audio_rule := SolidView{width: Fill height: 1 show_bg: true draw_bg.color: #x1e1e1e}
        destination_row := SolidView{width: Fill height: 24 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := Label{text: "Destination" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: 10 line_spacing: 1.0 top_drop: 0.0}}
            well := SolidView{width: 104 height: 18 flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: 6 right: 6} show_bg: true new_batch: true draw_bg.color: #x141414
                value := Label{text: "motolii.mp4" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: 9 line_spacing: 1.0 top_drop: 0.0}}
            }
        }
        destination_rule := SolidView{width: Fill height: 1 show_bg: true draw_bg.color: #x1e1e1e}

        // 進捗読取 — `0 / 300 (0%)`（ChromeProgressReadout の並びを手本、値はダミー・出典なし）
        progress_row := SolidView{width: Fill height: 24 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} spacing: 2 show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            progress_label := Label{text: "Progress" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: 10 line_spacing: 1.0 top_drop: 0.0}}
            done := Label{text: "0" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: 9 line_spacing: 1.0 top_drop: 0.0}}
            sep := Label{text: "/" width: Fit draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_code{font_size: 9 line_spacing: 1.0 top_drop: 0.0}}
            total := Label{text: "300" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: 9 line_spacing: 1.0 top_drop: 0.0}}
            pct := Label{text: "(0%)" width: Fit draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_code{font_size: 9 line_spacing: 1.0 top_drop: 0.0}}
        }
        // 細い溝 + 明るい塗り。塗り幅 120 はダミー（出典なし）
        progress_track := SolidView{width: Fill height: 3 flow: Right margin: Inset{left: 8 right: 8 bottom: 8} show_bg: true draw_bg.color: #x141414
            progress_fill := SolidView{width: 120 height: Fill show_bg: true draw_bg.color: #x8fc8db}
        }

        // 空域 — 面のまま中央に薄字だけ（Drop Audio Effects Here 調）
        export_empty := SolidView{width: Fill height: Fill flow: Down align: Align{x: 0.5 y: 0.5} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            empty_hint := Label{text: "No Export Running" width: Fit draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_regular{font_size: 9 line_spacing: 1.0 top_drop: 0.0}}
        }

        // 状態帯 — ChromeStatus の幾何（高 28・一行）を手本。文言はダミー（出典なし）
        status_rule := SolidView{width: Fill height: 1 show_bg: true draw_bg.color: #x1e1e1e}
        export_status := SolidView{width: Fill height: 28 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} spacing: 8 show_bg: true new_batch: true draw_bg.color: #x2b2b2b
            status_label := Label{text: "Ready" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: 10 line_spacing: 1.0 top_drop: 0.0}}
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
