use serde_json::Value;

use crate::param::DocParam;
use crate::schema::{
    ClipSource, CompCameraDoc, Group, PathOp, TrackItem, VectorContent, VectorRecipe,
};
use crate::Document;

use super::{DocumentCounts, MigrateError};

pub fn count_document(doc: &Document) -> DocumentCounts {
    let mut clip_count = 0usize;
    let mut keyframe_count = 0usize;
    for track in &doc.tracks {
        for item in &track.items {
            count_item(item, &mut clip_count, &mut keyframe_count);
        }
    }
    // D1l: effect paramsはUseではなくDefinition台帳が持つ(1回だけ数える。共有でも重複しない)。
    for def in &doc.effect_definitions {
        for param in def.params.values() {
            count_param(param, &mut keyframe_count);
        }
    }
    count_comp_camera(&doc.composition.camera, &mut keyframe_count);
    DocumentCounts {
        track_count: doc.tracks.len(),
        clip_count,
        keyframe_count,
    }
}

fn count_item(item: &TrackItem, clips: &mut usize, keys: &mut usize) {
    match item {
        TrackItem::Clip(clip) => {
            *clips += 1;
            count_envelope(&clip.envelope, keys);
            if let ClipSource::Vector { recipe } = &clip.source {
                count_vector_recipe(recipe, keys);
            }
            if let ClipSource::Plugin { params, .. } = &clip.source {
                for param in params.values() {
                    count_param(param, keys);
                }
            }
            if let ClipSource::Asset { audio, .. } = &clip.source {
                for comp in audio {
                    count_param(&comp.gain, keys);
                }
            }
        }
        TrackItem::Group(group) => count_group(group, clips, keys),
    }
}

fn count_group(group: &Group, clips: &mut usize, keys: &mut usize) {
    count_envelope(&group.envelope, keys);
    for child in &group.children {
        count_item(child, clips, keys);
    }
}

fn count_envelope(env: &crate::schema::ItemEnvelope, keys: &mut usize) {
    count_param(&env.transform.position, keys);
    count_param(&env.transform.anchor, keys);
    count_param(&env.transform.scale, keys);
    count_param(&env.transform.rotation, keys);
    count_param(&env.opacity, keys);
    // D1l: EffectUseはid参照のみ。paramsは`count_document`側でDefinition台帳を1回だけ数える。
}

fn count_vector_recipe(recipe: &VectorRecipe, keys: &mut usize) {
    count_vector_content(&recipe.content, keys);
    for op in &recipe.modifiers {
        count_path_op(op, keys);
    }
}

fn count_vector_content(content: &VectorContent, keys: &mut usize) {
    match content {
        VectorContent::StandardShape { shape } => match shape {
            crate::schema::StandardShape::Rect { width, height }
            | crate::schema::StandardShape::Ellipse { width, height } => {
                count_param(width, keys);
                count_param(height, keys);
            }
        },
        VectorContent::SvgAsset { .. } | VectorContent::TextPath { .. } => {}
        VectorContent::Group { children } => {
            for child in children {
                count_vector_content(child, keys);
            }
        }
    }
}

fn count_path_op(op: &PathOp, keys: &mut usize) {
    match op {
        PathOp::PuckerBloat { amount } => count_param(amount, keys),
        PathOp::ZigZag {
            amount,
            ridges,
            point_type: _,
        } => {
            count_param(amount, keys);
            count_param(ridges, keys);
        }
        PathOp::Offset {
            distance,
            line_join: _,
            miter_limit: _,
        } => count_param(distance, keys),
        PathOp::RoundCorners { radius } => count_param(radius, keys),
        PathOp::Trim {
            start,
            end,
            offset,
            mode: _,
        } => {
            count_param(start, keys);
            count_param(end, keys);
            count_param(offset, keys);
        }
        PathOp::Twist { angle, center } => {
            count_param(angle, keys);
            count_param(center, keys);
        }
        PathOp::Wiggle { amp, freq, seed: _ } => {
            count_param(amp, keys);
            count_param(freq, keys);
        }
        PathOp::Repeater {
            copies,
            offset,
            transform,
            composite: _,
            start_opacity,
            end_opacity,
        } => {
            count_param(copies, keys);
            count_param(offset, keys);
            count_param(&transform.position, keys);
            count_param(&transform.anchor, keys);
            count_param(&transform.scale, keys);
            count_param(&transform.rotation, keys);
            count_param(start_opacity, keys);
            count_param(end_opacity, keys);
        }
    }
}

fn count_param(param: &DocParam, keys: &mut usize) {
    match param {
        DocParam::Keyframes(track) => *keys += track.keys().len(),
        DocParam::Vec2Axes { x, y } => {
            count_param(x, keys);
            count_param(y, keys);
        }
        _ => {}
    }
}

