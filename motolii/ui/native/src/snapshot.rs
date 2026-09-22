#[allow(unused_imports)]
use crate::edit::{Animate, Document, Intent};
use crate::{EditorRuntime, editor};
use crate::doc::store::*;
use serde_json::{json,Value as Json};
fn e(error: impl std::fmt::Display)->String {error.to_string()}

pub(crate) fn authored_signature(doc:&Document)->Result<String,String>{
    let v=doc.view().without_transients();
    let mut ids=v.layers();ids.sort();
    let mut layers=Vec::new();
    for id in ids {
        let mut properties=v.properties(id);properties.sort();
        let properties:Result<Vec<_>,_>=properties.iter().map(|p|v.property_source(id,p).map(|s|(p.name().to_owned(),s))).collect();
        layers.push(json!([id,v.attrs(id).map_err(e)?,v.meta(id).map_err(e)?,properties.map_err(e)?,v.text_document(id).map_err(e)?,v.shapes(id).map_err(e)?,v.effects(id).map_err(e)?,v.masks(id).map_err(e)?]));
    }
    Ok(json!([v.composition().map_err(e)?,layers,v.assets().map_err(e)?,v.markers().map_err(e)?,v.notebook().map_err(e)?,v.slots().map_err(e)?]).to_string())
}
pub(crate) fn value(value:&Value)->Json{match value{
    Value::F64(v)=>json!(v),Value::Vec2(v)=>json!(v),Value::Color(v)=>json!(v),Value::Bool(v)=>json!(v),Value::Enum(v)=>json!(v),Value::LayerId(v)=>json!(v),Value::Path(v)=>json!(v),
}}
pub(crate) fn interp(i:Interp)->Json{
    let raw=serde_json::to_value(i).unwrap_or(Json::Null);
    let mut out=if let Some(s)=raw.as_str(){json!({"kind":s})}
    else if let Some((kind,params))=raw.as_object().and_then(|o|o.iter().next()){
        let mut out=params.as_object().cloned().unwrap_or_default();out.insert("kind".into(),json!(kind));Json::Object(out)
    }else{json!({"kind":"Unknown"})};
    let steps=if matches!(i,Interp::Linear|Interp::Hold){1}else{64};
    out["handles"]=json!(editor::ease_kinds::handles(i).iter().map(|h|[h.at.0,h.at.1]).collect::<Vec<_>>());
    out["overshoots"]=json!(editor::ease_kinds::overshoots(i));
    out["samples"]=json!((0..=steps).map(|step|{let u=step as f64/steps as f64;[u,i.ease(u)]}).collect::<Vec<_>>());
    out
}
fn prop(view:&StoreView<'_>,layer:LayerId,id:&str,label:&str,fallback:&Value,range:Option<(f64,f64)>,at:RationalTime,fps:Fps,live:bool)->Result<Json,String>{
    let p=PropertyId::new(id).map_err(e)?;
    let current=view.value_at(layer,&p,at).map_err(e)?.unwrap_or_else(||fallback.clone());
    if live {
        let here=at.try_to_frame_round(fps).map_err(e)?;
        let keyed=view.track(layer,&p).map_err(e)?.map(|track|track.keys().iter().any(|key|key.t.try_to_frame_round(fps).ok()==Some(here))).unwrap_or(false);
        return Ok(json!({"id":id,"value":value(&current),"keyedNow":keyed}));
    }
    let mut keys=Vec::new();
    if let Some(track)=view.track(layer,&p).map_err(e)?{for key in track.keys(){keys.push(json!({"frame":key.t.try_to_frame_round(fps).map_err(e)?,"value":value(&key.value),"interp":interp(key.interp)}));}}
    let here=at.try_to_frame_round(fps).map_err(e)?;
    Ok(json!({"id":id,"label":label,"kind":match current{Value::F64(_)=>"number",Value::Vec2(_)=>"vec2",Value::Color(_)=>"color",Value::Bool(_)|Value::Enum(_)|Value::LayerId(_)=>"enum",Value::Path(_)=>"text"},"value":value(&current),"min":range.map(|r|r.0),"max":range.map(|r|r.1),"keyedNow":keys.iter().any(|k|k["frame"]==here),"keys":keys}))
}
fn source_kind(source:&LayerSource)->&'static str{match source{
    LayerSource::Camera=>"Camera",LayerSource::Stage=>"Stage",LayerSource::Text=>"Text",LayerSource::Shape=>"Shape",LayerSource::Group=>"Group",LayerSource::Null=>"Null",LayerSource::Particles=>"Particles",
    LayerSource::File{path,..}=>{
        if crate::render::media::is_mesh_path(path){"Mesh"}else if crate::render::media::is_point_cloud_path(path){"PointCloud"}else{
            let mime=std::path::Path::new(path).extension().and_then(|e|e.to_str()).and_then(crate::render::media::asset_type_for_extension).unwrap_or_default();
            if mime.starts_with("audio/"){"Audio"}else if mime.starts_with("video/"){"Video"}else{"Image"}
        }
    }
}}
#[derive(Clone,Copy)]
struct Eye{time:RationalTime,comp:crate::doc::core::CompSpec,camera:crate::doc::core::ResolvedCamera,observer:crate::doc::core::ResolvedCamera,document:crate::doc::core::ResolvedCamera}
use crate::viewer::View;

