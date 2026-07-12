//! D3: F-3評価順ゴールデン。

use motolii_core::{ColorSpace, FrameDesc, PixelFormat, Quality, RationalTime};
use motolii_doc::{
    build_document_frame_graph, Clip, ClipSource, ClippingMaskSettings, DocParam, Document,
    EffectInstance, EvaluationTime, Group, ItemEnvelope, MaskMode, Track, TrackItem,
    RECT_LAYER_SOURCE,
};
use motolii_eval::DataTracks;
use motolii_gpu::download_rgba;
use motolii_plugin::reference::register_reference_plugins;
use motolii_plugin::PluginRegistry;
use motolii_render::{render_graph_cached, RenderGraphInputs, RenderSession};
use motolii_testkit::cpu_reference::{expected_rect_frame, premul_over_u8};
use motolii_testkit::{assert_rgba_close, gpu_or_skip, tol, RgbaImageDesc};
use std::collections::BTreeMap;

const W: u32 = 16;
const H: u32 = 8;
fn desc() -> FrameDesc {
    FrameDesc::packed(W, H, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true)
}

fn rect_clip(layer: u64, center: [f64; 2], size: [f64; 2], color: [f64; 4]) -> Clip {
    Clip {
        envelope: ItemEnvelope::new(motolii_doc::LayerId::from_raw(layer)),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(10, 1).unwrap(),
        time_map: motolii_core::TimeMap::identity(),
        source: ClipSource::Plugin {
            plugin_id: RECT_LAYER_SOURCE.into(),
            effect_version: 1,
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
    let gpu = gpu_or_skip()?;
    let mut registry = PluginRegistry::new();
    register_reference_plugins(&mut registry).unwrap();
    let built = build_document_frame_graph(
        doc,
        EvaluationTime::new(RationalTime::ZERO),
        desc(),
        &DataTracks::new(),
        &registry,
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
            video_sources: &[],
            source_time: Some(built.source_time),
            plugins: Some(&registry),
        },
        Quality::FINAL,
    )
    .unwrap();
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
    let mut mask_clip = rect_clip(
        mask_layer.get(),
        [-0.25, 0.0],
        [0.5, 1.0],
        [1.0, 1.0, 1.0, 1.0],
    );
    mask_clip.envelope.layer_id = mask_layer;
    let mut content_clip = rect_clip(
        content_layer.get(),
        [0.0, 0.0],
        [1.0, 1.0],
        [1.0, 1.0, 1.0, 1.0],
    );
    content_clip.envelope.layer_id = content_layer;
    content_clip.envelope.effects.push(EffectInstance {
        plugin_id: "core.filter.tint".into(),
        effect_version: 1,
        enabled: true,
        params: BTreeMap::from([("color".into(), DocParam::const_color([0.0, 1.0, 0.0, 1.0]))]),
        extra: Default::default(),
    });
    content_clip.envelope.clipping_mask = ClippingMaskSettings {
        enabled: true,
        mode: MaskMode::Luminance,
    };
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(mask_clip), TrackItem::Clip(content_clip)],
    });
    let actual = render_doc(&doc).expect("gpu");
    let expected = expected_rect_frame(
        desc(),
        [0, 0, 0, 0],
        [0, 255, 0, 255],
        [-0.25, 0.0],
        [0.5, 1.0],
    );
    assert_rgba_close(
        "d3-mask-after-effect",
        RgbaImageDesc {
            width: W,
            height: H,
        },
        &actual,
        &expected,
        tol::GPU_RASTER,
    );
    assert_eq!(actual[((H / 2) * W + W - 2) as usize * 4], 0);
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
    let mut child = rect_clip(
        child_layer.get(),
        [0.0, 0.0],
        [1.0, 1.0],
        [1.0, 0.0, 0.0, 1.0],
    );
    child.envelope.layer_id = child_layer;
    let group = Group {
        envelope: {
            let mut env = ItemEnvelope::new(group_layer);
            env.effects.push(EffectInstance {
                plugin_id: "core.filter.opacity".into(),
                effect_version: 1,
                enabled: true,
                params: BTreeMap::from([("amount".into(), DocParam::const_f64(0.5))]),
                extra: Default::default(),
            });
            env
        },
        children: vec![TrackItem::Clip(child)],
    };
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(bg), TrackItem::Group(group)],
    });
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
    assert_rgba_close(
        "d3-group-opacity",
        RgbaImageDesc {
            width: W,
            height: H,
        },
        &actual,
        &expected,
        tol::GPU_RASTER,
    );
}

