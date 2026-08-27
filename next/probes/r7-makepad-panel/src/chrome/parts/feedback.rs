//! empty state / tooltip 面 / badge / 進捗読取 / 状態帯。Document を持たない。
//!
//! ホスト向け型名:
//! `ChromeEmpty` / `ChromeTooltip` / `ChromeBadge` /
//! `ChromeProgressReadout` / `ChromeStatus`。
//!
//! 書き出し進捗のバーは `ChromeProgress`（`stepper.rs`）。針は `ChromePlayhead`
//! （`transport.rs`）。ここへ複製しない。ホストが E19 で
//! `ChromeProgressReadout` + `ChromeProgress` + `ChromePlayhead` を組む。
//! 理由印は `ChromeBadge`。新しい badge は置かない（T28）。
//!
//! `ScrollYView` は書かない。出典: `docs/reviews/2026-08-26-makepad-dock-panel-waves.md`
//! §2「Chrome / splash: ScrollYView 禁止（eval 白紙）」。
//!
//! 色・形の正本: 利用者添付の Live 12 Dark 実画面
//!   `assets/image-cf39df4e-cc7d-4299-9900-56934306be7e.png`（1024×554 から画素採取）。
//!   上バー #3d3d3d / パネル・空面・下帯 #4f4f4f / 窪み #282828 /
//!   明インク #b8b8b8 / 静インク #919191（"Drop Audio Effects Here" 実測）。
//!   この閉集合の外へ新色を置かない。
//! 形の言語: 面はベタ、角丸なし、枠線なし、影なし。状態帯は低いベタ面 + 小さい字。
//! tooltip の窓外重ねは Makepad `Tooltip`（技能 widgets / design-judgment overlay）。
//! この部品は content 面。iced は置かない。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // empty state — Live の空 Device 面。Q0: 効かない行を並べない。headline + hint だけ
    mod.widgets.ChromeEmpty = SolidView{
        width: Fill
        height: Fit
        flow: Down
        align: Align{x: 0.5}
        padding: 12
        spacing: 8
        show_bg: true
        new_batch: true
        draw_bg.color: #x4f4f4f
        headline := Label{
            width: Fit
            height: Fit
            text: "Nothing selected"
            draw_text.color: #xb8b8b8
            draw_text.text_style: theme.font_regular{font_size: 12}
        }
        hint := Label{
            width: Fit
            height: Fit
            text: "Select a layer to edit."
            draw_text.color: #x919191
            draw_text.text_style: theme.font_regular{font_size: 9}
        }
    }

    // tooltip 面 — 重ねの所有者は Makepad Tooltip。ここは content。
    // 枠線なし: 外皮と face を同色のベタ #282828 にして縁を消す（face は型合意のため残す）
    mod.widgets.ChromeTooltip = SolidView{
        width: Fit
        height: Fit
        padding: 0
        show_bg: true
        new_batch: true
        draw_bg.color: #x282828
        face := SolidView{
            width: Fit
            height: Fit
            padding: Inset{left: 8 right: 8 top: 4 bottom: 4}
            show_bg: true
            new_batch: true
            draw_bg.color: #x282828
            label := Label{
                width: Fit
                height: Fit
                text: "Tooltip"
                draw_text.color: #xb8b8b8
                draw_text.text_style: theme.font_regular{font_size: 9}
            }
        }
    }

    // badge — 文字+ベタ面（窪み #282828 に明字）。色だけに状態を預けない。read-only。micro 8
    mod.widgets.ChromeBadge = SolidView{
        width: Fit
        height: Fit
        padding: Inset{left: 4 right: 4 top: 2 bottom: 2}
        align: Align{y: 0.5}
        show_bg: true
        new_batch: true
        draw_bg.color: #x282828
        label := Label{
            width: Fit
            height: Fit
            text: "BADGE"
            draw_text.color: #xb8b8b8
            draw_text.text_style: theme.font_bold{font_size: 8}
        }
    }

    // 進捗読取 — E19 `123 / 300(41%)`。数字は明インク、区切りと率は静インク
    mod.widgets.ChromeProgressReadout = View{
        width: Fit
        height: Fit
        flow: Right
        align: Align{y: 0.5}
        spacing: 2
        new_batch: true
        done := ChromeInk{
            text: "0"
            draw_text.color: #xb8b8b8
            draw_text.text_style: theme.font_code{font_size: 11}
        }
        sep := ChromeInk{
            text: "/"
            draw_text.color: #x919191
            draw_text.text_style: theme.font_code{font_size: 11}
        }
        total := ChromeInk{
            text: "0"
            draw_text.color: #xb8b8b8
            draw_text.text_style: theme.font_code{font_size: 11}
        }
        pct := ChromeInk{
            text: "(0%)"
            draw_text.color: #x919191
            draw_text.text_style: theme.font_code{font_size: 11}
        }
    }

    // 状態帯 — Live 最下段の低いベタ面 #4f4f4f + 小さい字。T28 / ST-28 / E28 の一行。
    // 高 28。modal にしない。中身はホストが足す
    mod.widgets.ChromeStatus = SolidView{
        width: Fill
        height: 28
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 8 right: 8}
        spacing: 8
        show_bg: true
        new_batch: true
        draw_bg.color: #x4f4f4f
        label := ChromeInk{
            text: "Ready"
            draw_text.color: #xb8b8b8
            draw_text.text_style: theme.font_regular{font_size: 9}
        }
    }
}
