//! 領域固有の拒否を表示前の一時診断へ適応する境界。

use motolii_doc::{CommandError, CommandKind};

use crate::{CommandId, DocumentCommandRequestError, DomainIntent, InputRouterError, UiStateOwner};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticReasonCode {
    UnknownCommand,
    EmptyDocumentCommands,
    NonDocumentIntent,
    DocumentCommandKindMismatch,
    EffectDefinitionInUse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticActionKind {
    InvokeCommand,
    PrepareDocumentEdit,
    DeleteEffectDefinition,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DiagnosticSubject {
    AttemptedCommand(CommandId),
    EffectDefinition(u64),
    BlockingEffectUse(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticFact {
    CommandKindMismatch {
        index: usize,
        expected: CommandKind,
        actual: CommandKind,
    },
    RequestedIntent(DomainIntent),
    StateOwnerMismatch {
        expected: UiStateOwner,
        actual: UiStateOwner,
    },
    BlockingSubjectCount {
        count: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticRecoverability {
    RetryWithChangedInput,
    RequiresAnotherAction,
    Unrecoverable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticEnvelope {
    reason: DiagnosticReasonCode,
    action: DiagnosticActionKind,
    subjects: Vec<DiagnosticSubject>,
    facts: Vec<DiagnosticFact>,
    recoverability: DiagnosticRecoverability,
    recovery_candidates: Vec<DomainIntent>,
}

impl DiagnosticEnvelope {
    fn new(
        reason: DiagnosticReasonCode,
        action: DiagnosticActionKind,
        subjects: Vec<DiagnosticSubject>,
        facts: Vec<DiagnosticFact>,
        recoverability: DiagnosticRecoverability,
        recovery_candidates: Vec<DomainIntent>,
    ) -> Self {
        Self {
            reason,
            action,
            subjects,
            facts,
            recoverability,
            recovery_candidates,
        }
    }

    pub const fn reason(&self) -> DiagnosticReasonCode {
        self.reason
    }

    pub const fn action(&self) -> DiagnosticActionKind {
        self.action
    }

    pub fn subjects(&self) -> &[DiagnosticSubject] {
        &self.subjects
    }

    pub fn facts(&self) -> &[DiagnosticFact] {
        &self.facts
    }

    pub const fn recoverability(&self) -> DiagnosticRecoverability {
        self.recoverability
    }

    pub fn recovery_candidates(&self) -> &[DomainIntent] {
        &self.recovery_candidates
    }
}

pub fn adapt_input_router_error(error: &InputRouterError) -> DiagnosticEnvelope {
    match error {
        InputRouterError::UnknownCommandId { id } => DiagnosticEnvelope::new(
            DiagnosticReasonCode::UnknownCommand,
            DiagnosticActionKind::InvokeCommand,
            vec![DiagnosticSubject::AttemptedCommand(id.clone())],
            Vec::new(),
            DiagnosticRecoverability::RetryWithChangedInput,
            Vec::new(),
        ),
    }
}

pub fn adapt_document_command_request_error(
    error: &DocumentCommandRequestError,
) -> DiagnosticEnvelope {
    match error {
        DocumentCommandRequestError::EmptyCommands => DiagnosticEnvelope::new(
            DiagnosticReasonCode::EmptyDocumentCommands,
            DiagnosticActionKind::PrepareDocumentEdit,
            Vec::new(),
            Vec::new(),
            DiagnosticRecoverability::RetryWithChangedInput,
            Vec::new(),
        ),
        DocumentCommandRequestError::NonDocumentIntent { intent } => DiagnosticEnvelope::new(
            DiagnosticReasonCode::NonDocumentIntent,
            DiagnosticActionKind::PrepareDocumentEdit,
            Vec::new(),
            vec![
                DiagnosticFact::RequestedIntent(*intent),
                DiagnosticFact::StateOwnerMismatch {
                    expected: UiStateOwner::Document,
                    actual: intent.owner(),
                },
            ],
            DiagnosticRecoverability::RetryWithChangedInput,
            Vec::new(),
        ),
        DocumentCommandRequestError::CommandKindMismatch {
            intent,
            index,
            expected,
            actual,
        } => DiagnosticEnvelope::new(
            DiagnosticReasonCode::DocumentCommandKindMismatch,
            DiagnosticActionKind::PrepareDocumentEdit,
            Vec::new(),
            vec![
                DiagnosticFact::RequestedIntent(*intent),
                DiagnosticFact::CommandKindMismatch {
                    index: *index,
                    expected: *expected,
                    actual: *actual,
                },
            ],
            DiagnosticRecoverability::RetryWithChangedInput,
            Vec::new(),
        ),
    }
}

pub fn adapt_command_error(
    action: CommandKind,
    error: &CommandError,
) -> Result<DiagnosticEnvelope, UnsupportedDiagnosticSource> {
    match (action, error) {
        (CommandKind::DeleteEffectDefinition, CommandError::DefinitionInUse { id, use_ids }) => {
            let mut subjects = Vec::with_capacity(use_ids.len() + 1);
            subjects.push(DiagnosticSubject::EffectDefinition(*id));
            subjects.extend(
                use_ids
                    .iter()
                    .copied()
                    .map(DiagnosticSubject::BlockingEffectUse),
            );
            Ok(DiagnosticEnvelope::new(
                DiagnosticReasonCode::EffectDefinitionInUse,
                DiagnosticActionKind::DeleteEffectDefinition,
                subjects,
                vec![DiagnosticFact::BlockingSubjectCount {
                    count: use_ids.len(),
                }],
                DiagnosticRecoverability::RequiresAnotherAction,
                Vec::new(),
            ))
        }
        _ => Err(UnsupportedDiagnosticSource::UnsupportedCommandError { action }),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum UnsupportedDiagnosticSource {
    #[error("command error has no diagnostic adapter for action {action:?}")]
    UnsupportedCommandError { action: CommandKind },
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FutureCommandLookupRejection {
        id: CommandId,
    }

    fn adapt_future_command_lookup(error: &FutureCommandLookupRejection) -> DiagnosticEnvelope {
        DiagnosticEnvelope::new(
            DiagnosticReasonCode::UnknownCommand,
            DiagnosticActionKind::InvokeCommand,
            vec![DiagnosticSubject::AttemptedCommand(error.id.clone())],
            Vec::new(),
            DiagnosticRecoverability::RetryWithChangedInput,
            Vec::new(),
        )
    }

    #[test]
    fn a_future_domain_rejection_needs_no_common_error_variant_or_public_constructor() {
        let error = FutureCommandLookupRejection {
            id: CommandId::try_new("motolii.future.lookup").unwrap(),
        };
        let envelope = adapt_future_command_lookup(&error);
        let existing_domain_envelope =
            adapt_input_router_error(&InputRouterError::UnknownCommandId { id: error.id });

        assert_eq!(envelope, existing_domain_envelope);
    }
}
