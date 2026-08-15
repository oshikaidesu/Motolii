use super::json_scan::{
    find_key_object, find_matching_brace, find_matching_bracket, is_finite_f32_compatible,
    json_bool_value, json_string_value, json_u32_value, parse_json_f64,
};
use super::types::{
    HostCatalogEffect, HostCatalogProjection, HostCatalogSource, HostStageGeometry,
    HostStageGeometryLayer,
};

pub(crate) fn parse_catalog_projection(json: &str) -> Option<HostCatalogProjection> {
    parse_catalog(json)
}

fn parse_catalog(json: &str) -> Option<HostCatalogProjection> {
    let obj = find_key_object(json, "catalog")?;
    let effects = parse_catalog_entries(obj, "effects")?;
    // sources 壊れは sources だけ空へ。effects / catalog 全体は落とさない。
    let sources = match obj.find("\"sources\"") {
        None => Vec::new(),
        Some(_) => parse_catalog_entries(obj, "sources")
            .map(|entries| {
                entries
                    .into_iter()
                    .map(|entry| HostCatalogSource {
                        plugin_id: entry.plugin_id,
                        name: entry.name,
                        effect_version: entry.effect_version,
                    })
                    .collect()
            })
            .unwrap_or_default(),
    };
    Some(HostCatalogProjection { effects, sources })
}

fn parse_catalog_entries(obj: &str, key: &str) -> Option<Vec<HostCatalogEffect>> {
    let marker = format!("\"{key}\"");
    let at = obj.find(&marker)?;
    let after = obj[at + marker.len()..].trim_start().strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('[')?;
    let mut entries = Vec::new();
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
        let entry_obj = &rest[..=end];
        let plugin_id = json_string_value(entry_obj, "plugin_id")?.to_owned();
        let name = json_string_value(entry_obj, "name")?.to_owned();
        let effect_version = json_u32_value(entry_obj, "effect_version")?;
        entries.push(HostCatalogEffect {
            plugin_id,
            name,
            effect_version,
        });
        rest = &rest[end + 1..];
    }
    Some(entries)
}

pub(super) fn parse_stage_geometry(json: &str) -> Option<HostStageGeometry> {
    let obj = find_key_object(json, "stage_geometry")?;
    let layers_truncated = json_bool_value(obj, "layers_truncated")?;
    let marker = "\"layers\"";
    let at = obj.find(marker)?;
    let after = obj[at + marker.len()..].trim_start().strip_prefix(':')?;
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
        let layer_obj = &rest[..=end];
        let layer_id = json_string_value(layer_obj, "layer_id")?;
        let corners = parse_corners(layer_obj)?;
        let position = parse_vec2_field(layer_obj, "position").unwrap_or([0.0, 0.0]);
        let rotation = json_f64_value(layer_obj, "rotation").unwrap_or(0.0);
        let scale = parse_vec2_field(layer_obj, "scale").unwrap_or([1.0, 1.0]);
        layers.push(HostStageGeometryLayer {
            layer_id,
            corners,
            position,
            rotation,
            scale,
        });
        rest = &rest[end + 1..];
    }
    Some(HostStageGeometry {
        layers,
        layers_truncated,
    })
}

fn parse_corners(layer_obj: &str) -> Option<[[f64; 2]; 4]> {
    let marker = "\"corners\"";
    let at = layer_obj.find(marker)?;
    let after = layer_obj[at + marker.len()..]
        .trim_start()
        .strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('[')?;
    let mut points = Vec::with_capacity(4);
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
        if !rest.starts_with('[') {
            return None;
        }
        let end = find_matching_bracket(rest)?;
        let pair = &rest[1..end];
        let (x, after_x) = parse_json_f64(pair)?;
        if !is_finite_f32_compatible(x) {
            return None;
        }
        let after_x = after_x.trim_start();
        if !after_x.starts_with(',') {
            return None;
        }
        let (y, after_y) = parse_json_f64(&after_x[1..])?;
        if !is_finite_f32_compatible(y) {
            return None;
        }
        if !after_y.trim_start().is_empty() {
            return None;
        }
        points.push([x, y]);
        rest = &rest[end + 1..];
    }
    if points.len() != 4 {
        return None;
    }
    Some([points[0], points[1], points[2], points[3]])
}

fn parse_vec2_field(layer_obj: &str, key: &str) -> Option<[f64; 2]> {
    let needle = format!("\"{key}\"");
    let at = layer_obj.find(&needle)?;
    let after = layer_obj[at + needle.len()..]
        .trim_start()
        .strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('[')?;
    let (x, rest) = parse_json_f64(after)?;
    if !is_finite_f32_compatible(x) {
        return None;
    }
    let rest = rest.trim_start().strip_prefix(',')?;
    let (y, _) = parse_json_f64(rest)?;
    if !is_finite_f32_compatible(y) {
        return None;
    }
    Some([x, y])
}
