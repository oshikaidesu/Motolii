//! M2-D3e: Shared Effect Use の prepared 評価接続（P1–P6 / N1–N3）。

use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

use motolii_core::{ColorSpace, FrameDesc, PixelFormat, Quality, RationalTime, TimeMap};
use motolii_doc::{
    build_document_frame_graph, Clip, ClipSource, Composition, DocParam, Document, DocumentError,
    DocumentPluginError, EffectDefinition, EffectDefinitionId, EffectId, EffectUse, EvaluationTime,
    GraphError, Group, ItemEnvelope, PluginDiagnosticReason, PluginSlotId, Track, TrackItem,
    RECT_LAYER_SOURCE,
};
use motolii_eval::{DataTracks, Value};
use motolii_gpu::download_rgba;
use motolii_plugin::{
    FilterPlugin, GpuCtx, MigrationOp, MigrationStep, NodeDesc, ParamDef, PipelineCache,
    PluginCatalog, PluginCatalogBuilder, PluginContract, PluginError, PluginId, PluginKind,
    PluginRegistry, PluginRuntime, RenderCtx, ResolvedParams, TextureRef, ValueType,
};
use motolii_plugins_firstparty::first_party_runtime;
use motolii_render::{render_graph_cached, RenderGraphInputs, RenderSession, RenderStep};
use motolii_testkit::{assert_rgba_close, gpu_or_skip, tol, RgbaImageDesc};

const W: u32 = 48;
const H: u32 = 16;

fn desc() -> FrameDesc {
    FrameDesc::packed(W, H, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true)
}

fn image_desc() -> RgbaImageDesc {
    RgbaImageDesc {
        width: W,
        height: H,
    }
}

fn reference_runtime() -> PluginRuntime {
    first_party_runtime().unwrap()
}

fn rect_clip(layer: u64, center: [f64; 2], color: [f64; 4]) -> Clip {
    Clip {
        envelope: ItemEnvelope::new(motolii_doc::LayerId::from_raw(layer)),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(10, 1).unwrap(),
        time_map: TimeMap::identity(),
        source: ClipSource::Plugin {
            plugin_id: RECT_LAYER_SOURCE.into(),
            effect_version: 1,
            params: BTreeMap::from([
                ("center".into(), DocParam::const_vec2(center)),
                ("size".into(), DocParam::const_vec2([0.22, 0.9])),
                ("color".into(), DocParam::const_color(color)),
            ]),
            extra: Default::default(),
        },
    }
}

fn set_rect_size(clip: &mut Clip, size: [f64; 2]) {
    let ClipSource::Plugin { params, .. } = &mut clip.source else {
        panic!("expected rect plugin source");
    };
    params.insert("size".into(), DocParam::const_vec2(size));
}

fn add_definition(
    doc: &mut Document,
    plugin_id: &str,
    params: BTreeMap<String, DocParam>,
) -> EffectDefinitionId {
    let definition_id = EffectDefinitionId::from_raw(doc.next_stable_id.allocate().unwrap());
    doc.effect_definitions.push(EffectDefinition::new(
        definition_id,
        plugin_id,
        1,
        true,
        params,
        Default::default(),
    ));
    doc.version = doc
        .version
        .max(motolii_doc::MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS);
    doc.min_reader_version = doc
        .min_reader_version
        .max(motolii_doc::MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS);
    definition_id
}

fn link_use(doc: &mut Document, env: &mut ItemEnvelope, definition_id: EffectDefinitionId) {
    let use_id = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
    env.effects.push(EffectUse {
        id: use_id,
        definition_id,
    });
}

fn tint_params(color: [f64; 4]) -> BTreeMap<String, DocParam> {
    BTreeMap::from([("color".into(), DocParam::const_color(color))])
}

fn opacity_params(amount: f64) -> BTreeMap<String, DocParam> {
    BTreeMap::from([("amount".into(), DocParam::const_f64(amount))])
}

