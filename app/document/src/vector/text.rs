
//! 文字の組み方の設定 — 書類に保存される値。字を並べる仕事は絵の側(`picture::shaping`)。

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

/// CSS `text-autospace`(CSS Text 4 §8.4): 漢字と欧文の字・数字の境に、字送りの 1/8(0.125ic)を足す。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum TextAutospace {
    #[default]
    Normal,
    NoAutospace,
}

/// CSS `text-spacing-trim`(CSS Text 4 §8.5): 全角の約物を、行の端と隣どうしで半角に詰める。
/// 隣どうしは font の `chws`、行頭は行分割の後で左半分を削る(halt と同じ量)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum TextSpacingTrim {
    #[default]
    Normal,
    SpaceAll,
    SpaceFirst,
    TrimStart,
}

/// CSS `hanging-punctuation`(CSS Text 4 §9.2.1): `none | [ first || [ force-end | allow-end ] || last ]`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct HangingPunctuation {
    pub first: bool,
    pub end: HangEnd,
    pub last: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum HangEnd {
    #[default]
    None,
    AllowEnd,
    ForceEnd,
}

impl TextAutospace {
    pub const CHOICES: &'static [&'static str] = &["Normal", "No Autospace"];
    pub fn to_enum_value(self) -> i64 { self as i64 }
    pub fn from_enum_value(v: i64) -> Option<Self> { [Self::Normal, Self::NoAutospace].get(usize::try_from(v).ok()?).copied() }
}

impl TextSpacingTrim {
    pub const CHOICES: &'static [&'static str] = &["Normal", "Space All", "Space First", "Trim Start"];
    pub fn to_enum_value(self) -> i64 { self as i64 }
    pub fn from_enum_value(v: i64) -> Option<Self> { [Self::Normal, Self::SpaceAll, Self::SpaceFirst, Self::TrimStart].get(usize::try_from(v).ok()?).copied() }
}

impl HangingPunctuation {
    /// 文法の順(first, end, last)で全部の組合せ。
    pub const CHOICES: &'static [&'static str] = &[
        "None", "First", "Allow End", "Force End", "Last", "First Allow End", "First Force End", "First Last",
        "Allow End Last", "Force End Last", "First Allow End Last", "First Force End Last",
    ];
    const TABLE: [(bool, HangEnd, bool); 12] = [
        (false, HangEnd::None, false), (true, HangEnd::None, false), (false, HangEnd::AllowEnd, false), (false, HangEnd::ForceEnd, false),
        (false, HangEnd::None, true), (true, HangEnd::AllowEnd, false), (true, HangEnd::ForceEnd, false), (true, HangEnd::None, true),
        (false, HangEnd::AllowEnd, true), (false, HangEnd::ForceEnd, true), (true, HangEnd::AllowEnd, true), (true, HangEnd::ForceEnd, true),
    ];
    pub fn to_enum_value(self) -> i64 {
        Self::TABLE.iter().position(|&(first, end, last)| first == self.first && end == self.end && last == self.last).unwrap_or(0) as i64
    }
    pub fn from_enum_value(v: i64) -> Option<Self> {
        Self::TABLE.get(usize::try_from(v).ok()?).map(|&(first, end, last)| Self { first, end, last })
    }
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
    pub autospace: TextAutospace,
    pub spacing_trim: TextSpacingTrim,
    pub hanging: HangingPunctuation,
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
            autospace: TextAutospace::default(),
            spacing_trim: TextSpacingTrim::default(),
            hanging: HangingPunctuation::default(),
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
