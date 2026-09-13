use crate::doc::store::*;
use serde_json::Value as J;
use std::collections::BTreeMap;

pub(crate) fn edits(doc:&Document, layer:LayerId, time:RationalTime, j:&J) -> Result<Vec<Intent>,String> {
    let view=doc.view().without_transients();
    if view.attrs(layer).map_err(|e|e.to_string())?.ok_or("Layer not found")?.locked { return Err("Layer is locked".into()); }
    let mut document=view.text_document(layer).map_err(|e|e.to_string())?.ok_or("Select a Text layer")?;
    let original_document=document.clone();
    let resolved=view.resolved_text_document(layer,time).map_err(|e|e.to_string())?;
    let text=document.content.eval(time).to_owned();
    if j["text"].as_str().is_some_and(|t|t!=text) { return Err("Text changed; select the characters again".into()); }
    let scope=j["scope"].as_str().unwrap_or("all");
    if !text_edit::SCOPES.contains(&scope) { return Err("Unknown character selection".into()); }
    let start=j["start"].as_u64().unwrap_or(0) as usize;
    let end=j["end"].as_u64().unwrap_or(0) as usize;
    if scope=="selection" && (start>=end || end>text.encode_utf16().count()) { return Err("Select some characters".into()); }
    let selected=text_edit::selected(&text,start,end,scope);
    if !selected.iter().any(|v|*v) && !(text.is_empty() && scope=="all") { return Err("No matching characters".into()); }
    let family=j["family"].as_str();
    if family.is_some_and(|f|!crate::doc::vector::text::font_families().iter().any(|n|n==f)) { return Err("Font family is not installed".into()); }
    let size=if j.get("size").is_some() {Some(j["size"].as_f64().filter(|v|v.is_finite()&&(1.0..=1000.0).contains(v)).ok_or("Size must be between 1 and 1000")?)} else {None};
    if family.is_none() && size.is_none() { return Err("Choose a font or size".into()); }
    let mut ids=text_edit::style_ids(&document,&text);
    let mut counts:BTreeMap<TextStyleId,(usize,usize)>=BTreeMap::new();
    for (id,yes) in ids.iter().zip(&selected) { let count=counts.entry(*id).or_default();count.0+=1;if *yes {count.1+=1;} }
    if ids.is_empty() { let id=document.styles.first().ok_or("Text style missing")?.id;counts.insert(id,(1,1)); }
    let prefix=property::TEXT_STYLE_PREFIX;
    let mut next=document.styles.iter().map(|s|s.id.0).chain(view.properties(layer).iter().filter_map(|p|p.name().strip_prefix(prefix)?.split('.').next()?.parse::<u32>().ok())).max().unwrap_or(0)+1;
    let mut edits=Vec::new();let mut changed=BTreeMap::new();
    for (id,(total,chosen)) in counts {
        if chosen==0 { continue; }
        let source=document.styles.iter().find(|s|s.id==id).ok_or("Text style missing")?.clone();
        let effective=resolved.as_ref().and_then(|d|d.styles.iter().find(|s|s.id==id)).cloned().unwrap_or(source.clone());
        if family.is_none_or(|f|f==effective.font.family) && size.is_none_or(|v|v==f64::from(effective.size)) { changed.insert(id,id);continue; }
        let mut style=if chosen<total {effective} else {source.clone()};
        if chosen<total { style.id=TextStyleId(next);next+=1; }
        if let Some(family)=family {style.font=FontRef{family:family.into(),..Default::default()};}
        if let Some(size)=size {style.size=size as f32;}
        if style.id!=id {
            let old_prefix=format!("{prefix}{id}.");
            for property in view.properties(layer) {
                let Some(suffix)=property.name().strip_prefix(&old_prefix) else {continue};
                if size.is_some() && suffix=="size" {continue;}
                if let Some(source)=view.property_source(layer,&property).map_err(|e|e.to_string())? {
                    let target=PropertyId::new(&format!("{prefix}{}.{suffix}",style.id)).map_err(|e|e.to_string())?;
                    match source.base {
                        Some(PropertyBase::Constant(value))=>edits.push(Intent::SetConstant{layer,property:target.clone(),value}),
                        Some(PropertyBase::Track(track))=>edits.push(Intent::SetTrack{layer,property:target.clone(),track}),
                        Some(PropertyBase::Slot(slot))=>edits.push(Intent::SetPropertySlot{layer,property:target.clone(),slot}),
                        None=>{},
                    }
                    if !source.modulators.is_empty(){edits.push(Intent::SetPropertyModulators{layer,property:target,modulators:source.modulators});}
                }
            }
        }
        if let Some(size)=size {edits.push(Intent::SetConstant{layer,property:PropertyId::text_style_size(style.id),value:Value::F64(size)});}
        changed.insert(id,style.id);
        if style.id==id { *document.styles.iter_mut().find(|s|s.id==id).unwrap()=style; }
        else {document.styles.push(style);}
    }
    for (id,yes) in ids.iter_mut().zip(selected) {if yes {*id=changed[id];}}
    text_edit::set_runs(&mut document,&ids);
    if !ids.is_empty(){document.styles.retain(|s|ids.contains(&s.id));}
    text_edit::set_runs(&mut document,&ids);
    if edits.is_empty() && document==original_document {return Ok(Vec::new());}
    edits.insert(0,Intent::SetTextDocument{layer,document});
    Ok(edits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn project() -> Document {
        let mut doc=blank_project();
        doc.apply_all(crate::editor::create::new_layer_intents(LayerId(1),0,0,60,Fps::try_new(30,1).unwrap(),(1920.0,1080.0),crate::editor::create::NewKind::Text,None)).unwrap();
        crate::editor::text::write_content(&mut doc,LayerId(1),RationalTime::ZERO,"あカ漢か\u{3099}😀ab".into()).unwrap();
        doc.apply(Intent::SetConstant{layer:LayerId(1),property:PropertyId::text_style_size(TextStyleId(0)),value:Value::F64(40.0)}).unwrap();doc
    }
    #[test]
    fn script_formatting_preserves_other_chars_and_roundtrips_undo() {
        let mut doc=project();let layer=LayerId(1);let before=doc.view().text_document(layer).unwrap().unwrap();
        let changes=edits(&doc,layer,RationalTime::ZERO,&json!({"scope":"hiragana","size":80.0})).unwrap();
        doc.apply_all(changes).unwrap();
        let formatted=doc.view().resolved_text_document(layer,RationalTime::ZERO).unwrap().unwrap();
        let ids=text_edit::style_ids(&formatted,formatted.content.eval(RationalTime::ZERO));
        let sizes:Vec<_>=ids.iter().map(|id|formatted.styles.iter().find(|s|s.id==*id).unwrap().size).collect();
        assert_eq!(sizes,vec![80.0,40.0,40.0,80.0,40.0,40.0,40.0]);
        let path=std::env::temp_dir().join(format!("motolii-rich-text-{}.rrd",std::process::id()));
        doc.save(&path).unwrap();let loaded=Document::load(&path).unwrap();
        assert_eq!(loaded.view().text_document(layer).unwrap(),doc.view().text_document(layer).unwrap());std::fs::remove_file(path).unwrap();
        assert!(doc.undo());assert_eq!(doc.view().text_document(layer).unwrap().unwrap(),before);
    }
    #[test]
    fn partial_font_and_size_reach_raster_and_replacement_keeps_runs() {
        let mut doc=project();let layer=LayerId(1);let time=RationalTime::ZERO;
        let before=doc.view().resolved_text_document(layer,time).unwrap().unwrap();
        let canvas=crate::doc::vector::Canvas{width:800,height:240,origin_x:0,origin_y:0};
        let a=crate::render::engine::text::text_shapes(&before,time,&canvas).unwrap().unwrap();
        let family=crate::doc::vector::text::font_families().iter().find(|f|f.as_str()=="Georgia").unwrap();
        doc.apply_all(edits(&doc,layer,time,&json!({"scope":"selection","start":7,"end":8,"size":120.0,"family":family})).unwrap()).unwrap();
        let after=doc.view().resolved_text_document(layer,time).unwrap().unwrap();
        let ids=text_edit::style_ids(&after,after.content.eval(time));
        let selected=after.styles.iter().find(|s|s.id==ids[5]).unwrap();assert_eq!(selected.size,120.0);assert_eq!(&selected.font.family,family);
        let b=crate::render::engine::text::text_shapes(&after,time,&canvas).unwrap().unwrap();assert_ne!(a,b,"級数と書体の変更が輪郭に届く");
        crate::editor::text::write_content(&mut doc,layer,time,"あカ漢か\u{3099}😀a!b".into()).unwrap();
        let changed=doc.view().text_document(layer).unwrap().unwrap();let new_ids=text_edit::style_ids(&changed,changed.content.eval(time));
        assert_eq!(new_ids[5],ids[5]);assert_eq!(new_ids[6],ids[5]);assert_eq!(new_ids[7],ids[6]);
        assert!(edits(&doc,layer,time,&json!({"scope":"selection","start":1,"end":2,"text":"outdated","size":50})).is_err());
    }
    #[test]
    fn kana_mark_is_one_selection_and_empty_text_can_choose_size() {
        let mut doc=project();let layer=LayerId(1);let time=RationalTime::ZERO;
        doc.apply_all(edits(&doc,layer,time,&json!({"scope":"selection","start":4,"end":5,"size":90})).unwrap()).unwrap();
        let d=doc.view().resolved_text_document(layer,time).unwrap().unwrap();let ids=text_edit::style_ids(&d,d.content.eval(time));
        assert_eq!(d.styles.iter().find(|s|s.id==ids[3]).unwrap().size,90.0);
        crate::editor::text::write_content(&mut doc,layer,time,String::new()).unwrap();
        doc.apply_all(edits(&doc,layer,time,&json!({"scope":"all","size":55})).unwrap()).unwrap();
        assert_eq!(doc.view().resolved_text_document(layer,time).unwrap().unwrap().styles[0].size,55.0);
    }
}
