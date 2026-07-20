//! U0d-2: version付きkeymap JSON、原本保全、拒否境界の審判。

use motolii_doc::Document;
use motolii_ui::{
    builtin_command_registry, decode_keymap_json, encode_keymap_json, resolve_keymap, AsciiKey,
    Binding, BuiltinKeymap, CommandId, DeltaOperation, Gesture, InputPhase, KeyToken,
    KeymapApplyError, KeymapCodecDiagnostic, KeymapCodecError, KeymapCodecLimits, KeymapDelta,
    KeymapDiagnostic, LimitKind, Modifier, Modifiers, OpaqueOperationReason,
    PlatformBindingConstraints, PlatformCommandModifier, PointerButton, KEYMAP_CODEC_VERSION,
};

const DOCUMENTED_JSON: &str = r#"{
  "version": 1,
  "source": {
    "builtin_version": 1
  },
  "operations": [
    {
      "op": "add",
      "gesture": {
        "kind": "keyboard",
        "key": "d",
        "modifiers": [
          "primary"
        ],
        "phase": "press"
      },
      "command": "motolii.edit.delete_targeted_items"
    },
    {
      "op": "replace",
      "gesture": {
        "kind": "modifier_pointer",
        "button": "primary",
        "modifiers": [
          "alt"
        ],
        "phase": "drag_start"
      },
      "command": "motolii.view.fit_stage"
    },
    {
      "op": "disable",
      "gesture": {
        "kind": "key_toggle",
        "key": "t",
        "modifiers": []
      }
    }
  ]
}"#;

fn limits() -> KeymapCodecLimits {
    KeymapCodecLimits::new(64 * 1024, 16, 128, 1024)
}

fn base(version: u32) -> BuiltinKeymap {
    BuiltinKeymap::new(version, Vec::new())
}

fn id(value: &str) -> CommandId {
    CommandId::try_new(value).unwrap()
}

fn key(value: char) -> KeyToken {
    KeyToken::Ascii(AsciiKey::try_new(value).unwrap())
}

fn modifiers(values: &[Modifier]) -> Modifiers {
    Modifiers::try_new(values.iter().copied()).unwrap()
}

#[test]
fn documented_v1_decodes_all_operations_and_writes_canonical_json() {
    assert_eq!(KEYMAP_CODEC_VERSION, 1);
    let loaded = decode_keymap_json(DOCUMENTED_JSON.as_bytes(), limits()).unwrap();
    assert_eq!(loaded.source_builtin_version(), 1);
    assert!(loaded.diagnostics().is_empty());

    let delta = loaded.to_resolver_delta(&base(1)).unwrap();
    let expected = KeymapDelta::new(vec![
        DeltaOperation::Add(Binding {
            gesture: Gesture::Keyboard {
                key: key('d'),
                modifiers: modifiers(&[Modifier::Primary]),
                phase: InputPhase::Press,
            },
            command: id("motolii.edit.delete_targeted_items"),
        }),
        DeltaOperation::Replace(Binding {
            gesture: Gesture::ModifierPointer {
                button: PointerButton::Primary,
                modifiers: modifiers(&[Modifier::Alt]),
                phase: InputPhase::DragStart,
            },
            command: id("motolii.view.fit_stage"),
        }),
        DeltaOperation::Disable {
            gesture: Gesture::KeyToggle {
                key: key('t'),
                modifiers: Modifiers::default(),
            },
        },
    ]);
    assert_eq!(delta, expected);

    let encoded = encode_keymap_json(1, &delta).unwrap();
    assert_eq!(encoded, format!("{DOCUMENTED_JSON}\n").as_bytes());

    let decoded_again = decode_keymap_json(&encoded, limits()).unwrap();
    assert_eq!(decoded_again.to_resolver_delta(&base(1)).unwrap(), delta);
}

#[test]
fn new_write_normalizes_modifiers_and_operation_order() {
    let json = br#"{
      "version": 1,
      "source": {"builtin_version": 7},
      "operations": [
        {
          "op": "add",
          "gesture": {
            "kind": "keyboard",
            "key": "z",
            "modifiers": ["shift", "alt", "shift"],
            "phase": "release"
          },
          "command": "motolii.view.fit_stage"
        },
        {
          "op": "add",
          "gesture": {
            "kind": "keyboard",
            "key": "a",
            "modifiers": [],
            "phase": "press"
          },
          "command": "motolii.edit.delete_targeted_items"
        }
      ]
    }"#;
    let loaded = decode_keymap_json(json, limits()).unwrap();
    let delta = loaded.to_resolver_delta(&base(7)).unwrap();
    let encoded = String::from_utf8(encode_keymap_json(7, &delta).unwrap()).unwrap();

    assert!(encoded.find("\"key\": \"a\"").unwrap() < encoded.find("\"key\": \"z\"").unwrap());
    assert!(
        encoded.contains("\"modifiers\": [\n          \"alt\",\n          \"shift\"\n        ]")
    );
}

