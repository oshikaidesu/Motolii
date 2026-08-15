//! handle / FFI / projection stamp の試験。helper は tests。
use super::tests::*;
use super::*;

#[cfg(target_os = "macos")]
#[test]
fn timeline_registration_borrows_revisioned_document_projection() {
    let _lock = test_lock();
    let mut fixture = Fixture::new();
    let layer = fixture.push_rect_layer(
        "timeline-seat",
        [0.0, 0.0],
        [0.25, 0.25],
        Transform2D::identity(),
    );
    fixture.document.validate().expect("valid");
    let host = create_host_from_document("timeline-seat", &fixture.document);
    let timeline = with_registry(|registry| registry.register_timeline(host)).expect("timeline");

    let frame = with_registry(|registry| {
        let product = registry
            .hosts
            .get(&host)
            .ok_or(RnHostError::UnknownHost(host))?;
        product
            .timeline_frame_borrow()
            .map_err(|_| RnHostError::UnknownTimeline(timeline))
    })
    .expect("frame borrow");
    assert_eq!(frame.revision, 0);
    assert_eq!(frame.projection_generation, 0);
    assert_eq!(
        frame.document.layers.display_name(layer),
        Some("timeline-seat")
    );
    assert_eq!(frame.projection.bars().len(), 1);
    assert_eq!(frame.projection.bars()[0].layer, layer);
    assert_eq!(frame.primary, None);
    assert_eq!(frame.playhead, RationalTime::ZERO);

    with_registry(|registry| registry.destroy_timeline(timeline)).expect("destroy timeline");
    let double = with_registry(|registry| registry.destroy_timeline(timeline)).unwrap_err();
    assert!(matches!(double, RnHostError::DestroyedTimeline(_)));
    host_destroy_for_test(host).expect("destroy host");
}

#[cfg(target_os = "macos")]
#[test]
fn timeline_detach_reuses_surface_binding_lifecycle() {
    let mut timeline = RnTimelineSurface {
        host_handle: 1,
        destroyed: false,
        gpu: StageGpuBinding::detached(),
        raster_key: None,
    };
    timeline.gpu.layer_ptr = 7;
    timeline.gpu.physical_width = 640;
    timeline.gpu.physical_height = 240;
    timeline.gpu_detach_surface();
    assert!(!timeline.gpu.is_attached());
    assert_eq!(timeline.gpu.physical_width, 0);
    assert_eq!(timeline.gpu.physical_height, 0);
    assert_eq!(timeline.gpu.surface_epoch, 1);
}

#[test]
fn unknown_and_destroyed_handles_are_rejected_safely() {
    let _lock = test_lock();
    let host = create_host("handles");
    let err = host_read_snapshot_for_test(9_999).unwrap_err();
    assert!(matches!(err, RnHostError::UnknownHost(9_999)));

    let stage = host_register_stage_for_test(host).expect("stage");
    host_destroy_stage_for_test(stage).expect("destroy");
    let err = host_destroy_stage_for_test(stage).unwrap_err();
    assert!(matches!(err, RnHostError::DestroyedStage(_)));

    host_destroy_for_test(host).expect("destroy host");
    let err = host_destroy_for_test(host).unwrap_err();
    assert!(matches!(err, RnHostError::DestroyedHost(_)));

    let late = base_intent("stage_mount");
    assert!(matches!(
        host_dispatch_intent_for_test(host, late),
        Err(RnHostError::DestroyedHost(_))
    ));
}

#[test]
fn late_lifecycle_event_after_stage_destroy_is_rejected() {
    let _lock = test_lock();
    let host = create_host("late");
    let stage = host_register_stage_for_test(host).expect("stage");
    host_destroy_stage_for_test(stage).expect("destroy");

    let mut intent = base_intent("stage_resize");
    intent.stage_handle = Some(stage);
    intent.width = Some(640);
    intent.height = Some(480);
    let response = dispatch(host, intent);
    assert!(!response.accepted);
    assert_eq!(response.reason, Some(RnHostReasonCode::LateLifecycleEvent));
    let _ = host_destroy_for_test(host);
}

#[test]
fn second_host_and_invalid_path_are_rejected_without_replacing_active_host() {
    let _lock = test_lock();
    let host = create_host("single");
    let second_path = fixture_path("second");
    assert!(matches!(
        host_create_for_test(&second_path),
        Err(RnHostError::HostAlreadyExists)
    ));

    let missing_path = tmp_dir("rn-product-host-missing").join("missing.json");
    assert!(matches!(
        host_create_for_test(&missing_path),
        Err(RnHostError::HostAlreadyExists)
    ));
    assert!(host_read_snapshot_for_test(host).is_ok());
    host_destroy_for_test(host).expect("destroy host");
}

