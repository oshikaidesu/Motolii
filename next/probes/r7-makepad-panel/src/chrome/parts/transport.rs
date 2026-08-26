//! play/pause・step・loop・timecode・再生ヘッドの見た目。再生意味は持たない。
//!
//! ホスト向け型名:
//! `ChromePlay` / `ChromePause` / `ChromePlayPause` /
//! `ChromeToStart` / `ChromeStepBack` / `ChromeStepForward` / `ChromeToEnd` /
//! `ChromeLoop` / `ChromeTimecode` / `ChromePlayhead` / `ChromeTransport`。
//!
//! 再生ヘッドは Timeline / Stage / Export E19 が同じ針を読む（T01・ST-23・E19）。
//! 進捗バーは `ChromeProgress`（`stepper.rs`）。ここへ複製しない。
//!
//! `ScrollYView` は書かない。出典: `docs/reviews/2026-08-26-makepad-dock-panel-waves.md`
//! §2「Chrome / splash: ScrollYView 禁止（eval 白紙）」。
//!
//! 色・形の正本: 利用者添付の Live 12 Dark 実画面
//!   `assets/image-cf39df4e-cc7d-4299-9900-56934306be7e.png`（1024×554 から画素採取）。
//!   上バー #3d3d3d / 窪み #282828 / hover 面 #4f4f4f（パネル面）/
//!   再生三角 #f0f0f0・停止矩形 #ededed（主 glyph 白系）/ 副 glyph #b5b5b5 /
//!   窪み内の明数字 #b8b8b8 / 静インク #919191 / 最暗中立 #1d1d1d。
//!   針の実線は半透明黒の重なりで単色が採れないため最暗中立 #1d1d1d を採用。
//!   この閉集合の外へ新色を置かない。
//! 形の言語: glyph は小さい単色（三角・矩形・円）。面はフラット、角丸なし、
//!   枠線なし、影なし。timecode は窪み矩形に明数字。再生ヘッドは 1px 縦線（頭なし）。
//! 寸法: `timeline_transport` 節（帯高30・踏面30・gap2）を維持。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // 同一 script_mod 内で mod.widgets 登録名は素の名前で見えない(eval エラー実測)。
    // panel.splash と同じ let 束縛で持ち、登録は代入だけにする。
    // play — 停止中の顔。主 glyph 白 #f0f0f0。踏面は透明、hover でフラット矩形（角丸0）
    let PlayT = ButtonFlatterIcon{
        width: 30
        height: 30
        padding: 0
        align: Center        icon_walk: Walk{width: 14 height: 14}
        draw_bg.color: #x00000000
        draw_bg.color_hover: #x4f4f4f
        draw_bg.color_down: #x282828
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {
            svg: crate_resource("self://resources/icons/play.svg")
            color: #xf0f0f0
        }
    }
    mod.widgets.ChromePlay = PlayT

    // pause — 再生中の顔。Live 停止矩形と同じ白系 glyph。棒は面、svg を足さない
    let PauseT = ButtonFlatter{
        width: 30
        height: 30
        padding: 0
        flow: Right
        align: Center
        spacing: 3        text: ""
        draw_bg.color: #x00000000
        draw_bg.color_hover: #x4f4f4f
        draw_bg.color_down: #x282828
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        bar_l := SolidView{
            width: 3
            height: 12
            show_bg: true
            new_batch: true
            draw_bg.color: #xededed
        }
        bar_r := SolidView{
            width: 3
            height: 12
            show_bg: true
            new_batch: true
            draw_bg.color: #xededed
        }
    }
    mod.widgets.ChromePause = PauseT

    // play/pause — 一つの踏面。既定は停止顔。pause は隠す（再生は繋がない）
    let PlayPauseT = View{
        width: 30
        height: 30
        flow: Overlay
        play := PlayT{}
        pause := PauseT{visible: false}
    }
    mod.widgets.ChromePlayPause = PlayPauseT

    // 先頭 — S0。副 glyph #b5b5b5（上バーの +・draw ボタンと同格）。first.svg
    let ToStartT = ButtonFlatterIcon{
        width: 30
        height: 30
        padding: 0
        align: Center        icon_walk: Walk{width: 14 height: 14}
        draw_bg.color: #x00000000
        draw_bg.color_hover: #x4f4f4f
        draw_bg.color_down: #x282828
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {
            svg: crate_resource("self://resources/icons/first.svg")
            color: #xb5b5b5
        }
    }
    mod.widgets.ChromeToStart = ToStartT

    // 1コマ戻 — S0。Play の大三角と混ぜない（step_back.svg）
    let StepBackT = ButtonFlatterIcon{
        width: 30
        height: 30
        padding: 0
        align: Center        icon_walk: Walk{width: 14 height: 14}
        draw_bg.color: #x00000000
        draw_bg.color_hover: #x4f4f4f
        draw_bg.color_down: #x282828
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {
            svg: crate_resource("self://resources/icons/step_back.svg")
            color: #xb5b5b5
        }
    }
    mod.widgets.ChromeStepBack = StepBackT

    // 1コマ進 — S0。step_forward.svg
    let StepForwardT = ButtonFlatterIcon{
        width: 30
        height: 30
        padding: 0
        align: Center        icon_walk: Walk{width: 14 height: 14}
        draw_bg.color: #x00000000
        draw_bg.color_hover: #x4f4f4f
        draw_bg.color_down: #x282828
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {
            svg: crate_resource("self://resources/icons/step_forward.svg")
            color: #xb5b5b5
        }
    }
    mod.widgets.ChromeStepForward = StepForwardT

    // 末尾 — S0。last.svg
    let ToEndT = ButtonFlatterIcon{
        width: 30
        height: 30
        padding: 0
        align: Center        icon_walk: Walk{width: 14 height: 14}
        draw_bg.color: #x00000000
        draw_bg.color_hover: #x4f4f4f
        draw_bg.color_down: #x282828
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {
            svg: crate_resource("self://resources/icons/last.svg")
            color: #xb5b5b5
        }
    }
    mod.widgets.ChromeToEnd = ToEndT

    // loop — 状態の器。on はホストが draw_icon.color を accent へ。帯は持たない（T10）
    let LoopT = ButtonFlatterIcon{
        width: 30
        height: 30
        padding: 0
        align: Center        icon_walk: Walk{width: 14 height: 14}
        draw_bg.color: #x00000000
        draw_bg.color_hover: #x4f4f4f
        draw_bg.color_down: #x282828
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {
            svg: crate_resource("self://resources/icons/loop.svg")
            color: #xb5b5b5
        }
    }
    mod.widgets.ChromeLoop = LoopT

    // timecode — Live の位置表示(17.1.1)。窪み矩形 #282828 に明数字 #b8b8b8 / 等幅。
    // 枠線なし角丸なし。バー上に直に載る seconds は明数字、単位 f/s は静インクで一段小さく
    let TimecodeT = View{
        width: Fit
        height: Fit
        flow: Right
        align: Align{y: 0.5}
        spacing: 2
        new_batch: true
        frames := TextInputFlat{
            width: 64
            height: 20
            text: "0"
            empty_text: "0"
            is_numeric_only: true
            draw_bg.color: #x282828
            draw_bg.border_size: 0.0
            draw_bg.border_radius: 0.0
            draw_text.color: #xb8b8b8
            draw_text.text_style: theme.font_code{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
        }
        unit_f := ChromeInk{
            text: "f"
            draw_text.color: #x919191
            draw_text.text_style: theme.font_regular{font_size: 9 line_spacing: 1.0 top_drop: 0.0}
        }
        seconds := ChromeInk{
            text: "0.00"
            margin: Inset{left: 4}
            draw_text.color: #xb8b8b8
            draw_text.text_style: theme.font_code{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
        }
        unit_s := ChromeInk{
            text: "s"
            draw_text.color: #x919191
            draw_text.text_style: theme.font_regular{font_size: 9 line_spacing: 1.0 top_drop: 0.0}
        }
    }
    mod.widgets.ChromeTimecode = TimecodeT

    // 再生ヘッド — Live Arrangement の 1px 縦線。頭は付けない（cap は型合意のため残し非表示）。
    // 既定高 30（帯と同値）。時間面では height: Fill。親が Fit なら Fill 子は 0px（技能 layout）。
    // Export E19 は同じ針を重ねる。
    mod.widgets.ChromePlayhead = View{
        width: 7
        height: 30
        flow: Overlay
        align: Align{x: 0.5 y: 0.0}
        new_batch: true
        needle := SolidView{
            width: 1
            height: Fill
            show_bg: true
            new_batch: true
            draw_bg.color: #x1d1d1d
        }
        cap := SolidView{
            visible: false
            width: 7
            height: 7
            show_bg: true
            new_batch: true
            draw_bg.color: #x1d1d1d
        }
    }

    // 帯 — Live の上バー #3d3d3d。ベタ面のみ、線・角丸・影なし。S0 順 + loop + timecode
    mod.widgets.ChromeTransport = SolidView{
        width: Fill
        height: 30
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 6 right: 6}
        spacing: 2
        show_bg: true
        new_batch: true
        draw_bg.color: #x3d3d3d
        to_start := ToStartT{}
        step_back := StepBackT{}
        play_toggle := PlayPauseT{}
        step_forward := StepForwardT{}
        to_end := ToEndT{}
        loop_toggle := LoopT{}
        timecode := TimecodeT{}
    }
}
