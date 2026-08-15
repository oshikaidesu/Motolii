//! catalog / library / place_media の試験。helper は tests。
use super::tests::*;
use super::*;

pub(super) fn seed_primary(host: u64, target: LayerId) {
    with_registry(|registry| {
        let product = registry
            .hosts
            .get_mut(&host)
            .ok_or(RnHostError::UnknownHost(host))?;
        let mut queue = DocumentEditQueue::default();
        queue.push_replace_primary(target);
        let published = product
            .runtime
            .process_next(&mut queue, product.primary, product.projection_generation)
            .expect("process")
            .expect("published");
        product.primary = published.primary;
        product.projection_generation = published.projection_generation;
        Ok(())
    })
    .expect("seed primary");
}

pub(super) fn read_wire(host: u64) -> WireProductSnapshot {
    with_registry(|registry| registry.read_snapshot(host)).expect("wire snapshot")
}

pub(super) fn layer_effects<'a>(
    wire: &'a WireProductSnapshot,
    layer_id: &str,
) -> &'a [WireTimelineEffect] {
    &wire
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer")
        .effects
}

/// RN `readSnapshot` と同じ `encode_snapshot_json` 経路。
pub(super) fn read_snapshot_json(host: u64) -> serde_json::Value {
    let json = encode_snapshot_json(&read_wire(host)).expect("snapshot json");
    serde_json::from_str(&json).expect("parse snapshot json")
}

/// App.tsx `hostSnapshotStateFromParsed` と同じ: primary の timeline layer effects だけ。
pub(super) fn inspector_selected_effects(snapshot: &serde_json::Value) -> Vec<serde_json::Value> {
    let Some(primary) = snapshot
        .get("primary_layer_id")
        .and_then(|value| value.as_str())
    else {
        return Vec::new();
    };
    snapshot
        .get("timeline")
        .and_then(|timeline| timeline.get("layers"))
        .and_then(|layers| layers.as_array())
        .and_then(|layers| {
            layers
                .iter()
                .find(|layer| layer.get("layer_id").and_then(|id| id.as_str()) == Some(primary))
        })
        .and_then(|layer| layer.get("effects"))
        .and_then(|effects| effects.as_array())
        .cloned()
        .unwrap_or_default()
}

#[test]
fn snapshot_catalog_effects_lists_attachable_first_party_plugins() {
    let _lock = test_lock();
    let host = create_host("catalog-effects");
    let wire = read_wire(host);
    let ids: Vec<_> = wire
        .catalog
        .effects
        .iter()
        .map(|effect| effect.plugin_id.as_str())
        .collect();
    assert!(ids.contains(&"core.filter.opacity"));
    assert!(!ids.contains(&"core.param.sine"));
    let opacity = wire
        .catalog
        .effects
        .iter()
        .find(|effect| effect.plugin_id == "core.filter.opacity")
        .expect("opacity");
    assert_eq!(opacity.name, "Opacity");
    assert_eq!(opacity.effect_version, 1);
    let _ = host_destroy_for_test(host);
}

#[test]
fn snapshot_library_lists_starter_media_files_not_a_fake_grid() {
    let _lock = test_lock();
    let host = create_host("library-starter-media");
    let wire = read_wire(host);
    let names: Vec<_> = wire
        .library
        .items
        .iter()
        .map(|item| item.name.as_str())
        .collect();
    assert_eq!(
        names,
        [
            "starter-clip.mp4",
            "starter-mark.svg",
            "starter-still.png",
            "starter-tone.wav"
        ]
    );
    assert_eq!(wire.library.directories.len(), 1);
    assert_eq!(wire.library.directories[0].name, "media");
    assert_eq!(
        wire.library
            .items
            .iter()
            .map(|item| item.tags.as_slice())
            .collect::<Vec<_>>(),
        [
            ["video", "mp4"].as_slice(),
            ["image", "svg"].as_slice(),
            ["image", "png"].as_slice(),
            ["audio", "wav"].as_slice(),
        ]
    );
    let tag_ids: Vec<_> = wire
        .library
        .tags
        .iter()
        .map(|tag| (tag.id.as_str(), tag.label.as_str(), tag.count))
        .collect();
    assert_eq!(
        tag_ids,
        [
            ("audio", "Audio", 1),
            ("image", "Image", 2),
            ("mp4", "mp4", 1),
            ("png", "png", 1),
            ("svg", "svg", 1),
            ("video", "Video", 1),
            ("wav", "wav", 1),
        ]
    );
    let root = wire.library.root.expect("library root");
    assert!(
        root.path.ends_with("docs/mocks-ui/starter-media/media")
            || root.path.ends_with("docs/mocks-ui/starter-media/media/"),
        "{}",
        root.path
    );
    assert!(wire
        .library
        .items
        .iter()
        .all(|item| !item.name.starts_with("asset-")));
    assert!(wire.library.tags.iter().all(|tag| tag.id != "interview"));
    let _ = host_destroy_for_test(host);
}

