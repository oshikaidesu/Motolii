//! position keyのlive curve照合。curveが変わったら失敗閉じ。

use motolii_core::RationalTime;
use motolii_doc::{Command, DocParam, DocValue, KeyframeId, LayerId, TrackItem};

use crate::inspector_host_runtime::{InspectorPositionAxis, InspectorPositionGestureStart};

use super::easing::PositionActiveInterval;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PositionGestureBaseline {
    pub(super) session: u64,
    pub(super) target: LayerId,
    pub(super) playhead: RationalTime,
    pub(super) key: KeyframeId,
    pub(super) value: [f64; 2],
    pub(super) position: DocParam,
}

pub(super) fn position_gesture_baseline(
    document: &motolii_doc::Document,
    primary: Option<LayerId>,
    playhead: RationalTime,
    start: InspectorPositionGestureStart,
) -> Option<PositionGestureBaseline> {
    let target = primary?;
    let (key, value, position) = position_key_value_at(document, Some(target), playhead)?;
    if axis_value(value, start.axis) != start.value {
        return None;
    }
    Some(PositionGestureBaseline {
        session: start.session,
        target,
        playhead,
        key,
        value,
        position,
    })
}

pub(super) fn resolve_position_gesture_command(
    document: &motolii_doc::Document,
    primary: Option<LayerId>,
    playhead: RationalTime,
    baseline: &PositionGestureBaseline,
    axis: InspectorPositionAxis,
    value: f64,
) -> Option<Command> {
    if !value.is_finite() || primary != Some(baseline.target) || playhead != baseline.playhead {
        return None;
    }
    let (key, current, position) =
        position_key_value_at(document, Some(baseline.target), baseline.playhead)?;
    if key != baseline.key || current != baseline.value || position != baseline.position {
        return None;
    }
    let new = match axis {
        InspectorPositionAxis::X => [value, baseline.value[1]],
        InspectorPositionAxis::Y => [baseline.value[0], value],
    };
    (new != baseline.value).then_some(Command::SetPositionKeyValue {
        target: baseline.target,
        key: baseline.key,
        old: baseline.value,
        new,
    })
}

pub(super) fn axis_value(value: [f64; 2], axis: InspectorPositionAxis) -> f64 {
    match axis {
        InspectorPositionAxis::X => value[0],
        InspectorPositionAxis::Y => value[1],
    }
}

pub(super) fn position_key_value_at(
    document: &motolii_doc::Document,
    primary: Option<LayerId>,
    playhead: RationalTime,
) -> Option<(KeyframeId, [f64; 2], DocParam)> {
    fn find_envelope(items: &[TrackItem], target: LayerId) -> Option<&motolii_doc::ItemEnvelope> {
        for item in items {
            match item {
                TrackItem::Clip(clip) if clip.envelope.layer_id == target => {
                    return Some(&clip.envelope);
                }
                TrackItem::Group(group) if group.envelope.layer_id == target => {
                    return Some(&group.envelope);
                }
                TrackItem::Group(group) => {
                    if let Some(envelope) = find_envelope(&group.children, target) {
                        return Some(envelope);
                    }
                }
                TrackItem::Clip(_) => {}
            }
        }
        None
    }

    let target = primary?;
    let envelope = document
        .tracks
        .iter()
        .find_map(|track| find_envelope(&track.items, target))?;
    let DocParam::Keyframes(track) = &envelope.transform.position else {
        return None;
    };
    let keys = track.keys();
    if keys.is_empty()
        || track.validate().is_err()
        || keys.iter().any(|key| {
            !matches!(key.value, DocValue::Vec2(value) if value.iter().all(|value| value.is_finite()))
        })
    {
        return None;
    }
    let key = keys.iter().find(|key| key.t == playhead)?;
    let DocValue::Vec2(value) = key.value else {
        return None;
    };
    Some((key.id, value, envelope.transform.position.clone()))
}

pub(super) fn position_active_interval(
    document: &motolii_doc::Document,
    primary: Option<LayerId>,
    playhead: RationalTime,
) -> Option<PositionActiveInterval> {
    fn find_envelope(items: &[TrackItem], target: LayerId) -> Option<&motolii_doc::ItemEnvelope> {
        for item in items {
            match item {
                TrackItem::Clip(clip) if clip.envelope.layer_id == target => {
                    return Some(&clip.envelope);
                }
                TrackItem::Group(group) if group.envelope.layer_id == target => {
                    return Some(&group.envelope);
                }
                TrackItem::Group(group) => {
                    if let Some(envelope) = find_envelope(&group.children, target) {
                        return Some(envelope);
                    }
                }
                TrackItem::Clip(_) => {}
            }
        }
        None
    }

    let layer = primary?;
    let envelope = document
        .tracks
        .iter()
        .find_map(|track| find_envelope(&track.items, layer))?;
    let DocParam::Keyframes(track) = &envelope.transform.position else {
        return None;
    };
    let keys = track.keys();
    if keys.len() < 2
        || track.validate().is_err()
        || keys
            .iter()
            .any(|key| !matches!(key.value, DocValue::Vec2(_)))
    {
        return None;
    }
    keys.windows(2).find_map(|pair| {
        let [left, right] = pair else {
            return None;
        };
        (left.t < playhead && playhead < right.t).then_some(PositionActiveInterval {
            layer,
            left_id: left.id,
            left_t: left.t,
            right_id: right.id,
            right_t: right.t,
            left_interp: left.interp,
        })
    })
}
