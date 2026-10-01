use super::*;
use serde_json::{json, Value as Json};

impl EditorRuntime {
    /// The key intervals the selected keys mean: for every property with a selected key, each interval that starts on a
    /// selected key (with several frames selected, the last one only ends an interval). Every frontend asks the host, so
    /// the Ease desk of any face, a script and a shortcut mean the same intervals. `shape` is the curve of the interval.
    pub(crate) fn ease_intervals(&self) -> Vec<Json> {
        if self.viewer.selected_keys.is_empty() {
            return Vec::new();
        }
        let view = self.doc.view().without_transients();
        let Ok(Some(comp)) = view.composition() else { return Vec::new() };
        let fps = comp.fps;
        let mut out = Vec::new();
        for layer in view.layers() {
            let chosen: Vec<&crate::viewer::KeySel> = self.viewer.selected_keys.iter().filter(|k| k.layer == layer).collect();
            if chosen.is_empty() {
                continue;
            }
            let attrs = view.attrs(layer).ok().flatten().unwrap_or_default();
            for property in view.properties(layer) {
                let frames: std::collections::BTreeSet<i64> = chosen
                    .iter()
                    .filter(|k| k.property.as_ref().map_or(true, |p| *p == property))
                    .map(|k| (k.at_sec * fps.as_f64()).round() as i64)
                    .collect();
                let Some(&last) = frames.iter().next_back() else { continue };
                let Ok(Some(track)) = view.track(layer, &property) else { continue };
                let mut keys: Vec<(i64, _)> = track.keys().iter().filter_map(|k| Some((k.t.try_to_frame_round(fps).ok()?, k.interp))).collect();
                keys.sort_by_key(|k| k.0);
                for pair in keys.windows(2) {
                    let (frame, interp) = pair[0];
                    if !frames.contains(&frame) || (frames.len() > 1 && frame == last) {
                        continue;
                    }
                    out.push(json!({
                        "layer": layer.0, "name": attrs.name, "property": property.name(),
                        "frame": frame, "end": pair[1].0, "shape": crate::snapshot::interp(interp), "locked": attrs.locked,
                    }));
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two keys selected make one interval, with its curve; the last selected frame only ends one; nothing selected, nothing.
    #[test]
    fn selected_keys_mean_intervals_the_host_names() {
        let mut rt = crate::EditorRuntime::open("").unwrap();
        rt.request(json!({"op": "create", "kind": "rectangle"})).unwrap();
        let layer = rt.viewer.selected().unwrap();
        rt.request(json!({"op": "animate", "enabled": true})).unwrap();
        for (frame, x) in [(0, 100.0), (30, 200.0), (60, 300.0)] {
            rt.request(json!({"op": "seek", "frame": frame})).unwrap();
            rt.request(json!({"op": "setProperty", "layer": layer.0, "property": "position", "value": [x, 50.0]})).unwrap();
        }
        rt.request(json!({"op": "animate", "enabled": false})).unwrap();
        assert!(rt.ease_intervals().is_empty(), "no keys selected");
        let pick = |frames: &[i64]| json!({"op": "select", "ids": [layer.0], "keys": frames.iter().map(|f| json!({"layer": layer.0, "property": "position", "frame": f})).collect::<Vec<_>>()});
        rt.request(pick(&[0, 30])).unwrap();
        let spans: Vec<(i64, i64)> = rt.ease_intervals().iter().map(|i| (i["frame"].as_i64().unwrap(), i["end"].as_i64().unwrap())).collect();
        assert_eq!(spans, vec![(0, 30)], "the last selected key only ends an interval");
        rt.request(pick(&[30])).unwrap();
        let spans: Vec<(i64, i64)> = rt.ease_intervals().iter().map(|i| (i["frame"].as_i64().unwrap(), i["end"].as_i64().unwrap())).collect();
        assert_eq!(spans, vec![(30, 60)], "one key starts the interval after it");
        let first = &rt.ease_intervals()[0];
        assert_eq!(first["property"], "position");
        assert_eq!(first["shape"]["kind"], "Linear");
        assert_eq!(first["locked"], false);
    }
}
