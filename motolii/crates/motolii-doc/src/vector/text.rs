
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
    pub size: f32,
    pub line_height: Option<f32>,
    pub tracking: f32,
    pub justify: TextJustify,
    pub features: Vec<TextFeature>,
}

impl TextLayout {
    pub fn new(size: f32) -> Self {
        Self {
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
    pub lines: Vec<LineMeasure>,
}

pub fn shape_text(
    content: &str,
    font: &GlyphFont,
    layout: &TextLayout,
) -> Result<ShapedText, TextShapeError> {
    let mut db = fontdb::Database::new();
    db.load_font_file(&font.path)
        .map_err(|source| TextShapeError::FontFile {
            path: font.path.clone(),
            source,
        })?;
    let mut font_system = FontSystem::new_with_locale_and_db("en-US".to_owned(), db);

    let line_height = layout.line_height.unwrap_or(layout.size * 1.2);
    let metrics = Metrics::new(layout.size, line_height);

    let mut features = FontFeatures::new();
    for feature in &layout.features {
        let bytes: &[u8] = feature.tag.as_bytes();
        let tag: &[u8; 4] = bytes
            .try_into()
            .map_err(|_| TextShapeError::FeatureTag(feature.tag.clone()))?;
        features.set(FeatureTag::new(tag), feature.value);
    }

    let attrs = Attrs::new()
        .family(Family::Name(&font.family))
        .metrics(metrics)
        .letter_spacing(layout.tracking / 1000.0)
        .font_features(features);

    let align = match layout.justify {
        TextJustify::Left => Align::Left,
        TextJustify::Right => Align::Right,
        TextJustify::Center => Align::Center,
    };

    let mut buffer = Buffer::new(&mut font_system, metrics);
    buffer.set_size(None, None); // point text(折返し無し)。wrap は次切片(module doc 参照)。
    buffer.set_text(content, &attrs, Shaping::Advanced, Some(align));
    buffer.shape_until_scroll(&mut font_system, false);

    let mut swash_cache = SwashCache::new();
    let mut contours = Vec::new();
    let mut lines = Vec::new();
    for run in buffer.layout_runs() {
        let mut glyph_xs = Vec::with_capacity(run.glyphs.len());
        for glyph in run.glyphs {
            let pen_x = glyph.x + glyph.font_size * glyph.x_offset;
            let pen_y = run.line_y + glyph.y - glyph.font_size * glyph.y_offset;
            glyph_xs.push(glyph.x);
            let cache_key = glyph.physical((0.0, 0.0), 1.0).cache_key;
            let Some(commands) = swash_cache.get_outline_commands(&mut font_system, cache_key)
            else {
                continue; // 空白など、輪郭を持たない glyph。
            };
            commands_to_contours(commands, pen_x, pen_y, &mut contours);
        }
        lines.push(LineMeasure {
            baseline_y: run.line_y,
            width: run.line_w,
            glyph_xs,
        });
    }
    Ok(ShapedText { contours, lines })
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
