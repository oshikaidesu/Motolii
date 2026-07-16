//! D2: 複製時のID再写像(A8)。「subtree内参照は新ID再写像、外向き参照は維持」。
//!
//! **スコープ**: envelope本体(transform 4種+opacity)+effects[].params、
//! および`ClipSource::Plugin`/`VectorContent`/`PathOp`配下のDocParam(LookAt/Follow含む)。

use std::collections::HashMap;

use thiserror::Error;

use crate::command::{
    collect_layer_ids, envelope_of_mut, find_item_location, Command, CommandError,
};
use crate::doc_keyframe::DocKeyframeTrack;
use crate::param::DocParam;
use crate::schema::EffectDefinition;
use crate::schema::{
    ClipSource, ItemEnvelope, PathOp, StandardShape, TrackItem, Transform2D, VectorContent,
};
use crate::stable_id::{EffectId, KeyframeId, StableIdError, StableIdSeq};
use crate::{Document, LayerId, LayerIdError};

#[derive(Debug, Clone, PartialEq, Error)]
pub enum DuplicateError {
    #[error(transparent)]
    Command(#[from] CommandError),
    #[error(transparent)]
    LayerId(#[from] LayerIdError),
    #[error(transparent)]
    StableId(#[from] StableIdError),
}

/// `source`が指すTrackItem(Clip/Group、再帰する子も含む)を複製する準備をする。
///
/// 新規LayerId/EffectId/KeyframeIdを発行し(非再利用カウンタを実際に進める)、
/// subtree内の`transform.parent`/`LookAt`/`Follow`参照を新IDへ再写像する。
/// subtree外を指す参照(親が複製対象外、他レイヤーへのLookAt等)はそのまま維持する。
///
/// LayerIdは`reserve`のみ(台帳エントリは作らない)。エントリは戻り値の
/// `AddTrackItem.layer_names`経由でapply時に載る — undoのRemoveで台帳から外れ、
/// `max_layers`に孤児が溜まらない。
///
/// 戻り値の`Command::AddTrackItem`を`DocumentWriter::apply_command`へ渡すことで、
/// 単一writer境界を保ったまま実際にツリーへ挿入する(この関数自体はtracksを変更しない)。
pub fn duplicate_track_item(
    doc: &mut Document,
    source: LayerId,
) -> Result<Command, DuplicateError> {
    let (parent, index, item_ref) =
        find_item_location(doc, source).ok_or(CommandError::LayerNotFound(source.get()))?;
    let mut cloned = item_ref.clone();

    let mut old_ids = Vec::new();
    collect_layer_ids(&cloned, &mut old_ids);

    let mut id_map: HashMap<u64, LayerId> = HashMap::with_capacity(old_ids.len());
    let mut layer_names = std::collections::BTreeMap::new();
    for old in old_ids {
        let name = doc.layers.display_name(old).unwrap_or("layer").to_string();
        let new_id = doc.layers.reserve()?;
        id_map.insert(old.get(), new_id);
        layer_names.insert(new_id, name);
    }

    let before = doc.next_stable_id.peek_next();
    let mut seq = doc.next_stable_id;
    remap_item(&mut cloned, &id_map, &mut seq)?;
    doc.next_stable_id = seq;
    if seq.peek_next() != before {
        // 新規EffectId/KeyframeIdを発行した(subtreeにeffect/keyframeが存在した) —
        // ネスト永続フィールドの規律(M2E-11①)でversion/min_reader_versionを揃えて上げる。
        let floor = crate::validate::MIN_READER_VERSION_FOR_STABLE_IDS;
        doc.min_reader_version = doc.min_reader_version.max(floor);
        doc.version = doc.version.max(floor);
    }

    Ok(Command::AddTrackItem {
        parent,
        index: index + 1,
        item: cloned,
        layer_names,
    })
}

fn remap_item(
    item: &mut TrackItem,
    id_map: &HashMap<u64, LayerId>,
    seq: &mut StableIdSeq,
) -> Result<(), StableIdError> {
    remap_envelope(envelope_of_mut(item), id_map, seq)?;
    match item {
        TrackItem::Clip(clip) => remap_clip_source(&mut clip.source, id_map, seq)?,
        TrackItem::Group(group) => {
            for child in &mut group.children {
                remap_item(child, id_map, seq)?;
            }
        }
    }
    Ok(())
}

fn remap_clip_source(
    source: &mut ClipSource,
    id_map: &HashMap<u64, LayerId>,
    seq: &mut StableIdSeq,
) -> Result<(), StableIdError> {
    match source {
        ClipSource::Asset { audio, .. } => {
            for comp in audio {
                remap_doc_param(&mut comp.gain, id_map, seq)?;
            }
            Ok(())
        }
        ClipSource::Plugin { params, .. } => {
            for param in params.values_mut() {
                remap_doc_param(param, id_map, seq)?;
            }
            Ok(())
        }
        ClipSource::Vector { recipe } => {
            remap_vector_content(&mut recipe.content, id_map, seq)?;
            for op in &mut recipe.modifiers {
                remap_path_op(op, id_map, seq)?;
            }
            Ok(())
        }
    }
}

fn remap_vector_content(
    content: &mut VectorContent,
    id_map: &HashMap<u64, LayerId>,
    seq: &mut StableIdSeq,
) -> Result<(), StableIdError> {
    match content {
        VectorContent::StandardShape { shape } => match shape {
            StandardShape::Rect { width, height } | StandardShape::Ellipse { width, height } => {
                remap_doc_param(width, id_map, seq)?;
                remap_doc_param(height, id_map, seq)
            }
        },
        VectorContent::SvgAsset { .. } | VectorContent::TextPath { .. } => Ok(()),
        VectorContent::Group { children } => {
            for child in children {
                remap_vector_content(child, id_map, seq)?;
            }
            Ok(())
        }
    }
}

fn remap_path_op(
    op: &mut PathOp,
    id_map: &HashMap<u64, LayerId>,
    seq: &mut StableIdSeq,
) -> Result<(), StableIdError> {
    match op {
        PathOp::PuckerBloat { amount } => remap_doc_param(amount, id_map, seq),
        PathOp::ZigZag {
            amount,
            ridges,
            point_type: _,
        } => {
            remap_doc_param(amount, id_map, seq)?;
            remap_doc_param(ridges, id_map, seq)
        }
        PathOp::Offset {
            distance,
            line_join: _,
            miter_limit: _,
        } => remap_doc_param(distance, id_map, seq),
        PathOp::RoundCorners { radius } => remap_doc_param(radius, id_map, seq),
        PathOp::Trim {
            start,
            end,
            offset,
            mode: _,
        } => {
            remap_doc_param(start, id_map, seq)?;
            remap_doc_param(end, id_map, seq)?;
            remap_doc_param(offset, id_map, seq)
        }
        PathOp::Twist { angle, center } => {
            remap_doc_param(angle, id_map, seq)?;
            remap_doc_param(center, id_map, seq)
        }
        PathOp::Wiggle { amp, freq, seed: _ } => {
            remap_doc_param(amp, id_map, seq)?;
            remap_doc_param(freq, id_map, seq)
            // seedはu64固定(非DocParam) — キーフレーム再写像対象外。
        }
        PathOp::Repeater {
            copies,
            offset,
            transform,
            composite: _,
            start_opacity,
            end_opacity,
        } => {
            remap_doc_param(copies, id_map, seq)?;
            remap_doc_param(offset, id_map, seq)?;
            remap_transform2d(transform, id_map, seq)?;
            remap_doc_param(start_opacity, id_map, seq)?;
            remap_doc_param(end_opacity, id_map, seq)
        }
    }
}

fn remap_transform2d(
    transform: &mut Transform2D,
    id_map: &HashMap<u64, LayerId>,
    seq: &mut StableIdSeq,
) -> Result<(), StableIdError> {
    if let Some(parent) = transform.parent {
        if let Some(&new_parent) = id_map.get(&parent.get()) {
            transform.parent = Some(new_parent);
        }
    }
    remap_doc_param(&mut transform.position, id_map, seq)?;
    remap_doc_param(&mut transform.anchor, id_map, seq)?;
    remap_doc_param(&mut transform.scale, id_map, seq)?;
    remap_doc_param(&mut transform.rotation, id_map, seq)
}

fn remap_envelope(
    env: &mut ItemEnvelope,
    id_map: &HashMap<u64, LayerId>,
    seq: &mut StableIdSeq,
) -> Result<(), StableIdError> {
    if let Some(&new_id) = id_map.get(&env.layer_id.get()) {
        env.layer_id = new_id;
    }
    if let Some(parent) = env.transform.parent {
        // subtree内の親のみ再写像。subtree外(=id_mapに無い)は「外向き参照は維持」。
        if let Some(&new_parent) = id_map.get(&parent.get()) {
            env.transform.parent = Some(new_parent);
        }
    }
    remap_doc_param(&mut env.transform.position, id_map, seq)?;
    remap_doc_param(&mut env.transform.anchor, id_map, seq)?;
    remap_doc_param(&mut env.transform.scale, id_map, seq)?;
    remap_doc_param(&mut env.transform.rotation, id_map, seq)?;
    remap_doc_param(&mut env.opacity, id_map, seq)?;
    // D1l: definition_idは共有のまま(duplicate時にrecipeをmaterializeしない — GAP-14§1)。
    // Use identityだけ新規採番する。
    for effect in &mut env.effects {
        effect.id = EffectId::from_raw(seq.allocate()?);
    }
    Ok(())
}

fn remap_doc_param(
    param: &mut DocParam,
    id_map: &HashMap<u64, LayerId>,
    seq: &mut StableIdSeq,
) -> Result<(), StableIdError> {
    match param {
        DocParam::Const(_) | DocParam::Data { .. } => {}
        DocParam::Keyframes(track) => {
            remint_keyframes_in_param(track, seq)?;
        }
        DocParam::Vec2Axes { x, y } => {
            remap_doc_param(x, id_map, seq)?;
            remap_doc_param(y, id_map, seq)?;
        }
        DocParam::LookAt { target, .. } | DocParam::Follow { target, .. } => {
            if let Some(&new_id) = id_map.get(&target.get()) {
                *target = new_id;
            }
        }
    }
    Ok(())
}

/// Copy Local payload検証と予約区間の導入ID列挙で共用する固定走査(D1l/GAP-14§2.3)。
pub(crate) fn remint_order_keyframe_ids(definition: &EffectDefinition) -> Vec<KeyframeId> {
    let mut ids = Vec::new();
    for param in definition.params.values() {
        collect_keyframe_ids_param(param, &mut ids);
    }
    ids
}

fn collect_keyframe_ids_param(param: &DocParam, out: &mut Vec<KeyframeId>) {
    match param {
        DocParam::Const(_)
        | DocParam::Data { .. }
        | DocParam::LookAt { .. }
        | DocParam::Follow { .. } => {}
        DocParam::Keyframes(track) => {
            for key in track.keys() {
                out.push(key.id);
            }
        }
        DocParam::Vec2Axes { x, y } => {
            collect_keyframe_ids_param(x, out);
            collect_keyframe_ids_param(y, out);
        }
    }
}

fn remint_keyframes_in_param(
    track: &mut DocKeyframeTrack,
    seq: &mut StableIdSeq,
) -> Result<(), StableIdError> {
    let mut fresh = DocKeyframeTrack::new();
    for key in track.keys() {
        let mut k = key.clone();
        k.id = KeyframeId::from_raw(seq.allocate()?);
        fresh.insert(k);
    }
    *track = fresh;
    Ok(())
}

/// Create/Copy Local prepare: params辞書順・Vec2Axes x→yの固定採番順でKeyframeIdを再採番する。
pub(crate) fn remint_doc_param(
    param: &mut DocParam,
    seq: &mut StableIdSeq,
) -> Result<(), StableIdError> {
    match param {
        DocParam::Const(_)
        | DocParam::Data { .. }
        | DocParam::LookAt { .. }
        | DocParam::Follow { .. } => Ok(()),
        DocParam::Keyframes(track) => remint_keyframes_in_param(track, seq),
        DocParam::Vec2Axes { x, y } => {
            remint_doc_param(x, seq)?;
            remint_doc_param(y, seq)
        }
    }
}

/// Copy Local prepare: Definition本体のparamsを固定採番順でremintする。
pub(crate) fn remint_effect_definition(
    definition: &mut EffectDefinition,
    seq: &mut StableIdSeq,
) -> Result<(), StableIdError> {
    for param in definition.params.values_mut() {
        remint_doc_param(param, seq)?;
    }
    Ok(())
}

/// Copy Local apply: payloadのDefinition本体が参照元と意味同一(ID除く)か。
pub(crate) fn definition_semantic_body_eq(
    source: &EffectDefinition,
    payload: &EffectDefinition,
) -> bool {
    if source.plugin_id != payload.plugin_id
        || source.effect_version != payload.effect_version
        || source.enabled != payload.enabled
        || source.extra != payload.extra
        || source.params.len() != payload.params.len()
    {
        return false;
    }
    source.params.iter().all(|(name, src_param)| {
        payload
            .params
            .get(name)
            .is_some_and(|payload_param| doc_param_semantic_body_eq(src_param, payload_param))
    })
}

fn doc_param_semantic_body_eq(a: &DocParam, b: &DocParam) -> bool {
    match (a, b) {
        (DocParam::Const(va), DocParam::Const(vb)) => va == vb,
        (
            DocParam::Data {
                track: ta,
                fallback: fa,
            },
            DocParam::Data {
                track: tb,
                fallback: fb,
            },
        ) => ta == tb && fa == fb,
        (
            DocParam::LookAt {
                target: ta,
                axis: aa,
            },
            DocParam::LookAt {
                target: tb,
                axis: ab,
            },
        ) => ta == tb && aa == ab,
        (
            DocParam::Follow {
                target: ta,
                offset: oa,
            },
            DocParam::Follow {
                target: tb,
                offset: ob,
            },
        ) => ta == tb && oa == ob,
        (DocParam::Keyframes(ta), DocParam::Keyframes(tb)) => {
            let ka = ta.keys();
            let kb = tb.keys();
            ka.len() == kb.len()
                && ka
                    .iter()
                    .zip(kb.iter())
                    .all(|(a, b)| a.t == b.t && a.value == b.value && a.interp == b.interp)
        }
        (DocParam::Vec2Axes { x: ax, y: ay }, DocParam::Vec2Axes { x: bx, y: by }) => {
            doc_param_semantic_body_eq(ax, bx) && doc_param_semantic_body_eq(ay, by)
        }
        _ => false,
    }
}
