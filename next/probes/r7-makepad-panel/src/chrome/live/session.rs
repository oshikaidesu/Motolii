//! ホスト向け公開型: LiveClipLaunch / LiveClipStop / LiveClipSlotEmpty /
//! LiveClipSlot / LiveClipSlotPlaying / LiveGroupSlot / LiveSceneLaunch /
//! LiveClipTitleBar
//!
//! この1枚に Session グリッドは無い。型は残す。色は添付画面のパレットだけ使う。
//! 出典色: 利用者添付 Live 12 実画面。名称:
//!   https://www.ableton.com/en/live-manual/12/session-view/
//!   https://www.ableton.com/en/live-manual/12/clip-view/
//! クリップ面は画面の Arrangement 帯（#fba477）。再生インクは画面の Play #24e8a6。
//! Document を持たない。寸法 px は出典なし。別名は代入。`ScrollYView` は書かない。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.LiveClipLaunch = ButtonFlatter{
        width: 16
        height: 18
        padding: 0
        align: Center        text: ">"
        draw_bg.color: #x00000000
        draw_bg.color_hover: #xbdbdbd
        draw_bg.color_down: #x818181
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #x131313
        draw_text.text_style: theme.font_regular{font_size: 10 line_spacing: 1.0 top_drop: 0.0}
    }

    mod.widgets.LiveClipStop = ButtonFlatter{
        width: 16
        height: 18
        padding: 0
        align: Center        text: ""
        draw_bg.color: #x00000000
        draw_bg.color_hover: #xbdbdbd
        draw_bg.color_down: #x818181
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        stop := SolidView{
            width: 7
            height: 7
            show_bg: true
            new_batch: true
            draw_bg.color: #x818181
        }
    }

    mod.widgets.LiveClipSlotEmpty = SolidView{
        width: 88
        height: 20
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 2 right: 4}
        show_bg: true
        new_batch: true
        draw_bg.color: #xa5a5a5
        stop := LiveClipStop{}
    }

    mod.widgets.LiveClipSlot = SolidView{
        width: 88
        height: 20
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 2 right: 4}
        spacing: 2
        show_bg: true
        new_batch: true
        draw_bg.color: #xfba477
        launch := LiveClipLaunch{}
        name := Label{
            width: Fill
            height: Fit
            text: "Clip"
            draw_text.color: #x131313
            draw_text.text_style: theme.font_regular{font_size: 10 line_spacing: 1.0 top_drop: 0.0}
        }
    }

    mod.widgets.LiveClipSlotPlaying = SolidView{
        width: 88
        height: 20
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 2 right: 4}
        spacing: 2
        show_bg: true
        new_batch: true
        draw_bg.color: #xfba477
        launch := LiveClipLaunch{
            text: ">"
            draw_text.color: #x24e8a6
        }
        name := Label{
            width: Fill
            height: Fit
            text: "Clip"
            draw_text.color: #x131313
            draw_text.text_style: theme.font_regular{font_size: 10 line_spacing: 1.0 top_drop: 0.0}
        }
    }

    mod.widgets.LiveGroupSlot = SolidView{
        width: 88
        height: 20
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 2 right: 4}
        show_bg: true
        new_batch: true
        draw_bg.color: #xfba47799
        launch := LiveClipLaunch{draw_text.color: #x818181}
    }

    mod.widgets.LiveSceneLaunch = ButtonFlat{
        width: 20
        height: 20
        padding: 0
        align: Center        text: ">"
        draw_bg.color: #xa5a5a5
        draw_bg.color_hover: #xbdbdbd
        draw_bg.color_down: #x818181
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #x818181
        draw_text.text_style: theme.font_regular{font_size: 10 line_spacing: 1.0 top_drop: 0.0}
    }

    mod.widgets.LiveClipTitleBar = SolidView{
        width: Fill
        height: 20
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 4 right: 6}
        spacing: 6
        show_bg: true
        new_batch: true
        draw_bg.color: #x242424
        activator := ButtonFlat{
            width: 12
            height: 12
            padding: 0
            text: ""
            draw_bg.color: #xe9be3c
            draw_bg.color_hover: #xe9be3c
            draw_bg.color_down: #x131313
            draw_bg.border_size: 0.0
            draw_bg.border_radius: 2.0
        }
        swatch := SolidView{
            width: 10
            height: 10
            show_bg: true
            new_batch: true
            draw_bg.color: #xfba477
        }
        title := Label{
            width: Fill
            height: Fit
            text: "Clip"
            draw_text.color: #xa5a5a5
            draw_text.text_style: theme.font_regular{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
        }
    }
}
