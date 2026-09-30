#[cfg(test)]
mod camera {
    use crate::doc::store::*;

    const PROBE: &str = include_str!("../../../../../crates/motolii-script/probes/camera.js");

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
    const PROBE: &str = include_str!("../../../../../crates/motolii-script/probes/shatter.js");

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


/// Retained mapping over the store's existing PropertySource links (base + summed modulators). No new mechanism.
#[cfg(test)]
mod mapping {
    use crate::doc::store::*;
    use crate::doc::store::slot::{PropertyLink, PropertySource};
    use motolii_edit::Intent;

    struct Rig { rt: crate::EditorRuntime, control: LayerId, glass: LayerId }

    fn p(name: &str) -> PropertyId { PropertyId::new(name).unwrap() }
    fn link(source: LayerId, name: &str, plugin: &str, params: &[(&str, f64)]) -> PropertyLink {
        PropertyLink { source_layer: source, source_property: p(name), time_offset: RationalTime::ZERO, plugin_id: plugin.into(), params: params.iter().map(|(k, v)| ((*k).to_owned(), Value::F64(*v))).collect() }
    }
    fn at(seconds: f64) -> RationalTime { RationalTime::try_from_frame((seconds * 30.0).round() as i64, motolii_doc::core::Fps::try_new(30, 1).unwrap()).unwrap() }

    /// The Cassette: a control layer holding the exposed parameters, a Repeater layer whose rows are driven by them.
    fn shatter_cassette() -> Rig {
        let mut rt = crate::EditorRuntime::open("").unwrap();
        rt.run_script("nullLayer({ name: 'Shatter' }); rectangle({ name: 'Glass' }).effect('Repeater', { Along: 'Grid', Pick: 'Random', 'Delay Each': 0.03 });", "m.js").unwrap();
        let find = |rt: &crate::EditorRuntime, n: &str| { let v = rt.doc.view(); *v.layers().iter().find(|l| v.attrs(**l).unwrap().unwrap().name == n).unwrap() };
        let (control, glass) = (find(&rt, "Shatter"), find(&rt, "Glass"));
        let amount = KeyframeTrack::try_from_keys(vec![
            Keyframe { t: at(0.0), value: Value::F64(0.0), interp: Interp::Linear, spatial: None },
            Keyframe { t: at(1.5), value: Value::F64(1.0), interp: Interp::Linear, spatial: None },
        ]).unwrap();
        let effect = |name: &str| p(&format!("effect.0.param.{name}"));
        rt.doc.apply_all([
            Intent::SetConstant { layer: control, property: p("Pieces"), value: Value::F64(16.0) },
            Intent::SetTrack { layer: control, property: p("Amount"), track: amount },
            Intent::SetConstant { layer: control, property: p("Spread"), value: Value::Vec2([300.0, 300.0]) },
            Intent::SetConstant { layer: control, property: p("Seed"), value: Value::F64(7.0) },
            Intent::SetPropertyModulators { layer: glass, property: effect("count"), modulators: vec![link(control, "Pieces", "motolii.link.identity", &[])] },
            Intent::SetPropertyModulators { layer: glass, property: effect("seed"), modulators: vec![link(control, "Seed", "motolii.link.identity", &[])] },
            Intent::SetPropertyModulators { layer: glass, property: effect("rotation_random"), modulators: vec![link(control, "Amount", "motolii.link.remap", &[("out_max", 180.0)])] },
            Intent::SetPropertyModulators { layer: glass, property: effect("scale_random"), modulators: vec![link(control, "Amount", "motolii.link.linear", &[("scale", -0.6)])] },
            Intent::SetPropertyModulators { layer: glass, property: effect("position_random"), modulators: vec![link(control, "Spread", "motolii.link.identity", &[])] },
        ]).unwrap();
        Rig { rt, control, glass }
    }

    fn value(rig: &Rig, name: &str, seconds: f64) -> Option<Value> { rig.rt.doc.view().value_at(rig.glass, &p(&format!("effect.0.param.{name}")), at(seconds)).unwrap() }

    fn copies(rig: &mut Rig) -> usize {
        rig.rt.request(serde_json::json!({"op":"seek","frame":60})).unwrap();
        let view = rig.rt.doc.view();
        let time = rig.rt.time().unwrap();
        crate::render::picture::resolve::resolved_layers(&view, time).unwrap().iter().filter(|l| l.id == rig.glass).count()
    }

