//! ホスト向け公開型: ChromeInk / ChromeInkTitle / ChromeInkCaption / ChromeInkMicro
//!
//! 出典: 利用者添付の Live Dark 実画面
//!   `assets/image-cf39df4e-cc7d-4299-9900-56934306be7e.png`（1024×554 から画素採取）
//! 対比根拠: WCAG 2.2 Contrast Minimum
//!   https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html
//! 文字階段（利用者添付 dark 画面から目測）: 本文 cap≈5px・注 cap≈4px（比 0.8）、
//! 見出しは本文の約 1.1 倍で僅差、微はさらに一段下。小さく密。
//!   → px 固定: Title 11 / 本文 10 / Caption 8 / Micro 7
//! 行の詰まり: 既定のまま(line_spacing の既定は 1.0 — 上書き指定は死んだ呪文だった、裁定272)
//! 色（画像サンプル。暗面に明字）: 見出し #d0d0d0（device タイトル芯）/
//! 本文 #c1c1c1（バー数値芯）/ 注 #9d9d9d（節見出し Categories）/
//! 微 #757575（Drop Audio Effects Here）。色だけに状態を預けない。
//! 別名は代入。`set_type_default()` は使わない。`ScrollYView` は書かない。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // 本文 — Live バー・行の基本字。既定
    mod.widgets.ChromeInk = Label{
        width: Fit
        height: Fit
        padding: 0
        draw_text.color: #xc1c1c1
        draw_text.text_style: theme.font_regular{font_size: 10}
    }

    // 見出し — Live device タイトル帯の字。本文との差は僅か（1.1 倍）
    mod.widgets.ChromeInkTitle = Label{
        width: Fit
        height: Fit
        padding: 0
        draw_text.color: #xd0d0d0
        draw_text.text_style: theme.font_regular{font_size: 11}
    }

    // 注 — Live 節見出し（Categories / Places）。本文の 0.8 倍
    mod.widgets.ChromeInkCaption = Label{
        width: Fit
        height: Fit
        padding: 0
        draw_text.color: #x9d9d9d
        draw_text.text_style: theme.font_regular{font_size: 8}
    }

    // 微 — Live mixer の極小数値・空域の案内字
    mod.widgets.ChromeInkMicro = Label{
        width: Fit
        height: Fit
        padding: 0
        draw_text.color: #x757575
        draw_text.text_style: theme.font_regular{font_size: 7}
    }
}
