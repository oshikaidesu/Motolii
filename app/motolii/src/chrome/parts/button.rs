//! ホスト向け公開型: ChromeIcon / ChromeGhost
//! 出典: 利用者添付の Live 12 Dark 実画面
//!   `assets/image-cf39df4e-cc7d-4299-9900-56934306be7e.png`（1024×554 から画素実測）
//! 形: 完全な矩形のベタ面。角丸・枠線・影を作らない。状態は面色の切替だけ。
//! 色（画像サンプル）: ボタン面 #3d3d3d / hover は行地 #4f4f4f / 押下は窪み #282828
//!   明グリフ #efefef（上バー再生三角）/ 二次グリフ #a0a0a0
//! アイコン踏面だけ。文字の `ChromeButton`・錠の `ChromeLock` は置かない。
//! 技能の `ButtonFlatIcon` / `ButtonFlatterIcon` を載せる。iced は置かない。
//! ScrollYView は書かない（eval 白紙）。
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
        align: Center        text: ""
        icon_walk: Walk{width: 13 height: 13}
        draw_bg.color: #x3d3d3d
        draw_bg.color_hover: #x4f4f4f
        draw_bg.color_down: #x282828
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {color: #xefefef}
    }

    // ghost — ButtonFlatterIcon。面は無し。key / mute の二次グリフ
    mod.widgets.ChromeGhost = ButtonFlatterIcon{
        width: 24
        height: 24
        padding: 0
        margin: 0.
        spacing: 0
        align: Center        text: ""
        icon_walk: Walk{width: 13 height: 13}
        draw_bg.color: #x00000000
        draw_bg.color_hover: #x3d3d3d
        draw_bg.color_down: #x282828
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {color: #xa0a0a0}
    }
}
