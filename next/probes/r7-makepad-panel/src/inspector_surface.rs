//! Inspector パネル枠。property / key の意味書きは波1。iced は引かない。
//! 見た目: Ableton Live 12 Dark 実画面の Device 域（Channel EQ / Compressor）から実測サンプル。
//! 面 mod.tokens.face.panel / 窪み #x141414 / 帯 #x646464 / 選択帯 #x8dc9d9 / 墨 #x171717 /
//! 値シアン #x73acb3 / 線シアン #x8dc9d9 / 橙 mod.tokens.accent.on / ボタン mod.tokens.face.raised+#xe3e3e5 /
//! 指針 #xd6d0d4 / 境界 mod.tokens.rule.seam。
//! 形: 窪み矩形は枠線なし・角丸ゼロ。区切りは 1px 線。ヘッダは低い帯。
//! メーターは暗面に高彩度線1本。ノブは平坦な円 + 細い指針線（立体感なし）。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // 1px 区切り線 — Device 境界の実測 mod.tokens.rule.seam
    let InspectorRule = SolidView{width: Fill height: 1 show_bg: true new_batch: true draw_bg.color: mod.tokens.rule.seam}

    // 窪み数値欄 — 枠線なし・角丸ゼロ・シアン明字（Thresh 欄の実測）
    let InspectorField = ButtonFlat{
        width: 42
        height: 16
        padding: 0
        draw_bg.color: #x141414
        draw_bg.color_hover: mod.tokens.rule.seam
        draw_bg.color_down: #x141414
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #x73acb3
        draw_text.text_style: theme.font_code{font_size: 9}
    }

    // 裸のキー打鍵 — 面の上のアイコンだけ。地は塗らない
    let InspectorKey = ButtonFlatterIcon{
        margin: 0
        width: 26
        height: 16
        icon_walk: Walk{width: 11 height: 11}
        padding: 0
        draw_icon +: {color: #x2e2e2e}
    }

    // 平坦ノブ — 暗円 + 細い指針線。縁も影も置かない
    let InspectorKnob = SolidView{
        width: 24
        height: 24
        flow: Down
        align: Align{x: 0.5 y: 0.0}
        padding: Inset{top: 2}
        show_bg: true
        draw_bg.color: #x141414
        draw_bg.border_radius: 12.0
        pointer := SolidView{width: 2 height: 8 show_bg: true draw_bg.color: #xd6d0d4}
    }

    mod.widgets.InspectorSurfaceBase = #(InspectorSurface::register_widget(vm))
    mod.widgets.InspectorSurface = set_type_default() do mod.widgets.InspectorSurfaceBase{
        width: Fill
        height: Fill
        flow: Down
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.face.panel

        // 頭 = 選択中 Device のタイトル帯（シアン地・墨字・橙 LED）
        inspector_head := SolidView{width: Fill height: 20 flow: Right spacing: 6 align: Align{y: 0.5} padding: Inset{left: 6 right: 8} show_bg: true new_batch: true draw_bg.color: #x8dc9d9
            accent_dot := SolidView{width: 7 height: 7 show_bg: true draw_bg.color: mod.tokens.accent.on draw_bg.border_radius: 3.5}
            title := Label{text: "Inspector" width: Fill draw_text.color: #x171717 draw_text.text_style: theme.font_bold{font_size: 10}}
            context := Label{text: "Layer 7 · Solid" width: Fit draw_text.color: #x171717 draw_text.text_style: theme.font_code{font_size: 8}}
        }
        head_rule := InspectorRule{}

        // モード切替 = Peak/RMS 型セグメント。オンは橙+墨字、オフは mod.tokens.face.raised+明字。隙間 1px が線
        modes := SolidView{width: Fill height: 22 flow: Right spacing: 1 show_bg: true new_batch: true draw_bg.color: mod.tokens.rule.seam
            effect := ButtonFlat{text: "Effect" width: Fill height: 22 icon_walk: Walk{width: 11 height: 11} draw_icon +: {svg: crate_resource("self://resources/icons/effects.svg") color: #x171717} draw_bg.color: mod.tokens.accent.on draw_bg.color_hover: #xd8a160 draw_bg.color_down: #xbf8d4e draw_bg.border_size: 0.0 draw_bg.border_radius: 0.0 draw_text.color: #x171717 draw_text.text_style: theme.font_bold{font_size: 8}}
            layer_mode := ButtonFlat{text: "Custom" width: Fill height: 22 icon_walk: Walk{width: 11 height: 11} draw_icon +: {svg: crate_resource("self://resources/icons/shape.svg") color: #xe3e3e5} draw_bg.color: mod.tokens.face.raised draw_bg.color_hover: #x424242 draw_bg.color_down: #x141414 draw_bg.border_size: 0.0 draw_bg.border_radius: 0.0 draw_text.color: #xe3e3e5 draw_text.text_style: theme.font_regular{font_size: 8}}
        }
        modes_rule := InspectorRule{}

        // 選択 = 面の上に窪み表示欄。名はシアン、種別は琥珀（kHz 表示の実測）
        selection := SolidView{width: Fill height: 40 flow: Right spacing: 6 align: Align{y: 0.5} padding: Inset{left: 6 right: 6} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            glyph := Icon{width: 20 height: 20 icon_walk: Walk{width: 13 height: 13} draw_icon +: {svg: crate_resource("self://resources/icons/shape.svg") color: #x171717} draw_bg +: {color: mod.tokens.accent.on}}
            selection_copy := SolidView{width: Fill height: 28 flow: Down align: Align{y: 0.5} padding: Inset{left: 6} show_bg: true new_batch: true draw_bg.color: #x141414
                selection_name := Label{text: "Chorus Lyrics" width: Fill height: 13 draw_text.color: #x73acb3 draw_text.text_style: theme.font_bold{font_size: 9}}
                selection_type := Label{text: "Solid · selected · off frame" width: Fill height: 11 draw_text.color: #xbda37e draw_text.text_style: theme.font_code{font_size: 7.5}}
            }
            mute := ButtonFlatIcon{width: 20 height: 20 padding: 0 icon_walk: Walk{width: 11 height: 11} draw_bg.color: mod.tokens.face.raised draw_bg.color_hover: #x424242 draw_bg.color_down: #x141414 draw_bg.border_size: 0.0 draw_bg.border_radius: 0.0 draw_icon +: {svg: crate_resource("self://resources/icons/mute.svg") color: #xe3e3e5}}
            solo := ButtonFlatIcon{width: 20 height: 20 padding: 0 icon_walk: Walk{width: 11 height: 11} draw_bg.color: mod.tokens.accent.on draw_bg.color_hover: #xd8a160 draw_bg.color_down: #xbf8d4e draw_bg.border_size: 0.0 draw_bg.border_radius: 0.0 draw_icon +: {svg: crate_resource("self://resources/icons/solo.svg") color: #x171717}}
        }
        selection_rule := InspectorRule{}

        // メーター = 暗面に高彩度シアン線1本（EQ カーブ / GR 線の言語）
        meter := SolidView{width: Fill height: 26 flow: Down padding: Inset{top: 12 bottom: 2} show_bg: true new_batch: true draw_bg.color: #x141414
            meter_line := SolidView{width: Fill height: 1 show_bg: true draw_bg.color: #x8dc9d9}
            meter_caption := View{width: Fill height: Fit align: Align{x: 1.0} padding: Inset{right: 6}
                meter_value := Label{text: "0.00 dB" width: Fit draw_text.color: #x679299 draw_text.text_style: theme.font_code{font_size: 7.5}}
            }
        }
        meter_rule := InspectorRule{}

        // 見出し = 低い帯（未選択タイトル帯の実測 #x646464・墨字）
        transform_section := SolidView{width: Fill height: 17 flow: Right align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: #x646464
            section := Label{text: "TRANSFORM" width: Fill draw_text.color: #x171717 draw_text.text_style: theme.font_bold{font_size: 7.5}}
            section_count := Label{text: "1 PROP · 2 KEYS" width: Fit draw_text.color: #x171717 draw_text.text_style: theme.font_code{font_size: 7.5}}
        }
        columns := SolidView{width: Fill height: 15 flow: Right spacing: 3 align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            c0 := Label{text: "PROPERTY" width: Fill draw_text.color: #x2e2e2e draw_text.text_style: theme.font_bold{font_size: 7}}
            cx := Label{text: "X" width: 42 draw_text.color: #x2e2e2e draw_text.text_style: theme.font_code{font_size: 7.5}}
            cy := Label{text: "Y" width: 42 draw_text.color: #x2e2e2e draw_text.text_style: theme.font_code{font_size: 7.5}}
            cz := Label{text: "Z" width: 42 draw_text.color: #x2e2e2e draw_text.text_style: theme.font_code{font_size: 7.5}}
            ck := Icon{width: 26 height: 14 icon_walk: Walk{width: 10 height: 10} draw_icon +: {svg: crate_resource("self://resources/icons/keyframe.svg") color: #x2e2e2e}}
        }
        columns_rule := InspectorRule{}

        // 行 = 面に墨ラベル + 窪み欄の並び。色棒・枠線・角丸は置かない
        property_rows := SolidView{width: Fill height: Fill flow: Down show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            position_row := SolidView{width: Fill height: 22 flow: Right spacing: 3 align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
                label := Label{text: "Position" width: Fill draw_text.color: #x171717 draw_text.text_style: theme.font_regular{font_size: 9.5}}
                x := InspectorField{text: "960"}
                y := InspectorField{text: "540"}
                z := InspectorField{text: "0" draw_text.color: #x679299}
                key := InspectorKey{draw_icon +: {color: mod.tokens.accent.on}}
            }
            scale_row := SolidView{width: Fill height: 22 flow: Right spacing: 3 align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
                label := Label{text: "Scale" width: Fill draw_text.color: #x171717 draw_text.text_style: theme.font_regular{font_size: 9.5}}
                x := InspectorField{text: "1.000"}
                y := InspectorField{text: "1.000"}
                z := InspectorField{text: "1.000" draw_text.color: #x679299}
                key := InspectorKey{}
            }
            rotation_row := SolidView{width: Fill height: 22 flow: Right spacing: 3 align: Align{y: 0.5} padding: Inset{left: 8 right: 8} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
                label := Label{text: "Rotation" width: Fill draw_text.color: #x171717 draw_text.text_style: theme.font_regular{font_size: 9.5}}
                x := InspectorField{text: "0°"}
                y := InspectorField{text: "0°"}
                z := InspectorField{text: "0°" draw_text.color: #x679299}
                key := InspectorKey{}
            }
        }
        knob_rule := InspectorRule{}

        // 底 = Low/Mid/High/Output 型のノブ帯。縦 1px 線で区画
        knob_strip := SolidView{width: Fill height: 56 flow: Right show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
            opacity_group := View{width: Fill height: Fill flow: Down spacing: 2 align: Align{x: 0.5 y: 0.0} padding: Inset{top: 4}
                opacity_label := Label{text: "Opacity" width: Fit draw_text.color: #x171717 draw_text.text_style: theme.font_bold{font_size: 8}}
                opacity_knob := InspectorKnob{}
                opacity_value := Label{text: "100 %" width: Fit draw_text.color: #x171717 draw_text.text_style: theme.font_code{font_size: 8}}
            }
            knob_divider := SolidView{width: 1 height: Fill show_bg: true draw_bg.color: mod.tokens.rule.seam}
            blend_group := View{width: Fill height: Fill flow: Down spacing: 4 align: Align{x: 0.5 y: 0.0} padding: Inset{top: 4}
                blend_label := Label{text: "Blend" width: Fit draw_text.color: #x171717 draw_text.text_style: theme.font_bold{font_size: 8}}
                blend_value := InspectorField{text: "Normal" width: 84}
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
