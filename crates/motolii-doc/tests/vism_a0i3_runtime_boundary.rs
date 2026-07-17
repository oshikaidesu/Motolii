//! VSM-A0I-3: graphは検証済みruntimeと実行可能recipeだけを受ける。

use std::collections::BTreeMap;
use std::sync::Arc;

use motolii_core::{ColorSpace, FrameDesc, PixelFormat, RationalTime, TimeMap};
use motolii_doc::{
    build_document_frame_graph, Clip, ClipSource, DocParam, Document, EffectDefinition,
    EffectDefinitionId, EffectId, EffectUse, EvaluationTime, GraphError, ItemEnvelope,
    PluginDiagnosticReason, Track, TrackItem, RECT_LAYER_SOURCE,
};
use motolii_eval::DataTracks;
use motolii_plugin::reference::reference_catalog;
use motolii_plugin::{PluginRegistry, PluginRuntime};

fn opacity_document() -> Document {
    let mut doc = Document::new_current();
    let layer = doc.layers.allocate("subject").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    let use_id = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
    let definition_id = EffectDefinitionId::from_raw(doc.next_stable_id.allocate().unwrap());
    doc.effect_definitions.push(EffectDefinition::new(
        definition_id,
        "core.filter.opacity",
        1,
        true,
        BTreeMap::from([("amount".into(), DocParam::const_f64(0.5))]),
        Default::default(),
    ));
    let mut envelope = ItemEnvelope::new(layer);
    envelope.effects.push(EffectUse {
        id: use_id,
        definition_id,
    });
    doc.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope,
            start: RationalTime::ZERO,
            duration: RationalTime::from_seconds(1),
            time_map: TimeMap::identity(),
            source: ClipSource::Plugin {
                plugin_id: RECT_LAYER_SOURCE.into(),
                effect_version: 1,
                params: BTreeMap::from([
                    ("center".into(), DocParam::const_vec2([0.0, 0.0])),
                    ("size".into(), DocParam::const_vec2([1.0, 1.0])),
                    ("color".into(), DocParam::const_color([1.0, 1.0, 1.0, 1.0])),
                ]),
                extra: Default::default(),
            },
        })],
    });
    doc.validate().unwrap();
    doc
}

#[test]
fn contract_only_runtime_reports_executor_missing_before_graph_build() {
    let runtime = PluginRuntime::try_new(
        Arc::new(reference_catalog().unwrap()),
        PluginRegistry::new(),
    )
    .unwrap();
    let error = build_document_frame_graph(
        &opacity_document(),
        EvaluationTime::new(RationalTime::ZERO),
        FrameDesc::packed(16, 8, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true),
        &DataTracks::new(),
        &runtime,
        None,
    )
    .unwrap_err();

    let GraphError::PluginDiagnostics(diagnostics) = error else {
        panic!("expected runtime diagnostic, got {error:?}");
    };
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].plugin_id, "core.filter.opacity");
    assert_eq!(
        diagnostics[0].reason,
        PluginDiagnosticReason::ExecutorMissing
    );
}
