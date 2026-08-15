//! set_time / snapshot / lifecycle の試験。helper は tests。
use super::tests::*;
use super::*;

#[test]
fn snapshot_carries_revision_projection_generation_and_primary_layer_id() {
    let _lock = test_lock();
    let host = create_host("snapshot");
    let snapshot = read_snapshot(host);
    assert_eq!(snapshot.revision, "0");
    assert_eq!(snapshot.projection_generation, "0");
    assert_eq!(snapshot.current_time, RationalTime::ZERO);
    assert!(snapshot.primary_layer_id.is_none());
    assert!(!snapshot.layer_ids.is_empty());
    let _ = host_destroy_for_test(host);
}

#[test]
fn create_host_opens_the_project_runtime_without_seeding_a_fixture_document() {
    let source = include_str!("rn_product_host.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();
    let start = production
        .find("fn create_host(&mut self, project_path: &Path)")
        .expect("create_host");
    let body = &production[start..];
    let end = body[1..]
        .find("\n    fn ")
        .map(|index| index + 1)
        .unwrap_or(body.len());
    let create = &body[..end];
    assert!(create.contains("open_project_runtime(project_path)"));
    assert!(!create.contains("RECT_LAYER_SOURCE"));
    assert!(!create.contains("Document::new_current"));
}

#[test]
fn document_change_puts_the_same_live_snapshot_on_dispatch_and_read_mouths() {
    let _lock = test_lock();
    let host = create_host("snapshot-mouths");
    let via_read = read_snapshot(host);
    let via_dispatch = dispatch(host, base_intent("read_snapshot"))
        .snapshot
        .expect("read_snapshot kind");
    assert_eq!(via_read.revision, via_dispatch.revision);
    assert_eq!(via_read.layer_ids, via_dispatch.layer_ids);

    let before_layers = via_read.layer_ids.len();
    let response = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"place_rectangle","#,
                r#""host_handle":"{host}","position":[0.0,0.0],"playhead":{{"num":0,"den":1}}}}"#
            ),
            host = host,
        ),
    );
    assert!(response.accepted);
    let dispatched = response.snapshot.expect("place snapshot");
    let reread = read_snapshot(host);
    assert_eq!(dispatched.revision, reread.revision);
    assert_eq!(dispatched.layer_ids, reread.layer_ids);
    assert!(reread.layer_ids.len() > before_layers);
    let _ = host_destroy_for_test(host);
}

#[test]
fn set_time_accepts_valid_frame_and_advances_projection_generation() {
    let _lock = test_lock();
    let host = create_host("set-time-accept");
    let baseline = read_snapshot(host);
    assert_eq!(baseline.current_time, RationalTime::ZERO);
    assert_eq!(baseline.projection_generation, "0");

    // 既定 Composition は 30fps・duration 10s。frame 45 → 45/30 = 3/2。
    let response = dispatch_raw_json(host, &set_time_json(host, "45"));
    assert!(response.accepted);
    let snap = response.snapshot.expect("snapshot");
    assert_eq!(snap.current_time, RationalTime::try_new(3, 2).expect("3/2"));
    assert_eq!(snap.projection_generation, "1");
    assert_eq!(snap.revision, baseline.revision);
    assert_eq!(snap.primary_layer_id, baseline.primary_layer_id);

    let after = read_snapshot(host);
    assert_eq!(after.current_time, snap.current_time);
    assert_eq!(after.projection_generation, "1");
    assert_eq!(after.revision, baseline.revision);
    assert_eq!(after.primary_layer_id, baseline.primary_layer_id);

    let _ = host_destroy_for_test(host);
}

#[test]
fn set_time_frame_zero_resolves_to_rational_time_zero_via_ffi_json() {
    let _lock = test_lock();
    let host = create_host("set-time-zero");
    // いったん非 ZERO にしてから frame 0 へ戻し、解決結果を観測する。
    assert!(dispatch_raw_json(host, &set_time_json(host, "30")).accepted);
    let response = dispatch_raw_json(host, &set_time_json(host, "0"));
    assert!(response.accepted);
    let snap = response.snapshot.expect("snapshot");
    assert_eq!(snap.current_time, RationalTime::ZERO);
    let _ = host_destroy_for_test(host);
}

