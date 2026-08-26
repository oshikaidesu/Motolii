//! ホスト向け公開型: ChromeRow
//!
//! 出典: Material Design Metrics & keylines https://m1.material.io/layout/metrics-keylines.html
//! 行。Document を持たない。ツリー行は ChromeTreeRow（触らない）。
//! 色・寸法: `ui-scale-and-z.html` 候補B `--row` 20 / `--sp4` 8 / `--panel`。新色は置かない。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // 行 — mock --row。横並び、縦中央。padding / spacing は --sp4
    mod.widgets.ChromeRow = SolidView{
        width: Fill
        height: 20
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 8 right: 8}
        spacing: 8
        show_bg: true
        new_batch: true
        draw_bg.color: #x363636
    }
}
