//! U0d-2: keymap user deltaのversion付きJSON codec。

use std::{cell::RefCell, collections::HashSet, fmt, rc::Rc};

use serde::{
    de::{DeserializeSeed, MapAccess, SeqAccess, Visitor},
    ser::{SerializeMap, SerializeSeq},
    Serialize,
};
use serde_json::{Map, Number, Value};

use crate::{
    AsciiKey, Binding, BuiltinKeymap, CommandId, CommandIdError, DeltaOperation, Gesture,
    InputPhase, KeyToken, KeymapDelta, Modifier, Modifiers, PointerButton,
};

pub const KEYMAP_CODEC_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeymapCodecLimits {
    pub max_bytes: usize,
    pub max_depth: usize,
    pub max_operations: usize,
    pub max_string_bytes: usize,
}

impl KeymapCodecLimits {
    pub const fn new(
        max_bytes: usize,
        max_depth: usize,
        max_operations: usize,
        max_string_bytes: usize,
    ) -> Self {
        Self {
            max_bytes,
            max_depth,
            max_operations,
            max_string_bytes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitKind {
    Bytes,
    Depth,
    Operations,
    StringBytes,
}

#[derive(Debug, thiserror::Error)]
pub enum KeymapCodecError {
    #[error("keymap JSON limit exceeded for {kind:?}: observed {observed}, limit {limit}")]
    LimitExceeded {
        kind: LimitKind,
        observed: usize,
        limit: usize,
    },
    #[error("duplicate JSON object key: {key}")]
    DuplicateObjectKey { key: Box<str> },
    #[error("invalid keymap JSON")]
    InvalidJson {
        #[source]
        source: serde_json::Error,
    },
    #[error("keymap JSON top level must be an object")]
    TopLevelNotObject,
    #[error("missing required top-level field: {field}")]
    MissingTopLevelField { field: &'static str },
    #[error("invalid top-level field: {field}")]
    InvalidTopLevelField { field: &'static str },
    #[error("unsupported older keymap codec version: {version}")]
    UnsupportedOlderVersion { version: u64 },
    #[error("unsupported newer keymap codec version: {version}")]
    UnsupportedNewerVersion { version: u64 },
    #[error("source.builtin_version must be a positive u32")]
    InvalidBuiltinVersion,
    #[error("failed to serialize keymap JSON")]
    Serialize {
        #[source]
        source: serde_json::Error,
    },
    #[error("keymap operation {index} cannot be represented by the v1 wire schema")]
    InvalidWireOperation { index: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpaqueOperationReason {
    NotObject,
    MissingField { field: Box<str> },
    InvalidField { field: Box<str> },
    ForbiddenField { field: Box<str> },
    UnknownField { field: Box<str> },
    UnknownOperation { value: Box<str> },
    UnknownGestureKind { value: Box<str> },
    EmptyCommandId,
    InvalidCommandId { value: Box<str> },
    UnknownKey { value: Box<str> },
    UnknownModifier { value: Box<str> },
    InvalidModifierCombination,
    UnknownPointerButton { value: Box<str> },
    UnknownPhase { value: Box<str> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeymapCodecDiagnostic {
    UnknownEnvelopeFieldPreserved {
        path: Box<str>,
    },
    OpaqueOperationPreserved {
        index: usize,
        reason: OpaqueOperationReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum KeymapApplyError {
    #[error("keymap contains unknown envelope fields and is preservation-only")]
    UnknownEnvelopeFields,
    #[error(
        "keymap source builtin version {source_version} does not match base version {base_version}"
    )]
    SourceVersionMismatch {
        source_version: u32,
        base_version: u32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadedKeymap {
    original: Vec<u8>,
    preserved: Value,
    source_builtin_version: u32,
    understood_operations: Vec<DeltaOperation>,
    diagnostics: Vec<KeymapCodecDiagnostic>,
    unknown_envelope: bool,
}

impl LoadedKeymap {
    pub fn source_builtin_version(&self) -> u32 {
        self.source_builtin_version
    }

    pub fn diagnostics(&self) -> &[KeymapCodecDiagnostic] {
        &self.diagnostics
    }

    pub fn write_preserving(&self) -> &[u8] {
        &self.original
    }

    pub fn migrate_to_current(&self) -> Self {
        let _current_tree = &self.preserved;
        self.clone()
    }

    pub fn to_resolver_delta(&self, base: &BuiltinKeymap) -> Result<KeymapDelta, KeymapApplyError> {
        if self.unknown_envelope {
            return Err(KeymapApplyError::UnknownEnvelopeFields);
        }
        if self.source_builtin_version != base.version {
            return Err(KeymapApplyError::SourceVersionMismatch {
                source_version: self.source_builtin_version,
                base_version: base.version,
            });
        }
        Ok(KeymapDelta::new(self.understood_operations.clone()))
    }
}

pub fn decode_keymap_json(
    input: &[u8],
    limits: KeymapCodecLimits,
) -> Result<LoadedKeymap, KeymapCodecError> {
    if input.len() > limits.max_bytes {
        return Err(KeymapCodecError::LimitExceeded {
            kind: LimitKind::Bytes,
            observed: input.len(),
            limit: limits.max_bytes,
        });
    }

    let preserved = parse_checked_json(input, limits)?;
    let root = preserved
        .as_object()
        .ok_or(KeymapCodecError::TopLevelNotObject)?;
    let version = required_u64(root, "version")?;
    match version.cmp(&u64::from(KEYMAP_CODEC_VERSION)) {
        std::cmp::Ordering::Less => {
            return Err(KeymapCodecError::UnsupportedOlderVersion { version });
        }
        std::cmp::Ordering::Greater => {
            return Err(KeymapCodecError::UnsupportedNewerVersion { version });
        }
        std::cmp::Ordering::Equal => {}
    }

    let source = root
        .get("source")
        .ok_or(KeymapCodecError::MissingTopLevelField { field: "source" })?
        .as_object()
        .ok_or(KeymapCodecError::InvalidTopLevelField { field: "source" })?;
    let source_version = source
        .get("builtin_version")
        .ok_or(KeymapCodecError::MissingTopLevelField {
            field: "source.builtin_version",
        })?
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value > 0)
        .ok_or(KeymapCodecError::InvalidBuiltinVersion)?;

    let operations = root
        .get("operations")
        .ok_or(KeymapCodecError::MissingTopLevelField {
            field: "operations",
        })?
        .as_array()
        .ok_or(KeymapCodecError::InvalidTopLevelField {
            field: "operations",
        })?;
    let mut diagnostics = Vec::new();
    let mut unknown_envelope = false;
    for key in root.keys() {
        if !matches!(key.as_str(), "version" | "source" | "operations") {
            unknown_envelope = true;
            diagnostics.push(KeymapCodecDiagnostic::UnknownEnvelopeFieldPreserved {
                path: key.clone().into_boxed_str(),
            });
        }
    }
    for key in source.keys() {
        if key != "builtin_version" {
            unknown_envelope = true;
            diagnostics.push(KeymapCodecDiagnostic::UnknownEnvelopeFieldPreserved {
                path: format!("source.{key}").into_boxed_str(),
            });
        }
    }

    let mut understood_operations = Vec::new();
    for (index, value) in operations.iter().enumerate() {
        match parse_operation(value) {
            Ok(operation) => understood_operations.push(operation),
            Err(reason) => {
                diagnostics.push(KeymapCodecDiagnostic::OpaqueOperationPreserved { index, reason })
            }
        }
    }

    Ok(LoadedKeymap {
        original: input.to_vec(),
        preserved,
        source_builtin_version: source_version,
        understood_operations,
        diagnostics,
        unknown_envelope,
    })
}

pub fn encode_keymap_json(
    source_builtin_version: u32,
    delta: &KeymapDelta,
) -> Result<Vec<u8>, KeymapCodecError> {
    if source_builtin_version == 0 {
        return Err(KeymapCodecError::InvalidBuiltinVersion);
    }
    let mut operations: Vec<_> = delta.operations().iter().collect();
    for (index, operation) in operations.iter().enumerate() {
        if !wire_operation_is_valid(operation) {
            return Err(KeymapCodecError::InvalidWireOperation { index });
        }
    }
    operations.sort_by_key(|operation| operation_sort_key(operation));

    let document = WireDocument {
        source_builtin_version,
        operations,
    };
    let mut bytes = serde_json::to_vec_pretty(&document)
        .map_err(|source| KeymapCodecError::Serialize { source })?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn required_u64(root: &Map<String, Value>, field: &'static str) -> Result<u64, KeymapCodecError> {
    root.get(field)
        .ok_or(KeymapCodecError::MissingTopLevelField { field })?
        .as_u64()
        .ok_or(KeymapCodecError::InvalidTopLevelField { field })
}

fn parse_operation(value: &Value) -> Result<DeltaOperation, OpaqueOperationReason> {
    let object = value.as_object().ok_or(OpaqueOperationReason::NotObject)?;
    let op = string_field(object, "op")?;
    let allowed: &[&str] = match op {
        "add" | "replace" => &["op", "gesture", "command"],
        "disable" => &["op", "gesture"],
        other => {
            return Err(OpaqueOperationReason::UnknownOperation {
                value: other.into(),
            });
        }
    };
    if op == "disable" && object.contains_key("command") {
        return Err(OpaqueOperationReason::ForbiddenField {
            field: "command".into(),
        });
    }
    reject_unknown_fields(object, allowed)?;
    let gesture = parse_gesture(object.get("gesture").ok_or_else(|| {
        OpaqueOperationReason::MissingField {
            field: "gesture".into(),
        }
    })?)?;

    match op {
        "add" | "replace" => {
            let command_value = string_field(object, "command")?;
            let command = CommandId::try_new(command_value).map_err(|error| match error {
                CommandIdError::Empty => OpaqueOperationReason::EmptyCommandId,
                CommandIdError::InvalidGrammar { .. } => OpaqueOperationReason::InvalidCommandId {
                    value: command_value.into(),
                },
            })?;
            let binding = Binding { gesture, command };
            Ok(if op == "add" {
                DeltaOperation::Add(binding)
            } else {
                DeltaOperation::Replace(binding)
            })
        }
        _ => Ok(DeltaOperation::Disable { gesture }),
    }
}

fn parse_gesture(value: &Value) -> Result<Gesture, OpaqueOperationReason> {
    let object = value
        .as_object()
        .ok_or_else(|| OpaqueOperationReason::InvalidField {
            field: "gesture".into(),
        })?;
    let kind = string_field(object, "kind")?;
    let allowed: &[&str] = match kind {
        "keyboard" => &["kind", "key", "modifiers", "phase"],
        "modifier_pointer" => &["kind", "button", "modifiers", "phase"],
        "key_toggle" => &["kind", "key", "modifiers"],
        other => {
            return Err(OpaqueOperationReason::UnknownGestureKind {
                value: other.into(),
            });
        }
    };
    reject_unknown_fields(object, allowed)?;

    let modifiers = parse_modifiers(object.get("modifiers").ok_or_else(|| {
        OpaqueOperationReason::MissingField {
            field: "modifiers".into(),
        }
    })?)?;
    match kind {
        "keyboard" => Ok(Gesture::Keyboard {
            key: parse_key(string_field(object, "key")?)?,
            modifiers,
            phase: parse_phase(string_field(object, "phase")?, true)?,
        }),
        "modifier_pointer" => Ok(Gesture::ModifierPointer {
            button: parse_button(string_field(object, "button")?)?,
            modifiers,
            phase: parse_phase(string_field(object, "phase")?, false)?,
        }),
        _ => Ok(Gesture::KeyToggle {
            key: parse_key(string_field(object, "key")?)?,
            modifiers,
        }),
    }
}

fn reject_unknown_fields(
    object: &Map<String, Value>,
    allowed: &[&str],
) -> Result<(), OpaqueOperationReason> {
    if let Some(field) = object.keys().find(|key| !allowed.contains(&key.as_str())) {
        return Err(OpaqueOperationReason::UnknownField {
            field: field.clone().into_boxed_str(),
        });
    }
    Ok(())
}

fn string_field<'a>(
    object: &'a Map<String, Value>,
    field: &str,
) -> Result<&'a str, OpaqueOperationReason> {
    object
        .get(field)
        .ok_or_else(|| OpaqueOperationReason::MissingField {
            field: field.into(),
        })?
        .as_str()
        .ok_or_else(|| OpaqueOperationReason::InvalidField {
            field: field.into(),
        })
}

fn parse_key(value: &str) -> Result<KeyToken, OpaqueOperationReason> {
    if value.len() == 1 {
        let value_char = char::from(value.as_bytes()[0]);
        if let Ok(key) = AsciiKey::try_new(value_char) {
            return Ok(KeyToken::Ascii(key));
        }
    }
    let key = match value {
        "space" => KeyToken::Space,
        "enter" => KeyToken::Enter,
        "escape" => KeyToken::Escape,
        "delete" => KeyToken::Delete,
        "backspace" => KeyToken::Backspace,
        "tab" => KeyToken::Tab,
        "arrow_up" => KeyToken::ArrowUp,
        "arrow_down" => KeyToken::ArrowDown,
        "arrow_left" => KeyToken::ArrowLeft,
        "arrow_right" => KeyToken::ArrowRight,
        "home" => KeyToken::Home,
        "end" => KeyToken::End,
        "page_up" => KeyToken::PageUp,
        "page_down" => KeyToken::PageDown,
        _ => {
            return Err(OpaqueOperationReason::UnknownKey {
                value: value.into(),
            });
        }
    };
    Ok(key)
}

fn parse_modifiers(value: &Value) -> Result<Modifiers, OpaqueOperationReason> {
    let values = value
        .as_array()
        .ok_or_else(|| OpaqueOperationReason::InvalidField {
            field: "modifiers".into(),
        })?;
    let mut modifiers = Vec::with_capacity(values.len());
    for value in values {
        let value = value
            .as_str()
            .ok_or_else(|| OpaqueOperationReason::InvalidField {
                field: "modifiers".into(),
            })?;
        modifiers.push(match value {
            "primary" => Modifier::Primary,
            "control" => Modifier::Control,
            "meta" => Modifier::Meta,
            "alt" => Modifier::Alt,
            "shift" => Modifier::Shift,
            _ => {
                return Err(OpaqueOperationReason::UnknownModifier {
                    value: value.into(),
                });
            }
        });
    }
    Modifiers::try_new(modifiers).map_err(|_| OpaqueOperationReason::InvalidModifierCombination)
}

fn parse_button(value: &str) -> Result<PointerButton, OpaqueOperationReason> {
    match value {
        "primary" => Ok(PointerButton::Primary),
        "secondary" => Ok(PointerButton::Secondary),
        "middle" => Ok(PointerButton::Middle),
        "auxiliary_1" => Ok(PointerButton::Auxiliary1),
        "auxiliary_2" => Ok(PointerButton::Auxiliary2),
        _ => Err(OpaqueOperationReason::UnknownPointerButton {
            value: value.into(),
        }),
    }
}

fn parse_phase(value: &str, keyboard: bool) -> Result<InputPhase, OpaqueOperationReason> {
    let phase = match value {
        "press" => InputPhase::Press,
        "release" => InputPhase::Release,
        "click" if !keyboard => InputPhase::Click,
        "drag_start" if !keyboard => InputPhase::DragStart,
        "drag_end" if !keyboard => InputPhase::DragEnd,
        _ => {
            return Err(OpaqueOperationReason::UnknownPhase {
                value: value.into(),
            });
        }
    };
    Ok(phase)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct OperationSortKey {
    kind: u8,
    token: u8,
    modifiers: Vec<u8>,
    phase: u8,
    operation: u8,
    command: Box<str>,
}

fn operation_sort_key(operation: &DeltaOperation) -> OperationSortKey {
    let (gesture, operation_rank, command) = match operation {
        DeltaOperation::Add(binding) => (&binding.gesture, 0, binding.command.as_str()),
        DeltaOperation::Replace(binding) => (&binding.gesture, 1, binding.command.as_str()),
        DeltaOperation::Disable { gesture } => (gesture, 2, ""),
    };
    let (kind, token, modifiers, phase) = gesture_sort_key(gesture);
    OperationSortKey {
        kind,
        token,
        modifiers,
        phase,
        operation: operation_rank,
        command: command.into(),
    }
}

fn wire_operation_is_valid(operation: &DeltaOperation) -> bool {
    let gesture = match operation {
        DeltaOperation::Add(binding) | DeltaOperation::Replace(binding) => &binding.gesture,
        DeltaOperation::Disable { gesture } => gesture,
    };
    match gesture {
        Gesture::Keyboard { phase, .. } => {
            matches!(phase, InputPhase::Press | InputPhase::Release)
        }
        Gesture::ModifierPointer { phase, .. } => matches!(
            phase,
            InputPhase::Press
                | InputPhase::Release
                | InputPhase::Click
                | InputPhase::DragStart
                | InputPhase::DragEnd
        ),
        Gesture::KeyToggle { .. } => true,
    }
}

fn gesture_sort_key(gesture: &Gesture) -> (u8, u8, Vec<u8>, u8) {
    match gesture {
        Gesture::Keyboard {
            key,
            modifiers,
            phase,
        } => (
            0,
            key_rank(*key),
            modifier_ranks(modifiers),
            phase_rank(*phase),
        ),
        Gesture::ModifierPointer {
            button,
            modifiers,
            phase,
        } => (
            1,
            button_rank(*button),
            modifier_ranks(modifiers),
            phase_rank(*phase),
        ),
        Gesture::KeyToggle { key, modifiers } => (2, key_rank(*key), modifier_ranks(modifiers), 0),
    }
}

fn key_rank(key: KeyToken) -> u8 {
    match key {
        KeyToken::Ascii(key) if key.as_char().is_ascii_lowercase() => key.as_char() as u8 - b'a',
        KeyToken::Ascii(key) => 26 + (key.as_char() as u8 - b'0'),
        KeyToken::Space => 36,
        KeyToken::Enter => 37,
        KeyToken::Escape => 38,
        KeyToken::Delete => 39,
        KeyToken::Backspace => 40,
        KeyToken::Tab => 41,
        KeyToken::ArrowUp => 42,
        KeyToken::ArrowDown => 43,
        KeyToken::ArrowLeft => 44,
        KeyToken::ArrowRight => 45,
        KeyToken::Home => 46,
        KeyToken::End => 47,
        KeyToken::PageUp => 48,
        KeyToken::PageDown => 49,
    }
}

fn button_rank(button: PointerButton) -> u8 {
    match button {
        PointerButton::Primary => 0,
        PointerButton::Secondary => 1,
        PointerButton::Middle => 2,
        PointerButton::Auxiliary1 => 3,
        PointerButton::Auxiliary2 => 4,
    }
}

fn modifier_ranks(modifiers: &Modifiers) -> Vec<u8> {
    modifiers
        .iter()
        .map(|modifier| match modifier {
            Modifier::Primary => 0,
            Modifier::Control => 1,
            Modifier::Meta => 2,
            Modifier::Alt => 3,
            Modifier::Shift => 4,
        })
        .collect()
}

fn phase_rank(phase: InputPhase) -> u8 {
    match phase {
        InputPhase::Press => 0,
        InputPhase::Release => 1,
        InputPhase::Click => 2,
        InputPhase::DragStart => 3,
        InputPhase::DragEnd => 4,
        InputPhase::DragUpdate => 5,
        InputPhase::Cancel => 6,
    }
}

struct WireDocument<'a> {
    source_builtin_version: u32,
    operations: Vec<&'a DeltaOperation>,
}

impl Serialize for WireDocument<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("version", &KEYMAP_CODEC_VERSION)?;
        map.serialize_entry(
            "source",
            &WireSource {
                builtin_version: self.source_builtin_version,
            },
        )?;
        map.serialize_entry("operations", &WireOperations(&self.operations))?;
        map.end()
    }
}

struct WireSource {
    builtin_version: u32,
}

impl Serialize for WireSource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("builtin_version", &self.builtin_version)?;
        map.end()
    }
}

struct WireOperations<'a>(&'a [&'a DeltaOperation]);

impl Serialize for WireOperations<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for operation in self.0 {
            sequence.serialize_element(&WireOperation(operation))?;
        }
        sequence.end()
    }
}

struct WireOperation<'a>(&'a DeltaOperation);

impl Serialize for WireOperation<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.0 {
            DeltaOperation::Add(binding) | DeltaOperation::Replace(binding) => {
                let mut map = serializer.serialize_map(Some(3))?;
                let op = if matches!(self.0, DeltaOperation::Add(_)) {
                    "add"
                } else {
                    "replace"
                };
                map.serialize_entry("op", op)?;
                map.serialize_entry("gesture", &WireGesture(&binding.gesture))?;
                map.serialize_entry("command", binding.command.as_str())?;
                map.end()
            }
            DeltaOperation::Disable { gesture } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("op", "disable")?;
                map.serialize_entry("gesture", &WireGesture(gesture))?;
                map.end()
            }
        }
    }
}

