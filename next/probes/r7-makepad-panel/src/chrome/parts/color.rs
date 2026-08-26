//! 小さな色板と、見本+対象名の1行。面 = 色そのもの。Document を持たない。
//! 部品: `ChromeSwatch` / `ChromeColorField`。
//! ホスト向け: 上の2名を `mod.widgets` 代入別名として読む。`set_type_default()` は使わない。
//! `ScrollYView` は書かない（r7 splash eval 白紙。技能 layout の ScrollYView は頁本体向け。ここは Fit 原子）。
//! 出典: 利用者添付の Live 12 Dark 実画面
//!   `assets/image-cf39df4e-cc7d-4299-9900-56934306be7e.png`（1024×554 から画素サンプル）
//! 色（画像実測。記憶で埋めていない）:
//!   既定塗り = クリップシアン #02c1b2（Beat）。他クリップ色は上書き用:
//!   緑 #21ffa8 / 青 #8cc4ff / 赤 #fd3637 / 桃 #fc93a4 / 黄 #f9f47c / マゼンタ #d35197 / 橙 #bf9737
//!   Live のクリップ色見本はベタの小矩形（縁なし・角丸なし・影なし）。
//!   対象名は暗面（パネル #4f4f4f）に明字 #e4e4e4。寸法 `.sw` 10px は据え置き。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // 色板 — ベタの小矩形。縁なし・角丸なし。塗りはクリップ色（上書きして使う）
    // 同一 script_mod 内で mod.widgets 登録名は素の名前で見えない(eval エラー実測) → let 束縛
    let SwatchT = SolidView{
        width: 10
        height: 10
        show_bg: true
        new_batch: true        draw_bg.color: #x02c1b2
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
    }
    mod.widgets.ChromeSwatch = SwatchT

    // 見本+対象 — 暗面に明字。channel は ChromeScrub
    mod.widgets.ChromeColorField = View{
        width: Fit
        height: 20
        flow: Right
        align: Align{y: 0.5}
        spacing: 4
        new_batch: true
        swatch := SwatchT{}
        label := ChromeInk{
            text: "Fill"
            draw_text.color: #xe4e4e4
        }
    }
}
