//! Export パネル枠。範囲・形式の意味は波1。iced は引かない。
//! 見た目の正本は `mod.tokens`(裁定267: Ableton の identity は palette でなく形の文法。
//! 面/字/線/選択/accent は Live 12 `.ask` 実機抽出由来)。ここに生の hex を書かない —
//! 書いた瞬間、皮の差し替えがこの1枚だけ効かなくなる。
//! 進捗の塗りは `.ask` の `TransportProgress`(= 琥珀)。以前の青緑は出典なしの即興だった。
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

        // 頭帯 — Live の device title bar と同型(明帯 + 暗字 = 極性反転、橙の活性点)
        export_head := SolidView{width: Fill height: mod.tokens.size.toolbar flow: Right spacing: mod.tokens.space.s3 align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.head
            power_dot := SolidView{width: 7 height: 7 show_bg: true draw_bg.color: mod.tokens.accent.on}
            title := InkLabel{text: "Export" width: Fill draw_text.color: mod.tokens.ink.on_fill draw_text.text_style: theme.font_bold{font_size: mod.tokens.text.xl}}
        }
        head_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}

        // 以下の欄値はダミー（出典なし）。意味書きは波1
        range_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := InkLabel{text: "Range" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            well := SolidView{width: 104 height: mod.tokens.size.row_tight flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.display
                value := InkLabel{text: "0 – 300 F" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: mod.tokens.text.md}}
            }
        }
        range_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}
        format_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := InkLabel{text: "Format" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            well := SolidView{width: 104 height: mod.tokens.size.row_tight flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.display
                value := InkLabel{text: "MP4 · H.264" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: mod.tokens.text.md}}
            }
        }
        format_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}
        audio_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := InkLabel{text: "Audio" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            well := SolidView{width: 104 height: mod.tokens.size.row_tight flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.display
                value := InkLabel{text: "AAC · 48 kHz" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: mod.tokens.text.md}}
            }
        }
        audio_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}
        destination_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            label := InkLabel{text: "Destination" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            well := SolidView{width: 104 height: mod.tokens.size.row_tight flow: Right align: Align{x: 1.0 y: 0.5} padding: Inset{left: mod.tokens.space.s3 right: mod.tokens.space.s3} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.display
                value := InkLabel{text: "motolii.mp4" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: mod.tokens.text.md}}
            }
        }
        destination_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}

        // 進捗読取 — `0 / 300 (0%)`（ChromeProgressReadout の並びを手本、値はダミー・出典なし）
        progress_row := SolidView{width: Fill height: mod.tokens.size.form_row flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} spacing: 2 show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            progress_label := InkLabel{text: "Progress" width: Fill draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
            done := InkLabel{text: "0" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: mod.tokens.text.md}}
            sep := InkLabel{text: "/" width: Fit draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_code{font_size: mod.tokens.text.md}}
            total := InkLabel{text: "300" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_code{font_size: mod.tokens.text.md}}
            pct := InkLabel{text: "(0%)" width: Fit draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_code{font_size: mod.tokens.text.md}}
        }
        // 細い溝 + 明るい塗り。塗り幅 120 はダミー（出典なし）
        progress_track := SolidView{width: Fill height: 3 flow: Right margin: Inset{left: 8 right: 8 bottom: 8} show_bg: true draw_bg.color: mod.tokens.face.display
            progress_fill := SolidView{width: 120 height: Fill show_bg: true draw_bg.color: mod.tokens.accent.on}
        }

        // 空域 — 面のまま中央に薄字だけ（Drop Audio Effects Here 調）
        export_empty := SolidView{width: Fill height: Fill flow: Down align: Align{x: 0.5 y: 0.5} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            empty_hint := InkLabel{text: "No Export Running" width: Fit draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.md}}
        }

        // 状態帯 — ChromeStatus の幾何（高 28・一行）を手本。文言はダミー（出典なし）
        status_rule := SolidView{width: Fill height: mod.tokens.rule.size show_bg: true draw_bg.color: mod.tokens.rule.seam}
        export_status := SolidView{width: Fill height: 28 flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} spacing: 8 show_bg: true new_batch: true draw_bg.color: mod.tokens.face.bar
            status_label := InkLabel{text: "Ready" width: Fit draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.lg}}
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
