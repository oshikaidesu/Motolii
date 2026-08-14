//! egui Timelineのclipboard系ショートカットを既存Host経路へ渡す意味境界。
//!
//! ここではclipboardの内容やDocument writerを所有しない。Skia/Host側と同じく、
//! 選択対象を layer または key として表し、後段のadapterが既存のcopy/cut/paste/
//! duplicate/delete dispatchへ変換できる形だけを作る。

use motolii_doc::{KeyframeId, LayerId};

use crate::timeline_egui::TimelineCommand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimelineKeyProperty {
    Position,
    Scale,
    Rotation,
    Opacity,
}

impl TimelineKeyProperty {
    pub(crate) const fn wire_name(self) -> &'static str {
        match self {
            Self::Position => "position",
            Self::Scale => "scale",
            Self::Rotation => "rotation",
            Self::Opacity => "opacity",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimelineClipboardSelection {
    None,
    Layer(LayerId),
    Key {
        layer: LayerId,
        key: KeyframeId,
        property: TimelineKeyProperty,
    },
}

impl TimelineClipboardSelection {
    pub(crate) const fn layer(self) -> Option<LayerId> {
        match self {
            Self::None => None,
            Self::Layer(layer) | Self::Key { layer, .. } => Some(layer),
        }
    }

    pub(crate) const fn key_target(self) -> Option<TimelineClipboardTarget> {
        match self {
            Self::Key {
                layer,
                key,
                property,
            } => Some(TimelineClipboardTarget::Key {
                layer,
                key,
                property,
            }),
            Self::None | Self::Layer(_) => None,
        }
    }

    pub(crate) const fn target(self) -> Option<TimelineClipboardTarget> {
        match self {
            Self::None => None,
            Self::Layer(layer) => Some(TimelineClipboardTarget::Layer(layer)),
            Self::Key {
                layer,
                key,
                property,
            } => Some(TimelineClipboardTarget::Key {
                layer,
                key,
                property,
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimelineClipboardTarget {
    Layer(LayerId),
    Key {
        layer: LayerId,
        key: KeyframeId,
        property: TimelineKeyProperty,
    },
}

impl TimelineClipboardTarget {
    pub(crate) const fn layer(self) -> LayerId {
        match self {
            Self::Layer(layer) | Self::Key { layer, .. } => layer,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimelineClipboardIntent {
    Copy { target: TimelineClipboardTarget },
    Cut { target: TimelineClipboardTarget },
    Paste { destination: Option<LayerId> },
    Duplicate { target: TimelineClipboardTarget },
    Delete { target: TimelineClipboardTarget },
}

impl TimelineClipboardIntent {
    pub(crate) const fn kind(self) -> TimelineClipboardKind {
        match self {
            Self::Copy { .. } => TimelineClipboardKind::Copy,
            Self::Cut { .. } => TimelineClipboardKind::Cut,
            Self::Paste { .. } => TimelineClipboardKind::Paste,
            Self::Duplicate { .. } => TimelineClipboardKind::Duplicate,
            Self::Delete { .. } => TimelineClipboardKind::Delete,
        }
    }

    pub(crate) const fn target(self) -> Option<TimelineClipboardTarget> {
        match self {
            Self::Copy { target }
            | Self::Cut { target }
            | Self::Duplicate { target }
            | Self::Delete { target } => Some(target),
            Self::Paste { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimelineClipboardKind {
    Copy,
    Cut,
    Paste,
    Duplicate,
    Delete,
}

/// egui Timelineのショートカットを、選択対象を含むHost dispatch前のintentへ変換する。
///
/// Native Timelineと同じく、対象が必要な操作はselectionがNoneなら発行しない。
/// Pasteだけはclipboard側が保持する内容をprimaryへ貼るため、destinationがNoneでも発行する。
pub(crate) fn clipboard_intent_for_command(
    command: TimelineCommand,
    selection: TimelineClipboardSelection,
) -> Option<TimelineClipboardIntent> {
    match command {
        TimelineCommand::Copy => selection
            .target()
            .map(|target| TimelineClipboardIntent::Copy { target }),
        TimelineCommand::Cut => selection
            .target()
            .map(|target| TimelineClipboardIntent::Cut { target }),
        TimelineCommand::Paste => Some(TimelineClipboardIntent::Paste {
            destination: selection.layer(),
        }),
        TimelineCommand::Duplicate => selection
            .target()
            .map(|target| TimelineClipboardIntent::Duplicate { target }),
        TimelineCommand::Delete | TimelineCommand::Backspace => selection
            .target()
            .map(|target| TimelineClipboardIntent::Delete { target }),
        TimelineCommand::Escape
        | TimelineCommand::Undo
        | TimelineCommand::Redo
        | TimelineCommand::SelectAll => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAYER: LayerId = LayerId::from_raw(7);
    const KEY: KeyframeId = KeyframeId::from_raw(11);

    const fn position_key() -> TimelineClipboardSelection {
        TimelineClipboardSelection::Key {
            layer: LAYER,
            key: KEY,
            property: TimelineKeyProperty::Position,
        }
    }

    #[test]
    fn key_shortcuts_preserve_layer_key_and_property() {
        let selection = position_key();
        let Some(TimelineClipboardIntent::Copy { target }) =
            clipboard_intent_for_command(TimelineCommand::Copy, selection)
        else {
            panic!("copy intent");
        };
        assert_eq!(
            target,
            TimelineClipboardTarget::Key {
                layer: LAYER,
                key: KEY,
                property: TimelineKeyProperty::Position,
            }
        );
        assert_eq!(TimelineKeyProperty::Position.wire_name(), "position");
    }

    #[test]
    fn layer_shortcuts_target_the_existing_layer() {
        let selection = TimelineClipboardSelection::Layer(LAYER);
        assert_eq!(
            clipboard_intent_for_command(TimelineCommand::Cut, selection),
            Some(TimelineClipboardIntent::Cut {
                target: TimelineClipboardTarget::Layer(LAYER),
            })
        );
        assert_eq!(
            clipboard_intent_for_command(TimelineCommand::Duplicate, selection),
            Some(TimelineClipboardIntent::Duplicate {
                target: TimelineClipboardTarget::Layer(LAYER),
            })
        );
        assert_eq!(
            clipboard_intent_for_command(TimelineCommand::Delete, selection),
            Some(TimelineClipboardIntent::Delete {
                target: TimelineClipboardTarget::Layer(LAYER),
            })
        );
    }

    #[test]
    fn paste_uses_selection_as_destination_without_requiring_one() {
        assert_eq!(
            clipboard_intent_for_command(TimelineCommand::Paste, position_key()),
            Some(TimelineClipboardIntent::Paste {
                destination: Some(LAYER),
            })
        );
        assert_eq!(
            clipboard_intent_for_command(TimelineCommand::Paste, TimelineClipboardSelection::None,),
            Some(TimelineClipboardIntent::Paste { destination: None })
        );
    }

    #[test]
    fn target_required_commands_are_noops_without_selection() {
        for command in [
            TimelineCommand::Copy,
            TimelineCommand::Cut,
            TimelineCommand::Duplicate,
            TimelineCommand::Delete,
            TimelineCommand::Backspace,
        ] {
            assert_eq!(
                clipboard_intent_for_command(command, TimelineClipboardSelection::None),
                None
            );
        }
    }
}
