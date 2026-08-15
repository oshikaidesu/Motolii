use super::{
    bounds_from_snapshot, clear_slot, dispatch_kind, install_slot, read_host_projection,
    temp_project, test_lock,
};
use super::super::motolii_rnapp_host_dispatch_json;
use super::super::slot::{slice_from_written, MAX_SNAPSHOT_JSON_BYTES};
use crate::timeline_skia::TimelineScene;

#[test]
fn place_then_undo_changes_from_snapshot_band_count() {
    let _lock = test_lock();
    let path = temp_project("place-undo");
    let host = motolii_ui::host_create_for_test(&path).expect("create host");
    let baseline = motolii_ui::host_read_snapshot_for_test(host).expect("baseline");
    let baseline_scene = TimelineScene::from_snapshot(
        &bounds_from_snapshot(&baseline),
        baseline.primary_layer_id.as_deref(),
    );
    assert_eq!(baseline_scene.band_count(), 0);
    assert!(baseline_scene.real);

    dispatch_kind(
        host,
        "place_rectangle",
        r#","position":[0.25,-0.125],"playhead":{"num":0,"den":1}"#,
    );
    let placed_snap = motolii_ui::host_read_snapshot_for_test(host).expect("placed");
    let placed_scene = TimelineScene::from_snapshot(
        &bounds_from_snapshot(&placed_snap),
        placed_snap.primary_layer_id.as_deref(),
    );
    assert_eq!(placed_scene.band_count(), 1);

    dispatch_kind(host, "undo", "");
    let undone_snap = motolii_ui::host_read_snapshot_for_test(host).expect("undone");
    let undone_scene = TimelineScene::from_snapshot(
        &bounds_from_snapshot(&undone_snap),
        undone_snap.primary_layer_id.as_deref(),
    );
    assert_eq!(undone_scene.band_count(), 0);

    motolii_ui::host_destroy_for_test(host).expect("destroy");
}

#[test]
fn place_rectangle_snapshot_json_feeds_stage_geometry_identity() {
    let _lock = test_lock();
    let path = temp_project("place-rect-stage-geom");
    let host = motolii_ui::host_create_for_test(&path).expect("create host");
    let before = read_host_projection(host);
    assert!(before.primary_layer_id.is_none());
    assert!(
        before
            .stage_geometry
            .as_ref()
            .is_none_or(|geometry| geometry.layers.is_empty())
    );

    dispatch_kind(
        host,
        "place_rectangle",
        r#","position":[0.25,-0.125],"playhead":{"num":0,"den":1}"#,
    );
    let after = read_host_projection(host);
    let primary = after.primary_layer_id.expect("placed rectangle is primary");
    let geometry = after.stage_geometry.expect("stage_geometry after place");
    assert_eq!(geometry.layers.len(), 1);
    assert_eq!(geometry.layers[0].layer_id, primary);
    assert_eq!(geometry.layers[0].position, [0.25, -0.125]);
    assert_eq!(geometry.layers[0].rotation, 0.0);
    assert_eq!(geometry.layers[0].scale, [1.0, 1.0]);
    assert_eq!(after.bounds.len(), 1);
    assert_eq!(after.bounds[0].0, primary);
    let snap = motolii_ui::host_read_snapshot_for_test(host).expect("placed document layer");
    assert!(
        snap.layer_ids.iter().any(|id| id == &primary),
        "place_rectangle must write a Document layer, not only a snapshot field"
    );
    assert!(
        crate::rerun_stage::host_layer_fill_is_visible(
            geometry.layers[0].corners,
            false,
            false
        ),
        "placed rectangle must keep Stage fill until evaluated Image"
    );
    assert!(
        !crate::rerun_stage::host_layer_fill_is_visible(
            geometry.layers[0].corners,
            true,
            false
        ),
        "evaluated Image must hide the opaque fill"
    );
    assert!(
        crate::rerun_stage::host_layer_fill_is_visible(geometry.layers[0].corners, true, true),
        "gizmo preview must keep fill while the evaluated Image is stale"
    );

    install_slot(host);
    let revision = after.revision.parse::<u64>().expect("revision");
    let preview = try_preview_stage_transform(
        revision,
        &primary,
        AppStageTransformEdit::TranslateWorld([0.1, 0.0]),
    )
    .expect("host preview");
    assert_eq!(preview.layers.len(), 1);
    assert_ne!(
        preview.layers[0].corners, geometry.layers[0].corners,
        "gizmo preview must move Document path corners"
    );
    assert!((preview.layers[0].position[0] - 0.35).abs() < 1e-12);
    let unchanged = read_host_projection(host)
        .stage_geometry
        .expect("preview must not mutate live Document");
    assert_eq!(unchanged.layers[0].corners, geometry.layers[0].corners);
    let accepted_terminal = dispatch_commit_stage_transform(
        revision,
        &primary,
        AppStageTransformEdit::TranslateWorld([0.1, 0.0]),
    );
    assert!(accepted_terminal.accepted);
    assert!(accepted_terminal.projection.is_some());
    let committed = read_host_projection(host)
        .stage_geometry
        .expect("stage_geometry after gizmo commit");
    assert_eq!(committed.layers[0].corners, preview.layers[0].corners);
    assert!((committed.layers[0].position[0] - 0.35).abs() < 1e-12);
    assert_eq!(committed.layers[0].rotation, 0.0);

    let committed_rev = read_host_projection(host)
        .revision
        .parse::<u64>()
        .expect("revision after commit");
    let rejected_terminal = dispatch_commit_stage_transform(
        revision,
        &primary,
        AppStageTransformEdit::RotateZ(0.25),
    );
    assert!(!rejected_terminal.accepted);
    assert_eq!(rejected_terminal.diagnostics[0].reason, "stale_document");
    assert_eq!(
        rejected_terminal
            .projection
            .expect("authoritative snapshot")
            .stage_geometry
            .expect("authoritative geometry")
            .layers[0]
            .corners,
        committed.layers[0].corners
    );
    try_commit_stage_transform(
        committed_rev,
        &primary,
        AppStageTransformEdit::RotateZ(0.25),
    )
    .expect("gizmo rotate commit");
    let rotated = read_host_projection(host)
        .stage_geometry
        .expect("stage_geometry after rotate");
    assert!((rotated.layers[0].rotation - 0.25).abs() < 1e-12);
    assert!((rotated.layers[0].position[0] - 0.35).abs() < 1e-12);
    clear_slot();

    motolii_ui::host_destroy_for_test(host).expect("destroy");
}

