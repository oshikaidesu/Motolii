use super::*;
fn layer<'a>(status:&'a serde_json::Value,id:LayerId)->&'a serde_json::Value{status["layers"].as_array().unwrap().iter().find(|l|l["id"].as_u64()==Some(id.0)).unwrap()}
fn start(rt:&mut EditorRuntime,id:LayerId)->i64{layer(&rt.build_status().unwrap(),id)["start"].as_i64().unwrap()}
fn key_frames(rt:&mut EditorRuntime,id:LayerId,property:&str)->Vec<i64>{
    let status=rt.build_status().unwrap();
    let row=layer(&status,id)["properties"].as_array().unwrap().iter().find(|p|p["id"]==property).unwrap().clone();
    row["keys"].as_array().unwrap().iter().map(|k|k["frame"].as_i64().unwrap()).collect()
}
fn drag(rt:&mut EditorRuntime,a:LayerId,b:LayerId,base:(i64,i64),steps:&[i64]){
    for d in steps{rt.request(json!({"op":"previewTimings","changes":[
        {"layer":a.0,"start":base.0+d,"duration":40,"sourceIn":0},
        {"layer":b.0,"start":base.1+d,"duration":40,"sourceIn":0}]})).unwrap();}
    rt.request(json!({"op":"commitPreview"})).unwrap();
}
/// 二つ選んで帯を掴んで動かす: 途中の preview が何回来ても、離した所に両方が
/// 同じ差で着き、キーも帯と一緒に動く。二度目のドラッグも同じ。
#[test]
fn a_multi_layer_move_lands_both_layers_and_their_keys_by_one_delta(){
    let mut rt=EditorRuntime::open("").unwrap();
    rt.request(json!({"op":"create","kind":"rectangle"})).unwrap();let a=rt.viewer.selected().unwrap();
    rt.request(json!({"op":"create","kind":"rectangle"})).unwrap();let b=rt.viewer.selected().unwrap();
    rt.request(json!({"op":"setTiming","layer":a.0,"start":10,"duration":40,"sourceIn":0})).unwrap();
    rt.request(json!({"op":"setTiming","layer":b.0,"start":30,"duration":40,"sourceIn":0})).unwrap();
    rt.request(json!({"op":"select","ids":[a.0]})).unwrap();
    rt.request(json!({"op":"seek","frame":12})).unwrap();
    rt.request(json!({"op":"toggleKey","layer":a.0,"property":"opacity"})).unwrap();
    let key_before=key_frames(&mut rt,a,"opacity");
    rt.request(json!({"op":"select","ids":[a.0,b.0]})).unwrap();
    drag(&mut rt,a,b,(10,30),&[3,5]);
    assert_eq!((start(&mut rt,a),start(&mut rt,b)),(15,35),"first drag: both land at +5");
    assert_eq!(key_frames(&mut rt,a,"opacity"),key_before.iter().map(|f|f+5).collect::<Vec<_>>(),"the key rides with its layer");
    drag(&mut rt,a,b,(15,35),&[2,4]);
    assert_eq!((start(&mut rt,a),start(&mut rt,b)),(19,39),"second drag: +4 from where the first left them");
    assert_eq!(key_frames(&mut rt,a,"opacity"),key_before.iter().map(|f|f+9).collect::<Vec<_>>());
    // 後ろへ戻すのも同じ。
    drag(&mut rt,a,b,(19,39),&[-1,-3,-6]);
    assert_eq!((start(&mut rt,a),start(&mut rt,b)),(13,33));
    assert_eq!(key_frames(&mut rt,a,"opacity"),key_before.iter().map(|f|f+3).collect::<Vec<_>>());
}
fn key_at(rt:&mut EditorRuntime,id:LayerId,frame:i64){
    rt.request(json!({"op":"seek","frame":frame})).unwrap();
    rt.request(json!({"op":"toggleKey","layer":id.0,"property":"opacity"})).unwrap();
}
/// キーを複数選んで動かす: 同じ track で隣のキーの場所へ乗る動きでも 1 つも
/// 消えず、別の層のキーも同じ差で動き、選択は動いた先を指し続ける。
#[test]
fn selected_keys_on_one_track_and_two_layers_move_together(){
    let mut rt=EditorRuntime::open("").unwrap();
    rt.request(json!({"op":"create","kind":"rectangle"})).unwrap();let a=rt.viewer.selected().unwrap();
    rt.request(json!({"op":"create","kind":"rectangle"})).unwrap();let b=rt.viewer.selected().unwrap();
    rt.request(json!({"op":"select","ids":[a.0]})).unwrap();
    for f in [10,20,40]{key_at(&mut rt,a,f);}
    rt.request(json!({"op":"select","ids":[b.0]})).unwrap();
    key_at(&mut rt,b,15);
    assert_eq!(key_frames(&mut rt,a,"opacity"),vec![10,20,40]);
    rt.request(json!({"op":"select","ids":[a.0,b.0],"keys":[
        {"layer":a.0,"property":"opacity","frame":10},
        {"layer":a.0,"property":"opacity","frame":20},
        {"layer":b.0,"property":"opacity","frame":15}]})).unwrap();
    rt.request(json!({"op":"moveKeys","deltaFrames":10})).unwrap();
    assert_eq!(key_frames(&mut rt,a,"opacity"),vec![20,30,40],"10 lands where 20 was, and 20 has moved on");
    assert_eq!(key_frames(&mut rt,b,"opacity"),vec![25]);
    rt.request(json!({"op":"moveKeys","deltaFrames":-10})).unwrap();
    assert_eq!(key_frames(&mut rt,a,"opacity"),vec![10,20,40]);
    assert_eq!(key_frames(&mut rt,b,"opacity"),vec![15]);
    rt.request(json!({"op":"moveKeys","deltaFrames":3})).unwrap();
    assert_eq!(key_frames(&mut rt,a,"opacity"),vec![13,23,40],"the selection followed the keys");
    assert_eq!(key_frames(&mut rt,b,"opacity"),vec![18]);
}
/// 0 より前へは行かない: 一番早いキーが 0 で止まり、残りは間隔を保つ。
#[test]
fn keys_moved_past_frame_zero_stop_at_zero_together(){
    let mut rt=EditorRuntime::open("").unwrap();
    rt.request(json!({"op":"create","kind":"rectangle"})).unwrap();let a=rt.viewer.selected().unwrap();
    for f in [5,20]{key_at(&mut rt,a,f);}
    rt.request(json!({"op":"select","ids":[a.0],"keys":[
        {"layer":a.0,"property":"opacity","frame":5},
        {"layer":a.0,"property":"opacity","frame":20}]})).unwrap();
    let reply=rt.request(json!({"op":"moveKeys","deltaFrames":-10}));
    let frames=key_frames(&mut rt,a,"opacity");
    assert!(frames.iter().all(|f|*f>=0),"no key may sit before frame 0 (reply {reply:?}, keys {frames:?})");
    assert_eq!(frames,vec![0,15],"the earliest stops at 0, the other keeps its distance");
}
fn order(rt:&mut EditorRuntime)->Vec<u64>{rt.build_status().unwrap()["layers"].as_array().unwrap().iter().map(|l|l["id"].as_u64().unwrap()).collect()}
fn pos(order:&[u64],id:LayerId)->usize{order.iter().position(|x|*x==id.0).unwrap()}
/// 行を複数掴んで上下へ落とす: 選んだ順に関わらず、元の並びのまま 1 つの塊で
/// 落とし先の隣に着く。
#[test]
fn a_multi_row_drop_keeps_the_block_in_display_order(){
    let mut rt=EditorRuntime::open("").unwrap();
    let mut make=|rt:&mut EditorRuntime|{rt.request(json!({"op":"create","kind":"rectangle"})).unwrap();rt.viewer.selected().unwrap()};
    let (a,b,c,d)=(make(&mut rt),make(&mut rt),make(&mut rt),make(&mut rt));
    let before=order(&mut rt);
    let a_above_c=pos(&before,a)<pos(&before,c);
    // 選んだ順は c, a — 並びは a, c のまま。
    rt.request(json!({"op":"select","ids":[c.0,a.0]})).unwrap();
    rt.request(json!({"op":"moveLayers","layers":[c.0,a.0],"target":d.0,"placement":"after"})).unwrap();
    let now=order(&mut rt);
    assert_eq!(pos(&now,a)<pos(&now,c),a_above_c,"the block keeps its own order");
    assert_eq!((pos(&now,a) as i64-pos(&now,c) as i64).abs(),1,"the block stays contiguous");
    assert_eq!(pos(&now,a).min(pos(&now,c)),pos(&now,d)+1,"and sits right after the target");
    rt.request(json!({"op":"moveLayers","layers":[c.0,a.0],"target":b.0,"placement":"before"})).unwrap();
    let now=order(&mut rt);
    assert_eq!(pos(&now,a)<pos(&now,c),a_above_c);
    assert_eq!(pos(&now,a).max(pos(&now,c))+1,pos(&now,b),"right before the target");
}

mod font_shortcut_tests {
use super::*;
/// Fonts 棚の行を何も選ばずに押す = その書体の文字の層を 1 手で作る。無い書体は断る。
#[test]
fn creating_text_with_a_family_is_one_step() {
    let mut rt=EditorRuntime::open("").unwrap();
    let family=crate::render::picture::shaping::font_families().first().cloned().expect("a font");
    let before=rt.doc.history_depth().0;
    rt.request(json!({"op":"create","kind":"text","family":family})).unwrap();
    assert_eq!(rt.doc.history_depth().0,before+1);
    let id=rt.viewer.selected().expect("the new layer is picked");
    assert_eq!(rt.doc.view().text_document(id).unwrap().unwrap().styles[0].font.family,family);
    assert!(rt.request(json!({"op":"create","kind":"text","family":"No Such Face 9"})).is_err());
}
}
