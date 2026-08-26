//! ホスト向け公開型: ChromeStepper / ChromeProgress
//! ホスト向け: 上の2名を `mod.widgets` 代入別名として読む。`set_type_default()` は使わない。
//! 出典: After Effects — increment/decrement underlined value (Up/Down)
//!   https://helpx.adobe.com/after-effects/desktop/work-with-layers/layer-properties/layer-properties.html
//! +/- stepper と細い progress。Document を持たない。
//! 部品: `ChromeStepper` / `ChromeProgress`。
//! 数値スライダーは `ChromeScrub`（`scrub.rs`）。ここへ複製しない。
//! `ScrollYView` は書かない。出典: `docs/reviews/2026-08-26-makepad-dock-panel-waves.md`
//! §2「Chrome / splash: ScrollYView 禁止（eval 白紙）」。技能 widgets は
//! ScrollYView を推奨するが、r7 splash eval では葉が落ちる。
//! 色・寸法: `ui-scale-and-z.html` 候補B と r7 `panel.splash`。新色は置かない。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // +/- stepper — 踏面 24（ChromeButton / interactive_target_min）、間隔 --sp1
    // minus / value / plus は :=。click は ButtonFlat、値は表示だけ
    mod.widgets.ChromeStepper = View{
        width: Fit
        height: 24
        flow: Right
        spacing: 2
        align: Align{y: 0.5}
        new_batch: true

        minus := ButtonFlat{
            width: 24
            height: 24
            padding: 0
            cursor: MouseCursor.Hand
            text: "−"
            draw_bg.color: #x3e3e3e
            draw_bg.color_hover: #x464646
            draw_bg.color_down: #x242424
            draw_bg.border_size: 0.0
            draw_bg.border_radius: 0.0
            draw_text.color: #xb8b8b8
            draw_text.text_style: theme.font_regular{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
        }
        value := Label{
            width: Fit
            height: Fit
            padding: Inset{left: 8 right: 8}
            text: "0"
            draw_text.color: #xb8b8b8
            draw_text.text_style: theme.font_regular{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
        }
        plus := ButtonFlat{
            width: 24
            height: 24
            padding: 0
            cursor: MouseCursor.Hand
            text: "+"
            draw_bg.color: #x3e3e3e
            draw_bg.color_hover: #x464646
            draw_bg.color_down: #x242424
            draw_bg.border_size: 0.0
            draw_bg.border_radius: 0.0
            draw_text.color: #xb8b8b8
            draw_text.text_style: theme.font_regular{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
        }
    }

    // 細い progress — 高さ --sp1、面 --app、値 --accent。SliderMinimalFlat の fill
    // handle は 0。値は 0.0..=1.0。ChromeScrub と同系、高さだけ落とす
    mod.widgets.ChromeProgress = SliderMinimalFlat{
        width: Fill
        height: 2
        min: 0.0
        max: 1.0
        default: 0.0
        precision: 2
        text: ""
        draw_bg.color: #x242424
        draw_bg.val_color: #xd8b574
        draw_bg.handle_color: #xd8b574
        draw_bg.handle_size: 0.0
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #xb8b8b8
        draw_text.text_style: theme.font_regular{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
    }
}
