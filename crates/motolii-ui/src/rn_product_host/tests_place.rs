//! place / reconcile の試験。helper は tests。
use super::tests::*;
use super::*;

#[test]
fn seed_snapshot_projects_timeline_layer_interval_without_keys() {
    let _lock = test_lock();
    let host = create_host("timeline-seed");
    let snap = read_snapshot(host);
    assert_eq!(snap.timeline.layers.len(), 1);
    let layer = &snap.timeline.layers[0];
    assert_eq!(layer.layer_id, snap.layer_ids[0]);
    assert_eq!(layer.start, RationalTime::ZERO);
    assert_eq!(
        layer.duration,
        RationalTime::try_new(10, 1).expect("composition duration")
    );
    assert!(layer.position_keys.is_empty());
    assert!(!layer.keys_truncated);
    assert!(!snap.timeline.layers_truncated);
    assert_eq!(snap.timeline.fps.num(), 30);
    assert_eq!(snap.timeline.fps.den(), 1);
    let _ = host_destroy_for_test(host);
}

#[test]
fn seed_snapshot_projects_stage_geometry_corners_for_unit_rect() {
    let _lock = test_lock();
    let host = create_host("stage-geom-seed");
    let wire = with_registry(|registry| registry.read_snapshot(host)).expect("wire");
    assert_eq!(wire.stage_geometry.layers.len(), 1);
    assert!(!wire.stage_geometry.layers_truncated);
    // seed: center(0,0) size(1,1) · identity world → CCW 左下起点
    assert_eq!(
        wire.stage_geometry.layers[0].corners,
        [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]]
    );
    assert_eq!(wire.stage_geometry.layers[0].position, [0.0, 0.0]);
    assert_eq!(wire.stage_geometry.layers[0].rotation, 0.0);
    assert_eq!(wire.stage_geometry.layers[0].scale, [1.0, 1.0]);
    let _ = host_destroy_for_test(host);
}

#[test]
fn stage_transform_preview_moves_projected_path_corners() {
    let _lock = test_lock();
    let host = create_host("stage-gizmo-preview-path");
    let wire = with_registry(|registry| registry.read_snapshot(host)).expect("wire");
    let revision = wire.revision.parse::<u64>().expect("revision");
    let layer_id = wire.stage_geometry.layers[0]
        .layer_id
        .parse::<u64>()
        .expect("layer");
    let before = wire.stage_geometry.layers[0].corners;
    let preview = host_preview_stage_transform_for_app(
        host,
        revision,
        layer_id,
        AppStageTransformEdit::TranslateWorld([0.1, 0.0]),
    )
    .expect("preview");
    assert_eq!(preview.geometry.layers.len(), 1);
    assert_ne!(
        preview.geometry.layers[0].corners, before,
        "Stage path corners must move through Document clone preview"
    );
    assert!((preview.geometry.layers[0].position[0] - 0.1).abs() < 1e-12);
    let after = with_registry(|registry| registry.read_snapshot(host)).expect("unchanged");
    assert_eq!(
        after.stage_geometry.layers[0].corners, before,
        "preview must not mutate the live Document"
    );
    let _ = host_destroy_for_test(host);
}

fn mirror_signed_area(corners: &[[f64; 2]; 4]) -> f64 {
    let p0 = corners[0];
    let p1 = corners[1];
    let p2 = corners[2];
    let p3 = corners[3];
    0.5 * ((p0[0] * p1[1] - p1[0] * p0[1])
        + (p1[0] * p2[1] - p2[0] * p1[1])
        + (p2[0] * p3[1] - p3[0] * p2[1])
        + (p3[0] * p0[1] - p0[0] * p3[1]))
}

#[test]
fn mirrored_world_geometry_is_forced_to_ccw() {
    let corners = world_rect_corners(
        motolii_doc::Affine2D::scale(-1.0, 1.0),
        [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]],
    );
    assert!(mirror_signed_area(&corners) > 0.0);
    assert_eq!(
        corners,
        [[0.5, 0.5], [-0.5, 0.5], [-0.5, -0.5], [0.5, -0.5]]
    );
}