    #[test]
    fn one_parameter_drives_several_rows_and_stays_one_parameter() {
        let mut rig = shatter_cassette();
        // Amount 0..1 keyed once; every mapped row follows it (1:N), with the transforms the probe needed: identity, linear, remap.
        assert_eq!(value(&rig, "rotation_random", 0.0), Some(Value::F64(0.0)));
        let Some(Value::F64(mid)) = value(&rig, "rotation_random", 0.5) else { panic!() };
        assert!((mid - 60.0).abs() < 1e-6, "Amount is 1/3 at frame 15: {mid}");
        assert_eq!(value(&rig, "rotation_random", 1.5), Some(Value::F64(180.0)));
        let Some(Value::F64(scale)) = value(&rig, "scale_random", 1.5) else { panic!() };
        assert!((scale + 0.6).abs() < 1e-9);
        assert_eq!(value(&rig, "seed", 0.0), Some(Value::F64(7.0)));
        // Pieces -> Count is 1:1 and stays live: editing the exposed Pieces changes the copies.
        assert_eq!(copies(&mut rig), 16);
        rig.rt.doc.apply(Intent::SetConstant { layer: rig.control, property: p("Pieces"), value: Value::F64(5.0) }).unwrap();
        assert_eq!(copies(&mut rig), 5);
        // Spread (a pair) -> Position Random (a pair): identity link carries the pair.
        assert_eq!(value(&rig, "position_random", 0.0), Some(Value::Vec2([300.0, 300.0])));
    }

    /// A scalar cannot drive a pair today: the link keeps the scalar's type, so the row is not what the author meant.
    #[test]
    fn a_scalar_source_does_not_broadcast_into_a_pair() {
        let mut rig = shatter_cassette();
        let (control, glass) = (rig.control, rig.glass);
        rig.rt.doc.apply_all([
            Intent::SetConstant { layer: control, property: p("Spread1"), value: Value::F64(300.0) },
            Intent::SetPropertyModulators { layer: glass, property: p("effect.0.param.position_random"), modulators: vec![link(control, "Spread1", "motolii.link.identity", &[])] },
        ]).unwrap();
        let got = value(&rig, "position_random", 0.0);
        eprintln!("MAP scalar into pair reads {got:?}");
        assert_ne!(got, Some(Value::Vec2([300.0, 300.0])));
    }

    /// The window already owns this: a driven row refuses a hand edit until the driver is edited (ownership option A).
    /// The store itself lets a plain write replace the source and drop the link (option B); only the window's guard makes it A.
    #[test]
    fn the_window_refuses_a_hand_edit_of_a_driven_row() {
        let mut rig = shatter_cassette();
        rig.rt.request(serde_json::json!({"op":"animate","enabled":false})).unwrap();
        let refused = rig.rt.request(serde_json::json!({"op":"setProperty","layer":rig.glass.0,"property":"effect.0.param.rotation_random","value":10.0})).unwrap_err();
        eprintln!("MAP hand edit of a driven row: {refused:?}");
        // Store level: a plain write replaces the whole source, so the mapping is gone (option B) unless the window guards it.
        rig.rt.doc.apply(Intent::SetConstant { layer: rig.glass, property: p("effect.0.param.rotation_random"), value: Value::F64(10.0) }).unwrap();
        assert_eq!(value(&rig, "rotation_random", 0.5), Some(Value::F64(10.0)));
        assert!(rig.rt.doc.view().property_source(rig.glass, &p("effect.0.param.rotation_random")).unwrap().unwrap().modulators.is_empty());
    }

    /// Detach: bake every mapped value into ordinary rows and drop the links. The composition stays and nothing reads the control any more.
    #[test]
    fn detaching_keeps_the_composition_and_drops_only_the_mapping() {
        let mut rig = shatter_cassette();
        let before = value(&rig, "rotation_random", 0.0);
        let names = ["count", "seed", "rotation_random", "scale_random", "position_random"];
        for name in names {
            let property = p(&format!("effect.0.param.{name}"));
            let now = value(&rig, name, 0.0).unwrap();
            rig.rt.doc.apply_all([
                Intent::SetPropertyModulators { layer: rig.glass, property: property.clone(), modulators: vec![] },
                Intent::SetConstant { layer: rig.glass, property, value: now },
            ]).unwrap();
        }
        rig.rt.doc.apply(Intent::RemoveLayer(rig.control)).unwrap();
        assert_eq!(value(&rig, "rotation_random", 0.0), before);
        assert_eq!(value(&rig, "count", 0.0), Some(Value::F64(16.0)));
        assert_eq!(copies(&mut rig), 16);
    }

