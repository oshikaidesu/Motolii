//! stage_pointer 選択の試験。helper は tests。
use super::tests::*;
use super::*;

#[test]
fn stage_pointer_phases_record_transient_without_document_write() {
    let _lock = test_lock();
    let host = create_host("pointer-accept");
    let baseline = read_snapshot(host);
    let stage = host_register_stage_for_test(host).expect("stage");
    // resize 前は width/height 0 で selection が typed 拒否されるため、先に非正方形へ拡げる。
    mount_and_resize(host, stage, 1600, 900);
    let before_bytes = document_json_bytes(host);

    // 既定 Rect(半幅0.5)の外。down は Miss→clear no-op で generation / Document 不変。
    let phases = [
        ("down", 12.5, 34.0, 1_u64),
        ("drag", 18.0, 40.25, 2),
        ("up", 20.0, 41.0, 3),
        ("cancel", 1.0, 1.0, 4),
    ];
    for (phase, x, y, sequence) in phases {
        let response = dispatch_wire(host, pointer_intent(stage, phase, x, y, sequence));
        assert!(response.accepted, "phase {phase} should accept");
        let recorded = read_stage_pointer(stage).expect("transient pointer");
        assert_eq!(recorded.phase, phase);
        assert_eq!(recorded.view_local_x, x);
        assert_eq!(recorded.view_local_y, y);
        assert_eq!(recorded.sequence, sequence);
        let snap = response.snapshot.expect("snapshot");
        assert_eq!(snap.revision, baseline.revision);
        assert_eq!(snap.projection_generation, baseline.projection_generation);
        assert_eq!(snap.primary_layer_id, baseline.primary_layer_id);
    }

    let after = read_snapshot(host);
    assert_eq!(after.revision, baseline.revision);
    assert_eq!(after.projection_generation, baseline.projection_generation);
    assert_eq!(after.primary_layer_id, baseline.primary_layer_id);
    assert_eq!(after.layer_ids, baseline.layer_ids);
    assert_eq!(document_json_bytes(host), before_bytes);

    let _ = host_destroy_stage_for_test(stage);
    let _ = host_destroy_for_test(host);
}

#[test]
fn stage_pointer_rejects_invalid_payload_and_late_events() {
    let _lock = test_lock();
    let host = create_host("pointer-reject");
    let baseline = read_snapshot(host);
    let stage = host_register_stage_for_test(host).expect("stage");

    let mut mount = base_intent("stage_mount");
    mount.stage_handle = Some(stage);
    assert!(dispatch(host, mount).accepted);

    let mut unknown_phase = pointer_intent(stage, "move", 1.0, 2.0, 1);
    unknown_phase.phase = Some("move".to_owned());
    let rejected = dispatch_wire(host, unknown_phase);
    assert!(!rejected.accepted);
    assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));

    let mut non_finite = pointer_intent(stage, "down", f64::NAN, 2.0, 2);
    let rejected = dispatch_wire(host, non_finite);
    assert!(!rejected.accepted);
    assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));

    non_finite = pointer_intent(stage, "down", 1.0, f64::INFINITY, 3);
    let rejected = dispatch_wire(host, non_finite);
    assert!(!rejected.accepted);
    assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));

    let mut missing_sequence = pointer_intent(stage, "down", 1.0, 2.0, 4);
    missing_sequence.sequence = None;
    let rejected = dispatch_wire(host, missing_sequence);
    assert!(!rejected.accepted);
    assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));

    let unknown_stage = dispatch_wire(host, pointer_intent(99_999, "down", 1.0, 2.0, 5));
    assert!(!unknown_stage.accepted);
    assert_eq!(
        unknown_stage.reason,
        Some(RnHostReasonCode::UnknownStageHandle)
    );

    let mut unmount = base_intent("stage_unmount");
    unmount.stage_handle = Some(stage);
    assert!(dispatch(host, unmount).accepted);
    let late = dispatch_wire(host, pointer_intent(stage, "up", 1.0, 2.0, 6));
    assert!(!late.accepted);
    assert_eq!(late.reason, Some(RnHostReasonCode::LateLifecycleEvent));

    let after = read_snapshot(host);
    assert_eq!(after.revision, baseline.revision);
    assert_eq!(after.projection_generation, baseline.projection_generation);
    assert_eq!(after.primary_layer_id, baseline.primary_layer_id);

    let _ = host_destroy_stage_for_test(stage);
    let _ = host_destroy_for_test(host);
}

