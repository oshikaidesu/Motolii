//! The Catalog against real (temporary) folder trees: nothing here touches a person's own folders, and every test also
//! checks that no source file was changed.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::*;

struct Tree {
    dir: tempfile::TempDir,
    n: u64,
}

impl Tree {
    fn new() -> Self {
        Self { dir: tempfile::tempdir().unwrap(), n: 0 }
    }
    fn path(&self, rel: &str) -> PathBuf {
        self.dir.path().join(rel)
    }
    /// A file whose bytes are its own (so its fingerprint is its own).
    fn write(&mut self, rel: &str) -> PathBuf {
        self.n += 1;
        let p = self.path(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, format!("{rel}#{}", self.n).repeat(4)).unwrap();
        p
    }
    fn write_bytes(&self, rel: &str, bytes: &[u8]) -> PathBuf {
        let p = self.path(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, bytes).unwrap();
        p
    }
    fn mv(&self, from: &str, to: &str) {
        let dest = self.path(to);
        fs::create_dir_all(dest.parent().unwrap()).unwrap();
        fs::rename(self.path(from), dest).unwrap();
    }
    /// A copy with a new inode (what moving across volumes looks like to the catalog).
    fn copy(&self, from: &str, to: &str) {
        let dest = self.path(to);
        fs::create_dir_all(dest.parent().unwrap()).unwrap();
        fs::copy(self.path(from), dest).unwrap();
    }
    fn rm(&self, rel: &str) {
        fs::remove_file(self.path(rel)).unwrap();
    }
    /// Every file's bytes and modified time, to prove nothing was written.
    fn snapshot(&self) -> BTreeMap<String, (Vec<u8>, std::time::SystemTime)> {
        let mut out = BTreeMap::new();
        for e in walkdir::WalkDir::new(self.dir.path()).into_iter().flatten() {
            if e.file_type().is_file() {
                out.insert(e.path().to_string_lossy().into_owned(), (fs::read(e.path()).unwrap(), e.metadata().unwrap().modified().unwrap()));
            }
        }
        out
    }
}

fn catalog() -> Catalog {
    Catalog::open_memory().unwrap()
}

fn all(c: &Catalog) -> Vec<Entry> {
    c.query(&Query::default()).unwrap().entries
}

fn names(rs: &ResultSet) -> Vec<String> {
    rs.entries.iter().map(|e| e.rel_path.clone()).collect()
}

fn saved(path: &Path) -> SavedRef {
    SavedRef {
        path: path.to_string_lossy().into_owned(),
        size: fs::metadata(path).ok().map(|m| m.len()),
        content_hash: crate::doc::store::SourceFingerprintV1::from_edges(path).ok().map(|f| f.content_hash()),
        file_name: path.file_name().map(|n| n.to_string_lossy().into_owned()),
    }
}

/// A reference as a work remembers it before it knew its place changed: the path it had, canonical (the catalog's roots are).
fn saved_at(t: &Tree, rel: &str) -> SavedRef {
    let mut r = saved(&t.path(rel));
    r.path = fs::canonicalize(t.dir.path()).unwrap().join(rel).to_string_lossy().into_owned();
    r
}

