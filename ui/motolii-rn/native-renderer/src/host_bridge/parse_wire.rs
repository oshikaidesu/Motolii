use super::json_scan::{
    find_key_object, find_matching_brace, json_bool_value, json_f64_value, json_rational,
    json_string_value, parse_optional_vec2,
};
use super::parse_catalog::parse_stage_geometry;
use super::slot::{slice_from_written, MAX_SNAPSHOT_JSON_BYTES};
use super::types::{
    HostTimelineEffect, HostTimelineEffectParam, HostTimelineKey, HostTimelineLayer,
    HostTimelineProjection, HostTimelineSourceParam,
};

#[cfg(target_os = "macos")]
use motolii_ui::motolii_rn_host_read_snapshot_json;

pub(super) fn parse_timeline_projection(json: &str) -> Option<HostTimelineProjection> {
    let host_handle = json_string_value(json, "host_handle");
    let revision = json_string_value(json, "revision")?;
    let projection_generation =
        json_string_value(json, "projection_generation").unwrap_or_else(|| "0".into());
    let primary_layer_id = json_string_value(json, "primary_layer_id");
    let current_time = json_rational(json, "current_time").unwrap_or((0, 1));
    let timeline_duration =
        find_key_object(json, "timeline").and_then(|timeline| json_rational(timeline, "duration"));
    let fps = find_key_object(json, "timeline").and_then(|timeline| json_rational(timeline, "fps"));
    let bounds = parse_bounds(json)?;
    let timeline_layers = parse_timeline_layers(json);
    // 壊れていたら stage_geometry 全体を None へ（timeline は維持）。
    let stage_geometry = parse_stage_geometry(json);
    Some(HostTimelineProjection {
        host_handle,
        revision,
        projection_generation,
        primary_layer_id,
        current_time,
        timeline_duration,
        fps,
        bounds,
        timeline_layers,
        stage_geometry,
    })
}

pub(crate) fn snapshot_layers_from_projection(
    projection: &HostTimelineProjection,
) -> Vec<crate::timeline_skia::SnapshotLayerInput> {
    use crate::timeline_skia::{SnapshotKeyInput, SnapshotLayerInput};
    if let Some(layers) = &projection.timeline_layers {
        return layers
            .iter()
            .map(|layer| SnapshotLayerInput {
                layer_id: layer.layer_id.clone(),
                display_name: layer.display_name.clone(),
                interval_secs: Some((layer.start_secs, layer.duration_secs)),
                keys: layer
                    .position_keys
                    .iter()
                    .chain(layer.param_keys.iter())
                    .map(|key| SnapshotKeyInput {
                        key_id: key.key_id,
                        time_secs: key.time_secs,
                    })
                    .collect(),
            })
            .collect();
    }
    projection
        .bounds
        .iter()
        .map(|(layer_id, display_name)| SnapshotLayerInput {
            layer_id: layer_id.clone(),
            display_name: display_name.clone(),
            interval_secs: None,
            keys: Vec::new(),
        })
        .collect()
}

fn parse_bounds(json: &str) -> Option<Vec<(String, String)>> {
    let marker = "\"bounds\"";
    let at = json.find(marker)?;
    let after = json[at + marker.len()..].trim_start().strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('[')?;
    let mut bounds = Vec::new();
    let mut rest = after;
    loop {
        rest = rest.trim_start();
        if rest.starts_with(']') {
            break;
        }
        if rest.starts_with(',') {
            rest = rest[1..].trim_start();
            continue;
        }
        if !rest.starts_with('{') {
            return None;
        }
        let end = find_matching_brace(rest)?;
        let obj = &rest[..=end];
        let layer_id = json_string_value(obj, "layer_id")?.to_owned();
        let display_name = json_string_value(obj, "display_name")?.to_owned();
        bounds.push((layer_id, display_name));
        rest = &rest[end + 1..];
    }
    Some(bounds)
}

