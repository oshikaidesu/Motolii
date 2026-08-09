use std::collections::BTreeMap;
use std::sync::Mutex;

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::{
    journal_path_for_document, Clip, ClipSource, DocParam, Document, ItemEnvelope, ProjectSession,
    ResourceLimits, SaveProjectOptions, Track, TrackItem, RECT_LAYER_SOURCE,
};
use motolii_testkit::tmp_dir;
use motolii_ui::{
    host_create_for_test, host_destroy_for_test, host_dispatch_intent_for_test,
    host_read_snapshot_for_test, RnHostReasonCode, RnHostTestIntent,
};

static HOST_TEST_LOCK: Mutex<()> = Mutex::new(());

fn fixture_path(tag: &str) -> std::path::PathBuf {
    let path = tmp_dir(&format!("r1-rn-product-edit-{tag}")).join("project.json");
    let mut document = Document::new_current();
    let layer = document.layers.allocate("r1-layer").expect("layer");
    let track = document.track_ids.allocate("r1-track").expect("track");
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

fn legacy_intent(kind: &str) -> RnHostTestIntent {
    RnHostTestIntent {
        kind: kind.to_owned(),
        stage_handle: None,
        projection_generation: None,
        width: None,
        height: None,
        scale_factor: None,
        focused: None,
    }
}

fn place_json(host: u64, playhead: RationalTime) -> String {
    serde_json::json!({
        "version": 1,
        "direction": "rn-to-host",
        "kind": "place_rectangle",
        "host_handle": host.to_string(),
        "position": [0.25, -0.125],
        "playhead": serde_json::to_value(playhead).expect("playhead json"),
    })
    .to_string()
}

#[test]
fn rn_place_undo_redo_are_durable_document_edits() {
    let _lock = HOST_TEST_LOCK.lock().expect("host test lock");
    let path = fixture_path("oracle");
    let journal = journal_path_for_document(&path);
    let host = host_create_for_test(&path).expect("create host");
    let baseline = host_read_snapshot_for_test(host).expect("baseline");
    let initial_journal = std::fs::metadata(&journal).expect("initial journal").len();

    let placed = host_dispatch_intent_for_test(
        host,
        serde_json::from_str::<serde_json::Value>(&place_json(
            host,
            RationalTime::try_new(1, 1).expect("playhead"),
        ))
        .expect("place payload"),
    )
    .expect("place dispatch");
    assert!(placed.accepted);
    let placed = placed.snapshot.expect("placed snapshot");
    assert_eq!(placed.layer_ids.len(), baseline.layer_ids.len() + 1);
    assert_ne!(placed.revision, baseline.revision);
    let place_journal = std::fs::metadata(&journal).expect("place journal").len();
    assert!(place_journal > initial_journal);

    let undone = host_dispatch_intent_for_test(host, legacy_intent("undo")).expect("undo dispatch");
    assert!(undone.accepted);
    let undone = undone.snapshot.expect("undo snapshot");
    assert_eq!(undone.layer_ids, baseline.layer_ids);
    assert_ne!(undone.revision, placed.revision);
    let undo_journal = std::fs::metadata(&journal).expect("undo journal").len();
    assert!(undo_journal > place_journal);

    let redone = host_dispatch_intent_for_test(host, legacy_intent("redo")).expect("redo dispatch");
    assert!(redone.accepted);
    let redone = redone.snapshot.expect("redo snapshot");
    assert_eq!(redone.layer_ids, placed.layer_ids);
    assert_ne!(redone.revision, undone.revision);
    let redo_journal = std::fs::metadata(&journal).expect("redo journal").len();
    assert!(redo_journal > undo_journal);

    host_destroy_for_test(host).expect("destroy host");
}

#[test]
fn rn_edit_intents_reject_invalid_wire_without_panicking() {
    let _lock = HOST_TEST_LOCK.lock().expect("host test lock");
    let path = fixture_path("reject");
    let host = host_create_for_test(&path).expect("create host");
    let baseline = host_read_snapshot_for_test(host).expect("baseline");

    for payload in [
        serde_json::json!({
            "version": 1, "direction": "rn-to-host", "kind": "place_rectangle",
            "host_handle": host.to_string(), "position": [0.0, 0.0]
        }),
        serde_json::json!({
            "version": 1, "direction": "rn-to-host", "kind": "place_rectangle",
            "host_handle": host.to_string(), "position": ["bad", 0.0], "playhead": 0
        }),
    ] {
        let response = host_dispatch_intent_for_test(host, payload).expect("invalid dispatch");
        assert!(!response.accepted);
        assert_eq!(response.reason, Some(RnHostReasonCode::InvalidIntent));
    }

    for kind in ["unknown", "undo"] {
        let response =
            host_dispatch_intent_for_test(host, legacy_intent(kind)).expect("negative dispatch");
        assert!(!response.accepted);
        assert_eq!(response.reason, Some(RnHostReasonCode::InvalidIntent));
    }
    let after = host_read_snapshot_for_test(host).expect("after");
    assert_eq!(after, baseline);
    host_destroy_for_test(host).expect("destroy host");
}