fn count_comp_camera(camera: &CompCameraDoc, keys: &mut usize) {
    match camera {
        CompCameraDoc::PlanarOrthographic {
            center,
            roll_radians,
            height,
        } => {
            count_param(center, keys);
            count_param(roll_radians, keys);
            count_param(height, keys);
        }
    }
}

pub(super) fn assert_counts_preserved(
    before: DocumentCounts,
    after: DocumentCounts,
) -> Result<(), MigrateError> {
    if before == after {
        Ok(())
    } else {
        Err(MigrateError::InvariantViolation {
            before_tracks: before.track_count,
            before_clips: before.clip_count,
            before_keys: before.keyframe_count,
            after_tracks: after.track_count,
            after_clips: after.clip_count,
            after_keys: after.keyframe_count,
        })
    }
}

/// JSON走査の量的カウント(変換前の不変条件用。serde拒否前に測る)。
pub(super) fn count_json_document(root: &Value) -> DocumentCounts {
    let mut clip_count = 0usize;
    let mut keyframe_count = 0usize;
    let tracks = root
        .get("tracks")
        .and_then(|t| t.as_array())
        .map(|a| a.as_slice())
        .unwrap_or(&[]);
    for track in tracks {
        if let Some(items) = track.get("items").and_then(|i| i.as_array()) {
            for item in items {
                count_json_item(item, &mut clip_count, &mut keyframe_count);
            }
        }
    }
    // D1l: 旧inline effect.paramsはenvelope側で数える(count_json_envelope)。
    // 既に新形式(root.effect_definitions)のドキュメントはこちらで1回だけ数える。
    if let Some(defs) = root.get("effect_definitions").and_then(|d| d.as_array()) {
        for def in defs {
            if let Some(params) = def.get("params").and_then(|p| p.as_object()) {
                for param in params.values() {
                    count_json_param(Some(param), &mut keyframe_count);
                }
            }
        }
    }
    if let Some(camera) = root
        .get("composition")
        .and_then(|c| c.get("camera"))
        .and_then(|c| c.as_object())
    {
        for key in ["center", "roll_radians", "height"] {
            count_json_param(camera.get(key), &mut keyframe_count);
        }
    }
    DocumentCounts {
        track_count: tracks.len(),
        clip_count,
        keyframe_count,
    }
}

fn count_json_item(item: &Value, clips: &mut usize, keys: &mut usize) {
    let kind = item.get("kind").and_then(|k| k.as_str()).unwrap_or("");
    match kind {
        "clip" => {
            *clips += 1;
            count_json_envelope(item.get("envelope"), keys);
            if let Some(ops) = item.get("path_ops").and_then(|v| v.as_array()) {
                for op in ops {
                    count_json_path_op(op, keys);
                }
            }
            if let Some(source) = item.get("source") {
                count_json_source(source, keys);
            }
        }
        "group" => {
            count_json_envelope(item.get("envelope"), keys);
            if let Some(children) = item.get("children").and_then(|c| c.as_array()) {
                for child in children {
                    count_json_item(child, clips, keys);
                }
            }
        }
        _ => {}
    }
}

fn count_json_envelope(envelope: Option<&Value>, keys: &mut usize) {
    let Some(env) = envelope else {
        return;
    };
    if let Some(xf) = env.get("transform") {
        count_json_param(xf.get("position"), keys);
        count_json_param(xf.get("anchor"), keys);
        count_json_param(xf.get("scale"), keys);
        count_json_param(xf.get("rotation"), keys);
    }
    count_json_param(env.get("opacity"), keys);
    if let Some(effects) = env.get("effects").and_then(|e| e.as_array()) {
        for effect in effects {
            if let Some(params) = effect.get("params").and_then(|p| p.as_object()) {
                for param in params.values() {
                    count_json_param(Some(param), keys);
                }
            }
        }
    }
}

fn count_json_source(source: &Value, keys: &mut usize) {
    let tag = source.get("source").and_then(|s| s.as_str()).unwrap_or("");
    match tag {
        "plugin" => {
            if let Some(params) = source.get("params").and_then(|p| p.as_object()) {
                for param in params.values() {
                    count_json_param(Some(param), keys);
                }
            }
        }
        "vector" => {
            if let Some(recipe) = source.get("recipe") {
                count_json_vector_content(recipe.get("content"), keys);
                if let Some(mods) = recipe.get("modifiers").and_then(|m| m.as_array()) {
                    for op in mods {
                        count_json_path_op(op, keys);
                    }
                }
            }
        }
        "asset" => {
            if let Some(audio) = source.get("audio").and_then(|a| a.as_array()) {
                for comp in audio {
                    count_json_param(comp.get("gain"), keys);
                }
            }
        }
        _ => {}
    }
}

