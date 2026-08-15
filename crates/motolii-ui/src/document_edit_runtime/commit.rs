//! 準備済みcommandをjournalへ先に焼き、成功時だけlive writerへ載せる。

use motolii_doc::{
    Command, EffectId, JournalCommitReconcileOutcome, JournalEdit, LayerId, SaveProjectOptions,
};

use super::action::DocumentEditActionKind;
use super::error::{DocumentEditRuntimeError, PublishedDocument};
#[cfg(test)]
use super::runtime::RuntimeTestFailpoint;
use super::runtime::{
    DocumentEditRuntime, HistoryDirection, PendingCommit, PreparedCommit, PreparedHistoryAction,
    RuntimeHealth,
};

impl DocumentEditRuntime {
    pub(super) fn commit_command(
        &mut self,
        command: Command,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        success_primary: Option<LayerId>,
        projection_generation: u64,
        created_effect_use: Option<EffectId>,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let mut validated_candidate = (*self.writer.snapshot()).clone();
        command.apply(&mut validated_candidate)?;
        validated_candidate.validate()?;
        validated_candidate.prepare_plugins(&self.catalog)?;
        let mut writer = self.writer.clone();
        writer.apply_macro(vec![command.clone()])?;
        let mut history_projection = self.history_projection.clone();
        history_projection.record_forward(command.clone());
        let primary = success_primary
            .or(current_primary)
            .filter(|id| writer.find_envelope(*id).is_some());
        let prepared = PreparedCommit {
            writer,
            history_projection,
            kind,
            primary,
            projection_generation,
            created_effect_use,
        };
        self.commit_prepared(command, prepared)
    }

    pub(super) fn commit_history_action(
        &mut self,
        action: PreparedHistoryAction,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let mut writer = self.writer.clone();
        match action.direction {
            HistoryDirection::Undo => writer.undo()?,
            HistoryDirection::Redo => writer.redo()?,
        }
        let mut history_projection = self.history_projection.clone();
        history_projection.accept(action.direction)?;
        let primary = current_primary.filter(|id| writer.find_envelope(*id).is_some());
        let created_effect_use = if let HistoryDirection::Redo = action.direction {
            match &action.durable_command {
                Command::CreateEffect { target, use_, .. }
                    if current_primary.as_ref() == Some(target) =>
                {
                    Some(use_.id)
                }
                _ => None,
            }
        } else {
            None
        };
        let durable_command = action.durable_command;
        let prepared = PreparedCommit {
            writer,
            history_projection,
            kind,
            primary,
            projection_generation,
            created_effect_use,
        };
        self.commit_prepared(durable_command, prepared)
    }

    fn commit_prepared(
        &mut self,
        command: Command,
        prepared: PreparedCommit,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let candidate = prepared.writer.snapshot();
        candidate.validate()?;
        candidate.prepare_plugins(&self.catalog)?;
        let options = SaveProjectOptions {
            limits: *self.session.limits(),
            journal_edit: Some(JournalEdit::new(command)),
            checkpoint: false,
            ..SaveProjectOptions::default()
        };
        match self
            .session
            .save_with_journal_outcome(candidate.as_ref(), &options)
        {
            Ok(Some(_receipt)) => {
                #[cfg(test)]
                if self.test_hooks.failpoint == Some(RuntimeTestFailpoint::DeferAfterDurableCommit)
                {
                    self.test_hooks.failpoint = None;
                    self.health = RuntimeHealth::WriteBlocked(Box::new(PendingCommit {
                        receipt: _receipt,
                        prepared,
                        initial_error: None,
                    }));
                    return Err(DocumentEditRuntimeError::DocumentWriteBlocked {
                        receipt: _receipt,
                    });
                }
                Ok(Some(self.accept_prepared(prepared)))
            }
            Ok(None) => Err(DocumentEditRuntimeError::MissingJournalCommitReceipt),
            Err(error) => {
                let Some(receipt) = error.uncertain_commit_receipt() else {
                    return Err(DocumentEditRuntimeError::JournalCommit(error));
                };
                self.health = RuntimeHealth::WriteBlocked(Box::new(PendingCommit {
                    receipt,
                    prepared,
                    initial_error: Some(error),
                }));
                self.reconcile_pending_commit()
            }
        }
    }

    fn accept_prepared(&mut self, prepared: PreparedCommit) -> PublishedDocument {
        self.writer = prepared.writer;
        self.history_projection = prepared.history_projection;
        PublishedDocument {
            kind: prepared.kind,
            revision: self.writer.revision,
            snapshot: self.writer.snapshot(),
            primary: prepared.primary,
            projection_generation: prepared.projection_generation,
            created_effect_use: prepared.created_effect_use,
        }
    }

    pub(crate) fn reconcile_pending_commit(
        &mut self,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let RuntimeHealth::WriteBlocked(pending) =
            std::mem::replace(&mut self.health, RuntimeHealth::Healthy)
        else {
            return Ok(None);
        };
        let mut pending = *pending;

        #[cfg(test)]
        if self.test_hooks.failpoint == Some(RuntimeTestFailpoint::Reconcile) {
            self.test_hooks.failpoint = None;
            let receipt = pending.receipt;
            self.health = RuntimeHealth::WriteBlocked(Box::new(pending));
            return Err(DocumentEditRuntimeError::DocumentWriteBlocked { receipt });
        }

        match self.session.reconcile_journal_commit(pending.receipt) {
            Ok(JournalCommitReconcileOutcome::NotCommitted) => {
                let Some(error) = pending.initial_error.take() else {
                    let receipt = pending.receipt;
                    self.health = RuntimeHealth::WriteBlocked(Box::new(pending));
                    return Err(DocumentEditRuntimeError::CommitReceiptNotObserved { receipt });
                };
                Err(DocumentEditRuntimeError::JournalCommit(error))
            }
            Ok(JournalCommitReconcileOutcome::Committed(recovered)) => {
                if recovered.document != *pending.prepared.writer.snapshot() {
                    let receipt = pending.receipt;
                    self.health = RuntimeHealth::WriteBlocked(Box::new(pending));
                    return Err(DocumentEditRuntimeError::ReconciledDocumentMismatch { receipt });
                }
                Ok(Some(self.accept_prepared(pending.prepared)))
            }
            Err(source) => {
                let receipt = pending.receipt;
                self.health = RuntimeHealth::WriteBlocked(Box::new(pending));
                Err(DocumentEditRuntimeError::JournalReconcile { receipt, source })
            }
        }
    }

    #[cfg(test)]
    pub(super) fn history_lengths(&self) -> (usize, usize) {
        (self.writer.undo_len(), self.writer.redo_len())
    }

    #[cfg(test)]
    pub(super) fn revision(&self) -> u64 {
        self.writer.revision
    }
}
