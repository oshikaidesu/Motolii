#[allow(unused_imports)]
use crate::edit::{Animate, Document, Intent};
use crate::{EditorRuntime, doc::store::*};
use serde_json::Value as J;
use base64::Engine;
fn text<'a>(j:&'a J,key:&str)->Result<&'a str,String>{j[key].as_str().ok_or_else(||format!("Missing {key}"))}
impl EditorRuntime {
    pub(crate) fn edit_notes(&mut self,j:&J)->Result<(),String>{
        let mut book=self.doc.view().notebook().map_err(|e|e.to_string())?;
        let action=text(j,"action")?;
        // no page named: a new page takes the next free id; a block goes on the first page, made in the same edit
        // when the notebook has none yet
        let fresh;
        let id=match j["page"].as_str(){
            Some(id)=>id,
            None if action=="addPage"=>{let mut n=book.pages.len()+1;while book.pages.iter().any(|p|p.id==format!("p{n}")){n+=1;}fresh=format!("p{n}");fresh.as_str()}
            None=>{if book.pages.is_empty(){book.pages.push(NotePage{id:"p1".into(),title:"Page 1".into(),blocks:vec![]});}fresh=book.pages[0].id.clone();fresh.as_str()}
        };
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
                    let block=if action=="image"{image_block(j)?}else{serde_json::from_value(j["block"].clone()).map_err(|e|e.to_string())?};
                    put(page,block);
                }
                // several pictures dropped at once: one edit, one undo step
                "images"=>for item in j["images"].as_array().ok_or("Missing images")?{put(page,image_block(item)?)},
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

fn put(page:&mut NotePage,block:NoteBlock){
    if let Some(old)=page.blocks.iter_mut().find(|b|b.id==block.id){*old=block}else{page.blocks.push(block)}
}
/// An image card: the picture from `path` (or `png`, base64) kept as PNG, its height following the picture's shape.
fn image_block(j:&J)->Result<NoteBlock,String>{
    let mut raw=j["block"].clone();
    let bytes=if let Some(path)=j["path"].as_str(){std::fs::read(path).map_err(|e|e.to_string())?}else{base64::engine::general_purpose::STANDARD.decode(text(j,"png")?).map_err(|e|e.to_string())?};
    let image=image::load_from_memory(&bytes).map_err(|e|e.to_string())?;
    let mut png=std::io::Cursor::new(Vec::new());
    image.write_to(&mut png,image::ImageFormat::Png).map_err(|e|e.to_string())?;
    raw["kind"]="image".into();raw["png"]=base64::engine::general_purpose::STANDARD.encode(png.into_inner()).into();
    let width=raw["width"].as_f64().unwrap_or(240.0);
    raw["height"]=(width*image.height()as f64/image.width().max(1)as f64).max(40.0).into();
    serde_json::from_value(raw).map_err(|e|e.to_string())
}

#[cfg(test)]
mod tests {
    use crate::EditorRuntime;
    use base64::Engine;
    use serde_json::json;
    /// Pictures dropped together become cards in one step.
    #[test]
    fn dropped_pictures_are_one_step(){
        let mut rt=EditorRuntime::open("").unwrap();
        rt.request(json!({"op":"notes","page":"p1","action":"addPage"})).unwrap();
        let mut png=std::io::Cursor::new(Vec::new());
        image::RgbaImage::new(4,2).write_to(&mut png,image::ImageFormat::Png).unwrap();
        let png=base64::engine::general_purpose::STANDARD.encode(png.into_inner());
        let card=|id:&str|json!({"png":png,"block":{"id":id,"kind":"image","x":0.0,"y":0.0,"width":100.0,"height":80.0,"text":"","z":0}});
        rt.request(json!({"op":"notes","page":"p1","action":"images","images":[card("a"),card("b")]})).unwrap();
        let blocks=|rt:&EditorRuntime|rt.doc.view().notebook().unwrap().pages[0].blocks.len();
        assert_eq!(blocks(&rt),2);
        assert_eq!(rt.doc.view().notebook().unwrap().pages[0].blocks[0].height,50.0,"the card follows the picture's shape");
        rt.request(json!({"op":"undo"})).unwrap();
        assert_eq!(blocks(&rt),0,"one drop, one undo");
    }
}

#[cfg(test)]
mod pages {
    use crate::EditorRuntime;
    use serde_json::json;
    /// The host names new pages, and a first note on an empty notebook makes its page in the same step.
    #[test]
    fn the_host_names_pages_and_makes_the_first_one(){
        let mut rt=EditorRuntime::open("").unwrap();
        let block=json!({"id":"n1","kind":"text","x":0.0,"y":0.0,"width":100.0,"height":80.0,"text":"hi","z":0});
        rt.request(json!({"op":"notes","action":"putBlock","block":block})).unwrap();
        let book=rt.doc.view().notebook().unwrap();
        assert_eq!((book.pages.len(),book.pages[0].blocks.len()),(1,1));
        rt.request(json!({"op":"undo"})).unwrap();
        assert!(rt.doc.view().notebook().unwrap().pages.is_empty(),"page and note were one step");
        rt.request(json!({"op":"notes","action":"addPage","title":"A"})).unwrap();
        rt.request(json!({"op":"notes","action":"addPage","title":"B"})).unwrap();
        let ids:Vec<_>=rt.doc.view().notebook().unwrap().pages.iter().map(|p|p.id.clone()).collect();
        assert_eq!(ids,vec!["p1","p2"],"two quick presses, two pages");
    }
}
