//! D2: 複製時のID再写像(A8)。「subtree内参照は新ID再写像、外向き参照は維持」。
//!
//! **スコープ**: envelope本体(transform 4種+opacity)+effects[].params、
//! および`ClipSource::Plugin`/`VectorContent`/`PathOp`配下のDocParam(LookAt/Follow含む)。

use std::collections::HashMap;

use thiserror::Error;

use crate::command::{envelope_of, envelope_of_mut, find_item_location, Command, CommandError};
use crate::doc_keyframe::DocKeyframeTrack;
use crate::param::DocParam;
use crate::schema::{ClipSource, ItemEnvelope, PathOp, StandardShape, TrackItem, VectorContent};
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
    for old in old_ids {
        let name = doc.layers.display_name(old).unwrap_or("layer").to_string();
        let new_id = doc.layers.allocate(name)?;
        id_map.insert(old.get(), new_id);
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
    })
}

fn collect_layer_ids(item: &TrackItem, out: &mut Vec<LayerId>) {
    out.push(envelope_of(item).layer_id);
    if let TrackItem::Group(g) = item {
        for child in &g.children {
            collect_layer_ids(child, out);
        }
    }
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
        ClipSource::Asset { .. } => Ok(()),
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
        PathOp::ZigZag { amount, ridges } => {
            remap_doc_param(amount, id_map, seq)?;
            remap_doc_param(ridges, id_map, seq)
        }
        PathOp::Offset { distance } => remap_doc_param(distance, id_map, seq),
        PathOp::RoundCorners { radius } => remap_doc_param(radius, id_map, seq),
        PathOp::Trim {
            start, end, offset, ..
        } => {
            remap_doc_param(start, id_map, seq)?;
            remap_doc_param(end, id_map, seq)?;
            remap_doc_param(offset, id_map, seq)
        }
        PathOp::Twist { angle } => remap_doc_param(angle, id_map, seq),
        PathOp::Wiggle { amp, freq, seed } => {
            remap_doc_param(amp, id_map, seq)?;
            remap_doc_param(freq, id_map, seq)?;
            remap_doc_param(seed, id_map, seq)
        }
        PathOp::Repeater { copies, offset } => {
            remap_doc_param(copies, id_map, seq)?;
            remap_doc_param(offset, id_map, seq)
        }
    }
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
    for effect in &mut env.effects {
        effect.id = EffectId::from_raw(seq.allocate()?);
        for param in effect.params.values_mut() {
            remap_doc_param(param, id_map, seq)?;
        }
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
            let mut fresh = DocKeyframeTrack::new();
            for key in track.keys() {
                let mut k = key.clone();
                k.id = KeyframeId::from_raw(seq.allocate()?);
                fresh.insert(k);
            }
            *track = fresh;
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