#[test]
fn current_migration_is_idempotent_and_preserving_write_is_byte_exact() {
    let original = b"{ \"version\":1, \"source\":{\"builtin_version\":3}, \"operations\":[] }\n";
    let loaded = decode_keymap_json(original, limits()).unwrap();
    let once = loaded.migrate_to_current();
    let twice = once.migrate_to_current();

    assert_eq!(once, twice);
    assert_eq!(loaded.write_preserving(), original);
    assert_eq!(once.write_preserving(), original);
}

#[test]
fn unknown_envelope_fields_are_preserved_and_block_the_whole_delta() {
    for json in [
        r#"{
          "version":1,
          "future_policy":true,
          "source":{"builtin_version":1},
          "operations":[]
        }"#,
        r#"{
          "version":1,
          "source":{"builtin_version":1,"flavor":"future"},
          "operations":[]
        }"#,
    ] {
        let loaded = decode_keymap_json(json.as_bytes(), limits()).unwrap();
        assert_eq!(loaded.write_preserving(), json.as_bytes());
        assert!(loaded.diagnostics().iter().any(|diagnostic| matches!(
            diagnostic,
            KeymapCodecDiagnostic::UnknownEnvelopeFieldPreserved { .. }
        )));
        assert_eq!(
            loaded.to_resolver_delta(&base(1)),
            Err(KeymapApplyError::UnknownEnvelopeFields)
        );
    }
}

#[test]
fn opaque_operations_are_preserved_but_not_executable() {
    let cases: &[(&str, OpaqueOperationReason)] = &[
        (
            r#"{"op":"future","payload":1}"#,
            OpaqueOperationReason::UnknownOperation {
                value: "future".into(),
            },
        ),
        (
            r#"{"op":"add","gesture":{"kind":"keyboard","key":"a","modifiers":[],"phase":"press","future":true},"command":"motolii.view.fit_stage"}"#,
            OpaqueOperationReason::UnknownField {
                field: "future".into(),
            },
        ),
        (
            r#"{"op":"disable","gesture":{"kind":"key_toggle","key":"a","modifiers":[]},"command":"motolii.view.fit_stage"}"#,
            OpaqueOperationReason::ForbiddenField {
                field: "command".into(),
            },
        ),
        (
            r#"{"op":"add","gesture":{"kind":"keyboard","key":"a","modifiers":[],"phase":"press"},"command":"NOT VALID"}"#,
            OpaqueOperationReason::InvalidCommandId {
                value: "NOT VALID".into(),
            },
        ),
        (
            r#"{"op":"add","gesture":{"kind":"keyboard","key":"a","modifiers":["primary","control"],"phase":"press"},"command":"motolii.view.fit_stage"}"#,
            OpaqueOperationReason::InvalidModifierCombination,
        ),
        (
            r#"{"op":"add","gesture":{"kind":"keyboard","key":"a","modifiers":[],"phase":"click"},"command":"motolii.view.fit_stage"}"#,
            OpaqueOperationReason::UnknownPhase {
                value: "click".into(),
            },
        ),
        (
            r#"{"op":"add","gesture":{"kind":"future_kind","key":"a","modifiers":[],"phase":"press"},"command":"motolii.view.fit_stage"}"#,
            OpaqueOperationReason::UnknownGestureKind {
                value: "future_kind".into(),
            },
        ),
        (
            r#"{"op":"add","gesture":{"kind":"keyboard","key":"a","modifiers":[],"phase":"press"},"command":""}"#,
            OpaqueOperationReason::EmptyCommandId,
        ),
    ];

    for (operation, reason) in cases {
        let json = format!(
            r#"{{"version":1,"source":{{"builtin_version":1}},"operations":[{operation}]}}"#
        );
        let loaded = decode_keymap_json(json.as_bytes(), limits()).unwrap();
        assert_eq!(loaded.write_preserving(), json.as_bytes());
        assert_eq!(
            loaded.diagnostics(),
            &[KeymapCodecDiagnostic::OpaqueOperationPreserved {
                index: 0,
                reason: reason.clone()
            }]
        );
        assert!(loaded
            .to_resolver_delta(&base(1))
            .unwrap()
            .operations()
            .is_empty());
    }
}

