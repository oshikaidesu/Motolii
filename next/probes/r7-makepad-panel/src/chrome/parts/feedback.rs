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
//! §2「Chrome / splash: ScrollYView 禁止（eval 白紙）」。技能 widgets は
//! ScrollYView を推奨するが、r7 splash eval では葉が落ちる。
//!
//! 色・寸法: `ui-scale-and-z.html` 候補B と r7 `panel.splash`。新色は置かない。
//! tooltip の窓外重ねは Makepad `Tooltip`（技能 widgets / design-judgment overlay）。
//! この部品は content 面。iced は置かない。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // empty state — Q0: 効かない行を並べない。headline + hint だけ
    mod.widgets.ChromeEmpty = SolidView{
        width: Fill
        height: Fit
        flow: Down
        align: Align{x: 0.5}
        padding: 12
        spacing: 8
        show_bg: true
        new_batch: true
        draw_bg.color: #x363636
        headline := Label{
            width: Fit
            height: Fit
            text: "Nothing selected"
            draw_text.color: #xb8b8b8
            draw_text.text_style: theme.font_regular{font_size: 12 line_spacing: 1.0 top_drop: 0.0}
        }
        hint := Label{
            width: Fit
            height: Fit
            text: "Select a layer to edit."
            draw_text.color: #x757575
            draw_text.text_style: theme.font_regular{font_size: 9 line_spacing: 1.0 top_drop: 0.0}
        }
    }

    // tooltip 面 — 重ねの所有者は Makepad Tooltip。ここは content（caption 9 / --app）
    mod.widgets.ChromeTooltip = SolidView{
        width: Fit
        height: Fit
        padding: 1
        show_bg: true
        new_batch: true
        draw_bg.color: #x1d1d1d
        face := SolidView{
            width: Fit
            height: Fit
            padding: Inset{left: 8 right: 8 top: 4 bottom: 4}
            show_bg: true
            new_batch: true
            draw_bg.color: #x242424
            label := Label{
                width: Fit
                height: Fit
                text: "Tooltip"
                draw_text.color: #xb8b8b8
                draw_text.text_style: theme.font_regular{font_size: 9 line_spacing: 1.0 top_drop: 0.0}
            }
        }
    }

    // badge — 文字+面。色だけに状態を預けない。read-only。micro 8
    mod.widgets.ChromeBadge = SolidView{
        width: Fit
        height: Fit
        padding: Inset{left: 4 right: 4 top: 2 bottom: 2}
        align: Align{y: 0.5}
        show_bg: true
        new_batch: true
        draw_bg.color: #x3e3e3e
        label := Label{
            width: Fit
            height: Fit
            text: "BADGE"
            draw_text.color: #x8c8c8c
            draw_text.text_style: theme.font_bold{font_size: 8 line_spacing: 1.0 top_drop: 0.0}
        }
    }

    // 進捗読取 — E19 `123 / 300(41%)`。accent は針だけ（S5c）。数字は ink
    mod.widgets.ChromeProgressReadout = View{
        width: Fit
        height: Fit
        flow: Right
        align: Align{y: 0.5}
        spacing: 2
        new_batch: true
        done := ChromeInk{
            text: "0"
            draw_text.text_style: theme.font_code{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
        }
        sep := ChromeInk{
            text: "/"
            draw_text.color: #x8c8c8c
            draw_text.text_style: theme.font_code{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
        }
        total := ChromeInk{
            text: "0"
            draw_text.text_style: theme.font_code{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
        }
        pct := ChromeInk{
            text: "(0%)"
            draw_text.color: #x8c8c8c
            draw_text.text_style: theme.font_code{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
        }
    }

    // 状態帯 — T28 / ST-28 / E28 の一行。高 28。modal にしない。中身はホストが足す
    mod.widgets.ChromeStatus = SolidView{
        width: Fill
        height: 28
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 8 right: 8}
        spacing: 8
        show_bg: true
        new_batch: true
        draw_bg.color: #x363636
        label := ChromeInk{
            text: "Ready"
        }
    }
}