fn clear_params(color: [f64; 4]) -> BTreeMap<String, DocParam> {
    BTreeMap::from([("color".into(), DocParam::const_color(color))])
}

fn render_doc(doc: &mut Document) -> Option<Vec<u8>> {
    doc.composition = Composition::try_new(
        i64::from(W),
        i64::from(H),
        doc.composition.duration,
        doc.composition.fps,
    )
    .unwrap();
    let frame_desc = desc();
    let gpu = gpu_or_skip()?;
    let runtime = reference_runtime();
    let built = build_document_frame_graph(
        doc,
        EvaluationTime::new(RationalTime::ZERO),
        frame_desc,
        &DataTracks::new(),
        &runtime,
        None,
    )
    .unwrap();
    let mut session = RenderSession::new(&gpu);
    let rendered = render_graph_cached(
        &gpu,
        &mut session,
        RationalTime::ZERO,
        &built.graph,
        &RenderGraphInputs {
            camera: built.camera,
            video_sources: &[],
            source_time: Some(built.source_time),
            plugins: Some(runtime.executors()),
        },
        Quality::FINAL,
    )
    .unwrap();
    Some(download_rgba(&gpu, &rendered.texture).unwrap())
}

fn sample_pixel_at_center(pixels: &[u8], center: [f64; 2]) -> [u8; 4] {
    let x = (W as f64 * 0.5 + center[0] * H as f64).round() as u32;
    let y = (H as f64 * 0.5 - center[1] * H as f64).round() as u32;
    let x = x.min(W - 1);
    let y = y.min(H - 1);
    let idx = ((y * W + x) * 4) as usize;
    pixels[idx..idx + 4].try_into().expect("center sample")
}

fn assert_non_adjacent_shared_use_items(doc: &Document, shared_def: EffectDefinitionId) {
    let indices: Vec<usize> = doc.tracks[0]
        .items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            let TrackItem::Clip(clip) = item else {
                return None;
            };
            clip.envelope
                .effects
                .iter()
                .any(|effect| effect.definition_id == shared_def)
                .then_some(index)
        })
        .collect();
    assert_eq!(
        indices.len(),
        3,
        "fixture must have exactly three shared-use clips"
    );
    for window in indices.windows(2) {
        assert!(
            window[1] - window[0] > 1,
            "shared-use track items must be non-adjacent: {indices:?}"
        );
    }
}

fn append_three_layer_stack(doc: &mut Document, definition_id: EffectDefinitionId) {
    let l0 = doc.layers.allocate("left").unwrap();
    let l_spacer = doc.layers.allocate("spacer_a").unwrap();
    let l1 = doc.layers.allocate("mid").unwrap();
    let l_spacer_b = doc.layers.allocate("spacer_b").unwrap();
    let l2 = doc.layers.allocate("right").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let opacity_def = add_definition(doc, "core.filter.opacity", opacity_params(0.5));
    let spacer_def = add_definition(doc, "core.filter.opacity", opacity_params(0.25));

    let mut c0 = rect_clip(l0.get(), [0.0, 0.0], [1.0, 0.0, 0.0, 1.0]);
    set_rect_size(&mut c0, [2.8, 1.0]);
    c0.envelope.layer_id = l0;
    c0.envelope.transform.position = DocParam::const_vec2([-0.62, 0.0]);
    c0.envelope.transform.scale = DocParam::const_vec2([0.18, 0.75]);
    link_use(doc, &mut c0.envelope, definition_id);

    let mut c_spacer = rect_clip(l_spacer.get(), [-0.2, 0.0], [0.5, 0.5, 0.5, 1.0]);
    c_spacer.envelope.layer_id = l_spacer;
    link_use(doc, &mut c_spacer.envelope, spacer_def);

    let mut c1 = rect_clip(l1.get(), [0.0, 0.0], [1.0, 1.0, 1.0, 1.0]);
    set_rect_size(&mut c1, [2.8, 1.0]);
    c1.envelope.layer_id = l1;
    c1.envelope.transform.scale = DocParam::const_vec2([0.18, 0.75]);
    link_use(doc, &mut c1.envelope, opacity_def);
    link_use(doc, &mut c1.envelope, definition_id);

    let mut c_spacer_b = rect_clip(l_spacer_b.get(), [0.2, 0.0], [0.5, 0.5, 0.5, 1.0]);
    c_spacer_b.envelope.layer_id = l_spacer_b;
    link_use(doc, &mut c_spacer_b.envelope, spacer_def);

    let mut c2 = rect_clip(l2.get(), [0.0, 0.0], [0.0, 1.0, 0.0, 1.0]);
    set_rect_size(&mut c2, [2.8, 1.0]);
    c2.envelope.layer_id = l2;
    c2.envelope.transform.position = DocParam::const_vec2([0.62, 0.0]);
    c2.envelope.transform.scale = DocParam::const_vec2([0.18, 0.75]);
    link_use(doc, &mut c2.envelope, definition_id);
    link_use(doc, &mut c2.envelope, opacity_def);

    doc.tracks.push(Track {
        id: track_id,
        items: vec![
            TrackItem::Clip(c0),
            TrackItem::Clip(c_spacer),
            TrackItem::Clip(c1),
            TrackItem::Clip(c_spacer_b),
            TrackItem::Clip(c2),
        ],
    });
}

