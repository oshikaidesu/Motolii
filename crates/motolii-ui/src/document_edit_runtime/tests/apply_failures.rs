use super::super::*;
use super::fixtures::*;

#[test]
fn routed_history_commands_reconcile_selection_without_restoring_it_on_redo() {
    let (document, _) = fixture();
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_place_rectangle(PlaceRectangleRequest {
        position: [0.0, 0.0],
        playhead: RationalTime::ZERO,
    });
    let placed = runtime.process_next(&mut queue, None, 0).unwrap().unwrap();
    let selected = placed.primary.expect("Place selects the new Rectangle");

    queue
        .push_prepared(history_output(DomainIntent::Undo), None)
        .unwrap();
    let undone = runtime
        .process_next(&mut queue, Some(selected), 1)
        .unwrap()
        .unwrap();
    assert_eq!(undone.kind, DocumentEditActionKind::Undo);
    assert_eq!(undone.primary, None);
    assert!(runtime.writer.find_envelope(selected).is_none());

    queue
        .push_prepared(history_output(DomainIntent::Redo), None)
        .unwrap();
    let redone = runtime
        .process_next(&mut queue, undone.primary, 2)
        .unwrap()
        .unwrap();
    assert_eq!(redone.kind, DocumentEditActionKind::Redo);
    assert_eq!(redone.primary, None);
    assert!(runtime.writer.find_envelope(selected).is_some());
    assert_eq!(runtime.history_lengths(), (1, 0));
}

#[test]
fn missing_request_and_empty_history_publish_nothing() {
    let (document, _) = fixture();
    let initial_json = serde_json::to_vec(&document).unwrap();
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();

    assert_eq!(
        queue.push_prepared(delete_output(), None),
        Err(DocumentEditDispatchError::MissingPreparedRequest)
    );
    assert!(runtime.process_next(&mut queue, None, 0).unwrap().is_none());

    queue.push_undo();
    assert!(matches!(
        runtime.process_next(&mut queue, None, 0),
        Err(DocumentEditRuntimeError::NothingToUndo)
    ));
    assert_eq!(runtime.revision(), 0);
    assert_eq!(runtime.history_lengths(), (0, 0));
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        initial_json
    );
}

#[test]
fn failed_d2_action_is_consumed_without_snapshot_or_history_change() {
    let (document, _) = fixture();
    let initial_json = serde_json::to_vec(&document).unwrap();
    let track = document.tracks[0].id;
    let item = document.tracks[0].items[0].clone();
    let request = DocumentCommandRequest::try_new(
        DomainIntent::DeleteTargetedItems,
        vec![Command::RemoveTrackItem {
            parent: ParentLocator::Track(track),
            index: 1,
            layer_names: layer_names_for_item(&document, &item).unwrap(),
            item,
        }],
    )
    .unwrap();
    let (_path, mut runtime) = open_runtime(document);
    let pre_revision = runtime.revision();
    let pre_history = runtime.history_lengths();
    let mut queue = DocumentEditQueue::default();
    queue.push_prepared(delete_output(), Some(request)).unwrap();

    assert!(matches!(
        runtime.process_next(&mut queue, None, 0),
        Err(DocumentEditRuntimeError::Command(_))
    ));
    assert_preflight_rejection_invariants(
        &runtime,
        &queue,
        &initial_json,
        pre_revision,
        pre_history,
    );
}

#[test]
fn failed_validate_is_consumed_without_snapshot_or_history_change() {
    let (document, request) = dangling_parent_after_remove_fixture();
    let initial_json = serde_json::to_vec(&document).unwrap();
    let (_path, mut runtime) = open_runtime(document);
    let pre_revision = runtime.revision();
    let pre_history = runtime.history_lengths();
    let mut queue = DocumentEditQueue::default();
    queue.push_prepared(delete_output(), Some(request)).unwrap();

    let error = runtime
        .process_next(&mut queue, None, 0)
        .expect_err("dangling transform.parent must fail validate");
    assert!(matches!(
        error,
        DocumentEditRuntimeError::Document(DocumentError::UnknownLayerId { .. })
    ));
    assert_preflight_rejection_invariants(
        &runtime,
        &queue,
        &initial_json,
        pre_revision,
        pre_history,
    );
}

