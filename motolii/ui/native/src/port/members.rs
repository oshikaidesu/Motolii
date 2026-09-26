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

/// Known gap (2026-09-27, docs/stage5/members.md): the live frame graph evaluates every track at the frame time, so it does
/// not give children their member time (and it does not cut a split text into units). Copies work there because the copy
/// path samples at each copy's time. This test states the parity the renderer owes the document.
#[test]
#[ignore = "known gap: the frame graph has no member time for children or split units"]
fn the_live_scene_gives_children_their_member_time() {
    let (mut rt, kids, at) = staggered_children();
    let view = rt.doc.view();
    let scene = rt.engine.frame_graph_editor_scene(&view, at).unwrap();
    let read: Vec<f32> = kids.iter().map(|k| scene.layer(*k).map_or(f32::NAN, |l| l.opacity)).collect();
    let s = sorted(read.clone());
    assert!((s[0] - 0.5).abs() < 0.03 && (s[2] - 0.3).abs() < 0.03, "live scene: {read:?}");
}

/// Probe: does the live scene cut a split text into its units (the resolve path does, `push_split`)?
#[test]
#[ignore = "known gap: the live scene does not cut a split text into units; see docs/stage5/members.md"]
fn the_live_scene_cuts_a_split_text_into_units() {
    let mut rt = crate::EditorRuntime::open("").unwrap();
    rt.request(json!({"op": "create", "kind": "text"})).unwrap();
    let text = rt.viewer.selected().unwrap();
    rt.request(json!({"op": "setText", "layer": text.0, "content": "ONE TWO THREE"})).unwrap();
    rt.request(json!({"op": "setProperty", "layer": text.0, "property": "text_split", "value": 2})).unwrap();
    rt.request(json!({"op": "setProperty", "layer": text.0, "property": "layout.stagger", "value": 0.2})).unwrap();
    let fps = rt.doc.view().composition().unwrap().unwrap().fps;
    let at = RationalTime::try_from_frame(15, fps).unwrap();
    let view = rt.doc.view();
    let resolved = crate::render::picture::resolve::resolved_layers(&view, at).unwrap().iter().filter(|l| l.id == text).count();
    let scene = rt.engine.frame_graph_editor_scene(&view, at).unwrap();
    let live = scene.layers.iter().filter(|l| l.layer == text && !l.ghost).count();
    assert_eq!(live, resolved, "live scene units vs the document's");
}
