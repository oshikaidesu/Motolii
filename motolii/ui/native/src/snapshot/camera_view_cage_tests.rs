use super::*;
use crate::EditorRuntime;
use std::ffi::{CStr,CString};
fn request(rt:&mut EditorRuntime,command:Json)->Json{
    let command=CString::new(command.to_string()).unwrap();
    let reply=unsafe{crate::motolii_probe_request(rt,command.as_ptr())};
    let reply:Json=serde_json::from_str(unsafe{CStr::from_ptr(reply)}.to_str().unwrap()).unwrap();
    assert!(reply["error"].is_null(),"{reply}");reply
}
/// 2.5D の文字は canvas の中心で回る。籠も同じ中心で回らなければ、視差の角度が付くほど絵から離れる。
#[test]
fn the_cage_follows_the_drawn_text_under_an_orbited_camera(){
    let mut rt=EditorRuntime::open("").unwrap();
    request(&mut rt,json!({"op":"create","kind":"text"}));
    let text=rt.viewer.selected().unwrap();
    request(&mut rt,json!({"op":"setText","layer":text.0,"content":"Motolii"}));
    request(&mut rt,json!({"op":"styleText","layer":text.0,"scope":"all","size":250.0}));
    request(&mut rt,json!({"op":"setProperty","layer":text.0,"property":"position","value":[109.0,263.0]}));
    let comp=rt.doc.view().composition().unwrap().unwrap().spec();
    for (projection,orbit) in [("2.5D",[0.0,0.0]),("2.5D",[-20.0,40.0]),("2.5D",[35.0,-120.0]),("2D",[-20.0,40.0])] {
        request(&mut rt,json!({"op":"setAttrs","layers":[text.0],"patch":{"projection":projection}}));
        rt.viewer.user_camera=crate::doc::core::ResolvedCamera{orbit_degrees:orbit,..Default::default()};
        let time=rt.time().unwrap();
        let pixels=rt.engine.render_with_camera_override(&rt.doc.view(),time,true,Some(rt.viewer.user_camera)).unwrap();
        let background=&pixels[0..4];
        let (mut px0,mut py0,mut px1,mut py1)=(f64::MAX,f64::MAX,f64::MIN,f64::MIN);
        for (i,px) in pixels.chunks(4).enumerate(){
            if px[..3]!=background[..3]{let x=(i as u32%comp.width) as f64;let y=(i as u32/comp.width) as f64;px0=px0.min(x);py0=py0.min(y);px1=px1.max(x);py1=py1.max(y);}
        }
        assert!(px0<px1,"{projection} {orbit:?}: the text is drawn");
        let bounds=rt.bounds_seen(text,View::User).unwrap();
        let (mut cx0,mut cy0,mut cx1,mut cy1)=(f64::MAX,f64::MAX,f64::MIN,f64::MIN);
        for c in bounds["corners"].as_array().unwrap(){let x=c[0].as_f64().unwrap();let y=c[1].as_f64().unwrap();cx0=cx0.min(x);cy0=cy0.min(y);cx1=cx1.max(x);cy1=cy1.max(y);}
        // 絵は籠からはみ出さない。活字の局所 bounds は縁取りの余白ぶん(8 px 弱)絵より広く、
        // 透視で近い側ほど膨らむので、緩みは 1 辺 30 px までを許す。
        assert!(cx0-1.0<=px0&&cy0-1.0<=py0&&px1<=cx1+1.0&&py1<=cy1+1.0,
            "{projection} {orbit:?}: the drawn [{px0},{py0}]-[{px1},{py1}] escapes the cage [{cx0:.1},{cy0:.1}]-[{cx1:.1},{cy1:.1}]");
        let loose=30.0;
        assert!(px0-cx0<=loose&&py0-cy0<=loose&&cx1-px1<=loose&&cy1-py1<=loose,
            "{projection} {orbit:?}: the cage [{cx0:.1},{cy0:.1}]-[{cx1:.1},{cy1:.1}] is loose around the drawn [{px0},{py0}]-[{px1},{py1}]");
    }
}
/// Stage は世界そのもの(Boxcam): 出力枠の外に置いた層も Stage の窓には描かれ、mask から戻る枠は
/// comp 画像の px。Camera(出力)の絵には載らない。両方の絵が同時に生きる。
#[test]
fn the_stage_window_draws_beyond_the_frame_and_the_camera_does_not(){
    let mut rt=EditorRuntime::open("").unwrap();
    request(&mut rt,json!({"op":"create","kind":"text"}));
    let text=rt.viewer.selected().unwrap();
    request(&mut rt,json!({"op":"setText","layer":text.0,"content":"Outside"}));
    request(&mut rt,json!({"op":"styleText","layer":text.0,"scope":"all","size":200.0}));
    request(&mut rt,json!({"op":"setProperty","layer":text.0,"property":"position","value":[-1500.0,300.0]}));
    let comp=rt.doc.view().composition().unwrap().unwrap().spec();
    let make=|rt:&EditorRuntime,width,height|rt.engine.gpu_device().create_texture(&wgpu::TextureDescriptor {
        label:Some("two views test"),size:wgpu::Extent3d{width,height,depth_or_array_layers:1},mip_level_count:1,sample_count:1,
        dimension:wgpu::TextureDimension::D2,format:crate::render::compositor::PRESENTABLE_FORMAT,usage:wgpu::TextureUsages::RENDER_ATTACHMENT|wgpu::TextureUsages::TEXTURE_BINDING,view_formats:&[],
    });
    // Stage の窓: 出力枠の 4 倍を中央に、960×540 の texture へ。
    let (w,h)=(comp.width as f32,comp.height as f32);
    let reply=request(&mut rt,json!({"op":"stageWindow","width":960,"height":540,"roi":[-w*1.5,-h*1.5,w*4.0,h*4.0]}));
    assert_eq!(reply,json!({"needsRender":true}));
    let window=rt.window(View::User).unwrap();
    let stage=make(&rt,960,540);
    rt.render_into(&stage,View::User,window).unwrap();
    let b=rt.viewer.selection_bounds[&View::User].get(&text).copied().expect("the text is drawn in the Stage window");
    assert!(b[0]<0.0&&b[2]<0.0&&b[2]>b[0],"the cage sits left of the frame in comp px: {b:?}");
    assert!(b[1]>0.0&&b[3]<h,"the cage keeps the comp's vertical placement: {b:?}");
    let output=make(&rt,comp.width,comp.height);
    rt.render_into(&output,View::Camera,crate::render::engine::Window::output(comp)).unwrap();
    assert!(rt.viewer.selection_bounds[&View::Camera].get(&text).is_none_or(|b|b[2]<=b[0]),"the output never shows what lies outside the frame");
    let status=rt.build_status().unwrap();
    let row=status["layers"].as_array().unwrap().iter().find(|l|l["id"]==text.0).unwrap();
    assert!(row["stageBounds"]["corners"].is_array(),"the Stage cage rides on the row");
    assert!(status["observer"].is_object()&&status["cameraGizmos"].is_array(),"the Stage chrome is always there");
    // 正面の観測者は既定のカメラと同じ所に立つ。eye は写せなくても、箱(comp 面の 4 角)は必ず出る。
    request(&mut rt,json!({"op":"create","kind":"camera"}));
    let camera=rt.viewer.selected().unwrap();
    let gizmos=rt.camera_gizmos().unwrap();
    let g=gizmos.as_array().unwrap().first().expect("the camera box is drawn front-on");
    assert_eq!(g["points"].as_array().unwrap().len(),4,"{g}");
    assert!(g["eye"].is_null(),"the eye sits at the observer and cannot be projected: {g}");
    let corner=|i:usize|[g["points"][i][0].as_f64().unwrap(),g["points"][i][1].as_f64().unwrap()];
    // 観測者を回せば eye が写り、Blender の四角錐(枠 4 角 + 上の三角)が出る。正面では箱だけ。
    assert!(g["frustum"].as_array().unwrap().iter().all(|p|p.is_null())||g["eye"].is_null(),"front-on the pyramid is not drawable: {g}");
    rt.viewer.user_camera.orbit_degrees=[-20.0,35.0];
    let turned=rt.camera_gizmos().unwrap();
    let t=turned.as_array().unwrap().first().expect("the camera is still drawn from an orbited observer");
    assert!(!t["eye"].is_null()&&t["frustum"].as_array().unwrap().iter().all(|p|!p.is_null())&&!t["up"].is_null(),"orbited: eye, frame and up all project: {t}");
    rt.viewer.user_camera=Default::default();
    let (xs,ys):(Vec<f64>,Vec<f64>)=(0..4).map(corner).map(|c|(c[0],c[1])).unzip();
    let span=|v:&[f64]|(v.iter().cloned().fold(f64::MAX,f64::min),v.iter().cloned().fold(f64::MIN,f64::max));
    assert!(span(&xs).0.abs()<1.0&&(span(&xs).1-w as f64).abs()<1.0&&span(&ys).0.abs()<1.0&&(span(&ys).1-h as f64).abs()<1.0,"a default camera's box is the frame: {g}");
    // Boxcam: 作中カメラが回っても Stage(Original Comp)は動かない。動くのは箱だけ。
    request(&mut rt,json!({"op":"setProperty","layer":text.0,"property":"position","value":[400.0,300.0]}));
    let still=|rt:&mut EditorRuntime|{
        request(rt,json!({"op":"select","ids":[text.0]}));
        let window=crate::render::engine::Window{projection_camera:Some(Default::default()),..crate::render::engine::Window::output(comp)};
        rt.viewer.stage_window=Some(window);
        let stage=make(rt,comp.width,comp.height);
        rt.render_into(&stage,View::User,window).unwrap();
        rt.viewer.selection_bounds[&View::User][&text]
    };
    let before=still(&mut rt);
    request(&mut rt,json!({"op":"setProperty","layer":camera.0,"property":"camera.orbit","value":[-20.0,40.0]}));
    let after=still(&mut rt);
    assert!(before.iter().zip(after.iter()).all(|(a,b)|(a-b).abs()<1.0),"the Stage never moves with the camera: {before:?} vs {after:?}");
    let output=make(&rt,comp.width,comp.height);
    rt.render_into(&output,View::Camera,crate::render::engine::Window::output(comp)).unwrap();
    let seen=rt.viewer.selection_bounds[&View::Camera][&text];
    assert!(seen.iter().zip(after.iter()).any(|(a,b)|(a-b).abs()>1.0),"the box's contents do follow the camera: {seen:?}");
    // 2D は出力の画面の物: Stage でも箱に貼り付き、箱(カメラ)と一緒に動く。
    request(&mut rt,json!({"op":"setProperty","layer":camera.0,"property":"camera.orbit","value":[0.0,0.0]}));
    request(&mut rt,json!({"op":"setAttrs","layers":[text.0],"patch":{"projection":"2D"}}));
    let flat_before=still(&mut rt);
    request(&mut rt,json!({"op":"setProperty","layer":camera.0,"property":"camera.center","value":[300.0,0.0]}));
    let flat_after=still(&mut rt);
    assert!((flat_after[0]-flat_before[0]).abs()>100.0,"a 2D layer rides with the camera box in the Stage: {flat_before:?} vs {flat_after:?}");
    let cage=rt.bounds_seen(text,View::User).unwrap();
    let c:Vec<[f64;2]>=serde_json::from_value(cage["corners"].clone()).unwrap();
    assert!((c[0][0]-flat_after[0] as f64).abs()<3.0,"the Stage cage follows the moved 2D layer: {c:?} vs {flat_after:?}");
}
/// 選ばれた層の籠は、描いた画素そのものの範囲(GPU の mask)。3D を透視で回しても 1 px で合う。
#[test]
fn the_selected_cage_is_the_drawn_pixels(){
    let mut rt=EditorRuntime::open("").unwrap();
    request(&mut rt,json!({"op":"preferences","flatProjection":"3D"}));
    request(&mut rt,json!({"op":"create","kind":"cylinder"}));
    let mesh=rt.viewer.selected().unwrap();
    request(&mut rt,json!({"op":"setProperty","layer":mesh.0,"property":"rotation.y","value":-35.0}));
    request(&mut rt,json!({"op":"create","kind":"camera"}));
    let camera=rt.viewer.selected().unwrap();
    request(&mut rt,json!({"op":"setProperty","layer":camera.0,"property":"camera.orbit","value":[-20.0,40.0]}));
    request(&mut rt,json!({"op":"select","ids":[mesh.0]}));
    let comp=rt.doc.view().composition().unwrap().unwrap().spec();
    let texture=rt.engine.gpu_device().create_texture(&wgpu::TextureDescriptor {
        label:Some("selected cage test"),size:wgpu::Extent3d{width:comp.width,height:comp.height,depth_or_array_layers:1},mip_level_count:1,sample_count:1,
        dimension:wgpu::TextureDimension::D2,format:crate::render::compositor::PRESENTABLE_FORMAT,usage:wgpu::TextureUsages::RENDER_ATTACHMENT|wgpu::TextureUsages::TEXTURE_BINDING,view_formats:&[],
    });
    let time=rt.time().unwrap();
    let observer=rt.view_camera(View::Camera).unwrap();
    rt.engine.render_frame_into_with_camera(&rt.doc.view(),time,&texture,observer,true,&[mesh]).unwrap();
    rt.engine.gpu_device().poll(wgpu::PollType::wait_indefinitely()).unwrap();
    rt.take_selection_bounds(View::Camera,crate::render::engine::Window::output(comp));
    assert!(rt.viewer.selection_bounds[&View::Camera].contains_key(&mesh),"the mask reached the bridge");
    let pixels=rt.engine.render_frame(&rt.doc.view(),time).unwrap();
    let background=&pixels[0..4];
    let (mut px0,mut py0,mut px1,mut py1)=(f64::MAX,f64::MAX,f64::MIN,f64::MIN);
    for (i,px) in pixels.chunks(4).enumerate(){
        if px[..3]!=background[..3]{let x=(i as u32%comp.width) as f64;let y=(i as u32/comp.width) as f64;px0=px0.min(x);py0=py0.min(y);px1=px1.max(x+1.0);py1=py1.max(y+1.0);}
    }
    let bounds=rt.bounds(mesh).unwrap();
    let c=bounds["corners"].as_array().unwrap();
    assert_eq!(c.len(),4,"a selected 3D layer gets the screen rectangle, not the box: {c:?}");
    let (cx0,cy0,cx1,cy1)=(c[0][0].as_f64().unwrap(),c[0][1].as_f64().unwrap(),c[2][0].as_f64().unwrap(),c[2][1].as_f64().unwrap());
    assert!((cx0-px0).abs()<=1.0&&(cy0-py0).abs()<=1.0&&(cx1-px1).abs()<=1.0&&(cy1-py1).abs()<=1.0,
        "the cage [{cx0},{cy0}]-[{cx1},{cy1}] is not the drawn [{px0},{py0}]-[{px1},{py1}]");
}

