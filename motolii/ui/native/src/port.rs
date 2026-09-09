use crate::{EditorRuntime,editor,snapshot};
use crate::doc::store::*;
use serde_json::{Value as J,json};
use editor::session::{KeySel,ColorSlot};
fn e(error:impl std::fmt::Display)->String{error.to_string()}
pub(crate) const CAPABILITIES:&[&str]=&["status","notes","stageView","select","setProperty","previewProperties","commitPreview","cancelPreview","setText","previewText","styleText","setFont","setAttrs","create","duplicate","ghost","sequence","previewSequence","copy","cut","paste","delete","group","ungroup","reorder","split","setTiming","toggleKey","moveKeys","ease","setColor","previewColor","focusColor","applyPalette","applyEffect","removeEffect","expandEffect","moveEffect","enableEffect","animate","clip","addMarker","setMarker","deleteMarker","composition","import","placeAsset","removeAsset","replaceAsset","relinkAsset","save","new","undo","redo","seek","anchor","freeze","setFillMode","setGradient","previewBlend","setTimings","previewTimings","stageGesture","export","exportStatus","cancelExport","play","pause","tick","moveLayers","pickColor","historyGoto"];
fn num(j:&J,key:&str)->Result<f64,String>{j[key].as_f64().filter(|v|v.is_finite()).ok_or_else(||format!("Missing finite {key}"))}
fn integer(j:&J,key:&str)->Result<i64,String>{j[key].as_i64().ok_or_else(||format!("Missing integer {key}"))}
fn layer(j:&J)->Result<LayerId,String>{j["layer"].as_u64().map(LayerId).ok_or("Missing layer".into())}
fn string<'a>(j:&'a J,key:&str)->Result<&'a str,String>{j[key].as_str().ok_or_else(||format!("Missing {key}"))}
fn ids(j:&J)->Result<Vec<LayerId>,String>{j.as_array().ok_or("Expected layer array")?.iter().map(|v|v.as_u64().map(LayerId).ok_or("Invalid layer id".into())).collect()}
fn asset_id(j:&J)->Result<AssetId,String>{j["id"].as_str().and_then(|s|s.parse::<u64>().ok()).or_else(||j["id"].as_u64()).map(AssetId::from_raw).ok_or("Invalid asset id".into())}
fn rgba(j:&J)->Result<[f64;4],String>{let a:[f64;4]=serde_json::from_value(j["rgba"].clone()).map_err(e)?;if a.iter().any(|v|!v.is_finite()){return Err("Color must be finite".into())}Ok(a.map(|v|v.clamp(0.0,1.0)))}
fn decoded_value(raw:&J,expected:Option<&Value>)->Result<Value,String>{
    Ok(match expected{
        Some(Value::Enum(_))=>Value::Enum(raw.as_i64().ok_or("Expected enum")?),
        Some(Value::Bool(_))=>Value::Bool(raw.as_bool().ok_or("Expected bool")?),
        Some(Value::LayerId(_))=>Value::LayerId(raw.as_u64().ok_or("Expected layer id")?),
        Some(Value::Path(_))=>Value::Path(serde_json::from_value(raw.clone()).map_err(e)?),
        _=>if let Some(n)=raw.as_f64().filter(|n|n.is_finite()){Value::F64(n)}else if raw.as_array().is_some_and(|a|a.len()==2){Value::Vec2(serde_json::from_value(raw.clone()).map_err(e)?)}else if raw.as_array().is_some_and(|a|a.len()==4){Value::Color(serde_json::from_value(raw.clone()).map_err(e)?)}else{return Err("Unsupported property value".into())}
    })
}
fn attrs_patch(j:&J)->Result<LayerAttrsPatch,String>{
    let mut p=LayerAttrsPatch::default();
    macro_rules! boolean {($key:literal,$field:ident)=>{if !j[$key].is_null(){p.$field=Some(j[$key].as_bool().ok_or(concat!("Invalid ",$key))?);}}}
    boolean!("hidden",hidden);boolean!("solo",solo);boolean!("locked",locked);boolean!("flatten",flatten);boolean!("environment",environment);boolean!("clipToBelow",clip_to_below);boolean!("autoOrient",auto_orient);
    if let Some(s)=j["name"].as_str(){p.name=Some(s.into());}
    if j.get("parent").is_some(){p.parent=Some(if j["parent"].is_null(){None}else{Some(LayerId(j["parent"].as_u64().ok_or("Invalid parent")?))});}
    if j.get("blendMode").is_some(){p.blend_mode=Some(serde_json::from_value(j["blendMode"].clone()).map_err(e)?);}
    if j.get("projection").is_some(){p.projection=Some(serde_json::from_value(j["projection"].clone()).map_err(e)?);}
    if j.get("matte").is_some(){p.matte=Some(serde_json::from_value(j["matte"].clone()).map_err(e)?);}
    if j.get("ghost").is_some(){p.ghost=Some(if j["ghost"].is_null(){None}else{Some(j["ghost"].as_i64().filter(|d|*d!=0).ok_or("Ghost delay must be a non-zero frame count")?)});}
    Ok(p)
}
impl EditorRuntime{
    fn pick(&mut self,ids:Vec<LayerId>){self.selected_ids=ids;self.selected=self.selected_ids.last().copied();}
    fn selected_required(&self)->Result<&[LayerId],String>{if self.selected_ids.is_empty(){Err("Select layers".into())}else{Ok(&self.selected_ids)}}
    fn apply(&mut self,intents:impl IntoIterator<Item=Intent>)->Result<(),String>{
        let intents:Vec<_>=intents.into_iter().collect();
        let born=self.born_spans(&intents);
        self.doc.apply_all(intents).map_err(e)?;
        if !born.is_empty(){self.selected_keys=born;}
        Ok(())
    }
    /// The pairs Animate From is about to give birth to: both ends of a track a property did not have yet.
    /// They become the key selection, so the Ease desk lands on the new motion without a trip to the Timeline.
    fn born_spans(&self,intents:&[Intent])->Vec<KeySel>{
        let Animate::From{origin,..}=self.animate else{return Vec::new()};
        let view=self.doc.view().without_transients();
        let mut out=Vec::new();
        for intent in intents{
            let Intent::SetTrack{layer,property,track}=intent else{continue};
            if track.keys().len()!=2||!track.keys().iter().any(|k|k.t==origin){continue}
            if view.track(*layer,property).ok().flatten().is_some(){continue}
            for k in track.keys(){out.push(KeySel{layer:*layer,property:Some(property.clone()),at_sec:k.t.as_seconds_f64()});}
        }
        out
    }
    fn set_preview(&mut self,intents:Vec<Intent>)->Result<(),String>{
        let owner=if let Some((owner,_))=&self.preview{*owner}else{self.doc.begin_preview()};
        let projected:Vec<_>=intents.iter().filter_map(|intent|match intent{
            Intent::SetAttrs{layer,patch} if patch.blend_mode.is_some()=>Some(Intent::SetConstant{layer:*layer,property:PropertyId::blend_mode(),value:Value::Enum(patch.blend_mode.unwrap().to_enum_value())}),
            _=>Some(intent.clone()),
        }).collect();
        self.doc.preview_edits(owner,&projected).map_err(e)?;self.preview=Some((owner,intents));Ok(())
    }
    fn property_edits(&self,j:&J)->Result<Vec<Intent>,String>{
        let layer=layer(j)?;let name=string(j,"property")?;let p=PropertyId::new(name).map_err(e)?;
        let view=self.doc.view().without_transients();
        let current=view.value_at(layer,&p,self.time()?).map_err(e)?.or(view.default_value(layer,&p).map_err(e)?);
        let catalog=crate::render::engine::known_effects();let data=editor::functions::read::inspector_data_from_doc(&view,layer,self.time()?,&catalog);
        let rows=data.text.iter().chain(data.transform.iter()).chain(data.effects.iter().flat_map(|e|e.params.iter()));
        let row=rows.filter(|r|r.property.as_deref()==Some(name)).next();
        let mut v=decoded_value(&j["value"],current.as_ref().or(row.map(|r|&r.value)))?;
        if let Some(row)=row{if let(Value::F64(n),Some((min,max)))=(&mut v,row.range){*n=n.clamp(min,max);}}
        self.doc.place_checked(layer,&p,v,self.time()?,self.animate).map(|v|v.into_iter().collect()).map_err(e)
    }
    fn color_intent(&self,j:&J)->Result<Intent,String>{
        let slot:ColorSlot=serde_json::from_value(j["slot"].clone()).map_err(e)?;
        if j.get("layer").is_some()&&slot.layer()!=layer(j)?{return Err("Color target layer mismatch".into())}
        if let Some(reason)=editor::functions::lens::edit_rejection(&self.doc.view(),slot.layer()).map_err(e)?{return Err(reason.into())}
        let color=rgba(j)?;
        let mut intent=editor::color::write_color(&self.doc,&slot,[color[0],color[1],color[2]]).map_err(e)?;
        if let Intent::SetTextDocument{document,..}=&mut intent{
            let (style,stroke)=match slot{ColorSlot::TextFill{style,..}=>(style,false),ColorSlot::TextStroke{style,..}=>(style,true),_=>unreachable!()};
            if let Some(s)=document.styles.iter_mut().find(|s|s.id==style){if stroke{if let Some(c)=s.stroke_color.as_mut(){c[3]=color[3]}}else{s.fill[3]=color[3]}}
        }Ok(intent)
    }
    pub(crate) fn asset_used(&self,id:AssetId)->Result<bool,String>{
        let view=self.doc.view();let a=view.asset(id).map_err(e)?.ok_or("Asset missing")?;
        for layer in view.layers(){if let Some(meta)=view.meta(layer).map_err(e)?{if let LayerSource::File{path,..}=meta.source{if a.path_absolute.as_deref()==Some(&path){return Ok(true)}}}}
        Ok(false)
    }
    fn create_layer(&mut self,kind:editor::create::NewKind,visible:Option<i64>)->Result<(),String>{self.place_layer(kind,self.frame,None,visible)}
    /// 置く場所を指す作成。drop 先は moveLayers と同じ語彙(target/placement)、開始コマは落とした x。作る→並べるを 1 手(Undo 一発)に。
    fn place_layer(&mut self,kind:editor::create::NewKind,start:i64,landing:Option<(Option<LayerId>,String)>,visible:Option<i64>)->Result<(),String>{
        let view=self.doc.view();let comp=view.composition().map_err(e)?.ok_or("No composition")?;
        let id=LayerId(view.next_layer_id());let order=view.layers().iter().filter_map(|l|view.meta(*l).ok().flatten().map(|m|m.order)).max().unwrap_or(-1).checked_add(1).ok_or("Layer order full")?;
        let taken:Vec<_>=view.layers().iter().filter_map(|l|view.attrs(*l).ok().flatten().map(|a|a.name)).collect();
        let mut intents=editor::create::new_layer_intents(id,order,start,comp.duration_frames,comp.fps,(comp.width as f64,comp.height as f64),kind,editor::create::unbounded_frames(visible));
        if intents.iter().any(|i| matches!(i, Intent::SetMeta { meta, .. } if meta.source == LayerSource::Camera)) {
            let camera = view.resolve_camera(self.time()?).map_err(e)?;
            for (name,value) in [(property::CAMERA_CENTER,Value::Vec2(camera.center.map(f64::from))), (property::CAMERA_ZOOM,Value::F64(camera.zoom as f64)), (property::CAMERA_ROLL,Value::F64(camera.roll_degrees as f64))] {
                intents.push(Intent::SetConstant { layer:id, property:PropertyId::new(name).map_err(e)?, value });
            }
        }
        for i in &mut intents{if let Intent::SetAttrs{patch,..}=i{if let Some(name)=&mut patch.name{*name=editor::create::numbered(name,&taken)}}}
        let at=self.time()?;drop(view);
        match landing{None=>self.apply(intents)?,Some((target,placement))=>self.doc.apply_then(intents,|doc|doc.move_layer_intents(&[id],target,&placement,at).map(|(i,_)|i)).map_err(e)?}
        self.pick(vec![id]);self.selected_keys.clear();Ok(())
    }
    fn delete_selection(&mut self)->Result<(),String>{
        if !self.selected_keys.is_empty(){let keys:Vec<_>=self.selected_keys.iter().map(|k|(k.layer,k.property.clone(),k.at_sec)).collect();let edits=editor::timeline_edit::delete_key_selection_intents(&self.doc,&keys,self.time()?).map_err(e)?;self.apply(edits)?;self.selected_keys.clear();}
        else{let ids=self.selected_required()?.to_vec();self.apply(ids.into_iter().map(Intent::RemoveLayer))?;self.pick(vec![]);}
        Ok(())
    }
    pub(crate) fn request(&mut self,j:J)->Result<(),String>{
        let op=j["op"].as_str().unwrap_or("status");
        if op=="status"||op=="exportStatus"{return Ok(())}
        if op=="stageView" {
            self.cancel_preview();
            if let Some(mode)=j["mode"].as_str() { self.user_stage=matches!(mode,"User"|"3D"); }
            if let Some(orbit)=j["orbit"].as_array() {
                if orbit.len()!=2 { return Err("Expected two orbit angles".into()); }
                self.user_camera.orbit_degrees=[orbit[0].as_f64().filter(|v|v.is_finite()).ok_or("Invalid orbit")? as f32,orbit[1].as_f64().filter(|v|v.is_finite()).ok_or("Invalid orbit")? as f32];
            }
            if let Some(id)=j["focus"].as_u64() {
                let comp=self.doc.view().composition().map_err(e)?.ok_or("No composition")?.spec();
                let (centre,radius)=self.focus_sphere(LayerId(id))?.ok_or("Layer has no bounds to focus")?;
                self.user_camera=self.user_camera.looking_at(comp,centre,radius);
            }
            if j["reset"].as_bool()==Some(true) { self.user_camera=Default::default(); }
            if j["fit"].as_bool()==Some(true) {
                let view=self.doc.view();let comp=view.composition().map_err(e)?.ok_or("No composition")?.spec();
                let [x,y,w,h]=view.resolve_stage_extent(self.time()?).map_err(e)?.rect(comp);
                self.user_camera=crate::doc::core::ResolvedCamera { center:[x+w*0.5-comp.width as f32*0.5,y+h*0.5-comp.height as f32*0.5], distance_scale:(w/comp.width as f32).max(h/comp.height as f32), ..Default::default() };
            }
            return Ok(());
        }

        if op=="save" && j["copy"].as_bool()==Some(true) && self.preview.is_some(){return Err("Finish the active preview before creating a checkpoint".into());}
        if op=="tick"{self.clock_frame();self.error=None;return Ok(())}
        if matches!(op,"cancelPreview"|"commitPreview") && j["owner"].is_u64()
            && self.preview.as_ref().map(|p|p.0) != j["owner"].as_u64() { return Ok(()); }
        if op.starts_with("preview") || (op == "stageGesture" && !matches!(j["phase"].as_str(),Some("hover"|"configure"))) || (matches!(op,"setFont"|"styleText"|"setGradient") && j["preview"] == true) {
            let tag = j["interaction"].as_str().unwrap_or(op);
            if self.preview_tag.as_deref() != Some(tag) { self.cancel_preview(); }
            self.preview_tag = Some(tag.to_owned());
        }
        if !matches!(op,"previewText"|"styleText"|"setGradient"|"previewProperties"|"previewColor"|"previewBlend"|"previewX"|"commitPreview"|"commitX"|"previewTimings"|"previewSequence"|"stageGesture"){self.cancel_preview();}
        match op{
            "notes"=>self.edit_notes(&j)?,
            "select"=>{
                let chosen=if let Some(a)=j.get("ids"){ids(a)?}else{j["id"].as_u64().map(LayerId).into_iter().collect()};
                if chosen.iter().any(|id|!self.doc.view().has_layer(*id)){return Err("Unknown layer".into())}
                self.pick(chosen);self.selected_keys.clear();
                if let Some(keys)=j["keys"].as_array(){let fps=editor::keyframe_edit::document_fps(&self.doc).map_err(e)?.as_f64();
                    for k in keys{self.selected_keys.push(KeySel{layer:LayerId(k["layer"].as_u64().ok_or("Invalid key layer")?),property:k["property"].as_str().map(PropertyId::new).transpose().map_err(e)?,at_sec:integer(k,"frame")?as f64/fps});}
                }
            }
            "setProperty"=>{let edits=self.property_edits(&j)?;self.apply(edits)?;}
            "previewProperties"=>{let mut edits=Vec::new();for p in j["edits"].as_array().ok_or("Expected edits")?{edits.extend(self.property_edits(p)?);}self.set_preview(edits)?;}
            "commitPreview"|"commitX"=>{if let Some((owner,edits))=self.preview.take(){self.doc.clear_preview_edits(owner);self.apply(edits)?;}}
            "cancelPreview"=>{}
            "setX"=>{let edits=self.x_edit(num(&j,"value")?)?;self.apply(edits)?;}
            "previewX"=>{let edits=self.x_edit(num(&j,"value")?)?;self.set_preview(edits)?;}
            "previewText"=>{let id=layer(&j)?;if self.doc.view().attrs(id).map_err(e)?.ok_or("Layer not found")?.locked{return Err("Layer is locked".into())}let edit=editor::text::content_intent(&self.doc,id,self.time()?,string(&j,"content")?.into()).map_err(e)?;self.set_preview(vec![edit])?;}
            "styleText"=>{let edits=editor::text_format::edits(&self.doc,layer(&j)?,self.time()?,&j)?;if edits.is_empty(){self.cancel_preview();return Ok(());}if j["preview"]==true{let projected=edits.iter().filter(|e|!matches!(e,Intent::SetPropertySlot{..}|Intent::SetPropertyModulators{..})).cloned().collect();self.set_preview(projected)?;self.preview.as_mut().unwrap().1=edits;}else{self.cancel_preview();self.apply(edits)?;}}
            "setFont" if j["scope"].is_string()=>{let edits=editor::text_format::edits(&self.doc,layer(&j)?,self.time()?,&j)?;self.apply(edits)?;}
            "setFont"=>{let intent=editor::text::font_intent(&self.doc,layer(&j)?,string(&j,"family")?)?;if j["preview"].as_bool()==Some(true){self.set_preview(vec![intent])?}else{self.apply([intent])?}}
            "setText"=>{let id=layer(&j)?;if self.doc.view().text_document(id).map_err(e)?.is_none(){return Err("Select a Text layer".into())}let at=self.time()?;editor::text::write_content(&mut self.doc,id,at,string(&j,"content")?.into()).map_err(e)?;}
            "setAttrs"=>{let patch=attrs_patch(&j["patch"])?;let layers=ids(&j["layers"])?;
                if patch.ghost.is_some_and(|g|g.is_some()){if let Some(l)=layers.iter().find(|&&l|!editor::timeline_edit::ghostable(&self.doc.view(),l)){return Err(format!("Layer {} cannot carry a ghost",l.0))}}
                if patch.projection.is_some(){
                    let at=self.time()?;
                    let centers:Vec<_>={let view=self.doc.view();let resolved=view.resolved_layers(at).map_err(e)?;
                        layers.iter().map(|&id|(id,self.engine.selected_layer_bounds_in(&view,&resolved,id,at).map(|b|b.center()).unwrap_or([0.0;3]))).collect()};
                    self.doc.set_projection(&centers,patch,at).map_err(e)?;
                } else {self.apply(layers.into_iter().map(|layer|Intent::SetAttrs{layer,patch:patch.clone()}))?;}}
            "create"=>{let kind=match string(&j,"kind")?{"text"=>editor::create::NewKind::Text,"rectangle"=>editor::create::NewKind::Rectangle,"bezier"=>editor::create::NewKind::Bezier,"cube"=>editor::create::cube()?,"camera"=>editor::create::NewKind::Camera,"stage"=>editor::create::NewKind::Stage,k=>match k.strip_prefix("background:"){Some(id)=>editor::create::background(id)?,None=>return Err("Unsupported create kind".into())}};self.create_layer(kind,j["visibleFrames"].as_i64())?;}
            "copy"|"cut"=>{
                if self.selected_keys.is_empty(){self.clipboard.copy_layers(&self.doc,self.selected_required()?).map_err(e)?;}else{self.clipboard.copy_keys(&self.doc,&self.selected_keys).map_err(e)?;}
                if op=="cut"{self.delete_selection()?;}
            }
            "paste"=>{let result=self.clipboard.paste(&mut self.doc,self.selected,self.frame).map_err(e)?;self.accept_paste(result);}
            "duplicate"=>{
                if self.selected_keys.is_empty(){let ids=self.selected_required()?.to_vec();let copies=editor::timeline_edit::duplicate_layers(&mut self.doc,&ids).map_err(e)?;self.pick(copies);}
                else{let c=editor::clipboard::Clipboard::default();c.copy_keys(&self.doc,&self.selected_keys).map_err(e)?;let fps=editor::keyframe_edit::document_fps(&self.doc).map_err(e)?.as_f64();let at=self.selected_keys.iter().map(|k|(k.at_sec*fps).round()as i64).max().unwrap_or(self.frame).saturating_add(1);let result=c.paste(&mut self.doc,None,at).map_err(e)?;self.accept_paste(result);}
            }
            "delete"=>self.delete_selection()?,
            "group"=>{let ids=self.selected_required()?.to_vec();let group=self.doc.group_layers(&ids).map_err(e)?.ok_or("No group created")?;self.pick(vec![group]);self.selected_keys.clear();}
            "ungroup"=>{let ids=self.selected_required()?.to_vec();let children=self.doc.ungroup_layers(&ids).map_err(e)?;if children.is_empty(){return Err("Select a Group".into())}self.pick(children);self.selected_keys.clear();}
            "moveLayers"=>{let layers=ids(&j["layers"])?;let target=if j["target"].is_null(){None}else{Some(LayerId(j["target"].as_u64().ok_or("Invalid drop target")?))};let at=self.time()?;let moved=self.doc.move_layers(&layers,target,string(&j,"placement")?,at).map_err(e)?;self.pick(moved);self.selected_keys.clear();}
            "reorder"=>{let id=self.selected.ok_or("Select a layer")?;let order=self.doc.view().meta(id).map_err(e)?.ok_or("Layer metadata missing")?.order;let delta:i16=integer(&j,"delta")?.try_into().map_err(e)?;self.apply([Intent::SetOrder{layer:id,order:order.saturating_add(delta)}])?;}
            "split"=>{let ids=self.selected_required()?.to_vec();let copies=editor::timeline_edit::split_layers(&mut self.doc,&ids,self.frame).map_err(e)?;if copies.is_empty(){return Err("Playhead must be inside a layer".into())}self.pick(copies);}
            "setTimings"|"previewTimings"=>{let mut edits=Vec::new();for change in j["changes"].as_array().ok_or("Missing timing changes")?{edits.extend(self.timing_edits(change)?);}if op=="previewTimings"{self.set_preview(edits)?;}else{self.apply(edits)?;}}
            "stageGesture"=>self.stage_gesture(&j)?,
            "setTiming"=>{let id=layer(&j)?;let old=self.doc.view().meta(id).map_err(e)?.ok_or("Layer metadata missing")?.timing;let mut next=old;next.start=integer(&j,"start")?;next.duration=integer(&j,"duration")?;next.source_in=integer(&j,"sourceIn")?;let move_keys=next.duration==old.duration&&next.source_in==old.source_in;let edits=editor::functions::verb::retime_layer(&self.doc,id,old,next,move_keys).map_err(e)?;self.apply(edits)?;}
            "toggleKey"=>{let id=layer(&j)?;let name=string(&j,"property")?;let at=self.time()?;
                if name=="content"{editor::text::toggle_content_key(&mut self.doc,id,at,String::new()).map_err(e)?;}
                else{let p=PropertyId::new(name).map_err(e)?;editor::functions::lens::require_local_source(&self.doc.view(),id,&p).map_err(e)?;let track=self.doc.view().track(id,&p).map_err(e)?.unwrap_or_default();
                    if track.keys().iter().any(|k|k.t.try_to_frame_round(editor::keyframe_edit::document_fps(&self.doc).unwrap()).ok()==Some(self.frame)){let edits=editor::timeline_edit::delete_key_selection_intents(&self.doc,&[(id,Some(p),at.as_seconds_f64())],at).map_err(e)?;self.apply(edits)?;}
                    else{let data=editor::functions::read::inspector_data_from_doc(&self.doc.view(),id,at,&crate::render::engine::known_effects());let fallback=data.transform.iter().chain(data.text.iter()).chain(data.effects.iter().flat_map(|e|e.params.iter())).find(|r|r.property.as_deref()==Some(name)).map(|r|r.value.clone());let value=self.doc.view().value_at(id,&p,at).map_err(e)?.or(self.doc.view().default_value(id,&p).map_err(e)?).or(fallback).ok_or("No keyframe value")?;let mut track=track;track.insert(Keyframe{t:at,value,interp:Interp::Linear,spatial:None});self.apply([Intent::SetTrack{layer:id,property:p,track}])?;}
                }
            }
            "moveKeys"=>{if self.selected_keys.is_empty(){return Err("Select keyframes".into())}let delta=integer(&j,"deltaFrames")?;let keys:Vec<_>=self.selected_keys.iter().map(|k|(k.layer,k.property.clone(),k.at_sec)).collect();let edits=editor::keyframe_edit::key_selection_move_intents(&self.doc,&keys,delta).map_err(e)?;self.apply(edits)?;let fps=editor::keyframe_edit::document_fps(&self.doc).map_err(e)?.as_f64();for k in &mut self.selected_keys{k.at_sec+=delta as f64/fps;}}
            "ease"=>self.apply_ease(&j)?,
            "setColor"|"previewColor"=>{let intent=self.color_intent(&j)?;if op=="previewColor"{self.set_preview(vec![intent])?;}else{self.apply([intent])?;}}
            "focusColor"=>{let slot:ColorSlot=serde_json::from_value(j["slot"].clone()).map_err(e)?;if slot.layer()!=layer(&j)?{return Err("Color target mismatch".into())}self.color_target=Some(slot);}
            "applyPalette"=>{if let Some(slot)=self.color_target.clone(){let q=json!({"layer":slot.layer().0,"slot":slot,"rgba":rgba(&j)?});let edit=self.color_intent(&q)?;self.apply([edit])?;}else{let color=rgba(&j)?.map(|v|(v*255.0).round()as u8);let mut intents=Vec::new();for id in self.selected_required()?{intents.extend(editor::functions::verb::color_intents(&self.doc,*id,color).map_err(e)?);}self.apply(intents)?;}}
            "applyEffect"=>{let plugins:Vec<String>=if let Some(a)=j["pluginIds"].as_array(){a.iter().map(|p|p.as_str().map(str::to_owned).ok_or("Invalid plugin".into())).collect::<Result<_,String>>()?}else{vec![string(&j,"pluginId")?.into()]};let catalog=crate::render::engine::known_effects();if plugins.iter().any(|p|!catalog.iter().any(|c|c.plugin_id==*p)){return Err("Unknown effect".into())}let mut intents=Vec::new();for id in self.selected_required()?{intents.extend(editor::functions::verb::effect_batch_intents(&self.doc,*id,&plugins).map_err(e)?);}self.apply(intents)?;}
            "animate"=>{let on=j["enabled"].as_bool().ok_or("Missing enabled")?;let interp=if j["shape"].is_object(){editor::ease_kinds::decode(&j["shape"])?}else{Interp::Linear};self.animate=if !on{Animate::Off}else if j["from"].as_bool().unwrap_or(false){Animate::From{origin:self.time()?,interp}}else{Animate::Now{interp}};}
            "expandEffect"=>{let id=layer(&j)?;let effect=EffectId(integer(&j,"id")?as u32);let at=self.time()?;let (intents,copies)=editor::placement_edit::expand_intents(&self.doc,id,effect,at).map_err(e)?;self.apply(intents)?;self.pick(copies);}
            "enableEffect"=>{let id=layer(&j)?;let effect=EffectId(integer(&j,"id")?as u32);let on=j["enabled"].as_bool().ok_or("Expected bool")?;if !self.doc.view().effects(id).map_err(e)?.iter().any(|x|x.id==effect){return Err("Effect missing".into())}let at=self.time()?;let edits=self.doc.place_checked(id,&PropertyId::effect_enabled(effect),Value::Bool(on),at,Animate::Off).map_err(e)?;self.apply(edits)?;}
            "moveEffect"=>{let id=layer(&j)?;let effect=integer(&j,"id")?as u32;let to=integer(&j,"to")?;let mut effects=self.doc.view().effects(id).map_err(e)?;let from=effects.iter().position(|e|e.id.0==effect).ok_or("Effect missing")?;let to=(to.max(0) as usize).min(effects.len().saturating_sub(1));let moved=effects.remove(from);effects.insert(to,moved);self.apply([Intent::SetEffects{layer:id,effects}])?;}
            "removeEffect"=>{let id=layer(&j)?;let effect=integer(&j,"id")?as u32;let mut effects=self.doc.view().effects(id).map_err(e)?;if !effects.iter().any(|e|e.id.0==effect){return Err("Effect missing".into())}effects.retain(|e|e.id.0!=effect);self.apply([Intent::SetEffects{layer:id,effects}])?;}
            "ghost"=>{let ids:Vec<LayerId>=self.selected_required()?.iter().copied().filter(|&l|editor::timeline_edit::ghostable(&self.doc.view(),l)).collect();if ids.is_empty(){return Err("Nothing here can carry a ghost".into())}let on=j["enabled"].as_bool().unwrap_or(true);let intents:Vec<Intent>=ids.iter().map(|&layer|Intent::SetAttrs{layer,patch:LayerAttrsPatch{ghost:Some(on.then_some(editor::timeline_edit::GHOST_DEFAULT_DELAY)),..Default::default()}}).collect();self.apply(intents)?;}
            "sequence"|"previewSequence"=>{let layers=ids(&j["layers"])?;let delays=j["ghosts"].as_array().ok_or("Missing ghosts")?;if delays.len()!=layers.len(){return Err("layers and ghosts differ in length".into())}let intents:Vec<Intent>=layers.iter().zip(delays).filter(|(&layer,_)|editor::timeline_edit::ghostable(&self.doc.view(),layer)).map(|(&layer,d)|Intent::SetAttrs{layer,patch:LayerAttrsPatch{ghost:Some(d.as_i64().filter(|d|*d!=0)),..Default::default()}}).collect();if op=="previewSequence"{self.set_preview(intents)?;}else{self.apply(intents)?;}}
            "clip"=>{let id=layer(&j)?;let clipped=self.doc.view().attrs(id).map_err(e)?.unwrap_or_default().clip_to_below;self.apply([Intent::SetAttrs{layer:id,patch:LayerAttrsPatch{clip_to_below:Some(!clipped),..Default::default()}}])?;}
            "previewBlend"=>{let layers=if j["layers"].is_array(){ids(&j["layers"])?}else{vec![layer(&j)?]};let mode:BlendMode=serde_json::from_value(j["mode"].clone()).map_err(e)?;self.set_preview(layers.into_iter().map(|layer|Intent::SetAttrs{layer,patch:LayerAttrsPatch{blend_mode:Some(mode),..Default::default()}}).collect())?;}
            "addMarker"|"setMarker"|"deleteMarker"=>self.edit_marker(op,&j)?,
            "composition"=>{let current=self.doc.view().composition().map_err(e)?.ok_or("No composition")?;let or=|key:&str,fallback:i64|j[key].as_i64().unwrap_or(fallback);let width:u32=or("width",current.width as i64).try_into().map_err(e)?;let height:u32=or("height",current.height as i64).try_into().map_err(e)?;let fps=Fps::try_new(or("fpsNum",current.fps.num()),or("fpsDen",current.fps.den())).map_err(e)?;let duration_frames=or("durationFrames",current.duration_frames);let background=if j.get("background").is_some(){serde_json::from_value(j["background"].clone()).map_err(e)?}else{current.background};self.apply([Intent::SetComposition(Composition{width,height,fps,duration_frames,background})])?;}
            "import"=>{let paths:Vec<std::path::PathBuf>=j["paths"].as_array().ok_or("Expected paths")?.iter().map(|p|p.as_str().map(std::path::PathBuf::from).ok_or("Invalid path".into())).collect::<Result<_,String>>()?;let paths=editor::fixture::expand_folders(&paths);if paths.is_empty(){return Err("No files to import".into())}let mut intents=Vec::new();let mut known:std::collections::HashSet<_>=self.doc.view().assets().map_err(e)?.iter().map(|a|a.content_hash.clone()).collect();for path in paths{let draft=editor::fixture::prepare_path(&path,if j["role"]=="reference"{AssetRole::Reference}else{AssetRole::Material})?;if known.insert(draft.content_hash.clone()){intents.push(Intent::AdmitAsset{draft});}}self.apply(intents)?;}
            "placeAsset"|"replaceAsset"=>{let a=self.doc.view().asset(asset_id(&j)?).map_err(e)?.ok_or("Asset missing")?;let path=a.path_absolute.ok_or("Asset path missing")?;if !std::path::Path::new(&path).exists(){return Err("Asset file missing".into())}if op=="placeAsset"{let start=j["start"].as_i64().unwrap_or(self.frame);let landing=j["placement"].as_str().map(|p|(j["target"].as_u64().map(LayerId),p.to_owned()));self.place_layer(editor::create::NewKind::Media{path,name:a.name},start,landing,j["visibleFrames"].as_i64())?;}else{let layer=self.selected.ok_or("Select layer to replace")?;self.apply([Intent::SetSource{layer,source:LayerSource::File{path,fingerprint:None}}])?;}}
            "relinkAsset"=>{let id=asset_id(&j)?;let path=j["path"].as_str().ok_or("Missing path")?.to_owned();if !std::path::Path::new(&path).exists(){return Err("No file at that path".into())}let a=self.doc.view().asset(id).map_err(e)?.ok_or("Asset missing")?;let kind=std::path::Path::new(&path).extension().and_then(|x|x.to_str()).and_then(crate::render::media::asset_type_for_extension).ok_or("Unsupported file type")?;let same_family=kind.split('/').next()==a.asset_type.split('/').next();if !same_family{return Err(format!("Pick a {} file",a.asset_type.split('/').next().unwrap_or("matching")))}self.apply([Intent::RelinkAsset{asset:id,path_absolute:path,project_root:None}])?;}
            "removeAsset"=>{let id=asset_id(&j)?;if self.asset_used(id)?{return Err("Asset is still in use".into())}self.apply([Intent::RemoveAsset{asset:id}])?;}
            "export"=>self.exporter.start(&self.doc,std::path::PathBuf::from(string(&j,"path")?),integer(&j,"start")?,integer(&j,"end")?)?,
            "cancelExport"=>self.exporter.cancel(),
            "save"=>{let path=string(&j,"path")?;
                self.doc.save(path).map_err(e)?;
                if j["copy"].as_bool()!=Some(true){self.path=Some(path.into());self.saved_signature=snapshot::authored_signature(&self.doc)?;}
            }
            "new"=>{if self.clock.playing(){self.clock.toggle();}self.doc=blank_project();self.clock=editor::playback::Clock::from_document(&self.doc,60.0);self.path=None;self.pick(vec![]);self.selected_keys.clear();self.frame=0;self.saved_signature=snapshot::authored_signature(&self.doc)?;self.color_target=None;}
            "undo"=>{self.doc.undo();self.selected_keys.clear();}
            "redo"=>{self.doc.redo();self.selected_keys.clear();}
            // 履歴の点を押した時。段の番号まで戻る/進むを一手で。
            "historyGoto"=>{
                let target=integer(&j,"head")?;
                while self.doc.edit_head()>target&&self.doc.undo(){}
                while self.doc.edit_head()<target&&self.doc.redo(){}
                if self.doc.edit_head()!=target{return Err("That history point is no longer reachable".into())}
                self.selected_keys.clear();
            }
            "play"=>{if !self.clock.playing(){self.clock.toggle();}self.clock_frame();}
            "pause"=>{if self.clock.playing(){self.clock.toggle();}self.clock_frame();}
            "pickColor"=>{let comp=self.doc.view().composition().map_err(e)?.ok_or("No composition")?;let (w,h)=(comp.width as i64,comp.height as i64);let (x,y)=(num(&j,"x")?.floor() as i64,num(&j,"y")?.floor() as i64);if x<0||y<0||x>=w||y>=h{return Err("Point is outside the composition".into())}let time=self.time()?;let rgba=self.engine.render_frame(&self.doc.view(),time).map_err(e)?;let at=((y*w+x)*4) as usize;let px=rgba.get(at..at+4).ok_or("Frame is smaller than the composition")?;self.picked_color=Some([px[0],px[1],px[2],px[3]].map(|v|v as f64/255.0));self.pick_serial+=1;}
            "seek"=>{self.frame=integer(&j,"frame")?.max(0);self.clock.seek_frame(self.frame);}
            "anchor"=>{let id=layer(&j)?;let b=self.bounds(id).ok_or("Bounds unavailable until rendered")?;let min:[f64;3]=serde_json::from_value(b["localMin"].clone()).map_err(e)?;let max:[f64;3]=serde_json::from_value(b["localMax"].clone()).map_err(e)?;let point=[min[0]+(max[0]-min[0])*num(&j,"xFraction")?,min[1]+(max[1]-min[1])*num(&j,"yFraction")?];let intents=editor::functions::placement::anchor_point_plan(&self.doc,id,self.time()?,point).map_err(e)?;self.apply(intents)?;}
            "freeze"=>{let id=layer(&j)?;self.apply([if j["enabled"].as_bool().ok_or("Missing enabled")?{Intent::Freeze{group:id}}else{Intent::Unfreeze{group:id}}])?;}
            "setGradient"=>{let slot:ColorSlot=serde_json::from_value(j["slot"].clone()).map_err(e)?;let edit=editor::gradient::edit(&self.doc,&slot,&j)?;if j["preview"]==true{self.set_preview(vec![edit])?}else{self.cancel_preview();self.apply([edit])?;}self.color_target=None;}
            "setFillMode"=>{let slot:ColorSlot=serde_json::from_value(j["slot"].clone()).map_err(e)?;if !slot.is_shape_fill(){return Err("Select a shape fill".into())}editor::color::set_shape_gradient(&mut self.doc,&slot,j["gradient"].as_bool().ok_or("Missing gradient")?).map_err(e)?;}
            other=>return Err(format!("Unsupported operation: {other}")),
        }
        if self.doc.revision()!=self.clock_revision {
            self.clock.sync_document(&self.doc);
            self.clock_revision=self.doc.revision();
            self.clock_frame();
        }
        let live=self.doc.view().layers();self.selected_ids.retain(|id|live.contains(id));self.selected_keys.retain(|key|live.contains(&key.layer));if self.color_target.as_ref().is_some_and(|slot|!live.contains(&slot.layer())){self.color_target=None;}self.selected=self.selected_ids.last().copied();self.error=None;Ok(())
    }
    fn clock_frame(&mut self){
        // 尺は上限ではない: 再生は越えて進み、越えた先の層は帯の外なので描かれないだけ。
        self.frame=self.clock.current_frame().max(0);
    }
    fn timing_edits(&self,j:&J)->Result<Vec<Intent>,String>{
        let id=layer(j)?;let old=self.doc.view().without_transients().meta(id).map_err(e)?.ok_or("Layer metadata missing")?.timing;
        let mut next=old;next.start=integer(j,"start")?;next.duration=integer(j,"duration")?;next.source_in=integer(j,"sourceIn")?;
        editor::functions::verb::retime_layer(&self.doc,id,old,next,next.duration==old.duration&&next.source_in==old.source_in).map_err(e)
    }
    fn stage_gesture(&mut self,j:&J)->Result<(),String>{
        match string(j,"phase")?{
            "begin"=>{let interaction=self.preview_tag.take();self.cancel_preview();self.preview_tag=interaction;let ids=ids(&j["ids"])?;let start=serde_json::from_value(j["start"].clone()).map_err(e)?;
                let drag=editor::stage::DragSession::begin(&self.doc,&self.engine,&ids,string(j,"mode")?,j["handle"].as_str().unwrap_or("body"),start,self.time()?,self.view_camera()?)?;
                self.pick(ids);self.stage_drag=Some(drag);
            }
            "update"=>{let drag=self.stage_drag.as_ref().ok_or("No Stage gesture")?;let point=serde_json::from_value(j["point"].clone()).map_err(e)?;
                let edits=drag.edits(&self.doc,point,j["shift"].as_bool().unwrap_or(false),j["alt"].as_bool().unwrap_or(false),self.animate)?;self.set_preview(edits)?;
            }
            "commit"=>{self.stage_drag=None;if let Some((owner,edits))=self.preview.take(){self.doc.clear_preview_edits(owner);self.apply(edits)?;}}
            "cancel"=>self.cancel_preview(),
            _=>return Err("Unknown Stage gesture phase".into()),
        }Ok(())
    }
    fn accept_paste(&mut self,result:editor::clipboard::PasteResult){match result{
        editor::clipboard::PasteResult::Layers(ids)=>{self.pick(ids);self.selected_keys.clear();}
        editor::clipboard::PasteResult::Keys(keys)=>{let mut ids=Vec::new();for k in &keys{if !ids.contains(&k.layer){ids.push(k.layer)}}self.pick(ids);self.selected_keys=keys;}
    }}
    fn apply_ease(&mut self,j:&J)->Result<(),String>{
        if let Some(expected)=j.get("selection") {
            let fps=editor::keyframe_edit::document_fps(&self.doc).map_err(e)?.as_f64();
            let current:Vec<_>=self.selected_keys.iter().map(|key|json!({"layer":key.layer.0,"property":key.property.as_ref().map(|p|p.name()),"frame":(key.at_sec*fps).round()as i64})).collect();
            if *expected!=json!(current){return Err("Easing selection changed".into())}
        }
        if self.selected_keys.is_empty(){return Err("Select keyframes".into())}
        let kind=string(j,"kind")?;
        if kind.starts_with("EasyEase"){
            let side=match kind{"EasyEase"=>editor::keymap::EaseSide::Both,"EasyEaseIn"=>editor::keymap::EaseSide::In,"EasyEaseOut"=>editor::keymap::EaseSide::Out,_=>return Err("Unknown easing".into())};
            editor::ease::apply_easy(&mut self.doc,&self.selected_keys,side)?;
        }else{
            let shape=editor::ease_kinds::decode(j)?;
            let starts=editor::ease::segments(&self.selected_keys);editor::ease::apply(&mut self.doc,&starts,shape)?;
        }Ok(())
    }
    fn edit_marker(&mut self,op:&str,j:&J)->Result<(),String>{
        let mut markers=self.doc.view().markers().map_err(e)?;
        if op=="addMarker"{let at=self.time()?;if !markers.iter().any(|m|m.time==at){markers.push(Marker{name:format!("{}",self.frame),time:at,duration:RationalTime::ZERO,body:String::new()});}}
        else{let id=string(j,"id")?;let index=markers.iter().position(|m|format!("{}/{}",m.time.num(),m.time.den())==id).ok_or("Marker no longer exists")?;if op=="deleteMarker"{markers.remove(index);}else{if let Some(s)=j["name"].as_str(){markers[index].name=s.into()}if let Some(s)=j["body"].as_str(){markers[index].body=s.into()}}}
        markers.sort_by_key(|m|m.time);self.apply([Intent::SetMarkers{markers}])
    }
}

