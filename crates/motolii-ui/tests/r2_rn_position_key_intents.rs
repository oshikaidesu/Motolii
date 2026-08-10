use std::collections::BTreeMap;
use std::sync::Mutex;

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::{
    journal_path_for_document, Clip, ClipSource, DocKeyframe, DocKeyframeTrack, DocParam, DocValue,
    Document, ItemEnvelope, KeyframeId, ProjectSession, ResourceLimits, SaveProjectOptions, Track,
    TrackItem, RECT_LAYER_SOURCE,
};
use motolii_eval::Interp;
use motolii_testkit::tmp_dir;
use motolii_ui::{
    host_create_for_test, host_destroy_for_test, host_dispatch_intent_for_test,
    host_register_stage_for_test, RnHostReasonCode,
};

static HOST_TEST_LOCK: Mutex<()> = Mutex::new(());

fn fixture(tag: &str) -> (std::path::PathBuf, String) {
    let path = tmp_dir(&format!("r2-rn-position-key-{tag}")).join("project.json");
    let mut document = Document::new_current();
    let layer = document
        .layers
        .allocate("position-key-layer")
        .expect("layer");
    let track_id = document.track_ids.allocate("V1").expect("track");
    let mut position = DocKeyframeTrack::new();
    position.insert(DocKeyframe {
        id: KeyframeId::from_raw(document.next_stable_id.allocate().expect("key id")),
        t: RationalTime::ZERO,
        value: DocValue::Vec2([0.0, 0.0]),
        interp: Interp::Hold,
    });
    position.insert(DocKeyframe {
        id: KeyframeId::from_raw(document.next_stable_id.allocate().expect("key id")),
        t: RationalTime::from_seconds(1),
        value: DocValue::Vec2([0.1, 0.1]),
        interp: Interp::Linear,
    });
    let mut envelope = ItemEnvelope::new(layer);
    envelope.transform.position = DocParam::Keyframes(position);
    document.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(Clip {
            envelope,
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
    document.validate().expect("valid fixture");
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
    (path, layer.get().to_string())
}

fn intent(host: u64, kind: &str, target: &str, time: RationalTime) -> serde_json::Value {
    serde_json::json!({
        "version": 1,
        "direction": "rn-to-host",
        "kind": kind,
        "host_handle": host.to_string(),
        "target": target,
        "time": time,
    })
}

fn select(host: u64, stage: u64) {
    assert!(
        host_dispatch_intent_for_test(
            host,
            serde_json::json!({
                "version": 1, "direction": "rn-to-host", "kind": "stage_mount",
                "host_handle": host.to_string(), "stage_handle": stage.to_string()
            })
        )
        .expect("mount")
        .accepted
    );
    assert!(
        host_dispatch_intent_for_test(
            host,
            serde_json::json!({
                "version": 1, "direction": "rn-to-host", "kind": "stage_resize",
                "host_handle": host.to_string(), "stage_handle": stage.to_string(),
                "width": 1000, "height": 1000, "scale_factor": 1.0
            })
        )
        .expect("resize")
        .accepted
    );
    assert!(
        host_dispatch_intent_for_test(
            host,
            serde_json::json!({
                "version": 1, "direction": "rn-to-host", "kind": "stage_pointer",
                "host_handle": host.to_string(), "stage_handle": stage.to_string(),
                "phase": "down", "view_local_x": 500.0, "view_local_y": 500.0,
                "sequence": 1
            })
        )
        .expect("select")
        .accepted
    );
}

#[test]
fn rn_position_keys_use_target_time_addressing_and_are_durable() {
    let _lock = HOST_TEST_LOCK.lock().expect("lock");
    let (path, target) = fixture("oracle");
    let journal = journal_path_for_document(&path);
    let host = host_create_for_test(&path).expect("host");
    let stage = host_register_stage_for_test(host).expect("stage");
    select(host, stage);
    let mut journal_size = std::fs::metadata(&journal).expect("journal").len();

    let add = host_dispatch_intent_for_test(
        host,
        intent(
            host,
            "add_position_key",
            &target,
            RationalTime::from_seconds(2),
        ),
    )
    .expect("add");
    assert!(add.accepted);
    let after_add = add.snapshot.expect("snapshot");
    assert_ne!(after_add.revision, "0");
    let mut next = std::fs::metadata(&journal).expect("journal").len();
    assert!(next > journal_size);
    journal_size = next;

    let mut value = intent(
        host,
        "set_position_key_value",
        &target,
        RationalTime::from_seconds(2),
    );
    value["new"] = serde_json::json!([0.35, -0.2]);
    value["key"] = serde_json::json!("ignored");
    value["old"] = serde_json::json!([999.0, 999.0]);
    let changed = host_dispatch_intent_for_test(host, value).expect("value");
    assert!(changed.accepted);
    let changed_snapshot = changed.snapshot.expect("snapshot");
    assert_ne!(changed_snapshot.revision, after_add.revision);
    next = std::fs::metadata(&journal).expect("journal").len();
    assert!(next > journal_size);
    journal_size = next;

    let mut interp = intent(
        host,
        "set_position_key_interp",
        &target,
        RationalTime::from_seconds(2),
    );
    interp["interp"] = serde_json::json!({"Bezier": {"x1": 0.2, "y1": 0.8, "x2": 0.7, "y2": 0.1}});
    let finished = host_dispatch_intent_for_test(host, interp).expect("interp");
    assert!(finished.accepted);
    assert_ne!(
        finished.snapshot.expect("snapshot").revision,
        changed_snapshot.revision
    );
    assert!(std::fs::metadata(&journal).expect("journal").len() > journal_size);

    host_destroy_for_test(host).expect("destroy host");
    let limits = ResourceLimits::production();
    let (_session, opened) = ProjectSession::open(&path, &limits).expect("reopen");
    let item = &opened.document.tracks[0].items[0];
    let envelope = match item {
        TrackItem::Clip(clip) => &clip.envelope,
        TrackItem::Group(_) => panic!("unexpected group"),
    };
    let DocParam::Keyframes(keys) = &envelope.transform.position else {
        panic!("keys")
    };
    let key = keys
        .keys()
        .iter()
        .find(|key| key.t == RationalTime::from_seconds(2))
        .expect("added key");
    assert_eq!(key.value, DocValue::Vec2([0.35, -0.2]));
    assert_eq!(
        key.interp,
        Interp::Bezier {
            x1: 0.2,
            y1: 0.8,
            x2: 0.7,
            y2: 0.1
        }
    );
}

#[test]
fn rn_position_key_undo_removes_added_key() {
    let _lock = HOST_TEST_LOCK.lock().expect("lock");
    let (path, target) = fixture("undo");
    let host = host_create_for_test(&path).expect("host");
    let stage = host_register_stage_for_test(host).expect("stage");
    select(host, stage);
    assert!(
        host_dispatch_intent_for_test(
            host,
            intent(
                host,
                "add_position_key",
                &target,
                RationalTime::from_seconds(2)
            )
        )
        .expect("add")
        .accepted
    );
    assert!(host_dispatch_intent_for_test(
        host,
        serde_json::json!({"version": 1, "direction": "rn-to-host", "kind": "undo", "host_handle": host.to_string()})
    )
    .expect("undo")
    .accepted);
    host_destroy_for_test(host).expect("destroy host");
    let limits = ResourceLimits::production();
    let (_session, opened) = ProjectSession::open(&path, &limits).expect("reopen");
    let TrackItem::Clip(clip) = &opened.document.tracks[0].items[0] else {
        panic!("clip")
    };
    let DocParam::Keyframes(keys) = &clip.envelope.transform.position else {
        panic!("keys")
    };
    assert!(!keys
        .keys()
        .iter()
        .any(|key| key.t == RationalTime::from_seconds(2)));
}

#[test]
fn rn_position_key_rejects_missing_time_after_selection() {
    let _lock = HOST_TEST_LOCK.lock().expect("lock");
    let (path, target) = fixture("reject");
    let host = host_create_for_test(&path).expect("host");
    let stage = host_register_stage_for_test(host).expect("stage");
    select(host, stage);
    let response = host_dispatch_intent_for_test(
        host,
        intent(
            host,
            "set_position_key_interp",
            &target,
            RationalTime::from_seconds(9),
        ),
    )
    .expect("dispatch");
    assert!(!response.accepted);
    assert_eq!(response.reason, Some(RnHostReasonCode::InvalidIntent));
    host_destroy_for_test(host).expect("destroy host");
}
