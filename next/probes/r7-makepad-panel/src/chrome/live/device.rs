//! ホスト向け公開型: LiveDeviceActivator / LiveFold / LiveDeviceTitleBar /
//! LiveDeviceKnob / LiveDeviceParam / LiveSliderVertical / LiveSegmentButton /
//! LiveSegmentButtonOn / LiveVisualizerWave / LiveVisualizerEq / LiveDeviceChain
//!
//! 出典: 利用者添付の Live 12 実画面（下 Device）
//!   折り三角 / タイトル / ノブ / Dry/Wet 縦スライダ / 波形・EQ / セグメント
//! 名称: https://www.ableton.com/en/live-manual/12/working-with-instruments-and-effects/
//! 色（画像）: Device #242424 / インク #a5a5a5 / ノブ地 #131313
//!   波形黄 #e3b43e / EQ シアン #acc5ca / Dry/Wet #4dd37c / 折り #a5a5a5
//! Document を持たない。寸法 px は出典なし。別名は代入。`ScrollYView` は書かない。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.LiveDeviceActivator = ButtonFlat{
        width: 12
        height: 12
        padding: 0
        text: ""        draw_bg.color: #xe9be3c
        draw_bg.color_hover: #xe9be3c
        draw_bg.color_down: #x131313
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 2.0
    }

    // 折り三角
    mod.widgets.LiveFold = ButtonFlatter{
        width: 14
        height: 14
        padding: 0
        align: Center        text: "v"
        draw_bg.color: #x00000000
        draw_bg.color_hover: #x131313
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #xa5a5a5
        draw_text.text_style: theme.font_regular{font_size: 9}
    }

    mod.widgets.LiveDeviceTitleBar = SolidView{
        width: Fill
        height: 18
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 4 right: 6}
        spacing: 4
        show_bg: true
        new_batch: true
        draw_bg.color: #x242424
        fold := LiveFold{}
        activator := LiveDeviceActivator{}
        title := InkLabel{
            width: Fill
            height: Fit
            text: "Drifting Ambient Pad"
            draw_text.color: #xa5a5a5
            draw_text.text_style: theme.font_regular{font_size: 10}
        }
    }

    mod.widgets.LiveDeviceKnob = View{
        width: Fit
        height: Fit
        flow: Down
        align: Align{x: 0.5}
        spacing: 2
        poti := RoundedView{
            width: 26
            height: 26
            show_bg: true
            new_batch: true
            draw_bg.color: #x131313
            draw_bg.border_radius: 13.0
            draw_bg.border_size: 1.0
            draw_bg.border_color: #xe3b43e
            needle := SolidView{
                width: 2
                height: 9
                margin: Inset{top: 3 left: 12}
                show_bg: true
                new_batch: true
                draw_bg.color: #xe3b43e
            }
        }
        caption := InkLabel{
            width: Fit
            height: Fit
            text: "Tone"
            draw_text.color: #xa5a5a5
            draw_text.text_style: theme.font_regular{font_size: 8}
        }
    }

    mod.widgets.LiveDeviceParam = SliderFlat{
        width: 64
        height: 16
        min: 0.0
        max: 1.0
        default: 0.45
        precision: 2
        text: "Param"        draw_bg.color: #x131313
        draw_bg.color_hover: #x131313
        draw_bg.color_focus: #x131313
        draw_bg.color_drag: #x131313
        draw_bg.val_color: #xacc5ca
        draw_bg.val_color_hover: #xacc5ca
        draw_bg.handle_color: #xa5a5a5
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #xa5a5a5
        draw_text.text_style: theme.font_regular{font_size: 8}
    }

    // Dry/Wet 縦スライダ
    mod.widgets.LiveSliderVertical = View{
        width: Fit
        height: Fit
        flow: Down
        align: Align{x: 0.5}
        spacing: 2
        slot := SolidView{
            width: 8
            height: 56
            flow: Down
            show_bg: true
            new_batch: true
            draw_bg.color: #x131313
            fill := SolidView{
                width: Fill
                height: Fill
                margin: Inset{top: 18}
                show_bg: true
                new_batch: true
                draw_bg.color: #x4dd37c
            }
        }
        caption := InkLabel{
            width: Fit
            height: Fit
            text: "Dry/Wet"
            draw_text.color: #xa5a5a5
            draw_text.text_style: theme.font_regular{font_size: 8}
        }
    }

    mod.widgets.LiveSegmentButton = ButtonFlat{
        width: Fit
        height: 16
        padding: Inset{left: 8 right: 8}
        align: Center        text: "Osc 2"
        draw_bg.color: #x131313
        draw_bg.color_hover: #x131313
        draw_bg.color_down: #x131313
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #xa5a5a5
        draw_text.text_style: theme.font_regular{font_size: 8}
    }

    mod.widgets.LiveSegmentButtonOn = ButtonFlat{
        width: Fit
        height: 16
        padding: Inset{left: 8 right: 8}
        align: Center        text: "Osc 1"
        draw_bg.color: #x818181
        draw_bg.color_hover: #x818181
        draw_bg.color_down: #x131313
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #xa5a5a5
        draw_text.text_style: theme.font_regular{font_size: 8}
    }

    // 波形面 — 黄の横線。詳細波形は描かない
    mod.widgets.LiveVisualizerWave = SolidView{
        width: 72
        height: 36
        flow: Overlay
        align: Align{x: 0.0 y: 0.5}
        show_bg: true
        new_batch: true
        draw_bg.color: #x131313
        trace := SolidView{
            width: Fill
            height: 2
            margin: Inset{left: 4 right: 4}
            show_bg: true
            new_batch: true
            draw_bg.color: #xe3b43e
        }
    }

    // EQ カーブ面 — シアンの横線 + 節
    mod.widgets.LiveVisualizerEq = SolidView{
        width: 72
        height: 36
        flow: Overlay
        show_bg: true
        new_batch: true
        draw_bg.color: #x131313
        curve := SolidView{
            width: Fill
            height: 2
            margin: Inset{top: 16 left: 4 right: 4}
            show_bg: true
            new_batch: true
            draw_bg.color: #xacc5ca
        }
        node := SolidView{
            width: 6
            height: 6
            margin: Inset{top: 14 left: 28}
            show_bg: true
            new_batch: true
            draw_bg.color: #xe3b43e
        }
    }

    mod.widgets.LiveDeviceChain = SolidView{
        width: Fill
        height: Fit
        flow: Down
        show_bg: true
        new_batch: true
        draw_bg.color: #x242424
        bar := LiveDeviceTitleBar{}
        segs := View{
            width: Fill
            height: Fit
            flow: Right
            padding: Inset{left: 6 top: 4}
            spacing: 0
            LiveSegmentButtonOn{}
            LiveSegmentButton{}
        }
        body := View{
            width: Fill
            height: Fit
            flow: Right
            padding: 6
            spacing: 8
            align: Align{y: 0.5}
            LiveVisualizerWave{}
            LiveDeviceKnob{caption.text: "Tone"}
            LiveVisualizerEq{}
            LiveSliderVertical{}
        }
    }
}
