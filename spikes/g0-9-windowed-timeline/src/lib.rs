use std::{fmt, sync::Arc};

use fontique::{Collection, CollectionOptions, FontStyle, FontWeight, FontWidth};
use harfrust::{FontRef, ShaperData, UnicodeBuffer};
use serde::{Serialize, Serializer};
use sha2::{Digest, Sha256};
use thiserror::Error;

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
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

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MeasurementSummary {
    pub samples: usize,
    pub median_frame_ms: f64,
    pub p95_frame_ms: f64,
    pub max_frame_ms: f64,
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
    Some(MeasurementSummary {
        samples: ordered.len(),
        median_frame_ms: percentile(0.5),
        p95_frame_ms: percentile(0.95),
        max_frame_ms: *ordered.last().unwrap(),
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
            .and_then(|tail| tail.split("#[derive(Serialize)]").next())
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
}
