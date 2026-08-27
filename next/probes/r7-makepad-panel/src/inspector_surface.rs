//! Inspector パネル。**構造の正本は `next/reference/mocks/inspector-semantics.html`**
//! (第4号、propertyRow 25px = 比率の分母、inspector-ratio-ledger 実測基準)。
//! 第一波(M01)の実線分だけを描く: selection summary / 列見出し / TRANSFORM /
//! APPEARANCE / FX STACK の行 / footer ヒント。mode tabs・extension tabs・notes は
//! Q0 スコープ外(I-ratio 台帳)。左 3px = 値型の色。◆=keyed。
//! 皮は Ableton の形文法(裁定267): 平坦・角丸ゼロ・1px 線・琥珀 on。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    let InspectorRule = SolidView{width: Fill height: mod.tokens.rule.size show_bg: true new_batch: true draw_bg.color: mod.tokens.rule.seam}

    // 節見出し — 「TRANSFORM 3 · 2 keyed」。左が名、右が計数(薄)
    let SectionCap = SolidView{
        width: Fill
        height: 18
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4}
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.face.area
        fold := InkLabel{text: "▼" width: Fit padding: Inset{right: mod.tokens.space.s2} draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
        name := InkLabel{width: Fill draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_bold{font_size: mod.tokens.text.xs}}
        count := InkLabel{width: Fit draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
    }

    // property 行 — 分母 25px(inspector-ratio-ledger 実測基準)。
    // 左 3px = 値型の色。◆=keyed / ◇=非 keyed。値3列は等幅
    let PropertyRow = SolidView{
        width: Fill
        height: 25
        flow: Right
        align: Align{y: 0.5}
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.face.panel
        type_bar := SolidView{width: 3 height: Fill show_bg: true new_batch: true draw_bg.color: mod.tokens.ink.faint}
        name := InkLabel{width: 64 padding: Inset{left: mod.tokens.space.s3} draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}}
        vx := InkLabel{width: 52 draw_text.color: mod.tokens.ink.strong draw_text.text_style: theme.font_code{font_size: mod.tokens.text.sm}}
        vy := InkLabel{width: 52 draw_text.color: mod.tokens.ink.strong draw_text.text_style: theme.font_code{font_size: mod.tokens.text.sm}}
        vz := InkLabel{width: 52 draw_text.color: mod.tokens.ink.strong draw_text.text_style: theme.font_code{font_size: mod.tokens.text.sm}}
        keyed := InkLabel{width: Fill align: Align{x: 1.0} padding: Inset{right: mod.tokens.space.s4} draw_text.color: mod.tokens.accent.on draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}}
    }

    mod.widgets.InspectorSurfaceBase = #(InspectorSurface::register_widget(vm))
    mod.widgets.InspectorSurface = set_type_default() do mod.widgets.InspectorSurfaceBase{
        width: Fill
        height: Fill
        flow: Down
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.face.panel

        // selection summary — 46/25 = 1.84(inspector-ratio-ledger の実測比。モック表示の 40 は概形)
        summary := SolidView{width: Fill height: 46 flow: Down padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4 top: mod.tokens.space.s2} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            sel_name := InkLabel{text: "Rectangle" width: Fill height: Fit draw_text.color: mod.tokens.ink.strong draw_text.text_style: theme.font_bold{font_size: mod.tokens.text.lg}}
            sel_kind := InkLabel{text: "Shape layer · 3D transform · 2 keys" width: Fill height: Fit padding: Inset{top: mod.tokens.space.s1} draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
        }
        summary_rule := InspectorRule{}

        // 列見出し 21/25
        col_head := SolidView{width: Fill height: 21 flow: Right align: Align{y: 0.5} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            pad := View{width: 3 height: Fill}
            c_prop := InkLabel{text: "Property" width: 64 padding: Inset{left: mod.tokens.space.s3} draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
            c_x := InkLabel{text: "X" width: 52 draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
            c_y := InkLabel{text: "Y" width: 52 draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
            c_z := InkLabel{text: "Z" width: 52 draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
        }

        transform_cap := SectionCap{name.text: "TRANSFORM" count.text: "3 · 2 keyed"}
        row_position := PropertyRow{type_bar.draw_bg.color: mod.tokens.accent.alt name.text: "Position" vx.text: "0.125" vy.text: "-0.075" vz.text: "0.000" keyed.text: "◆"}
        row_rotation := PropertyRow{type_bar.draw_bg.color: mod.tokens.accent.alt name.text: "Rotation" vx.text: "0.0" vy.text: "0.0" vz.text: "24.0°" keyed.text: "◆"}
        row_scale := PropertyRow{type_bar.draw_bg.color: mod.tokens.accent.alt name.text: "Scale" vx.text: "1.000" vy.text: "1.000" vz.text: "1.000" keyed.text: "◇" keyed.draw_text.color: mod.tokens.ink.faint}

        appearance_cap := SectionCap{name.text: "APPEARANCE" count.text: "2 · 1 keyed"}
        row_fill := PropertyRow{type_bar.draw_bg.color: #xd8c97f name.text: "Fill" vx.text: "#D8C97F" vy.text: "" vz.text: "" keyed.text: "◇" keyed.draw_text.color: mod.tokens.ink.faint}
        row_opacity := PropertyRow{type_bar.draw_bg.color: mod.tokens.accent.on name.text: "Opacity" vx.text: "100%" vy.text: "" vz.text: "" keyed.text: "◆"}

        fx_cap := SectionCap{name.text: "FX STACK" count.text: "1 effect"}
        // effect 行 — 選択は行の地、証は色付き左端(裁定: ○ボタン/FXバッジは置かない)
        fx_row := SolidView{width: Fill height: 25 flow: Right align: Align{y: 0.5} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            fx_bar := SolidView{width: 3 height: Fill show_bg: true new_batch: true draw_bg.color: mod.tokens.accent.record}
            fx_name := InkLabel{text: "TURBULENT DISPLACE" width: Fill padding: Inset{left: mod.tokens.space.s3} draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}}
            fx_params := InkLabel{text: "8 params" width: Fit padding: Inset{right: mod.tokens.space.s3} draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
            fx_on := SolidView{width: 26 height: mod.tokens.size.chip margin: Inset{right: mod.tokens.space.s3} align: Align{x: 0.5 y: 0.5} show_bg: true new_batch: true draw_bg.color: mod.tokens.accent.on
                on_label := InkLabel{text: "ON" width: Fit height: Fit draw_text.color: mod.tokens.ink.on_fill draw_text.text_style: theme.font_bold{font_size: mod.tokens.text.xs}}
            }
        }
        row_amount := PropertyRow{type_bar.draw_bg.color: mod.tokens.accent.alt name.text: "Amount" vx.text: "42.0" vy.text: "" vz.text: "" keyed.text: "◇" keyed.draw_text.color: mod.tokens.ink.faint}
        advanced_cap := SectionCap{fold.text: "▶" name.text: "ADVANCED" count.text: "4 parameters"}

        body_fill := View{width: Fill height: Fill}

        footer_rule := InspectorRule{}
        hint_row := SolidView{width: Fill height: 18 flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.bar
            hint := InkLabel{text: "drag to scrub · click to type · Esc to cancel" width: Fill draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
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
