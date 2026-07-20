use std::fs;
use std::path::Path;

use motolii_ui::{InteractionState, InteractionStateMachine, InteractionTransitionError};

const STATES: [InteractionState; 6] = [
    InteractionState::Discover,
    InteractionState::Target,
    InteractionState::Preview,
    InteractionState::Commit,
    InteractionState::Cancel,
    InteractionState::Inspect,
];

fn machine_at(state: InteractionState) -> InteractionStateMachine {
    let mut machine = InteractionStateMachine::new();
    match state {
        InteractionState::Discover => {}
        InteractionState::Target => machine.transition(InteractionState::Target).unwrap(),
        InteractionState::Preview => {
            machine.transition(InteractionState::Target).unwrap();
            machine.transition(InteractionState::Preview).unwrap();
        }
        InteractionState::Commit => {
            machine.transition(InteractionState::Target).unwrap();
            machine.transition(InteractionState::Commit).unwrap();
        }
        InteractionState::Cancel => {
            machine.transition(InteractionState::Target).unwrap();
            machine.transition(InteractionState::Cancel).unwrap();
        }
        InteractionState::Inspect => {
            machine.transition(InteractionState::Target).unwrap();
            machine.transition(InteractionState::Commit).unwrap();
            machine.transition(InteractionState::Inspect).unwrap();
        }
    }
    machine
}

const CONTRACT_TRANSITIONS: [(InteractionState, InteractionState); 9] = [
    (InteractionState::Discover, InteractionState::Target),
    (InteractionState::Target, InteractionState::Preview),
    (InteractionState::Target, InteractionState::Commit),
    (InteractionState::Target, InteractionState::Cancel),
    (InteractionState::Preview, InteractionState::Commit),
    (InteractionState::Preview, InteractionState::Cancel),
    (InteractionState::Commit, InteractionState::Inspect),
    (InteractionState::Cancel, InteractionState::Discover),
    (InteractionState::Inspect, InteractionState::Discover),
];

#[test]
fn all_thirty_six_transition_pairs_match_the_contract() {
    let mut allowed_count = 0;
    let mut rejected_count = 0;

    for from in STATES {
        for to in STATES {
            let mut machine = machine_at(from);
            let result = machine.transition(to);
            if CONTRACT_TRANSITIONS.contains(&(from, to)) {
                result.unwrap();
                assert_eq!(machine.state(), to);
                allowed_count += 1;
            } else {
                let error = result.unwrap_err();
                assert_eq!(error.from(), from);
                assert_eq!(error.to(), to);
                assert_eq!(machine.state(), from);
                rejected_count += 1;
            }
        }
    }

    assert_eq!(allowed_count, 9);
    assert_eq!(rejected_count, 27);
}

#[test]
fn dangerous_shortcuts_and_same_state_transitions_are_rejected() {
    for (from, to) in [
        (InteractionState::Discover, InteractionState::Commit),
        (InteractionState::Commit, InteractionState::Cancel),
    ] {
        let mut machine = machine_at(from);
        let error = machine.transition(to).unwrap_err();
        assert_eq!((error.from(), error.to()), (from, to));
        assert_eq!(machine.state(), from);
    }

    for state in STATES {
        let mut machine = machine_at(state);
        let error = machine.transition(state).unwrap_err();
        assert_eq!((error.from(), error.to()), (state, state));
        assert_eq!(machine.state(), state);
    }
}

#[test]
fn preview_and_previewless_commit_paths_return_to_discover() {
    for path in [
        &[
            InteractionState::Target,
            InteractionState::Preview,
            InteractionState::Commit,
            InteractionState::Inspect,
            InteractionState::Discover,
        ][..],
        &[
            InteractionState::Target,
            InteractionState::Commit,
            InteractionState::Inspect,
            InteractionState::Discover,
        ][..],
    ] {
        let mut machine = InteractionStateMachine::new();
        for state in path {
            machine.transition(*state).unwrap();
        }
        assert_eq!(machine.state(), InteractionState::Discover);
    }
}

#[test]
fn public_types_do_not_gain_persistence_domain_or_toolkit_dependencies() {
    fn assert_toolkit_free_type<T>() {
        let name = std::any::type_name::<T>();
        for forbidden in ["egui", "eframe", "winit"] {
            assert!(!name.contains(forbidden), "{name} contains {forbidden}");
        }
    }

    assert_toolkit_free_type::<InteractionState>();
    assert_toolkit_free_type::<InteractionStateMachine>();
    assert_toolkit_free_type::<InteractionTransitionError>();

    let source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/interaction_state.rs"))
            .unwrap();
    for forbidden in [
        "Serialize",
        "Deserialize",
        "serde::",
        "egui::",
        "eframe::",
        "winit::",
        "Document",
        "Command",
        "DomainIntent",
        "EntryKind",
        "TargetId",
        "Pointer",
        "Position",
        "Widget",
        "String",
        "&str",
        "px:",
        "dpi:",
        "label:",
        "display_name",
    ] {
        assert!(
            !source.contains(forbidden),
            "interaction state source contains forbidden boundary {forbidden}"
        );
    }
}
