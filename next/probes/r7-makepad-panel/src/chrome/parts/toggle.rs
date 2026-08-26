//! ホスト向け公開型: ChromeCheck / ChromeToggle / ChromeLock
//! 出典: After Effects layer switches — https://helpx.adobe.com/after-effects/desktop/work-with-layers/manage-layers/layers.html
//! checkbox / toggle / lock。Document を持たない。
//! 技能の `CheckBoxFlat` / `ToggleFlat` / `ButtonFlatIcon` を載せる。iced は置かない。
//! ScrollYView は書かない（eval 白紙）。
//! 色・寸法: chrome 既存（面 #x363636 / raised #x3e3e3e / hover #x464646 / インク #xb8b8b8 / 行高 20）。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // checkbox — CheckBoxFlat。箱は --app、印は ink、行高 20
    mod.widgets.ChromeCheck = CheckBoxFlat{
        width: Fit
        height: 20
        padding: 0
        align: Align{y: 0.5}
        cursor: MouseCursor.Hand
        text: "Check"
        label_walk: Walk{width: Fit height: Fit margin: Inset{left: 16}}
        draw_bg.size: 12.0
        draw_bg.border_size: 1.0
        draw_bg.border_radius: 0.0
        draw_bg.color: #x242424
        draw_bg.color_hover: #x464646
        draw_bg.color_down: #x242424
        draw_bg.color_active: #x3e3e3e
        draw_bg.color_focus: #x464646
        draw_bg.color_disabled: #x363636
        draw_bg.border_color: #x1d1d1d
        draw_bg.border_color_hover: #x1d1d1d
        draw_bg.border_color_down: #x1d1d1d
        draw_bg.border_color_active: #x1d1d1d
        draw_bg.border_color_focus: #xd8b574
        draw_bg.border_color_disabled: #x1d1d1d
        draw_bg.mark_color: #x00000000
        draw_bg.mark_color_hover: #x00000000
        draw_bg.mark_color_down: #x00000000
        draw_bg.mark_color_active: #xb8b8b8
        draw_bg.mark_color_active_hover: #xcfcfcf
        draw_bg.mark_color_focus: #xb8b8b8
        draw_bg.mark_color_disabled: #x757575
        draw_text.color: #xb8b8b8
        draw_text.color_hover: #xcfcfcf
        draw_text.color_down: #xb8b8b8
        draw_text.color_active: #xb8b8b8
        draw_text.color_focus: #xb8b8b8
        draw_text.color_disabled: #x757575
        draw_text.text_style: theme.font_regular{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
    }

    // toggle — ToggleFlat。オフは raised、オンは inspector solo 面、摘みは --accent
    mod.widgets.ChromeToggle = ToggleFlat{
        width: Fit
        height: 20
        padding: 0
        align: Align{y: 0.5}
        cursor: MouseCursor.Hand
        text: "On"
        label_walk: Walk{width: Fit height: Fit margin: Inset{left: 28}}
        draw_bg.size: 12.0
        draw_bg.border_size: 1.0
        draw_bg.color: #x3e3e3e
        draw_bg.color_hover: #x464646
        draw_bg.color_down: #x242424
        draw_bg.color_active: #x443d2e
        draw_bg.color_focus: #x464646
        draw_bg.color_disabled: #x363636
        draw_bg.border_color: #x1d1d1d
        draw_bg.border_color_hover: #x1d1d1d
        draw_bg.border_color_down: #x1d1d1d
        draw_bg.border_color_active: #x1d1d1d
        draw_bg.border_color_focus: #xd8b574
        draw_bg.border_color_disabled: #x1d1d1d
        draw_bg.mark_color: #x8c8c8c
        draw_bg.mark_color_hover: #xb8b8b8
        draw_bg.mark_color_down: #x8c8c8c
        draw_bg.mark_color_active: #xd8b574
        draw_bg.mark_color_active_hover: #xd8b574
        draw_bg.mark_color_focus: #xd8b574
        draw_bg.mark_color_disabled: #x757575
        draw_text.color: #x757575
        draw_text.color_hover: #xb8b8b8
        draw_text.color_down: #x757575
        draw_text.color_active: #xb8b8b8
        draw_text.color_focus: #xb8b8b8
        draw_text.color_disabled: #x757575
        draw_text.text_style: theme.font_regular{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
    }

    // lock — ButtonFlatIcon + 既存 lock.svg。踏面 24、グリフ 13
    mod.widgets.ChromeLock = ButtonFlatIcon{
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
        draw_icon +: {svg: crate_resource("self://resources/icons/lock.svg") color: #xb7b7b7}
    }
}
