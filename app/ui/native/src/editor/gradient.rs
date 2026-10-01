#[allow(unused_imports)]
use crate::edit::{Animate, Document, Intent};
use crate::doc::store::{};
use crate::doc::vector::{Brush, Fill, Gradient, GradientBlend, GradientStop, GradientType, Point, Rgb};
use super::color::{leaf_mut, shape_location, gradient_axis};
use crate::viewer::ColorSlot;
use serde_json::{Value as J, json};

pub(crate) fn stops(j: &J) -> Result<Vec<GradientStop>, String> {
    let rows = j.as_array().filter(|v| (2..=32).contains(&v.len())).ok_or("Use 2 to 32 gradient stops")?;
    let mut result = rows.iter().enumerate().map(|(i, row)| {
        let rgba = if row.is_array() { row } else { &row["rgba"] };
        let a = rgba.as_array().filter(|v|v.len()>=3).ok_or("Missing RGB stop")?;
        let channel = |i:usize| a[i].as_f64().filter(|v|v.is_finite() && (0.0..=1.0).contains(v)).ok_or("Invalid stop color");
        let offset = if row.is_array() { i as f64 / (rows.len()-1) as f64 } else { row["offset"].as_f64().ok_or("Missing stop position")? };
        if !offset.is_finite() || !(0.0..=1.0).contains(&offset) { return Err("Stop position must be between 0 and 1".into()); }
        Ok(GradientStop{offset,color:Rgb{r:channel(0)?,g:channel(1)?,b:channel(2)?}})
    }).collect::<Result<Vec<_>,String>>()?;
    result.sort_by(|a,b|a.offset.total_cmp(&b.offset));
    Ok(result)
}

pub(crate) fn kind(j: &J) -> Result<GradientType, String> {
    match j.as_str().unwrap_or("linear") {
        "linear" => Ok(GradientType::Linear), "radial" => Ok(GradientType::Radial),
        "angular" => Ok(GradientType::Angular), "diamond" => Ok(GradientType::Diamond),
        _ => Err("Unknown gradient type".into()),
    }
}

pub(crate) fn edit(doc:&Document, slot:&ColorSlot, j:&J, at:crate::doc::store::RationalTime) -> Result<Intent,String> {
    let (layer,path)=shape_location(slot).ok_or("Select a shape fill")?;
    let view=doc.view().without_transients();
    if view.attrs(layer).map_err(|e|e.to_string())?.ok_or("Layer not found")?.locked { return Err("Layer is locked".into()); }
    let mut evaluated=crate::render::picture::shapes::shapes_at(&view, layer,at).map_err(|e|e.to_string())?;
    let shown=leaf_mut(&mut evaluated,path).and_then(|s|s.fill.as_ref()).map(|f|f.brush.clone()).unwrap_or_default();
    let mut shapes=view.shapes(layer).map_err(|e|e.to_string())?;
    let shape=leaf_mut(&mut shapes,path).ok_or("Shape not found")?;
    let (start,end)=gradient_axis(&shape.source);
    let fill=shape.fill.get_or_insert_with(Fill::default);
    let mut g=match &fill.brush {
        Brush::Gradient(g)=>g.clone(),
        Brush::Solid(c)=>{
            let c=match &shown {Brush::Solid(c)=>*c,_=>*c};
            Gradient{kind:GradientType::Linear,start,end,stops:if j.get("addStop").is_some(){vec![GradientStop{offset:0.0,color:c}]}else{vec![GradientStop{offset:0.0,color:c},GradientStop{offset:1.0,color:c}]},blend:Default::default(),stop_ids:Vec::new(),next_stop_id:0}
        },
    };
    g.identify_stops();
    // Old property identities remain reserved when a palette replaces the stops.
    for p in view.properties(layer) {
        if let Some(id)=p.name().strip_prefix(crate::doc::store::property::FILL_STOP_PREFIX).and_then(|s|s.split('.').next()).and_then(|s|s.parse::<usize>().ok()) {
            g.next_stop_id=g.next_stop_id.max(id.saturating_add(1));
        }
    }
    if !j["stops"].is_null() {
        let next=stops(&j["stops"])?;
        g.stop_ids=(g.next_stop_id..g.next_stop_id+next.len()).collect();
        g.next_stop_id+=next.len();g.stops=next;
    }
    if let Some(offset)=j.get("addStop") {
        let offset=offset.as_f64().filter(|n|n.is_finite()&&(0.0..=1.0).contains(n)).ok_or("Invalid stop position")?;
        if g.stops.len()>=32{return Err("Use at most 32 gradient stops".into())}
        let color=match &shown {Brush::Solid(c)=>*c,Brush::Gradient(shown)=>shown.color_at(offset)};
        let id=g.allocate_stop_id();
        let index=g.stops.partition_point(|s|s.offset<=offset);
        g.stops.insert(index,GradientStop{offset,color});g.stop_ids.insert(index,id);
    }
    if let Some(id)=j.get("removeStop") {
        let id=id.as_u64().ok_or("Invalid stop identity")? as usize;
        if g.stops.len()<=2{return Err("Keep at least two gradient stops".into())}
        let index=g.stop_index(id).ok_or("Color stop no longer exists")?;
        g.stops.remove(index);g.stop_ids.remove(index);
    }
    if !j["kind"].is_null() { g.kind=kind(&j["kind"])?; }
    if let Some(name)=j["blend"].as_str() { g.blend=GradientBlend::parse(name).ok_or("Unknown gradient blend")?; }
    if let Some(angle)=j["angle"].as_f64() {
        if !angle.is_finite() { return Err("Invalid gradient angle".into()); }
        let length=(g.end.x-g.start.x).hypot(g.end.y-g.start.y).max(1.0);
        let centre=Point{x:(g.start.x+g.end.x)*0.5,y:(g.start.y+g.end.y)*0.5};
        let (sin,cos)=angle.to_radians().sin_cos();
        g.start=Point{x:centre.x-cos*length*0.5,y:centre.y-sin*length*0.5};
        g.end=Point{x:centre.x+cos*length*0.5,y:centre.y+sin*length*0.5};
    }
    fill.brush=Brush::Gradient(g);
    Ok(Intent::SetShapes{layer,shapes})
}

