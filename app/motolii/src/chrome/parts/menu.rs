//! dropdown / context menu / menubar の見た目。
//! 閉集合: 面 / 項 / 注 / 線 / ドロップ / バー / 葉。Document を持たない。
//! 部品: `ChromeMenuFace` / `ChromeMenuItem` / `ChromeMenuHint` / `ChromeMenuRule` / `ChromeDrop` / `ChromeMenuBar` / `ChromeMenuLeaf`。
//! ホスト向け: 上の7名を `mod.widgets` 代入別名として読む。`set_type_default()` は使わない。
//! `ScrollYView` は書かない（r7 splash eval 白紙。技能 layout の ScrollYView は頁本体向け。ここは Fit 原子）。
//! 出典: 利用者添付の Live 12 Dark 実画面
//!   `assets/image-cf39df4e-cc7d-4299-9900-56934306be7e.png`（1024×554 から画素サンプル）
//! 色（画像実測）: 面 = 上バー #3d3d3d / 明字 #e4e4e4 / 副次字 #a0a0a0
//!   区切り 1px 暗線 #2a2a2a / 窪み（テンポ欄）#2a2a2a・窪み字 #f2f2f2 / 検索窪み #3c3c3c
//!   選択 = User Library 行ハイライト #6b8d96 + 濃字 #133342。
//! 形（Live の言語）: 暗面 + 明字 + 1px 区切り。角丸なし・枠線なし・影なし。
//! hover/down は静止画に写らないため、同画像の実測面 #4f4f4f / #2d2d2d を割り当て。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // 面 — 開いた menu。暗面のベタ。角丸なし・影なし。幅は menubar_menu_width
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
        draw_bg.color: #x3d3d3d
        draw_bg.border_radius: 0.0
        draw_bg.border_size: 0.0
    }

    // 項 — row_height 20。暗面に明字。hover はパネル面、down は暗
    mod.widgets.ChromeMenuItem = ButtonFlat{
        width: Fill
        height: 20
        align: Align{x: 0.0 y: 0.5}
        padding: Inset{left: 8 right: 8}        draw_bg.color: #x3d3d3d
        draw_bg.color_hover: #x4f4f4f
        draw_bg.color_down: #x2d2d2d
        draw_bg.color_disabled: #x3d3d3d
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #xe4e4e4
        draw_text.color_disabled: #xa0a0a0
        draw_text.text_style: theme.font_regular{font_size: 11}
    }

    // 注 — shortcut / muted。副次字
    // 同一 script_mod 内で mod.widgets 登録名は素の名前で見えない(eval エラー実測) → let 束縛
    let MenuHintT = Label{
        width: Fit
        height: Fit
        padding: 0
        draw_text.color: #xa0a0a0
        draw_text.text_style: theme.font_regular{font_size: 9}
    }
    mod.widgets.ChromeMenuHint = MenuHintT

    // 線 — menu 内 1px 暗線
    mod.widgets.ChromeMenuRule = SolidView{
        width: Fill
        height: 1
        margin: Inset{top: 2 bottom: 2}
        show_bg: true
        new_batch: true
        draw_bg.color: #x2a2a2a
    }

    // バー項目 — 常時輪郭なし・素の明字。hover で面。技能 ButtonFlatter
    mod.widgets.ChromeMenuBar = ButtonFlatter{
        width: Fit
        height: 24
        padding: Inset{left: 8 right: 8}        text: "File"
        draw_bg.color: #x00000000
        draw_bg.color_hover: #x4f4f4f
        draw_bg.color_down: #x2d2d2d
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #xe4e4e4
        draw_text.text_style: theme.font_regular{font_size: 11}
    }

    // 葉 — 動詞 + 右寄せ shortcut。ChromeMenuItem の面、ChromeMenuHint を再利用
    mod.widgets.ChromeMenuLeaf = ButtonFlat{
        width: Fill
        height: 20
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 8 right: 8}
        spacing: 8        text: ""
        draw_bg.color: #x3d3d3d
        draw_bg.color_hover: #x4f4f4f
        draw_bg.color_down: #x2d2d2d
        draw_bg.color_disabled: #x3d3d3d
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        label := ChromeInk{
            text: "Item"
            draw_text.color: #xe4e4e4
        }
        hint := MenuHintT{text: "Cmd+N"}
    }

    // ドロップ — Live の窪み欄（テンポ #2a2a2a・字 #f2f2f2）。角丸なし。
    // popup は menu と同じ暗面ベタ、選択行はハイライトのベタ塗り+濃字
    mod.widgets.ChromeDrop = DropDownFlat{
        width: Fit
        height: 24
        padding: Inset{left: 8 right: 22}
        labels: ["Item"]
        selected_item: 0
        draw_bg.color: #x2a2a2a
        draw_bg.color_hover: #x3c3c3c
        draw_bg.color_down: #x2a2a2a
        draw_bg.color_focus: #x3c3c3c
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_bg.arrow_color: #xa0a0a0
        draw_text.color: #xf2f2f2
        draw_text.text_style: theme.font_regular{font_size: 11}
        popup_menu: PopupMenuFlat{
            width: 192
            height: Fit
            flow: Down
            padding: 2
            draw_bg.color: #x3d3d3d
            draw_bg.border_radius: 0.0
            draw_bg.border_size: 0.0
            menu_item: PopupMenuItem{
                width: Fill
                height: 20
                align: Align{y: 0.5}
                padding: Inset{left: 8 right: 8}
                draw_bg.color: #x3d3d3d
                draw_bg.color_hover: #x4f4f4f
                draw_bg.color_active: #x6b8d96
                draw_bg.color_disabled: #x3d3d3d
                draw_bg.border_size: 0.0
                draw_bg.mark_color: #x3d3d3d
                draw_bg.mark_color_active: #x133342
                draw_text.color: #xe4e4e4
                draw_text.color_hover: #xe4e4e4
                draw_text.color_active: #x133342
                draw_text.color_disabled: #xa0a0a0
                draw_text.text_style: theme.font_regular{font_size: 11}
            }
        }
    }
}
