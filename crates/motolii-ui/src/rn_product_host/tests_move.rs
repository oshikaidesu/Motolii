//! move_layer_by の試験。helper は tests。
use super::tests::*;
use super::*;

fn position_const_at(document: &Document, target: LayerId) -> Option<[f64; 2]> {
    let envelope = find_envelope_in_document(document, target)?;
    match &envelope.transform.position {
        DocParam::Const(DocValue::Vec2(value)) => Some(*value),
        _ => None,
    }
}

#[test]
fn move_layer_by_const_updates_position_and_undo_restores() {
    let _lock = test_lock();
    let host = create_host("move-const");
    let baseline = read_snapshot(host);
    let layer_id = baseline.layer_ids[0].parse::<u64>().expect("layer id");
    let target = LayerId::from_raw(layer_id);
    let before = with_registry(|registry| {
        let product = registry
            .hosts
            .get(&host)
            .ok_or(RnHostError::UnknownHost(host))?;
        Ok(position_const_at(
            product.runtime.snapshot().as_ref(),
            target,
        ))
    })
    .expect("lookup")
    .expect("const position");

    let delta = [0.1, -0.05];
    let response = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"move_layer_by","#,
                r#""host_handle":"{host}","target":"{layer}","delta":[0.1,-0.05]}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(response.accepted);
    let after = with_registry(|registry| {
        let product = registry
            .hosts
            .get(&host)
            .ok_or(RnHostError::UnknownHost(host))?;
        Ok(position_const_at(
            product.runtime.snapshot().as_ref(),
            target,
        ))
    })
    .expect("lookup")
    .expect("const position");
    assert!((after[0] - (before[0] + delta[0])).abs() < 1e-12);
    assert!((after[1] - (before[1] + delta[1])).abs() < 1e-12);

    let undone = dispatch_raw_json(
        host,
        &format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"undo","host_handle":"{host}"}}"#
        ),
    );
    assert!(undone.accepted);
    let restored = with_registry(|registry| {
        let product = registry
            .hosts
            .get(&host)
            .ok_or(RnHostError::UnknownHost(host))?;
        Ok(position_const_at(
            product.runtime.snapshot().as_ref(),
            target,
        ))
    })
    .expect("lookup")
    .expect("const position");
    assert_eq!(restored, before);
    let _ = host_destroy_for_test(host);
}

#[test]
fn move_layer_by_exact_on_key_updates_only_that_key_value_and_off_key_rejects() {
    let _lock = test_lock();
    let host = create_host("move-on-key");
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

    assert!(
        dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"add_position_key","#,
                    r#""host_handle":"{host}","target":"{layer}","time":{{"num":0,"den":1}}}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        )
        .accepted
    );
    assert!(
        dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"add_position_key","#,
                    r#""host_handle":"{host}","target":"{layer}","time":{{"num":2,"den":1}}}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        )
        .accepted
    );
    let before = read_snapshot(host);
    let before_keys = before.timeline.layers[0].position_keys.clone();
    assert_eq!(before_keys.len(), 2);
    let on_key_id = before_keys[0].key_id.clone();
    let before_doc = document_json_bytes(host);

    // current_time は seed で 0。exact-on-key。
    let moved = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"move_layer_by","#,
                r#""host_handle":"{host}","target":"{layer}","delta":[0.2,0.1]}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(moved.accepted);
    let after = moved.snapshot.expect("after");
    let after_keys = &after.timeline.layers[0].position_keys;
    assert_eq!(after_keys.len(), 2);
    let mut before_keys_sorted = before_keys.to_vec();
    before_keys_sorted.sort_by_key(|key| key.key_id.clone());
    let mut after_keys = after_keys.to_vec();
    after_keys.sort_by_key(|key| key.key_id.clone());
    for before_key in before_keys_sorted {
        let after_key = after_keys
            .iter()
            .find(|key| key.key_id == before_key.key_id)
            .expect("all keys preserved");
        if before_key.key_id == on_key_id {
            assert_eq!(after_key.key_id, before_key.key_id);
            assert_eq!(after_key.time, before_key.time);
            assert_eq!(after_key.value, Some([0.2, 0.1]));
        } else {
            assert_eq!(after_key, &before_key);
        }
    }
    let after_on = after_keys
        .iter()
        .find(|key| key.key_id == on_key_id)
        .expect("on key");
    assert_eq!(after_on.key_id, on_key_id);
    assert_eq!(after_on.value, Some([0.2, 0.1]));

    // off-key: frame へ進めて拒否。Document 不変。
    assert!(dispatch_raw_json(host, &set_time_json(host, "15")).accepted);
    let before_off = document_json_bytes(host);
    let rejected = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"move_layer_by","#,
                r#""host_handle":"{host}","target":"{layer}","delta":[0.05,0.0]}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(!rejected.accepted);
    assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));
    assert_eq!(document_json_bytes(host), before_off);
    let _ = before_doc;
    let _ = host_destroy_for_test(host);
}

