//! ホスト向け公開型: ChromeSearch
//! ホスト向け: 上の1名を `mod.widgets` 代入別名として読む。`set_type_default()` は使わない。
//! 出典: 利用者添付の Ableton Live 12 Dark 実画面
//!   `assets/image-cf39df4e-cc7d-4299-9900-56934306be7e.png`（1024×554 から画素採取）
//!   Browser 左上の検索欄 — 窪みベタ矩形に明文字。虫眼鏡なし・枠線なし・
//!   角丸ゼロ・影なし。飾りを足さない。
//! 検索欄。Document を持たない。技能の `TextInputFlat` を載せる。iced は置かない。
//! `ScrollYView` は書かない。出典: `docs/reviews/2026-08-26-makepad-dock-panel-waves.md`
//! §2「Chrome / splash: ScrollYView 禁止（eval 白紙）」。技能 widgets は
//! ScrollYView を推奨するが、r7 splash eval では葉が落ちる。
//! 色（dark 画像実測のみ。記憶で埋めない）: 窪み #x282828 / 入力インク #xd4d4d4 /
//!   placeholder #x8f8f8f（画像の "Search (Cmd + F)" 文字から採取）。
//! hover / focus / empty も窪み色へ固定する — TextInputFlat の既定は theme の
//! inset 色へ飛ぶ（makepad-motolii `widgets/src/text_input.rs`）。フラットに反する。
//! 高さ 24 は chrome の interactive_target_min（寸法 px は出典なし）。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // 検索 — TextInputFlat。窪み #x282828 全状態、空は "Search"。高さ 24
    // （interactive_target_min）。グリフボタンは置かない。Browser の
    // IconFlatButton 検索は別席
    mod.widgets.ChromeSearch = TextInputFlat{
        width: Fill
        height: 24
        padding: Inset{left: 8 right: 8}
        empty_text: "Search"
        draw_bg.color: #x282828
        draw_bg.color_hover: #x282828
        draw_bg.color_focus: #x282828
        draw_bg.color_down: #x282828
        draw_bg.color_empty: #x282828
        draw_bg.color_disabled: #x3d3d3d
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #xd4d4d4
        draw_text.color_hover: #xd4d4d4
        draw_text.color_focus: #xd4d4d4
        draw_text.color_down: #xd4d4d4
        draw_text.color_disabled: #x8f8f8f
        draw_text.color_empty: #x8f8f8f
        draw_text.color_empty_hover: #x8f8f8f
        draw_text.color_empty_focus: #x8f8f8f
        draw_text.text_style: theme.font_regular{font_size: 11}
    }
}
