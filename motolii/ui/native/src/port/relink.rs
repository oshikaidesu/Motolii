//! A work's media that is not where the work last saw it, looked for in the catalog before it is called missing.
//!
//! The document remembers where a file was, how big it was and its fingerprint; the catalog remembers where every asset of
//! every registered source is now and where it has been. The resolver (`catalog/resolve.rs`) says exact / strong /
//! ambiguous / missing / source-unavailable. Here only the strong ones change the work (one relink each, in one undo step);
//! an ambiguous one is left as it is and reported with its candidates, so a person chooses; a missing one stays missing.
//! A wrong relink is worse than a missing file, so nothing weaker than the resolver's "strong" is ever applied.

use serde_json::json;

use super::*;
use crate::catalog::{Resolution, SavedRef};

impl EditorRuntime {
    pub(crate) fn relink_from_catalog(&mut self) -> Result<(), String> {
        let view = self.doc.view();
        let mut refs: Vec<(AssetId, String, SavedRef)> = Vec::new();
        for a in view.assets().map_err(e)? {
            let Some(path) = a.path_absolute.clone() else { continue };
            if std::path::Path::new(&path).exists() {
                continue;
            }
            let hash = a.content_hash.starts_with("motolii-source-v1:").then(|| a.content_hash.clone());
            refs.push((a.id, a.name.clone(), SavedRef { path, size: a.size_bytes, content_hash: hash, file_name: a.file_name.clone() }));
        }
        if refs.is_empty() {
            self.relink_report = Some(json!({"missing": 0, "relinked": [], "ambiguous": [], "unavailable": [], "unresolved": []}));
            return Ok(());
        }
        let results = crate::catalog::with(|c| c.resolve_many(&refs.iter().map(|(_, _, r)| r.clone()).collect::<Vec<_>>()))?;
        let (mut relinked, mut ambiguous, mut unavailable, mut unresolved) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let mut intents = Vec::new();
        for ((id, name, saved), result) in refs.iter().zip(results) {
            match result {
                Resolution::Exact(entry) | Resolution::Strong { entry, .. } => {
                    intents.push(Intent::RelinkAsset { asset: *id, path_absolute: entry.abs_path.clone(), project_root: None });
                    relinked.push(json!({"name": name, "from": saved.path, "to": entry.abs_path}));
                }
                Resolution::Ambiguous(candidates) => ambiguous.push(json!({"name": name, "from": saved.path, "candidates": candidates.iter().map(|c| c.abs_path.clone()).collect::<Vec<_>>()})),
                Resolution::SourceUnavailable { name: source, .. } => unavailable.push(json!({"name": name, "from": saved.path, "source": source})),
                Resolution::Missing => unresolved.push(json!({"name": name, "from": saved.path})),
            }
        }
        if !intents.is_empty() {
            self.apply(intents)?;
        }
        self.relink_report = Some(json!({"missing": refs.len(), "relinked": relinked, "ambiguous": ambiguous, "unavailable": unavailable, "unresolved": unresolved}));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(path: &std::path::Path, seed: u8) {
        let mut img = image::RgbImage::new(4, 4);
        for (i, p) in img.pixels_mut().enumerate() {
            *p = image::Rgb([seed, (i * 9) as u8, 200]);
        }
        img.save(path).unwrap();
    }

    fn asset_paths(rt: &EditorRuntime) -> Vec<String> {
        rt.doc.view().assets().unwrap().into_iter().filter_map(|a| a.path_absolute).collect()
    }

    /// A work whose media was moved while it was closed finds it again through the catalog: one undo step, only what is
    /// certain, and what is not certain is left alone and reported.
    #[test]
    fn a_work_finds_its_moved_media_through_the_catalog_and_nothing_uncertain() {
        crate::catalog::use_test_state();
        let folder = tempfile::tempdir().unwrap();
        let root = std::fs::canonicalize(folder.path()).unwrap();
        std::fs::create_dir_all(root.join("Assets/3D")).unwrap();
        let (a, b, c) = (root.join("Assets/3D/apple.png"), root.join("Assets/3D/pear.png"), root.join("Assets/3D/plum.png"));
        png(&a, 10);
        png(&b, 20);
        png(&c, 30);
        let mut rt = EditorRuntime::open("").unwrap();
        rt.request(json!({"op":"import","paths":[a.to_string_lossy(),b.to_string_lossy(),c.to_string_lossy()]})).unwrap();
        // apple and pear are moved into a new folder; plum is replaced by a look-alike of the same size (a different picture)
        std::fs::create_dir_all(root.join("Assets/models/fruit")).unwrap();
        std::fs::rename(&a, root.join("Assets/models/fruit/apple.png")).unwrap();
        std::fs::rename(&b, root.join("Assets/models/fruit/pear.png")).unwrap();
        std::fs::remove_file(&c).unwrap();
        // the catalog knows this folder as a source
        let source = crate::catalog::with(|cat| {
            let s = cat.add_source(&root, Some("relink-test")).unwrap();
            cat.refresh(Some(&s.id)).unwrap();
            s
        })
        .unwrap();

        rt.request(json!({"op":"relinkFromCatalog"})).unwrap();
        let report = rt.relink_report.clone().unwrap();
        assert_eq!(report["missing"], 3);
        assert_eq!(report["relinked"].as_array().unwrap().len(), 2, "{report}");
        assert_eq!(report["unresolved"].as_array().unwrap().len(), 1, "plum has nowhere to be found: it stays missing");
        let now = asset_paths(&rt);
        assert!(now.iter().any(|p| p.ends_with("Assets/models/fruit/apple.png")));
        assert!(now.iter().any(|p| p.ends_with("Assets/models/fruit/pear.png")));
        assert!(now.iter().any(|p| p.ends_with("Assets/3D/plum.png")), "the missing one still points where it did");

        // one step brings the old places back
        rt.request(json!({"op":"undo"})).unwrap();
        let back = asset_paths(&rt);
        assert!(back.iter().any(|p| p.ends_with("Assets/3D/apple.png")) && back.iter().any(|p| p.ends_with("Assets/3D/pear.png")), "one undo undoes both relinks: {back:?}");
        crate::catalog::with(|cat| cat.remove_source(&source.id).unwrap()).unwrap();
    }
}
