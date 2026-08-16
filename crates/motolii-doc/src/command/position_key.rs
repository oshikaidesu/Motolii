use motolii_core::RationalTime;

use crate::doc_keyframe::{validate_interp, DocKeyframe};
use crate::doc_value::DocValue;
use crate::param::DocParam;
use crate::position_key_prepare::{
    deterministic_new_position, deterministic_remove_position, transform_param, transform_param_mut,
};
use crate::stable_id::{KeyframeId, StableIdReservation};
use crate::{Document, LayerId};

use super::ids::ScalarPropertyId;
use super::locate::{find_envelope, find_envelope_mut};
use super::reservation::{
    apply_reservation_commit, swap_if_valid, validate_reservation_for_apply,
    validate_reservation_for_undo,
};
use super::{Command, CommandError};

/// Position key の outgoing interpolation だけを置換する command を構築する。
pub(crate) fn prepare_set_position_key_interp(
    doc: &Document,
    target: LayerId,
    key: KeyframeId,
    new: motolii_eval::Interp,
) -> Result<Option<Command>, CommandError> {
    let old = position_key_interp_for_command(doc, target, key, None, &new)?;
    if new == old {
        return Ok(None);
    }
    Ok(Some(Command::SetPositionKeyInterp {
        target,
        key,
        old,
        new,
    }))
}

/// Position key の Vec2 value だけを置換する command を構築する。
pub(crate) fn prepare_set_position_key_value(
    doc: &Document,
    target: LayerId,
    key: KeyframeId,
    new: [f64; 2],
) -> Result<Option<Command>, CommandError> {
    let old = position_key_value_for_command(doc, target, key, None, new)?;
    if new == old {
        return Ok(None);
    }
    Ok(Some(Command::SetPositionKeyValue {
        target,
        key,
        old,
        new,
    }))
}

/// Position key の時刻だけを移す command を構築する。same-time は `None`。
pub(crate) fn prepare_set_position_key_time(
    doc: &Document,
    target: LayerId,
    key: KeyframeId,
    new: RationalTime,
) -> Result<Option<Command>, CommandError> {
    let old = position_key_time_for_command(doc, target, key, None, new)?;
    if new == old {
        return Ok(None);
    }
    Ok(Some(Command::SetPositionKeyTime {
        target,
        key,
        old,
        new,
    }))
}

pub(super) fn apply_add_position_key(
    doc: &mut Document,
    target: LayerId,
    old_value: DocParam,
    new_value: DocParam,
    added_key_id: KeyframeId,
    stable_id_reservation: StableIdReservation,
) -> Result<(), CommandError> {
    let introduced = [added_key_id.get()];
    let commit = validate_reservation_for_apply(doc, stable_id_reservation, &introduced)?;
    validate_add_position_key_payload(
        doc,
        target,
        &old_value,
        &old_value,
        &new_value,
        added_key_id,
    )?;

    let mut next = doc.clone();
    find_envelope_mut(&mut next, target)?.transform.position = new_value;
    apply_reservation_commit(&mut next, commit);
    swap_if_valid(doc, next)
}

pub(super) fn undo_add_position_key(
    doc: &mut Document,
    target: LayerId,
    old_value: DocParam,
    new_value: DocParam,
    added_key_id: KeyframeId,
    stable_id_reservation: StableIdReservation,
) -> Result<(), CommandError> {
    let introduced = [added_key_id.get()];
    validate_reservation_for_undo(doc, stable_id_reservation, &introduced)?;
    validate_add_position_key_payload(
        doc,
        target,
        &new_value,
        &old_value,
        &new_value,
        added_key_id,
    )?;

    let mut next = doc.clone();
    find_envelope_mut(&mut next, target)?.transform.position = old_value;
    swap_if_valid(doc, next)
}

/// RemoveはAddの逆向き: forwardでkeyを外し、undoで同一KeyframeIdを戻す。
pub(super) fn apply_remove_position_key(
    doc: &mut Document,
    target: LayerId,
    old_value: DocParam,
    new_value: DocParam,
    removed_key_id: KeyframeId,
    stable_id_reservation: StableIdReservation,
) -> Result<(), CommandError> {
    let introduced = [removed_key_id.get()];
    validate_reservation_for_undo(doc, stable_id_reservation, &introduced)?;
    validate_remove_position_key_payload(
        doc,
        target,
        &old_value,
        &old_value,
        &new_value,
        removed_key_id,
    )?;

    let mut next = doc.clone();
    find_envelope_mut(&mut next, target)?.transform.position = new_value;
    swap_if_valid(doc, next)
}

