//! U0c-2: 正規event種別とIME preedit gateの審判。

use motolii_ui::{
    builtin_command_registry, CommandId, DomainIntent, ImeGateState, InputPhase, InputRouter,
    InputRouterError, NormalizedInput, RouterOutput, SafetyInterrupt,
};

fn router() -> InputRouter {
    InputRouter::new(builtin_command_registry().expect("built-in registry must be valid"))
}

fn command(id: &str, phase: InputPhase) -> NormalizedInput {
    NormalizedInput::Command {
        phase,
        id: CommandId::try_new(id).expect("fixture ID must be valid"),
    }
}

#[test]
fn all_seven_phases_remain_distinct() {
    let phases = [
        InputPhase::Press,
        InputPhase::Release,
        InputPhase::Click,
        InputPhase::DragStart,
        InputPhase::DragUpdate,
        InputPhase::DragEnd,
        InputPhase::Cancel,
    ];
    let mut phase_router = router();
    let outputs: Vec<_> = phases
        .into_iter()
        .map(|phase| phase_router.route(NormalizedInput::Phase(phase)).unwrap())
        .collect();

    assert_eq!(
        outputs,
        phases.map(RouterOutput::Phase),
        "Click must not collapse into Press or Release"
    );

    let mut state = router();
    state
        .route(NormalizedInput::Phase(InputPhase::Press))
        .unwrap();
    assert!(state.gesture_in_flight());
    state
        .route(NormalizedInput::Phase(InputPhase::Click))
        .unwrap();
    assert!(!state.gesture_in_flight());

    let mut press_router = router();
    let mut click_router = router();
    let press = press_router
        .route(command("motolii.view.fit_stage", InputPhase::Press))
        .unwrap();
    let click = click_router
        .route(command("motolii.view.fit_stage", InputPhase::Click))
        .unwrap();
    assert_ne!(press, click);
    assert!(matches!(
        press,
        RouterOutput::Intent {
            phase: InputPhase::Press,
            ..
        }
    ));
    assert!(matches!(
        click,
        RouterOutput::Intent {
            phase: InputPhase::Click,
            ..
        }
    ));
}

#[test]
fn preedit_suppresses_shortcuts_including_cancel_command() {
    let mut router = router();
    router
        .route(NormalizedInput::Phase(InputPhase::DragStart))
        .unwrap();
    router.set_ime_gate(ImeGateState::PreeditActive);

    for id in [
        "motolii.edit.delete_targeted_items",
        "motolii.gesture.cancel",
    ] {
        assert!(matches!(
            router.route(command(id, InputPhase::Press)).unwrap(),
            RouterOutput::ShortcutSuppressed { .. }
        ));
    }
    assert!(router.gesture_in_flight());
    assert_eq!(
        router.route(NormalizedInput::ImeOwned).unwrap(),
        RouterOutput::ImeOwned
    );
}

#[test]
fn safety_interrupt_cancels_in_flight_gesture_even_during_preedit() {
    for source in [
        SafetyInterrupt::PointerCaptureLost,
        SafetyInterrupt::WindowFocusLost,
    ] {
        let mut router = router();
        router
            .route(NormalizedInput::Phase(InputPhase::DragStart))
            .unwrap();
        router.set_ime_gate(ImeGateState::PreeditActive);

        assert_eq!(
            router
                .route(NormalizedInput::SafetyInterrupt(source))
                .unwrap(),
            RouterOutput::SafetyCancel {
                source,
                intent: DomainIntent::CancelInFlightGesture,
            }
        );
        assert!(!router.gesture_in_flight());
        assert_eq!(router.ime_gate(), ImeGateState::PreeditActive);
    }
}

#[test]
fn idle_safety_and_cancel_command_do_not_emit_cancel_intent() {
    let mut router = router();
    assert!(matches!(
        router
            .route(NormalizedInput::SafetyInterrupt(
                SafetyInterrupt::WindowFocusLost
            ))
            .unwrap(),
        RouterOutput::SafetyIgnored { .. }
    ));
    assert!(matches!(
        router
            .route(command("motolii.gesture.cancel", InputPhase::Press))
            .unwrap(),
        RouterOutput::CancelCommandIgnored { .. }
    ));
}

#[test]
fn registry_is_the_only_command_to_intent_mapping() {
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
    ] {
        let mut router = router();
        let id = CommandId::try_new(id).unwrap();
        assert_eq!(
            router
                .route(NormalizedInput::Command {
                    phase: InputPhase::Click,
                    id: id.clone(),
                })
                .unwrap(),
            RouterOutput::Intent {
                phase: InputPhase::Click,
                id,
                intent: expected_intent,
            }
        );
    }
}

#[test]
fn same_normalized_sequence_is_deterministic() {
    let sequence = [
        NormalizedInput::Phase(InputPhase::Press),
        command("motolii.view.fit_stage", InputPhase::Click),
        NormalizedInput::Phase(InputPhase::Release),
    ];
    let run = |mut router: InputRouter| {
        sequence
            .clone()
            .into_iter()
            .map(|input| router.route(input))
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    };

    assert_eq!(run(router()), run(router()));
}

#[test]
fn unknown_command_id_is_typed_rejection() {
    let id = CommandId::try_new("motolii.unknown.command").unwrap();
    for ime_gate in [ImeGateState::Inactive, ImeGateState::PreeditActive] {
        let mut router = router();
        router.set_ime_gate(ime_gate);
        assert_eq!(
            router
                .route(NormalizedInput::Command {
                    phase: InputPhase::Press,
                    id: id.clone(),
                })
                .unwrap_err(),
            InputRouterError::UnknownCommandId { id: id.clone() }
        );
    }
}

#[test]
fn router_source_has_no_physical_or_persistence_contract() {
    let source = include_str!("../src/input_router.rs");
    for token in [
        "egui::",
        "eframe::",
        "winit::",
        "KeyCode",
        "MouseButton",
        "Modifiers",
        "Serialize",
        "Deserialize",
        "motolii_doc::",
    ] {
        assert!(
            !source.contains(token),
            "input router boundary must not contain {token}"
        );
    }
}
