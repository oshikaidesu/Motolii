//! ホスト向け公開型: ChromeFace / ChromeFaceApp / ChromeFaceRaised
//!
//! 出典: 利用者添付の Live Dark 実画面
//!   `assets/image-cf39df4e-cc7d-4299-9900-56934306be7e.png`（1024×554 から画素採取）
//! 名称: Ableton Reference Manual Version 12
//!   https://www.ableton.com/en/live-manual/12/first-steps/
//! 形の言語（画像から目測）: 角丸ゼロの完全な矩形。影・グラデーション無し。
//! 区切りは枠線でなく面の明度差（窪み=一段暗い矩形、浮き=一段明るい矩形）。
//! 色（画像サンプル）: パネル #4f4f4f / 窪み・バー地 #282828 / 浮き帯 #646464。
//! 別名は代入。`set_type_default()` は使わない。`ScrollYView` は書かない。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // 面 — Live パネル（browser 側柱 / device 域 / arrangement 行地の最頻色）。
    // 既定。子を載せる。Fill-in-Fit 回避で height: Fit
    mod.widgets.ChromeFace = SolidView{
        width: Fill
        height: Fit
        show_bg: true
        new_batch: true
        draw_bg.color: #x4f4f4f
    }

    // 地 — Live 窪み（tempo 欄 / バー地）。入力・暗い入れ子。
    // 値の欄は枠線で囲まず、この一段暗い矩形だけで示す
    mod.widgets.ChromeFaceApp = SolidView{
        width: Fill
        height: Fit
        show_bg: true
        new_batch: true
        draw_bg.color: #x282828
    }

    // 浮き — Live device タイトル帯（パネルより一段明るい ident / 操作面）
    mod.widgets.ChromeFaceRaised = SolidView{
        width: Fill
        height: Fit
        show_bg: true
        new_batch: true
        draw_bg.color: #x646464
    }
}