#[test]
fn place_rectangle_adds_stage_geometry_layer_at_drop_position() {
    let _lock = test_lock();
    let host = create_host("stage-geom-place");
    let seed = with_registry(|registry| registry.read_snapshot(host)).expect("seed");
    let seed_layer = seed.stage_geometry.layers[0].layer_id.clone();
    let response = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"place_rectangle","#,
                r#""host_handle":"{host}","position":[0.25,-0.125],"playhead":{{"num":0,"den":1}}}}"#
            ),
            host = host,
        ),
    );
    assert!(response.accepted);
    let wire = with_registry(|registry| registry.read_snapshot(host)).expect("placed");
    assert_eq!(wire.stage_geometry.layers.len(), 2);
    let placed = wire
        .stage_geometry
        .layers
        .iter()
        .find(|layer| layer.layer_id != seed_layer)
        .expect("placed layer");
    let document = live_document(host);
    let placed_id = LayerId::from_raw(placed.layer_id.parse().expect("placed layer id"));
    assert_eq!(document.layers.display_name(placed_id), Some("Rectangle"));
    assert!(document_clip(document.as_ref(), placed_id).is_some());
    // place Vector rect 0.2×0.2 at transform.position — world 適用済み corners
    let expected = [
        [0.15, -0.225],
        [0.35, -0.225],
        [0.35, -0.025],
        [0.15, -0.025],
    ];
    for (got, want) in placed.corners.iter().zip(expected.iter()) {
        assert!(
            (got[0] - want[0]).abs() < 1e-12 && (got[1] - want[1]).abs() < 1e-12,
            "corners {got:?} vs {want:?}"
        );
    }
    let _ = host_destroy_for_test(host);
}

#[test]
fn add_position_key_appears_in_timeline_projection() {
    let _lock = test_lock();
    let host = create_host("timeline-add-key");
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
    let layer = &snap.timeline.layers[0];
    assert_eq!(layer.position_keys.len(), 1);
    assert!(!layer.position_keys[0].key_id.is_empty());
    assert_eq!(layer.position_keys[0].time, time);
    assert_eq!(layer.position_keys[0].interp, Some(Interp::Linear));
    assert!(!layer.keys_truncated);
    let _ = host_destroy_for_test(host);
}

fn keyed_scale_document() -> (Document, LayerId, KeyframeId) {
    let mut document = Document::new_current();
    let layer = document.layers.allocate("keyed-scale").expect("layer");
    let track = document.track_ids.allocate("track").expect("track");
    let key_id = KeyframeId::from_raw(document.next_stable_id.allocate().expect("key"));
    let mut keyframes = DocKeyframeTrack::new();
    keyframes.insert(DocKeyframe {
        id: key_id,
        t: RationalTime::try_new(1, 1).expect("1s"),
        value: DocValue::Vec2([1.0, 1.0]),
        interp: Interp::Linear,
    });
    let mut envelope = ItemEnvelope::new(layer);
    envelope.transform.scale = DocParam::Keyframes(keyframes);
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
    document.validate().expect("valid keyed scale");
    (document, layer, key_id)
}

#[test]
fn timeline_scale_keyframes_appear_in_param_keys() {
    let _lock = test_lock();
    let (document, _, key_id) = keyed_scale_document();
    let host = create_host_from_document("timeline-param-keys", &document);
    let wire = read_wire(host);
    let layer = &wire.timeline.layers[0];
    assert!(layer.position_keys.is_empty());
    assert_eq!(layer.param_keys.len(), 1);
    assert_eq!(layer.param_keys[0].property, "scale");
    assert_eq!(layer.param_keys[0].key_id, key_id.get().to_string());
    assert_eq!(
        layer.param_keys[0].time,
        RationalTime::try_new(1, 1).expect("1s")
    );
    assert_eq!(layer.param_keys[0].vec, Some([1.0, 1.0]));
    assert_eq!(layer.param_keys[0].value, None);
    let _ = host_destroy_for_test(host);
}

#[test]
fn place_ellipse_adds_stage_geometry_layer_at_drop_position() {
    let _lock = test_lock();
    let host = create_host("stage-geom-place-ellipse");
    let seed = with_registry(|registry| registry.read_snapshot(host)).expect("seed");
    let seed_layer = seed.stage_geometry.layers[0].layer_id.clone();
    let response = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"place_ellipse","#,
                r#""host_handle":"{host}","position":[0.25,-0.125],"playhead":{{"num":0,"den":1}}}}"#
            ),
            host = host,
        ),
    );
    assert!(response.accepted);
    let wire = with_registry(|registry| registry.read_snapshot(host)).expect("placed");
    assert_eq!(wire.stage_geometry.layers.len(), 2);
    let placed = wire
        .stage_geometry
        .layers
        .iter()
        .find(|layer| layer.layer_id != seed_layer)
        .expect("placed layer");
    let document = live_document(host);
    let placed_id = LayerId::from_raw(placed.layer_id.parse().expect("placed layer id"));
    assert_eq!(document.layers.display_name(placed_id), Some("Ellipse"));
    let clip = document_clip(document.as_ref(), placed_id).expect("ellipse clip");
    assert!(matches!(
        clip.source,
        ClipSource::Vector {
            recipe: motolii_doc::VectorRecipe {
                content: motolii_doc::VectorContent::StandardShape {
                    shape: motolii_doc::StandardShape::Ellipse { .. }
                },
                ..
            }
        }
    ));
    // 0.2×0.2 楕円のgizmoは同じAABB。rect place と同じ corners。
    let expected = [
        [0.15, -0.225],
        [0.35, -0.225],
        [0.35, -0.025],
        [0.15, -0.025],
    ];
    for (got, want) in placed.corners.iter().zip(expected.iter()) {
        assert!(
            (got[0] - want[0]).abs() < 1e-12 && (got[1] - want[1]).abs() < 1e-12,
            "corners {got:?} vs {want:?}"
        );
    }
    let _ = host_destroy_for_test(host);
}

