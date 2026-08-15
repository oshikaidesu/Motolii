use super::super::*;
use super::fixtures::*;

#[test]
fn position_interp_queue_commits_once_and_same_value_is_a_durable_noop() {
    let (document, _) = fixture();
    let primary = fixture_layer(&document);
    let time = RationalTime::try_new(1, 2).unwrap();
    let (path, mut runtime) = open_runtime(document);
    let journal = journal_path_for_document(&path);
    let mut queue = DocumentEditQueue::default();
    queue.push_add_position_key(AddPositionKeyRequest {
        target: primary,
        time,
    });
    let keyed = runtime
        .process_next(&mut queue, Some(primary), 0)
        .unwrap()
        .expect("Position key must publish");
    let key = match &runtime
        .writer
        .find_envelope(primary)
        .unwrap()
        .transform
        .position
    {
        motolii_doc::DocParam::Keyframes(track) => {
            track.keys().iter().find(|key| key.t == time).unwrap().id
        }
        _ => unreachable!(),
    };
    let journal_before = fs::metadata(&journal).map(|meta| meta.len()).unwrap_or(0);
    queue.push_set_position_key_interp(SetPositionKeyInterpRequest {
        target: primary,
        key,
        interp: Interp::Bezier {
            x1: 0.4,
            y1: 0.0,
            x2: 0.2,
            y2: 1.0,
        },
    });
    let changed = runtime
        .process_next(&mut queue, Some(primary), keyed.projection_generation)
        .unwrap()
        .expect("changed interpolation must publish once");
    assert_eq!(changed.kind, DocumentEditActionKind::SetPositionKeyInterp);
    assert_eq!(runtime.history_lengths(), (2, 0));
    let journal_after_change = fs::metadata(&journal).unwrap().len();
    assert!(journal_after_change > journal_before);
    queue.push_set_position_key_interp(SetPositionKeyInterpRequest {
        target: primary,
        key,
        interp: Interp::Bezier {
            x1: 0.4,
            y1: 0.0,
            x2: 0.2,
            y2: 1.0,
        },
    });
    assert!(runtime
        .process_next(&mut queue, Some(primary), changed.projection_generation)
        .unwrap()
        .is_none());
    assert_eq!(runtime.history_lengths(), (2, 0));
    assert_eq!(fs::metadata(&journal).unwrap().len(), journal_after_change);
}

