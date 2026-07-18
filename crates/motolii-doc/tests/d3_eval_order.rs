#![allow(deprecated)]

//! D3: F-3評価順・B④3軸・DataTrack型照合の意味論ゴールデン。

use std::collections::BTreeMap;

use motolii_core::{ColorSpace, FrameDesc, PixelFormat, Quality, RationalTime, TimeMap};
use motolii_doc::param_eval::eval_doc_param;
use motolii_doc::{
    build_document_frame_graph, Clip, ClipSource, ClippingMaskSettings, Composition, DocParam,
    DocValue, Document, EffectDefinition, EffectDefinitionId, EffectId, EffectUse, EvaluationTime,
    Group, ItemEnvelope, MaskMode, ParamEvalError, Track, TrackItem, RECT_LAYER_SOURCE,
};
use motolii_eval::{DataTrack, DataTrackId, DataTracks, Value};
use motolii_gpu::download_rgba;
use motolii_plugin::PluginRuntime;
use motolii_plugins_firstparty::first_party_runtime;
use motolii_render::{render_graph_cached, RenderGraphInputs, RenderSession};
use motolii_testkit::cpu_reference::{expected_rect_frame, premul_over_u8};
use motolii_testkit::{assert_rgba_close, gpu_or_skip, tol, RgbaImageDesc};

const W: u32 = 16;
const H: u32 = 8;

fn desc() -> FrameDesc {
    FrameDesc::packed(W, H, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true)
}

fn reference_runtime() -> PluginRuntime {
    first_party_runtime().unwrap()
}

/// D1l: `EffectUse`(env側)+`EffectDefinition`(doc側)を1回で作り、Useのidを返す。
fn push_effect(
    doc: &mut Document,
    env: &mut ItemEnvelope,
    plugin_id: &str,
    params: BTreeMap<String, DocParam>,
) -> EffectId {
    let use_id = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
    let definition_id = EffectDefinitionId::from_raw(doc.next_stable_id.allocate().unwrap());
    doc.effect_definitions.push(EffectDefinition::new(
        definition_id,
        plugin_id,
        1,
        true,
        params,
        Default::default(),
    ));
    env.effects.push(EffectUse {
        id: use_id,
        definition_id,
    });
    doc.version = doc
        .version
        .max(motolii_doc::MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS);
    doc.min_reader_version = doc
        .min_reader_version
        .max(motolii_doc::MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS);
    use_id
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

#[test]
fn masked_group_effect_applies_before_clipping_mask() {
    let Some(_) = gpu_or_skip() else { return };
    let mut doc = Document::new_current();
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
    push_effect(
        &mut doc,
        &mut content_clip.envelope,
        "core.filter.tint",
        BTreeMap::from([("color".into(), DocParam::const_color([0.0, 1.0, 0.0, 1.0]))]),
    );
    content_clip.envelope.clipping_mask = ClippingMaskSettings {
        enabled: true,
        mode: MaskMode::Luminance,
    };
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(mask_clip), TrackItem::Clip(content_clip)],
    });
    let actual = render_doc(&mut doc).expect("gpu");
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
    let mut doc = Document::new_current();
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
    let mut group_envelope = ItemEnvelope::new(group_layer);
    push_effect(
        &mut doc,
        &mut group_envelope,
        "core.filter.opacity",
        BTreeMap::from([("amount".into(), DocParam::const_f64(0.5))]),
    );
    let group = Group {
        envelope: group_envelope,
        children: vec![TrackItem::Clip(child)],
    };
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(bg), TrackItem::Group(group)],
    });
    let actual = render_doc(&mut doc).expect("gpu");
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
    let mut doc = Document::new_current();
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
    let actual = render_doc(&mut doc).expect("gpu");
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
    let mut doc = Document::new_current();
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
    let actual = render_doc(&mut doc).expect("gpu");
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
    let mut doc = Document::new_current();
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
    let actual = render_doc(&mut doc).expect("gpu");
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
    let mut doc = Document::new_current();
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
    assert!(matches!(err, motolii_doc::GraphError::InvalidClip { .. }));
}

#[test]
fn black_overrun_rejected_even_when_clip_inactive() {
    let mut doc = Document::new_current();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let layer = doc.layers.allocate("clip").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let mut clip = rect_clip(layer.get(), [0.0, 0.0], [1.0, 1.0], [1.0, 0.0, 0.0, 1.0]);
    clip.envelope.layer_id = layer;
    clip.start = RationalTime::try_new(5, 1).unwrap();
    clip.duration = RationalTime::try_new(1, 1).unwrap();
    clip.time_map =
        TimeMap::try_new(RationalTime::ZERO, 1, 1, motolii_core::OverrunMode::Black).unwrap();
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(clip)],
    });
    let runtime = reference_runtime();
    // t=0 ではクリップ非アクティブだが、Black を黙って Freeze 相当にしない。
    let err = build_document_frame_graph(
        &doc,
        EvaluationTime::new(RationalTime::ZERO),
        desc(),
        &DataTracks::new(),
        &runtime,
        None,
    )
    .unwrap_err();
    assert!(matches!(err, motolii_doc::GraphError::InvalidClip { .. }));
}

