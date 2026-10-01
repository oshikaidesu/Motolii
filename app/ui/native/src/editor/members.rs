//! Members: the addressable parts a layer is made of, as every frontend reads them. A container's children, a text's split
//! units and a Repeater's copies are members in order; the layer's order rows (Stagger, Stagger From, From End) apply to
//! them by one law (`schedule_delay`). None of them is a layer of its own: a member is found, never stored.
use crate::doc::store::*;
use crate::render::extensions::placement;
use serde_json::{json, Value as Json};

/// Which members the layer's order rows apply to, and how many there are now. `None`: the layer is one thing.
pub(crate) fn members(view: &StoreView<'_>, layer: LayerId, at: RationalTime) -> Result<Option<Json>, StoreError> {
    let Some(meta) = view.meta(layer)? else { return Ok(None) };
    if meta.source == LayerSource::Group {
        let count = view.layers().iter().filter(|&&l| view.attrs(l).ok().flatten().unwrap_or_default().parent == Some(layer)).count();
        return Ok((count > 0).then(|| json!({"kind": "children", "count": count})));
    }
    if meta.source == LayerSource::Text {
        let split = view.choice(layer, names::TEXT_SPLIT, at)?;
        if split > 0 {
            let kind = names::TEXT_SPLIT_CHOICES.get(split as usize).copied().unwrap_or("Units").to_lowercase();
            let count = crate::render::picture::text::text_units(view, layer, at)?.len();
            return Ok(Some(json!({"kind": kind, "count": count})));
        }
    }
    for effect in view.effects(layer)? {
        let Some(kind) = placement::kind(&effect.plugin_id) else { continue };
        let mut params = Vec::new();
        for p in kind.params {
            if let Some(v) = view.value_at(layer, &PropertyId::effect_param(effect.id, p.name)?, at)? {
                params.push((p.name.to_owned(), v));
            }
        }
        return Ok(Some(json!({"kind": "copies", "count": placement::placements(kind, &params).len(), "by": kind.label})));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    /// The three member sources, as the status says them: copies of a shape, children of a container, words of a text.
    #[test]
    fn the_status_names_each_layers_members() {
        let mut rt = crate::EditorRuntime::open("").unwrap();
        let members = |rt: &mut crate::EditorRuntime, id: u64| -> serde_json::Value {
            let status = rt.status().unwrap();
            status["layers"].as_array().unwrap().iter().find(|l| l["id"] == id).unwrap()["members"].clone()
        };
        rt.request(json!({"op": "create", "kind": "rectangle"})).unwrap();
        let a = rt.viewer.selected().unwrap().0;
        assert!(members(&mut rt, a).is_null(), "a shape alone is one thing");
        rt.request(json!({"op": "applyEffect", "pluginIds": ["motolii.repeat"]})).unwrap();
        assert_eq!(members(&mut rt, a), json!({"kind": "copies", "count": 3, "by": "Repeater"}));
        rt.request(json!({"op": "create", "kind": "ellipse"})).unwrap();
        let b = rt.viewer.selected().unwrap().0;
        rt.request(json!({"op": "select", "ids": [a, b]})).unwrap();
        rt.request(json!({"op": "group"})).unwrap();
        let group = rt.viewer.selected().unwrap().0;
        assert_eq!(members(&mut rt, group), json!({"kind": "children", "count": 2}));
        rt.request(json!({"op": "create", "kind": "text"})).unwrap();
        let text = rt.viewer.selected().unwrap().0;
        rt.request(json!({"op": "setText", "layer": text, "content": "ONE TWO THREE"})).unwrap();
        assert!(members(&mut rt, text).is_null(), "Split None: one thing");
        rt.request(json!({"op": "setProperty", "layer": text, "property": "text_split", "value": 2})).unwrap();
        assert_eq!(members(&mut rt, text), json!({"kind": "words", "count": 3}));
    }
}
