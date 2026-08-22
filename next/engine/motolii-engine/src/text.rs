//! owns: `TextDocument`(store の意味)→ `motolii-vector::text` の輪郭 →
//!       premultiplied RGBA8。裁定190 切片2 — `texture_for` の `LayerSource::Text`
//!       枝を繋ぐ実体そのもの。
//!
//! # まだ `texture_for` からは呼ばれていない
//!
//! `ResolvedLayer`(`motolii-store`)は `LayerId` を持たない —
//! [`motolii_store::ResolvedLayer::masks`] の doc がその理由を明示している:
//! 「別の口にすると `ResolvedLayer` から `LayerId` が引けず、描く側がマスクへ
//! 辿り着けない」。`texture_for` は `&LayerSource` と `source_frame` しか受け取らず、
//! `LayerSource::Text` はコンテンツを持たない印(unit variant)なので、**どの layer の
//! `TextDocument` を読むべきかを `texture_for` の中だけでは決められない**。
//!
//! **これは今回新しく見つかった穴ではない** — BL4(matte)レーンが全く同じ壁に
//! 当たり、`render_frame` への結線を明示的に保留している
//! (`02d5cec6 merge(engine): ...matteのrender_frame結線はstore要求
//! (ResolvedLayerにLayerId)待ちで明示Err維持`)。この発注は「store 読み専用」
//! (store の編集は範囲外)なので、ここでも同じ選択をする——`ResolvedLayer` に
//! layer 識別子を足す store 側の変更が済んだ日に、`texture_for` の `LayerSource::Text`
//! 枝は [`rasterize_text_document`] を1回呼ぶだけで繋がる形にしてある(黙って
//! 迂回のハックを積まない、裁定37/`wrapper-over-hack` と同じ判断)。
//!
//! # 変換の要点
//!
//! - **既定行(`styles[0]`、裁定98)を読む** — `TextDocument::styles` の
//!   rich-text 分割(`runs`)・range-selector/animator の適用は裁定191/probe の
//!   gap どおり範囲外(次切片)。style 表が空なら描く物が無い(`Ok(None)`、
//!   エラーではない — [`motolii_vector::text`] の「空文字列はエラーではない」と同じ形)。
//! - **content は Hold 評価**(`ContentTrack::eval(t)`)—— `TextDocument` の中で
//!   唯一時間変化するフィールド。
//! - fill/stroke は `motolii_vector::Shape` の語彙へそのまま写す —
//!   `mask.rs` が `ResolvedMask` → `Shape` を橋渡しするのと同じ形。

use motolii_store::{RationalTime, TextDocument, TextDocumentStyle, TextJustify as StoreJustify};
use motolii_vector::text::{shape_text, GlyphFont, TextFeature, TextJustify, TextLayout, TextShapeError};
use motolii_vector::{Brush, Canvas, Fill, FillRule, PathSource, Raster, Rgb, Shape, Stroke, VectorError};

