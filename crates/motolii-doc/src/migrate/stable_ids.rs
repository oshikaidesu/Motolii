use serde_json::{json, Value};

use crate::param::DocParam;
use crate::schema::{ClipSource, PathOp, TrackItem, VectorContent, VectorRecipe};
use crate::Document;

use super::MigrateError;

/// EffectInstance / DocKeyframe の欠落`id`をJSON段階で採番(D2必須。拒否→D1e変換)。
pub(super) fn inject_missing_stable_ids_json(root: &mut Value) -> Result<bool, MigrateError> {
    let mut next = root
        .get("next_stable_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let mut observed_max: Option<u64> = None;
    let mut injected = false;

    if let Some(tracks) = root.get_mut("tracks").and_then(|t| t.as_array_mut()) {
        for track in tracks.iter_mut() {
            let Some(items) = track.get_mut("items").and_then(|i| i.as_array_mut()) else {
                continue;
            };
            for item in items.iter_mut() {
                if inject_ids_in_item(item, &mut next, &mut observed_max)? {
                    injected = true;
                }
            }
        }
    }

    if inject_ids_in_effect_definitions(root, &mut next, &mut observed_max)? {
        injected = true;
    }

    // カウンタ整合はinjected(新規採番の有無)と無関係に行う: 既存idだけでも
    // 観測した既存idの最大値がカウンタ以上なら追い越して書き戻す。
    let mut counter_updated = false;
    if let Some(max_id) = observed_max {
        if next <= max_id {
            next = max_id
                .checked_add(1)
                .ok_or_else(|| MigrateError::StableId("stable id sequence exhausted".into()))?;
            counter_updated = true;
        }
    }
    if injected || counter_updated {
        if let Value::Object(map) = root {
            map.insert("next_stable_id".into(), json!(next));
        }
    }
    Ok(injected)
}

/// D1l: `root.effect_definitions[].params`内のキーフレームid欠落を採番する。
fn inject_ids_in_effect_definitions(
    root: &mut Value,
    next: &mut u64,
    observed_max: &mut Option<u64>,
) -> Result<bool, MigrateError> {
    let mut injected = false;
    if let Some(defs) = root
        .get_mut("effect_definitions")
        .and_then(|d| d.as_array_mut())
    {
        for def in defs.iter_mut() {
            let Value::Object(map) = def else {
                continue;
            };
            if let Some(id) = map.get("id").and_then(|v| v.as_u64()) {
                note_id(observed_max, id);
            }
            if let Some(params) = map.get_mut("params").and_then(|p| p.as_object_mut()) {
                for param in params.values_mut() {
                    if inject_ids_in_param(param, next, observed_max)? {
                        injected = true;
                    }
                }
            }
        }
    }
    Ok(injected)
}

fn inject_ids_in_item(
    item: &mut Value,
    next: &mut u64,
    observed_max: &mut Option<u64>,
) -> Result<bool, MigrateError> {
    let kind = item
        .get("kind")
        .and_then(|k| k.as_str())
        .unwrap_or("")
        .to_string();
    match kind.as_str() {
        "clip" => inject_ids_in_clip(item, next, observed_max),
        "group" => {
            let mut injected = false;
            if let Some(env) = item.get_mut("envelope") {
                if inject_ids_in_envelope(env, next, observed_max)? {
                    injected = true;
                }
            }
            if let Some(children) = item.get_mut("children").and_then(|c| c.as_array_mut()) {
                for child in children.iter_mut() {
                    if inject_ids_in_item(child, next, observed_max)? {
                        injected = true;
                    }
                }
            }
            Ok(injected)
        }
        _ => Ok(false),
    }
}

fn inject_ids_in_clip(
    clip: &mut Value,
    next: &mut u64,
    observed_max: &mut Option<u64>,
) -> Result<bool, MigrateError> {
    let mut injected = false;
    if let Some(env) = clip.get_mut("envelope") {
        if inject_ids_in_envelope(env, next, observed_max)? {
            injected = true;
        }
    }
    if let Some(source) = clip.get_mut("source") {
        if inject_ids_in_source(source, next, observed_max)? {
            injected = true;
        }
    }
    Ok(injected)
}

fn inject_ids_in_envelope(
    env: &mut Value,
    next: &mut u64,
    observed_max: &mut Option<u64>,
) -> Result<bool, MigrateError> {
    let mut injected = false;
    if let Some(xf) = env.get_mut("transform") {
        for key in ["position", "anchor", "scale", "rotation"] {
            if let Some(p) = xf.get_mut(key) {
                if inject_ids_in_param(p, next, observed_max)? {
                    injected = true;
                }
            }
        }
    }
    if let Some(opacity) = env.get_mut("opacity") {
        if inject_ids_in_param(opacity, next, observed_max)? {
            injected = true;
        }
    }
    if let Some(effects) = env.get_mut("effects").and_then(|e| e.as_array_mut()) {
        for effect in effects.iter_mut() {
            if let Value::Object(map) = effect {
                if !map.contains_key("id") {
                    let id = allocate_stable(next)?;
                    note_id(observed_max, id);
                    map.insert("id".into(), json!(id));
                    injected = true;
                } else if let Some(id) = map.get("id").and_then(|v| v.as_u64()) {
                    note_id(observed_max, id);
                }
                if let Some(params) = map.get_mut("params").and_then(|p| p.as_object_mut()) {
                    for param in params.values_mut() {
                        if inject_ids_in_param(param, next, observed_max)? {
                            injected = true;
                        }
                    }
                }
            }
        }
    }
    Ok(injected)
}

fn inject_ids_in_source(
    source: &mut Value,
    next: &mut u64,
    observed_max: &mut Option<u64>,
) -> Result<bool, MigrateError> {
    let tag = source
        .get("source")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    match tag.as_str() {
        "plugin" => {
            let mut injected = false;
            if let Some(params) = source.get_mut("params").and_then(|p| p.as_object_mut()) {
                for param in params.values_mut() {
                    if inject_ids_in_param(param, next, observed_max)? {
                        injected = true;
                    }
                }
            }
            Ok(injected)
        }
        "vector" => {
            let mut injected = false;
            if let Some(recipe) = source.get_mut("recipe") {
                if let Some(content) = recipe.get_mut("content") {
                    if inject_ids_in_vector_content(content, next, observed_max)? {
                        injected = true;
                    }
                }
                if let Some(mods) = recipe.get_mut("modifiers").and_then(|m| m.as_array_mut()) {
                    for op in mods.iter_mut() {
                        if inject_ids_in_path_op(op, next, observed_max)? {
                            injected = true;
                        }
                    }
                }
            }
            Ok(injected)
        }
        "asset" => {
            let mut injected = false;
            if let Some(audio) = source.get_mut("audio").and_then(|a| a.as_array_mut()) {
                for comp in audio.iter_mut() {
                    if let Some(gain) = comp.get_mut("gain") {
                        if inject_ids_in_param(gain, next, observed_max)? {
                            injected = true;
                        }
                    }
                }
            }
            Ok(injected)
        }
        _ => Ok(false),
    }
}

fn inject_ids_in_vector_content(
    content: &mut Value,
    next: &mut u64,
    observed_max: &mut Option<u64>,
) -> Result<bool, MigrateError> {
    let kind = content
        .get("kind")
        .and_then(|k| k.as_str())
        .unwrap_or("")
        .to_string();
    match kind.as_str() {
        "standard_shape" => {
            let mut injected = false;
            for key in ["width", "height"] {
                if let Some(p) = content.get_mut(key) {
                    if inject_ids_in_param(p, next, observed_max)? {
                        injected = true;
                    }
                }
            }
            Ok(injected)
        }
        "group" => {
            let mut injected = false;
            if let Some(children) = content.get_mut("children").and_then(|c| c.as_array_mut()) {
                for child in children.iter_mut() {
                    if inject_ids_in_vector_content(child, next, observed_max)? {
                        injected = true;
                    }
                }
            }
            Ok(injected)
        }
        _ => Ok(false),
    }
}

fn inject_ids_in_path_op(
    op: &mut Value,
    next: &mut u64,
    observed_max: &mut Option<u64>,
) -> Result<bool, MigrateError> {
    let mut injected = false;
    if let Value::Object(map) = op {
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
            if let Some(p) = map.get_mut(key) {
                if inject_ids_in_param(p, next, observed_max)? {
                    injected = true;
                }
            }
        }
        if let Some(xf) = map.get_mut("transform") {
            for key in ["position", "anchor", "scale", "rotation"] {
                if let Some(p) = xf.get_mut(key) {
                    if inject_ids_in_param(p, next, observed_max)? {
                        injected = true;
                    }
                }
            }
        }
    }
    Ok(injected)
}