#[cfg(target_os = "macos")]
#[test]
fn handle_free_rn_ellipse_intent_reaches_document_and_stage_projection() {
    let _lock = test_lock();
    clear_slot();
    let path = temp_project("handle-free-place-ellipse");
    let host = motolii_ui::host_create_for_test(&path).expect("create host");
    install_slot(host);
    let intent = br#"{"version":1,"direction":"rn-to-host","kind":"place_ellipse","position":[0.25,-0.125],"playhead":{"num":0,"den":1}}"#;
    let mut out = [0u8; MAX_SNAPSHOT_JSON_BYTES];

    let written = unsafe {
        motolii_rnapp_host_dispatch_json(
            intent.as_ptr(),
            intent.len(),
            out.as_mut_ptr(),
            out.len(),
        )
    };

    assert!(written > 0);
    let response = std::str::from_utf8(
        slice_from_written(&out, written).expect("dispatch response within buffer"),
    )
    .expect("utf8 response");
    assert!(response.contains(r#""accepted":true"#), "{response}");
    let after = read_host_projection(host);
    assert_eq!(after.revision, "1");
    assert_eq!(after.bounds.len(), 1);
    assert_eq!(after.bounds[0].1, "Ellipse");
    let primary = after.primary_layer_id.expect("placed ellipse primary");
    assert_eq!(after.bounds[0].0, primary);
    let geometry = after.stage_geometry.expect("stage geometry");
    assert_eq!(geometry.layers.len(), 1);
    assert_eq!(geometry.layers[0].layer_id, primary);
    assert_eq!(geometry.layers[0].position, [0.25, -0.125]);

    clear_slot();
    motolii_ui::host_destroy_for_test(host).expect("destroy");
}

#[test]
fn place_vism_snapshot_json_feeds_stage_identity() {
    let _lock = test_lock();
    let path = temp_project("place-vism-stage-id");
    let host = motolii_ui::host_create_for_test(&path).expect("create host");

    dispatch_kind(
        host,
        "place_vism",
        r#","plugin_id":"core.layer_source.radial_repeater","position":[0.0,0.0],"playhead":{"num":0,"den":1}"#,
    );
    let after = read_host_projection(host);
    let primary = after.primary_layer_id.expect("placed vism is primary");
    assert_eq!(after.bounds.len(), 1);
    assert_eq!(after.bounds[0].0, primary);

    motolii_ui::host_destroy_for_test(host).expect("destroy");
}

