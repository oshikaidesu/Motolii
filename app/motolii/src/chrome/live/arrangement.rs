//! ホスト向け公開型: LiveArrangementRuler / LiveArrangementClip / LivePlayhead /
//! LiveArrangementLane
//!
//! 出典: 利用者添付の Live 12 実画面（中央 Arrangement）
//!   ルーラー / グリッド / 色付きクリップ帯 / 再生ヘッド
//! 名称: https://www.ableton.com/en/live-manual/12/arrangement-view/
//! 色（画像）: パネル #a5a5a5 / ルーラー数字 #d0ced1 / 針 #111111
//!   クリップ 桃 #fba477 / 黄 #fde174 / ピンク #fc93a2 / シアン #58ffe5
//! Document を持たない。寸法 px は出典なし。別名は代入。`ScrollYView` は書かない。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.LiveArrangementRuler = SolidView{
        width: Fill
        height: 16
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 8 right: 8}
        spacing: 24
        show_bg: true
        new_batch: true
        draw_bg.color: #x818181
        m43 := InkLabel{
            width: Fit
            height: Fit
            text: "43"
            draw_text.color: #xd0ced1
            draw_text.text_style: theme.font_regular{font_size: 9}
        }
        m49 := InkLabel{
            width: Fit
            height: Fit
            text: "49"
            draw_text.color: #xd0ced1
            draw_text.text_style: theme.font_regular{font_size: 9}
        }
        m57 := InkLabel{
            width: Fit
            height: Fit
            text: "57"
            draw_text.color: #xd0ced1
            draw_text.text_style: theme.font_regular{font_size: 9}
        }
    }

    mod.widgets.LiveArrangementClip = SolidView{
        width: 96
        height: 18
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 6 right: 6}
        show_bg: true
        new_batch: true
        draw_bg.color: #xfba477
        title := InkLabel{
            width: Fill
            height: Fit
            text: "Pads"
            draw_text.color: #x131313
            draw_text.text_style: theme.font_regular{font_size: 9}
        }
    }

    // 画面の針は細い黒
    mod.widgets.LivePlayhead = SolidView{
        width: 1.5
        height: 24
        show_bg: true
        new_batch: true
        draw_bg.color: #x111111
    }

    mod.widgets.LiveArrangementLane = SolidView{
        width: Fill
        height: 22
        flow: Overlay
        show_bg: true
        new_batch: true
        draw_bg.color: #xa5a5a5
        grid := View{
            width: Fill
            height: Fill
            flow: Right
            spacing: 16
            SolidView{width: 1 height: Fill show_bg: true new_batch: true draw_bg.color: #x818181}
            SolidView{width: 1 height: Fill show_bg: true new_batch: true draw_bg.color: #x818181}
            SolidView{width: 1 height: Fill show_bg: true new_batch: true draw_bg.color: #x818181}
            SolidView{width: 1 height: Fill show_bg: true new_batch: true draw_bg.color: #x818181}
        }
        clips := View{
            width: Fill
            height: Fill
            flow: Right
            align: Align{y: 0.5}
            padding: Inset{left: 8}
            spacing: 4
            LiveArrangementClip{}
            LiveArrangementClip{
                width: 64
                title.text: "Keys"
                draw_bg.color: #xfde174
            }
            LiveArrangementClip{
                width: 48
                title.text: "Rain"
                draw_bg.color: #xfc93a2
            }
        }
        head := View{
            width: Fill
            height: Fill
            flow: Right
            LivePlayhead{margin: Inset{left: 72}}
        }
    }
}
