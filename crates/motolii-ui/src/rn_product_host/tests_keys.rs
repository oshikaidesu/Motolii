//! position/param key 投影の試験。helper は tests。
use super::tests::*;
use super::*;

#[test]
fn place_rectangle_add_param_key_scale_appears_in_param_keys() {
    let _lock = test_lock();
    let host = create_empty_track_host("add-param-key-scale");
    assert!(dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"place_rectangle","#,
                    r#""host_handle":"{host}","position":[0.0,0.0],"playhead":{{"num":0,"den":1}}}}"#
                ),
                host = host,
            ),
        )
        .accepted);
    let wire = read_wire(host);
    let layer_id = wire.primary_layer_id.clone().expect("placed primary");
    let time = RationalTime::try_new(1, 1).expect("1s");
    let response = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"add_param_key","#,
                r#""host_handle":"{host}","target":"{layer}","property":"scale","#,
                r#""time":{{"num":1,"den":1}}}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(response.accepted, "reason={:?}", response.reason);
    let snap = response.snapshot.expect("snapshot");
    let layer = snap
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("placed layer");
    assert!(layer.position_keys.is_empty());
    assert_eq!(layer.param_keys.len(), 1);
    assert_eq!(layer.param_keys[0].property, "scale");
    assert!(!layer.param_keys[0].key_id.is_empty());
    assert_eq!(layer.param_keys[0].time, time);
    assert_eq!(layer.param_keys[0].vec, Some([1.0, 1.0]));
    assert_eq!(layer.param_keys[0].value, None);
    let _ = host_destroy_for_test(host);
}

#[test]
fn stage_scale_keyframes_off_key_is_off_keyframe() {
    let (document, layer, _) = keyed_scale_document();
    let off = prepare_app_stage_transform_command(
        &document,
        RationalTime::ZERO,
        layer,
        AppStageTransformEdit::Scale([1.1, 1.1]),
    );
    assert!(matches!(off, Err(AppStageTransformError::OffKeyframe)));
    let on = prepare_app_stage_transform_command(
        &document,
        RationalTime::try_new(1, 1).expect("1s"),
        layer,
        AppStageTransformEdit::Scale([1.1, 1.1]),
    );
    assert!(matches!(
        on,
        Ok(Command::SetProperty {
            property: ScalarPropertyId::Scale,
            ..
        })
    ));
}

#[test]
fn add_position_key_snapshot_carries_document_vec2_value() {
    let _lock = test_lock();
    let host = create_host("timeline-add-key-value");
    let baseline = read_snapshot(host);
    let layer_id = baseline.layer_ids[0].parse::<u64>().expect("layer id");
    let target = LayerId::from_raw(layer_id);
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

    let time = RationalTime::try_new(1, 1).expect("1s");
    let response = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"add_position_key","#,
                r#""host_handle":"{host}","target":"{layer}","time":{{"num":1,"den":1}}}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(response.accepted);
    let snap = response.snapshot.expect("snapshot");
    let key = &snap.timeline.layers[0].position_keys[0];
    assert_eq!(key.time, time);
    assert_eq!(key.value, Some([0.0, 0.0]));
    let (doc_key, doc_value) = with_registry(|registry| {
        let product = registry
            .hosts
            .get(&host)
            .ok_or(RnHostError::UnknownHost(host))?;
        Ok(position_key_at(
            product.runtime.snapshot().as_ref(),
            target,
            time,
        ))
    })
    .expect("doc lookup")
    .expect("doc key");
    assert_eq!(key.key_id, doc_key.get().to_string());
    assert_eq!(key.value, Some(doc_value));
    let _ = host_destroy_for_test(host);
}

