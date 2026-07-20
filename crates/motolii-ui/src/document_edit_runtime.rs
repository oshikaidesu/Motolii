//! 確定済みDocument編集をsingle writerへ直列配送するprivate runtime。

use std::collections::VecDeque;
use std::sync::Arc;

use motolii_doc::{CommandError, Document, DocumentWriter, UndoError};

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
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let Some(action) = queue.pop_front() else {
            return Ok(None);
        };
        let kind = action.kind();
        match action {
            DocumentEditAction::Apply(request) => {
                self.writer.apply_macro(request.into_commands())?;
            }
            DocumentEditAction::Undo => self.writer.undo()?,
            DocumentEditAction::Redo => self.writer.redo()?,
        }
        Ok(Some(PublishedDocument {
            kind,
            revision: self.writer.revision,
            snapshot: self.writer.snapshot(),
        }))
    }

    #[cfg(test)]
    fn history_lengths(&self) -> (usize, usize) {
        (self.writer.undo_len(), self.writer.redo_len())
    }
}

pub(crate) struct PublishedDocument {
    pub(crate) kind: DocumentEditActionKind,
    pub(crate) revision: u64,
    pub(crate) snapshot: Arc<Document>,
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
    #[error(transparent)]
    Command(#[from] CommandError),
    #[error(transparent)]
    Undo(#[from] UndoError),
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use motolii_core::RationalTime;
    use motolii_doc::{
        layer_names_for_item, Clip, ClipSource, Command, Document, DocumentWriter, ItemEnvelope,
        ParentLocator, Track, TrackItem,
    };
    use motolii_plugin::PluginCatalogBuilder;

    use super::*;
    use crate::{builtin_command_registry, CommandId, InputRouter, NormalizedInput};

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

        let applied = runtime.process_next(&mut queue).unwrap().unwrap();
        let applied_json = serde_json::to_vec(&*applied.snapshot).unwrap();
        assert_eq!(applied.kind, DocumentEditActionKind::Apply);
        assert_eq!(applied.revision, 1);
        assert_eq!(runtime.history_lengths(), (1, 0));
        assert_ne!(applied_json, initial_json);
        assert_eq!(
            serde_json::to_vec(&*initial_snapshot).unwrap(),
            initial_json
        );

        let undone = runtime.process_next(&mut queue).unwrap().unwrap();
        assert_eq!(undone.kind, DocumentEditActionKind::Undo);
        assert_eq!(undone.revision, 2);
        assert_eq!(runtime.history_lengths(), (0, 1));
        assert_eq!(serde_json::to_vec(&*undone.snapshot).unwrap(), initial_json);
        assert_eq!(
            serde_json::to_vec(&*applied.snapshot).unwrap(),
            applied_json
        );

        let redone = runtime.process_next(&mut queue).unwrap().unwrap();
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
        assert!(runtime.process_next(&mut queue).unwrap().is_none());

        queue.push_undo();
        assert!(matches!(
            runtime.process_next(&mut queue),
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
            runtime.process_next(&mut queue),
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
}
