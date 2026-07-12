//! D3: F-3評価順ゴールデン。

use std::collections::BTreeMap;
use motolii_core::{ColorSpace, FrameDesc, PixelFormat, Quality, RationalTime};
use motolii_doc::{
    build_document_frame_graph, Clip, ClipSource, ClippingMaskSettings, DocParam, Document,
    EffectInstance, EvaluationTime, Group, ItemEnvelope, MaskMode, RECT_LAYER_SOURCE, Track, TrackItem,
};
use motolii_eval::DataTracks;
use motolii_gpu::download_rgba;
use motolii_plugin::reference::register_reference_plugins;
use motolii_plugin::PluginRegistry;
use motolii_render::{render_graph_cached, RenderGraphInputs, RenderSession};
use motolii_testkit::cpu_reference::{expected_rect_frame, premul_over_u8};
use motolii_testkit::{assert_rgba_close, gpu_or_skip, tol, RgbaImageDesc};

const W: u32 = 16;
const H: u32 = 8;
fn desc() -> FrameDesc { FrameDesc::packed(W, H, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true) }

fn rect_clip(layer: u64, center: [f64; 2], size: [f64; 2], color: [f64; 4]) -> Clip {
    Clip {
        envelope: ItemEnvelope::new(motolii_doc::LayerId::from_raw(layer)),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(10, 1).unwrap(),
        time_map: motolii_core::TimeMap::identity(),
        source: ClipSource::Plugin {
            plugin_id: RECT_LAYER_SOURCE.into(), effect_version: 1,
            params: BTreeMap::from([
                ("center".into(), DocParam::const_vec2(center)),
                ("size".into(), DocParam::const_vec2(size)),
                ("color".into(), DocParam::const_color(color)),
            ]),
            extra: Default::default(),
        },
        path_ops: Vec::new(),
    }
}

fn render_doc(doc: &Document) -> Option<Vec<u8>> {
    let Some(gpu) = gpu_or_skip() else { return None };
    let mut registry = PluginRegistry::new();
    register_reference_plugins(&mut registry).unwrap();
    let built = build_document_frame_graph(doc, EvaluationTime::new(RationalTime::ZERO), desc(), &DataTracks::new(), &registry, None).unwrap();
    let mut session = RenderSession::new(&gpu);
    let rendered = render_graph_cached(&gpu, &mut session, RationalTime::ZERO, &built.graph,
        &RenderGraphInputs { video_sources: &[], source_time: Some(built.source_time), plugins: Some(&registry) }, Quality::FINAL).unwrap();
    Some(download_rgba(&gpu, &rendered.texture).unwrap())
}

#[test]
fn masked_group_effect_applies_before_clipping_mask() {
    let Some(_) = gpu_or_skip() else { return };
    let mut doc = Document::new_v1();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let mask_layer = doc.layers.allocate("mask").unwrap();
    let content_layer = doc.layers.allocate("content").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let mut mask_clip = rect_clip(mask_layer.get(), [-0.25, 0.0], [0.5, 1.0], [1.0, 1.0, 1.0, 1.0]);
    mask_clip.envelope.layer_id = mask_layer;
    let mut content_clip = rect_clip(content_layer.get(), [0.0, 0.0], [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]);
    content_clip.envelope.layer_id = content_layer;
    content_clip.envelope.effects.push(EffectInstance {
        plugin_id: "core.filter.tint".into(), effect_version: 1, enabled: true,
        params: BTreeMap::from([("color".into(), DocParam::const_color([0.0, 1.0, 0.0, 1.0]))]), extra: Default::default(),
    });
    content_clip.envelope.clipping_mask = ClippingMaskSettings { enabled: true, mode: MaskMode::Luminance };
    doc.tracks.push(Track { id: track_id, items: vec![TrackItem::Clip(mask_clip), TrackItem::Clip(content_clip)] });
    let actual = render_doc(&doc).expect("gpu");
    let expected = expected_rect_frame(desc(), [0,0,0,0], [0,255,0,255], [-0.25, 0.0], [0.5, 1.0]);
    assert_rgba_close("d3-mask-after-effect", RgbaImageDesc { width: W, height: H }, &actual, &expected, tol::GPU_RASTER);
    assert_eq!(actual[((H/2)*W + W-2) as usize * 4], 0);
}

#[test]
fn group_effect_stack_applies_after_children_composite() {
    let Some(_) = gpu_or_skip() else { return };
    let mut doc = Document::new_v1();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let bg_layer = doc.layers.allocate("bg").unwrap();
    let child_layer = doc.layers.allocate("child").unwrap();
    let group_layer = doc.layers.allocate("group").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let mut bg = rect_clip(bg_layer.get(), [0.0, 0.0], [1.0, 1.0], [0.0, 0.0, 1.0, 1.0]);
    bg.envelope.layer_id = bg_layer;
    let mut child = rect_clip(child_layer.get(), [0.0, 0.0], [1.0, 1.0], [1.0, 0.0, 0.0, 1.0]);
    child.envelope.layer_id = child_layer;
    let group = Group {
        envelope: {
            let mut env = ItemEnvelope::new(group_layer);
            env.effects.push(EffectInstance {
                plugin_id: "core.filter.opacity".into(), effect_version: 1, enabled: true,
                params: BTreeMap::from([("amount".into(), DocParam::const_f64(0.5))]), extra: Default::default(),
            });
            env
        },
        children: vec![TrackItem::Clip(child)],
    };
    doc.tracks.push(Track { id: track_id, items: vec![TrackItem::Clip(bg), TrackItem::Group(group)] });
    let actual = render_doc(&doc).expect("gpu");
    let transparent = [0u8, 0, 0, 0];
    let blue = [0u8, 0, 255, 255];
    let red_premul = [127u8, 0, 0, 127];
    let blue_layer = expected_rect_frame(desc(), transparent, blue, [0.0, 0.0], [1.0, 1.0]);
    let red_layer = expected_rect_frame(desc(), transparent, red_premul, [0.0, 0.0], [1.0, 1.0]);
    let expected: Vec<u8> = blue_layer
        .chunks_exact(4)
        .zip(red_layer.chunks_exact(4))
        .flat_map(|(b, f)| premul_over_u8(b.try_into().unwrap(), f.try_into().unwrap()))
        .collect();
    assert_rgba_close("d3-group-opacity", RgbaImageDesc { width: W, height: H }, &actual, &expected, tol::GPU_RASTER);
}
