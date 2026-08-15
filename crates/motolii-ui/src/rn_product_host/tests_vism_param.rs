//! set_source_param / preview source の試験。helper は tests。
use super::tests::*;
use super::*;

#[test]
fn set_source_param_updates_count_preserves_others_and_undo_restores() {
    let _lock = test_lock();
    let host = create_empty_track_host("set-source-param");
    assert!(
        dispatch_raw_json(
            host,
            &place_vism_json(
                host,
                "core.layer_source.radial_repeater",
                [0.0, 0.0],
                r#"{"num":0,"den":1}"#
            ),
        )
        .accepted
    );
    let placed = read_wire(host);
    let layer_id = placed.primary_layer_id.clone().expect("placed primary");
    let before = placed
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("placed layer");
    let radius = before
        .source_params
        .iter()
        .find(|param| param.param_id == "radius")
        .expect("radius")
        .value;

    let changed = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"set_source_param","#,
                r#""host_handle":"{host}","target":"{layer}","param_id":"count","value":8.0}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(changed.accepted, "reason={:?}", changed.reason);
    let after = read_wire(host);
    let layer = after
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("updated layer");
    let by_id: BTreeMap<_, _> = layer
        .source_params
        .iter()
        .map(|param| (param.param_id.as_str(), param.value))
        .collect();
    assert_eq!(by_id.get("count"), Some(&8.0));
    assert_eq!(by_id.get("radius"), Some(&radius));

    assert!(
        dispatch_raw_json(
            host,
            &format!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"undo","host_handle":"{host}"}}"#,
                host = host,
            ),
        )
        .accepted
    );
    let restored = read_wire(host);
    let restored_layer = restored
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("restored layer");
    let restored_count = restored_layer
        .source_params
        .iter()
        .find(|param| param.param_id == "count")
        .expect("count")
        .value;
    assert_eq!(restored_count, 12.0);
    let _ = host_destroy_for_test(host);
}

#[test]
fn set_source_param_rejects_unknown_param_without_mutation() {
    let _lock = test_lock();
    let host = create_empty_track_host("set-source-param-unknown");
    assert!(
        dispatch_raw_json(
            host,
            &place_vism_json(
                host,
                "core.layer_source.radial_repeater",
                [0.0, 0.0],
                r#"{"num":0,"den":1}"#
            ),
        )
        .accepted
    );
    let before = document_json_bytes(host);
    let layer_id = read_wire(host).primary_layer_id.clone().expect("primary");
    let rejected = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"set_source_param","#,
                r#""host_handle":"{host}","target":"{layer}","param_id":"missing","value":1.0}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(!rejected.accepted);
    assert_eq!(document_json_bytes(host), before);
    let _ = host_destroy_for_test(host);
}

#[test]
fn preview_source_param_keeps_revision_and_commit_writes() {
    let _lock = test_lock();
    let host = create_empty_track_host("preview-source-param");
    assert!(
        dispatch_raw_json(
            host,
            &place_vism_json(
                host,
                "core.layer_source.radial_repeater",
                [0.0, 0.0],
                r#"{"num":0,"den":1}"#
            ),
        )
        .accepted
    );
    let before = read_snapshot(host);
    let layer_id = read_wire(host).primary_layer_id.clone().expect("primary");
    let previewed = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"preview_source_param","#,
                r#""host_handle":"{host}","target":"{layer}","param_id":"count","value":8.0}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(previewed.accepted, "reason={:?}", previewed.reason);
    let after_preview = read_snapshot(host);
    assert_eq!(after_preview.revision, before.revision);
    assert_ne!(
        after_preview.projection_generation,
        before.projection_generation
    );
    let preview_wire = read_wire(host);
    let timeline_count = preview_wire
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer")
        .source_params
        .iter()
        .find(|param| param.param_id == "count")
        .expect("count")
        .value;
    assert_eq!(timeline_count, 12.0);
    let preview_params = preview_wire
        .selected_doc_params
        .as_ref()
        .expect("selected_doc_params");
    let preview_count = preview_params
        .source_params
        .iter()
        .find(|param| param.param_id == "count")
        .expect("preview count")
        .value;
    assert_eq!(preview_count, 8.0);
    assert!(
        dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"set_source_param","#,
                    r#""host_handle":"{host}","target":"{layer}","param_id":"count","value":8.0}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        )
        .accepted
    );
    let after_commit = read_snapshot(host);
    assert_ne!(after_commit.revision, before.revision);
    let committed = read_wire(host)
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer")
        .source_params
        .iter()
        .find(|param| param.param_id == "count")
        .expect("count")
        .value;
    assert_eq!(committed, 8.0);
    let _ = host_destroy_for_test(host);
}