fn plugin_clip_survivor_fixture() -> (Document, DocumentCommandRequest) {
    let mut document = Document::new_current();
    let removable = document.layers.allocate("removable").unwrap();
    let plugin_layer = document.layers.allocate("plugin-survivor").unwrap();
    let track = document.track_ids.allocate("V1").unwrap();
    let asset = document
        .assets
        .allocate("media", "video/mp4", "hash")
        .unwrap();

    let removable_item = TrackItem::Clip(Clip {
        envelope: ItemEnvelope::new(removable),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(1, 1).unwrap(),
        time_map: Default::default(),
        source: ClipSource::asset_video_only(asset),
    });

    let plugin_item = TrackItem::Clip(Clip {
        envelope: ItemEnvelope::new(plugin_layer),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(1, 1).unwrap(),
        time_map: Default::default(),
        source: ClipSource::Plugin {
            plugin_id: "core.layer_source.radial_repeater".into(),
            effect_version: 1,
            params: BTreeMap::from([("count".into(), motolii_doc::DocParam::const_f64(12.0))]),
            extra: Default::default(),
        },
    });

    document.tracks.push(Track {
        id: track,
        items: vec![removable_item.clone(), plugin_item],
    });
    document.validate().unwrap();

    let request = DocumentCommandRequest::try_new(
        DomainIntent::DeleteTargetedItems,
        vec![Command::RemoveTrackItem {
            parent: ParentLocator::Track(track),
            index: 0,
            layer_names: layer_names_for_item(&document, &removable_item).unwrap(),
            item: removable_item,
        }],
    )
    .unwrap();

    (document, request)
}

#[test]
fn failed_prepare_plugins_is_consumed_without_snapshot_or_history_change() {
    use motolii_plugin::PluginCatalogBuilder;

    let (document, request) = plugin_clip_survivor_fixture();
    let initial_json = serde_json::to_vec(&document).unwrap();
    // CU-109 製品配線では writer と runtime が同一 catalog を共有するため到達不能。
    // prepare_plugins 前方ガードの単体試験としてだけ catalog 不一致を runtime に渡す。
    let mut mismatched_contract = first_party_catalog()
        .get("core.layer_source.radial_repeater")
        .expect("radial repeater contract")
        .clone();
    mismatched_contract
        .node
        .params
        .retain(|param| param.id != "count");
    let mut builder = PluginCatalogBuilder::new();
    builder
        .register(mismatched_contract)
        .expect("mismatched test contract");
    let mismatched_catalog = Arc::new(builder.build().expect("mismatched catalog"));
    let (_path, mut runtime) = open_runtime_with_catalog(document, mismatched_catalog);
    let pre_revision = runtime.revision();
    let pre_history = runtime.history_lengths();
    let mut queue = DocumentEditQueue::default();
    queue.push_prepared(delete_output(), Some(request)).unwrap();

    let error = runtime
        .process_next(&mut queue, None, 0)
        .expect_err("mismatched runtime catalog must fail prepare_plugins");
    assert!(matches!(
        error,
        DocumentEditRuntimeError::DocumentPlugin(
            DocumentPluginError::ContractViolation { ref param, .. }
        ) if param == "count"
    ));
    assert_preflight_rejection_invariants(
        &runtime,
        &queue,
        &initial_json,
        pre_revision,
        pre_history,
    );
}

#[test]
fn empty_queue_publishes_nothing_and_does_not_advance_generation() {
    let (document, _) = fixture();
    let layer = fixture_layer(&document);
    let initial_json = serde_json::to_vec(&document).unwrap();
    let (_path, mut runtime) = open_runtime(document);
    let pre_revision = runtime.revision();
    let mut queue = DocumentEditQueue::default();

    assert!(runtime
        .process_next(&mut queue, Some(layer), 5)
        .unwrap()
        .is_none());
    assert_eq!(runtime.revision(), pre_revision);
    assert_eq!(runtime.history_lengths(), (0, 0));
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        initial_json
    );
    assert_eq!(queue.len(), 0);
}