#[test]
fn several_roots_are_indexed_recursively_and_queried_together() {
    let (mut a, mut b) = (Tree::new(), Tree::new());
    a.write("same.png");
    a.write("nested/a.mov");
    a.write("notes.txt"); // not a media type: never enters the catalog
    b.write("same.png");
    b.write("deep/er/model.glb");
    let before = (a.snapshot(), b.snapshot());
    let mut c = catalog();
    let sa = c.add_source(a.dir.path(), Some("A")).unwrap();
    let sb = c.add_source(b.dir.path(), Some("B")).unwrap();
    let reports = c.refresh(None).unwrap();
    assert_eq!(reports.iter().map(|r| r.added).sum::<usize>(), 4);
    assert_eq!(all(&c).len(), 4);

    // the same file name in two roots is two assets
    let same = c.query(&Query { text: Some("same".into()), ..Default::default() }).unwrap();
    assert_eq!(same.total, 2);
    assert_ne!(same.entries[0].id, same.entries[1].id);

    // Where, What, Which are independent axes
    let only_a = c.query(&Query { sources: Some(vec![sa.id.clone()]), ..Default::default() }).unwrap();
    assert_eq!(names(&only_a).len(), 2);
    let videos = c.query(&Query { kinds: Some(vec![MediaKind::Video]), ..Default::default() }).unwrap();
    assert_eq!(names(&videos), vec!["nested/a.mov"]);
    let b_models = c.query(&Query { sources: Some(vec![sb.id.clone()]), kinds: Some(vec![MediaKind::Model]), text: Some("mod".into()), ..Default::default() }).unwrap();
    assert_eq!(names(&b_models), vec!["deep/er/model.glb"]);
    let both = c.query(&Query { sources: Some(vec![sa.id.clone(), sb.id.clone()]), kinds: Some(vec![MediaKind::Image]), ..Default::default() }).unwrap();
    assert_eq!(both.total, 2);
    // a one- or two-letter word still matches (no trigram to use)
    assert_eq!(c.query(&Query { text: Some("er".into()), ..Default::default() }).unwrap().total, 1);
    // several words all have to match
    assert_eq!(c.query(&Query { text: Some("deep model".into()), ..Default::default() }).unwrap().total, 1);
    assert_eq!(c.query(&Query { text: Some("deep same".into()), ..Default::default() }).unwrap().total, 0);
    assert_eq!((a.snapshot(), b.snapshot()), before, "no source file changed");
}

#[test]
fn folder_browsing_and_the_library_come_from_one_catalog() {
    let mut t = Tree::new();
    t.write("top.png");
    t.write("m/one.png");
    t.write("m/deep/two.png");
    t.write("n/three.png");
    let mut c = catalog();
    let s = c.add_source(t.dir.path(), None).unwrap();
    c.refresh(None).unwrap();
    let folder = |prefix: &str, direct: bool| c.query(&Query { folder: Some((s.id.clone(), prefix.into())), direct, ..Default::default() }).unwrap();
    assert_eq!(folder("m", false).total, 2);
    assert_eq!(names(&folder("m", true)), vec!["m/one.png"]);
    assert_eq!(names(&folder("", true)), vec!["top.png"]);
    assert_eq!(folder("m/deep", true).total, 1);
    assert_eq!(c.folders(&s.id, "").unwrap(), vec![FolderCount { name: "m".into(), assets: 2 }, FolderCount { name: "n".into(), assets: 1 }]);
    assert_eq!(c.folders(&s.id, "m").unwrap(), vec![FolderCount { name: "deep".into(), assets: 1 }]);
    // a folder query and a type query combine
    let q = Query { folder: Some((s.id.clone(), "m".into())), kinds: Some(vec![MediaKind::Video]), ..Default::default() };
    assert_eq!(c.query(&q).unwrap().total, 0);
}

#[test]
fn nothing_changed_means_nothing_written() {
    let mut t = Tree::new();
    t.write("a.png");
    let mut c = catalog();
    c.add_source(t.dir.path(), None).unwrap();
    c.refresh(None).unwrap();
    let rev = c.revision();
    let again = c.refresh(None).unwrap();
    assert_eq!((again[0].added, again[0].moved, again[0].missing, again[0].changed), (0, 0, 0, 0));
    assert_eq!(c.revision(), rev, "an unchanged refresh does not bump the revision");
}

