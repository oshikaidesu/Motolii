#[allow(unused_imports)]
use crate::edit::{Animate, Document, Intent};
use crate::{EditorRuntime,editor,snapshot};
use crate::doc::store::*;
use serde_json::{Value as J,json};
use crate::viewer::{KeySel,ColorSlot};
fn e(error:impl std::fmt::Display)->String{error.to_string()}
pub(crate) const CAPABILITIES:&[&str]=&["status","preferences","notes","stageView","stageWindow","select","setProperty","previewProperties","commitPreview","cancelPreview","setText","previewText","styleText","setFont","setAttrs","create","duplicate","ghost","sequence","previewSequence","copy","cut","paste","delete","group","ungroup","reorder","split","setTiming","toggleKey","moveKeys","ease","setColor","previewColor","focusColor","applyPalette","applyEffect","removeEffect","expandEffect","moveEffect","enableEffect","animate","clip","addMarker","setMarker","deleteMarker","composition","import","placeAsset","removeAsset","replaceAsset","relinkAsset","save","new","undo","redo","seek","anchor","freeze","setFillMode","setGradient","previewBlend","setTimings","previewTimings","stageGesture","export","exportStatus","cancelExport","play","pause","tick","moveLayers","pickColor","historyGoto","reloadEffects","runScript","rerunScript","scopeEffect"];
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
    fn pick(&mut self,ids:Vec<LayerId>){let changed=self.viewer.selected_ids!=ids;self.viewer.selected_ids=ids;if changed{self.viewer.color_target=self.viewer.selected().and_then(|id|editor::color::default_target(&self.doc,id));}}
    fn selected_required(&self)->Result<&[LayerId],String>{if self.viewer.selected_ids.is_empty(){Err("Select layers".into())}else{Ok(&self.viewer.selected_ids)}}
    fn apply(&mut self,intents:impl IntoIterator<Item=Intent>)->Result<(),String>{
        let intents:Vec<_>=intents.into_iter().collect();
        let born=self.born_spans(&intents);
        self.doc.apply_all(intents).map_err(e)?;
        if !born.is_empty(){self.viewer.selected_keys=born;}
        Ok(())
    }
    /// The pairs Animate From is about to give birth to: both ends of a track a property did not have yet.
    /// They become the key selection, so the Ease desk lands on the new motion without a trip to the Timeline.
    fn born_spans(&self,intents:&[Intent])->Vec<KeySel>{
        let Animate::From{origin,..}=self.viewer.animate else{return Vec::new()};
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
        let current=view.value_at(layer,&p,self.time()?).map_err(e)?.or(motolii_edit::document::edit::default_value(&view, layer,&p).map_err(e)?);
        let catalog=crate::render::engine::known_effects();let data=editor::functions::read::inspector_data_from_doc(&view,layer,self.time()?,&catalog);
        let rows=data.text.iter().chain(data.transform.iter()).chain(data.effects.iter().flat_map(|e|e.params.iter()));
        let row=rows.filter(|r|r.property.as_deref()==Some(name)).next();
        let mut v=decoded_value(&j["value"],current.as_ref().or(row.map(|r|&r.value)))?;
        if let Some(row)=row{if let(Value::F64(n),Some((min,max)))=(&mut v,row.range){*n=n.clamp(min,max);}}
        self.doc.place_checked(layer,&p,v,self.time()?,self.viewer.animate).map(|v|v.into_iter().collect()).map_err(e)
    }
    fn color_intent(&self,j:&J)->Result<Vec<Intent>,String>{
        let slot:ColorSlot=serde_json::from_value(j["slot"].clone()).map_err(e)?;
        if j.get("layer").is_some()&&slot.layer()!=Some(layer(j)?){return Err("Color target layer mismatch".into())}
        if let Some(id)=slot.layer(){if let Some(reason)=editor::functions::lens::edit_rejection(&self.doc.view(),id).map_err(e)?{return Err(reason.into())}}
        editor::color::write_color(&self.doc,&slot,rgba(j)?,self.time()?,self.viewer.animate).map_err(e)
    }
    pub(crate) fn asset_used(&self,id:AssetId)->Result<bool,String>{
        let view=self.doc.view();let a=view.asset(id).map_err(e)?.ok_or("Asset missing")?;
        for layer in view.layers(){if let Some(meta)=view.meta(layer).map_err(e)?{if let LayerSource::File{path,..}=meta.source{if a.path_absolute.as_deref()==Some(&path){return Ok(true)}}}}
        Ok(false)
    }
    fn create_layer(&mut self,kind:editor::create::NewKind,visible:Option<i64>,family:Option<&str>)->Result<(),String>{self.place_layer(kind,self.viewer.frame,None,visible,family)}
    /// 置く場所を指す作成。drop 先は moveLayers と同じ語彙(target/placement)、開始コマは落とした x。作る→並べるを 1 手(Undo 一発)に。
    /// `family` は文字の層をその書体で作る(Fonts 棚の行を押した時): 作る→着せるを 1 手に。
    fn place_layer(&mut self,kind:editor::create::NewKind,start:i64,landing:Option<(Option<LayerId>,String)>,visible:Option<i64>,family:Option<&str>)->Result<(),String>{
        let view=self.doc.view();let comp=view.composition().map_err(e)?.ok_or("No composition")?;
        let id=LayerId(view.next_layer_id());let order=view.layers().iter().filter_map(|l|view.meta(*l).ok().flatten().map(|m|m.order)).max().unwrap_or(-1).checked_add(1).ok_or("Layer order full")?;
        let taken:Vec<_>=view.layers().iter().filter_map(|l|view.attrs(*l).ok().flatten().map(|a|a.name)).collect();
        let mut intents=editor::create::new_layer_intents(id,order,start,comp.duration_frames,comp.fps,(comp.width as f64,comp.height as f64),kind,editor::create::unbounded_frames(visible));
        editor::create::prefer_projection(&mut intents,self.flat_projection);
        if let Some(family)=family{
            if !crate::render::picture::shaping::font_families().iter().any(|name|name==family){return Err("Font family is not installed".into())}
            for i in &mut intents{if let Intent::SetTextDocument{document,..}=i{for style in &mut document.styles{style.font=crate::doc::store::FontRef{family:family.into(),..Default::default()};}}}
        }
        if intents.iter().any(|i| matches!(i, Intent::SetMeta { meta, .. } if meta.source == LayerSource::Camera)) {
            let camera_time=self.time()?;
            let camera = self.engine.frame_graph_document_camera(&view,camera_time).map_err(e)?;
            for (name,value) in property::camera_values(&camera) {
                intents.push(Intent::SetConstant { layer:id, property:PropertyId::new(name).map_err(e)?, value });
            }
        }
        for i in &mut intents{if let Intent::SetAttrs{patch,..}=i{if let Some(name)=&mut patch.name{*name=editor::create::numbered(name,&taken)}}}
        let at=self.time()?;drop(view);
        match landing{None=>self.apply(intents)?,Some((target,placement))=>self.doc.apply_then(intents,|doc|doc.move_layer_intents(&[id],target,&placement,at).map(|(i,_)|i)).map_err(e)?}
        self.pick(vec![id]);self.viewer.selected_keys.clear();Ok(())
    }
    fn delete_selection(&mut self)->Result<(),String>{
        if !self.viewer.selected_keys.is_empty(){let keys:Vec<_>=self.viewer.selected_keys.iter().map(|k|(k.layer,k.property.clone(),k.at_sec)).collect();let edits=editor::timeline_edit::delete_key_selection_intents(&self.doc,&keys,self.time()?).map_err(e)?;self.apply(edits)?;self.viewer.selected_keys.clear();}
        else{let ids=self.selected_required()?.to_vec();self.apply(ids.into_iter().map(Intent::RemoveLayer))?;self.pick(vec![]);}
        Ok(())
    }
    pub(crate) fn request(&mut self,j:J)->Result<(),String>{
        self.viewer.clock.poll_audio(&self.doc.view());
        let op=j["op"].as_str().unwrap_or("status");
        // 棚(vism/)が変わっていれば読み直す。変わっていなければ lock 1 回で戻る。
        // 見張りの起こしは reloadEffects で来るが、他の操作の途中で変わっても次の請求で拾う。
        crate::render::engine::refresh_effect_catalog();
        // 焼いている間は disk にコマが増える: 「無い」の記憶を捨てて見に行く。
        if self.freezer.running().is_some(){self.engine.refresh_frozen();}
        if op=="status"||op=="exportStatus"||op=="reloadEffects"{return Ok(())}
        if op=="stageView" {
            self.cancel_preview();
            if let Some(orbit)=j["orbit"].as_array() {
                if orbit.len()!=2 { return Err("Expected two orbit angles".into()); }
                self.viewer.user_camera.orbit_degrees=[orbit[0].as_f64().filter(|v|v.is_finite()).ok_or("Invalid orbit")? as f32,orbit[1].as_f64().filter(|v|v.is_finite()).ok_or("Invalid orbit")? as f32];
            }
            if let Some(id)=j["focus"].as_u64() {
                let comp=self.doc.view().composition().map_err(e)?.ok_or("No composition")?.spec();
                let (centre,radius)=self.focus_sphere(LayerId(id))?.ok_or("Layer has no bounds to focus")?;
                self.viewer.user_camera=self.viewer.user_camera.looking_at(comp,centre,radius);
            }
            if j["reset"].as_bool()==Some(true) { self.viewer.user_camera=Default::default(); }
            if j["fit"].as_bool()==Some(true) {
                let view=self.doc.view();let comp=view.composition().map_err(e)?.ok_or("No composition")?.spec();
                let [x,y,w,h]=motolii_render::picture::resolve::camera::resolve_stage_extent(&view, self.time()?).map_err(e)?.rect(comp);
                self.viewer.user_camera=crate::doc::core::ResolvedCamera { center:[x+w*0.5-comp.width as f32*0.5,y+h*0.5-comp.height as f32*0.5], distance_scale:(w/comp.width as f32).max(h/comp.height as f32), ..Default::default() };
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
                self.pick(chosen);self.viewer.selected_keys.clear();
                if let Some(keys)=j["keys"].as_array(){let fps=editor::keyframe_edit::document_fps(&self.doc).map_err(e)?.as_f64();
                    for k in keys{self.viewer.selected_keys.push(KeySel{layer:LayerId(k["layer"].as_u64().ok_or("Invalid key layer")?),property:k["property"].as_str().map(PropertyId::new).transpose().map_err(e)?,at_sec:integer(k,"frame")?as f64/fps});}
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
                    let centers:Vec<_>={let view=self.doc.view();let resolved=self.engine.frame_graph_editor_layers(&view,at).map_err(e)?;
                        layers.iter().map(|&id|(id,self.engine.selected_layer_bounds_in(&view,&resolved,id,at).map(|b|b.center()).unwrap_or([0.0;3]))).collect()};
                    self.doc.set_projection(&centers,patch,at).map_err(e)?;
                } else {self.apply(layers.into_iter().map(|layer|Intent::SetAttrs{layer,patch:patch.clone()}))?;}}
            "create"=>{let kind=match string(&j,"kind")?{"text"=>editor::create::NewKind::Text,"rectangle"=>editor::create::NewKind::Rectangle,"roundedRectangle"=>editor::create::NewKind::RoundedRectangle,"ellipse"=>editor::create::NewKind::Ellipse,"star"=>editor::create::NewKind::Star,"polygon"=>editor::create::NewKind::Polygon,"line"=>editor::create::NewKind::Line,"bezier"=>editor::create::NewKind::Bezier,"null"=>editor::create::NewKind::Null,"camera"=>editor::create::NewKind::Camera,"stage"=>editor::create::NewKind::Stage,"particles"=>editor::create::NewKind::Particles,k=>match k.strip_prefix("background:"){Some(id)=>editor::create::background(id)?,None=>editor::create::primitive(k)?}};let family=j["family"].as_str().map(str::to_owned);self.create_layer(kind,j["visibleFrames"].as_i64(),family.as_deref())?;}
            "copy"|"cut"=>{
                if self.viewer.selected_keys.is_empty(){self.clipboard.copy_layers(&self.doc,self.selected_required()?).map_err(e)?;}else{self.clipboard.copy_keys(&self.doc,&self.viewer.selected_keys).map_err(e)?;}
                if op=="cut"{self.delete_selection()?;}
            }
            "paste"=>{let result=self.clipboard.paste(&mut self.doc,self.viewer.selected(),self.viewer.frame).map_err(e)?;self.accept_paste(result);}
            "duplicate"=>{
                if self.viewer.selected_keys.is_empty(){let ids=self.selected_required()?.to_vec();let copies=editor::timeline_edit::duplicate_layers(&mut self.doc,&ids).map_err(e)?;self.pick(copies);}
                else{let c=editor::clipboard::Clipboard::default();c.copy_keys(&self.doc,&self.viewer.selected_keys).map_err(e)?;let fps=editor::keyframe_edit::document_fps(&self.doc).map_err(e)?.as_f64();let at=self.viewer.selected_keys.iter().map(|k|(k.at_sec*fps).round()as i64).max().unwrap_or(self.viewer.frame).saturating_add(1);let result=c.paste(&mut self.doc,None,at).map_err(e)?;self.accept_paste(result);}
            }
            "delete"=>self.delete_selection()?,
            "group"=>{let ids=self.selected_required()?.to_vec();let group=self.doc.group_layers(&ids).map_err(e)?.ok_or("No group created")?;self.pick(vec![group]);self.viewer.selected_keys.clear();}
            "ungroup"=>{let ids=self.selected_required()?.to_vec();let children=self.doc.ungroup_layers(&ids).map_err(e)?;if children.is_empty(){return Err("Select a Group".into())}self.pick(children);self.viewer.selected_keys.clear();}
            "moveLayers"=>{let layers=ids(&j["layers"])?;let target=if j["target"].is_null(){None}else{Some(LayerId(j["target"].as_u64().ok_or("Invalid drop target")?))};let at=self.time()?;let moved=self.doc.move_layers(&layers,target,string(&j,"placement")?,at).map_err(e)?;self.pick(moved);self.viewer.selected_keys.clear();}
            "reorder"=>{let id=self.viewer.selected().ok_or("Select a layer")?;let order=self.doc.view().meta(id).map_err(e)?.ok_or("Layer metadata missing")?.order;let delta:i16=integer(&j,"delta")?.try_into().map_err(e)?;self.apply([Intent::SetOrder{layer:id,order:order.saturating_add(delta)}])?;}
            "split"=>{let ids=self.selected_required()?.to_vec();let copies=editor::timeline_edit::split_layers(&mut self.doc,&ids,self.viewer.frame).map_err(e)?;if copies.is_empty(){return Err("Playhead must be inside a layer".into())}self.pick(copies);}
            "setTimings"|"previewTimings"=>{let mut edits=Vec::new();for change in j["changes"].as_array().ok_or("Missing timing changes")?{edits.extend(self.timing_edits(change)?);}if op=="previewTimings"{self.set_preview(edits)?;}else{self.apply(edits)?;}}
            "stageGesture"=>self.stage_gesture(&j)?,
            "setTiming"=>{let id=layer(&j)?;let old=self.doc.view().meta(id).map_err(e)?.ok_or("Layer metadata missing")?.timing;let mut next=old;next.start=integer(&j,"start")?;next.duration=integer(&j,"duration")?;next.source_in=integer(&j,"sourceIn")?;let move_keys=next.duration==old.duration&&next.source_in==old.source_in;let edits=editor::functions::verb::retime_layer(&self.doc,id,old,next,move_keys).map_err(e)?;self.apply(edits)?;}
            "toggleKey"=>{let id=layer(&j)?;let name=string(&j,"property")?;let at=self.time()?;
                if name=="content"{editor::text::toggle_content_key(&mut self.doc,id,at,String::new()).map_err(e)?;}
                else{let p=PropertyId::new(name).map_err(e)?;editor::functions::lens::require_local_source(&self.doc.view(),id,&p).map_err(e)?;let track=self.doc.view().track(id,&p).map_err(e)?.unwrap_or_default();
                    if track.keys().iter().any(|k|k.t.try_to_frame_round(editor::keyframe_edit::document_fps(&self.doc).unwrap()).ok()==Some(self.viewer.frame)){let edits=editor::timeline_edit::delete_key_selection_intents(&self.doc,&[(id,Some(p),at.as_seconds_f64())],at).map_err(e)?;self.apply(edits)?;}
                    else{let data=editor::functions::read::inspector_data_from_doc(&self.doc.view(),id,at,&crate::render::engine::known_effects());let fallback=data.transform.iter().chain(data.text.iter()).chain(data.effects.iter().flat_map(|e|e.params.iter())).find(|r|r.property.as_deref()==Some(name)).map(|r|r.value.clone());let value=self.doc.view().value_at(id,&p,at).map_err(e)?.or(crate::edit::document::edit::default_value(&self.doc.view(),id,&p).map_err(e)?).or(fallback).ok_or("No keyframe value")?;let mut track=track;track.insert(Keyframe{t:at,value,interp:Interp::Linear,spatial:None});self.apply([Intent::SetTrack{layer:id,property:p,track}])?;}
                }
            }
            "moveKeys"=>{if self.viewer.selected_keys.is_empty(){return Err("Select keyframes".into())}let keys:Vec<_>=self.viewer.selected_keys.iter().map(|k|(k.layer,k.property.clone(),k.at_sec)).collect();let fps=editor::keyframe_edit::document_fps(&self.doc).map_err(e)?.as_f64();let delta=editor::keyframe_edit::clamped_key_delta(&keys,fps,integer(&j,"deltaFrames")?);let edits=editor::keyframe_edit::key_selection_move_intents(&self.doc,&keys,delta).map_err(e)?;self.apply(edits)?;for k in &mut self.viewer.selected_keys{k.at_sec+=delta as f64/fps;}}
            "ease"=>self.apply_ease(&j)?,
            "setColor"|"previewColor"=>{let intents=self.color_intent(&j)?;if op=="previewColor"{self.set_preview(intents)?;}else{self.apply(intents)?;}}
            "focusColor"=>{let slot:ColorSlot=if let Some(name)=j["property"].as_str(){editor::color::slot_of(&self.doc,layer(&j)?,name).ok_or("Not a color property")?}else{serde_json::from_value(j["slot"].clone()).map_err(e)?};if j.get("layer").is_some()&&slot.layer()!=Some(layer(&j)?){return Err("Color target mismatch".into())}self.viewer.color_target=Some(slot);}
            "applyPalette"=>{if let Some(slot)=self.viewer.color_target.clone(){let mut q=json!({"slot":slot,"rgba":rgba(&j)?});if let Some(id)=slot.layer(){q["layer"]=json!(id.0);}let edits=self.color_intent(&q)?;self.apply(edits)?;}else{let color=rgba(&j)?.map(|v|(v*255.0).round()as u8);let mut intents=Vec::new();for id in self.selected_required()?{intents.extend(editor::functions::verb::color_intents(&self.doc,*id,color).map_err(e)?);}self.apply(intents)?;}}
            "applyEffect"=>{let plugins:Vec<String>=if let Some(a)=j["pluginIds"].as_array(){a.iter().map(|p|p.as_str().map(str::to_owned).ok_or("Invalid plugin".into())).collect::<Result<_,String>>()?}else{vec![string(&j,"pluginId")?.into()]};let catalog=crate::render::engine::known_effects();if plugins.iter().any(|p|!catalog.iter().any(|c|c.plugin_id==*p)){return Err("Unknown effect".into())}let warp=plugins.iter().any(|p|catalog.iter().any(|d|d.plugin_id==*p&&d.stage==crate::render::compositor::EffectStage::Warp));if warp { for id in self.selected_required()? { let meta=self.doc.view().meta(*id).map_err(e)?.ok_or("Layer missing")?;let planar=match meta.source { LayerSource::Text|LayerSource::Shape=>true,LayerSource::File{path,..}=>!crate::render::media::is_mesh_path(&path)&&!crate::render::media::is_point_cloud_path(&path),_=>false };if !planar||self.doc.view().attrs(*id).map_err(e)?.is_some_and(|a|a.environment){return Err("2D warp requires a planar material".into())} } }let mut intents=Vec::new();let path_only=plugins.iter().any(|p|catalog.iter().any(|d|d.plugin_id==*p&&d.stage==crate::render::compositor::EffectStage::Path));let text_only=plugins.iter().any(|p|catalog.iter().any(|d|d.plugin_id==*p&&d.stage==crate::render::compositor::EffectStage::Text));for id in self.selected_required()?{if path_only&&self.doc.view().meta(*id).map_err(e)?.is_none_or(|m|m.source!=crate::doc::store::LayerSource::Shape){return Err("Path effects apply to shape layers".into())}if text_only&&self.doc.view().meta(*id).map_err(e)?.is_none_or(|m|m.source!=crate::doc::store::LayerSource::Text){return Err("Text effects apply to text layers".into())}intents.extend(editor::functions::verb::effect_batch_intents(&self.doc,*id,&plugins).map_err(e)?);}self.apply(intents)?;}
            "preferences"=>{if j.get("flatProjection").is_some(){self.flat_projection=serde_json::from_value(j["flatProjection"].clone()).map_err(e)?;}}
            "animate"=>{let on=j["enabled"].as_bool().ok_or("Missing enabled")?;let interp=if j["shape"].is_object(){editor::ease_kinds::decode(&j["shape"])?}else{Interp::Linear};self.viewer.animate=if !on{Animate::Off}else if j["from"].as_bool().unwrap_or(false){Animate::From{origin:self.time()?,interp}}else{Animate::Now{interp}};}
            "expandEffect"=>{let id=layer(&j)?;let effect=EffectId(integer(&j,"id")?as u32);let at=self.time()?;let (intents,copies)=editor::placement_edit::expand_intents(&self.doc,id,effect,at).map_err(e)?;self.apply(intents)?;self.pick(copies);}
            "scopeEffect"=>{let id=layer(&j)?;let effect=EffectId(integer(&j,"id")?as u32);let whole=j["whole"].as_bool().ok_or("Expected bool")?;if !self.doc.view().effects(id).map_err(e)?.iter().any(|x|x.id==effect){return Err("Effect missing".into())}let at=self.time()?;let scope=if whole{crate::doc::store::EffectScope::Whole}else{crate::doc::store::EffectScope::Each};let edits=self.doc.place_checked(id,&PropertyId::effect_scope(effect),Value::Enum(scope.enum_value()),at,Animate::Off).map_err(e)?;self.apply(edits)?;}
            "enableEffect"=>{let id=layer(&j)?;let effect=EffectId(integer(&j,"id")?as u32);let on=j["enabled"].as_bool().ok_or("Expected bool")?;if !self.doc.view().effects(id).map_err(e)?.iter().any(|x|x.id==effect){return Err("Effect missing".into())}let at=self.time()?;let edits=self.doc.place_checked(id,&PropertyId::effect_enabled(effect),Value::Bool(on),at,Animate::Off).map_err(e)?;self.apply(edits)?;}
            "moveEffect"=>{let id=layer(&j)?;let effect=integer(&j,"id")?as u32;let to=integer(&j,"to")?;let mut effects=self.doc.view().effects(id).map_err(e)?;let from=effects.iter().position(|e|e.id.0==effect).ok_or("Effect missing")?;let to=(to.max(0) as usize).min(effects.len().saturating_sub(1));let moved=effects.remove(from);effects.insert(to,moved);let catalog=crate::render::engine::known_effects();let mut spatial=false;for effect in &effects {match catalog.iter().find(|d|d.plugin_id==effect.plugin_id).map(|d|d.stage){Some(crate::render::compositor::EffectStage::Placement)=>spatial=false,Some(crate::render::compositor::EffectStage::Field|crate::render::compositor::EffectStage::Surface)=>spatial=true,Some(crate::render::compositor::EffectStage::Warp) if spatial=>return Err("2D warps run before spatial effects in the same material".into()),_=>{}}}self.apply([Intent::SetEffects{layer:id,effects}])?;}
            "removeEffect"=>{let id=layer(&j)?;let effect=integer(&j,"id")?as u32;let mut effects=self.doc.view().effects(id).map_err(e)?;if !effects.iter().any(|e|e.id.0==effect){return Err("Effect missing".into())}effects.retain(|e|e.id.0!=effect);self.apply([Intent::SetEffects{layer:id,effects}])?;}
            "ghost"=>{let ids:Vec<LayerId>=self.selected_required()?.iter().copied().filter(|&l|editor::timeline_edit::ghostable(&self.doc.view(),l)).collect();if ids.is_empty(){return Err("Nothing here can carry a ghost".into())}let on=j["enabled"].as_bool().unwrap_or(true);let intents:Vec<Intent>=ids.iter().map(|&layer|Intent::SetAttrs{layer,patch:LayerAttrsPatch{ghost:Some(on.then_some(editor::timeline_edit::GHOST_DEFAULT_DELAY)),..Default::default()}}).collect();self.apply(intents)?;}
            "sequence"|"previewSequence"=>{let layers=ids(&j["layers"])?;let delays=j["ghosts"].as_array().ok_or("Missing ghosts")?;if delays.len()!=layers.len(){return Err("layers and ghosts differ in length".into())}let intents:Vec<Intent>=layers.iter().zip(delays).filter(|(&layer,_)|editor::timeline_edit::ghostable(&self.doc.view(),layer)).map(|(&layer,d)|Intent::SetAttrs{layer,patch:LayerAttrsPatch{ghost:Some(d.as_i64().filter(|d|*d!=0)),..Default::default()}}).collect();if op=="previewSequence"{self.set_preview(intents)?;}else{self.apply(intents)?;}}
            "clip"=>{let id=layer(&j)?;let clipped=self.doc.view().attrs(id).map_err(e)?.unwrap_or_default().clip_to_below;self.apply([Intent::SetAttrs{layer:id,patch:LayerAttrsPatch{clip_to_below:Some(!clipped),..Default::default()}}])?;}
            "previewBlend"=>{let layers=if j["layers"].is_array(){ids(&j["layers"])?}else{vec![layer(&j)?]};let mode:BlendMode=serde_json::from_value(j["mode"].clone()).map_err(e)?;self.set_preview(layers.into_iter().map(|layer|Intent::SetAttrs{layer,patch:LayerAttrsPatch{blend_mode:Some(mode),..Default::default()}}).collect())?;}
            "addMarker"|"setMarker"|"deleteMarker"=>self.edit_marker(op,&j)?,
            "composition"=>{let current=self.doc.view().composition().map_err(e)?.ok_or("No composition")?;let or=|key:&str,fallback:i64|j[key].as_i64().unwrap_or(fallback);let width:u32=or("width",current.width as i64).try_into().map_err(e)?;let height:u32=or("height",current.height as i64).try_into().map_err(e)?;let fps=Fps::try_new(or("fpsNum",current.fps.num()),or("fpsDen",current.fps.den())).map_err(e)?;let duration_frames=or("durationFrames",current.duration_frames);let background=if j.get("background").is_some(){serde_json::from_value(j["background"].clone()).map_err(e)?}else{current.background};self.apply([Intent::SetComposition(Composition{width,height,fps,duration_frames,background})])?;}
            "import"=>{let paths:Vec<std::path::PathBuf>=j["paths"].as_array().ok_or("Expected paths")?.iter().map(|p|p.as_str().map(std::path::PathBuf::from).ok_or("Invalid path".into())).collect::<Result<_,String>>()?;let paths=editor::fixture::expand_folders(&paths);if paths.is_empty(){return Err("No files to import".into())}let mut intents=Vec::new();let mut known:std::collections::HashSet<_>=self.doc.view().assets().map_err(e)?.iter().map(|a|a.content_hash.clone()).collect();for path in paths{let draft=editor::fixture::prepare_path(&path,if j["role"]=="reference"{AssetRole::Reference}else{AssetRole::Material})?;if known.insert(draft.content_hash.clone()){intents.push(Intent::AdmitAsset{draft});}}self.apply(intents)?;}
            "placeAsset"|"replaceAsset"=>{let a=self.doc.view().asset(asset_id(&j)?).map_err(e)?.ok_or("Asset missing")?;let path=a.path_absolute.ok_or("Asset path missing")?;if !std::path::Path::new(&path).exists(){return Err("Asset file missing".into())}if op=="placeAsset"{let start=j["start"].as_i64().unwrap_or(self.viewer.frame);let landing=j["placement"].as_str().map(|p|(j["target"].as_u64().map(LayerId),p.to_owned()));self.place_layer(editor::create::NewKind::Media{path,name:a.name},start,landing,j["visibleFrames"].as_i64(),None)?;}else{let layer=self.viewer.selected().ok_or("Select layer to replace")?;self.apply([Intent::SetSource{layer,source:LayerSource::File{path,fingerprint:None}}])?;}}
            "relinkAsset"=>{let id=asset_id(&j)?;let path=j["path"].as_str().ok_or("Missing path")?.to_owned();if !std::path::Path::new(&path).exists(){return Err("No file at that path".into())}let a=self.doc.view().asset(id).map_err(e)?.ok_or("Asset missing")?;let kind=std::path::Path::new(&path).extension().and_then(|x|x.to_str()).and_then(crate::render::media::asset_type_for_extension).ok_or("Unsupported file type")?;let same_family=kind.split('/').next()==a.asset_type.split('/').next();if !same_family{return Err(format!("Pick a {} file",a.asset_type.split('/').next().unwrap_or("matching")))}self.apply([Intent::RelinkAsset{asset:id,path_absolute:path,project_root:None}])?;}
            "removeAsset"=>{let id=asset_id(&j)?;if self.asset_used(id)?{return Err("Asset is still in use".into())}self.apply([Intent::RemoveAsset{asset:id}])?;}
            "export"=>self.exporter.start(&self.doc.view(),||self.doc.flattened().map(|doc|doc.into_recording()).map_err(e),std::path::PathBuf::from(string(&j,"path")?),integer(&j,"start")?,integer(&j,"end")?)?,
            "cancelExport"=>self.exporter.cancel(),
            "runScript"=>self.run_script_file(string(&j,"path")?)?,
            "rerunScript"=>self.rerun_script()?,
            "save"=>{let path=string(&j,"path")?;
                self.doc.save(path).map_err(e)?;
                if j["copy"].as_bool()!=Some(true){self.path=Some(path.into());self.engine.set_cache_root(Self::cache_root_for(Some(path)));self.saved_signature=snapshot::authored_signature(&self.doc)?;}
            }
            "new"=>{if self.viewer.clock.playing(){self.viewer.clock.toggle();}self.doc=crate::work(None)?;self.viewer.clock=crate::render::playback::Clock::from_view(&self.doc.view(),60.0);self.path=None;self.pick(vec![]);self.viewer.selected_keys.clear();self.viewer.frame=0;self.saved_signature=snapshot::authored_signature(&self.doc)?;self.viewer.color_target=None;}
            "undo"=>{self.doc.undo();self.viewer.selected_keys.clear();}
            "redo"=>{self.doc.redo();self.viewer.selected_keys.clear();}
            // 履歴の点を押した時。段の番号まで戻る/進むを一手で。
            "historyGoto"=>{
                let target=integer(&j,"head")?;
                while self.doc.edit_head()>target&&self.doc.undo(){}
                while self.doc.edit_head()<target&&self.doc.redo(){}
                if self.doc.edit_head()!=target{return Err("That history point is no longer reachable".into())}
                self.viewer.selected_keys.clear();
            }
            "play"=>{if !self.viewer.clock.playing(){self.viewer.clock.toggle();}self.playback_rendered_frame=None;self.owners.clear(self.frames.skipped());self.clock_frame();}
            "pause"=>{if self.viewer.clock.playing(){self.viewer.clock.toggle();}self.playback_rendered_frame=None;self.report_owners();self.clock_frame();}
            "pickColor"=>{let comp=self.doc.view().composition().map_err(e)?.ok_or("No composition")?;let (w,h)=(comp.width as i64,comp.height as i64);let (x,y)=(num(&j,"x")?.floor() as i64,num(&j,"y")?.floor() as i64);if x<0||y<0||x>=w||y>=h{return Err("Point is outside the composition".into())}let time=self.time()?;let rgba=self.engine.render_frame(&self.doc.view(),time).map_err(e)?;let at=((y*w+x)*4) as usize;let px=rgba.get(at..at+4).ok_or("Frame is smaller than the composition")?;self.viewer.picked_color=Some([px[0],px[1],px[2],px[3]].map(|v|v as f64/255.0));self.viewer.pick_serial+=1;}
            "seek"=>{self.viewer.frame=integer(&j,"frame")?.max(0);self.viewer.clock.seek_frame(self.viewer.frame);self.playback_rendered_frame=None;}
            "anchor"=>{let id=layer(&j)?;let b=self.bounds(id).ok_or("Bounds unavailable until rendered")?;let min:[f64;3]=serde_json::from_value(b["localMin"].clone()).map_err(e)?;let max:[f64;3]=serde_json::from_value(b["localMax"].clone()).map_err(e)?;let point=[min[0]+(max[0]-min[0])*num(&j,"xFraction")?,min[1]+(max[1]-min[1])*num(&j,"yFraction")?];let intents=editor::functions::placement::anchor_point_plan(&self.doc,id,self.time()?,point).map_err(e)?;self.apply(intents)?;}
            "freeze"=>{
                let id=layer(&j)?;
                if j["enabled"].as_bool().ok_or("Missing enabled")? {
                    // 法 docs/freeze-and-flatten.md §2: 旗を立て、入点〜出点を裏で焼く。焼けたコマから cache の絵になる。
                    let meta=self.doc.view().meta(id).map_err(e)?.ok_or("Layer has no timing")?;
                    self.apply([Intent::Freeze{group:id}])?;
                    let root=Self::cache_root_for(self.path.as_deref());
                    if let Err(error)=self.freezer.start(||self.doc.flattened().map(|doc|doc.into_recording()).map_err(e),id,root,meta.timing.start,meta.timing.start+meta.timing.duration){
                        let _=self.apply([Intent::Unfreeze{group:id}]);
                        return Err(error);
                    }
                } else {
                    if self.freezer.running()==Some(id){self.freezer.cancel();}
                    self.apply([Intent::Unfreeze{group:id}])?;
                    self.engine.forget_frozen(id);
                }
            }
            "setGradient"=>{let slot:ColorSlot=serde_json::from_value(j["slot"].clone()).map_err(e)?;let edit=editor::gradient::edit(&self.doc,&slot,&j,self.time()?)?;if j["preview"]==true{self.set_preview(vec![edit])?}else{self.cancel_preview();self.apply([edit])?;}if j.get("addStop").is_some(){if let Some(model)=editor::gradient::model(&self.doc,&slot,self.time()?){self.viewer.color_target=model["stops"].as_array().and_then(|s|s.iter().max_by_key(|r|r["id"].as_u64())).and_then(|r|serde_json::from_value(r["slot"].clone()).ok());}}}
            "setFillMode"=>{let slot:ColorSlot=serde_json::from_value(j["slot"].clone()).map_err(e)?;if !slot.is_shape_fill(){return Err("Select a shape fill".into())}let at=self.time()?;editor::color::set_shape_gradient(&mut self.doc,&slot,j["gradient"].as_bool().ok_or("Missing gradient")?,at).map_err(e)?;}
            other=>return Err(format!("Unsupported operation: {other}")),
        }
        if self.doc.revision()!=self.viewer.clock_revision {
            self.viewer.clock.sync_view(&self.doc.view());
            self.viewer.clock_revision=self.doc.revision();
            self.clock_frame();
        }
        let live=self.doc.view().layers();self.viewer.selected_ids.retain(|id|live.contains(id));self.viewer.selected_keys.retain(|key|live.contains(&key.layer));if self.viewer.color_target.as_ref().is_some_and(|slot|slot.layer().is_some_and(|id|!live.contains(&id))){self.viewer.color_target=None;}if self.viewer.color_target.as_ref().is_none_or(|slot|editor::color::read_color(&self.doc,slot,self.time().unwrap_or(RationalTime::ZERO)).is_none()){self.viewer.color_target=self.viewer.selected().and_then(|id|editor::color::default_target(&self.doc,id));}self.error=None;Ok(())
    }
    pub(crate) fn clock_frame(&mut self){
        // 尺は上限ではない: 再生は越えて進み、越えた先の層は帯の外なので描かれないだけ。
        self.viewer.frame=self.viewer.clock.current_frame().max(0);
    }
    fn timing_edits(&self,j:&J)->Result<Vec<Intent>,String>{
        let id=layer(j)?;let old=self.doc.view().without_transients().meta(id).map_err(e)?.ok_or("Layer metadata missing")?.timing;
        let mut next=old;next.start=integer(j,"start")?;next.duration=integer(j,"duration")?;next.source_in=integer(j,"sourceIn")?;
        editor::functions::verb::retime_layer(&self.doc,id,old,next,next.duration==old.duration&&next.source_in==old.source_in).map_err(e)
    }
    fn stage_gesture(&mut self,j:&J)->Result<(),String>{
        if let Some(scale)=j["viewScale"].as_f64().filter(|s|s.is_finite()&&*s>0.0){self.viewer.stage_view_scale=scale;}
        if let Some(held)=j.get("held"){self.viewer.stage_held=held.as_str().map(str::to_owned);}
        // どの絵で掴んだか。Stage と Camera は投影が違う。
        let seen=crate::viewer::View::parse(j["view"].as_str().unwrap_or("Camera"))?;
        match string(j,"phase")?{
            // hover。掴まないので Document には触らず、ギズモの絵だけが変わる。
            "hover"=>{self.viewer.stage_pointer=serde_json::from_value(j["point"].clone()).ok();self.viewer.stage_view=seen;}
            "begin"=>{let interaction=self.preview_tag.take();self.cancel_preview();self.preview_tag=interaction;let ids=ids(&j["ids"])?;let start=serde_json::from_value(j["start"].clone()).map_err(e)?;
                let at=self.time()?;
                let projection=ids.last().and_then(|id|self.doc.view().attrs(*id).ok().flatten()).map_or(LayerProjection::ThreeD,|a|a.projection);
                let observer=self.view_camera(seen)?;
                let projection_camera=self.projection_camera(seen,projection)?;
                let drag=editor::stage::DragSession::begin(&self.doc,&mut self.engine,&ids,string(j,"mode")?,j["handle"].as_str().unwrap_or("body"),start,at,observer,projection_camera,self.viewer.stage_view_scale,self.viewer.stage_held.as_deref())?;
                self.pick(ids);self.stage_drag=Some(drag);
            }
            "update"=>{let drag=self.stage_drag.as_ref().ok_or("No Stage gesture")?;let point=serde_json::from_value(j["point"].clone()).map_err(e)?;
                let (point,guides)=drag.snap(point,j["snap"].as_bool().unwrap_or(false),self.viewer.stage_view_scale);self.viewer.stage_snap=guides;
                let edits=drag.edits(&self.doc,point,j["shift"].as_bool().unwrap_or(false),j["alt"].as_bool().unwrap_or(false),self.viewer.animate)?;self.set_preview(edits)?;
            }
            "commit"=>{self.stage_drag=None;self.viewer.stage_snap=[None,None];if let Some((owner,edits))=self.preview.take(){self.doc.clear_preview_edits(owner);self.apply(edits)?;}}
            "cancel"=>self.cancel_preview(),
            _=>return Err("Unknown Stage gesture phase".into()),
        }Ok(())
    }
    fn accept_paste(&mut self,result:editor::clipboard::PasteResult){match result{
        editor::clipboard::PasteResult::Layers(ids)=>{self.pick(ids);self.viewer.selected_keys.clear();}
        editor::clipboard::PasteResult::Keys(keys)=>{let mut ids=Vec::new();for k in &keys{if !ids.contains(&k.layer){ids.push(k.layer)}}self.pick(ids);self.viewer.selected_keys=keys;}
    }}
    fn apply_ease(&mut self,j:&J)->Result<(),String>{
        if let Some(expected)=j.get("selection") {
            let fps=editor::keyframe_edit::document_fps(&self.doc).map_err(e)?.as_f64();
            let current:Vec<_>=self.viewer.selected_keys.iter().map(|key|json!({"layer":key.layer.0,"property":key.property.as_ref().map(|p|p.name()),"frame":(key.at_sec*fps).round()as i64})).collect();
            if *expected!=json!(current){return Err("Easing selection changed".into())}
        }
        if self.viewer.selected_keys.is_empty(){return Err("Select keyframes".into())}
        let kind=string(j,"kind")?;
        if kind.starts_with("EasyEase"){
            let side=match kind{"EasyEase"=>editor::keymap::EaseSide::Both,"EasyEaseIn"=>editor::keymap::EaseSide::In,"EasyEaseOut"=>editor::keymap::EaseSide::Out,_=>return Err("Unknown easing".into())};
            editor::ease::apply_easy(&mut self.doc,&self.viewer.selected_keys,side)?;
        }else{
            let shape=editor::ease_kinds::decode(j)?;
            let starts=editor::ease::segments(&self.viewer.selected_keys);editor::ease::apply(&mut self.doc,&starts,shape)?;
        }Ok(())
    }
    fn edit_marker(&mut self,op:&str,j:&J)->Result<(),String>{
        let mut markers=self.doc.view().markers().map_err(e)?;
        if op=="addMarker"{let at=self.time()?;if !markers.iter().any(|m|m.time==at){markers.push(Marker{name:format!("{}",self.viewer.frame),time:at,duration:RationalTime::ZERO,body:String::new()});}}
        else{let id=string(j,"id")?;let index=markers.iter().position(|m|format!("{}/{}",m.time.num(),m.time.den())==id).ok_or("Marker no longer exists")?;if op=="deleteMarker"{markers.remove(index);}else{if let Some(s)=j["name"].as_str(){markers[index].name=s.into()}if let Some(s)=j["body"].as_str(){markers[index].body=s.into()}}}
        markers.sort_by_key(|m|m.time);self.apply([Intent::SetMarkers{markers}])
    }
}

