//! ホスト向け公開型: LiveFace / LiveFaceBar / LiveFacePanel / LiveFaceDevice /
//! LiveFaceRecess / LiveFaceArea / LiveFaceHighlight / LiveInk / LiveInkPanel /
//! LiveInkDisabled
//!
//! 出典: 利用者添付の Live 12 実画面
//!   `assets/image-bd719b8e-8793-4d66-a7e9-1080b06b7deb.png`（1024×640 から画素）
//! 名称: Ableton Reference Manual Version 12
//!   https://www.ableton.com/en/live-manual/12/first-steps/
//! 色（画像サンプル。Theme ファイルは使わない）:
//!   上バー #818181 / パネル #a5a5a5 / Device #242424 / 窪み #131313
//!   バー・Device インク #a5a5a5 / パネルインク #818181 / 選択シアン #acc5ca
//! 別名は代入。`set_type_default()` は使わない。`ScrollYView` は書かない。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // 上バー面
    mod.widgets.LiveFaceBar = SolidView{
        width: Fill
        height: Fit
        show_bg: true
        new_batch: true
        draw_bg.color: #x818181
    }

    // Browser / Arrangement / Mixer パネル
    mod.widgets.LiveFacePanel = SolidView{
        width: Fill
        height: Fit
        show_bg: true
        new_batch: true
        draw_bg.color: #xa5a5a5
    }

    // Device View
    mod.widgets.LiveFaceDevice = SolidView{
        width: Fill
        height: Fit
        show_bg: true
        new_batch: true
        draw_bg.color: #x242424
    }

    // 窪み（テンポ / 小節位置）
    mod.widgets.LiveFaceRecess = SolidView{
        width: Fill
        height: Fit
        show_bg: true
        new_batch: true
        draw_bg.color: #x131313
    }

    // 窪み別名（旧名）
    mod.widgets.LiveFaceArea = SolidView{
        width: Fill
        height: Fit
        show_bg: true
        new_batch: true
        draw_bg.color: #x131313
    }

    // 選択シアン面
    mod.widgets.LiveFaceHighlight = SolidView{
        width: Fill
        height: Fit
        show_bg: true
        new_batch: true
        draw_bg.color: #xacc5ca
    }

    // 既定面 = パネル
    mod.widgets.LiveFace = SolidView{
        width: Fill
        height: Fit
        show_bg: true
        new_batch: true
        draw_bg.color: #xa5a5a5
    }

    // バー・Device 上の文字
    mod.widgets.LiveInk = Label{
        width: Fit
        height: Fit
        padding: 0
        draw_text.color: #xa5a5a5
        draw_text.text_style: theme.font_regular{font_size: 11}
    }

    // パネル上の文字
    mod.widgets.LiveInkPanel = Label{
        width: Fit
        height: Fit
        padding: 0
        draw_text.color: #x818181
        draw_text.text_style: theme.font_regular{font_size: 11}
    }

    mod.widgets.LiveInkDisabled = Label{
        width: Fit
        height: Fit
        padding: 0
        draw_text.color: #x818181
        draw_text.text_style: theme.font_regular{font_size: 9}
    }
}