#[test]
fn set_time_ntsc_frame_is_exact_fraction_via_ffi_json() {
    let _lock = test_lock();
    let fps = motolii_core::Fps::try_new(30_000, 1_001).expect("29.97");
    let host = create_host_with_fps("set-time-ntsc", fps);
    // duration 10s 内に収まる N。N*1001/30000 を十進近似なしで観測する。
    let frame = 100i64;
    let response = dispatch_raw_json(host, &set_time_json(host, &frame.to_string()));
    assert!(response.accepted);
    let snap = response.snapshot.expect("snapshot");
    assert_eq!(
        snap.current_time,
        RationalTime::try_new(frame * 1_001, 30_000).expect("exact ntsc")
    );
    let _ = host_destroy_for_test(host);
}

#[test]
fn set_time_film_24_frame_is_exact_fraction_via_ffi_json() {
    let _lock = test_lock();
    let fps = motolii_core::Fps::try_new(24, 1).expect("24");
    let host = create_host_with_fps("set-time-24", fps);
    let frame = 48i64;
    let response = dispatch_raw_json(host, &set_time_json(host, &frame.to_string()));
    assert!(response.accepted);
    let snap = response.snapshot.expect("snapshot");
    assert_eq!(
        snap.current_time,
        RationalTime::try_new(frame, 24).expect("exact 24")
    );
    let _ = host_destroy_for_test(host);
}

#[test]
fn set_time_same_frame_is_noop_without_generation_advance() {
    let _lock = test_lock();
    let host = create_host("set-time-noop");
    assert!(dispatch_raw_json(host, &set_time_json(host, "60")).accepted);
    let after_first = read_snapshot(host);
    assert_eq!(after_first.projection_generation, "1");

    let noop = dispatch_raw_json(host, &set_time_json(host, "60"));
    assert!(noop.accepted);
    let snap = noop.snapshot.expect("snapshot");
    assert_eq!(snap.current_time, after_first.current_time);
    assert_eq!(
        snap.projection_generation,
        after_first.projection_generation
    );
    assert_eq!(snap.revision, after_first.revision);
    assert_eq!(snap.primary_layer_id, after_first.primary_layer_id);

    let _ = host_destroy_for_test(host);
}

#[test]
fn set_time_rejects_out_of_bounds_and_bad_wire_without_clamp_or_document_write() {
    let _lock = test_lock();
    let host = create_host("set-time-reject");
    let baseline = read_snapshot(host);

    let negative = dispatch_raw_json(host, &set_time_json(host, "-1"));
    assert!(!negative.accepted);
    assert_eq!(negative.reason, Some(RnHostReasonCode::InvalidIntent));

    // duration 10s / 30fps → frame 300 が境界。301 は超過。
    let over = dispatch_raw_json(host, &set_time_json(host, "301"));
    assert!(!over.accepted);
    assert_eq!(over.reason, Some(RnHostReasonCode::InvalidIntent));

    let missing = dispatch_raw_json(
        host,
        &format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"set_time","host_handle":"{host}"}}"#
        ),
    );
    assert!(!missing.accepted);
    assert_eq!(missing.reason, Some(RnHostReasonCode::InvalidIntent));

    let non_integer = dispatch_raw_json(host, &set_time_json(host, "1.5"));
    assert!(!non_integer.accepted);
    assert_eq!(non_integer.reason, Some(RnHostReasonCode::InvalidIntent));

    let legacy_time = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"set_time","#,
                r#""host_handle":"{host}","time":1.5}}"#
            ),
            host = host
        ),
    );
    assert!(!legacy_time.accepted);
    assert_eq!(legacy_time.reason, Some(RnHostReasonCode::InvalidIntent));

    let legacy_time_with_frame = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"set_time","#,
                r#""host_handle":"{host}","frame":1,"time":1.5}}"#
            ),
            host = host
        ),
    );
    assert!(!legacy_time_with_frame.accepted);
    assert_eq!(
        legacy_time_with_frame.reason,
        Some(RnHostReasonCode::InvalidIntent)
    );

    let after = read_snapshot(host);
    assert_eq!(after.current_time, RationalTime::ZERO);
    assert_eq!(after.revision, baseline.revision);
    assert_eq!(after.projection_generation, baseline.projection_generation);
    assert_eq!(after.primary_layer_id, baseline.primary_layer_id);
    assert_eq!(after.layer_ids, baseline.layer_ids);

    let _ = host_destroy_for_test(host);
}