#[cfg(test)]
mod playback_probe {
    /// 再生: play の後に時間が経てば tick で frame が進む。
    #[test]
    fn play_then_tick_advances_the_frame() {
        let path = std::env::var("MOTOLII_PROBE_DOC").unwrap_or_default();
        let mut rt = crate::EditorRuntime::open(&path).unwrap();
        rt.request(serde_json::json!({"op":"play"})).unwrap();
        assert!(rt.clock.playing(), "play の後に clock が走っていない");
        std::thread::sleep(std::time::Duration::from_millis(400));
        rt.request(serde_json::json!({"op":"tick"})).unwrap();
        assert!(rt.frame > 0, "0.4 秒経っても frame が {}", rt.frame);
        // 尺は壁ではない: マウスの seek も尺の先へ行ける(利用者 2026-09-07)。
        rt.request(serde_json::json!({"op":"pause"})).unwrap();
        rt.request(serde_json::json!({"op":"seek","frame":9_999})).unwrap();
        assert_eq!(rt.frame, 9_999);
        rt.request(serde_json::json!({"op":"play"})).unwrap();
        // 再生中の軽い status にも、Swift の render が毎コマ読む寸法が要る。
        let full = rt.status().unwrap();
        let light = rt.status().unwrap();
        assert!(light["liveLayers"].is_array(), "2 回目は軽い status のはず: {light}");
        for key in ["width", "height", "fps", "durationFrames"] {
            assert_eq!(light[key], full[key], "軽い status に {key} が無い");
        }
    }
}

