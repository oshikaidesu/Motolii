use std::sync::atomic::{AtomicU64, Ordering};

use motolii_doc::{layer_names_for_item, Clip, ItemEnvelope, ParentLocator, Track};
use motolii_plugin::PluginCatalog;
use motolii_testkit::tmp_dir;

use super::super::*;

pub(super) use std::collections::BTreeMap;
pub(super) use std::fs;
pub(super) use std::sync::Arc;

pub(super) use super::super::prepare_place::item_layer_id;
pub(super) use crate::timeline_move_gesture::TimelineMoveRequest;
pub(super) use crate::timeline_trim_gesture::TimelineTrimRequest;
pub(super) use crate::{
    adapt_document_command_request_error, builtin_command_registry, CommandId,
    DiagnosticReasonCode, DocumentCommandRequest, DocumentCommandRequestError, DomainIntent,
    InputPhase, InputRouter, InteractionState, InteractionStateMachine, NormalizedInput,
    RouterOutput,
};
pub(super) use motolii_core::RationalTime;
pub(super) use motolii_doc::{
    journal_path_for_document, motolii_dir_for_document, AssetId, ClipSource, Command, DocParam,
    Document, DocumentError, DocumentPluginError, DocumentWriter, LayerId, ProjectSession,
    ResourceLimits, SaveProjectOptions, ScalarPropertyId, TrackId, TrackItem,
};
pub(super) use motolii_eval::Interp;

pub(super) fn first_party_catalog() -> Arc<PluginCatalog> {
    Arc::new(motolii_plugins_firstparty::first_party_catalog().expect("first party catalog"))
}

static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(super) fn unique_tmp(tag: &str) -> std::path::PathBuf {
    let id = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    tmp_dir(&format!("{tag}-{id}"))
}

pub(super) fn persist_project(path: &std::path::Path, document: &Document) {
    let limits = ResourceLimits::production();
    let mut session = ProjectSession::acquire(path, &limits).expect("acquire");
    session
        .save_with_journal(
            document,
            &SaveProjectOptions {
                limits,
                checkpoint: true,
                ..SaveProjectOptions::default()
            },
        )
        .expect("initial checkpoint");
}

pub(super) fn open_runtime(document: Document) -> (std::path::PathBuf, DocumentEditRuntime) {
    let dir = unique_tmp("cu109-runtime-unit");
    let path = dir.join("proj.json");
    persist_project(&path, &document);
    let limits = ResourceLimits::production();
    let (session, opened) = ProjectSession::open(&path, &limits).expect("open");
    let catalog = first_party_catalog();
    let writer = DocumentWriter::new(opened.document, Arc::clone(&catalog)).expect("writer");
    let runtime = DocumentEditRuntime::new(session, writer, catalog);
    (path, runtime)
}

pub(super) fn fixture() -> (Document, DocumentCommandRequest) {
    let mut document = Document::new_current();
    let layer = document.layers.allocate("fixture").unwrap();
    let track = document.track_ids.allocate("V1").unwrap();
    let asset = document
        .assets
        .allocate("media", "video/mp4", "hash")
        .unwrap();
    let item = TrackItem::Clip(Clip {
        envelope: ItemEnvelope::new(layer),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(1, 1).unwrap(),
        time_map: Default::default(),
        source: ClipSource::asset_video_only(asset),
    });
    document.tracks.push(Track {
        id: track,
        items: vec![item.clone()],
    });
    document.validate().unwrap();
    let request = DocumentCommandRequest::try_new(
        DomainIntent::DeleteTargetedItems,
        vec![Command::RemoveTrackItem {
            parent: ParentLocator::Track(track),
            index: 0,
            layer_names: layer_names_for_item(&document, &item).unwrap(),
            item,
        }],
    )
    .unwrap();
    (document, request)
}

struct TwoTrackFixture {
    document: Document,
    delete_request: DocumentCommandRequest,
    surviving: LayerId,
    deleted: LayerId,
    surviving_track: TrackId,
    asset: AssetId,
}

