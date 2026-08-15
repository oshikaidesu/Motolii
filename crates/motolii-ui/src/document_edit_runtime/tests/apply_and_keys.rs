use super::super::*;
use super::fixtures::*;

#[test]
fn apply_publishes_once_and_drains_queue() {
    let (document, request) = fixture();
    let initial_json = serde_json::to_vec(&document).unwrap();
    let (_path, mut runtime) = open_runtime(document);
    let initial_snapshot = runtime.snapshot();
    let mut queue = DocumentEditQueue::default();
    queue.push_prepared(delete_output(), Some(request)).unwrap();

    let applied = runtime.process_next(&mut queue, None, 0).unwrap().unwrap();
    let applied_json = serde_json::to_vec(&*applied.snapshot).unwrap();
    assert_eq!(applied.kind, DocumentEditActionKind::Apply);
    assert_eq!(applied.revision, 1);
    assert_eq!(runtime.history_lengths(), (1, 0));
    assert_ne!(applied_json, initial_json);
    assert_eq!(
        serde_json::to_vec(&*initial_snapshot).unwrap(),
        initial_json
    );
    assert_eq!(queue.len(), 0);
    assert_eq!(applied.projection_generation, 1);
}

#[test]
fn add_position_key_commits_at_request_time_and_preserves_the_key_id_through_history() {
    let (document, _) = fixture();
    let primary = fixture_layer(&document);
    let expected_key_id = motolii_doc::KeyframeId::from_raw(document.next_stable_id.peek_next());
    let time = RationalTime::try_new(1, 2).unwrap();
    let (path, mut runtime) = open_runtime(document);
    let journal = journal_path_for_document(&path);
    let journal_before = fs::metadata(&journal).map(|meta| meta.len()).unwrap_or(0);
    let mut queue = DocumentEditQueue::default();
    let request = AddPositionKeyRequest {
        target: primary,
        time,
    };
    queue.push_add_position_key(request);

    let published = runtime
        .process_next(&mut queue, Some(primary), 4)
        .unwrap()
        .expect("new Position key must publish once");
    assert_eq!(published.kind, DocumentEditActionKind::AddPositionKey);
    assert_eq!(published.revision, 1);
    assert_eq!(published.primary, Some(primary));
    assert_eq!(published.projection_generation, 5);
    assert_eq!(runtime.history_lengths(), (1, 0));
    assert!(fs::metadata(&journal).unwrap().len() > journal_before);

    let key_id = match &runtime
        .writer
        .find_envelope(primary)
        .expect("primary envelope")
        .transform
        .position
    {
        motolii_doc::DocParam::Keyframes(track) => {
            let key = track
                .keys()
                .iter()
                .find(|key| key.t == time)
                .expect("request time key");
            assert_eq!(key.value, motolii_doc::DocValue::Vec2([0.0, 0.0]));
            key.id
        }
        other => panic!("expected Position keyframes, got {other:?}"),
    };
    assert_eq!(key_id, expected_key_id);

    let after_first_json = serde_json::to_vec(&*runtime.snapshot()).unwrap();
    let after_first_revision = runtime.revision();
    let after_first_history = runtime.history_lengths();
    let after_first_journal = fs::metadata(&journal).unwrap().len();
    queue.push_add_position_key(request);
    assert!(runtime
        .process_next(&mut queue, Some(primary), u64::MAX)
        .unwrap()
        .is_none());
    assert_eq!(runtime.revision(), after_first_revision);
    assert_eq!(runtime.history_lengths(), after_first_history);
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        after_first_json
    );
    assert_eq!(fs::metadata(&journal).unwrap().len(), after_first_journal);

    queue.push_undo();
    let undone = runtime
        .process_next(&mut queue, Some(primary), 5)
        .unwrap()
        .expect("Position key undo must publish");
    assert_eq!(undone.kind, DocumentEditActionKind::Undo);
    queue.push_redo();
    let redone = runtime
        .process_next(&mut queue, Some(primary), 6)
        .unwrap()
        .expect("Position key redo must publish");
    assert_eq!(redone.kind, DocumentEditActionKind::Redo);
    let redone_key_id = match &runtime
        .writer
        .find_envelope(primary)
        .expect("primary envelope")
        .transform
        .position
    {
        motolii_doc::DocParam::Keyframes(track) => {
            track
                .keys()
                .iter()
                .find(|key| key.t == time)
                .expect("redone request time key")
                .id
        }
        other => panic!("expected Position keyframes after redo, got {other:?}"),
    };
    assert_eq!(redone_key_id, key_id);
}

