//! 決定済みD2 commandを1回のDocument編集要求へ畳むruntime境界。

use motolii_doc::{Command, CommandKind};

use crate::DomainIntent;

/// 上流が完成させたcommand列を、その順序のままsingle writerへ渡す要求。
#[derive(Debug)]
pub struct DocumentCommandRequest {
    intent: DomainIntent,
    commands: Vec<Command>,
}

impl DocumentCommandRequest {
    pub fn try_new(
        intent: DomainIntent,
        commands: Vec<Command>,
    ) -> Result<Self, DocumentCommandRequestError> {
        if commands.is_empty() {
            return Err(DocumentCommandRequestError::EmptyCommands);
        }
        if intent != DomainIntent::DeleteTargetedItems {
            return Err(DocumentCommandRequestError::NonDocumentIntent { intent });
        }
        for (index, command) in commands.iter().enumerate() {
            let actual = command.kind();
            if actual != CommandKind::RemoveTrackItem {
                return Err(DocumentCommandRequestError::CommandKindMismatch {
                    intent,
                    index,
                    expected: CommandKind::RemoveTrackItem,
                    actual,
                });
            }
        }
        Ok(Self { intent, commands })
    }

    pub const fn intent(&self) -> DomainIntent {
        self.intent
    }

    /// command列を並べ替えず、U2a-0の`apply_macro`へ渡せる形で返す。
    pub fn into_commands(self) -> Vec<Command> {
        self.commands
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum DocumentCommandRequestError {
    #[error("document command request must contain at least one command")]
    EmptyCommands,
    #[error("intent {intent:?} does not own Document state")]
    NonDocumentIntent { intent: DomainIntent },
    #[error("intent {intent:?} requires {expected:?}, but command {index} has kind {actual:?}")]
    CommandKindMismatch {
        intent: DomainIntent,
        index: usize,
        expected: CommandKind,
        actual: CommandKind,
    },
}
