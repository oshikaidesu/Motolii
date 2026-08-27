//! ホスト向け公開型: ChromeScrub
//! ホスト向け: 上の1名を `mod.widgets` 代入別名として読む。`set_type_default()` は使わない。
//! 出典: 利用者添付の Ableton Live 12 Dark 実画面
//!   `assets/image-cf39df4e-cc7d-4299-9900-56934306be7e.png`（1024×554 から画素採取）
//!   上バーのテンポ 120.00 / 位置 17.1.1 — 値の欄は周囲より一段暗い窪みベタ矩形
//!   #x282828 に明るい数字 #xd4d4d4。枠線なし・角丸ゼロ・影なし。
//!   フェーダーは溝(暗)+ 明るい摘み。色 fill は持たない。
//! 数値スライダー。Document を持たない。細いバーは `ChromeProgress`（`stepper.rs`）。
//! 技能の `SliderFlat` を載せる。iced は置かない。
//! `ScrollYView` は書かない。出典: `docs/reviews/2026-08-26-makepad-dock-panel-waves.md`
//! §2「Chrome / splash: ScrollYView 禁止（eval 白紙）」。技能 widgets は
//! ScrollYView を推奨するが、r7 splash eval では葉が落ちる。
//! 色（dark 画像実測のみ。記憶で埋めない）: 窪み #x282828 / 上バー #x3d3d3d /
//!   数字 #xd4d4d4 / ノブ値インク #xc9c9c9 / 補助 #x8f8f8f / 選択シアン #x6a8b9a。
//! hover / focus / drag も窪み色へ固定する — SliderFlat の既定は theme の inset 色へ
//! 飛ぶ（makepad-motolii `widgets/src/slider.rs` SliderFlat 定義）。フラットに反する。
//! 数字は `text_input.draw_text`（値は埋め込み TextInput が描く。同 slider.rs）。
//! 高さ 24 は chrome の interactive_target_min（Live の 16px は採らない。寸法 px は出典なし）。
//! 目盛り 1px 線は SliderFlat に該当プロパティの出典が無く未適用。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // 数値 — SliderFlat。窪み #x282828 全状態(枠線・角丸・影なし)。fill は窪み同色
    // (Live の値の欄は fill を持たない)。摘みは明 #xc9c9c9、focus/drag は選択シアン。
    // 数字は明 #xd4d4d4、ラベルはノブ値インク。disabled は上バー灰 + 補助インク
    mod.widgets.ChromeScrub = SliderFlat{
        width: Fill
        height: 24
        min: 0.0
        max: 100.0
        default: 50.0
        precision: 0
        text: "Value"        draw_bg.color: #x282828
        draw_bg.color_hover: #x282828
        draw_bg.color_focus: #x282828
        draw_bg.color_drag: #x282828
        draw_bg.color_disabled: #x3d3d3d
        draw_bg.val_color: #x282828
        draw_bg.val_color_hover: #x282828
        draw_bg.val_color_focus: #x282828
        draw_bg.val_color_drag: #x282828
        draw_bg.val_color_disabled: #x3d3d3d
        draw_bg.handle_color: #xc9c9c9
        draw_bg.handle_color_hover: #xd4d4d4
        draw_bg.handle_color_focus: #x6a8b9a
        draw_bg.handle_color_drag: #x6a8b9a
        draw_bg.handle_color_disabled: #x8f8f8f
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #xc9c9c9
        draw_text.color_hover: #xc9c9c9
        draw_text.color_focus: #xc9c9c9
        draw_text.color_drag: #xc9c9c9
        draw_text.color_disabled: #x8f8f8f
        draw_text.text_style: theme.font_regular{font_size: 11}
        text_input.draw_text.color: #xd4d4d4
        text_input.draw_text.color_hover: #xd4d4d4
        text_input.draw_text.color_focus: #xd4d4d4
        text_input.draw_text.color_down: #xd4d4d4
        text_input.draw_text.color_disabled: #x8f8f8f
        text_input.draw_text.color_empty: #x8f8f8f
        text_input.draw_text.color_empty_hover: #x8f8f8f
        text_input.draw_text.color_empty_focus: #xd4d4d4
    }
}
