//! ホスト向け公開型: ChromeStepper / ChromeProgress
//! ホスト向け: 上の2名を `mod.widgets` 代入別名として読む。`set_type_default()` は使わない。
//! 出典: 利用者添付の Ableton Live 12 Dark 実画面
//!   `assets/image-cf39df4e-cc7d-4299-9900-56934306be7e.png`（1024×554 から画素採取）
//!   上バーの Tap も窪みベタ矩形 #x282828 + 明文字 — 押せる欄も窪み語法。
//!   Device のノブ値 / dB 表示は面 #x4f4f4f 上の明グレー #xc9c9c9（窪みなし）。
//!   進捗は細い溝(暗 #x282828)+ 明るい塗り。枠線なし・角丸ゼロ・影なし。
//! +/- stepper と細い progress。Document を持たない。
//! 部品: `ChromeStepper` / `ChromeProgress`。
//! 中央の値は**欄**であって表示ではない(2026-08-27 台帳 E1)。+/− は同じ値への
//! second path で、擦る・打つが一次。ホストは `value.min` / `value.max` /
//! `value.step` / `value.default` を渡す — 範囲は部品の持ち物ではない。
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
    // minus / value / plus は :=。click は ButtonFlat、値は擦れる/打てる欄
    // ボタンは窪み #x282828 + 明グリフ(Tap の語法)。hover は上バー灰、down は表示黒。
    // 値も窪み(Live のテンポ欄の語法) — 触れる物だけが窪む。枠線は足さない
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
        // 値 — 表示ではなく欄(E1)。押して横へ擦れば値が動き、押して離せば
        // その場で打てる(1個の欄で両方)。機構は Slider の
        // FingerDown→drag / FingerUp→text_input.is_read_only(false)
        // (makepad-motolii `widgets/src/slider.rs` Slider::handle_event)。
        // 見た目は Live のテンポ欄と同じ窪み #x282828 + 明数字 — 触れる物だけが窪む。
        // 面色は `pixel: fn()` で作る。uniform の色は draw call 共有で、
        // 同じ shader を使う兄弟(ChromeProgress)と衝突するため上書きしない
        // (memory `makepad-surface-colors-are-uniform`, 2026-08-27 実測)。
        // 範囲はホストの持ち物: `value.min` / `value.max` / `value.step` /
        // `value.default` を必ず上書きする。既定 0..100 は「置いていない」の意
        value := SliderMinimalFlat{
            width: 48
            height: 24
            margin: 0.
            padding: Inset{left: 6 right: 6}
            align: Align{y: 0.5}
            min: 0.0
            max: 100.0
            step: 1.0
            default: 0.0
            precision: 0
            // 名札は空(単位はホストが隣へ置く)。**`label_walk` を 0 幅にしない** —
            // `Slider::draw_walk_slider` は flow が Right の時
            // `cx.defer_walk_turtle(label_walk)` が None を返すと数字ごと描かない。
            // 既定の `width: Fill` が deferred walk の条件なので触らず、
            // 名札は空文字で消す(2026-08-27 実窓で実測: 0 幅にすると欄が空になった)。
            // 結果として数字は欄の右端へ寄る — Live の値の欄と同じ
            text: ""
            draw_bg +: {
                pixel: fn() {
                    let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                    // 窪みは1枚。hover で一段、掴む/打つ間はもう一段明るく。
                    // 進捗の塗りは持たない — stepper の値は量ではなく数
                    let edit = max(self.focus, self.drag)
                    let well = mix(mix(#x282828, #x343434, self.hover), #x3d3d3d, edit)
                    sdf.rect(0.0, 0.0, self.rect_size.x, self.rect_size.y)
                    sdf.fill(well)
                    return sdf.result
                }
            }
            text_input.draw_text.color: #xd4d4d4
            text_input.draw_text.color_hover: #xd4d4d4
            text_input.draw_text.color_focus: #xd4d4d4
            text_input.draw_text.color_down: #xd4d4d4
            text_input.draw_text.color_disabled: #x8f8f8f
            text_input.draw_text.color_empty: #x8f8f8f
            text_input.draw_text.color_empty_hover: #x8f8f8f
            text_input.draw_text.color_empty_focus: #xd4d4d4
            text_input.draw_text.text_style: theme.font_code{font_size: 11}
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