    #[test]
    fn save_load_undo_and_duplicate_keep_the_mapping_intact() {
        let mut rig = shatter_cassette();
        let out = std::env::temp_dir().join(format!("probe-mapping-{}.rrd", std::process::id()));
        rig.rt.request(serde_json::json!({"op":"save","path":out.to_string_lossy()})).unwrap();
        let loaded = crate::EditorRuntime::open(&out.to_string_lossy()).unwrap();
        let _ = std::fs::remove_file(&out);
        let again = Rig { rt: loaded, control: rig.control, glass: rig.glass };
        assert_eq!(value(&again, "rotation_random", 0.5), value(&rig, "rotation_random", 0.5));
        // Duplicate both layers: the copy's links point at the copy of the control; duplicate Glass alone: they keep pointing at the original.
        rig.rt.request(serde_json::json!({"op":"select","ids":[rig.control.0, rig.glass.0]})).unwrap();
        let before = rig.rt.doc.view().layers().len();
        rig.rt.request(serde_json::json!({"op":"duplicate"})).unwrap();
        let layers = rig.rt.doc.view().layers();
        assert_eq!(layers.len(), before + 2);
        let view = rig.rt.doc.view();
        let copy_of_glass = *layers.iter().filter(|l| **l != rig.glass && view.effects(**l).unwrap().len() == 1).next().unwrap();
        use motolii_script::{Host, Query};
        let row = rig.rt.query(Query::Layer(copy_of_glass.0)).unwrap();
        let id = row["effects"][0]["params"].as_array().unwrap().iter().find(|q| q["label"] == "Rotation" && q["id"].as_str().unwrap().contains("random")).unwrap()["id"].as_str().unwrap().to_owned();
        let source: PropertySource = view.property_source(copy_of_glass, &p(&id)).unwrap().expect("the copy keeps a driven source");
        let target = source.modulators[0].source_layer;
        eprintln!("MAP duplicate of both: copy's link source is layer {} (original control {}, glass {})", target.0, rig.control.0, rig.glass.0);
        assert_ne!(target, rig.control, "a duplicated pair carries its own control");
        // Undo: creating a cassette was one transaction, so one Undo takes every mapping (and the parameters) away together.
        while rig.rt.doc.view().layers().len() > before { rig.rt.doc.undo(); }
        rig.rt.doc.undo();
        assert!(rig.rt.doc.view().property_source(rig.glass, &p("effect.0.param.position_random")).unwrap().map_or(true, |s| s.modulators.is_empty()));
    }

    /// Second example, no Repeater: a title treatment. Three exposed parameters drive ordinary text rows (Size, Tracking, Stagger).
    #[test]
    fn a_title_treatment_maps_onto_ordinary_text_rows() {
        let mut rt = crate::EditorRuntime::open("").unwrap();
        rt.run_script("nullLayer({ name: 'Title Pop' }); text('HELLO', { name: 'Title' }).set('Split', 'Chars');", "t.js").unwrap();
        let find = |rt: &crate::EditorRuntime, n: &str| { let v = rt.doc.view(); *v.layers().iter().find(|l| v.attrs(**l).unwrap().unwrap().name == n).unwrap() };
        let (control, title) = (find(&rt, "Title Pop"), find(&rt, "Title"));
        use motolii_script::{Host, Query};
        let row = rt.query(Query::Layer(title.0)).unwrap();
        let id = |label: &str| row["properties"].as_array().unwrap().iter().find(|q| q["label"] == label).unwrap()["id"].as_str().unwrap().to_owned();
        let (size, tracking, stagger) = (id("Size"), id("Tracking"), id("Stagger"));
        eprintln!("MAP title rows: {size} {tracking} {stagger}");
        let control_row = rt.query(Query::Layer(control.0)).unwrap();
        eprintln!("MAP control layer rows before params: {}", control_row["properties"].as_array().unwrap().iter().map(|q| q["label"].as_str().unwrap_or("?").to_owned()).collect::<Vec<_>>().join(" | "));
        rt.doc.apply_all([
            Intent::SetConstant { layer: control, property: p("Punch"), value: Value::F64(0.5) },
            Intent::SetConstant { layer: control, property: p("Looseness"), value: Value::F64(0.25) },
            Intent::SetConstant { layer: control, property: p("Delay"), value: Value::F64(0.08) },
            Intent::SetPropertyModulators { layer: title, property: p(&size), modulators: vec![link(control, "Punch", "motolii.link.remap", &[("out_min", 48.0), ("out_max", 160.0)])] },
            Intent::SetPropertyModulators { layer: title, property: p(&tracking), modulators: vec![link(control, "Looseness", "motolii.link.linear", &[("scale", 40.0)])] },
            Intent::SetPropertyModulators { layer: title, property: p(&stagger), modulators: vec![link(control, "Delay", "motolii.link.identity", &[])] },
        ]).unwrap();
        let control_row = rt.query(Query::Layer(control.0)).unwrap();
        eprintln!("MAP control layer rows after params: {}", control_row["properties"].as_array().unwrap().iter().map(|q| q["label"].as_str().unwrap_or("?").to_owned()).collect::<Vec<_>>().join(" | "));
        let title_row = rt.query(Query::Layer(title.0)).unwrap();
        let size_row = title_row["properties"].as_array().unwrap().iter().find(|q| q["label"] == "Size").unwrap();
        eprintln!("MAP driven Size row json: {size_row}");
        let v = rt.doc.view();
        let at0 = RationalTime::ZERO;
        assert_eq!(v.value_at(title, &p(&size), at0).unwrap(), Some(Value::F64(104.0)));
        assert_eq!(v.value_at(title, &p(&tracking), at0).unwrap(), Some(Value::F64(10.0)));
        assert_eq!(v.value_at(title, &p(&stagger), at0).unwrap(), Some(Value::F64(0.08)));
        // One exposed knob edited later moves the row: the parameter identity survived materialization.
        rt.doc.apply(Intent::SetConstant { layer: control, property: p("Punch"), value: Value::F64(1.0) }).unwrap();
        assert_eq!(rt.doc.view().value_at(title, &p(&size), at0).unwrap(), Some(Value::F64(160.0)));
    }
}
