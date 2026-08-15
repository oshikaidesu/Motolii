use std::collections::BTreeMap;

use motolii_core::RationalTime;
use serde_json::{json, Value};

use super::MigrateError;

const SVG_ASSET_TYPE: &str = "image/svg+xml";

pub(super) fn rewrite_legacy_shapes(
    root: &mut Value,
    steps: &mut Vec<&'static str>,
) -> Result<(), MigrateError> {
    let asset_types = collect_asset_types(root);
    let Some(tracks) = root.get_mut("tracks").and_then(|t| t.as_array_mut()) else {
        return Ok(());
    };
    for (ti, track) in tracks.iter_mut().enumerate() {
        let Some(items) = track.get_mut("items").and_then(|i| i.as_array_mut()) else {
            continue;
        };
        for (ii, item) in items.iter_mut().enumerate() {
            rewrite_item(
                item,
                &format!("tracks[{ti}].items[{ii}]"),
                &asset_types,
                steps,
            )?;
        }
    }
    Ok(())
}

fn collect_asset_types(root: &Value) -> BTreeMap<u64, String> {
    let mut out = BTreeMap::new();
    let Some(entries) = root
        .get("assets")
        .and_then(|a| a.get("entries"))
        .and_then(|e| e.as_array())
    else {
        return out;
    };
    for entry in entries {
        let Some(id) = entry.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        if let Some(ty) = entry.get("asset_type").and_then(|v| v.as_str()) {
            out.insert(id, ty.to_string());
        }
    }
    out
}

