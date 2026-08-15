use super::super::*;
use super::fixtures::*;

#[test]
fn pre_replace_journal_failure_rejects_without_blocking_the_next_write() {
    let (document, request) = fixture();
    let initial_json = serde_json::to_vec(&document).unwrap();
    let (path, mut runtime) = open_runtime(document);
    let journal = journal_path_for_document(&path);
    let journal_before = fs::read(&journal).unwrap();
    drop(fs::remove_file(&journal));
    fs::create_dir_all(&journal).unwrap();

    let mut queue = DocumentEditQueue::default();
    queue.push_prepared(delete_output(), Some(request)).unwrap();

    assert!(matches!(
        runtime.process_next(&mut queue, None, 0),
        Err(DocumentEditRuntimeError::JournalCommit(_))
    ));
    assert!(!runtime.is_write_blocked());
    assert_eq!(runtime.revision(), 0);
    assert_eq!(runtime.history_lengths(), (0, 0));
    assert_eq!(queue.len(), 0);
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        initial_json
    );

    fs::remove_dir_all(&journal).unwrap();
    fs::write(&journal, journal_before).unwrap();
    let (_document, request) = fixture();
    queue.push_prepared(delete_output(), Some(request)).unwrap();
    let published = runtime
        .process_next(&mut queue, None, 0)
        .unwrap()
        .expect("retry after a rejected pre-replace failure");
    assert_eq!(published.revision, 1);
    assert!(!runtime.is_write_blocked());
}

#[test]
fn durable_commit_is_reconciled_into_the_same_live_writer() {
    let f = two_track_fixture();
    let initial_json = serde_json::to_vec(&f.document).unwrap();
    let (_path, mut runtime) = open_runtime(f.document);
    runtime.set_test_failpoint(RuntimeTestFailpoint::DeferAfterDurableCommit);

    let mut queue = DocumentEditQueue::default();
    queue
        .push_prepared(delete_output(), Some(f.delete_request))
        .unwrap();

    assert!(matches!(
        runtime.process_next(&mut queue, None, 0),
        Err(DocumentEditRuntimeError::DocumentWriteBlocked { .. })
    ));
    assert!(runtime.is_write_blocked());
    assert_eq!(runtime.revision(), 0);
    assert_eq!(runtime.history_lengths(), (0, 0));
    assert_eq!(queue.len(), 0);
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        initial_json
    );

    let published = runtime
        .reconcile_pending_commit()
        .unwrap()
        .expect("same-session reconciliation publishes the durable candidate");
    assert_eq!(published.revision, 1);
    assert!(!runtime.is_write_blocked());
    assert_eq!(runtime.history_lengths(), (1, 0));
}

#[test]
fn reconcile_successful_receipt_not_observed_stays_write_blocked() {
    let f = two_track_fixture();
    let (path, mut runtime) = open_runtime(f.document);
    let journal = journal_path_for_document(&path);
    let checkpoint_journal = fs::read(&journal).unwrap();
    runtime.set_test_failpoint(RuntimeTestFailpoint::DeferAfterDurableCommit);

    let mut queue = DocumentEditQueue::default();
    queue
        .push_prepared(delete_output(), Some(f.delete_request))
        .unwrap();
    assert!(matches!(
        runtime.process_next(&mut queue, None, 0),
        Err(DocumentEditRuntimeError::DocumentWriteBlocked { .. })
    ));

    fs::write(&journal, checkpoint_journal).unwrap();
    assert!(matches!(
        runtime.reconcile_pending_commit(),
        Err(DocumentEditRuntimeError::CommitReceiptNotObserved { .. })
    ));
    assert!(runtime.is_write_blocked());
    assert_eq!(runtime.revision(), 0);
}

#[test]
fn reconcile_failure_blocks_only_writes_and_is_retriable() {
    let f = two_track_fixture();
    let initial_json = serde_json::to_vec(&f.document).unwrap();
    let (_path, mut runtime) = open_runtime(f.document);
    runtime.set_test_failpoint(RuntimeTestFailpoint::DeferAfterDurableCommit);

    let mut queue = DocumentEditQueue::default();
    queue
        .push_prepared(delete_output(), Some(f.delete_request))
        .unwrap();
    let _ = runtime.process_next(&mut queue, None, 0);
    runtime.fail_reconcile_for_test();
    assert!(matches!(
        runtime.reconcile_pending_commit(),
        Err(DocumentEditRuntimeError::DocumentWriteBlocked { .. })
    ));
    assert!(runtime.is_write_blocked());
    assert_eq!(runtime.revision(), 0);
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        initial_json
    );

    queue.push_replace_primary(f.surviving);
    let selection = runtime
        .process_next(&mut queue, None, 0)
        .unwrap()
        .expect("selection remains live while writes are blocked");
    assert_eq!(selection.primary, Some(f.surviving));
    assert_eq!(selection.revision, 0);
    assert!(runtime.is_write_blocked());

    queue.push_clear_primary();
    let cleared = runtime
        .process_next(&mut queue, Some(f.surviving), 1)
        .unwrap()
        .expect("selection clear remains live while writes are blocked");
    assert_eq!(cleared.primary, None);

    queue.push_replace_primary(f.surviving);
    let latest_selection = runtime
        .process_next(&mut queue, None, 2)
        .unwrap()
        .expect("latest selection remains live while writes are blocked");
    assert_eq!(latest_selection.primary, Some(f.surviving));
    assert_eq!(latest_selection.projection_generation, 3);

    let published = runtime
        .reconcile_pending_commit()
        .unwrap()
        .expect("reconciliation retry");
    assert_eq!(published.revision, 1);
    assert_eq!(published.primary, Some(f.surviving));
    assert_eq!(published.projection_generation, 3);
    assert!(!runtime.is_write_blocked());
}