#[cfg(target_os = "macos")]
pub(super) fn parse_wire_response(buf: &[u8], len: i64) -> WireIntentResponse {
    assert!(len > 0);
    let json = std::str::from_utf8(&buf[..len as usize]).expect("utf8");
    serde_json::from_str(json).expect("wire response")
}

#[cfg(target_os = "macos")]
#[test]
fn ffi_create_register_read_destroy_emit_typed_envelopes() {
    let _lock = test_lock();
    let path = fixture_path("ffi-create");
    let path_bytes = path.to_string_lossy();
    let mut host_handle = 0u64;
    let mut out = [0u8; MAX_JSON_BYTES];
    let created = motolii_rn_host_create(
        path_bytes.as_bytes().as_ptr(),
        path_bytes.len(),
        &mut host_handle,
        out.as_mut_ptr(),
        out.len(),
    );
    assert!(created > 0);
    assert_ne!(host_handle, 0);
    let created_response = parse_wire_response(&out, created);
    assert!(created_response.accepted);
    let snapshot = created_response.snapshot.expect("create snapshot");
    assert_eq!(snapshot.host_handle, host_handle.to_string());
    assert_eq!(snapshot.revision, "0");
    assert_eq!(snapshot.projection_generation, "0");

    let mut stage_handle = 0u64;
    let registered =
        motolii_rn_stage_register(host_handle, &mut stage_handle, out.as_mut_ptr(), out.len());
    assert!(registered > 0);
    assert_ne!(stage_handle, 0);
    let registered_response = parse_wire_response(&out, registered);
    assert!(registered_response.accepted);
    assert_eq!(
        registered_response
            .snapshot
            .as_ref()
            .map(|snapshot| snapshot.revision.as_str()),
        Some("0")
    );

    let read = motolii_rn_host_read_snapshot_json(host_handle, out.as_mut_ptr(), out.len());
    assert!(read > 0);
    let read_snapshot: WireProductSnapshot =
        serde_json::from_slice(&out[..read as usize]).expect("read snapshot");
    assert_eq!(read_snapshot.revision, snapshot.revision);
    assert_eq!(
        read_snapshot.projection_generation,
        snapshot.projection_generation
    );

    let destroyed_stage = motolii_rn_stage_destroy(stage_handle, out.as_mut_ptr(), out.len());
    assert!(destroyed_stage > 0);
    let stage_destroy_response = parse_wire_response(&out, destroyed_stage);
    assert!(stage_destroy_response.accepted);
    assert!(stage_destroy_response.snapshot.is_none());

    let destroyed_host = motolii_rn_host_destroy(host_handle, out.as_mut_ptr(), out.len());
    assert!(destroyed_host > 0);
    let host_destroy_response = parse_wire_response(&out, destroyed_host);
    assert!(host_destroy_response.accepted);
    assert!(host_destroy_response.snapshot.is_none());
}

#[cfg(target_os = "macos")]
#[test]
fn ffi_create_reports_project_already_open_as_typed_reject() {
    let _lock = test_lock();
    let path = fixture_path("ffi-project-already-open");
    let limits = ResourceLimits::production();
    let _held = ProjectSession::acquire(&path, &limits).expect("hold project lock");
    let path_bytes = path.to_string_lossy();
    let mut host_handle = 1u64;
    let mut out = [0u8; MAX_JSON_BYTES];

    let written = motolii_rn_host_create(
        path_bytes.as_bytes().as_ptr(),
        path_bytes.len(),
        &mut host_handle,
        out.as_mut_ptr(),
        out.len(),
    );

    assert!(written > 0);
    assert_eq!(host_handle, 0);
    let response = parse_wire_response(&out, written);
    assert!(!response.accepted);
    assert_eq!(
        response.diagnostics[0].reason,
        RnHostReasonCode::ProjectAlreadyOpen
    );
}

