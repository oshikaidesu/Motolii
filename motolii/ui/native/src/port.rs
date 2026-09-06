use crate::{EditorRuntime,editor,snapshot};
use crate::doc::store::*;
use serde_json::{Value as J,json};
use editor::session::{KeySel,ColorSlot};
fn e(error:impl std::fmt::Display)->String{error.to_string()}
pub(crate) const CAPABILITIES:&[&str]=&["status","notes","stageView","select","setProperty","previewProperties","commitPreview","cancelPreview","setText","setAttrs","create","duplicate","copy","cut","paste","delete","group","ungroup","reorder","split","setTiming","toggleKey","moveKeys","ease","setColor","previewColor","focusColor","applyPalette","applyEffect","removeEffect","clip","addMarker","setMarker","deleteMarker","composition","import","placeAsset","removeAsset","replaceAsset","save","new","undo","redo","seek","anchor","freeze","setFillMode","previewBlend","setTimings","previewTimings","stageGesture","export","exportStatus","cancelExport","play","pause","tick","moveLayers"];
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
    boolean!("hidden",hidden);boolean!("solo",solo);boolean!("locked",locked);boolean!("flatten",flatten);boolean!("clipToBelow",clip_to_below);boolean!("autoOrient",auto_orient);
    if let Some(s)=j["name"].as_str(){p.name=Some(s.into());}
    if j.get("parent").is_some(){p.parent=Some(if j["parent"].is_null(){None}else{Some(LayerId(j["parent"].as_u64().ok_or("Invalid parent")?))});}
    if j.get("blendMode").is_some(){p.blend_mode=Some(serde_json::from_value(j["blendMode"].clone()).map_err(e)?);}
    if j.get("projection").is_some(){p.projection=Some(serde_json::from_value(j["projection"].clone()).map_err(e)?);}
    if j.get("matte").is_some(){p.matte=Some(serde_json::from_value(j["matte"].clone()).map_err(e)?);}
    Ok(p)
}
impl EditorRuntime{
    fn pick(&mut self,ids:Vec<LayerId>){self.selected_ids=ids;self.selected=self.selected_ids.last().copied();}
    fn selected_required(&self)->Result<&[LayerId],String>{if self.selected_ids.is_empty(){Err("Select layers".into())}else{Ok(&self.selected_ids)}}
    fn apply(&mut self,intents:impl IntoIterator<Item=Intent>)->Result<(),String>{self.doc.apply_all(intents).map_err(e)}
    fn set_preview(&mut self,intents:Vec<Intent>)->Result<(),String>{
        let owner=if let Some((owner,_))=&self.preview{*owner}else{self.doc.begin_preview()};
        let projected:Vec<_>=intents.iter().filter_map(|intent|match intent{
            Intent::SetShapes{..}=>None,
            Intent::SetAttrs{layer,patch} if patch.blend_mode.is_some()=>Some(Intent::SetConstant{layer:*layer,property:PropertyId::blend_mode(),value:Value::Enum(patch.blend_mode.unwrap().to_enum_value())}),
            _=>Some(intent.clone()),
        }).collect();
        self.doc.preview_edits(owner,&projected).map_err(e)?;self.preview=Some((owner,intents));Ok(())
    }
    fn property_edits(&self,j:&J)->Result<Vec<Intent>,String>{
        let layer=layer(j)?;let name=string(j,"property")?;let p=PropertyId::new(name).map_err(e)?;
        let view=self.doc.view().without_transients();
        let current=view.value_at(layer,&p,self.time()?).map_err(e)?.or(view.default_value(layer,&p).map_err(e)?);
        let mut v=decoded_value(&j["value"],current.as_ref())?;
        let catalog=crate::render::engine::known_effects();let data=editor::functions::read::inspector_data_from_doc(&view,layer,self.time()?,&catalog);
        let rows=data.text.iter().chain(data.transform.iter()).chain(data.effects.iter().flat_map(|e|e.params.iter()));
        if let Some(row)=rows.filter(|r|r.property.as_deref()==Some(name)).next(){if let(Value::F64(n),Some((min,max)))=(&mut v,row.range){*n=n.clamp(min,max);}}
        self.doc.place_checked(layer,&p,v,self.time()?).map(|v|v.into_iter().collect()).map_err(e)
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
    fn create_layer(&mut self,kind:editor::create::NewKind)->Result<(),String>{
        let view=self.doc.view();let comp=view.composition().map_err(e)?.ok_or("No composition")?;
        let id=LayerId(view.next_layer_id());let order=view.layers().iter().filter_map(|l|view.meta(*l).ok().flatten().map(|m|m.order)).max().unwrap_or(-1).checked_add(1).ok_or("Layer order full")?;
        let taken:Vec<_>=view.layers().iter().filter_map(|l|view.attrs(*l).ok().flatten().map(|a|a.name)).collect();
        let mut intents=editor::create::new_layer_intents(id,order,self.frame,comp.duration_frames,comp.fps,(comp.width as f64,comp.height as f64),kind);
        if intents.iter().any(|i| matches!(i, Intent::SetMeta { meta, .. } if meta.source == LayerSource::Camera)) {
            let camera = view.resolve_camera(self.time()?).map_err(e)?;
            for (name,value) in [(property::CAMERA_CENTER,Value::Vec2(camera.center.map(f64::from))), (property::CAMERA_ZOOM,Value::F64(camera.zoom as f64)), (property::CAMERA_ROLL,Value::F64(camera.roll_degrees as f64))] {
                intents.push(Intent::SetConstant { layer:id, property:PropertyId::new(name).map_err(e)?, value });
            }
        }
        for i in &mut intents{if let Intent::SetAttrs{patch,..}=i{if let Some(name)=&mut patch.name{*name=editor::create::numbered(name,&taken)}}}
        self.apply(intents)?;self.pick(vec![id]);self.selected_keys.clear();Ok(())
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
            return Ok(());
        }

        if op=="tick"{self.clock_frame();self.error=None;return Ok(())}
        if !matches!(op,"previewProperties"|"previewColor"|"previewBlend"|"previewX"|"commitPreview"|"commitX"|"previewTimings"|"stageGesture"){self.cancel_preview();}
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
            "setText"=>{let id=layer(&j)?;if self.doc.view().text_document(id).map_err(e)?.is_none(){return Err("Select a Text layer".into())}let at=self.time()?;editor::text::write_content(&mut self.doc,id,at,string(&j,"content")?.into()).map_err(e)?;}
            "setAttrs"=>{let patch=attrs_patch(&j["patch"])?;self.apply(ids(&j["layers"] )?.into_iter().map(|layer|Intent::SetAttrs{layer,patch:patch.clone()}))?;}
            "create"=>{let kind=match string(&j,"kind")?{"text"=>editor::create::NewKind::Text,"rectangle"=>editor::create::NewKind::Rectangle,"bezier"=>editor::create::NewKind::Bezier,"cube"=>editor::create::cube()?,"camera"=>editor::create::NewKind::Camera,_=>return Err("Unsupported create kind".into())};self.create_layer(kind)?;}
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
            "removeEffect"=>{let id=layer(&j)?;let effect=integer(&j,"id")?as u32;let mut effects=self.doc.view().effects(id).map_err(e)?;if !effects.iter().any(|e|e.id.0==effect){return Err("Effect missing".into())}effects.retain(|e|e.id.0!=effect);self.apply([Intent::SetEffects{layer:id,effects}])?;}
            "clip"=>{let id=layer(&j)?;let clipped=self.doc.view().attrs(id).map_err(e)?.unwrap_or_default().clip_to_below;self.apply([Intent::SetAttrs{layer:id,patch:LayerAttrsPatch{clip_to_below:Some(!clipped),..Default::default()}}])?;}
            "previewBlend"=>{let id=layer(&j)?;let mode:BlendMode=serde_json::from_value(j["mode"].clone()).map_err(e)?;self.set_preview(vec![Intent::SetAttrs{layer:id,patch:LayerAttrsPatch{blend_mode:Some(mode),..Default::default()}}])?;}
            "addMarker"|"setMarker"|"deleteMarker"=>self.edit_marker(op,&j)?,
            "composition"=>{let current=self.doc.view().composition().map_err(e)?.ok_or("No composition")?;let width:u32=integer(&j,"width")?.try_into().map_err(e)?;let height:u32=integer(&j,"height")?.try_into().map_err(e)?;let fps=Fps::try_new(integer(&j,"fpsNum")?,integer(&j,"fpsDen")?).map_err(e)?;let duration_frames=integer(&j,"durationFrames")?;let background=if j.get("background").is_some(){serde_json::from_value(j["background"].clone()).map_err(e)?}else{current.background};self.apply([Intent::SetComposition(Composition{width,height,fps,duration_frames,background})])?;}
            "import"=>{let paths:Vec<std::path::PathBuf>=j["paths"].as_array().ok_or("Expected paths")?.iter().map(|p|p.as_str().map(std::path::PathBuf::from).ok_or("Invalid path".into())).collect::<Result<_,String>>()?;let paths=editor::fixture::expand_folders(&paths);if paths.is_empty(){return Err("No files to import".into())}let mut intents=Vec::new();let mut known:std::collections::HashSet<_>=self.doc.view().assets().map_err(e)?.iter().map(|a|a.content_hash.clone()).collect();for path in paths{let draft=editor::fixture::prepare_path(&path,if j["role"]=="reference"{AssetRole::Reference}else{AssetRole::Material})?;if known.insert(draft.content_hash.clone()){intents.push(Intent::AdmitAsset{draft});}}self.apply(intents)?;}
            "placeAsset"|"replaceAsset"=>{let a=self.doc.view().asset(asset_id(&j)?).map_err(e)?.ok_or("Asset missing")?;let path=a.path_absolute.ok_or("Asset path missing")?;if !std::path::Path::new(&path).exists(){return Err("Asset file missing".into())}if op=="placeAsset"{self.create_layer(editor::create::NewKind::Media{path,name:a.name})?;}else{let layer=self.selected.ok_or("Select layer to replace")?;self.apply([Intent::SetSource{layer,source:LayerSource::File{path,fingerprint:None}}])?;}}
            "removeAsset"=>{let id=asset_id(&j)?;if self.asset_used(id)?{return Err("Asset is still in use".into())}self.apply([Intent::RemoveAsset{asset:id}])?;}
            "export"=>self.exporter.start(&self.doc,std::path::PathBuf::from(string(&j,"path")?),integer(&j,"start")?,integer(&j,"end")?)?,
            "cancelExport"=>self.exporter.cancel(),
            "save"=>{let path=string(&j,"path")?;self.doc.save(path).map_err(e)?;self.path=Some(path.into());self.saved_signature=snapshot::authored_signature(&self.doc)?;}
            "new"=>{if self.clock.playing(){self.clock.toggle();}self.doc=blank_project();self.clock=editor::playback::Clock::from_document(&self.doc,60.0);self.path=None;self.pick(vec![]);self.selected_keys.clear();self.frame=0;self.saved_signature=snapshot::authored_signature(&self.doc)?;self.color_target=None;}
            "undo"=>{self.doc.undo();self.selected_keys.clear();}
            "redo"=>{self.doc.redo();self.selected_keys.clear();}
            "play"=>{if !self.clock.playing(){self.clock.toggle();}self.clock_frame();}
            "pause"=>{if self.clock.playing(){self.clock.toggle();}self.clock_frame();}
            "seek"=>{let c=self.doc.view().composition().map_err(e)?.ok_or("No composition")?;self.frame=integer(&j,"frame")?.clamp(0,c.duration_frames.saturating_sub(1).max(0));self.clock.seek_frame(self.frame);}
            "anchor"=>{let id=layer(&j)?;let b=self.bounds(id).ok_or("Bounds unavailable until rendered")?;let min:[f64;3]=serde_json::from_value(b["localMin"].clone()).map_err(e)?;let max:[f64;3]=serde_json::from_value(b["localMax"].clone()).map_err(e)?;let point=[min[0]+(max[0]-min[0])*num(&j,"xFraction")?,min[1]+(max[1]-min[1])*num(&j,"yFraction")?];let intents=editor::functions::placement::anchor_point_plan(&self.doc,id,self.time()?,point).map_err(e)?;self.apply(intents)?;}
            "freeze"=>{let id=layer(&j)?;self.apply([if j["enabled"].as_bool().ok_or("Missing enabled")?{Intent::Freeze{group:id}}else{Intent::Unfreeze{group:id}}])?;}
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
        let frame=self.clock.current_frame();
        let end=self.doc.view().composition().ok().flatten().map_or(0,|c|c.duration_frames.saturating_sub(1).max(0));
        self.frame=frame.clamp(0,end);
    }
    fn timing_edits(&self,j:&J)->Result<Vec<Intent>,String>{
        let id=layer(j)?;let old=self.doc.view().without_transients().meta(id).map_err(e)?.ok_or("Layer metadata missing")?.timing;
        let mut next=old;next.start=integer(j,"start")?;next.duration=integer(j,"duration")?;next.source_in=integer(j,"sourceIn")?;
        editor::functions::verb::retime_layer(&self.doc,id,old,next,next.duration==old.duration&&next.source_in==old.source_in).map_err(e)
    }
    fn stage_gesture(&mut self,j:&J)->Result<(),String>{
        match string(j,"phase")?{
            "begin"=>{self.cancel_preview();let ids=ids(&j["ids"])?;let start=serde_json::from_value(j["start"].clone()).map_err(e)?;
                let drag=editor::stage::DragSession::begin(&self.doc,&self.engine,&ids,string(j,"mode")?,j["handle"].as_str().unwrap_or("body"),start,self.time()?,self.view_camera()?)?;
                self.pick(ids);self.stage_drag=Some(drag);
            }
            "update"=>{let drag=self.stage_drag.as_ref().ok_or("No Stage gesture")?;let point=serde_json::from_value(j["point"].clone()).map_err(e)?;
                let edits=drag.edits(&self.doc,point,j["shift"].as_bool().unwrap_or(false),j["alt"].as_bool().unwrap_or(false))?;self.set_preview(edits)?;
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