#[test]
fn write_block_does_not_consume_the_next_action() {
    let f = two_track_fixture();
    let (_path, mut runtime) = open_runtime(f.document);
    runtime.set_test_failpoint(RuntimeTestFailpoint::DeferAfterDurableCommit);
    let mut queue = DocumentEditQueue::default();
    queue
        .push_prepared(delete_output(), Some(f.delete_request))
        .unwrap();
    let _ = runtime.process_next(&mut queue, None, 0);
    assert!(runtime.is_write_blocked());
    let request2 = two_track_delete_request(runtime.snapshot().as_ref(), f.surviving);
    queue
        .push_prepared(delete_output(), Some(request2))
        .unwrap();
    let initial_len = queue.len();
    let initial_json = serde_json::to_vec(&*runtime.snapshot()).unwrap();
    let initial_revision = runtime.revision();
    let initial_history = runtime.history_lengths();

    assert!(matches!(
        runtime.process_next(&mut queue, None, 0),
        Err(DocumentEditRuntimeError::DocumentWriteBlocked { .. })
    ));
    assert_eq!(queue.len(), initial_len);
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        initial_json
    );
    assert_eq!(runtime.revision(), initial_revision);
    assert_eq!(runtime.history_lengths(), initial_history);
    runtime
        .reconcile_pending_commit()
        .unwrap()
        .expect("first commit recovered");
    let second = runtime
        .process_next(&mut queue, None, 1)
        .unwrap()
        .expect("queued action remains available");
    assert_eq!(second.revision, 2);
}

#[test]
fn reopen_recovers_a_deferred_commit_without_a_terminal_runtime_state() {
    let f = two_track_fixture();
    let (path, mut runtime) = open_runtime(f.document.clone());
    runtime.set_test_failpoint(RuntimeTestFailpoint::DeferAfterDurableCommit);
    let mut queue = DocumentEditQueue::default();
    queue
        .push_prepared(delete_output(), Some(f.delete_request))
        .unwrap();
    let _ = runtime.process_next(&mut queue, None, 0);
    assert!(runtime.is_write_blocked());
    assert_eq!(runtime.revision(), 0);
    drop(runtime);

    let limits = ResourceLimits::production();
    let (session, opened) = ProjectSession::open(&path, &limits).expect("reopen");
    let catalog = first_party_catalog();
    let writer = DocumentWriter::new(opened.document, Arc::clone(&catalog)).unwrap();
    let mut runtime = DocumentEditRuntime::new(session, writer, catalog);
    assert!(!runtime.is_write_blocked());

    let mut queue = DocumentEditQueue::default();
    queue
        .push_prepared(
            delete_output(),
            Some(two_track_delete_request(
                runtime.snapshot().as_ref(),
                f.surviving,
            )),
        )
        .unwrap();
    let applied = runtime
        .process_next(&mut queue, None, 0)
        .unwrap()
        .expect("reopened runtime must accept Apply");
    assert_eq!(applied.kind, DocumentEditActionKind::Apply);
    assert_eq!(applied.revision, 1);
    assert_eq!(queue.len(), 0);
}

#[test]
fn duplicate_apply_after_success_is_rejected_at_preflight() {
    let (document, request) = fixture();
    let initial_json = serde_json::to_vec(&document).unwrap();
    let track = document.tracks[0].id;
    let item = document.tracks[0].items[0].clone();
    let layer_names = layer_names_for_item(&document, &item).unwrap();
    let (path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_prepared(delete_output(), Some(request)).unwrap();
    let applied = runtime.process_next(&mut queue, None, 0).unwrap().unwrap();
    assert_eq!(applied.revision, 1);
    let post_apply_json = serde_json::to_vec(&*runtime.snapshot()).unwrap();
    assert_ne!(post_apply_json, initial_json);
    let journal = journal_path_for_document(&path);
    let journal_size = fs::metadata(&journal).unwrap().len();
    let request_again = DocumentCommandRequest::try_new(
        DomainIntent::DeleteTargetedItems,
        vec![Command::RemoveTrackItem {
            parent: ParentLocator::Track(track),
            index: 0,
            layer_names,
            item,
        }],
    )
    .unwrap();
    queue
        .push_prepared(delete_output(), Some(request_again))
        .unwrap();
    let pre_revision = runtime.revision();
    let pre_history = runtime.history_lengths();
    assert!(matches!(
        runtime.process_next(&mut queue, None, 1),
        Err(DocumentEditRuntimeError::Command(_))
    ));
    assert_eq!(queue.len(), 0);
    assert_eq!(runtime.revision(), pre_revision);
    assert_eq!(runtime.history_lengths(), pre_history);
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        post_apply_json
    );
    assert_eq!(fs::metadata(&journal).unwrap().len(), journal_size);
    assert!(!runtime.is_write_blocked());
}

#[test]
fn journal_commit_precedes_live_apply_on_success() {
    let (document, request) = fixture();
    let (path, mut runtime) = open_runtime(document);
    let journal = journal_path_for_document(&path);
    let size_before = fs::metadata(&journal).map(|meta| meta.len()).unwrap_or(0);

    let mut queue = DocumentEditQueue::default();
    queue.push_prepared(delete_output(), Some(request)).unwrap();
    let published = runtime.process_next(&mut queue, None, 0).unwrap().unwrap();
    assert_eq!(published.revision, 1);
    let size_after = fs::metadata(&journal).expect("journal").len();
    assert!(size_after > size_before);
    assert_eq!(queue.len(), 0);
    assert_eq!(published.projection_generation, 1);
}