#[cfg(test)]
mod sequence_preview {
    use crate::doc::store::*;

    /// Ease(Sequence)を掴んでいる間の下書き: previewSequence で status のゴーストが変わり、
    /// cancel で戻り、sequence で本書きになる。
    #[test]
    fn preview_sequence_shows_in_status_and_cancels() {
        let mut rt = crate::EditorRuntime::open("").unwrap();
        for id in [11u64, 12] {
            rt.doc.apply_all([
                Intent::AddLayer(LayerId(id)),
                Intent::SetMeta { layer: LayerId(id), meta: LayerMeta { source: LayerSource::Shape, order: id as i16, timing: LayerTiming::place(0, None, 60) } },
            ]).unwrap();
        }
        let ghost_of = |rt: &crate::EditorRuntime, id: u64| -> serde_json::Value {
            let status = rt.status().unwrap();
            status["layers"].as_array().unwrap().iter().find(|l| l["id"] == id).map(|l| l["ghost"].clone()).unwrap()
        };
        rt.request(serde_json::json!({"op":"previewSequence","layers":[11,12],"ghosts":[0,7]})).unwrap();
        assert_eq!(rt.doc.view().attrs(LayerId(12)).unwrap().unwrap().ghost, Some(7), "view().attrs に下書きが乗る");
        assert_eq!(ghost_of(&rt, 12), serde_json::json!(7), "下書きが status に出る");
        assert!(ghost_of(&rt, 11).is_null());
        rt.request(serde_json::json!({"op":"cancelPreview"})).unwrap();
        assert!(ghost_of(&rt, 12).is_null(), "cancel で戻る");
        rt.request(serde_json::json!({"op":"sequence","layers":[11,12],"ghosts":[0,7]})).unwrap();
        assert_eq!(ghost_of(&rt, 12), serde_json::json!(7), "本書き");
        assert_eq!(rt.doc.view().attrs(LayerId(12)).unwrap().unwrap().ghost, Some(7));
    }

