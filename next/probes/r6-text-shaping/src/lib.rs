//! owns: R6 probe の測定器具 — 「文字列 + store の text 意味([`TextDocumentStyle`])
//!       → cosmic-text(shaping/layout)→ glyph outline(swash/zeno)→
//!       `motolii_vector::PathSource::Bezier` → premultiplied RGBA8 → PNG」の
//!       1本道が通るか、lh(行送り)/tr(トラッキング)が**数値で**効くかの実証。
//!       長年 queue にあった「字形描画は harfrust/fontique の依存裁定が先」を
//!       閉じるための器具で、判定文書は
//!       `docs/reviews/2026-08-22-text-rendering-route-probe.md`。
//!
//! # なぜ cosmic-text か(この probe が確かめる仮説)
//!
//! iced fork(`iced_graphics`)が cosmic-text 0.19.0 を既に依存グラフへ引いている。
//! ここで direct 依存に昇格しても Cargo.lock 上は同一 package に unify され、
//! **依存の増分がゼロ**(notify / image の裁定117 昇格と同じ形)。さらに UI の
//! 文字と stage の文字が同じ shaper を通る = フォント解決の正本が1つで済む。
//!
//! # 変換の要点(コードの形に落ちている判定材料)
//!
//! - **outline は swash 経由で取れる**: `SwashCache::get_outline_commands` が
//!   `zeno::Command`(MoveTo/LineTo/QuadTo/CurveTo/Close)を返す。zeno は y-up、
//!   motolii-vector の正準空間は y-down なので baseline 回りで y を反転する。
//! - **QuadTo(TrueType の2次)は3次へ次数上げ**する(c1 = p0 + ⅔(q−p0)、
//!   c2 = p1 + ⅔(q−p1) — 厳密変換で近似ではない)。[`Vertex`] は Lottie `bezier`
//!   と同型の「頂点相対 cubic ハンドル」なので、そのまま `PathSource::Bezier` に載る。
//! - **tr(トラッキング)の単位**: store の `tracking` は Lottie `text-document tr`
//!   = AE の tracking(1/1000 em)。cosmic-text の `letter_spacing` は em 正規化
//!   された advance へ足される(shape.rs 実測: `x_advance / font_scale + spacing`、
//!   layout で `* font_size`)ので、**`tracking / 1000.0` の1行で写る**。
//! - **lh(行送り)**: `Metrics::line_height` がそのもの。`None`(フォントメトリクス
//!   準拠)はこの probe では `size * 1.2` に固定 — 実装切片では swash のフォント
//!   メトリクスから引く(判定文書 §切片割り)。
//! - **フォントは path で解決する**([`FontRef::path`] の意味そのまま):
//!   `fontdb::Database::load_font_file` → `FontSystem::new_with_locale_and_db`。
//!   システムフォント走査をしないので試験が決定的で速い。
//!
//! # この probe が**やらない**こと(実装切片の仕事)
//!
//! range-selector / animator の適用、style 表 + runs の rich text
//! (`Buffer::set_rich_text` が受け口 — API 存在は判定文書に記録)、可変フォント軸
//! (cosmic-text の公開 API に fvar 口が無い — 判定文書 §gap)、縦書き。

use cosmic_text::fontdb;
use cosmic_text::{
    Align, Attrs, Buffer, Command, Family, FeatureTag, FontFeatures, FontSystem, Metrics, Shaping,
    SwashCache,
};
use motolii_store::{TextDocumentStyle, TextJustify};
use motolii_vector::{
    render, Brush, Canvas, Contour, Fill, FillRule, PathSource, Point, Raster, Rgb, Shape, Stroke,
    Vertex,
};