#[test]
fn position_value_queue_commits_one_key_locally_and_roundtrips_history() {
    let (document, _) = fixture();
    let primary = fixture_layer(&document);
    let time = RationalTime::try_new(1, 2).unwrap();
    let (path, mut runtime) = open_runtime(document);
    let journal = journal_path_for_document(&path);
    let mut queue = DocumentEditQueue::default();
    queue.push_add_position_key(AddPositionKeyRequest {
        target: primary,
        time,
    });
    let keyed = runtime
        .process_next(&mut queue, Some(primary), 0)
        .unwrap()
        .expect("Position key must publish");
    let (key, interp, counter) = match &runtime
        .writer
        .find_envelope(primary)
        .unwrap()
        .transform
        .position
    {
        motolii_doc::DocParam::Keyframes(track) => {
            let key = track.keys().iter().find(|key| key.t == time).unwrap();
            (
                key.id,
                key.interp,
                runtime.snapshot().next_stable_id.peek_next(),
            )
        }
        other => panic!("expected Position keyframes, got {other:?}"),
    };
    let before_value = serde_json::to_vec(&*runtime.snapshot()).unwrap();
    let journal_before = fs::metadata(&journal).unwrap().len();
    queue.push_set_position_key_value(SetPositionKeyValueRequest {
        target: primary,
        key,
        old: [0.0, 0.0],
        new: [0.4, -0.2],
    });
    let changed = runtime
        .process_next(&mut queue, Some(primary), keyed.projection_generation)
        .unwrap()
        .expect("Position value must publish once");
    assert_eq!(changed.kind, DocumentEditActionKind::SetPositionKeyValue);
    assert_eq!(changed.revision, 2);
    assert_eq!(runtime.history_lengths(), (2, 0));
    assert!(fs::metadata(&journal).unwrap().len() > journal_before);
    match &changed
        .snapshot
        .tracks
        .iter()
        .flat_map(|track| track.items.iter())
        .find_map(|item| match item {
            TrackItem::Clip(clip) if clip.envelope.layer_id == primary => {
                Some(&clip.envelope.transform.position)
            }
            TrackItem::Group(_) | TrackItem::Clip(_) => None,
        })
        .unwrap()
    {
        motolii_doc::DocParam::Keyframes(track) => {
            let edited = track.get_by_id(key).unwrap();
            assert_eq!(edited.value, motolii_doc::DocValue::Vec2([0.4, -0.2]));
            assert_eq!(edited.t, time);
            assert_eq!(edited.interp, interp);
        }
        other => panic!("expected Position keyframes after value edit, got {other:?}"),
    }
    assert_eq!(changed.snapshot.next_stable_id.peek_next(), counter);
    assert_ne!(
        serde_json::to_vec(&*changed.snapshot).unwrap(),
        before_value
    );

    let changed_json = serde_json::to_vec(&*changed.snapshot).unwrap();
    let journal_after_change = fs::metadata(&journal).unwrap().len();
    queue.push_set_position_key_value(SetPositionKeyValueRequest {
        target: primary,
        key,
        old: [0.0, 0.0],
        new: [0.4, -0.2],
    });
    assert!(runtime
        .process_next(&mut queue, Some(primary), changed.projection_generation)
        .unwrap()
        .is_none());
    assert_eq!(runtime.history_lengths(), (2, 0));
    assert_eq!(runtime.revision(), 2);
    assert_eq!(fs::metadata(&journal).unwrap().len(), journal_after_change);
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        changed_json
    );

    queue.push_undo();
    let undone = runtime
        .process_next(&mut queue, Some(primary), changed.projection_generation)
        .unwrap()
        .expect("Position value undo must publish");
    assert_eq!(undone.kind, DocumentEditActionKind::Undo);
    queue.push_redo();
    let redone = runtime
        .process_next(&mut queue, Some(primary), undone.projection_generation)
        .unwrap()
        .expect("Position value redo must publish");
    assert_eq!(redone.kind, DocumentEditActionKind::Redo);
    assert_eq!(serde_json::to_vec(&*redone.snapshot).unwrap(), changed_json);

    let stale_json = serde_json::to_vec(&*runtime.snapshot()).unwrap();
    let stale_revision = runtime.revision();
    let stale_history = runtime.history_lengths();
    let stale_journal = fs::metadata(&journal).unwrap().len();
    queue.push_set_position_key_value(SetPositionKeyValueRequest {
        target: primary,
        key,
        old: [9.0, 9.0],
        new: [0.8, 0.9],
    });
    assert!(matches!(
        runtime.process_next(&mut queue, Some(primary), redone.projection_generation),
        Err(DocumentEditRuntimeError::PrepareRejected)
    ));
    assert_eq!(runtime.revision(), stale_revision);
    assert_eq!(runtime.history_lengths(), stale_history);
    assert_eq!(fs::metadata(&journal).unwrap().len(), stale_journal);
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        stale_json
    );

    drop(runtime);
    let limits = ResourceLimits::production();
    let (_session, reopened) = ProjectSession::open(&path, &limits).unwrap();
    assert_eq!(
        serde_json::to_vec(&reopened.document).unwrap(),
        changed_json
    );
}

