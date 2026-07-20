//! U0d-3: 全builtin commandの合成keymap再割当conformance。

use motolii_ui::{
    builtin_command_registry, resolve_keymap, AsciiKey, Binding, BuiltinKeymap, DeltaOperation,
    DomainIntent, EffectiveTrigger, Gesture, ImeGateState, InputPhase, InputRouter, KeyToken,
    KeymapDelta, Modifier, Modifiers, NormalizedInput, PlatformBindingConstraints,
    PlatformCommandModifier, RouterOutput,
};

fn synthetic_gestures() -> Vec<Gesture> {
    let modifier_sets = [
        vec![],
        vec![Modifier::Shift],
        vec![Modifier::Alt],
        vec![Modifier::Control],
        vec![Modifier::Alt, Modifier::Shift],
        vec![Modifier::Control, Modifier::Shift],
        vec![Modifier::Alt, Modifier::Control],
    ];
    let mut gestures = Vec::new();

    for phase in [InputPhase::Press, InputPhase::Release] {
        for modifier_set in &modifier_sets {
            let modifiers = Modifiers::try_new(modifier_set.iter().copied()).unwrap();
            for value in ('a'..='z').chain('0'..='9') {
                gestures.push(Gesture::Keyboard {
                    key: KeyToken::Ascii(AsciiKey::try_new(value).unwrap()),
                    modifiers: modifiers.clone(),
                    phase,
                });
            }
        }
    }
    gestures
}

fn effective_trigger(gesture: &Gesture) -> EffectiveTrigger {
    let Gesture::Keyboard {
        key,
        modifiers,
        phase,
    } = gesture
    else {
        panic!("synthetic conformance gestures must remain keyboard gestures");
    };
    EffectiveTrigger::Keyboard {
        key: *key,
        modifiers: modifiers.clone(),
        phase: *phase,
    }
}

#[test]
fn every_builtin_command_can_be_disabled_and_reassigned_to_the_same_intent() {
    let registry = builtin_command_registry().unwrap();
    let metadata: Vec<_> = registry.iter().cloned().collect();
    let gestures = synthetic_gestures();
    assert!(
        gestures.len() >= metadata.len() * 2,
        "synthetic gesture fixture exhausted for {} builtin commands",
        metadata.len()
    );

    let mut base_bindings = Vec::with_capacity(metadata.len());
    let mut operations = Vec::with_capacity(metadata.len() * 2);
    let mut pairs = Vec::with_capacity(metadata.len());
    for (index, item) in metadata.iter().enumerate() {
        let base = gestures[index * 2].clone();
        let alternate = gestures[index * 2 + 1].clone();
        base_bindings.push(Binding {
            gesture: base.clone(),
            command: item.id.clone(),
        });
        operations.push(DeltaOperation::Disable {
            gesture: base.clone(),
        });
        operations.push(DeltaOperation::Add(Binding {
            gesture: alternate.clone(),
            command: item.id.clone(),
        }));
        pairs.push((item, base, alternate));
    }

    let base = BuiltinKeymap::new(1, base_bindings);
    let platform = PlatformBindingConstraints::new(PlatformCommandModifier::Control, Vec::new());
    let original = resolve_keymap(&base, &KeymapDelta::default(), &platform, &registry);
    assert!(original.diagnostics().is_empty());
    assert_eq!(original.iter().len(), metadata.len());

    let reassigned = resolve_keymap(&base, &KeymapDelta::new(operations), &platform, &registry);
    assert!(reassigned.diagnostics().is_empty());
    assert_eq!(reassigned.iter().len(), metadata.len());

    for (item, base_gesture, alternate_gesture) in pairs {
        let old_trigger = effective_trigger(&base_gesture);
        assert_eq!(original.get(&old_trigger), Some(&item.id));
        assert_eq!(reassigned.get(&old_trigger), None);

        let alternate_trigger = effective_trigger(&alternate_gesture);
        let resolved_id = reassigned
            .get(&alternate_trigger)
            .unwrap_or_else(|| panic!("alternate trigger missing for {}", item.id))
            .clone();
        assert_eq!(resolved_id, item.id);

        let mut router = InputRouter::new(registry.clone());
        assert_eq!(router.ime_gate(), ImeGateState::Inactive);
        assert_eq!(
            router
                .route(NormalizedInput::Phase(InputPhase::DragStart))
                .unwrap(),
            RouterOutput::Phase(InputPhase::DragStart)
        );
        let output = router
            .route(NormalizedInput::Command {
                phase: InputPhase::Press,
                id: resolved_id.clone(),
            })
            .unwrap();
        let expected_phase = if item.intent == DomainIntent::CancelInFlightGesture {
            InputPhase::Cancel
        } else {
            InputPhase::Press
        };
        assert_eq!(
            output,
            RouterOutput::Intent {
                phase: expected_phase,
                id: resolved_id,
                intent: item.intent,
            }
        );
    }
}
