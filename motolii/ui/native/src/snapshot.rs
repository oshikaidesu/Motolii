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
    LayerSource::Camera=>"Camera",LayerSource::Stage=>"Stage",LayerSource::Text=>"Text",LayerSource::Shape=>"Shape",LayerSource::Group=>"Group",LayerSource::Null=>"Null",
    LayerSource::File{path,..}=>{
        if crate::render::media::is_mesh_path(path){"Mesh"}else if crate::render::media::is_point_cloud_path(path){"PointCloud"}else{
            let mime=std::path::Path::new(path).extension().and_then(|e|e.to_str()).and_then(crate::render::media::asset_type_for_extension).unwrap_or_default();
            if mime.starts_with("audio/"){"Audio"}else if mime.starts_with("video/"){"Video"}else{"Image"}
        }
    }
}}
#[derive(Clone,Copy)]
struct Eye{time:RationalTime,comp:crate::doc::core::CompSpec,camera:crate::doc::core::ResolvedCamera,observer:crate::doc::core::ResolvedCamera,document:crate::doc::core::ResolvedCamera}
/// 2 枚の絵: Stage(観測者が見る世界、カメラは箱)と Camera(箱の中身 = 出力)。両方同時に生きる。
#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub(crate) enum View{User,Camera}
impl View{
    pub(crate) fn parse(name:&str)->Result<Self,String>{match name{"User"|"Stage"=>Ok(Self::User),"Camera"=>Ok(Self::Camera),_=>Err(format!("Unknown view {name}"))}}
    pub(crate) fn name(self)->&'static str{match self{Self::User=>"User",Self::Camera=>"Camera"}}
}
impl EditorRuntime{
    pub(crate) fn view_camera(&self,view:View)->Result<crate::doc::core::ResolvedCamera,String>{
        match view{View::User=>Ok(self.user_camera),View::Camera=>self.engine.resolve_camera(&self.doc.view(),self.time()?).map_err(e)}
    }
    /// view の描く窓。Camera は出力寸法そのもの、Stage はタブが置いた窓(未設定なら出力寸法)。
    pub(crate) fn window(&self,view:View)->Result<crate::render::engine::Window,String>{
        let comp=self.doc.view().composition().map_err(e)?.ok_or("No composition")?.spec();
        Ok(match view{View::Camera=>crate::render::engine::Window::output(comp),View::User=>self.stage_window.unwrap_or(crate::render::engine::Window{projection_camera:Some(Default::default()),..crate::render::engine::Window::output(comp)})})
    }
    /// 層を置くカメラ。2D は出力の画面の物なのでどの view でも作中カメラの箱に貼り付く。
    /// 2.5D・3D は世界に居る: Stage は既定(Boxcam の Original Comp)、Camera は作中カメラ。
    pub(crate) fn projection_camera(&self,view:View,projection:LayerProjection)->Result<crate::doc::core::ResolvedCamera,String>{
        match (view,projection){(View::User,LayerProjection::TwoD)|(View::Camera,_)=>self.engine.resolve_camera(&self.doc.view(),self.time()?).map_err(e),(View::User,_)=>Ok(Default::default())}
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
        let changed=self.stage_window!=next;
        self.stage_window=next;
        Ok(changed)
    }
    fn depth_layout(&self,resolved:&[crate::doc::store::ResolvedLayer])->Result<Json,String>{
        let view=self.doc.view();let time=self.time()?;let comp=view.composition().map_err(e)?.ok_or("No composition")?.spec();
        // 原点は注視点。カメラは eye の位置に置き、drag で orbit と距離を author する。
        let seen=self.engine.resolve_camera_in(&view,resolved,time).map_err(e)?;
        let camera=crate::doc::core::camera_projection(comp,seen);
        let target=seen.target(comp);
        let camera_layer=view.active_camera_layer(time).map_err(e)?;
        let target_layer=camera_layer.map(|id|view.camera_target_layer(id,time)).transpose().map_err(e)?.flatten();
        let worlds=view.world_transforms3d(time).map_err(e)?;
        let mut items=Vec::new();
        for layer in resolved {
            // 配置効果の複製は層として 1 つ(元の姿)だけ並べる
            if matches!(layer.source,LayerSource::Camera|LayerSource::Stage)||layer.copy!=0 {continue}
            let Some(world)=worlds.get(&layer.id) else{continue};
            let attrs=view.attrs(layer.id).map_err(e)?.unwrap_or_default();
            let local=self.engine.selected_layer_bounds_in(&view,&resolved,layer.id,time).map(|b|glam::Vec3::from(b.center())).unwrap_or(glam::Vec3::ZERO);
            let center=world.transform_point3(local)-target;
            let parent=attrs.parent.and_then(|id|worlds.get(&id).copied()).unwrap_or(glam::Affine3A::IDENTITY);
            let inverse=parent.inverse();
            let get=|name|view.value_at(layer.id,&PropertyId::new(name).unwrap(),time).ok().flatten();
            let position=match get(property::POSITION){Some(Value::Vec2(v))=>v,_=>[0.0,0.0]};
            let z=match get(property::POSITION_Z){Some(Value::F64(v))=>v,_=>0.0};
            items.push(json!({"id":layer.id.0,"name":attrs.name,"point":[center.x,center.z],"local":[position[0],position[1],z],"inverseX":inverse.transform_vector3(glam::Vec3::X).to_array(),"inverseZ":inverse.transform_vector3(glam::Vec3::Z).to_array(),"locked":attrs.locked || !inverse.is_finite(),"color":attrs.label_color}));
        }
        let eye=camera.eye-target;
        Ok(json!({"items":items,"halfFov":(camera.vertical_fov_radians*0.5).tan()*camera.aspect_ratio,
            "camera":{"point":[eye.x,eye.z],"layer":camera_layer.map(|l|l.0),"target":target_layer.map(|l|l.0),"orbit":seen.orbit_degrees,"distance":seen.distance_scale,"baseDistance":crate::doc::core::distance_from_camera(comp,0.0)}}))
    }
    /// 注視の球: 層の world 中心と、局所 bounds の 8 角を包む半径(rerun `focus_entity` の bounding sphere)。
    pub(crate) fn focus_sphere(&self,id:LayerId)->Result<Option<(glam::Vec3,f32)>,String>{
        let view=self.doc.view();let time=self.time()?;
        let resolved=view.resolved_layers(time).map_err(e)?;
        let Some(world)=view.world_transforms3d(time).map_err(e)?.get(&id).copied() else{return Ok(None)};
        let Some(b)=self.engine.selected_layer_bounds_in(&view,&resolved,id,time) else{return Ok(None)};
        let centre=world.transform_point3(glam::Vec3::from(b.center()));
        let radius=(0..8).map(|i|world.transform_point3(glam::vec3(if i&1==0{b.min[0]}else{b.max[0]},if i&2==0{b.min[1]}else{b.max[1]},if i&4==0{b.min[2]}else{b.max[2]})).distance(centre)).fold(0.0,f32::max);
        Ok(Some((centre,radius)))
    }
    /// 観測者の画面へ world 点を写す。カメラの後ろは None。
    fn observer_screen(&self)->Result<impl Fn(glam::Vec3)->Option<[f32;2]>,String>{
        let view=self.doc.view();let comp=view.composition().map_err(e)?.ok_or("No composition")?.spec();
        let observer=crate::doc::core::camera_projection(comp,self.user_camera);
        let matrix=observer.projection_matrix()*observer.view_matrix();
        Ok(move|p:glam::Vec3|{let c=matrix*p.extend(1.0);if c.w<=0.0{return None}Some([(c.x/c.w+1.0)*0.5*comp.width as f32,(1.0-c.y/c.w)*0.5*comp.height as f32])})
    }
    /// Stage の観測者: 正面か、注視点、comp 面の 4 角(Stage の comp 画像 px)。
    fn observer_status(&self)->Result<Json,String>{
        let comp=self.doc.view().composition().map_err(e)?.ok_or("No composition")?.spec();
        let screen=self.observer_screen()?;
        let (w,h)=(comp.width as f32,comp.height as f32);
        let quad=|[x,y,w,h]:[f32;4]|{let q:Vec<_>=[glam::vec3(x,y,0.0),glam::vec3(x+w,y,0.0),glam::vec3(x+w,y+h,0.0),glam::vec3(x,y+h,0.0)].into_iter().map(&screen).collect();if q.iter().all(Option::is_some){json!(q)}else{Json::Null}};
        let extent=self.doc.view().resolve_stage_extent(self.time()?).map_err(e)?;
        Ok(json!({"front":self.user_camera.orbit_degrees==[0.0;2],"home":self.user_camera==Default::default(),"scale":self.user_camera.distance_scale,"orbit":self.user_camera.orbit_degrees,"target":screen(self.user_camera.target(comp)),"frame":quad([0.0,0.0,w,h]),
            "extent":extent.layer.map(|id|json!({"layer":id.0,"margins":extent.margins,"rect":extent.rect(comp),"points":quad(extent.rect(comp))}))}))
    }
    /// 3D 層の 3 軸ギズモ。頂点は comp 座標 —— Stage は掴む所も描く所も同じ写像で扱う。
    /// 3D 層を選んでいない時は Null。2D・2.5D の平面ケージはここを通らない。
    pub(crate) fn spatial_gizmo(&self,seen:View)->Result<Json,String>{
        let view=self.doc.view();let time=self.time()?;
        let Some(comp)=view.composition().map_err(e)? else{return Ok(Json::Null)};
        let Ok(targets)=editor::gizmo3d::spatial_targets(&view,&self.selected_ids,time) else{return Ok(Json::Null)};
        let pointer=self.stage_pointer.filter(|_|self.stage_view==seen);
        let Some(data)=editor::gizmo3d::draw_data(comp.spec(),self.view_camera(seen)?,&targets,pointer,self.stage_view_scale,self.stage_held.as_deref()) else{return Ok(Json::Null)};
        Ok(json!({"vertices":data.vertices,"colors":data.colors,"indices":data.indices}))
    }
    /// Stage に置いた箱(Boxcam): Camera 層ごとの frustum と取っ手。Stage の comp 画像 px。
    fn camera_gizmos(&self)->Result<Json,String>{
        let view=self.doc.view();let time=self.time()?;let comp=view.composition().map_err(e)?.ok_or("No composition")?.spec();
        let screen=self.observer_screen()?;
        let resolved=view.resolved_layers(time).map_err(e)?;
        let mut gizmos=Vec::new();
        for id in view.layers(){
            let Some(meta)=view.meta(id).map_err(e)? else{continue};
            if meta.source!=LayerSource::Camera || view.attrs(id).map_err(e)?.unwrap_or_default().hidden || !meta.timing.covers(self.frame){continue}
            let camera=self.engine.camera_of_layer_in(&view,&resolved,id,time).map_err(e)?;
            let projection=crate::doc::core::camera_projection(comp,camera);
            let rotation=projection.rotation.inverse();
            // 箱 = カメラが comp 面(z=0)のどこを見ているか: frustum の 4 本の稜線と comp 面の交わり。
            // 正面(orbit 0・層ターゲット無し)ではそれが枠と同じ形の箱で、辺・角・取っ手で author できる。
            // 回ったカメラでは面の上の台形になる。稜線が面へ届かない時だけ注視距離の角で代える。
            let depth=crate::doc::core::distance_from_camera(comp,0.0)*camera.distance_scale;
            let height=depth*(projection.vertical_fov_radians*0.5).tan();let width=height*projection.aspect_ratio;
            let corners=[glam::vec3(-width,-height,-depth),glam::vec3(width,-height,-depth),glam::vec3(width,height,-depth),glam::vec3(-width,height,-depth)];
            let eye=projection.eye;
            let corners:Vec<_>=corners.into_iter().map(|p|{
                let ray=rotation*p;
                let t=if ray.z.abs()>1e-6{-eye.z/ray.z}else{-1.0};
                screen(if t>0.0{eye+ray*t}else{eye+ray})
            }).collect();
            // 箱は注視面の 4 角が写れば描く。eye は観測者と同じ深さか後ろに居るのが普通で(正面の観測者は
            // 既定のカメラと同じ距離に立つ)、写せない。eye と frustum の線は写せた時だけの飾り。
            let eye=screen(eye);
            let authorable=camera.orbit_degrees==[0.0;2] && view.camera_target_layer(id,time).map_err(e)?.is_none();
            // 届かない角は null。2 角以上写れば箱を出し、Flutter は写った角だけ結ぶ。author できるのは 4 角揃った時だけ。
            let seen=corners.iter().filter(|c|c.is_some()).count();
            if seen>=2{gizmos.push(json!({"id":id.0,"points":corners,"eye":eye,"target":screen(camera.target(comp)),"authorable":authorable&&seen==4,"center":camera.center,"zoom":camera.zoom,"roll":camera.roll_degrees}));}
        }
        Ok(json!(gizmos))
    }

