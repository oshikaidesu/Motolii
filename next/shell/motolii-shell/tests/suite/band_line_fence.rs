//! 線化 D5 ORACLE(裁定179 文法1、`docs/reviews/2026-08-22-chrome-grammar-audit.md`):
//! shell chrome の帯(header 帯・status 帯が共有する
//! [`motolii_shell::band_chrome_style`])の常時輪郭を廃止する — 帯は
//! `surface_panel` の面が app 地から明度1段浮くことが輪郭で、border は
//! **透明のまま幅だけ残す**(幾何不変)。`chip_outline_fence`(browser-pane、
//! D4)と同型の style 関数レベルの機械照合。

use motolii_shell::band_chrome_style;
use motolii_shell::tokens::{Colors, Dimensions};

/// **本命(D5)**: 帯の border は透明・幅は `dims.border_width` のまま
/// (幾何不変)。
#[test]
fn band_border_is_transparent_but_keeps_its_width() {
    let dims = Dimensions::default();
    let colors = Colors::default();
    let style = band_chrome_style(dims, colors);
    assert_eq!(
        style.border.color.a, 0.0,
        "帯の border が透明でない(常時輪郭 D5 が残存): {:?}",
        style.border.color
    );
    assert_eq!(
        style.border.width, dims.border_width,
        "帯の border 幅が dims.border_width から動いた(幾何不変違反)"
    );
    assert_eq!(style.border.radius, 0.0.into(), "帯の角丸が動いた(幾何不変違反)");
}

/// 明度段は残る(消しすぎ防止 — S6「区切りが見えなくなった」は不合格):
/// 帯の面は `surface_panel`(app 地 `surface_app` より明度1段上)。status 帯は
/// 旧「border のみ・背景なし」からこの面へ移った — 帯が見えなくなっていない
/// ことをここで固定する。
#[test]
fn band_keeps_its_panel_face() {
    let dims = Dimensions::default();
    let colors = Colors::default();
    let style = band_chrome_style(dims, colors);
    assert_eq!(
        style.background,
        Some(iced::Background::Color(colors.surface_panel)),
        "帯の面が surface_panel でない(明度段が消えた)"
    );
}
