//! `text` module(裁定190 切片1)の受入条件。`next/probes/r6-text-shaping` の
//! 合格線をそのまま product 試験へ昇格した(probe の verdict は緑、
//! `docs/reviews/2026-08-22-text-rendering-route-probe.md` 参照)。
//!
//! フォントは macOS 実機の Arial / ヒラギノ角ゴシック(path 解決)。この試験は
//! この機械で回す前提(r3 が実 PLY を前提にするのと同じ、probe の module doc 参照)。

use motolii_vector::text::{shape_text, GlyphFont, TextFeature, TextJustify, TextLayout};
use motolii_vector::{render, Brush, Canvas, Fill, FillRule, PathSource, Raster, Rgb, Shape};

const ARIAL: &str = "/System/Library/Fonts/Supplemental/Arial.ttf";
const HIRAGINO: &str = "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc";

fn arial() -> GlyphFont {
    GlyphFont {
        path: ARIAL.to_owned(),
        family: "Arial".to_owned(),
    }
}

fn hiragino() -> GlyphFont {
    GlyphFont {
        path: HIRAGINO.to_owned(),
        family: "Hiragino Sans".to_owned(),
    }
}

fn layout(size: f32, line_height: Option<f32>, tracking: f32) -> TextLayout {
    TextLayout {
        size,
        line_height,
        tracking,
        justify: TextJustify::Left,
        features: Vec::new(),
    }
}

/// 輪郭を赤の fill で塗った `Shape`(fill 色が画素まで届いた事の検証にも使う)。
fn red_shape(contours: Vec<motolii_vector::Contour>) -> Shape {
    let mut shape = Shape::new(PathSource::Bezier(contours));
    shape.fill = Some(Fill {
        brush: Brush::Solid(Rgb {
            r: 1.0,
            g: 0.0,
            b: 0.0,
        }),
        rule: FillRule::NonZero,
        opacity: 1.0,
        hidden: false,
    });
    shape
}

fn visible_pixels(raster: &Raster) -> usize {
    raster
        .premultiplied_rgba8
        .chunks_exact(4)
        .filter(|p| p[3] > 0)
        .count()
}

/// 合格線1: 文字列 → 輪郭 → raster の1本道。
#[test]
fn a_string_becomes_outlines_then_pixels() {
    let shaped = shape_text("Motolii", &arial(), &layout(96.0, None, 0.0)).expect("shape");

    // "Motolii" は 7 glyph。o の中マド・i の点で輪郭は glyph 数より多い。
    assert!(
        shaped.contours.len() >= 8,
        "輪郭が少なすぎる: {}(shaping か outline 抽出が落ちている)",
        shaped.contours.len()
    );
    assert!(
        shaped.contours.iter().all(|c| c.closed),
        "フォント輪郭は全て閉じているはず"
    );

    let canvas = Canvas {
        width: 512,
        height: 160,
        origin_x: 0,
        origin_y: 0,
    };
    let raster = render(&red_shape(shaped.contours), &canvas).expect("render");
    let visible = visible_pixels(&raster);
    assert!(
        visible > 2_000,
        "可視画素 {visible} — 96px の 7 文字なら数千画素は塗られるはず"
    );
    // fill [1,0,0,1](赤)が画素まで届いている(premultiplied なので r=a の画素がある)。
    assert!(
        raster
            .premultiplied_rgba8
            .chunks_exact(4)
            .any(|p| p[0] > 200 && p[1] == 0 && p[2] == 0),
        "赤の fill が画素へ届いていない"
    );
}

/// 合格線2: lh は baseline 差そのもの。90 → 90、140 → 140(±0.01)。
#[test]
fn line_height_is_the_baseline_delta_numerically() {
    for lh in [90.0_f32, 140.0] {
        let shaped =
            shape_text("MO\nLII", &arial(), &layout(64.0, Some(lh), 0.0)).expect("shape");
        assert_eq!(shaped.lines.len(), 2, "2行になるはず");
        let delta = shaped.lines[1].baseline_y - shaped.lines[0].baseline_y;
        assert!(
            (delta - lh).abs() < 0.01,
            "lh={lh} を指定したのに baseline 差が {delta}"
        );
    }
}