struct WireGesture<'a>(&'a Gesture);

impl Serialize for WireGesture<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.0 {
            Gesture::Keyboard {
                key,
                modifiers,
                phase,
            } => {
                let mut map = serializer.serialize_map(Some(4))?;
                map.serialize_entry("kind", "keyboard")?;
                map.serialize_entry("key", &WireKey(*key))?;
                map.serialize_entry("modifiers", &WireModifiers(modifiers))?;
                let phase = phase_name(*phase)
                    .ok_or_else(|| serde::ser::Error::custom("invalid keyboard phase"))?;
                map.serialize_entry("phase", phase)?;
                map.end()
            }
            Gesture::ModifierPointer {
                button,
                modifiers,
                phase,
            } => {
                let mut map = serializer.serialize_map(Some(4))?;
                map.serialize_entry("kind", "modifier_pointer")?;
                map.serialize_entry("button", button_name(*button))?;
                map.serialize_entry("modifiers", &WireModifiers(modifiers))?;
                let phase = phase_name(*phase)
                    .ok_or_else(|| serde::ser::Error::custom("invalid pointer phase"))?;
                map.serialize_entry("phase", phase)?;
                map.end()
            }
            Gesture::KeyToggle { key, modifiers } => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("kind", "key_toggle")?;
                map.serialize_entry("key", &WireKey(*key))?;
                map.serialize_entry("modifiers", &WireModifiers(modifiers))?;
                map.end()
            }
        }
    }
}