#[test]
fn syntactically_valid_unknown_command_reaches_resolver_diagnostic_and_survives_write() {
    let json = r#"{
      "version":1,
      "source":{"builtin_version":1},
      "operations":[{
        "op":"add",
        "gesture":{"kind":"keyboard","key":"u","modifiers":[],"phase":"press"},
        "command":"motolii.unknown.command"
      }]
    }"#;
    let loaded = decode_keymap_json(json.as_bytes(), limits()).unwrap();
    assert!(loaded.diagnostics().is_empty());
    let delta = loaded.to_resolver_delta(&base(1)).unwrap();
    let result = resolve_keymap(
        &base(1),
        &delta,
        &PlatformBindingConstraints::new(PlatformCommandModifier::Control, []),
        &builtin_command_registry().unwrap(),
    );
    assert!(result
        .diagnostics()
        .contains(&KeymapDiagnostic::UnknownCommandId {
            id: id("motolii.unknown.command")
        }));
    assert_eq!(loaded.write_preserving(), json.as_bytes());
}

#[test]
fn source_mismatch_is_an_apply_error_without_losing_original() {
    let loaded = decode_keymap_json(DOCUMENTED_JSON.as_bytes(), limits()).unwrap();
    assert_eq!(
        loaded.to_resolver_delta(&base(2)),
        Err(KeymapApplyError::SourceVersionMismatch {
            source_version: 1,
            base_version: 2
        })
    );
    assert_eq!(loaded.write_preserving(), DOCUMENTED_JSON.as_bytes());
}

#[test]
fn rejects_top_level_and_version_errors_without_downgrade() {
    let cases: &[(&[u8], u8)] = &[
        (br#"[]"#, 0),
        (br#"{"source":{"builtin_version":1},"operations":[]}"#, 1),
        (
            br#"{"version":0,"source":{"builtin_version":1},"operations":[]}"#,
            2,
        ),
        (
            br#"{"version":2,"source":{"builtin_version":1},"operations":[]}"#,
            3,
        ),
    ];

    for (json, expected) in cases {
        let error = decode_keymap_json(json, limits()).unwrap_err();
        let matches_expected = match expected {
            0 => matches!(error, KeymapCodecError::TopLevelNotObject),
            1 => matches!(
                error,
                KeymapCodecError::MissingTopLevelField { field: "version" }
            ),
            2 => matches!(
                error,
                KeymapCodecError::UnsupportedOlderVersion { version: 0 }
            ),
            _ => matches!(
                error,
                KeymapCodecError::UnsupportedNewerVersion { version: 2 }
            ),
        };
        assert!(matches_expected, "unexpected error: {error:?}");
    }
}

#[test]
fn rejects_duplicate_keys_at_every_object_layer() {
    for json in [
        r#"{"version":1,"version":1,"source":{"builtin_version":1},"operations":[]}"#,
        r#"{"version":1,"source":{"builtin_version":1},"operations":[{"op":"disable","op":"disable","gesture":{"kind":"key_toggle","key":"a","modifiers":[]}}]}"#,
        r#"{"version":1,"source":{"builtin_version":1},"operations":[{"op":"disable","gesture":{"kind":"key_toggle","key":"a","key":"b","modifiers":[]}}]}"#,
    ] {
        assert!(matches!(
            decode_keymap_json(json.as_bytes(), limits()),
            Err(KeymapCodecError::DuplicateObjectKey { .. })
        ));
    }
}

#[test]
fn rejects_each_injected_resource_limit() {
    let bytes = DOCUMENTED_JSON.as_bytes();
    assert!(matches!(
        decode_keymap_json(
            bytes,
            KeymapCodecLimits::new(bytes.len() - 1, 16, 128, 1024)
        ),
        Err(KeymapCodecError::LimitExceeded {
            kind: LimitKind::Bytes,
            ..
        })
    ));
    assert!(matches!(
        decode_keymap_json(bytes, KeymapCodecLimits::new(64 * 1024, 2, 128, 1024)),
        Err(KeymapCodecError::LimitExceeded {
            kind: LimitKind::Depth,
            ..
        })
    ));
    assert!(matches!(
        decode_keymap_json(bytes, KeymapCodecLimits::new(64 * 1024, 16, 2, 1024)),
        Err(KeymapCodecError::LimitExceeded {
            kind: LimitKind::Operations,
            observed: 3,
            limit: 2
        })
    ));
    assert!(matches!(
        decode_keymap_json(bytes, KeymapCodecLimits::new(64 * 1024, 16, 128, 8)),
        Err(KeymapCodecError::LimitExceeded {
            kind: LimitKind::StringBytes,
            ..
        })
    ));

    let deep_overflow = br#"{
      "version":1,
      "source":{"builtin_version":1},
      "operations":[[[[[[[]]]]]]]
    }"#;
    assert!(matches!(
        decode_keymap_json(deep_overflow, KeymapCodecLimits::new(64 * 1024, 4, 0, 1024)),
        Err(KeymapCodecError::LimitExceeded {
            kind: LimitKind::Depth,
            ..
        })
    ));

    let long_discarded = br#"{
      "version":1,
      "source":{"builtin_version":1},
      "operations":["a string beyond the injected limit"]
    }"#;
    assert!(matches!(
        decode_keymap_json(long_discarded, KeymapCodecLimits::new(64 * 1024, 16, 0, 20)),
        Err(KeymapCodecError::LimitExceeded {
            kind: LimitKind::StringBytes,
            ..
        })
    ));

    let long_duplicate_value = br#"{"version":1,"version":"a string beyond the injected limit"}"#;
    assert!(matches!(
        decode_keymap_json(
            long_duplicate_value,
            KeymapCodecLimits::new(64 * 1024, 16, 128, 8)
        ),
        Err(KeymapCodecError::LimitExceeded {
            kind: LimitKind::StringBytes,
            ..
        })
    ));
}