#[test]
fn rejected_source_commit_and_exhausted_preview_keep_the_same_snapshot() {
    let _lock = test_lock();
    let host = create_empty_track_host("preview-source-reject");
    assert!(
        dispatch_raw_json(
            host,
            &place_vism_json(
                host,
                "core.layer_source.radial_repeater",
                [0.0, 0.0],
                r#"{"num":0,"den":1}"#
            ),
        )
        .accepted
    );
    let layer_id = read_wire(host).primary_layer_id.clone().expect("primary");
    assert!(
        dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"preview_source_param","#,
                    r#""host_handle":"{host}","target":"{layer}","param_id":"count","value":8.0}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        )
        .accepted
    );

    let previewed = read_wire(host);
    let document = document_json_bytes(host);
    let rejected_commit = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"set_source_param","#,
                r#""host_handle":"{host}","target":"{layer}","param_id":"missing","value":12.0}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(!rejected_commit.accepted);
    assert_eq!(
        rejected_commit.reason,
        Some(RnHostReasonCode::InvalidIntent)
    );
    assert_eq!(read_wire(host), previewed);
    assert_eq!(document_json_bytes(host), document);

    let cancelled = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"preview_source_param","#,
                r#""host_handle":"{host}","target":"{layer}","param_id":"count","value":12.0}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(cancelled.accepted);
    assert_ne!(
        read_wire(host).projection_generation,
        previewed.projection_generation
    );
    assert_eq!(document_json_bytes(host), document);
    assert!(
        dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"preview_source_param","#,
                    r#""host_handle":"{host}","target":"{layer}","param_id":"count","value":8.0}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        )
        .accepted
    );

    with_registry(|registry| {
        let product = registry
            .hosts
            .get_mut(&host)
            .ok_or(RnHostError::UnknownHost(host))?;
        product.projection_generation = u64::MAX;
        Ok(())
    })
    .expect("force exhaustion");
    let at_max = read_wire(host);
    let rejected_preview = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"preview_source_param","#,
                r#""host_handle":"{host}","target":"{layer}","param_id":"count","value":9.0}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(!rejected_preview.accepted);
    assert_eq!(
        rejected_preview.reason,
        Some(RnHostReasonCode::ProjectionGenerationExhausted)
    );
    assert_eq!(read_wire(host), at_max);
    assert_eq!(document_json_bytes(host), document);
    let _ = host_destroy_for_test(host);
}

#[test]
fn timeline_source_params_cap_marks_truncated() {
    let _lock = test_lock();
    let mut document = Document::new_current();
    let layer = document.layers.allocate("many-params").expect("layer");
    let track = document.track_ids.allocate("track").expect("track");
    let mut params = BTreeMap::new();
    for i in 0..(MAX_SOURCE_PARAMS_PER_LAYER + 1) {
        params.insert(format!("p{i:02}"), DocParam::Const(DocValue::F64(i as f64)));
    }
    document.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: document.composition.duration,
            time_map: TimeMap::identity(),
            source: ClipSource::Plugin {
                plugin_id: "core.layer_source.radial_repeater".into(),
                effect_version: 1,
                params,
                extra: Default::default(),
            },
        })],
    });
    document.validate().expect("structurally valid");
    let (timeline, truncated_total) = project_timeline(&document);
    assert_eq!(timeline.layers.len(), 1);
    let projected = &timeline.layers[0];
    assert_eq!(projected.source_params.len(), MAX_SOURCE_PARAMS_PER_LAYER);
    assert!(projected.source_params_truncated);
    assert_eq!(truncated_total, 1);
    assert_eq!(projected.source_params[0].param_id, "p00");
    let last = MAX_SOURCE_PARAMS_PER_LAYER - 1;
    assert_eq!(
        projected.source_params[last].param_id,
        format!("p{last:02}")
    );
    assert_eq!(projected.source_params[last].value, last as f64);
}