    /// 出力(Camera)で見た枠。anchor など view を問わない用途。
    pub(crate) fn bounds(&self,layer:LayerId)->Option<Json>{ self.bounds_seen(layer,View::Camera) }
    pub(crate) fn bounds_seen(&self,layer:LayerId,seen:View)->Option<Json>{
        let resolved=self.doc.view().resolved_layers(self.time().ok()?).ok()?;
        self.bounds_from(&self.eye(seen)?,resolved.as_slice(),layer,seen)
    }
    /// 見ている姿勢 —— comp・作中カメラ・その view の観測者。層ごとに解き直さず、1 フレームに 1 回だけ組む。
    fn eye(&self,seen:View)->Option<Eye>{
        let view=self.doc.view();let time=self.time().ok()?;
        Some(Eye{time,comp:view.composition().ok()??.spec(),camera:self.projection_camera(seen,LayerProjection::ThreeD).ok()?,observer:self.view_camera(seen).ok()?,document:self.engine.resolve_camera(&view,time).ok()?})
    }
    /// 描いた直後に GPU の mask から届いた範囲を、窓の px から comp 画像の px へ戻して取り込む。選択が変わるまで使う。
    pub(crate) fn take_selection_bounds(&mut self,seen:View,window:crate::render::engine::Window){
        if let Some(found)=self.engine.take_selection_bounds(){
            let [x,y,w,h]=window.roi;let (sx,sy)=(w/window.width.max(1) as f32,h/window.height.max(1) as f32);
            self.selection_bounds.insert(seen,found.into_iter().map(|(id,[x0,y0,x1,y1])|(id,[x+x0*sx,y+y0*sy,x+x1*sx,y+y1*sy])).collect());
        }
    }
    fn bounds_from(&self,eye:&Eye,resolved:&[crate::doc::store::ResolvedLayer],layer:LayerId,seen:View)->Option<Json>{
        let view=self.doc.view();let Eye{time,comp,camera,observer,document}=*eye;
        let r=resolved.iter().find(|r|r.id==layer&&!r.ghost)?;
        // 2D は箱に貼り付いているので、どの view でも作中カメラで置く。
        let camera=if r.projection==LayerProjection::TwoD{document}else{camera};
        let b=self.engine.selected_layer_bounds_in(&view,resolved,layer,time)?;
        let world=crate::doc::core::depth_scaled(r.placement.world_transform?);
        // 選ばれた層は絵そのもの(mask)の広がり: 透視・effect・変位込みで 1 px。届く前は写した点で包む。
        // 2D・2.5D の籠は向きを持たない: こちらを向いた矩形。向きは Inspector が決める。
        let corners:Vec<_>=match (self.selection_bounds.get(&seen).and_then(|m|m.get(&layer)),r.projection){
            (Some(&[x0,y0,x1,y1]),_)=>vec![[x0 as f64,y0 as f64],[x1 as f64,y0 as f64],[x1 as f64,y1 as f64],[x0 as f64,y1 as f64]],
            (None,projection)=>match projection{
            LayerProjection::ThreeD=>crate::doc::core::projected_screen_corners(comp,camera,observer,r.projection,world,b.min,b.max).iter().map(|p|[p.x as f64,p.y as f64]).collect(),
            _=>{
                let outline=self.engine.selected_layer_outline_in(&view,resolved,layer,time).unwrap_or_else(||b.corners().to_vec());
                crate::doc::core::facing_frame(comp,camera,observer,r.projection,world,b.min,b.max,&outline).iter().map(|p|[p.x as f64,p.y as f64]).collect()
            }
        }};
        let anchor=match view.value_at(layer,&PropertyId::new(property::ANCHOR).ok()?,time).ok().flatten(){Some(Value::Vec2(v))=>v,_=>[0.0,0.0]};
        let fractions:[f64;2]=std::array::from_fn(|i|(anchor[i]-b.min[i]as f64)/(b.max[i]-b.min[i]).max(1e-6)as f64);
        Some(json!({"layer":layer.0,"corners":corners,"localMin":b.min,"localMax":b.max,"anchorFraction":fractions}))
    }
    /// 今の姿 —— 観測者と時刻で動く物。cache した行の上へ毎回これを載せる。
    /// `bounds` は Camera(出力)、`stageBounds` は Stage(観測者)で見た枠。
    fn overlay_geometry(&self,row:&mut Json,eyes:&(Eye,Eye),resolved:&[crate::doc::store::ResolvedLayer],id:LayerId,live:bool)->Result<(),String>{
        let position=self.position(id)?;let bounds=self.bounds_from(&eyes.0,resolved,id,View::Camera);
        row["corners"]=json!(bounds.as_ref().and_then(|b|b["corners"].as_array()).map(|c|if c.len()==8{vec![c[0].clone(),c[1].clone(),c[3].clone(),c[2].clone()]}else{c.clone()}));
        row["x"]=json!(position[0]);row["y"]=json!(position[1]);
        if !live {row["anchorFraction"]=bounds.as_ref().map(|b|b["anchorFraction"].clone()).unwrap_or(Json::Null);}
        row["bounds"]=json!(bounds);
        row["stageBounds"]=json!(self.bounds_from(&eyes.1,resolved,id,View::User));
        Ok(())
    }
    pub(crate) fn build_status(&self)->Result<Json,String>{
        let view=self.doc.view();let comp=view.composition().map_err(e)?.ok_or("No composition")?;let at=self.time()?;
        let catalog=crate::render::engine::known_effects();
        let resolved=view.resolved_layers(at).map_err(e)?;
        let clipping=view.clipping_bases().map_err(e)?;
        let eyes=(self.eye(View::Camera).ok_or("No composition")?,self.eye(View::User).ok_or("No composition")?);
        // 再生中で Document が変わっていなければ、時刻で変わる物(値・枠)だけの軽い status にする。
        let revision=format!("{:?}",self.doc.revision());
        let live=self.clock.playing()&&self.full_status_revision.borrow().as_deref()==Some(revision.as_str());
        let mut ids=view.layers();ids.sort_by_key(|id|std::cmp::Reverse((view.meta(*id).ok().flatten().map_or(0,|m|m.order),id.0)));
        let mut layers=Vec::new();
        let keys=self.layer_keys(&view,at,&resolved,&clipping,live)?;
        for id in ids{
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
            if let Some(mut row)=row {self.overlay_geometry(&mut row,&eyes,&resolved,id,live)?;layers.push(row);}
        }
        self.snapshot_cache.borrow_mut().rows.retain(|id,_|keys.contains_key(id));
        if live {
            let (undo,redo)=self.doc.history_depth();
            let selected_keys:Vec<_>=self.selected_keys.iter().map(|k|json!({"layer":k.layer.0,"property":k.property.as_ref().map(|p|p.name()),"frame":(k.at_sec*comp.fps.as_f64()).round()as i64})).collect();
            let point=self.selected.map(|id|self.position(id)).transpose()?.unwrap_or([0.0,0.0]);
            let live_layers=layers;
            // 寸法は Swift の render が毎コマ読む。軽い status でも落とさない(落とすと再生 2 コマ目で render が失敗し、再生が止まる)。
            return Ok(json!({"frame":self.frame,"playing":self.clock.playing(),"documentRevision":revision,"preview":self.preview.is_some(),"previewOwner":self.preview.as_ref().map(|p|p.0),"previewInteraction":self.preview_tag,"undo":undo,"redo":redo,"width":comp.width,"height":comp.height,"fps":comp.fps.as_f64(),"durationFrames":comp.duration_frames,
                "selectedId":self.selected.map(|s|s.0),"selectedIds":self.selected_ids.iter().map(|s|s.0).collect::<Vec<_>>(),"selectedKeys":selected_keys,"x":point[0],"y":point[1],
                "animate":self.animate!=Animate::Off,"renderCount":self.render_count,"pickedColor":self.picked_color,"pickSerial":self.pick_serial,"renderMs":self.render_ms,"liveLayers":live_layers}));
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
        let playback_health=match self.clock.health(){
            editor::playback::PlaybackHealth::AudioReady=>json!({"kind":"AudioReady","reason":null,"detail":null}),
            editor::playback::PlaybackHealth::VisualOnly(reason)=>match reason{
                editor::playback::VisualFallback::NoAudio=>json!({"kind":"VisualOnly","reason":"NoAudio","detail":null}),
                editor::playback::VisualFallback::Program(detail)=>json!({"kind":"VisualOnly","reason":"Program","detail":detail}),
                editor::playback::VisualFallback::Device(detail)=>json!({"kind":"VisualOnly","reason":"Device","detail":detail}),
            }
        };
        let waveforms:Vec<_>=self.clock.waveform_tracks().iter().map(|track|json!({"layer":track.layer.0,"columns":track.columns(0.0,self.clock.duration(),256.0/self.clock.duration().max(0.01)).unwrap_or_default().iter().map(|c|json!({"frame":c.at_sec*comp.fps.as_f64(),"min":c.min,"max":c.max})).collect::<Vec<_>>()})).collect();
        let (undo,redo)=self.doc.history_depth();let point=self.selected.map(|id|self.position(id)).transpose()?.unwrap_or([0.0,0.0]);
        let color_target=self.color_target.as_ref().and_then(|slot|editor::color::read_color(&self.doc,slot).map(|rgba|json!({"layer":slot.layer().0,"slot":slot,"label":"Color","rgba":rgba})));
        let selected_keys:Vec<_>=self.selected_keys.iter().map(|k|json!({"layer":k.layer.0,"property":k.property.as_ref().map(|p|p.name()),"frame":(k.at_sec*comp.fps.as_f64()).round()as i64})).collect();
        let generation=crate::render::engine::catalog_generation();
        let catalog_rows=catalog.iter().map(|e|json!({"id":e.plugin_id,"name":e.label,"stage":format!("{:?}",e.stage),"generation":generation})).collect::<Vec<_>>();
        let mut status=json!({"observer":self.observer_status()?,"cameraGizmos":self.camera_gizmos()?,"width":comp.width,"height":comp.height,"fps":comp.fps.as_f64(),"fpsNum":comp.fps.num(),"fpsDen":comp.fps.den(),"durationFrames":comp.duration_frames,"background":comp.background,"frame":self.frame,"playing":self.clock.playing(),"playbackHealth":playback_health,"waveforms":waveforms,"undo":undo,"redo":redo,"path":self.path,"dirty":self.is_dirty()?,"layers":layers,"selectedId":self.selected.map(|id|id.0),"selectedIds":self.selected_ids.iter().map(|id|id.0).collect::<Vec<_>>(),"selectedKeys":selected_keys,"selectedBounds":self.selected.and_then(|id|self.bounds(id)),"x":point[0],"y":point[1],"assets":assets?,"catalog":catalog_rows,"palette":palette,"markers":markers?,"colorTarget":color_target,"capabilities":crate::port::CAPABILITIES,"easeKinds":if self.clock.playing(){Json::Null}else{json!(editor::ease_kinds::KINDS.iter().copied().map(interp).collect::<Vec<_>>())},"documentRevision":format!("{:?}",self.doc.revision()),"deviceId":self.device_id.to_string(),"renderCount":self.render_count,"renderMs":self.render_ms,"interopCopies":0,"readbacks":0,"error":self.error,"preview":self.preview.is_some(),"export":self.exporter.status()});
        status["previewOwner"] = json!(self.preview.as_ref().map(|p|p.0));
        status["previewInteraction"] = json!(self.preview_tag);
        status["visualSamples"]=json!(true);
        status["fontFamilies"]=json!(crate::doc::vector::text::font_families());
        status["history"]=self.history.snapshot(self.doc.edit_head());
        status["spatialGizmo"]=self.spatial_gizmo(View::Camera)?;
        status["stageSpatialGizmo"]=self.spatial_gizmo(View::User)?;
        status["notebook"]=serde_json::to_value(view.notebook().map_err(e)?).map_err(e)?;
        status["depthLayout"]=self.depth_layout(&resolved)?;
        status["backgrounds"]=json!(editor::create::backgrounds().iter().map(|b|json!({"id":b.id,"name":b.name,"thumbnail":editor::thumbnail::image_data_uri(&b.path)})).collect::<Vec<_>>());
        status["primitives"]=json!(editor::create::primitives().iter().map(|p|json!({"id":p.id,"name":p.name})).collect::<Vec<_>>());
        status["animate"]=json!(self.animate!=Animate::Off);
        if status["easeKinds"].is_null(){status.as_object_mut().unwrap().remove("easeKinds");}
        status["importExtensions"]=json!(crate::render::media::import_extensions());
        Ok(status)
    }
    #[allow(clippy::too_many_arguments)]
    fn layer_json(&self,view:&StoreView<'_>,id:LayerId,at:RationalTime,fps:Fps,catalog:&[crate::render::engine::EffectDescriptor],clipping:&std::collections::HashMap<LayerId,Option<LayerId>>,live:bool)->Result<Option<Json>,String>{
        let attrs=view.attrs(id).map_err(e)?.unwrap_or_default();let Some(meta)=view.meta(id).map_err(e)? else{return Ok(None)};
        let data=editor::functions::read::inspector_data_from_doc(view,id,at,catalog);
        let mut properties=Vec::new();let mut seen=std::collections::BTreeSet::new();
        for row in data.transform.iter().chain(data.text.iter()){
            if let Some(p)=&row.property{
                if seen.insert(p.clone()){properties.push(prop(view,id,p,&row.label,&row.value,row.range,at,fps,live)?);}
            }
            for (index,axis) in row.axis.iter().enumerate(){if let Some((p,v))=axis{
                if seen.insert(p.clone()){properties.push(prop(view,id,p,&format!("{} {}",row.label,["X","Y","Z"][index]),v,row.range,at,fps,live)?);}
            }}
        }
        for p in view.properties(id){if !p.name().starts_with(property::EFFECT_PREFIX)&&seen.insert(p.name().into()){
            if let Some(v)=view.value_at(id,&p,at).map_err(e)?{properties.push(prop(view,id,p.name(),p.name(),&v,None,at,fps,live)?);}
        }}
        if meta.source == LayerSource::Camera {
            properties = property::CAMERA_ROWS.iter().map(|(p,label,v,range)| prop(view,id,p,label,v,*range,at,fps,live)).collect::<Result<Vec<_>,_>>()?;
        }
        if meta.source == LayerSource::Stage {
            properties = property::STAGE_MARGINS.iter().zip(["Left","Top","Right","Bottom"])
                .map(|(p,label)| prop(view,id,p,label,&Value::F64(0.0),Some((0.0,100000.0)),at,fps,live)).collect::<Result<Vec<_>,_>>()?;
        }
        let text=if let Some(t)=view.text_document(id).map_err(e)?{
            let keyed=t.content.keys().iter().any(|k|k.t.try_to_frame_round(fps).ok()==Some(self.frame));
            let content=if live { json!({"id":"content","value":t.content.eval(at),"keyedNow":keyed}) } else {
                let keys:Result<Vec<_>,String>=t.content.keys().iter().map(|k|Ok(json!({"frame":k.t.try_to_frame_round(fps).map_err(e)?,"value":k.content,"interp":{"kind":"Hold"}}))).collect();
                json!({"id":"content","label":"Content","kind":"text","value":t.content.eval(at),"min":null,"max":null,"keyedNow":keyed,"keys":keys?})
            };
            properties.insert(0,content);
            let resolved=view.resolved_text_document(id,at).map_err(e)?.unwrap_or(t.clone());
            properties.retain(|p|p["id"]!="text_justify");
            let mut justify=prop(view,id,"text_justify","Alignment",&Value::Enum(resolved.justify.to_enum_value()),None,at,fps,live)?;
            justify["choices"]=json!(["Left","Right","Center"]);properties.push(justify);
            let s=resolved.styles.first();
            json!({"classes":crate::doc::store::text_edit::classifications(t.content.eval(at)),"styles":resolved.styles,"runs":resolved.runs,"fontFamily":s.map(|s|&s.font.family),"content":t.content.eval(at),"size":s.map(|s|s.size),"lineHeight":s.and_then(|s|s.line_height),"tracking":s.map(|s|s.tracking)})
        }else{Json::Null};
        let colors:Vec<_>=data.colors.iter().filter(|_|!live).map(|c|json!({"label":c.label,"slot":c.slot,"rgba":editor::color::read_color(&self.doc,&c.slot).unwrap_or([0.0,0.0,0.0,1.0])})).collect();
        let effects:Result<Vec<_>,String>=data.effects.iter().map(|effect|{
            let kind=crate::doc::store::placement::kind(&effect.plugin_id);
            let mut params:Vec<Json>=effect.params.iter().filter_map(|p|p.property.as_ref().map(|idp|prop(view,id,idp,&p.label,&p.value,p.range,at,fps,live))).collect::<Result<_,_>>()?;
            if live { return Ok(json!({"id":effect.id,"params":params})); }
            for row in &mut params{
                let name=row["id"].as_str().unwrap_or_default().rsplit(".param.").next().unwrap_or_default().to_owned();
                if let Some(param)=crate::doc::store::kind::kind(&effect.plugin_id).and_then(|k|k.params.iter().find(|p|p.name==name)){
                    if !param.section.is_empty(){row["section"]=json!(param.section);}
                }
                if let Some(p)=catalog.iter().find(|d|d.plugin_id==effect.plugin_id).and_then(|d|d.params.iter().find(|p|p.name==name)){
                    if let Some(choices)=p.choices.clone(){row["choices"]=json!(choices);}
                    if let Some(s)=&p.subtype{row["subtype"]=json!(s);}
                    if let Some(u)=&p.unit{row["unit"]=json!(u);}
                    if let Some(g)=&p.group{row["group"]=json!(format!("effect.{}.param.{}",effect.id,g));}
                    if p.advanced{row["advanced"]=json!(true);}
                    if p.hero{row["hero"]=json!(true);}
                    row["default"]=json!(p.default);
                }
            }
            let layout=kind.map(|k|{
                let pid=|name:&str|format!("effect.{}.param.{}",effect.id,name);
                let shown=|name:&str|params.iter().any(|r|r["id"]==pid(name));
                let share=format!("effect.{}.param.{}",effect.id,crate::doc::store::placement::SHARE_PREFIX);
                json!({"columns":["Each","Random"],"count":pid("count"),"along":pid("mode"),"pick":pid("pick"),"seed":pid("seed"),
                    "materials":params.iter().filter(|r|r["id"].as_str().is_some_and(|i|i.starts_with(&share))).map(|r|json!({"id":r["id"],"label":r["label"]})).collect::<Vec<_>>(),
                    "shape":k.shape.iter().filter(|n|shown(n)).map(|n|json!({"id":pid(n),"label":k.params.iter().find(|p|p.name==*n).map_or(*n,|p|p.label),"unit":crate::doc::store::placement::unit(n)})).collect::<Vec<_>>(),
                    "rows":k.grid.iter().map(|r|json!({"label":r.label,"unit":r.unit,"each":r.each.map(pid),"random":r.random.map(pid),"axis":r.axis,"advanced":r.advanced})).collect::<Vec<_>>()})
            });
            let enabled=!matches!(view.value_at(id,&PropertyId::effect_enabled(EffectId(effect.id)),at).map_err(e)?,Some(Value::Bool(false)));let whole=matches!(view.value_at(id,&PropertyId::effect_scope(EffectId(effect.id)),at).map_err(e)?,Some(Value::Enum(v)) if v==crate::doc::store::EffectScope::Whole.enum_value());Ok(json!({"id":effect.id,"enabled":enabled,"whole":whole,"pluginId":effect.plugin_id,"name":catalog.iter().find(|d|d.plugin_id==effect.plugin_id).map_or(effect.plugin_id.as_str(),|d|d.label.as_str()),"placement":kind.is_some(),"layout":layout,"params":params}))
        }).collect();
        if live {
            let mut row=json!({"id":id.0,"text":text,"properties":properties,"effects":effects?});
            if let Some(color)=data.colors.first(){row["fill"]=editor::gradient::model(&self.doc,&color.slot).unwrap_or(Json::Null);}
            return Ok(Some(row));
        }
        let content_keys:Vec<_>=properties.iter().find(|p|p["id"]=="content").and_then(|p|p["keys"].as_array()).into_iter().flatten().map(|k|json!({"frame":k["frame"],"content":k["value"]})).collect();
        let mut row=json!({"id":id.0,"name":attrs.name,"kind":source_kind(&meta.source),"ghost":attrs.ghost,"ghostable":crate::editor::timeline_edit::ghostable(view,id),"parent":attrs.parent.map(|p|p.0),"order":meta.order,"hidden":attrs.hidden,"solo":attrs.solo,"blocksLight":attrs.blocks_light,"locked":attrs.locked,"clipToBelow":attrs.clip_to_below,"clipBase":clipping.get(&id).copied().flatten().map(|b|b.0),"projection":match attrs.projection{LayerProjection::TwoD=>"2D",LayerProjection::TwoPointFiveD=>"2.5D",LayerProjection::ThreeD=>"3D"},"flatten":attrs.flatten,"environment":attrs.environment,"frozen":attrs.frozen,"blendMode":attrs.blend_mode,"matte":attrs.matte,"start":meta.timing.start,"duration":meta.timing.duration,"sourceIn":meta.timing.source_in,"properties":properties,"text":text,"colors":colors,"effects":effects?,"contentKeys":content_keys});
        if let Some(color)=data.colors.first(){row["fill"]=editor::gradient::model(&self.doc,&color.slot).unwrap_or(Json::Null);}
        Ok(Some(row))
    }
}

#[cfg(test)]
mod frame_cost_probe {
    use crate::doc::store::*;