    /// 姿を持たない層(Null・HDR・音声・カメラ)はゴーストを持てない(裁定 2026-09-07 利用者「旨みがない」)。
    /// status は `ghostable` で知らせ、sequence はその層を飛ばし、setAttrs は断る。
    #[test]
    fn layers_without_a_look_cannot_carry_a_ghost() {
        use crate::doc::store::{Intent, LayerId, LayerMeta, LayerSource, LayerTiming};
        let mut rt = crate::EditorRuntime::open("").unwrap();
        let sources = [
            (21u64, LayerSource::Shape),
            (22, LayerSource::Null),
            (23, LayerSource::File { path: "sky.hdr".into(), fingerprint: None }),
            (24, LayerSource::File { path: "voice.wav".into(), fingerprint: None }),
            (25, LayerSource::Camera),
        ];
        for (id, source) in sources {
            rt.doc.apply_all([
                Intent::AddLayer(LayerId(id)),
                Intent::SetMeta { layer: LayerId(id), meta: LayerMeta { source, order: id as i16, timing: LayerTiming::place(0, None, 60) } },
            ]).unwrap();
        }
        let status = rt.status().unwrap();
        let ghostable = |id: u64| status["layers"].as_array().unwrap().iter().find(|l| l["id"] == id).unwrap()["ghostable"] == true;
        assert!(ghostable(21));
        for id in [22, 23, 24, 25] { assert!(!ghostable(id), "layer {id}"); }
        rt.request(serde_json::json!({"op":"sequence","layers":[21,22,23,24],"ghosts":[0,3,6,9]})).unwrap();
        for id in [22, 23, 24] {
            assert_eq!(rt.doc.view().attrs(LayerId(id)).unwrap().unwrap_or_default().ghost, None, "layer {id} は飛ばされる");
        }
        assert!(rt.request(serde_json::json!({"op":"setAttrs","layers":[24],"patch":{"ghost":5}})).is_err(), "音声に直接付けても断る");
        // 環境層は doc 自身が断り、環境にした瞬間に既存のゴーストも落ちる。
        rt.request(serde_json::json!({"op":"setAttrs","layers":[21],"patch":{"ghost":5}})).unwrap();
        rt.request(serde_json::json!({"op":"setAttrs","layers":[21],"patch":{"environment":true}})).unwrap();
        assert_eq!(rt.doc.view().attrs(LayerId(21)).unwrap().unwrap().ghost, None);
        assert!(rt.request(serde_json::json!({"op":"setAttrs","layers":[21],"patch":{"ghost":5}})).is_err());
    }
}


