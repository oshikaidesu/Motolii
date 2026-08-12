//! 一時診断を表示密度だけが異なる製品copyへ投影する純粋境界。

use motolii_doc::CommandKind;

use crate::{
    DiagnosticActionKind, DiagnosticEnvelope, DiagnosticFact, DiagnosticReasonCode,
    DiagnosticRecoverability, DiagnosticSubject, DomainIntent, UiStateOwner,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticDensity {
    Brief,
    Context,
    Inspect,
    Assistive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticProjection {
    density: DiagnosticDensity,
    reason: DiagnosticReasonCode,
    action: DiagnosticActionKind,
    subjects: Vec<DiagnosticSubject>,
    facts: Vec<DiagnosticFact>,
    recoverability: DiagnosticRecoverability,
    recovery_candidates: Vec<DomainIntent>,
    result: &'static str,
    cause: &'static str,
    recovery: &'static str,
    details: Vec<String>,
    assistive_text: String,
}

impl DiagnosticProjection {
    pub const fn density(&self) -> DiagnosticDensity {
        self.density
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

    pub const fn result(&self) -> &'static str {
        self.result
    }

    pub const fn cause(&self) -> &'static str {
        self.cause
    }

    pub const fn recovery(&self) -> Option<&'static str> {
        match self.density {
            DiagnosticDensity::Brief => None,
            DiagnosticDensity::Context
            | DiagnosticDensity::Inspect
            | DiagnosticDensity::Assistive => Some(self.recovery),
        }
    }

    pub fn details(&self) -> &[String] {
        match self.density {
            DiagnosticDensity::Brief | DiagnosticDensity::Context => &[],
            DiagnosticDensity::Inspect | DiagnosticDensity::Assistive => &self.details,
        }
    }

    pub fn assistive_text(&self) -> Option<&str> {
        (self.density == DiagnosticDensity::Assistive).then_some(&self.assistive_text)
    }
}

// CU-204Pへ渡すcrate-private seamを、公開re-exportや仮consumerなしで型検査する。
const _: [DiagnosticDensity; 4] = [
    DiagnosticDensity::Brief,
    DiagnosticDensity::Context,
    DiagnosticDensity::Inspect,
    DiagnosticDensity::Assistive,
];
const _: fn(&DiagnosticEnvelope, DiagnosticDensity) -> DiagnosticProjection = project_diagnostic;
const _: fn(&DiagnosticProjection) -> DiagnosticDensity = DiagnosticProjection::density;
const _: fn(&DiagnosticProjection) -> DiagnosticReasonCode = DiagnosticProjection::reason;
const _: fn(&DiagnosticProjection) -> DiagnosticActionKind = DiagnosticProjection::action;
const _: for<'a> fn(&'a DiagnosticProjection) -> &'a [DiagnosticSubject] =
    DiagnosticProjection::subjects;
const _: for<'a> fn(&'a DiagnosticProjection) -> &'a [DiagnosticFact] = DiagnosticProjection::facts;
const _: fn(&DiagnosticProjection) -> DiagnosticRecoverability =
    DiagnosticProjection::recoverability;
const _: for<'a> fn(&'a DiagnosticProjection) -> &'a [DomainIntent] =
    DiagnosticProjection::recovery_candidates;
const _: fn(&DiagnosticProjection) -> &'static str = DiagnosticProjection::result;
const _: fn(&DiagnosticProjection) -> &'static str = DiagnosticProjection::cause;
const _: fn(&DiagnosticProjection) -> Option<&'static str> = DiagnosticProjection::recovery;
const _: for<'a> fn(&'a DiagnosticProjection) -> &'a [String] = DiagnosticProjection::details;
const _: for<'a> fn(&'a DiagnosticProjection) -> Option<&'a str> =
    DiagnosticProjection::assistive_text;

pub fn project_diagnostic(
    envelope: &DiagnosticEnvelope,
    density: DiagnosticDensity,
) -> DiagnosticProjection {
    let (result, cause) = reason_copy(envelope.reason());
    let recovery = recovery_copy(envelope.recoverability());
    let details = project_details(envelope);
    let assistive_text = assistive_copy(result, cause, recovery, &details);

    DiagnosticProjection {
        density,
        reason: envelope.reason(),
        action: envelope.action(),
        subjects: envelope.subjects().to_vec(),
        facts: envelope.facts().to_vec(),
        recoverability: envelope.recoverability(),
        recovery_candidates: envelope.recovery_candidates().to_vec(),
        result,
        cause,
        recovery,
        details,
        assistive_text,
    }
}

