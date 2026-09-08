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
fn prop(view:&StoreView<'_>,layer:LayerId,id:&str,label:&str,fallback:&Value,range:Option<(f64,f64)>,at:RationalTime,fps:Fps)->Result<Json,String>{
    let p=PropertyId::new(id).map_err(e)?;
    let current=view.value_at(layer,&p,at).map_err(e)?.unwrap_or_else(||fallback.clone());
    let mut keys=Vec::new();
    if let Some(track)=view.track(layer,&p).map_err(e)?{for key in track.keys(){keys.push(json!({"frame":key.t.try_to_frame_round(fps).map_err(e)?,"value":value(&key.value),"interp":interp(key.interp)}));}}
    let here=at.try_to_frame_round(fps).map_err(e)?;
    Ok(json!({"id":id,"label":label,"kind":match current{Value::F64(_)=>"number",Value::Vec2(_)=>"vec2",Value::Color(_)=>"color",Value::Bool(_)|Value::Enum(_)|Value::LayerId(_)=>"enum",Value::Path(_)=>"text"},"value":value(&current),"min":range.map(|r|r.0),"max":range.map(|r|r.1),"keyedNow":keys.iter().any(|k|k["frame"]==here),"keys":keys}))
}
fn source_kind(source:&LayerSource)->&'static str{match source{
    LayerSource::Camera=>"Camera",LayerSource::Text=>"Text",LayerSource::Shape=>"Shape",LayerSource::Group=>"Group",LayerSource::Null=>"Null",
    LayerSource::File{path,..}=>{
        if crate::render::media::is_mesh_path(path){"Mesh"}else if crate::render::media::is_point_cloud_path(path){"PointCloud"}else{
            let mime=std::path::Path::new(path).extension().and_then(|e|e.to_str()).and_then(crate::render::media::asset_type_for_extension).unwrap_or_default();
            if mime.starts_with("audio/"){"Audio"}else if mime.starts_with("video/"){"Video"}else{"Image"}
        }
    }
}}
impl EditorRuntime{
    pub(crate) fn view_camera(&self)->Result<crate::doc::core::ResolvedCamera,String>{
        if self.user_stage { Ok(self.user_camera) } else { self.doc.view().resolve_camera(self.time()?).map_err(e) }
    }
    fn depth_layout(&self,resolved:&[crate::doc::store::ResolvedLayer])->Result<Json,String>{
        let view=self.doc.view();let time=self.time()?;let comp=view.composition().map_err(e)?.ok_or("No composition")?.spec();
        let camera=crate::doc::core::camera_projection(comp,view.resolve_camera(time).map_err(e)?);
        let worlds=view.world_transforms3d(time).map_err(e)?;
        let mut items=Vec::new();
        for layer in resolved {
            // 配置効果の複製は層として 1 つ(元の姿)だけ並べる
            if layer.source==LayerSource::Camera||layer.copy!=0 {continue}
            let Some(world)=worlds.get(&layer.id) else{continue};
            let attrs=view.attrs(layer.id).map_err(e)?.unwrap_or_default();
            let local=self.engine.selected_layer_bounds_in(&view,&resolved,layer.id,time).map(|b|glam::Vec3::from(b.center())).unwrap_or(glam::Vec3::ZERO);
            let center=world.transform_point3(local)-camera.eye;
            let parent=attrs.parent.and_then(|id|worlds.get(&id).copied()).unwrap_or(glam::Affine3A::IDENTITY);
            let inverse=parent.inverse();
            let get=|name|view.value_at(layer.id,&PropertyId::new(name).unwrap(),time).ok().flatten();
            let position=match get(property::POSITION){Some(Value::Vec2(v))=>v,_=>[0.0,0.0]};
            let z=match get(property::POSITION_Z){Some(Value::F64(v))=>v,_=>0.0};
            items.push(json!({"id":layer.id.0,"name":attrs.name,"point":[center.x,center.z],"local":[position[0],position[1],z],"inverseX":inverse.transform_vector3(glam::Vec3::X).to_array(),"inverseZ":inverse.transform_vector3(glam::Vec3::Z).to_array(),"locked":attrs.locked || !inverse.is_finite(),"color":attrs.label_color}));
        }
        Ok(json!({"items":items,"halfFov":(camera.vertical_fov_radians*0.5).tan()*camera.aspect_ratio}))
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
        let observer=crate::doc::core::camera_projection(comp,self.view_camera()?);
        let matrix=observer.projection_matrix()*observer.view_matrix();
        Ok(move|p:glam::Vec3|{let c=matrix*p.extend(1.0);if c.w<=0.0{return None}Some([(c.x/c.w+1.0)*0.5*comp.width as f32,(1.0-c.y/c.w)*0.5*comp.height as f32])})
    }
    /// User Stage の観測者: 正面か、注視点、comp 面の 4 角。Camera View では Null。
    fn observer_status(&self)->Result<Json,String>{
        if !self.user_stage { return Ok(Json::Null); }
        let comp=self.doc.view().composition().map_err(e)?.ok_or("No composition")?.spec();
        let screen=self.observer_screen()?;
        let (w,h)=(comp.width as f32,comp.height as f32);
        let frame:Vec<_>=[glam::Vec3::ZERO,glam::vec3(w,0.0,0.0),glam::vec3(w,h,0.0),glam::vec3(0.0,h,0.0)].into_iter().map(&screen).collect();
        Ok(json!({"front":self.user_camera==Default::default(),"orbit":self.user_camera.orbit_degrees,"target":screen(self.user_camera.target(comp)),"frame":if frame.iter().all(Option::is_some){json!(frame)}else{Json::Null}}))
    }
    fn camera_gizmos(&self)->Result<Json,String>{
        if !self.user_stage { return Ok(json!([])); }
        let view=self.doc.view();let time=self.time()?;let comp=view.composition().map_err(e)?.ok_or("No composition")?.spec();
        let screen=self.observer_screen()?;
        let mut gizmos=Vec::new();
        for id in view.layers(){
            let Some(meta)=view.meta(id).map_err(e)? else{continue};
            if meta.source!=LayerSource::Camera || view.attrs(id).map_err(e)?.unwrap_or_default().hidden || !meta.timing.covers(self.frame){continue}
            let read=|name| view.value_at(id,&PropertyId::new(name).unwrap(),time).ok().flatten();
            let camera=crate::doc::core::ResolvedCamera {
                center:match read(property::CAMERA_CENTER){Some(Value::Vec2(v))=>v.map(|x|x as f32),_=>[0.0;2]},
                zoom:match read(property::CAMERA_ZOOM){Some(Value::F64(v))=>v as f32,_=>1.0},
                roll_degrees:match read(property::CAMERA_ROLL){Some(Value::F64(v))=>v as f32,_=>0.0},..Default::default()
            };
            let projection=crate::doc::core::camera_projection(comp,camera);
            let rotation=projection.rotation.inverse();
            let depth=crate::doc::core::distance_from_camera(comp,0.0);
            let height=depth*(projection.vertical_fov_radians*0.5).tan();let width=height*projection.aspect_ratio;
            let points=[glam::Vec3::ZERO,glam::vec3(-width,-height,-depth),glam::vec3(width,-height,-depth),glam::vec3(width,height,-depth),glam::vec3(-width,height,-depth)];
            let points:Vec<_>=points.into_iter().map(|p|screen(projection.eye+rotation*p)).collect();
            if points.iter().all(Option::is_some){gizmos.push(json!({"id":id.0,"points":points,"target":screen(camera.target(comp)),"center":camera.center,"zoom":camera.zoom,"roll":camera.roll_degrees}));}
        }
        Ok(json!(gizmos))
    }