#[test]
fn source_time_comes_from_video_clip_not_overlay() {
    use motolii_doc::{Asset, AssetId};

    let mut doc = Document::new_current();
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
        source: ClipSource::asset_video_only(asset_id),
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
    let runtime = reference_runtime();
    let t = RationalTime::try_new(1, 1).unwrap();
    let built = build_document_frame_graph(
        &doc,
        EvaluationTime::new(t),
        desc(),
        &DataTracks::new(),
        &runtime,
        None,
    )
    .unwrap();
    assert_eq!(
        built.source_time,
        RationalTime::try_new(4, 1).unwrap(),
        "source_time must follow video TimeMap (3+1), not overlay"
    );
}

#[test]
fn f3_effect_before_transform_in_graph_steps() {
    let mut doc = Document::new_current();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let layer = doc.layers.allocate("rect").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let mut clip = rect_clip(layer.get(), [0.0, 0.0], [0.4, 0.4], [1.0, 0.0, 0.0, 1.0]);
    clip.envelope.layer_id = layer;
    clip.envelope.transform.position = DocParam::const_vec2([0.25, 0.0]);
    push_effect(
        &mut doc,
        &mut clip.envelope,
        "core.filter.tint",
        BTreeMap::from([("color".into(), DocParam::const_color([0.0, 1.0, 0.0, 1.0]))]),
    );
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(clip)],
    });
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
    use motolii_render::RenderStep;
    let mut saw_overlay = false;
    let mut saw_effect = false;
    let mut saw_place = false;
    for step in &built.graph.steps {
        match step {
            RenderStep::OverlayRect { .. } => {
                assert!(!saw_effect && !saw_place, "source before effect/transform");
                saw_overlay = true;
            }
            RenderStep::Plugin { .. } if saw_overlay && !saw_place => {
                saw_effect = true;
            }
            RenderStep::AffinePlace { .. } => {
                assert!(saw_effect, "effect must precede AffinePlace");
                saw_place = true;
            }
            _ => {}
        }
    }
    assert!(saw_overlay && saw_effect && saw_place);
}

#[test]
fn parent_transform_composes_into_affine_place() {
    let mut doc = Document::new_current();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let parent_layer = doc.layers.allocate("parent").unwrap();
    let child_layer = doc.layers.allocate("child").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let mut parent = rect_clip(
        parent_layer.get(),
        [0.0, 0.0],
        [0.1, 0.1],
        [0.0, 0.0, 1.0, 1.0],
    );
    parent.envelope.layer_id = parent_layer;
    parent.envelope.transform.position = DocParam::const_vec2([0.2, 0.0]);
    parent.envelope.visible = false;
    let mut child = rect_clip(
        child_layer.get(),
        [0.0, 0.0],
        [0.1, 0.1],
        [1.0, 0.0, 0.0, 1.0],
    );
    child.envelope.layer_id = child_layer;
    child.envelope.transform.position = DocParam::const_vec2([0.1, 0.0]);
    child.envelope.transform.parent = Some(parent_layer);
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(parent), TrackItem::Clip(child)],
    });
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
    // parent(0.2)+child(0.1)=0.3 の平行移動が AffinePlace に載る。
    let place = built
        .graph
        .steps
        .iter()
        .find_map(|s| match s {
            motolii_render::RenderStep::AffinePlace { inverse_uv, .. } => Some(*inverse_uv),
            _ => None,
        })
        .expect("child must emit AffinePlace for composed parent transform");
    // 恒等以外であること(合成済み)。
    assert!(
        (place[2]).abs() > 1e-4 || (place[5]).abs() > 1e-4 || (place[0] - 1.0).abs() > 1e-4,
        "expected non-identity inverse_uv, got {place:?}"
    );
}

#[test]
fn two_video_assets_get_independent_slots() {
    use motolii_doc::{Asset, AssetId};

    let mut doc = Document::new_current();
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
    let c0 = Clip {
        envelope: ItemEnvelope::new(l0),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(10, 1).unwrap(),
        time_map: TimeMap::offset(RationalTime::try_new(1, 1).unwrap()),
        source: ClipSource::asset_video_only(a0),
    };
    let c1 = Clip {
        envelope: ItemEnvelope::new(l1),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(10, 1).unwrap(),
        time_map: TimeMap::offset(RationalTime::try_new(2, 1).unwrap()),
        source: ClipSource::asset_video_only(a1),
    };
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(c0), TrackItem::Clip(c1)],
    });
    let runtime = reference_runtime();
    let t = RationalTime::try_new(1, 1).unwrap();
    let built = build_document_frame_graph(
        &doc,
        EvaluationTime::new(t),
        desc(),
        &DataTracks::new(),
        &runtime,
        None,
    )
    .unwrap();
    assert_eq!(built.video_slots.len(), 2);
    assert_eq!(built.video_slots[0].asset, a0);
    assert_eq!(built.video_slots[1].asset, a1);
    assert_eq!(
        built.video_slots[0].source_time,
        RationalTime::try_new(2, 1).unwrap()
    );
    assert_eq!(
        built.video_slots[1].source_time,
        RationalTime::try_new(3, 1).unwrap()
    );
    assert_eq!(built.source_time, built.video_slots[0].source_time);
}
