//! ホスト向け公開型: ChromeIcon / ChromeGhost
//! 出典: Material 3 Icon buttons — https://m3.material.io/components/icon-buttons/overview
//! アイコン踏面だけ。文字の `ChromeButton`・錠の `ChromeLock` は置かない。
//! 技能の `ButtonFlatIcon` / `ButtonFlatterIcon` を載せる。iced は置かない。
//! ScrollYView は書かない（eval 白紙）。
//! 色・寸法: chrome 既存（raised #x3e3e3e / hover #x464646 / インク #xb7b7b7 / 踏面 24 / グリフ 13）。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // icon — ButtonFlatIcon。踏面 24（interactive_target_min）。svg は使う側が上書き
    mod.widgets.ChromeIcon = ButtonFlatIcon{
        width: 24
        height: 24
        padding: 0
        margin: 0.
        spacing: 0
        align: Center
        cursor: MouseCursor.Hand
        text: ""
        icon_walk: Walk{width: 13 height: 13}
        draw_bg.color: #x3e3e3e
        draw_bg.color_hover: #x464646
        draw_bg.color_down: #x242424
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {color: #xb7b7b7}
    }

    // ghost — ButtonFlatterIcon。面は無し。key / mute の二次グリフ
    mod.widgets.ChromeGhost = ButtonFlatterIcon{
        width: 24
        height: 24
        padding: 0
        margin: 0.
        spacing: 0
        align: Center
        cursor: MouseCursor.Hand
        text: ""
        icon_walk: Walk{width: 13 height: 13}
        draw_bg.color: #x00000000
        draw_bg.color_hover: #x464646
        draw_bg.color_down: #x242424
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {color: #xb7b7b7}
    }
}
