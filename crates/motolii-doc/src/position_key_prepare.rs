use motolii_core::RationalTime;
use motolii_eval::{Interp, TrackError, Value};
use thiserror::Error;

use crate::command::{find_envelope, Command, CommandError};
use crate::doc_keyframe::{DocKeyframe, DocKeyframeError, DocKeyframeTrack};
use crate::doc_value::DocValue;
use crate::param::DocParam;
use crate::stable_id::{KeyframeId, StableIdError, StableIdReservation};
use crate::{Document, LayerId};

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // 閉じた公開契約がCommand値保持を要求するため、Box化でshapeを変えない。
pub enum AddPositionKeyPreparation {
    Prepared {
        key_id: KeyframeId,
        command: Command,
    },
    AlreadyPresent {
        key_id: KeyframeId,
    },
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum AddPositionKeyPrepareError {
    #[error(transparent)]
    Command(#[from] CommandError),
    #[error(transparent)]
    StableId(#[from] StableIdError),
    #[error(transparent)]
    Keyframe(#[from] DocKeyframeError),
    #[error(transparent)]
    Track(#[from] TrackError),
    #[error("position source unsupported for layer {layer}")]
    PositionSourceUnsupported { layer: u64 },
    #[error("position value type mismatch for layer {layer}")]
    PositionValueTypeMismatch { layer: u64 },
}

pub fn prepare_add_position_key(
    doc: &Document,
    target: LayerId,
    t: RationalTime,
) -> Result<AddPositionKeyPreparation, AddPositionKeyPrepareError> {
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(target.get()))?;
    let old_value = env.transform.position.clone();
    match &old_value {
        DocParam::Const(DocValue::Vec2(_)) => {}
        DocParam::Const(_) => {
            return Err(AddPositionKeyPrepareError::PositionValueTypeMismatch {
                layer: target.get(),
            });
        }
        DocParam::Keyframes(track) => {
            if track.keys().is_empty() {
                return Err(AddPositionKeyPrepareError::PositionSourceUnsupported {
                    layer: target.get(),
                });
            }
            for key in track.keys() {
                if !matches!(key.value, DocValue::Vec2(_)) {
                    return Err(AddPositionKeyPrepareError::PositionValueTypeMismatch {
                        layer: target.get(),
                    });
                }
            }
            if let Some(found) = track.keys().iter().find(|key| key.t == t) {
                return Ok(AddPositionKeyPreparation::AlreadyPresent { key_id: found.id });
            }
        }
        DocParam::Data { .. }
        | DocParam::Vec2Axes { .. }
        | DocParam::LookAt { .. }
        | DocParam::Follow { .. } => {
            return Err(AddPositionKeyPrepareError::PositionSourceUnsupported {
                layer: target.get(),
            });
        }
    }

    let before = doc.next_stable_id.peek_next();
    let mut next = doc.next_stable_id;
    let key_id = KeyframeId::from_raw(next.allocate()?);
    let stable_id_reservation = StableIdReservation::new(before, next.peek_next());
    let new_value = deterministic_new_position(&old_value, target, t, key_id)?;
    Ok(AddPositionKeyPreparation::Prepared {
        key_id,
        command: Command::AddPositionKey {
            target,
            old_value,
            new_value,
            added_key_id: key_id,
            stable_id_reservation,
        },
    })
}

pub(crate) fn deterministic_new_position(
    old_value: &DocParam,
    target: LayerId,
    t: RationalTime,
    id: KeyframeId,
) -> Result<DocParam, AddPositionKeyPrepareError> {
    let layer = target.get();

    match old_value {
        DocParam::Const(DocValue::Vec2(value)) => {
            let mut track = DocKeyframeTrack::new();
            track.insert(DocKeyframe {
                id,
                t,
                value: DocValue::Vec2(*value),
                interp: Interp::Linear,
            });
            track.validate()?;
            Ok(DocParam::Keyframes(track))
        }
        DocParam::Keyframes(track) => {
            if track.keys().is_empty() {
                return Err(AddPositionKeyPrepareError::PositionSourceUnsupported { layer });
            }
            if track.keys().iter().any(|key| key.t == t) {
                return Err(AddPositionKeyPrepareError::Keyframe(
                    DocKeyframeError::UnsortedOrDuplicateKeys,
                ));
            }

            let mut new_track = track.clone();
            let keys = new_track.keys();
            for key in keys {
                if !matches!(key.value, DocValue::Vec2(_)) {
                    return Err(AddPositionKeyPrepareError::PositionValueTypeMismatch { layer });
                }
            }

            let first = &keys[0];
            let last = &keys[keys.len() - 1];

            if t <= first.t {
                let value = vec2_position_value(&first.value, layer)?;
                new_track.insert(DocKeyframe {
                    id,
                    t,
                    value: DocValue::Vec2(value),
                    interp: Interp::Linear,
                });
                new_track.validate()?;
                return Ok(DocParam::Keyframes(new_track));
            }

            if t >= last.t {
                let value = vec2_position_value(&last.value, layer)?;
                new_track.insert(DocKeyframe {
                    id,
                    t,
                    value: DocValue::Vec2(value),
                    interp: Interp::Linear,
                });
                new_track.validate()?;
                return Ok(DocParam::Keyframes(new_track));
            }

            let right_index = keys.iter().position(|key| t < key.t).ok_or(
                AddPositionKeyPrepareError::Keyframe(DocKeyframeError::UnsortedOrDuplicateKeys),
            )?;
            if right_index == 0 {
                return Err(AddPositionKeyPrepareError::Track(
                    TrackError::UnsortedOrDuplicateKeys,
                ));
            }
            let left_index = right_index - 1;
            let left = &keys[left_index];
            let right = &keys[right_index];

            let left_value = vec2_position_value(&left.value, layer)?;
            let right_value = vec2_position_value(&right.value, layer)?;

            let (left_interp, inserted_interp, value) = if left_value == right_value {
                (Interp::Hold, Interp::Hold, left_value)
            } else {
                match left.interp {
                    Interp::Hold => (Interp::Hold, Interp::Hold, left_value),
                    Interp::Linear => {
                        let value = eval_position(track, t, layer)?;
                        (Interp::Linear, Interp::Linear, value)
                    }
                    Interp::Bezier { .. } => {
                        let value = eval_position(track, t, layer)?;
                        let progress = split_progress(left.t, right.t, t);
                        let (left_interp, inserted_interp) = left.interp.split_at(progress)?;
                        (left_interp, inserted_interp, value)
                    }
                }
            };

            new_track.insert(DocKeyframe {
                id: left.id,
                t: left.t,
                value: DocValue::Vec2(left_value),
                interp: left_interp,
            });

            let inserted = DocKeyframe {
                id,
                t,
                value: DocValue::Vec2(value),
                interp: inserted_interp,
            };
            new_track.insert(inserted);
            new_track.validate()?;
            Ok(DocParam::Keyframes(new_track))
        }
        _ => Err(AddPositionKeyPrepareError::PositionValueTypeMismatch { layer }),
    }
}

fn split_progress(a: RationalTime, b: RationalTime, t: RationalTime) -> f64 {
    let progress_num = match t.try_sub(a) {
        Ok(delta) => delta.as_seconds_f64(),
        Err(_) => t.as_seconds_f64() - a.as_seconds_f64(),
    };
    let progress_den = match b.try_sub(a) {
        Ok(delta) => delta.as_seconds_f64(),
        Err(_) => b.as_seconds_f64() - a.as_seconds_f64(),
    };
    progress_num / progress_den
}

fn eval_position(
    track: &DocKeyframeTrack,
    t: RationalTime,
    layer: u64,
) -> Result<[f64; 2], AddPositionKeyPrepareError> {
    match track.eval(t) {
        Value::Vec2(v) => Ok(v),
        _ => Err(AddPositionKeyPrepareError::PositionValueTypeMismatch { layer }),
    }
}

fn vec2_position_value(
    value: &DocValue,
    layer: u64,
) -> Result<[f64; 2], AddPositionKeyPrepareError> {
    match value {
        DocValue::Vec2(value) => Ok(*value),
        _ => Err(AddPositionKeyPrepareError::PositionValueTypeMismatch { layer }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{find_envelope_mut, CommandError as CmdErr};
    use crate::{Clip, ClipSource, ItemEnvelope, Track, TrackItem};

    fn minimal_doc(position: DocParam) -> (Document, LayerId) {
        let mut doc = Document::new_current();
        let layer = doc.layers.allocate("target").unwrap();
        let track_id = doc.track_ids.allocate("V1").unwrap();
        let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();

        if let DocParam::Keyframes(track) = &position {
            let mut max_id = doc.next_stable_id.peek_next();
            for key in track.keys() {
                let id = key.id.get();
                if id >= max_id {
                    max_id = id + 1;
                }
            }
            while doc.next_stable_id.peek_next() < max_id {
                doc.next_stable_id.allocate().unwrap();
            }
        }

        let mut envelope = ItemEnvelope::new(layer);
        envelope.transform.position = position;
        doc.tracks.push(Track {
            id: track_id,
            items: vec![TrackItem::Clip(Clip {
                envelope,
                start: RationalTime::ZERO,
                duration: RationalTime::from_seconds(10),
                time_map: Default::default(),
                source: ClipSource::asset_video_only(asset),
            })],
        });

        doc.validate().unwrap();
        (doc, layer)
    }

    fn keyframe(id: u64, t: RationalTime, value: [f64; 2], interp: Interp) -> DocKeyframe {
        DocKeyframe {
            id: KeyframeId::from_raw(id),
            t,
            value: DocValue::Vec2(value),
            interp,
        }
    }

    fn keyframe_track(keys: Vec<(u64, RationalTime, [f64; 2], Interp)>) -> DocParam {
        let mut track = DocKeyframeTrack::new();
        for (id, t, value, interp) in keys {
            track.insert(keyframe(id, t, value, interp));
        }
        DocParam::Keyframes(track)
    }

    fn as_vec2(value: &Value) -> [f64; 2] {
        match value {
            Value::Vec2(v) => *v,
            _ => panic!("expected Vec2 value"),
        }
    }

    fn set_position(doc: &mut Document, layer: LayerId, value: DocParam) {
        find_envelope_mut(doc, layer).unwrap().transform.position = value;
    }

    #[test]
    fn const_vec2_prepared_command_reserves_once_and_keeps_counter_for_undo_id() {
        let (doc, target) = minimal_doc(DocParam::const_vec2([1.0, 2.0]));
        let t = RationalTime::try_new(2, 1).unwrap();
        let before = doc.next_stable_id.peek_next();

        let prepared = prepare_add_position_key(&doc, target, t).unwrap();
        let (key_id, command) = match prepared {
            AddPositionKeyPreparation::Prepared { key_id, command } => (key_id, command),
            AddPositionKeyPreparation::AlreadyPresent { .. } => {
                panic!("const source should not already exist")
            }
        };

        assert_eq!(doc.next_stable_id.peek_next(), before);
        let Command::AddPositionKey {
            added_key_id,
            old_value,
            stable_id_reservation,
            ..
        } = &command
        else {
            panic!("unexpected command")
        };

        assert_eq!(*added_key_id, key_id);
        assert_eq!(*added_key_id, KeyframeId::from_raw(before));
        assert_eq!(stable_id_reservation.before(), before);
        assert_eq!(stable_id_reservation.after(), before + 1);
        assert_eq!(old_value, &DocParam::const_vec2([1.0, 2.0]));

        let mut live = doc.clone();
        command.apply(&mut live).unwrap();
        assert_eq!(live.next_stable_id.peek_next(), before + 1);

        let inverse = command.inverse();
        let Command::UndoAddPositionKey {
            added_key_id: inverse_id,
            ..
        } = inverse
        else {
            panic!("unexpected inverse command")
        };
        assert_eq!(inverse_id, key_id);

        inverse.apply(&mut live).unwrap();
        assert_eq!(live.next_stable_id.peek_next(), before + 1);

        let redo = inverse.inverse();
        let Command::AddPositionKey {
            added_key_id: redo_id,
            ..
        } = redo
        else {
            panic!("unexpected redo command")
        };
        assert_eq!(redo_id, key_id);

        redo.apply(&mut live).unwrap();
        assert_eq!(live.next_stable_id.peek_next(), before + 1);
    }

    #[test]
    fn prepare_add_position_key_returns_already_present_for_exact_same_time() {
        let mut doc = Document::new_current();
        let layer = doc.layers.allocate("target").unwrap();
        let _ = doc.track_ids.allocate("V1").unwrap();
        let _ = doc.assets.allocate("media", "video/mp4", "hash").unwrap();

        let a = doc.next_stable_id.allocate().unwrap();
        let b = doc.next_stable_id.allocate().unwrap();
        let same_time = RationalTime::from_seconds(1);
        let track = keyframe_track(vec![
            (a, RationalTime::ZERO, [0.0, 0.0], Interp::Hold),
            (b, same_time, [5.0, 5.0], Interp::Linear),
        ]);

        doc.tracks.push(Track {
            id: doc.track_ids.allocate("V2").unwrap(),
            items: vec![TrackItem::Clip(Clip {
                envelope: {
                    let mut envelope = ItemEnvelope::new(layer);
                    envelope.transform.position = track;
                    envelope
                },
                start: RationalTime::ZERO,
                duration: RationalTime::from_seconds(10),
                time_map: Default::default(),
                source: ClipSource::asset_video_only(
                    doc.assets.allocate("media2", "video/mp4", "hash2").unwrap(),
                ),
            })],
        });

        doc.validate().unwrap();
        let before = doc.next_stable_id.peek_next();

        let result = prepare_add_position_key(&doc, layer, same_time).unwrap();
        let found = match result {
            AddPositionKeyPreparation::AlreadyPresent { key_id } => key_id,
            AddPositionKeyPreparation::Prepared { .. } => panic!("expected already-present key"),
        };

        assert_eq!(found, KeyframeId::from_raw(b));
        assert_eq!(doc.next_stable_id.peek_next(), before);
    }

    #[test]
    fn interior_hold_linear_and_equal_endpoint_values_normalize_hold() {
        let track = keyframe_track(vec![
            (0, RationalTime::ZERO, [1.0, 1.0], Interp::Hold),
            (1, RationalTime::from_seconds(1), [2.0, 2.0], Interp::Linear),
        ]);
        let (doc_hold, target) = minimal_doc(track);
        let t = RationalTime::try_new(1, 2).unwrap();

        let cmd_hold = match prepare_add_position_key(&doc_hold, target, t).unwrap() {
            AddPositionKeyPreparation::Prepared { command, .. } => command,
            AddPositionKeyPreparation::AlreadyPresent { .. } => {
                panic!("interior keyframe should prepare")
            }
        };
        let Command::AddPositionKey {
            new_value: DocParam::Keyframes(track_hold),
            ..
        } = cmd_hold
        else {
            panic!("expected keyframe payload");
        };
        let hold_left = track_hold.keys()[0].t;
        let inserted = track_hold.keys()[1].t;
        assert_eq!(hold_left, RationalTime::ZERO);
        assert_eq!(inserted, t);
        assert_eq!(track_hold.keys()[1].interp, Interp::Hold);
        assert_eq!(track_hold.keys()[0].interp, Interp::Hold);

        let track = keyframe_track(vec![
            (2, RationalTime::ZERO, [1.0, 1.0], Interp::Linear),
            (3, RationalTime::from_seconds(1), [2.0, 2.0], Interp::Linear),
        ]);
        let (doc_linear, target_linear) = minimal_doc(track);
        let cmd_linear = match prepare_add_position_key(&doc_linear, target_linear, t).unwrap() {
            AddPositionKeyPreparation::Prepared { command, .. } => command,
            AddPositionKeyPreparation::AlreadyPresent { .. } => {
                panic!("interior keyframe should prepare")
            }
        };
        let Command::AddPositionKey {
            new_value: DocParam::Keyframes(track_linear),
            ..
        } = cmd_linear
        else {
            panic!("expected keyframe payload");
        };
        assert_eq!(track_linear.keys()[1].interp, Interp::Linear);
        assert_eq!(track_linear.keys()[0].interp, Interp::Linear);
        assert_eq!(track_linear.keys()[1].value, DocValue::Vec2([1.5, 1.5]));

        let track = keyframe_track(vec![
            (4, RationalTime::ZERO, [3.0, 3.0], Interp::Linear),
            (5, RationalTime::from_seconds(1), [3.0, 3.0], Interp::Linear),
        ]);
        let (doc_equal, target_equal) = minimal_doc(track);
        let cmd_equal = match prepare_add_position_key(&doc_equal, target_equal, t).unwrap() {
            AddPositionKeyPreparation::Prepared { command, .. } => command,
            AddPositionKeyPreparation::AlreadyPresent { .. } => {
                panic!("interior keyframe should prepare")
            }
        };
        let Command::AddPositionKey {
            new_value: DocParam::Keyframes(track_equal),
            ..
        } = cmd_equal
        else {
            panic!("expected keyframe payload");
        };
        assert_eq!(track_equal.keys()[0].interp, Interp::Hold);
        assert_eq!(track_equal.keys()[1].interp, Interp::Hold);
        assert_eq!(track_equal.keys()[1].value, DocValue::Vec2([3.0, 3.0]));
    }

    #[test]
    fn interior_bezier_preserves_curve_samples_on_both_sides_and_splits() {
        let track = keyframe_track(vec![
            (
                0,
                RationalTime::ZERO,
                [0.0, 0.0],
                Interp::Bezier {
                    x1: 0.28,
                    y1: -0.2,
                    x2: 0.84,
                    y2: 0.8,
                },
            ),
            (
                1,
                RationalTime::try_new(10, 1).unwrap(),
                [10.0, 10.0],
                Interp::Linear,
            ),
        ]);
        let before_track = match &track {
            DocParam::Keyframes(keys) => keys.clone(),
            _ => unreachable!(),
        };
        let insert_t = RationalTime::try_new(3, 1).unwrap();
        let (doc, target) = minimal_doc(track);

        let cmd = match prepare_add_position_key(&doc, target, insert_t).unwrap() {
            AddPositionKeyPreparation::Prepared { command, .. } => command,
            _ => panic!("interior keyframe should prepare"),
        };

        let Command::AddPositionKey {
            new_value: DocParam::Keyframes(after),
            ..
        } = cmd
        else {
            panic!("expected keyframe payload");
        };

        let after_keys = after.keys();
        assert!(matches!(after_keys[0].interp, Interp::Bezier { .. }));
        assert!(matches!(after_keys[1].interp, Interp::Bezier { .. }));

        let after_track = after;
        for i in 0..=100 {
            let t = RationalTime::try_new(i, 10).unwrap();
            if t == insert_t {
                continue;
            }
            let expected = as_vec2(&before_track.eval(t));
            let actual = as_vec2(&after_track.eval(t));
            assert!(
                (expected[0] - actual[0]).abs() <= 1e-6,
                "x mismatch at {t:?}"
            );
            assert!(
                (expected[1] - actual[1]).abs() <= 1e-6,
                "y mismatch at {t:?}"
            );
        }
    }

    #[test]
    fn prepare_before_and_after_edges_keep_linear_extension() {
        let track = keyframe_track(vec![
            (0, RationalTime::from_seconds(1), [0.0, 0.0], Interp::Linear),
            (1, RationalTime::from_seconds(3), [2.0, 2.0], Interp::Linear),
        ]);
        let (doc, target) = minimal_doc(track);

        let before_cmd =
            match prepare_add_position_key(&doc, target, RationalTime::from_seconds(0)).unwrap() {
                AddPositionKeyPreparation::Prepared { command, .. } => command,
                _ => panic!("edge keyframe should prepare"),
            };
        let Command::AddPositionKey {
            new_value: DocParam::Keyframes(track_before),
            ..
        } = before_cmd
        else {
            panic!("expected keyframe payload");
        };
        assert_eq!(track_before.keys().len(), 3);
        assert_eq!(track_before.keys()[0].t, RationalTime::from_seconds(0));
        assert_eq!(track_before.keys()[0].interp, Interp::Linear);

        let after_cmd =
            match prepare_add_position_key(&doc, target, RationalTime::from_seconds(4)).unwrap() {
                AddPositionKeyPreparation::Prepared { command, .. } => command,
                _ => panic!("edge keyframe should prepare"),
            };
        let Command::AddPositionKey {
            new_value: DocParam::Keyframes(track_after),
            ..
        } = after_cmd
        else {
            panic!("expected keyframe payload");
        };
        assert_eq!(track_after.keys().len(), 3);
        assert_eq!(track_after.keys()[2].t, RationalTime::from_seconds(4));
        assert_eq!(track_after.keys()[2].interp, Interp::Linear);
    }

    #[test]
    fn unsupported_source_and_non_vec2_and_empty_track_rejected() {
        let (doc_data, target) = minimal_doc(DocParam::Data {
            track: motolii_eval::DataTrackId("v".into()),
            fallback: DocValue::Vec2([0.0, 0.0]),
        });
        let err = prepare_add_position_key(&doc_data, target, RationalTime::ZERO)
            .expect_err("data source unsupported");
        assert!(matches!(
            err,
            AddPositionKeyPrepareError::PositionSourceUnsupported { .. }
        ));

        let (mut doc_key_non_vec2, target_non_vec2) = minimal_doc(DocParam::const_vec2([0.0, 0.0]));
        {
            let mut track = DocKeyframeTrack::new();
            track.insert(DocKeyframe {
                id: KeyframeId::from_raw(42),
                t: RationalTime::ZERO,
                value: DocValue::F64(0.5),
                interp: Interp::Hold,
            });
            set_position(
                &mut doc_key_non_vec2,
                target_non_vec2,
                DocParam::Keyframes(track),
            );
        }
        let err = prepare_add_position_key(
            &doc_key_non_vec2,
            target_non_vec2,
            RationalTime::from_seconds(1),
        )
        .expect_err("non-vec2 keyframe should fail");
        assert!(matches!(
            err,
            AddPositionKeyPrepareError::PositionValueTypeMismatch { .. }
        ));

        let (mut doc_empty, target_empty) = minimal_doc(DocParam::const_vec2([0.0, 0.0]));
        set_position(
            &mut doc_empty,
            target_empty,
            DocParam::Keyframes(DocKeyframeTrack::new()),
        );
        let err = prepare_add_position_key(&doc_empty, target_empty, RationalTime::ZERO)
            .expect_err("empty track should fail");
        assert!(matches!(
            err,
            AddPositionKeyPrepareError::PositionSourceUnsupported { .. }
        ));
    }

    #[test]
    fn stale_position_and_malformed_reservation_and_id_collision_and_tampered_payload_reject_atomically(
    ) {
        let (doc, target) = minimal_doc(DocParam::const_vec2([0.0, 0.0]));
        let before = doc.next_stable_id.peek_next();
        let t = RationalTime::from_seconds(1);
        let command = match prepare_add_position_key(&doc, target, t).unwrap() {
            AddPositionKeyPreparation::Prepared { command, .. } => command,
            AddPositionKeyPreparation::AlreadyPresent { .. } => {
                panic!("const source should not be present")
            }
        };
        let Command::AddPositionKey {
            target,
            added_key_id,
            old_value,
            new_value,
            stable_id_reservation: _,
        } = command
        else {
            panic!("expected add position command")
        };

        let mut stale = doc.clone();
        set_position(&mut stale, target, DocParam::const_vec2([9.0, 9.0]));
        let before_stale = stale.clone();
        let stale_cmd = Command::AddPositionKey {
            target,
            old_value: old_value.clone(),
            new_value: new_value.clone(),
            added_key_id,
            stable_id_reservation: StableIdReservation::new(before, before + 1),
        };
        let err = stale_cmd.apply(&mut stale).unwrap_err();
        assert!(matches!(err, CmdErr::AddPositionKeyPayloadMismatch { .. }));
        assert_eq!(stale, before_stale);

        let malformed = Command::AddPositionKey {
            target,
            old_value: old_value.clone(),
            new_value: new_value.clone(),
            added_key_id,
            stable_id_reservation: StableIdReservation::new(before + 1, before + 2),
        };
        let mut malformed_doc = doc.clone();
        let malformed_before = malformed_doc.clone();
        let err = malformed
            .apply(&mut malformed_doc)
            .expect_err("bad reservation");
        assert!(matches!(err, CmdErr::StableIdReservationMismatch { .. }));
        assert_eq!(malformed_doc, malformed_before);

        let (collision_doc, collision_target) = {
            let (mut doc, layer) = minimal_doc(DocParam::const_vec2([0.0, 0.0]));
            let collision_id = 1u64;
            let mut collision_track = DocKeyframeTrack::new();
            collision_track.insert(keyframe(
                collision_id,
                RationalTime::ZERO,
                [9.0, 9.0],
                Interp::Hold,
            ));

            let collision_layer = doc.layers.allocate("collision").unwrap();
            let mut envelope = ItemEnvelope::new(collision_layer);
            envelope.transform.position = DocParam::Keyframes(collision_track);
            let asset = doc
                .assets
                .allocate("collision-media", "video/mp4", "collision-hash")
                .unwrap();
            let collision_track_id = doc.track_ids.allocate("V_collision").unwrap();
            doc.tracks.push(Track {
                id: collision_track_id,
                items: vec![TrackItem::Clip(Clip {
                    envelope,
                    start: RationalTime::ZERO,
                    duration: RationalTime::from_seconds(10),
                    time_map: Default::default(),
                    source: ClipSource::asset_video_only(asset),
                })],
            });
            while doc.next_stable_id.peek_next() <= collision_id {
                doc.next_stable_id.allocate().unwrap();
            }
            doc.validate().unwrap();
            (doc, layer)
        };
        let collision_id = 1u64;
        let collision_payload = deterministic_new_position(
            &DocParam::const_vec2([0.0, 0.0]),
            collision_target,
            RationalTime::from_seconds(1),
            KeyframeId::from_raw(collision_id),
        )
        .unwrap();
        let collision_cmd = Command::AddPositionKey {
            target: collision_target,
            old_value: DocParam::const_vec2([0.0, 0.0]),
            new_value: collision_payload,
            added_key_id: KeyframeId::from_raw(collision_id),
            stable_id_reservation: StableIdReservation::new(collision_id, collision_id + 1),
        };
        let mut with_collision = collision_doc.clone();
        let with_collision_before = with_collision.clone();
        let err = collision_cmd
            .apply(&mut with_collision)
            .expect_err("id collision");
        assert!(
            matches!(err, CmdErr::StableIdCollision { .. }),
            "unexpected collision error: {err:?}"
        );
        assert_eq!(with_collision, with_collision_before);

        let tampered_cmd = Command::AddPositionKey {
            target,
            old_value,
            new_value: DocParam::const_vec2([99.0, 99.0]),
            added_key_id,
            stable_id_reservation: StableIdReservation::new(before, before + 1),
        };
        let mut tampered = doc.clone();
        let before_tampered = tampered.clone();
        let err = tampered_cmd
            .apply(&mut tampered)
            .expect_err("tampered payload");
        assert!(matches!(err, CmdErr::AddPositionKeyPayloadMismatch { .. }));
        assert_eq!(tampered, before_tampered);
    }
}
