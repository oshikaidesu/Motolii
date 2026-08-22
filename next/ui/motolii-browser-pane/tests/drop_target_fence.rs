//! drop 先ハイライト(B08 続編)ORACLE — `container_line_fence.rs` と同型の
//! style 関数レベル照合。
//!
//! [`motolii_browser_pane::drop_target_style`] は `motolii-shell` の pane_grid
//! `hovered_region`(題帯レーン #3)と**同じ文法・同じロール**であること:
//! 面=`surface_hover`(drag 中に cursor が乗っている受け入れ面)+ 縁=
//! `focus`(操作が着地する場所の合図)× 太さ `border_width * 2.0`(強調線の
//! 既存導出)。S4: 新ロールを起こさない — 既存ロールの読み替えのみ。
//!
//! **テストは書くが実行しない**(裁定189 追いつきターンの規律)。

use motolii_browser_pane::{drop_target_style, panel_container_style};
use motolii_tokens_rs::{Colors, Dimensions};

/// **本命**: pane_grid の `hovered_region` と同じ3点(面・縁色・縁太さ)。
#[test]
fn drop_target_matches_the_pane_grid_hovered_region_grammar() {
    let dims = Dimensions::default();
    let colors = Colors::default();
    let style = drop_target_style(dims, colors);
    assert_eq!(
        style.background,
        Some(iced::Background::Color(colors.surface_hover)),
        "drop 面が surface_hover でない(pane_grid 文法との乖離)"
    );
    assert_eq!(
        style.border.color, colors.focus,
        "drop 縁が focus でない(着地の合図ロールとの乖離)"
    );
    assert_eq!(
        style.border.width,
        dims.border_width * 2.0,
        "drop 縁の太さが border_width×2 でない(強調線の既存導出との乖離)"
    );
    assert_eq!(style.border.radius, 0.0.into(), "drop 縁に角丸が生えた");
}

/// 非 drop 時の容器は従来どおり [`panel_container_style`](D5 線化のまま) —
/// drop ハイライトの導入が常時輪郭を復活させていない(裁定179 逆流防止)。
#[test]
fn the_default_container_grammar_is_untouched() {
    let dims = Dimensions::default();
    let colors = Colors::default();
    let style = panel_container_style(dims, colors);
    assert_eq!(style.border.color.a, 0.0, "非 drop 時に常時輪郭が復活");
    assert_eq!(
        style.background,
        Some(iced::Background::Color(colors.surface_panel)),
        "非 drop 時の面が surface_panel でない"
    );
}
