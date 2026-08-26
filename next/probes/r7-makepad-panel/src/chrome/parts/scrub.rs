//! ホスト向け公開型: ChromeScrub
//! ホスト向け: 上の1名を `mod.widgets` 代入別名として読む。`set_type_default()` は使わない。
//! 出典: After Effects — drag the slider / underlined value
//!   https://helpx.adobe.com/after-effects/desktop/work-with-layers/layer-properties/layer-properties.html
//! 数値スライダー。Document を持たない。細いバーは `ChromeProgress`（`stepper.rs`）。
//! 技能の `SliderFlat` を載せる。iced は置かない。
//! `ScrollYView` は書かない。出典: `docs/reviews/2026-08-26-makepad-dock-panel-waves.md`
//! §2「Chrome / splash: ScrollYView 禁止（eval 白紙）」。技能 widgets は
//! ScrollYView を推奨するが、r7 splash eval では葉が落ちる。
//! 色・寸法: chrome 既存（面 --app #x242424 / 値 --accent / 摘み ink / 踏面 24）。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // 数値 — SliderFlat。面 --app、値 --accent、摘み ink。高さ 24（interactive_target_min）
    // ChromeProgress と同系、高さだけ上げる。handle は残す
    mod.widgets.ChromeScrub = SliderFlat{
        width: Fill
        height: 24
        min: 0.0
        max: 100.0
        default: 50.0
        precision: 0
        text: "Value"
        cursor: MouseCursor.Hand
        draw_bg.color: #x242424
        draw_bg.color_hover: #x242424
        draw_bg.color_focus: #x242424
        draw_bg.color_drag: #x242424
        draw_bg.color_disabled: #x363636
        draw_bg.val_color: #xd8b574
        draw_bg.val_color_hover: #xd8b574
        draw_bg.val_color_focus: #xd8b574
        draw_bg.val_color_drag: #xd8b574
        draw_bg.val_color_disabled: #x757575
        draw_bg.handle_color: #xb8b8b8
        draw_bg.handle_color_hover: #xcfcfcf
        draw_bg.handle_color_focus: #xd8b574
        draw_bg.handle_color_drag: #xd8b574
        draw_bg.handle_color_disabled: #x757575
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #xb8b8b8
        draw_text.text_style: theme.font_regular{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
    }
}
