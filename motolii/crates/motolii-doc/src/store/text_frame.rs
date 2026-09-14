//! 文字の書類を枠に組む — 描く側(render の文字の層)と、層の箱を測る側(並べる法)が同じ組み方を読む。

use crate::doc::core::RationalTime;
use crate::doc::store::{TextDocument, TextDocumentStyle, TextJustify as StoreJustify};
use crate::doc::vector::text::{
    shape_rich_text, shape_text, GlyphFont, ShapedText, StyledText, TextFeature, TextJustify, TextLayout, TextShapeError,
};
use crate::doc::vector::Canvas;

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

/// 文字を組んで枠の縦中央へ置いた輪郭。
pub fn shape_document(
    document: &TextDocument,
    t: RationalTime,
    canvas: &Canvas,
) -> Result<Option<ShapedText>, TextShapeError> {
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

/// 組んだ文字の行の箱(素材座標 = 枠の座標)。字形のインクではなく、行の幅と、上に約 1 級・下に約 1/4 級の縦。
/// 文字を差し替えても縦がぶれない(並べる法の「文字は行の箱」)。
pub fn line_box(document: &TextDocument, shaped: &ShapedText, canvas: &Canvas) -> Option<[f32; 4]> {
    let (first, last) = (shaped.lines.first()?, shaped.lines.last()?);
    let style = document.styles.first()?;
    let size = shaped.contour_styles.iter().map(|i| document.styles[*i].size).fold(style.size, f32::max);
    let height = last.baseline_y - first.baseline_y + size * 1.25;
    let top = (canvas.height as f32 - height) * 0.5;
    let (mut left, mut right) = (f32::INFINITY, f32::NEG_INFINITY);
    for line in &shaped.lines {
        let x = line.glyph_xs.first().copied().unwrap_or(0.0);
        left = left.min(x);
        right = right.max(x + line.width);
    }
    left.is_finite().then_some([left, top, right, top + height])
}
