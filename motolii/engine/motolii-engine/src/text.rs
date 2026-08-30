
use motolii_store::{RationalTime, TextDocument, TextDocumentStyle, TextJustify as StoreJustify};
use motolii_vector::text::{shape_text, GlyphFont, TextFeature, TextJustify, TextLayout, TextShapeError};
use motolii_vector::{Brush, Canvas, Fill, FillRule, PathSource, Raster, Rgb, Shape, Stroke, VectorError};

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
        let tall = document_with("A\nB", style(64.0, Some(400.0), 0.0, [1.0, 1.0, 1.0, 1.0]));
        let tight = document_with("A\nB", style(64.0, Some(70.0), 0.0, [1.0, 1.0, 1.0, 1.0]));

        let tall_raster = rasterize_text_document(&tall, t0(), &canvas(128, 128))
            .expect("render")
            .expect("非空");
        let tight_raster = rasterize_text_document(&tight, t0(), &canvas(128, 480))
            .expect("render")
            .expect("非空");

        assert!(visible_pixels(&tight_raster) > visible_pixels(&tall_raster));
    }

    fn max_visible_row(raster: &Raster) -> Option<u32> {
        (0..raster.height).rev().find(|&y| {
            let row_start = (y * raster.width * 4) as usize;
            let row_end = row_start + (raster.width * 4) as usize;
            raster.premultiplied_rgba8[row_start..row_end]
                .chunks_exact(4)
                .any(|pixel| pixel[3] > 0)
        })
    }

    #[test]
    fn two_line_content_reaches_further_down_the_canvas_than_one_line() {
        let one_line = document_with("A", style(64.0, None, 0.0, [1.0, 1.0, 1.0, 1.0]));
        let two_line = document_with("A\nB", style(64.0, None, 0.0, [1.0, 1.0, 1.0, 1.0]));

        let one_raster = rasterize_text_document(&one_line, t0(), &canvas(128, 256))
            .expect("render")
            .expect("1行は非空のはず");
        let two_raster = rasterize_text_document(&two_line, t0(), &canvas(128, 256))
            .expect("render")
            .expect("2行は非空のはず");

        let one_bottom = max_visible_row(&one_raster).expect("1行は可視画素を持つはず");
        let two_bottom = max_visible_row(&two_raster).expect("2行は可視画素を持つはず");

        assert!(
            two_bottom > one_bottom,
            "2行の Content が1行より下まで描かれていない(1行下端={one_bottom}, 2行下端={two_bottom})"
        );
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
