//! ホスト向け公開型: ChromeRule
//!
//! 出典: 利用者添付の Live Dark 実画面
//!   `assets/image-cf39df4e-cc7d-4299-9900-56934306be7e.png`（1024×554 から画素採取）
//! 形の言語（画像から目測）: 区切りは 1px の暗線か面の明度差だけ。太い枠線を描かない。
//! まず余白と明度差、必要なときだけこの 1px。
//! 色（画像サンプル）: トラック区切り・パネル継ぎ目 #2d2d2d。
//! 別名は代入。`set_type_default()` は使わない。`ScrollYView` は書かない。
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
        draw_bg.color: #x2d2d2d
    }
}
