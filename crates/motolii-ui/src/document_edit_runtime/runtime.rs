//! Document single-writerと履歴投影を持つruntime本体。

use std::path::PathBuf;
use std::sync::Arc;

use motolii_core::RationalTime;
use motolii_doc::{
    Command, Document, DocumentWriter, EffectId, JournalCommitReceipt, LayerId, ProjectError,
    ProjectSession,
};
use motolii_plugin::PluginCatalog;

use crate::timeline_trim_gesture::TimelineTrimRequest;

use super::action::DocumentEditActionKind;
use super::error::{DocumentEditRuntimeError, PublishedDocument};
use super::prepare_place::{prepare_vector_shape_command, VectorShapeKind};
use super::requests::PlaceRectangleRequest;

#[derive(Debug)]
pub(super) enum RuntimeHealth {
    Healthy,
    WriteBlocked(Box<PendingCommit>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HistoryDirection {
    Undo,
    Redo,
}

#[derive(Debug)]
pub(super) struct PreparedHistoryAction {
    pub(super) direction: HistoryDirection,
    pub(super) durable_command: Command,
}

#[derive(Debug, Clone, Default)]
pub(super) struct HistoryProjection {
    undo: Vec<Command>,
    redo: Vec<Command>,
}

impl HistoryProjection {
    pub(super) fn record_forward(&mut self, command: Command) {
        self.undo.push(command);
        self.redo.clear();
    }

    pub(super) fn prepare(
        &self,
        direction: HistoryDirection,
        writer: &DocumentWriter,
    ) -> Result<PreparedHistoryAction, DocumentEditRuntimeError> {
        if self.undo.len() != writer.undo_len() || self.redo.len() != writer.redo_len() {
            return Err(DocumentEditRuntimeError::HistoryProjectionMismatch);
        }
        let forward = match direction {
            HistoryDirection::Undo => self
                .undo
                .last()
                .ok_or(DocumentEditRuntimeError::NothingToUndo)?,
            HistoryDirection::Redo => self
                .redo
                .last()
                .ok_or(DocumentEditRuntimeError::NothingToRedo)?,
        };
        let durable_command = match direction {
            HistoryDirection::Undo => forward.inverse(),
            HistoryDirection::Redo => forward.clone(),
        };
        Ok(PreparedHistoryAction {
            direction,
            durable_command,
        })
    }

    pub(super) fn accept(
        &mut self,
        direction: HistoryDirection,
    ) -> Result<(), DocumentEditRuntimeError> {
        match direction {
            HistoryDirection::Undo => {
                let command = self
                    .undo
                    .pop()
                    .ok_or(DocumentEditRuntimeError::HistoryProjectionMismatch)?;
                self.redo.push(command);
            }
            HistoryDirection::Redo => {
                let command = self
                    .redo
                    .pop()
                    .ok_or(DocumentEditRuntimeError::HistoryProjectionMismatch)?;
                self.undo.push(command);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeTestFailpoint {
    DeferAfterDurableCommit,
    Reconcile,
}

#[cfg(test)]
#[derive(Debug, Default)]
pub(super) struct RuntimeTestHooks {
    pub(super) failpoint: Option<RuntimeTestFailpoint>,
}

#[derive(Debug)]
pub(super) struct PreparedCommit {
    pub(super) writer: DocumentWriter,
    pub(super) history_projection: HistoryProjection,
    pub(super) kind: DocumentEditActionKind,
    pub(super) primary: Option<LayerId>,
    pub(super) projection_generation: u64,
    pub(super) created_effect_use: Option<EffectId>,
}

#[derive(Debug)]
pub(super) struct PendingCommit {
    pub(super) receipt: JournalCommitReceipt,
    pub(super) prepared: PreparedCommit,
    pub(super) initial_error: Option<Box<ProjectError>>,
}

pub(crate) struct DocumentEditRuntime {
    pub(super) session: ProjectSession,
    pub(super) writer: DocumentWriter,
    pub(super) catalog: Arc<PluginCatalog>,
    pub(super) health: RuntimeHealth,
    pub(super) history_projection: HistoryProjection,
    #[cfg(test)]
    pub(super) test_hooks: RuntimeTestHooks,
}

impl DocumentEditRuntime {
    pub(crate) fn new(
        session: ProjectSession,
        document_writer: DocumentWriter,
        catalog: Arc<PluginCatalog>,
    ) -> Self {
        Self {
            session,
            writer: document_writer,
            catalog,
            health: RuntimeHealth::Healthy,
            history_projection: HistoryProjection::default(),
            #[cfg(test)]
            test_hooks: RuntimeTestHooks::default(),
        }
    }

    #[cfg(test)]
    pub(crate) fn set_test_failpoint(&mut self, failpoint: RuntimeTestFailpoint) {
        self.test_hooks.failpoint = Some(failpoint);
    }

    pub(crate) fn snapshot(&self) -> Arc<Document> {
        self.writer.snapshot()
    }

    pub(crate) fn document_revision(&self) -> u64 {
        self.writer.revision
    }

    pub(crate) fn is_write_blocked(&self) -> bool {
        matches!(self.health, RuntimeHealth::WriteBlocked(_))
    }

    pub(crate) fn blocked_commit_receipt(&self) -> Option<JournalCommitReceipt> {
        match &self.health {
            RuntimeHealth::Healthy => None,
            RuntimeHealth::WriteBlocked(pending) => Some(pending.receipt),
        }
    }

    #[cfg(test)]
    pub(crate) fn fail_reconcile_for_test(&mut self) {
        self.test_hooks.failpoint = Some(RuntimeTestFailpoint::Reconcile);
    }

    /// NothingToUndo/Redo の事前投影。空履歴時の沈黙入口を塞ぐ。
    pub(crate) fn can_undo(&self) -> bool {
        self.writer.can_undo()
    }

    pub(crate) fn can_redo(&self) -> bool {
        self.writer.can_redo()
    }

    pub(crate) fn project_root(&self) -> Option<PathBuf> {
        self.session.document_path().parent().map(PathBuf::from)
    }

    pub(crate) fn preview_trim(
        &self,
        request: TimelineTrimRequest,
    ) -> Result<Option<Arc<Document>>, DocumentEditRuntimeError> {
        let command = match request {
            TimelineTrimRequest::In { layer, new_start } => {
                self.writer.prepare_trim_clip_in(layer, new_start)?
            }
            TimelineTrimRequest::Out { layer, new_end } => {
                self.writer.prepare_trim_clip_out(layer, new_end)?
            }
        };
        let Some(command) = command else {
            return Ok(None);
        };
        let mut preview = (*self.writer.snapshot()).clone();
        command.apply(&mut preview)?;
        preview.validate()?;
        Ok(Some(Arc::new(preview)))
    }

    pub(super) fn commit_vector_shape(
        &mut self,
        request: PlaceRectangleRequest,
        shape: VectorShapeKind,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let next_projection_generation = next_projection_generation(current_projection_generation)?;
        let (command, layer_id, expected_live_next) =
            prepare_vector_shape_command(&self.writer.snapshot(), current_primary, request, shape)?;
        if self.writer.snapshot().layers.peek_next() != expected_live_next {
            return Err(DocumentEditRuntimeError::LayerIdReservationChanged);
        }
        self.commit_command(
            command,
            kind,
            current_primary,
            Some(layer_id),
            next_projection_generation,
            None,
        )
    }
}

pub(super) fn next_projection_generation(current: u64) -> Result<u64, DocumentEditRuntimeError> {
    current
        .checked_add(1)
        .ok_or(DocumentEditRuntimeError::ProjectionGenerationExhausted)
}
