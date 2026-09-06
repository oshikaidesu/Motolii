use serde::{Deserialize, Serialize};
use super::StoreError;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Notebook {
    pub pages: Vec<NotePage>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NotePage {
    pub id: String,
    pub title: String,
    pub blocks: Vec<NoteBlock>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NoteBlock {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(flatten)]
    pub content: NoteContent,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum NoteContent {
    Text { text: String },
    Image { png: String },
    Reference { label: String, layer: Option<u64>, start: i64, end: i64 },
}
impl Notebook {
    pub fn validate(&self) -> Result<(), StoreError> {
        let mut ids = std::collections::HashSet::new();
        for page in &self.pages {
            if page.id.is_empty() || !ids.insert(&page.id) { return Err(StoreError::Property("Duplicate or empty note page ID".into())); }
            let mut blocks = std::collections::HashSet::new();
            for b in &page.blocks {
                if b.id.is_empty() || !blocks.insert(&b.id) || ![b.x,b.y,b.width,b.height].iter().all(|n|n.is_finite()) || b.x < 0.0 || b.y < 0.0 || b.width < 40.0 || b.height < 32.0 {
                    return Err(StoreError::Property("Invalid note block identity or bounds".into()));
                }
                if let NoteContent::Reference{start,end,..} = &b.content { if end < start { return Err(StoreError::Property("Invalid note reference span".into())); } }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::{blank_project, Document, Intent};
    #[test]
    fn freeform_pages_persist_and_undo_without_touching_the_composition() {
        let mut doc=blank_project();
        let comp=doc.view().composition().unwrap();
        let book=Notebook{pages:vec![NotePage{id:"page".into(),title:"References".into(),blocks:vec![NoteBlock{id:"text".into(),x:24.0,y:80.0,width:220.0,height:120.0,content:NoteContent::Text{text:"Lighting study".into()}}]}]};
        doc.apply(Intent::SetNotebook{notebook:book.clone()}).unwrap();
        let mut moved=book.clone();moved.pages[0].blocks[0].x=340.0;
        doc.apply(Intent::SetNotebook{notebook:moved.clone()}).unwrap();
        assert!(doc.undo());assert_eq!(doc.view().notebook().unwrap(),book);
        let mut invalid=moved.clone();invalid.pages[0].blocks[0].width=-1.0;
        assert!(doc.apply(Intent::SetNotebook{notebook:invalid}).is_err());
        assert!(doc.redo());assert_eq!(doc.view().notebook().unwrap(),moved);
        let path=std::env::temp_dir().join(format!("motolii-notes-{}.rrd",std::process::id()));
        doc.save(&path).unwrap();let loaded=Document::load(&path).unwrap();std::fs::remove_file(path).unwrap();
        assert_eq!(loaded.view().notebook().unwrap(),moved);assert_eq!(loaded.view().composition().unwrap(),comp);
    }
}
