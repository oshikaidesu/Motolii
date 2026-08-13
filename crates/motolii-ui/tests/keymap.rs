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
fn ableton_default_connects_existing_play_undo_delete_without_new_commands() {
    let registry = builtin_command_registry().unwrap();
    let none = modifiers(&[]);
    let meta = modifiers(&[Modifier::Meta]);
    let meta_shift = modifiers(&[Modifier::Meta, Modifier::Shift]);
    let platform = PlatformCommandModifier::Meta;

    let space = EffectiveTrigger::Keyboard {
        key: KeyToken::Space,
        modifiers: none.clone(),
        phase: InputPhase::Press,
    };
    let delete = EffectiveTrigger::Keyboard {
        key: KeyToken::Delete,
        modifiers: none.clone(),
        phase: InputPhase::Press,
    };
    let backspace = EffectiveTrigger::Keyboard {
        key: KeyToken::Backspace,
        modifiers: none.clone(),
        phase: InputPhase::Press,
    };
    let undo = EffectiveTrigger::Keyboard {
        key: key('z'),
        modifiers: meta,
        phase: InputPhase::Press,
    };
    let redo = EffectiveTrigger::Keyboard {
        key: key('z'),
        modifiers: meta_shift,
        phase: InputPhase::Press,
    };
    let duplicate = EffectiveTrigger::Keyboard {
        key: key('d'),
        modifiers: modifiers(&[Modifier::Meta]),
        phase: InputPhase::Press,
    };
    let split = EffectiveTrigger::Keyboard {
        key: key('k'),
        modifiers: modifiers(&[Modifier::Meta]),
        phase: InputPhase::Press,
    };
    let solo = EffectiveTrigger::Keyboard {
        key: key('s'),
        modifiers: none.clone(),
        phase: InputPhase::Press,
    };
    let mute = EffectiveTrigger::Keyboard {
        key: key('m'),
        modifiers: none.clone(),
        phase: InputPhase::Press,
    };
    let shuttle_forward = EffectiveTrigger::Keyboard {
        key: key('l'),
        modifiers: none.clone(),
        phase: InputPhase::Press,
    };
    let shuttle_reverse = EffectiveTrigger::Keyboard {
        key: key('j'),
        modifiers: none.clone(),
        phase: InputPhase::Press,
    };
    let shuttle_stop = EffectiveTrigger::Keyboard {
        key: key('k'),
        modifiers: none.clone(),
        phase: InputPhase::Press,
    };
    let mark_in = EffectiveTrigger::Keyboard {
        key: key('i'),
        modifiers: none.clone(),
        phase: InputPhase::Press,
    };
    let mark_out = EffectiveTrigger::Keyboard {
        key: key('o'),
        modifiers: none,
        phase: InputPhase::Press,
    };

    assert_eq!(motolii_ui::PRODUCT_KEYMAP_PROFILE_ID, "ableton");
    assert_eq!(
        motolii_ui::resolve_product_action(&space, &registry, &empty_delta(), platform),
        Some(motolii_ui::ProductAction::HostKind(
            motolii_ui::PRODUCT_HOST_KIND_TOGGLE_PLAYBACK.into()
        ))
    );
    assert_eq!(
        motolii_ui::product_action_host_kind(
            &motolii_ui::resolve_product_action(&space, &registry, &empty_delta(), platform)
                .unwrap()
        ),
        Some("toggle_playback")
    );
    assert_eq!(
        motolii_ui::product_action_host_kind(
            &motolii_ui::resolve_product_action(&delete, &registry, &empty_delta(), platform)
                .unwrap()
        ),
        Some("delete_layer")
    );
    assert_eq!(
        motolii_ui::product_action_host_kind(
            &motolii_ui::resolve_product_action(&backspace, &registry, &empty_delta(), platform)
                .unwrap()
        ),
        Some("delete_layer")
    );
    assert_eq!(
        motolii_ui::product_action_host_kind(
            &motolii_ui::resolve_product_action(&undo, &registry, &empty_delta(), platform)
                .unwrap()
        ),
        Some("undo")
    );
    assert_eq!(
        motolii_ui::product_action_host_kind(
            &motolii_ui::resolve_product_action(&redo, &registry, &empty_delta(), platform)
                .unwrap()
        ),
        Some("redo")
    );
    assert_eq!(
        motolii_ui::product_action_host_kind(
            &motolii_ui::resolve_product_action(&duplicate, &registry, &empty_delta(), platform)
                .unwrap()
        ),
        Some(motolii_ui::PRODUCT_HOST_KIND_DUPLICATE)
    );
    assert_eq!(
        motolii_ui::PRODUCT_UNWIRED_DUPLICATE,
        motolii_ui::PRODUCT_HOST_KIND_DUPLICATE
    );
    assert_eq!(
        motolii_ui::resolve_product_action(&split, &registry, &empty_delta(), platform),
        Some(motolii_ui::ProductAction::HostKind(
            motolii_ui::PRODUCT_UNWIRED_SPLIT.into()
        ))
    );
    assert_eq!(
        motolii_ui::product_action_host_kind(
            &motolii_ui::resolve_product_action(&split, &registry, &empty_delta(), platform)
                .unwrap()
        ),
        Some(motolii_ui::PRODUCT_UNWIRED_SPLIT)
    );
    assert_eq!(motolii_ui::PRODUCT_UNWIRED_SPLIT, "split");
    assert_eq!(
        motolii_ui::PRODUCT_UNWIRED_SOLO,
        motolii_ui::PRODUCT_HOST_KIND_SOLO
    );
    assert_eq!(
        motolii_ui::PRODUCT_UNWIRED_MUTE,
        motolii_ui::PRODUCT_HOST_KIND_MUTE
    );
    assert_eq!(
        motolii_ui::product_action_host_kind(
            &motolii_ui::resolve_product_action(&solo, &registry, &empty_delta(), platform)
                .unwrap()
        ),
        Some(motolii_ui::PRODUCT_HOST_KIND_SOLO)
    );
    assert_eq!(
        motolii_ui::product_action_host_kind(
            &motolii_ui::resolve_product_action(&mute, &registry, &empty_delta(), platform)
                .unwrap()
        ),
        Some(motolii_ui::PRODUCT_HOST_KIND_MUTE)
    );
    assert_eq!(
        motolii_ui::product_action_host_kind(
            &motolii_ui::resolve_product_action(
                &shuttle_forward,
                &registry,
                &empty_delta(),
                platform
            )
            .unwrap()
        ),
        Some(motolii_ui::PRODUCT_HOST_KIND_SHUTTLE_FORWARD)
    );
    assert_eq!(
        motolii_ui::product_action_host_kind(
            &motolii_ui::resolve_product_action(
                &shuttle_reverse,
                &registry,
                &empty_delta(),
                platform
            )
            .unwrap()
        ),
        Some(motolii_ui::PRODUCT_HOST_KIND_SHUTTLE_REVERSE)
    );
    assert_eq!(
        motolii_ui::product_action_host_kind(
            &motolii_ui::resolve_product_action(&shuttle_stop, &registry, &empty_delta(), platform)
                .unwrap()
        ),
        Some(motolii_ui::PRODUCT_HOST_KIND_SHUTTLE_STOP)
    );
    assert_eq!(
        motolii_ui::product_action_host_kind(
            &motolii_ui::resolve_product_action(&mark_in, &registry, &empty_delta(), platform)
                .unwrap()
        ),
        Some(motolii_ui::PRODUCT_HOST_KIND_TRIM_CLIP_IN)
    );
    assert_eq!(
        motolii_ui::product_action_host_kind(
            &motolii_ui::resolve_product_action(&mark_out, &registry, &empty_delta(), platform)
                .unwrap()
        ),
        Some(motolii_ui::PRODUCT_HOST_KIND_TRIM_CLIP_OUT)
    );
}

#[test]
fn user_delta_can_disable_space_play_without_new_command() {
    let registry = builtin_command_registry().unwrap();
    let space_gesture = Gesture::Keyboard {
        key: KeyToken::Space,
        modifiers: Modifiers::default(),
        phase: InputPhase::Press,
    };
    let space = EffectiveTrigger::Keyboard {
        key: KeyToken::Space,
        modifiers: Modifiers::default(),
        phase: InputPhase::Press,
    };
    let delta = KeymapDelta::new(vec![DeltaOperation::Disable {
        gesture: space_gesture,
    }]);
    assert_eq!(
        motolii_ui::resolve_product_action(
            &space,
            &registry,
            &delta,
            PlatformCommandModifier::Meta
        ),
        None
    );
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
