//! VSM-A3-1c: catalog-backed LayerSource の一般 lowering（ID allowlist なし）。

use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

use motolii_core::{ColorSpace, FrameDesc, PixelFormat, RationalTime, TimeMap};
use motolii_doc::{
    build_document_frame_graph, Clip, ClipSource, DocParam, Document, EvaluationTime, GraphError,
    ItemEnvelope, PluginDiagnosticReason, Track, TrackItem, CLEAR_LAYER_SOURCE, RECT_LAYER_SOURCE,
};
use motolii_eval::{DataTracks, Value};
use motolii_plugin::reference::{register_reference_contracts, CLEAR_LAYER_SOURCE as CLEAR_LS};
use motolii_plugin::{
    GpuCtx, LayerSourceContext, LayerSourcePlugin, NodeDesc, ParamDef, PipelineCache,
    PluginCatalog, PluginCatalogBuilder, PluginContract, PluginError, PluginId, PluginKind,
    PluginRegistry, PluginRuntime, ResolvedParams, TextureRef, ValueType,
};
use motolii_plugins_firstparty::first_party_catalog;
use motolii_render::RenderStep;

const P1_PLUGIN_ID: &str = "test.layer_source.zero_input_fixture";
const P1_COLOR: [f64; 4] = [0.1, 0.2, 0.3, 1.0];
const P2_COLOR: [f64; 4] = [0.2, 0.4, 0.6, 1.0];
const P4_CENTER: [f64; 2] = [0.1, -0.2];
const P4_SIZE: [f64; 2] = [0.3, 0.4];
const P4_COLOR: [f64; 4] = [0.5, 0.6, 0.7, 0.8];

struct TestZeroInputLayerSource;

static TEST_ZERO_INPUT_DESC: OnceLock<NodeDesc> = OnceLock::new();

fn test_zero_input_desc() -> &'static NodeDesc {
    TEST_ZERO_INPUT_DESC.get_or_init(|| NodeDesc {
        id: PluginId(P1_PLUGIN_ID),
        version: 1,
        display_name: "Test Zero Input Layer Source",
        category: "Generate",
        tags: &["test", "fixture"],
        params: vec![ParamDef {
            id: "color",
            value_type: ValueType::Color,
            default: Value::Color([0.0, 0.0, 0.0, 0.0]),
            f64_domain: None,
        }],
        min_inputs: 0,
        max_inputs: 0,
    })
}

impl LayerSourcePlugin for TestZeroInputLayerSource {
    fn desc(&self) -> &NodeDesc {
        test_zero_input_desc()
    }

    fn render(
        &self,
        _gpu: &GpuCtx,
        _pipelines: &mut PipelineCache,
        _encoder: &mut motolii_plugin::wgpu::CommandEncoder,
        _t: RationalTime,
        _params: &ResolvedParams,
        _ctx: LayerSourceContext,
        _output: TextureRef<'_>,
    ) -> Result<(), PluginError> {
        Ok(())
    }
}

static TEST_ZERO_INPUT: TestZeroInputLayerSource = TestZeroInputLayerSource;

fn p1_catalog() -> PluginCatalog {
    let mut builder = PluginCatalogBuilder::new();
    builder
        .register(PluginContract {
            kind: PluginKind::LayerSource,
            node: test_zero_input_desc().clone(),
            migrations: vec![],
        })
        .unwrap();
    builder.build().unwrap()
}

fn p1_runtime() -> PluginRuntime {
    let mut registry = PluginRegistry::new();
    registry.register_layer_source(&TEST_ZERO_INPUT).unwrap();
    PluginRuntime::try_new(Arc::new(p1_catalog()), registry).unwrap()
}

fn p1_document() -> Document {
    let mut doc = Document::new_current();
    let layer = doc.layers.allocate("zero-input").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    doc.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::from_seconds(1),
            time_map: TimeMap::identity(),
            source: ClipSource::Plugin {
                plugin_id: P1_PLUGIN_ID.into(),
                effect_version: 1,
                params: BTreeMap::from([("color".into(), DocParam::const_color(P1_COLOR))]),
                extra: Default::default(),
            },
        })],
    });
    doc.validate().unwrap();
    doc
}

fn clear_layer_source_document(color: [f64; 4]) -> Document {
    let mut doc = Document::new_current();
    let layer = doc.layers.allocate("clear").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    doc.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::from_seconds(1),
            time_map: TimeMap::identity(),
            source: ClipSource::Plugin {
                plugin_id: CLEAR_LAYER_SOURCE.into(),
                effect_version: 1,
                params: BTreeMap::from([("color".into(), DocParam::const_color(color))]),
                extra: Default::default(),
            },
        })],
    });
    doc.validate().unwrap();
    doc
}

fn clear_layer_source_catalog() -> PluginCatalog {
    let mut builder = PluginCatalogBuilder::new();
    register_reference_contracts(&mut builder).unwrap();
    builder.build().unwrap()
}

fn clear_layer_source_runtime() -> PluginRuntime {
    let mut registry = PluginRegistry::new();
    registry.register_layer_source(&CLEAR_LS).unwrap();
    PluginRuntime::try_new(Arc::new(clear_layer_source_catalog()), registry).unwrap()
}