fn rewrite_item(
    item: &mut Value,
    path: &str,
    asset_types: &BTreeMap<u64, String>,
    steps: &mut Vec<&'static str>,
) -> Result<(), MigrateError> {
    let kind = item
        .get("kind")
        .and_then(|k| k.as_str())
        .unwrap_or("")
        .to_string();
    match kind.as_str() {
        "clip" => rewrite_clip(item, path, asset_types, steps),
        "group" => {
            if let Some(children) = item.get_mut("children").and_then(|c| c.as_array_mut()) {
                for (ci, child) in children.iter_mut().enumerate() {
                    rewrite_item(child, &format!("{path}.children[{ci}]"), asset_types, steps)?;
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn rewrite_clip(
    clip: &mut Value,
    path: &str,
    asset_types: &BTreeMap<u64, String>,
    steps: &mut Vec<&'static str>,
) -> Result<(), MigrateError> {
    let clip_start = clip.get("start").cloned();
    if let Some(tm) = clip.get_mut("time_map") {
        rewrite_timemap(tm, path, &clip_start, steps)?;
    }
    rewrite_path_ops(clip, path, asset_types, steps)?;
    Ok(())
}

fn rewrite_timemap(
    tm: &mut Value,
    path: &str,
    clip_start: &Option<Value>,
    steps: &mut Vec<&'static str>,
) -> Result<(), MigrateError> {
    let Value::Object(map) = tm else {
        return Ok(());
    };
    if !map.contains_key("timeline_start") {
        return Ok(());
    }
    let timeline_start_val =
        map.remove("timeline_start")
            .ok_or_else(|| MigrateError::TimeMapRewrite {
                path: path.to_string(),
                detail: "timeline_start missing after check".into(),
            })?;

    // 現行TimeMapは source_start/speed 必須。欠ければ補正も写像保存もできない。
    if !map.contains_key("source_start")
        || !map.contains_key("speed_num")
        || !map.contains_key("speed_den")
    {
        return Err(MigrateError::TimeMapRewrite {
            path: path.to_string(),
            detail: "legacy TimeMap missing source_start/speed fields".into(),
        });
    }

    let timeline_start: RationalTime =
        serde_json::from_value(timeline_start_val).map_err(|e| MigrateError::TimeMapRewrite {
            path: path.to_string(),
            detail: format!("invalid timeline_start: {e}"),
        })?;

    // clip.start が無いと clip_local 契約へ写せない — 警告黙殺せず拒否。
    let Some(clip_start_val) = clip_start.as_ref() else {
        return Err(MigrateError::TimeMapRewrite {
            path: path.to_string(),
            detail: "clip.start missing; cannot reconcile timeline_start".into(),
        });
    };
    let clip_start_rt: RationalTime =
        serde_json::from_value(clip_start_val.clone()).map_err(|e| {
            MigrateError::TimeMapRewrite {
                path: path.to_string(),
                detail: format!("invalid clip.start: {e}"),
            }
        })?;

    // 旧: source = source_start + (t - timeline_start) * speed
    // 新: source = source_start' + (t - clip.start) * speed
    // → source_start' = source_start + (clip.start - timeline_start) * speed
    if clip_start_rt != timeline_start {
        let source_start: RationalTime =
            serde_json::from_value(map.get("source_start").cloned().ok_or_else(|| {
                MigrateError::TimeMapRewrite {
                    path: path.to_string(),
                    detail: "source_start missing after check".into(),
                }
            })?)
            .map_err(|e| MigrateError::TimeMapRewrite {
                path: path.to_string(),
                detail: format!("invalid source_start: {e}"),
            })?;
        let speed_num = map
            .get("speed_num")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| MigrateError::TimeMapRewrite {
                path: path.to_string(),
                detail: "speed_num must be i64".into(),
            })?;
        let speed_den = map
            .get("speed_den")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| MigrateError::TimeMapRewrite {
                path: path.to_string(),
                detail: "speed_den must be i64".into(),
            })?;

        let corrected = adjust_source_start_for_clip_anchor(
            source_start,
            timeline_start,
            clip_start_rt,
            speed_num,
            speed_den,
        )
        .map_err(|detail| MigrateError::TimeMapRewrite {
            path: path.to_string(),
            detail,
        })?;
        map.insert(
            "source_start".into(),
            serde_json::to_value(corrected).map_err(|e| MigrateError::TimeMapRewrite {
                path: path.to_string(),
                detail: format!("serialize corrected source_start: {e}"),
            })?,
        );
        if !steps.contains(&"adjust_source_start_for_timeline_start") {
            steps.push("adjust_source_start_for_timeline_start");
        }
    }

    if !steps.contains(&"drop_timeline_start") {
        steps.push("drop_timeline_start");
    }
    Ok(())
}

/// 旧 timeline_start 基準写像を clip.start 基準へ移す source_start 補正。
fn adjust_source_start_for_clip_anchor(
    source_start: RationalTime,
    timeline_start: RationalTime,
    clip_start: RationalTime,
    speed_num: i64,
    speed_den: i64,
) -> Result<RationalTime, String> {
    let delta = clip_start
        .try_sub(timeline_start)
        .map_err(|e| format!("clip.start - timeline_start: {e}"))?;
    let scaled = delta
        .try_mul_i64(speed_num)
        .map_err(|e| format!("delta * speed_num: {e}"))?;
    let unit = RationalTime::try_new(1, speed_den).map_err(|e| format!("1/speed_den: {e}"))?;
    let mapped = scaled
        .try_mul(unit)
        .map_err(|e| format!("scaled * unit: {e}"))?;
    source_start
        .try_add(mapped)
        .map_err(|e| format!("source_start + offset: {e}"))
}

fn rewrite_path_ops(
    clip: &mut Value,
    path: &str,
    asset_types: &BTreeMap<u64, String>,
    steps: &mut Vec<&'static str>,
) -> Result<(), MigrateError> {
    let Value::Object(map) = clip else {
        return Ok(());
    };
    let Some(raw_ops) = map.remove("path_ops") else {
        return Ok(());
    };

    // null / 空配列はフィールド削除だけで現行へ。
    let ops_arr = match raw_ops {
        Value::Null => Vec::new(),
        Value::Array(a) => a,
        other => {
            return Err(MigrateError::PathOpsRewrite {
                path: path.to_string(),
                detail: format!("path_ops must be array or null, got {other}"),
            });
        }
    };

    let upgraded: Vec<Value> = ops_arr
        .into_iter()
        .map(|op| upgrade_legacy_path_op(op, path))
        .collect::<Result<_, _>>()?;

    if !steps.contains(&"move_path_ops_to_recipe") {
        steps.push("move_path_ops_to_recipe");
    }

    if upgraded.is_empty() {
        return Ok(());
    }

    let source = map
        .get_mut("source")
        .ok_or_else(|| MigrateError::PathOpsRewrite {
            path: path.to_string(),
            detail: "clip missing source while path_ops present".into(),
        })?;

    let tag = source
        .get("source")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();

    match tag.as_str() {
        "vector" => {
            let recipe = source
                .get_mut("recipe")
                .and_then(|r| r.as_object_mut())
                .ok_or_else(|| MigrateError::PathOpsRewrite {
                    path: path.to_string(),
                    detail: "vector source missing recipe object".into(),
                })?;
            let existing = recipe
                .remove("modifiers")
                .and_then(|m| match m {
                    Value::Array(a) => Some(a),
                    Value::Null => Some(Vec::new()),
                    _ => None,
                })
                .unwrap_or_default();
            let mut merged = upgraded;
            merged.extend(existing);
            recipe.insert("modifiers".into(), Value::Array(merged));
            Ok(())
        }
        "asset" => {
            let asset_id = source
                .get("asset")
                .and_then(|a| a.as_u64())
                .ok_or_else(|| MigrateError::PathOpsRewrite {
                    path: path.to_string(),
                    detail: "asset source missing asset id".into(),
                })?;
            let ty = asset_types.get(&asset_id).map(String::as_str).unwrap_or("");
            if ty != SVG_ASSET_TYPE {
                return Err(MigrateError::PathOpsOnRaster {
                    path: path.to_string(),
                    detail: format!(
                        "asset {asset_id} type `{ty}` cannot host path_ops; only {SVG_ASSET_TYPE} converts to Vector"
                    ),
                });
            }
            *source = json!({
                "source": "vector",
                "recipe": {
                    "content": {
                        "kind": "svg_asset",
                        "asset": asset_id
                    },
                    "modifiers": upgraded
                }
            });
            Ok(())
        }
        other => Err(MigrateError::PathOpsOnRaster {
            path: path.to_string(),
            detail: format!("source `{other}` cannot host path_ops"),
        }),
    }
}

fn upgrade_legacy_path_op(mut op: Value, path: &str) -> Result<Value, MigrateError> {
    let Value::Object(map) = &mut op else {
        return Err(MigrateError::PathOpsRewrite {
            path: path.to_string(),
            detail: "path_op must be object".into(),
        });
    };
    let op_name = map
        .get("op")
        .and_then(|o| o.as_str())
        .unwrap_or("")
        .to_string();

    match op_name.as_str() {
        "twist" => {
            // D1i-2: center必須。旧JSONは原点を注入(正準空間の形状中心既定)。
            if !map.contains_key("center") {
                map.insert("center".into(), json!({"const": {"Vec2": [0.0, 0.0]}}));
            }
        }
        "wiggle" => {
            if let Some(seed) = map.get("seed") {
                if seed.as_u64().is_none() && seed.as_i64().is_none() {
                    let as_u64 =
                        extract_seed_u64(seed).ok_or_else(|| MigrateError::PathOpsRewrite {
                            path: path.to_string(),
                            detail: format!("cannot coerce Wiggle.seed {seed} to u64"),
                        })?;
                    map.insert("seed".into(), json!(as_u64));
                }
            } else {
                map.insert("seed".into(), json!(0u64));
            }
        }
        _ => {}
    }
    Ok(op)
}

fn extract_seed_u64(seed: &Value) -> Option<u64> {
    if let Some(n) = seed.as_u64() {
        return Some(n);
    }
    if let Some(n) = seed.as_i64() {
        return u64::try_from(n).ok();
    }
    // DocParam::Const F64
    if let Some(v) = seed
        .get("const")
        .and_then(|c| c.get("F64"))
        .and_then(|f| f.as_f64())
    {
        if v.is_finite() && v >= 0.0 && v == v.trunc() {
            return Some(v as u64);
        }
    }
    None
}