#[test]
fn place_media_puts_library_file_on_timeline_and_stage_bounds() {
    let _lock = test_lock();
    let host = create_empty_track_host("place-media-starter-clip");
    let baseline = read_wire(host);
    assert!(baseline.timeline.layers.is_empty());
    let clip_id = baseline
        .library
        .items
        .iter()
        .find(|item| item.name == "starter-clip.mp4")
        .map(|item| item.id.as_str())
        .expect("starter clip in library");

    let placed = dispatch_raw_json(
        host,
        &place_media_json(host, clip_id, [0.1, -0.2], r#"{"num":0,"den":1}"#),
    );
    assert!(placed.accepted, "reason={:?}", placed.reason);
    let after = read_wire(host);
    assert_eq!(after.timeline.layers.len(), 1);
    assert_eq!(after.timeline.layers[0].display_name, "starter-clip.mp4");
    let layer_id = after.timeline.layers[0].layer_id.clone();
    assert_eq!(after.primary_layer_id.as_deref(), Some(layer_id.as_str()));
    assert!(after
        .stage
        .bounds
        .iter()
        .any(|bound| bound.layer_id == layer_id));
    let document = live_document(host);
    let placed_id = LayerId::from_raw(layer_id.parse().expect("placed layer id"));
    assert_eq!(
        document.layers.display_name(placed_id),
        Some("starter-clip.mp4")
    );
    assert_eq!(document.assets.len(), 1);
    assert!(document.assets.iter().any(|asset| {
        asset.file_name.as_deref() == Some("starter-clip.mp4")
            && asset
                .path_absolute
                .as_ref()
                .is_some_and(|path| std::path::Path::new(path).ends_with("starter-clip.mp4"))
    }));
    let clip = document_clip(document.as_ref(), placed_id).expect("Document clip");
    assert!(matches!(
        clip.source,
        ClipSource::Asset { video: Some(_), .. }
    ));
    assert_eq!(
        clip.envelope.transform.position,
        motolii_doc::DocParam::const_vec2([0.1, -0.2])
    );
    let geometry = after
        .stage_geometry
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("placed media must appear in stage_geometry");
    assert_eq!(geometry.position, [0.1, -0.2]);
    assert!(
        geometry
            .corners
            .iter()
            .all(|c| c[0].is_finite() && c[1].is_finite()),
        "stage fill corners {:?}",
        geometry.corners
    );
    let area = geometry
        .corners
        .iter()
        .enumerate()
        .fold(0.0, |acc, (i, c)| {
            let n = geometry.corners[(i + 1) % 4];
            acc + (c[0] * n[1] - n[0] * c[1])
        });
    assert!(
        area.abs() > 1e-9,
        "stage fill must have area, corners={:?}",
        geometry.corners
    );
    assert_ne!(after.revision, baseline.revision);

    let runtime = first_party_runtime().expect("first_party_runtime");
    let desc = frame_desc_from_composition(document.as_ref()).expect("desc");
    let built = build_document_frame_graph(
        document.as_ref(),
        EvaluationTime::new(RationalTime::ZERO),
        desc,
        &DataTracks::new(),
        &runtime,
        None,
    )
    .expect("graph after place_media");
    assert!(
        !built.video_slots.is_empty(),
        "place_media must lower to VideoSource slots"
    );
    if motolii_testkit::ffmpeg_or_skip() {
        if let Some(gpu) = motolii_testkit::gpu_or_skip() {
            let mut binder = VideoSourceBinder::new(&gpu);
            let bound = binder
                .bind(&gpu, document.as_ref(), None, &built.video_slots, desc)
                .expect("FrameReader bind after place_media");
            assert!(
                !bound.is_empty(),
                "Preview video_sources must be filled from FrameReader"
            );
            let mut session = RenderSession::new(&gpu);
            let mut frame = None;
            assert_eq!(
                host_render_frame_for_app(host, &gpu, &mut session, &mut frame),
                HostRenderFrameResult::Rendered,
                "place_media Stage eval must render FrameReader pixels"
            );
            let rendered = frame.take().expect("stage frame");
            let bytes = download_rgba(&gpu, &rendered.texture).expect("readback");
            let background = pixel_at(&bytes, rendered.width, 0, 0);
            assert!(
                background[3] != 0
                    || has_non_background_pixel(
                        &bytes,
                        rendered.width,
                        rendered.height,
                        background
                    ),
                "place_media Stage frame must show asset pixels"
            );
        }
    }

    let unknown = dispatch_raw_json(
        host,
        &place_media_json(
            host,
            "root-0:missing.mp4",
            [0.0, 0.0],
            r#"{"num":0,"den":1}"#,
        ),
    );
    assert!(!unknown.accepted);
    assert_eq!(unknown.reason, Some(RnHostReasonCode::InvalidIntent));
    let rejected = read_wire(host);
    assert_eq!(rejected.timeline.layers.len(), 1);
    assert_eq!(rejected.timeline.layers[0].layer_id, layer_id);

    let undone = dispatch_raw_json(
        host,
        &format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"undo","host_handle":"{host}"}}"#,
            host = host,
        ),
    );
    assert!(undone.accepted);
    let after_undo = read_wire(host);
    assert!(after_undo.timeline.layers.is_empty());
    assert!(after_undo
        .stage
        .bounds
        .iter()
        .all(|bound| bound.layer_id != layer_id));
    assert!(after_undo
        .stage_geometry
        .layers
        .iter()
        .all(|layer| layer.layer_id != layer_id));
    let _ = host_destroy_for_test(host);
}

