use super::*;
use serde_json::{json, Value as Json};

/// One edit put to the whole selection: the shown layer takes the value, and every other selected, unlocked layer that
/// has the property takes it in the way the edit says. The rule is the host's so that every frontend edits a selection
/// the same way; a frontend sends one edit and the gesture, never a value per layer.
///
/// `offset`   a drag: each layer keeps its own distance from the shown one (own + change), numbers and the numbers of a list.
/// `typed`    a typed value: only the axes that differ from the shown layer's committed value are set, the rest stay.
/// `absolute` every target takes the value as it is (a choice, a whole typed vector).
/// The change is read against the committed document, so a running preview never drifts.
impl EditorRuntime {
    pub(super) fn spread_edits(&self, j: &J) -> Result<Vec<Intent>, String> {
        let shown = layer(j)?;
        let name = string(j, "property")?;
        let mode = j["spread"].as_str().unwrap_or("absolute");
        let property = PropertyId::new(name).map_err(e)?;
        let view = self.doc.view().without_transients();
        let at = self.time()?;
        // An effect's parameter never set has no stored value: it is the effect's declared default (what the
        // Inspector shows), read from the catalog by the layer's effect in that slot.
        let catalog = crate::render::engine::known_effects();
        let declared = |l: LayerId| -> Option<Json> {
            let (slot, param) = name.strip_prefix(property::EFFECT_PREFIX)?.split_once(".param.")?;
            let plugin = view.effects(l).ok()?.into_iter().find(|fx| fx.id.to_string() == slot)?.plugin_id;
            let p = catalog.iter().find(|d| d.plugin_id == plugin)?.params.iter().find(|p| p.name == param)?.clone();
            Some(p.color.map(|c| json!(c)).or(p.point.map(|v| json!(v))).unwrap_or(json!(p.default)))
        };
        let committed = |l: LayerId| -> Option<Json> {
            let value = view.value_at(l, &property, at).ok().flatten().or_else(|| motolii_edit::document::edit::default_value(&view, l, &property).ok().flatten());
            value.map(|v| crate::snapshot::value(&v)).or_else(|| declared(l))
        };
        let base = committed(shown);
        let mut targets: Vec<LayerId> = self
            .viewer
            .selected_ids
            .iter()
            .copied()
            .filter(|&l| editor::functions::lens::edit_rejection(&view, l).ok().flatten().is_none())
            .collect();
        if !targets.contains(&shown) {
            targets = vec![shown];
        }
        // An effect's parameters are named by the layer's own effect id: on another layer the same name can belong to
        // a different effect. Only layers where it is the same kind of effect take the edit.
        if let Some(rest) = name.strip_prefix(property::EFFECT_PREFIX) {
            let slot = rest.split('.').next().unwrap_or_default();
            let kind = |l: LayerId| view.effects(l).ok().and_then(|list| list.into_iter().find(|fx| fx.id.to_string() == slot)).map(|fx| fx.plugin_id);
            let own = kind(shown);
            targets.retain(|&l| l == shown || (own.is_some() && kind(l) == own));
        }
        let mut out = Vec::new();
        for target in targets {
            let value = if target == shown {
                j["value"].clone()
            } else {
                let Some(own) = committed(target) else { continue };
                match (mode, &base) {
                    ("offset", Some(base)) => offset(base, &own, &j["value"]),
                    ("typed", Some(base)) => typed(base, &own, &j["value"]),
                    _ => j["value"].clone(),
                }
            };
            out.extend(self.property_edits(&json!({"layer": target.0, "property": name, "value": value}))?);
        }
        Ok(out)
    }
}

fn number(v: &Json) -> Option<f64> {
    v.as_f64()
}

fn offset(base: &Json, own: &Json, next: &Json) -> Json {
    match (number(base), number(own), number(next)) {
        (Some(b), Some(o), Some(n)) => return json!(o + (n - b)),
        _ => {}
    }
    match (base.as_array(), own.as_array(), next.as_array()) {
        (Some(b), Some(o), Some(n)) => Json::Array(
            n.iter()
                .enumerate()
                .map(|(i, v)| match (b.get(i).and_then(number), o.get(i).and_then(number), number(v)) {
                    (Some(b), Some(o), Some(n)) => json!(o + (n - b)),
                    _ => v.clone(),
                })
                .collect(),
        ),
        _ => next.clone(),
    }
}