#[cfg(target_os = "macos")]
#[test]
fn ffi_rejects_preserve_typed_reasons_and_skip_registry_mutation_on_bad_out() {
    let _lock = test_lock();
    let path = fixture_path("ffi-reject");
    let path_bytes = path.to_string_lossy();
    let mut host_handle = 0u64;
    let mut out = [0u8; MAX_JSON_BYTES];

    let missing = tmp_dir("rn-product-host-ffi-missing").join("missing.json");
    let missing_bytes = missing.to_string_lossy();
    let mut missing_handle = 1u64;
    let missing_result = motolii_rn_host_create(
        missing_bytes.as_bytes().as_ptr(),
        missing_bytes.len(),
        &mut missing_handle,
        out.as_mut_ptr(),
        out.len(),
    );
    assert!(missing_result > 0);
    assert_eq!(missing_handle, 0);
    let missing_response = parse_wire_response(&out, missing_result);
    assert!(!missing_response.accepted);
    assert_eq!(
        missing_response.diagnostics[0].reason,
        RnHostReasonCode::InvalidProjectPath
    );

    let created = motolii_rn_host_create(
        path_bytes.as_bytes().as_ptr(),
        path_bytes.len(),
        &mut host_handle,
        out.as_mut_ptr(),
        out.len(),
    );
    assert!(created > 0);
    assert_ne!(host_handle, 0);

    let mut second_handle = 1u64;
    let second = motolii_rn_host_create(
        path_bytes.as_bytes().as_ptr(),
        path_bytes.len(),
        &mut second_handle,
        out.as_mut_ptr(),
        out.len(),
    );
    assert!(second > 0);
    assert_eq!(second_handle, 0);
    let second_response = parse_wire_response(&out, second);
    assert!(!second_response.accepted);
    assert_eq!(
        second_response.diagnostics[0].reason,
        RnHostReasonCode::HostAlreadyExists
    );
    assert!(motolii_rn_host_read_snapshot_json(host_handle, out.as_mut_ptr(), out.len()) > 0);

    let unknown_read = motolii_rn_host_read_snapshot_json(9_999, out.as_mut_ptr(), out.len());
    assert!(unknown_read > 0);
    let unknown_response = parse_wire_response(&out, unknown_read);
    assert!(!unknown_response.accepted);
    assert_eq!(
        unknown_response.diagnostics[0].reason,
        RnHostReasonCode::UnknownHostHandle
    );

    let mut stage_handle = 0u64;
    let registered =
        motolii_rn_stage_register(host_handle, &mut stage_handle, out.as_mut_ptr(), out.len());
    assert!(registered > 0);
    assert_ne!(stage_handle, 0);
    assert!(motolii_rn_stage_destroy(stage_handle, out.as_mut_ptr(), out.len()) > 0);
    let double_stage = motolii_rn_stage_destroy(stage_handle, out.as_mut_ptr(), out.len());
    assert!(double_stage > 0);
    let double_stage_response = parse_wire_response(&out, double_stage);
    assert!(!double_stage_response.accepted);
    assert_eq!(
        double_stage_response.diagnostics[0].reason,
        RnHostReasonCode::DoubleDestroy
    );

    let unknown_stage = motolii_rn_stage_destroy(42_042, out.as_mut_ptr(), out.len());
    assert!(unknown_stage > 0);
    let unknown_stage_response = parse_wire_response(&out, unknown_stage);
    assert!(!unknown_stage_response.accepted);
    assert_eq!(
        unknown_stage_response.diagnostics[0].reason,
        RnHostReasonCode::UnknownStageHandle
    );

    let null_create = motolii_rn_host_create(
        path_bytes.as_bytes().as_ptr(),
        path_bytes.len(),
        std::ptr::null_mut(),
        out.as_mut_ptr(),
        out.len(),
    );
    assert_eq!(null_create, -1);

    let undersized = motolii_rn_host_create(
        path_bytes.as_bytes().as_ptr(),
        path_bytes.len(),
        &mut second_handle,
        out.as_mut_ptr(),
        1,
    );
    assert!(undersized < 0);
    assert_eq!(second_handle, 0);
    assert!(motolii_rn_host_read_snapshot_json(host_handle, out.as_mut_ptr(), out.len()) > 0);

    assert!(motolii_rn_host_destroy(host_handle, out.as_mut_ptr(), out.len()) > 0);
    let destroyed_read =
        motolii_rn_host_read_snapshot_json(host_handle, out.as_mut_ptr(), out.len());
    assert!(destroyed_read > 0);
    let destroyed_response = parse_wire_response(&out, destroyed_read);
    assert!(!destroyed_response.accepted);
    assert_eq!(
        destroyed_response.diagnostics[0].reason,
        RnHostReasonCode::DestroyedHostHandle
    );
    let double_host = motolii_rn_host_destroy(host_handle, out.as_mut_ptr(), out.len());
    assert!(double_host > 0);
    let double_host_response = parse_wire_response(&out, double_host);
    assert!(!double_host_response.accepted);
    assert_eq!(
        double_host_response.diagnostics[0].reason,
        RnHostReasonCode::DoubleDestroy
    );
}

