//! egui Timelineの操作を既存Document編集queueへ変換する薄い境界。

use motolii_core::RationalTime;
use motolii_doc::{KeyframeId, LayerId};

use crate::document_edit_runtime::{
    AddPositionKeyRequest, DocumentEditDispatchError, DocumentEditQueue, RemovePositionKeyRequest,
    SetPositionKeyTimeRequest,
};
use crate::timeline_move_gesture::TimelineMoveRequest;
use crate::timeline_trim_gesture::TimelineTrimRequest;
use crate::{
    CommandId, DomainIntent, InputPhase, InputRouter, NormalizedInput, builtin_command_registry,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimelineIntent {
    MoveClip {
        layer: LayerId,
        new_start: RationalTime,
    },
    TrimClip(TimelineTrimRequest),
    AddPositionKey {
        target: LayerId,
        time: RationalTime,
    },
    MovePositionKey {
        target: LayerId,
        key: KeyframeId,
        old: RationalTime,
        new: RationalTime,
    },
    RemovePositionKey {
        target: LayerId,
        key: KeyframeId,
    },
    ToggleVisible(LayerId),
    ToggleSolo(LayerId),
    Select(Option<LayerId>),
    Undo,
    Redo,
    DuplicateLayer(LayerId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub(crate) enum TimelineIntentAdapterError {
    #[error("timeline history command could not be routed")]
    HistoryRoute,
    #[error(transparent)]
    Queue(#[from] DocumentEditDispatchError),
}

pub(crate) fn enqueue_timeline_intent(
    queue: &mut DocumentEditQueue,
    intent: TimelineIntent,
) -> Result<(), TimelineIntentAdapterError> {
    match intent {
        TimelineIntent::MoveClip { layer, new_start } => {
            queue.push_move_clip(TimelineMoveRequest { layer, new_start });
        }
        TimelineIntent::TrimClip(request) => queue.push_trim_clip(request),
        TimelineIntent::AddPositionKey { target, time } => {
            queue.push_add_position_key(AddPositionKeyRequest { target, time });
        }
        TimelineIntent::MovePositionKey {
            target,
            key,
            old,
            new,
        } => queue.push_set_position_key_time(SetPositionKeyTimeRequest {
            target,
            key,
            old,
            new,
        }),
        TimelineIntent::RemovePositionKey { target, key } => {
            queue.push_remove_position_key(RemovePositionKeyRequest { target, key });
        }
        TimelineIntent::ToggleVisible(layer) => queue.push_toggle_visible(layer),
        TimelineIntent::ToggleSolo(layer) => queue.push_toggle_solo(layer),
        TimelineIntent::Select(Some(layer)) => queue.push_replace_primary(layer),
        TimelineIntent::Select(None) => queue.push_clear_primary(),
        TimelineIntent::DuplicateLayer(layer) => queue.push_duplicate_layer(layer),
        TimelineIntent::Undo => enqueue_history(queue, DomainIntent::Undo)?,
        TimelineIntent::Redo => enqueue_history(queue, DomainIntent::Redo)?,
    }
    Ok(())
}

fn enqueue_history(
    queue: &mut DocumentEditQueue,
    intent: DomainIntent,
) -> Result<(), TimelineIntentAdapterError> {
    let command_id = match intent {
        DomainIntent::Undo => "motolii.edit.undo",
        DomainIntent::Redo => "motolii.edit.redo",
        _ => return Err(TimelineIntentAdapterError::HistoryRoute),
    };
    let registry =
        builtin_command_registry().map_err(|_| TimelineIntentAdapterError::HistoryRoute)?;
    let mut router = InputRouter::new(registry);
    let output = router
        .route(NormalizedInput::Command {
            phase: InputPhase::Press,
            id: CommandId::try_new(command_id)
                .map_err(|_| TimelineIntentAdapterError::HistoryRoute)?,
        })
        .map_err(|_| TimelineIntentAdapterError::HistoryRoute)?;
    queue.push_prepared(output, None)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn time(value: i64) -> RationalTime {
        RationalTime::try_new(value, 10).unwrap()
    }

    #[test]
    fn maps_timeline_edits_to_existing_queue_actions() {
        let layer = LayerId::from_raw(7);
        let key = KeyframeId::from_raw(3);
        let mut queue = DocumentEditQueue::default();

        for intent in [
            TimelineIntent::MoveClip {
                layer,
                new_start: time(20),
            },
            TimelineIntent::TrimClip(TimelineTrimRequest::Out {
                layer,
                new_end: time(80),
            }),
            TimelineIntent::AddPositionKey {
                target: layer,
                time: time(30),
            },
            TimelineIntent::MovePositionKey {
                target: layer,
                key,
                old: time(30),
                new: time(40),
            },
            TimelineIntent::RemovePositionKey { target: layer, key },
            TimelineIntent::ToggleVisible(layer),
            TimelineIntent::ToggleSolo(layer),
            TimelineIntent::Select(Some(layer)),
            TimelineIntent::Select(None),
            TimelineIntent::DuplicateLayer(layer),
        ] {
            enqueue_timeline_intent(&mut queue, intent).unwrap();
        }

        assert_eq!(queue.len(), 10);
    }

    #[test]
    fn maps_history_to_existing_prepared_route() {
        let mut queue = DocumentEditQueue::default();
        enqueue_timeline_intent(&mut queue, TimelineIntent::Undo).unwrap();
        enqueue_timeline_intent(&mut queue, TimelineIntent::Redo).unwrap();
        assert_eq!(queue.len(), 2);
    }
}
