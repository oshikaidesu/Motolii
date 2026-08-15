use super::super::*;
use super::fixtures::*;

#[test]
fn selection_actions_publish_only_accepted_primary_changes() {
    let (document, _) = fixture();
    let selected = fixture_layer(&document);
    let initial_json = serde_json::to_vec(&document).unwrap();
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();

    queue.push_replace_primary(selected);
    let selected_publish = runtime
        .process_next(&mut queue, None, 7)
        .unwrap()
        .expect("live target selection publishes");
    assert_eq!(
        selected_publish.kind,
        DocumentEditActionKind::ReplacePrimary
    );
    assert_eq!(selected_publish.primary, Some(selected));
    assert_eq!(selected_publish.projection_generation, 8);
    assert_eq!(selected_publish.revision, 0);

    queue.push_replace_primary(selected);
    assert!(runtime
        .process_next(&mut queue, Some(selected), u64::MAX)
        .unwrap()
        .is_none());

    queue.push_clear_primary();
    let clear_publish = runtime
        .process_next(&mut queue, Some(selected), 8)
        .unwrap()
        .expect("non-empty primary clear publishes");
    assert_eq!(clear_publish.kind, DocumentEditActionKind::ClearPrimary);
    assert_eq!(clear_publish.primary, None);
    assert_eq!(clear_publish.projection_generation, 9);
    assert_eq!(clear_publish.revision, 0);

    queue.push_clear_primary();
    assert!(runtime
        .process_next(&mut queue, None, u64::MAX)
        .unwrap()
        .is_none());

    assert_eq!(queue.len(), 0);
    assert_eq!(runtime.revision(), 0);
    assert_eq!(runtime.history_lengths(), (0, 0));
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        initial_json
    );
}

#[test]
fn nonexistent_selection_rejects_before_same_id_or_generation_preflight() {
    let (mut document, _) = fixture();
    let table_only = document.layers.allocate("table-only").unwrap();
    let initial_json = serde_json::to_vec(&document).unwrap();
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_replace_primary(table_only);

    assert!(matches!(
        runtime.process_next(&mut queue, Some(table_only), u64::MAX),
        Err(DocumentEditRuntimeError::SelectionTargetNotFound(target))
            if target == table_only
    ));
    assert_eq!(queue.len(), 0);
    assert_eq!(runtime.revision(), 0);
    assert_eq!(runtime.history_lengths(), (0, 0));
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        initial_json
    );
}

#[test]
fn accepted_selection_change_rejects_generation_exhaustion_without_mutation() {
    let (document, _) = fixture();
    let selected = fixture_layer(&document);
    let initial_json = serde_json::to_vec(&document).unwrap();
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_replace_primary(selected);

    assert!(matches!(
        runtime.process_next(&mut queue, None, u64::MAX),
        Err(DocumentEditRuntimeError::ProjectionGenerationExhausted)
    ));
    assert_eq!(queue.len(), 0);
    assert_eq!(runtime.revision(), 0);
    assert_eq!(runtime.history_lengths(), (0, 0));
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        initial_json
    );
}

#[test]
fn toggle_visible_and_solo_write_envelope_flags_unknown_layer_writes_nothing() {
    let (document, _) = fixture();
    let layer = fixture_layer(&document);
    let initial_json = serde_json::to_vec(&document).unwrap();
    let (_path, mut runtime) = open_runtime(document);
    let missing = LayerId::from_raw(999_999);

    let mut queue = DocumentEditQueue::default();
    queue.push_toggle_visible(missing);
    queue.push_toggle_solo(missing);
    assert!(matches!(
        runtime.process_next(&mut queue, Some(layer), 0),
        Err(DocumentEditRuntimeError::SelectionTargetNotFound(id)) if id == missing
    ));
    assert!(matches!(
        runtime.process_next(&mut queue, Some(layer), 0),
        Err(DocumentEditRuntimeError::SelectionTargetNotFound(id)) if id == missing
    ));
    assert_eq!(runtime.revision(), 0);
    assert_eq!(runtime.history_lengths(), (0, 0));
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        initial_json
    );

    queue.push_toggle_visible(layer);
    let muted = runtime
        .process_next(&mut queue, Some(layer), 0)
        .unwrap()
        .expect("mute publishes");
    let TrackItem::Clip(clip) = &muted.snapshot.tracks[0].items[0] else {
        panic!("fixture clip");
    };
    assert!(!clip.envelope.visible);
    assert!(!clip.envelope.solo);
    assert_eq!(muted.kind, DocumentEditActionKind::ToggleVisible);

    queue.push_toggle_solo(layer);
    let soloed = runtime
        .process_next(&mut queue, Some(layer), muted.projection_generation)
        .unwrap()
        .expect("solo publishes");
    let TrackItem::Clip(clip) = &soloed.snapshot.tracks[0].items[0] else {
        panic!("fixture clip");
    };
    assert!(!clip.envelope.visible);
    assert!(clip.envelope.solo);
    assert_eq!(soloed.kind, DocumentEditActionKind::ToggleSolo);
}

#[test]
fn sidecar_absent_before_initialized_project_open() {
    let dir = unique_tmp("cu109-no-sidecar");
    let path = dir.join("missing.json");
    let limits = ResourceLimits::production();
    assert!(ProjectSession::open(&path, &limits).is_err());
    assert!(!motolii_dir_for_document(&path).exists());
}
