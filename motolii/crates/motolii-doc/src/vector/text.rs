
use cosmic_text::fontdb;
use cosmic_text::{
    Align, Attrs, Buffer, Command, Family, FeatureTag, FontFeatures, FontSystem, Metrics, Shaping,
    SwashCache,
};

use crate::doc::vector::{Contour, Point, Vertex};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphFont {
    pub path: String,
    pub family: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextJustify {
    #[default]
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextFeature {
    pub tag: String,
    pub value: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextLayout {
    /// 折返しと揃えの幅。None なら point text(揃えは効かない — cosmic-text は幅が無いと補正 0)。
    pub wrap_width: Option<f32>,
    /// 折り返すか。幅は揃え(Center/Right)にも要るので、point text は幅を持って折り返さない。
    pub wrap: bool,
    pub size: f32,
    pub line_height: Option<f32>,
    pub tracking: f32,
    pub justify: TextJustify,
    pub features: Vec<TextFeature>,
}

impl TextLayout {
    pub fn new(size: f32) -> Self {
        Self {
            wrap_width: None,
            wrap: false,
            size,
            line_height: None,
            tracking: 0.0,
            justify: TextJustify::default(),
            features: Vec::new(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TextShapeError {
    #[error("font file {path} could not be read: {source}")]
    FontFile {
        path: String,
        source: std::io::Error,
    },
    #[error("OpenType feature tag {0:?} is not 4 bytes")]
    FeatureTag(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineMeasure {
    pub baseline_y: f32,
    pub width: f32,
    pub glyph_xs: Vec<f32>,
    /// 字ごとの元の文字の byte の位置(組み直しても同じ字を対にする。移り方と物差し)。
    pub glyph_bytes: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ShapedText {
    pub contours: Vec<Contour>,
    pub contour_styles: Vec<usize>,
    /// 輪郭ごとの文字の番(組んだ順)。morph が同じ番同士を対にする。
    pub contour_glyphs: Vec<usize>,
    pub lines: Vec<LineMeasure>,
}

/// 書体の台帳と shaper は process に 1 つ(cosmic-text の指示: 作るのは 1 秒級、一度だけ共有せよ)。
/// OS の書体を全部積み、locale は ja-JP — 漢字の後詰めがヒラギノになり、絵文字・欧文は OS の
/// fallback(Apple Color Emoji・.SF NS)へ落ちる。`path` は台帳に無い書体を足す口として残す。
fn font_system() -> std::sync::MutexGuard<'static, FontSystem> {
    static SYSTEM: std::sync::OnceLock<std::sync::Mutex<FontSystem>> = std::sync::OnceLock::new();
    SYSTEM
        .get_or_init(|| {
            let mut db = fontdb::Database::new();
            db.load_system_fonts();
            std::sync::Mutex::new(FontSystem::new_with_locale_and_db("ja-JP".to_owned(), db))
        })
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

/// Families available to the same database used to shape the document.
pub fn font_families() -> &'static [String] {
    static FAMILIES: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    FAMILIES.get_or_init(|| {
        let system = font_system();
        let names: std::collections::BTreeSet<_> = system.db().faces()
            .flat_map(|face| face.families.iter().map(|(name, _)| name.clone()))
            .collect();
        names.into_iter().collect()
    })
}

/// 台帳に在る family の一覧。face を 1 枚ずつ名前で照合すると、組む度に台帳を全走査する
/// —— 書体見本のように 1 行ごとに組み直す所では、それが組む時間の大半になる。
/// 台帳の枚数が変わった時だけ作り直す。
fn family_is_known(db: &fontdb::Database, family: &str) -> bool {
    static KNOWN: std::sync::OnceLock<std::sync::Mutex<(usize, std::collections::HashSet<String>)>> = std::sync::OnceLock::new();
    let mut cache = KNOWN
        .get_or_init(|| std::sync::Mutex::new((usize::MAX, std::collections::HashSet::new())))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache.0 != db.len() {
        cache.1 = db.faces().flat_map(|f| f.families.iter().map(|(name, _)| name.clone())).collect();
        cache.0 = db.len();
    }
    cache.1.contains(family)
}

/// family が台帳に無ければ path の file を足す。path も読めなければ、その時だけ誤り。
fn ensure_font(system: &mut FontSystem, font: &GlyphFont) -> Result<(), TextShapeError> {
    let known = |db: &fontdb::Database| family_is_known(db, &font.family);
    if known(system.db()) || font.path.is_empty() {
        return Ok(());
    }
    system
        .db_mut()
        .load_font_file(&font.path)
        .map_err(|source| TextShapeError::FontFile { path: font.path.clone(), source })
}

pub struct StyledText<'a> {
    pub text: &'a str,
    pub font: &'a GlyphFont,
    pub layout: &'a TextLayout,
    pub style: usize,
}

pub fn shape_text(content: &str, font: &GlyphFont, layout: &TextLayout) -> Result<ShapedText, TextShapeError> {
    shape_rich_text(&[StyledText { text: content, font, layout, style: 0 }], layout)
}

pub fn shape_rich_text(spans: &[StyledText<'_>], layout: &TextLayout) -> Result<ShapedText, TextShapeError> {
    let mut guard = font_system();
    let font_system = &mut *guard;
    let attributes = span_attributes(font_system, spans)?;
    let metrics = Metrics::new(layout.size, layout.line_height.unwrap_or(layout.size * 1.2));
    let align = to_align(layout.justify);

    let mut buffer = Buffer::new(font_system, metrics);
    // 幅が無いと Align::Center / Right が常に 0 補正になる。幅は wrap_size か枠の幅。
    buffer.set_size(layout.wrap_width, None);
    buffer.set_wrap(if layout.wrap { cosmic_text::Wrap::WordOrGlyph } else { cosmic_text::Wrap::None });
    let default_attrs = attributes.first().cloned().unwrap_or_else(Attrs::new);
    buffer.set_rich_text(spans.iter().zip(attributes.iter()).map(|(span, attrs)| (span.text, attrs.clone())), &default_attrs, Shaping::Advanced, Some(align));
    buffer.shape_until_scroll(font_system, false);

    let mut swash_cache = SwashCache::new();
    let mut out = ShapedText::default();
    let mut glyph_ordinal = 0usize;
    let content: String = spans.iter().map(|s| s.text).collect();
    let line_starts: Vec<usize> = std::iter::once(0).chain(content.match_indices('\n').map(|(i, _)| i + 1)).collect();
    for run in buffer.layout_runs() {
        let base = line_starts.get(run.line_i).copied().unwrap_or(0);
        emit_run(font_system, &mut swash_cache, &run, 0.0, 0.0, base, &mut glyph_ordinal, &mut out);
    }
    Ok(out)
}

/// 障害物を避けて流す組み方(CSS `shape-outside`、CSS Exclusions の `wrap-flow: both`)。
/// 行ごとに `segments(行の上, 行の下)` が空いている横の区間を返し、行はその区間を左から順に埋める。
/// 区間ごとに残りの文字をその幅で組み、1 行目だけを取る(折り返しの規則は cosmic-text のまま)。
/// 語が入らない区間は飛ばし、どの区間にも入らない行は 1 行下へ(CSS の float の横に入らない語は下へ)。
pub fn shape_rich_text_around(
    spans: &[StyledText<'_>],
    layout: &TextLayout,
    segments: &dyn Fn(f32, f32) -> Vec<(f32, f32)>,
) -> Result<ShapedText, TextShapeError> {
    let mut guard = font_system();
    let font_system = &mut *guard;
    let attributes = span_attributes(font_system, spans)?;
    let align = to_align(layout.justify);
    let full = layout.wrap_width.unwrap_or(f32::MAX);
    let min_size = spans.iter().map(|s| s.layout.size).fold(layout.size, f32::min).max(1.0);
    // 文字全体を 1 本に、スタイルの切れ目は byte で持つ。
    let text: String = spans.iter().map(|s| s.text).collect();
    let mut bounds = Vec::with_capacity(spans.len());
    let mut at = 0usize;
    for span in spans {
        bounds.push((at, at + span.text.len()));
        at += span.text.len();
    }
    let pieces = |from: usize, to: usize| -> Vec<(&str, Attrs)> {
        spans.iter().zip(&bounds).zip(&attributes).filter_map(|((_, &(a, b)), attrs)| {
            let (a, b) = (a.max(from), b.min(to));
            (a < b).then(|| (&text[a..b], attrs.clone()))
        }).collect()
    };
    let default_attrs = attributes.first().cloned().unwrap_or_else(Attrs::new);
    let line_height = layout.line_height.unwrap_or(layout.size * 1.2);

    let mut swash_cache = SwashCache::new();
    let mut out = ShapedText::default();
    let mut glyph_ordinal = 0usize;
    let mut start = 0usize;
    let mut top = 0.0f32;
    let mut rows = 0;
    while start < text.len() && rows < 4096 {
        rows += 1;
        // 段落の終わり(改行)まで。
        let paragraph_end = text[start..].find('\n').map_or(text.len(), |i| start + i);
        if start == paragraph_end {
            start += 1;
            top += line_height;
            continue;
        }
        let spaces = |mut i: usize| { while i < paragraph_end && text[i..].starts_with(' ') { i += 1; } i };
        let mut row_height = line_height;
        let open = segments(top, top + line_height);
        let only = open.len() == 1 && (open[0].1 - open[0].0) >= full - 0.5;
        for (x0, x1) in open {
            let width = x1 - x0;
            if start >= paragraph_end || width < min_size {
                continue;
            }
            // 組む窓は区間に入る字数より十分長く(残り全部を毎回組まない)。
            let window = ((width / (min_size * 0.25)) as usize + 32).min(paragraph_end - start);
            let mut end = start + window;
            while !text.is_char_boundary(end) { end += 1; }
            let mut buffer = Buffer::new(font_system, Metrics::new(layout.size, line_height));
            buffer.set_size(Some(width), None);
            buffer.set_wrap(if only { cosmic_text::Wrap::WordOrGlyph } else { cosmic_text::Wrap::Word });
            buffer.set_rich_text(pieces(start, end), &default_attrs, Shaping::Advanced, Some(align));
            buffer.shape_until_scroll(font_system, false);
            let Some(run) = buffer.layout_runs().next() else { continue };
            let Some(consumed) = run.glyphs.iter().map(|g| g.end).max() else { continue };
            if run.line_w > width + 0.5 && !only {
                continue;
            }
            emit_run(font_system, &mut swash_cache, &run, x0, top, start, &mut glyph_ordinal, &mut out);
            row_height = row_height.max(run.line_height);
            start = spaces(start + consumed);
        }
        if start >= paragraph_end && paragraph_end < text.len() {
            start = paragraph_end + 1;
        }
        top += row_height;
    }
    Ok(out)
}

fn to_align(justify: TextJustify) -> Align {
    match justify {
        TextJustify::Left => Align::Left,
        TextJustify::Right => Align::Right,
        TextJustify::Center => Align::Center,
    }
}

/// 組んだ 1 行を輪郭と行の寸法へ(`dx`・`dy` だけずらして)。
fn emit_run(font_system: &mut FontSystem, swash_cache: &mut SwashCache, run: &cosmic_text::LayoutRun<'_>, dx: f32, dy: f32, byte_base: usize, glyph_ordinal: &mut usize, out: &mut ShapedText) {
    let mut glyph_xs = Vec::with_capacity(run.glyphs.len());
    let mut glyph_bytes = Vec::with_capacity(run.glyphs.len());
    for glyph in run.glyphs {
        glyph_bytes.push(byte_base + glyph.start);
        let pen_x = dx + glyph.x + glyph.font_size * glyph.x_offset;
        let pen_y = dy + run.line_y + glyph.y - glyph.font_size * glyph.y_offset;
        glyph_xs.push(dx + glyph.x);
        let cache_key = glyph.physical((0.0, 0.0), 1.0).cache_key;
        let ordinal = *glyph_ordinal;
        *glyph_ordinal += 1;
        let Some(commands) = swash_cache.get_outline_commands(font_system, cache_key) else {
            continue; // 空白など、輪郭を持たない glyph。
        };
        let before = out.contours.len();
        commands_to_contours(commands, pen_x, pen_y, &mut out.contours);
        out.contour_styles.extend(std::iter::repeat_n(glyph.metadata, out.contours.len() - before));
        out.contour_glyphs.extend(std::iter::repeat_n(ordinal, out.contours.len() - before));
    }
    out.lines.push(LineMeasure { baseline_y: dy + run.line_y, width: run.line_w, glyph_xs, glyph_bytes });
}

fn span_attributes<'a>(font_system: &mut FontSystem, spans: &[StyledText<'a>]) -> Result<Vec<Attrs<'a>>, TextShapeError> {
    for span in spans { ensure_font(font_system, span.font)?; }
    let mut attributes = Vec::new();
    for span in spans {
        let mut features = FontFeatures::new();
        for feature in &span.layout.features {
            let tag: &[u8;4] = feature.tag.as_bytes().try_into().map_err(|_| TextShapeError::FeatureTag(feature.tag.clone()))?;
            features.set(FeatureTag::new(tag), feature.value);
        }
        attributes.push(Attrs::new().family(Family::Name(&span.font.family))
            .metrics(Metrics::new(span.layout.size, span.layout.line_height.unwrap_or(span.layout.size * 1.2)))
            .letter_spacing(span.layout.tracking / 1000.0).font_features(features).metadata(span.style));
    }
    Ok(attributes)
}

fn commands_to_contours(commands: &[Command], pen_x: f32, pen_y: f32, out: &mut Vec<Contour>) {
    let at = |x: f32, y: f32| Point {
        x: f64::from(pen_x + x),
        y: f64::from(pen_y - y), // zeno は y-up、正準空間は y-down。
    };
    let mut vertices: Vec<Vertex> = Vec::new();
    for command in commands {
        match *command {
            Command::MoveTo(p) => {
                if !vertices.is_empty() {
                    out.push(Contour {
                        vertices: std::mem::take(&mut vertices),
                        closed: false,
                    });
                }
                vertices.push(Vertex::corner(at(p.x, p.y)));
            }
            Command::LineTo(p) => vertices.push(Vertex::corner(at(p.x, p.y))),
            Command::CurveTo(c1, c2, p) => {
                let to = at(p.x, p.y);
                if let Some(last) = vertices.last_mut() {
                    last.out_tangent = at(c1.x, c1.y).sub(last.point);
                }
                vertices.push(Vertex {
                    point: to,
                    in_tangent: at(c2.x, c2.y).sub(to),
                    out_tangent: Point::ZERO,
                });
            }
            Command::QuadTo(q, p) => {
                let to = at(p.x, p.y);
                let control = at(q.x, q.y);
                if let Some(last) = vertices.last_mut() {
                    last.out_tangent = control.sub(last.point).scale(2.0 / 3.0);
                }
                vertices.push(Vertex {
                    point: to,
                    in_tangent: control.sub(to).scale(2.0 / 3.0),
                    out_tangent: Point::ZERO,
                });
            }
            Command::Close => {
                if vertices.len() > 1 {
                    let first = vertices[0];
                    let last = vertices[vertices.len() - 1];
                    let dx = last.point.x - first.point.x;
                    let dy = last.point.y - first.point.y;
                    if (dx * dx + dy * dy) < 1e-9 {
                        vertices[0].in_tangent = last.in_tangent;
                        vertices.pop();
                    }
                }
                if !vertices.is_empty() {
                    out.push(Contour {
                        vertices: std::mem::take(&mut vertices),
                        closed: true,
                    });
                }
            }
        }
    }
    if !vertices.is_empty() {
        out.push(Contour {
            vertices,
            closed: false,
        });
    }
}

/// A specimen must not silently display a different fallback family.
pub fn font_supports_sample(family: &str, text: &str) -> bool {
    let mut system = font_system();
    let Some(id) = system.db().query(&fontdb::Query {
        families: &[fontdb::Family::Name(family)], ..Default::default()
    }) else { return false };
    let Some(font) = system.get_font(id, fontdb::Weight::NORMAL) else { return false };
    text.chars().filter(|c| !c.is_whitespace()).all(|c| font.as_swash().charmap().map(c) != 0)
}

/// What a family is, read from the font itself once: how many faces and
/// weights, whether it is monospaced, its variable axes, the scripts its
/// charmap covers, and whether it carries colour glyphs. Labels for a shelf,
/// not shaping input.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct FontFacts {
    pub family: String,
    pub styles: usize,
    pub weights: usize,
    pub monospaced: bool,
    /// fvar の軸の tag(`wght` など)。無ければ静的。
    pub axes: Vec<String>,
    /// 覆う文字種の名前(Latin / Kana / Kanji / Hangul / Cyrillic / Arabic)。
    pub scripts: Vec<&'static str>,
    pub color: bool,
}

const SCRIPT_PROBES: &[(&str, char)] = &[("Latin", 'A'), ("Kana", 'あ'), ("Kanji", '永'), ("Hangul", '한'), ("Cyrillic", 'Я'), ("Arabic", 'ع')];

pub fn font_facts() -> &'static [FontFacts] {
    static FACTS: std::sync::OnceLock<Vec<FontFacts>> = std::sync::OnceLock::new();
    FACTS.get_or_init(|| {
        let mut system = font_system();
        let mut by_family: std::collections::BTreeMap<String, Vec<(fontdb::ID, fontdb::Weight, bool)>> = std::collections::BTreeMap::new();
        for face in system.db().faces() {
            if let Some((name, _)) = face.families.first() {
                by_family.entry(name.clone()).or_default().push((face.id, face.weight, face.monospaced));
            }
        }
        by_family.into_iter().map(|(family, faces)| {
            let weights = faces.iter().map(|f| f.1).collect::<std::collections::BTreeSet<_>>().len();
            let monospaced = faces.iter().any(|f| f.2);
            let (axes, scripts, color) = match system.get_font(faces[0].0, faces[0].1) {
                Some(font) => {
                    let swash = font.as_swash();
                    let axes = swash.variations().map(|v| String::from_utf8_lossy(&v.tag().to_be_bytes()).into_owned()).collect();
                    let charmap = swash.charmap();
                    let scripts = SCRIPT_PROBES.iter().filter(|(_, c)| charmap.map(*c) != 0).map(|(name, _)| *name).collect();
                    (axes, scripts, swash.color_palettes().len() > 0)
                }
                None => (Vec::new(), Vec::new(), false),
            };
            FontFacts { family, styles: faces.len(), weights, monospaced, axes, scripts, color }
        }).collect()
    })
}

#[cfg(test)]
mod facts_tests {
    use super::*;

    /// 台帳の書体が事実を持つ: ヒラギノ角ゴは仮名と漢字を覆い、10 の太さの静的書体。
    #[test]
    fn facts_come_from_the_font_itself() {
        let facts = font_facts();
        let Some(hiragino) = facts.iter().find(|f| f.family == "Hiragino Sans") else {
            eprintln!("skipped: Hiragino Sans missing");
            return;
        };
        assert!(hiragino.scripts.contains(&"Kana") && hiragino.scripts.contains(&"Kanji") && hiragino.scripts.contains(&"Latin"));
        assert!(hiragino.weights >= 5, "{hiragino:?}");
        assert!(hiragino.axes.is_empty());
        assert!(!hiragino.monospaced);
        assert!(facts.iter().all(|f| f.styles >= 1));
    }
}