const fn reason_copy(reason: DiagnosticReasonCode) -> (&'static str, &'static str) {
    match reason {
        DiagnosticReasonCode::UnknownCommand => {
            ("Command unavailable", "The command is not registered.")
        }
        DiagnosticReasonCode::EmptyDocumentCommands => {
            ("Edit not prepared", "No document commands were prepared.")
        }
        DiagnosticReasonCode::NonDocumentIntent => (
            "Edit not prepared",
            "The requested action does not own Document state.",
        ),
        DiagnosticReasonCode::DocumentCommandKindMismatch => (
            "Edit not prepared",
            "A prepared command does not match the requested document edit.",
        ),
        DiagnosticReasonCode::EffectDefinitionInUse => (
            "Effect definition is in use",
            "Blocking effect uses must be removed before deleting this definition.",
        ),
    }
}

const fn recovery_copy(recoverability: DiagnosticRecoverability) -> &'static str {
    match recoverability {
        DiagnosticRecoverability::RetryWithChangedInput => "Change the input and try again.",
        DiagnosticRecoverability::RequiresAnotherAction => "Complete the required action first.",
        DiagnosticRecoverability::Unrecoverable => "This action cannot be recovered.",
    }
}

fn project_details(envelope: &DiagnosticEnvelope) -> Vec<String> {
    let mut details = Vec::with_capacity(envelope.subjects().len() + envelope.facts().len());
    details.extend(envelope.subjects().iter().map(subject_copy));
    details.extend(envelope.facts().iter().map(fact_copy));
    details
}

fn subject_copy(subject: &DiagnosticSubject) -> String {
    match subject {
        DiagnosticSubject::AttemptedCommand(id) => {
            format!("Attempted command: {}.", id.as_str())
        }
        DiagnosticSubject::EffectDefinition(id) => {
            format!("Effect definition: {id}.")
        }
        DiagnosticSubject::BlockingEffectUse(id) => {
            format!("Blocking effect use: {id}.")
        }
    }
}

fn fact_copy(fact: &DiagnosticFact) -> String {
    match fact {
        DiagnosticFact::CommandKindMismatch {
            index,
            expected,
            actual,
        } => format!(
            "Command {index} expected {} but received {}.",
            command_kind_copy(*expected),
            command_kind_copy(*actual),
        ),
        DiagnosticFact::RequestedIntent(intent) => {
            format!("Requested intent: {}.", intent_copy(*intent))
        }
        DiagnosticFact::StateOwnerMismatch { expected, actual } => format!(
            "Expected state owner {} but received {}.",
            owner_copy(*expected),
            owner_copy(*actual),
        ),
        DiagnosticFact::BlockingSubjectCount { count } => {
            format!("Blocking effect uses: {count}.")
        }
    }
}

fn assistive_copy(result: &str, cause: &str, recovery: &str, details: &[String]) -> String {
    let mut text = format!("{result}. {cause} {recovery}");
    for detail in details {
        text.push(' ');
        text.push_str(detail);
    }
    text
}

const fn intent_copy(intent: DomainIntent) -> &'static str {
    match intent {
        DomainIntent::DeleteTargetedItems => "Delete targeted items",
        DomainIntent::EnableReduceMotion => "Enable reduce motion",
        DomainIntent::ResetWorkspaceProfile => "Reset workspace profile",
        DomainIntent::FitStageView => "Fit Stage view",
        DomainIntent::CancelInFlightGesture => "Cancel in-flight gesture",
        DomainIntent::Undo => "Undo",
        DomainIntent::Redo => "Redo",
    }
}

const fn owner_copy(owner: UiStateOwner) -> &'static str {
    match owner {
        UiStateOwner::Document => "Document",
        UiStateOwner::UserSettings => "User settings",
        UiStateOwner::WorkspaceProfile => "Workspace profile",
        UiStateOwner::ProjectSession => "Project session",
        UiStateOwner::Transient => "Transient",
    }
}

const fn command_kind_copy(kind: CommandKind) -> &'static str {
    match kind {
        CommandKind::SetProperty => "Set property",
        CommandKind::SetBlendMode => "Set blend mode",
        CommandKind::SetClippingMask => "Set clipping mask",
        CommandKind::SetTransformParent => "Set transform parent",
        CommandKind::AddEffect => "Add effect",
        CommandKind::RemoveEffect => "Remove effect",
        CommandKind::CreateEffect => "Create effect",
        CommandKind::LinkEffectUse => "Link effect use",
        CommandKind::UnlinkEffectUse => "Unlink effect use",
        CommandKind::SetEffectEnabled => "Set effect enabled",
        CommandKind::DeleteEffectDefinition => "Delete effect definition",
        CommandKind::CopyLocalEffect => "Copy local effect",
        CommandKind::AssetLifecycle => "Asset lifecycle",
        CommandKind::SetAudioComponentEnabled => "Set audio component enabled",
        CommandKind::SetAudioComponentGain => "Set audio component gain",
        CommandKind::AddTrackItem => "Add track item",
        CommandKind::RemoveTrackItem => "Remove track item",
        CommandKind::AddPositionKey => "Add position key",
        CommandKind::SetPositionKeyInterp => "Set position key interpolation",
        CommandKind::SetPositionKeyValue => "Set position key value",
        CommandKind::SetPositionKeyTime => "Set position key time",
        CommandKind::RemovePositionKey => "Remove position key",
        CommandKind::SetClipStart => "Set clip start",
        CommandKind::TrimClipIn => "Trim clip in",
        CommandKind::TrimClipOut => "Trim clip out",
    }
}

