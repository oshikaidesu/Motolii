use super::*;
use serde_json::{json, Value as Json};

/// Relations v0: one source property, normalised over an input range, drives one property of several things over an
/// output range. Nothing new is stored: each member takes a `PropertyLink` (the document's retained binding, which
/// replaces the property's own value) with the `motolii.link.remap` translation. A relation is those links seen
/// together; a member set is their destinations. One `relate` is one undo step.
impl EditorRuntime {
    pub(super) fn relate(&mut self, j: &J) -> Result<(), String> {
        let source_layer = LayerId(j["source"]["layer"].as_u64().ok_or("Missing source layer")?);
        let source_property = PropertyId::new(string(&j["source"], "property")?).map_err(e)?;
        let component = j["source"]["component"].as_u64().unwrap_or(0) as f64;
        let (in_min, in_max) = (number(j, "inMin")?, number(j, "inMax")?);
        let (out_min, out_max) = (number(j, "outMin")?, number(j, "outMax")?);
        if !(in_min.is_finite() && in_max.is_finite() && out_min.is_finite() && out_max.is_finite()) {
            return Err("Ranges must be numbers".into());
        }
        if (in_max - in_min).abs() < 1e-9 {
            return Err("The source range is empty".into());
        }
        let destination = PropertyId::new(string(j, "property")?).map_err(e)?;
        let members = ids(&j["members"])?;
        if members.is_empty() {
            return Err("Choose at least one thing".into());
        }
        let view = self.doc.view().without_transients();
        let at = self.time()?;
        let mut intents = Vec::new();
        // The source has to be a property of its own for the renderer's graph to read it: a value never written is
        // only a default. Pinning the current value changes nothing the eye can see.
        if view.property_source(source_layer, &source_property).map_err(e)?.is_none() {
            let current = view.value_at(source_layer, &source_property, at).map_err(e)?
                .or(motolii_edit::document::edit::default_value(&view, source_layer, &source_property).map_err(e)?)
                .ok_or("The source has no value")?;
            intents.push(Intent::SetConstant { layer: source_layer, property: source_property.clone(), value: current });
        }
        for &member in &members {
            if member == source_layer && destination == source_property {
                return Err("A property cannot drive itself".into());
            }
            let shape = view.value_at(member, &destination, at).map_err(e)?
                .or(motolii_edit::document::edit::default_value(&view, member, &destination).map_err(e)?);
            let out_components = match shape {
                Some(Value::Vec2(_)) => 2.0,
                Some(Value::F64(_)) => 1.0,
                _ => return Err(format!("{} is not a number on layer {}", destination.name(), member.0)),
            };
            intents.push(Intent::SetPropertyLink {
                layer: member,
                property: destination.clone(),
                link: PropertyLink {
                    source_layer,
                    source_property: source_property.clone(),
                    time_offset: RationalTime::ZERO,
                    plugin_id: "motolii.link.remap".into(),
                    params: vec![
                        ("in_min".into(), Value::F64(in_min)),
                        ("in_max".into(), Value::F64(in_max)),
                        ("out_min".into(), Value::F64(out_min)),
                        ("out_max".into(), Value::F64(out_max)),
                        ("clamp".into(), Value::Bool(true)),
                        ("in_component".into(), Value::F64(component)),
                        ("out_components".into(), Value::F64(out_components)),
                    ],
                },
            });
        }
        drop(view);
        if j["preview"] == true { self.set_preview(intents) } else { self.apply(intents) }
    }

    /// The links come off and each thing keeps the value it shows now, as its own.
    pub(super) fn unrelate(&mut self, j: &J) -> Result<(), String> {
        let destination = PropertyId::new(string(j, "property")?).map_err(e)?;
        let members = ids(&j["layers"])?;
        let view = self.doc.view().without_transients();
        let at = self.time()?;
        let mut intents = Vec::new();
        for &member in &members {
            let Some(source) = view.property_source(member, &destination).map_err(e)? else { continue };
            if source.as_link_only().is_none() { continue }
            let current = view.value_at(member, &destination, at).map_err(e)?
                .or(motolii_edit::document::edit::default_value(&view, member, &destination).map_err(e)?)
                .ok_or("No value to keep")?;
            intents.push(Intent::SetConstant { layer: member, property: destination.clone(), value: current });
        }
        drop(view);
        if intents.is_empty() { return Ok(()) }
        self.apply(intents)
    }
}