fn build_inline_stack_positions() -> Document {
    let mut doc = Document::new_current();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let l0 = doc.layers.allocate("left").unwrap();
    let l_spacer = doc.layers.allocate("spacer_a").unwrap();
    let l1 = doc.layers.allocate("mid").unwrap();
    let l_spacer_b = doc.layers.allocate("spacer_b").unwrap();
    let l2 = doc.layers.allocate("right").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let shared_clear = clear_params([0.0, 1.0, 0.0, 1.0]);
    let spacer_opacity = opacity_params(0.25);

    let mut c0 = rect_clip(l0.get(), [0.0, 0.0], [1.0, 0.0, 0.0, 1.0]);
    set_rect_size(&mut c0, [2.8, 1.0]);
    c0.envelope.layer_id = l0;
    c0.envelope.transform.position = DocParam::const_vec2([-0.62, 0.0]);
    c0.envelope.transform.scale = DocParam::const_vec2([0.18, 0.75]);
    let d0 = add_definition(&mut doc, "core.filter.clear", shared_clear.clone());
    link_use(&mut doc, &mut c0.envelope, d0);

    let mut c_spacer = rect_clip(l_spacer.get(), [-0.2, 0.0], [0.5, 0.5, 0.5, 1.0]);
    c_spacer.envelope.layer_id = l_spacer;
    let s0 = add_definition(&mut doc, "core.filter.opacity", spacer_opacity.clone());
    link_use(&mut doc, &mut c_spacer.envelope, s0);

    let mut c1 = rect_clip(l1.get(), [0.0, 0.0], [1.0, 1.0, 1.0, 1.0]);
    set_rect_size(&mut c1, [2.8, 1.0]);
    c1.envelope.layer_id = l1;
    c1.envelope.transform.scale = DocParam::const_vec2([0.18, 0.75]);
    let o1 = add_definition(&mut doc, "core.filter.opacity", opacity_params(0.5));
    let d1 = add_definition(&mut doc, "core.filter.clear", shared_clear.clone());
    link_use(&mut doc, &mut c1.envelope, o1);
    link_use(&mut doc, &mut c1.envelope, d1);

    let mut c_spacer_b = rect_clip(l_spacer_b.get(), [0.2, 0.0], [0.5, 0.5, 0.5, 1.0]);
    c_spacer_b.envelope.layer_id = l_spacer_b;
    let s1 = add_definition(&mut doc, "core.filter.opacity", spacer_opacity);
    link_use(&mut doc, &mut c_spacer_b.envelope, s1);

    let mut c2 = rect_clip(l2.get(), [0.0, 0.0], [0.0, 1.0, 0.0, 1.0]);
    set_rect_size(&mut c2, [2.8, 1.0]);
    c2.envelope.layer_id = l2;
    c2.envelope.transform.position = DocParam::const_vec2([0.62, 0.0]);
    c2.envelope.transform.scale = DocParam::const_vec2([0.18, 0.75]);
    let d2 = add_definition(&mut doc, "core.filter.clear", shared_clear.clone());
    let o2 = add_definition(&mut doc, "core.filter.opacity", opacity_params(0.5));
    link_use(&mut doc, &mut c2.envelope, d2);
    link_use(&mut doc, &mut c2.envelope, o2);

    doc.tracks.push(Track {
        id: track_id,
        items: vec![
            TrackItem::Clip(c0),
            TrackItem::Clip(c_spacer),
            TrackItem::Clip(c1),
            TrackItem::Clip(c_spacer_b),
            TrackItem::Clip(c2),
        ],
    });
    doc.validate().unwrap();
    doc
}

