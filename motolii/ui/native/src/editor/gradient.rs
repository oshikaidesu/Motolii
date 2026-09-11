use crate::doc::store::{Document, Intent};
use crate::doc::vector::{Brush, Fill, Gradient, GradientStop, GradientType, Point, Rgb};
use super::{color::{leaf_mut, shape_location, gradient_axis}, session::ColorSlot};
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

pub(crate) fn edit(doc:&Document, slot:&ColorSlot, j:&J) -> Result<Intent,String> {
    let (layer,path)=shape_location(slot).ok_or("Select a shape fill")?;
    let view=doc.view().without_transients();
    if view.attrs(layer).map_err(|e|e.to_string())?.ok_or("Layer not found")?.locked { return Err("Layer is locked".into()); }
    let mut shapes=view.shapes(layer).map_err(|e|e.to_string())?;
    let shape=leaf_mut(&mut shapes,path).ok_or("Shape not found")?;
    let (start,end)=gradient_axis(&shape.source);
    let fill=shape.fill.get_or_insert_with(Fill::default);
    let mut g=match &fill.brush {
        Brush::Gradient(g)=>g.clone(),
        Brush::Solid(c)=>Gradient{kind:GradientType::Linear,start,end,stops:vec![GradientStop{offset:0.0,color:*c},GradientStop{offset:1.0,color:*c}]},
    };
    if !j["stops"].is_null() { g.stops=stops(&j["stops"])?; }
    if !j["kind"].is_null() { g.kind=kind(&j["kind"])?; }
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

pub(crate) fn model(doc:&Document, slot:&ColorSlot) -> Option<J> {
    let (layer,path)=shape_location(slot)?;
    let mut shapes=doc.view().shapes(layer).ok()?;
    let fill=leaf_mut(&mut shapes,path)?.fill.as_ref()?;
    let slot=ColorSlot::ShapeFill{layer,path:path.to_vec()};
    Some(match &fill.brush {
        Brush::Solid(c)=>json!({"slot":slot,"kind":"solid","angle":0,"stops":[{"offset":0,"rgba":[c.r,c.g,c.b,1.0]}]}),
        Brush::Gradient(g)=>json!({"slot":slot,"kind":match g.kind{GradientType::Linear=>"linear",GradientType::Radial=>"radial",GradientType::Angular=>"angular",GradientType::Diamond=>"diamond"},"angle":(g.end.y-g.start.y).atan2(g.end.x-g.start.x).to_degrees(),"stops":g.stops.iter().enumerate().map(|(index,s)|json!({"offset":s.offset,"rgba":[s.color.r,s.color.g,s.color.b,1.0],"slot":ColorSlot::ShapeGradientPoint{layer,path:path.to_vec(),index}})).collect::<Vec<_>>()}),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::*;
    #[test]
    fn stops_direction_preview_and_undo_share_document_shapes() {
        let mut doc=blank_project();let layer=LayerId(1);
        doc.apply_all(crate::editor::create::new_layer_intents(layer,0,0,60,Fps::try_new(30,1).unwrap(),(1920.0,1080.0),crate::editor::create::NewKind::Rectangle,None)).unwrap();
        let slot=ColorSlot::ShapeFill{layer,path:vec![0]};
        let before=doc.view().shapes(layer).unwrap();let history=doc.history_depth();
        let intent=edit(&doc,&slot,&json!({"kind":"radial","angle":90,"stops":[[1,0,0,1],[0,1,0,1],[0,0,1,1]]})).unwrap();
        let owner=doc.begin_preview();doc.preview_edits(owner,&[intent.clone()]).unwrap();
        let projected=model(&doc,&slot).unwrap();
        assert_eq!(projected["kind"],"radial");assert_eq!(projected["stops"].as_array().unwrap().len(),3);
        assert!((projected["angle"].as_f64().unwrap()-90.0).abs()<0.001);
        assert_eq!(doc.view().without_transients().shapes(layer).unwrap(),before);
        assert_eq!(doc.history_depth(),history);
        doc.clear_preview_edits(owner);assert_eq!(doc.view().shapes(layer).unwrap(),before);
        doc.apply(intent).unwrap();assert_ne!(doc.view().shapes(layer).unwrap(),before);
        let middle=ColorSlot::ShapeGradientPoint{layer,path:vec![0],index:1};
        assert_eq!(super::super::color::read_color(&doc,&middle),Some([0.0,1.0,0.0,1.0]));
        assert!(doc.undo());assert_eq!(doc.view().shapes(layer).unwrap(),before);
        assert!(edit(&doc,&slot,&json!({"stops":[{"offset":-1,"rgba":[0,0,0,1]}, {"offset":1,"rgba":[1,1,1,1]}]})).is_err());
        doc.apply(Intent::SetAttrs{layer,patch:LayerAttrsPatch{locked:Some(true),..Default::default()}}).unwrap();
        assert!(edit(&doc,&slot,&json!({"kind":"linear"})).is_err());
    }
}
