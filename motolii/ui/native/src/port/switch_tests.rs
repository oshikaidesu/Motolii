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