    fn runtime(layers: u32, copies: f64) -> crate::EditorRuntime {
        let mut rt = crate::EditorRuntime::open("").unwrap();
        for i in 0..layers {
            let layer = LayerId(100 + u64::from(i));
            let repeat = EffectId(0);
            rt.doc.apply_all([
                Intent::AddLayer(layer),
                Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Shape, order: i as i16, timing: LayerTiming::place(0, None, 1800) } },
                Intent::SetShapes { layer, shapes: vec![rect_shape([200, 80, 40, 255], [60.0, 60.0])] },
                Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([100.0 + 40.0 * f64::from(i), 100.0 + 150.0 * f64::from(i)]) },
                Intent::SetEffects { layer, effects: vec![EffectInstance { id: repeat, plugin_id: placement::REPEAT.to_owned() }] },
                Intent::SetConstant { layer, property: PropertyId::effect_param(repeat, "count").unwrap(), value: Value::F64(copies) },
                Intent::SetConstant { layer, property: PropertyId::effect_param(repeat, "position_each").unwrap(), value: Value::Vec2([90.0, 0.0]) },
            ]).unwrap();
        }
        rt
    }

    #[test]
    fn playback_values_match_full_status_without_keyframe_metadata() {
        let mut rt = runtime(2, 1.0);
        let p = PropertyId::new(property::OPACITY).unwrap();
        let mut track = KeyframeTrack::new();
        for frame in [0, 10] {
            track.insert(Keyframe { t: RationalTime::try_from_frame(frame, rt.doc.view().composition().unwrap().unwrap().fps).unwrap(), value: Value::F64(frame as f64 / 10.0), interp: Interp::Linear, spatial: None });
        }
        rt.doc.apply(Intent::SetTrack { layer: LayerId(100), property: p, track }).unwrap();
        for frame in [0, 5, 10] {
            rt.frame = frame;
            let full = rt.status().unwrap();
            rt.clock.toggle();
            let live = rt.status().unwrap();
            rt.clock.toggle();
            for (expected, actual) in full["layers"].as_array().unwrap().iter().zip(live["liveLayers"].as_array().unwrap()) {
                for field in ["id", "x", "y", "bounds", "corners", "text"] { assert_eq!(expected[field], actual[field], "{field}"); }
                let compare = |a: &serde_json::Value, b: &serde_json::Value| {
                    assert_eq!(a.as_array().unwrap().len(), b.as_array().unwrap().len());
                    for (a,b) in a.as_array().unwrap().iter().zip(b.as_array().unwrap()) {
                        for field in ["id", "value", "keyedNow"] { assert_eq!(a[field],b[field], "{field}"); }
                        assert!(b.get("keys").is_none());
                    }
                };
                compare(&expected["properties"], &actual["properties"]);
                for (a,b) in expected["effects"].as_array().unwrap().iter().zip(actual["effects"].as_array().unwrap()) { compare(&a["params"], &b["params"]); }
            }
        }
        let before = rt.full_status_revision.borrow().clone();
        let reply = unsafe { crate::motolii_probe_request(&mut rt, c"{\"op\":\"renderInfo\"}".as_ptr()) };
        let info: serde_json::Value = serde_json::from_str(unsafe { std::ffi::CStr::from_ptr(reply) }.to_str().unwrap()).unwrap();
        assert_eq!(info.as_object().unwrap().len(), 3, "width, height, views");
        assert_eq!(info["width"], rt.status().unwrap()["width"]);
        assert_eq!(*rt.full_status_revision.borrow(), before);
        rt.request(serde_json::json!({"op":"composition","width":640,"height":480})).unwrap();
        let reply = unsafe { crate::motolii_probe_request(&mut rt, c"{\"op\":\"renderInfo\"}".as_ptr()) };
        let info: serde_json::Value = serde_json::from_str(unsafe { std::ffi::CStr::from_ptr(reply) }.to_str().unwrap()).unwrap();
        assert_eq!(info, serde_json::json!({"width":640,"height":480,"views":[{"view":"Camera","width":640,"height":480}]}));
    }

    #[test]
    #[ignore]
    fn probe() {
        for (layers, copies) in [(5, 1.0), (5, 30.0), (5, 100.0)] {
            let mut rt = runtime(layers, copies);
            let t = RationalTime::ZERO;
            let time = |f: &mut dyn FnMut()| { let s = std::time::Instant::now(); for _ in 0..5 { f(); } s.elapsed() / 5 };
            let resolve = time(&mut || { rt.doc.view().resolved_layers(t).unwrap(); });
            let status = time(&mut || { rt.status().unwrap(); });
            let status_bytes = rt.status().unwrap().to_string().len();
            *rt.full_status_revision.borrow_mut() = Some(format!("{:?}", rt.doc.revision()));
            rt.clock.toggle();
            let live = time(&mut || { rt.status().unwrap(); });
            let live_bytes = rt.status().unwrap().to_string().len();
            rt.clock.toggle();
            eprintln!("  live status={live:?} ({live_bytes} bytes)");
            let sig = time(&mut || { crate::snapshot::authored_signature(&rt.doc).unwrap(); });
            let depth = time(&mut || { let r = rt.doc.view().resolved_layers(t).unwrap(); rt.depth_layout(&r).unwrap(); });
            let catalog = crate::render::engine::known_effects();
            let inspector = time(&mut || { for id in rt.doc.view().layers() { crate::editor::functions::read::inspector_data_from_doc(&rt.doc.view(), id, t, &catalog); } });
            eprintln!("  signature={sig:?} depth={depth:?} inspector={inspector:?}");
            if copies == 1.0 {
                let status = rt.status().unwrap();
                let mut top: Vec<(usize, String)> = status.as_object().unwrap().iter().map(|(k, v)| (v.to_string().len(), k.clone())).collect();
                top.sort(); top.reverse();
                eprintln!("  top-level: {:?}", &top[..top.len().min(8)]);
                if let Some(layer) = status["layers"].as_array().and_then(|l| l.first()) {
                    let mut fields: Vec<(usize, String)> = layer.as_object().unwrap().iter().map(|(k, v)| (v.to_string().len(), k.clone())).collect();
                    fields.sort(); fields.reverse();
                    eprintln!("  one layer: {:?}", &fields[..fields.len().min(8)]);
                }
            }
            let render = time(&mut || { rt.engine.render_frame_without_background(&rt.doc.view(), t).unwrap(); });
            eprintln!("layers={layers} copies={copies:4}: resolve={resolve:?} status={status:?} ({status_bytes} bytes) render={render:?}");
        }
    }
}

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
    /// 層ターゲットは Stage の枠・Depth の点と同じ「bounds の中心」を見る。anchor をずらしても注視点は形の中心に残る。
    #[test]
    fn a_layer_target_aims_at_the_bounds_centre_even_when_the_anchor_moves(){
        let mut rt=EditorRuntime::open("").unwrap();
        request(&mut rt,json!({"op":"create","kind":"rectangle"}));
        let shape=rt.selected.unwrap();
        request(&mut rt,json!({"op":"create","kind":"camera"}));
        let camera=rt.selected.unwrap();
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
        let authored=view.resolve_camera(time).unwrap();
        assert!((seen.center[0]-authored.center[0]).abs()>100.0,"Document alone looks at the anchor; the engine looks at the shape");
    }
}