struct WireKey(KeyToken);

impl Serialize for WireKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.0 {
            KeyToken::Ascii(key) => serializer.serialize_char(key.as_char()),
            key => key_name(key)
                .map(|name| serializer.serialize_str(name))
                .unwrap_or_else(|| Err(serde::ser::Error::custom("invalid named key"))),
        }
    }
}

struct WireModifiers<'a>(&'a Modifiers);

impl Serialize for WireModifiers<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.0.iter().len()))?;
        for modifier in self.0.iter() {
            sequence.serialize_element(modifier_name(modifier))?;
        }
        sequence.end()
    }
}

fn key_name(key: KeyToken) -> Option<&'static str> {
    match key {
        KeyToken::Ascii(_) => None,
        KeyToken::Space => Some("space"),
        KeyToken::Enter => Some("enter"),
        KeyToken::Escape => Some("escape"),
        KeyToken::Delete => Some("delete"),
        KeyToken::Backspace => Some("backspace"),
        KeyToken::Tab => Some("tab"),
        KeyToken::ArrowUp => Some("arrow_up"),
        KeyToken::ArrowDown => Some("arrow_down"),
        KeyToken::ArrowLeft => Some("arrow_left"),
        KeyToken::ArrowRight => Some("arrow_right"),
        KeyToken::Home => Some("home"),
        KeyToken::End => Some("end"),
        KeyToken::PageUp => Some("page_up"),
        KeyToken::PageDown => Some("page_down"),
    }
}

