//! ホスト向け公開型: ChromeDisclosure / ChromeFold / ChromeTreeRow
//! 出典: 利用者添付の Live 12 Dark 実画面
//!   `assets/image-cf39df4e-cc7d-4299-9900-56934306be7e.png`（1024×554 から画素実測）
//! 形: 折り三角は小さい単色 glyph（Live Device fold ▼/▶。技能 FoldButton の
//! pixel が同形なので残す）、余白最小。見出しは枠線でなく面の明度差
//! （題帯 #646464 = Device 題帯実測、上に濃字 #252525）。
//! 色（画像サンプル）: 行地 #3d3d3d / hover 行地明 #4f4f4f / 明グリフ #a0a0a0→#efefef
//! disclosure / fold / 1行ツリー。Document を持たない。
//! 技能の `FoldButton` / `FoldHeader` を載せる。自前ドック・iced は置かない。
//! ScrollYView は書かない（eval 白紙）。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // disclosure — FoldButton。▼/▶ の単色三角。暗面上は明灰、hover で明字
    // 同一 script_mod 内で mod.widgets 登録名は素の名前で見えない(eval エラー実測)。
    // panel.splash と同じ let 束縛で持ち、登録は代入だけにする
    let DisclosureT = FoldButton{
        width: 12
        height: 20
        margin: 0.
        draw_bg.color: #xa0a0a0
        draw_bg.color_hover: #xefefef
        draw_bg.color_active: #xa0a0a0
    }
    mod.widgets.ChromeDisclosure = DisclosureT

    // fold — FoldHeader。題帯は明灰 #646464 のベタ面 + 濃字（Live Device 題帯）
    mod.widgets.ChromeFold = FoldHeader{
        width: Fill
        height: Fit
        flow: Down
        body_walk: Walk{width: Fill height: Fit}

        header: SolidView{
            width: Fill
            height: 20
            flow: Right
            align: Align{y: 0.5}
            padding: Inset{left: 4 right: 8}
            spacing: 4
            show_bg: true
            new_batch: true
            draw_bg.color: #x646464
            fold_button := DisclosureT{
                draw_bg.color: #x252525
                draw_bg.color_hover: #x282828
                draw_bg.color_active: #x252525
            }
            title := ChromeInk{
                text: "Section"
                draw_text.color: #x252525
            }
        }

        body: View{
            width: Fill
            height: Fit
            flow: Down
            padding: Inset{left: 8}
            new_batch: true
        }
    }

    // 1行ツリー — 行高 20。面の明度差だけで示す（#3d3d3d → hover #4f4f4f）。
    // indent 幅を深さに、fold_button を葉で隠す
    //
    // hover は **animator が `draw_bg.color` を差し替える**形で入れる(台帳 E3)。
    // 以前ここが落ちた原因は判っている: `draw_bg +:` へ `hover: instance(0.0)` を
    // 足したこと。GPU の layout はコンパイル時なので instance の追加は
    // "cannot push to frozen vec" で eval ごと落ちる(makepad-2.0-troubleshooting
    // Pitfall #34)。そして足せていない instance を `self.hover` と読んだのが
    // 「self 不在」の正体。**だから shader は触らない。**
    // `SolidView` の `draw_bg.color` は instance(`widgets/src/view_ui.rs`)なので
    // animator が直接動かせる。View が hit を拾う条件は
    // `cursor.is_some() || animator.is_defined`(`widgets/src/view.rs`)。
    // `down` 群は置かない — 無い群への `animator_play` は None を返す no-op
    // (`widgets/src/animator.rs` Animator::play)。
    // 反応は即時(利用者裁定 2026-08-27)。text が消えないよう new_batch は必須
    mod.widgets.ChromeTreeRow = SolidView{
        width: Fill
        height: 20
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{right: 8}
        spacing: 4
        show_bg: true
        new_batch: true
        cursor: MouseCursor.Hand
        draw_bg.color: #x3d3d3d
        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward{duration: 0.0}}
                    apply: {draw_bg: {color: #x3d3d3d}}
                }
                on: AnimatorState{
                    cursor: MouseCursor.Hand
                    from: {all: Forward{duration: 0.0}}
                    apply: {draw_bg: {color: #x4f4f4f}}
                }
            }
        }
        indent := View{width: 0 height: Fill}
        fold_button := DisclosureT{}
        title := ChromeInk{text: "Node"}
    }
}
