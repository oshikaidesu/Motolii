//! U0d-1: builtin base + user deltaの純粋resolver審判。

use motolii_ui::{
    builtin_command_registry, resolve_keymap, AsciiKey, AsciiKeyError, Binding, BuiltinKeymap,
    CommandId, DeltaOperation, EffectiveTrigger, Gesture, InputPhase, KeyToken, KeymapDelta,
    KeymapDiagnostic, Modifier, ModifierError, Modifiers, PlatformBindingConstraints,
    PlatformCommandModifier,
};

fn id(value: &str) -> CommandId {
    CommandId::try_new(value).unwrap()
}

fn modifiers(values: &[Modifier]) -> Modifiers {
    Modifiers::try_new(values.iter().copied()).unwrap()
}

fn key(value: char) -> KeyToken {
    KeyToken::Ascii(AsciiKey::try_new(value).unwrap())
}

fn keyboard(value: char, mods: &[Modifier], phase: InputPhase) -> Gesture {
    Gesture::Keyboard {
        key: key(value),
        modifiers: modifiers(mods),
        phase,
    }
}

fn platform(
    command_modifier: PlatformCommandModifier,
    reserved: Vec<EffectiveTrigger>,
) -> PlatformBindingConstraints {
    PlatformBindingConstraints::new(command_modifier, reserved)
}

fn empty_delta() -> KeymapDelta {
    KeymapDelta::default()
}

fn base() -> BuiltinKeymap {
    BuiltinKeymap::new(
        1,
        vec![
            Binding {
                gesture: Gesture::Keyboard {
                    key: KeyToken::Delete,
                    modifiers: Modifiers::default(),
                    phase: InputPhase::Press,
                },
                command: id("motolii.edit.delete_targeted_items"),
            },
            Binding {
                gesture: keyboard('f', &[Modifier::Primary], InputPhase::Press),
                command: id("motolii.view.fit_stage"),
            },
        ],
    )
}

#[test]
fn token_and_modifiers_follow_the_decided_normal_form() {
    assert_eq!(AsciiKey::try_new('a').unwrap().as_char(), 'a');
    assert_eq!(
        AsciiKey::try_new('A'),
        Err(AsciiKeyError::NotLowercaseLetterOrDigit { value: 'A' })
    );
    assert_eq!(
        modifiers(&[Modifier::Shift, Modifier::Alt, Modifier::Shift])
            .iter()
            .collect::<Vec<_>>(),
        vec![Modifier::Alt, Modifier::Shift]
    );
    assert_eq!(
        Modifiers::try_new([Modifier::Primary, Modifier::Control]),
        Err(ModifierError::PrimaryWithExplicitCommandModifier)
    );
}

#[test]
fn add_supports_multiple_gestures_without_mutating_base() {
    let base = base();
    let before = base.clone();
    let added = keyboard('d', &[Modifier::Primary], InputPhase::Press);
    let delta = KeymapDelta::new(vec![DeltaOperation::Add(Binding {
        gesture: added,
        command: id("motolii.edit.delete_targeted_items"),
    })]);

    let result = resolve_keymap(
        &base,
        &delta,
        &platform(PlatformCommandModifier::Control, vec![]),
        &builtin_command_registry().unwrap(),
    );

    assert_eq!(base, before);
    assert_eq!(result.iter().len(), 3);
    assert!(result.diagnostics().is_empty());
}

#[test]
fn replace_and_disable_target_exact_base_gestures() {
    let base = base();
    let delete = base.bindings()[0].gesture.clone();
    let fit = base.bindings()[1].gesture.clone();
    let delta = KeymapDelta::new(vec![
        DeltaOperation::Replace(Binding {
            gesture: delete,
            command: id("motolii.view.fit_stage"),
        }),
        DeltaOperation::Disable { gesture: fit },
    ]);
    let result = resolve_keymap(
        &base,
        &delta,
        &platform(PlatformCommandModifier::Control, vec![]),
        &builtin_command_registry().unwrap(),
    );

    assert_eq!(result.iter().len(), 1);
    assert_eq!(
        result.iter().next().unwrap().1.as_str(),
        "motolii.view.fit_stage"
    );
}

#[test]
fn invalid_delta_targets_are_diagnostics_and_not_applied() {
    let base = base();
    let delete = base.bindings()[0].gesture.clone();
    let missing = keyboard('m', &[], InputPhase::Press);
    let result = resolve_keymap(
        &base,
        &KeymapDelta::new(vec![
            DeltaOperation::Add(Binding {
                gesture: delete.clone(),
                command: id("motolii.view.fit_stage"),
            }),
            DeltaOperation::Replace(Binding {
                gesture: missing.clone(),
                command: id("motolii.view.fit_stage"),
            }),
            DeltaOperation::Disable {
                gesture: keyboard('n', &[], InputPhase::Press),
            },
        ]),
        &platform(PlatformCommandModifier::Control, vec![]),
        &builtin_command_registry().unwrap(),
    );

    assert!(result
        .diagnostics()
        .contains(&KeymapDiagnostic::AddTargetsBase { gesture: delete }));
    assert!(result
        .diagnostics()
        .contains(&KeymapDiagnostic::ReplaceTargetMissing { gesture: missing }));
    assert!(result
        .diagnostics()
        .iter()
        .any(|item| matches!(item, KeymapDiagnostic::DisableTargetMissing { .. })));
}

