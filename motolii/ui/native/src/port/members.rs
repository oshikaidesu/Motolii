use super::*;
use serde_json::{json, Value as Json};

/// Members, through the operations every frontend sends: a Repeater is added to a shape with `applyEffect`, its copies are
/// members in order, and the shape's order rows (the same three a text and a container have) shift them in time on the
/// live renderer's scene, by the one law children and split units read.
#[test]
fn copies_are_members_in_order_on_the_live_scene() {
    let mut rt = crate::EditorRuntime::open("").unwrap();
    rt.request(json!({"op": "create", "kind": "rectangle"})).unwrap();
    let layer = rt.viewer.selected().unwrap();
    let rows = |rt: &mut EditorRuntime| -> Vec<String> {
        let status = rt.status().unwrap();
        let l = status["layers"].as_array().unwrap().iter().find(|l| l["id"] == layer.0).unwrap().clone();
        l["properties"].as_array().unwrap().iter().map(|p| p["id"].as_str().unwrap().to_owned()).collect()
    };
    assert!(!rows(&mut rt).contains(&"layout.stagger".to_owned()), "a plain shape has no members, so no order rows");
    rt.request(json!({"op": "applyEffect", "pluginIds": ["motolii.repeat"]})).unwrap();
    assert!(rows(&mut rt).contains(&"layout.stagger".to_owned()), "copies are members: the order rows are offered");
    let effect = rt.doc.view().effects(layer).unwrap()[0].id.0;
    let param = |name: &str| format!("effect.{effect}.param.{name}");
    rt.request(json!({"op": "setProperty", "layer": layer.0, "property": param("count"), "value": 3.0})).unwrap();
    rt.request(json!({"op": "animate", "enabled": true})).unwrap();
    for (frame, v) in [(0, 0.0), (30, 1.0)] {
        rt.request(json!({"op": "seek", "frame": frame})).unwrap();
        rt.request(json!({"op": "setProperty", "layer": layer.0, "property": "opacity", "value": v})).unwrap();
    }
    rt.request(json!({"op": "animate", "enabled": false})).unwrap();
    let scene_opacities = |rt: &mut EditorRuntime| -> Vec<f32> {
        let fps = rt.doc.view().composition().unwrap().unwrap().fps;
        let at = RationalTime::try_from_frame(15, fps).unwrap();
        let view = rt.doc.view();
        let scene = rt.engine.frame_graph_editor_scene(&view, at).unwrap();
        let mut out: Vec<(u32, f32)> = scene.layers.iter().filter(|l| l.layer == layer && !l.ghost).map(|l| (l.instance, l.opacity)).collect();
        out.sort_by_key(|c| c.0);
        out.into_iter().map(|c| c.1).collect()
    };
    let before = scene_opacities(&mut rt);
    assert_eq!(before.len(), 3, "{before:?}");
    assert!(before.iter().all(|o| (o - 0.5).abs() < 0.03), "no Stagger: as before: {before:?}");
    rt.request(json!({"op": "setProperty", "layer": layer.0, "property": "layout.stagger", "value": 0.2})).unwrap();
    let after = scene_opacities(&mut rt);
    assert!((after[0] - 0.5).abs() < 0.03 && (after[1] - 0.4).abs() < 0.03 && (after[2] - 0.3).abs() < 0.03, "the words' law on the live scene: {after:?}");
    // Undo takes the Stagger away and the copies are together again.
    rt.request(json!({"op": "undo"})).unwrap();
    let undone = scene_opacities(&mut rt);
    assert!(undone.iter().all(|o| (o - 0.5).abs() < 0.03), "{undone:?}");
    let _: Json = json!(null);
}

