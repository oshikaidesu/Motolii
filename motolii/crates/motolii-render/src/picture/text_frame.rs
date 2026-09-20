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

fn to_layout(style: &TextDocumentStyle, document: &TextDocument, wrap_width: Option<f32>, wrap: bool) -> TextLayout {
    TextLayout {
        autospace: document.alignment.autospace,
        spacing_trim: document.alignment.spacing_trim,
        hanging: document.alignment.hanging,
        wrap_width,
        wrap,
        size: style.size,
        line_height: style.line_height,
        tracking: style.tracking,
        justify: to_justify(document.justify),
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

/// 文字が避けて流れる物(CSS `shape-outside`)。枠の座標の多角形と、その外へ取る間(`shape-margin`)。
#[derive(Clone, Debug, PartialEq)]
pub struct Obstacle {
    pub margin: f32,
    /// 移り方の重み(少し前の時刻の形ほど軽い)。1 つの時刻の形だけなら 1。
    pub weight: f32,
    pub points: Vec<[f32; 2]>,
}

/// 行の帯 [y0, y1] で多角形が占める横の範囲(間を足す)。凹んだ形も 1 本の区間(CSS の float の行と同じ)。
fn occupied(obstacle: &Obstacle, y0: f32, y1: f32) -> Option<(f32, f32)> {
    let (y0, y1) = (y0 - obstacle.margin, y1 + obstacle.margin);
    let n = obstacle.points.len();
    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    for i in 0..n {
        let (a, b) = (obstacle.points[i], obstacle.points[(i + 1) % n]);
        for p in [a, b] {
            if p[1] >= y0 && p[1] <= y1 {
                lo = lo.min(p[0]);
                hi = hi.max(p[0]);
            }
        }
        // 辺が帯の上下の線を横切る所。
        for y in [y0, y1] {
            if (a[1] - y) * (b[1] - y) < 0.0 {
                let x = a[0] + (b[0] - a[0]) * (y - a[1]) / (b[1] - a[1]);
                lo = lo.min(x);
                hi = hi.max(x);
            }
        }
    }
    lo.is_finite().then_some((lo - obstacle.margin, hi + obstacle.margin))
}

/// 幅 [0, width] から、帯にかかる物の範囲を抜いた区間。範囲は重みで数え、覆う重みが半分に届く所を塞ぐ
/// (移り方を持つ物は、遡る時刻の形の過半が覆う所。1 つの時刻だけの物は覆えば塞ぐ)。
fn open_segments(around: &[Obstacle], width: f32, y0: f32, y1: f32) -> Vec<(f32, f32)> {
    let mut edges: Vec<(f32, f32)> = Vec::new();
    for obstacle in around {
        if let Some((lo, hi)) = occupied(obstacle, y0, y1) {
            edges.push((lo, obstacle.weight));
            edges.push((hi, -obstacle.weight));
        }
    }
    edges.sort_by(|a, b| a.0.total_cmp(&b.0).then(b.1.total_cmp(&a.1)));
    let mut open = Vec::new();
    let mut cover = 0.0f32;
    let mut free_from = Some(0.0f32);
    for (x, dw) in edges {
        cover += dw;
        let blocked = cover >= 0.5 - 1e-4;
        match (blocked, free_from) {
            (true, Some(a)) => {
                if x.min(width) > a { open.push((a, x.min(width))); }
                free_from = None;
            }
            (false, None) => free_from = Some(x.max(0.0)),
            _ => {}
        }
    }
    if let Some(a) = free_from.filter(|a| *a < width) {
        open.push((a, width));
    }
    open
}

/// 文字を組んで枠の縦中央へ置いた輪郭。
pub fn shape_document(
    document: &TextDocument,
    t: RationalTime,
    canvas: &Canvas,
) -> Result<Option<ShapedText>, TextShapeError> {
    shape_document_around(document, t, canvas, &[])
}

/// 物を避けて流す組み方(折り返す文字だけ。折り返さない文字は物を見ない)。
pub fn shape_document_around(
    document: &TextDocument,
    t: RationalTime,
    canvas: &Canvas,
    around: &[Obstacle],
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
    let layout = to_layout(style, document, Some(document.wrap_size.map(|s| s[0]).unwrap_or(canvas.width as f32)), document.wrap_size.is_some());
    let fonts: Vec<_> = document.styles.iter().map(to_glyph_font).collect();
    let layouts: Vec<_> = document.styles.iter().map(|s| to_layout(s, document, layout.wrap_width, layout.wrap)).collect();
    let mut pieces: Vec<(String, usize)> = Vec::new();
    if document.runs.is_empty() {
        pieces.push((content.to_owned(), 0));
    } else {
        let ids = crate::doc::store::text_read::style_ids(document, content);
        for (g,id) in crate::doc::store::text_read::graphemes(content).into_iter().zip(ids) {
            let index = document.styles.iter().position(|s| s.id == id).unwrap_or(0);
            if let Some((text, _)) = pieces.last_mut().filter(|(_,i)|*i==index) { text.push_str(g); }
            else { pieces.push((g.to_owned(), index)); }
        }
    }
    let size = pieces.iter().map(|(_, i)| f64::from(document.styles[*i].size)).fold(f64::from(style.size), f64::max);
    // 文字の塊を枠の縦中央へ。1 行目のベースラインの上に約 1 級、最終行の下に約 1/4 級を見る。
    let centre = |shaped: &ShapedText| match (shaped.lines.first(), shaped.lines.last()) {
        (Some(first), Some(last)) => {
            let top = f64::from(first.baseline_y) - size;
            let bottom = f64::from(last.baseline_y) + size * 0.25;
            (f64::from(canvas.height) - (bottom - top)) * 0.5 - top
        }
        _ => 0.0,
    };
    let shape_plain = || -> Result<ShapedText, TextShapeError> {
        if document.runs.is_empty() {
            shape_text(content, &font, &layout)
        } else {
            let spans: Vec<_> = pieces.iter().map(|(text,i)|StyledText{text,font:&fonts[*i],layout:&layouts[*i],style:*i}).collect();
            shape_rich_text(&spans, &layout)
        }
    };
    let mut shaped = shape_plain()?;
    let dy = centre(&shaped);
    if layout.wrap && !around.is_empty() {
        // 縦の寄せ方は避けない組みで決める(避けて行が増えれば下へ伸びる、CSS の block と同じ)。
        // 避けた組みで寄せ直すと、帯が動いて組みが変わり、2 つの組みの間を往復する。
        let spans: Vec<_> = pieces.iter().map(|(text,i)|StyledText{text,font:&fonts[*i],layout:&layouts[*i],style:*i}).collect();
        let width = layout.wrap_width.unwrap_or(canvas.width as f32);
        let band = dy as f32;
        shaped = crate::doc::vector::text::shape_rich_text_around(&spans, &layout, &|y0, y1| open_segments(around, width, y0 + band, y1 + band))?;
    }
    for contour in &mut shaped.contours {
        for v in &mut contour.vertices {
            // 接線は点からの相対。動かすのは点だけ。
            v.point.y += dy;
        }
    }
    Ok(Some(shaped))
}

/// Split の単位の箱(素材座標、読む順)。1 = 字(空白は単位にならない)、2 = 語(空白で区切る)、3 = 行。
/// 箱は字の送り幅 × 行の箱の縦(`line_box` と同じ、上に約 1 級・下に約 1/4 級)。隣の字と辺を分け合う(重ねない)。
pub fn split_boxes(document: &TextDocument, shaped: &ShapedText, canvas: &Canvas, content: &str, split: i64) -> Vec<[f32; 4]> {
    let (Some(first), Some(last), Some(style)) = (shaped.lines.first(), shaped.lines.last(), document.styles.first()) else { return Vec::new() };
    let size = shaped.contour_styles.iter().map(|i| document.styles[*i].size).fold(style.size, f32::max);
    // 縦の寄せは shape_document と同じ式(輪郭には足してあるが、行の物差しには足していない)。
    let (top, bottom) = (first.baseline_y - size, last.baseline_y + size * 0.25);
    let dy = (canvas.height as f32 - (bottom - top)) * 0.5 - top;
    let blank = |byte: usize| content.get(byte..).and_then(|s| s.chars().next()).is_some_and(char::is_whitespace);
    // (箱, 元の byte) を読む順に。
    let mut glyphs: Vec<([f32; 4], usize)> = Vec::new();
    let mut lines: Vec<[f32; 4]> = Vec::new();
    for line in &shaped.lines {
        let (y0, y1) = (line.baseline_y + dy - size, line.baseline_y + dy + size * 0.25);
        let left = line.glyph_xs.first().copied().unwrap_or(0.0);
        lines.push([left, y0, left + line.width, y1]);
        for (i, &x0) in line.glyph_xs.iter().enumerate() {
            let x1 = line.glyph_xs.get(i + 1).copied().unwrap_or(left + line.width).max(x0);
            glyphs.push(([x0, y0, x1, y1], line.glyph_bytes.get(i).copied().unwrap_or(usize::MAX)));
        }
    }
    let union = |boxes: &mut dyn Iterator<Item = [f32; 4]>| boxes.fold(None, |acc: Option<[f32; 4]>, b| Some(match acc {
        Some(a) => [a[0].min(b[0]), a[1].min(b[1]), a[2].max(b[2]), a[3].max(b[3])],
        None => b,
    }));
    match split {
        1 => glyphs.iter().filter(|(_, byte)| !blank(*byte)).map(|(b, _)| *b).collect(),
        2 => {
            let mut words: Vec<(usize, usize)> = Vec::new();
            let mut start = None;
            for (i, ch) in content.char_indices() {
                match (ch.is_whitespace(), start) {
                    (false, None) => start = Some(i),
                    (true, Some(s)) => { words.push((s, i)); start = None; }
                    _ => {}
                }
            }
            if let Some(s) = start { words.push((s, content.len())); }
            words.iter().filter_map(|&(s, e)| union(&mut glyphs.iter().filter(|(_, byte)| (s..e).contains(byte)).map(|(b, _)| *b))).collect()
        }
        3 => lines,
        _ => Vec::new(),
    }
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
