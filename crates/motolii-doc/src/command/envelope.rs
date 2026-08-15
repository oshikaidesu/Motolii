use crate::param::DocParam;
use crate::schema::{ClipSource, ItemEnvelope, TrackItem};
use crate::validate::stable_id_in_use;
use crate::{Document, LayerId};

use super::locate::{find_envelope, find_envelope_mut, find_track_item_mut};
use super::{CommandError, ScalarPropertyId};

fn keyframe_ids(param: &DocParam) -> Vec<u64> {
    match param {
        DocParam::Keyframes(track) => track.keys().iter().map(|key| key.id.get()).collect(),
        _ => Vec::new(),
    }
}

fn live_envelope_param<'a>(
    env: &'a ItemEnvelope,
    property: &ScalarPropertyId,
) -> Option<&'a DocParam> {
    match property {
        ScalarPropertyId::Position => Some(&env.transform.position),
        ScalarPropertyId::Anchor => Some(&env.transform.anchor),
        ScalarPropertyId::Scale => Some(&env.transform.scale),
        ScalarPropertyId::Rotation => Some(&env.transform.rotation),
        ScalarPropertyId::Opacity => Some(&env.opacity),
        ScalarPropertyId::EffectParam(_, _) | ScalarPropertyId::SourceParam(_) => None,
    }
}

pub(super) fn apply_envelope_set_property(
    doc: &mut Document,
    target: LayerId,
    property: &ScalarPropertyId,
    new_value: &DocParam,
) -> Result<(), CommandError> {
    let live_ids = {
        let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(target.get()))?;
        let live =
            live_envelope_param(env, property).ok_or(CommandError::LayerNotFound(target.get()))?;
        keyframe_ids(live)
    };
    let introduced: Vec<u64> = keyframe_ids(new_value)
        .into_iter()
        .filter(|id| !live_ids.contains(id))
        .collect();
    for id in &introduced {
        if stable_id_in_use(doc, *id) {
            return Err(CommandError::StableIdCollision { id: *id });
        }
    }
    let env = find_envelope_mut(doc, target)?;
    write_property(env, property, new_value.clone())?;
    // なぜ: SetProperty は reservation を持たない。導入した KeyframeId まで next を進めないと validate が落ちる。
    if let Some(max_id) = introduced.iter().copied().max() {
        let after = max_id
            .checked_add(1)
            .ok_or(crate::stable_id::StableIdError::Exhausted)?;
        if after > doc.next_stable_id.peek_next() {
            doc.next_stable_id.commit_validated_reservation(after);
        }
    }
    Ok(())
}

/// `ScalarPropertyId::EffectParam`は`Command::apply`側で`Document.effect_definitions`を
/// 直接書き換える(D1l: paramsはUseではなくDefinitionが持つ)。ここには到達しない防御的分岐。
fn write_property(
    env: &mut ItemEnvelope,
    property: &ScalarPropertyId,
    value: DocParam,
) -> Result<(), CommandError> {
    match property {
        ScalarPropertyId::Position => env.transform.position = value,
        ScalarPropertyId::Anchor => env.transform.anchor = value,
        ScalarPropertyId::Scale => env.transform.scale = value,
        ScalarPropertyId::Rotation => env.transform.rotation = value,
        ScalarPropertyId::Opacity => env.opacity = value,
        ScalarPropertyId::EffectParam(effect_id, _) => {
            return Err(CommandError::EffectNotFound {
                effect: effect_id.get(),
                layer: env.layer_id.get(),
            });
        }
        ScalarPropertyId::SourceParam(name) => {
            return Err(CommandError::SourceParamNotFound {
                layer: env.layer_id.get(),
                param: name.clone(),
            });
        }
    }
    Ok(())
}

pub(super) fn write_source_param(
    doc: &mut Document,
    target: LayerId,
    name: &str,
    value: DocParam,
) -> Result<(), CommandError> {
    let layer = target.get();
    let item = find_track_item_mut(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    let TrackItem::Clip(clip) = item else {
        return Err(CommandError::TrackItemNotClip { layer });
    };
    let ClipSource::Plugin { params, .. } = &mut clip.source else {
        return Err(CommandError::SourceNotPlugin { layer });
    };
    let Some(slot) = params.get_mut(name) else {
        return Err(CommandError::SourceParamNotFound {
            layer,
            param: name.to_owned(),
        });
    };
    *slot = value;
    Ok(())
}
