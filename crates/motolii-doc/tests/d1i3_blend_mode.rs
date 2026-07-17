#![allow(deprecated)]

//! D1i-3: BlendMode の意味論ゴールデン(S16)。
//! 閉集合・D3→CompositeMode 1:1 写像・premul 合成式を固定する。
//! 期待値の正本は `oracles/d1i3_blend_mode.tsv`。本ファイルは変更可能なharness。

use std::collections::BTreeMap;

use motolii_core::{ColorSpace, FrameDesc, PixelFormat, RationalTime, TimeMap};
use motolii_doc::{
    build_document_frame_graph, BlendMode, Clip, ClipSource, DocParam, Document, EvaluationTime,
    ItemEnvelope, Track, TrackItem, RECT_LAYER_SOURCE,
};
use motolii_eval::DataTracks;
use motolii_nodes::CompositeMode;
use motolii_plugin::reference::{reference_catalog, register_reference_plugins};
use motolii_plugin::{PluginRegistry, PluginRuntime};
use motolii_render::RenderStep;
use motolii_testkit::cpu_reference::{premul_add_u8, premul_multiply_u8, premul_over_u8};

const ORACLE: &str = include_str!("oracles/d1i3_blend_mode.tsv");

#[derive(Debug)]
struct MapCase {
    serde_name: &'static str,
    composite_name: &'static str,
}

#[derive(Debug)]
struct PremulCase {
    group: &'static str,
    operation: &'static str,
    bg: [u8; 4],
    fg: [u8; 4],
    expected: [u8; 4],
}

fn rgba8(value: &str) -> [u8; 4] {
    let values = value
        .split(',')
        .map(|component| component.parse::<u8>().expect("oracle rgba8 component"))
        .collect::<Vec<_>>();
    values
        .try_into()
        .expect("oracle rgba8 must have 4 components")
}

fn oracle_cases() -> (Vec<MapCase>, Vec<PremulCase>) {
    let mut maps = Vec::new();
    let mut premul = Vec::new();
    for line in ORACLE.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        match fields.as_slice() {
            ["map", serde_name, composite_name] => maps.push(MapCase {
                serde_name,
                composite_name,
            }),
            ["premul", group, operation, bg, fg, expected] => premul.push(PremulCase {
                group,
                operation,
                bg: rgba8(bg),
                fg: rgba8(fg),
                expected: rgba8(expected),
            }),
            _ => panic!("malformed BlendMode semantic oracle line: {line}"),
        }
    }
    assert!(!maps.is_empty(), "BlendMode map oracle must not be empty");
    assert!(
        !premul.is_empty(),
        "BlendMode premul oracle must not be empty"
    );
    (maps, premul)
}

fn blend_mode(serde_name: &str) -> BlendMode {
    serde_json::from_str(&format!("\"{serde_name}\"")).expect("oracle BlendMode serde name")
}

fn composite_mode(name: &str) -> CompositeMode {
    match name {
        "normal" => CompositeMode::Normal,
        "add" => CompositeMode::Add,
        "multiply" => CompositeMode::Multiply,
        other => panic!("unknown CompositeMode name in semantic oracle: {other}"),
    }
}

fn assert_premul_group(group: &str) {
    let (_, cases) = oracle_cases();
    let mut matched = 0;
    for case in cases.into_iter().filter(|case| case.group == group) {
        matched += 1;
        let actual = match case.operation {
            "over" => premul_over_u8(case.bg, case.fg),
            "add" => premul_add_u8(case.bg, case.fg),
            "multiply" => premul_multiply_u8(case.bg, case.fg),
            other => panic!("unknown premul operation in semantic oracle: {other}"),
        };
        assert_eq!(actual, case.expected, "oracle case: {case:?}");
    }
    assert!(
        matched > 0,
        "premul oracle group must not be empty: {group}"
    );
}

fn desc() -> FrameDesc {
    FrameDesc::packed(8, 4, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true)
}

fn reference_runtime() -> PluginRuntime {
    let mut registry = PluginRegistry::new();
    register_reference_plugins(&mut registry).unwrap();
    PluginRuntime::try_new(std::sync::Arc::new(reference_catalog().unwrap()), registry).unwrap()
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
    // A4: 同一Track内の時間重なりは禁止。合成検査は別Trackへ分ける。
    let track_bg = doc.track_ids.allocate("V1").unwrap();
    let track_fg = doc.track_ids.allocate("V2").unwrap();
    let mut bg_clip = rect_clip(bg.get(), [0.0, 0.0, 1.0, 1.0]);
    bg_clip.envelope.layer_id = bg;
    let mut fg_clip = rect_clip(fg.get(), [1.0, 0.0, 0.0, 0.5]);
    fg_clip.envelope.layer_id = fg;
    fg_clip.envelope.blend = blend;
    doc.tracks.push(Track {
        id: track_bg,
        items: vec![TrackItem::Clip(bg_clip)],
    });
    doc.tracks.push(Track {
        id: track_fg,
        items: vec![TrackItem::Clip(fg_clip)],
    });
    doc.validate()
        .expect("blend golden document must validate (A4)");
    let runtime = reference_runtime();
    let built = build_document_frame_graph(
        &doc,
        EvaluationTime::new(RationalTime::ZERO),
        desc(),
        &DataTracks::new(),
        &runtime,
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
    let (maps, _) = oracle_cases();
    for case in maps {
        let mode = blend_mode(case.serde_name);
        let json = serde_json::to_string(&mode).unwrap();
        let back: BlendMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, mode);
        assert_eq!(json, format!("\"{}\"", case.serde_name));
    }
}

#[test]
fn doc_blend_maps_one_to_one_onto_composite_mode() {
    let (maps, _) = oracle_cases();
    for case in maps {
        assert_eq!(
            composite_mode_in_graph(blend_mode(case.serde_name)),
            composite_mode(case.composite_name)
        );
    }
}

#[test]
fn premul_blend_formulas_match_cpu_reference() {
    assert_premul_group("formula");
}

#[test]
fn premul_multiply_keeps_uncovered_and_source_over_alpha() {
    assert_premul_group("multiply_edge");
}
