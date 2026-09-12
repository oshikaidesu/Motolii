//! 効果の札 = 作者の snapshot 画像(VST3 の Plug-in Snapshot と同じ置き方)。
use base64::Engine as _;
use serde_json::{json, Value as J};

/// `{"op":"visualSample","kind":"effect","id":plugin_id}` の返事。絵が無い効果は札を名前のままにする(error)。
pub(crate) fn reply(j: &J) -> Result<J, String> {
    let id = j["id"].as_str().ok_or("Missing effect id")?;
    let catalog = crate::render::engine::known_effects();
    let effect = catalog.iter().find(|d| d.plugin_id == id).ok_or("Unknown effect")?;
    let snapshot = effect.snapshot.as_ref().ok_or("No snapshot")?;
    Ok(json!({"image": base64::engine::general_purpose::STANDARD.encode(snapshot)}))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 棚の全部の札に絵がある(shader の隣の `<id>_snapshot.png`)。
    #[test]
    fn every_effect_on_the_shelf_has_a_snapshot() {
        let missing = crate::render::engine::known_effects().iter().filter(|d| d.snapshot.is_none()).map(|d| d.plugin_id.clone()).collect::<Vec<_>>();
        assert!(missing.is_empty(), "no snapshot beside the shader: {missing:?}");
        let gain = reply(&json!({"id": "motolii.gain"})).unwrap();
        assert!(gain["image"].as_str().is_some_and(|s| s.starts_with("iVBOR")), "a PNG, base64");
        assert!(reply(&json!({"id": "motolii.nope"})).is_err());
    }
}
