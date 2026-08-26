//! ホスト向け公開型: ChromeRow
//!
//! 出典: 利用者添付の Live Dark 実画面
//!   `assets/image-cf39df4e-cc7d-4299-9900-56934306be7e.png`（1024×554 から画素採取）
//! 形の言語（画像から目測）: 行は低く詰める。選択行の帯高 ≈ 本文フォントの 1.6 倍
//! → 本文 10px に対し行高 16。左右余白は最小（6）。角丸・枠線・影は無し。
//! 色（画像サンプル）: 行地はパネル #4f4f4f（選択・帯色は面差しで上書きする）。
//! ツリー行は ChromeTreeRow（触らない）。
//! 別名は代入。`set_type_default()` は使わない。`ScrollYView` は書かない。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // 行 — 横並び、縦中央。低く詰める
    mod.widgets.ChromeRow = SolidView{
        width: Fill
        height: 16
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 6 right: 6}
        spacing: 6
        show_bg: true
        new_batch: true
        draw_bg.color: #x4f4f4f
    }
}