#[test]
fn stale_projection_generation_is_zero_write() {
    let _lock = test_lock();
    let host = create_host("stale");
    let before = read_snapshot(host);
    let mut intent = base_intent("read_snapshot");
    intent.projection_generation = Some("99".to_owned());
    let response = dispatch(host, intent);
    assert!(!response.accepted);
    assert_eq!(
        response.reason,
        Some(RnHostReasonCode::StaleProjectionGeneration)
    );
    let after = read_snapshot(host);
    assert_eq!(after.revision, before.revision);
    assert_eq!(after.projection_generation, before.projection_generation);
    let _ = host_destroy_for_test(host);
}

#[test]
fn stage_pointer_down_selects_rotated_rect_via_json_snapshot() {
    let _lock = test_lock();
    let mut fixture = Fixture::new();
    let layer = fixture.push_rect_layer(
        "rotated",
        [0.05, -0.08],
        [0.35, 0.22],
        Transform2D {
            position: DocParam::const_vec2([0.18, 0.12]),
            rotation: DocParam::const_f64(0.55),
            scale: DocParam::const_vec2([1.15, 0.9]),
            ..Transform2D::identity()
        },
    );
    fixture.document.composition.camera = CompCameraDoc::PlanarOrthographic {
        center: DocParam::const_vec2([0.03, -0.02]),
        roll_radians: DocParam::const_f64(0.2),
        height: DocParam::const_f64(1.0),
    };
    fixture.document.validate().expect("valid");
    let host = create_host_from_document("sel-rotated", &fixture.document);
    let stage = host_register_stage_for_test(host).expect("stage");
    mount_and_resize(host, stage, 1600, 900);
    let before_bytes = document_json_bytes(host);
    let before = read_snapshot(host);

    // 局所原点付近を camera∘world で正準へ写し、非対称な view-local へ戻す。
    let tracks = DataTracks::new();
    let proj = project_stage_geometry(
        &fixture.document,
        EvaluationTime::new(RationalTime::ZERO),
        &tracks,
    )
    .expect("geometry");
    let crate::stage_geometry_projection::StageLayerProjection::Available(geo) =
        proj.get(layer).expect("layer")
    else {
        panic!("available");
    };
    let composed = geo.camera_view * geo.world;
    let [cx, cy] = composed.transform_point(geo.local_rect.center.x, geo.local_rect.center.y);
    let (vx, vy) = canonical_to_view_local(cx, cy, 1600, 900);

    let response = dispatch_raw_json(host, &pointer_json(host, stage, "down", vx, vy, 1));
    assert!(response.accepted);
    let snap = response.snapshot.expect("snapshot");
    assert_eq!(snap.primary_layer_id, Some(layer.get().to_string()));
    assert_eq!(snap.projection_generation, "1");
    assert_eq!(snap.revision, before.revision);
    assert_eq!(document_json_bytes(host), before_bytes);

    let _ = host_destroy_stage_for_test(stage);
    let _ = host_destroy_for_test(host);
}

#[test]
fn stage_pointer_down_x_uses_height_denominator_on_portrait_stage() {
    let _lock = test_lock();
    // h>w でないと /height hit かつ /width miss が作れない。
    let mut fixture = Fixture::new();
    let layer = fixture.push_rect_layer(
        "x-height",
        [0.0, 0.0],
        [0.7, 0.7],
        Transform2D {
            // 非 identity（平行移動）。逆写像を無視すると中心がずれる。
            position: DocParam::const_vec2([0.0, 0.05]),
            ..Transform2D::identity()
        },
    );
    fixture.document.validate().expect("valid");
    let host = create_host_from_document("sel-x-height", &fixture.document);
    let stage = host_register_stage_for_test(host).expect("stage");
    let (width, height) = (900_u32, 1600_u32);
    mount_and_resize(host, stage, width, height);

    // local x=0.25 → 正準 x=0.25。/height は半幅0.35内、/width なら ≈0.444 で外れ。
    let (vx, vy) = canonical_to_view_local(0.25, 0.05, width, height);
    assert!(vx >= 0.0 && vx <= f64::from(width));
    let wrong_x = (vx - f64::from(width) * 0.5) / f64::from(width);
    assert!(
        wrong_x.abs() > 0.35,
        "oracle requires /width miss: wrong_x={wrong_x}"
    );

    let response = dispatch_raw_json(host, &pointer_json(host, stage, "down", vx, vy, 1));
    assert!(response.accepted);
    assert_eq!(
        response.snapshot.expect("snapshot").primary_layer_id,
        Some(layer.get().to_string())
    );

    let _ = host_destroy_stage_for_test(stage);
    let _ = host_destroy_for_test(host);
}