fn staggered_children() -> (crate::EditorRuntime, Vec<LayerId>, RationalTime) {
    let mut rt = crate::EditorRuntime::open("").unwrap();
    let mut kids = Vec::new();
    for _ in 0..3 {
        rt.request(json!({"op": "create", "kind": "rectangle"})).unwrap();
        kids.push(rt.viewer.selected().unwrap());
    }
    rt.request(json!({"op": "animate", "enabled": true})).unwrap();
    for (frame, v) in [(0, 0.0), (30, 1.0)] {
        rt.request(json!({"op": "seek", "frame": frame})).unwrap();
        for k in &kids {
            rt.request(json!({"op": "setProperty", "layer": k.0, "property": "opacity", "value": v})).unwrap();
        }
    }
    rt.request(json!({"op": "animate", "enabled": false})).unwrap();
    rt.request(json!({"op": "select", "ids": kids.iter().map(|k| k.0).collect::<Vec<_>>()})).unwrap();
    rt.request(json!({"op": "group"})).unwrap();
    let group = rt.viewer.selected().unwrap();
    rt.request(json!({"op": "setProperty", "layer": group.0, "property": "layout.stagger", "value": 0.2})).unwrap();
    let fps = rt.doc.view().composition().unwrap().unwrap().fps;
    let at = RationalTime::try_from_frame(15, fps).unwrap();
    let view = rt.doc.view();
    let mut order: Vec<(i16, LayerId)> = kids.iter().map(|&k| (view.meta(k).unwrap().unwrap().order, k)).collect();
    order.sort();
    drop(view);
    (rt, order.into_iter().map(|(_, k)| k).collect(), at)
}

fn sorted(mut v: Vec<f32>) -> Vec<f32> {
    v.sort_by(|a, b| b.total_cmp(a));
    v
}

/// The second member source through the same operations: the children of a plain group, keyed alike, under the group's
/// Stagger read their keys by the same law as the copies above (0.5 / 0.4 / 0.3), in the document.
#[test]
fn children_are_members_in_order_by_the_same_law() {
    let (rt, kids, at) = staggered_children();
    let view = rt.doc.view();
    let read: Vec<f32> = kids.iter().map(|k| match view.value_at(*k, &PropertyId::new(property::OPACITY).unwrap(), at).unwrap() {
        Some(Value::F64(v)) => v as f32,
        other => panic!("{other:?}"),
    }).collect();
    let s = sorted(read.clone());
    assert!((s[0] - 0.5).abs() < 0.03 && (s[1] - 0.4).abs() < 0.03 && (s[2] - 0.3).abs() < 0.03, "{read:?}");
}

/// The live scene reads children at the time the document does: their values go through the layer clock in the graph
/// (`PropertyClock`), the same clock `value_at` uses. Undo takes the Stagger away on both.
#[test]
fn the_live_scene_gives_children_their_member_time() {
    let (mut rt, kids, at) = staggered_children();
    let view = rt.doc.view();
    let scene = rt.engine.frame_graph_editor_scene(&view, at).unwrap();
    let read: Vec<f32> = kids.iter().map(|k| scene.layer(*k).map_or(f32::NAN, |l| l.opacity)).collect();
    let s = sorted(read.clone());
    assert!((s[0] - 0.5).abs() < 0.03 && (s[1] - 0.4).abs() < 0.03 && (s[2] - 0.3).abs() < 0.03, "live scene: {read:?}");
    let doc: Vec<f32> = kids.iter().map(|k| match view.value_at(*k, &PropertyId::new(property::OPACITY).unwrap(), at).unwrap() { Some(Value::F64(v)) => v as f32, _ => f32::NAN }).collect();
    for (live, doc) in read.iter().zip(&doc) {
        assert!((live - doc).abs() < 1e-4, "live {read:?} vs document {doc:?}");
    }
    drop(view);
    rt.request(json!({"op": "undo"})).unwrap();
    let view = rt.doc.view();
    let scene = rt.engine.frame_graph_editor_scene(&view, at).unwrap();
    let undone: Vec<f32> = kids.iter().map(|k| scene.layer(*k).map_or(f32::NAN, |l| l.opacity)).collect();
    assert!(undone.iter().all(|o| (o - 0.5).abs() < 0.03), "undo: {undone:?}");
}

