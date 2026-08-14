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

/// 仮の default profile。後から同じ owner へ Premiere 等を載せる。
pub const PRODUCT_KEYMAP_PROFILE_ID: &str = "ableton";
pub const PRODUCT_BUILTIN_KEYMAP_VERSION: u32 = 3;

/// 既存 RN host kind。CommandId ではない。
pub const PRODUCT_HOST_KIND_TOGGLE_PLAYBACK: &str = "toggle_playback";
pub const PRODUCT_HOST_KIND_SHUTTLE_FORWARD: &str = "shuttle_forward";
pub const PRODUCT_HOST_KIND_SHUTTLE_REVERSE: &str = "shuttle_reverse";
pub const PRODUCT_HOST_KIND_SHUTTLE_STOP: &str = "shuttle_stop";
pub const PRODUCT_HOST_KIND_TRIM_CLIP_IN: &str = "trim_clip_in";
pub const PRODUCT_HOST_KIND_TRIM_CLIP_OUT: &str = "trim_clip_out";
pub const PRODUCT_HOST_KIND_DUPLICATE: &str = "duplicate";
pub const PRODUCT_HOST_KIND_SOLO: &str = "solo";
pub const PRODUCT_HOST_KIND_MUTE: &str = "mute";
pub const PRODUCT_HOST_KIND_SPLIT: &str = "split";
pub const PRODUCT_HOST_KIND_COPY: &str = "copy";
pub const PRODUCT_HOST_KIND_CUT: &str = "cut";
pub const PRODUCT_HOST_KIND_PASTE: &str = "paste";
pub const PRODUCT_HOST_KIND_SELECT_ALL: &str = "select_all";
pub const PRODUCT_HOST_KIND_GOTO_NEXT_KEY: &str = "goto_next_key";
pub const PRODUCT_HOST_KIND_GOTO_PREV_KEY: &str = "goto_prev_key";
pub const PRODUCT_HOST_KIND_GOTO_NEXT_STEP: &str = "goto_next_step";
pub const PRODUCT_HOST_KIND_GOTO_PREV_STEP: &str = "goto_prev_step";
/// 互換 alias。Cmd+D / S / M / Cmd+K の正は HOST_KIND。
pub const PRODUCT_UNWIRED_DUPLICATE: &str = PRODUCT_HOST_KIND_DUPLICATE;
pub const PRODUCT_UNWIRED_SOLO: &str = PRODUCT_HOST_KIND_SOLO;
pub const PRODUCT_UNWIRED_MUTE: &str = PRODUCT_HOST_KIND_MUTE;
pub const PRODUCT_UNWIRED_SPLIT: &str = PRODUCT_HOST_KIND_SPLIT;

