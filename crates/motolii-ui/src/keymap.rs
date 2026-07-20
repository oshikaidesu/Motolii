//! builtin baseとuser deltaを合成する純粋keymap resolver。

use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

use crate::{CommandId, CommandRegistry, InputPhase};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AsciiKey(u8);

impl AsciiKey {
    pub fn try_new(value: char) -> Result<Self, AsciiKeyError> {
        if value.is_ascii_lowercase() || value.is_ascii_digit() {
            Ok(Self(value as u8))
        } else {
            Err(AsciiKeyError::NotLowercaseLetterOrDigit { value })
        }
    }

    pub const fn as_char(self) -> char {
        self.0 as char
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AsciiKeyError {
    #[error("key token must be a lowercase ASCII letter or digit: {value}")]
    NotLowercaseLetterOrDigit { value: char },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyToken {
    Ascii(AsciiKey),
    Space,
    Enter,
    Escape,
    Delete,
    Backspace,
    Tab,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    PageUp,
    PageDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Modifier {
    Primary,
    Control,
    Meta,
    Alt,
    Shift,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Modifiers(Vec<Modifier>);

impl Modifiers {
    pub fn try_new(modifiers: impl IntoIterator<Item = Modifier>) -> Result<Self, ModifierError> {
        let modifiers: BTreeSet<_> = modifiers.into_iter().collect();
        if modifiers.contains(&Modifier::Primary)
            && (modifiers.contains(&Modifier::Control) || modifiers.contains(&Modifier::Meta))
        {
            return Err(ModifierError::PrimaryWithExplicitCommandModifier);
        }
        Ok(Self(modifiers.into_iter().collect()))
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = Modifier> + '_ {
        self.0.iter().copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ModifierError {
    #[error("Primary cannot be combined with explicit Control or Meta")]
    PrimaryWithExplicitCommandModifier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
    Auxiliary1,
    Auxiliary2,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Gesture {
    Keyboard {
        key: KeyToken,
        modifiers: Modifiers,
        phase: InputPhase,
    },
    ModifierPointer {
        button: PointerButton,
        modifiers: Modifiers,
        phase: InputPhase,
    },
    KeyToggle {
        key: KeyToken,
        modifiers: Modifiers,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    pub gesture: Gesture,
    pub command: CommandId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltinKeymap {
    pub version: u32,
    bindings: Vec<Binding>,
}

impl BuiltinKeymap {
    pub fn new(version: u32, bindings: Vec<Binding>) -> Self {
        Self { version, bindings }
    }

    pub fn bindings(&self) -> &[Binding] {
        &self.bindings
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeltaOperation {
    Add(Binding),
    Replace(Binding),
    Disable { gesture: Gesture },
}

impl DeltaOperation {
    fn gesture(&self) -> &Gesture {
        match self {
            Self::Add(binding) | Self::Replace(binding) => &binding.gesture,
            Self::Disable { gesture } => gesture,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KeymapDelta {
    operations: Vec<DeltaOperation>,
}

impl KeymapDelta {
    pub fn new(operations: Vec<DeltaOperation>) -> Self {
        Self { operations }
    }

    pub fn operations(&self) -> &[DeltaOperation] {
        &self.operations
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformCommandModifier {
    Control,
    Meta,
}

impl PlatformCommandModifier {
    const fn modifier(self) -> Modifier {
        match self {
            Self::Control => Modifier::Control,
            Self::Meta => Modifier::Meta,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EffectiveTrigger {
    Keyboard {
        key: KeyToken,
        modifiers: Modifiers,
        phase: InputPhase,
    },
    Pointer {
        button: PointerButton,
        modifiers: Modifiers,
        phase: InputPhase,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformBindingConstraints {
    pub command_modifier: PlatformCommandModifier,
    reserved: BTreeSet<EffectiveTrigger>,
}

impl PlatformBindingConstraints {
    pub fn new(
        command_modifier: PlatformCommandModifier,
        reserved: impl IntoIterator<Item = EffectiveTrigger>,
    ) -> Self {
        Self {
            command_modifier,
            reserved: reserved.into_iter().collect(),
        }
    }

    pub fn is_reserved(&self, trigger: &EffectiveTrigger) -> bool {
        self.reserved.contains(trigger)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeymapDiagnostic {
    DuplicateBaseGesture {
        gesture: Gesture,
    },
    MultipleDeltaOperations {
        gesture: Gesture,
    },
    AddTargetsBase {
        gesture: Gesture,
    },
    ReplaceTargetMissing {
        gesture: Gesture,
    },
    DisableTargetMissing {
        gesture: Gesture,
    },
    UnknownCommandId {
        id: CommandId,
    },
    InvalidGesturePhase {
        gesture: Gesture,
    },
    Conflict {
        trigger: EffectiveTrigger,
        commands: Vec<CommandId>,
    },
    UnavailableOnPlatform {
        trigger: EffectiveTrigger,
        command: CommandId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeymapResolution {
    bindings: BTreeMap<EffectiveTrigger, CommandId>,
    diagnostics: Vec<KeymapDiagnostic>,
}

impl KeymapResolution {
    pub fn get(&self, trigger: &EffectiveTrigger) -> Option<&CommandId> {
        self.bindings.get(trigger)
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&EffectiveTrigger, &CommandId)> {
        self.bindings.iter()
    }

    pub fn diagnostics(&self) -> &[KeymapDiagnostic] {
        &self.diagnostics
    }
}

pub fn resolve_keymap(
    base: &BuiltinKeymap,
    delta: &KeymapDelta,
    platform: &PlatformBindingConstraints,
    registry: &CommandRegistry,
) -> KeymapResolution {
    let mut diagnostics = Vec::new();
    let mut overlay = BTreeMap::<Gesture, CommandId>::new();
    let mut duplicate_base = BTreeSet::new();

    for binding in &base.bindings {
        if overlay
            .insert(binding.gesture.clone(), binding.command.clone())
            .is_some()
        {
            duplicate_base.insert(binding.gesture.clone());
        }
    }
    for gesture in duplicate_base {
        overlay.remove(&gesture);
        diagnostics.push(KeymapDiagnostic::DuplicateBaseGesture { gesture });
    }

    let mut operations = BTreeMap::<Gesture, Vec<&DeltaOperation>>::new();
    for operation in &delta.operations {
        operations
            .entry(operation.gesture().clone())
            .or_default()
            .push(operation);
    }

    for (gesture, group) in operations {
        if group.len() != 1 {
            diagnostics.push(KeymapDiagnostic::MultipleDeltaOperations { gesture });
            continue;
        }
        match group[0] {
            DeltaOperation::Add(binding) => match overlay.entry(gesture) {
                Entry::Occupied(entry) => {
                    diagnostics.push(KeymapDiagnostic::AddTargetsBase {
                        gesture: entry.key().clone(),
                    });
                }
                Entry::Vacant(entry)
                    if registered(&binding.command, registry, &mut diagnostics) =>
                {
                    entry.insert(binding.command.clone());
                }
                Entry::Vacant(_) => {}
            },
            DeltaOperation::Replace(binding) => match overlay.entry(gesture) {
                Entry::Vacant(entry) => {
                    diagnostics.push(KeymapDiagnostic::ReplaceTargetMissing {
                        gesture: entry.into_key(),
                    });
                }
                Entry::Occupied(mut entry)
                    if registered(&binding.command, registry, &mut diagnostics) =>
                {
                    entry.insert(binding.command.clone());
                }
                Entry::Occupied(_) => {}
            },
            DeltaOperation::Disable { .. } => {
                if overlay.remove(&gesture).is_none() {
                    diagnostics.push(KeymapDiagnostic::DisableTargetMissing { gesture });
                }
            }
        }
    }

    overlay.retain(|_, command| registered(command, registry, &mut diagnostics));

    let mut candidates = BTreeMap::<EffectiveTrigger, BTreeSet<CommandId>>::new();
    for (gesture, command) in overlay {
        match expand_gesture(&gesture, platform.command_modifier) {
            Ok(triggers) => {
                for trigger in triggers {
                    candidates
                        .entry(trigger)
                        .or_default()
                        .insert(command.clone());
                }
            }
            Err(()) => diagnostics.push(KeymapDiagnostic::InvalidGesturePhase { gesture }),
        }
    }

    let mut bindings = BTreeMap::new();
    for (trigger, commands) in candidates {
        let commands: Vec<_> = commands.into_iter().collect();
        if commands.len() > 1 {
            diagnostics.push(KeymapDiagnostic::Conflict { trigger, commands });
            continue;
        }
        let Some(command) = commands.into_iter().next() else {
            continue;
        };
        if platform.is_reserved(&trigger) {
            diagnostics.push(KeymapDiagnostic::UnavailableOnPlatform { trigger, command });
        } else {
            bindings.insert(trigger, command);
        }
    }

    KeymapResolution {
        bindings,
        diagnostics,
    }
}

fn registered(
    id: &CommandId,
    registry: &CommandRegistry,
    diagnostics: &mut Vec<KeymapDiagnostic>,
) -> bool {
    if registry.get(id).is_some() {
        true
    } else {
        if !diagnostics.iter().any(
            |item| matches!(item, KeymapDiagnostic::UnknownCommandId { id: seen } if seen == id),
        ) {
            diagnostics.push(KeymapDiagnostic::UnknownCommandId { id: id.clone() });
        }
        false
    }
}

fn expand_gesture(
    gesture: &Gesture,
    command_modifier: PlatformCommandModifier,
) -> Result<Vec<EffectiveTrigger>, ()> {
    match gesture {
        Gesture::Keyboard {
            key,
            modifiers,
            phase,
        } if matches!(phase, InputPhase::Press | InputPhase::Release) => {
            Ok(vec![EffectiveTrigger::Keyboard {
                key: *key,
                modifiers: expand_modifiers(modifiers, command_modifier)?,
                phase: *phase,
            }])
        }
        Gesture::ModifierPointer {
            button,
            modifiers,
            phase,
        } if matches!(
            phase,
            InputPhase::Press
                | InputPhase::Release
                | InputPhase::Click
                | InputPhase::DragStart
                | InputPhase::DragEnd
        ) =>
        {
            Ok(vec![EffectiveTrigger::Pointer {
                button: *button,
                modifiers: expand_modifiers(modifiers, command_modifier)?,
                phase: *phase,
            }])
        }
        Gesture::KeyToggle { key, modifiers } => {
            let modifiers = expand_modifiers(modifiers, command_modifier)?;
            Ok(vec![
                EffectiveTrigger::Keyboard {
                    key: *key,
                    modifiers: modifiers.clone(),
                    phase: InputPhase::Press,
                },
                EffectiveTrigger::Keyboard {
                    key: *key,
                    modifiers,
                    phase: InputPhase::Release,
                },
            ])
        }
        _ => Err(()),
    }
}

fn expand_modifiers(
    modifiers: &Modifiers,
    command_modifier: PlatformCommandModifier,
) -> Result<Modifiers, ()> {
    let expanded = modifiers.iter().map(|modifier| {
        if modifier == Modifier::Primary {
            command_modifier.modifier()
        } else {
            modifier
        }
    });
    Modifiers::try_new(expanded).map_err(|_| ())
}