#[test]
fn bottom_layer_envelope_opacity_is_applied() {
    let Some(_) = gpu_or_skip() else { return };
    let mut doc = Document::new_v1();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let layer = doc.layers.allocate("solo").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let mut clip = rect_clip(layer.get(), [0.0, 0.0], [1.0, 1.0], [1.0, 0.0, 0.0, 1.0]);
    clip.envelope.layer_id = layer;
    clip.envelope.opacity = DocParam::const_f64(0.5);
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(clip)],
    });
    let actual = render_doc(&doc).expect("gpu");
    let expected = expected_rect_frame(
        desc(),
        [0, 0, 0, 0],
        [127, 0, 0, 127],
        [0.0, 0.0],
        [1.0, 1.0],
    );
    assert_rgba_close(
        "d3-bottom-opacity",
        RgbaImageDesc {
            width: W,
            height: H,
        },
        &actual,
        &expected,
        tol::GPU_RASTER,
    );
}

#[test]
fn rect_envelope_opacity_applies_once_via_filter() {
    let mut doc = Document::new_v1();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let layer = doc.layers.allocate("rect").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let mut clip = rect_clip(layer.get(), [0.0, 0.0], [1.0, 1.0], [1.0, 0.0, 0.0, 1.0]);
    clip.envelope.layer_id = layer;
    clip.envelope.opacity = DocParam::const_f64(0.5);
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(clip)],
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
    let opacity_plugins = built
        .graph
        .steps
        .iter()
        .filter(|s| {
            matches!(
                s,
                motolii_render::RenderStep::Plugin {
                    id,
                    ..
                } if id.0 == "core.filter.opacity"
            )
        })
        .count();
    assert_eq!(
        opacity_plugins, 1,
        "envelope opacity must be a single filter step"
    );
    let overlay_alpha = built.graph.steps.iter().find_map(|s| match s {
        motolii_render::RenderStep::OverlayRect { overlay, .. } => Some(overlay.color[3]),
        _ => None,
    });
    assert_eq!(
        overlay_alpha,
        Some(1.0),
        "rect color alpha must stay unbaked; opacity is applied later"
    );
}

#[test]
fn source_time_comes_from_video_clip_not_last_clip() {
    use motolii_core::TimeMap;
    use motolii_doc::{Asset, AssetId};

    let mut doc = Document::new_v1();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let asset_id = AssetId::from_raw(0);
    doc.assets
        .insert(Asset {
            id: asset_id,
            name: "bg".into(),
            asset_type: "video/mp4".into(),
            content_hash: "sha256:x".into(),
            path_absolute: None,
            path_project_relative: None,
            file_name: Some("bg.mp4".into()),
            size_bytes: None,
            head_hash: None,
            tail_hash: None,
        })
        .unwrap();
    let video_layer = doc.layers.allocate("video").unwrap();
    let overlay_layer = doc.layers.allocate("overlay").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let video = Clip {
        envelope: ItemEnvelope::new(video_layer),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(10, 1).unwrap(),
        // timeline 1s → source 3s
        time_map: TimeMap::offset(RationalTime::try_new(3, 1).unwrap(), RationalTime::ZERO),
        source: ClipSource::Asset { asset: asset_id },
        path_ops: Vec::new(),
    };
    let mut overlay = rect_clip(
        overlay_layer.get(),
        [0.0, 0.0],
        [0.2, 0.2],
        [1.0, 0.0, 0.0, 1.0],
    );
    overlay.envelope.layer_id = overlay_layer;
    overlay.time_map = TimeMap::offset(RationalTime::try_new(9, 1).unwrap(), RationalTime::ZERO);
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(video), TrackItem::Clip(overlay)],
    });
    let mut registry = PluginRegistry::new();
    register_reference_plugins(&mut registry).unwrap();
    let t = RationalTime::try_new(1, 1).unwrap();
    let built = build_document_frame_graph(
        &doc,
        EvaluationTime::new(t),
        desc(),
        &DataTracks::new(),
        &registry,
        None,
    )
    .unwrap();
    assert_eq!(
        built.source_time,
        RationalTime::try_new(4, 1).unwrap(),
        "source_time must follow video TimeMap (3+1), not overlay's 9+1"
    );
}

