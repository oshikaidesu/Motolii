//! 音MAD personaのDocument→render graph要求生成を測る手動bench。

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat, RationalTime, TimeMap};
use motolii_doc::{
    build_document_frame_graph, Clip, ClipSource, Composition, DocParam, Document,
    EffectDefinition, EffectDefinitionId, EffectId, EffectUse, EvaluationTime, ItemEnvelope, Track,
    TrackItem, MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS,
};
use motolii_eval::DataTracks;
use motolii_plugins_firstparty::first_party_runtime;
use serde::Serialize;

const OUTPUT_ENV: &str = "MOTOLII_AUDIO_MAD_DEMAND_OUT";
const CLIP_COUNT: usize = 1_000;
const ASSET_COUNT: usize = 16;
const EFFECTS_PER_CLIP: usize = 3;
const SAMPLE_COUNT: usize = 300;

#[derive(Debug, Serialize)]
struct GraphDemandSample {
    frame: i64,
    elapsed_ms: f64,
    active_video_slots: usize,
    graph_steps: usize,
}

#[derive(Debug, Serialize)]
struct AudioMadDemandReport {
    schema_version: u32,
    clip_count: usize,
    asset_count: usize,
    effects_per_clip: usize,
    clip_duration_frames: i64,
    sequential: Vec<GraphDemandSample>,
    scrub: Vec<GraphDemandSample>,
}

fn fixture() -> Document {
    let fps = Fps::try_new(30, 1).unwrap();
    let mut doc = Document::new_current();
    doc.composition =
        Composition::try_new(1920, 1080, RationalTime::try_new(60, 1).unwrap(), fps).unwrap();
    let assets: Vec<_> = (0..ASSET_COUNT)
        .map(|index| {
            doc.assets
                .allocate(
                    format!("source-{index}"),
                    "video/mp4",
                    format!("fixture-hash-{index}"),
                )
                .unwrap()
        })
        .collect();
    let definition_id = EffectDefinitionId::from_raw(doc.next_stable_id.allocate().unwrap());
    doc.effect_definitions.push(EffectDefinition::new(
        definition_id,
        "core.filter.opacity",
        1,
        true,
        BTreeMap::from([("amount".into(), DocParam::const_f64(0.9))]),
        Default::default(),
    ));
    doc.version = doc.version.max(MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS);
    doc.min_reader_version = doc
        .min_reader_version
        .max(MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS);

    let track_id = doc.track_ids.allocate("V1").unwrap();
    let items = (0..CLIP_COUNT)
        .map(|index| {
            let layer = doc.layers.allocate(format!("clip-{index}")).unwrap();
            let effects = (0..EFFECTS_PER_CLIP)
                .map(|_| EffectUse {
                    id: EffectId::from_raw(doc.next_stable_id.allocate().unwrap()),
                    definition_id,
                })
                .collect();
            let mut envelope = ItemEnvelope::new(layer);
            envelope.effects = effects;
            TrackItem::Clip(Clip {
                envelope,
                start: RationalTime::try_from_frame(index as i64, fps).unwrap(),
                duration: RationalTime::try_from_frame(4, fps).unwrap(),
                time_map: TimeMap::identity(),
                source: ClipSource::asset_video_only(assets[index % assets.len()]),
            })
        })
        .collect();
    doc.tracks.push(Track {
        id: track_id,
        items,
    });
    doc.validate().unwrap();
    doc
}

fn measure_frames(doc: &Document, frames: impl Iterator<Item = i64>) -> Vec<GraphDemandSample> {
    let runtime = first_party_runtime().unwrap();
    let data_tracks = DataTracks::new();
    let fps = Fps::try_new(30, 1).unwrap();
    let desc = FrameDesc::packed(1920, 1080, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
    frames
        .map(|frame| {
            let start = Instant::now();
            let built = build_document_frame_graph(
                doc,
                EvaluationTime::new(RationalTime::try_from_frame(frame, fps).unwrap()),
                desc,
                &data_tracks,
                &runtime,
                None,
            )
            .unwrap();
            GraphDemandSample {
                frame,
                elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
                active_video_slots: built.video_slots.len(),
                graph_steps: built.graph.steps.len(),
            }
        })
        .collect()
}

#[test]
#[ignore = "manual density benchmark; run --release with --ignored --nocapture"]
fn record_audio_mad_graph_demand_without_thresholds() {
    let doc = fixture();
    let sequential = measure_frames(&doc, (0..SAMPLE_COUNT).map(|frame| frame as i64));
    let scrub = measure_frames(
        &doc,
        (0..SAMPLE_COUNT).map(|index| ((index * 137) % CLIP_COUNT) as i64),
    );
    assert!(sequential
        .iter()
        .chain(&scrub)
        .all(|sample| sample.active_video_slots <= 4));
    let report = AudioMadDemandReport {
        schema_version: 1,
        clip_count: CLIP_COUNT,
        asset_count: ASSET_COUNT,
        effects_per_clip: EFFECTS_PER_CLIP,
        clip_duration_frames: 4,
        sequential,
        scrub,
    };
    let json = serde_json::to_string_pretty(&report).unwrap();
    eprintln!("{json}");
    if let Some(path) = std::env::var_os(OUTPUT_ENV) {
        std::fs::write(&path, &json)
            .unwrap_or_else(|error| panic!("write {}: {error}", PathBuf::from(path).display()));
    }
}
