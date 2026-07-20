//! 安定Command IDと最小metadata registry。

use std::collections::{BTreeMap, HashSet};

use crate::DomainIntent;

/// 表示名や物理入力から独立したcommandの意味ID。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommandId(Box<str>);

impl CommandId {
    pub fn try_new(value: impl Into<Box<str>>) -> Result<Self, CommandIdError> {
        let value = value.into();
        if value.is_empty() {
            return Err(CommandIdError::Empty);
        }
        if !has_valid_command_id_grammar(&value) {
            return Err(CommandIdError::InvalidGrammar { value });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn has_valid_command_id_grammar(value: &str) -> bool {
    let Some(rest) = value.strip_prefix("motolii.") else {
        return false;
    };
    !rest.is_empty() && rest.split('.').all(has_valid_segment)
}

fn has_valid_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CommandIdError {
    #[error("command ID must not be empty")]
    Empty,
    #[error("command ID does not follow the built-in grammar: {value}")]
    InvalidGrammar { value: Box<str> },
}

/// Commandの安定IDと表示情報、発行する目的。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandMetadata {
    pub id: CommandId,
    pub display_name: Box<str>,
    pub intent: DomainIntent,
}

impl CommandMetadata {
    pub fn new(id: CommandId, display_name: impl Into<Box<str>>, intent: DomainIntent) -> Self {
        Self {
            id,
            display_name: display_name.into(),
            intent,
        }
    }
}

/// 登録済みcommandを安定IDから引く閉じたcatalog。
#[derive(Debug, Clone)]
pub struct CommandRegistry {
    entries: BTreeMap<CommandId, CommandMetadata>,
}

impl CommandRegistry {
    pub fn try_new(
        metadata: impl IntoIterator<Item = CommandMetadata>,
    ) -> Result<Self, CommandRegistryError> {
        let mut entries = BTreeMap::new();
        let mut intents = HashSet::new();

        for item in metadata {
            if entries.contains_key(&item.id) {
                return Err(CommandRegistryError::DuplicateId { id: item.id });
            }
            if !intents.insert(item.intent) {
                return Err(CommandRegistryError::DuplicateIntent {
                    intent: item.intent,
                });
            }
            entries.insert(item.id.clone(), item);
        }

        for intent in DomainIntent::ALL {
            if !intents.contains(&intent) {
                return Err(CommandRegistryError::MissingIntent { intent });
            }
        }

        Ok(Self { entries })
    }

    pub fn get(&self, id: &CommandId) -> Option<&CommandMetadata> {
        self.entries.get(id)
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &CommandMetadata> {
        self.entries.values()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CommandRegistryError {
    #[error(transparent)]
    InvalidId(#[from] CommandIdError),
    #[error("command ID is registered more than once: {id}")]
    DuplicateId { id: CommandId },
    #[error("domain intent is registered more than once: {intent:?}")]
    DuplicateIntent { intent: DomainIntent },
    #[error("domain intent has no registered command: {intent:?}")]
    MissingIntent { intent: DomainIntent },
}

impl std::fmt::Display for CommandId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// U0c-1で閉じた5代表commandのregistryを構築する。
pub fn builtin_command_registry() -> Result<CommandRegistry, CommandRegistryError> {
    let entries = [
        builtin_metadata(
            "motolii.edit.delete_targeted_items",
            "Delete targeted items",
            DomainIntent::DeleteTargetedItems,
        )?,
        builtin_metadata(
            "motolii.settings.enable_reduce_motion",
            "Enable reduce motion",
            DomainIntent::EnableReduceMotion,
        )?,
        builtin_metadata(
            "motolii.workspace.reset_profile",
            "Reset workspace profile",
            DomainIntent::ResetWorkspaceProfile,
        )?,
        builtin_metadata(
            "motolii.view.fit_stage",
            "Fit stage view",
            DomainIntent::FitStageView,
        )?,
        builtin_metadata(
            "motolii.gesture.cancel",
            "Cancel in-flight gesture",
            DomainIntent::CancelInFlightGesture,
        )?,
    ];
    CommandRegistry::try_new(entries)
}

fn builtin_metadata(
    id: &'static str,
    display_name: &'static str,
    intent: DomainIntent,
) -> Result<CommandMetadata, CommandIdError> {
    Ok(CommandMetadata::new(
        CommandId::try_new(id)?,
        display_name,
        intent,
    ))
}
