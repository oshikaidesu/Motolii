//! ホスト向け公開型: ChromeRule
//!
//! 出典: Material Design Structure https://m1.material.io/layout/structure.html
//! 線。Document を持たない。二次領域はまず余白。必要なときだけ 1px。
//! 色・寸法: `ui-scale-and-z.html` 候補B `--line`（物理 1px）と r7 `panel.splash` `#x1d1d1d`。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // 線 — 物理 1px。menu 内線は ChromeMenuRule（触らない）
    mod.widgets.ChromeRule = SolidView{
        width: Fill
        height: 1
        show_bg: true
        new_batch: true
        draw_bg.color: #x1d1d1d
    }
}
