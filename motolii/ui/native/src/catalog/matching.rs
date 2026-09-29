//! Which vanished entries are which new ones (a move or a rename), decided from the strongest evidence down, and never
//! guessed: an ambiguous pair stays unlinked (a wrong link is worse than a missing file).
//!
//! Evidence, strongest first:
//!   1. the platform's file identity (device, inode) with the same size and modified time: a rename or move on one volume;
//!   2. an equal cheap fingerprint (size + the first and last megabyte), one vanished entry to one new one.
//! Nothing weaker links by itself: same name and size alone is only a candidate, shown to a person.

use std::collections::HashMap;

use super::scan::FsEntry;

/// A catalog row whose file is no longer where it was.
#[derive(Clone, Debug)]
pub(crate) struct Gone {
    pub id: i64,
    pub rel: String,
    pub size: u64,
    pub mtime_ns: i64,
    pub dev: u64,
    pub ino: u64,
    pub fp: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Evidence {
    PlatformId,
    Fingerprint,
}

pub(crate) struct Pairing {
    /// (index into gone, index into fresh, why)
    pub linked: Vec<(usize, usize, Evidence)>,
    /// Several vanished or several new entries fit one another equally: left unlinked.
    pub ambiguous: usize,
}

/// `fresh_fp` computes a new entry's cheap fingerprint on demand (it reads the file); it is asked only for the size
/// matches of a vanished entry that has a fingerprint, so a big scan does not read every new file.
pub(crate) fn pair(gone: &[Gone], fresh: &[FsEntry], fresh_fp: &mut dyn FnMut(&FsEntry) -> Option<String>) -> Pairing {
    let mut linked = Vec::new();
    let mut ambiguous = 0;
    let mut taken_gone = vec![false; gone.len()];
    let mut taken_fresh = vec![false; fresh.len()];

    // 1. platform identity
    let mut by_id: HashMap<(u64, u64), Vec<usize>> = HashMap::new();
    for (i, f) in fresh.iter().enumerate() {
        if f.ino != 0 {
            by_id.entry((f.dev, f.ino)).or_default().push(i);
        }
    }
    let mut claims: HashMap<(u64, u64), usize> = HashMap::new();
    for g in gone {
        if g.ino != 0 {
            *claims.entry((g.dev, g.ino)).or_default() += 1;
        }
    }
    for (gi, g) in gone.iter().enumerate() {
        if g.ino == 0 {
            continue;
        }
        let Some(cands) = by_id.get(&(g.dev, g.ino)) else { continue };
        // a hard link (two vanished rows, or two new files, with one identity) proves nothing
        if cands.len() != 1 || claims.get(&(g.dev, g.ino)) != Some(&1) {
            ambiguous += 1;
            continue;
        }
        let fi = cands[0];
        let f = &fresh[fi];
        // an inode can be reused by a different file: the same size and modified time must hold too
        if f.size == g.size && f.mtime_ns == g.mtime_ns && !taken_fresh[fi] {
            linked.push((gi, fi, Evidence::PlatformId));
            taken_gone[gi] = true;
            taken_fresh[fi] = true;
        }
    }

    // 2. cheap fingerprint (one to one)
    let mut fresh_fps: HashMap<usize, Option<String>> = HashMap::new();
    let mut gone_by_fp: HashMap<&str, Vec<usize>> = HashMap::new();
    for (gi, g) in gone.iter().enumerate() {
        if !taken_gone[gi] {
            if let Some(fp) = g.fp.as_deref() {
                gone_by_fp.entry(fp).or_default().push(gi);
            }
        }
    }
    for (gi, g) in gone.iter().enumerate() {
        if taken_gone[gi] {
            continue;
        }
        let Some(fp) = g.fp.as_deref() else { continue };
        let mut matches = Vec::new();
        for (fi, f) in fresh.iter().enumerate() {
            if taken_fresh[fi] || f.size != g.size {
                continue;
            }
            let got = fresh_fps.entry(fi).or_insert_with(|| fresh_fp(f));
            if got.as_deref() == Some(fp) {
                matches.push(fi);
            }
        }
        if matches.len() == 1 && gone_by_fp.get(fp).map(Vec::len) == Some(1) {
            linked.push((gi, matches[0], Evidence::Fingerprint));
            taken_gone[gi] = true;
            taken_fresh[matches[0]] = true;
        } else if !matches.is_empty() {
            ambiguous += 1;
        }
    }
    Pairing { linked, ambiguous }
}

/// The folder that moved, read off one pair: the two directories once their identical tails are cut away
/// (`a/x/f.png` → `b/x/f.png` moved the folder `a` to `b`). None when the file stayed in its folder.
pub(crate) fn folder_move(old_rel: &str, new_rel: &str) -> Option<(String, String)> {
    let old: Vec<&str> = old_rel.split('/').collect();
    let new: Vec<&str> = new_rel.split('/').collect();
    let (old_dir, new_dir) = (&old[..old.len() - 1], &new[..new.len() - 1]);
    if old_dir == new_dir {
        return None;
    }
    let mut tail = 0;
    while tail < old_dir.len() && tail < new_dir.len() && old_dir[old_dir.len() - 1 - tail] == new_dir[new_dir.len() - 1 - tail] {
        tail += 1;
    }
    Some((old_dir[..old_dir.len() - tail].join("/"), new_dir[..new_dir.len() - tail].join("/")))
}

/// `rel` re-rooted from `old_prefix` to `new_prefix` (empty prefixes are the source root), or None when it is not under it.
pub(crate) fn rebase(rel: &str, old_prefix: &str, new_prefix: &str) -> Option<String> {
    let rest = if old_prefix.is_empty() { rel } else { rel.strip_prefix(old_prefix)?.strip_prefix('/')? };
    Some(if new_prefix.is_empty() { rest.to_owned() } else { format!("{new_prefix}/{rest}") })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh(rel: &str, size: u64, mtime: i64, ino: u64) -> FsEntry {
        FsEntry { rel: rel.into(), filename: rel.rsplit('/').next().unwrap().into(), size, mtime_ns: mtime, dev: 1, ino, media_type: "image/png".into() }
    }
    fn gone(id: i64, rel: &str, size: u64, mtime: i64, ino: u64, fp: Option<&str>) -> Gone {
        Gone { id, rel: rel.into(), size, mtime_ns: mtime, dev: 1, ino, fp: fp.map(Into::into) }
    }

    #[test]
    fn a_rename_is_found_by_the_platform_id() {
        let g = [gone(1, "a/x.png", 10, 5, 77, None)];
        let f = [fresh("a/y.png", 10, 5, 77)];
        let p = pair(&g, &f, &mut |_| None);
        assert_eq!(p.linked, vec![(0, 0, Evidence::PlatformId)]);
    }

    #[test]
    fn a_reused_inode_with_other_content_is_not_a_move() {
        let g = [gone(1, "a/x.png", 10, 5, 77, None)];
        let f = [fresh("b/z.png", 10, 6, 77)];
        assert!(pair(&g, &f, &mut |_| None).linked.is_empty());
        let f = [fresh("b/z.png", 11, 5, 77)];
        assert!(pair(&g, &f, &mut |_| None).linked.is_empty());
    }

    #[test]
    fn a_hard_link_proves_nothing() {
        let g = [gone(1, "a/x.png", 10, 5, 77, None)];
        let f = [fresh("b/one.png", 10, 5, 77), fresh("b/two.png", 10, 5, 77)];
        let p = pair(&g, &f, &mut |_| None);
        assert!(p.linked.is_empty());
        assert_eq!(p.ambiguous, 1);
    }

    #[test]
    fn a_copy_to_another_volume_is_found_by_fingerprint_only_when_unique() {
        let g = [gone(1, "a/x.png", 10, 5, 0, Some("fp1"))];
        let f = [fresh("other/x.png", 10, 9, 0)];
        let p = pair(&g, &f, &mut |_| Some("fp1".into()));
        assert_eq!(p.linked, vec![(0, 0, Evidence::Fingerprint)]);
        // two new files with the same content: ambiguous, nothing linked
        let f = [fresh("o/x.png", 10, 9, 0), fresh("o/copy.png", 10, 9, 0)];
        let p = pair(&g, &f, &mut |_| Some("fp1".into()));
        assert!(p.linked.is_empty());
        assert_eq!(p.ambiguous, 1);
    }

    #[test]
    fn the_same_name_and_size_alone_links_nothing() {
        let g = [gone(1, "a/x.png", 10, 5, 0, None)];
        let f = [fresh("b/x.png", 10, 5, 0)];
        assert!(pair(&g, &f, &mut |_| None).linked.is_empty());
    }

    #[test]
    fn a_folder_move_is_read_off_a_pair() {
        assert_eq!(folder_move("A/models/x.glb", "Archive/A/models/x.glb"), Some(("".into(), "Archive".into())));
        assert_eq!(folder_move("A/x.png", "B/x.png"), Some(("A".into(), "B".into())));
        assert_eq!(folder_move("A/x.png", "A/y.png"), None);
        assert_eq!(folder_move("x.png", "sub/x.png"), Some(("".into(), "sub".into())));
    }

    #[test]
    fn a_path_is_rebased_under_a_folder_move() {
        assert_eq!(rebase("A/f/x.png", "A", "Archive/A").as_deref(), Some("Archive/A/f/x.png"));
        assert_eq!(rebase("A/x.png", "", "Archive").as_deref(), Some("Archive/A/x.png"));
        assert_eq!(rebase("B/x.png", "A", "Z"), None);
        assert_eq!(rebase("AA/x.png", "A", "Z"), None);
    }
}
