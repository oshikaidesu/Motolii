//! dropdown / context menu / menubar の見た目。
//! 閉集合: 面 / 項 / 注 / 線 / ドロップ / バー / 葉。Document を持たない。
//! 部品: `ChromeMenuFace` / `ChromeMenuItem` / `ChromeMenuHint` / `ChromeMenuRule` / `ChromeDrop` / `ChromeMenuBar` / `ChromeMenuLeaf`。
//! ホスト向け: 上の7名を `mod.widgets` 代入別名として読む。`set_type_default()` は使わない。
//! `ScrollYView` は書かない（r7 splash eval 白紙。技能 layout の ScrollYView は頁本体向け。ここは Fit 原子）。
//! 出典: 裁定179（バー=素の文字+hover面、開いた面=raised+角丸）。寸法は
//! `dimensions.json` `menubar_menu_width` 192 / `menubar_corner_radius` 4。
//! 葉の label+shortcut は Premiere Pro メニューと同型
//! https://helpx.adobe.com/premiere-pro/using/using-keyboard-shortcuts.html
//! 新色は置かない。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // 面 — 開いた menu。raised + 角丸4(裁定179)。幅は menubar_menu_width
    mod.widgets.ChromeMenuFace = RoundedView{
        width: 192
        height: Fit
        flow: Down
        padding: 2
        spacing: 0
        show_bg: true
        new_batch: true
        clip_x: false
        clip_y: false
        draw_bg.color: #x3e3e3e
        draw_bg.border_radius: 4.0
        draw_bg.border_size: 0.0
    }

    // 項 — row_height 20、hover は mock --hover、down は mock --app
    mod.widgets.ChromeMenuItem = ButtonFlat{
        width: Fill
        height: 20
        align: Align{x: 0.0 y: 0.5}
        padding: Inset{left: 8 right: 8}
        cursor: MouseCursor.Hand
        draw_bg.color: #x3e3e3e
        draw_bg.color_hover: #x464646
        draw_bg.color_down: #x242424
        draw_bg.color_disabled: #x3e3e3e
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #xb8b8b8
        draw_text.color_disabled: #x757575
        draw_text.text_style: theme.font_regular{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
    }

    // 注 — shortcut / muted。mock --ink3、caption 9
    mod.widgets.ChromeMenuHint = Label{
        width: Fit
        height: Fit
        padding: 0
        draw_text.color: #x757575
        draw_text.text_style: theme.font_regular{font_size: 9 line_spacing: 1.0 top_drop: 0.0}
    }

    // 線 — menu 内 hairline。r7 SurfaceDivider
    mod.widgets.ChromeMenuRule = SolidView{
        width: Fill
        height: 1
        margin: Inset{top: 2 bottom: 2}
        show_bg: true
        new_batch: true
        draw_bg.color: #x1d1d1d
    }

    // バー項目 — 裁定179。常時輪郭なし・素の文字。hover で面。技能 ButtonFlatter
    mod.widgets.ChromeMenuBar = ButtonFlatter{
        width: Fit
        height: 24
        padding: Inset{left: 8 right: 8}
        cursor: MouseCursor.Hand
        text: "File"
        draw_bg.color: #x00000000
        draw_bg.color_hover: #x464646
        draw_bg.color_down: #x242424
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #xb8b8b8
        draw_text.text_style: theme.font_regular{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
    }

    // 葉 — 動詞 + 右寄せ shortcut。ChromeMenuItem の面、ChromeMenuHint を再利用
    mod.widgets.ChromeMenuLeaf = ButtonFlat{
        width: Fill
        height: 20
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 8 right: 8}
        spacing: 8
        cursor: MouseCursor.Hand
        text: ""
        draw_bg.color: #x3e3e3e
        draw_bg.color_hover: #x464646
        draw_bg.color_down: #x242424
        draw_bg.color_disabled: #x3e3e3e
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        label := ChromeInk{text: "Item"}
        hint := ChromeMenuHint{text: "Cmd+N"}
    }

    // ドロップ — Studio DropDownFlat。popup は menu 面と同じ raised + 角丸
    mod.widgets.ChromeDrop = DropDownFlat{
        width: Fit
        height: 24
        padding: Inset{left: 8 right: 22}
        labels: ["Item"]
        selected_item: 0
        draw_bg.color: #x3e3e3e
        draw_bg.color_hover: #x464646
        draw_bg.color_down: #x242424
        draw_bg.color_focus: #x464646
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_bg.arrow_color: #xb8b8b8
        draw_text.color: #xb8b8b8
        draw_text.text_style: theme.font_regular{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
        popup_menu: PopupMenuFlat{
            width: 192
            height: Fit
            flow: Down
            padding: 2
            draw_bg.color: #x3e3e3e
            draw_bg.border_radius: 4.0
            draw_bg.border_size: 0.0
            menu_item: PopupMenuItem{
                width: Fill
                height: 20
                align: Align{y: 0.5}
                padding: Inset{left: 8 right: 8}
                draw_bg.color: #x3e3e3e
                draw_bg.color_hover: #x464646
                draw_bg.color_active: #x464646
                draw_bg.color_disabled: #x3e3e3e
                draw_bg.border_size: 0.0
                draw_bg.mark_color: #x3e3e3e
                draw_bg.mark_color_active: #xb8b8b8
                draw_text.color: #xb8b8b8
                draw_text.color_hover: #xb8b8b8
                draw_text.color_active: #xb8b8b8
                draw_text.color_disabled: #x757575
                draw_text.text_style: theme.font_regular{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
            }
        }
    }
}
