use crate::doc::store::{
    RationalTime, TextDocument, TextDocumentStyle, TextJustify as StoreJustify,
};
use crate::doc::vector::text::{
    shape_text, GlyphFont, TextFeature, TextJustify, TextLayout, TextShapeError,
};
use crate::doc::vector::{
    Brush, Canvas, Fill, FillRule, PathSource, Raster, Rgb, Shape, Stroke, VectorError,
};

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
    Ok(Some(crate::doc::vector::render(&shape, canvas)?))
}