#[cfg(test)]
mod playback_probe;

/// 効果のホットリロードの一連: 見張りが起きる → reloadEffects で世代が進む → 壊しても前の物が残り理由が出る → 消せば理由が変わる。
/// 本物の vism/ に file を置くので、他の試験と並べず単独で回す(cargo test -p motolii-ui --lib hot_reload -- --ignored)。
#[cfg(test)]
mod hot_reload_probe;

/// Freeze の口: 旗が立ち、裏で入点〜出点が焼かれ、status に進みが出る。Unfreeze で cache が消える。
#[cfg(test)]
mod freeze_op {
    use crate::edit::{Animate, Document, Intent};
    use crate::doc::store::*;
    use serde_json::json;

    #[test]
    fn freeze_bakes_the_layer_in_the_background_and_unfreeze_forgets() {
        let dir = std::env::temp_dir().join(format!("motolii-freeze-op-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let png = dir.join("dot.png");
        image::save_buffer(&png, &vec![200u8; 32 * 32 * 4], 32, 32, image::ColorType::Rgba8).unwrap();
        let mut rt = crate::EditorRuntime::open("").unwrap();
        let layer = LayerId(1);
        rt.doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: png.to_string_lossy().into_owned(), fingerprint: None }, order: 0, timing: LayerTiming::place(3, None, 11) } },
            Intent::SetEffects { layer, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.blur".into() }] },
        ]).unwrap();
        rt.request(json!({"op":"freeze","layer":1,"enabled":true})).unwrap();
        assert!(rt.doc.view().attrs(layer).unwrap().unwrap().frozen, "旗が立つ");
        let started = std::time::Instant::now();
        loop {
            let status = rt.freezer.status();
            if status["phase"] != "running" { assert_eq!(status["phase"], "complete", "{status}"); break; }
            assert!(started.elapsed() < std::time::Duration::from_secs(60), "焼き終わらない: {status}");
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        assert_eq!(rt.freezer.status()["total"], 8);
        assert_eq!(rt.engine.frozen_frames_on_disk(layer), 8, "入点 3 から 8 コマ");
        // 凍っている間、効果の欄は断られる(理由付き)。
        let refused = rt.request(json!({"op":"setProperty","layer":1,"property":"effect.0.param.radius","value":3.0}));
        assert!(refused.is_err() || rt.error.is_some());
        rt.request(json!({"op":"freeze","layer":1,"enabled":false})).unwrap();
        assert!(!rt.doc.view().attrs(layer).unwrap().unwrap().frozen);
        assert_eq!(rt.engine.frozen_frames_on_disk(layer), 0, "Unfreeze で cache が消える");
    }
}

