//! ホスト向け公開型: ChromeDisclosure / ChromeFold / ChromeTreeRow
//! 出典: After Effects twirl-down property groups — https://helpx.adobe.com/after-effects/desktop/work-with-layers/layer-properties/layer-properties.html
//! disclosure / fold / 1行ツリー。Document を持たない。
//! 部品: `ChromeDisclosure` / `ChromeFold` / `ChromeTreeRow`。
//! 技能の `FoldButton` / `FoldHeader` を載せる。自前ドック・iced は置かない。
//! ScrollYView は書かない（eval 白紙）。
//! 色・寸法: chrome 既存（面 #x363636 / 見出し #x2f2f2f / インク #xb8b8b8 / 行高 20）。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // disclosure — FoldButton。三角は技能の pixel を残し、色だけ chrome インクへ寄せる
    mod.widgets.ChromeDisclosure = FoldButton{
        width: 15
        height: 20
        margin: 0.
        draw_bg.color: #xb8b8b8
        draw_bg.color_hover: #xcfcfcf
        draw_bg.color_active: #xb8b8b8
    }

    // fold — FoldHeader。header/body は live 枠。開閉は header 内の fold_button
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
            padding: Inset{left: 8 right: 8}
            spacing: 8
            show_bg: true
            new_batch: true
            draw_bg.color: #x2f2f2f
            fold_button := ChromeDisclosure{}
            title := ChromeInk{text: "Section"}
        }

        body: View{
            width: Fill
            height: Fit
            flow: Down
            padding: Inset{left: 8}
            new_batch: true
        }
    }

    // 1行ツリー — 行高 20。indent 幅を深さに、fold_button を葉で隠す
    mod.widgets.ChromeTreeRow = SolidView{
        width: Fill
        height: 20
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{right: 8}
        spacing: 8
        show_bg: true
        new_batch: true
        cursor: MouseCursor.Hand
        draw_bg +: {
            instance hover: 0.0
            color: mix(#x363636, #x464646, self.hover)
        }
        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_bg: {hover: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_bg: {hover: 1.0}}
                }
            }
        }
        indent := View{width: 0 height: Fill}
        fold_button := ChromeDisclosure{}
        title := ChromeInk{text: "Node"}
    }
}