/// キー表の解決結果。新しい command ではなく既存意味への接続。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductAction {
    Command(CommandId),
    HostKind(Box<str>),
    Unwired(Box<str>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatDisposition {
    Dispatch,
    ConsumeWithoutDispatch,
}

/// 押しっぱなしを一度だけの製品操作へ再送しない共通規則。
pub fn product_action_repeat_disposition(action: &ProductAction) -> RepeatDisposition {
    match product_action_host_kind(action) {
        Some(
            PRODUCT_HOST_KIND_TOGGLE_PLAYBACK
            | PRODUCT_HOST_KIND_SHUTTLE_FORWARD
            | PRODUCT_HOST_KIND_SHUTTLE_STOP
            | PRODUCT_HOST_KIND_TRIM_CLIP_IN
            | PRODUCT_HOST_KIND_TRIM_CLIP_OUT,
        ) => RepeatDisposition::ConsumeWithoutDispatch,
        _ => RepeatDisposition::Dispatch,
    }
}

fn static_ascii(value: char) -> KeyToken {
    KeyToken::Ascii(AsciiKey::try_new(value).expect("static ascii key"))
}

fn static_mods(modifiers: &[Modifier]) -> Modifiers {
    Modifiers::try_new(modifiers.iter().copied()).expect("static modifiers")
}

fn press(key: KeyToken, modifiers: Modifiers) -> Gesture {
    Gesture::Keyboard {
        key,
        modifiers,
        phase: InputPhase::Press,
    }
}

fn command_binding(gesture: Gesture, id: &str) -> Binding {
    Binding {
        gesture,
        command: CommandId::try_new(id).expect("static command id"),
    }
}

/// Ableton 仮 default。登録済み CommandId だけを builtin に載せる。
pub fn product_builtin_keymap() -> BuiltinKeymap {
    let primary = static_mods(&[Modifier::Primary]);
    let primary_shift = static_mods(&[Modifier::Primary, Modifier::Shift]);
    let none = Modifiers::default();
    let z = static_ascii('z');
    BuiltinKeymap::new(
        PRODUCT_BUILTIN_KEYMAP_VERSION,
        vec![
            command_binding(press(z, primary), "motolii.edit.undo"),
            command_binding(press(z, primary_shift), "motolii.edit.redo"),
            command_binding(
                press(KeyToken::Escape, none.clone()),
                "motolii.gesture.cancel",
            ),
            command_binding(
                press(KeyToken::Delete, none.clone()),
                "motolii.edit.delete_targeted_items",
            ),
            command_binding(
                press(KeyToken::Backspace, none),
                "motolii.edit.delete_targeted_items",
            ),
        ],
    )
}

fn host_kind_rows() -> Vec<(Gesture, &'static str)> {
    let primary = static_mods(&[Modifier::Primary]);
    let none = Modifiers::default();
    vec![
        (
            press(KeyToken::Space, none.clone()),
            PRODUCT_HOST_KIND_TOGGLE_PLAYBACK,
        ),
        (
            press(static_ascii('l'), none.clone()),
            PRODUCT_HOST_KIND_SHUTTLE_FORWARD,
        ),
        (
            press(static_ascii('j'), none.clone()),
            PRODUCT_HOST_KIND_SHUTTLE_REVERSE,
        ),
        (
            press(static_ascii('k'), none.clone()),
            PRODUCT_HOST_KIND_SHUTTLE_STOP,
        ),
        (
            press(static_ascii('i'), none.clone()),
            PRODUCT_HOST_KIND_TRIM_CLIP_IN,
        ),
        (
            press(static_ascii('o'), none.clone()),
            PRODUCT_HOST_KIND_TRIM_CLIP_OUT,
        ),
        (
            press(static_ascii('d'), primary.clone()),
            PRODUCT_HOST_KIND_DUPLICATE,
        ),
        (
            press(static_ascii('k'), primary.clone()),
            PRODUCT_HOST_KIND_SPLIT,
        ),
        (
            press(static_ascii('c'), primary.clone()),
            PRODUCT_HOST_KIND_COPY,
        ),
        (
            press(static_ascii('x'), primary.clone()),
            PRODUCT_HOST_KIND_CUT,
        ),
        (
            press(static_ascii('v'), primary.clone()),
            PRODUCT_HOST_KIND_PASTE,
        ),
        (
            press(static_ascii('a'), primary.clone()),
            PRODUCT_HOST_KIND_SELECT_ALL,
        ),
        (
            press(
                static_ascii('d'),
                static_mods(&[Modifier::Shift, Modifier::Alt]),
            ),
            PRODUCT_HOST_KIND_GOTO_NEXT_KEY,
        ),
        (
            press(
                static_ascii('a'),
                static_mods(&[Modifier::Shift, Modifier::Alt]),
            ),
            PRODUCT_HOST_KIND_GOTO_PREV_KEY,
        ),
        (
            press(KeyToken::ArrowRight, primary.clone()),
            PRODUCT_HOST_KIND_GOTO_NEXT_STEP,
        ),
        (
            press(KeyToken::ArrowLeft, primary),
            PRODUCT_HOST_KIND_GOTO_PREV_STEP,
        ),
        (
            press(static_ascii('s'), none.clone()),
            PRODUCT_HOST_KIND_SOLO,
        ),
        (press(static_ascii('m'), none), PRODUCT_HOST_KIND_MUTE),
    ]
}

fn unwired_rows() -> Vec<(Gesture, &'static str)> {
    Vec::new()
}

fn expand_rows(
    rows: Vec<(Gesture, &'static str)>,
    delta: &KeymapDelta,
    platform: PlatformCommandModifier,
) -> BTreeMap<EffectiveTrigger, &'static str> {
    let disabled: BTreeSet<Gesture> = delta
        .operations()
        .iter()
        .filter_map(|operation| match operation {
            DeltaOperation::Disable { gesture } => Some(gesture.clone()),
            _ => None,
        })
        .collect();
    let mut map = BTreeMap::new();
    for (gesture, action) in rows {
        if disabled.contains(&gesture) {
            continue;
        }
        if let Ok(triggers) = expand_gesture(&gesture, platform) {
            for trigger in triggers {
                map.insert(trigger, action);
            }
        }
    }
    map
}

/// builtin CommandId 行と既存 host kind 行を合成する。
pub fn resolve_product_action(
    trigger: &EffectiveTrigger,
    registry: &CommandRegistry,
    delta: &KeymapDelta,
    platform: PlatformCommandModifier,
) -> Option<ProductAction> {
    let resolution = resolve_keymap(
        &product_builtin_keymap(),
        delta,
        &PlatformBindingConstraints::new(platform, Vec::new()),
        registry,
    );
    if let Some(command) = resolution.get(trigger) {
        return Some(ProductAction::Command(command.clone()));
    }
    if let Some(kind) = expand_rows(host_kind_rows(), delta, platform).get(trigger) {
        return Some(ProductAction::HostKind((*kind).into()));
    }
    expand_rows(unwired_rows(), delta, platform)
        .get(trigger)
        .map(|kind| ProductAction::Unwired((*kind).into()))
}

/// 既存 `try_dispatch_keymap` kind へ。未接続は None。
pub fn product_action_host_kind(action: &ProductAction) -> Option<&'static str> {
    match action {
        ProductAction::Command(id) => match id.as_str() {
            "motolii.edit.undo" => Some("undo"),
            "motolii.edit.redo" => Some("redo"),
            "motolii.edit.delete_targeted_items" => Some("delete_layer"),
            _ => None,
        },
        ProductAction::HostKind(kind) => [
            PRODUCT_HOST_KIND_TOGGLE_PLAYBACK,
            PRODUCT_HOST_KIND_SHUTTLE_FORWARD,
            PRODUCT_HOST_KIND_SHUTTLE_REVERSE,
            PRODUCT_HOST_KIND_SHUTTLE_STOP,
            PRODUCT_HOST_KIND_TRIM_CLIP_IN,
            PRODUCT_HOST_KIND_TRIM_CLIP_OUT,
            PRODUCT_HOST_KIND_DUPLICATE,
            PRODUCT_HOST_KIND_SOLO,
            PRODUCT_HOST_KIND_MUTE,
            PRODUCT_HOST_KIND_SPLIT,
            PRODUCT_HOST_KIND_COPY,
            PRODUCT_HOST_KIND_CUT,
            PRODUCT_HOST_KIND_PASTE,
            PRODUCT_HOST_KIND_SELECT_ALL,
            PRODUCT_HOST_KIND_GOTO_NEXT_KEY,
            PRODUCT_HOST_KIND_GOTO_PREV_KEY,
            PRODUCT_HOST_KIND_GOTO_NEXT_STEP,
            PRODUCT_HOST_KIND_GOTO_PREV_STEP,
        ]
        .into_iter()
        .find(|candidate| kind.as_ref() == *candidate),
        ProductAction::Unwired(_) => None,
    }
}
