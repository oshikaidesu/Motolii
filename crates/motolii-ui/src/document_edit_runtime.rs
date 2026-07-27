//! 確定済みDocument編集をsingle writerへ直列配送するprivate runtime。

use std::collections::VecDeque;
use std::sync::Arc;

use motolii_doc::{CommandError, Document, DocumentWriter, LayerId, UndoError};

use crate::{DocumentCommandRequest, DomainIntent, InputPhase, RouterOutput};

#[derive(Debug)]
pub(crate) enum DocumentEditAction {
    Apply(DocumentCommandRequest),
    Undo,
    Redo,
}

impl DocumentEditAction {
    pub(crate) const fn kind(&self) -> DocumentEditActionKind {
        match self {
            Self::Apply(_) => DocumentEditActionKind::Apply,
            Self::Undo => DocumentEditActionKind::Undo,
            Self::Redo => DocumentEditActionKind::Redo,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DocumentEditActionKind {
    Apply,
    Undo,
    Redo,
}

#[derive(Debug, Default)]
pub(crate) struct DocumentEditQueue {
    pending: VecDeque<DocumentEditAction>,
}

impl DocumentEditQueue {
    pub(crate) fn push_prepared(
        &mut self,
        output: RouterOutput,
        request: Option<DocumentCommandRequest>,
    ) -> Result<(), DocumentEditDispatchError> {
        let RouterOutput::Intent { phase, intent, .. } = output else {
            return Err(DocumentEditDispatchError::NotCommitIntent);
        };
        if phase != InputPhase::Click || intent != DomainIntent::DeleteTargetedItems {
            return Err(DocumentEditDispatchError::NotCommitIntent);
        }
        let request = request.ok_or(DocumentEditDispatchError::MissingPreparedRequest)?;
        if request.intent() != intent {
            return Err(DocumentEditDispatchError::IntentMismatch {
                routed: intent,
                request: request.intent(),
            });
        }
        self.pending.push_back(DocumentEditAction::Apply(request));
        Ok(())
    }

    pub(crate) fn push_undo(&mut self) {
        self.pending.push_back(DocumentEditAction::Undo);
    }

    pub(crate) fn push_redo(&mut self) {
        self.pending.push_back(DocumentEditAction::Redo);
    }

    fn pop_front(&mut self) -> Option<DocumentEditAction> {
        self.pending.pop_front()
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.pending.len()
    }
}

pub(crate) struct DocumentEditRuntime {
    writer: DocumentWriter,
}

impl DocumentEditRuntime {
    pub(crate) fn new(writer: DocumentWriter) -> Self {
        Self { writer }
    }

    pub(crate) fn snapshot(&self) -> Arc<Document> {
        self.writer.snapshot()
    }

    pub(crate) fn process_next(
        &mut self,
        queue: &mut DocumentEditQueue,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let Some(action) = queue.pop_front() else {
            return Ok(None);
        };
        let kind = action.kind();
        let Some(next_projection_generation) = current_projection_generation.checked_add(1) else {
            return Err(DocumentEditRuntimeError::ProjectionGenerationExhausted);
        };
        match action {
            DocumentEditAction::Apply(request) => {
                self.writer.apply_macro(request.into_commands())?;
            }
            DocumentEditAction::Undo => self.writer.undo()?,
            DocumentEditAction::Redo => self.writer.redo()?,
        }
        let snapshot = self.writer.snapshot();
        let primary = current_primary.filter(|id| self.writer.find_envelope(*id).is_some());
        // snapshot()が同一writerのdocを複製し、find_envelopeと同じ内容を見て同一生成時点で整合するため。
        Ok(Some(PublishedDocument {
            kind,
            revision: self.writer.revision,
            snapshot,
            primary,
            projection_generation: next_projection_generation,
        }))
    }

    #[cfg(test)]
    fn history_lengths(&self) -> (usize, usize) {
        (self.writer.undo_len(), self.writer.redo_len())
    }
}

#[derive(Debug)]
pub(crate) struct PublishedDocument {
    pub(crate) kind: DocumentEditActionKind,
    pub(crate) revision: u64,
    pub(crate) snapshot: Arc<Document>,
    pub(crate) primary: Option<LayerId>,
    pub(crate) projection_generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub(crate) enum DocumentEditDispatchError {
    #[error("router output is not a committed delete intent")]
    NotCommitIntent,
    #[error("committed delete intent has no prepared Document request")]
    MissingPreparedRequest,
    #[error("routed intent {routed:?} does not match request intent {request:?}")]
    IntentMismatch {
        routed: DomainIntent,
        request: DomainIntent,
    },
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum DocumentEditRuntimeError {
    #[error("projection generation is exhausted at u64::MAX")]
    ProjectionGenerationExhausted,
    #[error(transparent)]
    Command(#[from] CommandError),
    #[error(transparent)]
    Undo(#[from] UndoError),
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::Arc;

    use motolii_core::RationalTime;
    use motolii_doc::{
        layer_names_for_item, AssetId, Clip, ClipSource, Command, Document, DocumentWriter,
        ItemEnvelope, LayerId, ParentLocator, Track, TrackId, TrackItem,
    };
    use motolii_plugin::PluginCatalogBuilder;

    use super::*;
    use crate::{
        adapt_document_command_request_error, builtin_command_registry, CommandId,
        DiagnosticReasonCode, DocumentCommandRequestError, InputRouter, InteractionState,
        InteractionStateMachine, NormalizedInput,
    };

    fn fixture() -> (Document, DocumentCommandRequest) {
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

    fn two_track_fixture() -> TwoTrackFixture {
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

    fn fixture_layer(document: &Document) -> LayerId {
        match &document.tracks[0].items[0] {
            TrackItem::Clip(clip) => clip.envelope.layer_id,
            _ => panic!("fixtureはclipを含む前提"),
        }
    }

    fn two_track_delete_request(document: &Document, deleted: LayerId) -> DocumentCommandRequest {
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

    fn runtime(document: Document) -> DocumentEditRuntime {
        let catalog = PluginCatalogBuilder::new().build().unwrap();
        DocumentEditRuntime::new(DocumentWriter::new(document, Arc::new(catalog)).unwrap())
    }

    fn delete_output() -> RouterOutput {
        let mut router = InputRouter::new(builtin_command_registry().unwrap());
        router
            .route(NormalizedInput::Command {
                phase: InputPhase::Click,
                id: CommandId::try_new("motolii.edit.delete_targeted_items").unwrap(),
            })
            .unwrap()
    }

    #[test]
    fn apply_undo_redo_publish_new_snapshots_without_mutating_old_ones() {
        let (document, request) = fixture();
        let initial_json = serde_json::to_vec(&document).unwrap();
        let mut runtime = runtime(document);
        let initial_snapshot = runtime.snapshot();
        let mut queue = DocumentEditQueue::default();
        queue.push_prepared(delete_output(), Some(request)).unwrap();
        queue.push_undo();
        queue.push_redo();

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

        let undone = runtime
            .process_next(&mut queue, applied.primary, applied.projection_generation)
            .unwrap()
            .unwrap();
        assert_eq!(undone.kind, DocumentEditActionKind::Undo);
        assert_eq!(undone.revision, 2);
        assert_eq!(runtime.history_lengths(), (0, 1));
        assert_eq!(serde_json::to_vec(&*undone.snapshot).unwrap(), initial_json);
        assert_eq!(
            serde_json::to_vec(&*applied.snapshot).unwrap(),
            applied_json
        );

        let redone = runtime
            .process_next(&mut queue, undone.primary, undone.projection_generation)
            .unwrap()
            .unwrap();
        assert_eq!(redone.kind, DocumentEditActionKind::Redo);
        assert_eq!(redone.revision, 3);
        assert_eq!(runtime.history_lengths(), (1, 0));
        assert_eq!(serde_json::to_vec(&*redone.snapshot).unwrap(), applied_json);
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn missing_request_and_empty_history_publish_nothing() {
        let (document, _) = fixture();
        let initial_json = serde_json::to_vec(&document).unwrap();
        let mut runtime = runtime(document);
        let mut queue = DocumentEditQueue::default();

        assert_eq!(
            queue.push_prepared(delete_output(), None),
            Err(DocumentEditDispatchError::MissingPreparedRequest)
        );
        assert!(runtime.process_next(&mut queue, None, 0).unwrap().is_none());

        queue.push_undo();
        assert!(matches!(
            runtime.process_next(&mut queue, None, 0),
            Err(DocumentEditRuntimeError::Undo(_))
        ));
        assert_eq!(runtime.writer.revision, 0);
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
        let mut runtime = runtime(document);
        let mut queue = DocumentEditQueue::default();
        queue.push_prepared(delete_output(), Some(request)).unwrap();

        assert!(matches!(
            runtime.process_next(&mut queue, None, 0),
            Err(DocumentEditRuntimeError::Command(_))
        ));
        assert_eq!(queue.len(), 0);
        assert_eq!(runtime.writer.revision, 0);
        assert_eq!(runtime.history_lengths(), (0, 0));
        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            initial_json
        );
    }

    #[test]
    fn empty_queue_publishes_nothing_and_does_not_advance_generation() {
        let (document, _) = fixture();
        let layer = fixture_layer(&document);
        let initial_json = serde_json::to_vec(&document).unwrap();
        let mut runtime = runtime(document);
        let pre_revision = runtime.writer.revision;
        let mut queue = DocumentEditQueue::default();

        assert!(runtime
            .process_next(&mut queue, Some(layer), 5)
            .unwrap()
            .is_none());
        assert_eq!(runtime.writer.revision, pre_revision);
        assert_eq!(runtime.history_lengths(), (0, 0));
        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            initial_json
        );
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn failed_apply_publishes_nothing_and_advances_no_generation() {
        let (document, _) = fixture();
        let initial_json = serde_json::to_vec(&document).unwrap();
        let layer = fixture_layer(&document);
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
        let mut runtime = runtime(document);
        let pre_revision = runtime.writer.revision;
        let mut queue = DocumentEditQueue::default();
        queue.push_prepared(delete_output(), Some(request)).unwrap();

        assert!(matches!(
            runtime.process_next(&mut queue, Some(layer), 0),
            Err(DocumentEditRuntimeError::Command(_))
        ));
        assert_eq!(queue.len(), 0);
        assert_eq!(runtime.writer.revision, pre_revision);
        assert_eq!(runtime.history_lengths(), (0, 0));
        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            initial_json
        );
    }

    #[test]
    fn failed_undo_and_redo_publish_nothing() {
        let (document, _) = fixture();
        let initial_json = serde_json::to_vec(&document).unwrap();
        let mut runtime = runtime(document);
        let pre_revision = runtime.writer.revision;

        let mut queue = DocumentEditQueue::default();
        queue.push_undo();
        assert!(matches!(
            runtime.process_next(&mut queue, None, 0),
            Err(DocumentEditRuntimeError::Undo(_))
        ));
        assert_eq!(queue.len(), 0);
        assert_eq!(runtime.writer.revision, pre_revision);
        assert_eq!(runtime.history_lengths(), (0, 0));
        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            initial_json
        );

        let mut queue = DocumentEditQueue::default();
        queue.push_redo();
        assert!(matches!(
            runtime.process_next(&mut queue, None, 0),
            Err(DocumentEditRuntimeError::Undo(_))
        ));
        assert_eq!(queue.len(), 0);
        assert_eq!(runtime.writer.revision, pre_revision);
        assert_eq!(runtime.history_lengths(), (0, 0));
        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            initial_json
        );
    }

    #[test]
    fn diagnostic_adaptation_does_not_enqueue_or_mutate_document_runtime() {
        let (document, _) = fixture();
        let initial_json = serde_json::to_vec(&document).unwrap();
        let mut runtime = runtime(document);
        let mut queue = DocumentEditQueue::default();

        let envelope =
            adapt_document_command_request_error(&DocumentCommandRequestError::EmptyCommands);

        assert_eq!(
            envelope.reason(),
            DiagnosticReasonCode::EmptyDocumentCommands
        );
        assert!(runtime.process_next(&mut queue, None, 0).unwrap().is_none());
        assert_eq!(queue.len(), 0);
        assert_eq!(runtime.writer.revision, 0);
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
            let mut runtime = runtime(document);
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
            assert_eq!(runtime.writer.revision, 0);
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
        let mut runtime = runtime(f.document);
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
        assert!(runtime
            .process_next(&mut queue, applied.primary, applied.projection_generation)
            .unwrap()
            .is_none());
    }

    #[test]
    fn apply_success_clears_primary_deleted_by_the_apply() {
        let (document, request) = fixture();
        let deleted = fixture_layer(&document);
        let mut runtime = runtime(document);
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
    fn undo_success_clears_primary_removed_by_the_undo() {
        let f = two_track_fixture();
        let mut runtime = runtime(f.document.clone());
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
        assert!(runtime.writer.find_envelope(added_layer).is_some());
        assert_eq!(runtime.writer.revision, 1);
        assert_eq!(runtime.history_lengths(), (1, 0));

        let mut queue = DocumentEditQueue::default();
        queue.push_undo();
        let undone = runtime
            .process_next(&mut queue, Some(added_layer), 4)
            .unwrap()
            .unwrap();
        assert_eq!(undone.kind, DocumentEditActionKind::Undo);
        assert_eq!(undone.revision, 2);
        assert_eq!(undone.projection_generation, 5);
        assert_eq!(undone.primary, None);
        assert!(runtime.writer.find_envelope(added_layer).is_none());
        assert_eq!(queue.len(), 0);
        assert!(runtime
            .process_next(&mut queue, undone.primary, undone.projection_generation)
            .unwrap()
            .is_none());
        assert_eq!(runtime.history_lengths(), (0, 1));
    }

    #[test]
    fn undo_success_retains_primary_that_the_undo_restores() {
        let (document, request) = fixture();
        let layer = fixture_layer(&document);
        let mut runtime = runtime(document);
        let mut queue = DocumentEditQueue::default();
        queue.push_prepared(delete_output(), Some(request)).unwrap();

        let applied = runtime
            .process_next(&mut queue, Some(layer), 0)
            .unwrap()
            .unwrap();
        assert_eq!(applied.kind, DocumentEditActionKind::Apply);
        assert_eq!(applied.projection_generation, 1);
        assert_eq!(applied.primary, None);

        queue.push_undo();
        let undone = runtime
            .process_next(&mut queue, Some(layer), applied.projection_generation)
            .unwrap()
            .unwrap();
        // valid-retain under Undo: undo restores the target, so primary should remain Some.
        assert_eq!(undone.kind, DocumentEditActionKind::Undo);
        assert_eq!(undone.revision, 2);
        assert_eq!(undone.projection_generation, 2);
        assert_eq!(undone.primary, Some(layer));
        assert!(runtime.writer.find_envelope(layer).is_some());
    }

    #[test]
    fn redo_success_does_not_restore_cleared_primary() {
        let (document, request) = fixture();
        let layer = fixture_layer(&document);
        let mut runtime = runtime(document);
        let mut queue = DocumentEditQueue::default();
        queue.push_prepared(delete_output(), Some(request)).unwrap();

        let applied = runtime
            .process_next(&mut queue, Some(layer), 0)
            .unwrap()
            .unwrap();
        queue.push_undo();
        let undone = runtime
            .process_next(&mut queue, applied.primary, applied.projection_generation)
            .unwrap()
            .unwrap();
        queue.push_redo();
        let redone = runtime
            .process_next(&mut queue, undone.primary, undone.projection_generation)
            .unwrap()
            .unwrap();
        assert_eq!(redone.kind, DocumentEditActionKind::Redo);
        assert_eq!(redone.revision, 3);
        assert_eq!(redone.projection_generation, 3);
        assert_eq!(redone.primary, None);
    }

    #[test]
    fn generation_at_max_minus_one_advances_to_max() {
        let (document, request) = fixture();
        let mut runtime = runtime(document);
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
        let _ = (&f.delete_request, f.surviving);
        let mut runtime = runtime(f.document.clone());
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
        assert_eq!(runtime.writer.revision, 1);
        assert_eq!(runtime.history_lengths(), (1, 0));

        let actions = ["apply", "undo", "redo"];
        for action in actions {
            let mut queue = DocumentEditQueue::default();
            match action {
                "apply" => queue
                    .push_prepared(
                        delete_output(),
                        Some(two_track_delete_request(&f.document, f.deleted)),
                    )
                    .unwrap(),
                "undo" => queue.push_undo(),
                "redo" => {
                    queue.push_undo();
                    runtime
                        .process_next(&mut queue, Some(added_layer), 0)
                        .unwrap()
                        .unwrap();
                }
                _ => unreachable!(),
            }
            if action == "redo" {
                queue.push_redo();
            }

            let initial_json = serde_json::to_vec(&*runtime.snapshot()).unwrap();
            let pre_revision = runtime.writer.revision;
            let pre_history = runtime.history_lengths();

            match runtime.process_next(&mut queue, Some(added_layer), u64::MAX) {
                Err(DocumentEditRuntimeError::ProjectionGenerationExhausted) => {}
                result => panic!("unexpected result: {result:?}"),
            }
            assert_eq!(queue.len(), 0);
            assert_eq!(runtime.writer.revision, pre_revision);
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
        let _ = (&f.delete_request, f.surviving);
        let mut runtime = runtime(f.document.clone());
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
        queue.push_redo();

        assert!(matches!(
            runtime.process_next(&mut queue, None, u64::MAX),
            Err(DocumentEditRuntimeError::ProjectionGenerationExhausted)
        ));
        assert_eq!(queue.len(), 2);

        let published = runtime.process_next(&mut queue, None, 4).unwrap().unwrap();
        assert_eq!(published.primary, None);
        assert_eq!(published.projection_generation, 5);
        assert_eq!(runtime.writer.revision, 2);
        assert_eq!(runtime.history_lengths(), (0, 1));

        assert!(matches!(
            runtime.process_next(&mut queue, published.primary, u64::MAX),
            Err(DocumentEditRuntimeError::ProjectionGenerationExhausted)
        ));
        assert_eq!(queue.len(), 0);
    }
}