#[test]
fn a_rename_keeps_the_asset_and_an_old_reference_finds_it() {
    let mut t = Tree::new();
    t.write("A/same.png");
    t.write("A/other.png");
    let mut c = catalog();
    c.add_source(t.dir.path(), None).unwrap();
    c.refresh(None).unwrap();
    let id = all(&c).into_iter().find(|e| e.filename == "same.png").unwrap().id;
    let reference = saved_at(&t, "A/same.png");
    t.mv("A/same.png", "A/renamed.png");
    let r = &c.refresh(None).unwrap()[0];
    assert_eq!((r.moved, r.added, r.missing), (1, 0, 0));
    let now = all(&c).into_iter().find(|e| e.filename == "renamed.png").unwrap();
    assert_eq!(now.id, id, "the asset is the same one");
    match &c.resolve_many(&[reference])[0] {
        Resolution::Strong { entry, .. } => assert_eq!(entry.id, id),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_move_between_subdirectories_keeps_the_asset() {
    let mut t = Tree::new();
    t.write("A/nested/a.mov");
    let mut c = catalog();
    c.add_source(t.dir.path(), None).unwrap();
    c.refresh(None).unwrap();
    let id = all(&c)[0].id.clone();
    let reference = saved_at(&t, "A/nested/a.mov");
    t.mv("A/nested/a.mov", "A/x/y/a.mov");
    c.refresh(None).unwrap();
    assert_eq!(all(&c)[0].id, id);
    assert!(matches!(&c.resolve_many(&[reference])[0], Resolution::Strong { entry, .. } if entry.id == id));
}

#[test]
fn a_whole_folder_moving_is_one_fact() {
    let mut t = Tree::new();
    for rel in ["A/foo.png", "A/bar.mov", "A/models/x.glb"] {
        t.write(rel);
    }
    let mut c = catalog();
    c.add_source(t.dir.path(), None).unwrap();
    c.refresh(None).unwrap();
    let refs: Vec<SavedRef> = ["A/foo.png", "A/bar.mov", "A/models/x.glb"].iter().map(|r| saved_at(&t, r)).collect();
    let mut before: Vec<String> = all(&c).into_iter().map(|e| e.id).collect();
    fs::create_dir_all(t.path("Archive")).unwrap();
    t.mv("A", "Archive/A");
    let r = &c.refresh(None).unwrap()[0];
    assert_eq!((r.moved, r.missing, r.added), (3, 0, 0));
    assert_eq!(r.folder_moves, vec![("".to_string(), "Archive".to_string())]);
    let mut after: Vec<String> = all(&c).into_iter().map(|e| e.id).collect();
    after.sort();
    before.sort();
    assert_eq!(after, before, "every asset survived the move");
    for res in c.resolve_many(&refs) {
        assert!(matches!(res, Resolution::Strong { .. }), "{res:?}");
    }
}

#[test]
fn a_folder_that_moved_while_unseen_is_recognised_from_one_file_for_the_rest() {
    // the references were saved before the move and the catalog never saw the old place: the first one's content finds it,
    // and what that shows about the folder finds the ones that have no content hash saved
    let mut t = Tree::new();
    t.write("Old/kept.png");
    t.write("Old/a.png");
    t.write("Old/sub/b.png");
    let mut refs = vec![saved_at(&t, "Old/kept.png"), saved_at(&t, "Old/a.png"), saved_at(&t, "Old/sub/b.png")];
    for r in &mut refs[1..] {
        r.content_hash = None;
        r.file_name = None;
    }
    t.mv("Old", "New/Old");
    let mut c = catalog();
    c.add_source(t.dir.path(), None).unwrap();
    c.refresh(None).unwrap();
    let out = c.resolve_many(&refs);
    assert!(matches!(&out[0], Resolution::Strong { .. }), "{:?}", out[0]);
    // the two without a hash are Strong through the folder, not merely candidates
    assert!(matches!(&out[1], Resolution::Strong { why, .. } if why.contains("folder")), "{:?}", out[1]);
    assert!(matches!(&out[2], Resolution::Strong { why, .. } if why.contains("folder")), "{:?}", out[2]);
}

#[test]
fn a_deleted_file_is_missing_and_stays_known() {
    let mut t = Tree::new();
    t.write("gone.png");
    t.write("kept.png");
    let mut c = catalog();
    let s = c.add_source(t.dir.path(), None).unwrap();
    c.refresh(None).unwrap();
    let reference = saved_at(&t, "gone.png");
    t.rm("gone.png");
    let r = &c.refresh(None).unwrap()[0];
    assert_eq!((r.missing, r.added, r.moved), (1, 0, 0));
    assert_eq!(all(&c).len(), 1, "a library view lists what is there");
    let with_missing = c.query(&Query { include_missing: true, ..Default::default() }).unwrap();
    assert_eq!(with_missing.total, 2);
    assert!(with_missing.entries.iter().any(|e| e.missing && e.filename == "gone.png"));
    assert!(matches!(c.resolve_many(&[reference])[0], Resolution::Missing));
    assert_eq!(c.sources()[0].assets, 1);
    // forgetting is a person's act and touches no file
    assert_eq!(c.forget_missing(&s.id).unwrap(), 1);
    assert_eq!(c.query(&Query { include_missing: true, ..Default::default() }).unwrap().total, 1);
}

#[test]
fn a_different_file_with_the_old_name_is_not_the_old_asset() {
    let mut t = Tree::new();
    t.write("A/same.png");
    let mut c = catalog();
    c.add_source(t.dir.path(), None).unwrap();
    c.refresh(None).unwrap();
    c.enrich(100);
    let old_id = all(&c)[0].id.clone();
    let reference = saved_at(&t, "A/same.png");
    t.rm("A/same.png");
    t.write_bytes("A/same.png", b"a completely different picture");
    let r = &c.refresh(None).unwrap()[0];
    assert_eq!((r.added, r.missing), (1, 1));
    let now = all(&c);
    assert_eq!(now.len(), 1);
    assert_ne!(now[0].id, old_id, "a new asset stands at the old name");
    // the work's reference (the old content) is not joined to it
    assert!(matches!(c.resolve_many(&[reference])[0], Resolution::Missing));
}

#[test]
fn an_identical_replacement_is_still_the_same_asset() {
    let mut t = Tree::new();
    let p = t.write("A/same.png");
    let mut c = catalog();
    c.add_source(t.dir.path(), None).unwrap();
    c.refresh(None).unwrap();
    c.enrich(100);
    let id = all(&c)[0].id.clone();
    let bytes = fs::read(&p).unwrap();
    t.rm("A/same.png");
    t.write_bytes("A/same.png", &bytes); // a save that swapped the file: new inode, same content
    c.refresh(None).unwrap();
    assert_eq!(all(&c)[0].id, id);
}

#[test]
fn a_copy_is_a_second_asset_and_nothing_is_merged() {
    let mut t = Tree::new();
    t.write("A/original.mov");
    let mut c = catalog();
    c.add_source(t.dir.path(), None).unwrap();
    c.refresh(None).unwrap();
    c.enrich(100);
    let id = all(&c)[0].id.clone();
    let reference = saved_at(&t, "A/original.mov");
    t.copy("A/original.mov", "A/copy.mov");
    c.refresh(None).unwrap();
    let now = all(&c);
    assert_eq!(now.len(), 2);
    assert!(now.iter().any(|e| e.id == id && e.filename == "original.mov"), "the original keeps its identity");
    assert_ne!(now[0].id, now[1].id, "the copy is its own asset");
    // the work remembers the original: it is still there, so it is found exactly, not swapped for its twin
    assert!(matches!(&c.resolve_many(&[reference.clone()])[0], Resolution::Exact(e) if e.id == id));
    // with the original gone, the twin is the same content elsewhere: that is one strong match, found by content
    c.enrich(100);
    t.rm("A/original.mov");
    c.refresh(None).unwrap();
    assert!(matches!(&c.resolve_many(&[reference])[0], Resolution::Strong { entry, .. } if entry.filename == "copy.mov"));
}

#[test]
fn a_reference_with_two_equal_candidates_is_ambiguous() {
    let mut t = Tree::new();
    let content = b"the same bytes";
    t.write_bytes("A/one.png", content);
    t.write_bytes("A/two.png", content);
    let mut c = catalog();
    c.add_source(t.dir.path(), None).unwrap();
    c.refresh(None).unwrap();
    c.enrich(100);
    let mut r = saved_at(&t, "A/one.png");
    r.path = fs::canonicalize(t.dir.path()).unwrap().join("Elsewhere/three.png").to_string_lossy().into_owned();
    assert!(matches!(&c.resolve_many(&[r])[0], Resolution::Ambiguous(v) if v.len() == 2));
}

#[test]
fn two_identical_files_that_moved_across_volumes_are_not_paired_by_guess() {
    let mut t = Tree::new();
    let content = b"the very same bytes in two files";
    t.write_bytes("A/x.png", content);
    t.write_bytes("A/y.png", content);
    let mut c = catalog();
    c.add_source(t.dir.path(), None).unwrap();
    c.refresh(None).unwrap();
    c.enrich(100);
    // carried elsewhere as copies (new inodes), originals removed
    t.copy("A/x.png", "B/x.png");
    t.copy("A/y.png", "B/y.png");
    t.rm("A/x.png");
    t.rm("A/y.png");
    let r = &c.refresh(None).unwrap()[0];
    assert_eq!(r.moved, 0, "equally good pairings are left unlinked");
    assert!(r.ambiguous >= 1);
    assert_eq!((r.added, r.missing), (2, 2));
}

#[test]
fn a_copy_carried_to_another_volume_is_followed_when_it_is_unique() {
    let mut t = Tree::new();
    t.write("A/x.png");
    let mut c = catalog();
    c.add_source(t.dir.path(), None).unwrap();
    c.refresh(None).unwrap();
    c.enrich(100);
    let id = all(&c)[0].id.clone();
    t.copy("A/x.png", "B/x.png");
    t.rm("A/x.png");
    let r = &c.refresh(None).unwrap()[0];
    assert_eq!((r.moved, r.added, r.missing), (1, 0, 0));
    assert_eq!(all(&c)[0].id, id);
}

#[test]
fn a_disabled_source_is_out_of_every_query_and_refresh() {
    let (mut a, mut b) = (Tree::new(), Tree::new());
    a.write("a.png");
    b.write("b.png");
    let mut c = catalog();
    let sa = c.add_source(a.dir.path(), None).unwrap();
    c.add_source(b.dir.path(), None).unwrap();
    c.refresh(None).unwrap();
    c.set_enabled(&sa.id, false).unwrap();
    assert_eq!(names(&c.query(&Query::default()).unwrap()), vec!["b.png"]);
    a.write("later.png");
    c.refresh(None).unwrap();
    c.set_enabled(&sa.id, true).unwrap();
    assert_eq!(all(&c).len(), 2, "the change made while it was off is not seen until the next refresh");
    c.refresh(None).unwrap();
    assert_eq!(all(&c).len(), 3);
}

#[cfg(unix)]
#[test]
fn unsupported_hidden_broken_and_looping_entries_do_not_stop_the_scan() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let mut t = Tree::new();
    t.write("ok.png");
    t.write("notes.txt");
    t.write(".hidden.png");
    t.write(".cache/inside.png");
    // a folder link back to the root would loop if followed
    symlink(t.dir.path(), t.path("loop")).unwrap();
    // a link to a file is read through, once; a broken link is an error counted, never fatal
    symlink(t.path("ok.png"), t.path("alias.png")).unwrap();
    symlink(t.path("no-such.png"), t.path("broken.png")).unwrap();
    // an unreadable folder
    let locked = t.path("locked");
    fs::create_dir_all(&locked).unwrap();
    fs::write(locked.join("hidden-from-us.png"), b"x").unwrap();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
    let mut c = catalog();
    c.add_source(t.dir.path(), None).unwrap();
    let r = c.refresh(None).unwrap().remove(0);
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
    let mut got = names(&c.query(&Query::default()).unwrap());
    got.sort();
    assert_eq!(got, vec!["alias.png", "ok.png"]);
    assert!(r.errors.len() >= 2, "the broken link and the locked folder are counted: {:?}", r.errors);
    assert!(!r.unavailable);
}

#[test]
fn an_unreadable_source_is_unavailable_not_a_pile_of_missing_assets() {
    let mut t = Tree::new();
    t.write("S/a.png");
    t.write("S/b.mov");
    let mut c = catalog();
    let s = c.add_source(&t.path("S"), Some("Drive")).unwrap();
    c.refresh(None).unwrap();
    let reference = saved_at(&t, "S/a.png");
    t.mv("S", "S-unplugged");
    let r = &c.refresh(None).unwrap()[0];
    assert!(r.unavailable);
    assert_eq!(r.missing, 0);
    assert!(!c.sources()[0].available);
    let listed = all(&c);
    assert_eq!(listed.len(), 2, "what was indexed is still shown");
    assert!(listed.iter().all(|e| !e.missing && !e.source_available));
    assert!(matches!(&c.resolve_many(&[reference])[0], Resolution::SourceUnavailable { source, .. } if *source == s.id));
}

#[test]
fn an_emptied_root_is_not_taken_for_a_deletion_of_everything() {
    let mut t = Tree::new();
    t.write("S/a.png");
    t.write("S/b.png");
    let mut c = catalog();
    c.add_source(&t.path("S"), None).unwrap();
    c.refresh(None).unwrap();
    t.rm("S/a.png");
    t.rm("S/b.png"); // the mount point stands, nothing in it
    let r = &c.refresh(None).unwrap()[0];
    assert!(r.unavailable);
    assert_eq!(c.sources()[0].assets, 2);
}

#[test]
fn a_source_root_that_moved_is_reconnected_with_its_index() {
    let mut t = Tree::new();
    t.write("Vol/a.png");
    t.write("Vol/sub/b.mov");
    let mut c = catalog();
    let s = c.add_source(&t.path("Vol"), None).unwrap();
    c.refresh(None).unwrap();
    let ids: Vec<String> = all(&c).into_iter().map(|e| e.id).collect();
    t.mv("Vol", "Mounted/Elsewhere");
    assert!(c.refresh(None).unwrap()[0].unavailable);
    // the wrong folder is refused
    t.write("Decoy/x.png");
    assert!(c.relocate_source(&s.id, &t.path("Decoy")).is_err());
    c.relocate_source(&s.id, &t.path("Mounted/Elsewhere")).unwrap();
    let r = &c.refresh(None).unwrap()[0];
    assert_eq!((r.added, r.moved, r.missing, r.changed), (0, 0, 0, 0), "the index carried over");
    let now = all(&c);
    assert_eq!(now.len(), ids.len());
    assert!(now.iter().all(|e| ids.contains(&e.id) && e.source_available));
    assert!(now[0].abs_path.contains("Elsewhere"));
}

#[test]
fn overlapping_sources_are_refused() {
    let mut t = Tree::new();
    t.write("A/x.png");
    let mut c = catalog();
    c.add_source(&t.path("A"), None).unwrap();
    assert!(c.add_source(t.dir.path(), None).is_err());
    fs::create_dir_all(t.path("A/inner")).unwrap();
    assert!(c.add_source(&t.path("A/inner"), None).is_err());
}

#[test]
fn removing_a_source_or_the_index_never_touches_a_file() {
    let mut t = Tree::new();
    t.write("A/x.png");
    t.write("A/y.mov");
    let before = t.snapshot();
    let mut c = catalog();
    let s = c.add_source(&t.path("A"), None).unwrap();
    c.refresh(None).unwrap();
    c.enrich(100);
    c.relocate_source(&s.id, &t.path("A")).unwrap();
    c.remove_source(&s.id).unwrap();
    assert!(c.sources().is_empty());
    assert!(all(&c).is_empty());
    assert_eq!(t.snapshot(), before);
}

#[test]
fn the_index_persists_and_reopens() {
    let mut t = Tree::new();
    t.write("A/x.png");
    t.write("A/y.mov");
    let db = t.path("state/catalog.sqlite");
    let (id, rev) = {
        let mut c = Catalog::open(&db).unwrap();
        c.add_source(&t.path("A"), Some("A")).unwrap();
        c.refresh(None).unwrap();
        (all(&c)[0].id.clone(), c.revision())
    };
    let mut c = Catalog::open(&db).unwrap();
    assert_eq!(c.sources().len(), 1);
    assert_eq!(all(&c).len(), 2);
    assert_eq!(all(&c)[0].id, id);
    assert_eq!(c.revision(), rev);
    // and a rename made while it was closed is a rename, not a new file
    t.mv("A/x.png", "A/z.png");
    let r = &c.refresh(None).unwrap()[0];
    assert_eq!((r.moved, r.added, r.missing), (1, 0, 0));
}

#[test]
fn enrichment_fills_dimensions_and_fingerprints_in_batches() {
    let mut t = Tree::new();
    let mut png = Vec::new();
    image::DynamicImage::new_rgb8(7, 5).write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).unwrap();
    t.write_bytes("a/pic.png", &png);
    t.write("a/clip.mov");
    let mut c = catalog();
    c.add_source(t.dir.path(), None).unwrap();
    c.refresh(None).unwrap();
    assert_eq!(c.pending_enrich(), 2);
    assert_eq!(c.enrich(1), 1);
    assert_eq!(c.enrich(10), 1);
    assert_eq!(c.pending_enrich(), 0);
    let pic = all(&c).into_iter().find(|e| e.filename == "pic.png").unwrap();
    assert_eq!((pic.width, pic.height), (Some(7), Some(5)));
    assert!(pic.fingerprint.as_deref().is_some_and(|f| f.starts_with("motolii-source-v1:")));
}