#[cfg(test)]
mod camera_view_cage_tests {
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
        let text=rt.selected.unwrap();
        request(&mut rt,json!({"op":"setText","layer":text.0,"content":"Motolii"}));
        request(&mut rt,json!({"op":"styleText","layer":text.0,"scope":"all","size":250.0}));
        request(&mut rt,json!({"op":"setProperty","layer":text.0,"property":"position","value":[109.0,263.0]}));
        let comp=rt.doc.view().composition().unwrap().unwrap().spec();
        for (projection,orbit) in [("2.5D",[0.0,0.0]),("2.5D",[-20.0,40.0]),("2.5D",[35.0,-120.0]),("2D",[-20.0,40.0])] {
            request(&mut rt,json!({"op":"setAttrs","layers":[text.0],"patch":{"projection":projection}}));
            rt.user_camera=crate::doc::core::ResolvedCamera{orbit_degrees:orbit,..Default::default()};
            let time=rt.time().unwrap();
            let pixels=rt.engine.render_with_camera_override(&rt.doc.view(),time,true,Some(rt.user_camera)).unwrap();
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
        let text=rt.selected.unwrap();
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
        let b=rt.selection_bounds[&View::User].get(&text).copied().expect("the text is drawn in the Stage window");
        assert!(b[0]<0.0&&b[2]<0.0&&b[2]>b[0],"the cage sits left of the frame in comp px: {b:?}");
        assert!(b[1]>0.0&&b[3]<h,"the cage keeps the comp's vertical placement: {b:?}");
        let output=make(&rt,comp.width,comp.height);
        rt.render_into(&output,View::Camera,crate::render::engine::Window::output(comp)).unwrap();
        assert!(rt.selection_bounds[&View::Camera].get(&text).is_none_or(|b|b[2]<=b[0]),"the output never shows what lies outside the frame");
        let status=rt.build_status().unwrap();
        let row=status["layers"].as_array().unwrap().iter().find(|l|l["id"]==text.0).unwrap();
        assert!(row["stageBounds"]["corners"].is_array(),"the Stage cage rides on the row");
        assert!(status["observer"].is_object()&&status["cameraGizmos"].is_array(),"the Stage chrome is always there");
        // 正面の観測者は既定のカメラと同じ所に立つ。eye は写せなくても、箱(comp 面の 4 角)は必ず出る。
        request(&mut rt,json!({"op":"create","kind":"camera"}));
        let camera=rt.selected.unwrap();
        let gizmos=rt.camera_gizmos().unwrap();
        let g=gizmos.as_array().unwrap().first().expect("the camera box is drawn front-on");
        assert_eq!(g["points"].as_array().unwrap().len(),4,"{g}");
        assert!(g["eye"].is_null(),"the eye sits at the observer and cannot be projected: {g}");
        let corner=|i:usize|[g["points"][i][0].as_f64().unwrap(),g["points"][i][1].as_f64().unwrap()];
        let (xs,ys):(Vec<f64>,Vec<f64>)=(0..4).map(corner).map(|c|(c[0],c[1])).unzip();
        let span=|v:&[f64]|(v.iter().cloned().fold(f64::MAX,f64::min),v.iter().cloned().fold(f64::MIN,f64::max));
        assert!(span(&xs).0.abs()<1.0&&(span(&xs).1-w as f64).abs()<1.0&&span(&ys).0.abs()<1.0&&(span(&ys).1-h as f64).abs()<1.0,"a default camera's box is the frame: {g}");
        // Boxcam: 作中カメラが回っても Stage(Original Comp)は動かない。動くのは箱だけ。
        request(&mut rt,json!({"op":"setProperty","layer":text.0,"property":"position","value":[400.0,300.0]}));
        let still=|rt:&mut EditorRuntime|{
            request(rt,json!({"op":"select","ids":[text.0]}));
            let window=crate::render::engine::Window{projection_camera:Some(Default::default()),..crate::render::engine::Window::output(comp)};
            rt.stage_window=Some(window);
            let stage=make(rt,comp.width,comp.height);
            rt.render_into(&stage,View::User,window).unwrap();
            rt.selection_bounds[&View::User][&text]
        };
        let before=still(&mut rt);
        request(&mut rt,json!({"op":"setProperty","layer":camera.0,"property":"camera.orbit","value":[-20.0,40.0]}));
        let after=still(&mut rt);
        assert!(before.iter().zip(after.iter()).all(|(a,b)|(a-b).abs()<1.0),"the Stage never moves with the camera: {before:?} vs {after:?}");
        let output=make(&rt,comp.width,comp.height);
        rt.render_into(&output,View::Camera,crate::render::engine::Window::output(comp)).unwrap();
        let seen=rt.selection_bounds[&View::Camera][&text];
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
        let mesh=rt.selected.unwrap();
        request(&mut rt,json!({"op":"setProperty","layer":mesh.0,"property":"rotation.y","value":-35.0}));
        request(&mut rt,json!({"op":"create","kind":"camera"}));
        let camera=rt.selected.unwrap();
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
        assert!(rt.selection_bounds[&View::Camera].contains_key(&mesh),"the mask reached the bridge");
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
        let mesh=rt.selected.unwrap();
        request(&mut rt,json!({"op":"setProperty","layer":mesh.0,"property":"rotation.y","value":-20.0}));
        request(&mut rt,json!({"op":"setProperty","layer":mesh.0,"property":"rotation","value":30.0}));
        request(&mut rt,json!({"op":"create","kind":"camera"}));
        let camera=rt.selected.unwrap();
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
            let world=view.world_transform3d(mesh,time).unwrap();
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
        let layer=rt.selected.unwrap();
        request(&mut rt,json!({"op":"setProperty","layer":layer.0,"property":"rotation.y","value":60.0}));
        request(&mut rt,json!({"op":"setProperty","layer":layer.0,"property":"rotation","value":20.0}));
        let comp=rt.doc.view().composition().unwrap().unwrap().spec();
        // 形の bounds は 1 度描いてから測れる。
        let time=rt.time().unwrap();
        rt.engine.render_frame(&rt.doc.view(),time).unwrap();
        for (projection,observer) in [("2D",crate::doc::core::ResolvedCamera::default()),("2.5D",crate::doc::core::ResolvedCamera{orbit_degrees:[-15.0,35.0],distance_scale:1.4,..Default::default()})] {
            request(&mut rt,json!({"op":"setAttrs","layers":[layer.0],"patch":{"projection":projection}}));
            rt.user_camera=observer;
            let time=rt.time().unwrap();
            let bounds=rt.bounds_seen(layer,View::User).unwrap();
            let c:Vec<[f64;2]>=serde_json::from_value(bounds["corners"].clone()).unwrap();
            assert_eq!(c.len(),4,"{projection}: a planar layer gets the 4-corner frame");
            assert!((c[0][1]-c[1][1]).abs()<1e-3&&(c[1][0]-c[2][0]).abs()<1e-3&&(c[2][1]-c[3][1]).abs()<1e-3&&(c[3][0]-c[0][0]).abs()<1e-3,"{projection}: the frame faces the observer: {c:?}");
            assert!(c[0][0]<c[1][0]&&c[0][1]<c[3][1],"{projection}: nw, ne, se, sw: {c:?}");
            let view=rt.doc.view();
            let resolved=view.resolved_layers(time).unwrap();
            let r=resolved.iter().find(|r|r.id==layer).unwrap();
            let b=rt.engine.selected_layer_bounds_in(&view,&resolved,layer,time).unwrap();
            let projected=crate::doc::core::projected_screen_corners(comp,rt.engine.resolve_camera(&view,time).unwrap(),observer,r.projection,crate::doc::core::depth_scaled(r.placement.world_transform.unwrap()),b.min,b.max);
            for p in projected{assert!(c[0][0]-1e-3<=p.x as f64&&p.x as f64<=c[2][0]+1e-3&&c[0][1]-1e-3<=p.y as f64&&p.y as f64<=c[2][1]+1e-3,"{projection}: corner {p:?} outside the frame {c:?}");}
            assert!((c[1][0]-c[0][0])>1.0&&(c[3][1]-c[0][1])>1.0,"{projection}: the frame is not edge-on even though the plane may be");
            // 角を外へ引く → scale が伸びる。
            let se=[c[2][0],c[2][1]];let nw=[c[0][0],c[0][1]];
            let drag=editor::stage::DragSession::begin(&rt.doc,&rt.engine,&[layer],"scale","se",se,time,observer,Default::default(),1.0,None).unwrap();
            let far=[nw[0]+(se[0]-nw[0])*2.0,nw[1]+(se[1]-nw[1])*2.0];
            let edits=drag.edits(&rt.doc,far,false,false,Animate::Off).unwrap();
            let scale=edits.iter().find_map(|i|match i{Intent::SetConstant{property,value:Value::Vec2(v),..} if *property==PropertyId::new(property::SCALE).unwrap()=>Some(*v),_=>None}).expect("scale edit");
            assert!(scale[0]>1.5&&scale[1]>1.5,"{projection}: pulling the corner out grows the layer: {scale:?}");
            // 上の取っ手を画面で 90° 回す → rotation が 90° 動く(相似で結ぶので画面の角度がそのまま値)。
            let top=[(c[0][0]+c[1][0])*0.5,c[0][1]-22.0];
            let drag=editor::stage::DragSession::begin(&rt.doc,&rt.engine,&[layer],"rotate","",top,time,observer,Default::default(),1.0,None).unwrap();
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
}
