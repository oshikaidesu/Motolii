//! ホスト向け公開型: ChromeStepper / ChromeProgress
//! ホスト向け: 上の2名を `mod.widgets` 代入別名として読む。`set_type_default()` は使わない。
//! 出典: 利用者添付の Ableton Live 12 Dark 実画面
//!   `assets/image-cf39df4e-cc7d-4299-9900-56934306be7e.png`（1024×554 から画素採取）
//!   上バーの Tap も窪みベタ矩形 #x282828 + 明文字 — 押せる欄も窪み語法。
//!   Device のノブ値 / dB 表示は面 #x4f4f4f 上の明グレー #xc9c9c9（窪みなし）。
//!   進捗は細い溝(暗 #x282828)+ 明るい塗り。枠線なし・角丸ゼロ・影なし。
//! +/- stepper と細い progress。Document を持たない。
//! 部品: `ChromeStepper` / `ChromeProgress`。
//! 数値スライダーは `ChromeScrub`（`scrub.rs`）。ここへ複製しない。
//! `ScrollYView` は書かない。出典: `docs/reviews/2026-08-26-makepad-dock-panel-waves.md`
//! §2「Chrome / splash: ScrollYView 禁止（eval 白紙）」。技能 widgets は
//! ScrollYView を推奨するが、r7 splash eval では葉が落ちる。
//! 色（dark 画像実測のみ。記憶で埋めない）: 窪み #x282828 / 上バー #x3d3d3d /
//!   表示黒 #x141414 / ノブ値インク #xc9c9c9 / 数字 #xd4d4d4 / 補助 #x8f8f8f。
//! ChromeProgress の溝は `border_color` / `border_color_2` で塗る —
//! SliderMinimal の pixel は溝を上下2半分ともこの2色で描き、`color` は溝に
//! 使われない（makepad-motolii `widgets/src/slider.rs` SliderMinimal /
//! SliderMinimalFlat 定義）。既定のままだと theme の bevel 色が残る。
//! 目盛り 1px 線は SliderMinimalFlat に該当プロパティの出典が無く未適用。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // +/- stepper — 踏面 24（ChromeButton / interactive_target_min）、間隔 --sp1
    // minus / value / plus は :=。click は ButtonFlat、値は表示だけ
    // ボタンは窪み #x282828 + 明グリフ(Tap の語法)。hover は上バー灰、down は表示黒。
    // 値は面の上の明グレー(dB 表示の語法)。窪み・枠線を足さない
    mod.widgets.ChromeStepper = View{
        width: Fit
        height: 24
        flow: Right
        spacing: 2
        align: Align{y: 0.5}
        new_batch: true

        minus := ButtonFlat{
            width: 24
            height: 24
            padding: 0            text: "−"
            draw_bg.color: #x282828
            draw_bg.color_hover: #x3d3d3d
            draw_bg.color_down: #x141414
            draw_bg.color_focus: #x282828
            draw_bg.color_disabled: #x3d3d3d
            draw_bg.border_size: 0.0
            draw_bg.border_radius: 0.0
            draw_text.color: #xc9c9c9
            draw_text.color_hover: #xd4d4d4
            draw_text.color_down: #xd4d4d4
            draw_text.color_focus: #xc9c9c9
            draw_text.color_disabled: #x8f8f8f
            draw_text.text_style: theme.font_regular{font_size: 11}
        }
        value := Label{
            width: Fit
            height: Fit
            padding: Inset{left: 8 right: 8}
            text: "0"
            draw_text.color: #xc9c9c9
            draw_text.text_style: theme.font_regular{font_size: 11}
        }
        plus := ButtonFlat{
            width: 24
            height: 24
            padding: 0            text: "+"
            draw_bg.color: #x282828
            draw_bg.color_hover: #x3d3d3d
            draw_bg.color_down: #x141414
            draw_bg.color_focus: #x282828
            draw_bg.color_disabled: #x3d3d3d
            draw_bg.border_size: 0.0
            draw_bg.border_radius: 0.0
            draw_text.color: #xc9c9c9
            draw_text.color_hover: #xd4d4d4
            draw_text.color_down: #xd4d4d4
            draw_text.color_focus: #xc9c9c9
            draw_text.color_disabled: #x8f8f8f
            draw_text.text_style: theme.font_regular{font_size: 11}
        }
    }

    // 細い progress — 溝は暗 #x282828(全状態)、塗りは明 #xc9c9c9。
    // handle は 0。値は 0.0..=1.0。ChromeScrub と同系、高さだけ落とす
    mod.widgets.ChromeProgress = SliderMinimalFlat{
        width: Fill
        height: 2
        min: 0.0
        max: 1.0
        default: 0.0
        precision: 2
        text: ""
        draw_bg.color: #x282828
        draw_bg.color_hover: #x282828
        draw_bg.color_focus: #x282828
        draw_bg.color_drag: #x282828
        draw_bg.color_disabled: #x3d3d3d
        draw_bg.border_color: #x282828
        draw_bg.border_color_hover: #x282828
        draw_bg.border_color_focus: #x282828
        draw_bg.border_color_drag: #x282828
        draw_bg.border_color_disabled: #x3d3d3d
        draw_bg.border_color_2: #x282828
        draw_bg.border_color_2_hover: #x282828
        draw_bg.border_color_2_focus: #x282828
        draw_bg.border_color_2_drag: #x282828
        draw_bg.border_color_2_disabled: #x3d3d3d
        draw_bg.val_color: #xc9c9c9
        draw_bg.val_color_hover: #xc9c9c9
        draw_bg.val_color_focus: #xc9c9c9
        draw_bg.val_color_drag: #xc9c9c9
        draw_bg.val_color_disabled: #x8f8f8f
        draw_bg.handle_color: #xc9c9c9
        draw_bg.handle_size: 0.0
        draw_bg.border_size: 0.0
        draw_text.color: #xc9c9c9
        draw_text.text_style: theme.font_regular{font_size: 11}
    }
}
