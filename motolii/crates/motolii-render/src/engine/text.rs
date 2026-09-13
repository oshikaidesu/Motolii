use crate::doc::store::{
    RationalTime, TextDocument, TextDocumentStyle, TextJustify as StoreJustify,
};
use crate::doc::vector::text::{
    shape_text, shape_rich_text, StyledText, GlyphFont, TextFeature, TextJustify, TextLayout, TextShapeError,
};
use crate::doc::store::ShapeNode;
use crate::doc::vector::{
    Brush, Canvas, Fill, FillRule, PathSource, Rgb, Shape, VectorError,
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

fn to_fill(style: &TextDocumentStyle) -> Fill {
    Fill {
        brush: Brush::Solid(Rgb {
            r: style.fill[0],
            g: style.fill[1],
            b: style.fill[2],
        }),
        rule: FillRule::NonZero,
        opacity: style.fill[3],
        hidden: false,
    }
}

/// 文字を形の木に組む(描くのは形の層と同じ paths renderer)。
pub fn text_shapes(
    document: &TextDocument,
    t: RationalTime,
    canvas: &Canvas,
) -> Result<Option<Vec<ShapeNode>>, TextRenderError> {
    text_shapes_morphed(document, None, t, canvas)
}

/// `morph` は Text Morph 効果の相手(その書類で同じ枠に組んだ文字)と混合率 0..1。
/// 文字は組んだ順の番で対になり、塗りはこの層の style のまま。
pub fn text_shapes_morphed(
    document: &TextDocument,
    morph: Option<(&TextDocument, f64)>,
    t: RationalTime,
    canvas: &Canvas,
) -> Result<Option<Vec<ShapeNode>>, TextRenderError> {
    let Some(mut shaped) = shape_document(document, t, canvas)? else {
        return Ok(None);
    };
    if let Some((target, amount)) = morph.filter(|(_, amount)| *amount > 0.0) {
        if let Some(other) = shape_document(target, t, canvas)? {
            let pairs = crate::doc::vector::morph::morph_glyphs(&shaped.contours, &shaped.contour_glyphs, &other.contours, &other.contour_glyphs, amount);
            let styles: Vec<usize> = pairs.iter().map(|(_, glyph)| {
                shaped.contour_glyphs.iter().position(|g| g == glyph).map_or(0, |i| shaped.contour_styles[i])
            }).collect();
            shaped.contour_styles = styles;
            shaped.contour_glyphs = pairs.iter().map(|(_, g)| *g).collect();
            shaped.contours = pairs.into_iter().map(|(c, _)| c).collect();
        }
    }

    let mut batches: Vec<(usize, Vec<crate::doc::vector::Contour>)> = Vec::new();
    for (contour, index) in shaped.contours.into_iter().zip(shaped.contour_styles) {
        if let Some((_,contours)) = batches.last_mut().filter(|(i,_)|*i==index) { contours.push(contour); }
        else { batches.push((index,vec![contour])); }
    }
    let mut result = Vec::new();
    for (index, contours) in batches {
        result.push(paint_contours(contours, &document.styles[index]));
    }
    Ok(Some(result))
}

/// 文字を組んで枠の縦中央へ置いた輪郭。
fn shape_document(
    document: &TextDocument,
    t: RationalTime,
    canvas: &Canvas,
) -> Result<Option<crate::doc::vector::text::ShapedText>, TextRenderError> {
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
    Ok(Some(shaped))
}

/// 層に積まれた Text Morph の相手(組んである文字書類)と混合率。相手が無ければ効かない。
pub(crate) fn morph_partner<'a>(
    layer: &crate::doc::store::ResolvedLayer,
    documents: &'a std::collections::HashMap<crate::doc::store::LayerId, TextDocument>,
) -> Option<(&'a TextDocument, f64)> {
    let effects: Vec<_> = layer.effects.iter().chain(&layer.after_effects).cloned().collect();
    let (target, amount) = crate::doc::store::textop::morph(&effects)?;
    documents.get(&target).map(|d| (d, amount))
}