#[test]
fn a_page_of_a_query_is_a_slice_with_the_total() {
    let mut t = Tree::new();
    for i in 0..25 {
        t.write(&format!("f{i:02}.png"));
    }
    let mut c = catalog();
    c.add_source(t.dir.path(), None).unwrap();
    c.refresh(None).unwrap();
    let page = c.query(&Query { offset: 20, limit: 10, ..Default::default() }).unwrap();
    assert_eq!((page.total, page.entries.len(), page.offset), (25, 5, 20));
    assert_eq!(page.revision, c.revision());
}

#[test]
fn the_owner_orders_a_result_set() {
    let mut t = Tree::new();
    t.write_bytes("b.png", &[0; 30]);
    t.write_bytes("a.mov", &[0; 10]);
    t.write_bytes("c.wav", &[0; 20]);
    let mut c = catalog();
    c.add_source(t.dir.path(), None).unwrap();
    c.refresh(None).unwrap();
    let order = |sort, descending| names(&c.query(&Query { sort, descending, ..Default::default() }).unwrap());
    assert_eq!(order(Sort::Name, false), vec!["a.mov", "b.png", "c.wav"]);
    assert_eq!(order(Sort::Name, true), vec!["c.wav", "b.png", "a.mov"]);
    assert_eq!(order(Sort::Size, false), vec!["a.mov", "c.wav", "b.png"]);
    assert_eq!(order(Sort::Kind, false), vec!["c.wav", "b.png", "a.mov"]);
}



