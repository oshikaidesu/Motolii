//! Wave R0 product runtime seat integration oracle.

use std::collections::BTreeMap;
use std::sync::Mutex;

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::{
    Clip, ClipSource, DocParam, Document, ItemEnvelope, ProjectSession, ResourceLimits,
    SaveProjectOptions, Track, TrackItem, RECT_LAYER_SOURCE,
};
use motolii_testkit::tmp_dir;
use motolii_ui::{
    host_create_for_test, host_destroy_for_test, host_destroy_stage_for_test,
    host_dispatch_intent_for_test, host_read_snapshot_for_test, host_register_stage_for_test,
    RnHostError, RnHostReasonCode, RnHostTestIntent,
};

static HOST_TEST_LOCK: Mutex<()> = Mutex::new(());

fn fixture_path(tag: &str) -> std::path::PathBuf {
    let path = tmp_dir(&format!("r0-rn-product-seat-{tag}")).join("project.json");
    let mut document = Document::new_current();
    let layer = document.layers.allocate("r0-layer").expect("layer");
    let track = document.track_ids.allocate("r0-track").expect("track");
    document.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: document.composition.duration,
            time_map: TimeMap::identity(),
            source: ClipSource::Plugin {
                plugin_id: RECT_LAYER_SOURCE.into(),
                effect_version: 1,
                params: BTreeMap::from([
                    ("center".into(), DocParam::const_vec2([0.0, 0.0])),
                    ("size".into(), DocParam::const_vec2([1.0, 1.0])),
                    ("color".into(), DocParam::const_color([0.0, 1.0, 0.0, 1.0])),
                ]),
                extra: Default::default(),
            },
        })],
    });
    document.validate().expect("valid fixture document");
    let limits = ResourceLimits::production();
    let mut session = ProjectSession::acquire(&path, &limits).expect("acquire fixture");
    session
        .save_with_journal(
            &document,
            &SaveProjectOptions {
                limits,
                checkpoint: true,
                ..SaveProjectOptions::default()
            },
        )
        .expect("save fixture");
    path
}

fn lifecycle_intent(kind: &str, stage: u64) -> RnHostTestIntent {
    RnHostTestIntent {
        kind: kind.to_owned(),
        stage_handle: Some(stage),
        projection_generation: None,
        width: None,
        height: None,
        scale_factor: Some(2.0),
        focused: None,
    }
}

#[test]
fn r0_product_seat_preserves_snapshot_across_mount_resize_focus_unmount_remount() {
    let _lock = HOST_TEST_LOCK.lock().expect("host test lock");
    let host = host_create_for_test(&fixture_path("lifecycle")).expect("create host");
    let baseline = host_read_snapshot_for_test(host).expect("baseline snapshot");
    assert_eq!(baseline.revision, "0");
    assert_eq!(baseline.projection_generation, "0");
    assert!(!baseline.layer_ids.is_empty());

    let stage = host_register_stage_for_test(host).expect("register stage");
    for mut intent in [
        lifecycle_intent("stage_mount", stage),
        lifecycle_intent("stage_resize", stage),
        lifecycle_intent("stage_focus", stage),
        lifecycle_intent("stage_unmount", stage),
    ] {
        if intent.kind == "stage_resize" {
            intent.width = Some(1920);
            intent.height = Some(1080);
        }
        if intent.kind == "stage_focus" {
            intent.focused = Some(true);
        }
        let response = host_dispatch_intent_for_test(host, intent).expect("dispatch lifecycle");
        assert!(response.accepted);
        let snapshot = response.snapshot.expect("snapshot payload");
        assert_eq!(snapshot.revision, baseline.revision);
        assert_eq!(
            snapshot.projection_generation,
            baseline.projection_generation
        );
        assert_eq!(snapshot.primary_layer_id, baseline.primary_layer_id);
        assert_eq!(snapshot.layer_ids, baseline.layer_ids);
    }

    host_destroy_stage_for_test(stage).expect("unmount stage");
    let remount_stage = host_register_stage_for_test(host).expect("register remount stage");
    let remount =
        host_dispatch_intent_for_test(host, lifecycle_intent("stage_mount", remount_stage))
            .expect("remount");
    assert!(remount.accepted);
    let after = remount.snapshot.expect("remounted snapshot");
    assert_eq!(after.revision, baseline.revision);
    assert_eq!(after.projection_generation, baseline.projection_generation);
    assert_eq!(after.layer_ids, baseline.layer_ids);

    host_destroy_stage_for_test(remount_stage).expect("destroy remount stage");
    host_destroy_for_test(host).expect("destroy host");
}

