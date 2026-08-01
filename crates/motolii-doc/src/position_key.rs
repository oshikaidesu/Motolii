//! U4b-0: explicit Add Position Keyのpure prepare。

use motolii_core::RationalTime;
use motolii_eval::{Interp, TrackError, Value as EvalValue};

use crate::command::{find_envelope, validate_reservation_closure};
use crate::{
    Command, CommandError, DocKeyframe, DocKeyframeTrack, DocParam, DocValue, Document, KeyframeId,
    LayerId, StableIdReservation,
};

#[derive(Debug, Clone, PartialEq)]
pub enum PreparedAddPositionKey {
    Edit {
        command: Box<Command>,
        key_id: KeyframeId,
    },
    AlreadyPresent {
        key_id: KeyframeId,
    },
}

enum BuildPositionKey {
    Added(DocParam),
    AlreadyPresent,
}

pub(crate) fn prepare_add_position_key(
    doc: &Document,
    target: LayerId,
    playhead: RationalTime,
) -> Result<PreparedAddPositionKey, CommandError> {
    guard_stable_id_document(doc)?;
    let envelope = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(target.get()))?;
    let old_value = envelope.transform.position.clone();
    if let Some(key_id) = existing_key_at(&old_value, playhead)? {
        return Ok(PreparedAddPositionKey::AlreadyPresent { key_id });
    }

    let before = doc.next_stable_id.peek_next();
    let mut sequence = doc.next_stable_id;
    let key_id = KeyframeId::from_raw(sequence.allocate()?);
    let reservation = StableIdReservation::new(before, sequence.peek_next());
    validate_reservation_closure(reservation, &[key_id.get()])?;
    let BuildPositionKey::Added(new_value) =
        build_position_key_value(&old_value, playhead, key_id, target)?
    else {
        return Err(CommandError::PositionKeyPayloadMismatch {
            layer: target.get(),
        });
    };
    let command = Command::AddPositionKey {
        target,
        old_value,
        new_value,
        added_key_id: key_id,
        stable_id_reservation: reservation,
    };
    let mut candidate = doc.clone();
    command.apply(&mut candidate)?;
    Ok(PreparedAddPositionKey::Edit {
        command: Box::new(command),
        key_id,
    })
}

pub(crate) fn validate_add_position_key_payload(
    old_value: &DocParam,
    new_value: &DocParam,
    added_key_id: KeyframeId,
    target: LayerId,
) -> Result<(), CommandError> {
    let playhead = match new_value {
        DocParam::Keyframes(track) => track.get_by_id(added_key_id).map(|key| key.t).ok_or(
            CommandError::PositionKeyPayloadMismatch {
                layer: target.get(),
            },
        )?,
        _ => {
            return Err(CommandError::PositionKeyPayloadMismatch {
                layer: target.get(),
            })
        }
    };
    match build_position_key_value(old_value, playhead, added_key_id, target)? {
        BuildPositionKey::Added(expected) if expected == *new_value => Ok(()),
        _ => Err(CommandError::PositionKeyPayloadMismatch {
            layer: target.get(),
        }),
    }
}

pub(crate) fn guard_stable_id_document(doc: &Document) -> Result<(), CommandError> {
    if doc.min_reader_version < 2 {
        return Err(CommandError::PositionKeyRequiresStableIds {
            version: doc.version,
            min_reader_version: doc.min_reader_version,
        });
    }
    doc.validate().map_err(CommandError::Validate)
}

pub(crate) fn prepare_set_position_key_interp(
    doc: &Document,
    target: LayerId,
    left_key_id: KeyframeId,
    new: Interp,
) -> Result<Option<Command>, CommandError> {
    guard_stable_id_document(doc)?;
    crate::doc_keyframe::validate_interp(&new)?;
    let envelope = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(target.get()))?;
    let DocParam::Keyframes(track) = &envelope.transform.position else {
        return Err(CommandError::UnsupportedPositionKeySource);
    };
    let (_, right_key_id, old) = position_interval(track, target, left_key_id)?;
    if old == new {
        return Ok(None);
    }
    Ok(Some(Command::SetPositionKeyInterp {
        target,
        left_key_id,
        right_key_id,
        old,
        new,
    }))
}