#[test]
fn set_position_key_value_updates_wire_value_and_preserves_identity_other_keys() {
    let _lock = test_lock();
    let host = create_host("timeline-set-key-value");
    let baseline = read_snapshot(host);
    let layer_id = baseline.layer_ids[0].parse::<u64>().expect("layer id");
    let target = LayerId::from_raw(layer_id);
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

    let add = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"add_position_key","#,
                r#""host_handle":"{host}","target":"{layer}","time":{{"num":1,"den":2}}}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(add.accepted);
    let before = add.snapshot.expect("before");
    let before_key = before.timeline.layers[0].position_keys[0].clone();
    assert_eq!(before_key.value, Some([0.0, 0.0]));

    let second = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"add_position_key","#,
                r#""host_handle":"{host}","target":"{layer}","time":{{"num":2,"den":1}}}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(second.accepted);
    let before_second_key = second.snapshot.expect("before second").timeline.layers[0]
        .position_keys
        .iter()
        .find(|key| key.key_id != before_key.key_id)
        .cloned()
        .expect("other key");

    let response = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"set_position_key_value","#,
                r#""host_handle":"{host}","target":"{layer}","time":{{"num":2,"den":4}},"#,
                r#""new":[0.25,-0.5]}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(response.accepted);
    let after = response.snapshot.expect("after");
    assert_eq!(after.timeline.layers[0].position_keys.len(), 2);
    let after_key = after.timeline.layers[0]
        .position_keys
        .iter()
        .find(|key| key.key_id == before_key.key_id)
        .expect("target key");
    let after_other_key = after.timeline.layers[0]
        .position_keys
        .iter()
        .find(|key| key.key_id == before_second_key.key_id)
        .expect("other key");
    assert_eq!(after_key.key_id, before_key.key_id);
    assert_eq!(after_key.time, before_key.time);
    assert_eq!(after_key.value, Some([0.25, -0.5]));
    assert_eq!(after_other_key.key_id, before_second_key.key_id);
    assert_eq!(after_other_key.time, before_second_key.time);
    assert_eq!(after_other_key.value, Some([0.0, 0.0]));
    let _ = host_destroy_for_test(host);
}

#[test]
fn timeline_position_keys_cap_at_64_and_mark_truncated() {
    let _lock = test_lock();
    let mut document = Document::new_current();
    let layer = document.layers.allocate("keyed").expect("layer");
    let track = document.track_ids.allocate("track").expect("track");
    let mut keyframes = DocKeyframeTrack::new();
    for i in 0..65 {
        let id = document.next_stable_id.allocate().expect("key id");
        keyframes.insert(DocKeyframe {
            id: KeyframeId::from_raw(id),
            t: RationalTime::try_new(i, 10).expect("key time"),
            value: DocValue::Vec2([0.0, 0.0]),
            interp: Interp::Linear,
        });
    }
    let mut envelope = ItemEnvelope::new(layer);
    envelope.transform.position = DocParam::Keyframes(keyframes);
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
    document.validate().expect("valid keyed document");
    let host = create_host_from_document("timeline-keys-cap", &document);
    let snap = read_snapshot(host);
    let layer = &snap.timeline.layers[0];
    assert_eq!(layer.position_keys.len(), 64);
    assert!(layer.keys_truncated);
    assert_eq!(
        layer.position_keys[0].time,
        RationalTime::try_new(0, 10).expect("first")
    );
    assert_eq!(
        layer.position_keys[63].time,
        RationalTime::try_new(63, 10).expect("64th")
    );
    let wire = read_wire(host);
    assert_eq!(wire.truncated_total, 1);
    let _ = host_destroy_for_test(host);
}

#[test]
fn snapshot_history_flags_project_nothing_to_undo_redo() {
    let _lock = test_lock();
    let host = create_empty_track_host("history-empty");
    let empty = read_wire(host);
    assert!(!empty.history.can_undo);
    assert!(!empty.history.can_redo);
    assert_eq!(empty.truncated_total, 0);

    let placed = dispatch_raw_json(
        host,
        &format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"place_rectangle","host_handle":"{host}","position":[0.0,0.0],"playhead":{{"num":0,"den":1}}}}"#
        ),
    );
    assert!(placed.accepted);
    let after = read_wire(host);
    assert!(after.history.can_undo);
    assert!(!after.history.can_redo);

    let undone = dispatch_raw_json(
        host,
        &format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"undo","host_handle":"{host}"}}"#
        ),
    );
    assert!(undone.accepted);
    let restored = read_wire(host);
    assert!(!restored.history.can_undo);
    assert!(restored.history.can_redo);
    let _ = host_destroy_for_test(host);
}