#[test]
fn stage_pointer_down_scale_factor_oracle_preserves_logical_hit_target() {
    let _lock = test_lock();
    let mut fixture = Fixture::new();
    let layer = fixture.push_rect_layer("base", [0.0, 0.0], [0.5, 0.5], Transform2D::identity());
    fixture.document.validate().expect("valid");
    let host = create_host_from_document("sel-scale-factor", &fixture.document);
    let stage = host_register_stage_for_test(host).expect("stage");

    let mut mount = base_intent("stage_mount");
    mount.stage_handle = Some(stage);
    assert!(dispatch(host, mount).accepted);

    let mut resize = base_intent("stage_resize");
    resize.stage_handle = Some(stage);
    resize.width = Some(1600);
    resize.height = Some(900);
    resize.scale_factor = Some(1.0);
    assert!(dispatch(host, resize).accepted);

    let tracks = DataTracks::new();
    let proj = project_stage_geometry(
        &fixture.document,
        EvaluationTime::new(RationalTime::ZERO),
        &tracks,
    )
    .expect("geometry");
    let crate::stage_geometry_projection::StageLayerProjection::Available(geo) =
        proj.get(layer).expect("layer")
    else {
        panic!("available");
    };
    let [cx, cy] = (geo.camera_view * geo.world)
        .transform_point(geo.local_rect.center.x, geo.local_rect.center.y);
    let (vx, vy) = canonical_to_view_local(cx, cy, 1600, 900);

    let selected_once = dispatch_raw_json(host, &pointer_json(host, stage, "down", vx, vy, 1));
    assert!(selected_once.accepted);
    let primary_once = selected_once
        .snapshot
        .expect("snapshot")
        .primary_layer_id
        .expect("primary");
    assert_eq!(primary_once, layer.get().to_string());

    let mut resize_scaled = base_intent("stage_resize");
    resize_scaled.stage_handle = Some(stage);
    resize_scaled.width = Some(1600);
    resize_scaled.height = Some(900);
    resize_scaled.scale_factor = Some(2.0);
    assert!(dispatch(host, resize_scaled).accepted);

    let selected_twice = dispatch_raw_json(host, &pointer_json(host, stage, "down", vx, vy, 2));
    assert_eq!(
        selected_twice
            .snapshot
            .expect("snapshot")
            .primary_layer_id
            .expect("primary"),
        layer.get().to_string()
    );

    let _ = host_destroy_stage_for_test(stage);
    let _ = host_destroy_for_test(host);
}

#[test]
fn stage_pointer_down_y_up_hits_upper_half_rect() {
    let _lock = test_lock();
    let mut fixture = Fixture::new();
    let layer = fixture.push_rect_layer("upper", [0.0, 0.25], [0.4, 0.3], Transform2D::identity());
    fixture.document.validate().expect("valid");
    let host = create_host_from_document("sel-y-up", &fixture.document);
    let stage = host_register_stage_for_test(host).expect("stage");
    mount_and_resize(host, stage, 1600, 900);
    // view-local の大きい y（上方向）。Y 反転なら負の正準 y になり外れる。
    let (vx, vy) = canonical_to_view_local(0.0, 0.25, 1600, 900);
    assert!(vy > 450.0);

    let response = dispatch_raw_json(host, &pointer_json(host, stage, "down", vx, vy, 1));
    assert!(response.accepted);
    assert_eq!(
        response.snapshot.expect("snapshot").primary_layer_id,
        Some(layer.get().to_string())
    );

    let _ = host_destroy_stage_for_test(stage);
    let _ = host_destroy_for_test(host);
}

