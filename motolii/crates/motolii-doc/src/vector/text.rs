
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
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ShapedText {
    pub contours: Vec<Contour>,
    pub contour_styles: Vec<usize>,
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

/// family が台帳に無ければ path の file を足す。path も読めなければ、その時だけ誤り。
fn ensure_font(system: &mut FontSystem, font: &GlyphFont) -> Result<(), TextShapeError> {
    let known = |db: &fontdb::Database| {
        db.faces().any(|f| f.families.iter().any(|(name, _)| name == &font.family))
    };
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
    for span in spans { ensure_font(font_system, span.font)?; }
    let metrics = Metrics::new(layout.size, layout.line_height.unwrap_or(layout.size * 1.2));
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
    let align = match layout.justify {
        TextJustify::Left => Align::Left,
        TextJustify::Right => Align::Right,
        TextJustify::Center => Align::Center,
    };

    let mut buffer = Buffer::new(font_system, metrics);
    // 幅が無いと Align::Center / Right が常に 0 補正になる。幅は wrap_size か枠の幅。
    buffer.set_size(layout.wrap_width, None);
    buffer.set_wrap(if layout.wrap { cosmic_text::Wrap::WordOrGlyph } else { cosmic_text::Wrap::None });
    let default_attrs = attributes.first().cloned().unwrap_or_else(Attrs::new);
    buffer.set_rich_text(spans.iter().zip(attributes.iter()).map(|(span, attrs)| (span.text, attrs.clone())), &default_attrs, Shaping::Advanced, Some(align));
    buffer.shape_until_scroll(font_system, false);

    let mut swash_cache = SwashCache::new();
    let mut contours = Vec::new();
    let mut contour_styles = Vec::new();
    let mut lines = Vec::new();
    for run in buffer.layout_runs() {
        let mut glyph_xs = Vec::with_capacity(run.glyphs.len());
        for glyph in run.glyphs {
            let pen_x = glyph.x + glyph.font_size * glyph.x_offset;
            let pen_y = run.line_y + glyph.y - glyph.font_size * glyph.y_offset;
            glyph_xs.push(glyph.x);
            let cache_key = glyph.physical((0.0, 0.0), 1.0).cache_key;
            let Some(commands) = swash_cache.get_outline_commands(font_system, cache_key)
            else {
                continue; // 空白など、輪郭を持たない glyph。
            };
            let before = contours.len();
            commands_to_contours(commands, pen_x, pen_y, &mut contours);
            contour_styles.extend(std::iter::repeat_n(glyph.metadata, contours.len() - before));
        }
        lines.push(LineMeasure {
            baseline_y: run.line_y,
            width: run.line_w,
            glyph_xs,
        });
    }
    Ok(ShapedText { contours, contour_styles, lines })
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
