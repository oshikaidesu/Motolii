//! R6 の合格線(発注 2026-08-22)を落ちるテストの形で固定する:
//! 1. 「文字列 + フォント + サイズ → tiny-skia Path(輪郭)→ raster → PNG」が1本通る
//! 2. lh(行送り)が**数値で**効く(layout の baseline 差 = lh そのもの)
//! 3. tr(トラッキング)が**数値で**効く(glyph ペン位置の差 = tr/1000 em)
//!
//! フォントは macOS 実機の Arial(path 解決 — `FontRef::path` の意味そのまま)。
//! この probe はこの機械で回す測定器具なので、システムフォントの実在を前提にする
//! (r3 が実 PLY を前提にするのと同じ)。

use motolii_store::{FontRef, TextDocumentStyle, TextJustify, TextStyleId};
use r6_text_shaping::{rasterize, shape_text, to_shape};
use std::path::PathBuf;

const ARIAL: &str = "/System/Library/Fonts/Supplemental/Arial.ttf";
const OUT_DIR: &str = "/private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/3e854a2b-6802-4028-acd7-851a0222b45a/scratchpad/r6";

fn style(size: f32, line_height: Option<f32>, tracking: f32) -> TextDocumentStyle {
    TextDocumentStyle {
        id: TextStyleId(0),
        font: FontRef {
            path: ARIAL.to_owned(),
            fingerprint: None,
            family: "Arial".to_owned(),
            style: "Regular".to_owned(),
        },
        size,
        fill: [1.0, 0.0, 0.0, 1.0], // 赤 — fill 色が画素まで届いた事の検証にも使う
        line_height,
        tracking,
        stroke_color: None,
        stroke_width: 0.0,
        stroke_over_fill: false,
        axes: Vec::new(),
        features: Vec::new(),
    }
}

/// 合格線1: 文字列 → 輪郭 → raster → PNG の1本道。
#[test]
fn a_string_becomes_outlines_then_pixels_then_a_png() {
    let style = style(96.0, None, 0.0);
    let shaped = shape_text("Motolii", &style, TextJustify::Left).expect("shape");

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

    let raster = rasterize(&to_shape(&shaped, &style), 512, 160).expect("render");
    let px = &raster.premultiplied_rgba8;
    let visible = px.chunks_exact(4).filter(|p| p[3] > 0).count();
    assert!(
        visible > 2_000,
        "可視画素 {visible} — 96px の 7 文字なら数千画素は塗られるはず"
    );
    // fill [1,0,0,1](赤)が画素まで届いている(premultiplied なので r=a の画素がある)。
    assert!(
        px.chunks_exact(4).any(|p| p[0] > 200 && p[1] == 0 && p[2] == 0),
        "赤の fill が画素へ届いていない"
    );

    std::fs::create_dir_all(OUT_DIR).expect("scratchpad dir");
    let path = PathBuf::from(OUT_DIR).join("motolii-arial-96.png");
    image::save_buffer(
        &path,
        px,
        raster.width,
        raster.height,
        image::ColorType::Rgba8,
    )
    .expect("png");
    assert!(path.exists());
}

/// 合格線2: lh は baseline 差そのもの。90 → 90、140 → 140(±0.01)。
#[test]
fn line_height_is_the_baseline_delta_numerically() {
    for lh in [90.0_f32, 140.0] {
        let style = style(64.0, Some(lh), 0.0);
        let shaped = shape_text("MO\nLII", &style, TextJustify::Left).expect("shape");
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
    let plain = shape_text("III", &style(size, None, 0.0), TextJustify::Left).expect("shape");
    let tracked = shape_text("III", &style(size, None, tracking), TextJustify::Left).expect("shape");

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
    // 行幅にも同じ量が乗る(最後の glyph の advance にも spacing が付くので 3 glyph 分)。
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
    let mut style = style(80.0, None, 0.0);
    style.font = FontRef {
        path: "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc".to_owned(),
        fingerprint: None,
        family: "Hiragino Sans".to_owned(),
        style: "W3".to_owned(),
    };
    let shaped = shape_text("字形が画素になる", &style, TextJustify::Left).expect("shape");
    assert_eq!(shaped.lines[0].glyph_xs.len(), 8, "8 文字が 8 glyph になるはず");
    assert!(shaped.contours.len() >= 8);

    let raster = rasterize(&to_shape(&shaped, &style), 704, 128).expect("render");
    let visible = raster
        .premultiplied_rgba8
        .chunks_exact(4)
        .filter(|p| p[3] > 0)
        .count();
    assert!(visible > 4_000, "可視画素 {visible}");

    std::fs::create_dir_all(OUT_DIR).expect("scratchpad dir");
    image::save_buffer(
        PathBuf::from(OUT_DIR).join("cjk-hiragino-80.png"),
        &raster.premultiplied_rgba8,
        raster.width,
        raster.height,
        image::ColorType::Rgba8,
    )
    .expect("png");
}

/// tr=0 の対照: 同じ入力を2回組んで同じ実測値になる(器具自体の再現性)。
#[test]
fn shaping_is_deterministic_for_the_same_input() {
    let style = style(72.0, Some(100.0), 50.0);
    let a = shape_text("Motolii 96", &style, TextJustify::Left).expect("shape");
    let b = shape_text("Motolii 96", &style, TextJustify::Left).expect("shape");
    assert_eq!(a, b);
}
