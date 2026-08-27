//! owns: 字形→輪郭の口(cosmic-text 0.19 でシェイピング → swash で輪郭取得 →
//!       3次ベジエへ次数上げ → [`Contour`])。`next/probes/r6-text-shaping` の実証
//!       (`docs/reviews/2026-08-22-text-rendering-route-probe.md`、裁定190)を
//!       ほぼそのまま昇格した — 検証済みの1本道を書き直さない。
//!
//! # 依存増分ゼロ(裁定190 の判定材料そのまま、Cargo.toml のコメント参照)
//!
//! cosmic-text は `workspace.dependencies` へ足さない(probe と同じ理由)——
//! iced fork(`iced_graphics`)が既に 0.19.0 を引いており、この crate だけの
//! direct 依存へ昇格しても Cargo.lock 上は同一 package に unify される。
//!
//! # 独立した名前空間にした理由
//!
//! [`coverage`](crate::coverage)と同じ理由 — [`GlyphFont`]/[`TextLayout`]/
//! [`TextFeature`] は crate 根の「1つの Shape の記述」語彙(`Shape`/`PathSource`
//! 等)とは別の関心事(字形シェイピングの入力)なので、crate 根の `pub use` には
//! 混ぜない。
//!
//! OWNS-JUSTIFICATION(B): 探索対象=`next/probes/r6-text-shaping` の実証(裁定190)
//! — cosmic-text→swash→3次ベジエの1本道を probe で実際に検証済みの上で、自前分は
//! 「次数上げ(quad→cubic)」の変換だけに絞っている(裁定215 棚卸し 2026-08-23
//! #17、所有範囲を最小限に絞った模範例)。
//!
//! # この module が持つ語彙・持たない語彙
//!
//! **`motolii-store` を知らない**(`motolii-store` が `motolii-vector` を引く
//! 片方向 — crate root の doc「保存」節参照。逆方向の依存は循環になる)。
//! [`GlyphFont`]/[`TextLayout`]/[`TextFeature`] は `motolii_store::FontRef`/
//! `TextDocumentStyle`/`TextStyleFeature` と**値の意味は同じ**だが独立した型 ——
//! 呼び手(`motolii-engine`)が store の型からここへ値を写す(`motolii-engine`の
//! `mask.rs` が `ResolvedMask` → `motolii_vector::Shape` を橋渡しするのと同じ形)。
//! fill/stroke はここでは持たない — 塗りは呼び手が [`ShapedText::contours`] から
//! 普通の `Shape`(`PathSource::Bezier`)を組み立てて決める仕事。
//!
//! # やらないこと(probe の gap のまま、裁定190 切片1+2の範囲外)
//!
//! rich-text(style 表 + `runs` の分割適用)・可変フォント軸・縦書き・折返し
//! (`wrap_size`、point text 固定)。**フォントが読めない時は黙って既定フォントへ
//! 落とさない**([`TextShapeError::FontFile`]、裁定37「無い」と「壊れている」を
//! 同義にしない)。空文字列は**エラーではない**——輪郭0個の [`ShapedText`] を返す
//! (`PathSource::PolyStar` の `points < 3` が空パスを返すのと同じ扱い、描く物が
//! 無いのは壊れた入力ではない)。

use cosmic_text::fontdb;
use cosmic_text::{
    Align, Attrs, Buffer, Command, Family, FeatureTag, FontFeatures, FontSystem, Metrics, Shaping,
    SwashCache,
};

use crate::{Contour, Point, Vertex};

/// フォント参照。`motolii_store::FontRef` と値の意味は同じ(module doc 参照)。
///
/// **path で解決する**(`fontdb::Database::load_font_file`) — システムフォント
/// 走査はしない。試験が決定的で速い(probe の実測どおり)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphFont {
    /// フォント実体の在処。
    pub path: String,
    /// family 名。実体解決のキー。
    pub family: String,
}

/// 水平揃え。`motolii_store::TextJustify` と同じ3値(独立した型 — module doc 参照)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextJustify {
    #[default]
    Left,
    Right,
    Center,
}

/// OpenType feature 1本。`motolii_store::TextStyleFeature` と同じ形。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextFeature {
    /// 4文字の feature タグ(`"liga"`/`"palt"` 等)。
    pub tag: String,
    pub value: u32,
}