#[test]
fn snapshot_catalog_effects_are_all_attachable_filter_plugins() {
    let _lock = test_lock();
    let host = create_host("catalog-attach-loop");
    let baseline = read_snapshot(host);
    let wire = read_wire(host);
    let layer_id = baseline.layer_ids[0].clone();
    let target = LayerId::from_raw(layer_id.parse().expect("layer"));
    seed_primary(host, target);
    assert_eq!(
        read_snapshot(host).primary_layer_id,
        Some(layer_id.clone()),
        "primary should be seeded to target before loop"
    );

    let mut expected = layer_effects(&wire, &layer_id).len();
    for effect in &wire.catalog.effects {
        let intent_json = format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"attach_effect","#,
                r#""host_handle":"{host}","target":"{layer}","plugin_id":"{plugin}"}}"#,
            ),
            host = host,
            layer = layer_id,
            plugin = effect.plugin_id,
        );
        let response = dispatch_raw_json(host, &intent_json);
        assert!(
            response.accepted,
            "attach failed for {}: accepted={} reason={:?}",
            effect.plugin_id, response.accepted, response.reason,
        );
        assert_eq!(response.reason, None);
        expected += 1;
        assert_eq!(layer_effects(&read_wire(host), &layer_id).len(), expected);
    }
    let _ = host_destroy_for_test(host);
}

#[test]
fn snapshot_catalog_effects_mark_projection_truncated_at_eight_plus_nine() {
    let _lock = test_lock();
    let host = create_host("catalog-attach-truncation");
    let baseline = read_snapshot(host);
    let layer_id = baseline.layer_ids[0].clone();
    let target = LayerId::from_raw(layer_id.parse().expect("layer"));
    seed_primary(host, target);
    assert_eq!(
        read_snapshot(host).primary_layer_id,
        Some(layer_id.clone()),
        "primary should be seeded to target before truncation test"
    );
    let plugin = read_wire(host)
        .catalog
        .effects
        .first()
        .expect("catalog should have at least one filter effect")
        .plugin_id
        .clone();

    let attach_effect = |host: u64, layer_id: &str| {
        dispatch_wire(
            host,
            WireIntentEnvelope {
                version: WIRE_VERSION,
                direction: RN_TO_HOST.to_owned(),
                kind: "attach_effect".into(),
                host_handle: String::new(),
                stage_handle: None,
                projection_generation: None,
                width: None,
                height: None,
                scale_factor: None,
                focused: None,
                phase: None,
                view_local_x: None,
                view_local_y: None,
                sequence: None,
                frame: None,
                position: None,
                playhead: None,
                target: Some(layer_id.to_string()),
                dest: None,
                key_id: None,
                property: None,
                time: None,
                new: None,
                interp: None,
                delta: None,
                plugin_id: Some(plugin.clone()),
                item_id: None,
                effect_use_id: None,
                param_id: None,
                value: None,
                output_path: None,
                color: None,
            },
        )
    };

    for index in 1..=8 {
        let response = attach_effect(host, &layer_id);
        assert!(
            response.accepted,
            "attach failed at {index} plugin={plugin}: accepted={} reason={:?}",
            response.accepted, response.reason
        );
        let wire = read_wire(host);
        let layer = wire
            .timeline
            .layers
            .iter()
            .find(|layer| layer.layer_id == layer_id)
            .expect("layer");
        assert_eq!(layer.effects.len(), index);
        assert!(!layer.effects_truncated);
    }

    assert!(attach_effect(host, &layer_id).accepted);
    let wire = read_wire(host);
    let layer = wire
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer");
    assert_eq!(layer.effects.len(), 8);
    assert!(layer.effects_truncated);

    let _ = host_destroy_for_test(host);
}