#[test]
fn set_time_accepts_top_level_host_handle_when_nested_host_handle_appears_first() {
    let _lock = test_lock();
    let host = create_host("host-handle-nested-then-top-level");
    let nested = 999_999_u64;
    let mut intent = serde_json::Map::<String, serde_json::Value>::new();
    intent.insert(
        "nested".into(),
        serde_json::json!({ "host_handle": nested.to_string() }),
    );
    intent.insert("version".into(), serde_json::json!(1));
    intent.insert("direction".into(), serde_json::json!("rn-to-host"));
    intent.insert("kind".into(), serde_json::json!("set_time"));
    intent.insert("frame".into(), serde_json::json!(0));
    intent.insert("host_handle".into(), serde_json::json!(host.to_string()));
    let response = dispatch_raw_json(
        host,
        &serde_json::to_string(&serde_json::Value::Object(intent)).expect("intent json"),
    );
    assert!(response.accepted);
    let _ = host_destroy_for_test(host);
}

#[test]
fn move_layer_by_rotated_scaled_layer_uses_world_inverse_delta() {
    let _lock = test_lock();
    let mut fixture = Fixture::new();
    let layer = fixture.push_rect_layer(
        "rotScaled",
        [0.0, 0.0],
        [0.4, 0.4],
        Transform2D {
            position: DocParam::const_vec2([0.1, -0.08]),
            rotation: DocParam::const_f64(0.55),
            scale: DocParam::const_vec2([1.25, 0.8]),
            ..Transform2D::identity()
        },
    );
    fixture.document.validate().expect("valid");
    let host = create_host_from_document("move-rot-scale", &fixture.document);
    let layer_id = layer.get().to_string();

    let tracks = DataTracks::new();
    let projection = project_stage_geometry(
        &fixture.document,
        EvaluationTime::new(RationalTime::ZERO),
        &tracks,
    )
    .expect("geometry");
    let crate::stage_geometry_projection::StageLayerProjection::Available(geo) =
        projection.get(layer).expect("layer")
    else {
        panic!("available");
    };

    let delta = [0.18, -0.12];
    let expected_local =
        world_delta_to_position_local(geo.world, delta).expect("local inverse exists");
    let before = with_registry(|registry| {
        let product = registry
            .hosts
            .get(&host)
            .ok_or(RnHostError::UnknownHost(host))?;
        Ok(position_const_at(
            product.runtime.snapshot().as_ref(),
            LayerId::from_raw(layer_id.parse::<u64>().expect("id")),
        ))
    })
    .expect("lookup")
    .expect("const position");

    let moved = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"move_layer_by","#,
                r#""host_handle":"{host}","target":"{layer}","delta":[{dx},{dy}]}}"#
            ),
            host = host,
            layer = layer_id,
            dx = delta[0],
            dy = delta[1],
        ),
    );
    assert!(moved.accepted);
    let after = with_registry(|registry| {
        let product = registry
            .hosts
            .get(&host)
            .ok_or(RnHostError::UnknownHost(host))?;
        Ok(position_const_at(
            product.runtime.snapshot().as_ref(),
            LayerId::from_raw(layer_id.parse::<u64>().expect("id")),
        ))
    })
    .expect("lookup")
    .expect("const position");
    assert!((after[0] - (before[0] + expected_local[0])).abs() < 1e-12);
    assert!((after[1] - (before[1] + expected_local[1])).abs() < 1e-12);

    let _ = host_destroy_for_test(host);
}

#[test]
fn move_layer_by_rejects_non_finite_delta_and_missing_target() {
    let _lock = test_lock();
    let host = create_host("move-reject");
    let baseline = read_snapshot(host);
    let layer_id = baseline.layer_ids[0].clone();
    let before = document_json_bytes(host);

    let missing = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"move_layer_by","#,
                r#""host_handle":"{host}","target":"999999","delta":[0.1,0.0]}}"#
            ),
            host = host,
        ),
    );
    assert!(!missing.accepted);
    assert_eq!(missing.reason, Some(RnHostReasonCode::InvalidIntent));
    assert_eq!(document_json_bytes(host), before);

    let mut intent = WireIntentEnvelope {
        version: WIRE_VERSION,
        direction: RN_TO_HOST.to_owned(),
        kind: "move_layer_by".to_owned(),
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
        target: Some(layer_id),
        dest: None,
        key_id: None,
        property: None,
        time: None,
        new: None,
        interp: None,
        delta: Some([f64::INFINITY, 0.0]),
        plugin_id: None,
        item_id: None,
        effect_use_id: None,
        param_id: None,
        value: None,
        output_path: None,
        color: None,
    };
    let non_finite = dispatch_wire(host, intent.clone());
    assert!(!non_finite.accepted);
    assert_eq!(non_finite.reason, Some(RnHostReasonCode::InvalidIntent));
    assert_eq!(document_json_bytes(host), before);

    intent.delta = Some([0.0, f64::NAN]);
    let nan = dispatch_wire(host, intent);
    assert!(!nan.accepted);
    assert_eq!(document_json_bytes(host), before);
    let _ = host_destroy_for_test(host);
}
