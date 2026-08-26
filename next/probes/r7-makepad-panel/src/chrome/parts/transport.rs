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
//! §2「Chrome / splash: ScrollYView 禁止（eval 白紙）」。技能 widgets は
//! ScrollYView を推奨するが、r7 splash eval では葉が落ちる。
//!
//! 色・寸法: chrome 閉集合と `timeline_transport` 節（帯高30・踏面30・gap2）。
//! playhead は mock `.play` 1.5px + `--accent`（`timeline-semantics.html` S5c）。
//! 新色は置かない。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // play — 停止中の顔。常時輪郭なし、hover で面が浮く。三角は play.svg
    mod.widgets.ChromePlay = ButtonFlatterIcon{
        width: 30
        height: 30
        padding: 0
        align: Center
        cursor: MouseCursor.Hand
        icon_walk: Walk{width: 14 height: 14}
        draw_bg.color: #x00000000
        draw_bg.color_hover: #x464646
        draw_bg.color_down: #x242424
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {
            svg: crate_resource("self://resources/icons/play.svg")
            color: #xb8b8b8
        }
    }

    // pause — 再生中の顔。器としての accent ink。棒は面、svg を足さない
    mod.widgets.ChromePause = ButtonFlatter{
        width: 30
        height: 30
        padding: 0
        flow: Right
        align: Center
        spacing: 3
        cursor: MouseCursor.Hand
        text: ""
        draw_bg.color: #x00000000
        draw_bg.color_hover: #x464646
        draw_bg.color_down: #x242424
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        bar_l := SolidView{
            width: 3
            height: 12
            show_bg: true
            new_batch: true
            draw_bg.color: #xd8b574
        }
        bar_r := SolidView{
            width: 3
            height: 12
            show_bg: true
            new_batch: true
            draw_bg.color: #xd8b574
        }
    }

    // play/pause — 一つの踏面。既定は停止顔。pause は隠す（再生は繋がない）
    mod.widgets.ChromePlayPause = View{
        width: 30
        height: 30
        flow: Overlay
        play := ChromePlay{}
        pause := ChromePause{visible: false}
    }

    // 先頭 — S0。踏面は ChromePlay と同じ。svg は first.svg
    mod.widgets.ChromeToStart = ButtonFlatterIcon{
        width: 30
        height: 30
        padding: 0
        align: Center
        cursor: MouseCursor.Hand
        icon_walk: Walk{width: 14 height: 14}
        draw_bg.color: #x00000000
        draw_bg.color_hover: #x464646
        draw_bg.color_down: #x242424
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {
            svg: crate_resource("self://resources/icons/first.svg")
            color: #xb8b8b8
        }
    }

    // 1コマ戻 — S0。Play の大三角と混ぜない（step_back.svg）
    mod.widgets.ChromeStepBack = ButtonFlatterIcon{
        width: 30
        height: 30
        padding: 0
        align: Center
        cursor: MouseCursor.Hand
        icon_walk: Walk{width: 14 height: 14}
        draw_bg.color: #x00000000
        draw_bg.color_hover: #x464646
        draw_bg.color_down: #x242424
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {
            svg: crate_resource("self://resources/icons/step_back.svg")
            color: #xb8b8b8
        }
    }

    // 1コマ進 — S0。step_forward.svg
    mod.widgets.ChromeStepForward = ButtonFlatterIcon{
        width: 30
        height: 30
        padding: 0
        align: Center
        cursor: MouseCursor.Hand
        icon_walk: Walk{width: 14 height: 14}
        draw_bg.color: #x00000000
        draw_bg.color_hover: #x464646
        draw_bg.color_down: #x242424
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {
            svg: crate_resource("self://resources/icons/step_forward.svg")
            color: #xb8b8b8
        }
    }

    // 末尾 — S0。last.svg
    mod.widgets.ChromeToEnd = ButtonFlatterIcon{
        width: 30
        height: 30
        padding: 0
        align: Center
        cursor: MouseCursor.Hand
        icon_walk: Walk{width: 14 height: 14}
        draw_bg.color: #x00000000
        draw_bg.color_hover: #x464646
        draw_bg.color_down: #x242424
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {
            svg: crate_resource("self://resources/icons/last.svg")
            color: #xb8b8b8
        }
    }

    // loop — 状態の器。on はホストが draw_icon.color を --accent へ。帯は持たない（T10）
    mod.widgets.ChromeLoop = ButtonFlatterIcon{
        width: 30
        height: 30
        padding: 0
        align: Center
        cursor: MouseCursor.Hand
        icon_walk: Walk{width: 14 height: 14}
        draw_bg.color: #x00000000
        draw_bg.color_hover: #x464646
        draw_bg.color_down: #x242424
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {
            svg: crate_resource("self://resources/icons/loop.svg")
            color: #xb8b8b8
        }
    }

    // timecode — 数字だけ accent / 等幅。単位 f/s は一段静か。値は見た目の種
    mod.widgets.ChromeTimecode = View{
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
            draw_bg.color: #x242424
            draw_bg.border_size: 0.0
            draw_bg.border_radius: 0.0
            draw_text.color: #xd8b574
            draw_text.text_style: theme.font_code{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
        }
        unit_f := ChromeInk{
            text: "f"
            draw_text.color: #x8c8c8c
            draw_text.text_style: theme.font_regular{font_size: 9 line_spacing: 1.0 top_drop: 0.0}
        }
        seconds := ChromeInk{
            text: "0.00"
            margin: Inset{left: 4}
            draw_text.color: #xd8b574
            draw_text.text_style: theme.font_code{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
        }
        unit_s := ChromeInk{
            text: "s"
            draw_text.color: #x8c8c8c
            draw_text.text_style: theme.font_regular{font_size: 9 line_spacing: 1.0 top_drop: 0.0}
        }
    }

    // 再生ヘッド — 針 1.5 / 頭 7。既定高 30（帯と同値）。時間面では height: Fill。
    // 親が Fit なら Fill 子は 0px（技能 layout）。Export E19 は同じ針を重ねる。
    mod.widgets.ChromePlayhead = View{
        width: 7
        height: 30
        flow: Overlay
        align: Align{x: 0.5 y: 0.0}
        new_batch: true
        needle := SolidView{
            width: 1.5
            height: Fill
            show_bg: true
            new_batch: true
            draw_bg.color: #xd8b574
        }
        cap := SolidView{
            width: 7
            height: 7
            show_bg: true
            new_batch: true
            draw_bg.color: #xd8b574
        }
    }

    // 帯 — クリップ面より一段明るい panel。線は引かない。S0 順 + loop + timecode
    mod.widgets.ChromeTransport = SolidView{
        width: Fill
        height: 30
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 6 right: 6}
        spacing: 2
        show_bg: true
        new_batch: true
        draw_bg.color: #x363636
        to_start := ChromeToStart{}
        step_back := ChromeStepBack{}
        play_toggle := ChromePlayPause{}
        step_forward := ChromeStepForward{}
        to_end := ChromeToEnd{}
        loop_toggle := ChromeLoop{}
        timecode := ChromeTimecode{}
    }
}