#[test]
fn p1_non_adjacent_shared_definition_matches_inline_copies() {
    let Some(_) = gpu_or_skip() else { return };
    let mut shared_doc = Document::new_current();
    shared_doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let shared_def = add_definition(
        &mut shared_doc,
        "core.filter.clear",
        clear_params([0.0, 1.0, 0.0, 1.0]),
    );
    append_three_layer_stack(&mut shared_doc, shared_def);
    shared_doc.validate().unwrap();
    assert_non_adjacent_shared_use_items(&shared_doc, shared_def);
    let mut inline = build_inline_stack_positions();

    let shared_pixels = render_doc(&mut shared_doc).expect("gpu");
    let inline_pixels = render_doc(&mut inline).expect("gpu");
    assert_eq!(
        sample_pixel_at_center(&shared_pixels, [-1.4, 0.45])[3],
        0,
        "per-Use evaluation must leave space outside transformed layers transparent"
    );
    assert_rgba_close(
        "d3e-p1-shared-vs-inline",
        image_desc(),
        &shared_pixels,
        &inline_pixels,
        tol::GPU_RASTER,
    );
}

#[test]
fn p2_definition_param_change_updates_all_uses() {
    let Some(_) = gpu_or_skip() else { return };
    let mut built = Document::new_current();
    built.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let shared_def = add_definition(
        &mut built,
        "core.filter.tint",
        tint_params([0.0, 1.0, 0.0, 1.0]),
    );
    append_three_layer_stack(&mut built, shared_def);
    built.validate().unwrap();
    let before = render_doc(&mut built).expect("gpu");

    let definition = built.effect_definition_mut(shared_def).unwrap();
    definition.params = tint_params([1.0, 0.0, 0.0, 1.0]);
    let after = render_doc(&mut built).expect("gpu");

    assert_ne!(
        sample_pixel_at_center(&before, [-0.62, 0.0]),
        sample_pixel_at_center(&after, [-0.62, 0.0]),
        "left use must follow definition"
    );
    assert_ne!(
        sample_pixel_at_center(&before, [0.0, 0.0]),
        sample_pixel_at_center(&after, [0.0, 0.0]),
        "mid use must follow definition"
    );
    assert_ne!(
        sample_pixel_at_center(&before, [0.62, 0.0]),
        sample_pixel_at_center(&after, [0.62, 0.0]),
        "right use must follow definition"
    );
}