fn modifier_name(modifier: Modifier) -> &'static str {
    match modifier {
        Modifier::Primary => "primary",
        Modifier::Control => "control",
        Modifier::Meta => "meta",
        Modifier::Alt => "alt",
        Modifier::Shift => "shift",
    }
}

fn button_name(button: PointerButton) -> &'static str {
    match button {
        PointerButton::Primary => "primary",
        PointerButton::Secondary => "secondary",
        PointerButton::Middle => "middle",
        PointerButton::Auxiliary1 => "auxiliary_1",
        PointerButton::Auxiliary2 => "auxiliary_2",
    }
}

fn phase_name(phase: InputPhase) -> Option<&'static str> {
    match phase {
        InputPhase::Press => Some("press"),
        InputPhase::Release => Some("release"),
        InputPhase::Click => Some("click"),
        InputPhase::DragStart => Some("drag_start"),
        InputPhase::DragEnd => Some("drag_end"),
        InputPhase::DragUpdate | InputPhase::Cancel => None,
    }
}

#[derive(Debug, Clone)]
enum ParseViolation {
    Duplicate {
        key: Box<str>,
    },
    Limit {
        kind: LimitKind,
        observed: usize,
        limit: usize,
    },
}

fn parse_checked_json(input: &[u8], limits: KeymapCodecLimits) -> Result<Value, KeymapCodecError> {
    let violation = Rc::new(RefCell::new(None));
    let mut deserializer = serde_json::Deserializer::from_slice(input);
    let seed = CheckedValueSeed {
        limits,
        depth: 1,
        sequence_limit: None,
        violation: Rc::clone(&violation),
    };
    let value = seed.deserialize(&mut deserializer);
    if let Some(violation) = violation.borrow_mut().take() {
        return Err(match violation {
            ParseViolation::Duplicate { key } => KeymapCodecError::DuplicateObjectKey { key },
            ParseViolation::Limit {
                kind,
                observed,
                limit,
            } => KeymapCodecError::LimitExceeded {
                kind,
                observed,
                limit,
            },
        });
    }
    let value = value.map_err(|source| KeymapCodecError::InvalidJson { source })?;
    deserializer
        .end()
        .map_err(|source| KeymapCodecError::InvalidJson { source })?;
    Ok(value)
}

