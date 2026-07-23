use std::{fmt, sync::Arc};

use fontique::{Collection, CollectionOptions, FontStyle, FontWeight, FontWidth};
use harfrust::{FontRef, ShaperData, UnicodeBuffer};
use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use sha2::{Digest, Sha256};
use thiserror::Error;
use vello::{
    kurbo::{Affine, Rect},
    peniko::{Blob, Color, Fill, FontData},
    Glyph, Scene,
};

pub const KEYFRAME_COUNT: usize = 100_000;
pub const DEFAULT_WARMUP_FRAMES: u32 = 120;
pub const DEFAULT_MEASURE_FRAMES: u32 = 100;
pub const DEFAULT_MEASURE_SECONDS: f64 = 30.0;

pub const FIXTURE_CJK_LABELS: &[&str] = &["タイムライン", "位置 X", "不透明度", "キーフレーム"];

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FaceDescriptor {
    pub family: String,
    pub width: String,
    pub style: String,
    pub weight: f32,
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum DescriptorError {
    #[error("font descriptor must contain exactly four pipe-delimited fields")]
    FieldCount,
    #[error("font descriptor field `{field}` must not be empty or padded")]
    InvalidField { field: &'static str },
    #[error("font descriptor width must be `normal`")]
    Width,
    #[error("font descriptor style must be `normal`")]
    Style,
    #[error("font descriptor weight must be finite and within 1..=1000")]
    Weight,
}

impl FaceDescriptor {
    pub fn parse(input: &str) -> Result<Self, DescriptorError> {
        let fields: Vec<_> = input.split('|').collect();
        if fields.len() != 4 {
            return Err(DescriptorError::FieldCount);
        }
        for (field, name) in fields.iter().zip(["family", "width", "style", "weight"]) {
            if field.is_empty() || field.trim() != *field {
                return Err(DescriptorError::InvalidField { field: name });
            }
        }
        if fields[1] != "normal" {
            return Err(DescriptorError::Width);
        }
        if fields[2] != "normal" {
            return Err(DescriptorError::Style);
        }
        let weight = fields[3]
            .parse::<f32>()
            .map_err(|_| DescriptorError::Weight)?;
        if !weight.is_finite() || !(1.0..=1000.0).contains(&weight) {
            return Err(DescriptorError::Weight);
        }
        Ok(Self {
            family: fields[0].to_owned(),
            width: fields[1].to_owned(),
            style: fields[2].to_owned(),
            weight,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PositionedGlyph {
    pub glyph_id: u32,
    pub cluster: u32,
    pub x_advance: i32,
    pub y_advance: i32,
    pub x_offset: i32,
    pub y_offset: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ShapedLabel {
    pub label: String,
    pub glyphs: Vec<PositionedGlyph>,
}

#[derive(Clone, PartialEq)]
pub struct FixtureFont {
    pub descriptor: FaceDescriptor,
    pub face_index: u32,
    font_bytes: Arc<[u8]>,
    pub labels: Vec<ShapedLabel>,
    pub coverage_codepoint_count: usize,
    pub font_sha256: String,
    pub glyph_digest: String,
}

impl FixtureFont {
    pub fn build(descriptor: FaceDescriptor) -> Result<Self, FixtureError> {
        let mut collection = Collection::new(CollectionOptions::default());
        collection.load_system_fonts();
        let family = collection
            .family_by_name(&descriptor.family)
            .ok_or_else(|| FixtureError::FamilyNotFound {
                family: descriptor.family.clone(),
            })?;
        ensure_canonical_family_name(&descriptor.family, family.name())?;

        let width = FontWidth::NORMAL;
        let style = FontStyle::Normal;
        let weight = FontWeight::new(descriptor.weight);
        let candidates: Vec<_> = family
            .fonts()
            .iter()
            .filter(|font| {
                font.width() == width && font.style() == style && font.weight() == weight
            })
            .collect();
        let face = exactly_one(candidates)?;
        let bytes: Arc<[u8]> = Arc::from(
            face.load(None)
                .ok_or(FixtureError::FontBytesUnavailable)?
                .as_ref(),
        );
        let charmap = face
            .charmap_index()
            .charmap(&bytes)
            .ok_or(FixtureError::CharmapUnavailable)?;
        let coverage_codepoint_count = verify_coverage(&charmap, FIXTURE_CJK_LABELS)?;
        let labels = shape_labels(&bytes, face.index(), FIXTURE_CJK_LABELS)?;
        let font_sha256 = sha256_hex(&bytes);
        let glyph_digest = glyph_digest(
            &descriptor,
            face.index(),
            coverage_codepoint_count,
            &font_sha256,
            &labels,
        );
        Ok(Self {
            descriptor,
            face_index: face.index(),
            font_bytes: bytes,
            labels,
            coverage_codepoint_count,
            font_sha256,
            glyph_digest,
        })
    }

    pub fn run_counts(&self) -> Vec<(&str, usize)> {
        self.labels
            .iter()
            .map(|label| (label.label.as_str(), label.glyphs.len()))
            .collect()
    }
}

impl fmt::Debug for FixtureFont {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FixtureFont")
            .field("descriptor", &self.descriptor)
            .field("face_index", &self.face_index)
            .field("font_byte_len", &self.font_bytes.len())
            .field("labels", &self.labels)
            .field("coverage_codepoint_count", &self.coverage_codepoint_count)
            .field("font_sha256", &self.font_sha256)
            .field("glyph_digest", &self.glyph_digest)
            .finish()
    }
}

impl Serialize for FixtureFont {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Sanitized<'a> {
            descriptor: &'a FaceDescriptor,
            face_index: u32,
            labels: &'a [ShapedLabel],
            coverage_codepoint_count: usize,
            font_sha256: &'a str,
            glyph_digest: &'a str,
        }
        Sanitized {
            descriptor: &self.descriptor,
            face_index: self.face_index,
            labels: &self.labels,
            coverage_codepoint_count: self.coverage_codepoint_count,
            font_sha256: &self.font_sha256,
            glyph_digest: &self.glyph_digest,
        }
        .serialize(serializer)
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum FixtureError {
    #[error(transparent)]
    Descriptor(#[from] DescriptorError),
    #[error("no system family canonically named `{family}`")]
    FamilyNotFound { family: String },
    #[error("requested family is not the canonical system family name: `{requested}`")]
    FamilyNotCanonical { requested: String },
    #[error("exact font face resolution requires exactly one candidate, found {count}")]
    CandidateCount { count: usize },
    #[error("selected font face bytes could not be loaded")]
    FontBytesUnavailable,
    #[error("selected font face has no usable charmap")]
    CharmapUnavailable,
    #[error("selected face does not cover U+{codepoint:04X} in label `{label}`")]
    CoverageMissing { label: String, codepoint: u32 },
    #[error("harfrust produced an empty glyph run for label `{label}`")]
    EmptyGlyphRun { label: String },
    #[error("harfrust produced glyph 0 for label `{label}`")]
    MissingGlyph { label: String },
    #[error("harfrust produced a non-finite position for label `{label}`")]
    NonFiniteGlyph { label: String },
    #[error("font bytes do not contain face index {index}")]
    InvalidFaceIndex { index: u32 },
}

fn exactly_one<T>(candidates: Vec<T>) -> Result<T, FixtureError> {
    match candidates.len() {
        1 => match candidates.into_iter().next() {
            Some(candidate) => Ok(candidate),
            None => Err(FixtureError::CandidateCount { count: 0 }),
        },
        count => Err(FixtureError::CandidateCount { count }),
    }
}

fn ensure_canonical_family_name(requested: &str, canonical: &str) -> Result<(), FixtureError> {
    if requested == canonical {
        Ok(())
    } else {
        Err(FixtureError::FamilyNotCanonical {
            requested: requested.to_owned(),
        })
    }
}

fn verify_coverage(
    charmap: &fontique::Charmap<'_>,
    labels: &[&str],
) -> Result<usize, FixtureError> {
    verify_coverage_with(labels, |character| charmap.map(character))
}

fn verify_coverage_with(
    labels: &[&str],
    mut glyph_for: impl FnMut(char) -> Option<u32>,
) -> Result<usize, FixtureError> {
    let mut count = 0;
    for label in labels {
        for character in label.chars().filter(|character| !character.is_whitespace()) {
            count += 1;
            if glyph_for(character)
                .filter(|glyph_id| *glyph_id != 0)
                .is_none()
            {
                return Err(FixtureError::CoverageMissing {
                    label: (*label).to_owned(),
                    codepoint: character as u32,
                });
            }
        }
    }
    Ok(count)
}

fn shape_labels(
    bytes: &[u8],
    face_index: u32,
    labels: &[&str],
) -> Result<Vec<ShapedLabel>, FixtureError> {
    let font = FontRef::from_index(bytes, face_index)
        .map_err(|_| FixtureError::InvalidFaceIndex { index: face_index })?;
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    labels
        .iter()
        .map(|label| shape_label(&shaper, label))
        .collect()
}

fn shape_label(shaper: &harfrust::Shaper<'_>, label: &str) -> Result<ShapedLabel, FixtureError> {
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(label);
    buffer.guess_segment_properties();
    let glyph_buffer = shaper.shape(buffer, &[]);
    if glyph_buffer.is_empty() {
        return Err(FixtureError::EmptyGlyphRun {
            label: label.to_owned(),
        });
    }
    let glyphs = glyph_buffer
        .glyph_infos()
        .iter()
        .zip(glyph_buffer.glyph_positions())
        .map(|(info, position)| {
            positioned_glyph(
                label,
                info.glyph_id,
                info.cluster,
                f64::from(position.x_advance),
                f64::from(position.y_advance),
                f64::from(position.x_offset),
                f64::from(position.y_offset),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ShapedLabel {
        label: label.to_owned(),
        glyphs,
    })
}

fn positioned_glyph(
    label: &str,
    glyph_id: u32,
    cluster: u32,
    x_advance: f64,
    y_advance: f64,
    x_offset: f64,
    y_offset: f64,
) -> Result<PositionedGlyph, FixtureError> {
    if glyph_id == 0 {
        return Err(FixtureError::MissingGlyph {
            label: label.to_owned(),
        });
    }
    if [x_advance, y_advance, x_offset, y_offset]
        .iter()
        .any(|value| !value.is_finite())
    {
        return Err(FixtureError::NonFiniteGlyph {
            label: label.to_owned(),
        });
    }
    Ok(PositionedGlyph {
        glyph_id,
        cluster,
        x_advance: x_advance as i32,
        y_advance: y_advance as i32,
        x_offset: x_offset as i32,
        y_offset: y_offset as i32,
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn glyph_digest(
    descriptor: &FaceDescriptor,
    face_index: u32,
    coverage_codepoint_count: usize,
    font_sha256: &str,
    labels: &[ShapedLabel],
) -> String {
    let mut canonical = Vec::new();
    canonical.extend_from_slice(b"motolii.g0_9.fixture-glyphs.v1\0");
    push_bytes(&mut canonical, descriptor.family.as_bytes());
    push_bytes(&mut canonical, descriptor.width.as_bytes());
    push_bytes(&mut canonical, descriptor.style.as_bytes());
    canonical.extend_from_slice(&descriptor.weight.to_bits().to_le_bytes());
    canonical.extend_from_slice(&face_index.to_le_bytes());
    canonical.extend_from_slice(&(coverage_codepoint_count as u64).to_le_bytes());
    push_bytes(&mut canonical, font_sha256.as_bytes());
    for label in labels {
        push_bytes(&mut canonical, label.label.as_bytes());
        canonical.extend_from_slice(&(label.glyphs.len() as u64).to_le_bytes());
        for glyph in &label.glyphs {
            canonical.extend_from_slice(&glyph.glyph_id.to_le_bytes());
            canonical.extend_from_slice(&glyph.cluster.to_le_bytes());
            canonical.extend_from_slice(&glyph.x_advance.to_le_bytes());
            canonical.extend_from_slice(&glyph.y_advance.to_le_bytes());
            canonical.extend_from_slice(&glyph.x_offset.to_le_bytes());
            canonical.extend_from_slice(&glyph.y_offset.to_le_bytes());
        }
    }
    sha256_hex(&canonical)
}

fn push_bytes(bytes: &mut Vec<u8>, value: &[u8]) {
    bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
    bytes.extend_from_slice(value);
}

/// この spike を構成する固定入力だけを同じ順で束ね、実行条件との混同を防ぐ。
pub fn source_digest() -> String {
    let mut source = Vec::new();
    source.extend_from_slice(b"motolii.g0_9.source.v1\0");
    for bytes in [
        include_bytes!("../Cargo.toml").as_slice(),
        include_bytes!("../Cargo.lock").as_slice(),
        include_bytes!("lib.rs").as_slice(),
        include_bytes!("main.rs").as_slice(),
        include_bytes!("timeline.wgsl").as_slice(),
        include_bytes!("overlay_composite.wgsl").as_slice(),
    ] {
        push_bytes(&mut source, bytes);
    }
    sha256_hex(&source)
}

pub const FIXTURE_OVERLAY_FONT_SIZE: f32 = 16.0;

const FIXTURE_OVERLAY_ORIGINS: [(f32, f32); 4] =
    [(16.0, 24.0), (16.0, 52.0), (16.0, 80.0), (16.0, 108.0)];

struct FixtureFontBytes(Arc<[u8]>);

impl AsRef<[u8]> for FixtureFontBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OverlayGlyph {
    pub glyph_id: u32,
    pub cluster: u32,
    pub x: f32,
    pub y: f32,
    pub x_advance: f32,
    pub y_advance: f32,
    pub x_offset: f32,
    pub y_offset: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OverlayRun {
    pub label: String,
    pub origin_x: f32,
    pub origin_y: f32,
    pub glyphs: Vec<OverlayGlyph>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OverlayMetadata {
    pub fixture_digest: String,
    pub font_sha256: String,
    pub glyph_digest: String,
    pub label_count: usize,
    pub run_count: usize,
    pub glyph_count: usize,
    pub path_count: usize,
}

pub struct VelloOverlayAsset {
    pub metadata: OverlayMetadata,
    pub runs: Vec<OverlayRun>,
    scene: Scene,
}

impl VelloOverlayAsset {
    pub fn scene(&self) -> &Scene {
        &self.scene
    }
}

impl fmt::Debug for VelloOverlayAsset {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VelloOverlayAsset")
            .field("metadata", &self.metadata)
            .field("runs", &self.runs)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum OverlayError {
    #[error(transparent)]
    Scenario(#[from] ScenarioError),
    #[error("fixture font digest does not match its exact font bytes")]
    FontDigestMismatch,
    #[error("fixture glyph digest does not match its shaped labels")]
    GlyphDigestMismatch,
    #[error("fixture labels do not match the fixed overlay order")]
    LabelOrder,
    #[error("fixture face index {index} is not available in its exact bytes")]
    InvalidFaceIndex { index: u32 },
    #[error("fixture face has a zero units-per-em value")]
    ZeroUnitsPerEm,
    #[error("overlay font size must be finite and positive")]
    InvalidFontSize,
    #[error("overlay origin for label `{label}` is non-finite")]
    NonFiniteOrigin { label: String },
    #[error("overlay glyph run for label `{label}` is empty")]
    EmptyGlyphRun { label: String },
    #[error("overlay glyph run for label `{label}` contains glyph 0")]
    MissingGlyph { label: String },
    #[error("overlay glyph conversion for label `{label}` is non-finite")]
    NonFiniteGlyph { label: String },
}

pub fn build_vello_overlay_asset(fixture: &FixtureFont) -> Result<VelloOverlayAsset, OverlayError> {
    let font_size = FIXTURE_OVERLAY_FONT_SIZE;
    if !font_size.is_finite() || font_size <= 0.0 {
        return Err(OverlayError::InvalidFontSize);
    }
    if sha256_hex(&fixture.font_bytes) != fixture.font_sha256 {
        return Err(OverlayError::FontDigestMismatch);
    }
    if glyph_digest(
        &fixture.descriptor,
        fixture.face_index,
        fixture.coverage_codepoint_count,
        &fixture.font_sha256,
        &fixture.labels,
    ) != fixture.glyph_digest
    {
        return Err(OverlayError::GlyphDigestMismatch);
    }
    if fixture.labels.len() != FIXTURE_CJK_LABELS.len()
        || fixture
            .labels
            .iter()
            .map(|label| label.label.as_str())
            .ne(FIXTURE_CJK_LABELS.iter().copied())
    {
        return Err(OverlayError::LabelOrder);
    }
    let fixture_digest = ScenarioDefinition::fixed().digests()?.scenario_sha256;

    let face = FontRef::from_index(&fixture.font_bytes, fixture.face_index).map_err(|_| {
        OverlayError::InvalidFaceIndex {
            index: fixture.face_index,
        }
    })?;
    let shaping_data = ShaperData::new(&face);
    let units_per_em = shaping_data.shaper(&face).build().units_per_em();
    let scale = overlay_scale(font_size, units_per_em)?;
    let runs = fixture
        .labels
        .iter()
        .zip(FIXTURE_OVERLAY_ORIGINS)
        .map(|(label, (origin_x, origin_y))| overlay_run(label, origin_x, origin_y, scale))
        .collect::<Result<Vec<_>, _>>()?;

    let font = FontData::new(
        Blob::new(Arc::new(FixtureFontBytes(fixture.font_bytes.clone()))),
        fixture.face_index,
    );
    let mut scene = Scene::new();
    let path_count = append_overlay_paths(&mut scene);
    for run in &runs {
        scene
            .draw_glyphs(&font)
            .font_size(font_size)
            .transform(Affine::IDENTITY)
            .brush(Color::from_rgba8(230, 230, 230, 255))
            .draw(
                Fill::NonZero,
                run.glyphs.iter().map(|glyph| Glyph {
                    id: glyph.glyph_id,
                    x: glyph.x,
                    y: glyph.y,
                }),
            );
    }
    let glyph_count = runs.iter().map(|run| run.glyphs.len()).sum();
    Ok(VelloOverlayAsset {
        metadata: OverlayMetadata {
            fixture_digest,
            font_sha256: fixture.font_sha256.clone(),
            glyph_digest: fixture.glyph_digest.clone(),
            label_count: FIXTURE_CJK_LABELS.len(),
            run_count: runs.len(),
            glyph_count,
            path_count,
        },
        runs,
        scene,
    })
}

fn overlay_scale(font_size: f32, units_per_em: i32) -> Result<f32, OverlayError> {
    if units_per_em == 0 {
        return Err(OverlayError::ZeroUnitsPerEm);
    }
    Ok(font_size / units_per_em as f32)
}

fn overlay_run(
    label: &ShapedLabel,
    origin_x: f32,
    origin_y: f32,
    scale: f32,
) -> Result<OverlayRun, OverlayError> {
    if !origin_x.is_finite() || !origin_y.is_finite() {
        return Err(OverlayError::NonFiniteOrigin {
            label: label.label.clone(),
        });
    }
    if label.glyphs.is_empty() {
        return Err(OverlayError::EmptyGlyphRun {
            label: label.label.clone(),
        });
    }
    let mut pen_x = origin_x;
    let mut pen_y = origin_y;
    let glyphs = label
        .glyphs
        .iter()
        .map(|glyph| {
            if glyph.glyph_id == 0 {
                return Err(OverlayError::MissingGlyph {
                    label: label.label.clone(),
                });
            }
            let x_advance = glyph.x_advance as f32 * scale;
            // Velloのglyph座標は画面Y-downで、harfrustのfont-unit Y-upをここで反転する。
            let y_advance = -(glyph.y_advance as f32 * scale);
            let x_offset = glyph.x_offset as f32 * scale;
            let y_offset = -(glyph.y_offset as f32 * scale);
            let x = pen_x + x_offset;
            let y = pen_y + y_offset;
            if [x, y, x_advance, y_advance, x_offset, y_offset]
                .iter()
                .any(|value| !value.is_finite())
            {
                return Err(OverlayError::NonFiniteGlyph {
                    label: label.label.clone(),
                });
            }
            pen_x += x_advance;
            pen_y += y_advance;
            Ok(OverlayGlyph {
                glyph_id: glyph.glyph_id,
                cluster: glyph.cluster,
                x,
                y,
                x_advance,
                y_advance,
                x_offset,
                y_offset,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(OverlayRun {
        label: label.label.clone(),
        origin_x,
        origin_y,
        glyphs,
    })
}

fn append_overlay_paths(scene: &mut Scene) -> usize {
    for (rect, color) in [
        (
            Rect::new(0.0, 0.0, 240.0, 32.0),
            Color::from_rgba8(34, 34, 34, 255),
        ),
        (
            Rect::new(0.0, 32.0, 240.0, 128.0),
            Color::from_rgba8(24, 24, 24, 255),
        ),
        (
            Rect::new(176.0, 32.0, 178.0, 128.0),
            Color::from_rgba8(255, 92, 92, 255),
        ),
    ] {
        scene.fill(Fill::NonZero, Affine::IDENTITY, color, None, &rect);
    }
    3
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct KeyInstance {
    pub time_seconds: f32,
    pub track: f32,
    pub selected: u32,
    pub _padding: u32,
}

pub fn make_key_instances(count: usize) -> Vec<KeyInstance> {
    (0..count)
        .map(|index| KeyInstance {
            time_seconds: (index % 10_000) as f32 * 0.01,
            track: (index % 32) as f32,
            selected: u32::from(index % 10 == 0),
            _padding: 0,
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceCreationCounters {
    pub pipelines: u64,
    pub buffers: u64,
    pub bind_groups: u64,
    pub textures: u64,
}

impl ResourceCreationCounters {
    pub fn delta(self, baseline: Self) -> Self {
        Self {
            pipelines: self.pipelines.saturating_sub(baseline.pipelines),
            buffers: self.buffers.saturating_sub(baseline.buffers),
            bind_groups: self.bind_groups.saturating_sub(baseline.bind_groups),
            textures: self.textures.saturating_sub(baseline.textures),
        }
    }

    pub fn is_zero(self) -> bool {
        self == Self::default()
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MeasurementSummary {
    pub samples: usize,
    pub median_frame_ms: f64,
    pub p95_frame_ms: f64,
    pub max_frame_ms: f64,
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum TimingError {
    #[error("timing summary must contain at least one sample")]
    Empty,
    #[error("timing summary values must be finite and non-negative")]
    InvalidValue,
    #[error("timing summary percentiles must be ordered")]
    Unordered,
}

impl MeasurementSummary {
    pub fn checked(
        samples: usize,
        median_frame_ms: f64,
        p95_frame_ms: f64,
        max_frame_ms: f64,
    ) -> Result<Self, TimingError> {
        let summary = Self {
            samples,
            median_frame_ms,
            p95_frame_ms,
            max_frame_ms,
        };
        summary.validate()?;
        Ok(summary)
    }

    pub fn validate(&self) -> Result<(), TimingError> {
        if self.samples == 0 {
            return Err(TimingError::Empty);
        }
        if [self.median_frame_ms, self.p95_frame_ms, self.max_frame_ms]
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
        {
            return Err(TimingError::InvalidValue);
        }
        if self.median_frame_ms > self.p95_frame_ms || self.p95_frame_ms > self.max_frame_ms {
            return Err(TimingError::Unordered);
        }
        Ok(())
    }
}

pub fn summarize_samples(samples: &[f64]) -> Option<MeasurementSummary> {
    if samples.is_empty()
        || samples
            .iter()
            .any(|sample| !sample.is_finite() || *sample < 0.0)
    {
        return None;
    }
    let mut ordered = samples.to_vec();
    ordered.sort_by(f64::total_cmp);
    let percentile = |fraction: f64| {
        let index = ((ordered.len() - 1) as f64 * fraction).ceil() as usize;
        ordered[index]
    };
    MeasurementSummary::checked(
        ordered.len(),
        percentile(0.5),
        percentile(0.95),
        *ordered.last().unwrap(),
    )
    .ok()
}

pub const FIXTURE_VERSION: &str = "g0-9-windowed-timeline.v1";
pub const FIXTURE_CLIP_COUNT: u32 = 1_000;
pub const FIXTURE_SELECTED_KEY_COUNT: u32 = 10_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RendererMode {
    DirectVello,
    EguiVello,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum RendererModeError {
    #[error("renderer mode must not be empty")]
    Empty,
    #[error("unknown renderer mode `{value}`")]
    Unknown { value: String },
}

impl std::str::FromStr for RendererMode {
    type Err = RendererModeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "direct_vello" => Ok(Self::DirectVello),
            "egui_vello" => Ok(Self::EguiVello),
            "" => Err(RendererModeError::Empty),
            _ => Err(RendererModeError::Unknown {
                value: value.to_owned(),
            }),
        }
    }
}

impl<'de> Deserialize<'de> for RendererMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(de::Error::custom)
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum ScenarioError {
    #[error("fixture version, label, and path must be non-empty and unpadded")]
    InvalidText,
    #[error("scenario must contain exactly {expected} keys, found {actual}")]
    KeyCount { expected: usize, actual: usize },
    #[error("scenario must contain exactly {expected} selected keys, found {actual}")]
    SelectedKeyCount { expected: u32, actual: u32 },
    #[error("scenario must contain exactly {expected} clips, found {actual}")]
    ClipCount { expected: u32, actual: u32 },
    #[error("scenario playhead must be finite and non-negative")]
    Playhead,
    #[error("scenario frame index is outside the fixture")]
    FrameIndex,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioDefinition {
    pub fixture_version: String,
    pub key_count: usize,
    pub selected_key_count: u32,
    pub clip_count: u32,
    pub label: String,
    pub path: String,
    pub playhead_seconds: f64,
}

impl ScenarioDefinition {
    pub fn fixed() -> Self {
        Self {
            fixture_version: FIXTURE_VERSION.to_owned(),
            key_count: KEYFRAME_COUNT,
            selected_key_count: FIXTURE_SELECTED_KEY_COUNT,
            clip_count: FIXTURE_CLIP_COUNT,
            label: "タイムライン".to_owned(),
            path: "/fixture/windowed-timeline".to_owned(),
            playhead_seconds: 42.0,
        }
    }

    pub fn validate(&self) -> Result<(), ScenarioError> {
        if [
            self.fixture_version.as_str(),
            self.label.as_str(),
            self.path.as_str(),
        ]
        .iter()
        .any(|value| value.is_empty() || value.trim() != *value)
        {
            return Err(ScenarioError::InvalidText);
        }
        if self.key_count != KEYFRAME_COUNT {
            return Err(ScenarioError::KeyCount {
                expected: KEYFRAME_COUNT,
                actual: self.key_count,
            });
        }
        if self.selected_key_count != FIXTURE_SELECTED_KEY_COUNT {
            return Err(ScenarioError::SelectedKeyCount {
                expected: FIXTURE_SELECTED_KEY_COUNT,
                actual: self.selected_key_count,
            });
        }
        if self.clip_count != FIXTURE_CLIP_COUNT {
            return Err(ScenarioError::ClipCount {
                expected: FIXTURE_CLIP_COUNT,
                actual: self.clip_count,
            });
        }
        if !self.playhead_seconds.is_finite() || self.playhead_seconds < 0.0 {
            return Err(ScenarioError::Playhead);
        }
        Ok(())
    }

    pub fn at(&self, index: u64) -> Result<ScenarioFrame, ScenarioError> {
        self.validate()?;
        ScenarioFrame::from_definition(self, index)
    }

    pub fn digests(&self) -> Result<ScenarioDigests, ScenarioError> {
        self.validate()?;
        let mut scenario = Vec::new();
        scenario.extend_from_slice(b"motolii.g0_9.scenario.v1\0");
        push_bytes(&mut scenario, self.fixture_version.as_bytes());
        scenario.extend_from_slice(&(self.key_count as u64).to_le_bytes());
        scenario.extend_from_slice(&self.selected_key_count.to_le_bytes());
        scenario.extend_from_slice(&self.clip_count.to_le_bytes());
        push_bytes(&mut scenario, self.label.as_bytes());
        push_bytes(&mut scenario, self.path.as_bytes());
        scenario.extend_from_slice(&self.playhead_seconds.to_bits().to_le_bytes());

        let mut inputs = Vec::new();
        inputs.extend_from_slice(b"motolii.g0_9.inputs.v1\0");
        for index in 0..self.key_count as u64 {
            self.at(index)?.encode_input(&mut inputs);
        }
        Ok(ScenarioDigests {
            scenario_sha256: sha256_hex(&scenario),
            input_sequence_sha256: sha256_hex(&inputs),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ScenarioDigests {
    pub scenario_sha256: String,
    pub input_sequence_sha256: String,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScriptedInput {
    PanLeft,
    PanRight,
    ZoomIn,
    ZoomOut,
    Select,
}

impl ScriptedInput {
    fn for_index(index: u64) -> Self {
        match index % 5 {
            0 => Self::PanLeft,
            1 => Self::PanRight,
            2 => Self::ZoomIn,
            3 => Self::ZoomOut,
            _ => Self::Select,
        }
    }

    fn discriminant(self) -> u8 {
        match self {
            Self::PanLeft => 0,
            Self::PanRight => 1,
            Self::ZoomIn => 2,
            Self::ZoomOut => 3,
            Self::Select => 4,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioFrame {
    pub index: u64,
    pub pan_seconds: f64,
    pub zoom_pixels_per_second: f64,
    pub selected_key_index: u32,
    pub input: ScriptedInput,
}

impl ScenarioFrame {
    pub fn at(index: u64) -> Result<Self, ScenarioError> {
        ScenarioDefinition::fixed().at(index)
    }

    fn from_definition(definition: &ScenarioDefinition, index: u64) -> Result<Self, ScenarioError> {
        if index >= definition.key_count as u64 {
            return Err(ScenarioError::FrameIndex);
        }
        let phase = index as f64 * 0.0125;
        let zoom = 18.0 + phase.sin().abs() * 72.0;
        let visible_seconds = 1_000.0 / zoom;
        let pan_seconds =
            (phase * 0.37).sin().mul_add(0.5, 0.5) * (100.0 - visible_seconds).max(0.0);
        Ok(ScenarioFrame {
            index,
            pan_seconds,
            zoom_pixels_per_second: zoom,
            selected_key_index: (index as usize % definition.key_count) as u32,
            input: ScriptedInput::for_index(index),
        })
    }
    fn encode_input(&self, target: &mut Vec<u8>) {
        target.extend_from_slice(&self.index.to_le_bytes());
        target.extend_from_slice(&self.pan_seconds.to_bits().to_le_bytes());
        target.extend_from_slice(&self.zoom_pixels_per_second.to_bits().to_le_bytes());
        target.extend_from_slice(&self.selected_key_index.to_le_bytes());
        target.push(self.input.discriminant());
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Rss {
    Available { bytes: u64 },
    Unavailable { reason: String },
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum RssError {
    #[error("available RSS bytes must be non-zero")]
    ZeroAvailable,
    #[error("unavailable RSS reason must be non-empty and unpadded")]
    EmptyReason,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum PsRssError {
    #[error("ps RSS output must contain one positive integer")]
    Shape,
    #[error("ps RSS KiB value overflows bytes")]
    Overflow,
}

pub fn rss_from_ps_output(output: &str) -> Result<Rss, PsRssError> {
    let value = output.trim();
    if value.is_empty()
        || value.split_whitespace().count() != 1
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(PsRssError::Shape);
    }
    let kib = value.parse::<u64>().map_err(|_| PsRssError::Shape)?;
    if kib == 0 {
        return Err(PsRssError::Shape);
    }
    let bytes = kib.checked_mul(1024).ok_or(PsRssError::Overflow)?;
    Ok(Rss::Available { bytes })
}

impl Rss {
    pub fn validate(&self) -> Result<(), RssError> {
        match self {
            Self::Available { bytes: 0 } => Err(RssError::ZeroAvailable),
            Self::Available { .. } => Ok(()),
            Self::Unavailable { reason } if reason.is_empty() || reason.trim() != reason => {
                Err(RssError::EmptyReason)
            }
            Self::Unavailable { .. } => Ok(()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceCreationPhases {
    pub initialization: ResourceCreationCounters,
    pub warmup: ResourceCreationCounters,
    pub measured: ResourceCreationCounters,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum EvidenceCompleteness {
    Complete,
    Incomplete { reason: String },
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum CompletenessError {
    #[error("comparison requires complete evidence")]
    Incomplete,
    #[error("incomplete evidence reason must be non-empty and unpadded")]
    EmptyReason,
}

impl EvidenceCompleteness {
    fn validate_shape(&self) -> Result<(), CompletenessError> {
        match self {
            Self::Complete => Ok(()),
            Self::Incomplete { reason } if reason.is_empty() || reason.trim() != reason => {
                Err(CompletenessError::EmptyReason)
            }
            Self::Incomplete { .. } => Ok(()),
        }
    }

    pub fn validate(&self) -> Result<(), CompletenessError> {
        self.validate_shape()?;
        match self {
            Self::Complete => Ok(()),
            Self::Incomplete { .. } => Err(CompletenessError::Incomplete),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReportConditions {
    pub device: String,
    pub surface: String,
    pub window: String,
    pub webview: String,
    pub fixture: String,
    pub target: String,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ConditionError {
    #[error("report condition `{field}` must be non-empty and unpadded")]
    Invalid { field: &'static str },
}

impl ReportConditions {
    pub fn validate(&self) -> Result<(), ConditionError> {
        for (field, value) in [
            ("device", &self.device),
            ("surface", &self.surface),
            ("window", &self.window),
            ("webview", &self.webview),
            ("fixture", &self.fixture),
            ("target", &self.target),
        ] {
            if value.is_empty() || value.trim() != value {
                return Err(ConditionError::Invalid { field });
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RawReport {
    pub renderer: RendererMode,
    pub scenario_digest: String,
    pub input_digest: String,
    pub source_digest: String,
    pub font_digest: String,
    pub glyph_digest: String,
    pub conditions: ReportConditions,
    pub measured_frames: u32,
    pub measured_seconds: f64,
    pub acquire_count: u64,
    pub present_count: u64,
    pub skip_count: u64,
    pub reconfigure_count: u64,
    pub readback_count: u64,
    pub frame_timing: MeasurementSummary,
    pub present_timing: MeasurementSummary,
    pub input_timing: MeasurementSummary,
    pub rss: Rss,
    pub resource_creations: ResourceCreationPhases,
    pub completeness: EvidenceCompleteness,
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum ReportError {
    #[error("report digest `{field}` must be non-empty and unpadded")]
    InvalidDigest { field: &'static str },
    #[error(transparent)]
    Conditions(#[from] ConditionError),
    #[error(transparent)]
    Timing(#[from] TimingError),
    #[error(transparent)]
    Rss(#[from] RssError),
    #[error(transparent)]
    Completeness(#[from] CompletenessError),
    #[error("measured duration must be finite and non-negative")]
    Duration,
    #[error("input timing must have one sample for each measured frame")]
    InputSamples,
}

impl RawReport {
    pub fn validate(&self) -> Result<(), ReportError> {
        for (field, value) in [
            ("scenario_digest", &self.scenario_digest),
            ("input_digest", &self.input_digest),
            ("source_digest", &self.source_digest),
            ("font_digest", &self.font_digest),
            ("glyph_digest", &self.glyph_digest),
        ] {
            if value.is_empty() || value.trim() != value {
                return Err(ReportError::InvalidDigest { field });
            }
        }
        self.conditions.validate()?;
        self.frame_timing.validate()?;
        self.present_timing.validate()?;
        self.input_timing.validate()?;
        self.rss.validate()?;
        if !self.measured_seconds.is_finite() || self.measured_seconds < 0.0 {
            return Err(ReportError::Duration);
        }
        if self.input_timing.samples != self.measured_frames as usize {
            return Err(ReportError::InputSamples);
        }
        self.completeness.validate_shape()?;
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum RawReportField {
    Renderer,
    ScenarioDigest,
    InputDigest,
    SourceDigest,
    FontDigest,
    GlyphDigest,
    Conditions,
    MeasuredFrames,
    MeasuredSeconds,
    AcquireCount,
    PresentCount,
    SkipCount,
    ReconfigureCount,
    ReadbackCount,
    FrameTiming,
    PresentTiming,
    InputTiming,
    Rss,
    ResourceCreations,
    Completeness,
}

impl<'de> Deserialize<'de> for RawReport {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RawReportVisitor;
        impl<'de> Visitor<'de> for RawReportVisitor {
            type Value = RawReport;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a complete strict raw comparison report")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                macro_rules! fields {
                    ($($field:ident => $name:ident: $ty:ty),+ $(,)?) => {
                        $(let mut $name: Option<$ty> = None;)+
                        while let Some(field) = map.next_key()? {
                            match field {
                                $(RawReportField::$field => {
                                    if $name.is_some() {
                                        return Err(de::Error::duplicate_field(stringify!($name)));
                                    }
                                    $name = Some(map.next_value()?);
                                })+
                            }
                        }
                        $(let $name = $name.ok_or_else(|| de::Error::missing_field(stringify!($name)))?;)+
                    };
                }
                fields! {
                    Renderer => renderer: RendererMode,
                    ScenarioDigest => scenario_digest: String,
                    InputDigest => input_digest: String,
                    SourceDigest => source_digest: String,
                    FontDigest => font_digest: String,
                    GlyphDigest => glyph_digest: String,
                    Conditions => conditions: ReportConditions,
                    MeasuredFrames => measured_frames: u32,
                    MeasuredSeconds => measured_seconds: f64,
                    AcquireCount => acquire_count: u64,
                    PresentCount => present_count: u64,
                    SkipCount => skip_count: u64,
                    ReconfigureCount => reconfigure_count: u64,
                    ReadbackCount => readback_count: u64,
                    FrameTiming => frame_timing: MeasurementSummary,
                    PresentTiming => present_timing: MeasurementSummary,
                    InputTiming => input_timing: MeasurementSummary,
                    Rss => rss: Rss,
                    ResourceCreations => resource_creations: ResourceCreationPhases,
                    Completeness => completeness: EvidenceCompleteness,
                }
                let report = RawReport {
                    renderer,
                    scenario_digest,
                    input_digest,
                    source_digest,
                    font_digest,
                    glyph_digest,
                    conditions,
                    measured_frames,
                    measured_seconds,
                    acquire_count,
                    present_count,
                    skip_count,
                    reconfigure_count,
                    readback_count,
                    frame_timing,
                    present_timing,
                    input_timing,
                    rss,
                    resource_creations,
                    completeness,
                };
                report.validate().map_err(de::Error::custom)?;
                Ok(report)
            }
        }
        deserializer.deserialize_map(RawReportVisitor)
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum ComparisonError {
    #[error(transparent)]
    Report(#[from] ReportError),
    #[error("reports must be ordered direct_vello then egui_vello")]
    ModeOrder,
    #[error("reports differ in `{field}`")]
    Mismatch { field: &'static str },
    #[error("report duration must be at least 30 seconds")]
    Duration,
    #[error("report frame count must be at least 100")]
    FrameCount,
    #[error("acquire and present counts must be equal and non-zero")]
    AcquirePresent,
    #[error("readback count must be zero")]
    Readback,
    #[error("measured phase resource creation must be zero")]
    MeasuredResourceCreation,
    #[error("comparison ratio denominator must be finite and positive")]
    RatioDenominator,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TimingRatios {
    pub frame_median: f64,
    pub frame_p95: f64,
    pub frame_max: f64,
    pub present_median: f64,
    pub present_p95: f64,
    pub present_max: f64,
    pub input_median: f64,
    pub input_p95: f64,
    pub input_max: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ComparisonResult {
    pub direct: RawReport,
    pub egui: RawReport,
    pub ratios: TimingRatios,
}

#[derive(Debug, Error)]
pub enum ComparisonArtifactError {
    #[error(transparent)]
    Comparison(#[from] ComparisonError),
    #[error("failed to serialize comparison artifact: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("comparison artifact I/O failed: {0}")]
    Io(#[from] std::io::Error),
}

pub fn write_comparison_artifact(
    path: &std::path::Path,
    direct: RawReport,
    egui: RawReport,
) -> Result<ComparisonResult, ComparisonArtifactError> {
    let result = compare_reports(direct, egui)?;
    let encoded = serde_json::to_vec_pretty(&result)?;
    let parent = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "comparison path has no UTF-8 file name",
            )
        })?;
    let temporary = parent.join(format!(".{file_name}.{}.tmp", std::process::id()));
    {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        use std::io::Write;
        file.write_all(&encoded)?;
        file.sync_all()?;
    }
    if let Err(error) = std::fs::rename(&temporary, path) {
        let _ = std::fs::remove_file(&temporary);
        return Err(error.into());
    }
    Ok(result)
}

pub fn compare_reports(
    direct: RawReport,
    egui: RawReport,
) -> Result<ComparisonResult, ComparisonError> {
    direct.validate()?;
    egui.validate()?;
    if direct.renderer != RendererMode::DirectVello || egui.renderer != RendererMode::EguiVello {
        return Err(ComparisonError::ModeOrder);
    }
    for (field, equal) in [
        (
            "scenario_digest",
            direct.scenario_digest == egui.scenario_digest,
        ),
        ("input_digest", direct.input_digest == egui.input_digest),
        ("source_digest", direct.source_digest == egui.source_digest),
        ("font_digest", direct.font_digest == egui.font_digest),
        ("glyph_digest", direct.glyph_digest == egui.glyph_digest),
        ("device", direct.conditions.device == egui.conditions.device),
        (
            "surface",
            direct.conditions.surface == egui.conditions.surface,
        ),
        ("window", direct.conditions.window == egui.conditions.window),
        (
            "webview",
            direct.conditions.webview == egui.conditions.webview,
        ),
        (
            "fixture",
            direct.conditions.fixture == egui.conditions.fixture,
        ),
        ("target", direct.conditions.target == egui.conditions.target),
    ] {
        if !equal {
            return Err(ComparisonError::Mismatch { field });
        }
    }
    for report in [&direct, &egui] {
        report.completeness.validate().map_err(ReportError::from)?;
        if report.measured_seconds < DEFAULT_MEASURE_SECONDS {
            return Err(ComparisonError::Duration);
        }
        if report.measured_frames < DEFAULT_MEASURE_FRAMES {
            return Err(ComparisonError::FrameCount);
        }
        if report.acquire_count == 0 || report.acquire_count != report.present_count {
            return Err(ComparisonError::AcquirePresent);
        }
        if report.readback_count != 0 {
            return Err(ComparisonError::Readback);
        }
        if !report.resource_creations.measured.is_zero() {
            return Err(ComparisonError::MeasuredResourceCreation);
        }
    }
    let ratio = |numerator: f64, denominator: f64| {
        if !denominator.is_finite() || denominator <= 0.0 {
            Err(ComparisonError::RatioDenominator)
        } else {
            let value = numerator / denominator;
            if value.is_finite() {
                Ok(value)
            } else {
                Err(ComparisonError::RatioDenominator)
            }
        }
    };
    let ratios = TimingRatios {
        frame_median: ratio(
            egui.frame_timing.median_frame_ms,
            direct.frame_timing.median_frame_ms,
        )?,
        frame_p95: ratio(
            egui.frame_timing.p95_frame_ms,
            direct.frame_timing.p95_frame_ms,
        )?,
        frame_max: ratio(
            egui.frame_timing.max_frame_ms,
            direct.frame_timing.max_frame_ms,
        )?,
        present_median: ratio(
            egui.present_timing.median_frame_ms,
            direct.present_timing.median_frame_ms,
        )?,
        present_p95: ratio(
            egui.present_timing.p95_frame_ms,
            direct.present_timing.p95_frame_ms,
        )?,
        present_max: ratio(
            egui.present_timing.max_frame_ms,
            direct.present_timing.max_frame_ms,
        )?,
        input_median: ratio(
            egui.input_timing.median_frame_ms,
            direct.input_timing.median_frame_ms,
        )?,
        input_p95: ratio(
            egui.input_timing.p95_frame_ms,
            direct.input_timing.p95_frame_ms,
        )?,
        input_max: ratio(
            egui.input_timing.max_frame_ms,
            direct.input_timing.max_frame_ms,
        )?,
    };
    Ok(ComparisonResult {
        direct,
        egui,
        ratios,
    })
}

#[derive(Clone, Copy, Debug)]
pub struct AcceptanceInput {
    pub measured_frames: u32,
    pub target_frames: u32,
    pub measured_seconds: f64,
    pub target_seconds: f64,
    pub acquire_count: u64,
    pub present_count: u64,
    pub readback_count: u64,
    pub frame_creations: ResourceCreationCounters,
}

pub fn acceptance_passes(input: AcceptanceInput) -> bool {
    input.measured_frames >= input.target_frames
        && input.measured_seconds >= input.target_seconds
        && input.acquire_count > 0
        && input.acquire_count == input.present_count
        && input.readback_count == 0
        && input.frame_creations.is_zero()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_summary() -> MeasurementSummary {
        MeasurementSummary::checked(100, 1.0, 2.0, 3.0).unwrap()
    }

    fn valid_report(renderer: RendererMode) -> RawReport {
        RawReport {
            renderer,
            scenario_digest: "scenario".to_owned(),
            input_digest: "inputs".to_owned(),
            source_digest: "source".to_owned(),
            font_digest: "font".to_owned(),
            glyph_digest: "glyphs".to_owned(),
            conditions: ReportConditions {
                device: "Apple M4".to_owned(),
                surface: "metal-fifo".to_owned(),
                window: "1440x900".to_owned(),
                webview: "2-opaque".to_owned(),
                fixture: FIXTURE_VERSION.to_owned(),
                target: "aarch64-apple-darwin".to_owned(),
            },
            measured_frames: DEFAULT_MEASURE_FRAMES,
            measured_seconds: DEFAULT_MEASURE_SECONDS,
            acquire_count: 100,
            present_count: 100,
            skip_count: 0,
            reconfigure_count: 0,
            readback_count: 0,
            frame_timing: valid_summary(),
            present_timing: valid_summary(),
            input_timing: valid_summary(),
            rss: Rss::Available { bytes: 1 },
            resource_creations: ResourceCreationPhases {
                initialization: ResourceCreationCounters::default(),
                warmup: ResourceCreationCounters::default(),
                measured: ResourceCreationCounters::default(),
            },
            completeness: EvidenceCompleteness::Complete,
        }
    }

    #[test]
    fn fixture_has_exactly_one_hundred_thousand_stable_instances() {
        let instances = make_key_instances(KEYFRAME_COUNT);
        assert_eq!(instances.len(), KEYFRAME_COUNT);
        assert_eq!(instances[0].time_seconds, 0.0);
        assert_eq!(instances[99_999].time_seconds, 99.99);
        assert_eq!(
            instances.iter().filter(|key| key.selected == 1).count(),
            10_000
        );
    }

    #[test]
    fn summary_is_deterministic_and_rejects_invalid_samples() {
        let summary = summarize_samples(&[4.0, 1.0, 3.0, 2.0]).unwrap();
        assert_eq!(summary.samples, 4);
        assert_eq!(summary.median_frame_ms, 3.0);
        assert_eq!(summary.p95_frame_ms, 4.0);
        assert_eq!(summary.max_frame_ms, 4.0);
        assert!(summarize_samples(&[]).is_none());
        assert!(summarize_samples(&[f64::NAN]).is_none());
    }

    #[test]
    fn acceptance_requires_duration_repetitions_and_zero_hot_loop_creations() {
        let zero = ResourceCreationCounters::default();
        let good = AcceptanceInput {
            measured_frames: 100,
            target_frames: 100,
            measured_seconds: 30.0,
            target_seconds: 30.0,
            acquire_count: 100,
            present_count: 100,
            readback_count: 0,
            frame_creations: zero,
        };
        assert!(acceptance_passes(good));
        assert!(!acceptance_passes(AcceptanceInput {
            measured_frames: 99,
            ..good
        }));
        assert!(!acceptance_passes(AcceptanceInput {
            measured_seconds: 29.9,
            ..good
        }));
        assert!(!acceptance_passes(AcceptanceInput {
            present_count: 99,
            ..good
        }));
        assert!(!acceptance_passes(AcceptanceInput {
            frame_creations: ResourceCreationCounters {
                bind_groups: 1,
                ..zero
            },
            ..good
        }));
    }

    #[test]
    fn render_hot_loop_has_no_tracked_resource_creation_or_readback_call() {
        let source = include_str!("main.rs");
        let render = source
            .split("fn render(&mut self")
            .nth(1)
            .and_then(|tail| tail.split("\n    fn render_egui(").next())
            .expect("render source section");
        for forbidden in [
            "create_buffer(",
            "create_bind_group(",
            "create_render_pipeline(",
            "create_texture(",
            "copy_texture",
            "map_async",
            "PollType::wait",
        ] {
            assert!(
                !render.contains(forbidden),
                "render hot loop contains forbidden call: {forbidden}",
            );
        }
    }

    #[test]
    fn face_descriptor_rejects_non_exact_forms() {
        for descriptor in [
            "",
            "Hiragino Sans|normal|normal",
            "Hiragino Sans|normal|normal|300|extra",
            " Hiragino Sans|normal|normal|300",
            "Hiragino Sans |normal|normal|300",
            "Hiragino Sans|wide|normal|300",
            "Hiragino Sans|normal|italic|300",
            "Hiragino Sans|normal|normal|NaN",
            "Hiragino Sans|normal|normal|inf",
            "Hiragino Sans|normal|normal|0",
            "Hiragino Sans|normal|normal|1001",
        ] {
            assert!(FaceDescriptor::parse(descriptor).is_err(), "{descriptor}");
        }
    }

    #[test]
    fn exact_candidate_resolution_rejects_zero_and_multiple() {
        assert_eq!(
            exactly_one::<u8>(vec![]),
            Err(FixtureError::CandidateCount { count: 0 })
        );
        assert_eq!(exactly_one(vec![7]), Ok(7));
        assert_eq!(
            exactly_one(vec![7, 8]),
            Err(FixtureError::CandidateCount { count: 2 })
        );
    }

    #[test]
    fn canonical_family_comparison_rejects_partial_case_fold_and_localized_names() {
        for requested in ["Hiragino", "hiragino sans", "ヒラギノ角ゴシック"] {
            assert_eq!(
                ensure_canonical_family_name(requested, "Hiragino Sans"),
                Err(FixtureError::FamilyNotCanonical {
                    requested: requested.to_owned(),
                })
            );
        }
    }

    #[test]
    fn coverage_and_glyph_negatives_are_rejected() {
        assert_eq!(
            verify_coverage_with(&["A B"], |_| None),
            Err(FixtureError::CoverageMissing {
                label: "A B".to_owned(),
                codepoint: 'A' as u32,
            })
        );
        assert_eq!(
            positioned_glyph("label", 0, 0, 1.0, 0.0, 0.0, 0.0),
            Err(FixtureError::MissingGlyph {
                label: "label".to_owned()
            })
        );
        assert_eq!(
            positioned_glyph("label", 1, 0, f64::NAN, 0.0, 0.0, 0.0),
            Err(FixtureError::NonFiniteGlyph {
                label: "label".to_owned()
            })
        );
    }

    #[test]
    fn glyph_digest_is_sensitive_to_run_order() {
        let descriptor = FaceDescriptor::parse("Hiragino Sans|normal|normal|300").unwrap();
        let glyph = PositionedGlyph {
            glyph_id: 1,
            cluster: 0,
            x_advance: 10,
            y_advance: 0,
            x_offset: 0,
            y_offset: 0,
        };
        let first = ShapedLabel {
            label: "A".to_owned(),
            glyphs: vec![glyph.clone()],
        };
        let second = ShapedLabel {
            label: "B".to_owned(),
            glyphs: vec![glyph],
        };
        assert_ne!(
            glyph_digest(&descriptor, 0, 2, "font", &[first.clone(), second.clone()]),
            glyph_digest(&descriptor, 0, 2, "font", &[second, first])
        );
    }

    #[test]
    fn fixture_font_debug_and_json_do_not_expose_font_bytes() {
        let fixture = FixtureFont {
            descriptor: FaceDescriptor::parse("Fixture|normal|normal|300").unwrap(),
            face_index: 0,
            font_bytes: Arc::from(&b"secret-font-bytes"[..]),
            labels: vec![],
            coverage_codepoint_count: 0,
            font_sha256: "digest".to_owned(),
            glyph_digest: "glyphs".to_owned(),
        };
        assert!(!format!("{fixture:?}").contains("secret-font-bytes"));
        assert!(!serde_json::to_string(&fixture)
            .unwrap()
            .contains("secret-font-bytes"));
    }

    #[test]
    fn overlay_run_preserves_clusters_and_converts_font_units_to_vello_pixels() {
        let label = ShapedLabel {
            label: "label".to_owned(),
            glyphs: vec![
                PositionedGlyph {
                    glyph_id: 7,
                    cluster: 3,
                    x_advance: 500,
                    y_advance: 100,
                    x_offset: 25,
                    y_offset: 50,
                },
                PositionedGlyph {
                    glyph_id: 9,
                    cluster: 6,
                    x_advance: 250,
                    y_advance: -50,
                    x_offset: -10,
                    y_offset: 20,
                },
            ],
        };
        let run = overlay_run(&label, 10.0, 20.0, 0.02).unwrap();
        assert_eq!(run.glyphs[0].glyph_id, 7);
        assert_eq!(run.glyphs[0].cluster, 3);
        assert_eq!(run.glyphs[0].x, 10.5);
        assert_eq!(run.glyphs[0].y, 19.0);
        assert_eq!(run.glyphs[1].glyph_id, 9);
        assert_eq!(run.glyphs[1].cluster, 6);
        assert_eq!(run.glyphs[1].x, 19.8);
        assert_eq!(run.glyphs[1].y, 17.6);
        assert_eq!(run.glyphs[1].x_advance, 5.0);
        assert_eq!(run.glyphs[1].y_advance, 1.0);
        assert_eq!(
            overlay_run(&label, 10.0, 20.0, f32::INFINITY),
            Err(OverlayError::NonFiniteGlyph {
                label: "label".to_owned(),
            })
        );
    }

    #[test]
    fn overlay_rejects_empty_runs_and_glyph_zero_with_typed_errors() {
        let empty = ShapedLabel {
            label: "empty".to_owned(),
            glyphs: vec![],
        };
        assert_eq!(
            overlay_run(&empty, 0.0, 0.0, 1.0),
            Err(OverlayError::EmptyGlyphRun {
                label: "empty".to_owned(),
            })
        );

        let missing = ShapedLabel {
            label: "missing".to_owned(),
            glyphs: vec![PositionedGlyph {
                glyph_id: 0,
                cluster: 0,
                x_advance: 0,
                y_advance: 0,
                x_offset: 0,
                y_offset: 0,
            }],
        };
        assert_eq!(
            overlay_run(&missing, 0.0, 0.0, 1.0),
            Err(OverlayError::MissingGlyph {
                label: "missing".to_owned(),
            })
        );
    }

    #[test]
    fn overlay_scale_rejects_zero_units_per_em() {
        assert_eq!(
            overlay_scale(FIXTURE_OVERLAY_FONT_SIZE, 0),
            Err(OverlayError::ZeroUnitsPerEm)
        );
    }

    #[test]
    fn exact_fixture_builds_a_deterministic_vello_overlay_when_preflight_is_available() {
        let Some(descriptor) = std::env::var("G0_9_CJK_FACE").ok() else {
            eprintln!("SKIP: G0_9_CJK_FACE exact-face preflight is not configured");
            return;
        };
        let descriptor = FaceDescriptor::parse(&descriptor).unwrap();
        let fixture = match FixtureFont::build(descriptor) {
            Ok(fixture) => fixture,
            Err(error) => {
                eprintln!("SKIP: exact-face preflight is unavailable: {error}");
                return;
            }
        };
        let first = build_vello_overlay_asset(&fixture).unwrap();
        let second = build_vello_overlay_asset(&fixture).unwrap();
        let scenario_digest = ScenarioDefinition::fixed()
            .digests()
            .unwrap()
            .scenario_sha256;
        assert_eq!(first.metadata, second.metadata);
        assert_eq!(first.runs, second.runs);
        assert_eq!(first.metadata.fixture_digest, scenario_digest);
        assert_ne!(first.metadata.fixture_digest, FIXTURE_VERSION);
        assert_eq!(first.metadata.font_sha256, fixture.font_sha256);
        assert_eq!(first.metadata.glyph_digest, fixture.glyph_digest);
        assert_eq!(first.metadata.label_count, FIXTURE_CJK_LABELS.len());
        assert_eq!(first.metadata.run_count, fixture.labels.len());
        assert_eq!(first.metadata.run_count, FIXTURE_OVERLAY_ORIGINS.len());
        assert_eq!(first.metadata.path_count, 3);
        let expected_glyph_count: usize =
            fixture.labels.iter().map(|label| label.glyphs.len()).sum();
        assert_eq!(first.metadata.glyph_count, expected_glyph_count);
        assert_eq!(
            first.metadata.glyph_count,
            first.runs.iter().map(|run| run.glyphs.len()).sum::<usize>()
        );
        assert_ne!(first.metadata.glyph_count, KEYFRAME_COUNT);
        let face = FontRef::from_index(&fixture.font_bytes, fixture.face_index).unwrap();
        let units_per_em = ShaperData::new(&face).shaper(&face).build().units_per_em();
        let scale = overlay_scale(FIXTURE_OVERLAY_FONT_SIZE, units_per_em).unwrap();
        for ((run, shaped), expected_label) in first
            .runs
            .iter()
            .zip(&fixture.labels)
            .zip(FIXTURE_CJK_LABELS)
        {
            assert_eq!(run.label, *expected_label);
            assert_eq!(run.glyphs.len(), shaped.glyphs.len());
            let mut pen_x = run.origin_x;
            let mut pen_y = run.origin_y;
            for (glyph, shaped_glyph) in run.glyphs.iter().zip(&shaped.glyphs) {
                assert_eq!(glyph.glyph_id, shaped_glyph.glyph_id);
                assert_eq!(glyph.cluster, shaped_glyph.cluster);
                let x_advance = shaped_glyph.x_advance as f32 * scale;
                let y_advance = -(shaped_glyph.y_advance as f32 * scale);
                let x_offset = shaped_glyph.x_offset as f32 * scale;
                let y_offset = -(shaped_glyph.y_offset as f32 * scale);
                assert_eq!(glyph.x, pen_x + x_offset);
                assert_eq!(glyph.y, pen_y + y_offset);
                assert_eq!(glyph.x_advance, x_advance);
                assert_eq!(glyph.y_advance, y_advance);
                assert_eq!(glyph.x_offset, x_offset);
                assert_eq!(glyph.y_offset, y_offset);
                pen_x += x_advance;
                pen_y += y_advance;
            }
        }

        let mut label_order = fixture.clone();
        label_order.labels.swap(0, 1);
        label_order.glyph_digest = glyph_digest(
            &label_order.descriptor,
            label_order.face_index,
            label_order.coverage_codepoint_count,
            &label_order.font_sha256,
            &label_order.labels,
        );
        assert!(matches!(
            build_vello_overlay_asset(&label_order),
            Err(OverlayError::LabelOrder)
        ));

        let mut font_digest = fixture.clone();
        font_digest.font_sha256 = "altered".to_owned();
        assert!(matches!(
            build_vello_overlay_asset(&font_digest),
            Err(OverlayError::FontDigestMismatch)
        ));

        let mut glyph_digest = fixture;
        glyph_digest.glyph_digest = "altered".to_owned();
        assert!(matches!(
            build_vello_overlay_asset(&glyph_digest),
            Err(OverlayError::GlyphDigestMismatch)
        ));
    }

    #[test]
    fn scenario_is_fixed_deterministic_and_finite() {
        let scenario = ScenarioDefinition::fixed();
        assert!(scenario.validate().is_ok());
        assert_eq!(scenario.at(17), scenario.at(17));
        let frame = scenario.at(17).unwrap();
        assert!(frame.pan_seconds.is_finite());
        assert!(frame.zoom_pixels_per_second.is_finite());
        assert_ne!(scenario.digests().unwrap().scenario_sha256, "");
        assert_ne!(scenario.digests().unwrap().input_sequence_sha256, "");
        assert_eq!(
            scenario.at(KEYFRAME_COUNT as u64),
            Err(ScenarioError::FrameIndex)
        );
    }

    #[test]
    fn strict_mode_timing_and_rss_reject_invalid_values() {
        for mode in ["", "direct", "historical", "egui"] {
            assert!(mode.parse::<RendererMode>().is_err(), "{mode}");
        }
        assert!(MeasurementSummary::checked(0, 1.0, 1.0, 1.0).is_err());
        assert!(MeasurementSummary::checked(1, -1.0, 1.0, 1.0).is_err());
        assert!(MeasurementSummary::checked(1, 2.0, 1.0, 3.0).is_err());
        assert!(Rss::Available { bytes: 0 }.validate().is_err());
        assert!(Rss::Unavailable {
            reason: "".to_owned()
        }
        .validate()
        .is_err());
    }

    #[test]
    fn ps_rss_parser_rejects_malformed_zero_overflow_and_trailing_junk() {
        assert_eq!(
            rss_from_ps_output(" 12\n"),
            Ok(Rss::Available { bytes: 12 * 1024 })
        );
        for output in ["", "0", "12 KiB", "12x", "12\n13"] {
            assert!(
                matches!(rss_from_ps_output(output), Err(PsRssError::Shape)),
                "{output:?}"
            );
        }
        assert_eq!(
            rss_from_ps_output(&format!("{}", u64::MAX)),
            Err(PsRssError::Overflow)
        );
    }

    #[test]
    fn source_digest_is_stable_and_not_renderer_specific() {
        let first = source_digest();
        assert_eq!(first, source_digest());
        assert_eq!(first.len(), 64);
        assert!(first.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }

    #[test]
    fn measurement_summary_checked_rejects_non_finite_values() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                MeasurementSummary::checked(1, value, value, value),
                Err(TimingError::InvalidValue)
            );
        }
    }

    #[test]
    fn comparison_success_preserves_raw_reports_and_has_only_finite_ratios() {
        let direct = valid_report(RendererMode::DirectVello);
        let egui = valid_report(RendererMode::EguiVello);

        let result = compare_reports(direct.clone(), egui.clone()).unwrap();

        assert_eq!(result.direct, direct);
        assert_eq!(result.egui, egui);
        for ratio in [
            result.ratios.frame_median,
            result.ratios.frame_p95,
            result.ratios.frame_max,
            result.ratios.present_median,
            result.ratios.present_p95,
            result.ratios.present_max,
            result.ratios.input_median,
            result.ratios.input_p95,
            result.ratios.input_max,
        ] {
            assert!(ratio.is_finite());
        }
    }

    #[test]
    fn comparison_rejects_every_condition_and_digest_difference_before_ratios() {
        let direct = valid_report(RendererMode::DirectVello);
        let mut egui = valid_report(RendererMode::EguiVello);
        for (field, replacement) in [
            ("device", "other-device"),
            ("surface", "other-surface"),
            ("window", "other-window"),
            ("webview", "other-webview"),
            ("fixture", "other-fixture"),
            ("target", "other-target"),
        ] {
            match field {
                "device" => egui.conditions.device = replacement.to_owned(),
                "surface" => egui.conditions.surface = replacement.to_owned(),
                "window" => egui.conditions.window = replacement.to_owned(),
                "webview" => egui.conditions.webview = replacement.to_owned(),
                "fixture" => egui.conditions.fixture = replacement.to_owned(),
                "target" => egui.conditions.target = replacement.to_owned(),
                _ => unreachable!(),
            }
            assert_eq!(
                compare_reports(direct.clone(), egui.clone()),
                Err(ComparisonError::Mismatch { field })
            );
            egui.conditions = direct.conditions.clone();
        }
        for field in [
            "scenario_digest",
            "input_digest",
            "source_digest",
            "font_digest",
            "glyph_digest",
        ] {
            match field {
                "scenario_digest" => egui.scenario_digest = "other".to_owned(),
                "input_digest" => egui.input_digest = "other".to_owned(),
                "source_digest" => egui.source_digest = "other".to_owned(),
                "font_digest" => egui.font_digest = "other".to_owned(),
                "glyph_digest" => egui.glyph_digest = "other".to_owned(),
                _ => unreachable!(),
            }
            assert_eq!(
                compare_reports(direct.clone(), egui.clone()),
                Err(ComparisonError::Mismatch { field })
            );
            egui = valid_report(RendererMode::EguiVello);
        }
    }

    #[test]
    fn comparison_rejects_historical_early_incomplete_and_invalid_evidence() {
        let direct = valid_report(RendererMode::DirectVello);
        let egui = valid_report(RendererMode::EguiVello);
        assert!(compare_reports(egui.clone(), direct.clone()).is_err());
        for mut invalid in [direct.clone(), egui.clone()] {
            invalid.measured_seconds = 29.9;
            assert_eq!(
                compare_reports(
                    if invalid.renderer == RendererMode::DirectVello {
                        invalid.clone()
                    } else {
                        direct.clone()
                    },
                    if invalid.renderer == RendererMode::EguiVello {
                        invalid
                    } else {
                        egui.clone()
                    },
                ),
                Err(ComparisonError::Duration)
            );
        }
        let mut early = direct.clone();
        early.measured_frames = 99;
        early.input_timing.samples = 99;
        assert_eq!(
            compare_reports(early, egui.clone()),
            Err(ComparisonError::FrameCount)
        );
        let mut acquire_mismatch = direct.clone();
        acquire_mismatch.present_count = 99;
        assert_eq!(
            compare_reports(acquire_mismatch, egui.clone()),
            Err(ComparisonError::AcquirePresent)
        );
        let mut readback = direct.clone();
        readback.readback_count = 1;
        assert_eq!(
            compare_reports(readback, egui.clone()),
            Err(ComparisonError::Readback)
        );
        let mut creations = direct.clone();
        creations.resource_creations.measured.buffers = 1;
        assert_eq!(
            compare_reports(creations, egui.clone()),
            Err(ComparisonError::MeasuredResourceCreation)
        );
        let mut incomplete = direct;
        incomplete.completeness = EvidenceCompleteness::Incomplete {
            reason: "missing rss".to_owned(),
        };
        assert!(matches!(
            compare_reports(incomplete, egui),
            Err(ComparisonError::Report(ReportError::Completeness(
                CompletenessError::Incomplete
            )))
        ));
    }

    #[test]
    fn comparison_artifact_is_atomic_and_failure_preserves_existing_output() {
        let directory = std::env::temp_dir().join(format!(
            "g0-9-compare-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        let output = directory.join("comparison.json");
        std::fs::write(&output, b"old artifact").unwrap();
        let mut invalid = valid_report(RendererMode::DirectVello);
        invalid.input_timing.samples = 99;
        assert!(
            write_comparison_artifact(&output, invalid, valid_report(RendererMode::EguiVello),)
                .is_err()
        );
        assert_eq!(std::fs::read(&output).unwrap(), b"old artifact");
        write_comparison_artifact(
            &output,
            valid_report(RendererMode::DirectVello),
            valid_report(RendererMode::EguiVello),
        )
        .unwrap();
        assert_ne!(std::fs::read(&output).unwrap(), b"old artifact");
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn raw_report_json_rejects_historical_overlap_fixture() {
        let historical = r#"{
            "renderer": "direct_vello",
            "pass": true,
            "ticket": "CU-0G02",
            "acquire_count": 100,
            "median_frame_ms": 16.0,
            "present_count": 100,
            "readback_count": 0
        }"#;

        assert!(serde_json::from_str::<RawReport>(historical).is_err());
    }

    #[test]
    fn raw_report_json_rejects_unknown_missing_and_duplicate_fields() {
        let report = valid_report(RendererMode::DirectVello);
        let encoded = serde_json::to_string(&report).unwrap();
        assert_eq!(serde_json::from_str::<RawReport>(&encoded).unwrap(), report);

        let mut unknown: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        unknown["historical_winner"] = serde_json::Value::String("no".to_owned());
        assert!(serde_json::from_value::<RawReport>(unknown).is_err());

        let mut missing: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        missing.as_object_mut().unwrap().remove("rss");
        assert!(serde_json::from_value::<RawReport>(missing).is_err());

        let duplicate = encoded.replacen('{', "{\"renderer\":\"direct_vello\",", 1);
        assert!(serde_json::from_str::<RawReport>(&duplicate).is_err());
    }
}
