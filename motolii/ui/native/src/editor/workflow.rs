//! One authoring workflow on the real document and renderer, without any window: the operations the shells send
//! (create, edit, key, scrub, undo, redo, save, open, export), in the order a person uses them.
use crate::doc::store::{property, PropertyId, RationalTime};
use crate::EditorRuntime;
use serde_json::{json, Value};

fn request(rt: &mut EditorRuntime, op: Value) {
    rt.request(op.clone()).unwrap_or_else(|e| panic!("{op} was refused: {e}"));
}

fn keys(rt: &EditorRuntime, layer: crate::doc::store::LayerId) -> usize {
    let view = rt.doc.view();
    view.track(layer, &PropertyId::new(property::POSITION).unwrap()).unwrap().map_or(0, |t| t.keys().len())
}

fn position_at(rt: &EditorRuntime, layer: crate::doc::store::LayerId, frame: i64) -> Vec<f64> {
    let view = rt.doc.view();
    let fps = view.composition().unwrap().unwrap().fps;
    let at = RationalTime::try_from_frame(frame, fps).unwrap();
    match view.value_at(layer, &PropertyId::new(property::POSITION).unwrap(), at).unwrap() {
        Some(crate::doc::store::Value::Vec2(v)) => v.to_vec(),
        other => panic!("position is {other:?}"),
    }
}

/// About five minutes in a debug build: the first shape drawn starts re_renderer's shader hot-reload watcher, which blocks
/// for a minute or more. Run it with `cargo test -p motolii-ui --lib workflow -- --ignored`.
#[test]
#[ignore = "slow in debug builds (shader watcher cold start); run on demand"]
fn make_move_key_scrub_undo_redo_save_reopen_and_export() {
    let mut rt = EditorRuntime::open("").unwrap();

    // Make a thing, and move it (still: no key).
    request(&mut rt, json!({"op": "create", "kind": "rectangle"}));
    let layer = rt.viewer.selected().expect("the new layer is selected");
    request(&mut rt, json!({"op": "setProperty", "layer": layer.0, "property": "position", "value": [400.0, 300.0]}));
    assert_eq!(keys(&rt, layer), 0, "a still value has no keys");
    assert_eq!(position_at(&rt, layer, 0), vec![400.0, 300.0]);

    // Animate: a value touched becomes a key at that frame. Two keys make a move.
    request(&mut rt, json!({"op": "animate", "enabled": true}));
    request(&mut rt, json!({"op": "seek", "frame": 0}));
    request(&mut rt, json!({"op": "setProperty", "layer": layer.0, "property": "position", "value": [400.0, 300.0]}));
    request(&mut rt, json!({"op": "seek", "frame": 30}));
    request(&mut rt, json!({"op": "setProperty", "layer": layer.0, "property": "position", "value": [800.0, 300.0]}));
    request(&mut rt, json!({"op": "animate", "enabled": false}));
    assert_eq!(keys(&rt, layer), 2);

    // Scrub: halfway is halfway.
    request(&mut rt, json!({"op": "seek", "frame": 15}));
    let mid = position_at(&rt, layer, 15);
    assert!((mid[0] - 600.0).abs() < 1.0 && (mid[1] - 300.0).abs() < 1e-6, "linear halfway: {mid:?}");

    // Undo takes the last key away; Redo brings it back, and the frame is where it was.
    request(&mut rt, json!({"op": "undo"}));
    assert_eq!(keys(&rt, layer), 1);
    request(&mut rt, json!({"op": "redo"}));
    assert_eq!(keys(&rt, layer), 2);
    assert_eq!(position_at(&rt, layer, 30), vec![800.0, 300.0]);

    // Save, and a fresh session opens it with the same move.
    let dir = std::env::temp_dir().join(format!("motolii-workflow-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("work.rrd");
    request(&mut rt, json!({"op": "save", "path": file.to_string_lossy()}));
    let mut again = EditorRuntime::open(&file.to_string_lossy()).unwrap();
    let reopened = again.doc.view().layers()[0];
    assert_eq!(keys(&again, reopened), 2);
    assert_eq!(position_at(&again, reopened, 30), vec![800.0, 300.0]);
    request(&mut again, json!({"op": "undo"}));
    let _ = again;

    // Export a few frames of it.
    let out = dir.join("out.mp4");
    request(&mut rt, json!({"op": "export", "path": out.to_string_lossy(), "start": 0, "end": 6}));
    let started = std::time::Instant::now();
    let status = loop {
        let status = rt.exporter.status();
        if !matches!(status["phase"].as_str(), Some("running" | "cancelling" | "starting")) {
            break status;
        }
        assert!(started.elapsed().as_secs() < 400, "the export never finished: {status}");
        std::thread::sleep(std::time::Duration::from_millis(200));
    };
    eprintln!("WORKFLOW export status: {status}");
    assert_eq!(status["phase"], "complete", "{status}");
    assert!(std::fs::metadata(&out).map(|m| m.len() > 0).unwrap_or(false), "an export file was written");
    let _ = std::fs::remove_dir_all(&dir);
}
