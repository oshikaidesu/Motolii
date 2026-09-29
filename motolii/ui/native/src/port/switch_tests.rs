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