impl EditorRuntime{
    pub(crate) fn view_camera(&mut self,view:View)->Result<crate::doc::core::ResolvedCamera,String>{
        if view==View::User{return Ok(self.viewer.user_camera)}
        let time=self.time()?;let doc=self.doc.view();
        self.engine.frame_graph_document_camera(&doc,time).map_err(e)
    }
    /// view の描く窓。Camera は出力寸法そのもの、Stage はタブが置いた窓(未設定なら出力寸法)。
    pub(crate) fn window(&self,view:View)->Result<crate::render::engine::Window,String>{
        let comp=self.doc.view().composition().map_err(e)?.ok_or("No composition")?.spec();
        Ok(match view{View::Camera=>crate::render::engine::Window::output(comp),View::User=>self.viewer.stage_window.unwrap_or(crate::render::engine::Window{projection_camera:Some(Default::default()),..crate::render::engine::Window::output(comp)})})
    }
    /// 層を置くカメラ。2D は出力の画面の物なのでどの view でも作中カメラの箱に貼り付く。
    /// 2.5D・3D は世界に居る: Stage は既定(Boxcam の Original Comp)、Camera は作中カメラ。
    pub(crate) fn projection_camera(&mut self,view:View,projection:LayerProjection)->Result<crate::doc::core::ResolvedCamera,String>{
        match (view,projection){
            (View::User,LayerProjection::TwoD)|(View::Camera,_)=>{
                let time=self.time()?;let doc=self.doc.view();
                self.engine.frame_graph_document_camera(&doc,time).map_err(e)
            }
            (View::User,_)=>Ok(Default::default())
        }
    }
    /// Flutter の Stage タブが窓を置く: 画素寸法と、comp 画像のどこを写すか。幅 0 は「隠れた」。
    pub(crate) fn set_stage_window(&mut self,j:&Json)->Result<bool,String>{
        let size=|k:&str|j[k].as_u64().map(|v|v as u32).ok_or_else(||format!("stageWindow needs {k}"));
        let next=match (size("width")?,size("height")?){
            (0,_)|(_,0)=>None,
            (width,height)=>{
                let roi:[f32;4]=serde_json::from_value(j["roi"].clone()).map_err(|_|"stageWindow needs roi [x, y, w, h]")?;
                if roi.iter().any(|v|!v.is_finite())||roi[2]<=0.0||roi[3]<=0.0{return Err("Invalid stageWindow roi".into())}
                Some(crate::render::engine::Window{width,height,roi,projection_camera:Some(Default::default())})
            }
        };
        let changed=self.viewer.stage_window!=next;
        self.viewer.stage_window=next;
        Ok(changed)
    }
    fn depth_layout(&self,scene:&crate::render::frame_graph::SceneValue)->Result<Json,String>{
        let view=self.doc.view();let time=self.time()?;let comp=view.composition().map_err(e)?.ok_or("No composition")?.spec();
        // 原点は注視点。カメラは eye の位置に置き、drag で orbit と距離を author する。
        let seen=self.engine.resolve_camera(&view,time).map_err(e)?;
        let camera=crate::doc::core::camera_projection(comp,seen);
        let target=seen.target(comp);
        let camera_layer=motolii_render::picture::resolve::camera::active_camera_layer(&view, time).map_err(e)?;
        let target_layer=camera_layer.map(|id|motolii_render::picture::resolve::camera::camera_target_layer(&view, id,time)).transpose().map_err(e)?.flatten();
        let worlds:std::collections::HashMap<LayerId,glam::Affine3A>=view.layers().into_iter()
            .filter_map(|id|scene.layer(id).map(|layer|(id,layer.transform.spatial)))
            .collect();
        let mut items=Vec::new();
        for id in view.layers() {
            let Some(layer)=scene.layer(id) else{continue};
            if matches!(layer.source,LayerSource::Camera|LayerSource::Stage){continue}
            let Some(world)=worlds.get(&id) else{continue};
            let attrs=view.attrs(id).map_err(e)?.unwrap_or_default();
            let local=self.engine.selected_scene_layer_bounds_in(&view,&scene.layers,id,time).map(|b|glam::Vec3::from(b.center())).unwrap_or(glam::Vec3::ZERO);
            let center=world.transform_point3(local)-target;
            let parent=attrs.parent.and_then(|parent|worlds.get(&parent).copied()).unwrap_or(glam::Affine3A::IDENTITY);
            let inverse=parent.inverse();
            let get=|name|view.value_at(id,&PropertyId::new(name).unwrap(),time).ok().flatten();
            let position=match get(property::POSITION){Some(Value::Vec2(v))=>v,_=>[0.0,0.0]};
            let z=match get(property::POSITION_Z){Some(Value::F64(v))=>v,_=>0.0};
            items.push(json!({"id":id.0,"name":attrs.name,"point":[center.x,center.z],"local":[position[0],position[1],z],"inverseX":inverse.transform_vector3(glam::Vec3::X).to_array(),"inverseZ":inverse.transform_vector3(glam::Vec3::Z).to_array(),"locked":attrs.locked || !inverse.is_finite(),"color":attrs.label_color}));
        }
        let eye=camera.eye-target;
        Ok(json!({"items":items,"halfFov":(camera.vertical_fov_radians*0.5).tan()*camera.aspect_ratio,
            "camera":{"point":[eye.x,eye.z],"layer":camera_layer.map(|l|l.0),"target":target_layer.map(|l|l.0),"orbit":seen.orbit_degrees,"distance":seen.distance_scale,"baseDistance":crate::doc::core::distance_from_camera(comp,0.0)}}))
    }
    /// 注視の球: 層の world 中心と、局所 bounds の 8 角を包む半径(rerun `focus_entity` の bounding sphere)。
    pub(crate) fn focus_sphere(&mut self,id:LayerId)->Result<Option<(glam::Vec3,f32)>,String>{
        let time=self.time()?;let view=self.doc.view();
        let scene=self.engine.frame_graph_editor_scene(&view,time).map_err(e)?;
        let Some(layer)=scene.layer(id) else{return Ok(None)};
        let world=layer.transform.spatial;
        let Some(b)=self.engine.selected_scene_layer_bounds_in(&view,&scene.layers,id,time) else{return Ok(None)};
        let centre=world.transform_point3(glam::Vec3::from(b.center()));
        let radius=(0..8).map(|i|world.transform_point3(glam::vec3(if i&1==0{b.min[0]}else{b.max[0]},if i&2==0{b.min[1]}else{b.max[1]},if i&4==0{b.min[2]}else{b.max[2]})).distance(centre)).fold(0.0,f32::max);
        Ok(Some((centre,radius)))
    }
    /// 観測者の画面へ world 点を写す。カメラの後ろは None。
    fn observer_screen(&self)->Result<impl Fn(glam::Vec3)->Option<[f32;2]>,String>{
        let view=self.doc.view();let comp=view.composition().map_err(e)?.ok_or("No composition")?.spec();
        let observer=crate::doc::core::camera_projection(comp,self.viewer.user_camera);
        let matrix=observer.projection_matrix()*observer.view_matrix();
        Ok(move|p:glam::Vec3|{let c=matrix*p.extend(1.0);if c.w<=0.0{return None}Some([(c.x/c.w+1.0)*0.5*comp.width as f32,(1.0-c.y/c.w)*0.5*comp.height as f32])})
    }
    /// Stage の観測者: 正面か、注視点、comp 面の 4 角(Stage の comp 画像 px)。
    fn observer_status(&self)->Result<Json,String>{
        let comp=self.doc.view().composition().map_err(e)?.ok_or("No composition")?.spec();
        let screen=self.observer_screen()?;
        let (w,h)=(comp.width as f32,comp.height as f32);
        let quad=|[x,y,w,h]:[f32;4]|{let q:Vec<_>=[glam::vec3(x,y,0.0),glam::vec3(x+w,y,0.0),glam::vec3(x+w,y+h,0.0),glam::vec3(x,y+h,0.0)].into_iter().map(&screen).collect();if q.iter().all(Option::is_some){json!(q)}else{Json::Null}};
        let extent=motolii_render::picture::resolve::camera::resolve_stage_extent(&self.doc.view(), self.time()?).map_err(e)?;
        Ok(json!({"snapGuides":self.viewer.stage_snap,"front":self.viewer.user_camera.orbit_degrees==[0.0;2],"home":self.viewer.user_camera==Default::default(),"scale":self.viewer.user_camera.distance_scale,"orbit":self.viewer.user_camera.orbit_degrees,"target":screen(self.viewer.user_camera.target(comp)),"frame":quad([0.0,0.0,w,h]),
            "extent":extent.layer.map(|id|json!({"layer":id.0,"margins":extent.margins,"rect":extent.rect(comp),"points":quad(extent.rect(comp))}))}))
    }
    /// 3D 層の 3 軸ギズモ。頂点は comp 座標 —— Stage は掴む所も描く所も同じ写像で扱う。
    /// 3D 層を選んでいない時は Null。2D・2.5D の平面ケージはここを通らない。
    pub(crate) fn spatial_gizmo(&self,seen:View)->Result<Json,String>{
        let view=self.doc.view();let time=self.time()?;
        let Some(comp)=view.composition().map_err(e)? else{return Ok(Json::Null)};
        let Ok(targets)=editor::gizmo3d::spatial_targets(&view,&self.viewer.selected_ids,time) else{return Ok(Json::Null)};
        let pointer=self.viewer.stage_pointer.filter(|_|self.viewer.stage_view==seen);
        let camera=if seen==View::User{self.viewer.user_camera}else{self.engine.resolve_camera(&view,time).map_err(e)?};
        let Some(data)=editor::gizmo3d::draw_data(comp.spec(),camera,&targets,pointer,self.viewer.stage_view_scale,self.viewer.stage_held.as_deref()) else{return Ok(Json::Null)};
        Ok(json!({"vertices":data.vertices,"colors":data.colors,"indices":data.indices}))
    }
    /// Stage に置いた箱(Boxcam): Camera 層ごとの frustum と取っ手。Stage の comp 画像 px。
    fn camera_gizmos(&self)->Result<Json,String>{
        let view=self.doc.view();let time=self.time()?;let comp=view.composition().map_err(e)?.ok_or("No composition")?.spec();
        let screen=self.observer_screen()?;
        let scene=self.engine.frame_graph_cached_scene(&view,time).ok_or("FrameGraph editor scene is not prepared")?;
        let mut gizmos=Vec::new();
        for id in view.layers(){
            let Some(meta)=view.meta(id).map_err(e)? else{continue};
            if meta.source!=LayerSource::Camera || view.attrs(id).map_err(e)?.unwrap_or_default().hidden || !meta.timing.covers(self.viewer.frame){continue}
            let camera=self.engine.camera_of_scene_layer_in(&view,&scene.layers,id,time).map_err(e)?;
            let projection=crate::doc::core::camera_projection(comp,camera);
            let rotation=projection.rotation.inverse();
            let depth=crate::doc::core::distance_from_camera(comp,0.0)*camera.distance_scale;
            let height=depth*(projection.vertical_fov_radians*0.5).tan();let width=height*projection.aspect_ratio;
            let corners=[glam::vec3(-width,-height,-depth),glam::vec3(width,-height,-depth),glam::vec3(width,height,-depth),glam::vec3(-width,height,-depth)];
            let eye=projection.eye;
            let corners:Vec<_>=corners.into_iter().map(|p|{
                let ray=rotation*p;
                let t=if ray.z.abs()>1e-6{-eye.z/ray.z}else{-1.0};
                screen(if t>0.0{eye+ray*t}else{eye+ray})
            }).collect();
            let display=crate::doc::core::distance_from_camera(comp,0.0)*0.15;
            let fh=display*(projection.vertical_fov_radians*0.5).tan();let fw=fh*projection.aspect_ratio;
            let frustum:Vec<_>=[(-fw,-fh),(fw,-fh),(fw,fh),(-fw,fh)].into_iter().map(|(x,y)|screen(eye+rotation*glam::vec3(x,y,-display))).collect();
            let up=screen(eye+rotation*glam::vec3(0.0,-fh*1.7,-display));
            let eye=screen(eye);
            let authorable=camera.orbit_degrees==[0.0;2] && motolii_render::picture::resolve::camera::camera_target_layer(&view, id,time).map_err(e)?.is_none();
            let seen=corners.iter().filter(|c|c.is_some()).count();
            let pyramid=eye.is_some()&&frustum.iter().all(Option::is_some);
            if seen>=2||pyramid{gizmos.push(json!({"id":id.0,"points":corners,"eye":eye,"frustum":frustum,"up":up,"target":screen(camera.target(comp)),"authorable":authorable&&seen==4,"center":camera.center,"zoom":camera.zoom,"roll":camera.roll_degrees}));}
        }
        Ok(json!(gizmos))
    }