fn p4_rect_document() -> Document {
    let mut doc = Document::new_current();
    let layer = doc.layers.allocate("rect").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    doc.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::from_seconds(1),
            time_map: TimeMap::identity(),
            source: ClipSource::Plugin {
                plugin_id: RECT_LAYER_SOURCE.into(),
                effect_version: 1,
                params: BTreeMap::from([
                    ("center".into(), DocParam::const_vec2(P4_CENTER)),
                    ("size".into(), DocParam::const_vec2(P4_SIZE)),
                    ("color".into(), DocParam::const_color(P4_COLOR)),
                ]),
                extra: Default::default(),
            },
        })],
    });
    doc.validate().unwrap();
    doc
}

fn p4_empty_runtime() -> PluginRuntime {
    PluginRuntime::try_new(
        Arc::new(PluginCatalogBuilder::new().build().unwrap()),
        PluginRegistry::new(),
    )
    .unwrap()
}

fn plugin_step(graph: &motolii_doc::DocumentFrameGraph) -> (&PluginId, &ResolvedParams) {
    graph
        .graph
        .steps
        .iter()
        .find_map(|step| match step {
            RenderStep::Plugin { id, params, .. } => Some((id, params)),
            _ => None,
        })
        .expect("expected RenderStep::Plugin")
}

#[test]
fn p1_registered_zero_input_layer_source_no_allowlist() {
    let runtime = p1_runtime();
    let built = build_document_frame_graph(
        &p1_document(),
        EvaluationTime::new(RationalTime::ZERO),
        FrameDesc::packed(16, 8, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true),
        &DataTracks::new(),
        &runtime,
        None,
    )
    .unwrap();

    let (plugin_id, params) = plugin_step(&built);
    assert_eq!(plugin_id.0, P1_PLUGIN_ID);
    assert_eq!(params.get("color"), Some(&Value::Color(P1_COLOR)));
}

#[test]
fn p2_clear_general_path_same_semantics() {
    let runtime = clear_layer_source_runtime();
    let built = build_document_frame_graph(
        &clear_layer_source_document(P2_COLOR),
        EvaluationTime::new(RationalTime::ZERO),
        FrameDesc::packed(16, 8, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true),
        &DataTracks::new(),
        &runtime,
        None,
    )
    .unwrap();

    let (plugin_id, params) = plugin_step(&built);
    assert_eq!(plugin_id.0, CLEAR_LAYER_SOURCE);
    assert_eq!(params.get("color"), Some(&Value::Color(P2_COLOR)));
}

#[test]
fn n1_executor_missing_export_graph_rejected_layer_source() {
    let runtime = PluginRuntime::try_new(
        Arc::new(clear_layer_source_catalog()),
        PluginRegistry::new(),
    )
    .unwrap();
    let error = build_document_frame_graph(
        &clear_layer_source_document(P2_COLOR),
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
    assert_eq!(diagnostics[0].plugin_id, CLEAR_LAYER_SOURCE);
    assert_eq!(
        diagnostics[0].reason,
        PluginDiagnosticReason::ExecutorMissing
    );
}

#[test]
fn n2_contract_only_runtime_rejected_layer_source() {
    let runtime = PluginRuntime::try_new(
        Arc::new(first_party_catalog().unwrap()),
        PluginRegistry::new(),
    )
    .unwrap();
    let error = build_document_frame_graph(
        &clear_layer_source_document(P2_COLOR),
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
    assert!(diagnostics.iter().any(|d| {
        d.plugin_id == CLEAR_LAYER_SOURCE && d.reason == PluginDiagnosticReason::ExecutorMissing
    }));
}

#[test]
fn p4_rect_overlay_path_unchanged() {
    let runtime = p4_empty_runtime();
    let built = build_document_frame_graph(
        &p4_rect_document(),
        EvaluationTime::new(RationalTime::ZERO),
        FrameDesc::packed(16, 8, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true),
        &DataTracks::new(),
        &runtime,
        None,
    )
    .unwrap();

    let mut overlay_rect_count = 0usize;
    let mut plugin_count = 0usize;
    let mut overlay_rect_step = None;
    for step in &built.graph.steps {
        match step {
            RenderStep::OverlayRect { overlay, .. } => {
                overlay_rect_count += 1;
                overlay_rect_step = Some(overlay);
            }
            RenderStep::Plugin { .. } => plugin_count += 1,
            _ => {}
        }
    }
    assert_eq!(
        overlay_rect_count, 1,
        "rect must lower to exactly one OverlayRect"
    );
    assert_eq!(
        plugin_count, 0,
        "effects-free rect must not emit Plugin steps"
    );

    let overlay = overlay_rect_step.expect("expected OverlayRect step");
    assert_eq!(overlay.center.x, P4_CENTER[0]);
    assert_eq!(overlay.center.y, P4_CENTER[1]);
    assert_eq!(overlay.size.width, P4_SIZE[0]);
    assert_eq!(overlay.size.height, P4_SIZE[1]);
    assert_eq!(
        overlay.color,
        [
            P4_COLOR[0] as f32,
            P4_COLOR[1] as f32,
            P4_COLOR[2] as f32,
            P4_COLOR[3] as f32,
        ]
    );
}

#[test]
fn n7_id_allowlist_unsupported_source_plugin_abolished() {
    let runtime = p1_runtime();
    let result = build_document_frame_graph(
        &p1_document(),
        EvaluationTime::new(RationalTime::ZERO),
        FrameDesc::packed(16, 8, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true),
        &DataTracks::new(),
        &runtime,
        None,
    );
    assert!(
        !matches!(result, Err(GraphError::UnsupportedSourcePlugin(_))),
        "registered non-clear LayerSource must not be rejected by UnsupportedSourcePlugin"
    );
    result.unwrap();
}
