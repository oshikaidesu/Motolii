//! 共通 chrome 部品 — tab / chip / rail。Document を持たない。
//! 部品: `ChromeTabStrip` / `ChromeTab` / `ChromeTabOn` / `ChromeChip` / `ChromeChipOn` / `ChromeChipStrip` / `ChromeRail` / `ChromeRailItem` / `ChromeRailItemOn`。
//! ホスト向け: 上の9名を `mod.widgets` 代入別名として読む。`set_type_default()` は使わない。
//! `ScrollYView` は書かない（r7 splash eval 白紙。技能 layout の ScrollYView は頁本体向け。ここは Fit 原子）。
//! 出典: 利用者添付の Live 12 Dark 実画面
//!   `assets/image-cf39df4e-cc7d-4299-9900-56934306be7e.png`（1024×554 から画素サンプル）
//! 色（画像実測）: パネル #4f4f4f / 上バー #3d3d3d / 行明字 #e4e4e4 / 見出し字 #a0a0a0
//!   選択 = 左 Browser「User Library」行の青系ハイライト #6b8d96 + 濃字 #133342。
//! 形（Live の言語）: 選択は行全体のベタ塗り+濃字。角丸なし・枠線なし・影なし。
//!   rail 行は低く詰める（画像の行ピッチに合わせ 16）。区切りは明度差だけ。
//! hover/down は静止画に写らないため、同画像の実測面 #5c5c5c / #2d2d2d を割り当て。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // tab strip — 上バー面 #3d3d3d
    mod.widgets.ChromeTabStrip = SolidView{
        width: Fill
        height: 26
        flow: Right
        align: Align{y: 0.5}
        spacing: 0
        show_bg: true
        new_batch: true
        draw_bg.color: #x3d3d3d
    }

    // tab — micro 8。idle はバー面に見出し字、hover はパネル面
    mod.widgets.ChromeTab = ButtonFlat{
        width: Fill
        height: 26
        padding: Inset{left: 8 right: 8}
        draw_bg.color: #x3d3d3d
        draw_bg.color_hover: #x4f4f4f
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #xa0a0a0
        draw_text.text_style: theme.font_regular{font_size: 8}
    }

    // tab on — 選択はベタの色面+濃字。mark も同色（行全体を一枚に塗る。下線を立てない）
    mod.widgets.ChromeTabOn = View{
        width: Fill
        height: 26
        flow: Down
        new_batch: true
        tab := ButtonFlat{
            width: Fill
            height: Fill
            padding: Inset{left: 8 right: 8}
            draw_bg.color: #x6b8d96
            draw_bg.color_hover: #x6b8d96
            draw_bg.border_size: 0.0
            draw_bg.border_radius: 0.0
            draw_text.color: #x133342
            draw_text.text_style: theme.font_regular{font_size: 8}
        }
        mark := SolidView{
            width: Fill
            height: 2
            show_bg: true
            new_batch: true
            draw_bg.color: #x6b8d96
        }
    }

    // chip — ベタ・角丸なし。idle はパネル面に見出し字
    mod.widgets.ChromeChip = ButtonFlat{
        width: Fit
        height: 17
        padding: Inset{left: 4 right: 4 top: 2 bottom: 2}
        draw_bg.color: #x4f4f4f
        draw_bg.color_hover: #x5c5c5c
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #xa0a0a0
        draw_text.text_style: theme.font_regular{font_size: 8}
    }

    // chip on — 選択ハイライトのベタ塗り+濃字。縁を引かない
    mod.widgets.ChromeChipOn = ButtonFlat{
        width: Fit
        height: 17
        padding: Inset{left: 4 right: 4 top: 2 bottom: 2}
        draw_bg.color: #x6b8d96
        draw_bg.color_hover: #x6b8d96
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #x133342
        draw_text.text_style: theme.font_regular{font_size: 8}
    }

    // chip strip — パネル面
    mod.widgets.ChromeChipStrip = SolidView{
        width: Fill
        height: 24
        flow: Right
        align: Align{y: 0.5}
        spacing: 2
        padding: Inset{left: 4 right: 4}
        show_bg: true
        new_batch: true
        draw_bg.color: #x4f4f4f
    }

    // rail — Browser カテゴリ列。パネル面。高さは Fit（Fill-in-Fit 0px を避ける）
    mod.widgets.ChromeRail = SolidView{
        width: 112
        height: Fit
        flow: Down
        padding: Inset{top: 2 bottom: 2}
        spacing: 0
        show_bg: true
        new_batch: true
        draw_bg.color: #x4f4f4f
    }

    // rail 行 — 低く詰める（16）。暗面に明字。区切りは明度差だけ（線を引かない）
    mod.widgets.ChromeRailItem = ButtonFlat{
        width: Fill
        height: 16
        padding: Inset{left: 8 right: 8}
        align: Align{x: 0.0 y: 0.5}
        draw_bg.color: #x4f4f4f
        draw_bg.color_hover: #x5c5c5c
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #xe4e4e4
        draw_text.text_style: theme.font_regular{font_size: 8}
    }

    // rail 行 on — User Library 行。行全体をハイライトでベタ塗り+濃字。
    // mark も同色に沈め、左バーを立てない（Live に左マークは無い）
    mod.widgets.ChromeRailItemOn = View{
        width: Fill
        height: 16
        flow: Right
        new_batch: true
        mark := SolidView{
            width: 2
            height: Fill
            show_bg: true
            new_batch: true
            draw_bg.color: #x6b8d96
        }
        item := ButtonFlat{
            width: Fill
            height: Fill
            padding: Inset{left: 6 right: 8}
            align: Align{x: 0.0 y: 0.5}
            draw_bg.color: #x6b8d96
            draw_bg.color_hover: #x6b8d96
            draw_bg.border_size: 0.0
            draw_bg.border_radius: 0.0
            draw_text.color: #x133342
            draw_text.text_style: theme.font_regular{font_size: 8}
        }
    }
}
