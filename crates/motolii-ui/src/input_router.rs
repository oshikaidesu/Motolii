//! 正規入力の位相、IME優先、command解決を担うtoolkit非依存router。

use crate::{CommandId, CommandRegistry, DomainIntent};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InputPhase {
    Press,
    Release,
    Click,
    DragStart,
    DragUpdate,
    DragEnd,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImeGateState {
    Inactive,
    PreeditActive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SafetyInterrupt {
    PointerCaptureLost,
    WindowFocusLost,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NormalizedInput {
    Phase(InputPhase),
    Command { phase: InputPhase, id: CommandId },
    SafetyInterrupt(SafetyInterrupt),
    ImeOwned,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouterOutput {
    Phase(InputPhase),
    Intent {
        phase: InputPhase,
        id: CommandId,
        intent: DomainIntent,
    },
    ShortcutSuppressed {
        phase: InputPhase,
        id: CommandId,
    },
    ImeOwned,
    SafetyCancel {
        source: SafetyInterrupt,
        intent: DomainIntent,
    },
    SafetyIgnored {
        source: SafetyInterrupt,
    },
    CancelCommandIgnored {
        id: CommandId,
    },
}

#[derive(Debug, Clone)]
pub struct InputRouter {
    registry: CommandRegistry,
    ime_gate: ImeGateState,
    gesture_in_flight: bool,
}

impl InputRouter {
    pub fn new(registry: CommandRegistry) -> Self {
        Self {
            registry,
            ime_gate: ImeGateState::Inactive,
            gesture_in_flight: false,
        }
    }

    pub fn set_ime_gate(&mut self, state: ImeGateState) {
        self.ime_gate = state;
    }

    pub const fn ime_gate(&self) -> ImeGateState {
        self.ime_gate
    }

    pub const fn gesture_in_flight(&self) -> bool {
        self.gesture_in_flight
    }

    pub fn route(&mut self, input: NormalizedInput) -> Result<RouterOutput, InputRouterError> {
        match input {
            NormalizedInput::SafetyInterrupt(source) => Ok(self.route_safety_interrupt(source)),
            NormalizedInput::ImeOwned => Ok(RouterOutput::ImeOwned),
            NormalizedInput::Phase(phase) => {
                self.observe_phase(phase);
                Ok(RouterOutput::Phase(phase))
            }
            NormalizedInput::Command { phase, id } => self.route_command(phase, id),
        }
    }

    fn route_safety_interrupt(&mut self, source: SafetyInterrupt) -> RouterOutput {
        if !self.gesture_in_flight {
            return RouterOutput::SafetyIgnored { source };
        }
        self.gesture_in_flight = false;
        RouterOutput::SafetyCancel {
            source,
            intent: DomainIntent::CancelInFlightGesture,
        }
    }

    fn route_command(
        &mut self,
        phase: InputPhase,
        id: CommandId,
    ) -> Result<RouterOutput, InputRouterError> {
        let intent = self
            .registry
            .get(&id)
            .map(|metadata| metadata.intent)
            .ok_or_else(|| InputRouterError::UnknownCommandId { id: id.clone() })?;
        if self.ime_gate == ImeGateState::PreeditActive {
            return Ok(RouterOutput::ShortcutSuppressed { phase, id });
        }

        if intent == DomainIntent::CancelInFlightGesture {
            if !self.gesture_in_flight {
                return Ok(RouterOutput::CancelCommandIgnored { id });
            }
            self.gesture_in_flight = false;
            return Ok(RouterOutput::Intent {
                phase: InputPhase::Cancel,
                id,
                intent,
            });
        }

        Ok(RouterOutput::Intent { phase, id, intent })
    }

    fn observe_phase(&mut self, phase: InputPhase) {
        match phase {
            InputPhase::Press | InputPhase::DragStart | InputPhase::DragUpdate => {
                self.gesture_in_flight = true;
            }
            InputPhase::Release | InputPhase::Click | InputPhase::DragEnd | InputPhase::Cancel => {
                self.gesture_in_flight = false;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InputRouterError {
    #[error("command ID is not registered: {id}")]
    UnknownCommandId { id: CommandId },
}