fn parse_timeline_layers(json: &str) -> Option<Vec<HostTimelineLayer>> {
    let timeline = find_key_object(json, "timeline")?;
    let marker = "\"layers\"";
    let at = timeline.find(marker)?;
    let after = timeline[at + marker.len()..]
        .trim_start()
        .strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('[')?;
    let mut layers = Vec::new();
    let mut rest = after;
    loop {
        rest = rest.trim_start();
        if rest.starts_with(']') {
            break;
        }
        if rest.starts_with(',') {
            rest = rest[1..].trim_start();
            continue;
        }
        if !rest.starts_with('{') {
            return None;
        }
        let end = find_matching_brace(rest)?;
        let obj = &rest[..=end];
        let layer_id = json_string_value(obj, "layer_id")?.to_owned();
        let display_name = json_string_value(obj, "display_name")?.to_owned();
        let (start_num, start_den) = json_rational(obj, "start")?;
        let (duration_num, duration_den) = json_rational(obj, "duration")?;
        if start_den == 0 || duration_den == 0 {
            return None;
        }
        let position_keys = parse_position_keys(obj)?;
        // param_keys 欠落・壊れは空。layer 自体は落とさない。
        let param_keys = parse_param_keys(obj).unwrap_or_default();
        // effects 欠落は空。壊れ値は空へ fallback（layer 自体は落とさない）。
        let (effects, effects_truncated) = parse_layer_effects(obj).unwrap_or_default();
        let (source_params, source_params_truncated) =
            parse_layer_source_params(obj).unwrap_or_default();
        layers.push(HostTimelineLayer {
            layer_id,
            display_name,
            start_secs: start_num as f64 / start_den as f64,
            duration_secs: duration_num as f64 / duration_den as f64,
            position_keys,
            param_keys,
            effects,
            effects_truncated,
            source_params,
            source_params_truncated,
            visible: json_bool_value(obj, "visible").unwrap_or(true),
            solo: json_bool_value(obj, "solo").unwrap_or(false),
        });
        rest = &rest[end + 1..];
    }
    Some(layers)
}

fn parse_layer_effects(layer_obj: &str) -> Option<(Vec<HostTimelineEffect>, bool)> {
    let marker = "\"effects\"";
    let Some(at) = layer_obj.find(marker) else {
        return Some((Vec::new(), false));
    };
    let after = layer_obj[at + marker.len()..]
        .trim_start()
        .strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('[')?;
    let effects_truncated = json_bool_value(layer_obj, "effects_truncated").unwrap_or(false);
    let mut effects = Vec::new();
    let mut rest = after;
    loop {
        rest = rest.trim_start();
        if rest.starts_with(']') {
            break;
        }
        if rest.starts_with(',') {
            rest = rest[1..].trim_start();
            continue;
        }
        if !rest.starts_with('{') {
            return None;
        }
        let end = find_matching_brace(rest)?;
        let obj = &rest[..=end];
        let effect_use_id = json_string_value(obj, "effect_use_id")?.to_owned();
        let plugin_id = json_string_value(obj, "plugin_id")?.to_owned();
        let params = parse_effect_params(obj)?;
        effects.push(HostTimelineEffect {
            effect_use_id,
            plugin_id,
            params,
        });
        rest = &rest[end + 1..];
    }
    Some((effects, effects_truncated))
}

fn parse_layer_source_params(layer_obj: &str) -> Option<(Vec<HostTimelineSourceParam>, bool)> {
    let marker = "\"source_params\"";
    let Some(at) = layer_obj.find(marker) else {
        return Some((Vec::new(), false));
    };
    let after = layer_obj[at + marker.len()..]
        .trim_start()
        .strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('[')?;
    let source_params_truncated =
        json_bool_value(layer_obj, "source_params_truncated").unwrap_or(false);
    let mut source_params = Vec::new();
    let mut rest = after;
    loop {
        rest = rest.trim_start();
        if rest.starts_with(']') {
            break;
        }
        if rest.starts_with(',') {
            rest = rest[1..].trim_start();
            continue;
        }
        if !rest.starts_with('{') {
            return None;
        }
        let end = find_matching_brace(rest)?;
        let obj = &rest[..=end];
        let param_id = json_string_value(obj, "param_id")?.to_owned();
        let value = json_f64_value(obj, "value")?;
        if !value.is_finite() {
            return None;
        }
        source_params.push(HostTimelineSourceParam { param_id, value });
        rest = &rest[end + 1..];
    }
    Some((source_params, source_params_truncated))
}

