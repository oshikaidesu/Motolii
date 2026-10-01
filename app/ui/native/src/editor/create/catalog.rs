//! The shelf of the host: what can be created, and which shelf entries are the host's to understand.
use super::{background, primitive, primitives, NewKind};

/// What can be created, once: the id a frontend sends, the name and the line it shows, the shelf it is filed on, and what
/// it makes. The Browser, a script and the `create` operation all read this table; a kind added here exists everywhere.
/// The bundled 3D bodies and backgrounds join it from their own tables.
pub(crate) fn kinds() -> Vec<(&'static str, &'static str, &'static str, &'static str, NewKind)> {
    vec![
        ("text", "Text", "Adds a text layer", "Text", NewKind::Text),
        ("rectangle", "Rectangle", "Adds a shape layer", "Shapes", NewKind::Rectangle),
        ("roundedRectangle", "Rounded Rectangle", "Adds a shape layer", "Shapes", NewKind::RoundedRectangle),
        ("ellipse", "Ellipse", "Adds a shape layer", "Shapes", NewKind::Ellipse),
        ("star", "Star", "Adds a shape layer", "Shapes", NewKind::Star),
        ("polygon", "Polygon", "Adds a shape layer", "Shapes", NewKind::Polygon),
        ("null", "Null", "Adds an empty layer to parent others to", "Helpers", NewKind::Null),
        ("camera", "Camera", "Adds a camera layer", "3D", NewKind::Camera),
        ("particles", "Particles", "Adds a particle emitter", "Other", NewKind::Particles),
        ("stage", "Stage", "Widens the working area around the frame", "3D", NewKind::Stage),
        ("line", "Line", "Adds a straight stroked path", "Paths", NewKind::Line),
        ("bezier", "Bezier", "Adds a path layer", "Paths", NewKind::Bezier),
    ]
}

/// Who owns a shelf entry. `host`: the host has to understand it (members, their order, their identity, their time), so it
/// is a Create capability. `effect`: it turns an input picture into another (WGSL), so it is an Effect. The placement seat
/// (Repeater, Mirror: one source into addressable copies) is the host's; every other seat is an effect.
pub(crate) fn owner_of(stage: &str) -> &'static str {
    if stage == "Placement" { "host" } else { "effect" }
}

/// The host capabilities a frontend lists beside the kinds it can create: applied to a layer with `applyEffect`, as always,
/// stored where they always were. Only where they are found changes.
pub(crate) fn host_capabilities(catalog: &[crate::render::compositor::EffectDescriptor]) -> serde_json::Value {
    serde_json::Value::Array(
        catalog
            .iter()
            .filter(|e| owner_of(&format!("{:?}", e.stage)) == "host")
            .map(|e| {
                let detail = match e.plugin_id.as_str() {
                    crate::render::extensions::placement::REPEAT => "Copies of the layer, along a line, a circle or a grid",
                    crate::render::extensions::placement::MIRROR => "The layer and its mirror images",
                    _ => "Copies of the layer",
                };
                serde_json::json!({"id": e.plugin_id, "name": e.label, "detail": detail, "rail": "Copies", "apply": "applyEffect"})
            })
            .collect(),
    )
}

/// The kind a `create` names: a plain kind, a bundled background (`background:<id>`) or a bundled 3D body.
pub(crate) fn kind_named(id: &str) -> Result<NewKind, String> {
    if let Some((_, _, _, _, kind)) = kinds().into_iter().find(|k| k.0 == id) {
        return Ok(kind);
    }
    match id.strip_prefix("background:") {
        Some(background_id) => background(background_id),
        None => primitive(id),
    }
}

/// The table as a frontend reads it (status `createKinds`), bundled 3D bodies included.
pub(crate) fn kinds_json() -> serde_json::Value {
    let mut out: Vec<serde_json::Value> = kinds().into_iter().map(|(id, name, detail, rail, _)| serde_json::json!({"id": id, "name": name, "detail": detail, "rail": rail})).collect();
    out.extend(primitives().iter().map(|p| serde_json::json!({"id": p.id, "name": p.name, "detail": format!("Adds a 3D {}", p.name.to_lowercase()), "rail": "3D"})));
    serde_json::Value::Array(out)
}

#[cfg(test)]
mod tests {
    /// Every kind the table lists is one `create` accepts, and the table is what the status publishes.
    #[test]
    fn every_listed_kind_can_be_created_and_is_published() {
        for (id, ..) in super::kinds() {
            assert!(super::kind_named(id).is_ok(), "{id}");
        }
        let published = super::kinds_json();
        assert!(published.as_array().unwrap().iter().any(|k| k["id"] == "rectangle" && k["rail"] == "Shapes"));
        assert!(published.as_array().unwrap().iter().any(|k| k["id"] == "cube" && k["rail"] == "3D"));
        assert!(super::kind_named("nonsense").is_err());
    }

    /// The placement seat is the host's (Create); every picture seat is an effect. The shelf entry keeps its id, so a
    /// layer that carries a Repeater and the Script name `effect("Repeater")` are the same as before.
    #[test]
    fn repeater_and_mirror_are_host_capabilities_and_pictures_are_effects() {
        assert_eq!(super::owner_of("Placement"), "host");
        assert_eq!(super::owner_of("Pass"), "effect");
        let catalog = crate::render::engine::known_effects();
        let host = super::host_capabilities(&catalog);
        let ids: Vec<&str> = host.as_array().unwrap().iter().map(|c| c["id"].as_str().unwrap()).collect();
        assert!(ids.contains(&"motolii.repeat") && ids.contains(&"motolii.mirror"), "{ids:?}");
        assert!(!ids.contains(&"motolii.blur"));
        assert!(host.as_array().unwrap().iter().all(|c| c["apply"] == "applyEffect" && c["rail"] == "Copies"));
    }
}