    /// 出力(Camera)で見た枠。anchor など view を問わない用途。
    pub(crate) fn bounds(&self,layer:LayerId)->Option<Json>{ self.bounds_seen(layer,View::Camera) }
    pub(crate) fn bounds_seen(&self,layer:LayerId,seen:View)->Option<Json>{
        let view=self.doc.view();let at=self.time().ok()?;
        let scene=self.engine.frame_graph_cached_scene(&view,at)?;
        self.bounds_from(&view,&self.eye(seen)?,scene,layer,seen)
    }
    /// 見ている姿勢 —— comp・作中カメラ・その view の観測者。層ごとに解き直さず、1 フレームに 1 回だけ組む。
    fn eye(&self,seen:View)->Option<Eye>{
        let view=self.doc.view();let time=self.time().ok()?;
        let document=self.engine.resolve_camera(&view,time).ok()?;
        let camera=if seen==View::Camera{document}else{Default::default()};
        let observer=if seen==View::Camera{document}else{self.viewer.user_camera};
        Some(Eye{time,comp:view.composition().ok()??.spec(),camera,observer,document})
    }
    /// 描いた直後に GPU の mask から届いた範囲を、窓の px から comp 画像の px へ戻して取り込む。選択が変わるまで使う。
    pub(crate) fn take_selection_bounds(&mut self,seen:View,window:crate::render::engine::Window){
        if let Some(found)=self.engine.take_selection_bounds(){
            let [x,y,w,h]=window.roi;let (sx,sy)=(w/window.width.max(1) as f32,h/window.height.max(1) as f32);
            self.viewer.selection_bounds.insert(seen,found.into_iter().map(|(id,[x0,y0,x1,y1])|(id,[x+x0*sx,y+y0*sy,x+x1*sx,y+y1*sy])).collect());
        }
    }
    fn bounds_from(&self,view:&crate::doc::store::StoreView<'_>,eye:&Eye,scene:&crate::render::frame_graph::SceneValue,layer:LayerId,seen:View)->Option<Json>{
        let Eye{time,comp,camera,observer,document}=*eye;
        let r=scene_layer(scene,layer)?;
        let camera=if r.projection==LayerProjection::TwoD{document}else{camera};
        let b=self.engine.selected_scene_layer_bounds_in(view,&scene.layers,layer,time)?;
        let world=crate::doc::core::depth_scaled(r.transform.spatial);
        let corners:Vec<_>=match (self.viewer.selection_bounds.get(&seen).and_then(|m|m.get(&layer)),r.projection){
            (Some(&[x0,y0,x1,y1]),_)=>vec![[x0 as f64,y0 as f64],[x1 as f64,y0 as f64],[x1 as f64,y1 as f64],[x0 as f64,y1 as f64]],
            (None,projection)=>match projection{
            LayerProjection::ThreeD=>crate::doc::core::projected_screen_corners(comp,camera,observer,r.projection,world,b.min,b.max).iter().map(|p|[p.x as f64,p.y as f64]).collect(),
            _=>{
                let outline=self.engine.selected_scene_layer_outline_in(view,&scene.layers,layer,time).unwrap_or_else(||b.corners().to_vec());
                crate::doc::core::facing_frame(comp,camera,observer,r.projection,world,b.min,b.max,&outline).iter().map(|p|[p.x as f64,p.y as f64]).collect()
            }
        }};
        let anchor=match view.value_at(layer,&PropertyId::new(property::ANCHOR).ok()?,time).ok().flatten(){Some(Value::Vec2(v))=>v,_=>[0.0,0.0]};
        let fractions:[f64;2]=std::array::from_fn(|i|(anchor[i]-b.min[i]as f64)/(b.max[i]-b.min[i]).max(1e-6)as f64);
        Some(json!({"layer":layer.0,"corners":corners,"localMin":b.min,"localMax":b.max,"anchorFraction":fractions}))
    }
    /// 今の姿 —— 観測者と時刻で動く物。cache した行の上へ毎回これを載せる。
    /// `bounds` は Camera(出力)、`stageBounds` は Stage(観測者)で見た枠。
    fn overlay_geometry(&self,view:&crate::doc::store::StoreView<'_>,row:&mut Json,eyes:&(Eye,Eye),scene:&crate::render::frame_graph::SceneValue,id:LayerId,live:bool)->Result<(),String>{
        let position=self.position_in(view,id)?;let bounds=self.bounds_from(view,&eyes.0,scene,id,View::Camera);
        row["corners"]=json!(bounds.as_ref().and_then(|b|b["corners"].as_array()).map(|c|if c.len()==8{vec![c[0].clone(),c[1].clone(),c[3].clone(),c[2].clone()]}else{c.clone()}));
        row["x"]=json!(position[0]);row["y"]=json!(position[1]);
        if !live {row["anchorFraction"]=bounds.as_ref().map(|b|b["anchorFraction"].clone()).unwrap_or(Json::Null);}
        row["bounds"]=json!(bounds);
        row["stageBounds"]=json!(self.bounds_from(view,&eyes.1,scene,id,View::User));
        Ok(())
    }
    pub(crate) fn build_status(&mut self)->Result<Json,String>{
        let at=self.time()?;let view=self.doc.view();let comp=view.composition().map_err(e)?.ok_or("No composition")?;
        let catalog=crate::render::engine::known_effects();
        let scene=self.engine.frame_graph_editor_scene(&view,at).map_err(e)?;
        let clipping=view.clipping_bases().map_err(e)?;
        let eyes=(self.eye(View::Camera).ok_or("No composition")?,self.eye(View::User).ok_or("No composition")?);
        // 再生中で Document が変わっていなければ、時刻で変わる物(値・枠)だけの軽い status にする。
        let revision=format!("{:?}",self.doc.revision());
        let live=self.viewer.clock.playing()&&self.full_status_revision.borrow().as_deref()==Some(revision.as_str());
        // 並べ替えの鍵も 1 層 1 回。sort_by_key は比較のたびに鍵を引くので、store を O(n log n) 回叩いていた。
        let mut ids=view.layers();
        let order:std::collections::HashMap<LayerId,i16>=ids.iter().map(|id|(*id,view.meta(*id).ok().flatten().map_or(0,|m|m.order))).collect();
        ids.sort_by_key(|id|std::cmp::Reverse((order.get(id).copied().unwrap_or(0),id.0)));
        let mut layers=Vec::new();
        let keys=self.layer_keys(&view,at,&scene,&clipping,live)?;
        // 軽い status で行(値・効果)を持つのは選択中の層だけ —— Inspector と Stage の枠が読む物。
        // 他の層は今の姿(枠・位置)だけ: Stage の当たり判定はそれで足りる。
        let wanted=|id:LayerId| !live||self.viewer.selected()==Some(id)||self.viewer.selected_ids.contains(&id);
        for id in ids{
            if !wanted(id){
                let mut row=json!({"id":id.0});
                self.overlay_geometry(&view,&mut row,&eyes,&scene,id,live)?;
                layers.push(row);
                continue;
            }
            let key=keys.get(&id).copied();
            let hit=key.and_then(|key|self.snapshot_cache.borrow().rows.get(&id).filter(|(had,_)|*had==key).map(|(_,row)|row.clone()));
            let row=match hit {
                Some(row)=>Some(row),
                None=>{
                    let built=self.layer_json(&view,id,at,comp.fps,&catalog,&clipping,live)?;
                    match (&built,key) {
                        (Some(row),Some(key))=>{self.snapshot_cache.borrow_mut().rows.insert(id,(key,row.clone()));}
                        (None,_)=>{self.snapshot_cache.borrow_mut().rows.remove(&id);}
                        _=>{}
                    }
                    built
                }
            };
            if let Some(mut row)=row {self.overlay_geometry(&view,&mut row,&eyes,&scene,id,live)?;layers.push(row);}
        }
        self.snapshot_cache.borrow_mut().rows.retain(|id,_|keys.contains_key(id));
        if live {
            let (undo,redo)=self.doc.history_depth();
            let selected_keys:Vec<_>=self.viewer.selected_keys.iter().map(|k|json!({"layer":k.layer.0,"property":k.property.as_ref().map(|p|p.name()),"frame":(k.at_sec*comp.fps.as_f64()).round()as i64})).collect();
            let point=self.viewer.selected().map(|id|self.position(id)).transpose()?.unwrap_or([0.0,0.0]);
            let live_layers=layers;
            // 寸法は Swift の render が毎コマ読む。軽い status でも落とさない(落とすと再生 2 コマ目で render が失敗し、再生が止まる)。
            return Ok(json!({"frame":self.viewer.frame,"playing":self.viewer.clock.playing(),"documentRevision":revision,"preview":self.preview.is_some(),"previewOwner":self.preview.as_ref().map(|p|p.0),"previewInteraction":self.preview_tag,"undo":undo,"redo":redo,"width":comp.width,"height":comp.height,"stageWindow":self.viewer.stage_window.map(|w|json!({"width":w.width,"height":w.height,"roi":w.roi})),"fps":comp.fps.as_f64(),"durationFrames":comp.duration_frames,
                "selectedId":self.viewer.selected().map(|s|s.0),"selectedIds":self.viewer.selected_ids.iter().map(|s|s.0).collect::<Vec<_>>(),"selectedKeys":selected_keys,"x":point[0],"y":point[1],
                "animate":self.viewer.animate!=Animate::Off,"renderCount":self.render_count,"framesSkipped":self.frames.skipped(),"pickedColor":self.viewer.picked_color,"pickSerial":self.viewer.pick_serial,"renderMs":self.render_ms,"liveLayers":live_layers}));
        }
        *self.full_status_revision.borrow_mut()=Some(revision);
        let assets:Result<Vec<_>,String>=view.assets().map_err(e)?.into_iter().map(|a|{
            let used=self.asset_used(a.id)?;
            let path=a.path_absolute.clone();let missing=path.as_ref().is_none_or(|p|!std::path::Path::new(p).exists());
            Ok(json!({"id":a.id.to_string(),"name":a.name,"path":path,"mime":a.asset_type,"used":used,"missing":missing,"thumbnail":path.as_ref().and_then(|p|if a.asset_type.starts_with("image/"){editor::thumbnail::image_data_uri(p)}else if a.asset_type.starts_with("video/"){editor::thumbnail::video_data_uri(p)}else{None}),"role":match a.role{AssetRole::Reference=>"reference",_=>"material"},"facts":path.as_ref().filter(|_|!missing).and_then(|p|editor::thumbnail::facts(p,&a.asset_type)),"seconds":a.duration.map(|d|d.as_seconds_f64())}))
        }).collect();
        let used=editor::fixture::used_colors_from_doc(&self.doc);let authored=!used.is_empty();
        let swatches=if authored{used}else{editor::fixture::default_palette()};
        let palette:Vec<_>=swatches.iter().map(|s|json!({"rgba":s.rgba.map(|x|x as f64/255.0),"hex":s.hex,"used":authored})).collect();
        let markers:Result<Vec<_>,String>=view.markers().map_err(e)?.into_iter().map(|m|Ok(json!({"id":format!("{}/{}",m.time.num(),m.time.den()),"frame":m.time.try_to_frame_round(comp.fps).map_err(e)?,"name":m.name,"body":m.body}))).collect();
        let playback_health=match self.viewer.clock.health(){
            crate::render::playback::PlaybackHealth::AudioReady=>json!({"kind":"AudioReady","reason":null,"detail":null}),
            crate::render::playback::PlaybackHealth::VisualOnly(reason)=>match reason{
                crate::render::playback::VisualFallback::NoAudio=>json!({"kind":"VisualOnly","reason":"NoAudio","detail":null}),
                crate::render::playback::VisualFallback::Program(detail)=>json!({"kind":"VisualOnly","reason":"Program","detail":detail}),
                crate::render::playback::VisualFallback::Device(detail)=>json!({"kind":"VisualOnly","reason":"Device","detail":detail}),
            }
        };
        let waveforms:Vec<_>=self.viewer.clock.waveform_tracks().iter().map(|track|json!({"layer":track.layer.0,"columns":track.columns(0.0,self.viewer.clock.duration(),256.0/self.viewer.clock.duration().max(0.01)).unwrap_or_default().iter().map(|c|json!({"frame":c.at_sec*comp.fps.as_f64(),"min":c.min,"max":c.max})).collect::<Vec<_>>()})).collect();
        let (undo,redo)=self.doc.history_depth();let point=self.viewer.selected().map(|id|self.position(id)).transpose()?.unwrap_or([0.0,0.0]);
        let color_target=self.viewer.color_target.as_ref().and_then(|slot|editor::color::read_color(&self.doc,slot,self.time().ok()?).map(|rgba|json!({"layer":slot.layer().map(|l|l.0),"slot":slot,"label":"Color","rgba":rgba,"alpha":editor::color::has_alpha(slot)})));
        let selected_keys:Vec<_>=self.viewer.selected_keys.iter().map(|k|json!({"layer":k.layer.0,"property":k.property.as_ref().map(|p|p.name()),"frame":(k.at_sec*comp.fps.as_f64()).round()as i64})).collect();
        let generation=crate::render::engine::catalog_generation();
        let catalog_rows=catalog.iter().map(|e|json!({"id":e.plugin_id,"name":e.label,"stage":format!("{:?}",e.stage),"generation":generation,"usesClock":e.uses_clock,"persistent":e.persistent,"readsBackdrop":e.reads_backdrop,"layerInputs":e.image_layer_fields.len()+e.params.iter().filter(|p|p.layer).count(),"paramCount":e.params.len()})).collect::<Vec<_>>();
        let mut status=json!({"observer":self.observer_status()?,"cameraGizmos":self.camera_gizmos()?,"width":comp.width,"height":comp.height,"stageWindow":self.viewer.stage_window.map(|w|json!({"width":w.width,"height":w.height,"roi":w.roi})),"fps":comp.fps.as_f64(),"fpsNum":comp.fps.num(),"fpsDen":comp.fps.den(),"durationFrames":comp.duration_frames,"background":comp.background,"frame":self.viewer.frame,"playing":self.viewer.clock.playing(),"playbackHealth":playback_health,"waveforms":waveforms,"undo":undo,"redo":redo,"path":self.path,"dirty":self.is_dirty()?,"layers":layers,"selectedId":self.viewer.selected().map(|id|id.0),"selectedIds":self.viewer.selected_ids.iter().map(|id|id.0).collect::<Vec<_>>(),"selectedKeys":selected_keys,"selectedBounds":self.viewer.selected().and_then(|id|self.bounds(id)),"x":point[0],"y":point[1],"assets":assets?,"catalog":catalog_rows,"catalogErrors":crate::render::engine::catalog_errors(),"palette":palette,"markers":markers?,"colorTarget":color_target,"capabilities":crate::port::CAPABILITIES,"easeKinds":if self.viewer.clock.playing(){Json::Null}else{json!(editor::ease_kinds::KINDS.iter().copied().map(interp).collect::<Vec<_>>())},"documentRevision":format!("{:?}",self.doc.revision()),"deviceId":self.device_id.to_string(),"renderCount":self.render_count,"framesSkipped":self.frames.skipped(),"renderMs":self.render_ms,"interopCopies":0,"readbacks":0,"error":self.error,"preview":self.preview.is_some(),"export":self.exporter.status(),"freeze":self.freezer.status()});
        status["previewOwner"] = json!(self.preview.as_ref().map(|p|p.0));
        status["previewInteraction"] = json!(self.preview_tag);
        status["visualSamples"]=json!(true);
        status["fontFamilies"]=json!(crate::render::picture::shaping::font_families());
        status["history"]=self.history.snapshot(self.doc.edit_head());
        status["spatialGizmo"]=self.spatial_gizmo(View::Camera)?;
        status["stageSpatialGizmo"]=self.spatial_gizmo(View::User)?;
        status["notebook"]=serde_json::to_value(view.notebook().map_err(e)?).map_err(e)?;
        status["depthLayout"]=self.depth_layout(&scene)?;
        status["backgrounds"]=json!(editor::create::backgrounds().iter().map(|b|json!({"id":b.id,"name":b.name,"thumbnail":editor::thumbnail::image_data_uri(&b.path)})).collect::<Vec<_>>());
        status["primitives"]=json!(editor::create::primitives().iter().map(|p|json!({"id":p.id,"name":p.name})).collect::<Vec<_>>());
        status["animate"]=json!(self.viewer.animate!=Animate::Off);
        if status["easeKinds"].is_null(){status.as_object_mut().unwrap().remove("easeKinds");}
        status["importExtensions"]=json!(crate::render::media::import_extensions());
        status["names"]=json!(names::fixed().collect::<std::collections::BTreeMap<_,_>>());
        Ok(status)
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn layer_json(&self,view:&StoreView<'_>,id:LayerId,at:RationalTime,fps:Fps,catalog:&[crate::render::engine::EffectDescriptor],clipping:&std::collections::HashMap<LayerId,Option<LayerId>>,live:bool)->Result<Option<Json>,String>{
        let attrs=view.attrs(id).map_err(e)?.unwrap_or_default();let Some(meta)=view.meta(id).map_err(e)? else{return Ok(None)};
        let data=editor::functions::read::inspector_data_from_doc(view,id,at,catalog);
        let mut properties=Vec::new();let mut seen=std::collections::BTreeSet::new();
        for row in data.transform.iter().chain(data.text.iter()){
            if let Some(p)=&row.property{
                if seen.insert(p.clone()){properties.push(prop(view,id,p,&row.label,&row.value,row.range,at,fps,live)?);}
            }
            for (index,axis) in row.axis.iter().enumerate(){if let Some((p,v))=axis{
                if seen.insert(p.clone()){properties.push(prop(view,id,p,&names::label_or_id(p),v,row.range,at,fps,live)?);}
            }}
        }
        for p in view.properties(id){if !p.name().starts_with(property::EFFECT_PREFIX)&&seen.insert(p.name().into()){
            if let Some(v)=view.value_at(id,&p,at).map_err(e)?{properties.push(prop(view,id,p.name(),&names::label_or_id(p.name()),&v,None,at,fps,live)?);}
        }}
        if meta.source == LayerSource::Camera {
            properties = property::CAMERA_ROWS.iter().map(|(p,label,v,range)| prop(view,id,p,label,v,*range,at,fps,live)).collect::<Result<Vec<_>,_>>()?;
        }
        if meta.source == LayerSource::Particles {
            properties = crate::doc::store::particles::ROWS.iter().map(|(p,label,v,range)| prop(view,id,p,label,v,*range,at,fps,live)).collect::<Result<Vec<_>,_>>()?;
        }
        // 並べる法の欄: Group は Display と、Display に応じた欄。並ぶ子は子の欄。表の順と選択肢で出す。
        {
            use crate::doc::store::layout;
            properties.retain(|p| !p["id"].as_str().is_some_and(|id| id.starts_with("layout.") || id.starts_with("connect.")));
            let number = |layer: LayerId, name: &str| -> Result<f64, String> {
                Ok(match view.value_at(layer, &PropertyId::new(name).map_err(e)?, at).map_err(e)? { Some(Value::Enum(v)) => v as f64, Some(Value::F64(v)) => v, _ => 0.0 })
            };
            let push = |properties: &mut Vec<Json>, layer: LayerId, row: &layout::Row| -> Result<(), String> {
                let mut json = prop(view, layer, row.0, row.1, &row.2, row.3, at, fps, live)?;
                if !live && !row.4.is_empty() { json["choices"] = json!(row.4); }
                properties.push(json);
                Ok(())
            };
            if meta.source == LayerSource::Group {
                let display = number(id, layout::DISPLAY)?.round() as i64;
                for row in layout::GROUP_ROWS {
                    let flex = matches!(row.0, layout::FLEX_DIRECTION | layout::FLEX_WRAP | layout::JUSTIFY_CONTENT | layout::ALIGN_ITEMS);
                    let grid = matches!(row.0, layout::GRID_COLUMNS | layout::GRID_ROWS);
                    if row.0 == layout::DISPLAY || (display == 1 && !grid) || (display == 2 && !flex) { push(&mut properties, id, row)?; }
                }
                if display == 2 {
                    for (count, prefix, default) in [(layout::GRID_COLUMNS, layout::COLUMN_PREFIX, 2.0), (layout::GRID_ROWS, layout::ROW_PREFIX, 0.0)] {
                        let n = match view.value_at(id, &PropertyId::new(count).map_err(e)?, at).map_err(e)? { Some(Value::F64(v)) => v, _ => default }.round().clamp(0.0, 64.0) as u32;
                        for i in 1..=n {
                            let track = format!("{prefix}{i}");
                            let label = names::label_or_id(&track).into_owned();
                            properties.push(prop(view, id, &track, &label, &Value::F64(layout::TRACK_DEFAULT), Some((0.0, 1000.0)), at, fps, live)?);
                        }
                    }
                }
            }
            if !matches!(meta.source, LayerSource::Camera | LayerSource::Stage) {
                for row in layout::SPACE_ROWS { push(&mut properties, id, row)?; }
            }
            // Camera は移り方だけ(Framing Size で箱から箱へ移る時)。
            if meta.source == LayerSource::Camera {
                for row in layout::SPACE_ROWS.iter().filter(|r| matches!(r.0, layout::TRANSITION_DURATION | layout::TRANSITION_EASING)) { push(&mut properties, id, row)?; }
            }
            if meta.source == LayerSource::Text {
                for row in layout::READOUT_ROWS { push(&mut properties, id, row)?; }
                // Split の単位の順番の札(箱の子と同じ 3 欄)。
                for row in layout::GROUP_ROWS.iter().filter(|r| layout::is_schedule_row(r.0)) { push(&mut properties, id, row)?; }
            }
            if meta.source == LayerSource::Shape {
                for row in layout::CONNECT_ROWS { push(&mut properties, id, row)?; }
                // 線の太さ(`shape.stroke_width`、書類の線の太さが既定)。つなぐ線・なぞる形の細さもこれで決める。
                if !properties.iter().any(|p| p["id"] == property::SHAPE_STROKE_WIDTH) {
                    fn first_width(nodes: &[crate::doc::store::ShapeNode]) -> Option<f64> {
                        nodes.iter().find_map(|n| match n {
                            crate::doc::store::ShapeNode::Leaf(shape) => Some(shape.stroke.as_ref().map_or(0.0, |s| s.width)),
                            crate::doc::store::ShapeNode::Group(g) => first_width(&g.children),
                        })
                    }
                    let width = first_width(&view.shapes(id).map_err(e)?).unwrap_or(0.0);
                    properties.push(prop(view, id, property::SHAPE_STROKE_WIDTH, "Stroke Width", &Value::F64(width), Some((0.0, 1000.0)), at, fps, live)?);
                }
            }
            if let Some(parent) = attrs.parent.filter(|p| view.meta(*p).ok().flatten().is_some_and(|m| m.source == LayerSource::Group)) {
                let display = number(parent, layout::DISPLAY)?.round() as i64;
                if display != 0 {
                    for row in layout::ITEM_ROWS {
                        let grid = matches!(row.0, layout::COLUMN_START | layout::ROW_START | layout::COLUMN_SPAN | layout::ROW_SPAN);
                        if display == 2 || !grid { push(&mut properties, id, row)?; }
                    }
                }
            }
        }
        if meta.source == LayerSource::Stage {
            properties = property::STAGE_MARGINS.iter()
                .map(|p| prop(view,id,p,&names::label_or_id(p),&Value::F64(0.0),Some((0.0,100000.0)),at,fps,live)).collect::<Result<Vec<_>,_>>()?;
        }
        let text=if let Some(t)=view.text_document(id).map_err(e)?{
            let keyed=t.content.keys().iter().any(|k|k.t.try_to_frame_round(fps).ok()==Some(self.viewer.frame));
            let content=if live { json!({"id":"content","value":t.content.eval(at),"keyedNow":keyed}) } else {
                let keys:Result<Vec<_>,String>=t.content.keys().iter().map(|k|Ok(json!({"frame":k.t.try_to_frame_round(fps).map_err(e)?,"value":k.content,"interp":{"kind":"Hold"}}))).collect();
                json!({"id":names::TEXT_CONTENT,"label":names::label_or_id(names::TEXT_CONTENT),"kind":"text","value":t.content.eval(at),"min":null,"max":null,"keyedNow":keyed,"keys":keys?})
            };
            properties.insert(0,content);
            let resolved=motolii_render::picture::resolve::text::resolved_text_document(view, id,at).map_err(e)?.unwrap_or(t.clone());
            // 段落の欄(選択肢は名前で書ける): 揃えと文字組みの 3 法。
            let laws = resolved.alignment;
            let split = match view.value_at(id, &PropertyId::new(names::TEXT_SPLIT).map_err(e)?, at).map_err(e)? { Some(Value::Enum(v)) => v, _ => 0 };
            let paragraph: [(&str, i64, &[&str]); 5] = [
                (names::TEXT_JUSTIFY, resolved.justify.to_enum_value(), &["Left","Right","Center"]),
                (names::TEXT_SPLIT, split, names::TEXT_SPLIT_CHOICES),
                (names::TEXT_AUTOSPACE, laws.autospace.to_enum_value(), crate::doc::store::TextAutospace::CHOICES),
                (names::TEXT_SPACING_TRIM, laws.spacing_trim.to_enum_value(), crate::doc::store::TextSpacingTrim::CHOICES),
                (names::HANGING_PUNCTUATION, laws.hanging.to_enum_value(), crate::doc::store::HangingPunctuation::CHOICES),
            ];
            for (name, value, choices) in paragraph {
                properties.retain(|p|p["id"]!=name);
                let mut row=prop(view,id,name,&names::label_or_id(name),&Value::Enum(value),None,at,fps,live)?;
                row["choices"]=json!(choices);
                properties.push(row);
            }
            let s=resolved.styles.first();
            json!({"classes":crate::edit::text_edit::classifications(t.content.eval(at)),"styles":resolved.styles,"runs":resolved.runs,"fontFamily":s.map(|s|&s.font.family),"content":t.content.eval(at),"size":s.map(|s|s.size),"lineHeight":s.and_then(|s|s.line_height),"tracking":s.map(|s|s.tracking)})
        }else{Json::Null};
        // 色の行は property。Browser の輪へ焦点を渡す slot を添える。
        for row in properties.iter_mut(){if row["kind"]=="color"{if let Some(slot)=row["id"].as_str().and_then(|name|editor::color::slot_of(&self.doc,id,name)){row["alpha"]=json!(matches!(slot,crate::viewer::ColorSlot::TextFill{..}|crate::viewer::ColorSlot::Property{..}));row["slot"]=json!(slot);}}}
        let fill_slot=(meta.source==LayerSource::Shape).then(||view.shapes(id).ok()).flatten().and_then(|shapes|editor::functions::read::first_shape_fill(&shapes,Vec::new())).map(|(path,_)|crate::viewer::ColorSlot::ShapeFill{layer:id,path});
        let effects:Result<Vec<_>,String>=data.effects.iter().map(|effect|{
            let kind=crate::render::extensions::placement::kind(&effect.plugin_id);
            let mut params:Vec<Json>=effect.params.iter().filter_map(|p|p.property.as_ref().map(|idp|prop(view,id,idp,&p.label,&p.value,p.range,at,fps,live))).collect::<Result<_,_>>()?;
            if live { return Ok(json!({"id":effect.id,"params":params})); }
            for row in &mut params{
                let name=row["id"].as_str().unwrap_or_default().rsplit(".param.").next().unwrap_or_default().to_owned();
                if let Some(param)=crate::render::extensions::kind(&effect.plugin_id).and_then(|k|k.params.iter().find(|p|p.name==name)){
                    if !param.section.is_empty(){row["section"]=json!(param.section);}
                }
                if let Some(p)=catalog.iter().find(|d|d.plugin_id==effect.plugin_id).and_then(|d|d.params.iter().find(|p|p.name==name)){
                    if let Some(choices)=p.choices.clone(){row["choices"]=json!(choices);}
                    if let Some(s)=&p.subtype{row["subtype"]=json!(s);}
                    if let Some(u)=&p.unit{row["unit"]=json!(u);}
                    if let Some(g)=&p.group{row["group"]=json!(format!("effect.{}.param.{}",effect.id,g));}
                    if p.advanced{row["advanced"]=json!(true);}
                    if p.hero{row["hero"]=json!(true);}
                    if p.layer{row["layer"]=json!(true);}
                    row["default"]=json!(p.default);
                }
            }
            let layout=kind.map(|k|{
                let pid=|name:&str|format!("effect.{}.param.{}",effect.id,name);
                let shown=|name:&str|params.iter().any(|r|r["id"]==pid(name));
                let share=format!("effect.{}.param.{}",effect.id,crate::render::extensions::placement::SHARE_PREFIX);
                json!({"columns":["Each","Random"],"count":pid("count"),"along":pid("mode"),"pick":pid("pick"),"seed":pid("seed"),
                    "materials":params.iter().filter(|r|r["id"].as_str().is_some_and(|i|i.starts_with(&share))).map(|r|json!({"id":r["id"],"label":r["label"]})).collect::<Vec<_>>(),
                    "shape":k.shape.iter().filter(|n|shown(n)).map(|n|json!({"id":pid(n),"label":k.params.iter().find(|p|p.name==*n).map_or(*n,|p|p.label),"unit":crate::render::extensions::placement::unit(n)})).collect::<Vec<_>>(),
                    "rows":k.grid.iter().map(|r|json!({"label":r.label,"unit":r.unit,"each":r.each.map(pid),"random":r.random.map(pid),"axis":r.axis,"advanced":r.advanced})).collect::<Vec<_>>()})
            });
            let enabled=!matches!(view.value_at(id,&PropertyId::effect_enabled(EffectId(effect.id)),at).map_err(e)?,Some(Value::Bool(false)));let whole=matches!(view.value_at(id,&PropertyId::effect_scope(EffectId(effect.id)),at).map_err(e)?,Some(Value::Enum(v)) if v==crate::doc::store::EffectScope::Whole.enum_value());Ok(json!({"id":effect.id,"enabled":enabled,"whole":whole,"pluginId":effect.plugin_id,"name":catalog.iter().find(|d|d.plugin_id==effect.plugin_id).map_or(effect.plugin_id.as_str(),|d|d.label.as_str()),"placement":kind.is_some(),"layout":layout,"params":params}))
        }).collect();
        if live {
            let mut row=json!({"id":id.0,"text":text,"properties":properties,"effects":effects?});
            if let Some(slot)=&fill_slot{row["fill"]=editor::gradient::model(&self.doc,slot,at).unwrap_or(Json::Null);}
            return Ok(Some(row));
        }
        let content_keys:Vec<_>=properties.iter().find(|p|p["id"]=="content").and_then(|p|p["keys"].as_array()).into_iter().flatten().map(|k|json!({"frame":k["frame"],"content":k["value"]})).collect();
        let mut row=json!({"id":id.0,"name":attrs.name,"kind":source_kind(&meta.source),"ghost":attrs.ghost,"ghostable":crate::editor::timeline_edit::ghostable(view,id),"parent":attrs.parent.map(|p|p.0),"order":meta.order,"hidden":attrs.hidden,"solo":attrs.solo,"locked":attrs.locked,"clipToBelow":attrs.clip_to_below,"clipBase":clipping.get(&id).copied().flatten().map(|b|b.0),"projection":match attrs.projection{LayerProjection::TwoD=>"2D",LayerProjection::TwoPointFiveD=>"2.5D",LayerProjection::ThreeD=>"3D"},"flatten":attrs.flatten,"environment":attrs.environment,"frozen":attrs.frozen,"blendMode":attrs.blend_mode,"matte":attrs.matte,"start":meta.timing.start,"duration":meta.timing.duration,"sourceIn":meta.timing.source_in,"properties":properties,"text":text,"effects":effects?,"contentKeys":content_keys});
        if let Some(slot)=&fill_slot{row["fill"]=editor::gradient::model(&self.doc,slot,at).unwrap_or(Json::Null);}
        Ok(Some(row))
    }
}

#[cfg(test)]
mod frame_cost_probe;

#[cfg(test)]
mod camera_target_tests {
    use super::*;
    use crate::EditorRuntime;
    use std::ffi::{CStr,CString};
    fn request(rt:&mut EditorRuntime,command:Json)->Json{
        let command=CString::new(command.to_string()).unwrap();
        let reply=unsafe{crate::motolii_probe_request(rt,command.as_ptr())};
        let reply:Json=serde_json::from_str(unsafe{CStr::from_ptr(reply)}.to_str().unwrap()).unwrap();
        assert!(reply["error"].is_null(),"{reply}");reply
    }
    /// Create の particles で粒子の層ができ、Inspector には粒子の欄の表(Rate から Seed まで)が並び、鍵の打てる値として書ける。
    #[test]
    fn creating_particles_shows_the_particle_rows(){
        let mut rt=EditorRuntime::open("").unwrap();
        request(&mut rt,json!({"op":"create","kind":"particles"}));
        let layer=rt.viewer.selected().unwrap();
        assert_eq!(rt.doc.view().meta(layer).unwrap().unwrap().source,LayerSource::Particles);
        request(&mut rt,json!({"op":"setProperty","layer":layer.0,"property":"particles.rate","value":120.0}));
        let status=rt.status().unwrap();
        let row=status["layers"].as_array().unwrap().iter().find(|l|l["id"]==layer.0).expect("the layer is listed");
        let ids:Vec<&str>=row["properties"].as_array().unwrap().iter().filter_map(|p|p["id"].as_str()).collect();
        let expected:Vec<&str>=crate::doc::store::particles::ROWS.iter().map(|r|r.0).collect();
        assert_eq!(ids,expected);
        let rate=row["properties"].as_array().unwrap().iter().find(|p|p["id"]=="particles.rate").unwrap();
        assert_eq!(rate["value"],json!(120.0));
    }
    /// 層ターゲットは Stage の枠・Depth の点と同じ「bounds の中心」を見る。anchor をずらしても注視点は形の中心に残る。
    #[test]
    fn a_layer_target_aims_at_the_bounds_centre_even_when_the_anchor_moves(){
        let mut rt=EditorRuntime::open("").unwrap();
        request(&mut rt,json!({"op":"create","kind":"rectangle"}));
        let shape=rt.viewer.selected().unwrap();
        request(&mut rt,json!({"op":"create","kind":"camera"}));
        let camera=rt.viewer.selected().unwrap();
        request(&mut rt,json!({"op":"setProperty","layer":camera.0,"property":"camera.target","value":shape.0}));
        request(&mut rt,json!({"op":"setProperty","layer":shape.0,"property":"anchor","value":[200.0,-80.0]}));
        // 形の bounds は 1 度描いてから測れる。
        let comp=rt.doc.view().composition().unwrap().unwrap().spec();
        let texture=rt.engine.gpu_device().create_texture(&wgpu::TextureDescriptor {
            label:Some("camera target test"),size:wgpu::Extent3d{width:comp.width,height:comp.height,depth_or_array_layers:1},mip_level_count:1,sample_count:1,
            dimension:wgpu::TextureDimension::D2,format:wgpu::TextureFormat::Bgra8Unorm,usage:wgpu::TextureUsages::RENDER_ATTACHMENT|wgpu::TextureUsages::TEXTURE_BINDING,view_formats:&[],
        });
        let time=rt.time().unwrap();
        rt.engine.render_frame_into(&rt.doc.view(),time,&texture).unwrap();
        let status=rt.build_status().unwrap();
        let depth=&status["depthLayout"];
        let item=depth["items"].as_array().unwrap().iter().find(|i|i["id"]==shape.0).unwrap();
        let point=[item["point"][0].as_f64().unwrap(),item["point"][1].as_f64().unwrap()];
        assert!(point[0].abs()<1e-2&&point[1].abs()<1e-2,"the target sits on the origin of Depth: {point:?}");
        assert_eq!(depth["camera"]["target"],json!(shape.0));
        let view=rt.doc.view();
        let seen=rt.engine.resolve_camera(&view,time).unwrap();
        let authored=motolii_render::picture::resolve::camera::resolve_camera(&view, time).unwrap();
        assert!((seen.center[0]-authored.center[0]).abs()>100.0,"Document alone looks at the anchor; the engine looks at the shape");
    }
}

#[cfg(test)]
mod camera_view_cage_tests;