/// [`rasterize_text_document`] が返しうる失敗。シェイピング側([`TextShapeError`])と
/// ラスタライズ側([`VectorError`])の両方をここへ畳む(`mask.rs::MaskFoldError` と同型)。
#[derive(Debug, thiserror::Error)]
pub enum TextRenderError {
    #[error(transparent)]
    Shape(#[from] TextShapeError),
    #[error(transparent)]
    Rasterize(#[from] VectorError),
}

fn to_glyph_font(style: &TextDocumentStyle) -> GlyphFont {
    GlyphFont {
        path: style.font.path.clone(),
        family: style.font.family.clone(),
    }
}

fn to_justify(justify: StoreJustify) -> TextJustify {
    match justify {
        StoreJustify::Left => TextJustify::Left,
        StoreJustify::Right => TextJustify::Right,
        StoreJustify::Center => TextJustify::Center,
    }
}

fn to_layout(style: &TextDocumentStyle, justify: StoreJustify) -> TextLayout {
    TextLayout {
        size: style.size,
        line_height: style.line_height,
        tracking: style.tracking,
        justify: to_justify(justify),
        features: style
            .features
            .iter()
            .map(|f| TextFeature {
                tag: f.tag.clone(),
                value: f.value,
            })
            .collect(),
    }
}

/// `TextDocumentStyle` の fill/stroke を `motolii_vector::Shape` の語彙へ写す。
/// `stroke_color` があっても `stroke_width <= 0.0` なら stroke は無い扱い
/// (`0` 幅の線を描いても画素は出ないので、`None` と同じ結果を先に決めておく)。
fn to_fill_stroke(style: &TextDocumentStyle) -> (Option<Fill>, Option<Stroke>) {
    let fill = Some(Fill {
        brush: Brush::Solid(Rgb {
            r: style.fill[0],
            g: style.fill[1],
            b: style.fill[2],
        }),
        rule: FillRule::NonZero,
        opacity: style.fill[3],
        hidden: false,
    });
    let stroke = style
        .stroke_color
        .filter(|_| style.stroke_width > 0.0)
        .map(|stroke_color| Stroke {
            brush: Brush::Solid(Rgb {
                r: stroke_color[0],
                g: stroke_color[1],
                b: stroke_color[2],
            }),
            width: f64::from(style.stroke_width),
            opacity: stroke_color[3],
            ..Stroke::default()
        });
    (fill, stroke)
}

/// `document` の**既定行**(裁定98、`styles.first()`)を時刻 `t` で評価し、`canvas` へ
/// ラスタライズする。
///
/// `Ok(None)` になるのは2通り(どちらもエラーではない — 描く物が無いだけ):
/// - `styles` が空(まだ一度もスタイルが設定されていない `TextDocument`)
/// - `content.eval(t)` が空文字列(Hold の初期値、または利用者が意図的に空にした)
///
/// フォントが読めない/OpenType feature タグが不正なら [`TextRenderError::Shape`]、
/// canvas が描けない大きさなら [`TextRenderError::Rasterize`] —
/// どちらも黙って空の絵へ落とさない(裁定37、[`text`] module doc と同じ判断)。
pub fn rasterize_text_document(
    document: &TextDocument,
    t: RationalTime,
    canvas: &Canvas,
) -> Result<Option<Raster>, TextRenderError> {
    let Some(style) = document.styles.first() else {
        return Ok(None);
    };
    let content = document.content.eval(t);
    if content.is_empty() {
        return Ok(None);
    }

    let font = to_glyph_font(style);
    let layout = to_layout(style, document.justify);
    let shaped = shape_text(content, &font, &layout)?;

    let (fill, stroke) = to_fill_stroke(style);
    let shape = Shape {
        source: PathSource::Bezier(shaped.contours),
        ops: Vec::new(),
        fill,
        stroke,
    };
    Ok(Some(motolii_vector::render(&shape, canvas)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_store::{
        ContentKeyframe, ContentTrack, FontRef, Fps, TextAlignmentOptions, TextDocumentStyle,
        TextStyleId,
    };

    const ARIAL: &str = "/System/Library/Fonts/Supplemental/Arial.ttf";

    fn t0() -> RationalTime {
        RationalTime::try_from_frame(0, Fps::try_new(30, 1).unwrap()).unwrap()
    }

    fn font_ref() -> FontRef {
        FontRef {
            path: ARIAL.to_owned(),
            fingerprint: None,
            family: "Arial".to_owned(),
            style: "Regular".to_owned(),
        }
    }

    fn style(size: f32, line_height: Option<f32>, tracking: f32, fill: [f64; 4]) -> TextDocumentStyle {
        TextDocumentStyle {
            id: TextStyleId(0),
            font: font_ref(),
            size,
            fill,
            line_height,
            tracking,
            stroke_color: None,
            stroke_width: 0.0,
            stroke_over_fill: false,
            axes: Vec::new(),
            features: Vec::new(),
        }
    }

    fn document_with(content: &str, style_row: TextDocumentStyle) -> TextDocument {
        let mut content_track = ContentTrack::new();
        content_track.insert(ContentKeyframe {
            t: t0(),
            content: content.to_owned(),
        });
        TextDocument {
            content: content_track,
            justify: StoreJustify::Left,
            wrap_size: None,
            styles: vec![style_row],
            slot_id: None,
            ranges: Vec::new(),
            alignment: TextAlignmentOptions::default(),
            runs: Vec::new(),
        }
    }

    fn canvas(width: u32, height: u32) -> Canvas {
        Canvas {
            width,
            height,
            origin_x: 0,
            origin_y: 0,
        }
    }

    fn visible_pixels(raster: &Raster) -> usize {
        raster
            .premultiplied_rgba8
            .chunks_exact(4)
            .filter(|p| p[3] > 0)
            .count()
    }

    #[test]
    fn text_layer_produces_non_empty_pixels_english() {
        let document = document_with("Motolii", style(96.0, None, 0.0, [1.0, 0.0, 0.0, 1.0]));
        let raster = rasterize_text_document(&document, t0(), &canvas(512, 160))
            .expect("render")
            .expect("非空文字列は Some のはず");
        assert!(visible_pixels(&raster) > 2_000);
        // fill 色(赤)が画素へ届いている。
        assert!(raster
            .premultiplied_rgba8
            .chunks_exact(4)
            .any(|p| p[0] > 200 && p[1] == 0 && p[2] == 0));
    }

    #[test]
    fn text_layer_produces_non_empty_pixels_japanese() {
        const HIRAGINO: &str = "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc";
        let mut style_row = style(80.0, None, 0.0, [0.0, 0.0, 1.0, 1.0]);
        style_row.font = FontRef {
            path: HIRAGINO.to_owned(),
            fingerprint: None,
            family: "Hiragino Sans".to_owned(),
            style: "W3".to_owned(),
        };
        let document = document_with("字形が画素になる", style_row);
        let raster = rasterize_text_document(&document, t0(), &canvas(704, 128))
            .expect("render")
            .expect("非空文字列は Some のはず");
        assert!(visible_pixels(&raster) > 4_000);
    }

    #[test]
    fn line_height_from_style_moves_the_second_line_baseline() {
        // lh が Shape の輪郭へ実際に反映されている証拠として、lh=200 で2行目が
        // canvas の外(y>=200)へ落ちる = 1行目の可視画素だけが残ることを確かめる。
        let tall = document_with("A\nB", style(64.0, Some(400.0), 0.0, [1.0, 1.0, 1.0, 1.0]));
        let tight = document_with("A\nB", style(64.0, Some(70.0), 0.0, [1.0, 1.0, 1.0, 1.0]));

        let tall_raster = rasterize_text_document(&tall, t0(), &canvas(128, 128))
            .expect("render")
            .expect("非空");
        let tight_raster = rasterize_text_document(&tight, t0(), &canvas(128, 480))
            .expect("render")
            .expect("非空");

        // lh=400 は2行目が 128px の canvas をはみ出す一方、lh=70 なら同じ2文字が
        // 480px 高の canvas に収まるので、後者の可視画素の方が多いはず
        // (両方とも「A」「B」1文字ずつなので、収まりきる方が塗り面積が単調に多い)。
        assert!(visible_pixels(&tight_raster) > visible_pixels(&tall_raster));
    }

    #[test]
    fn empty_style_table_yields_no_texture() {
        let document = TextDocument {
            content: ContentTrack::new(),
            justify: StoreJustify::Left,
            wrap_size: None,
            styles: Vec::new(),
            slot_id: None,
            ranges: Vec::new(),
            alignment: TextAlignmentOptions::default(),
            runs: Vec::new(),
        };
        let result = rasterize_text_document(&document, t0(), &canvas(64, 64)).expect("no error");
        assert!(result.is_none());
    }

    #[test]
    fn empty_content_yields_no_texture() {
        let document = document_with("", style(64.0, None, 0.0, [1.0, 1.0, 1.0, 1.0]));
        let result = rasterize_text_document(&document, t0(), &canvas(64, 64)).expect("no error");
        assert!(result.is_none());
    }

    #[test]
    fn missing_font_is_an_explicit_error() {
        let mut style_row = style(32.0, None, 0.0, [1.0, 1.0, 1.0, 1.0]);
        style_row.font.path = "/nonexistent/nope.ttf".to_owned();
        let document = document_with("hi", style_row);
        let err = rasterize_text_document(&document, t0(), &canvas(64, 64)).unwrap_err();
        assert!(matches!(err, TextRenderError::Shape(TextShapeError::FontFile { .. })));
    }
}
