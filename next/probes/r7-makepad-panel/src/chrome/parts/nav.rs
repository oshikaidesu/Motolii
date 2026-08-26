//! 共通 chrome 部品 — tab / chip / rail。Document を持たない。
//! 部品: `ChromeTabStrip` / `ChromeTab` / `ChromeTabOn` / `ChromeChip` / `ChromeChipOn` / `ChromeChipStrip` / `ChromeRail` / `ChromeRailItem` / `ChromeRailItemOn`。
//! ホスト向け: 上の9名を `mod.widgets` 代入別名として読む。`set_type_default()` は使わない。
//! `ScrollYView` は書かない（r7 splash eval 白紙。技能 layout の ScrollYView は頁本体向け。ここは Fit 原子）。
//! 出典: Browser mock `browser-library.html` `.libraryTabs` / `.filterShelf button` / `.locationRow`
//! （内部モック）。タブ帯は Premiere Pro bins と同型
//! https://helpx.adobe.com/premiere-pro/using/organizing-assets-project-panel.html
//! 選択の器は裁定179（非選択=輪郭なし、選択= accent 縁）。新色は置かない。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // tab strip — --section 26、面は panel
    mod.widgets.ChromeTabStrip = SolidView{
        width: Fill
        height: 26
        flow: Right
        align: Align{y: 0.5}
        spacing: 0
        show_bg: true
        new_batch: true
        draw_bg.color: #x363636
    }

    // tab — micro 8、idle は panel、hover は --hover
    mod.widgets.ChromeTab = ButtonFlat{
        width: Fill
        height: 26
        padding: Inset{left: 8 right: 8}
        draw_bg.color: #x363636
        draw_bg.color_hover: #x464646
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #x757575
        draw_text.text_style: theme.font_regular{font_size: 8 line_spacing: 1.0 top_drop: 0.0}
    }

    // tab on — mock `[aria-selected=true]`。下線 2 は `browser_tab_underline` / --accent
    mod.widgets.ChromeTabOn = View{
        width: Fill
        height: 26
        flow: Down
        new_batch: true
        tab := ButtonFlat{
            width: Fill
            height: Fill
            padding: Inset{left: 8 right: 8}
            draw_bg.color: #x242424
            draw_bg.color_hover: #x242424
            draw_bg.border_size: 0.0
            draw_bg.border_radius: 0.0
            draw_text.color: #xcfcfcf
            draw_text.text_style: theme.font_regular{font_size: 8 line_spacing: 1.0 top_drop: 0.0}
        }
        mark := SolidView{
            width: Fill
            height: 2
            show_bg: true
            new_batch: true
            draw_bg.color: #xd8b574
        }
    }

    // chip — 角丸 0.4×row_height=8、高さ 17、面は raised
    mod.widgets.ChromeChip = ButtonFlat{
        width: Fit
        height: 17
        padding: Inset{left: 4 right: 4 top: 2 bottom: 2}
        draw_bg.color: #x3e3e3e
        draw_bg.color_hover: #x464646
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 8.0
        draw_text.color: #xb8b8b8
        draw_text.text_style: theme.font_regular{font_size: 8 line_spacing: 1.0 top_drop: 0.0}
    }

    // chip on — 選択の器。面は ChromeToggle と同じ solo、縁/字は --accent
    mod.widgets.ChromeChipOn = ButtonFlat{
        width: Fit
        height: 17
        padding: Inset{left: 4 right: 4 top: 2 bottom: 2}
        draw_bg.color: #x443d2e
        draw_bg.color_hover: #x443d2e
        draw_bg.border_size: 1.0
        draw_bg.border_color: #xd8b574
        draw_bg.border_radius: 8.0
        draw_text.color: #xd8b574
        draw_text.text_style: theme.font_regular{font_size: 8 line_spacing: 1.0 top_drop: 0.0}
    }

    // chip strip — mock `.filterShelf` 24。TabStrip とは高さ/間隔が違う
    mod.widgets.ChromeChipStrip = SolidView{
        width: Fill
        height: 24
        flow: Right
        align: Align{y: 0.5}
        spacing: 2
        padding: Inset{left: 4 right: 4}
        show_bg: true
        new_batch: true
        draw_bg.color: #x363636
    }

    // rail — mock `.librarySidebar` 112。高さは Fit（Fill-in-Fit 0px を避ける）
    mod.widgets.ChromeRail = SolidView{
        width: 112
        height: Fit
        flow: Down
        padding: Inset{top: 2 bottom: 6}
        spacing: 0
        show_bg: true
        new_batch: true
        draw_bg.color: #x363636
    }

    // rail 行 — mock `.locationRow`。角丸 0、micro 8
    mod.widgets.ChromeRailItem = ButtonFlat{
        width: Fill
        height: 20
        padding: Inset{left: 8 right: 8}
        align: Align{x: 0.0 y: 0.5}
        draw_bg.color: #x363636
        draw_bg.color_hover: #x464646
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #xb8b8b8
        draw_text.text_style: theme.font_regular{font_size: 8 line_spacing: 1.0 top_drop: 0.0}
    }

    // rail 行 on — 左 2px --accent（`.locationRow.selected`）。親高 20 なので mark の Fill は落ちない
    mod.widgets.ChromeRailItemOn = View{
        width: Fill
        height: 20
        flow: Right
        new_batch: true
        mark := SolidView{
            width: 2
            height: Fill
            show_bg: true
            new_batch: true
            draw_bg.color: #xd8b574
        }
        item := ButtonFlat{
            width: Fill
            height: Fill
            padding: Inset{left: 6 right: 8}
            align: Align{x: 0.0 y: 0.5}
            draw_bg.color: #x3e3e3e
            draw_bg.color_hover: #x3e3e3e
            draw_bg.border_size: 0.0
            draw_bg.border_radius: 0.0
            draw_text.color: #xcfcfcf
            draw_text.text_style: theme.font_regular{font_size: 8 line_spacing: 1.0 top_drop: 0.0}
        }
    }
}
