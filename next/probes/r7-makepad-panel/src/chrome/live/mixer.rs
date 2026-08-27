//! ホスト向け公開型: LiveMeter / LiveVolume / LivePan / LiveTrackActivator /
//! LiveSolo / LiveArm / LiveTrackTitleBar / LiveTrackMixer
//!
//! 出典: 利用者添付の Live 12 実画面（ミキサー）
//!   色付きトラック頭 / 縦フェーダー / パンノブ / ミュート黄（番号）/ ソロ青
//! 名称: https://www.ableton.com/en/live-manual/12/mixing/
//!   Track Activator が画面の黄色い番号ボタン。Mute というラベルは画面に無い。
//! 色（画像）: パネル #a5a5a5 / メーター緑 #27e466 / アクティベータ #e9be3c
//!   ソロ #9aa8f1 / パンノブ #131313 / クリップ頭 #fba477
//! Document を持たない。寸法 px は出典なし。別名は代入。`ScrollYView` は書かない。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.LiveMeter = SolidView{
        width: 6
        height: 64
        flow: Down
        show_bg: true
        new_batch: true
        draw_bg.color: #x131313
        hot := SolidView{
            width: Fill
            height: 8
            show_bg: true
            new_batch: true
            draw_bg.color: #xe3b43e
        }
        body := SolidView{
            width: Fill
            height: Fill
            show_bg: true
            new_batch: true
            draw_bg.color: #x27e466
        }
    }

    // 縦フェーダー — 黒い槽 + 三角ハンドル
    mod.widgets.LiveVolume = View{
        width: 14
        height: 64
        flow: Overlay
        track := SolidView{
            width: 4
            height: Fill
            margin: Inset{left: 5}
            show_bg: true
            new_batch: true
            draw_bg.color: #x131313
        }
        handle := SolidView{
            width: 10
            height: 6
            margin: Inset{top: 22 left: 2}
            show_bg: true
            new_batch: true
            draw_bg.color: #x111111
        }
    }

    // パンノブ
    mod.widgets.LivePan = View{
        width: Fit
        height: Fit
        flow: Down
        align: Align{x: 0.5}
        poti := RoundedView{
            width: 18
            height: 18
            show_bg: true
            new_batch: true
            draw_bg.color: #x131313
            draw_bg.border_radius: 9.0
            draw_bg.border_size: 0.0
            needle := SolidView{
                width: 2
                height: 6
                margin: Inset{top: 2 left: 8}
                show_bg: true
                new_batch: true
                draw_bg.color: #xacc5ca
            }
        }
    }

    // 黄色い番号 = Track Activator（画面のミュート相当）
    mod.widgets.LiveTrackActivator = ButtonFlat{
        width: 16
        height: 14
        padding: 0
        align: Center        text: "8"
        draw_bg.color: #xe9be3c
        draw_bg.color_hover: #xe9be3c
        draw_bg.color_down: #x818181
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 2.0
        draw_text.color: #x131313
        draw_text.text_style: theme.font_regular{font_size: 8}
    }

    mod.widgets.LiveSolo = ButtonFlat{
        width: 16
        height: 14
        padding: 0
        align: Center        text: "S"
        draw_bg.color: #x9aa8f1
        draw_bg.color_hover: #x9aa8f1
        draw_bg.color_down: #x818181
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 7.0
        draw_text.color: #x131313
        draw_text.text_style: theme.font_regular{font_size: 8}
    }

    // この1枚のミキサーに Arm は見えない。型は残す。赤は画像に無いので灰円
    mod.widgets.LiveArm = ButtonFlat{
        width: 16
        height: 14
        padding: 0
        align: Center        text: ""
        draw_bg.color: #x818181
        draw_bg.color_hover: #xbdbdbd
        draw_bg.color_down: #x131313
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 7.0
        draw_text.color: #x00000000
    }

    mod.widgets.LiveTrackTitleBar = SolidView{
        width: Fill
        height: 16
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 3 right: 3}
        show_bg: true
        new_batch: true
        draw_bg.color: #xfba477
        title := InkLabel{
            width: Fill
            height: Fit
            text: "Pads"
            draw_text.color: #x131313
            draw_text.text_style: theme.font_regular{font_size: 8}
        }
    }

    mod.widgets.LiveTrackMixer = SolidView{
        width: 52
        height: Fit
        flow: Down
        align: Align{x: 0.5}
        padding: 3
        spacing: 3
        show_bg: true
        new_batch: true
        draw_bg.color: #xa5a5a5
        title := LiveTrackTitleBar{}
        meters := View{
            width: Fill
            height: 64
            flow: Right
            align: Align{x: 0.5}
            spacing: 3
            LiveMeter{}
            LiveVolume{}
        }
        pan := LivePan{}
        switches := View{
            width: Fill
            height: Fit
            flow: Right
            align: Align{x: 0.5}
            spacing: 2
            LiveTrackActivator{}
            LiveSolo{}
        }
    }
}
