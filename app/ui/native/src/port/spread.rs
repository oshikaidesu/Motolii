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
        let committed = |l: LayerId| -> Option<Json> {
            let value = view.value_at(l, &property, at).ok().flatten().or_else(|| motolii_edit::document::edit::default_value(&view, l, &property).ok().flatten());
            value.map(|v| crate::snapshot::value(&v))
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