#[test]
fn add_transform_param_key_commits_const_scale_to_keyframes() {
    let (document, _) = fixture();
    let primary = fixture_layer(&document);
    let time = RationalTime::try_new(1, 2).unwrap();
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_add_transform_param_key(AddTransformParamKeyRequest {
        target: primary,
        property: ScalarPropertyId::Scale,
        time,
    });

    let published = runtime
        .process_next(&mut queue, Some(primary), 0)
        .unwrap()
        .expect("const Scale must publish a key");
    assert_eq!(published.kind, DocumentEditActionKind::AddTransformParamKey);
    match &runtime
        .writer
        .find_envelope(primary)
        .expect("primary envelope")
        .transform
        .scale
    {
        motolii_doc::DocParam::Keyframes(track) => {
            let key = track
                .keys()
                .iter()
                .find(|key| key.t == time)
                .expect("request time Scale key");
            assert_eq!(key.value, motolii_doc::DocValue::Vec2([1.0, 1.0]));
        }
        other => panic!("expected Scale keyframes, got {other:?}"),
    }

    queue.push_add_transform_param_key(AddTransformParamKeyRequest {
        target: primary,
        property: ScalarPropertyId::Scale,
        time,
    });
    assert!(runtime
        .process_next(&mut queue, Some(primary), published.projection_generation)
        .unwrap()
        .is_none());
}

#[test]
fn add_transform_param_key_without_primary_is_no_primary_selection() {
    let (document, _) = fixture();
    let primary = fixture_layer(&document);
    let initial_json = serde_json::to_vec(&document).unwrap();
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_add_transform_param_key(AddTransformParamKeyRequest {
        target: primary,
        property: ScalarPropertyId::Scale,
        time: RationalTime::try_new(1, 2).unwrap(),
    });
    assert!(matches!(
        runtime.process_next(&mut queue, None, 0),
        Err(DocumentEditRuntimeError::NoPrimarySelection)
    ));
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        initial_json
    );
    assert_eq!(runtime.revision(), 0);
    assert_eq!(runtime.history_lengths(), (0, 0));
}

#[test]
fn set_opacity_on_keyframes_updates_on_key_and_rejects_off_key() {
    let (document, _) = fixture();
    let primary = fixture_layer(&document);
    let time = RationalTime::try_new(1, 2).unwrap();
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_add_transform_param_key(AddTransformParamKeyRequest {
        target: primary,
        property: ScalarPropertyId::Opacity,
        time,
    });
    let keyed = runtime
        .process_next(&mut queue, Some(primary), 0)
        .unwrap()
        .expect("opacity key");

    queue.push_set_opacity_at(
        SetOpacityRequest {
            target: primary,
            value: 0.25,
        },
        time,
    );
    let changed = runtime
        .process_next(&mut queue, Some(primary), keyed.projection_generation)
        .unwrap()
        .expect("on-key opacity dial must write");
    match &changed
        .snapshot
        .tracks
        .iter()
        .flat_map(|track| track.items.iter())
        .find_map(|item| match item {
            TrackItem::Clip(clip) if clip.envelope.layer_id == primary => {
                Some(&clip.envelope.opacity)
            }
            _ => None,
        })
        .expect("primary opacity")
    {
        motolii_doc::DocParam::Keyframes(track) => {
            let key = track
                .keys()
                .iter()
                .find(|key| key.t == time)
                .expect("opacity key at request time");
            assert_eq!(key.value, motolii_doc::DocValue::F64(0.25));
            assert_eq!(track.keys().len(), 1);
        }
        other => panic!("opacity dial must not collapse Keyframes, got {other:?}"),
    }

    let after = serde_json::to_vec(&*runtime.snapshot()).unwrap();
    queue.push_set_opacity_at(
        SetOpacityRequest {
            target: primary,
            value: 0.5,
        },
        RationalTime::ZERO,
    );
    assert!(matches!(
        runtime.process_next(&mut queue, Some(primary), changed.projection_generation),
        Err(DocumentEditRuntimeError::PrepareRejected)
    ));
    assert_eq!(serde_json::to_vec(&*runtime.snapshot()).unwrap(), after);
    match &runtime.writer.find_envelope(primary).unwrap().opacity {
        motolii_doc::DocParam::Keyframes(_) => {}
        other => panic!("off-key must leave Keyframes, got {other:?}"),
    }
}
