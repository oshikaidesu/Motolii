use std::fs;
use std::path::Path;

use motolii_doc::{CommandError, CommandKind};
use motolii_ui::{
    adapt_command_error, adapt_document_command_request_error, adapt_input_router_error, CommandId,
    DiagnosticActionKind, DiagnosticEnvelope, DiagnosticFact, DiagnosticReasonCode,
    DiagnosticRecoverability, DiagnosticSubject, DocumentCommandRequestError, DomainIntent,
    InputRouterError, UiStateOwner, UnsupportedDiagnosticSource,
};

fn assert_common(
    envelope: &DiagnosticEnvelope,
    reason: DiagnosticReasonCode,
    action: DiagnosticActionKind,
    recoverability: DiagnosticRecoverability,
) {
    assert_eq!(envelope.reason(), reason);
    assert_eq!(envelope.action(), action);
    assert_eq!(envelope.recoverability(), recoverability);
    assert!(envelope.recovery_candidates().is_empty());
}

#[test]
fn unknown_command_preserves_the_attempted_stable_id() {
    let id = CommandId::try_new("motolii.missing.command").unwrap();
    let envelope = adapt_input_router_error(&InputRouterError::UnknownCommandId { id: id.clone() });
    assert_common(
        &envelope,
        DiagnosticReasonCode::UnknownCommand,
        DiagnosticActionKind::InvokeCommand,
        DiagnosticRecoverability::RetryWithChangedInput,
    );
    assert_eq!(
        envelope.subjects(),
        &[DiagnosticSubject::AttemptedCommand(id)]
    );
    assert!(envelope.facts().is_empty());
}

#[test]
fn every_prepared_request_rejection_matches_the_complete_mapping_table() {
    let cases = [
        (
            DocumentCommandRequestError::EmptyCommands,
            DiagnosticReasonCode::EmptyDocumentCommands,
            Vec::new(),
        ),
        (
            DocumentCommandRequestError::NonDocumentIntent {
                intent: DomainIntent::FitStageView,
            },
            DiagnosticReasonCode::NonDocumentIntent,
            vec![
                DiagnosticFact::RequestedIntent(DomainIntent::FitStageView),
                DiagnosticFact::StateOwnerMismatch {
                    expected: UiStateOwner::Document,
                    actual: UiStateOwner::ProjectSession,
                },
            ],
        ),
        (
            DocumentCommandRequestError::CommandKindMismatch {
                intent: DomainIntent::DeleteTargetedItems,
                index: 2,
                expected: CommandKind::RemoveTrackItem,
                actual: CommandKind::SetProperty,
            },
            DiagnosticReasonCode::DocumentCommandKindMismatch,
            vec![
                DiagnosticFact::RequestedIntent(DomainIntent::DeleteTargetedItems),
                DiagnosticFact::CommandKindMismatch {
                    index: 2,
                    expected: CommandKind::RemoveTrackItem,
                    actual: CommandKind::SetProperty,
                },
            ],
        ),
    ];

    for (error, reason, expected_facts) in cases {
        let envelope = adapt_document_command_request_error(&error);
        assert_common(
            &envelope,
            reason,
            DiagnosticActionKind::PrepareDocumentEdit,
            DiagnosticRecoverability::RetryWithChangedInput,
        );
        assert!(envelope.subjects().is_empty());
        assert_eq!(envelope.facts(), expected_facts);
    }
}

#[test]
fn definition_in_use_preserves_all_subjects_in_source_order() {
    let error = CommandError::DefinitionInUse {
        id: 41,
        use_ids: vec![9, 3, 17],
    };
    let envelope =
        adapt_command_error(CommandKind::DeleteEffectDefinition, &error).expect("supported pair");
    assert_common(
        &envelope,
        DiagnosticReasonCode::EffectDefinitionInUse,
        DiagnosticActionKind::DeleteEffectDefinition,
        DiagnosticRecoverability::RequiresAnotherAction,
    );
    assert_eq!(
        envelope.subjects(),
        &[
            DiagnosticSubject::EffectDefinition(41),
            DiagnosticSubject::BlockingEffectUse(9),
            DiagnosticSubject::BlockingEffectUse(3),
            DiagnosticSubject::BlockingEffectUse(17),
        ]
    );
    assert_eq!(
        envelope.facts(),
        &[DiagnosticFact::BlockingSubjectCount { count: 3 }]
    );
}

#[test]
fn unsupported_command_pairs_never_fall_back_to_a_generic_envelope() {
    let in_use = CommandError::DefinitionInUse {
        id: 1,
        use_ids: vec![2],
    };
    assert_eq!(
        adapt_command_error(CommandKind::RemoveEffect, &in_use),
        Err(UnsupportedDiagnosticSource::UnsupportedCommandError {
            action: CommandKind::RemoveEffect,
        })
    );
    assert_eq!(
        adapt_command_error(
            CommandKind::DeleteEffectDefinition,
            &CommandError::LayerNotFound(7),
        ),
        Err(UnsupportedDiagnosticSource::UnsupportedCommandError {
            action: CommandKind::DeleteEffectDefinition,
        })
    );
}

#[test]
fn public_adapter_signatures_and_envelope_types_stay_read_only_and_transient() {
    let _: fn(&InputRouterError) -> DiagnosticEnvelope = adapt_input_router_error;
    let _: fn(&DocumentCommandRequestError) -> DiagnosticEnvelope =
        adapt_document_command_request_error;
    let _: fn(
        CommandKind,
        &CommandError,
    ) -> Result<DiagnosticEnvelope, UnsupportedDiagnosticSource> = adapt_command_error;

    for name in [
        std::any::type_name::<DiagnosticEnvelope>(),
        std::any::type_name::<DiagnosticReasonCode>(),
        std::any::type_name::<DiagnosticActionKind>(),
        std::any::type_name::<DiagnosticSubject>(),
        std::any::type_name::<DiagnosticFact>(),
        std::any::type_name::<DiagnosticRecoverability>(),
    ] {
        for forbidden in ["egui", "eframe", "winit"] {
            assert!(!name.contains(forbidden), "{name} contains {forbidden}");
        }
    }

    let source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/diagnostic.rs"))
            .unwrap();
    for forbidden in [
        "Serialize",
        "Deserialize",
        "serde::",
        ".to_string(",
        "format!(",
        "egui::",
        "eframe::",
        "winit::",
        "DocumentWriter",
        "RenderWorker",
    ] {
        assert!(
            !source.contains(forbidden),
            "diagnostic source contains forbidden boundary {forbidden}"
        );
    }
    assert!(!source.contains("pub fn new("));
}
