//! U0b-2: UI由来の操作をtoolkit非依存の目的へ畳むfixture。

use motolii_ui::{DomainIntent, DomainIntentError, UiStateOwner};

const REPRESENTATIVE_OPERATIONS: [(u16, DomainIntent, UiStateOwner); 5] = [
    (0, DomainIntent::DeleteTargetedItems, UiStateOwner::Document),
    (
        1,
        DomainIntent::EnableReduceMotion,
        UiStateOwner::UserSettings,
    ),
    (
        2,
        DomainIntent::ResetWorkspaceProfile,
        UiStateOwner::WorkspaceProfile,
    ),
    (3, DomainIntent::FitStageView, UiStateOwner::ProjectSession),
    (
        4,
        DomainIntent::CancelInFlightGesture,
        UiStateOwner::Transient,
    ),
];

#[test]
fn representative_ui_operations_become_typed_domain_intents() {
    for (adapter_kind, expected_intent, expected_owner) in REPRESENTATIVE_OPERATIONS {
        let intent = DomainIntent::try_from_adapter_kind(adapter_kind)
            .expect("representative adapter kind is known");
        assert_eq!(intent, expected_intent);
        assert_eq!(intent.owner(), expected_owner);
    }
}

#[test]
fn unknown_adapter_kind_is_a_typed_rejection() {
    assert_eq!(
        DomainIntent::try_from_adapter_kind(u16::MAX),
        Err(DomainIntentError::UnknownAdapterKind { got: u16::MAX })
    );
}

#[test]
fn domain_intent_source_has_no_input_or_persistence_contract() {
    let source = include_str!("../src/domain_intent.rs");
    let forbidden = [
        "egui::",
        "eframe::",
        "winit::",
        "KeyCode",
        "MouseButton",
        "CommandId",
        "Press",
        "Release",
        "DragStart",
        "DragUpdate",
        "DragEnd",
        "Serialize",
        "Deserialize",
        "motolii_doc::Command",
    ];

    for token in forbidden {
        assert!(
            !source.contains(token),
            "domain intent boundary must not contain {token}"
        );
    }
}