fn count_json_vector_content(content: Option<&Value>, keys: &mut usize) {
    let Some(c) = content else {
        return;
    };
    match c.get("kind").and_then(|k| k.as_str()).unwrap_or("") {
        "standard_shape" => {
            count_json_param(c.get("width"), keys);
            count_json_param(c.get("height"), keys);
        }
        "group" => {
            if let Some(children) = c.get("children").and_then(|ch| ch.as_array()) {
                for child in children {
                    count_json_vector_content(Some(child), keys);
                }
            }
        }
        _ => {}
    }
}

fn count_json_path_op(op: &Value, keys: &mut usize) {
    // 旧Twistはcenter無し。新形式はcenterあり — 件数はキーフレーム数のみ。
    for key in [
        "amount",
        "ridges",
        "distance",
        "radius",
        "start",
        "end",
        "offset",
        "angle",
        "center",
        "amp",
        "freq",
        "copies",
        "start_opacity",
        "end_opacity",
    ] {
        count_json_param(op.get(key), keys);
    }
    if let Some(xf) = op.get("transform") {
        count_json_param(xf.get("position"), keys);
        count_json_param(xf.get("anchor"), keys);
        count_json_param(xf.get("scale"), keys);
        count_json_param(xf.get("rotation"), keys);
    }
    // 旧Wiggle.seedがDocParam(Keyframes)だった場合のみキー数に入る。
    if op.get("seed").and_then(|s| s.as_object()).is_some() {
        count_json_param(op.get("seed"), keys);
    }
}

fn count_json_param(param: Option<&Value>, keys: &mut usize) {
    let Some(p) = param else {
        return;
    };
    if let Some(kf) = p.get("keyframes") {
        if let Some(arr) = kf.get("keys").and_then(|k| k.as_array()) {
            *keys += arr.len();
        }
    } else if p.get("x").is_some() || p.get("y").is_some() {
        // Vec2Axes
        count_json_param(p.get("x"), keys);
        count_json_param(p.get("y"), keys);
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use crate::schema::ItemEnvelope;
    use crate::{
        Clip, ClipSource, DocParam, Document, Group, PathOp, TrackItem, VectorContent, VectorRecipe,
    };
    use motolii_core::{RationalTime, TimeMap};

    #[test]
    fn counts_include_vector_modifiers_and_nested_groups() {
        let mut doc = Document::new_v1();
        let layer_a = doc.layers.allocate("a").unwrap();
        let layer_g = doc.layers.allocate("g").unwrap();
        let layer_c = doc.layers.allocate("c").unwrap();
        let tid = doc.track_ids.allocate("V1").unwrap();

        let mut keys = crate::DocKeyframeTrack::new();
        keys.insert(crate::DocKeyframe {
            id: crate::KeyframeId::from_raw(0),
            t: RationalTime::ZERO,
            value: crate::DocValue::F64(0.0),
            interp: motolii_eval::Interp::Linear,
        });
        keys.insert(crate::DocKeyframe {
            id: crate::KeyframeId::from_raw(1),
            t: RationalTime::try_new(1, 1).unwrap(),
            value: crate::DocValue::F64(1.0),
            interp: motolii_eval::Interp::Hold,
        });
        doc.next_stable_id = {
            let mut seq = crate::StableIdSeq::new();
            let _ = seq.allocate();
            let _ = seq.allocate();
            seq
        };

        let nested = Clip {
            envelope: {
                let mut e = ItemEnvelope::new(layer_c);
                e.opacity = DocParam::Keyframes(keys);
                e
            },
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: TimeMap::identity(),
            source: ClipSource::Vector {
                recipe: VectorRecipe {
                    content: VectorContent::StandardShape {
                        shape: crate::schema::StandardShape::Rect {
                            width: DocParam::const_f64(1.0),
                            height: DocParam::const_f64(1.0),
                        },
                    },
                    modifiers: vec![PathOp::Offset {
                        distance: DocParam::const_f64(0.1),
                        line_join: Default::default(),
                        miter_limit: 4.0,
                    }],
                },
            },
        };
        let top = Clip {
            envelope: ItemEnvelope::new(layer_a),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: TimeMap::identity(),
            source: ClipSource::asset_video_only(crate::AssetId::from_raw(0)),
        };
        doc.tracks.push(crate::Track {
            id: tid,
            items: vec![
                TrackItem::Clip(top),
                TrackItem::Group(Group {
                    envelope: ItemEnvelope::new(layer_g),
                    children: vec![TrackItem::Clip(nested)],
                }),
            ],
        });

        let counts = count_document(&doc);
        assert_eq!(counts.track_count, 1);
        assert_eq!(counts.clip_count, 2);
        assert_eq!(counts.keyframe_count, 2);
    }
}