#[test]
fn stage_pointer_down_clear_requires_prior_primary() {
    let _lock = test_lock();
    let mut fixture = Fixture::new();
    let layer = fixture.push_rect_layer(
        "target",
        [0.0, 0.0],
        [0.4, 0.4],
        Transform2D {
            position: DocParam::const_vec2([-0.2, 0.15]),
            rotation: DocParam::const_f64(-0.3),
            ..Transform2D::identity()
        },
    );
    fixture.document.validate().expect("valid");
    let host = create_host_from_document("sel-clear", &fixture.document);
    let stage = host_register_stage_for_test(host).expect("stage");
    mount_and_resize(host, stage, 1600, 900);
    let before_bytes = document_json_bytes(host);

    let tracks = DataTracks::new();
    let proj = project_stage_geometry(
        &fixture.document,
        EvaluationTime::new(RationalTime::ZERO),
        &tracks,
    )
    .expect("geometry");
    let crate::stage_geometry_projection::StageLayerProjection::Available(geo) =
        proj.get(layer).expect("layer")
    else {
        panic!("available");
    };
    let [cx, cy] = (geo.camera_view * geo.world)
        .transform_point(geo.local_rect.center.x, geo.local_rect.center.y);
    let (vx, vy) = canonical_to_view_local(cx, cy, 1600, 900);
    let selected = dispatch_raw_json(host, &pointer_json(host, stage, "down", vx, vy, 1));
    assert_eq!(
        selected.snapshot.expect("snapshot").primary_layer_id,
        Some(layer.get().to_string())
    );

    let cleared = dispatch_raw_json(host, &pointer_json(host, stage, "down", 10.0, 10.0, 2));
    assert!(cleared.accepted);
    let snap = cleared.snapshot.expect("snapshot");
    assert!(snap.primary_layer_id.is_none());
    assert_eq!(snap.projection_generation, "2");
    assert_eq!(document_json_bytes(host), before_bytes);

    let _ = host_destroy_stage_for_test(stage);
    let _ = host_destroy_for_test(host);
}

#[test]
fn stage_pointer_down_overlap_prefers_later_projection_layer() {
    let _lock = test_lock();
    let mut fixture = Fixture::new();
    let back = fixture.push_rect_layer("back", [0.0, 0.0], [0.8, 0.8], Transform2D::identity());
    let front = fixture.push_rect_layer(
        "front",
        [0.0, 0.0],
        [0.5, 0.5],
        Transform2D {
            position: DocParam::const_vec2([0.05, -0.04]),
            ..Transform2D::identity()
        },
    );
    fixture.document.validate().expect("valid");
    let host = create_host_from_document("sel-overlap", &fixture.document);
    let stage = host_register_stage_for_test(host).expect("stage");
    mount_and_resize(host, stage, 1600, 900);
    let (vx, vy) = canonical_to_view_local(0.05, -0.04, 1600, 900);
    let response = dispatch_raw_json(host, &pointer_json(host, stage, "down", vx, vy, 1));
    assert_eq!(
        response.snapshot.expect("snapshot").primary_layer_id,
        Some(front.get().to_string())
    );
    assert_ne!(front, back);

    let _ = host_destroy_stage_for_test(stage);
    let _ = host_destroy_for_test(host);
}

#[test]
fn stage_pointer_same_id_down_is_noop_for_generation() {
    let _lock = test_lock();
    let mut fixture = Fixture::new();
    let layer = fixture.push_rect_layer(
        "same",
        [0.0, 0.0],
        [0.5, 0.5],
        Transform2D {
            position: DocParam::const_vec2([0.2, -0.1]),
            rotation: DocParam::const_f64(0.4),
            ..Transform2D::identity()
        },
    );
    fixture.document.validate().expect("valid");
    let host = create_host_from_document("sel-same-id", &fixture.document);
    let stage = host_register_stage_for_test(host).expect("stage");
    mount_and_resize(host, stage, 1600, 900);
    let tracks = DataTracks::new();
    let proj = project_stage_geometry(
        &fixture.document,
        EvaluationTime::new(RationalTime::ZERO),
        &tracks,
    )
    .expect("geometry");
    let crate::stage_geometry_projection::StageLayerProjection::Available(geo) =
        proj.get(layer).expect("layer")
    else {
        panic!("available");
    };
    let [cx, cy] = (geo.camera_view * geo.world)
        .transform_point(geo.local_rect.center.x, geo.local_rect.center.y);
    let (vx, vy) = canonical_to_view_local(cx, cy, 1600, 900);

    let first = dispatch_raw_json(host, &pointer_json(host, stage, "down", vx, vy, 1));
    assert_eq!(first.snapshot.expect("snapshot").projection_generation, "1");
    let second = dispatch_raw_json(host, &pointer_json(host, stage, "down", vx, vy, 2));
    let snap = second.snapshot.expect("snapshot");
    assert_eq!(snap.projection_generation, "1");
    assert_eq!(snap.primary_layer_id, Some(layer.get().to_string()));

    let _ = host_destroy_stage_for_test(stage);
    let _ = host_destroy_for_test(host);
}