#[test]
fn r0_product_seat_rejects_stale_generation_without_document_write() {
    let _lock = HOST_TEST_LOCK.lock().expect("host test lock");
    let host = host_create_for_test(&fixture_path("stale")).expect("create host");
    let before = host_read_snapshot_for_test(host).expect("before");
    let remount = host_dispatch_intent_for_test(
        host,
        RnHostTestIntent {
            kind: "read_snapshot".to_owned(),
            stage_handle: None,
            projection_generation: Some("42".to_owned()),
            width: None,
            height: None,
            scale_factor: None,
            focused: None,
        },
    )
    .expect("stale dispatch");
    assert!(!remount.accepted);
    assert_eq!(
        remount.reason,
        Some(RnHostReasonCode::StaleProjectionGeneration)
    );
    let after = host_read_snapshot_for_test(host).expect("after");
    assert_eq!(after.revision, before.revision);
    assert_eq!(after.projection_generation, before.projection_generation);
    host_destroy_for_test(host).expect("destroy stale host");
}

#[test]
fn r0_product_seat_rejects_incomplete_or_invalid_lifecycle_payloads() {
    let _lock = HOST_TEST_LOCK.lock().expect("host test lock");
    let host = host_create_for_test(&fixture_path("invalid-lifecycle")).expect("create host");
    let stage = host_register_stage_for_test(host).expect("register stage");
    let baseline = host_read_snapshot_for_test(host).expect("baseline");

    for intent in [
        RnHostTestIntent {
            kind: "stage_resize".to_owned(),
            stage_handle: Some(stage),
            projection_generation: None,
            width: Some(1920),
            height: None,
            scale_factor: Some(2.0),
            focused: None,
        },
        RnHostTestIntent {
            kind: "stage_resize".to_owned(),
            stage_handle: Some(stage),
            projection_generation: None,
            width: Some(1920),
            height: Some(1080),
            scale_factor: Some(0.0),
            focused: None,
        },
        RnHostTestIntent {
            kind: "stage_focus".to_owned(),
            stage_handle: Some(stage),
            projection_generation: None,
            width: None,
            height: None,
            scale_factor: None,
            focused: None,
        },
    ] {
        let response =
            host_dispatch_intent_for_test(host, intent).expect("dispatch invalid intent");
        assert!(!response.accepted);
        assert_eq!(response.reason, Some(RnHostReasonCode::InvalidIntent));
    }

    let after = host_read_snapshot_for_test(host).expect("after");
    assert_eq!(after, baseline);
    host_destroy_stage_for_test(stage).expect("destroy stage");
    host_destroy_for_test(host).expect("destroy host");
}

#[test]
fn r0_product_seat_rejects_single_host_invalid_path_and_late_handles() {
    let _lock = HOST_TEST_LOCK.lock().expect("host test lock");
    let host = host_create_for_test(&fixture_path("single")).expect("create host");

    assert!(matches!(
        host_create_for_test(&fixture_path("second")),
        Err(RnHostError::HostAlreadyExists)
    ));
    assert!(matches!(
        host_create_for_test(&tmp_dir("r0-rn-invalid").join("missing.json")),
        Err(RnHostError::HostAlreadyExists)
    ));

    let stage = host_register_stage_for_test(host).expect("register stage");
    host_destroy_stage_for_test(stage).expect("destroy stage");
    assert!(matches!(
        host_destroy_stage_for_test(stage),
        Err(RnHostError::DestroyedStage(_))
    ));

    let late = host_dispatch_intent_for_test(host, lifecycle_intent("stage_mount", stage))
        .expect("late stage dispatch");
    assert!(!late.accepted);
    assert_eq!(late.reason, Some(RnHostReasonCode::LateLifecycleEvent));

    host_destroy_for_test(host).expect("destroy host");
    assert!(matches!(
        host_destroy_for_test(host),
        Err(RnHostError::DestroyedHost(_))
    ));
    assert!(matches!(
        host_create_for_test(&tmp_dir("r0-rn-invalid").join("missing-project")),
        Err(RnHostError::OpenProject(_))
    ));
}
