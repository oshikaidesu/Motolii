use crate::doc::store::{Document, LayerId, RationalTime, ContentTrack, ContentKeyframe, TextJustify};
use crate::doc::vector::{Brush, Canvas, Fill, Gradient, PathSource, Point, Shape};
use serde_json::{Value as J, json};
use base64::Engine as _;

pub(crate) fn reply(doc:&Document, time:RationalTime, j:&J) -> Result<J,String> {
    if j["kind"] == "blendBehavior" {
        let mode:crate::doc::store::BlendMode=serde_json::from_value(j["mode"].clone()).map_err(|e|e.to_string())?;
        return super::blend_preview::specimen(mode);
    }
    let raster = match j["kind"].as_str() {
        Some("font")=>{
            let id=LayerId(j["layer"].as_u64().ok_or("Missing layer")?);
            let mut text=doc.view().resolved_text_document(id,time).map_err(|e|e.to_string())?.ok_or("Select a Text layer")?;
            let family=j["family"].as_str().ok_or("Missing family")?;
            let content=text.content.eval(time).lines().next().unwrap_or("").split_whitespace().take(8).collect::<Vec<_>>().join(" ");
            if content.chars().count()>256 || !crate::doc::vector::text::font_supports_sample(family,&content) { return Ok(json!({"image":null})); }
            text.content=ContentTrack::new();text.content.insert(ContentKeyframe{t:RationalTime::ZERO,content});
            text.justify=TextJustify::Left;text.wrap_size=None;
            let scale=40.0/text.styles.first().ok_or("Text style missing")?.size.max(1.0);
            for style in &mut text.styles {
                style.font.family=family.into();style.font.path.clear();style.size*=scale;
                style.line_height=style.line_height.map(|h|h*scale);
                style.fill=[0.86,0.86,0.86,1.0];style.stroke_color=None;
            }
            crate::render::engine::text::rasterize_text_document(&text,RationalTime::ZERO,&Canvas{width:560,height:100,origin_x:0,origin_y:0}).map_err(|e|e.to_string())?
        },
        Some("gradient")=>{
            let stops=super::gradient::stops(&j["stops"])?;
            let kind=super::gradient::kind(&j["type"])?;
            let angle=j["angle"].as_f64().unwrap_or(0.0).to_radians();
            if !angle.is_finite() { return Err("Invalid angle".into()); }
            let (sin,cos)=angle.sin_cos();
            let extent=cos.abs()*140.0+sin.abs()*32.0;
            let gradient=Gradient{kind,start:Point{x:-cos*extent,y:-sin*extent},end:Point{x:cos*extent,y:sin*extent},stops};
            let shape=Shape{source:PathSource::Rectangle{size:Point{x:280.0,y:64.0}},ops:vec![],stroke:None,fill:Some(Fill{brush:Brush::Gradient(gradient),..Default::default()})};
            Some(crate::doc::vector::render(&shape,&Canvas::centered(280,64)).map_err(|e|e.to_string())?)
        },
        _=>return Err("Unknown visual sample".into()),
    };
    let Some(raster)=raster else{return Ok(json!({"image":null}))};
    let mut pixels=raster.premultiplied_rgba8;
    for pixel in pixels.chunks_exact_mut(4) { let a=pixel[3] as u32;if a>0 {for c in &mut pixel[..3]{*c=((*c as u32*255+a/2)/a).min(255) as u8;}} }
    let image=image::RgbaImage::from_raw(raster.width,raster.height,pixels).ok_or("Invalid sample")?;
    let mut png=std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image).write_to(&mut png,image::ImageFormat::Png).map_err(|e|e.to_string())?;
    Ok(json!({"image":base64::engine::general_purpose::STANDARD.encode(png.into_inner())}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gradient_sample_has_the_authored_end_colors_and_no_history() {
        let doc=crate::doc::store::blank_project();let before=doc.history_depth();
        let result=reply(&doc,RationalTime::ZERO,&json!({"kind":"gradient","stops":[[1,0,0,1],[0,0,1,1]]})).unwrap();
        let png=base64::engine::general_purpose::STANDARD.decode(result["image"].as_str().unwrap()).unwrap();
        let image=image::load_from_memory(&png).unwrap().to_rgba8();
        assert!(image.get_pixel(1,32)[0]>250);assert!(image.get_pixel(278,32)[2]>250);
        assert_eq!(doc.history_depth(),before);
    }
    #[test]
    fn font_sample_uses_selected_content_without_a_document_edit() {
        use crate::doc::store::*;
        let mut doc=blank_project();let layer=LayerId(1);
        doc.apply_all(crate::editor::create::new_layer_intents(layer,0,0,60,Fps::try_new(30,1).unwrap(),(1920.0,1080.0),crate::editor::create::NewKind::Text,None)).unwrap();
        let before=doc.view().text_document(layer).unwrap();let history=doc.history_depth();
        let family=&before.as_ref().unwrap().styles[0].font.family;
        let result=reply(&doc,RationalTime::ZERO,&json!({"kind":"font","layer":1,"family":family})).unwrap();
        assert!(result["image"].is_string());
        assert!(!crate::doc::vector::text::font_supports_sample(family,"\u{10ffff}"));
        assert_eq!(doc.view().text_document(layer).unwrap(),before);assert_eq!(doc.history_depth(),history);
    }
}
