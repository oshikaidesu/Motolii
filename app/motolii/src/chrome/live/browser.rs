//! ホスト向け公開型: LiveBrowserSearch / LiveBrowserLabel / LiveBrowserLabelOn /
//! LiveFilterTag / LiveFilterTagOn / LiveBrowserItem / LiveBrowserItemOn /
//! LiveBrowserCollection
//!
//! 出典: 利用者添付の Live 12 実画面（左 Browser）
//!   検索 / カテゴリ（Sounds 選択シアン）/ Character・Genres タグ（選択オレンジ）/ .adv 行
//! 名称: https://www.ableton.com/en/live-manual/12/working-with-the-browser/
//! 色（画像）: パネル #a5a5a5 / インク #818181 / 選択 #acc5ca / タグオン #e3b43e
//!   行文字 #bdbdbd / 窪み検索 #131313
//! Document を持たない。寸法 px は出典なし。別名は代入。`ScrollYView` は書かない。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.LiveBrowserSearch = TextInputFlat{
        width: Fill
        height: 20
        padding: Inset{left: 8 right: 8}
        empty_text: "Search"
        draw_bg.color: #x131313
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #xa5a5a5
        draw_text.text_style: theme.font_regular{font_size: 11}
    }

    mod.widgets.LiveBrowserLabel = ButtonFlat{
        width: Fill
        height: 18
        align: Align{x: 0.0 y: 0.5}
        padding: Inset{left: 8 right: 8}        text: "Instruments"
        draw_bg.color: #xa5a5a5
        draw_bg.color_hover: #xbdbdbd
        draw_bg.color_down: #x818181
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #x818181
        draw_text.text_style: theme.font_regular{font_size: 11}
    }

    // Sounds 選択
    mod.widgets.LiveBrowserLabelOn = ButtonFlat{
        width: Fill
        height: 18
        align: Align{x: 0.0 y: 0.5}
        padding: Inset{left: 8 right: 8}        text: "Sounds"
        draw_bg.color: #xacc5ca
        draw_bg.color_hover: #xacc5ca
        draw_bg.color_down: #x818181
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #x818181
        draw_text.text_style: theme.font_regular{font_size: 11}
    }

    mod.widgets.LiveFilterTag = ButtonFlat{
        width: Fit
        height: 16
        padding: Inset{left: 8 right: 8}
        align: Center        text: "Analog"
        draw_bg.color: #x818181
        draw_bg.color_hover: #xbdbdbd
        draw_bg.color_down: #x818181
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 3.0
        draw_text.color: #xa5a5a5
        draw_text.text_style: theme.font_regular{font_size: 9}
    }

    // Pad / Evolving
    mod.widgets.LiveFilterTagOn = ButtonFlat{
        width: Fit
        height: 16
        padding: Inset{left: 8 right: 8}
        align: Center        text: "Pad"
        draw_bg.color: #xe3b43e
        draw_bg.color_hover: #xe3b43e
        draw_bg.color_down: #x818181
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 3.0
        draw_text.color: #x131313
        draw_text.text_style: theme.font_regular{font_size: 9}
    }

    mod.widgets.LiveBrowserItem = SolidView{
        width: Fill
        height: 18
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 8 right: 8}
        show_bg: true
        new_batch: true
        draw_bg.color: #xa5a5a5
        title := InkLabel{
            width: Fill
            height: Fit
            text: "Analog Slow Sweep Pad.adv"
            draw_text.color: #x818181
            draw_text.text_style: theme.font_regular{font_size: 10}
        }
    }

    // Drifting Ambient Pad.adv
    mod.widgets.LiveBrowserItemOn = SolidView{
        width: Fill
        height: 18
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 8 right: 8}
        show_bg: true
        new_batch: true
        draw_bg.color: #xacc5ca
        title := InkLabel{
            width: Fill
            height: Fit
            text: "Drifting Ambient Pad.adv"
            draw_text.color: #x818181
            draw_text.text_style: theme.font_regular{font_size: 10}
        }
    }

    // Collections 行（画面左の Favorites 等）。チップはタグオレンジ
    mod.widgets.LiveBrowserCollection = SolidView{
        width: Fill
        height: 18
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 8 right: 8}
        spacing: 6
        show_bg: true
        new_batch: true
        draw_bg.color: #xa5a5a5
        swatch := SolidView{
            width: 8
            height: 8
            show_bg: true
            new_batch: true
            draw_bg.color: #xe3b43e
        }
        title := InkLabel{
            width: Fill
            height: Fit
            text: "Favorites"
            draw_text.color: #x818181
            draw_text.text_style: theme.font_regular{font_size: 11}
        }
    }
}