/// 合格線3: tr は glyph ペン位置の差へ tr/1000 em で写る。
/// size=100 なら tr=250 → 1 glyph あたり +25px(±0.05)。
#[test]
fn tracking_widens_glyph_advances_numerically() {
    let size = 100.0_f32;
    let tracking = 250.0_f32;
    let plain = shape_text("III", &arial(), &layout(size, None, 0.0)).expect("shape");
    let tracked = shape_text("III", &arial(), &layout(size, None, tracking)).expect("shape");

    let xs0 = &plain.lines[0].glyph_xs;
    let xs1 = &tracked.lines[0].glyph_xs;
    assert_eq!(xs0.len(), 3);
    assert_eq!(xs1.len(), 3);

    let expected = tracking / 1000.0 * size; // 25px
    for i in 0..2 {
        let advance_plain = xs0[i + 1] - xs0[i];
        let advance_tracked = xs1[i + 1] - xs1[i];
        let widened = advance_tracked - advance_plain;
        assert!(
            (widened - expected).abs() < 0.05,
            "glyph {i}→{}: advance が {widened}px 広がった(期待 {expected}px)",
            i + 1
        );
    }
    let width_delta = tracked.lines[0].width - plain.lines[0].width;
    assert!(
        (width_delta - 3.0 * expected).abs() < 0.15,
        "行幅の増分 {width_delta}px(期待 {}px)",
        3.0 * expected
    );
}

/// 製品の核は日本語歌詞 — CJK が同じ1本道(ttc 読み込み込み)を通ることも1本で固定する。
#[test]
fn cjk_text_goes_through_the_same_route() {
    let shaped =
        shape_text("字形が画素になる", &hiragino(), &layout(80.0, None, 0.0)).expect("shape");
    assert_eq!(shaped.lines[0].glyph_xs.len(), 8, "8 文字が 8 glyph になるはず");
    assert!(shaped.contours.len() >= 8);

    let canvas = Canvas {
        width: 704,
        height: 128,
        origin_x: 0,
        origin_y: 0,
    };
    let raster = render(&red_shape(shaped.contours), &canvas).expect("render");
    assert!(visible_pixels(&raster) > 4_000, "可視画素が少なすぎる");
}

/// 空文字列は空輪郭(エラーではない) — rasterize すれば alpha=0 の raster になる。
#[test]
fn empty_string_rasterizes_to_a_fully_transparent_raster() {
    let shaped = shape_text("", &arial(), &layout(64.0, None, 0.0)).expect("shape");
    assert!(shaped.contours.is_empty());

    let canvas = Canvas {
        width: 64,
        height: 64,
        origin_x: 0,
        origin_y: 0,
    };
    let raster = render(&red_shape(shaped.contours), &canvas).expect("render");
    assert_eq!(visible_pixels(&raster), 0, "空文字列は画素を塗らないはず");
}

/// フォント欠落は明示 Err(裁定37) — 黙って既定フォントへ落とさない。
#[test]
fn missing_font_is_an_explicit_error_not_a_silent_fallback() {
    let bad = GlyphFont {
        path: "/System/Library/Fonts/does-not-exist-anywhere.ttf".to_owned(),
        family: "Nope".to_owned(),
    };
    let err = shape_text("hi", &bad, &layout(32.0, None, 0.0)).unwrap_err();
    assert!(matches!(
        err,
        motolii_vector::text::TextShapeError::FontFile { .. }
    ));
}

/// OpenType feature(タグ+値)がエラーなく通ることの確認(4byte でないタグは拒否)。
#[test]
fn feature_tag_must_be_four_bytes() {
    let mut l = layout(32.0, None, 0.0);
    l.features.push(TextFeature {
        tag: "toolong".to_owned(),
        value: 1,
    });
    let err = shape_text("hi", &arial(), &l).unwrap_err();
    assert!(matches!(
        err,
        motolii_vector::text::TextShapeError::FeatureTag(_)
    ));
}

/// 器具自体の再現性(tr=0 の対照): 同じ入力を2回組んで同じ実測値になる。
#[test]
fn shaping_is_deterministic_for_the_same_input() {
    let l = layout(72.0, Some(100.0), 50.0);
    let a = shape_text("Motolii 96", &arial(), &l).expect("shape");
    let b = shape_text("Motolii 96", &arial(), &l).expect("shape");
    assert_eq!(a, b);
}