pub(super) fn undo_remove_position_key(
    doc: &mut Document,
    target: LayerId,
    old_value: DocParam,
    new_value: DocParam,
    removed_key_id: KeyframeId,
    stable_id_reservation: StableIdReservation,
) -> Result<(), CommandError> {
    let introduced = [removed_key_id.get()];
    let commit = validate_reservation_for_apply(doc, stable_id_reservation, &introduced)?;
    validate_remove_position_key_payload(
        doc,
        target,
        &new_value,
        &old_value,
        &new_value,
        removed_key_id,
    )?;

    let mut next = doc.clone();
    find_envelope_mut(&mut next, target)?.transform.position = old_value;
    apply_reservation_commit(&mut next, commit);
    swap_if_valid(doc, next)
}

pub(super) fn apply_set_position_key_interp(
    doc: &mut Document,
    target: LayerId,
    key: KeyframeId,
    old: &motolii_eval::Interp,
    new: &motolii_eval::Interp,
) -> Result<(), CommandError> {
    position_key_interp_for_command(doc, target, key, Some(old), new)?;

    let mut next = doc.clone();
    let env = find_envelope_mut(&mut next, target)?;
    let layer = target.get();
    let DocParam::Keyframes(track) = &mut env.transform.position else {
        return Err(CommandError::PositionKeyInterpSourceUnsupported { layer });
    };
    let mut replacement = track
        .get_by_id(key)
        .ok_or(CommandError::PositionKeyNotFound {
            layer,
            key_id: key.get(),
        })?
        .clone();
    replacement.interp = *new;
    track.insert(replacement);
    swap_if_valid(doc, next)
}

pub(super) fn apply_set_position_key_value(
    doc: &mut Document,
    target: LayerId,
    key: KeyframeId,
    old: [f64; 2],
    new: [f64; 2],
) -> Result<(), CommandError> {
    position_key_value_for_command(doc, target, key, Some(old), new)?;
    if old == new {
        return Ok(());
    }

    let mut next = doc.clone();
    let env = find_envelope_mut(&mut next, target)?;
    let layer = target.get();
    let DocParam::Keyframes(track) = &mut env.transform.position else {
        return Err(CommandError::PositionKeyValueSourceUnsupported { layer });
    };
    let mut replacement = track
        .get_by_id(key)
        .ok_or(CommandError::PositionKeyValueNotFound {
            layer,
            key_id: key.get(),
        })?
        .clone();
    replacement.value = DocValue::Vec2(new);
    track.insert(replacement);
    swap_if_valid(doc, next)
}

pub(super) fn apply_set_position_key_time(
    doc: &mut Document,
    target: LayerId,
    key: KeyframeId,
    old: RationalTime,
    new: RationalTime,
) -> Result<(), CommandError> {
    position_key_time_for_command(doc, target, key, Some(old), new)?;
    if old == new {
        return Ok(());
    }

    let mut next = doc.clone();
    let env = find_envelope_mut(&mut next, target)?;
    let layer = target.get();
    let DocParam::Keyframes(track) = &mut env.transform.position else {
        return Err(CommandError::PositionKeyTimeSourceUnsupported { layer });
    };
    let removed = track
        .remove_by_id(key)
        .ok_or(CommandError::PositionKeyTimeNotFound {
            layer,
            key_id: key.get(),
        })?;
    track.insert(DocKeyframe {
        id: removed.id,
        t: new,
        value: removed.value,
        interp: removed.interp,
    });
    swap_if_valid(doc, next)
}

pub(super) fn apply_set_transform_param_key_time(
    doc: &mut Document,
    target: LayerId,
    property: &ScalarPropertyId,
    key: KeyframeId,
    old: RationalTime,
    new: RationalTime,
) -> Result<(), CommandError> {
    transform_param_key_time_for_command(doc, target, property, key, Some(old), new)?;
    if old == new {
        return Ok(());
    }

    let mut next = doc.clone();
    let layer = target.get();
    let env = find_envelope_mut(&mut next, target)?;
    let param = transform_param_mut(env, property)
        .ok_or(CommandError::TransformParamKeyTimeSourceUnsupported { layer })?;
    let DocParam::Keyframes(track) = param else {
        return Err(CommandError::TransformParamKeyTimeSourceUnsupported { layer });
    };
    let removed = track
        .remove_by_id(key)
        .ok_or(CommandError::TransformParamKeyTimeNotFound {
            layer,
            key_id: key.get(),
        })?;
    track.insert(DocKeyframe {
        id: removed.id,
        t: new,
        value: removed.value,
        interp: removed.interp,
    });
    swap_if_valid(doc, next)
}