pub(super) fn two_track_fixture() -> TwoTrackFixture {
    let mut document = Document::new_current();
    let surviving = document.layers.allocate("surviving").unwrap();
    let deleted = document.layers.allocate("deleted").unwrap();
    let surviving_track = document.track_ids.allocate("V1").unwrap();
    let deleted_track = document.track_ids.allocate("V2").unwrap();
    let asset = document
        .assets
        .allocate("media", "video/mp4", "hash")
        .unwrap();

    let surviving_item = TrackItem::Clip(Clip {
        envelope: ItemEnvelope::new(surviving),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(1, 1).unwrap(),
        time_map: Default::default(),
        source: ClipSource::asset_video_only(asset),
    });
    document.tracks.push(Track {
        id: surviving_track,
        items: vec![surviving_item.clone()],
    });

    let deleted_item = TrackItem::Clip(Clip {
        envelope: ItemEnvelope::new(deleted),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(1, 1).unwrap(),
        time_map: Default::default(),
        source: ClipSource::asset_video_only(asset),
    });
    document.tracks.push(Track {
        id: deleted_track,
        items: vec![deleted_item.clone()],
    });
    document.validate().unwrap();

    let delete_request = DocumentCommandRequest::try_new(
        DomainIntent::DeleteTargetedItems,
        vec![Command::RemoveTrackItem {
            parent: ParentLocator::Track(deleted_track),
            index: 0,
            layer_names: layer_names_for_item(&document, &deleted_item).unwrap(),
            item: deleted_item,
        }],
    )
    .unwrap();

    TwoTrackFixture {
        document,
        delete_request,
        surviving,
        deleted,
        surviving_track,
        asset,
    }
}

pub(super) fn fixture_layer(document: &Document) -> LayerId {
    match &document.tracks[0].items[0] {
        TrackItem::Clip(clip) => clip.envelope.layer_id,
        _ => panic!("fixtureはclipを含む前提"),
    }
}

pub(super) fn two_track_delete_request(
    document: &Document,
    deleted: LayerId,
) -> DocumentCommandRequest {
    let mut deleted_track = None;
    let mut deleted_item = None;

    'search: for track in &document.tracks {
        for item in &track.items {
            if let TrackItem::Clip(clip) = item {
                if clip.envelope.layer_id == deleted {
                    deleted_track = Some(track.id);
                    deleted_item = Some(item.clone());
                    break 'search;
                }
            }
        }
    }

    let deleted_track = deleted_track.unwrap_or_else(|| panic!("deleted track not found"));
    let deleted_item = deleted_item.unwrap_or_else(|| panic!("deleted item not found"));
    let layer_names = layer_names_for_item(document, &deleted_item).unwrap();

    DocumentCommandRequest::try_new(
        DomainIntent::DeleteTargetedItems,
        vec![Command::RemoveTrackItem {
            parent: ParentLocator::Track(deleted_track),
            index: 0,
            layer_names,
            item: deleted_item,
        }],
    )
    .unwrap()
}

pub(super) fn delete_output() -> RouterOutput {
    let mut router = InputRouter::new(builtin_command_registry().unwrap());
    router
        .route(NormalizedInput::Command {
            phase: InputPhase::Click,
            id: CommandId::try_new("motolii.edit.delete_targeted_items").unwrap(),
        })
        .unwrap()
}

pub(super) fn history_output(intent: DomainIntent) -> RouterOutput {
    let id = match intent {
        DomainIntent::Undo => "motolii.edit.undo",
        DomainIntent::Redo => "motolii.edit.redo",
        _ => panic!("history output requires Undo or Redo"),
    };
    let mut router = InputRouter::new(builtin_command_registry().unwrap());
    router
        .route(NormalizedInput::Command {
            phase: InputPhase::Press,
            id: CommandId::try_new(id).unwrap(),
        })
        .unwrap()
}