#[cfg(target_os = "macos")]
pub(super) fn read_projection_stamp(host: u64) -> (u64, u64) {
    let mut revision = 0u64;
    let mut generation = 0u64;
    assert!(
        motolii_rn_host_projection_stamp(host, &mut revision, &mut generation),
        "stamp ffi"
    );
    (revision, generation)
}

#[cfg(target_os = "macos")]
pub(super) fn read_snapshot_json_bytes(host: u64) -> Vec<u8> {
    let mut out = vec![0u8; MAX_SNAPSHOT_JSON_BYTES];
    let written = motolii_rn_host_read_snapshot_json(host, out.as_mut_ptr(), out.len());
    assert!(written > 0, "snapshot read failed: {written}");
    out[..written as usize].to_vec()
}

/// F9: stampはsnapshot JSONが変わり得る全変更で必ず動く。no-opでは不変、stamp不変⇒snapshot不変。
#[cfg(target_os = "macos")]
#[test]
fn projection_stamp_tracks_snapshot_mutating_intents_and_stays_on_noop() {
    let _lock = test_lock();
    let host = create_host("projection-stamp");
    let mut previous_stamp = read_projection_stamp(host);
    let mut previous_json = read_snapshot_json_bytes(host);
    assert_eq!(previous_stamp, (0, 0));

    let mut check_mutating = |name: &str, json: String| {
        let response = dispatch_raw_json(host, &json);
        assert!(response.accepted, "{name} must be accepted");
        let next_stamp = read_projection_stamp(host);
        let next_json = read_snapshot_json_bytes(host);
        assert_ne!(next_stamp, previous_stamp, "{name} should mutate stamp");
        assert_ne!(next_json, previous_json, "{name} should mutate snapshot");
        previous_stamp = next_stamp;
        previous_json = next_json;
        response
    };

    let baseline_wire = read_wire(host);
    let baseline_layer_id = baseline_wire
        .timeline
        .layers
        .first()
        .expect("baseline layer")
        .layer_id
        .clone();

    let _ = check_mutating(
        "set_time",
        format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"set_time","#,
                r#""host_handle":"{host}","frame":12}}"#
            ),
            host = host,
        ),
    );

    let _ = check_mutating(
        "place_rectangle",
        format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"place_rectangle","#,
                r#""host_handle":"{host}","position":[0.0,0.0],"playhead":{{"num":0,"den":1}}}}"#
            ),
            host = host,
        ),
    );

    // move_layer_by用: rect layer(この時点のprimary)を捕まえておく。
    let rect_target = read_wire(host)
        .primary_layer_id
        .expect("primary after place_rectangle");

    let _ = check_mutating(
        "place_ellipse",
        format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"place_ellipse","#,
                r#""host_handle":"{host}","position":[0.0,0.0],"playhead":{{"num":0,"den":1}}}}"#
            ),
            host = host,
        ),
    );

    let vism_source = read_wire(host)
        .catalog
        .sources
        .first()
        .expect("vism source exists")
        .plugin_id
        .clone();
    let _ = check_mutating(
        "place_vism",
        place_vism_json(host, &vism_source, [0.0, 0.0], r#"{"num":0,"den":1}"#),
    );

    // AddPositionKey系はprimary限定(非primaryは拒否)なので、
    // place後の現primary(=配置layer)を対象にする。
    let key_target = read_wire(host)
        .primary_layer_id
        .unwrap_or_else(|| baseline_layer_id.clone());
    // moveはAvailableなConst rectへ(明示targetなのでprimary不要)。
    // key化後のoff-key moveはU4b-0Vでtyped拒否が正仕様。
    let _ = check_mutating(
        "move_layer_by",
        format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"move_layer_by","#,
                r#""host_handle":"{host}","target":"{layer}","delta":[0.1,-0.05]}}"#
            ),
            host = host,
            layer = rect_target,
        ),
    );
    let add_key = check_mutating(
        "add_position_key",
        format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"add_position_key","#,
                r#""host_handle":"{host}","target":"{layer}","time":{{"num":1,"den":1}}}}"#
            ),
            host = host,
            layer = key_target,
        ),
    );
    let key_id = add_key
        .snapshot
        .as_ref()
        .expect("snapshot after add_position_key")
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == key_target)
        .expect("layer for key")
        .position_keys
        .first()
        .expect("added key")
        .key_id
        .clone();
    let _ = check_mutating(
        "set_position_key_value",
        format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"set_position_key_value","#,
                r#""host_handle":"{host}","target":"{layer}","key_id":"{key}","new":[0.25,-0.5],"time":{{"num":1,"den":1}}}}"#
            ),
            host = host,
            layer = key_target,
            key = key_id,
        ),
    );
    let _ = check_mutating(
        "set_position_key_time",
        format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"set_position_key_time","#,
                r#""host_handle":"{host}","target":"{layer}","key_id":"{key}","time":{{"num":2,"den":1}}}}"#
            ),
            host = host,
            layer = key_target,
            key = key_id,
        ),
    );
    let _ = check_mutating(
        "remove_position_key",
        format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"remove_position_key","#,
                r#""host_handle":"{host}","target":"{layer}","key_id":"{key}"}}"#
            ),
            host = host,
            layer = key_target,
            key = key_id,
        ),
    );

    // 配置直後のclipはcomposition一杯なので、先にtrimして縮めてからstartを動かす。
    let _ = check_mutating(
        "trim_clip_in",
        format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"trim_clip_in","#,
                r#""host_handle":"{host}","target":"{layer}","time":{{"num":1,"den":1}}}}"#
            ),
            host = host,
            layer = key_target,
        ),
    );
    let _ = check_mutating(
        "set_clip_start",
        format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"set_clip_start","#,
                r#""host_handle":"{host}","target":"{layer}","time":{{"num":1,"den":2}}}}"#
            ),
            host = host,
            layer = key_target,
        ),
    );

    let select_target = read_wire(host)
        .timeline
        .layers
        .into_iter()
        .map(|layer| layer.layer_id)
        .find(|id| id != &key_target)
        .unwrap_or_else(|| key_target.clone());
    let _ = check_mutating(
        "select_layer",
        format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"select_layer","#,
                r#""host_handle":"{host}","target":"{target}"}}"#
            ),
            host = host,
            target = select_target,
        ),
    );
    let _ = check_mutating(
        "clear_selection",
        format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"clear_selection","host_handle":"{host}"}}"#,
            host = host,
        ),
    );
    let _ = check_mutating(
        "delete_layer",
        format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"delete_layer","#,
                r#""host_handle":"{host}","target":"{target}"}}"#
            ),
            host = host,
            target = select_target,
        ),
    );

    let effect_target = read_wire(host)
        .primary_layer_id
        .or_else(|| {
            read_wire(host)
                .timeline
                .layers
                .first()
                .map(|layer| layer.layer_id.clone())
        })
        .unwrap_or_else(|| baseline_layer_id.clone());
    seed_primary(
        host,
        LayerId::from_raw(effect_target.parse::<u64>().expect("layer id")),
    );
    let _ = check_mutating(
        "attach_effect",
        format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"attach_effect","#,
                r#""host_handle":"{host}","target":"{layer}","plugin_id":"core.filter.opacity"}}"#
            ),
            host = host,
            layer = effect_target,
        ),
    );
    let effect_use_id = layer_effects(&read_wire(host), &effect_target)[0]
        .effect_use_id
        .clone();
    let _ = check_mutating(
        "set_effect_param",
        format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"set_effect_param","#,
                r#""host_handle":"{host}","target":"{layer}","effect_use_id":"{effect}","param_id":"amount","value":0.4}}"#
            ),
            host = host,
            layer = effect_target,
            effect = effect_use_id,
        ),
    );

    let _ = check_mutating(
        "undo",
        format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"undo","host_handle":"{host}"}}"#,
            host = host,
        ),
    );
    let _ = check_mutating(
        "redo",
        format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"redo","host_handle":"{host}"}}"#,
            host = host,
        ),
    );

    // no-op: dispatchなし2回読んでもstamp/snapshot不変
    assert_eq!(read_projection_stamp(host), previous_stamp);
    assert_eq!(read_snapshot_json_bytes(host), previous_json);
    assert_eq!(read_projection_stamp(host), previous_stamp);
    assert_eq!(read_snapshot_json_bytes(host), previous_json);

    let _ = host_destroy_for_test(host);
}