#[test]
fn snapshot_stage_bounds_and_timeline_layers_share_layer_ids_in_order() {
    let _lock = test_lock();
    let host = create_host("bounds-timeline-alignment");
    let snap = read_snapshot(host);
    let timeline_ids: Vec<String> = snap
        .timeline
        .layers
        .iter()
        .map(|layer| layer.layer_id.clone())
        .collect();
    assert_eq!(snap.layer_ids, timeline_ids);
    let _ = host_destroy_for_test(host);
}

#[test]
fn timeline_and_bounds_follow_track_item_order_not_layer_id_allocation() {
    let _lock = test_lock();
    let mut document = Document::new_current();
    // 採番順: first(=低id) → second。track順は逆(second track0, first track1)。
    let first = document.layers.allocate("first").expect("first");
    let second = document.layers.allocate("second").expect("second");
    let track_a = document.track_ids.allocate("V1").expect("V1");
    let track_b = document.track_ids.allocate("V2").expect("V2");
    let duration = document.composition.duration;
    let mk_clip = |layer: LayerId| {
        TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration,
            time_map: TimeMap::identity(),
            source: ClipSource::Plugin {
                plugin_id: RECT_LAYER_SOURCE.into(),
                effect_version: 1,
                params: rect_params([0.0, 0.0], [0.2, 0.2]),
                extra: Default::default(),
            },
        })
    };
    document.tracks.push(Track {
        id: track_a,
        items: vec![mk_clip(second)],
    });
    document.tracks.push(Track {
        id: track_b,
        items: vec![mk_clip(first)],
    });
    let alloc_order: Vec<_> = document
        .layers
        .iter()
        .map(|(id, _)| id.get().to_string())
        .collect();
    assert_eq!(
        alloc_order,
        vec![first.get().to_string(), second.get().to_string()]
    );

    let host = create_host_from_document("track-order-projection", &document);
    let snap = read_snapshot(host);
    let expected = vec![second.get().to_string(), first.get().to_string()];
    assert_eq!(snap.layer_ids, expected);
    let timeline_ids: Vec<_> = snap
        .timeline
        .layers
        .iter()
        .map(|layer| layer.layer_id.clone())
        .collect();
    assert_eq!(timeline_ids, expected);
    let _ = host_destroy_for_test(host);
}

#[test]
fn snapshot_json_of_16_layers_64_keys_stays_under_the_snapshot_cap_and_untruncated() {
    let _lock = test_lock();
    let document = make_16_layers_64_keys_document();
    let host = create_host_from_document("snapshot-16x64", &document);
    let mut out = vec![0_u8; MAX_SNAPSHOT_JSON_BYTES];
    let written = motolii_rn_host_read_snapshot_json(host, out.as_mut_ptr(), out.len());
    assert!(written > 0);
    assert!((written as usize) < MAX_SNAPSHOT_JSON_BYTES);

    let snapshot: WireProductSnapshot =
        serde_json::from_slice(&out[..written as usize]).expect("snapshot json parse");
    assert_eq!(snapshot.timeline.layers.len(), 16);
    for layer in snapshot.timeline.layers.iter() {
        assert_eq!(layer.position_keys.len(), 64);
        assert!(!layer.keys_truncated);
    }
    let _ = host_destroy_for_test(host);
}

#[test]
fn dispatch_response_with_inflated_layers_keys_effects_fits_snapshot_cap() {
    let _lock = test_lock();
    let document = make_16_layers_64_keys_document();
    let host = create_host_from_document("dispatch-inflated-response", &document);
    let intent = format!(
        concat!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"set_time","#,
            r#""host_handle":"{host}","frame":0}}"#
        ),
        host = host
    );
    let mut out = vec![0_u8; MAX_SNAPSHOT_JSON_BYTES];
    let written = motolii_rn_host_dispatch_intent_json(
        host,
        intent.as_ptr(),
        intent.len(),
        out.as_mut_ptr(),
        out.len(),
    );
    assert!(written > 0, "dispatch inflated response failed: {written}");
    assert!(
        (written as usize) > MAX_JSON_BYTES,
        "oracle requires response larger than intent cap: {}",
        written
    );
    assert!((written as usize) <= MAX_SNAPSHOT_JSON_BYTES);
    let response: WireIntentResponse =
        serde_json::from_slice(&out[..written as usize]).expect("response json");
    assert!(response.accepted);
    assert!(response.snapshot.is_some());
    let _ = host_destroy_for_test(host);
}