/// 組版パラメータ。`motolii_store::TextDocumentStyle` のうちシェイピングが直接
/// 必要とする分だけ(fill/stroke は塗りの話 — module doc 参照)。
#[derive(Debug, Clone, PartialEq)]
pub struct TextLayout {
    /// フォントサイズ(pt)。
    pub size: f32,
    /// 行送り。`None` = フォントメトリクス未使用の既定(`size * 1.2`)。実測に
    /// 基づくフォントメトリクス由来の既定値は次切片(probe module doc の
    /// 「なぜ cosmic-text か」節と同じ留保)。
    pub line_height: Option<f32>,
    /// トラッキング。**AE tracking 単位(1/1000 em)** ——
    /// `motolii_store::TextDocumentStyle::tracking` と同じ定義。
    /// `letter_spacing = tracking / 1000.0` の1行で cosmic-text の em 正規化
    /// スペーシングへ写る(probe の実測: advance の差が数値でこの式に一致)。
    pub tracking: f32,
    pub justify: TextJustify,
    /// OpenType feature の静止設定(`text-style-feature`、v1 はキーフレーム化しない)。
    pub features: Vec<TextFeature>,
}

impl TextLayout {
    /// 既定値だけの組版。`size` は呼び手が必ず決めるので既定を持たない。
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

/// この module の入り口が返しうる失敗。**黙って空の画/既定フォントへ落とさない**
/// (裁定37 と同じ理由 — フォントが読めなかったのと「字が無い」を同義にすると、
/// 壊れた入力が既定値へ落ちる)。
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
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ShapedText {
    /// 全 glyph の輪郭。正準空間(y-down)、原点 = 1行目の組み始め(左上)。
    /// 空文字列なら空(エラーではない、module doc 参照)。
    pub contours: Vec<Contour>,
    pub lines: Vec<LineMeasure>,
}

/// 文字列 + フォント + 組版 → 輪郭と実測値。改行は cosmic-text がそのまま行分割する。
pub fn shape_text(
    content: &str,
    font: &GlyphFont,
    layout: &TextLayout,
) -> Result<ShapedText, TextShapeError> {
    // フォントは path で解決する(module doc 参照)。システム走査はしない。
    let mut db = fontdb::Database::new();
    db.load_font_file(&font.path)
        .map_err(|source| TextShapeError::FontFile {
            path: font.path.clone(),
            source,
        })?;
    let mut font_system = FontSystem::new_with_locale_and_db("en-US".to_owned(), db);

    // lh: None = フォントメトリクス準拠の既定(次切片で swash から引く、module doc 参照)。
    let line_height = layout.line_height.unwrap_or(layout.size * 1.2);
    let metrics = Metrics::new(layout.size, line_height);

    // OpenType feature はタグ + 値がそのまま写る。
    let mut features = FontFeatures::new();
    for feature in &layout.features {
        let bytes: &[u8] = feature.tag.as_bytes();
        let tag: &[u8; 4] = bytes
            .try_into()
            .map_err(|_| TextShapeError::FeatureTag(feature.tag.clone()))?;
        features.set(FeatureTag::new(tag), feature.value);
    }

    // tr: AE tracking(1/1000 em)→ cosmic-text letter_spacing(em)。
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
                    // c1 = p0 + ⅔(q − p0) — 頂点相対ハンドルなので ⅔(q − p0) そのもの。
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

#[cfg(test)]
mod tests {
    use super::*;

    const ARIAL: &str = "/System/Library/Fonts/Supplemental/Arial.ttf";

    fn font() -> GlyphFont {
        GlyphFont {
            path: ARIAL.to_owned(),
            family: "Arial".to_owned(),
        }
    }

    #[test]
    fn empty_content_yields_no_contours_and_is_not_an_error() {
        // cosmic-text は空文字列でも「空の1行」を返す(layout 上は行が存在する) —
        // ここで縛るのは「描く輪郭が無い」ことだけで、行の有無までは主張しない。
        let shaped = shape_text("", &font(), &TextLayout::new(96.0)).expect("shape");
        assert!(shaped.contours.is_empty());
        assert!(shaped.lines.iter().all(|l| l.glyph_xs.is_empty()));
    }

    #[test]
    fn missing_font_file_is_an_explicit_error() {
        let bad = GlyphFont {
            path: "/nonexistent/does-not-exist.ttf".to_owned(),
            family: "Nonexistent".to_owned(),
        };
        let err = shape_text("hi", &bad, &TextLayout::new(32.0)).unwrap_err();
        assert!(matches!(err, TextShapeError::FontFile { .. }));
    }
}
