use super::*;
/// 揃った素材で静的な 1 枚を組む道具。`MOTOLII_DESIGN_OUT` に書類を保存する。
/// `cargo test -p motolii-ui --lib poster -- --ignored` で走る。
#[test]
#[ignore]
fn compose_static_poster() {
    let Some(out)=std::env::var_os("MOTOLII_DESIGN_OUT") else { return };
    let mut rt=EditorRuntime::open("").unwrap();
    fn go(rt:&mut EditorRuntime,j:serde_json::Value){rt.request(j.clone()).unwrap_or_else(|e|panic!("{j}: {e}"));}
    go(&mut rt,json!({"op":"composition","width":1920,"height":1080,"background":[0.06,0.06,0.08,1.0]}));
    fn make(rt:&mut EditorRuntime,kind:&str,name:&str)->LayerId{go(rt,json!({"op":"create","kind":kind}));let id=rt.viewer.selected().unwrap();go(rt,json!({"op":"setAttrs","layers":[id.0],"patch":{"name":name}}));id}
    let set=|rt:&mut EditorRuntime,id:LayerId,prop:&str,v:serde_json::Value|rt.request(json!({"op":"setProperty","layer":id.0,"property":prop,"value":v})).unwrap();
    let paint=|rt:&mut EditorRuntime,id:LayerId,rgba:[f64;4]|{rt.request(json!({"op":"select","ids":[id.0]})).unwrap();rt.request(json!({"op":"applyPalette","rgba":rgba})).unwrap();};
    let effect=|rt:&mut EditorRuntime,id:LayerId,plugin:&str,params:&[(&str,serde_json::Value)]|{
        rt.request(json!({"op":"select","ids":[id.0]})).unwrap();rt.request(json!({"op":"applyEffect","pluginId":plugin})).unwrap();
        let eid=rt.doc.view().effects(id).unwrap().last().unwrap().id.0;
        for (name,v) in params { rt.request(json!({"op":"setProperty","layer":id.0,"property":format!("effect.{eid}.param.{name}"),"value":v})).unwrap(); }
    };
    // 1. 光: 大きな楕円に放射の gradient。右上へ寄せる。
    let glow=make(&mut rt,"ellipse","Glow");
    // gradient の軸は書類の形の大きさで決まるので、広げるのは層の scale で。
    set(&mut rt,glow,"scale",json!([5.5,5.5]));set(&mut rt,glow,"position",json!([708.0,-492.0]));
    rt.request(json!({"op":"setGradient","slot":{"ShapeFill":{"layer":glow.0,"path":[0]}},"kind":"radial","stops":[[0.98,0.62,0.22],[0.55,0.18,0.30],[0.06,0.06,0.08]]})).unwrap();
    // 2. 花: 12 角の星を Pucker & Bloat で丸める。深い青。
    let bloom=make(&mut rt,"star","Bloom");
    set(&mut rt,bloom,"shape.points",json!(12.0));set(&mut rt,bloom,"shape.outer_radius",json!(300.0));set(&mut rt,bloom,"shape.inner_radius",json!(190.0));
    set(&mut rt,bloom,"position",json!([260.0,330.0]));paint(&mut rt,bloom,[0.16,0.28,0.78,1.0]);
    effect(&mut rt,bloom,crate::render::extensions::pathop::PUCKER_BLOAT,&[("amount",json!(45.0))]);
    // 3. 荒い円盤: 細かい星を Wiggle で毛羽立たせる。珊瑚色。
    let disc=make(&mut rt,"star","Rough disc");
    set(&mut rt,disc,"shape.points",json!(28.0));set(&mut rt,disc,"shape.outer_radius",json!(330.0));set(&mut rt,disc,"shape.inner_radius",json!(300.0));
    set(&mut rt,disc,"position",json!([950.0,300.0]));paint(&mut rt,disc,[0.98,0.42,0.36,1.0]);
    effect(&mut rt,disc,crate::render::extensions::pathop::WIGGLE_PATHS,&[("size",json!(7.0)),("detail",json!(2.0)),("seed",json!(4.0))]);
    // 4. 曲がった帯: 細長い矩形を Bend で弓なりに。薄い黄。
    let band=make(&mut rt,"rectangle","Band");
    set(&mut rt,band,"shape.size",json!([1500.0,46.0]));set(&mut rt,band,"position",json!([220.0,820.0]));paint(&mut rt,band,[0.99,0.86,0.45,1.0]);
    effect(&mut rt,band,crate::render::extensions::pathop::BEND,&[("angle",json!(-28.0)),("center",json!([750.0,23.0]))]);
    // 5. 破線: 線を横へ伸ばし、Chop で刻む。
    let dash=make(&mut rt,"line","Dashes");
    set(&mut rt,dash,"position",json!([136.0,470.0]));set(&mut rt,dash,"scale",json!([3.2,1.0]));
    effect(&mut rt,dash,crate::render::extensions::pathop::CHOP_PATH,&[("length",json!(18.0)),("gap",json!(9.0))]);
    // 6. 波線: Zig Zag を Smooth 点で。
    let wave=make(&mut rt,"line","Wave");
    set(&mut rt,wave,"position",json!([136.0,512.0]));set(&mut rt,wave,"scale",json!([3.2,1.0]));
    effect(&mut rt,wave,crate::render::extensions::pathop::ZIG_ZAG,&[("amplitude",json!(12.0)),("frequency",json!(9.0)),("point_type",json!(1.0))]);
    // 7. 六角の滴: 多角形を Subdivide → Smooth → Wiggle で有機的に。回して 2.5D。
    let drop=make(&mut rt,"polygon","Drop");
    set(&mut rt,drop,"shape.points",json!(6.0));set(&mut rt,drop,"shape.outer_radius",json!(150.0));
    set(&mut rt,drop,"position",json!([1420.0,120.0]));set(&mut rt,drop,"rotation",json!(18.0));paint(&mut rt,drop,[0.93,0.95,0.98,1.0]);
    effect(&mut rt,drop,crate::render::extensions::pathop::SUBDIVIDE,&[("divisions",json!(3.0))]);
    effect(&mut rt,drop,crate::render::extensions::pathop::WIGGLE_PATHS,&[("size",json!(14.0)),("detail",json!(1.0)),("point_type",json!(1.0)),("seed",json!(9.0))]);
    // 8. 円柱: 右下に小さく。3D のまま。
    let can=make(&mut rt,"cylinder","Can");
    set(&mut rt,can,"position",json!([1660.0,900.0]));set(&mut rt,can,"scale",json!([0.55,0.55]));set(&mut rt,can,"rotation",json!(22.0));
    // 9. 文字: 題と添え書き。
    let title=make(&mut rt,"text","Title");
    go(&mut rt,json!({"op":"setText","layer":title.0,"content":"MOTOLII"}));
    go(&mut rt,json!({"op":"styleText","layer":title.0,"scope":"all","size":230.0}));
    set(&mut rt,title,"position",json!([130.0,110.0]));paint(&mut rt,title,[0.98,0.97,0.95,1.0]);
    let sub=make(&mut rt,"text","Subtitle");
    go(&mut rt,json!({"op":"setText","layer":sub.0,"content":"shapes · paths · effects"}));
    go(&mut rt,json!({"op":"styleText","layer":sub.0,"scope":"all","size":54.0}));
    set(&mut rt,sub,"position",json!([136.0,350.0]));paint(&mut rt,sub,[0.98,0.86,0.45,1.0]);
    rt.request(json!({"op":"save","path":out.to_string_lossy()})).unwrap();
}
