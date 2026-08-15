//! EffectId/KeyframeId 共有空間の走査。検証入口と AssetRef 棚卸しから切り離すため。

use std::collections::HashSet;

use crate::param::DocParam;
use crate::schema::{
    asset_components_require_newer_reader, ClipSource, CompCameraDoc, ItemEnvelope, PathOp,
    StandardShape, TrackItem, Transform2D, VectorContent,
};
use crate::Document;

use super::DocumentError;

/// Document内の全stable ID(EffectUse/EffectDefinition/Keyframe共有空間)を収集する。
/// command/migrationの衝突検査で`validate_stable_ids`と同じ走査を再利用する(D1l)。
pub(crate) fn collect_document_stable_ids(doc: &Document) -> HashSet<u64> {
    let mut seen = HashSet::new();
    let mut max_observed = None;
    for def in &doc.effect_definitions {
        let _ = note_stable_id(def.id.get(), &mut seen, &mut max_observed);
        for param in def.params.values() {
            let _ = collect_stable_ids_param(param, &mut seen, &mut max_observed);
        }
    }
    for track in &doc.tracks {
        for item in &track.items {
            let _ = collect_stable_ids_item(item, &mut seen, &mut max_observed);
        }
    }
    let _ = collect_stable_ids_comp_camera(&doc.composition.camera, &mut seen, &mut max_observed);
    seen
}

pub(crate) fn stable_id_in_use(doc: &Document, id: u64) -> bool {
    collect_document_stable_ids(doc).contains(&id)
}

pub(super) fn note_stable_id(
    id: u64,
    seen: &mut HashSet<u64>,
    max_observed: &mut Option<u64>,
) -> Result<(), DocumentError> {
    if !seen.insert(id) {
        return Err(DocumentError::DuplicateStableId { id });
    }
    *max_observed = Some(max_observed.map_or(id, |m| m.max(id)));
    Ok(())
}

pub(super) fn item_uses_asset_components(item: &TrackItem) -> bool {
    match item {
        TrackItem::Clip(clip) => match &clip.source {
            ClipSource::Asset { video, audio, .. } => {
                asset_components_require_newer_reader(video, audio)
            }
            _ => false,
        },
        TrackItem::Group(group) => group.children.iter().any(item_uses_asset_components),
    }
}

pub(super) fn collect_stable_ids_item(
    item: &TrackItem,
    seen: &mut HashSet<u64>,
    max_observed: &mut Option<u64>,
) -> Result<(), DocumentError> {
    match item {
        TrackItem::Clip(clip) => {
            collect_stable_ids_envelope(&clip.envelope, seen, max_observed)?;
            match &clip.source {
                ClipSource::Asset { audio, .. } => {
                    for comp in audio {
                        collect_stable_ids_param(&comp.gain, seen, max_observed)?;
                    }
                    Ok(())
                }
                ClipSource::Plugin { params, .. } => {
                    for param in params.values() {
                        collect_stable_ids_param(param, seen, max_observed)?;
                    }
                    Ok(())
                }
                ClipSource::Vector { recipe } => {
                    collect_stable_ids_vector_content(&recipe.content, seen, max_observed)?;
                    for op in &recipe.modifiers {
                        collect_stable_ids_path_op(op, seen, max_observed)?;
                    }
                    Ok(())
                }
            }
        }
        TrackItem::Group(group) => {
            collect_stable_ids_envelope(&group.envelope, seen, max_observed)?;
            for child in &group.children {
                collect_stable_ids_item(child, seen, max_observed)?;
            }
            Ok(())
        }
    }
}

fn collect_stable_ids_envelope(
    env: &ItemEnvelope,
    seen: &mut HashSet<u64>,
    max_observed: &mut Option<u64>,
) -> Result<(), DocumentError> {
    collect_stable_ids_param(&env.transform.position, seen, max_observed)?;
    collect_stable_ids_param(&env.transform.anchor, seen, max_observed)?;
    collect_stable_ids_param(&env.transform.scale, seen, max_observed)?;
    collect_stable_ids_param(&env.transform.rotation, seen, max_observed)?;
    collect_stable_ids_param(&env.opacity, seen, max_observed)?;
    // D1l: EffectUseのparamsは持たない(paramsはeffect_definitions側でcollectする)。
    for use_ in &env.effects {
        note_stable_id(use_.id.get(), seen, max_observed)?;
    }
    Ok(())
}

