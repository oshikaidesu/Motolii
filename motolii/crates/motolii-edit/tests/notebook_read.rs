//! コアの読みを、編集で組んだ作品で確かめる。組むのに編集が要るのでこの家に置く。

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use motolii_doc::store::*;
    use motolii_edit::{blank_project, Animate, Document, Intent};
    use super::*;
    use motolii_doc::store::{};
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