    pub(crate) fn bounds(&self,layer:LayerId)->Option<Json>{
        let resolved=self.doc.view().resolved_layers(self.time().ok()?).ok()?;
        self.bounds_in(&resolved,layer)
    }
    /// 解決済みの層の並びから枠を取る。status は 1 フレームに 1 回だけ解いて、全層でこれを使う。
    pub(crate) fn bounds_in(&self,resolved:&[crate::doc::store::ResolvedLayer],layer:LayerId)->Option<Json>{
        let view=self.doc.view();let time=self.time().ok()?;let comp=view.composition().ok()??.spec();let camera=view.resolve_camera(time).ok()?;let observer=self.view_camera().ok()?;
        let r=resolved.iter().find(|r|r.id==layer&&!r.ghost)?;
        let b=self.engine.selected_layer_bounds_in(&view,resolved,layer,time)?;
        let world=r.placement.world_transform?;
        let corners:Vec<_>=crate::doc::core::projected_screen_corners(comp,camera,observer,r.projection,world,b.min,b.max).into_iter().map(|p|[p.x as f64,p.y as f64]).collect();
        let anchor=match view.value_at(layer,&PropertyId::new(property::ANCHOR).ok()?,time).ok().flatten(){Some(Value::Vec2(v))=>v,_=>[0.0,0.0]};
        let fractions:[f64;2]=std::array::from_fn(|i|(anchor[i]-b.min[i]as f64)/(b.max[i]-b.min[i]).max(1e-6)as f64);
        Some(json!({"layer":layer.0,"corners":corners,"localMin":b.min,"localMax":b.max,"anchorFraction":fractions}))
    }
    pub(crate) fn status(&self)->Result<Json,String>{
        let view=self.doc.view();let comp=view.composition().map_err(e)?.ok_or("No composition")?;let at=self.time()?;
        let catalog=crate::render::engine::known_effects();
        let resolved=view.resolved_layers(at).map_err(e)?;
        // 再生中で Document が変わっていなければ、時刻で変わる物(値・枠)だけの軽い status にする。
        let revision=format!("{:?}",self.doc.revision());
        let live=self.clock.playing()&&self.full_status_revision.borrow().as_deref()==Some(revision.as_str());
        let mut ids=view.layers();ids.sort_by_key(|id|std::cmp::Reverse((view.meta(*id).ok().flatten().map_or(0,|m|m.order),id.0)));
        let mut layers=Vec::new();
        for id in ids{
            let attrs=view.attrs(id).map_err(e)?.unwrap_or_default();let Some(meta)=view.meta(id).map_err(e)? else{continue};
            let data=editor::functions::read::inspector_data_from_doc(&view,id,at,&catalog);
            let mut properties=Vec::new();let mut seen=std::collections::BTreeSet::new();
            for row in data.transform.iter().chain(data.text.iter()){
                if let Some(p)=&row.property{
                    if seen.insert(p.clone()){properties.push(prop(&view,id,p,&row.label,&row.value,row.range,at,comp.fps)?);}
                }
                for (index,axis) in row.axis.iter().enumerate(){if let Some((p,v))=axis{
                    if seen.insert(p.clone()){properties.push(prop(&view,id,p,&format!("{} {}",row.label,["X","Y","Z"][index]),v,row.range,at,comp.fps)?);}
                }}
            }
            for p in view.properties(id){if !p.name().starts_with(property::EFFECT_PREFIX)&&seen.insert(p.name().into()){
                if let Some(v)=view.value_at(id,&p,at).map_err(e)?{properties.push(prop(&view,id,p.name(),p.name(),&v,None,at,comp.fps)?);}
            }}
            if meta.source == LayerSource::Camera {
                properties = [(property::CAMERA_CENTER,"Center",Value::Vec2([0.0,0.0]),None), (property::CAMERA_ZOOM,"Zoom",Value::F64(1.0),Some((0.01,100.0))), (property::CAMERA_ROLL,"Roll",Value::F64(0.0),None)]
                    .into_iter().map(|(p,label,v,range)| prop(&view,id,p,label,&v,range,at,comp.fps)).collect::<Result<Vec<_>,_>>()?;
            }
            let text=if let Some(t)=view.text_document(id).map_err(e)?{
                let keys:Result<Vec<_>,String>=t.content.keys().iter().map(|k|Ok(json!({"frame":k.t.try_to_frame_round(comp.fps).map_err(e)?,"value":k.content,"interp":{"kind":"Hold"}}))).collect();
                let keys=keys?;
                properties.insert(0,json!({"id":"content","label":"Content","kind":"text","value":t.content.eval(at),"min":null,"max":null,"keyedNow":keys.iter().any(|k|k["frame"]==self.frame),"keys":keys}));
                let resolved=view.resolved_text_document(id,at).map_err(e)?.unwrap_or(t.clone());
                let s=resolved.styles.first();
                json!({"content":t.content.eval(at),"size":s.map(|s|s.size),"lineHeight":s.and_then(|s|s.line_height),"tracking":s.map(|s|s.tracking)})
            }else{Json::Null};
            let colors:Vec<_>=data.colors.iter().map(|c|json!({"label":c.label,"slot":c.slot,"rgba":editor::color::read_color(&self.doc,&c.slot).unwrap_or([0.0,0.0,0.0,1.0])})).collect();
            let effects:Result<Vec<_>,String>=data.effects.iter().map(|effect|{
                let kind=crate::doc::store::placement::kind(&effect.plugin_id);
                let mut params:Vec<Json>=effect.params.iter().filter_map(|p|p.property.as_ref().map(|idp|prop(&view,id,idp,&p.label,&p.value,p.range,at,comp.fps))).collect::<Result<_,_>>()?;
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
                    json!({"columns":["Each","Random"],"count":pid("count"),"along":pid("mode"),"pick":pid("pick"),"subject":pid("subject"),"seed":pid("seed"),
                        "materials":params.iter().filter(|r|r["id"].as_str().is_some_and(|i|i.starts_with(&share))).map(|r|json!({"id":r["id"],"label":r["label"]})).collect::<Vec<_>>(),
                        "shape":k.shape.iter().filter(|n|shown(n)).map(|n|json!({"id":pid(n),"label":k.params.iter().find(|p|p.name==*n).map_or(*n,|p|p.label),"unit":crate::doc::store::placement::unit(n)})).collect::<Vec<_>>(),
                        "rows":k.grid.iter().map(|r|json!({"label":r.label,"unit":r.unit,"each":r.each.map(pid),"random":r.random.map(pid),"axis":r.axis,"advanced":r.advanced})).collect::<Vec<_>>()})
                });
                let enabled=!matches!(view.value_at(id,&PropertyId::effect_enabled(EffectId(effect.id)),at).map_err(e)?,Some(Value::Bool(false)));Ok(json!({"id":effect.id,"enabled":enabled,"pluginId":effect.plugin_id,"name":catalog.iter().find(|d|d.plugin_id==effect.plugin_id).map_or(effect.plugin_id.as_str(),|d|d.label.as_str()),"placement":kind.is_some(),"layout":layout,"params":params}))
            }).collect();
            let position=self.position(id)?;let bounds=self.bounds_in(&resolved,id);
            let tint=data.colors.first().and_then(|c|editor::color::read_color(&self.doc,&c.slot)).unwrap_or([0.9,0.5,0.2,1.0]);
            // 混ぜ方の見本は選んだ層だけ。1 フレームごとの JSON を層数に比例させない。
            let blend_previews:serde_json::Map<String,Json>=if self.selected_ids.contains(&id){(0..18).filter_map(BlendMode::from_enum_value).map(|mode|(format!("{:?}",mode),json!(editor::blend_preview::BEDS.map(|bed|editor::blend_preview::blend(mode,[tint[0]as f32,tint[1]as f32,tint[2]as f32],bed))))).collect()}else{Default::default()};
            let content_keys:Vec<_>=properties.iter().find(|p|p["id"]=="content").and_then(|p|p["keys"].as_array()).into_iter().flatten().map(|k|json!({"frame":k["frame"],"content":k["value"]})).collect();
            let corners=bounds.as_ref().and_then(|b|b["corners"].as_array()).map(|c|[c[0].clone(),c[1].clone(),c[3].clone(),c[2].clone()]);
            layers.push(json!({"id":id.0,"name":attrs.name,"kind":source_kind(&meta.source),"ghost":attrs.ghost,"ghostable":crate::editor::timeline_edit::ghostable(&view,id),"parent":attrs.parent.map(|p|p.0),"order":meta.order,"hidden":attrs.hidden,"solo":attrs.solo,"locked":attrs.locked,"clipToBelow":attrs.clip_to_below,"clipBase":view.clipping_base(id).map_err(e)?.map(|b|b.0),"projection":match attrs.projection{LayerProjection::TwoD=>"2D",LayerProjection::TwoPointFiveD=>"2.5D",LayerProjection::ThreeD=>"3D"},"flatten":attrs.flatten,"environment":attrs.environment,"frozen":attrs.frozen,"blendMode":attrs.blend_mode,"matte":attrs.matte,"start":meta.timing.start,"duration":meta.timing.duration,"sourceIn":meta.timing.source_in,"properties":properties,"text":text,"colors":colors,"effects":effects?,"contentKeys":content_keys,"corners":corners,"blendPreviews":blend_previews,"x":position[0],"y":position[1],"bounds":bounds,"anchorFraction":bounds.as_ref().map(|b|b["anchorFraction"].clone())}));
        }
        if live {
            let (undo,redo)=self.doc.history_depth();
            let selected_keys:Vec<_>=self.selected_keys.iter().map(|k|json!({"layer":k.layer.0,"property":k.property.as_ref().map(|p|p.name()),"frame":(k.at_sec*comp.fps.as_f64()).round()as i64})).collect();
            let point=self.selected.map(|id|self.position(id)).transpose()?.unwrap_or([0.0,0.0]);
            let live_layers:Vec<Json>=layers.iter().map(|l|{
                let rows=|v:&Json|v.as_array().into_iter().flatten().map(|r|json!({"id":r["id"],"value":r["value"],"keyedNow":r["keyedNow"]})).collect::<Vec<_>>();
                json!({"id":l["id"],"x":l["x"],"y":l["y"],"bounds":l["bounds"],"corners":l["corners"],"text":l["text"],"properties":rows(&l["properties"]),
                    "effects":l["effects"].as_array().into_iter().flatten().map(|e|json!({"id":e["id"],"params":rows(&e["params"])})).collect::<Vec<_>>()})
            }).collect();
            // 寸法は Swift の render が毎コマ読む。軽い status でも落とさない(落とすと再生 2 コマ目で render が失敗し、再生が止まる)。
            return Ok(json!({"frame":self.frame,"playing":self.clock.playing(),"documentRevision":revision,"undo":undo,"redo":redo,"width":comp.width,"height":comp.height,"fps":comp.fps.as_f64(),"durationFrames":comp.duration_frames,
                "selectedId":self.selected.map(|s|s.0),"selectedIds":self.selected_ids.iter().map(|s|s.0).collect::<Vec<_>>(),"selectedKeys":selected_keys,"x":point[0],"y":point[1],
                "stageView":if self.user_stage{"User"}else{"Camera"},"animate":self.animate,"renderCount":self.render_count,"pickedColor":self.picked_color,"pickSerial":self.pick_serial,"renderMs":self.render_ms,"liveLayers":live_layers}));
        }
        *self.full_status_revision.borrow_mut()=Some(revision);
        let assets:Result<Vec<_>,String>=view.assets().map_err(e)?.into_iter().map(|a|{
            let used=self.asset_used(a.id)?;
            let path=a.path_absolute.clone();let missing=path.as_ref().is_none_or(|p|!std::path::Path::new(p).exists());
            Ok(json!({"id":a.id.to_string(),"name":a.name,"path":path,"mime":a.asset_type,"used":used,"missing":missing,"thumbnail":path.as_ref().and_then(|p|if a.asset_type.starts_with("image/"){editor::thumbnail::image_data_uri(p)}else if a.asset_type.starts_with("video/"){editor::thumbnail::video_data_uri(p)}else{None}),"role":match a.role{AssetRole::Reference=>"reference",_=>"material"}}))
        }).collect();
        let used=editor::fixture::used_colors_from_doc(&self.doc);let swatches=if used.is_empty(){editor::fixture::default_palette()}else{used};
        let palette:Vec<_>=swatches.iter().map(|s|json!({"rgba":s.rgba.map(|x|x as f64/255.0),"hex":s.hex,"used":!editor::fixture::used_colors_from_doc(&self.doc).is_empty()})).collect();
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
        let mut status=json!({"stageView":if self.user_stage{"User"}else{"Camera"},"observer":self.observer_status()?,"cameraGizmos":self.camera_gizmos()?,"width":comp.width,"height":comp.height,"fps":comp.fps.as_f64(),"fpsNum":comp.fps.num(),"fpsDen":comp.fps.den(),"durationFrames":comp.duration_frames,"background":comp.background,"frame":self.frame,"playing":self.clock.playing(),"playbackHealth":playback_health,"waveforms":waveforms,"undo":undo,"redo":redo,"path":self.path,"dirty":authored_signature(&self.doc)?!=self.saved_signature,"layers":layers,"selectedId":self.selected.map(|id|id.0),"selectedIds":self.selected_ids.iter().map(|id|id.0).collect::<Vec<_>>(),"selectedKeys":selected_keys,"selectedBounds":self.selected.and_then(|id|self.bounds(id)),"x":point[0],"y":point[1],"assets":assets?,"catalog":catalog.iter().map(|e|json!({"id":e.plugin_id,"name":e.label})).collect::<Vec<_>>(),"palette":palette,"markers":markers?,"colorTarget":color_target,"capabilities":crate::port::CAPABILITIES,"easeKinds":if self.clock.playing(){Json::Null}else{json!(editor::ease_kinds::KINDS.iter().copied().map(interp).collect::<Vec<_>>())},"documentRevision":format!("{:?}",self.doc.revision()),"deviceId":self.device_id.to_string(),"renderCount":self.render_count,"renderMs":self.render_ms,"interopCopies":0,"readbacks":0,"error":self.error,"preview":self.preview.is_some(),"export":self.exporter.status()});
        status["notebook"]=serde_json::to_value(view.notebook().map_err(e)?).map_err(e)?;
        status["depthLayout"]=self.depth_layout(&resolved)?;
        status["animate"]=json!(self.animate);
        if status["easeKinds"].is_null(){status.as_object_mut().unwrap().remove("easeKinds");}
        status["importExtensions"]=json!(crate::render::media::import_extensions());
        Ok(status)
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
