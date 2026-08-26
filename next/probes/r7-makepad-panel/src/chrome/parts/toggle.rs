//! ホスト向け公開型: ChromeCheck / ChromeToggle / ChromeLock
//! 出典: 利用者添付の Live 12 Dark 実画面
//!   `assets/image-cf39df4e-cc7d-4299-9900-56934306be7e.png`（1024×554 から画素実測）
//! 形: 矩形のベタ面。枠線で囲まず面の明度差で領域を示す。状態は面色の切替だけ
//! （活性=色付き面に濃字 #252525、非活性=暗面 #323232 に灰字 #505050）。
//! 状態色（画像サンプル）:
//!   Check 活性 = ミキサー Track Activator「1」の明灰面 #a0a0a0（(950,74) 付近）
//!   Toggle 活性 = 上バー / Device「RMS」の活性橙 #xe89b3f（(157,445) 付近）
//!   Lock 押下 = ソロ「S」の青 #x2b7ad0（(969,177) 付近）
//! checkbox / toggle / lock。Document を持たない。
//! 技能の `CheckBoxFlat` / `ToggleFlat` / `ButtonFlatIcon` を載せる。iced は置かない。
//! ScrollYView は書かない（eval 白紙）。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // checkbox — CheckBoxFlat。箱は窪み #282828、活性で明灰面 + 濃印。枠線ゼロ、行高 20
    mod.widgets.ChromeCheck = CheckBoxFlat{
        width: Fit
        height: 20
        padding: 0
        align: Align{y: 0.5}        text: "Check"
        label_walk: Walk{width: Fit height: Fit margin: Inset{left: 16}}
        draw_bg.size: 12.0
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_bg.color: #x282828
        draw_bg.color_hover: #x343434
        draw_bg.color_down: #x282828
        draw_bg.color_active: #xa0a0a0
        draw_bg.color_focus: #x343434
        draw_bg.color_disabled: #x323232
        draw_bg.mark_color: #x00000000
        draw_bg.mark_color_hover: #x00000000
        draw_bg.mark_color_down: #x00000000
        draw_bg.mark_color_active: #x252525
        draw_bg.mark_color_active_hover: #x252525
        draw_bg.mark_color_focus: #xefefef
        draw_bg.mark_color_disabled: #x505050
        draw_text.color: #xa0a0a0
        draw_text.color_hover: #xefefef
        draw_text.color_down: #xa0a0a0
        draw_text.color_active: #xefefef
        draw_text.color_focus: #xefefef
        draw_text.color_disabled: #x505050
        draw_text.text_style: theme.font_regular{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
    }

    // toggle — ToggleFlat。槽は窪み #282828、活性で橙面 + 濃摘み。枠線ゼロ
    mod.widgets.ChromeToggle = ToggleFlat{
        width: Fit
        height: 20
        padding: 0
        align: Align{y: 0.5}        text: "On"
        label_walk: Walk{width: Fit height: Fit margin: Inset{left: 28}}
        draw_bg.size: 12.0
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_bg.color: #x282828
        draw_bg.color_hover: #x343434
        draw_bg.color_down: #x282828
        draw_bg.color_active: #xe89b3f
        draw_bg.color_focus: #x343434
        draw_bg.color_disabled: #x323232
        draw_bg.mark_color: #xa0a0a0
        draw_bg.mark_color_hover: #xefefef
        draw_bg.mark_color_down: #xa0a0a0
        draw_bg.mark_color_active: #x252525
        draw_bg.mark_color_active_hover: #x252525
        draw_bg.mark_color_focus: #xefefef
        draw_bg.mark_color_disabled: #x505050
        draw_text.color: #xa0a0a0
        draw_text.color_hover: #xefefef
        draw_text.color_down: #xa0a0a0
        draw_text.color_active: #xefefef
        draw_text.color_focus: #xefefef
        draw_text.color_disabled: #x505050
        draw_text.text_style: theme.font_regular{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
    }

    // lock — ButtonFlatIcon + 既存 lock.svg。押下保持はソロ青面。踏面 24、グリフ 13
    mod.widgets.ChromeLock = ButtonFlatIcon{
        width: 24
        height: 24
        padding: 0
        margin: 0.
        spacing: 0
        align: Center        text: ""
        icon_walk: Walk{width: 13 height: 13}
        draw_bg.color: #x3d3d3d
        draw_bg.color_hover: #x4f4f4f
        draw_bg.color_down: #x2b7ad0
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_icon +: {svg: crate_resource("self://resources/icons/lock.svg") color: #xefefef}
    }
}