fn parse_effect_params(effect_obj: &str) -> Option<Vec<HostTimelineEffectParam>> {
    let marker = "\"params\"";
    let at = effect_obj.find(marker)?;
    let after = effect_obj[at + marker.len()..]
        .trim_start()
        .strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('[')?;
    let mut params = Vec::new();
    let mut rest = after;
    loop {
        rest = rest.trim_start();
        if rest.starts_with(']') {
            break;
        }
        if rest.starts_with(',') {
            rest = rest[1..].trim_start();
            continue;
        }
        if !rest.starts_with('{') {
            return None;
        }
        let end = find_matching_brace(rest)?;
        let obj = &rest[..=end];
        let param_id = json_string_value(obj, "param_id")?.to_owned();
        let value = json_f64_value(obj, "value")?;
        if !value.is_finite() {
            return None;
        }
        params.push(HostTimelineEffectParam { param_id, value });
        rest = &rest[end + 1..];
    }
    Some(params)
}

fn parse_position_keys(layer_obj: &str) -> Option<Vec<HostTimelineKey>> {
    parse_key_array(layer_obj, "\"position_keys\"")
}

fn parse_param_keys(layer_obj: &str) -> Option<Vec<HostTimelineKey>> {
    parse_key_array(layer_obj, "\"param_keys\"")
}

fn parse_key_array(layer_obj: &str, marker: &str) -> Option<Vec<HostTimelineKey>> {
    let at = layer_obj.find(marker)?;
    let after = layer_obj[at + marker.len()..]
        .trim_start()
        .strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('[')?;
    let mut keys = Vec::new();
    let mut rest = after;
    loop {
        rest = rest.trim_start();
        if rest.starts_with(']') {
            break;
        }
        if rest.starts_with(',') {
            rest = rest[1..].trim_start();
            continue;
        }
        if !rest.starts_with('{') {
            return None;
        }
        let end = find_matching_brace(rest)?;
        let obj = &rest[..=end];
        let key_id = json_string_value(obj, "key_id")?.parse::<u64>().ok()?;
        let (time_num, time_den) = json_rational(obj, "time")?;
        if time_den == 0 {
            return None;
        }
        let value = parse_optional_vec2(obj);
        keys.push(HostTimelineKey {
            key_id,
            time_secs: time_num as f64 / time_den as f64,
            value,
        });
        rest = &rest[end + 1..];
    }
    Some(keys)
}

#[cfg(target_os = "macos")]
pub(super) fn snapshot_has_position_key(handle: u64, layer_id: &str, key_id: u64) -> bool {
    let mut out = [0u8; MAX_SNAPSHOT_JSON_BYTES];
    let written =
        unsafe { motolii_rn_host_read_snapshot_json(handle, out.as_mut_ptr(), out.len()) };
    if written <= 0 {
        return false;
    }
    let Some(json_bytes) = slice_from_written(&out, written) else {
        return false;
    };
    let Ok(json) = std::str::from_utf8(json_bytes) else {
        return false;
    };
    let Some(proj) = parse_timeline_projection(json) else {
        return false;
    };
    let Some(layers) = proj.timeline_layers.as_deref() else {
        return false;
    };
    layer_has_position_key(layers, layer_id, key_id)
}

pub(super) fn layer_has_position_key(layers: &[HostTimelineLayer], layer_id: &str, key_id: u64) -> bool {
    layers.iter().any(|layer| {
        layer.layer_id == layer_id && layer.position_keys.iter().any(|key| key.key_id == key_id)
    })
}
