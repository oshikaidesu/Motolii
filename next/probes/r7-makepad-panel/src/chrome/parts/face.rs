//! ホスト向け公開型: ChromeFace / ChromeFaceApp / ChromeFaceRaised
//!
//! 出典: Ableton Live https://www.ableton.com/en/live-manual/12/first-steps/
//! 面。Document を持たない。二次領域は余白で分ける（線を増やさない）。
//! 色・寸法: `ui-scale-and-z.html` 候補B `--app` / `--panel` / `--raised`。新色は置かない。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // 面 — mock --panel。既定。子を載せる。Fill-in-Fit 回避で height: Fit
    mod.widgets.ChromeFace = SolidView{
        width: Fill
        height: Fit
        show_bg: true
        new_batch: true
        draw_bg.color: #x363636
    }

    // 地 — mock --app。入力・暗い入れ子
    mod.widgets.ChromeFaceApp = SolidView{
        width: Fill
        height: Fit
        show_bg: true
        new_batch: true
        draw_bg.color: #x242424
    }

    // 浮き — mock --raised。ident / 操作面
    mod.widgets.ChromeFaceRaised = SolidView{
        width: Fill
        height: Fit
        show_bg: true
        new_batch: true
        draw_bg.color: #x3e3e3e
    }
}
