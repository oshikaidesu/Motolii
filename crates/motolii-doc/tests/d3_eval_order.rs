//! D3: F-3評価順・B④3軸・DataTrack型照合の意味論ゴールデン。

use std::collections::BTreeMap;

use motolii_core::{ColorSpace, FrameDesc, PixelFormat, Quality, RationalTime, TimeMap};
use motolii_doc::param_eval::eval_doc_param;
use motolii_doc::{
    build_document_frame_graph, Clip, ClipSource, ClippingMaskSettings, DocParam, DocValue,
    Document, EffectId, EffectInstance, EvaluationTime, Group, ItemEnvelope, MaskMode,
    ParamEvalError, Track, TrackItem, RECT_LAYER_SOURCE,
};
use motolii_eval::{DataTrack, DataTrackId, DataTracks, Value};
use motolii_gpu::download_rgba;
use motolii_plugin::reference::register_reference_plugins;
use motolii_plugin::PluginRegistry;
use motolii_render::{render_graph_cached, RenderGraphInputs, RenderSession};
use motolii_testkit::cpu_reference::{expected_rect_frame, premul_over_u8};
use motolii_testkit::{assert_rgba_close, gpu_or_skip, tol, RgbaImageDesc};

const W: u32 = 16;
const H: u32 = 8;

fn desc() -> FrameDesc {
    FrameDesc::packed(W, H, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true)
}

fn alloc_effect(doc: &mut Document) -> EffectId {
    let id = doc.next_stable_id.allocate().unwrap();
    doc.version = doc.version.max(2);
    doc.min_reader_version = doc.min_reader_version.max(2);
    EffectId::from_raw(id)
}