#[test]
fn p3_use_reorder_affects_only_target_layer_region() {
    let Some(_) = gpu_or_skip() else { return };
    let mut doc = Document::new_current();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let clear_red = add_definition(
        &mut doc,
        "core.filter.clear",
        clear_params([1.0, 0.0, 0.0, 1.0]),
    );
    let green_def = add_definition(
        &mut doc,
        "core.filter.tint",
        tint_params([0.0, 1.0, 0.0, 1.0]),
    );

    let left = doc.layers.allocate("left").unwrap();
    let right = doc.layers.allocate("right").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();

    let mut left_clip = rect_clip(left.get(), [0.0, 0.0], [1.0, 1.0, 1.0, 1.0]);
    left_clip.envelope.layer_id = left;
    left_clip.envelope.transform.position = DocParam::const_vec2([-0.55, 0.0]);
    left_clip.envelope.transform.scale = DocParam::const_vec2([0.25, 0.8]);
    link_use(&mut doc, &mut left_clip.envelope, clear_red);
    link_use(&mut doc, &mut left_clip.envelope, green_def);

    let mut right_clip = rect_clip(right.get(), [0.0, 0.0], [1.0, 0.0, 0.0, 1.0]);
    right_clip.envelope.layer_id = right;
    right_clip.envelope.transform.position = DocParam::const_vec2([0.55, 0.0]);
    right_clip.envelope.transform.scale = DocParam::const_vec2([0.25, 0.8]);
    link_use(&mut doc, &mut right_clip.envelope, clear_red);
    link_use(&mut doc, &mut right_clip.envelope, green_def);

    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(left_clip), TrackItem::Clip(right_clip)],
    });
    doc.validate().unwrap();

    let before = render_doc(&mut doc).expect("gpu");

    match &mut doc.tracks[0].items[0] {
        TrackItem::Clip(clip) => clip.envelope.effects.reverse(),
        _ => panic!("expected clip"),
    };
    let after = render_doc(&mut doc).expect("gpu");

    assert_ne!(
        sample_pixel_at_center(&before, [-0.55, 0.0]),
        sample_pixel_at_center(&after, [-0.55, 0.0]),
        "reordered layer must change"
    );
    assert_eq!(
        sample_pixel_at_center(&before, [0.55, 0.0]),
        sample_pixel_at_center(&after, [0.55, 0.0]),
        "other layer center unchanged"
    );
}

#[test]
fn p4_group_effect_applies_once_after_child_composite() {
    let Some(_) = gpu_or_skip() else { return };
    let mut group_doc = Document::new_current();
    group_doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let shared_def = add_definition(&mut group_doc, "core.filter.opacity", opacity_params(0.5));

    let child_a = group_doc.layers.allocate("child_a").unwrap();
    let child_b = group_doc.layers.allocate("child_b").unwrap();
    let group_layer = group_doc.layers.allocate("group").unwrap();
    let track_id = group_doc.track_ids.allocate("V1").unwrap();

    let mut a = rect_clip(child_a.get(), [0.0, 0.0], [1.0, 0.0, 0.0, 1.0]);
    a.envelope.layer_id = child_a;
    let mut b = rect_clip(child_b.get(), [0.0, 0.0], [0.0, 0.0, 1.0, 1.0]);
    b.envelope.layer_id = child_b;

    let mut group_env = ItemEnvelope::new(group_layer);
    link_use(&mut group_doc, &mut group_env, shared_def);
    let group = Group {
        envelope: group_env,
        children: vec![TrackItem::Clip(a), TrackItem::Clip(b)],
    };

    let external_layer = group_doc.layers.allocate("external").unwrap();
    let mut external = rect_clip(external_layer.get(), [0.62, 0.0], [0.0, 1.0, 0.0, 1.0]);
    external.envelope.layer_id = external_layer;
    link_use(&mut group_doc, &mut external.envelope, shared_def);

    group_doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Group(group), TrackItem::Clip(external)],
    });
    group_doc.validate().unwrap();
    let group_once = render_doc(&mut group_doc).expect("gpu");
    let external_pixel = sample_pixel_at_center(&group_once, [0.62, 0.0]);
    assert_eq!(external_pixel[0], 0);
    assert_eq!(external_pixel[2], 0);
    assert!(
        external_pixel[1] > 0 && external_pixel[1] < 255,
        "external shared Use must apply opacity"
    );
    assert_eq!(
        external_pixel[1], external_pixel[3],
        "external green must remain premultiplied after opacity"
    );

    let mut per_child_doc = Document::new_current();
    per_child_doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let child_def = add_definition(
        &mut per_child_doc,
        "core.filter.opacity",
        opacity_params(0.5),
    );
    let ca = per_child_doc.layers.allocate("child_a").unwrap();
    let cb = per_child_doc.layers.allocate("child_b").unwrap();
    let track = per_child_doc.track_ids.allocate("V1").unwrap();
    let mut ca_clip = rect_clip(ca.get(), [0.0, 0.0], [1.0, 0.0, 0.0, 1.0]);
    ca_clip.envelope.layer_id = ca;
    link_use(&mut per_child_doc, &mut ca_clip.envelope, child_def);
    let mut cb_clip = rect_clip(cb.get(), [0.0, 0.0], [0.0, 0.0, 1.0, 1.0]);
    cb_clip.envelope.layer_id = cb;
    link_use(&mut per_child_doc, &mut cb_clip.envelope, child_def);
    per_child_doc.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(ca_clip), TrackItem::Clip(cb_clip)],
    });
    per_child_doc.validate().unwrap();
    let per_child = render_doc(&mut per_child_doc).expect("gpu");

    assert_ne!(
        sample_pixel_at_center(&group_once, [0.0, 0.0]),
        sample_pixel_at_center(&per_child, [0.0, 0.0]),
        "group stack must apply once after composite, not per child"
    );
}