#[test]
fn set_time_rejects_try_from_frame_overflow_without_panic() {
    let _lock = test_lock();
    // fps=1/2 かつ frame=i64::MAX だと try_from_frame が Overflow になる。
    let fps = motolii_core::Fps::try_new(1, 2).expect("1/2");
    let host = create_host_with_fps("set-time-overflow", fps);
    let baseline = read_snapshot(host);
    let rejected = dispatch_raw_json(host, &set_time_json(host, &i64::MAX.to_string()));
    assert!(!rejected.accepted);
    assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));
    let after = read_snapshot(host);
    assert_eq!(after.current_time, baseline.current_time);
    assert_eq!(after.projection_generation, baseline.projection_generation);
    assert_eq!(after.revision, baseline.revision);
    let _ = host_destroy_for_test(host);
}

#[test]
fn set_time_rejects_projection_generation_exhaustion_without_saturation() {
    let _lock = test_lock();
    let host = create_host("set-time-gen-exhaust");
    with_registry(|registry| {
        let product = registry
            .hosts
            .get_mut(&host)
            .ok_or(RnHostError::UnknownHost(host))?;
        product.projection_generation = u64::MAX;
        Ok(())
    })
    .expect("force exhaustion");

    // 同一 ZERO は no-op で受理し、枯渇でも generation を触らない。
    let noop = dispatch_raw_json(host, &set_time_json(host, "0"));
    assert!(noop.accepted);
    assert_eq!(
        noop.snapshot.expect("snapshot").projection_generation,
        u64::MAX.to_string()
    );

    // 異なる frame は前進不能なので typed 拒否。飽和させない。
    let rejected = dispatch_raw_json(host, &set_time_json(host, "1"));
    assert!(!rejected.accepted);
    assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));
    let after = read_snapshot(host);
    assert_eq!(after.current_time, RationalTime::ZERO);
    assert_eq!(after.projection_generation, u64::MAX.to_string());

    let _ = host_destroy_for_test(host);
}

#[test]
fn lifecycle_sequence_preserves_revision_and_projection_generation() {
    let _lock = test_lock();
    let host = create_host("lifecycle");
    let baseline = read_snapshot(host);
    let stage = host_register_stage_for_test(host).expect("stage");

    let mut intent = base_intent("stage_mount");
    intent.stage_handle = Some(stage);
    let mounted = dispatch(host, intent);
    assert!(mounted.accepted);

    let mut resize = base_intent("stage_resize");
    resize.stage_handle = Some(stage);
    resize.width = Some(1280);
    resize.height = Some(720);
    resize.scale_factor = Some(2.0);
    let resized = dispatch(host, resize);
    assert!(resized.accepted);

    let mut focus = base_intent("stage_focus");
    focus.stage_handle = Some(stage);
    focus.focused = Some(true);
    let focused = dispatch(host, focus);
    assert!(focused.accepted);

    let mut unmount = base_intent("stage_unmount");
    unmount.stage_handle = Some(stage);
    let unmounted = dispatch(host, unmount);
    assert!(unmounted.accepted);

    let mut remount = base_intent("stage_mount");
    remount.stage_handle = Some(stage);
    let remounted = dispatch(host, remount);
    assert!(remounted.accepted);

    let after = read_snapshot(host);
    assert_eq!(after.revision, baseline.revision);
    assert_eq!(after.projection_generation, baseline.projection_generation);
    assert_eq!(after.primary_layer_id, baseline.primary_layer_id);
    assert_eq!(after.layer_ids, baseline.layer_ids);

    let _ = host_destroy_stage_for_test(stage);
    let _ = host_destroy_for_test(host);
}
