//! 機能横断で共有する一時的な操作進行。

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionState {
    Discover,
    Target,
    Preview,
    Commit,
    Cancel,
    Inspect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InteractionStateMachine {
    state: InteractionState,
}

impl InteractionStateMachine {
    pub const fn new() -> Self {
        Self {
            state: InteractionState::Discover,
        }
    }

    pub const fn state(&self) -> InteractionState {
        self.state
    }

    pub fn transition(&mut self, to: InteractionState) -> Result<(), InteractionTransitionError> {
        let from = self.state;
        if !is_allowed_transition(from, to) {
            return Err(InteractionTransitionError { from, to });
        }
        self.state = to;
        Ok(())
    }
}

impl Default for InteractionStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("invalid interaction transition: {from:?} -> {to:?}")]
pub struct InteractionTransitionError {
    from: InteractionState,
    to: InteractionState,
}

impl InteractionTransitionError {
    pub const fn from(&self) -> InteractionState {
        self.from
    }

    pub const fn to(&self) -> InteractionState {
        self.to
    }
}

const fn is_allowed_transition(from: InteractionState, to: InteractionState) -> bool {
    matches!(
        (from, to),
        (InteractionState::Discover, InteractionState::Target)
            | (InteractionState::Target, InteractionState::Preview)
            | (InteractionState::Target, InteractionState::Commit)
            | (InteractionState::Target, InteractionState::Cancel)
            | (InteractionState::Preview, InteractionState::Commit)
            | (InteractionState::Preview, InteractionState::Cancel)
            | (InteractionState::Commit, InteractionState::Inspect)
            | (InteractionState::Cancel, InteractionState::Discover)
            | (InteractionState::Inspect, InteractionState::Discover)
    )
}