#[cfg(test)]
mod sequence_preview;


#[cfg(test)]
mod blend_interaction_tests {
    use super::*;
    #[test]
    fn blend_preview_batches_layers_and_cannot_cancel_a_newer_interaction() {
        let mut rt=EditorRuntime::open("").unwrap();
        rt.request(json!({"op":"create","kind":"rectangle"})).unwrap();
        let a=rt.viewer.selected().unwrap();
        rt.request(json!({"op":"create","kind":"rectangle"})).unwrap();
        let b=rt.viewer.selected().unwrap();
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

#[cfg(test)]
mod timing_drag_tests;

#[cfg(test)]
mod path_effect_tests {
    use super::*;
    /// 棚の Path 族は形の層にだけ掛かり、掛かると描く輪郭に演算が積まれる。角丸矩形は最初から 1 枚積んで生まれる。
    #[test]
    fn path_effects_apply_to_shape_layers_only_and_reach_the_drawn_outline() {
        let mut rt=EditorRuntime::open("").unwrap();
        for kind in ["ellipse","star","polygon","line","null"] { rt.request(json!({"op":"create","kind":kind})).unwrap(); }
        rt.request(json!({"op":"create","kind":"text"})).unwrap();
        assert_eq!(rt.request(json!({"op":"applyEffect","pluginId":crate::render::extensions::pathop::PUCKER_BLOAT})).unwrap_err(),"Path effects apply to shape layers");
        rt.request(json!({"op":"create","kind":"roundedRectangle"})).unwrap();
        let rect=rt.viewer.selected().unwrap();
        assert_eq!(rt.doc.view().effects(rect).unwrap().iter().map(|e|e.plugin_id.as_str()).collect::<Vec<_>>(),vec![crate::render::extensions::pathop::ROUNDED_CORNERS]);
        rt.request(json!({"op":"applyEffect","pluginId":crate::render::extensions::pathop::PUCKER_BLOAT})).unwrap();
        let view=rt.doc.view();
        let effects=crate::render::picture::resolve::effects::resolved_effects(&view, rect,crate::doc::core::RationalTime::ZERO).unwrap();
        let shown=crate::render::extensions::pathop::with_effects(&view.shapes(rect).unwrap(),&effects);
        let crate::doc::store::ShapeNode::Leaf(leaf)=&shown[0] else { panic!("葉") };
        assert_eq!(leaf.ops.iter().map(|o|std::mem::discriminant(&o.kind)).collect::<Vec<_>>().len(),2);
        assert!(matches!(leaf.ops[0].kind,crate::doc::vector::OpKind::RoundedCorners{radius} if radius==10.0));
        assert!(view.shapes(rect).unwrap().iter().all(|n|matches!(n,crate::doc::store::ShapeNode::Leaf(s) if s.ops.is_empty())),"書類そのものには積まれない");
        let catalog=crate::render::engine::known_effects();
        assert!(crate::render::extensions::pathop::KINDS.iter().all(|k|catalog.iter().any(|d|d.plugin_id==k.plugin_id)),"全枚が棚に居る");
    }
    /// 形の元の値は property: 星の頂点数を欄で変えると描く形に届き、書類の形は既定のまま残る。
    #[test]
    fn shape_values_are_properties_that_reach_the_drawn_outline() {
        let mut rt=EditorRuntime::open("").unwrap();
        rt.request(json!({"op":"create","kind":"star"})).unwrap();
        let star=rt.viewer.selected().unwrap();
        let read=crate::editor::functions::read::inspector_data_from_doc(&rt.doc.view(),star,crate::doc::core::RationalTime::ZERO,&crate::render::engine::known_effects());
        assert!(!read.text.iter().any(|r|r.label=="Points"),"形の寸法は欄にならない(2D は path、8/29)");
        rt.request(json!({"op":"setProperty","layer":star.0,"property":"shape.points","value":8.0})).unwrap();
        let view=rt.doc.view();
        let at=crate::doc::core::RationalTime::ZERO;
        let crate::doc::store::ShapeNode::Leaf(shown)=&crate::render::picture::shapes::shapes_at(&view, star,at).unwrap()[0] else { panic!("葉") };
        assert!(matches!(shown.source,crate::doc::vector::PathSource::PolyStar{points,..} if points==8.0));
        let crate::doc::store::ShapeNode::Leaf(stored)=&view.shapes(star).unwrap()[0] else { panic!("葉") };
        assert!(matches!(stored.source,crate::doc::vector::PathSource::PolyStar{points,..} if points==5.0),"書類の形は既定のまま");
    }
}

#[cfg(test)]
mod poster;

#[cfg(test)]
mod poster_probe {
    use crate::edit::{Animate, Document, Intent};
    use super::*;
    #[test]
    #[ignore]
    fn probe_saved_poster() {
        let Some(path)=std::env::var_os("MOTOLII_DESIGN_OUT") else { return };
        let mut rt=EditorRuntime::open(path.to_str().unwrap()).unwrap();
        let time=rt.time().unwrap();
        let view=rt.doc.view();
        let comp=view.composition().unwrap().unwrap().spec();
        let pixels=rt.engine.render_frame(&view,time).unwrap();
        eprintln!("failures: {:?}",rt.engine.layer_failures());
        for (x,y) in [(1450u32,250u32),(1200,200),(1800,600),(960,540)] {
            let i=((y*comp.width+x)*4) as usize;
            eprintln!("({x},{y}) = {:?}",&pixels[i..i+4]);
        }
        let resolved=rt.engine.resolved_for(&view,time).unwrap();
        for id in view.layers(){ let name=view.attrs(id).unwrap().unwrap().name; let b=rt.engine.selected_layer_bounds_in(&view,&resolved,id,time); let pos=view.value_at(id,&PropertyId::new(property::POSITION).unwrap(),time).unwrap(); eprintln!("{name}: pos {pos:?} bounds {b:?} corners {}", rt.bounds(id).map(|b|b["corners"].to_string()).unwrap_or_default()); }
    }
}

#[cfg(test)]
mod swiss {
    use crate::edit::{Animate, Document, Intent};
    use super::*;
    /// スイス風の下地: 白い地と黒い活字 2 つ。図形はアプリで置く。
    #[test]
    #[ignore]
    fn compose_swiss_base() {
        let Some(out)=std::env::var_os("MOTOLII_DESIGN_OUT") else { return };
        let mut rt=EditorRuntime::open("").unwrap();
        let go=|rt:&mut EditorRuntime,j:serde_json::Value|{rt.request(j.clone()).unwrap_or_else(|e|panic!("{j}: {e}"));};
        go(&mut rt,json!({"op":"composition","width":1920,"height":1080,"background":[0.96,0.95,0.92,1.0]}));
        let text=|rt:&mut EditorRuntime,name:&str,content:&str,size:f64,pos:[f64;2],rgba:[f64;4]|{
            go(rt,json!({"op":"create","kind":"text"}));let id=rt.viewer.selected().unwrap();
            go(rt,json!({"op":"setAttrs","layers":[id.0],"patch":{"name":name}}));
            go(rt,json!({"op":"setText","layer":id.0,"content":content}));
            go(rt,json!({"op":"styleText","layer":id.0,"scope":"all","size":size}));
            go(rt,json!({"op":"setProperty","layer":id.0,"property":"position","value":pos}));
            go(rt,json!({"op":"applyPalette","rgba":rgba}));
            let mut document=rt.doc.view().text_document(id).unwrap().unwrap();
            document.justify=crate::doc::store::TextJustify::Left;
            rt.doc.apply(Intent::SetTextDocument{layer:id,document}).unwrap();
        };
        // 文字の position は canvas の左上。活字の縁は局所 bounds の min だけ内側(探針で測った値)。
        text(&mut rt,"Title","Motolii",250.0,[109.0,263.0],[0.08,0.08,0.08,1.0]);
        text(&mut rt,"Column","Shapes\nPaths\nEffects\n2026",40.0,[126.0,-180.0],[0.08,0.08,0.08,1.0]);
        text(&mut rt,"Caption","Static composition — International style",28.0,[1018.0,401.0],[0.08,0.08,0.08,1.0]);
        rt.request(json!({"op":"save","path":out.to_string_lossy()})).unwrap();
    }
}