fn inject_ids_in_param(
    param: &mut Value,
    next: &mut u64,
    observed_max: &mut Option<u64>,
) -> Result<bool, MigrateError> {
    let mut injected = false;
    if let Some(keys) = param
        .get_mut("keyframes")
        .and_then(|kf| kf.get_mut("keys"))
        .and_then(|k| k.as_array_mut())
    {
        for key in keys.iter_mut() {
            if let Value::Object(map) = key {
                if !map.contains_key("id") {
                    let id = allocate_stable(next)?;
                    note_id(observed_max, id);
                    map.insert("id".into(), json!(id));
                    injected = true;
                } else if let Some(id) = map.get("id").and_then(|v| v.as_u64()) {
                    note_id(observed_max, id);
                }
            }
        }
    }
    if param.get("x").is_some() || param.get("y").is_some() {
        if let Some(x) = param.get_mut("x") {
            if inject_ids_in_param(x, next, observed_max)? {
                injected = true;
            }
        }
        if let Some(y) = param.get_mut("y") {
            if inject_ids_in_param(y, next, observed_max)? {
                injected = true;
            }
        }
    }
    Ok(injected)
}

fn allocate_stable(next: &mut u64) -> Result<u64, MigrateError> {
    let id = *next;
    *next = next
        .checked_add(1)
        .ok_or_else(|| MigrateError::StableId("stable id sequence exhausted".into()))?;
    Ok(id)
}

