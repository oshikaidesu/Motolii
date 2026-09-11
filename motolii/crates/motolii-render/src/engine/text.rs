use crate::doc::store::{
    RationalTime, TextDocument, TextDocumentStyle, TextJustify as StoreJustify,
};
use crate::doc::vector::text::{
    shape_text, shape_rich_text, StyledText, GlyphFont, TextFeature, TextJustify, TextLayout, TextShapeError,
};
use crate::doc::store::ShapeNode;
use crate::doc::vector::{
    Brush, Canvas, Fill, FillRule, PathSource, Rgb, Shape, Stroke, VectorError,
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

fn to_layout(style: &TextDocumentStyle, justify: StoreJustify, wrap_width: Option<f32>, wrap: bool) -> TextLayout {
    TextLayout {
        wrap_width,
        wrap,
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

/// 文字を形の木に組む(描くのは形の層と同じ paths renderer)。
pub fn text_shapes(
    document: &TextDocument,
    t: RationalTime,
    canvas: &Canvas,
) -> Result<Option<Vec<ShapeNode>>, TextRenderError> {
    let Some(style) = document.styles.first() else {
        return Ok(None);
    };
    let content = document.content.eval(t);
    if content.is_empty() {
        return Ok(None);
    }

    let font = to_glyph_font(style);
    // 幅は揃えに要る(無いと Center/Right が効かない)が、折り返すのは wrap 箱を持つ層だけ。
    let layout = to_layout(style, document.justify, Some(document.wrap_size.map(|s| s[0]).unwrap_or(canvas.width as f32)), document.wrap_size.is_some());
    let mut shaped = if document.runs.is_empty() {
        shape_text(content, &font, &layout)?
    } else {
        let ids = crate::doc::store::text_edit::style_ids(document, content);
        let mut pieces: Vec<(String, usize)> = Vec::new();
        for (g,id) in crate::doc::store::text_edit::graphemes(content).into_iter().zip(ids) {
            let index = document.styles.iter().position(|s| s.id == id).unwrap_or(0);
            if let Some((text, _)) = pieces.last_mut().filter(|(_,i)|*i==index) { text.push_str(g); }
            else { pieces.push((g.to_owned(), index)); }
        }
        let fonts: Vec<_> = document.styles.iter().map(to_glyph_font).collect();
        let layouts: Vec<_> = document.styles.iter().map(|s|to_layout(s, document.justify, layout.wrap_width, layout.wrap)).collect();
        let spans: Vec<_> = pieces.iter().map(|(text,i)|StyledText{text,font:&fonts[*i],layout:&layouts[*i],style:*i}).collect();
        shape_rich_text(&spans, &layout)?
    };

    // 文字の塊を枠の縦中央へ。1 行目のベースラインの上に約 1 級、最終行の下に約 1/4 級を見る。
    if let (Some(first), Some(last)) = (shaped.lines.first(), shaped.lines.last()) {
        let size = shaped.contour_styles.iter().map(|i| f64::from(document.styles[*i].size)).fold(f64::from(style.size), f64::max);
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

    let mut batches: Vec<(usize, Vec<crate::doc::vector::Contour>)> = Vec::new();
    for (contour, index) in shaped.contours.into_iter().zip(shaped.contour_styles) {
        if let Some((_,contours)) = batches.last_mut().filter(|(i,_)|*i==index) { contours.push(contour); }
        else { batches.push((index,vec![contour])); }
    }
    let mut result = Vec::new();
    for (index, contours) in batches {
        result.extend(paint_contours(contours, &document.styles[index]));
    }
    Ok(Some(result))
}

fn paint_contours(contours: Vec<crate::doc::vector::Contour>, style: &TextDocumentStyle) -> Vec<ShapeNode> {
    let (fill, stroke) = to_fill_stroke(style);
    let leaf = |source, fill, stroke| ShapeNode::Leaf(Shape { source, ops: Vec::new(), fill, stroke });
    // 縁取りは既定で fill の**下**(stroke_over_fill が false)。輪郭中心の stroke は外側半分しか
    // 見えないので幅を 2 倍にし、先に置いてから fill を上に重ねる — 字が痩せない。
    if let (Some(stroke), false) = (stroke.clone(), style.stroke_over_fill) {
        return vec![
            leaf(PathSource::Bezier(contours.clone()), None, Some(Stroke { width: stroke.width * 2.0, ..stroke })),
            leaf(PathSource::Bezier(contours), fill, None),
        ];
    }
    vec![leaf(PathSource::Bezier(contours), fill, stroke)]
}

