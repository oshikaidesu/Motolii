use super::*;
/// 目・solo・鍵は押した数だけ切り替わる(今の値は host が読む)。1 押しが 1 手。
#[test]
fn a_switch_flips_what_the_document_holds(){
    let mut rt=EditorRuntime::open("").unwrap();
    rt.request(json!({"op":"create","kind":"rectangle"})).unwrap();
    let id=rt.viewer.selected().unwrap();
    let hidden=|rt:&EditorRuntime|rt.doc.view().attrs(id).unwrap().unwrap().hidden;
    rt.request(json!({"op":"toggle","layer":id.0,"flag":"hidden"})).unwrap();
    assert!(hidden(&rt));
    rt.request(json!({"op":"toggle","layer":id.0,"flag":"hidden"})).unwrap();
    assert!(!hidden(&rt),"a second press shows it again");
    rt.request(json!({"op":"undo"})).unwrap();
    assert!(hidden(&rt),"one press, one undo");
    // the lock turns on, and off again (a locked layer's other switches stay as the document rules)
    rt.request(json!({"op":"toggle","layer":id.0,"flag":"locked"})).unwrap();
    assert!(rt.doc.view().attrs(id).unwrap().unwrap().locked);
    rt.request(json!({"op":"toggle","layer":id.0,"flag":"locked"})).unwrap();
    assert!(!rt.doc.view().attrs(id).unwrap().unwrap().locked);
}

/// The host decides which layers an edit reaches: an attribute edit spread from the shown layer reaches the other
/// selected layers that can be edited, not a locked one.
#[test]
fn an_attribute_spread_reaches_the_editable_selection(){
    let mut rt=EditorRuntime::open("").unwrap();
    let mut made=Vec::new();
    for _ in 0..3{rt.request(json!({"op":"create","kind":"rectangle"})).unwrap();made.push(rt.viewer.selected().unwrap());}
    rt.request(json!({"op":"toggle","layer":made[2].0,"flag":"locked"})).unwrap();
    rt.request(json!({"op":"select","ids":[made[0].0,made[1].0,made[2].0]})).unwrap();
    rt.request(json!({"op":"setAttrs","layer":made[0].0,"spread":true,"patch":{"projection":"3D"}})).unwrap();
    let projection=|rt:&EditorRuntime,l:LayerId|rt.doc.view().attrs(l).unwrap().unwrap().projection;
    assert_eq!(projection(&rt,made[0]),LayerProjection::ThreeD);
    assert_eq!(projection(&rt,made[1]),LayerProjection::ThreeD);
    assert_ne!(projection(&rt,made[2]),LayerProjection::ThreeD,"a locked layer is left as it is");
}

/// Up and Down step through the layers in the order the status lists them, from the last chosen one, and stop at the ends.
#[test]
fn stepping_through_layers_follows_the_listed_order(){
    let mut rt=EditorRuntime::open("").unwrap();
    for _ in 0..3{rt.request(json!({"op":"create","kind":"rectangle"})).unwrap();}
    let order=crate::snapshot::stacking_order(&rt.doc.view());
    rt.request(json!({"op":"select","ids":[]})).unwrap();
    rt.request(json!({"op":"select","step":1})).unwrap();
    assert_eq!(rt.viewer.selected_ids,vec![order[0]],"from nothing, the top layer");
    rt.request(json!({"op":"select","step":1})).unwrap();
    assert_eq!(rt.viewer.selected_ids,vec![order[1]]);
    rt.request(json!({"op":"select","step":10})).unwrap();
    assert_eq!(rt.viewer.selected_ids,vec![*order.last().unwrap()],"stops at the bottom");
    rt.request(json!({"op":"select","step":-1})).unwrap();
    assert_eq!(rt.viewer.selected_ids,vec![order[order.len()-2]]);
}

/// A key made with the diamond takes the same ease as a key made by editing the value while animating.
#[test]
fn a_diamond_key_takes_the_animate_ease(){
    let mut rt=EditorRuntime::open("").unwrap();
    rt.request(json!({"op":"create","kind":"rectangle"})).unwrap();
    let id=rt.viewer.selected().unwrap();
    rt.request(json!({"op":"animate","enabled":true,"shape":{"kind":"power2.out"}})).unwrap();
    let chosen=rt.viewer.animate.interp();
    assert_ne!(chosen,Interp::Linear);
    rt.request(json!({"op":"toggleKey","layer":id.0,"property":"opacity"})).unwrap();
    let track=rt.doc.view().track(id,&PropertyId::new("opacity").unwrap()).unwrap().unwrap();
    assert_eq!(track.keys()[0].interp,chosen);
    rt.request(json!({"op":"animate","enabled":false})).unwrap();
    rt.request(json!({"op":"seek","frame":10})).unwrap();
    rt.request(json!({"op":"toggleKey","layer":id.0,"property":"opacity"})).unwrap();
    let track=rt.doc.view().track(id,&PropertyId::new("opacity").unwrap()).unwrap().unwrap();
    assert_eq!(track.keys()[1].interp,Interp::Linear,"not animating: Linear, as before");
}