#[test]
fn p5_timeline_order_and_layer_rename_leave_pixels_unchanged() {
    let Some(_) = gpu_or_skip() else { return };
    let mut doc = Document::new_current();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let shared_def = add_definition(
        &mut doc,
        "core.filter.tint",
        tint_params([0.0, 1.0, 0.0, 1.0]),
    );
    let distinct_def = add_definition(
        &mut doc,
        "core.filter.tint",
        tint_params([1.0, 0.0, 0.0, 1.0]),
    );

    let left = doc.layers.allocate("left").unwrap();
    let middle = doc.layers.allocate("middle").unwrap();
    let right = doc.layers.allocate("right").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();

    let mut left_clip = rect_clip(left.get(), [-0.65, 0.0], [1.0, 1.0, 1.0, 1.0]);
    left_clip.envelope.layer_id = left;
    link_use(&mut doc, &mut left_clip.envelope, shared_def);
    let mut middle_clip = rect_clip(middle.get(), [0.0, 0.0], [1.0, 1.0, 1.0, 1.0]);
    middle_clip.envelope.layer_id = middle;
    link_use(&mut doc, &mut middle_clip.envelope, distinct_def);
    let mut right_clip = rect_clip(right.get(), [0.65, 0.0], [1.0, 1.0, 1.0, 1.0]);
    right_clip.envelope.layer_id = right;
    link_use(&mut doc, &mut right_clip.envelope, shared_def);

    doc.tracks.push(Track {
        id: track_id,
        items: vec![
            TrackItem::Clip(left_clip),
            TrackItem::Clip(middle_clip),
            TrackItem::Clip(right_clip),
        ],
    });
    doc.validate().unwrap();
    let baseline = render_doc(&mut doc).expect("gpu");
    assert_eq!(
        sample_pixel_at_center(&baseline, [-0.65, 0.0]),
        [0, 255, 0, 255]
    );
    assert_eq!(
        sample_pixel_at_center(&baseline, [0.0, 0.0]),
        [255, 0, 0, 255]
    );
    assert_eq!(
        sample_pixel_at_center(&baseline, [0.65, 0.0]),
        [0, 255, 0, 255]
    );

    doc.tracks[0].items.rotate_left(1);
    let reordered = render_doc(&mut doc).expect("gpu");
    assert_rgba_close(
        "d3e-p5-timeline-order",
        image_desc(),
        &reordered,
        &baseline,
        tol::GPU_RASTER,
    );

    doc.layers.remove(left).unwrap();
    doc.layers.restore(left, "renamed-left").unwrap();
    assert_eq!(doc.layers.display_name(left), Some("renamed-left"));
    let renamed = render_doc(&mut doc).expect("gpu");
    assert_rgba_close(
        "d3e-p5-layer-rename",
        image_desc(),
        &renamed,
        &baseline,
        tol::GPU_RASTER,
    );
}

