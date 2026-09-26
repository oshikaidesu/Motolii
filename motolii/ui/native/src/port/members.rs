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