/// この probe の入り口が返しうる失敗。**黙って空の画を返さない**(裁定37 と同じ理由 —
/// フォントが読めなかったのと「字が無い」を同義にすると、壊れた入力が既定値へ落ちる)。
#[derive(Debug, thiserror::Error)]
pub enum TextShapeError {
    #[error("font file {path} could not be read: {source}")]
    FontFile {
        path: String,
        source: std::io::Error,
    },
    #[error("OpenType feature tag {0:?} is not 4 bytes")]
    FeatureTag(String),
    #[error(transparent)]
    Vector(#[from] motolii_vector::VectorError),
}

/// 1行ぶんの実測値。lh/tr の数値検証はこの構造体だけで完結する(画素を数えて
/// 「なんとなく広がった」を見るのではなく、layout の座標そのものを固定する)。
#[derive(Debug, Clone, PartialEq)]
pub struct LineMeasure {
    /// baseline の y(正準空間、上原点)。行送り lh の検証はこの差分。
    pub baseline_y: f32,
    /// 行の幅(cosmic-text `LayoutRun::line_w`)。
    pub width: f32,
    /// 各 glyph のペン位置 x(行内オフセット)。トラッキング tr の検証はこの差分。
    pub glyph_xs: Vec<f32>,
}

/// 組み上がったテキスト。輪郭(描く物)と実測値(検証する物)を一緒に返す。
#[derive(Debug, Clone, PartialEq)]
pub struct ShapedText {
    /// 全 glyph の輪郭。正準空間(y-down)、原点 = 1行目の組み始め(左上)。
    pub contours: Vec<Contour>,
    pub lines: Vec<LineMeasure>,
}

/// 文字列 + スタイル表の1行 → 輪郭と実測値。改行は cosmic-text がそのまま行分割する。
pub fn shape_text(
    content: &str,
    style: &TextDocumentStyle,
    justify: TextJustify,
) -> Result<ShapedText, TextShapeError> {
    // フォントは FontRef.path で解決する(store の意味そのまま)。システム走査はしない。
    let mut db = fontdb::Database::new();
    db.load_font_file(&style.font.path)
        .map_err(|source| TextShapeError::FontFile {
            path: style.font.path.clone(),
            source,
        })?;
    let mut font_system = FontSystem::new_with_locale_and_db("en-US".to_owned(), db);

    // lh: None = フォントメトリクス準拠(実装切片で swash から引く)。probe は 1.2em 固定。
    let line_height = style.line_height.unwrap_or(style.size * 1.2);
    let metrics = Metrics::new(style.size, line_height);

    // OpenType feature(`text-style-feature`)はタグ + 値がそのまま写る。
    let mut features = FontFeatures::new();
    for feature in &style.features {
        let bytes: &[u8] = feature.tag.as_bytes();
        let tag: &[u8; 4] = bytes
            .try_into()
            .map_err(|_| TextShapeError::FeatureTag(feature.tag.clone()))?;
        features.set(FeatureTag::new(tag), feature.value);
    }

    // tr: AE tracking(1/1000 em)→ cosmic-text letter_spacing(em)。
    let attrs = Attrs::new()
        .family(Family::Name(&style.font.family))
        .metrics(metrics)
        .letter_spacing(style.tracking / 1000.0)
        .font_features(features);

    let align = match justify {
        TextJustify::Left => Align::Left,
        TextJustify::Right => Align::Right,
        TextJustify::Center => Align::Center,
    };

    let mut buffer = Buffer::new(&mut font_system, metrics);
    buffer.set_size(None, None); // point text(折返し無し)。wrap は `wrap_size` の切片で。
    buffer.set_text(content, &attrs, Shaping::Advanced, Some(align));
    buffer.shape_until_scroll(&mut font_system, false);

    let mut swash_cache = SwashCache::new();
    let mut contours = Vec::new();
    let mut lines = Vec::new();
    for run in buffer.layout_runs() {
        let mut glyph_xs = Vec::with_capacity(run.glyphs.len());
        for glyph in run.glyphs {
            // ペン位置は float のまま使う(LayoutGlyph doc の「vectorial text」経路)。
            // physical() は hinting 用に整数へ丸めるので、cache_key だけ借りる。
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

/// 輪郭 + スタイル表の1行 → motolii-vector の図形1つ。fill/stroke の色・幅・
/// fill rule(TrueType/CFF の輪郭は winding 前提なので NonZero)が store から写る。
pub fn to_shape(shaped: &ShapedText, style: &TextDocumentStyle) -> Shape {
    let mut shape = Shape::new(PathSource::Bezier(shaped.contours.clone()));
    shape.fill = Some(Fill {
        brush: Brush::Solid(Rgb {
            r: style.fill[0],
            g: style.fill[1],
            b: style.fill[2],
        }),
        rule: FillRule::NonZero,
        opacity: style.fill[3],
        hidden: false,
    });
    if let Some(stroke_color) = style.stroke_color {
        if style.stroke_width > 0.0 {
            shape.stroke = Some(Stroke {
                brush: Brush::Solid(Rgb {
                    r: stroke_color[0],
                    g: stroke_color[1],
                    b: stroke_color[2],
                }),
                width: f64::from(style.stroke_width),
                opacity: stroke_color[3],
                ..Stroke::default()
            });
        }
    }
    shape
}

/// 図形 → premultiplied RGBA8(上左原点の板)。`motolii_vector::render` の薄い皮 —
/// ラスタライズ経路は上流の1本のまま、ここでは原点だけ組版の左上に合わせる。
pub fn rasterize(shape: &Shape, width: u32, height: u32) -> Result<Raster, TextShapeError> {
    let canvas = Canvas {
        width,
        height,
        origin_x: 0,
        origin_y: 0,
    };
    Ok(render(shape, &canvas)?)
}

/// zeno::Command 列(y-up、glyph ローカル)→ [`Contour`] 列(y-down、正準空間)。
///
/// - `QuadTo` は次数上げ(⅔則)で cubic へ — 厳密変換。
/// - `Close` で「終点 ≈ 始点」なら終点を始点へ併合する(ゼロ長の閉じ辺を作らない)。
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
                    // フォント輪郭は必ず Close で終わるが、来ても落とさず開輪郭で残す。
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
                    last.out_tangent = at(c1.x, c1.y).sub_point(last.point);
                }
                vertices.push(Vertex {
                    point: to,
                    in_tangent: at(c2.x, c2.y).sub_point(to),
                    out_tangent: Point::ZERO,
                });
            }
            Command::QuadTo(q, p) => {
                let to = at(p.x, p.y);
                let control = at(q.x, q.y);
                if let Some(last) = vertices.last_mut() {
                    // c1 = p0 + ⅔(q − p0) — 頂点相対ハンドルなので ⅔(q − p0) そのもの。
                    last.out_tangent = control.sub_point(last.point).scale_by(2.0 / 3.0);
                }
                vertices.push(Vertex {
                    point: to,
                    in_tangent: control.sub_point(to).scale_by(2.0 / 3.0),
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

/// [`Point`] の算術は crate 内 `pub(crate)` なので、ここで足す最小限の2つ。
trait PointExt {
    fn sub_point(self, other: Point) -> Point;
    fn scale_by(self, s: f64) -> Point;
}

impl PointExt for Point {
    fn sub_point(self, other: Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
    fn scale_by(self, s: f64) -> Point {
        Point {
            x: self.x * s,
            y: self.y * s,
        }
    }
}