static P6_TINT_DESC: OnceLock<NodeDesc> = OnceLock::new();
static TEST_TINT_RENAME: TestTintRename = TestTintRename;

const TEST_TINT_RENAME_ID: &str = "test.filter.tint_rename";

struct TestTintRename;

impl FilterPlugin for TestTintRename {
    fn desc(&self) -> &NodeDesc {
        p6_tint_desc()
    }

    fn render(
        &self,
        _gpu: &GpuCtx,
        _pipelines: &mut PipelineCache,
        _encoder: &mut motolii_plugin::wgpu::CommandEncoder,
        _ctx: &RenderCtx,
        _params: &ResolvedParams,
        _input: TextureRef<'_>,
        _output: TextureRef<'_>,
    ) -> Result<(), PluginError> {
        Ok(())
    }
}

fn p6_tint_desc() -> &'static NodeDesc {
    P6_TINT_DESC.get_or_init(|| NodeDesc {
        id: PluginId(TEST_TINT_RENAME_ID),
        version: 2,
        display_name: "Tint rename test",
        category: "Test",
        tags: &["test"],
        params: vec![ParamDef {
            id: "color",
            value_type: ValueType::Color,
            default: Value::Color([1.0, 1.0, 1.0, 1.0]),
            f64_domain: None,
        }],
        min_inputs: 1,
        max_inputs: 1,
    })
}

fn p6_rename_catalog() -> PluginCatalog {
    let mut builder = PluginCatalogBuilder::new();
    builder
        .register(PluginContract {
            kind: PluginKind::Filter,
            node: p6_tint_desc().clone(),
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

fn p6_runtime() -> PluginRuntime {
    let mut registry = PluginRegistry::new();
    registry.register_filter(&TEST_TINT_RENAME).unwrap();
    PluginRuntime::try_new(Arc::new(p6_rename_catalog()), registry).unwrap()
}

fn p6_rename_document() -> Document {
    let mut doc = Document::new_current();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let layer = doc.layers.allocate("subject").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    let definition_id = add_definition(
        &mut doc,
        TEST_TINT_RENAME_ID,
        BTreeMap::from([(
            "old_color".into(),
            DocParam::const_color([0.0, 0.75, 0.25, 1.0]),
        )]),
    );
    let mut clip = rect_clip(layer.get(), [0.0, 0.0], [1.0, 0.0, 0.0, 1.0]);
    clip.envelope.layer_id = layer;
    link_use(&mut doc, &mut clip.envelope, definition_id);
    doc.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(clip)],
    });
    doc.validate().unwrap();
    doc
}