/// Loop, the other time mapping the clock carries: a keyed layer with Loop Duration 1 s reads 0.5 at 2.5 s, on the live
/// scene as in the document; Alternate reads the way back.
#[test]
fn the_live_scene_folds_time_by_loop_as_the_document_does() {
    let mut rt = crate::EditorRuntime::open("").unwrap();
    rt.request(json!({"op": "create", "kind": "rectangle"})).unwrap();
    let layer = rt.viewer.selected().unwrap();
    rt.request(json!({"op": "animate", "enabled": true})).unwrap();
    for (frame, v) in [(0, 0.0), (30, 1.0)] {
        rt.request(json!({"op": "seek", "frame": frame})).unwrap();
        rt.request(json!({"op": "setProperty", "layer": layer.0, "property": "opacity", "value": v})).unwrap();
    }
    rt.request(json!({"op": "animate", "enabled": false})).unwrap();
    rt.request(json!({"op": "setProperty", "layer": layer.0, "property": "layout.loop_duration", "value": 1.0})).unwrap();
    let fps = rt.doc.view().composition().unwrap().unwrap().fps;
    let read = |rt: &mut EditorRuntime, frame: i64| -> (f32, f32) {
        let at = RationalTime::try_from_frame(frame, fps).unwrap();
        let view = rt.doc.view();
        let doc = match view.value_at(layer, &PropertyId::new(property::OPACITY).unwrap(), at).unwrap() { Some(Value::F64(v)) => v as f32, _ => f32::NAN };
        let live = rt.engine.frame_graph_editor_scene(&view, at).unwrap().layer(layer).map_or(f32::NAN, |l| l.opacity);
        (doc, live)
    };
    let (doc, live) = read(&mut rt, 75);
    assert!((doc - 0.5).abs() < 0.02 && (live - doc).abs() < 1e-4, "Normal at 2.5 s: document {doc}, live {live}");
    rt.request(json!({"op": "setProperty", "layer": layer.0, "property": "layout.loop_direction", "value": 2})).unwrap();
    let (doc, live) = read(&mut rt, 36);
    assert!((doc - 0.8).abs() < 0.02 && (live - doc).abs() < 1e-4, "Alternate at 1.2 s: document {doc}, live {live}");
}


/// The third member source on the live scene: a split text's words, each the whole text at its own time cut to its box,
/// as the document's resolve path makes them (`push_split`). Same count, same opacity per unit, same cut.
#[test]
fn the_live_scene_cuts_a_split_text_into_units() {
    let mut rt = crate::EditorRuntime::open("").unwrap();
    rt.request(json!({"op": "create", "kind": "text"})).unwrap();
    let text = rt.viewer.selected().unwrap();
    rt.request(json!({"op": "setText", "layer": text.0, "content": "ONE TWO THREE"})).unwrap();
    rt.request(json!({"op": "animate", "enabled": true})).unwrap();
    for (frame, v) in [(0, 0.0), (30, 1.0)] {
        rt.request(json!({"op": "seek", "frame": frame})).unwrap();
        rt.request(json!({"op": "setProperty", "layer": text.0, "property": "opacity", "value": v})).unwrap();
    }
    rt.request(json!({"op": "animate", "enabled": false})).unwrap();
    rt.request(json!({"op": "setProperty", "layer": text.0, "property": "text_split", "value": 2})).unwrap();
    let fps = rt.doc.view().composition().unwrap().unwrap().fps;
    let at = RationalTime::try_from_frame(15, fps).unwrap();
    let units = |rt: &mut EditorRuntime| -> (Vec<(u32, f32, usize)>, Vec<(u32, f32, usize)>) {
        let view = rt.doc.view();
        let mut doc: Vec<(u32, f32, usize)> = crate::render::picture::resolve::resolved_layers(&view, at).unwrap().iter().filter(|l| l.id == text).map(|l| (l.copy, l.placement.opacity, l.masks.len())).collect();
        let scene = rt.engine.frame_graph_editor_scene(&view, at).unwrap();
        let mut live: Vec<(u32, f32, usize)> = scene.layers.iter().filter(|l| l.layer == text && !l.ghost).map(|l| (l.instance, l.opacity, l.masks.len())).collect();
        doc.sort_by_key(|u| u.0);
        live.sort_by_key(|u| u.0);
        (doc, live)
    };
    let (doc, live) = units(&mut rt);
    assert_eq!((doc.len(), live.len()), (1, 1), "Split with no Stagger: one layer on both");
    rt.request(json!({"op": "setProperty", "layer": text.0, "property": "layout.stagger", "value": 0.2})).unwrap();
    let (doc, live) = units(&mut rt);
    assert_eq!(doc.len(), 3);
    assert_eq!(live.len(), doc.len(), "live {live:?} vs document {doc:?}");
    for (l, d) in live.iter().zip(&doc) {
        assert_eq!(l.0, d.0);
        assert!((l.1 - d.1).abs() < 1e-3, "unit {}: live {} vs document {}", l.0, l.1, d.1);
        assert_eq!(l.2, d.2, "each unit carries its cut");
    }
    let mut opacities: Vec<f32> = live.iter().map(|u| u.1).collect();
    opacities.sort_by(|a, b| b.total_cmp(a));
    assert!((opacities[0] - 0.5).abs() < 0.03 && (opacities[2] - 0.3).abs() < 0.03, "{opacities:?}");
}

