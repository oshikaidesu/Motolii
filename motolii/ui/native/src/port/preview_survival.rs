//! Which requests end a live preview. A preview is the hand's unfinished edit: an operation that does not touch the
//! document (or the time the preview was made at) must not silently end it, or the Inspector keeps a number the Stage
//! no longer shows. Operations that replace or edit the document, move the time or change the selection do end it.
use super::*;

fn live() -> EditorRuntime {
    let mut rt = EditorRuntime::open("").unwrap();
    rt.request(json!({"op":"create","kind":"rectangle"})).unwrap();
    let layer = rt.viewer.selected().unwrap();
    rt.request(json!({"op":"previewProperties","edits":[{"layer":layer.0,"property":"opacity","value":0.5}]})).unwrap();
    assert!(rt.preview.is_some(), "a preview is open");
    rt
}

fn survives(op: serde_json::Value) {
    let mut rt = live();
    let owner = rt.preview.as_ref().unwrap().0;
    let name = op["op"].as_str().unwrap().to_owned();
    let _ = rt.request(op);
    assert_eq!(rt.preview.as_ref().map(|p| p.0), Some(owner), "{name} must leave the live preview alone");
}

fn ends(op: serde_json::Value) {
    let mut rt = live();
    let name = op["op"].as_str().unwrap().to_owned();
    let _ = rt.request(op);
    assert!(rt.preview.is_none(), "{name} ends the live preview");
}

#[test]
fn viewer_and_read_only_requests_leave_a_live_preview() {
    survives(json!({"op":"status"}));
    survives(json!({"op":"tick"}));
    survives(json!({"op":"reloadEffects"}));
    survives(json!({"op":"preferences","flatProjection":true}));
    survives(json!({"op":"pause"}));
    survives(json!({"op":"stageView","orbit":[10.0,20.0]}));
    survives(json!({"op":"stageView","fit":true}));
    survives(json!({"op":"stageView","reset":true}));
    survives(json!({"op":"stageGesture","phase":"hover","view":"User","point":[1.0,1.0]}));
}

#[test]
fn edits_time_selection_and_replacement_end_a_live_preview() {
    ends(json!({"op":"select","ids":[]}));
    ends(json!({"op":"seek","frame":3}));
    ends(json!({"op":"setProperty","layer":1,"property":"opacity","value":0.2}));
    ends(json!({"op":"undo"}));
    ends(json!({"op":"redo"}));
    ends(json!({"op":"animate","enabled":true}));
    ends(json!({"op":"nudge","dx":1,"dy":0}));
    ends(json!({"op":"delete"}));
    ends(json!({"op":"new"}));
}