#[cfg(test)]
mod tests {
    use motolii_doc::{CommandError, CommandKind};

    use super::*;
    use crate::{
        adapt_command_error, adapt_document_command_request_error, adapt_input_router_error,
        CommandId, DocumentCommandRequestError, InputRouterError,
    };

    fn envelopes() -> Vec<DiagnosticEnvelope> {
        vec![
            adapt_input_router_error(&InputRouterError::UnknownCommandId {
                id: CommandId::try_new("motolii.missing.command").unwrap(),
            }),
            adapt_document_command_request_error(&DocumentCommandRequestError::EmptyCommands),
            adapt_document_command_request_error(&DocumentCommandRequestError::NonDocumentIntent {
                intent: DomainIntent::FitStageView,
            }),
            adapt_document_command_request_error(
                &DocumentCommandRequestError::CommandKindMismatch {
                    intent: DomainIntent::DeleteTargetedItems,
                    index: 2,
                    expected: CommandKind::RemoveTrackItem,
                    actual: CommandKind::SetProperty,
                },
            ),
            adapt_command_error(
                CommandKind::DeleteEffectDefinition,
                &CommandError::DefinitionInUse {
                    id: 41,
                    use_ids: vec![9, 3, 17],
                },
            )
            .unwrap(),
        ]
    }

    #[test]
    fn every_reason_preserves_typed_identity_across_all_densities() {
        let densities = [
            DiagnosticDensity::Brief,
            DiagnosticDensity::Context,
            DiagnosticDensity::Inspect,
            DiagnosticDensity::Assistive,
        ];
        let envelopes = envelopes();
        assert_eq!(envelopes.len(), 5);

        for envelope in &envelopes {
            for density in densities {
                let projection = project_diagnostic(envelope, density);
                assert_eq!(projection.density(), density);
                assert_eq!(projection.reason(), envelope.reason());
                assert_eq!(projection.action(), envelope.action());
                assert_eq!(projection.subjects(), envelope.subjects());
                assert_eq!(projection.facts(), envelope.facts());
                assert_eq!(projection.recoverability(), envelope.recoverability());
                assert_eq!(
                    projection.recovery_candidates(),
                    envelope.recovery_candidates()
                );
                assert!(projection.recovery_candidates().is_empty());
                assert!(!projection.result().is_empty());
                assert!(!projection.cause().is_empty());
            }
        }
    }

    #[test]
    fn density_only_changes_the_visible_amount() {
        for envelope in envelopes() {
            let brief = project_diagnostic(&envelope, DiagnosticDensity::Brief);
            let context = project_diagnostic(&envelope, DiagnosticDensity::Context);
            let inspect = project_diagnostic(&envelope, DiagnosticDensity::Inspect);
            let assistive = project_diagnostic(&envelope, DiagnosticDensity::Assistive);

            assert_eq!(brief.result(), context.result());
            assert_eq!(brief.cause(), context.cause());
            assert_eq!(brief.recovery(), None);
            assert!(brief.details().is_empty());
            assert!(brief.assistive_text().is_none());

            assert!(context.recovery().is_some());
            assert!(context.details().is_empty());
            assert!(context.assistive_text().is_none());

            assert!(inspect.recovery().is_some());
            assert_eq!(
                inspect.details().len(),
                envelope.subjects().len() + envelope.facts().len()
            );
            assert!(inspect.assistive_text().is_none());

            assert_eq!(assistive.details(), inspect.details());
            assert!(assistive
                .assistive_text()
                .is_some_and(|text| text.starts_with(assistive.result())));
        }
    }

    #[test]
    fn inspect_preserves_blocking_subject_order_with_no_actions() {
        let envelope = envelopes().pop().unwrap();
        let projection = project_diagnostic(&envelope, DiagnosticDensity::Inspect);
        assert_eq!(
            projection.details(),
            [
                "Effect definition: 41.",
                "Blocking effect use: 9.",
                "Blocking effect use: 3.",
                "Blocking effect use: 17.",
                "Blocking effect uses: 3.",
            ]
        );
        assert!(projection.recovery_candidates().is_empty());
    }

    #[test]
    fn clip_start_command_uses_the_existing_diagnostic_copy_route() {
        assert_eq!(
            command_kind_copy(CommandKind::SetClipStart),
            "Set clip start"
        );
    }

    #[test]
    fn position_key_interp_command_uses_the_existing_diagnostic_copy_route() {
        assert_eq!(
            command_kind_copy(CommandKind::SetPositionKeyInterp),
            "Set position key interpolation"
        );
    }
}
