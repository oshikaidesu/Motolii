//! 小さな色板と、見本+対象名の1行。面 = 色そのもの。Document を持たない。
//! 部品: `ChromeSwatch` / `ChromeColorField`。
//! ホスト向け: 上の2名を `mod.widgets` 代入別名として読む。`set_type_default()` は使わない。
//! `ScrollYView` は書かない（r7 splash eval 白紙。技能 layout の ScrollYView は頁本体向け。ここは Fit 原子）。
//! 出典: Ableton Live 12 Arrangement レーン色見本
//! https://www.ableton.com/en/manual/arrangement-view/
//! 寸法は `ui-scale-and-z.html` 候補B `.sw` 10px。役割は Inspector `color_row`
//! （面=色そのもの、箱ではない）。hex 欄は足さない（I22 草案・既存8-bit convention）。
//! 色・寸法: 候補B と r7 `panel.splash`。新色は置かない。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // 色板 — mock `.sw` 10px。縁は `--bd`。塗りは行スウォッチ（上書きして使う）
    mod.widgets.ChromeSwatch = SolidView{
        width: 10
        height: 10
        show_bg: true
        new_batch: true
        cursor: MouseCursor.Hand
        draw_bg.color: #x6f8fb5
        draw_bg.border_size: 1.0
        draw_bg.border_color: #x5b5b5b
        draw_bg.border_radius: 0.0
    }

    // 見本+対象 — Inspector Fill/Stroke・Settings 背景の共通粒。channel は ChromeScrub
    mod.widgets.ChromeColorField = View{
        width: Fit
        height: 20
        flow: Right
        align: Align{y: 0.5}
        spacing: 4
        new_batch: true
        swatch := ChromeSwatch{}
        label := ChromeInk{text: "Fill"}
    }
}