/// Camera View で、3D 層の描画は Stage の籠(8 角)の中に収まる。観測者と描画が別の行列なら、ここで露見する。
#[test]
fn the_cage_contains_the_drawn_mesh_under_the_scene_camera(){
    let mut rt=EditorRuntime::open("").unwrap();
    request(&mut rt,json!({"op":"preferences","flatProjection":"3D"}));
    request(&mut rt,json!({"op":"create","kind":"cylinder"}));
    let mesh=rt.viewer.selected().unwrap();
    request(&mut rt,json!({"op":"setProperty","layer":mesh.0,"property":"rotation.y","value":-20.0}));
    request(&mut rt,json!({"op":"setProperty","layer":mesh.0,"property":"rotation","value":30.0}));
    request(&mut rt,json!({"op":"create","kind":"camera"}));
    let camera=rt.viewer.selected().unwrap();
    request(&mut rt,json!({"op":"select","ids":[mesh.0]}));
    let comp=rt.doc.view().composition().unwrap().unwrap().spec();
    for (projection,orbit,distance) in [("3D",[0.0,0.0],1.0),("3D",[-20.0,146.0],1.0),("3D",[-20.0,-166.0],4.58),("3D",[-20.0,40.0],0.5),("3D",[35.0,-120.0],0.7),
                                        ("2D",[0.0,0.0],1.0),("2D",[-20.0,40.0],0.7),("2.5D",[0.0,0.0],1.0),("2.5D",[-20.0,40.0],0.7),("2.5D",[35.0,-120.0],1.3)] {
        request(&mut rt,json!({"op":"setAttrs","layers":[mesh.0],"patch":{"projection":projection}}));
        request(&mut rt,json!({"op":"setProperty","layer":mesh.0,"property":"rotation","value":80.0}));
        request(&mut rt,json!({"op":"setProperty","layer":camera.0,"property":"camera.orbit","value":orbit}));
        request(&mut rt,json!({"op":"setProperty","layer":camera.0,"property":"camera.distance","value":distance}));
        let orbit=format!("{projection} {orbit:?}");
        let time=rt.time().unwrap();
        let pixels=rt.engine.render_frame(&rt.doc.view(),time).unwrap();
        assert!(rt.engine.layer_failures().is_empty(),"{:?}",rt.engine.layer_failures());
        let background=&pixels[0..4];
        let (mut px0,mut py0,mut px1,mut py1)=(f64::MAX,f64::MAX,f64::MIN,f64::MIN);
        for (i,px) in pixels.chunks(4).enumerate(){
            if px[..3]!=background[..3]{let x=(i as u32%comp.width) as f64;let y=(i as u32/comp.width) as f64;px0=px0.min(x);py0=py0.min(y);px1=px1.max(x);py1=py1.max(y);}
        }
        assert!(px0<px1,"{orbit:?}/{distance}: the mesh is drawn");
        let bounds=rt.bounds(mesh).unwrap();
        let corners=bounds["corners"].as_array().unwrap();
        let (mut cx0,mut cy0,mut cx1,mut cy1)=(f64::MAX,f64::MAX,f64::MIN,f64::MIN);
        for c in corners{let x=c[0].as_f64().unwrap();let y=c[1].as_f64().unwrap();cx0=cx0.min(x);cy0=cy0.min(y);cx1=cx1.max(x);cy1=cy1.max(y);}
        let slack=3.0;
        assert!(cx0-slack<=px0&&cy0-slack<=py0&&px1<=cx1+slack&&py1<=cy1+slack,
            "{orbit:?}/{distance}: drawn [{px0},{py0}]-[{px1},{py1}] escapes the cage [{cx0:.1},{cy0:.1}]-[{cx1:.1},{cy1:.1}]");
        // 2D・2.5D の籠は輪郭にぴったり: 4 辺とも描いた画素の縁から数画素以内。
        if projection!="3D" {
            assert!((cx0-px0).abs()<=slack&&(cy0-py0).abs()<=slack&&(cx1-px1).abs()<=slack&&(cy1-py1).abs()<=slack,
                "{orbit:?}/{distance}: the frame [{cx0:.1},{cy0:.1}]-[{cx1:.1},{cy1:.1}] is loose around the drawn [{px0},{py0}]-[{px1},{py1}]");
        }
        if projection!="3D" { continue; }
        // 3 軸ギズモは層の anchor(観測者で写した点)に立ち、描いた層の上に乗る。
        let view=rt.doc.view();
        let world=motolii_render::picture::resolve::transform::world_transform3d(&view, mesh,time).unwrap();
        let anchor=match view.value_at(mesh,&PropertyId::new(property::ANCHOR).unwrap(),time).unwrap(){Some(Value::Vec2(v))=>v,_=>[0.0,0.0]};
        let projection=crate::doc::core::camera_projection(comp,rt.view_camera(View::Camera).unwrap());
        let c=projection.projection_matrix()*projection.view_matrix()*world.transform_point3(glam::vec3(anchor[0]as f32,anchor[1]as f32,0.0)).extend(1.0);
        let hub=[((c.x/c.w+1.0)*0.5*comp.width as f32)as f64,((1.0-c.y/c.w)*0.5*comp.height as f32)as f64];
        let gizmo=rt.spatial_gizmo(View::Camera).unwrap();
        let vertices=gizmo["vertices"].as_array().unwrap_or_else(||panic!("{orbit:?}/{distance}: no 3-axis gizmo: {gizmo}"));
        let (mut gx0,mut gy0,mut gx1,mut gy1)=(f64::MAX,f64::MAX,f64::MIN,f64::MIN);
        for v in vertices{let x=v[0].as_f64().unwrap();let y=v[1].as_f64().unwrap();if x.is_finite()&&y.is_finite(){gx0=gx0.min(x);gy0=gy0.min(y);gx1=gx1.max(x);gy1=gy1.max(y);}}
        assert!(gx0<=hub[0]&&hub[0]<=gx1&&gy0<=hub[1]&&hub[1]<=gy1,"{orbit:?}/{distance}: gizmo [{gx0:.1},{gy0:.1}]-[{gx1:.1},{gy1:.1}] does not stand on the anchor {hub:?}");
        assert!(px0-slack<=hub[0]&&hub[0]<=px1+slack&&py0-slack<=hub[1]&&hub[1]<=py1+slack,"{orbit:?}/{distance}: anchor {hub:?} is off the drawn mesh [{px0},{py0}]-[{px1},{py1}]");
    }
}
/// 2D・2.5D の籠は向きを持たない。層が傾いていても台を回していても、写した 8 角を包む正立の矩形が出て、
/// その角を引けば scale が、上の取っ手を画面で 90° 回せば rotation が 90° 動く。
#[test]
fn a_tilted_planar_layer_keeps_a_facing_frame_and_its_handles_write_scale_and_rotation(){
    let mut rt=EditorRuntime::open("").unwrap();
    request(&mut rt,json!({"op":"create","kind":"rectangle"}));
    let layer=rt.viewer.selected().unwrap();
    request(&mut rt,json!({"op":"setProperty","layer":layer.0,"property":"rotation.y","value":60.0}));
    request(&mut rt,json!({"op":"setProperty","layer":layer.0,"property":"rotation","value":20.0}));
    let comp=rt.doc.view().composition().unwrap().unwrap().spec();
    // 形の bounds は 1 度描いてから測れる。
    let time=rt.time().unwrap();
    rt.engine.render_frame(&rt.doc.view(),time).unwrap();
    for (projection,observer) in [("2D",crate::doc::core::ResolvedCamera::default()),("2.5D",crate::doc::core::ResolvedCamera{orbit_degrees:[-15.0,35.0],distance_scale:1.4,..Default::default()})] {
        request(&mut rt,json!({"op":"setAttrs","layers":[layer.0],"patch":{"projection":projection}}));
        rt.viewer.user_camera=observer;
        let time=rt.time().unwrap();
        rt.engine.frame_graph_editor_scene(&rt.doc.view(),time).unwrap();
        let bounds=rt.bounds_seen(layer,View::User).unwrap();
        let c:Vec<[f64;2]>=serde_json::from_value(bounds["corners"].clone()).unwrap();
        assert_eq!(c.len(),4,"{projection}: a planar layer gets the 4-corner frame");
        assert!((c[0][1]-c[1][1]).abs()<1e-3&&(c[1][0]-c[2][0]).abs()<1e-3&&(c[2][1]-c[3][1]).abs()<1e-3&&(c[3][0]-c[0][0]).abs()<1e-3,"{projection}: the frame faces the observer: {c:?}");
        assert!(c[0][0]<c[1][0]&&c[0][1]<c[3][1],"{projection}: nw, ne, se, sw: {c:?}");
        let view=rt.doc.view();
        let resolved=crate::render::picture::resolve::resolved_layers(&view, time).unwrap();
        let r=resolved.iter().find(|r|r.id==layer).unwrap();
        let b=rt.engine.selected_layer_bounds_in(&view,&resolved,layer,time).unwrap();
        let projected=crate::doc::core::projected_screen_corners(comp,rt.engine.resolve_camera(&view,time).unwrap(),observer,r.projection,crate::doc::core::depth_scaled(r.placement.world_transform.unwrap()),b.min,b.max);
        for p in projected{assert!(c[0][0]-1e-3<=p.x as f64&&p.x as f64<=c[2][0]+1e-3&&c[0][1]-1e-3<=p.y as f64&&p.y as f64<=c[2][1]+1e-3,"{projection}: corner {p:?} outside the frame {c:?}");}
        assert!((c[1][0]-c[0][0])>1.0&&(c[3][1]-c[0][1])>1.0,"{projection}: the frame is not edge-on even though the plane may be");
        // 角を外へ引く → scale が伸びる。
        let se=[c[2][0],c[2][1]];let nw=[c[0][0],c[0][1]];
        let drag=editor::stage::DragSession::begin(&rt.doc,&mut rt.engine,&[layer],"scale","se",se,time,observer,Default::default(),1.0,None).unwrap();
        let far=[nw[0]+(se[0]-nw[0])*2.0,nw[1]+(se[1]-nw[1])*2.0];
        let edits=drag.edits(&rt.doc,far,false,false,Animate::Off).unwrap();
        let scale=edits.iter().find_map(|i|match i{Intent::SetConstant{property,value:Value::Vec2(v),..} if *property==PropertyId::new(property::SCALE).unwrap()=>Some(*v),_=>None}).expect("scale edit");
        assert!(scale[0]>1.5&&scale[1]>1.5,"{projection}: pulling the corner out grows the layer: {scale:?}");
        // 上の取っ手を画面で 90° 回す → rotation が 90° 動く(相似で結ぶので画面の角度がそのまま値)。
        let top=[(c[0][0]+c[1][0])*0.5,c[0][1]-22.0];
        let drag=editor::stage::DragSession::begin(&rt.doc,&mut rt.engine,&[layer],"rotate","",top,time,observer,Default::default(),1.0,None).unwrap();
        // 回転の軸は anchor を写した点(取っ手と同じ写像: 中間奥行きの面の 4 角を anchor の比で結ぶ)。
        let m:Vec<[f64;2]>=(0..4).map(|i|[((projected[i].x+projected[i+4].x)*0.5)as f64,((projected[i].y+projected[i+4].y)*0.5)as f64]).collect();
        let f:[f64;2]=serde_json::from_value(bounds["anchorFraction"].clone()).unwrap();
        let centre=[m[0][0]+(m[1][0]-m[0][0])*f[0]+(m[2][0]-m[0][0])*f[1],m[0][1]+(m[1][1]-m[0][1])*f[0]+(m[2][1]-m[0][1])*f[1]];
        let arm=[top[0]-centre[0],top[1]-centre[1]];
        let turned=[centre[0]-arm[1],centre[1]+arm[0]];
        let edits=drag.edits(&rt.doc,turned,false,false,Animate::Off).unwrap();
        let rotation=edits.iter().find_map(|i|match i{Intent::SetConstant{property,value:Value::F64(v),..} if *property==PropertyId::new(property::ROTATION).unwrap()=>Some(*v),_=>None}).expect("rotation edit");
        assert!((rotation-110.0).abs()<3.0,"{projection}: a quarter turn on screen is a quarter turn of the layer: {rotation}");
    }
}

