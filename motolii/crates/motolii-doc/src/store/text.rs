
use serde::{Deserialize, Serialize};

use crate::doc::core::RationalTime;

use crate::doc::store::SlotId;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FontRef {
    pub path: String,
    pub fingerprint: Option<String>,
    pub family: String,
    pub style: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextJustify {
    Left,
    Right,
    Center,
}

impl TextJustify {
    pub fn to_enum_value(self) -> i64 {
        match self {
            TextJustify::Left => 0,
            TextJustify::Right => 1,
            TextJustify::Center => 2,
        }
    }

    pub fn from_enum_value(v: i64) -> Option<Self> {
        match v {
            0 => Some(TextJustify::Left),
            1 => Some(TextJustify::Right),
            2 => Some(TextJustify::Center),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContentKeyframe {
    pub t: RationalTime,
    pub content: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ContentTrack {
    keys: Vec<ContentKeyframe>,
}

impl ContentTrack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: ContentKeyframe) {
        match self.keys.binary_search_by(|k| k.t.cmp(&key.t)) {
            Ok(i) => self.keys[i] = key,
            Err(i) => self.keys.insert(i, key),
        }
    }

    pub fn keys(&self) -> &[ContentKeyframe] {
        &self.keys
    }

    pub fn eval(&self, t: RationalTime) -> &str {
        let keys = &self.keys;
        let Some(first) = keys.first() else {
            return "";
        };
        if t <= first.t {
            return &first.content;
        }
        let last = keys.len() - 1;
        if t >= keys[last].t {
            return &keys[last].content;
        }
        let i = match keys.binary_search_by(|k| k.t.cmp(&t)) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        &keys[i].content
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TextStyleId(pub u32);

impl std::fmt::Display for TextStyleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextStyleAxis {
    pub tag: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextStyleFeature {
    pub tag: String,
    pub value: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextDocumentStyle {
    pub id: TextStyleId,
    pub font: FontRef,
    pub size: f32,
    pub fill: [f64; 4],
    pub line_height: Option<f32>,
    pub tracking: f32,
    pub stroke_color: Option<[f64; 4]>,
    pub stroke_width: f32,
    pub stroke_over_fill: bool,
    pub axes: Vec<TextStyleAxis>,
    pub features: Vec<TextStyleFeature>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TextRangeId(pub u32);

impl std::fmt::Display for TextRangeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextBasedOn {
    Characters,
    CharactersExcludingSpaces,
    Words,
    Lines,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextRangeUnits {
    Percent,
    Index,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextShape {
    Square,
    RampUp,
    RampDown,
    Triangle,
    Round,
    Smooth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextGrouping {
    Characters,
    Word,
    Line,
    All,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextRandomize {
    pub seed: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextRangeSelector {
    pub based_on: TextBasedOn,
    pub range_units: TextRangeUnits,
    pub shape: TextShape,
    pub randomize: Option<TextRandomize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextVariationAxis {
    pub tag: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextRange {
    pub id: TextRangeId,
    pub name: String,
    pub selector: TextRangeSelector,
    pub variation_axes: Vec<TextVariationAxis>,
}

fn validate_unique_range_ids(ranges: &[TextRange]) -> Result<(), crate::doc::store::StoreError> {
    for (i, range) in ranges.iter().enumerate() {
        if ranges[..i].iter().any(|other| other.id == range.id) {
            return Err(crate::doc::store::StoreError::Property(format!(
                "text-range id {} が2枚ある",
                range.id
            )));
        }
    }
    Ok(())
}

fn duplicate_tag<'a>(mut tags: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    let mut seen = std::collections::HashSet::new();
    tags.find(|tag| !seen.insert(*tag))
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextAlignmentOptions {
    pub anchor_offset: [f32; 2],
    pub grouping: TextGrouping,
}

impl Default for TextAlignmentOptions {
    fn default() -> Self {
        Self {
            anchor_offset: [0.0, 0.0],
            grouping: TextGrouping::Characters,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextRun {
    pub len: u32,
    pub style: TextStyleId,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextDocument {
    pub content: ContentTrack,
    pub justify: TextJustify,
    pub wrap_size: Option<[f32; 2]>,
    pub styles: Vec<TextDocumentStyle>,
    pub slot_id: Option<SlotId>,
    pub ranges: Vec<TextRange>,
    pub alignment: TextAlignmentOptions,
    pub runs: Vec<TextRun>,
}

pub(crate) fn validate(document: &TextDocument) -> Result<(), crate::doc::store::StoreError> {
    validate_unique_range_ids(&document.ranges)?;
    validate_unique_style_ids(&document.styles)?;
    for range in &document.ranges {
        if let Some(tag) = duplicate_tag(range.variation_axes.iter().map(|a| a.tag.as_str())) {
            return Err(crate::doc::store::StoreError::Property(format!(
                "text-range {} の variation axis タグ「{tag}」が2枚ある",
                range.id
            )));
        }
    }
    for style in &document.styles {
        if let Some(tag) = duplicate_tag(style.axes.iter().map(|a| a.tag.as_str())) {
            return Err(crate::doc::store::StoreError::Property(format!(
                "text-style {} の axis タグ「{tag}」が2枚ある",
                style.id
            )));
        }
        if let Some(tag) = duplicate_tag(style.features.iter().map(|f| f.tag.as_str())) {
            return Err(crate::doc::store::StoreError::Property(format!(
                "text-style {} の feature タグ「{tag}」が2枚ある",
                style.id
            )));
        }
    }
    validate_runs(&document.styles, &document.runs)?;
    Ok(())
}

fn validate_unique_style_ids(styles: &[TextDocumentStyle]) -> Result<(), crate::doc::store::StoreError> {
    for (i, style) in styles.iter().enumerate() {
        if styles[..i].iter().any(|other| other.id == style.id) {
            return Err(crate::doc::store::StoreError::Property(format!(
                "text-style id {} が2枚ある",
                style.id
            )));
        }
    }
    Ok(())
}

fn validate_runs(styles: &[TextDocumentStyle], runs: &[TextRun]) -> Result<(), crate::doc::store::StoreError> {
    for (i, run) in runs.iter().enumerate() {
        if run.len == 0 {
            return Err(crate::doc::store::StoreError::Property(format!(
                "text-value-run[{i}] の len が 0(本文を1文字も覆わない run は無意味)"
            )));
        }
        if !styles.iter().any(|style| style.id == run.style) {
            return Err(crate::doc::store::StoreError::Property(format!(
                "text-value-run[{i}] が styleId {} を指すが、その id のスタイル行が無い",
                run.style
            )));
        }
        if i > 0 && runs[i - 1].style == run.style {
            return Err(crate::doc::store::StoreError::Property(format!(
                "text-value-run[{}] と [{i}] が同じ styleId {} を指して隣接している。\
                 隣接同値ランは併合すること(裁定89)",
                i - 1,
                run.style
            )));
        }
    }
    Ok(())
}

impl crate::doc::store::PropertyId {
    pub fn text_style_size(style: TextStyleId) -> Self {
        Self::text_style_layout_property(style, "size")
    }

    pub fn text_style_line_height(style: TextStyleId) -> Self {
        Self::text_style_layout_property(style, "line_height")
    }

    pub fn text_style_tracking(style: TextStyleId) -> Self {
        Self::text_style_layout_property(style, "tracking")
    }

    pub fn text_style_fill_color(style: TextStyleId) -> Self {
        Self::text_style_layout_property(style, "fill_color")
    }

    pub fn text_style_stroke_color(style: TextStyleId) -> Self {
        Self::text_style_layout_property(style, "stroke_color")
    }

    fn text_style_layout_property(style: TextStyleId, attr: &str) -> Self {
        let name = format!("{}{style}.{attr}", crate::doc::store::property::TEXT_STYLE_PREFIX);
        Self::new(&name).expect("text-style の property 名は予約語でも空でもない")
    }

    pub fn text_justify() -> Self {
        Self::new("text_justify").expect("`text_justify` は予約語でも空でもない")
    }
}