#[test]
fn position_interp_d2_changes_only_the_requested_left_key_outgoing_interp() {
    let (mut document, _) = fixture();
    let primary = fixture_layer(&document);
    let left = motolii_doc::KeyframeId::from_raw(document.next_stable_id.allocate().unwrap());
    let right = motolii_doc::KeyframeId::from_raw(document.next_stable_id.allocate().unwrap());
    let mut keys = motolii_doc::DocKeyframeTrack::new();
    keys.insert(motolii_doc::DocKeyframe {
        id: left,
        t: RationalTime::ZERO,
        value: motolii_doc::DocValue::Vec2([0.0, 0.0]),
        interp: Interp::Linear,
    });
    keys.insert(motolii_doc::DocKeyframe {
        id: right,
        t: RationalTime::from_seconds(1),
        value: motolii_doc::DocValue::Vec2([1.0, 1.0]),
        interp: Interp::Hold,
    });
    match &mut document.tracks[0].items[0] {
        TrackItem::Clip(clip) => {
            clip.envelope.transform.position = motolii_doc::DocParam::Keyframes(keys);
        }
        TrackItem::Group(_) => unreachable!("fixture begins with a clip"),
    }
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    let replacement = Interp::Bezier {
        x1: 0.4,
        y1: 0.0,
        x2: 0.2,
        y2: 1.0,
    };
    queue.push_set_position_key_interp(SetPositionKeyInterpRequest {
        target: primary,
        key: left,
        interp: replacement,
    });

    assert_eq!(
        runtime
            .process_next(&mut queue, Some(primary), 0)
            .unwrap()
            .unwrap()
            .kind,
        DocumentEditActionKind::SetPositionKeyInterp,
    );
    let track = match &runtime
        .writer
        .find_envelope(primary)
        .unwrap()
        .transform
        .position
    {
        motolii_doc::DocParam::Keyframes(track) => track,
        other => {
            panic!("expected Position keyframes after interpolation change, got {other:?}")
        }
    };
    assert_eq!(
        track
            .keys()
            .iter()
            .find(|key| key.id == left)
            .unwrap()
            .interp,
        replacement,
    );
    assert_eq!(
        track
            .keys()
            .iter()
            .find(|key| key.id == right)
            .unwrap()
            .interp,
        Interp::Hold,
    );
}

#[test]
fn add_position_key_product_negatives_are_noops_before_any_durable_write() {
    let time = RationalTime::try_new(1, 2).unwrap();

    let (document, _) = fixture();
    let primary = fixture_layer(&document);
    assert_add_position_key_noop(
        document,
        None,
        AddPositionKeyRequest {
            target: primary,
            time,
        },
    );

    let (document, _) = fixture();
    let primary = fixture_layer(&document);
    assert_add_position_key_noop(
        document,
        Some(LayerId::from_raw(u64::MAX)),
        AddPositionKeyRequest {
            target: primary,
            time,
        },
    );

    let (document, _) = fixture();
    let missing = LayerId::from_raw(u64::MAX);
    assert_add_position_key_noop(
        document,
        Some(missing),
        AddPositionKeyRequest {
            target: missing,
            time,
        },
    );

    let (mut document, _) = fixture();
    let primary = fixture_layer(&document);
    match &mut document.tracks[0].items[0] {
        TrackItem::Clip(clip) => {
            clip.envelope.transform.position = motolii_doc::DocParam::Vec2Axes {
                x: Box::new(motolii_doc::DocParam::const_f64(0.0)),
                y: Box::new(motolii_doc::DocParam::const_f64(0.0)),
            };
        }
        TrackItem::Group(_) => unreachable!("fixture is a clip"),
    }
    assert_add_position_key_noop(
        document,
        Some(primary),
        AddPositionKeyRequest {
            target: primary,
            time,
        },
    );
}

#[test]
fn add_position_key_type_mismatch_is_a_noop_without_a_second_runtime_mutation() {
    let (document, _) = fixture();
    let primary = fixture_layer(&document);
    let (path, mut runtime) = open_runtime(document);
    runtime
        .writer
        .edit(|document| match &mut document.tracks[0].items[0] {
            TrackItem::Clip(clip) => {
                clip.envelope.transform.position = motolii_doc::DocParam::const_f64(1.0);
            }
            TrackItem::Group(_) => unreachable!("fixture is a clip"),
        });
    let initial_json = serde_json::to_vec(&*runtime.snapshot()).unwrap();
    let initial_revision = runtime.revision();
    let initial_stable = runtime.snapshot().next_stable_id.peek_next();
    let journal = journal_path_for_document(&path);
    let journal_size = fs::metadata(&journal).map(|meta| meta.len()).unwrap_or(0);
    let mut queue = DocumentEditQueue::default();
    queue.push_add_position_key(AddPositionKeyRequest {
        target: primary,
        time: RationalTime::try_new(1, 2).unwrap(),
    });

    assert!(matches!(
        runtime.process_next(&mut queue, Some(primary), u64::MAX),
        Err(DocumentEditRuntimeError::PositionKeyPrepare(_))
    ));
    assert_preflight_rejection_invariants(
        &runtime,
        &queue,
        &initial_json,
        initial_revision,
        (0, 0),
    );
    assert_eq!(
        runtime.snapshot().next_stable_id.peek_next(),
        initial_stable
    );
    assert_eq!(
        fs::metadata(&journal).map(|meta| meta.len()).unwrap_or(0),
        journal_size
    );
}