/// 実験(2026-09-19): 縦長の Stage 窓で球(3D の網)が潰れるか。窓の縦横比と roi の縦横比が同じなら、
/// 出力(Camera)と同じ丸さで出るはず。潰れたら投影の縦横比が窓に引きずられている。
#[test]
#[ignore = "GPU で 10 分。2026-09-19 の実験: 窓と roi の縦横比が揃っていれば 3D も 2.5D も潰れない(結果は日報)。cargo test -- --ignored で回す"]
fn a_sphere_keeps_its_roundness_in_a_tall_stage_window(){
    let mut rt=EditorRuntime::open("").unwrap();
    request(&mut rt,json!({"op":"create","kind":"sphere"}));
    let sphere=rt.viewer.selected().unwrap();
    request(&mut rt,json!({"op":"select","ids":[]}));
    let comp=rt.doc.view().composition().unwrap().unwrap().spec();
    let (w,h)=(comp.width as f32,comp.height as f32);
    let make=|rt:&EditorRuntime,width,height|rt.engine.gpu_device().create_texture(&wgpu::TextureDescriptor {
        label:Some("aspect test"),size:wgpu::Extent3d{width,height,depth_or_array_layers:1},mip_level_count:1,sample_count:1,
        dimension:wgpu::TextureDimension::D2,format:crate::render::compositor::PRESENTABLE_FORMAT,usage:wgpu::TextureUsages::RENDER_ATTACHMENT|wgpu::TextureUsages::TEXTURE_BINDING|wgpu::TextureUsages::COPY_SRC,view_formats:&[],
    });
    let bbox=|rt:&EditorRuntime,texture:&wgpu::Texture,width:u32,height:u32|->(f32,f32){
        let device=rt.engine.gpu_device();
        let bytes_per_row=((width*4+255)/256)*256;
        let buffer=device.create_buffer(&wgpu::BufferDescriptor{label:None,size:(bytes_per_row*height) as u64,usage:wgpu::BufferUsages::COPY_DST|wgpu::BufferUsages::MAP_READ,mapped_at_creation:false});
        let mut encoder=device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(texture.as_image_copy(),wgpu::TexelCopyBufferInfo{buffer:&buffer,layout:wgpu::TexelCopyBufferLayout{offset:0,bytes_per_row:Some(bytes_per_row),rows_per_image:Some(height)}},wgpu::Extent3d{width,height,depth_or_array_layers:1});
        rt.engine.gpu_queue().submit([encoder.finish()]);
        let slice=buffer.slice(..);slice.map_async(wgpu::MapMode::Read,|_|{});device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        let data=slice.get_mapped_range();
        let (mut x0,mut y0,mut x1,mut y1)=(u32::MAX,u32::MAX,0u32,0u32);
        for y in 0..height{for x in 0..width{let p=&data[(y*bytes_per_row+x*4) as usize..][..3];if p.iter().any(|&c|c>24){x0=x0.min(x);y0=y0.min(y);x1=x1.max(x);y1=y1.max(y);}}}
        assert!(x1>x0&&y1>y0,"nothing drawn");
        ((x1-x0+1) as f32,(y1-y0+1) as f32)
    };
    // 出力: 960×540(comp と同じ縦横比)。
    let out=make(&rt,960,540);
    rt.render_into(&out,View::Camera,crate::render::engine::Window{width:960,height:540,..crate::render::engine::Window::output(comp)}).unwrap();
    let (ow,oh)=bbox(&rt,&out,960,540);
    // Stage: 400×900 の縦長の窓。roi は窓と同じ縦横比で comp の幅を収める(= 一様な倍率 400/w)。
    let roi_h=w*900.0/400.0;
    let window=crate::render::engine::Window{width:400,height:900,roi:[0.0,(h-roi_h)*0.5,w,roi_h],projection_camera:Some(Default::default())};
    rt.viewer.stage_window=Some(window);
    let stage=make(&rt,400,900);
    rt.render_into(&stage,View::User,window).unwrap();
    let (sw,sh)=bbox(&rt,&stage,400,900);
    let (output_ratio,stage_ratio)=(ow/oh,sw/sh);
    assert!((output_ratio-stage_ratio).abs()<0.1,"sphere {sphere:?}: output {ow}x{oh} (ratio {output_ratio:.2}) vs tall stage {sw}x{sh} (ratio {stage_ratio:.2})");
    // 2.5D の球、右寄り、13 % の縮尺、縦長と横長の窓: 比を出して見る(観察、落とさない)。
    request(&mut rt,json!({"op":"setAttrs","layers":[sphere.0],"patch":{"projection":"2.5D"}}));
    request(&mut rt,json!({"op":"setProperty","layer":sphere.0,"property":"position","value":[w*0.75,h*0.5]}));
    request(&mut rt,json!({"op":"select","ids":[]}));
    let out=make(&rt,960,540);
    rt.render_into(&out,View::Camera,crate::render::engine::Window{width:960,height:540,..crate::render::engine::Window::output(comp)}).unwrap();
    let (ow,oh)=bbox(&rt,&out,960,540);
    for (ww,wh) in [(400u32,900u32),(1200,400)] {
        let scale=0.13f32;
        let (rw,rh)=(ww as f32/scale,wh as f32/scale);
        let window=crate::render::engine::Window{width:ww,height:wh,roi:[(w-rw)*0.5,(h-rh)*0.5,rw,rh],projection_camera:Some(Default::default())};
        rt.viewer.stage_window=Some(window);
        let stage=make(&rt,ww,wh);
        rt.render_into(&stage,View::User,window).unwrap();
        let (sw,sh)=bbox(&rt,&stage,ww,wh);
        eprintln!("ASPECT 2.5D sphere: output {ow}x{oh} ratio {:.2}; stage {ww}x{wh} at 13%: {sw}x{sh} ratio {:.2}",ow/oh,sw/sh);
    }
}
