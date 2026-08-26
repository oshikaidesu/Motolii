//! ホスト向け公開型: ChromeSearch
//! ホスト向け: 上の1名を `mod.widgets` 代入別名として読む。`set_type_default()` は使わない。
//! 出典: Ableton Live 12 Browser search bar
//!   https://www.ableton.com/en/live-manual/12/working-with-the-browser/
//! 検索欄。Document を持たない。技能の `TextInputFlat` を載せる。iced は置かない。
//! `ScrollYView` は書かない。出典: `docs/reviews/2026-08-26-makepad-dock-panel-waves.md`
//! §2「Chrome / splash: ScrollYView 禁止（eval 白紙）」。技能 widgets は
//! ScrollYView を推奨するが、r7 splash eval では葉が落ちる。
//! 色・寸法: chrome 既存（面 --app #x242424 / インク #xb8b8b8 / 高さ 24）。
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // 検索 — TextInputFlat。面 --app、空は "Search"。高さ 24（interactive_target_min）
    // グリフボタンは置かない。Browser の IconFlatButton 検索は別席
    mod.widgets.ChromeSearch = TextInputFlat{
        width: Fill
        height: 24
        padding: Inset{left: 8 right: 8}
        empty_text: "Search"
        draw_bg.color: #x242424
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: #xb8b8b8
        draw_text.text_style: theme.font_regular{font_size: 11 line_spacing: 1.0 top_drop: 0.0}
    }
}
