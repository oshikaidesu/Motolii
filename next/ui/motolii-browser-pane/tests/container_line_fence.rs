//! 線化 D5 ORACLE(裁定179 文法1、`docs/reviews/2026-08-22-chrome-grammar-audit.md`):
//! rail/catalog 容器([`motolii_browser_pane::panel_container_style`] — 2つの
//! 容器が共有する1本)の常時輪郭を廃止する — 容器の輪郭は `surface_panel` の
//! 面が app 地から明度1段浮くことが担い、border は**透明のまま幅だけ残す**
//! (幾何不変)。`chip_outline_fence.rs`(D4)と同型の style 関数レベルの
//! 両側チェック。

use motolii_browser_pane::panel_container_style;
use motolii_tokens_rs::{Colors, Dimensions};

/// **本命(D5)**: 容器の border は透明・幅は `dims.border_width` のまま
/// (幾何不変)。
#[test]
fn container_border_is_transparent_but_keeps_its_width() {
    let dims = Dimensions::default();
    let colors = Colors::default();
    let style = panel_container_style(dims, colors);
    assert_eq!(
        style.border.color.a, 0.0,
        "容器の border が透明でない(常時輪郭 D5 が残存): {:?}",
        style.border.color
    );
    assert_eq!(
        style.border.width, dims.border_width,
        "容器の border 幅が dims.border_width から動いた(幾何不変違反)"
    );
    assert_eq!(style.border.radius, 0.0.into(), "容器の角丸が動いた(幾何不変違反)");
}

/// 明度段は残る(消しすぎ防止 — S6): 面は `surface_panel`(app 地
/// `surface_app` より明度1段上のパネル面ロール)。
#[test]
fn container_keeps_its_panel_face() {
    let dims = Dimensions::default();
    let colors = Colors::default();
    let style = panel_container_style(dims, colors);
    assert_eq!(
        style.background,
        Some(iced::Background::Color(colors.surface_panel)),
        "容器の面が surface_panel でない(明度段が消えた)"
    );
}
