//! ホスト向け公開型: ChromeInk / ChromeInkTitle / ChromeInkCaption / ChromeInkMicro
//!
//! 出典: WCAG 2.2 Contrast Minimum https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html
//! 文字。Document を持たない。色だけに状態を預けない。
//! 色・寸法: `ui-scale-and-z.html` 候補B 正典バンド {12,11,9,8} と `--ink` / `--ink2` / `--ink3`。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // 本文 — mock --t-base 11 / --ink。既定
    mod.widgets.ChromeInk = Label{
        width: Fit
        height: Fit
        padding: 0
        draw_text.color: #xb8b8b8
        draw_text.text_style: theme.font_regular{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
    }

    // 見出し — mock --t-title 12 / --ink
    mod.widgets.ChromeInkTitle = Label{
        width: Fit
        height: Fit
        padding: 0
        draw_text.color: #xb8b8b8
        draw_text.text_style: theme.font_regular{font_size: 12 line_spacing: 1.0 top_drop: 0.0}
    }

    // 注 — mock --t-dense 9 / --ink2
    mod.widgets.ChromeInkCaption = Label{
        width: Fit
        height: Fit
        padding: 0
        draw_text.color: #x8c8c8c
        draw_text.text_style: theme.font_regular{font_size: 9 line_spacing: 1.0 top_drop: 0.0}
    }

    // 微 — mock --t-micro 8 / --ink3
    mod.widgets.ChromeInkMicro = Label{
        width: Fit
        height: Fit
        padding: 0
        draw_text.color: #x757575
        draw_text.text_style: theme.font_regular{font_size: 8 line_spacing: 1.0 top_drop: 0.0}
    }
}