#[test]
fn delta_order_is_irrelevant_and_duplicate_target_is_not_applied() {
    let gesture = keyboard('x', &[], InputPhase::Press);
    let first = DeltaOperation::Add(Binding {
        gesture: gesture.clone(),
        command: id("motolii.view.fit_stage"),
    });
    let second = DeltaOperation::Disable {
        gesture: gesture.clone(),
    };
    let resolve = |operations| {
        resolve_keymap(
            &base(),
            &KeymapDelta::new(operations),
            &platform(PlatformCommandModifier::Control, vec![]),
            &builtin_command_registry().unwrap(),
        )
    };

    let a = resolve(vec![first.clone(), second.clone()]);
    let b = resolve(vec![second, first]);
    assert_eq!(a, b);
    assert!(a
        .diagnostics()
        .contains(&KeymapDiagnostic::MultipleDeltaOperations { gesture }));
}

#[test]
fn primary_conflict_is_decided_after_platform_expansion() {
    let control_f = keyboard('f', &[Modifier::Control], InputPhase::Press);
    let delta = KeymapDelta::new(vec![DeltaOperation::Add(Binding {
        gesture: control_f,
        command: id("motolii.edit.delete_targeted_items"),
    })]);
    let registry = builtin_command_registry().unwrap();

    let windows = resolve_keymap(
        &base(),
        &delta,
        &platform(PlatformCommandModifier::Control, vec![]),
        &registry,
    );
    assert!(windows
        .diagnostics()
        .iter()
        .any(|item| matches!(item, KeymapDiagnostic::Conflict { .. })));
    assert_eq!(windows.iter().len(), 1);

    let mac = resolve_keymap(
        &base(),
        &delta,
        &platform(PlatformCommandModifier::Meta, vec![]),
        &registry,
    );
    assert_eq!(mac.iter().len(), 3);
}

#[test]
fn key_toggle_conflicts_with_keyboard_press_and_release() {
    let toggle = Gesture::KeyToggle {
        key: key('t'),
        modifiers: Modifiers::default(),
    };
    let press = keyboard('t', &[], InputPhase::Press);
    let delta = KeymapDelta::new(vec![
        DeltaOperation::Add(Binding {
            gesture: toggle,
            command: id("motolii.view.fit_stage"),
        }),
        DeltaOperation::Add(Binding {
            gesture: press,
            command: id("motolii.edit.delete_targeted_items"),
        }),
    ]);
    let result = resolve_keymap(
        &base(),
        &delta,
        &platform(PlatformCommandModifier::Control, vec![]),
        &builtin_command_registry().unwrap(),
    );

    assert!(result
        .diagnostics()
        .iter()
        .any(|item| matches!(item, KeymapDiagnostic::Conflict { .. })));
    let press = EffectiveTrigger::Keyboard {
        key: key('t'),
        modifiers: Modifiers::default(),
        phase: InputPhase::Press,
    };
    assert!(result.get(&press).is_none());
}

#[test]
fn reserved_and_unknown_bindings_are_diagnostics_not_executable() {
    let trigger = EffectiveTrigger::Keyboard {
        key: key('f'),
        modifiers: modifiers(&[Modifier::Control]),
        phase: InputPhase::Press,
    };
    let unknown = id("motolii.unknown.command");
    let unknown_gesture = keyboard('u', &[], InputPhase::Press);
    let result = resolve_keymap(
        &base(),
        &KeymapDelta::new(vec![DeltaOperation::Add(Binding {
            gesture: unknown_gesture,
            command: unknown.clone(),
        })]),
        &platform(PlatformCommandModifier::Control, vec![trigger.clone()]),
        &builtin_command_registry().unwrap(),
    );

    assert!(result
        .diagnostics()
        .contains(&KeymapDiagnostic::UnavailableOnPlatform {
            trigger: trigger.clone(),
            command: id("motolii.view.fit_stage"),
        }));
    assert!(result
        .diagnostics()
        .contains(&KeymapDiagnostic::UnknownCommandId { id: unknown }));
    assert!(result.get(&trigger).is_none());
    assert!(!result
        .iter()
        .any(|(_, command)| command.as_str() == "motolii.unknown.command"));
}

#[test]
fn invalid_phases_are_not_executable() {
    let invalid = Gesture::Keyboard {
        key: key('q'),
        modifiers: Modifiers::default(),
        phase: InputPhase::Click,
    };
    let result = resolve_keymap(
        &BuiltinKeymap::new(
            1,
            vec![Binding {
                gesture: invalid.clone(),
                command: id("motolii.view.fit_stage"),
            }],
        ),
        &empty_delta(),
        &platform(PlatformCommandModifier::Control, vec![]),
        &builtin_command_registry().unwrap(),
    );
    assert!(result
        .diagnostics()
        .contains(&KeymapDiagnostic::InvalidGesturePhase { gesture: invalid }));
    assert_eq!(result.iter().len(), 0);
}

#[test]
fn keymap_source_has_no_persistence_or_toolkit_contract() {
    let source = include_str!("../src/keymap.rs");
    for token in [
        "egui::",
        "eframe::",
        "winit::",
        "KeyCode",
        "MouseButton",
        "Serialize",
        "Deserialize",
        "serde",
        "std::fs",
        "motolii_doc::",
    ] {
        assert!(!source.contains(token), "keymap must not contain {token}");
    }
}
