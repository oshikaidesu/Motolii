//! Web窓がdesktop窓と同じ入力resolver/routerを使うための薄いWasm境界。

use motolii_input::{
    builtin_command_registry, product_action_repeat_disposition, resolve_product_action,
    EffectiveTrigger, ImeGateState, InputPhase, InputRouter, KeyToken, KeymapDelta, Modifier,
    Modifiers, NormalizedInput, PlatformCommandModifier, ProductAction, RepeatDisposition,
    RouterOutput, SafetyInterrupt,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Debug, thiserror::Error)]
enum CoreError {
    #[error("built-in command registry is invalid: {0}")]
    InvalidRegistry(#[from] motolii_input::CommandRegistryError),
    #[error("normalized input JSON is invalid: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("normalized input is invalid: {0}")]
    InvalidInput(String),
    #[error("input router rejected the command: {0}")]
    Router(#[from] motolii_input::InputRouterError),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum NormalizedRequest {
    Key {
        key: String,
        #[serde(default)]
        modifiers: Vec<NormalizedModifier>,
        phase: NormalizedPhase,
        #[serde(default)]
        composing: bool,
        #[serde(default)]
        editable: bool,
        #[serde(default)]
        repeat: bool,
    },
    Phase {
        phase: NormalizedPhase,
    },
    SafetyInterrupt {
        source: NormalizedSafetyInterrupt,
    },
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum NormalizedModifier {
    Control,
    Meta,
    Alt,
    Shift,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum NormalizedPhase {
    Press,
    Release,
    Click,
    DragStart,
    DragUpdate,
    DragEnd,
    Cancel,
}

impl From<NormalizedPhase> for InputPhase {
    fn from(value: NormalizedPhase) -> Self {
        match value {
            NormalizedPhase::Press => Self::Press,
            NormalizedPhase::Release => Self::Release,
            NormalizedPhase::Click => Self::Click,
            NormalizedPhase::DragStart => Self::DragStart,
            NormalizedPhase::DragUpdate => Self::DragUpdate,
            NormalizedPhase::DragEnd => Self::DragEnd,
            NormalizedPhase::Cancel => Self::Cancel,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum NormalizedSafetyInterrupt {
    PointerCaptureLost,
    WindowFocusLost,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum InputResolution {
    Command { command_id: String },
    ShortcutSuppressed { command_id: String },
    HostKind { kind: String },
    Unwired { kind: String },
    Phase { phase: String },
    SafetyCancel { source: String },
    SafetyIgnored { source: String },
    CancelCommandIgnored { command_id: String },
    RepeatConsumed,
    IgnoredEditable,
    Unbound,
}

struct ProductInputSession {
    platform: PlatformCommandModifier,
    registry: motolii_input::CommandRegistry,
    delta: KeymapDelta,
    router: InputRouter,
}

impl ProductInputSession {
    fn create(platform: PlatformCommandModifier) -> Result<Self, CoreError> {
        let registry = builtin_command_registry()?;
        let router = InputRouter::new(registry.clone());
        Ok(Self {
            platform,
            registry,
            delta: KeymapDelta::default(),
            router,
        })
    }

    fn dispatch(&mut self, request: NormalizedRequest) -> Result<InputResolution, CoreError> {
        match request {
            NormalizedRequest::Key {
                key,
                modifiers,
                phase,
                composing,
                editable,
                repeat,
            } => self.dispatch_key(key, modifiers, phase, composing, editable, repeat),
            NormalizedRequest::Phase { phase } => {
                let phase = InputPhase::from(phase);
                let output = self.router.route(NormalizedInput::Phase(phase))?;
                Ok(project_router_output(output))
            }
            NormalizedRequest::SafetyInterrupt { source } => {
                let source = match source {
                    NormalizedSafetyInterrupt::PointerCaptureLost => {
                        SafetyInterrupt::PointerCaptureLost
                    }
                    NormalizedSafetyInterrupt::WindowFocusLost => SafetyInterrupt::WindowFocusLost,
                };
                let output = self
                    .router
                    .route(NormalizedInput::SafetyInterrupt(source))?;
                Ok(project_router_output(output))
            }
        }
    }

    fn dispatch_key(
        &mut self,
        key: String,
        modifiers: Vec<NormalizedModifier>,
        phase: NormalizedPhase,
        composing: bool,
        editable: bool,
        repeat: bool,
    ) -> Result<InputResolution, CoreError> {
        if editable {
            return Ok(InputResolution::IgnoredEditable);
        }
        self.router.set_ime_gate(if composing {
            ImeGateState::PreeditActive
        } else {
            ImeGateState::Inactive
        });

        let phase = InputPhase::from(phase);
        let trigger = EffectiveTrigger::Keyboard {
            key: parse_key(&key)?,
            modifiers: Modifiers::try_new(modifiers.into_iter().map(|modifier| match modifier {
                NormalizedModifier::Control => Modifier::Control,
                NormalizedModifier::Meta => Modifier::Meta,
                NormalizedModifier::Alt => Modifier::Alt,
                NormalizedModifier::Shift => Modifier::Shift,
            }))
            .map_err(|error| CoreError::InvalidInput(error.to_string()))?,
            phase,
        };
        let Some(action) =
            resolve_product_action(&trigger, &self.registry, &self.delta, self.platform)
        else {
            return Ok(InputResolution::Unbound);
        };
        if repeat
            && product_action_repeat_disposition(&action)
                == RepeatDisposition::ConsumeWithoutDispatch
        {
            return Ok(InputResolution::RepeatConsumed);
        }

        match action {
            ProductAction::Command(id) => {
                let output = self.router.route(NormalizedInput::Command { phase, id })?;
                Ok(project_router_output(output))
            }
            ProductAction::HostKind(kind) => Ok(InputResolution::HostKind {
                kind: kind.into_string(),
            }),
            ProductAction::Unwired(kind) => Ok(InputResolution::Unwired {
                kind: kind.into_string(),
            }),
        }
    }
}

fn project_router_output(output: RouterOutput) -> InputResolution {
    match output {
        RouterOutput::Phase(phase) => InputResolution::Phase {
            phase: phase_name(phase).into(),
        },
        RouterOutput::Intent { id, .. } => InputResolution::Command {
            command_id: id.to_string(),
        },
        RouterOutput::ShortcutSuppressed { id, .. } => InputResolution::ShortcutSuppressed {
            command_id: id.to_string(),
        },
        RouterOutput::ImeOwned => InputResolution::Unbound,
        RouterOutput::SafetyCancel { source, .. } => InputResolution::SafetyCancel {
            source: safety_name(source).into(),
        },
        RouterOutput::SafetyIgnored { source } => InputResolution::SafetyIgnored {
            source: safety_name(source).into(),
        },
        RouterOutput::CancelCommandIgnored { id } => InputResolution::CancelCommandIgnored {
            command_id: id.to_string(),
        },
    }
}

fn phase_name(phase: InputPhase) -> &'static str {
    match phase {
        InputPhase::Press => "press",
        InputPhase::Release => "release",
        InputPhase::Click => "click",
        InputPhase::DragStart => "drag_start",
        InputPhase::DragUpdate => "drag_update",
        InputPhase::DragEnd => "drag_end",
        InputPhase::Cancel => "cancel",
    }
}

fn safety_name(source: SafetyInterrupt) -> &'static str {
    match source {
        SafetyInterrupt::PointerCaptureLost => "pointer_capture_lost",
        SafetyInterrupt::WindowFocusLost => "window_focus_lost",
    }
}

fn parse_key(value: &str) -> Result<KeyToken, CoreError> {
    let normalized = value.to_ascii_lowercase();
    if let Some(letter) = normalized.strip_prefix("key") {
        if letter.len() == 1 {
            return motolii_input::AsciiKey::try_new(
                letter.chars().next().expect("one physical key letter"),
            )
            .map(KeyToken::Ascii)
            .map_err(|error| CoreError::InvalidInput(error.to_string()));
        }
    }
    if let Some(digit) = normalized.strip_prefix("digit") {
        if digit.len() == 1 {
            return motolii_input::AsciiKey::try_new(
                digit.chars().next().expect("one physical key digit"),
            )
            .map(KeyToken::Ascii)
            .map_err(|error| CoreError::InvalidInput(error.to_string()));
        }
    }
    let key = match normalized.as_str() {
        "space" => KeyToken::Space,
        "enter" => KeyToken::Enter,
        "escape" => KeyToken::Escape,
        "delete" => KeyToken::Delete,
        "backspace" => KeyToken::Backspace,
        "tab" => KeyToken::Tab,
        "arrowup" | "arrow_up" => KeyToken::ArrowUp,
        "arrowdown" | "arrow_down" => KeyToken::ArrowDown,
        "arrowleft" | "arrow_left" => KeyToken::ArrowLeft,
        "arrowright" | "arrow_right" => KeyToken::ArrowRight,
        "home" => KeyToken::Home,
        "end" => KeyToken::End,
        "pageup" | "page_up" => KeyToken::PageUp,
        "pagedown" | "page_down" => KeyToken::PageDown,
        value if value.len() == 1 => KeyToken::Ascii(
            motolii_input::AsciiKey::try_new(value.chars().next().expect("one character"))
                .map_err(|error| CoreError::InvalidInput(error.to_string()))?,
        ),
        _ => {
            return Err(CoreError::InvalidInput(format!(
                "unknown key token: {value}"
            )))
        }
    };
    Ok(key)
}

fn parse_platform(value: &str) -> Result<PlatformCommandModifier, CoreError> {
    match value {
        "macos" => Ok(PlatformCommandModifier::Meta),
        "other" => Ok(PlatformCommandModifier::Control),
        _ => Err(CoreError::InvalidInput(format!(
            "unknown platform token: {value}"
        ))),
    }
}

#[wasm_bindgen]
pub struct WebInputCore {
    inner: ProductInputSession,
}

#[wasm_bindgen]
impl WebInputCore {
    #[wasm_bindgen(js_name = create)]
    pub fn create(platform: &str) -> Result<WebInputCore, JsValue> {
        ProductInputSession::create(parse_platform(platform).map_err(js_error)?)
            .map(|inner| Self { inner })
            .map_err(js_error)
    }

    pub fn dispatch(&mut self, request_json: &str) -> Result<String, JsValue> {
        let request = serde_json::from_str(request_json).map_err(CoreError::InvalidJson);
        request
            .and_then(|request| self.inner.dispatch(request))
            .and_then(|result| serde_json::to_string(&result).map_err(CoreError::InvalidJson))
            .map_err(js_error)
    }
}

fn js_error(error: CoreError) -> JsValue {
    JsValue::from_str(&error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dispatch(session: &mut ProductInputSession, request: &str) -> String {
        let request = serde_json::from_str(request).unwrap();
        serde_json::to_string(&session.dispatch(request).unwrap()).unwrap()
    }

    #[test]
    fn two_window_sessions_resolve_undo_redo_ime_and_safety_identically() {
        let mut desktop = ProductInputSession::create(PlatformCommandModifier::Meta).unwrap();
        let mut web = ProductInputSession::create(PlatformCommandModifier::Meta).unwrap();
        let sequence = [
            r#"{"type":"key","key":"z","modifiers":["meta"],"phase":"press"}"#,
            r#"{"type":"key","key":"z","modifiers":["meta","shift"],"phase":"press"}"#,
            r#"{"type":"key","key":"z","modifiers":["meta"],"phase":"press","composing":true}"#,
            r#"{"type":"phase","phase":"drag_start"}"#,
            r#"{"type":"safety_interrupt","source":"pointer_capture_lost"}"#,
        ];

        let desktop_results: Vec<_> = sequence
            .iter()
            .map(|request| dispatch(&mut desktop, request))
            .collect();
        let web_results: Vec<_> = sequence
            .iter()
            .map(|request| dispatch(&mut web, request))
            .collect();
        assert_eq!(web_results, desktop_results);
        assert!(web_results[0].contains("motolii.edit.undo"));
        assert!(web_results[1].contains("motolii.edit.redo"));
        assert!(web_results[2].contains("shortcut_suppressed"));
        assert!(web_results[4].contains("safety_cancel"));
    }

    #[test]
    fn dom_physical_codes_resolve_without_locale_text() {
        let mut session = ProductInputSession::create(PlatformCommandModifier::Meta).unwrap();
        let undo = dispatch(
            &mut session,
            r#"{"type":"key","key":"KeyZ","modifiers":["meta"],"phase":"press"}"#,
        );
        assert!(undo.contains("motolii.edit.undo"));

        let next_step = dispatch(
            &mut session,
            r#"{"type":"key","key":"ArrowRight","modifiers":["meta"],"phase":"press"}"#,
        );
        assert!(next_step.contains("goto_next_step"));
        assert!(matches!(parse_key("Digit7"), Ok(KeyToken::Ascii(_))));
    }

    #[test]
    fn repeat_policy_is_shared_with_the_native_bridge() {
        let mut session = ProductInputSession::create(PlatformCommandModifier::Meta).unwrap();
        let repeated_play = dispatch(
            &mut session,
            r#"{"type":"key","key":"Space","modifiers":[],"phase":"press","repeat":true}"#,
        );
        assert!(repeated_play.contains("repeat_consumed"));

        let repeated_step = dispatch(
            &mut session,
            r#"{"type":"key","key":"ArrowRight","modifiers":["meta"],"phase":"press","repeat":true}"#,
        );
        assert!(repeated_step.contains("goto_next_step"));
    }
}
