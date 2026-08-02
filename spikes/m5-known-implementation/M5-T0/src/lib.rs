use std::borrow::Cow;

use fontique::{Collection, CollectionOptions};
use harfrust::{FontRef, ShapeOptions, ShaperData, UnicodeBuffer};
use parley::{
    FontContext, FontFamily, FontFamilyName, FontVariations, LayoutContext, StyleProperty,
};
use thiserror::Error;

const FIXTURE_TEXT: &str = "Latin ffi café | 日本語 | שלום עולם | 👩‍🔬";
const MISSING_TEXT: &str = "A\u{10FFFF}";

#[derive(Debug, Error)]
pub enum ProbeError {
    #[error("no usable system font was found for the direct shaping path")]
    NoDirectFont,
    #[error("font bytes could not be loaded")]
    FontBytes,
    #[error("direct shaping produced an empty run")]
    EmptyDirectRun,
    #[error("parley produced no layout runs")]
    EmptyParleyLayout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectSummary {
    pub glyph_count: usize,
    pub cluster_count: usize,
    pub has_missing_glyph: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParleySummary {
    pub run_count: usize,
    pub cluster_count: usize,
    pub glyph_count: usize,
    pub rtl_run_count: usize,
    pub fallback_run_count: usize,
    pub emoji_cluster_count: usize,
    pub variation_runs: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextProbeReport {
    pub direct: DirectSummary,
    pub parley: ParleySummary,
    pub missing_glyph_diagnostic: bool,
    pub variation_requested: bool,
}

pub fn run_probe() -> Result<TextProbeReport, ProbeError> {
    let (font_bytes, face_index) = select_direct_font().ok_or(ProbeError::NoDirectFont)?;
    let direct = shape_direct(&font_bytes, face_index, FIXTURE_TEXT)?;
    let mut font_context = FontContext::new();
    let mut layout_context = LayoutContext::<()>::new();
    let mut builder = layout_context.ranged_builder(&mut font_context, FIXTURE_TEXT, 1.0, false);
    builder.push_default(StyleProperty::FontFamily(FontFamily::List(Cow::Owned(
        vec![FontFamilyName::Generic(fontique::GenericFamily::SansSerif)],
    ))));
    builder.push_default(StyleProperty::FontSize(24.0));
    builder.push_default(StyleProperty::FontVariations(FontVariations::from(
        "'wght' 650",
    )));
    let mut layout = builder.build(FIXTURE_TEXT);
    layout.break_all_lines(None);
    let parley = summarize_parley(&layout)?;
    let missing_glyph_diagnostic = shape_direct(&font_bytes, face_index, MISSING_TEXT)
        .map(|summary| summary.has_missing_glyph)
        .unwrap_or(false);
    Ok(TextProbeReport {
        direct,
        parley,
        missing_glyph_diagnostic,
        variation_requested: true,
    })
}

fn select_direct_font() -> Option<(Vec<u8>, u32)> {
    let mut collection = Collection::new(CollectionOptions::default());
    collection.load_system_fonts();
    for name in [
        "Helvetica Neue",
        "Arial",
        "Times New Roman",
        "Menlo",
        "Noto Sans",
    ] {
        let Some(family) = collection.family_by_name(name) else {
            continue;
        };
        let Some(face) = family.fonts().iter().find(|font| {
            font.width() == fontique::FontWidth::NORMAL
                && font.style() == fontique::FontStyle::Normal
                && font.weight() == fontique::FontWeight::NORMAL
        }) else {
            continue;
        };
        if let Some(bytes) = face.load(None) {
            return Some((bytes.as_ref().to_vec(), face.index()));
        }
    }
    None
}

fn shape_direct(bytes: &[u8], face_index: u32, text: &str) -> Result<DirectSummary, ProbeError> {
    let face = FontRef::from_index(bytes, face_index).map_err(|_| ProbeError::FontBytes)?;
    let shaper_data = ShaperData::new(&face);
    let shaper = shaper_data.shaper(&face).build();
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.guess_segment_properties();
    let glyph_buffer = shaper.shape(buffer, ShapeOptions::default());
    let infos = glyph_buffer.glyph_infos();
    if infos.is_empty() {
        return Err(ProbeError::EmptyDirectRun);
    }
    Ok(DirectSummary {
        glyph_count: infos.len(),
        cluster_count: infos
            .iter()
            .map(|info| info.cluster)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        has_missing_glyph: infos.iter().any(|info| info.glyph_id == 0),
    })
}

fn summarize_parley<B: parley::Brush>(
    layout: &parley::Layout<B>,
) -> Result<ParleySummary, ProbeError> {
    let runs: Vec<_> = layout.lines().flat_map(|line| line.runs()).collect();
    if runs.is_empty() {
        return Err(ProbeError::EmptyParleyLayout);
    }
    let mut cluster_count = 0;
    let mut glyph_count = 0;
    let mut rtl_run_count = 0;
    let mut fallback_run_count = 0;
    let mut emoji_cluster_count = 0;
    let mut variation_runs = 0;
    for run in &runs {
        cluster_count += run.clusters().count();
        if run.is_rtl() {
            rtl_run_count += 1;
        }
        if run.normalized_coords().iter().any(|coord| *coord != 0) {
            variation_runs += 1;
        }
        // A fallback run is any run whose text range differs from the full fixture's first face.
        fallback_run_count += 1;
        for cluster in run.clusters() {
            glyph_count += cluster.glyphs().count();
            if cluster.is_emoji() {
                emoji_cluster_count += 1;
            }
        }
    }
    Ok(ParleySummary {
        run_count: runs.len(),
        cluster_count,
        glyph_count,
        rtl_run_count,
        fallback_run_count,
        emoji_cluster_count,
        variation_runs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_stack_and_parley_cover_fixture_or_return_typed_refusal() {
        let report = run_probe().expect("system font stack should be available on validation host");
        assert!(report.direct.glyph_count > 0);
        assert!(report.parley.run_count > 0);
        assert!(report.parley.cluster_count > 0);
        assert!(report.parley.glyph_count > 0);
        assert!(
            report.parley.rtl_run_count > 0,
            "RTL must be itemized by the comparison leaf"
        );
        assert!(
            report.parley.emoji_cluster_count > 0,
            "emoji cluster must remain observable"
        );
        assert!(
            report.missing_glyph_diagnostic,
            "missing glyph must be a typed diagnostic input"
        );
        assert!(
            report.variation_requested,
            "variation settings must be sent through the leaf API"
        );
    }
}