pub(crate) fn apply_set_position_key_interp(
    doc: &mut Document,
    target: LayerId,
    left_key_id: KeyframeId,
    right_key_id: KeyframeId,
    old: Interp,
    new: Interp,
) -> Result<(), CommandError> {
    guard_stable_id_document(doc)?;
    crate::doc_keyframe::validate_interp(&new)?;
    let envelope = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(target.get()))?;
    let DocParam::Keyframes(track) = &envelope.transform.position else {
        return Err(CommandError::UnsupportedPositionKeySource);
    };
    let left_index = track
        .keys()
        .iter()
        .position(|key| key.id == left_key_id)
        .ok_or(CommandError::PositionKeyNotFound {
            layer: target.get(),
            key: left_key_id.get(),
        })?;
    let actual_right = track.keys().get(left_index + 1).map(|key| key.id);
    if actual_right != Some(right_key_id) {
        return Err(CommandError::PositionIntervalMismatch {
            layer: target.get(),
            left: left_key_id.get(),
            expected_right: right_key_id.get(),
            found_right: actual_right.map(KeyframeId::get),
        });
    }
    let actual_old = track.keys()[left_index].interp;
    if actual_old != old {
        return Err(CommandError::PositionKeyInterpPayloadMismatch {
            layer: target.get(),
            key: left_key_id.get(),
        });
    }
    let mut next = doc.clone();
    let next_envelope = crate::command::find_envelope_mut(&mut next, target)?;
    let DocParam::Keyframes(next_track) = &mut next_envelope.transform.position else {
        return Err(CommandError::UnsupportedPositionKeySource);
    };
    next_track
        .get_by_id_mut(left_key_id)
        .ok_or(CommandError::PositionKeyNotFound {
            layer: target.get(),
            key: left_key_id.get(),
        })?
        .interp = new;
    next.validate().map_err(CommandError::Validate)?;
    *doc = next;
    Ok(())
}

fn position_interval(
    track: &DocKeyframeTrack,
    target: LayerId,
    left_key_id: KeyframeId,
) -> Result<(usize, KeyframeId, Interp), CommandError> {
    let left_index = track
        .keys()
        .iter()
        .position(|key| key.id == left_key_id)
        .ok_or(CommandError::PositionKeyNotFound {
            layer: target.get(),
            key: left_key_id.get(),
        })?;
    let right =
        track
            .keys()
            .get(left_index + 1)
            .ok_or(CommandError::PositionKeyHasNoRightInterval {
                layer: target.get(),
                key: left_key_id.get(),
            })?;
    Ok((left_index, right.id, track.keys()[left_index].interp))
}

fn existing_key_at(
    value: &DocParam,
    playhead: RationalTime,
) -> Result<Option<KeyframeId>, CommandError> {
    match value {
        DocParam::Const(DocValue::Vec2(_)) => Ok(None),
        DocParam::Const(_) => Err(CommandError::PositionValueNotVec2),
        DocParam::Keyframes(track) => {
            validate_vec2_track(track)?;
            Ok(track
                .keys()
                .iter()
                .find(|key| key.t == playhead)
                .map(|key| key.id))
        }
        _ => Err(CommandError::UnsupportedPositionKeySource),
    }
}

fn build_position_key_value(
    old_value: &DocParam,
    playhead: RationalTime,
    key_id: KeyframeId,
    target: LayerId,
) -> Result<BuildPositionKey, CommandError> {
    match old_value {
        DocParam::Const(DocValue::Vec2(value)) => {
            let mut track = DocKeyframeTrack::new();
            track.insert(DocKeyframe {
                id: key_id,
                t: playhead,
                value: DocValue::Vec2(*value),
                interp: Interp::Linear,
            });
            Ok(BuildPositionKey::Added(DocParam::Keyframes(track)))
        }
        DocParam::Const(_) => Err(CommandError::PositionValueNotVec2),
        DocParam::Keyframes(track) => build_from_track(track, playhead, key_id, target),
        _ => Err(CommandError::UnsupportedPositionKeySource),
    }
}

