//! U0c-1: 安定Command IDとmetadata registryの審判。

use motolii_ui::{
    builtin_command_registry, CommandId, CommandIdError, CommandMetadata, CommandRegistry,
    CommandRegistryError, DomainIntent,
};

fn metadata(id: &str, display_name: &str, intent: DomainIntent) -> CommandMetadata {
    CommandMetadata::new(
        CommandId::try_new(id).expect("fixture command ID must be valid"),
        display_name,
        intent,
    )
}

fn complete_metadata() -> Vec<CommandMetadata> {
    vec![
        metadata(
            "motolii.edit.delete_targeted_items",
            "Delete",
            DomainIntent::DeleteTargetedItems,
        ),
        metadata(
            "motolii.settings.enable_reduce_motion",
            "Reduce motion",
            DomainIntent::EnableReduceMotion,
        ),
        metadata(
            "motolii.workspace.reset_profile",
            "Reset workspace",
            DomainIntent::ResetWorkspaceProfile,
        ),
        metadata(
            "motolii.view.fit_stage",
            "Fit stage",
            DomainIntent::FitStageView,
        ),
        metadata(
            "motolii.gesture.cancel",
            "Cancel",
            DomainIntent::CancelInFlightGesture,
        ),
    ]
}

#[test]
fn command_id_accepts_the_decided_grammar() {
    for value in [
        "motolii.relative_move_drag",
        "motolii.edit.delete_targeted_items",
        "motolii.a.b2_c",
    ] {
        assert_eq!(
            CommandId::try_new(value)
                .expect("decided grammar must accept fixture")
                .as_str(),
            value
        );
    }
}

#[test]
fn command_id_rejects_empty_and_invalid_grammar() {
    assert_eq!(CommandId::try_new(""), Err(CommandIdError::Empty));

    for value in [
        "motolii",
        "other.edit",
        "motolii.",
        "motolii..edit",
        "motolii.Edit",
        "motolii.2edit",
        "motolii.edit-name",
        "motolii.edit name",
    ] {
        assert!(matches!(
            CommandId::try_new(value),
            Err(CommandIdError::InvalidGrammar { .. })
        ));
    }
}

#[test]
fn duplicate_id_is_rejected() {
    let mut entries = complete_metadata();
    entries[1].id = entries[0].id.clone();

    assert!(matches!(
        CommandRegistry::try_new(entries),
        Err(CommandRegistryError::DuplicateId { .. })
    ));
}

#[test]
fn missing_and_duplicate_intent_are_rejected() {
    let mut missing = complete_metadata();
    missing.pop();
    assert_eq!(
        CommandRegistry::try_new(missing).unwrap_err(),
        CommandRegistryError::MissingIntent {
            intent: DomainIntent::CancelInFlightGesture
        }
    );

    let mut duplicate = complete_metadata();
    duplicate[1].intent = DomainIntent::DeleteTargetedItems;
    assert_eq!(
        CommandRegistry::try_new(duplicate).unwrap_err(),
        CommandRegistryError::DuplicateIntent {
            intent: DomainIntent::DeleteTargetedItems
        }
    );
}

#[test]
fn changing_display_name_does_not_change_command_id() {
    let id = CommandId::try_new("motolii.view.fit_stage").unwrap();
    let before = CommandMetadata::new(id.clone(), "Fit stage", DomainIntent::FitStageView);
    let after = CommandMetadata::new(id, "Stageを全体表示", DomainIntent::FitStageView);

    assert_eq!(before.id, after.id);
    assert_ne!(before.display_name, after.display_name);
}

#[test]
fn builtin_registry_covers_every_domain_intent_once() {
    let registry = builtin_command_registry().expect("built-in registry must be valid");
    assert_eq!(registry.iter().len(), DomainIntent::ALL.len());

    for intent in DomainIntent::ALL {
        assert_eq!(
            registry
                .iter()
                .filter(|metadata| metadata.intent == intent)
                .count(),
            1
        );
    }

    for (id, expected_intent) in [
        (
            "motolii.edit.delete_targeted_items",
            DomainIntent::DeleteTargetedItems,
        ),
        (
            "motolii.settings.enable_reduce_motion",
            DomainIntent::EnableReduceMotion,
        ),
        (
            "motolii.workspace.reset_profile",
            DomainIntent::ResetWorkspaceProfile,
        ),
        ("motolii.view.fit_stage", DomainIntent::FitStageView),
        (
            "motolii.gesture.cancel",
            DomainIntent::CancelInFlightGesture,
        ),
    ] {
        let id = CommandId::try_new(id).unwrap();
        assert_eq!(
            registry
                .get(&id)
                .expect("built-in ID must be registered")
                .intent,
            expected_intent
        );
    }
}
