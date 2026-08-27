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
//!   対象名は暗面（パネル #4f4f4f）に明字 #e4e4e4。**色の面は 10px 据え置き** —
//!   周りに hover 枠 2px が付くので部品の踏面は 14px になる。
//! 塗りの上書き先は `fill`: `ChromeSwatch{fill.draw_bg.color: #x21ffa8}` /
//!   `ChromeColorField{swatch.fill.draw_bg.color: …}`。外枠は hover 専用で、
//!   ここへクリップ色を入れない（色が hover で変わる嘘になる）。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // 色板 — ベタの小矩形。縁なし・角丸なし。塗りはクリップ色（上書きして使う）
    // 同一 script_mod 内で mod.widgets 登録名は素の名前で見えない(eval エラー実測) → let 束縛
    //
    // 構造は「hover を語る枠」+「色そのものの面」の2枚として実装済み
    // (下の Animator に hover/down の色が残っている)。だが**ピッカー本体が
    // リポのどこにも無い**(main.rs / chrome/*.rs のどこにも ChromeSwatch の
    // クリック受け口が無い、2026-08-28 統合で確認)。押しても開く物が無いのに
    // hover 枠と Hand カーソルだけ出すのは「押せる」と名乗って何も起きない
    // 虚報 — 実装しただけで見た目まで有効化するのは Q0 に対して後退、という
    // のが裁定(a)(2026-08-27/28)の一般則。ピッカー本体が発注・着地するまで
    // (Q6, 台帳 §4)、この2つの出口(cursor と hover/down の色)だけ塞ぐ。
    //   - 塗りは個体ごとに違う(クリップ色)。`SolidView` の `draw_bg.color` は
    //     instance なので個体で効く。ここへ hover を混ぜると色が嘘になるので混ぜない
    //   - hover / 押下は外枠 2px の色で語る想定だった。こちらは全個体で同じ
    //     literal なので uniform でも draw call 共有でも壊れない
    //     (memory `makepad-surface-colors-are-uniform`, 2026-08-27 実測)
    // 面に hover の instance を足す道は塞がっている(`draw_bg +:` への instance 追加は
    // "cannot push to frozen vec" で eval が落ちる)。だから animator が色を差し替える形は
    // そのまま残す — ピッカーが着地したら on 状態の色を #xa0a0a0 / #xefefef へ戻し、
    // cursor: MouseCursor.Hand を復活させるだけで良いように。
    let SwatchT = SolidView{
        width: 14
        height: 14
        padding: 2
        show_bg: true
        new_batch: true
        draw_bg.color: #x00000000
        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward{duration: 0.0}}
                    apply: {draw_bg: {color: #x00000000}}
                }
                on: AnimatorState{
                    from: {all: Forward{duration: 0.0}}
                    apply: {draw_bg: {color: #x00000000}}
                }
            }
            down: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward{duration: 0.0}}
                    apply: {draw_bg: {color: #x00000000}}
                }
                on: AnimatorState{
                    from: {all: Forward{duration: 0.0}}
                    apply: {draw_bg: {color: #x00000000}}
                }
            }
        }
        fill := SolidView{
            width: Fill
            height: Fill
            show_bg: true
            new_batch: true
            draw_bg.color: #x02c1b2
            draw_bg.border_size: 0.0
            draw_bg.border_radius: 0.0
        }
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