#[cfg(test)]
mod blend_interaction_tests {
    use super::*;
    #[test]
    fn blend_preview_batches_layers_and_cannot_cancel_a_newer_interaction() {
        let mut rt=EditorRuntime::open("").unwrap();
        rt.request(json!({"op":"create","kind":"rectangle"})).unwrap();
        let a=rt.selected.unwrap();
        rt.request(json!({"op":"create","kind":"rectangle"})).unwrap();
        let b=rt.selected.unwrap();
        let history=rt.doc.history_depth();
        rt.request(json!({"op":"previewBlend","layers":[a.0,b.0],"mode":"Multiply","interaction":"desk"})).unwrap();
        let owner=rt.preview.as_ref().unwrap().0;
        assert_eq!(rt.doc.history_depth(),history);
        assert_eq!(rt.preview.as_ref().unwrap().1.len(),2);
        rt.request(json!({"op":"commitPreview","owner":owner})).unwrap();
        assert_eq!(rt.doc.history_depth().0,history.0+1);
        assert_eq!(rt.doc.view().attrs(a).unwrap().unwrap().blend_mode,BlendMode::Multiply);
        assert_eq!(rt.doc.view().attrs(b).unwrap().unwrap().blend_mode,BlendMode::Multiply);
        rt.request(json!({"op":"previewBlend","layers":[a.0],"mode":"Screen","interaction":"desk"})).unwrap();
        let owner=rt.preview.as_ref().unwrap().0;
        rt.request(json!({"op":"previewProperties","interaction":"inspector","edits":[{"layer":a.0,"property":"opacity","value":0.5}]})).unwrap();
        let newer=rt.preview.as_ref().unwrap().0;
        assert_ne!(owner,newer);
        rt.request(json!({"op":"cancelPreview","owner":owner})).unwrap();
        assert_eq!(rt.preview.as_ref().unwrap().0,newer);
    }
}
