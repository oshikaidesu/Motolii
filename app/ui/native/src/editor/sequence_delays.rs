use crate::doc::store::*;

/// How a Sequence spreads its delay over the layers, chosen in the order they were picked: layer i of N is put through
/// the curve at i / (N - 1), and the last layer's delay is the whole spread: the largest ghost already there, or six
/// frames a step. The first layer has no delay. A desk sends the curve; what it means for the layers is decided here.
pub(crate) fn spread(view: &StoreView<'_>, layers: &[LayerId], shape: &serde_json::Value) -> Result<Vec<serde_json::Value>, String> {
    let curve = crate::editor::ease_kinds::decode(shape)?;
    let steps = layers.len().saturating_sub(1).max(1) as i64;
    let existing = layers.iter().filter_map(|&layer| view.attrs(layer).ok().flatten().and_then(|attrs| attrs.ghost)).max();
    let total = existing.unwrap_or(6 * steps) as f64;
    Ok((0..layers.len())
        .map(|i| {
            let u = i as f64 / steps as f64;
            // Hold waits, then the last layer takes the whole spread.
            let value = if matches!(curve, Interp::Hold) { if u >= 1.0 { 1.0 } else { 0.0 } } else { curve.ease(u) };
            serde_json::json!(((total * value).round() as i64).clamp(-(1 << 20), 1 << 20))
        })
        .collect())
}