#[test]
fn diagnostic_adaptation_does_not_enqueue_or_mutate_document_runtime() {
    let (document, _) = fixture();
    let initial_json = serde_json::to_vec(&document).unwrap();
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();

    let envelope =
        adapt_document_command_request_error(&DocumentCommandRequestError::EmptyCommands);

    assert_eq!(
        envelope.reason(),
        DiagnosticReasonCode::EmptyDocumentCommands
    );
    assert!(runtime.process_next(&mut queue, None, 0).unwrap().is_none());
    assert_eq!(queue.len(), 0);
    assert_eq!(runtime.revision(), 0);
    assert_eq!(runtime.history_lengths(), (0, 0));
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        initial_json
    );
}

#[test]
fn target_and_preview_cancel_leave_edit_and_render_delivery_unchanged() {
    for with_preview in [false, true] {
        let (document, _) = fixture();
        let initial_json = serde_json::to_vec(&document).unwrap();
        let (_path, mut runtime) = open_runtime(document);
        let mut queue = DocumentEditQueue::default();
        let mut machine = InteractionStateMachine::new();
        machine.transition(InteractionState::Target).unwrap();
        if with_preview {
            machine.transition(InteractionState::Preview).unwrap();
        }

        machine.transition(InteractionState::Cancel).unwrap();
        machine.transition(InteractionState::Discover).unwrap();

        assert!(runtime.process_next(&mut queue, None, 0).unwrap().is_none());
        assert_eq!(queue.len(), 0);
        assert_eq!(runtime.revision(), 0);
        assert_eq!(runtime.history_lengths(), (0, 0));
        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            initial_json
        );
    }
}

#[test]
fn apply_success_retains_valid_primary_and_advances_generation_once() {
    let f = two_track_fixture();
    let surviving = f.surviving;
    let (_path, mut runtime) = open_runtime(f.document);
    let mut queue = DocumentEditQueue::default();
    queue
        .push_prepared(delete_output(), Some(f.delete_request))
        .unwrap();

    let applied = runtime
        .process_next(&mut queue, Some(surviving), 7)
        .unwrap()
        .unwrap();
    assert_eq!(applied.kind, DocumentEditActionKind::Apply);
    assert_eq!(applied.revision, 1);
    assert_eq!(applied.primary, Some(surviving));
    assert_eq!(applied.projection_generation, 8);
    assert_eq!(runtime.history_lengths(), (1, 0));
    assert_eq!(queue.len(), 0);
}

#[test]
fn apply_success_clears_primary_deleted_by_the_apply() {
    let (document, request) = fixture();
    let deleted = fixture_layer(&document);
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_prepared(delete_output(), Some(request)).unwrap();

    let applied = runtime
        .process_next(&mut queue, Some(deleted), 7)
        .unwrap()
        .unwrap();
    assert_eq!(applied.kind, DocumentEditActionKind::Apply);
    assert_eq!(applied.revision, 1);
    assert_eq!(applied.primary, None);
    assert_eq!(applied.projection_generation, 8);
    assert!(runtime.writer.find_envelope(deleted).is_none());
    assert_eq!(runtime.history_lengths(), (1, 0));
}

#[test]
fn generation_at_max_minus_one_advances_to_max() {
    let (document, request) = fixture();
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_prepared(delete_output(), Some(request)).unwrap();

    let published = runtime
        .process_next(&mut queue, None, u64::MAX - 1)
        .unwrap()
        .unwrap();
    assert_eq!(published.kind, DocumentEditActionKind::Apply);
    assert_eq!(published.projection_generation, u64::MAX);
}