/// The link a property wears, as the status says it (Flutter and a script read the same fact).
pub(crate) fn link_json(link: &PropertyLink) -> Json {
    let p = |name: &str| link.params.iter().find(|(n, _)| n == name).and_then(|(_, v)| match v { Value::F64(v) => Some(*v), _ => None });
    json!({
        "layer": link.source_layer.0, "property": link.source_property.name(), "component": p("in_component").unwrap_or(0.0) as i64,
        "kind": link.plugin_id, "inMin": p("in_min"), "inMax": p("in_max"), "outMin": p("out_min"), "outMax": p("out_max"),
    })
}

fn number(j: &J, key: &str) -> Result<f64, String> {
    j[key].as_f64().ok_or_else(|| format!("Missing {key}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rig() -> (EditorRuntime, LayerId, Vec<LayerId>) {
        let mut rt = crate::EditorRuntime::open("").unwrap();
        rt.request(json!({"op": "create", "kind": "null"})).unwrap();
        let origin = rt.viewer.selected().unwrap();
        let mut circles = Vec::new();
        for _ in 0..4 {
            rt.request(json!({"op": "create", "kind": "ellipse"})).unwrap();
            circles.push(rt.viewer.selected().unwrap());
        }
        rt.request(json!({"op": "setProperty", "layer": origin.0, "property": "position", "value": [500.0, 300.0]})).unwrap();
        (rt, origin, circles)
    }

    fn scale(rt: &EditorRuntime, l: LayerId) -> f64 {
        match rt.doc.view().value_at(l, &PropertyId::new(property::SCALE).unwrap(), rt.time().unwrap()).unwrap() {
            Some(Value::Vec2(v)) => v[0],
            other => panic!("{other:?}"),
        }
    }
    fn rotation(rt: &EditorRuntime, l: LayerId) -> f64 {
        match rt.doc.view().value_at(l, &PropertyId::new(property::ROTATION).unwrap(), rt.time().unwrap()).unwrap() {
            Some(Value::F64(v)) => v,
            None => 0.0,
            other => panic!("{other:?}"),
        }
    }
    fn relate(rt: &mut EditorRuntime, origin: LayerId, members: &[LayerId], prop: &str, lo: f64, hi: f64) {
        rt.request(json!({"op": "relate", "source": {"layer": origin.0, "property": "position", "component": 0}, "inMin": 200.0, "inMax": 800.0,
            "members": members.iter().map(|m| m.0).collect::<Vec<_>>(), "property": prop, "outMin": lo, "outMax": hi})).unwrap();
    }

    /// The acceptance fixture, on the document: Null.X 200..800 drives four circles' Scale 0.5..1.5, then Rotation -30..30
    /// from the same source; endpoints, clamping, a reversed range, undo and redo.
    #[test]
    fn null_x_drives_four_circles_scale_and_rotation() {
        let (mut rt, origin, circles) = rig();
        relate(&mut rt, origin, &circles, "scale", 0.5, 1.5);
        for &c in &circles { assert!((scale(&rt, c) - 1.0).abs() < 1e-9, "500 is halfway: 1.0"); }
        let put = |rt: &mut EditorRuntime, x: f64| rt.request(json!({"op": "setProperty", "layer": origin.0, "property": "position", "value": [x, 300.0]})).unwrap();
        put(&mut rt, 200.0);
        assert!((scale(&rt, circles[0]) - 0.5).abs() < 1e-9);
        put(&mut rt, 800.0);
        assert!((scale(&rt, circles[3]) - 1.5).abs() < 1e-9);
        put(&mut rt, 2000.0);
        assert!((scale(&rt, circles[1]) - 1.5).abs() < 1e-9, "clamped outside the source range");
        put(&mut rt, 350.0);
        relate(&mut rt, origin, &circles, "rotation", -30.0, 30.0);
        assert!((rotation(&rt, circles[2]) - (-15.0)).abs() < 1e-9, "the same source drives a second property: {}", rotation(&rt, circles[2]));
        assert!((scale(&rt, circles[2]) - 0.75).abs() < 1e-9);
        relate(&mut rt, origin, &circles[..2], "opacity", 1.0, 0.2);
        assert!((rt.doc.view().value_at(circles[0], &PropertyId::new(property::OPACITY).unwrap(), rt.time().unwrap()).unwrap().map(|v| match v { Value::F64(v) => v, _ => f64::NAN }).unwrap() - 0.8).abs() < 1e-9, "a reversed output range");
        rt.request(json!({"op": "undo"})).unwrap();
        rt.request(json!({"op": "undo"})).unwrap();
        assert!((rotation(&rt, circles[2]) - 0.0).abs() < 1e-9, "one relate is one step");
        rt.request(json!({"op": "redo"})).unwrap();
        assert!((rotation(&rt, circles[2]) - (-15.0)).abs() < 1e-9);
        // The status says the link on each destination, and the source's move is a plain preview: no drift.
        let status = rt.status().unwrap();
        let row = status["layers"].as_array().unwrap().iter().find(|l| l["id"] == circles[0].0).unwrap()["properties"].as_array().unwrap().iter().find(|p| p["id"] == "scale").unwrap().clone();
        assert_eq!(row["link"]["layer"], origin.0);
        assert_eq!(row["link"]["property"], "position");
        assert_eq!(row["link"]["outMax"], 1.5);
        for x in [300.0, 400.0, 300.0] {
            rt.request(json!({"op": "previewProperties", "edits": [{"layer": origin.0, "property": "position", "value": [x, 300.0]}]})).unwrap();
        }
        rt.request(json!({"op": "commitPreview"})).unwrap();
        assert!((scale(&rt, circles[0]) - (0.5 + 100.0 / 600.0)).abs() < 1e-9, "the value follows the source, never accumulates");
    }

    /// Unrelate keeps what each thing shows; a deleted source leaves the members at their own values; a deleted member
    /// takes its link with it; the relation survives save and reopen.
    #[test]
    fn relations_are_kept_on_disk_and_come_off_cleanly() {
        let (mut rt, origin, circles) = rig();
        relate(&mut rt, origin, &circles, "scale", 0.5, 1.5);
        rt.request(json!({"op": "setProperty", "layer": origin.0, "property": "position", "value": [650.0, 300.0]})).unwrap();
        let dir = std::env::temp_dir().join(format!("motolii-relate-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("work.rrd");
        rt.request(json!({"op": "save", "path": file.to_string_lossy()})).unwrap();
        let again = EditorRuntime::open(&file.to_string_lossy()).unwrap();
        let ids: Vec<LayerId> = again.doc.view().layers().to_vec();
        let reopened = ids.iter().copied().filter(|l| again.doc.view().property_source(*l, &PropertyId::new(property::SCALE).unwrap()).unwrap().is_some_and(|s| s.as_link_only().is_some())).count();
        assert_eq!(reopened, 4, "four links reopened");
        assert!((scale(&again, ids[1]) - 1.25).abs() < 1e-9);
        let _ = std::fs::remove_dir_all(&dir);

        rt.request(json!({"op": "unrelate", "layers": [circles[0].0], "property": "scale"})).unwrap();
        assert!((scale(&rt, circles[0]) - 1.25).abs() < 1e-9, "keeps the shown value as its own");
        assert!(rt.doc.view().property_source(circles[0], &PropertyId::new(property::SCALE).unwrap()).unwrap().unwrap().as_link_only().is_none());
        rt.request(json!({"op": "select", "ids": [circles[1].0]})).unwrap();
        rt.request(json!({"op": "delete"})).unwrap();
        assert!(!rt.doc.view().has_layer(circles[1]), "a member can go");
        assert!((scale(&rt, circles[2]) - 1.25).abs() < 1e-9, "the others still follow");
        rt.request(json!({"op": "select", "ids": [origin.0]})).unwrap();
        rt.request(json!({"op": "delete"})).unwrap();
        let after = rt.doc.view().value_at(circles[2], &PropertyId::new(property::SCALE).unwrap(), rt.time().unwrap()).unwrap();
        assert!(matches!(after, None | Some(Value::Vec2(_))), "a gone source leaves a value or a default, never an error: {after:?}");
        let _ = rt.status().unwrap();
    }

    #[test]
    fn a_relation_refuses_an_empty_range_a_self_link_and_a_cycle() {
        let (mut rt, origin, circles) = rig();
        assert!(rt.request(json!({"op": "relate", "source": {"layer": origin.0, "property": "position"}, "inMin": 1.0, "inMax": 1.0, "members": [circles[0].0], "property": "scale", "outMin": 0.0, "outMax": 1.0})).is_err());
        assert!(rt.request(json!({"op": "relate", "source": {"layer": origin.0, "property": "opacity"}, "inMin": 0.0, "inMax": 1.0, "members": [origin.0], "property": "opacity", "outMin": 0.0, "outMax": 1.0})).is_err());
        relate(&mut rt, origin, &circles[..1], "scale", 0.5, 1.5);
        let back = rt.request(json!({"op": "relate", "source": {"layer": circles[0].0, "property": "scale"}, "inMin": 0.0, "inMax": 1.0, "members": [origin.0], "property": "position", "outMin": 0.0, "outMax": 1.0}));
        assert!(back.is_err(), "circle.scale → origin.position → circle.scale is a cycle");
    }
}