fn rect_clip(layer: u64, center: [f64; 2], size: [f64; 2], color: [f64; 4]) -> Clip {
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
                ("size".into(), DocParam::const_vec2(size)),
                ("color".into(), DocParam::const_color(color)),
            ]),
            extra: Default::default(),
        },
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
    let tint_id = alloc_effect(&mut doc);
    content_clip.envelope.effects.push(EffectInstance {
        id: tint_id,
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
    let opacity_id = alloc_effect(&mut doc);
    let group = Group {
        envelope: {
            let mut env = ItemEnvelope::new(group_layer);
            env.effects.push(EffectInstance {
                id: opacity_id,
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
fn visible_false_excludes_draw_but_keeps_mask_source() {
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
    mask_clip.envelope.visible = false;
    let mut content_clip = rect_clip(
        content_layer.get(),
        [0.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0, 0.0, 1.0],
    );
    content_clip.envelope.layer_id = content_layer;
    content_clip.envelope.clipping_mask = ClippingMaskSettings {
        enabled: true,
        mode: MaskMode::Luminance,
    };
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(mask_clip), TrackItem::Clip(content_clip)],
    });
    let actual = render_doc(&doc).expect("gpu");
    // マスク自体は描画されないが、マスク形状は content に効く。
    let expected = expected_rect_frame(
        desc(),
        [0, 0, 0, 0],
        [0, 255, 0, 255],
        [-0.25, 0.0],
        [0.5, 1.0],
    );
    assert_rgba_close(
        "d3-visible-false-mask",
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
fn solo_draws_only_solo_set() {
    let Some(_) = gpu_or_skip() else { return };
    let mut doc = Document::new_v1();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let a = doc.layers.allocate("a").unwrap();
    let b = doc.layers.allocate("b").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let mut red = rect_clip(a.get(), [0.0, 0.0], [1.0, 1.0], [1.0, 0.0, 0.0, 1.0]);
    red.envelope.layer_id = a;
    red.envelope.solo = true;
    let mut blue = rect_clip(b.get(), [0.0, 0.0], [1.0, 1.0], [0.0, 0.0, 1.0, 1.0]);
    blue.envelope.layer_id = b;
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(red), TrackItem::Clip(blue)],
    });
    let actual = render_doc(&doc).expect("gpu");
    let expected = expected_rect_frame(
        desc(),
        [0, 0, 0, 0],
        [255, 0, 0, 255],
        [0.0, 0.0],
        [1.0, 1.0],
    );
    assert_rgba_close(
        "d3-solo-filter",
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
fn lock_does_not_affect_draw_or_eval() {
    let Some(_) = gpu_or_skip() else { return };
    let mut doc = Document::new_v1();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let layer = doc.layers.allocate("locked").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let mut clip = rect_clip(layer.get(), [0.0, 0.0], [1.0, 1.0], [1.0, 0.0, 0.0, 1.0]);
    clip.envelope.layer_id = layer;
    clip.envelope.lock = true;
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(clip)],
    });
    let actual = render_doc(&doc).expect("gpu");
    let expected = expected_rect_frame(
        desc(),
        [0, 0, 0, 0],
        [255, 0, 0, 255],
        [0.0, 0.0],
        [1.0, 1.0],
    );
    assert_rgba_close(
        "d3-lock-noop",
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
fn data_track_output_type_must_match_fallback() {
    use motolii_core::Fps;
    let track_id = DataTrackId("pos".into());
    let mut tracks = DataTracks::new();
    // fallback は Vec2 期待だが、実トラックは F64。
    tracks.insert(
        track_id.clone(),
        DataTrack {
            start: RationalTime::ZERO,
            sample_rate: Fps::try_new(1, 1).unwrap(),
            values: vec![Value::F64(0.5)],
        },
    );
    let param = DocParam::Data {
        track: track_id,
        fallback: DocValue::Vec2([0.0, 0.0]),
    };
    let err = eval_doc_param(&param, RationalTime::ZERO, &tracks, &Default::default()).unwrap_err();
    assert!(matches!(err, ParamEvalError::DataTrackTypeMismatch { .. }));
}

#[test]
fn data_track_matching_type_evaluates() {
    use motolii_core::Fps;
    let track_id = DataTrackId("pos".into());
    let mut tracks = DataTracks::new();
    tracks.insert(
        track_id.clone(),
        DataTrack {
            start: RationalTime::ZERO,
            sample_rate: Fps::try_new(1, 1).unwrap(),
            values: vec![Value::Vec2([0.25, -0.5])],
        },
    );
    let param = DocParam::Data {
        track: track_id,
        fallback: DocValue::Vec2([0.0, 0.0]),
    };
    let got = eval_doc_param(&param, RationalTime::ZERO, &tracks, &Default::default()).unwrap();
    assert_eq!(got, Value::Vec2([0.25, -0.5]));
}

#[test]
fn black_overrun_is_typed_error_not_silent_freeze() {
    let mut doc = Document::new_v1();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let layer = doc.layers.allocate("clip").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let mut clip = rect_clip(layer.get(), [0.0, 0.0], [1.0, 1.0], [1.0, 0.0, 0.0, 1.0]);
    clip.envelope.layer_id = layer;
    clip.time_map =
        TimeMap::try_new(RationalTime::ZERO, 1, 1, motolii_core::OverrunMode::Black).unwrap();
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(clip)],
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
    assert!(matches!(err, motolii_doc::GraphError::InvalidClip { .. }));
}

#[test]
fn source_time_comes_from_video_clip_not_overlay() {
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
        time_map: TimeMap::offset(RationalTime::try_new(3, 1).unwrap()),
        source: ClipSource::Asset { asset: asset_id },
    };
    let mut overlay = rect_clip(
        overlay_layer.get(),
        [0.0, 0.0],
        [0.2, 0.2],
        [1.0, 0.0, 0.0, 1.0],
    );
    overlay.envelope.layer_id = overlay_layer;
    overlay.time_map = TimeMap::offset(RationalTime::try_new(9, 1).unwrap());
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
        "source_time must follow video TimeMap (3+1), not overlay"
    );
}