struct CheckedValueSeed {
    limits: KeymapCodecLimits,
    depth: usize,
    sequence_limit: Option<usize>,
    violation: Rc<RefCell<Option<ParseViolation>>>,
}

impl<'de> DeserializeSeed<'de> for CheckedValueSeed {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(CheckedValueVisitor(self))
    }
}

struct CheckedValueVisitor(CheckedValueSeed);

impl CheckedValueVisitor {
    fn check_string<E: serde::de::Error>(&self, value: &str) -> Result<(), E> {
        if value.len() > self.0.limits.max_string_bytes {
            self.record(ParseViolation::Limit {
                kind: LimitKind::StringBytes,
                observed: value.len(),
                limit: self.0.limits.max_string_bytes,
            });
            return Err(E::custom("keymap string limit exceeded"));
        }
        Ok(())
    }

    fn check_depth<E: serde::de::Error>(&self) -> Result<(), E> {
        if self.0.depth > self.0.limits.max_depth {
            self.record(ParseViolation::Limit {
                kind: LimitKind::Depth,
                observed: self.0.depth,
                limit: self.0.limits.max_depth,
            });
            return Err(E::custom("keymap depth limit exceeded"));
        }
        Ok(())
    }

    fn record(&self, violation: ParseViolation) {
        let mut slot = self.0.violation.borrow_mut();
        if slot.is_none() {
            *slot = Some(violation);
        }
    }