/// The eyedropper picks and applies in one request: a pick outside the frame applies nothing (not the last colour),
/// a pick inside is one step.
#[test]
fn a_failed_pick_applies_nothing(){
    let mut rt=EditorRuntime::open("").unwrap();
    rt.request(json!({"op":"create","kind":"rectangle"})).unwrap();
    rt.request(json!({"op":"pickColor","x":10.0,"y":10.0,"apply":true})).unwrap();
    let (undo,_)=rt.doc.history_depth();
    assert!(rt.request(json!({"op":"pickColor","x":-5.0,"y":10.0,"apply":true})).is_err());
    assert_eq!(rt.doc.history_depth().0,undo,"nothing applied");
    rt.request(json!({"op":"undo"})).unwrap();
    assert_eq!(rt.doc.history_depth().0,undo-1,"the pick that worked was one step");
}

/// Presses the host resolves from what it holds: Cmd-click in and out, Cmd+A, the Animate switch, an effect's bypass
/// and its Earlier / Later. Two presses before the UI hears back still undo each other.
#[test]
fn presses_are_resolved_by_the_host(){
    let mut rt=EditorRuntime::open("").unwrap();
    let mut made=Vec::new();
    for _ in 0..3{rt.request(json!({"op":"create","kind":"rectangle"})).unwrap();made.push(rt.viewer.selected().unwrap());}
    rt.request(json!({"op":"select","ids":[made[0].0]})).unwrap();
    rt.request(json!({"op":"select","toggle":made[1].0})).unwrap();
    assert_eq!(rt.viewer.selected_ids,vec![made[0],made[1]]);
    rt.request(json!({"op":"select","toggle":made[1].0})).unwrap();
    assert_eq!(rt.viewer.selected_ids,vec![made[0]]);
    rt.request(json!({"op":"select","all":true})).unwrap();
    assert_eq!(rt.viewer.selected_ids.len(),3);

    rt.request(json!({"op":"animate","toggle":true})).unwrap();
    rt.request(json!({"op":"animate","toggle":true})).unwrap();
    assert!(matches!(rt.viewer.animate,crate::edit::Animate::Off),"two presses: off again");

    let id=made[0];
    rt.request(json!({"op":"select","ids":[id.0]})).unwrap();
    let plugins:Vec<String>=crate::render::engine::known_effects().iter().take(2).map(|d|d.plugin_id.clone()).collect();
    rt.request(json!({"op":"applyEffect","pluginIds":plugins})).unwrap();
    let effects=|rt:&EditorRuntime|rt.doc.view().effects(id).unwrap();
    let first=effects(&rt)[0].id;
    let enabled=|rt:&EditorRuntime|!matches!(rt.doc.view().value_at(id,&PropertyId::effect_enabled(first),rt.time().unwrap()).unwrap(),Some(Value::Bool(false)));
    rt.request(json!({"op":"enableEffect","layer":id.0,"id":first.0})).unwrap();
    assert!(!enabled(&rt));
    rt.request(json!({"op":"enableEffect","layer":id.0,"id":first.0})).unwrap();
    assert!(enabled(&rt),"two presses: on again");
    rt.request(json!({"op":"moveEffect","layer":id.0,"id":first.0,"step":1})).unwrap();
    assert_eq!(effects(&rt)[1].id,first,"later by one");
    rt.request(json!({"op":"moveEffect","layer":id.0,"id":first.0,"step":1})).unwrap();
    assert_eq!(effects(&rt)[1].id,first,"already last: stays");
}

/// A group's diamond and ↺ are one step each; a reset spread over the selection puts every layer to its own default.
#[test]
fn a_group_key_and_a_reset_are_one_step(){
    let mut rt=EditorRuntime::open("").unwrap();
    let mut made=Vec::new();
    for x in [100.0,300.0]{rt.request(json!({"op":"create","kind":"rectangle"})).unwrap();let l=rt.viewer.selected().unwrap();
        rt.request(json!({"op":"setProperty","layer":l.0,"property":"position","value":[x,40.0]})).unwrap();made.push(l);}
    let depth=|rt:&EditorRuntime|rt.doc.history_depth().0;
    let value=|rt:&EditorRuntime,l:LayerId,p:&str|rt.doc.view().value_at(l,&PropertyId::new(p).unwrap(),rt.time().unwrap()).unwrap();

    let before=depth(&rt);
    rt.request(json!({"op":"toggleKey","layer":made[0].0,"properties":["position","scale","rotation"]})).unwrap();
    assert_eq!(depth(&rt),before+1,"three rows keyed in one step");
    for p in ["position","scale","rotation"]{assert!(rt.doc.view().track(made[0],&PropertyId::new(p).unwrap()).unwrap().is_some_and(|t|!t.keys().is_empty()),"{p}");}
    rt.request(json!({"op":"undo"})).unwrap();

    rt.request(json!({"op":"select","ids":[made[0].0,made[1].0]})).unwrap();
    let before=depth(&rt);
    rt.request(json!({"op":"reset","layer":made[0].0,"properties":["position"],"spread":true})).unwrap();
    assert_eq!(depth(&rt),before+1);
    assert_eq!(value(&rt,made[0],"position"),Some(Value::Vec2([0.0,0.0])));
    assert_eq!(value(&rt,made[1],"position"),Some(Value::Vec2([0.0,0.0])),"every chosen layer to its own default, both axes");
}