pub(super) fn collect_stable_ids_comp_camera(
    camera: &CompCameraDoc,
    seen: &mut HashSet<u64>,
    max_observed: &mut Option<u64>,
) -> Result<(), DocumentError> {
    match camera {
        CompCameraDoc::PlanarOrthographic {
            center,
            roll_radians,
            height,
        } => {
            collect_stable_ids_param(center, seen, max_observed)?;
            collect_stable_ids_param(roll_radians, seen, max_observed)?;
            collect_stable_ids_param(height, seen, max_observed)
        }
    }
}

pub(super) fn collect_stable_ids_param(
    param: &DocParam,
    seen: &mut HashSet<u64>,
    max_observed: &mut Option<u64>,
) -> Result<(), DocumentError> {
    match param {
        DocParam::Const(_)
        | DocParam::Data { .. }
        | DocParam::LookAt { .. }
        | DocParam::Follow { .. } => Ok(()),
        DocParam::Keyframes(track) => {
            for key in track.keys() {
                note_stable_id(key.id.get(), seen, max_observed)?;
            }
            Ok(())
        }
        DocParam::Vec2Axes { x, y } => {
            collect_stable_ids_param(x, seen, max_observed)?;
            collect_stable_ids_param(y, seen, max_observed)
        }
    }
}

fn collect_stable_ids_vector_content(
    content: &VectorContent,
    seen: &mut HashSet<u64>,
    max_observed: &mut Option<u64>,
) -> Result<(), DocumentError> {
    match content {
        VectorContent::StandardShape { shape } => match shape {
            StandardShape::Rect { width, height } | StandardShape::Ellipse { width, height } => {
                collect_stable_ids_param(width, seen, max_observed)?;
                collect_stable_ids_param(height, seen, max_observed)
            }
        },
        VectorContent::SvgAsset { .. } | VectorContent::TextPath { .. } => Ok(()),
        VectorContent::Group { children } => {
            for child in children {
                collect_stable_ids_vector_content(child, seen, max_observed)?;
            }
            Ok(())
        }
    }
}

fn collect_stable_ids_path_op(
    op: &PathOp,
    seen: &mut HashSet<u64>,
    max_observed: &mut Option<u64>,
) -> Result<(), DocumentError> {
    match op {
        PathOp::PuckerBloat { amount } => collect_stable_ids_param(amount, seen, max_observed),
        PathOp::ZigZag {
            amount,
            ridges,
            point_type: _,
        } => {
            collect_stable_ids_param(amount, seen, max_observed)?;
            collect_stable_ids_param(ridges, seen, max_observed)
        }
        PathOp::Offset {
            distance,
            line_join: _,
            miter_limit: _,
        } => collect_stable_ids_param(distance, seen, max_observed),
        PathOp::RoundCorners { radius } => collect_stable_ids_param(radius, seen, max_observed),
        PathOp::Trim {
            start,
            end,
            offset,
            mode: _,
        } => {
            collect_stable_ids_param(start, seen, max_observed)?;
            collect_stable_ids_param(end, seen, max_observed)?;
            collect_stable_ids_param(offset, seen, max_observed)
        }
        PathOp::Twist { angle, center } => {
            collect_stable_ids_param(angle, seen, max_observed)?;
            collect_stable_ids_param(center, seen, max_observed)
        }
        PathOp::Wiggle { amp, freq, seed: _ } => {
            collect_stable_ids_param(amp, seen, max_observed)?;
            collect_stable_ids_param(freq, seen, max_observed)
            // seedはu64固定(非DocParam) — stable id走査対象外。
        }
        PathOp::Repeater {
            copies,
            offset,
            transform,
            composite: _,
            start_opacity,
            end_opacity,
        } => {
            collect_stable_ids_param(copies, seen, max_observed)?;
            collect_stable_ids_param(offset, seen, max_observed)?;
            collect_stable_ids_transform2d(transform, seen, max_observed)?;
            collect_stable_ids_param(start_opacity, seen, max_observed)?;
            collect_stable_ids_param(end_opacity, seen, max_observed)
        }
    }
}

fn collect_stable_ids_transform2d(
    transform: &Transform2D,
    seen: &mut HashSet<u64>,
    max_observed: &mut Option<u64>,
) -> Result<(), DocumentError> {
    collect_stable_ids_param(&transform.position, seen, max_observed)?;
    collect_stable_ids_param(&transform.anchor, seen, max_observed)?;
    collect_stable_ids_param(&transform.scale, seen, max_observed)?;
    collect_stable_ids_param(&transform.rotation, seen, max_observed)
}
