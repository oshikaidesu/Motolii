use super::*;

impl EditorRuntime {
    /// The layers an attribute edit reaches: `layers` as named, or the shown `layer` — with `spread`, also every other
    /// selected layer that can be edited (the same rule a property edit spreads by).
    pub(crate) fn attrs_targets(&self, j: &J) -> Result<Vec<LayerId>, String> {
        if j.get("layers").is_some() { return ids(&j["layers"]); }
        let shown = layer(j)?;
        let mut targets = vec![shown];
        if j["spread"] == true {
            let view = self.doc.view().without_transients();
            targets.extend(self.viewer.selected_ids.iter().copied()
                .filter(|&l| l != shown && editor::functions::lens::edit_rejection(&view, l).ok().flatten().is_none()));
        }
        Ok(targets)
    }

    /// The layers a blend mode applies to: the selected layers that can be edited (not locked, not gone) and are not a
    /// camera. Every frontend asks the host, so a desk, a script and a shortcut mean the same layers.
    pub(crate) fn blend_targets(&self) -> Vec<LayerId> {
        let view = self.doc.view().without_transients();
        self.viewer
            .selected_ids
            .iter()
            .copied()
            .filter(|&layer| editor::functions::lens::edit_rejection(&view, layer).ok().flatten().is_none())
            .filter(|&layer| !matches!(view.meta(layer), Ok(Some(meta)) if matches!(meta.source, LayerSource::Camera)))
            .collect()
    }

    /// `applyBlend`: the mode on every target that does not wear it yet, as one step. Nothing to do is not an error.
    pub(super) fn apply_blend(&mut self, j: &J) -> Result<(), String> {
        let mode: BlendMode = serde_json::from_value(j["mode"].clone()).map_err(e)?;
        let view = self.doc.view().without_transients();
        let intents: Vec<Intent> = self
            .blend_targets()
            .into_iter()
            .filter(|&layer| view.attrs(layer).ok().flatten().map(|attrs| attrs.blend_mode != mode).unwrap_or(true))
            .map(|layer| Intent::SetAttrs { layer, patch: LayerAttrsPatch { blend_mode: Some(mode), ..Default::default() } })
            .collect();
        drop(view);
        if intents.is_empty() {
            return Ok(());
        }
        self.apply(intents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A blend applies to the selected layers that can take it: locked layers and cameras are left, layers already in
    /// the mode are not touched, and a preview on the desk's behalf shows the last target.
    #[test]
    fn a_blend_reaches_the_selected_layers_that_can_take_it() {
        let mut rt = crate::EditorRuntime::open("").unwrap();
        for _ in 0..3 {
            rt.request(json!({"op": "create", "kind": "rectangle"})).unwrap();
        }
        rt.request(json!({"op": "create", "kind": "camera"})).unwrap();
        let ids: Vec<LayerId> = rt.doc.view().layers().to_vec();
        assert_eq!(ids.len(), 4);
        let (a, b, locked, camera) = (ids[0], ids[1], ids[2], ids[3]);
        rt.request(json!({"op": "setAttrs", "layers": [locked.0], "patch": {"locked": true}})).unwrap();
        rt.request(json!({"op": "select", "ids": [a.0, b.0, locked.0, camera.0]})).unwrap();
        assert_eq!(rt.blend_targets(), vec![a, b]);
        let mode = |rt: &EditorRuntime, l: LayerId| format!("{:?}", rt.doc.view().attrs(l).unwrap().unwrap().blend_mode);
        rt.request(json!({"op": "previewBlend", "mode": "Multiply"})).unwrap();
        let previewed = rt.preview.as_ref().unwrap().1.clone();
        assert_eq!(previewed.len(), 1, "the desk previews on one layer");
        assert!(matches!(&previewed[0], Intent::SetAttrs { layer, .. } if *layer == b), "the last target");
        rt.request(json!({"op": "cancelPreview"})).unwrap();
        rt.request(json!({"op": "applyBlend", "mode": "Multiply"})).unwrap();
        assert_eq!((mode(&rt, a), mode(&rt, b)), ("Multiply".into(), "Multiply".into()));
        assert_eq!(mode(&rt, locked), "Normal");
        assert_eq!(mode(&rt, camera), "Normal");
        let (undo, _) = rt.doc.history_depth();
        rt.request(json!({"op": "applyBlend", "mode": "Multiply"})).unwrap();
        assert_eq!(rt.doc.history_depth().0, undo, "already in the mode: no step");
    }
}