#[test]
fn multiple_video_assets_return_typed_error() {
    use motolii_doc::{Asset, AssetId, GraphError};

    let mut doc = Document::new_v1();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let a0 = AssetId::from_raw(0);
    let a1 = AssetId::from_raw(1);
    for (id, name) in [(a0, "a"), (a1, "b")] {
        doc.assets
            .insert(Asset {
                id,
                name: name.into(),
                asset_type: "video/mp4".into(),
                content_hash: format!("sha256:{name}"),
                path_absolute: None,
                path_project_relative: None,
                file_name: Some(format!("{name}.mp4")),
                size_bytes: None,
                head_hash: None,
                tail_hash: None,
            })
            .unwrap();
    }
    let l0 = doc.layers.allocate("v0").unwrap();
    let l1 = doc.layers.allocate("v1").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let clip = |layer, asset| Clip {
        envelope: ItemEnvelope::new(layer),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(10, 1).unwrap(),
        time_map: motolii_core::TimeMap::identity(),
        source: ClipSource::Asset { asset },
        path_ops: Vec::new(),
    };
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(clip(l0, a0)), TrackItem::Clip(clip(l1, a1))],
    });
    let mut registry = PluginRegistry::new();
    register_reference_plugins(&mut registry).unwrap();
    let err = build_document_frame_graph(
        &doc,
        EvaluationTime::new(RationalTime::ZERO),
        desc(),
        &DataTracks::new(),
        &registry,
        None,
    )
    .unwrap_err();
    assert!(matches!(err, GraphError::MultipleVideoSources));
}

#[test]
fn plugin_ids_reuse_registry_static_str() {
    let mut doc = Document::new_v1();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let layer = doc.layers.allocate("c").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let mut clip = rect_clip(layer.get(), [0.0, 0.0], [1.0, 1.0], [1.0, 0.0, 0.0, 1.0]);
    clip.envelope.layer_id = layer;
    clip.envelope.effects.push(EffectInstance {
        plugin_id: "core.filter.opacity".into(),
        effect_version: 1,
        enabled: true,
        params: BTreeMap::from([("amount".into(), DocParam::const_f64(0.5))]),
        extra: Default::default(),
    });
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(clip)],
    });
    let mut registry = PluginRegistry::new();
    register_reference_plugins(&mut registry).unwrap();
    let expected = registry
        .filter_by_name("core.filter.opacity")
        .unwrap()
        .desc()
        .id
        .0;
    let built = build_document_frame_graph(
        &doc,
        EvaluationTime::new(RationalTime::ZERO),
        desc(),
        &DataTracks::new(),
        &registry,
        None,
    )
    .unwrap();
    let plugin_id = built.graph.steps.iter().find_map(|s| match s {
        motolii_render::RenderStep::Plugin { id, .. } if id.0.contains("opacity") => Some(id.0),
        _ => None,
    });
    assert_eq!(plugin_id, Some(expected));
    // 同一静的領域を指す(毎フレーム leak していないことの機械判定)。
    assert!(std::ptr::eq(plugin_id.unwrap(), expected));
}
