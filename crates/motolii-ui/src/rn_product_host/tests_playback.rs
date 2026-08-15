//! playback / export の試験。helper は tests。
use super::tests::*;
use super::*;

#[test]
fn toggle_playback_without_session_does_not_move_time() {
    let _lock = test_lock();
    let host = create_host("toggle-playback");
    let before = read_snapshot(host);
    let response = dispatch_raw_json(
        host,
        &format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"toggle_playback","host_handle":"{host}"}}"#,
            host = host,
        ),
    );
    let after = read_snapshot(host);
    assert_eq!(after.current_time, before.current_time);
    if response.accepted {
        assert_eq!(read_wire(host).playback_state, WirePlaybackState::Playing);
        assert!(dispatch_raw_json(
                host,
                &format!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"toggle_playback","host_handle":"{host}"}}"#,
                    host = host,
                ),
            )
            .accepted);
        assert_eq!(read_wire(host).playback_state, WirePlaybackState::Idle);
    } else {
        assert_eq!(response.reason, Some(RnHostReasonCode::InvalidIntent));
        assert_eq!(read_wire(host).playback_state, WirePlaybackState::Idle);
    }
    let _ = host_destroy_for_test(host);
}

#[test]
fn shuttle_forward_starts_and_does_not_toggle_off() {
    let _lock = test_lock();
    let host = create_host("shuttle-forward");
    let before = read_snapshot(host);
    let start = dispatch_raw_json(host, &host_kind_json(host, "shuttle_forward"));
    assert_eq!(read_snapshot(host).current_time, before.current_time);
    if start.accepted {
        assert_eq!(read_wire(host).playback_state, WirePlaybackState::Playing);
        assert!(dispatch_raw_json(host, &host_kind_json(host, "shuttle_forward")).accepted);
        assert_eq!(read_wire(host).playback_state, WirePlaybackState::Playing);
        assert!(dispatch_raw_json(host, &host_kind_json(host, "shuttle_stop")).accepted);
        assert_eq!(read_wire(host).playback_state, WirePlaybackState::Idle);
    } else {
        assert_eq!(start.reason, Some(RnHostReasonCode::InvalidIntent));
        assert_eq!(read_wire(host).playback_state, WirePlaybackState::Idle);
    }
    let _ = host_destroy_for_test(host);
}

#[test]
fn shuttle_stop_does_not_start_playback() {
    let _lock = test_lock();
    let host = create_host("shuttle-stop");
    let before = read_snapshot(host);
    assert_eq!(read_wire(host).playback_state, WirePlaybackState::Idle);
    assert!(dispatch_raw_json(host, &host_kind_json(host, "shuttle_stop")).accepted);
    let after = read_snapshot(host);
    assert_eq!(after.current_time, before.current_time);
    assert_eq!(read_wire(host).playback_state, WirePlaybackState::Idle);
    let _ = host_destroy_for_test(host);
}

#[test]
fn shuttle_reverse_steps_playhead_via_set_time_and_stops_at_zero() {
    let _lock = test_lock();
    let host = create_host("shuttle-reverse");
    let fps = read_snapshot(host).timeline.fps;
    assert!(dispatch_raw_json(host, &set_time_json(host, "45")).accepted);
    assert!(dispatch_raw_json(host, &host_kind_json(host, "shuttle_reverse")).accepted);
    assert_eq!(
        read_snapshot(host).current_time,
        RationalTime::try_from_frame(44, fps).expect("frame 44")
    );
    assert_eq!(read_wire(host).playback_state, WirePlaybackState::Idle);

    assert!(dispatch_raw_json(host, &set_time_json(host, "0")).accepted);
    let at_zero = read_snapshot(host);
    assert!(dispatch_raw_json(host, &host_kind_json(host, "shuttle_reverse")).accepted);
    let still = read_snapshot(host);
    assert_eq!(still.current_time, RationalTime::ZERO);
    assert_eq!(still.current_time, at_zero.current_time);
    let _ = host_destroy_for_test(host);
}