fn build_from_track(
    track: &DocKeyframeTrack,
    playhead: RationalTime,
    key_id: KeyframeId,
    target: LayerId,
) -> Result<BuildPositionKey, CommandError> {
    validate_vec2_track(track)?;
    if track.keys().iter().any(|key| key.t == playhead) {
        return Ok(BuildPositionKey::AlreadyPresent);
    }
    let keys = track.keys();
    let insert_index = keys.partition_point(|key| key.t < playhead);
    let value = match track.eval(playhead) {
        EvalValue::Vec2(value) => DocValue::Vec2(value),
        _ => return Err(CommandError::PositionValueNotVec2),
    };
    let mut rebuilt = DocKeyframeTrack::new();

    if insert_index == 0 {
        rebuilt.insert(DocKeyframe {
            id: key_id,
            t: playhead,
            value,
            interp: Interp::Hold,
        });
        for key in keys {
            rebuilt.insert(key.clone());
        }
        return Ok(BuildPositionKey::Added(DocParam::Keyframes(rebuilt)));
    }

    if insert_index == keys.len() {
        for (index, key) in keys.iter().enumerate() {
            let mut key = key.clone();
            if index + 1 == keys.len() {
                key.interp = Interp::Hold;
            }
            rebuilt.insert(key);
        }
        rebuilt.insert(DocKeyframe {
            id: key_id,
            t: playhead,
            value,
            interp: Interp::Linear,
        });
        return Ok(BuildPositionKey::Added(DocParam::Keyframes(rebuilt)));
    }

    let left = &keys[insert_index - 1];
    let right = &keys[insert_index];
    let progress = segment_progress(left.t, right.t, playhead)?;
    let (left_interp, right_interp) = left
        .interp
        .split_at_progress(progress)
        .map_err(|error| map_split_error(target, error))?;
    for (index, key) in keys.iter().enumerate() {
        let mut key = key.clone();
        if index + 1 == insert_index {
            key.interp = left_interp;
        }
        rebuilt.insert(key);
        if index + 1 == insert_index {
            rebuilt.insert(DocKeyframe {
                id: key_id,
                t: playhead,
                value: value.clone(),
                interp: right_interp,
            });
        }
    }
    Ok(BuildPositionKey::Added(DocParam::Keyframes(rebuilt)))
}

fn validate_vec2_track(track: &DocKeyframeTrack) -> Result<(), CommandError> {
    if track.keys().is_empty() {
        return Err(CommandError::EmptyPositionKeyTrack);
    }
    track
        .validate()
        .map_err(|_| CommandError::InvalidPositionKeyTrack)?;
    if track
        .keys()
        .iter()
        .any(|key| !matches!(key.value, DocValue::Vec2(_)))
    {
        return Err(CommandError::PositionValueNotVec2);
    }
    Ok(())
}

fn segment_progress(
    start: RationalTime,
    end: RationalTime,
    time: RationalTime,
) -> Result<f64, CommandError> {
    let duration = end
        .try_sub(start)
        .map_err(|_| CommandError::PositionCurveSplit)?;
    let offset = time
        .try_sub(start)
        .map_err(|_| CommandError::PositionCurveSplit)?;
    let progress = offset.as_seconds_f64() / duration.as_seconds_f64();
    if progress.is_finite() && 0.0 < progress && progress < 1.0 {
        Ok(progress)
    } else {
        Err(CommandError::PositionCurveSplit)
    }
}

fn map_split_error(_target: LayerId, _error: TrackError) -> CommandError {
    CommandError::PositionCurveSplit
}