#[test]
fn stage_pointer_drag_up_cancel_keep_prior_primary() {
    let _lock = test_lock();
    let mut fixture = Fixture::new();
    let layer = fixture.push_rect_layer(
        "keep",
        [0.0, 0.0],
        [0.4, 0.4],
        Transform2D {
            position: DocParam::const_vec2([0.1, 0.1]),
            ..Transform2D::identity()
        },
    );
    fixture.document.validate().expect("valid");
    let host = create_host_from_document("sel-phase-keep", &fixture.document);
    let stage = host_register_stage_for_test(host).expect("stage");
    mount_and_resize(host, stage, 1600, 900);
    let (vx, vy) = canonical_to_view_local(0.1, 0.1, 1600, 900);
    let selected = dispatch_raw_json(host, &pointer_json(host, stage, "down", vx, vy, 1));
    assert_eq!(
        selected.snapshot.expect("snapshot").primary_layer_id,
        Some(layer.get().to_string())
    );
    let gen = read_snapshot(host).projection_generation;

    for (phase, seq) in [("drag", 2_u64), ("up", 3), ("cancel", 4)] {
        // 空領域へ送っても selection は変えない。
        let response = dispatch_raw_json(host, &pointer_json(host, stage, phase, 10.0, 10.0, seq));
        assert!(response.accepted, "{phase}");
        let snap = response.snapshot.expect("snapshot");
        assert_eq!(snap.primary_layer_id, Some(layer.get().to_string()));
        assert_eq!(snap.projection_generation, gen);
    }

    let _ = host_destroy_stage_for_test(stage);
    let _ = host_destroy_for_test(host);
}

#[test]
fn stage_pointer_down_zero_extent_rejects_without_changing_primary() {
    let _lock = test_lock();
    let mut fixture = Fixture::new();
    let layer = fixture.push_rect_layer("z", [0.0, 0.0], [0.4, 0.4], Transform2D::identity());
    fixture.document.validate().expect("valid");
    let host = create_host_from_document("sel-zero", &fixture.document);
    let stage = host_register_stage_for_test(host).expect("stage");
    // いったん選択してから width=0 相当（resize 無し）で down する。
    mount_and_resize(host, stage, 1600, 900);
    let (vx, vy) = canonical_to_view_local(0.0, 0.0, 1600, 900);
    assert!(dispatch_raw_json(host, &pointer_json(host, stage, "down", vx, vy, 1)).accepted);
    assert_eq!(
        read_snapshot(host).primary_layer_id,
        Some(layer.get().to_string())
    );

    // resize で正のサイズは必須なので、内部 state を 0 にして zero-extent を再現する。
    with_registry(|registry| {
        let product = registry
            .hosts
            .get_mut(&host)
            .ok_or(RnHostError::UnknownHost(host))?;
        let surface = product
            .stages
            .get_mut(&stage)
            .ok_or(RnHostError::UnknownStage(stage))?;
        surface.width = 0;
        surface.height = 900;
        Ok(())
    })
    .expect("zero width");

    let before = read_snapshot(host);
    let rejected = dispatch_wire(host, pointer_intent(stage, "down", 100.0, 100.0, 2));
    assert!(!rejected.accepted);
    assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));
    let recorded = read_stage_pointer(stage).expect("pointer recorded");
    assert_eq!(recorded.phase, "down");
    assert_eq!(recorded.sequence, 2);
    let after = read_snapshot(host);
    assert_eq!(after.primary_layer_id, before.primary_layer_id);
    assert_eq!(after.projection_generation, before.projection_generation);

    let _ = host_destroy_stage_for_test(stage);
    let _ = host_destroy_for_test(host);
}

