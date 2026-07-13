//! D1i-3: BlendMode の意味論ゴールデン(S16)。
//! 閉集合・D3→CompositeMode 1:1 写像・premul 合成式を固定する。
//! 本ファイルのアサーション更新は禁止(新variant+新ファイルのみ)。

use std::collections::BTreeMap;

use motolii_core::{ColorSpace, FrameDesc, PixelFormat, RationalTime, TimeMap};
use motolii_doc::{
    build_document_frame_graph, BlendMode, Clip, ClipSource, DocParam, Document, EvaluationTime,
    ItemEnvelope, Track, TrackItem, RECT_LAYER_SOURCE,
};
use motolii_eval::DataTracks;
use motolii_nodes::CompositeMode;
use motolii_plugin::reference::register_reference_plugins;
use motolii_plugin::PluginRegistry;
use motolii_render::RenderStep;
use motolii_testkit::cpu_reference::{premul_add_u8, premul_multiply_u8, premul_over_u8};

fn desc() -> FrameDesc {
    FrameDesc::packed(8, 4, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true)
}

fn rect_clip(layer: u64, color: [f64; 4]) -> Clip {
    Clip {
        envelope: ItemEnvelope::new(motolii_doc::LayerId::from_raw(layer)),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(10, 1).unwrap(),
        time_map: TimeMap::identity(),
        source: ClipSource::Plugin {
            plugin_id: RECT_LAYER_SOURCE.into(),
            effect_version: 1,
            params: BTreeMap::from([
                ("center".into(), DocParam::const_vec2([0.0, 0.0])),
                ("size".into(), DocParam::const_vec2([1.0, 1.0])),
                ("color".into(), DocParam::const_color(color)),
            ]),
            extra: Default::default(),
        },
    }
}

fn composite_mode_in_graph(blend: BlendMode) -> CompositeMode {
    let mut doc = Document::new_v1();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let bg = doc.layers.allocate("bg").unwrap();
    let fg = doc.layers.allocate("fg").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let mut bg_clip = rect_clip(bg.get(), [0.0, 0.0, 1.0, 1.0]);
    bg_clip.envelope.layer_id = bg;
    let mut fg_clip = rect_clip(fg.get(), [1.0, 0.0, 0.0, 0.5]);
    fg_clip.envelope.layer_id = fg;
    fg_clip.envelope.blend = blend;
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(bg_clip), TrackItem::Clip(fg_clip)],
    });
    let mut registry = PluginRegistry::new();
    register_reference_plugins(&mut registry).unwrap();
    let built = build_document_frame_graph(
        &doc,
        EvaluationTime::new(RationalTime::ZERO),
        desc(),
        &DataTracks::new(),
        &registry,
        None,
    )
    .unwrap();
    built
        .graph
        .steps
        .iter()
        .find_map(|s| match s {
            RenderStep::Composite { mode, .. } => Some(*mode),
            _ => None,
        })
        .expect("graph must contain Composite step for two drawn clips")
}

#[test]
fn blend_mode_closed_set_rejects_unknown_on_deserialize() {
    let err = serde_json::from_str::<BlendMode>(r#""screen""#).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("unknown variant") || msg.contains("did not match"),
        "unknown BlendMode must hard-fail deserialize, got: {msg}"
    );
}

#[test]
fn blend_mode_serde_roundtrip_closed_variants() {
    for mode in [BlendMode::Normal, BlendMode::Add, BlendMode::Multiply] {
        let json = serde_json::to_string(&mode).unwrap();
        let back: BlendMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, mode);
    }
    assert_eq!(
        serde_json::from_str::<BlendMode>(r#""normal""#).unwrap(),
        BlendMode::Normal
    );
    assert_eq!(
        serde_json::from_str::<BlendMode>(r#""add""#).unwrap(),
        BlendMode::Add
    );
    assert_eq!(
        serde_json::from_str::<BlendMode>(r#""multiply""#).unwrap(),
        BlendMode::Multiply
    );
}

#[test]
fn doc_blend_maps_one_to_one_onto_composite_mode() {
    assert_eq!(
        composite_mode_in_graph(BlendMode::Normal),
        CompositeMode::Normal
    );
    assert_eq!(composite_mode_in_graph(BlendMode::Add), CompositeMode::Add);
    assert_eq!(
        composite_mode_in_graph(BlendMode::Multiply),
        CompositeMode::Multiply
    );
}

#[test]
fn premul_blend_formulas_match_cpu_reference() {
    // GPU シェーダ(composite_blend.wgsl)と同一式の CPU 正本。画素審判の期待値置き場。
    let bg = [0u8, 128, 0, 128];
    let fg = [128u8, 0, 0, 128];
    assert_eq!(premul_over_u8(bg, fg), [128, 64, 0, 192]);
    assert_eq!(premul_add_u8(bg, fg), [128, 128, 0, 255]);
    assert_eq!(premul_multiply_u8(bg, fg), [64, 64, 0, 192]);
}

#[test]
fn premul_multiply_keeps_uncovered_and_source_over_alpha() {
    // fg 透明 → bg を残す。両不透明 → rgb 積・α source-over。
    assert_eq!(
        premul_multiply_u8([0, 200, 100, 255], [0, 0, 0, 0]),
        [0, 200, 100, 255]
    );
    assert_eq!(
        premul_multiply_u8([200, 100, 50, 255], [128, 64, 32, 255]),
        [100, 25, 6, 255]
    );
}