/// `SetTransformParamKeyTime` の CAS/時刻検証。`position_key_time_for_command` の鏡映で、
/// 対象paramだけが `transform_param` 経由に替わる。
pub(crate) fn transform_param_key_time_for_command(
    doc: &Document,
    target: LayerId,
    property: &ScalarPropertyId,
    key: KeyframeId,
    expected_old: Option<RationalTime>,
    new: RationalTime,
) -> Result<RationalTime, CommandError> {
    let layer = target.get();
    if new < RationalTime::ZERO {
        return Err(CommandError::TransformParamKeyTimeNegative {
            layer,
            key_id: key.get(),
        });
    }
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    let param = transform_param(env, property)
        .ok_or(CommandError::TransformParamKeyTimeSourceUnsupported { layer })?;
    let DocParam::Keyframes(track) = param else {
        return Err(CommandError::TransformParamKeyTimeSourceUnsupported { layer });
    };
    if track.keys().is_empty() {
        return Err(CommandError::TransformParamKeyTimeSourceUnsupported { layer });
    }
    let current = track
        .get_by_id(key)
        .ok_or(CommandError::TransformParamKeyTimeNotFound {
            layer,
            key_id: key.get(),
        })?;
    if expected_old.is_some_and(|old| old != current.t) {
        return Err(CommandError::TransformParamKeyTimePayloadMismatch {
            layer,
            key_id: key.get(),
        });
    }
    if new != current.t
        && track
            .keys()
            .iter()
            .any(|candidate| candidate.id != key && candidate.t == new)
    {
        return Err(CommandError::TransformParamKeyTimeOccupied {
            layer,
            key_id: key.get(),
        });
    }
    Ok(current.t)
}

fn position_key_time_for_command(
    doc: &Document,
    target: LayerId,
    key: KeyframeId,
    expected_old: Option<RationalTime>,
    new: RationalTime,
) -> Result<RationalTime, CommandError> {
    let layer = target.get();
    if new < RationalTime::ZERO {
        return Err(CommandError::PositionKeyTimeNegative {
            layer,
            key_id: key.get(),
        });
    }
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    let DocParam::Keyframes(track) = &env.transform.position else {
        return Err(CommandError::PositionKeyTimeSourceUnsupported { layer });
    };
    if track.keys().is_empty() {
        return Err(CommandError::PositionKeyTimeSourceUnsupported { layer });
    }
    let current = track
        .get_by_id(key)
        .ok_or(CommandError::PositionKeyTimeNotFound {
            layer,
            key_id: key.get(),
        })?;
    if expected_old.is_some_and(|old| old != current.t) {
        return Err(CommandError::PositionKeyTimePayloadMismatch {
            layer,
            key_id: key.get(),
        });
    }
    if new != current.t
        && track
            .keys()
            .iter()
            .any(|candidate| candidate.id != key && candidate.t == new)
    {
        return Err(CommandError::PositionKeyTimeOccupied {
            layer,
            key_id: key.get(),
        });
    }
    Ok(current.t)
}

fn position_key_value_for_command(
    doc: &Document,
    target: LayerId,
    key: KeyframeId,
    expected_old: Option<[f64; 2]>,
    new: [f64; 2],
) -> Result<[f64; 2], CommandError> {
    let layer = target.get();
    if !new.iter().all(|value| value.is_finite()) {
        return Err(CommandError::PositionKeyValueNonFinite {
            layer,
            key_id: key.get(),
        });
    }
    if let Some(old) = expected_old {
        if !old.iter().all(|value| value.is_finite()) {
            return Err(CommandError::PositionKeyValueNonFinite {
                layer,
                key_id: key.get(),
            });
        }
    }

    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    let DocParam::Keyframes(track) = &env.transform.position else {
        return Err(CommandError::PositionKeyValueSourceUnsupported { layer });
    };
    if track.keys().is_empty() {
        return Err(CommandError::PositionKeyValueSourceUnsupported { layer });
    }
    if track
        .keys()
        .iter()
        .any(|candidate| !matches!(candidate.value, DocValue::Vec2(_)))
    {
        return Err(CommandError::PositionKeyValueTypeMismatch { layer });
    }
    let current = track
        .get_by_id(key)
        .ok_or(CommandError::PositionKeyValueNotFound {
            layer,
            key_id: key.get(),
        })?;
    let DocValue::Vec2(current_value) = current.value else {
        return Err(CommandError::PositionKeyValueTypeMismatch { layer });
    };
    if !current_value.iter().all(|value| value.is_finite()) {
        return Err(CommandError::PositionKeyValueNonFinite {
            layer,
            key_id: key.get(),
        });
    }
    if expected_old.is_some_and(|old| old != current_value) {
        return Err(CommandError::PositionKeyValuePayloadMismatch {
            layer,
            key_id: key.get(),
        });
    }
    Ok(current_value)
}