    fn child_seed(&self) -> CheckedValueSeed {
        CheckedValueSeed {
            limits: self.0.limits,
            depth: self.0.depth + 1,
            sequence_limit: None,
            violation: Rc::clone(&self.0.violation),
        }
    }

    fn operations_seed(&self) -> CheckedValueSeed {
        CheckedValueSeed {
            limits: self.0.limits,
            depth: self.0.depth + 1,
            sequence_limit: Some(self.0.limits.max_operations),
            violation: Rc::clone(&self.0.violation),
        }
    }

    fn discard_child_seed(&self) -> CheckedDiscardSeed {
        CheckedDiscardSeed {
            limits: self.0.limits,
            depth: self.0.depth + 1,
            violation: Rc::clone(&self.0.violation),
        }
    }
}

impl<'de> Visitor<'de> for CheckedValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value within keymap codec limits")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(Value::Number(Number::from(value)))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(Value::Number(Number::from(value)))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.check_string(value)?;
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.check_string(&value)?;
        Ok(Value::String(value))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        self.check_depth()?;
        let mut values = Vec::new();
        loop {
            if self
                .0
                .sequence_limit
                .is_some_and(|limit| values.len() == limit)
            {
                if sequence
                    .next_element_seed(self.discard_child_seed())?
                    .is_some()
                {
                    let limit = self.0.sequence_limit.unwrap_or(0);
                    self.record(ParseViolation::Limit {
                        kind: LimitKind::Operations,
                        observed: limit.saturating_add(1),
                        limit,
                    });
                    return Err(serde::de::Error::custom(
                        "keymap operation count limit exceeded",
                    ));
                }
                break;
            }
            let Some(value) = sequence.next_element_seed(self.child_seed())? else {
                break;
            };
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A>(self, mut object: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        self.check_depth()?;
        let mut values = Map::new();
        let mut keys = HashSet::new();
        while let Some(key) = object.next_key::<String>()? {
            self.check_string(&key)?;
            if !keys.insert(key.clone()) {
                object.next_value_seed(self.discard_child_seed())?;
                self.record(ParseViolation::Duplicate {
                    key: key.clone().into_boxed_str(),
                });
                return Err(serde::de::Error::custom("duplicate keymap object key"));
            }
            let value = if self.0.depth == 1 && key == "operations" {
                object.next_value_seed(self.operations_seed())?
            } else {
                object.next_value_seed(self.child_seed())?
            };
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
}

struct CheckedDiscardSeed {
    limits: KeymapCodecLimits,
    depth: usize,
    violation: Rc<RefCell<Option<ParseViolation>>>,
}

impl<'de> DeserializeSeed<'de> for CheckedDiscardSeed {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(CheckedDiscardVisitor(self))
    }
}

struct CheckedDiscardVisitor(CheckedDiscardSeed);

impl CheckedDiscardVisitor {
    fn check_string<E: serde::de::Error>(&self, value: &str) -> Result<(), E> {
        if value.len() > self.0.limits.max_string_bytes {
            self.record(ParseViolation::Limit {
                kind: LimitKind::StringBytes,
                observed: value.len(),
                limit: self.0.limits.max_string_bytes,
            });
            return Err(E::custom("discarded keymap string limit exceeded"));
        }
        Ok(())
    }

    fn check_depth<E: serde::de::Error>(&self) -> Result<(), E> {
        if self.0.depth > self.0.limits.max_depth {
            self.record(ParseViolation::Limit {
                kind: LimitKind::Depth,
                observed: self.0.depth,
                limit: self.0.limits.max_depth,
            });
            return Err(E::custom("discarded keymap depth limit exceeded"));
        }
        Ok(())
    }

    fn record(&self, violation: ParseViolation) {
        let mut slot = self.0.violation.borrow_mut();
        if slot.is_none() {
            *slot = Some(violation);
        }
    }

    fn child_seed(&self) -> CheckedDiscardSeed {
        CheckedDiscardSeed {
            limits: self.0.limits,
            depth: self.0.depth + 1,
            violation: Rc::clone(&self.0.violation),
        }
    }
}

impl<'de> Visitor<'de> for CheckedDiscardVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a discarded JSON value within keymap codec limits")
    }

    fn visit_bool<E>(self, _value: bool) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_i64<E>(self, _value: i64) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_u64<E>(self, _value: u64) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.check_string(value)
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.check_string(&value)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        self.check_depth()?;
        while sequence.next_element_seed(self.child_seed())?.is_some() {}
        Ok(())
    }

    fn visit_map<A>(self, mut object: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        self.check_depth()?;
        let mut keys = HashSet::new();
        while let Some(key) = object.next_key::<String>()? {
            self.check_string(&key)?;
            if !keys.insert(key.clone()) {
                object.next_value_seed(self.child_seed())?;
                self.record(ParseViolation::Duplicate {
                    key: key.into_boxed_str(),
                });
                return Err(serde::de::Error::custom(
                    "duplicate discarded keymap object key",
                ));
            }
            object.next_value_seed(self.child_seed())?;
        }
        Ok(())
    }
}