pub(crate) fn model(doc:&Document, slot:&ColorSlot, time:crate::doc::store::RationalTime) -> Option<J> {
    let (layer,path)=shape_location(slot)?;
    let mut shapes=crate::render::picture::shapes::shapes_at(&doc.view(), layer,time).ok()?;
    let fill=leaf_mut(&mut shapes,path)?.fill.as_ref()?;
    let slot=ColorSlot::ShapeFill{layer,path:path.to_vec()};
    Some(match &fill.brush {
        Brush::Solid(c)=>json!({"slot":slot,"kind":"solid","angle":0,"blend":GradientBlend::default().name(),"stops":[{"offset":0,"rgba":[c.r,c.g,c.b,1.0],"slot":slot}]}),
        Brush::Gradient(g)=>json!({"slot":slot,"kind":match g.kind{GradientType::Linear=>"linear",GradientType::Radial=>"radial",GradientType::Angular=>"angular",GradientType::Diamond=>"diamond"},"angle":(g.end.y-g.start.y).atan2(g.end.x-g.start.x).to_degrees(),"blend":g.blend.name(),"stops":g.stops.iter().enumerate().map(|(index,s)|{
            let id=g.stop_id(index);
            json!({"id":id,"offset":s.offset,"rgba":[s.color.r,s.color.g,s.color.b,1.0],"positionProperty":format!("fill.stop.{id}.offset"),"colorProperty":format!("fill.stop.{id}.color"),"slot":ColorSlot::ShapeGradientPoint{layer,path:path.to_vec(),index:id}})
        }).collect::<Vec<_>>()}),
    })
}