fn note_id(observed_max: &mut Option<u64>, id: u64) {
    *observed_max = Some(observed_max.map_or(id, |m| m.max(id)));
}

pub(super) fn doc_has_stable_ids(doc: &Document) -> bool {
    fn walk_item(item: &TrackItem) -> bool {
        match item {
            TrackItem::Clip(clip) => {
                if !clip.envelope.effects.is_empty() {
                    return true;
                }
                param_has_keys(&clip.envelope.opacity)
                    || param_has_keys(&clip.envelope.transform.position)
                    || param_has_keys(&clip.envelope.transform.anchor)
                    || param_has_keys(&clip.envelope.transform.scale)
                    || param_has_keys(&clip.envelope.transform.rotation)
                    || match &clip.source {
                        ClipSource::Plugin { params, .. } => params.values().any(param_has_keys),
                        ClipSource::Vector { recipe } => recipe_has_keys(recipe),
                        ClipSource::Asset { audio, .. } => {
                            audio.iter().any(|comp| param_has_keys(&comp.gain))
                        }
                    }
            }
            TrackItem::Group(group) => {
                !group.envelope.effects.is_empty()
                    || param_has_keys(&group.envelope.opacity)
                    || group.children.iter().any(walk_item)
            }
        }
    }
    fn param_has_keys(p: &DocParam) -> bool {
        match p {
            DocParam::Keyframes(k) => !k.keys().is_empty(),
            DocParam::Vec2Axes { x, y } => param_has_keys(x) || param_has_keys(y),
            _ => false,
        }
    }
    fn recipe_has_keys(recipe: &VectorRecipe) -> bool {
        content_has_keys(&recipe.content)
            || recipe.modifiers.iter().any(|op| match op {
                PathOp::PuckerBloat { amount }
                | PathOp::Offset {
                    distance: amount, ..
                }
                | PathOp::RoundCorners { radius: amount } => param_has_keys(amount),
                PathOp::ZigZag { amount, ridges, .. } => {
                    param_has_keys(amount) || param_has_keys(ridges)
                }
                PathOp::Trim {
                    start, end, offset, ..
                } => param_has_keys(start) || param_has_keys(end) || param_has_keys(offset),
                PathOp::Twist { angle, center } => param_has_keys(angle) || param_has_keys(center),
                PathOp::Wiggle { amp, freq, .. } => param_has_keys(amp) || param_has_keys(freq),
                PathOp::Repeater {
                    copies,
                    offset,
                    transform,
                    start_opacity,
                    end_opacity,
                    ..
                } => {
                    param_has_keys(copies)
                        || param_has_keys(offset)
                        || param_has_keys(&transform.position)
                        || param_has_keys(start_opacity)
                        || param_has_keys(end_opacity)
                }
            })
    }
    fn content_has_keys(c: &VectorContent) -> bool {
        match c {
            VectorContent::StandardShape { shape } => match shape {
                crate::schema::StandardShape::Rect { width, height }
                | crate::schema::StandardShape::Ellipse { width, height } => {
                    param_has_keys(width) || param_has_keys(height)
                }
            },
            VectorContent::Group { children } => children.iter().any(content_has_keys),
            _ => false,
        }
    }
    !doc.effect_definitions.is_empty()
        || doc
            .tracks
            .iter()
            .flat_map(|t| t.items.iter())
            .any(walk_item)
}