#[test]
fn stage_pointer_down_on_singular_layer_clears_primary() {
    let _lock = test_lock();
    let mut fixture = Fixture::new();
    let layer = fixture.push_rect_layer(
        "singular",
        [0.0, 0.0],
        [0.5, 0.5],
        Transform2D {
            scale: DocParam::const_vec2([0.0, 1.0]),
            ..Transform2D::identity()
        },
    );
    fixture.document.validate().expect("valid");
    let host = create_host_from_document("sel-geom-err", &fixture.document);
    // 幾何は壊れていても ReplacePrimary は envelope 存在だけで受理できる。
    // layer 単位の特異は projection 全体を落とさず Unavailable になるため、
    // hit は Miss へ落ちて primary が clear される。
    with_registry(|registry| {
        let product = registry
            .hosts
            .get_mut(&host)
            .ok_or(RnHostError::UnknownHost(host))?;
        let mut queue = DocumentEditQueue::default();
        queue.push_replace_primary(layer);
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
    let stage = host_register_stage_for_test(host).expect("stage");
    mount_and_resize(host, stage, 1600, 900);
    let before = read_snapshot(host);
    assert_eq!(before.primary_layer_id, Some(layer.get().to_string()));

    let selected = dispatch_raw_json(host, &pointer_json(host, stage, "down", 800.0, 450.0, 1));
    assert!(selected.accepted);
    assert_eq!(selected.snapshot.expect("snapshot").primary_layer_id, None);
    let after = read_snapshot(host);
    assert_eq!(after.primary_layer_id, None);
    assert_eq!(read_stage_pointer(stage).expect("pointer").sequence, 1);

    let _ = host_destroy_stage_for_test(stage);
    let _ = host_destroy_for_test(host);
}

#[test]
fn stage_pointer_down_miss_clears_primary() {
    let _lock = test_lock();
    let mut fixture = Fixture::new();
    let layer = fixture.push_rect_layer("healthy", [0.0, 0.0], [0.2, 0.2], Transform2D::identity());
    fixture.document.validate().expect("valid");
    let host = create_host_from_document("sel-miss-clear", &fixture.document);
    with_registry(|registry| {
        let product = registry
            .hosts
            .get_mut(&host)
            .ok_or(RnHostError::UnknownHost(host))?;
        let mut queue = DocumentEditQueue::default();
        queue.push_replace_primary(layer);
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
    let stage = host_register_stage_for_test(host).expect("stage");
    mount_and_resize(host, stage, 1600, 900);
    let before = read_snapshot(host);
    assert_eq!(before.primary_layer_id, Some(layer.get().to_string()));

    // 健全な rect の外側を押す。空き領域の click は選択解除である。
    let missed = dispatch_raw_json(host, &pointer_json(host, stage, "down", 20.0, 20.0, 1));
    assert!(missed.accepted);
    assert_eq!(missed.snapshot.expect("snapshot").primary_layer_id, None);
    assert_eq!(read_snapshot(host).primary_layer_id, None);

    let _ = host_destroy_stage_for_test(stage);
    let _ = host_destroy_for_test(host);
}

#[test]
fn stage_pointer_down_skips_degenerate_and_unavailable_layers() {
    let _lock = test_lock();
    let mut fixture = Fixture::new();
    let _degenerate =
        fixture.push_rect_layer("degen", [0.0, 0.0], [0.0, 0.5], Transform2D::identity());
    let group_id = fixture.document.layers.allocate("g").expect("group");
    fixture.document.tracks[0]
        .items
        .push(TrackItem::Group(motolii_doc::Group {
            envelope: ItemEnvelope::new(group_id),
            children: vec![],
        }));
    fixture.document.validate().expect("valid");
    let host = create_host_from_document("sel-skip", &fixture.document);
    let stage = host_register_stage_for_test(host).expect("stage");
    mount_and_resize(host, stage, 1600, 900);
    // 事前 primary を group 以外の存在 layer で持てないので、先に Replace で degenerate を primary にしない。
    // Miss（退化・Unavailable 除外）→ clear no-op（primary None のまま）。
    let response = dispatch_raw_json(host, &pointer_json(host, stage, "down", 800.0, 450.0, 1));
    assert!(response.accepted);
    assert!(response
        .snapshot
        .expect("snapshot")
        .primary_layer_id
        .is_none());

    let _ = host_destroy_stage_for_test(stage);
    let _ = host_destroy_for_test(host);
}
