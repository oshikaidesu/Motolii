//! 線化 D5 ORACLE(裁定179 文法1、`docs/reviews/2026-08-22-chrome-grammar-audit.md`):
//! Inspector のプロパティ行罫線(`.cols`/`.prow`/Blend/Speed 行が共有する
//! [`motolii_inspector_pane::row_band_style`])を廃止する — 行の区切りは固定
//! 行高+間隔が担い、border は**透明のまま幅だけ残す**(幾何不変)。
//! `chip_outline_fence.rs`(browser-pane、D4)と同型の style 関数レベルの
//! 機械照合。

use motolii_inspector_pane::row_band_style;
use motolii_tokens_rs::Dimensions;

/// **本命(D5)**: 行の border は透明・幅は `dims.border_width` のまま
/// (幾何不変)・面は塗らない(行はゼブラも面も持たない — 区切りは間隔)。
#[test]
fn row_rule_is_transparent_but_keeps_its_width() {
    let dims = Dimensions::default();
    let style = row_band_style(dims);
    assert_eq!(
        style.border.color.a, 0.0,
        "行罫線が透明でない(プロパティ行罫線 D5 が残存): {:?}",
        style.border.color
    );
    assert_eq!(
        style.border.width, dims.border_width,
        "行の border 幅が dims.border_width から動いた(幾何不変違反)"
    );
    assert_eq!(style.border.radius, 0.0.into(), "行の角丸が動いた(幾何不変違反)");
    assert!(
        style.background.is_none(),
        "行に面が乗った(素の行でない — 面の発明は D5 の範囲外): {:?}",
        style.background
    );
}
