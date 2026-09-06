use crate::{EditorRuntime, doc::store::*};
use serde_json::Value as J;
use base64::Engine;
fn text<'a>(j:&'a J,key:&str)->Result<&'a str,String>{j[key].as_str().ok_or_else(||format!("Missing {key}"))}
impl EditorRuntime {
    pub(crate) fn edit_notes(&mut self,j:&J)->Result<(),String>{
        let mut book=self.doc.view().notebook().map_err(|e|e.to_string())?;
        let id=text(j,"page")?;
        let action=text(j,"action")?;
        if action=="addPage" {
            if book.pages.iter().any(|p|p.id==id){return Err("Page already exists".into())}
            book.pages.push(NotePage{id:id.into(),title:j["title"].as_str().unwrap_or("Untitled page").into(),blocks:vec![]});
        } else if action=="deletePage" {
            if !book.pages.iter().any(|p|p.id==id){return Err("Page no longer exists".into())}
            book.pages.retain(|p|p.id!=id);
        } else {
            let page=book.pages.iter_mut().find(|p|p.id==id).ok_or("Page no longer exists")?;
            match action {
                "renamePage"=>page.title=text(j,"title")?.into(),
                "putBlock"|"image"=>{
                    let mut raw=j["block"].clone();
                    if action=="image" {
                        let bytes=if let Some(path)=j["path"].as_str(){std::fs::read(path).map_err(|e|e.to_string())?}else{base64::engine::general_purpose::STANDARD.decode(text(j,"png")?).map_err(|e|e.to_string())?};
                        let image=image::load_from_memory(&bytes).map_err(|e|e.to_string())?;
                        let mut png=std::io::Cursor::new(Vec::new());
                        image.write_to(&mut png,image::ImageFormat::Png).map_err(|e|e.to_string())?;
                        raw["kind"]="image".into();raw["png"]=base64::engine::general_purpose::STANDARD.encode(png.into_inner()).into();
                        let width=raw["width"].as_f64().unwrap_or(240.0);
                        raw["height"]=(width*image.height()as f64/image.width().max(1)as f64).max(40.0).into();
                    }
                    let block:NoteBlock=serde_json::from_value(raw).map_err(|e|e.to_string())?;
                    if let Some(old)=page.blocks.iter_mut().find(|b|b.id==block.id){*old=block}else{page.blocks.push(block)}
                }
                "patchBlock"=>{
                    let block=page.blocks.iter_mut().find(|b|b.id==j["id"].as_str().unwrap_or("")).ok_or("Block no longer exists")?;
                    let mut raw=serde_json::to_value(&*block).map_err(|e|e.to_string())?;
                    let patch=j["patch"].as_object().ok_or("Missing patch")?;
                    for(key,value)in patch {if !["x","y","width","height","text"].contains(&key.as_str()){return Err("Unsupported note field".into())} raw[key]=value.clone();}
                    *block=serde_json::from_value(raw).map_err(|e|e.to_string())?;
                }
                "deleteBlock"=>page.blocks.retain(|b|Some(b.id.as_str())!=j["id"].as_str()),
                _=>return Err("Unknown notes action".into()),
            }
        }
        self.doc.apply(Intent::SetNotebook{notebook:book}).map_err(|e|e.to_string())
    }
}
