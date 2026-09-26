#[cfg(test)]
mod camera {
    use crate::doc::store::*;

    const PROBE: &str = include_str!("../../../../extensions/script/probes/camera.js");

    fn keys(rt: &crate::EditorRuntime, name: &str, row: &str) -> Vec<(i64, Value)> {
        let view = rt.doc.view();
        let id = *view.layers().iter().find(|l| view.attrs(**l).unwrap().unwrap().name == name).unwrap();
        let track = view.track(id, &PropertyId::new(row).unwrap()).unwrap();
        track.map(|t| t.keys().iter().map(|k| (k.t.num(), k.value.clone())).collect()).unwrap_or_default()
    }

    #[test]
    fn handheld_orbit_is_one_bake_of_ordinary_keys() {
        let mut rt = crate::EditorRuntime::open("").unwrap();
        rt.run_script(PROBE, "camera.js").unwrap();
        for row in ["camera.center", "camera.orbit", "camera.distance", "camera.roll"] {
            assert_eq!(keys(&rt, "Chase cam", row).len(), 37, "{row}");
        }
        let mut again = crate::EditorRuntime::open("").unwrap();
        again.run_script(PROBE, "camera.js").unwrap();
        assert_eq!(keys(&rt, "Chase cam", "camera.orbit"), keys(&again, "Chase cam", "camera.orbit"), "same seed, same shot");

        // Save and load: the shot survives as ordinary keys.
        let out = std::env::temp_dir().join(format!("probe-camera-{}.rrd", std::process::id()));
        rt.request(serde_json::json!({"op":"save","path":out.to_string_lossy()})).unwrap();
        let loaded = crate::EditorRuntime::open(&out.to_string_lossy()).unwrap();
        assert_eq!(keys(&rt, "Chase cam", "camera.center"), keys(&loaded, "Chase cam", "camera.center"));
        let _ = std::fs::remove_file(&out);

        // Manual editing afterwards: a key moved by hand stays one key (the Timeline and Inspector see nothing special).
        let id = { let v = rt.doc.view(); *v.layers().iter().find(|l| v.attrs(**l).unwrap().unwrap().name == "Chase cam").unwrap() };
        rt.request(serde_json::json!({"op":"animate","enabled":true})).unwrap();
        rt.request(serde_json::json!({"op":"seek","frame":30})).unwrap();
        rt.request(serde_json::json!({"op":"setProperty","layer":id.0,"property":"camera.roll","value":9.0})).unwrap();
        let roll = keys(&rt, "Chase cam", "camera.roll");
        assert_eq!(roll.len(), 37);
        assert!(roll.iter().any(|(_, v)| *v == Value::F64(9.0)));
    }

    /// A refused script leaves nothing behind, and a rerun rebuilds the same shot.
    #[test]
    fn a_refused_camera_script_rolls_back() {
        let mut rt = crate::EditorRuntime::open("").unwrap();
        let outcome = rt.run_script("const c = camera(); c.key('Roll', 0, 1); c.key('Shake', 0, 1);", "bad.js");
        assert!(outcome.unwrap_err().contains("has no \"Shake\""));
        assert!(rt.doc.view().layers().is_empty());
        let outcome = rt.run_script("const c = camera(); c.set('Orbit', 'north');", "bad2.js");
        outcome.unwrap_err();
        assert!(rt.doc.view().layers().is_empty());
    }
}

#[cfg(test)]
mod shatter {
    const PROBE: &str = include_str!("../../../../extensions/script/probes/shatter.js");

    fn copies(rt: &mut crate::EditorRuntime) -> usize {
        rt.request(serde_json::json!({"op":"seek","frame":60})).unwrap();
        let view = rt.doc.view();
        let time = rt.time().unwrap();
        let resolved = crate::render::picture::resolve::resolved_layers(&view, time).unwrap().len();
        let scene = rt.engine.frame_graph_editor_scene(&view, time).unwrap();
        eprintln!("SHATTER resolved={resolved} scene={}", scene.layers.len());
        resolved
    }

    fn param(rt: &crate::EditorRuntime, label: &str) -> serde_json::Value {
        use motolii_script::{Host, Query};
        let id = rt.doc.view().layers()[0];
        let row = rt.query(Query::Layer(id.0)).unwrap();
        row["effects"][0]["params"].as_array().unwrap().iter().find(|p| p["label"] == label || p["id"] == label).cloned().unwrap_or(serde_json::Value::Null)
    }

    #[test]
    fn shatter_runs_on_the_existing_vocabulary() {
        let mut rt = crate::EditorRuntime::open("").unwrap();
        rt.run_script(PROBE, "shatter.js").unwrap();
        assert_eq!(rt.doc.view().layers().len(), 1, "one ordinary layer, no hidden helper layers");
        assert_eq!(rt.doc.view().effects(rt.doc.view().layers()[0]).unwrap().len(), 1, "one ordinary Repeater");
        let before = copies(&mut rt);
        eprintln!("SHATTER copies at frame 0 = {before}; Count row = {}", param(&rt, "Count"));
        // The user changes Pieces by hand in the Inspector: the Repeater's own Count.
        let layer = rt.doc.view().layers()[0];
        let count = param(&rt, "Count")["id"].as_str().unwrap().to_owned();
        rt.request(serde_json::json!({"op":"animate","enabled":false})).unwrap();
        rt.request(serde_json::json!({"op":"setProperty","layer":layer.0,"property":count,"value":5})).unwrap();
        let after = copies(&mut rt);
        eprintln!("SHATTER copies after Count 16 -> 5 = {after}");
        assert!(after < before);
        // But Amount and Spread are baked into key values of five other rows: nothing ties them to one knob.
        use motolii_script::{Host, Query};
        let row = rt.query(Query::Layer(layer.0)).unwrap();
        let keyed: Vec<_> = row["effects"][0]["params"].as_array().unwrap().iter().filter(|p| p["keys"].as_array().map_or(false, |k| k.len() == 2)).map(|p| p["label"].as_str().unwrap_or("?").to_owned()).collect();
        eprintln!("SHATTER keyed rows: {keyed:?}");
        assert!(keyed.len() >= 4, "amount and spread live only as key values on these rows");
    }
}