fn position_key_interp_for_command(
    doc: &Document,
    target: LayerId,
    key: KeyframeId,
    expected_old: Option<&motolii_eval::Interp>,
    new: &motolii_eval::Interp,
) -> Result<motolii_eval::Interp, CommandError> {
    let layer = target.get();
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    let DocParam::Keyframes(track) = &env.transform.position else {
        return Err(CommandError::PositionKeyInterpSourceUnsupported { layer });
    };
    if track
        .keys()
        .iter()
        .any(|candidate| !matches!(candidate.value, DocValue::Vec2(_)))
    {
        return Err(CommandError::PositionKeyInterpValueTypeMismatch { layer });
    }
    let current = track
        .get_by_id(key)
        .ok_or(CommandError::PositionKeyNotFound {
            layer,
            key_id: key.get(),
        })?;
    let old = expected_old.unwrap_or(&current.interp);
    validate_interp(old).map_err(|source| CommandError::PositionKeyInterpInvalid {
        layer,
        key_id: key.get(),
        source,
    })?;
    validate_interp(new).map_err(|source| CommandError::PositionKeyInterpInvalid {
        layer,
        key_id: key.get(),
        source,
    })?;
    if current.interp != *old {
        return Err(CommandError::PositionKeyInterpPayloadMismatch {
            layer,
            key_id: key.get(),
        });
    }
    Ok(current.interp)
}

fn validate_add_position_key_payload(
    doc: &Document,
    target: LayerId,
    expected_current_position: &DocParam,
    old_value: &DocParam,
    new_value: &DocParam,
    added_key_id: KeyframeId,
) -> Result<RationalTime, CommandError> {
    let layer = target.get();
    let mismatch = || CommandError::AddPositionKeyPayloadMismatch {
        layer,
        key_id: added_key_id.get(),
    };
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    if env.transform.position != *expected_current_position {
        return Err(mismatch());
    }
    if let DocParam::Keyframes(track) = old_value {
        if track.get_by_id(added_key_id).is_some() {
            return Err(mismatch());
        }
    }

    let DocParam::Keyframes(track) = new_value else {
        return Err(mismatch());
    };
    let mut matches = track.keys().iter().filter(|key| key.id == added_key_id);
    let t = matches.next().map(|key| key.t).ok_or_else(mismatch)?;
    if matches.next().is_some() {
        return Err(mismatch());
    }

    let expected =
        deterministic_new_position(old_value, target, t, added_key_id).map_err(|_| mismatch())?;
    if expected != *new_value {
        return Err(mismatch());
    }
    Ok(t)
}

fn validate_remove_position_key_payload(
    doc: &Document,
    target: LayerId,
    expected_current_position: &DocParam,
    old_value: &DocParam,
    new_value: &DocParam,
    removed_key_id: KeyframeId,
) -> Result<(), CommandError> {
    let layer = target.get();
    let mismatch = || CommandError::RemovePositionKeyPayloadMismatch {
        layer,
        key_id: removed_key_id.get(),
    };
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    if env.transform.position != *expected_current_position {
        return Err(mismatch());
    }
    let DocParam::Keyframes(old_track) = old_value else {
        return Err(mismatch());
    };
    if old_track.get_by_id(removed_key_id).is_none() {
        return Err(mismatch());
    }
    if let DocParam::Keyframes(track) = new_value {
        if track.get_by_id(removed_key_id).is_some() {
            return Err(mismatch());
        }
    }

    let expected =
        deterministic_remove_position(old_value, target, removed_key_id).map_err(|_| mismatch())?;
    if expected != *new_value {
        return Err(mismatch());
    }
    Ok(())
}

#[cfg(test)]
#[path = "position_key_time_tests.rs"]
mod set_position_key_time_tests;

#[cfg(test)]
#[path = "transform_param_key_time_tests.rs"]
mod set_transform_param_key_time_tests;
