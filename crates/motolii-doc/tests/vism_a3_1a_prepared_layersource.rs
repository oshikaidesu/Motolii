//! VSM-A3-1a: P3_prepared_rename_raw_unchanged — clear LayerSource は prepared.params のみを評価する。

use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

use motolii_core::{ColorSpace, FrameDesc, PixelFormat, RationalTime, TimeMap};
use motolii_doc::{
    build_document_frame_graph, Clip, ClipSource, DocParam, Document, EvaluationTime, ItemEnvelope,
    Track, TrackItem, CLEAR_LAYER_SOURCE,
};
use motolii_eval::{DataTracks, Value};
use motolii_plugin::{
    GpuCtx, LayerSourceContext, LayerSourcePlugin, MigrationOp, MigrationStep, NodeDesc, ParamDef,
    PipelineCache, PluginCatalog, PluginCatalogBuilder, PluginContract, PluginError, PluginId,
    PluginKind, PluginRegistry, PluginRuntime, ResolvedParams, TextureRef, ValueType,
};
use motolii_render::RenderStep;

const P3_COLOR: [f64; 4] = [0.2, 0.4, 0.6, 1.0];

struct TestClearV2;

static TEST_CLEAR_V2_DESC: OnceLock<NodeDesc> = OnceLock::new();

fn test_clear_v2_desc() -> &'static NodeDesc {
    TEST_CLEAR_V2_DESC.get_or_init(|| NodeDesc {
        id: PluginId("core.layer_source.clear"),
        version: 2,
        display_name: "Clear Layer Source",
        category: "Generate",
        tags: &["clear", "test"],
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

impl LayerSourcePlugin for TestClearV2 {
    fn desc(&self) -> &NodeDesc {
        test_clear_v2_desc()
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

static TEST_CLEAR_V2: TestClearV2 = TestClearV2;

fn p3_rename_catalog() -> PluginCatalog {
    let mut builder = PluginCatalogBuilder::new();
    builder
        .register(PluginContract {
            kind: PluginKind::LayerSource,
            node: test_clear_v2_desc().clone(),
            migrations: vec![MigrationStep {
                from_version: 1,
                to_version: 2,
                ops: vec![MigrationOp::RenameParam {
                    from: "old_color",
                    to: "color",
                }],
            }],
        })
        .unwrap();
    builder.build().unwrap()
}

fn p3_runtime() -> PluginRuntime {
    let mut registry = PluginRegistry::new();
    registry.register_layer_source(&TEST_CLEAR_V2).unwrap();
    PluginRuntime::try_new(Arc::new(p3_rename_catalog()), registry).unwrap()
}

fn p3_clear_document() -> Document {
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
                params: BTreeMap::from([("old_color".into(), DocParam::const_color(P3_COLOR))]),
                extra: Default::default(),
            },
        })],
    });
    doc.validate().unwrap();
    doc
}

fn clip_params(doc: &Document) -> BTreeMap<String, DocParam> {
    match &doc.tracks[0].items[0] {
        TrackItem::Clip(clip) => match &clip.source {
            ClipSource::Plugin { params, .. } => params.clone(),
            _ => panic!("expected plugin clip source"),
        },
        TrackItem::Group(_) => panic!("expected clip"),
    }
}

#[test]
fn p3_prepared_rename_raw_unchanged() {
    let runtime = p3_runtime();
    let doc = p3_clear_document();
    let doc_before = doc.clone();
    let raw_before = clip_params(&doc);
    assert!(raw_before.contains_key("old_color"));
    assert!(!raw_before.contains_key("color"));

    let built = build_document_frame_graph(
        &doc,
        EvaluationTime::new(RationalTime::ZERO),
        FrameDesc::packed(16, 8, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true),
        &DataTracks::new(),
        &runtime,
        None,
    )
    .unwrap();

    let raw_after = clip_params(&doc);
    assert_eq!(doc, doc_before);
    assert_eq!(raw_before, raw_after);
    assert!(raw_after.contains_key("old_color"));
    assert!(!raw_after.contains_key("color"));

    let (plugin_id, plugin_params) = built
        .graph
        .steps
        .iter()
        .find_map(|step| match step {
            RenderStep::Plugin { id, params, .. } => Some((id, params)),
            _ => None,
        })
        .expect("clear clip should lower to RenderStep::Plugin");

    assert_eq!(plugin_id.0, CLEAR_LAYER_SOURCE);
    assert_eq!(plugin_params.get("color"), Some(&Value::Color(P3_COLOR)));
    assert!(plugin_params.get("old_color").is_none());
}