#[cfg(test)]
mod tests {
    use crate::edit::{Animate, Document, Intent};
    use super::*;
    use crate::doc::store::*;
    #[test]
    fn stop_identity_survives_insert_remove_undo_and_serialization() {
        use crate::doc::eval::{Value, KeyframeTrack, Keyframe, Interp};
        let mut doc=crate::edit::blank_project().with_programs(crate::render::extensions::bundled());let layer=LayerId(1);
        doc.apply_all(crate::editor::create::new_layer_intents(layer,0,0,60,Fps::try_new(30,1).unwrap(),(1920.0,1080.0),crate::editor::create::NewKind::Rectangle,None)).unwrap();
        let slot=ColorSlot::ShapeFill{layer,path:vec![0]};
        let at=RationalTime::ZERO;
        doc.apply(edit(&doc,&slot,&json!({"stops":[[1,0,0,1],[0,0,1,1]]}),at).unwrap()).unwrap();
        let initial=model(&doc,&slot,at).unwrap();
        let id=initial["stops"][1]["id"].as_u64().unwrap() as usize;
        let color_slot=ColorSlot::ShapeGradientPoint{layer,path:vec![0],index:id};
        let p=PropertyId::new(&format!("fill.stop.{id}.color")).unwrap();
        let later=RationalTime::try_from_frame(30,Fps::try_new(30,1).unwrap()).unwrap();
        let mut track=KeyframeTrack::new();
        track.insert(Keyframe{t:at,value:Value::Color([0.0,0.0,1.0,1.0]),interp:Interp::Linear,spatial:None});
        track.insert(Keyframe{t:later,value:Value::Color([0.0,1.0,0.0,1.0]),interp:Interp::Linear,spatial:None});
        doc.apply(Intent::SetTrack{layer,property:p.clone(),track:track.clone()}).unwrap();
        let before=doc.view().shapes(layer).unwrap();
        let mut evaluated=crate::render::picture::shapes::shapes_at(&doc.view(), layer,at).unwrap();
        let Brush::Gradient(g)=&leaf_mut(&mut evaluated,&[0]).unwrap().fill.as_ref().unwrap().brush else{panic!()};
        let expected=g.color_at(0.25);
        doc.apply(edit(&doc,&slot,&json!({"addStop":0.25}),at).unwrap()).unwrap();
        let added=model(&doc,&slot,at).unwrap();
        assert_eq!(added["stops"].as_array().unwrap().len(),3);
        assert_eq!(added["stops"][1]["rgba"],json!([expected.r,expected.g,expected.b,1.0]));
        assert_eq!(added["stops"][2]["id"],json!(id));
        assert_eq!(super::super::color::read_color(&doc,&color_slot,later),Some([0.0,1.0,0.0,1.0]));
        assert_eq!(doc.view().track(layer,&p).unwrap(),Some(track.clone()));
        let inserted=doc.view().shapes(layer).unwrap();
        assert!(doc.undo());assert_eq!(doc.view().shapes(layer).unwrap(),before);
        assert!(doc.redo());assert_eq!(doc.view().shapes(layer).unwrap(),inserted);
        let roundtrip:Vec<crate::doc::store::ShapeNode>=serde_json::from_str(&serde_json::to_string(&inserted).unwrap()).unwrap();
        assert_eq!(roundtrip,inserted);
        doc.apply(edit(&doc,&slot,&json!({"removeStop":id}),at).unwrap()).unwrap();
        assert_eq!(super::super::color::read_color(&doc,&color_slot,at),None);
        doc.apply(edit(&doc,&slot,&json!({"addStop":0.75}),at).unwrap()).unwrap();
        assert!(model(&doc,&slot,at).unwrap()["stops"].as_array().unwrap().iter().all(|s|s["id"]!=json!(id)));
        assert_eq!(doc.view().track(layer,&p).unwrap(),Some(track));
    }
    #[test]
    fn stops_direction_preview_and_undo_share_document_shapes() {
        let mut doc=crate::edit::blank_project().with_programs(crate::render::extensions::bundled());let layer=LayerId(1);
        doc.apply_all(crate::editor::create::new_layer_intents(layer,0,0,60,Fps::try_new(30,1).unwrap(),(1920.0,1080.0),crate::editor::create::NewKind::Rectangle,None)).unwrap();
        let slot=ColorSlot::ShapeFill{layer,path:vec![0]};
        let before=doc.view().shapes(layer).unwrap();let history=doc.history_depth();
        let intent=edit(&doc,&slot,&json!({"kind":"radial","angle":90,"stops":[[1,0,0,1],[0,1,0,1],[0,0,1,1]]}),RationalTime::ZERO).unwrap();
        let owner=doc.begin_preview();doc.preview_edits(owner,&[intent.clone()]).unwrap();
        let projected=model(&doc,&slot,RationalTime::ZERO).unwrap();
        assert_eq!(projected["kind"],"radial");assert_eq!(projected["stops"].as_array().unwrap().len(),3);
        assert!((projected["angle"].as_f64().unwrap()-90.0).abs()<0.001);
        assert_eq!(doc.view().without_transients().shapes(layer).unwrap(),before);
        assert_eq!(doc.history_depth(),history);
        doc.clear_preview_edits(owner);assert_eq!(doc.view().shapes(layer).unwrap(),before);
        doc.apply(intent).unwrap();assert_ne!(doc.view().shapes(layer).unwrap(),before);
        let axis_owner=doc.begin_preview();
        doc.preview_edits(axis_owner,&[Intent::SetConstant{layer,property:PropertyId::new(property::FILL_ANGLE).unwrap(),value:Value::F64(35.0)}]).unwrap();
        assert!((model(&doc,&slot,RationalTime::ZERO).unwrap()["angle"].as_f64().unwrap()-35.0).abs()<0.001);
        doc.clear_preview_edits(axis_owner);
        assert!((model(&doc,&slot,RationalTime::ZERO).unwrap()["angle"].as_f64().unwrap()-90.0).abs()<0.001);
        let projected=model(&doc,&slot,RationalTime::ZERO).unwrap();
        let middle:ColorSlot=serde_json::from_value(projected["stops"][1]["slot"].clone()).unwrap();
        assert_eq!(super::super::color::read_color(&doc,&middle,RationalTime::ZERO),Some([0.0,1.0,0.0,1.0]));
        assert!(doc.undo());assert_eq!(doc.view().shapes(layer).unwrap(),before);
        assert!(edit(&doc,&slot,&json!({"stops":[{"offset":-1,"rgba":[0,0,0,1]}, {"offset":1,"rgba":[1,1,1,1]}]}),RationalTime::ZERO).is_err());
        doc.apply(Intent::SetAttrs{layer,patch:LayerAttrsPatch{locked:Some(true),..Default::default()}}).unwrap();
        assert!(edit(&doc,&slot,&json!({"kind":"linear"}),RationalTime::ZERO).is_err());
    }
}