#[test]
fn placement_runtime_rejects_keep_typed_reason_and_zero_write_state() {
    let _lock = test_lock();

    pub(super) fn assert_rejected_without_state_change(
        host: u64,
        kind: &str,
        playhead: RationalTime,
        expected: RnHostReasonCode,
    ) {
        let before_wire = read_wire(host);
        let before_document = document_json_bytes(host);
        let response = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"{kind}","#,
                    r#""host_handle":"{host}","position":[0.0,0.0],"#,
                    r#""playhead":{{"num":{num},"den":{den}}}}}"#
                ),
                kind = kind,
                host = host,
                num = playhead.num(),
                den = playhead.den(),
            ),
        );
        assert!(!response.accepted);
        assert_eq!(response.reason, Some(expected));

        let after_wire = read_wire(host);
        assert_eq!(after_wire.revision, before_wire.revision);
        assert_eq!(
            after_wire.projection_generation,
            before_wire.projection_generation
        );
        assert_eq!(after_wire.history, before_wire.history);
        assert_eq!(document_json_bytes(host), before_document);
    }

    let no_track = create_host_from_document("place-no-track", &Document::new_current());
    assert_rejected_without_state_change(
        no_track,
        "place_ellipse",
        RationalTime::ZERO,
        RnHostReasonCode::NoTrackForRectangle,
    );
    let _ = host_destroy_for_test(no_track);

    let negative = create_host("place-negative-time");
    assert_rejected_without_state_change(
        negative,
        "place_rectangle",
        RationalTime::try_new(-1, 1).expect("negative time"),
        RnHostReasonCode::PlayheadOutsideComposition,
    );
    let _ = host_destroy_for_test(negative);

    let no_remaining = create_host("place-no-remaining-duration");
    let duration = live_document(no_remaining).composition.duration;
    assert_rejected_without_state_change(
        no_remaining,
        "place_ellipse",
        duration,
        RnHostReasonCode::RemainingDurationBelowOneFrame,
    );
    let _ = host_destroy_for_test(no_remaining);
}

#[test]
fn deferred_commit_keeps_reads_live_and_reconciles_on_snapshot_poll() {
    let _lock = test_lock();
    let host = create_host("reconcile-failure-host");
    let before = read_wire(host);
    let before_document = document_json_bytes(host);
    with_registry(|registry| {
        let product = registry
            .hosts
            .get_mut(&host)
            .ok_or(RnHostError::UnknownHost(host))?;
        product.runtime.set_test_failpoint(
            crate::document_edit_runtime::RuntimeTestFailpoint::DeferAfterDurableCommit,
        );
        Ok(())
    })
    .expect("inject reconcile failure");

    let rejected = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"place_rectangle","#,
                r#""host_handle":"{host}","position":[0.0,0.0],"playhead":{{"num":0,"den":1}}}}"#
            ),
            host = host,
        ),
    );
    assert!(!rejected.accepted);
    assert_eq!(
        rejected.reason,
        Some(RnHostReasonCode::DocumentWriteBlocked)
    );
    assert!(rejected.snapshot.is_some());
    assert_eq!(document_json_bytes(host), before_document);

    with_registry(|registry| {
        let product = registry
            .hosts
            .get_mut(&host)
            .ok_or(RnHostError::UnknownHost(host))?;
        assert!(product.runtime.is_write_blocked());
        product.runtime.fail_reconcile_for_test();
        assert_eq!(
            product.primary.as_ref().map(|id| id.get().to_string()),
            before.primary_layer_id
        );
        assert_eq!(
            product.projection_generation.to_string(),
            before.projection_generation
        );
        Ok(())
    })
    .expect("write-blocked host state");

    let first_poll = host_read_snapshot_for_test(host).expect("read stays live");
    assert_eq!(first_poll.revision, before.revision);
    let second_poll = host_read_snapshot_for_test(host).expect("reconcile retry");
    assert_ne!(second_poll.revision, before.revision);
    assert_ne!(document_json_bytes(host), before_document);
    with_registry(|registry| {
        let product = registry
            .hosts
            .get(&host)
            .ok_or(RnHostError::UnknownHost(host))?;
        assert!(!product.runtime.is_write_blocked());
        Ok(())
    })
    .expect("reconciled host state");

    #[cfg(target_os = "macos")]
    {
        let mut revision = 0;
        let mut generation = 0;
        assert!(motolii_rn_host_projection_stamp(
            host,
            &mut revision,
            &mut generation
        ));
    }

    let _ = host_destroy_for_test(host);
}

