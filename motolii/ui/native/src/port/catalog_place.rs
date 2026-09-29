//! Using a file the catalog knows: the catalog only says what exists in the world; a work owns an asset from the moment it
//! uses one. So browsing writes nothing to the work, and placing a catalog asset admits it into the work's own asset table
//! (the same admission an import makes, deduplicated by content) and places it as a layer, in one undo step.

use super::*;

impl EditorRuntime {
    pub(crate) fn place_catalog_asset(&mut self, j: &J) -> Result<(), String> {
        let uid = j["id"].as_str().ok_or("Missing catalog id")?;
        let entry = crate::catalog::with(|c| c.entry(uid))?.ok_or("That asset is not available (its source cannot be read or the file is gone)")?;
        if !std::path::Path::new(&entry.abs_path).exists() {
            return Err("Asset file missing".into());
        }
        let draft = editor::fixture::prepare_path(std::path::Path::new(&entry.abs_path), AssetRole::Material)?;
        let known = self.doc.view().assets().map_err(e)?.into_iter().any(|a| a.content_hash == draft.content_hash);
        // an asset the work already holds is placed as it is: nothing is admitted twice
        let prelude = if known { Vec::new() } else { vec![Intent::AdmitAsset { draft }] };
        let start = j["start"].as_i64().unwrap_or(self.viewer.frame);
        let landing = j["placement"].as_str().map(|p| (j["target"].as_u64().map(LayerId), p.to_owned()));
        self.place_layer_with(prelude, editor::create::NewKind::Media { path: entry.abs_path, name: entry.filename }, start, landing, j["visibleFrames"].as_i64(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn placing_a_catalog_asset_admits_it_once_and_undoes_in_one_step() {
        crate::catalog::use_test_state();
        let folder = tempfile::tempdir().unwrap();
        let root = std::fs::canonicalize(folder.path()).unwrap();
        let mut img = image::RgbImage::new(4, 4);
        for (i, p) in img.pixels_mut().enumerate() {
            *p = image::Rgb([90, (i * 7) as u8, 30]);
        }
        img.save(root.join("cloud.png")).unwrap();
        let mut rt = EditorRuntime::open("").unwrap();
        let assets = |rt: &EditorRuntime| rt.doc.view().assets().unwrap().len();
        let layers = |rt: &EditorRuntime| rt.doc.view().layers().len();
        let (a0, l0) = (assets(&rt), layers(&rt));
        let (source, uid) = crate::catalog::with(|c| {
            let s = c.add_source(&root, Some("place-test")).unwrap();
            c.refresh(Some(&s.id)).unwrap();
            let uid = c.query(&crate::catalog::Query { sources: Some(vec![s.id.clone()]), ..Default::default() }).unwrap().entries[0].id.clone();
            (s, uid)
        })
        .unwrap();
        // browsing changed nothing in the work
        assert_eq!((assets(&rt), layers(&rt)), (a0, l0));
        rt.request(json!({"op":"placeCatalogAsset","id":uid})).unwrap();
        assert_eq!((assets(&rt), layers(&rt)), (a0 + 1, l0 + 1), "the first use admits the asset and places a layer");
        rt.request(json!({"op":"placeCatalogAsset","id":uid})).unwrap();
        assert_eq!((assets(&rt), layers(&rt)), (a0 + 1, l0 + 2), "the second use places again without admitting twice");
        rt.request(json!({"op":"undo"})).unwrap();
        rt.request(json!({"op":"undo"})).unwrap();
        assert_eq!((assets(&rt), layers(&rt)), (a0, l0), "one undo per use: the layer and the admission go together");
        assert!(rt.request(json!({"op":"placeCatalogAsset","id":"nope"})).is_err());
        crate::catalog::with(|c| c.remove_source(&source.id).unwrap()).unwrap();
    }
}