fn paint_contours(contours: Vec<crate::doc::vector::Contour>, style: &TextDocumentStyle) -> ShapeNode {
    ShapeNode::Leaf(Shape {
        source: PathSource::Bezier(contours),
        ops: Vec::new(),
        fill: Some(to_fill(style)),
        stroke: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::{ContentKeyframe, ContentTrack, FontRef, TextDocumentStyle, TextStyleId};

    fn document(family: &str, content: &str) -> TextDocument {
        let mut track = ContentTrack::new();
        track.insert(ContentKeyframe { t: RationalTime::ZERO, content: content.to_owned() });
        TextDocument {
            content: track,
            justify: StoreJustify::Center,
            wrap_size: None,
            styles: vec![TextDocumentStyle {
                id: TextStyleId(0),
                font: FontRef { path: String::new(), fingerprint: None, family: family.to_owned(), style: String::new() },
                size: 96.0, fill: [1.0; 4], line_height: None, tracking: 0.0, axes: vec![], features: vec![],
            }],
            slot_id: None, ranges: vec![], alignment: Default::default(), runs: vec![],
        }
    }

    fn contours(shapes: &[ShapeNode]) -> Vec<crate::doc::vector::Contour> {
        shapes.iter().flat_map(|n| match n { ShapeNode::Leaf(Shape { source: PathSource::Bezier(path), .. }) => path.clone(), _ => vec![] }).collect()
    }

    fn bounds(contours: &[crate::doc::vector::Contour]) -> [f64; 4] {
        contours.iter().flat_map(|c| &c.vertices).fold([f64::MAX, f64::MAX, f64::MIN, f64::MIN], |b, v| [b[0].min(v.point.x), b[1].min(v.point.y), b[2].max(v.point.x), b[3].max(v.point.y)])
    }

    /// Text Morph: 量 0 は自分の輪郭のまま、1 は相手の輪郭そのもの、途中は文字ごとに対になった折れ線で
    /// 大きさは両者の間。相手の文字数が違っても文字の番で対になる。
    #[test]
    fn morph_walks_from_this_face_to_the_target_face() {
        let (a, b) = ("Hiragino Sans", "Hiragino Mincho ProN");
        if !(crate::doc::vector::text::font_supports_sample(a, "永") && crate::doc::vector::text::font_supports_sample(b, "永")) {
            eprintln!("skipped: fonts missing");
            return;
        }
        let canvas = Canvas { width: 400, height: 200, origin_x: 0, origin_y: 0 };
        let mine = document(a, "永");
        let theirs = document(b, "永");
        let at = |amount: f64| contours(&text_shapes_morphed(&mine, Some((&theirs, amount)), RationalTime::ZERO, &canvas).unwrap().unwrap());
        let (start, end) = (at(0.0), at(1.0));
        assert_eq!(start, contours(&text_shapes(&mine, RationalTime::ZERO, &canvas).unwrap().unwrap()));
        assert_eq!(end, contours(&text_shapes(&theirs, RationalTime::ZERO, &canvas).unwrap().unwrap()));
        assert_ne!(start, end, "two faces must differ for the test to mean anything");
        let mid = at(0.5);
        assert!(mid != start && mid != end);
        let (bs, bm, be) = (bounds(&start), bounds(&mid), bounds(&end));
        for i in 0..4 {
            assert!(bm[i] >= bs[i].min(be[i]) - 1.0 && bm[i] <= bs[i].max(be[i]) + 1.0, "{i}: {bs:?} {bm:?} {be:?}");
        }
        // 文字数が違う相手: 2 文字目は相手が無く、自分の重心へ潰れる途中の形になる。
        let two = document(a, "永永");
        let mid = contours(&text_shapes_morphed(&two, Some((&theirs, 0.5)), RationalTime::ZERO, &canvas).unwrap().unwrap());
        assert!(!mid.is_empty());
        // 塗りは自分の style のまま(相手の style は使わない)。
        let painted = text_shapes_morphed(&mine, Some((&theirs, 0.5)), RationalTime::ZERO, &canvas).unwrap().unwrap();
        assert_eq!(painted.len(), 1);
    }
}