#[test]
fn read_snapshot_preserves_exact_reconcile_reason_until_unblocked() {
    let _lock = test_lock();
    let path = fixture_path("reconcile-reason-snapshot");
    let host = host_create_for_test(&path).expect("host");
    let before = read_wire(host);
    let before_document = document_json_bytes(host);
    let target = before.stage.bounds[0].layer_id.clone();
    with_registry(|registry| {
        let product = registry
            .hosts
            .get_mut(&host)
            .ok_or(RnHostError::UnknownHost(host))?;
        product.runtime.set_test_failpoint(
            crate::document_edit_runtime::RuntimeTestFailpoint::DeferAfterDurableCommit,
        );
        Ok(())
    })
    .expect("defer durable commit publication");

    let rejected = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"place_rectangle","#,
                r#""host_handle":"{host}","position":[0.0,0.0],"playhead":{{"num":0,"den":1}}}}"#
            ),
            host = host,
        ),
    );
    assert!(!rejected.accepted);
    assert_eq!(
        rejected.reason,
        Some(RnHostReasonCode::DocumentWriteBlocked)
    );
    assert_eq!(document_json_bytes(host), before_document);

    let journal = motolii_doc::journal_path_for_document(&path);
    let committed_journal = fs::read(&journal).expect("committed journal");
    fs::write(&journal, b"not a journal").expect("inject reconcile read failure");
    let failed_read = read_wire(host);
    assert_eq!(failed_read.revision, before.revision);
    assert_eq!(
        failed_read
            .diagnostics
            .first()
            .map(|diagnostic| diagnostic.reason),
        Some(RnHostReasonCode::JournalReconcile)
    );
    let failed_json: serde_json::Value =
        serde_json::from_str(&encode_snapshot_json(&failed_read).expect("encode failed snapshot"))
            .expect("failed snapshot json");
    assert_eq!(
        failed_json
            .pointer("/diagnostics/0/reason")
            .and_then(serde_json::Value::as_str),
        Some("journal_reconcile")
    );
    assert_eq!(
        read_wire(host)
            .diagnostics
            .first()
            .map(|diagnostic| diagnostic.reason),
        Some(RnHostReasonCode::JournalReconcile),
        "each blocked read must preserve the current reconcile reason"
    );
    fs::write(&journal, committed_journal).expect("restore committed journal");
    assert_eq!(document_json_bytes(host), before_document);

    let selection = dispatch_raw_json(
        host,
        &format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"select_layer","host_handle":"{host}","target":"{target}"}}"#,
        ),
    );
    assert!(
        selection.accepted,
        "selection must remain live while blocked"
    );
    assert_eq!(document_json_bytes(host), before_document);

    let blocked_write = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"place_rectangle","#,
                r#""host_handle":"{host}","position":[0.25,0.25],"playhead":{{"num":0,"den":1}}}}"#
            ),
            host = host,
        ),
    );
    assert!(!blocked_write.accepted);
    assert_eq!(
        blocked_write.reason,
        Some(RnHostReasonCode::DocumentWriteBlocked)
    );
    assert_eq!(document_json_bytes(host), before_document);

    let reconciled = read_wire(host);
    assert_eq!(
        reconciled.revision.parse::<u64>().expect("revision"),
        before.revision.parse::<u64>().expect("before revision") + 1
    );
    assert_eq!(reconciled.primary_layer_id, Some(target));
    assert!(reconciled.diagnostics.is_empty());
    assert_eq!(read_wire(host).revision, reconciled.revision);
    with_registry(|registry| {
        let product = registry
            .hosts
            .get(&host)
            .ok_or(RnHostError::UnknownHost(host))?;
        assert!(!product.runtime.is_write_blocked());
        Ok(())
    })
    .expect("reconciled host state");

    let _ = host_destroy_for_test(host);
}