#[test]
fn place_vism_rejects_missing_filter_kind_and_non_finite_without_mutation() {
    let _lock = test_lock();
    let host = create_empty_track_host("place-vism-rejects");
    let before = serde_json::to_vec(&read_wire(host)).expect("before");

    let missing = dispatch_raw_json(
        host,
        &place_vism_json(
            host,
            "core.layer_source.missing",
            [0.0, 0.0],
            r#"{"num":0,"den":1}"#,
        ),
    );
    assert!(!missing.accepted);
    assert_eq!(missing.reason, Some(RnHostReasonCode::DocumentPluginError));

    let filter_kind = dispatch_raw_json(
        host,
        &place_vism_json(
            host,
            "core.filter.opacity",
            [0.0, 0.0],
            r#"{"num":0,"den":1}"#,
        ),
    );
    assert!(!filter_kind.accepted);
    assert_eq!(
        filter_kind.reason,
        Some(RnHostReasonCode::DocumentPluginError)
    );

    let non_finite = dispatch_wire(
        host,
        WireIntentEnvelope {
            version: 1,
            direction: RN_TO_HOST.to_owned(),
            kind: "place_vism".into(),
            host_handle: host.to_string(),
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
            position: Some([f64::NAN, 0.0]),
            playhead: Some(RationalTime::ZERO),
            target: None,
            dest: None,
            key_id: None,
            property: None,
            time: None,
            new: None,
            interp: None,
            delta: None,
            plugin_id: Some("core.layer_source.radial_repeater".into()),
            item_id: None,
            effect_use_id: None,
            param_id: None,
            value: None,
            output_path: None,
            color: None,
        },
    );
    assert!(!non_finite.accepted);
    assert_eq!(non_finite.reason, Some(RnHostReasonCode::InvalidIntent));

    let after = serde_json::to_vec(&read_wire(host)).expect("after");
    assert_eq!(after, before);
    let _ = host_destroy_for_test(host);
}

#[test]
fn place_vism_frame_graph_passes_unused_texture_write_wiring() {
    let _lock = test_lock();
    let host = create_empty_track_host("place-vism-graph-wiring");
    assert!(
        dispatch_raw_json(
            host,
            &place_vism_json(
                host,
                "core.layer_source.radial_repeater",
                [0.0, 0.0],
                r#"{"num":0,"den":1}"#
            ),
        )
        .accepted
    );

    let document =
        with_registry(|registry| Ok(registry.hosts.get(&host).expect("host").runtime.snapshot()))
            .expect("snapshot");
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
    .expect("build graph after place_vism");
    validate_render_graph_wiring(
        &built.graph,
        RationalTime::ZERO,
        &RenderGraphInputs {
            camera: built.camera,
            video_sources: &[],
            source_time: Some(built.source_time),
            plugins: Some(runtime.executors()),
        },
    )
    .expect("place_vism graph must pass UnusedTextureWrite wiring");
    let _ = host_destroy_for_test(host);
}

#[test]
fn host_render_frame_after_place_vism_has_non_uniform_pixels() {
    let _lock = test_lock();
    let Some(gpu) = motolii_testkit::gpu_or_skip() else {
        return;
    };
    let mut session = RenderSession::new(&gpu);
    let host = create_empty_track_host("place-vism-frame-readback");
    assert!(
        dispatch_raw_json(
            host,
            &place_vism_json(
                host,
                "core.layer_source.radial_repeater",
                [0.0, 0.0],
                r#"{"num":0,"den":1}"#
            ),
        )
        .accepted
    );

    let mut frame = None;
    assert_eq!(
        host_render_frame_for_app(host, &gpu, &mut session, &mut frame),
        HostRenderFrameResult::Rendered
    );
    let first = frame.take().expect("frame");
    let bytes = download_rgba(&gpu, &first.texture).expect("frame readback");
    assert_eq!(
        bytes.len(),
        (first.width as usize) * (first.height as usize) * 4
    );
    let background = pixel_at(&bytes, first.width, 0, 0);
    assert!(has_non_background_pixel(
        &bytes,
        first.width,
        first.height,
        background
    ));
    let _ = host_destroy_for_test(host);
}