#[test]
fn new_write_rejects_runtime_only_phases() {
    let delta = KeymapDelta::new(vec![DeltaOperation::Add(Binding {
        gesture: Gesture::Keyboard {
            key: key('a'),
            modifiers: Modifiers::default(),
            phase: InputPhase::DragUpdate,
        },
        command: id("motolii.view.fit_stage"),
    })]);
    assert!(matches!(
        encode_keymap_json(1, &delta),
        Err(KeymapCodecError::InvalidWireOperation { index: 0 })
    ));
}

#[test]
fn codec_activity_does_not_change_document_or_add_toolkit_storage_contracts() {
    let document = Document::new_current();
    let before = serde_json::to_vec(&document).unwrap();
    assert!(!serde_json::to_value(&document)
        .unwrap()
        .as_object()
        .unwrap()
        .contains_key("keymap"));
    let loaded = decode_keymap_json(DOCUMENTED_JSON.as_bytes(), limits()).unwrap();
    let delta = loaded.to_resolver_delta(&base(1)).unwrap();
    let _encoded = encode_keymap_json(1, &delta).unwrap();
    assert_eq!(serde_json::to_vec(&document).unwrap(), before);

    let source = include_str!("../src/keymap_codec.rs");
    for forbidden in [
        "egui::",
        "eframe::",
        "winit::",
        "std::fs",
        "std::path",
        "motolii_doc::",
        "serde_json::from_str",
        "serde_json::from_slice",
    ] {
        assert!(
            !source.contains(forbidden),
            "codec must not contain {forbidden}"
        );
    }
    let runtime = include_str!("../src/keymap.rs");
    assert!(!runtime.contains("Serialize"));
    assert!(!runtime.contains("Deserialize"));
}

#[test]
fn pointer_and_toggle_wire_forms_roundtrip_without_toolkit_types() {
    let delta = KeymapDelta::new(vec![
        DeltaOperation::Add(Binding {
            gesture: Gesture::ModifierPointer {
                button: PointerButton::Auxiliary2,
                modifiers: modifiers(&[Modifier::Meta]),
                phase: InputPhase::DragEnd,
            },
            command: id("motolii.view.fit_stage"),
        }),
        DeltaOperation::Disable {
            gesture: Gesture::KeyToggle {
                key: KeyToken::Space,
                modifiers: Modifiers::default(),
            },
        },
    ]);
    let encoded = encode_keymap_json(4, &delta).unwrap();
    let decoded = decode_keymap_json(&encoded, limits()).unwrap();
    assert_eq!(decoded.to_resolver_delta(&base(4)).unwrap(), delta);
}