fn typed(base: &Json, own: &Json, next: &Json) -> Json {
    match (base.as_array(), own.as_array(), next.as_array()) {
        (Some(b), Some(o), Some(n)) => Json::Array(
            n.iter()
                .enumerate()
                .map(|(i, v)| if b.get(i) != Some(v) { v.clone() } else { o.get(i).cloned().unwrap_or_else(|| v.clone()) })
                .collect(),
        ),
        _ => next.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rig() -> EditorRuntime {
        let mut rt = crate::EditorRuntime::open("").unwrap();
        for (id, x) in [(41u64, 100.0), (42, 300.0), (43, 500.0)] {
            rt.request(json!({"op": "create", "kind": "rectangle"})).unwrap();
            let made = rt.doc.view().layers().last().copied().unwrap();
            let _ = (id, x, made);
        }
        rt
    }

    fn position(rt: &EditorRuntime, layer: LayerId) -> Vec<f64> {
        let p = PropertyId::new("position").unwrap();
        let v = rt.doc.view().value_at(layer, &p, rt.time().unwrap()).unwrap().unwrap();
        crate::snapshot::value(&v).as_array().unwrap().iter().map(|n| n.as_f64().unwrap()).collect()
    }

    /// Selection semantics, once: a drag keeps each layer's own offset, a typed value sets only the axes that changed,
    /// a locked layer refuses, and a layer without the property is passed over.
    #[test]
    fn a_selection_is_edited_by_the_host_in_the_way_the_edit_says() {
        let mut rt = rig();
        let ids: Vec<LayerId> = rt.doc.view().layers().to_vec();
        assert_eq!(ids.len(), 3);
        let (a, b, c) = (ids[0], ids[1], ids[2]);
        for (l, x) in [(a, 100.0), (b, 300.0), (c, 500.0)] {
            rt.request(json!({"op": "setProperty", "layer": l.0, "property": "position", "value": [x, 50.0]})).unwrap();
        }
        rt.request(json!({"op": "setAttrs", "layers": [c.0], "patch": {"locked": true}})).unwrap();
        rt.request(json!({"op": "select", "ids": [a.0, b.0, c.0]})).unwrap();

        // A drag on b by +30 in x: a follows by the same amount, c is locked and stays.
        rt.request(json!({"op": "previewProperties", "edits": [{"layer": b.0, "property": "position", "value": [330.0, 50.0], "spread": "offset"}]})).unwrap();
        assert_eq!(position(&rt, b), vec![330.0, 50.0]);
        assert_eq!(position(&rt, a), vec![130.0, 50.0]);
        assert_eq!(position(&rt, c), vec![500.0, 50.0], "a locked layer refuses");
        // A second preview of the same drag is read against the committed values, so nothing accumulates.
        rt.request(json!({"op": "previewProperties", "edits": [{"layer": b.0, "property": "position", "value": [340.0, 50.0], "spread": "offset"}]})).unwrap();
        assert_eq!(position(&rt, a), vec![140.0, 50.0]);
        rt.request(json!({"op": "cancelPreview"})).unwrap();
        assert_eq!(position(&rt, a), vec![100.0, 50.0], "cancel puts every layer back");

        // A typed y of 80 on b: only y is set on a; a keeps its own x.
        rt.request(json!({"op": "previewProperties", "edits": [{"layer": b.0, "property": "position", "value": [300.0, 80.0], "spread": "typed"}]})).unwrap();
        rt.request(json!({"op": "commitPreview"})).unwrap();
        assert_eq!(position(&rt, a), vec![100.0, 80.0]);
        assert_eq!(position(&rt, b), vec![300.0, 80.0]);

        // Absolute gives every target the value as it is.
        rt.request(json!({"op": "previewProperties", "edits": [{"layer": b.0, "property": "position", "value": [10.0, 20.0], "spread": "absolute"}]})).unwrap();
        rt.request(json!({"op": "commitPreview"})).unwrap();
        assert_eq!(position(&rt, a), vec![10.0, 20.0]);
        assert_eq!(position(&rt, c), vec![500.0, 50.0], "still locked");
    }
}

#[cfg(test)]
mod effect_spread {
    use super::*;
    /// Effect parameters are named by each layer's own effect slot: dragging one layer's effect reaches another
    /// selected layer only where that slot holds the same effect, never a different effect that shares the name.
    #[test]
    fn an_effect_drag_reaches_only_the_same_effect() {
        // two different effects that share a numeric parameter name
        let catalog = crate::render::engine::known_effects();
        let numeric = |d: &crate::render::engine::EffectDescriptor| d.params.iter().map(|p| p.name.clone()).collect::<Vec<_>>();
        let (first, second, shared) = catalog.iter().flat_map(|a| catalog.iter().map(move |b| (a, b)))
            .filter(|(a, b)| a.plugin_id != b.plugin_id)
            .find_map(|(a, b)| numeric(a).into_iter().find(|n| numeric(b).contains(n)).map(|n| (a.plugin_id.clone(), b.plugin_id.clone(), n)))
            .expect("two effects share a parameter name");
        let mut rt = EditorRuntime::open("").unwrap();
        let mut layers = Vec::new();
        for plugin in [&first, &second, &first] {
            rt.request(json!({"op": "create", "kind": "rectangle"})).unwrap();
            let l = rt.viewer.selected().unwrap();
            rt.request(json!({"op": "applyEffect", "pluginId": plugin})).unwrap();
            layers.push(l);
        }
        let slot = |rt: &EditorRuntime, l: LayerId| rt.doc.view().effects(l).unwrap()[0].id;
        let name = |rt: &EditorRuntime, l: LayerId| PropertyId::effect_param(slot(rt, l), &shared).unwrap();
        let value = |rt: &EditorRuntime, l: LayerId| rt.doc.view().value_at(l, &name(rt, l), rt.time().unwrap()).unwrap();
        let before: Vec<_> = layers.iter().map(|&l| value(&rt, l)).collect();
        rt.request(json!({"op": "select", "ids": layers.iter().map(|l| l.0).collect::<Vec<_>>()})).unwrap();
        let shown = name(&rt, layers[0]).name().to_owned();
        let default = catalog.iter().find(|d| d.plugin_id == first).unwrap().params.iter().find(|p| p.name == shared).unwrap().default;
        let next = json!(default + 0.25);
        rt.request(json!({"op": "previewProperties", "edits": [{"layer": layers[0].0, "property": shown, "value": next, "spread": "typed"}]})).unwrap();
        rt.request(json!({"op": "commitPreview"})).unwrap();
        assert_ne!(value(&rt, layers[0]), before[0], "the shown layer took the edit");
        assert_eq!(value(&rt, layers[1]), before[1], "a different effect in the same slot is left alone");
        assert_ne!(value(&rt, layers[2]), before[2], "the same effect in the same slot follows");
    }
}