#[test]
fn exhausted_generation_refuses_before_mutation_and_consumes_action() {
    let f = two_track_fixture();
    let (_path, mut runtime) = open_runtime(f.document.clone());
    let added_layer = runtime.writer.reserve_layer_id().unwrap();
    let item = TrackItem::Clip(Clip {
        envelope: ItemEnvelope::new(added_layer),
        start: RationalTime::try_new(1, 1).unwrap(),
        duration: RationalTime::try_new(1, 1).unwrap(),
        time_map: Default::default(),
        source: ClipSource::asset_video_only(f.asset),
    });
    let mut layer_names = BTreeMap::new();
    layer_names.insert(added_layer, "seeded".to_string());
    runtime
        .writer
        .apply_macro(vec![Command::AddTrackItem {
            parent: ParentLocator::Track(f.surviving_track),
            index: 1,
            item,
            layer_names,
        }])
        .unwrap();
    assert_eq!(runtime.revision(), 1);

    for action in ["apply", "undo"] {
        let mut queue = DocumentEditQueue::default();
        match action {
            "apply" => queue
                .push_prepared(
                    delete_output(),
                    Some(two_track_delete_request(&f.document, f.deleted)),
                )
                .unwrap(),
            "undo" => queue.push_undo(),
            _ => unreachable!(),
        }

        let initial_json = serde_json::to_vec(&*runtime.snapshot()).unwrap();
        let pre_revision = runtime.revision();
        let pre_history = runtime.history_lengths();

        match runtime.process_next(&mut queue, Some(added_layer), u64::MAX) {
            Err(DocumentEditRuntimeError::ProjectionGenerationExhausted) => {}
            result => panic!("unexpected result: {result:?}"),
        }
        assert_eq!(queue.len(), 0);
        assert_eq!(runtime.revision(), pre_revision);
        assert_eq!(runtime.history_lengths(), pre_history);
        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            initial_json
        );
    }
}

#[test]
fn remaining_queue_drains_after_exhaustion_refusal() {
    let f = two_track_fixture();
    let (_path, mut runtime) = open_runtime(f.document.clone());
    let added_layer = runtime.writer.reserve_layer_id().unwrap();
    let item = TrackItem::Clip(Clip {
        envelope: ItemEnvelope::new(added_layer),
        start: RationalTime::try_new(1, 1).unwrap(),
        duration: RationalTime::try_new(1, 1).unwrap(),
        time_map: Default::default(),
        source: ClipSource::asset_video_only(f.asset),
    });
    let mut layer_names = BTreeMap::new();
    layer_names.insert(added_layer, "seeded".to_string());
    runtime
        .writer
        .apply_macro(vec![Command::AddTrackItem {
            parent: ParentLocator::Track(f.surviving_track),
            index: 1,
            item,
            layer_names,
        }])
        .unwrap();

    let mut queue = DocumentEditQueue::default();
    queue
        .push_prepared(
            delete_output(),
            Some(two_track_delete_request(&f.document, f.deleted)),
        )
        .unwrap();
    queue.push_undo();

    assert!(matches!(
        runtime.process_next(&mut queue, None, u64::MAX),
        Err(DocumentEditRuntimeError::ProjectionGenerationExhausted)
    ));
    assert_eq!(queue.len(), 1);

    assert!(matches!(
        runtime.process_next(&mut queue, None, 4),
        Err(DocumentEditRuntimeError::HistoryProjectionMismatch)
    ));
    assert_eq!(queue.len(), 0);
    assert_eq!(runtime.revision(), 1);
}

#[test]
fn multi_command_apply_is_rejected_pre_mutation() {
    let (document, _) = fixture();
    let initial_json = serde_json::to_vec(&document).unwrap();
    let track = document.tracks[0].id;
    let item = document.tracks[0].items[0].clone();
    let request = DocumentCommandRequest::try_new(
        DomainIntent::DeleteTargetedItems,
        vec![
            Command::RemoveTrackItem {
                parent: ParentLocator::Track(track),
                index: 0,
                layer_names: layer_names_for_item(&document, &item).unwrap(),
                item: item.clone(),
            },
            Command::RemoveTrackItem {
                parent: ParentLocator::Track(track),
                index: 0,
                layer_names: layer_names_for_item(&document, &item).unwrap(),
                item,
            },
        ],
    )
    .unwrap();
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_prepared(delete_output(), Some(request)).unwrap();

    assert!(matches!(
        runtime.process_next(&mut queue, None, 0),
        Err(DocumentEditRuntimeError::MultiCommandActionRejected)
    ));
    assert_eq!(runtime.revision(), 0);
    assert!(!runtime.is_write_blocked());
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        initial_json
    );
}