/// Every member of every layer, live against the document's resolve path, at a few frames: (layer, instance) → opacity.
fn assert_parity(rt: &mut EditorRuntime, frames: &[i64], what: &str) {
    let fps = rt.doc.view().composition().unwrap().unwrap().fps;
    for &f in frames {
        let at = RationalTime::try_from_frame(f, fps).unwrap();
        let view = rt.doc.view();
        let mut doc: Vec<(u64, u32, i32)> = crate::render::picture::resolve::resolved_layers(&view, at).unwrap().iter()
            .filter(|l| !l.ghost).map(|l| (l.id.0, l.copy, (l.placement.opacity * 1000.0).round() as i32)).collect();
        let scene = rt.engine.frame_graph_editor_scene(&view, at).unwrap();
        let mut live: Vec<(u64, u32, i32)> = scene.layers.iter().filter(|l| !l.ghost && l.source != LayerSource::Group)
            .map(|l| (l.layer.0, l.instance, (l.opacity * 1000.0).round() as i32)).collect();
        doc.retain(|d| live.iter().any(|l| l.0 == d.0) || view.meta(LayerId(d.0)).unwrap().is_some_and(|m| m.source != LayerSource::Group));
        doc.sort();
        live.sort();
        assert_eq!(live, doc, "{what} at frame {f}");
    }
}

fn keyed_opacity(rt: &mut EditorRuntime, layers: &[LayerId]) {
    rt.request(json!({"op": "animate", "enabled": true})).unwrap();
    for (frame, v) in [(0, 0.0), (30, 1.0)] {
        rt.request(json!({"op": "seek", "frame": frame})).unwrap();
        for l in layers {
            rt.request(json!({"op": "setProperty", "layer": l.0, "property": "opacity", "value": v})).unwrap();
        }
    }
    rt.request(json!({"op": "animate", "enabled": false})).unwrap();
}

/// Nested members: a staggered group holding a split, staggered text and a shape with a staggered Repeater and a Delay
/// Each. Each member reads global time, less its own member delay, then its ancestors' order shifts, then its Loop:
/// live and document agree member by member.
#[test]
fn nested_members_keep_their_own_time_live_as_in_the_document() {
    let mut rt = crate::EditorRuntime::open("").unwrap();
    rt.request(json!({"op": "create", "kind": "text"})).unwrap();
    let text = rt.viewer.selected().unwrap();
    rt.request(json!({"op": "setText", "layer": text.0, "content": "ONE TWO THREE"})).unwrap();
    rt.request(json!({"op": "create", "kind": "rectangle"})).unwrap();
    let shape = rt.viewer.selected().unwrap();
    rt.request(json!({"op": "applyEffect", "pluginIds": ["motolii.repeat"]})).unwrap();
    let effect = rt.doc.view().effects(shape).unwrap()[0].id.0;
    rt.request(json!({"op": "setProperty", "layer": shape.0, "property": format!("effect.{effect}.param.delay_each"), "value": 0.05})).unwrap();
    keyed_opacity(&mut rt, &[text, shape]);
    rt.request(json!({"op": "select", "ids": [text.0, shape.0]})).unwrap();
    rt.request(json!({"op": "group"})).unwrap();
    let group = rt.viewer.selected().unwrap();
    assert_parity(&mut rt, &[0, 10, 20], "no stagger");
    rt.request(json!({"op": "setProperty", "layer": group.0, "property": "layout.stagger", "value": 0.2})).unwrap();
    rt.request(json!({"op": "setProperty", "layer": text.0, "property": "text_split", "value": 2})).unwrap();
    rt.request(json!({"op": "setProperty", "layer": text.0, "property": "layout.stagger", "value": 0.2})).unwrap();
    rt.request(json!({"op": "setProperty", "layer": shape.0, "property": "layout.stagger", "value": 0.2})).unwrap();
    rt.request(json!({"op": "setProperty", "layer": shape.0, "property": "layout.stagger_from", "value": 2})).unwrap();
    assert_parity(&mut rt, &[5, 12, 20, 33], "nested staggers");
    rt.request(json!({"op": "setProperty", "layer": text.0, "property": "layout.loop_duration", "value": 0.6})).unwrap();
    assert_parity(&mut rt, &[5, 12, 20, 33], "nested staggers and a text loop");
    rt.request(json!({"op": "undo"})).unwrap();
    rt.request(json!({"op": "undo"})).unwrap();
    assert_parity(&mut rt, &[5, 20], "after undo");
}