#[test]
fn preview_effect_param_keeps_revision_and_commit_writes() {
    let _lock = test_lock();
    let host = create_host("preview-effect-param");
    let baseline = read_snapshot(host);
    let layer_id = baseline.layer_ids[0].clone();
    let target = LayerId::from_raw(layer_id.parse().expect("layer"));
    seed_primary(host, target);
    assert!(dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"attach_effect","#,
                    r#""host_handle":"{host}","target":"{layer}","plugin_id":"core.filter.opacity"}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        )
        .accepted);
    let attached_wire = read_wire(host);
    let attached = &layer_effects(&attached_wire, &layer_id)[0];
    let effect_use_id = attached.effect_use_id.clone();
    let before = read_snapshot(host);
    let previewed = dispatch_raw_json(
        host,
        &format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"preview_effect_param","host_handle":"{host}","target":"{layer}","effect_use_id":"{effect}","param_id":"amount","value":0.25}}"#,
            host = host,
            layer = layer_id,
            effect = effect_use_id,
        ),
    );
    assert!(previewed.accepted, "preview reason={:?}", previewed.reason);
    let after_preview = read_snapshot(host);
    assert_eq!(after_preview.revision, before.revision);
    assert_ne!(
        after_preview.projection_generation,
        before.projection_generation
    );
    let preview_wire = read_wire(host);
    assert_eq!(
        layer_effects(&preview_wire, &layer_id)[0].params[0].value,
        1.0
    );
    let preview_params = preview_wire
        .selected_doc_params
        .as_ref()
        .expect("selected_doc_params");
    assert_eq!(preview_params.layer_id, layer_id);
    assert_eq!(preview_params.effects[0].params[0].value, 0.25);
    assert!(dispatch_raw_json(
            host,
            &format!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"set_effect_param","host_handle":"{host}","target":"{layer}","effect_use_id":"{effect}","param_id":"amount","value":0.25}}"#,
                host = host,
                layer = layer_id,
                effect = effect_use_id,
            ),
        )
        .accepted);
    let after_commit = read_snapshot(host);
    assert_ne!(after_commit.revision, before.revision);
    assert_eq!(
        layer_effects(&read_wire(host), &layer_id)[0].params[0].value,
        0.25
    );
    let _ = host_destroy_for_test(host);
}

#[test]
fn rejected_effect_commit_preserves_preview_and_same_live_cancels_it() {
    let _lock = test_lock();
    let host = create_host("preview-effect-reject");
    let layer_id = read_snapshot(host).layer_ids[0].clone();
    seed_primary(host, LayerId::from_raw(layer_id.parse().expect("layer")));
    assert!(dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"attach_effect","#,
                    r#""host_handle":"{host}","target":"{layer}","plugin_id":"core.filter.opacity"}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        )
        .accepted);
    let effect_use_id = layer_effects(&read_wire(host), &layer_id)[0]
        .effect_use_id
        .clone();
    assert!(dispatch_raw_json(
            host,
            &format!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"preview_effect_param","host_handle":"{host}","target":"{layer}","effect_use_id":"{effect}","param_id":"amount","value":0.25}}"#,
                host = host,
                layer = layer_id,
                effect = effect_use_id,
            ),
        )
        .accepted);

    let previewed = read_wire(host);
    let document = document_json_bytes(host);
    let rejected = dispatch_raw_json(
        host,
        &format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"set_effect_param","host_handle":"{host}","target":"{layer}","effect_use_id":"{effect}","param_id":"missing","value":1.0}}"#,
            host = host,
            layer = layer_id,
            effect = effect_use_id,
        ),
    );
    assert!(!rejected.accepted);
    assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));
    assert_eq!(read_wire(host), previewed);
    assert_eq!(document_json_bytes(host), document);

    let cancelled = dispatch_raw_json(
        host,
        &format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"preview_effect_param","host_handle":"{host}","target":"{layer}","effect_use_id":"{effect}","param_id":"amount","value":1.0}}"#,
            host = host,
            layer = layer_id,
            effect = effect_use_id,
        ),
    );
    assert!(cancelled.accepted);
    assert_ne!(
        read_wire(host).projection_generation,
        previewed.projection_generation
    );
    assert_eq!(document_json_bytes(host), document);
    let _ = host_destroy_for_test(host);
}

#[test]
fn export_document_rejects_empty_path_without_writing_document() {
    let _lock = test_lock();
    let host = create_host("export-empty-path");
    let before = document_json_bytes(host);
    let rejected = dispatch_raw_json(
        host,
        &format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"export_document","host_handle":"{host}","output_path":""}}"#
        ),
    );
    assert!(!rejected.accepted);
    assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));
    assert_eq!(rejected.message.as_deref(), Some("output path is required"));
    assert_eq!(document_json_bytes(host), before);
    let _ = host_destroy_for_test(host);
}

#[test]
fn export_document_rejects_missing_path_without_writing_document() {
    let _lock = test_lock();
    let host = create_host("export-missing-path");
    let before = document_json_bytes(host);
    let rejected = dispatch_raw_json(
        host,
        &format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"export_document","host_handle":"{host}"}}"#
        ),
    );
    assert!(!rejected.accepted);
    assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));
    assert_eq!(rejected.message.as_deref(), Some("output path is required"));
    assert_eq!(document_json_bytes(host), before);
    let _ = host_destroy_for_test(host);
}