#[test]
fn p6_prepared_rename_executes_new_key_while_raw_unchanged() {
    let Some(_) = gpu_or_skip() else { return };
    let runtime = p6_runtime();
    let doc = p6_rename_document();
    let doc_before = doc.clone();
    let raw_before = doc.effect_definitions[0].params.clone();

    let built = build_document_frame_graph(
        &doc,
        EvaluationTime::new(RationalTime::ZERO),
        desc(),
        &DataTracks::new(),
        &runtime,
        None,
    )
    .unwrap();

    assert_eq!(doc, doc_before);
    assert_eq!(doc.effect_definitions[0].params, raw_before);
    assert!(raw_before.contains_key("old_color"));
    assert!(!raw_before.contains_key("color"));

    let (plugin_id, plugin_params) = built
        .graph
        .steps
        .iter()
        .find_map(|step| match step {
            RenderStep::Plugin { id, params, .. } => Some((id, params)),
            _ => None,
        })
        .expect("effect should lower to RenderStep::Plugin");
    assert_eq!(plugin_id.0, TEST_TINT_RENAME_ID);
    assert_eq!(
        plugin_params.get("color"),
        Some(&Value::Color([0.0, 0.75, 0.25, 1.0]))
    );
    assert!(plugin_params.get("old_color").is_none());

    let prepared = doc.prepare_plugins(runtime.catalog()).unwrap();
    let recipe = prepared
        .get(&PluginSlotId::EffectDefinition(
            doc.effect_definitions[0].id,
        ))
        .expect("prepared effect recipe");
    assert!(!recipe.params.contains_key("old_color"));
    assert_eq!(
        recipe.params.get("color"),
        Some(&DocParam::const_color([0.0, 0.75, 0.25, 1.0]))
    );
}

#[test]
fn n1_dangling_definition_is_typed_error() {
    let mut doc = Document::new_current();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let layer = doc.layers.allocate("clip").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    let dangling = EffectDefinitionId::from_raw(doc.next_stable_id.allocate().unwrap());
    let mut clip = rect_clip(layer.get(), [0.0, 0.0], [1.0, 0.0, 0.0, 1.0]);
    clip.envelope.layer_id = layer;
    clip.envelope.effects.push(EffectUse {
        id: EffectId::from_raw(doc.next_stable_id.allocate().unwrap()),
        definition_id: dangling,
    });
    doc.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(clip)],
    });

    assert!(matches!(
        doc.validate().unwrap_err(),
        DocumentError::DanglingEffectDefinition { .. }
    ));

    let runtime = reference_runtime();
    let err = build_document_frame_graph(
        &doc,
        EvaluationTime::new(RationalTime::ZERO),
        desc(),
        &DataTracks::new(),
        &runtime,
        None,
    )
    .unwrap_err();
    assert!(matches!(
        err,
        GraphError::PluginDocument(DocumentPluginError::Structural(
            DocumentError::DanglingEffectDefinition { .. }
        ))
    ));
}

#[test]
fn n3_degraded_prepared_recipe_blocks_graph_execution() {
    let mut doc = Document::new_current();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let layer = doc.layers.allocate("clip").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    let definition_id = add_definition(&mut doc, "vendor.filter.absent", opacity_params(0.5));
    let mut clip = rect_clip(layer.get(), [0.0, 0.0], [1.0, 0.0, 0.0, 1.0]);
    clip.envelope.layer_id = layer;
    link_use(&mut doc, &mut clip.envelope, definition_id);
    doc.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(clip)],
    });
    doc.validate().unwrap();

    let runtime = reference_runtime();
    let prepared = doc.prepare_plugins(runtime.catalog()).unwrap();
    assert!(!prepared.is_fully_prepared());
    assert_eq!(
        prepared.diagnostics()[0].reason,
        PluginDiagnosticReason::ContractMissing
    );
    assert!(prepared
        .get(&PluginSlotId::EffectDefinition(definition_id))
        .is_none());

    let err = build_document_frame_graph(
        &doc,
        EvaluationTime::new(RationalTime::ZERO),
        desc(),
        &DataTracks::new(),
        &runtime,
        None,
    )
    .unwrap_err();
    let GraphError::PluginDiagnostics(diagnostics) = err else {
        panic!("expected plugin diagnostics");
    };
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].slot,
        PluginSlotId::EffectDefinition(definition_id)
    );
    assert_eq!(diagnostics[0].plugin_id, "vendor.filter.absent");
    assert_eq!(
        diagnostics[0].reason,
        PluginDiagnosticReason::ContractMissing
    );
}
