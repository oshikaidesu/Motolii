//! ホスト向け公開型: LiveLink / LiveTap / LiveTempo / LiveTimeSignature /
//! LiveMetronome / LivePlay / LiveStop / LiveRecord / LiveArrangementPosition /
//! LiveQuantization / LiveMidi / LiveCpu / LiveTransport
//!
//! 出典: 利用者添付の Live 12 実画面（上バー）
//!   Link / Tap / 175.00 / 4 4 / メトロノーム / 再生・停止・録音 / 57.3.1 / MIDI・CPU
//! 名称: https://www.ableton.com/en/live-manual/12/live-concepts/
//! 色（画像）: バー #818181 / 窪み #131313 / インク #a5a5a5 / 再生 #24e8a6
//!   ループ黄 #e9be3c（Quantization 選択見本）
//! Document を持たない。寸法 px は出典なし。別名は代入。`ScrollYView` は書かない。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.LiveLink = ButtonFlatter{
        width: Fit
        height: 18
        padding: Inset{left: 6 right: 6}
        align: Center
        cursor: MouseCursor.Hand
        text: "Link"
        draw_bg.color: #x00000000
        draw_bg.color_hover: #x818181
        draw_bg.color_down: #x131313
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #xa5a5a5
        draw_text.text_style: theme.font_regular{font_size: 10 line_spacing: 1.0 top_drop: 0.0}
    }

    mod.widgets.LiveTap = ButtonFlat{
        width: Fit
        height: 16
        padding: Inset{left: 8 right: 8}
        align: Center
        cursor: MouseCursor.Hand
        text: "Tap"
        draw_bg.color: #xa5a5a5
        draw_bg.color_hover: #xbdbdbd
        draw_bg.color_down: #x818181
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 3.0
        draw_text.color: #x818181
        draw_text.text_style: theme.font_regular{font_size: 9 line_spacing: 1.0 top_drop: 0.0}
    }

    mod.widgets.LiveTempo = View{
        width: Fit
        height: Fit
        flow: Right
        align: Align{y: 0.5}
        spacing: 4
        bpm := TextInputFlat{
            width: 48
            height: 16
            text: "175.00"
            empty_text: "BPM"
            is_numeric_only: true
            draw_bg.color: #x131313
            draw_bg.border_size: 0.0
            draw_bg.border_radius: 0.0
            draw_text.color: #xa5a5a5
            draw_text.text_style: theme.font_code{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
        }
    }

    mod.widgets.LiveTimeSignature = View{
        width: Fit
        height: Fit
        flow: Right
        align: Align{y: 0.5}
        spacing: 2
        num := TextInputFlat{
            width: 18
            height: 16
            text: "4"
            is_numeric_only: true
            draw_bg.color: #x131313
            draw_bg.border_size: 0.0
            draw_bg.border_radius: 0.0
            draw_text.color: #xa5a5a5
            draw_text.text_style: theme.font_code{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
        }
        den := TextInputFlat{
            width: 18
            height: 16
            text: "4"
            is_numeric_only: true
            draw_bg.color: #x131313
            draw_bg.border_size: 0.0
            draw_bg.border_radius: 0.0
            draw_text.color: #xa5a5a5
            draw_text.text_style: theme.font_code{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
        }
    }

    // メトロノーム — 円の拍インジケータ
    mod.widgets.LiveMetronome = RoundedView{
        width: 10
        height: 10
        show_bg: true
        new_batch: true
        draw_bg.color: #xa5a5a5
        draw_bg.border_radius: 5.0
        draw_bg.border_size: 0.0
    }

    mod.widgets.LivePlay = ButtonFlatter{
        width: 22
        height: 22
        padding: 0
        align: Center
        cursor: MouseCursor.Hand
        text: ">"
        draw_bg.color: #x00000000
        draw_bg.color_hover: #x818181
        draw_bg.color_down: #x131313
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #x24e8a6
        draw_text.text_style: theme.font_regular{font_size: 12 line_spacing: 1.0 top_drop: 0.0}
    }

    mod.widgets.LiveStop = ButtonFlatter{
        width: 22
        height: 22
        padding: 0
        align: Center
        cursor: MouseCursor.Hand
        text: ""
        draw_bg.color: #x00000000
        draw_bg.color_hover: #x818181
        draw_bg.color_down: #x131313
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        stop := SolidView{
            width: 8
            height: 8
            show_bg: true
            new_batch: true
            draw_bg.color: #xa5a5a5
        }
    }

    // 録音 — 画面上は灰の円。赤はこの1枚に無い
    mod.widgets.LiveRecord = ButtonFlatter{
        width: 22
        height: 22
        padding: 0
        align: Center
        cursor: MouseCursor.Hand
        text: ""
        draw_bg.color: #x00000000
        draw_bg.color_hover: #x818181
        draw_bg.color_down: #x131313
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        rec := RoundedView{
            width: 10
            height: 10
            show_bg: true
            new_batch: true
            draw_bg.color: #xa5a5a5
            draw_bg.border_radius: 5.0
        }
    }

    mod.widgets.LiveArrangementPosition = View{
        width: Fit
        height: Fit
        flow: Right
        align: Align{y: 0.5}
        spacing: 2
        bars := TextInputFlat{
            width: 28
            height: 16
            text: "57"
            is_numeric_only: true
            draw_bg.color: #x131313
            draw_bg.border_size: 0.0
            draw_bg.border_radius: 0.0
            draw_text.color: #xa5a5a5
            draw_text.text_style: theme.font_code{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
        }
        beats := TextInputFlat{
            width: 16
            height: 16
            text: "3"
            is_numeric_only: true
            draw_bg.color: #x131313
            draw_bg.border_size: 0.0
            draw_bg.border_radius: 0.0
            draw_text.color: #xa5a5a5
            draw_text.text_style: theme.font_code{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
        }
        sixteenths := TextInputFlat{
            width: 16
            height: 16
            text: "1"
            is_numeric_only: true
            draw_bg.color: #x131313
            draw_bg.border_size: 0.0
            draw_bg.border_radius: 0.0
            draw_text.color: #xa5a5a5
            draw_text.text_style: theme.font_code{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
        }
    }

    // 画面の Global Quantization は "2 Bars"
    mod.widgets.LiveQuantization = DropDownFlat{
        width: Fit
        height: 16
        padding: Inset{left: 6 right: 16}
        labels: ["2 Bars" "1 Bar" "1/4" "1/8" "1/16"]
        selected_item: 0
        draw_bg.color: #x131313
        draw_bg.color_hover: #x818181
        draw_bg.color_down: #x131313
        draw_bg.color_focus: #x818181
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_bg.arrow_color: #xa5a5a5
        draw_text.color: #xa5a5a5
        draw_text.text_style: theme.font_regular{font_size: 9 line_spacing: 1.0 top_drop: 0.0}
        popup_menu: PopupMenuFlat{
            width: 88
            height: Fit
            flow: Down
            padding: 2
            draw_bg.color: #x818181
            draw_bg.border_radius: 0.0
            draw_bg.border_size: 0.0
            menu_item: PopupMenuItem{
                width: Fill
                height: 16
                align: Align{y: 0.5}
                padding: Inset{left: 6 right: 6}
                draw_bg.color: #x818181
                draw_bg.color_hover: #xacc5ca
                draw_bg.color_active: #xacc5ca
                draw_bg.mark_color: #x818181
                draw_bg.mark_color_active: #xe3b43e
                draw_text.color: #xa5a5a5
                draw_text.color_hover: #x818181
                draw_text.color_active: #x818181
                draw_text.text_style: theme.font_regular{font_size: 9 line_spacing: 1.0 top_drop: 0.0}
            }
        }
    }

    mod.widgets.LiveMidi = ButtonFlatter{
        width: Fit
        height: 16
        padding: Inset{left: 4 right: 4}
        text: "MIDI"
        draw_bg.color: #x00000000
        draw_bg.color_hover: #x818181
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #xa5a5a5
        draw_text.text_style: theme.font_regular{font_size: 8 line_spacing: 1.0 top_drop: 0.0}
    }

    mod.widgets.LiveCpu = View{
        width: Fit
        height: Fit
        flow: Right
        align: Align{y: 0.5}
        spacing: 2
        value := Label{
            width: Fit
            height: Fit
            text: "14%"
            draw_text.color: #xa5a5a5
            draw_text.text_style: theme.font_regular{font_size: 8 line_spacing: 1.0 top_drop: 0.0}
        }
    }

    mod.widgets.LiveTransport = SolidView{
        width: Fill
        height: 28
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 6 right: 6}
        spacing: 4
        show_bg: true
        new_batch: true
        draw_bg.color: #x818181
        link := LiveLink{}
        tap := LiveTap{}
        tempo := LiveTempo{}
        signature := LiveTimeSignature{}
        metro := LiveMetronome{}
        stop := LiveStop{}
        play := LivePlay{}
        record := LiveRecord{}
        position := LiveArrangementPosition{}
        midi := LiveMidi{}
        cpu := LiveCpu{}
    }
}