#[test]
fn export_document_refuses_plugin_only_document_without_creating_file() {
    let _lock = test_lock();
    let host = create_host("export-no-video");
    let before = document_json_bytes(host);
    let output = tmp_dir("rn-export-no-video").join("out.mp4");
    let rejected = dispatch_raw_json(
        host,
        &format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"export_document","host_handle":"{host}","output_path":"{}"}}"#,
            output.display()
        ),
    );
    assert!(!rejected.accepted);
    assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));
    assert!(
        rejected
            .message
            .as_deref()
            .is_some_and(|message| !message.is_empty()),
        "refusal must carry an immediate reason, got {:?}",
        rejected.message
    );
    assert!(
        !output.exists(),
        "refused export must not create {}",
        output.display()
    );
    assert_eq!(document_json_bytes(host), before);
    let _ = host_destroy_for_test(host);
}

#[test]
fn set_opacity_and_attach_effect_reject_without_primary() {
    let _lock = test_lock();
    let host = create_host("edit-without-primary");
    let baseline = read_snapshot(host);
    assert!(baseline.primary_layer_id.is_none());
    let layer_id = baseline.layer_ids[0].clone();
    let before = document_json_bytes(host);

    let opacity = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"set_opacity","#,
                r#""host_handle":"{host}","target":"{layer}","value":0.4}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(!opacity.accepted);
    assert_eq!(opacity.reason, Some(RnHostReasonCode::InvalidIntent));

    let attach = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"attach_effect","#,
                r#""host_handle":"{host}","target":"{layer}","plugin_id":"core.filter.opacity"}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(!attach.accepted);
    assert_eq!(attach.reason, Some(RnHostReasonCode::InvalidIntent));
    assert_eq!(document_json_bytes(host), before);
    let _ = host_destroy_for_test(host);
}

#[test]
fn seventeen_place_rectangle_layers_are_not_truncated() {
    let _lock = test_lock();
    let host = create_empty_track_host("seventeen-layers");
    for _ in 0..17 {
        let placed = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"place_rectangle","#,
                    r#""host_handle":"{host}","position":[0.0,0.0],"playhead":{{"num":0,"den":1}}}}"#
                ),
                host = host,
            ),
        );
        assert!(placed.accepted, "reason={:?}", placed.reason);
    }
    let wire = read_wire(host);
    assert_eq!(wire.timeline.layers.len(), 17);
    assert!(!wire.timeline.layers_truncated);
    let _ = host_destroy_for_test(host);
}

#[test]
fn set_source_param_color_writes_and_projects() {
    let _lock = test_lock();
    let host = create_empty_track_host("set-source-param-color");
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
    let changed = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"set_source_param","#,
                r#""host_handle":"{host}","target":"{layer}","param_id":"color","#,
                r#""color":[0.2,0.4,0.6,1.0]}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(changed.accepted, "reason={:?}", changed.reason);
    let wire = read_wire(host);
    let layer = wire
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer");
    let color = layer
        .source_params
        .iter()
        .find(|param| param.param_id == "color")
        .expect("color");
    assert_eq!(color.color, Some([0.2, 0.4, 0.6, 1.0]));
    assert_eq!(color.value, 0.0);
    let _ = host_destroy_for_test(host);
}

#[test]
fn timeline_effect_color_const_is_projected() {
    let mut document = Document::new_current();
    let layer = document.layers.allocate("fx").expect("layer");
    let track = document.track_ids.allocate("track").expect("track");
    let def_id = EffectDefinitionId::from_raw(document.next_stable_id.allocate().expect("def"));
    let use_id = EffectId::from_raw(document.next_stable_id.allocate().expect("use"));
    document.effect_definitions.push(EffectDefinition::new(
        def_id,
        "vendor.filter.fixture",
        1,
        true,
        BTreeMap::from([
            ("amount".into(), DocParam::const_f64(0.5)),
            ("tint".into(), DocParam::const_color([0.1, 0.2, 0.3, 1.0])),
        ]),
        Default::default(),
    ));
    let mut envelope = ItemEnvelope::new(layer);
    envelope.effects.push(EffectUse {
        id: use_id,
        definition_id: def_id,
    });
    document.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope,
            start: RationalTime::ZERO,
            duration: document.composition.duration,
            time_map: TimeMap::identity(),
            source: ClipSource::Plugin {
                plugin_id: RECT_LAYER_SOURCE.into(),
                effect_version: 1,
                params: rect_params([0.0, 0.0], [1.0, 1.0]),
                extra: Default::default(),
            },
        })],
    });
    let (effects, truncated, hidden) = project_layer_effects(&document, layer);
    assert!(!truncated);
    assert_eq!(hidden, 0);
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].params.len(), 2);
    let amount = effects[0]
        .params
        .iter()
        .find(|param| param.param_id == "amount")
        .expect("amount");
    assert_eq!(amount.value, 0.5);
    assert_eq!(amount.color, None);
    let tint = effects[0]
        .params
        .iter()
        .find(|param| param.param_id == "tint")
        .expect("tint");
    assert_eq!(tint.value, 0.0);
    assert_eq!(tint.color, Some([0.1, 0.2, 0.3, 1.0]));
}