#[test]
fn a_change_under_a_watched_source_is_indexed_without_asking() {
    use std::time::{Duration, Instant};
    let mut t = Tree::new();
    t.write("Src/a.png");
    let root = fs::canonicalize(t.path("Src")).unwrap();
    super::use_test_state();
    let id = super::with(|c| {
        let s = c.add_source(&root, Some("watched")).unwrap();
        c.refresh(Some(&s.id)).unwrap();
        s.id
    })
    .unwrap();
    assert_eq!(super::watch::start().unwrap() >= 1, true);
    let count = |id: &str| super::with(|c| c.query(&Query { sources: Some(vec![id.to_owned()]), ..Default::default() }).unwrap().total).unwrap();
    assert_eq!(count(&id), 1);
    t.write("Src/sub/b.mov");
    t.mv("Src/a.png", "Src/renamed.png");
    let until = Instant::now() + Duration::from_secs(10);
    while Instant::now() < until && super::with(|c| c.query(&Query { sources: Some(vec![id.clone()]), text: Some("renamed".into()), ..Default::default() }).unwrap().total).unwrap() == 0 {
        std::thread::sleep(Duration::from_millis(100));
    }
    assert_eq!(count(&id), 2, "the new file and the rename were picked up by the watcher");
    // the rename kept the asset (the watcher only signals; the refresh is the ordinary diff)
    let names = super::with(|c| c.query(&Query { sources: Some(vec![id.clone()]), ..Default::default() }).unwrap().entries.into_iter().map(|e| e.rel_path).collect::<Vec<_>>()).unwrap();
    assert!(names.contains(&"renamed.png".to_owned()) && names.contains(&"sub/b.mov".to_owned()), "{names:?}");
    super::watch::stop();
    super::with(|c| c.remove_source(&id).unwrap()).unwrap();
}
