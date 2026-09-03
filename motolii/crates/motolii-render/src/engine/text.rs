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

fn to_layout(style: &TextDocumentStyle, justify: StoreJustify, wrap_width: Option<f32>) -> TextLayout {
    TextLayout {
        wrap_width,
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
    let layout = to_layout(style, document.justify, Some(document.wrap_size.map(|s| s[0]).unwrap_or(canvas.width as f32)));
    let mut shaped = shape_text(content, &font, &layout)?;

    // 文字の塊を枠の縦中央へ。1 行目のベースラインの上に約 1 級、最終行の下に約 1/4 級を見る。
    if let (Some(first), Some(last)) = (shaped.lines.first(), shaped.lines.last()) {
        let size = f64::from(style.size);
        let top = f64::from(first.baseline_y) - size;
        let bottom = f64::from(last.baseline_y) + size * 0.25;
        let dy = (f64::from(canvas.height) - (bottom - top)) * 0.5 - top;
        for contour in &mut shaped.contours {
            for v in &mut contour.vertices {
                // 接線は点からの相対。動かすのは点だけ。
                v.point.y += dy;
            }
        }
    }

    let (fill, stroke) = to_fill_stroke(style);
    // 縁取りは既定で fill の**下**(stroke_over_fill が false)。輪郭中心の stroke は外側半分しか
    // 見えないので幅を 2 倍にし、先に焼いてから fill を上に重ねる — 字が痩せない。
    if let (Some(stroke), false) = (stroke.clone(), style.stroke_over_fill) {
        let under = Shape {
            source: PathSource::Bezier(shaped.contours.clone()),
            ops: Vec::new(),
            fill: None,
            stroke: Some(Stroke { width: stroke.width * 2.0, ..stroke }),
        };
        let over = Shape {
            source: PathSource::Bezier(shaped.contours),
            ops: Vec::new(),
            fill,
            stroke: None,
        };
        let mut base = crate::doc::vector::render(&under, canvas)?;
        let top = crate::doc::vector::render(&over, canvas)?;
        composite_over(&mut base, &top);
        return Ok(Some(base));
    }
    let shape = Shape {
        source: PathSource::Bezier(shaped.contours),
        ops: Vec::new(),
        fill,
        stroke,
    };
    Ok(Some(crate::doc::vector::render(&shape, canvas)?))
}

/// 乗算済み RGBA の `top` を `base` の上に重ねる(out = top + base × (1 − top.a))。
fn composite_over(base: &mut Raster, top: &Raster) {
    for (b, t) in base.premultiplied_rgba8.chunks_exact_mut(4).zip(top.premultiplied_rgba8.chunks_exact(4)) {
        let ta = t[3] as u32;
        if ta == 0 {
            continue;
        }
        let keep = 255 - ta;
        for i in 0..4 {
            b[i] = (t[i] as u32 + b[i] as u32 * keep / 255).min(255) as u8;
        }
    }
}
