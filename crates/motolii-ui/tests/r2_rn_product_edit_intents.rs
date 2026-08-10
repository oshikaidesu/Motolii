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
    RnHostReasonCode,
};

static HOST_TEST_LOCK: Mutex<()> = Mutex::new(());

fn fixture_path(tag: &str) -> std::path::PathBuf {
    let path = tmp_dir(&format!("r2-rn-product-edit-{tag}")).join("project.json");
    let mut document = Document::new_current();
    let layer = document.layers.allocate("r2-layer").expect("layer");
    let track = document.track_ids.allocate("r2-track").expect("track");
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

fn dispatch(host: u64, payload: serde_json::Value) -> motolii_ui::RnHostTestResponse {
    host_dispatch_intent_for_test(host, payload).expect("dispatch")
}

fn undo(host: u64) {
    let response = dispatch(
        host,
        serde_json::json!({
            "version": 1, "direction": "rn-to-host", "kind": "undo",
            "host_handle": host.to_string()
        }),
    );
    assert!(response.accepted);
}

fn add_key(host: u64, target: u64) -> motolii_ui::RnProductSnapshotForTest {
    let response = dispatch(
        host,
        serde_json::json!({
            "version": 1, "direction": "rn-to-host", "kind": "add_position_key",
            "host_handle": host.to_string(), "target": target,
            "time": serde_json::to_value(RationalTime::from_seconds(1)).unwrap()
        }),
    );
    assert!(response.accepted);
    response.snapshot.unwrap()
}

fn select_primary(host: u64, target: u64) {
    let stage = host_register_stage_for_test(host).expect("register stage");
    for payload in [
        serde_json::json!({
            "version": 1, "direction": "rn-to-host", "kind": "stage_mount",
            "host_handle": host.to_string(), "stage_handle": stage.to_string()
        }),
        serde_json::json!({
            "version": 1, "direction": "rn-to-host", "kind": "stage_resize",
            "host_handle": host.to_string(), "stage_handle": stage.to_string(),
            "width": 1600, "height": 900, "scale_factor": 1.0
        }),
        serde_json::json!({
            "version": 1, "direction": "rn-to-host", "kind": "stage_pointer",
            "host_handle": host.to_string(), "stage_handle": stage.to_string(),
            "phase": "down", "view_local_x": 800.0, "view_local_y": 450.0,
            "sequence": 1
        }),
    ] {
        let response = dispatch(host, payload);
        assert!(response.accepted);
    }
    let selected = host_read_snapshot_for_test(host).expect("selected snapshot");
    assert_eq!(selected.primary_layer_id, Some(target.to_string()));
    host_destroy_stage_for_test(stage).expect("destroy stage");
}

#[test]
fn rn_r2_edit_intents_without_primary_selection_are_noops() {
    let _lock = HOST_TEST_LOCK.lock().expect("host test lock");
    let host = host_create_for_test(&fixture_path("unselected-noop")).expect("create host");
    let baseline = host_read_snapshot_for_test(host).expect("baseline");
    let target = baseline.layer_ids[0].parse::<u64>().expect("layer id");

    let after = add_key(host, target);
    assert_eq!(after.revision, baseline.revision);
    assert_eq!(after.primary_layer_id, None);

    host_destroy_for_test(host).expect("destroy host");
}

#[test]
fn rn_r2_edit_intents_reach_document_queue_and_undo() {
    let _lock = HOST_TEST_LOCK.lock().expect("host test lock");
    let host = host_create_for_test(&fixture_path("oracle")).expect("create host");
    let baseline = host_read_snapshot_for_test(host).expect("baseline");
    let target = baseline.layer_ids[0].parse::<u64>().expect("layer id");
    select_primary(host, target);
    let selected = host_read_snapshot_for_test(host).expect("selected snapshot");
    assert_eq!(selected.primary_layer_id, Some(target.to_string()));

    let added = add_key(host, target);
    assert_ne!(added.revision, baseline.revision);
    undo(host);

    let added = add_key(host, target);
    let changed = dispatch(
        host,
        serde_json::json!({
            "version": 1, "direction": "rn-to-host", "kind": "set_position_key_value",
            "host_handle": host.to_string(), "target": target, "key": 1,
            "old": [0.0, 0.0], "new": [0.25, -0.5]
        }),
    );
    assert!(changed.accepted);
    assert_ne!(changed.snapshot.unwrap().revision, added.revision);
    undo(host);

    let added = add_key(host, target);
    let changed = dispatch(
        host,
        serde_json::json!({
            "version": 1, "direction": "rn-to-host", "kind": "set_position_key_interp",
            "host_handle": host.to_string(), "target": target, "key": 2,
            "interp": "Hold"
        }),
    );
    assert!(changed.accepted);
    assert_ne!(changed.snapshot.unwrap().revision, added.revision);
    undo(host);

    let rejected = dispatch(
        host,
        serde_json::json!({
            "version": 1, "direction": "rn-to-host", "kind": "set_effect_param",
            "host_handle": host.to_string(), "layer_id": target, "effect_use_id": 999,
            "definition_id": 999, "plugin_id": "missing", "effect_version": 1,
            "param_id": "amount", "value": 0.5
        }),
    );
    assert!(!rejected.accepted);
    assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));

    host_destroy_for_test(host).expect("destroy host");
}