pub(super) fn assert_preflight_rejection_invariants(
    runtime: &DocumentEditRuntime,
    queue: &DocumentEditQueue,
    initial_json: &[u8],
    pre_revision: u64,
    pre_history: (usize, usize),
) {
    assert_eq!(queue.len(), 0);
    assert_eq!(runtime.revision(), pre_revision);
    assert_eq!(runtime.history_lengths(), pre_history);
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        initial_json
    );
    assert!(!runtime.is_write_blocked());
}

pub(super) fn assert_add_position_key_noop(
    document: Document,
    current_primary: Option<LayerId>,
    request: AddPositionKeyRequest,
) {
    let initial_json = serde_json::to_vec(&document).unwrap();
    let initial_stable = document.next_stable_id.peek_next();
    let (path, mut runtime) = open_runtime(document);
    let journal = journal_path_for_document(&path);
    let journal_size = fs::metadata(&journal).map(|meta| meta.len()).unwrap_or(0);
    let mut queue = DocumentEditQueue::default();
    queue.push_add_position_key(request);

    let error = runtime
        .process_next(&mut queue, current_primary, u64::MAX)
        .expect_err("add position key negative must not silent-accept");
    assert!(matches!(
        error,
        DocumentEditRuntimeError::NoPrimarySelection
            | DocumentEditRuntimeError::PrepareRejected
            | DocumentEditRuntimeError::PositionKeyPrepare(_)
            | DocumentEditRuntimeError::Command(_)
    ));
    assert_preflight_rejection_invariants(&runtime, &queue, &initial_json, 0, (0, 0));
    assert_eq!(
        runtime.snapshot().next_stable_id.peek_next(),
        initial_stable
    );
    assert_eq!(
        fs::metadata(&journal).map(|meta| meta.len()).unwrap_or(0),
        journal_size
    );
}

pub(super) fn dangling_parent_after_remove_fixture() -> (Document, DocumentCommandRequest) {
    let mut document = Document::new_current();
    let deleted_layer = document.layers.allocate("deleted-parent").unwrap();
    let survivor_layer = document.layers.allocate("survivor").unwrap();
    let track = document.track_ids.allocate("V1").unwrap();
    let asset = document
        .assets
        .allocate("media", "video/mp4", "hash")
        .unwrap();

    let deleted_item = TrackItem::Clip(Clip {
        envelope: ItemEnvelope::new(deleted_layer),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(1, 1).unwrap(),
        time_map: Default::default(),
        source: ClipSource::asset_video_only(asset),
    });

    let mut survivor_envelope = ItemEnvelope::new(survivor_layer);
    survivor_envelope.transform.parent = Some(deleted_layer);
    let survivor_item = TrackItem::Clip(Clip {
        envelope: survivor_envelope,
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(1, 1).unwrap(),
        time_map: Default::default(),
        source: ClipSource::asset_video_only(asset),
    });

    document.tracks.push(Track {
        id: track,
        items: vec![deleted_item.clone(), survivor_item],
    });
    document.validate().unwrap();

    let request = DocumentCommandRequest::try_new(
        DomainIntent::DeleteTargetedItems,
        vec![Command::RemoveTrackItem {
            parent: ParentLocator::Track(track),
            index: 0,
            layer_names: layer_names_for_item(&document, &deleted_item).unwrap(),
            item: deleted_item,
        }],
    )
    .unwrap();

    (document, request)
}

pub(super) fn open_runtime_with_catalog(
    document: Document,
    catalog: Arc<PluginCatalog>,
) -> (std::path::PathBuf, DocumentEditRuntime) {
    let dir = unique_tmp("cu109-runtime-unit");
    let path = dir.join("proj.json");
    persist_project(&path, &document);
    let limits = ResourceLimits::production();
    let (session, opened) = ProjectSession::open(&path, &limits).expect("open");
    let writer_catalog = first_party_catalog();
    let writer = DocumentWriter::new(opened.document, Arc::clone(&writer_catalog)).expect("writer");
    let runtime = DocumentEditRuntime::new(session, writer, catalog);
    (path, runtime)
}
