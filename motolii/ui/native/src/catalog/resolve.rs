//! The relocation resolver: a saved reference (where a work last found a file, and what the file was) is looked for in
//! the catalog before it is called missing, and is never joined to a different file on a guess.
//!
//!   saved reference
//!     the same place, the same file            → Exact
//!     a place this asset is known to have left → Strong (it moved)
//!     a folder seen to move as a whole         → Strong (its folder moved)
//!     the same content found elsewhere         → Strong (unique) / Ambiguous (several)
//!     the same name and size, content unknown  → Ambiguous (a person chooses)
//!     the source itself cannot be read         → SourceUnavailable (not a pile of missing assets)
//!     nothing                                  → Missing
//!
//! Once a few references of one folder are found at a new place, the folder's move is one fact: the rest of that folder's
//! references are looked at again under it, so a person is not asked file by file.

use std::collections::HashMap;
use std::path::Path;

use rusqlite::types::Value;

use super::matching::{folder_move, rebase};
use super::query::Entry;
use super::Catalog;

/// What a work remembers of a media file.
#[derive(Clone, Debug, Default)]
pub(crate) struct SavedRef {
    pub path: String,
    pub size: Option<u64>,
    /// The document's own fingerprint text (`motolii-source-v1:sha256[-edges]:…`), when it has one.
    pub content_hash: Option<String>,
    pub file_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Resolution {
    Exact(Entry),
    Strong { entry: Entry, why: String },
    Ambiguous(Vec<Entry>),
    SourceUnavailable { source: String, name: String },
    Missing,
}

const FULL_HASH_LIMIT: u64 = 256 << 20;
const CANDIDATE_CAP: usize = 64;

fn text(s: &str) -> Value {
    Value::Text(s.to_owned())
}

impl Catalog {
    /// The source a path is under (the deepest root that contains it) and the path below the root.
    fn source_of(&self, abs: &str) -> Option<(super::Source, String)> {
        let path = Path::new(abs);
        self.sources().into_iter().filter(|s| path.starts_with(&s.root)).max_by_key(|s| s.root.components().count()).map(|s| {
            let rel = path.strip_prefix(&s.root).map(|r| r.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect::<Vec<_>>().join("/")).unwrap_or_default();
            (s, rel)
        })
    }

    fn present_at(&self, source: &str, rel: &str) -> Option<Entry> {
        self.entries_where("s.uid = ? AND a.rel = ? AND a.state = 0 AND s.enabled = 1", &[text(source), text(rel)]).into_iter().next()
    }

    /// Whether `entry` is the file the reference remembers, as far as it can be told without reading a big file whole:
    /// Some(true) yes, Some(false) no (a different file), None it cannot be told (nothing to compare).
    fn same_file(&mut self, entry: &Entry, r: &SavedRef) -> Option<bool> {
        if let Some(size) = r.size {
            if entry.size != size {
                return Some(false);
            }
        }
        let want = r.content_hash.as_deref()?;
        let decoded = crate::doc::store::SourceFingerprintV1::decode_persisted(want, r.size.or(Some(entry.size)));
        let crate::doc::store::SourceFingerprintDecode::V1(fp) = decoded else { return None };
        let path = Path::new(&entry.abs_path);
        match fp.algorithm() {
            crate::doc::store::fingerprint::Algorithm::Sha256Edges => {
                let have = match &entry.fingerprint {
                    Some(h) => h.clone(),
                    None => self.fingerprint_now(entry)?,
                };
                Some(have == want)
            }
            crate::doc::store::fingerprint::Algorithm::Sha256Full => {
                if entry.size > FULL_HASH_LIMIT {
                    return None;
                }
                let file = std::fs::File::open(path).ok()?;
                let got = crate::doc::store::SourceFingerprintV1::from_reader(file).ok()?;
                Some(got.content_hash() == want)
            }
        }
    }

    /// The entry's cheap fingerprint, read now and kept.
    fn fingerprint_now(&mut self, entry: &Entry) -> Option<String> {
        let fp = crate::doc::store::SourceFingerprintV1::from_edges(&entry.abs_path).ok()?.content_hash();
        let _ = self.db.execute("UPDATE assets SET fp = ?2 WHERE uid = ?1", rusqlite::params![entry.id, fp]);
        Some(fp)
    }

    fn resolve_one(&mut self, r: &SavedRef) -> Resolution {
        let located = self.source_of(&r.path);
        let name = r.file_name.clone().or_else(|| Path::new(&r.path).file_name().map(|n| n.to_string_lossy().into_owned())).unwrap_or_default();

        // a source that cannot be read says nothing about its files: they are neither found there nor missing
        let readable = located.as_ref().is_none_or(|(s, _)| s.available);
        if let Some((source, rel)) = located.as_ref().filter(|_| readable) {
            // 1. the same place
            if let Some(e) = self.present_at(&source.id, rel) {
                if self.same_file(&e, r) != Some(false) {
                    return Resolution::Exact(e);
                }
            }
            // 2. a place this asset is known to have left
            let moved = self.entries_where(
                "a.state = 0 AND s.enabled = 1 AND a.id IN (SELECT pp.asset_id FROM past_places pp JOIN sources ps ON ps.id = pp.source_id WHERE ps.uid = ? AND pp.rel = ?)",
                &[text(&source.id), text(rel)],
            );
            let mut hits = Vec::new();
            for e in moved {
                if self.same_file(&e, r) != Some(false) {
                    hits.push(e);
                }
            }
            match hits.len() {
                1 => return Resolution::Strong { entry: hits.remove(0), why: "it moved".into() },
                n if n > 1 => return Resolution::Ambiguous(hits),
                _ => {}
            }
            // 3. a folder that was seen to move as a whole
            for (old, new) in self.folder_moves_of(&source.id) {
                let Some(to) = rebase(rel, &old, &new) else { continue };
                if let Some(e) = self.present_at(&source.id, &to) {
                    if self.same_file(&e, r) == Some(true) || (r.content_hash.is_none() && r.size.is_none_or(|s| s == e.size)) {
                        return Resolution::Strong { entry: e, why: "its folder moved".into() };
                    }
                }
            }
        }
        // 4. the same content, anywhere in the catalog
        if let (Some(hash), size) = (r.content_hash.as_deref(), r.size) {
            let mut cands: Vec<Entry> = Vec::new();
            match crate::doc::store::SourceFingerprintV1::decode_persisted(hash, size) {
                crate::doc::store::SourceFingerprintDecode::V1(fp) if fp.algorithm() == crate::doc::store::fingerprint::Algorithm::Sha256Edges => {
                    cands.extend(self.entries_where("a.state = 0 AND s.enabled = 1 AND a.fp = ?", &[text(hash)]));
                    if cands.is_empty() || cands.len() > 1 {
                        // entries the index has not fingerprinted yet, of the same size: read them now (bounded)
                        let todo: Vec<Entry> = self.entries_where("a.state = 0 AND s.enabled = 1 AND s.available = 1 AND a.fp IS NULL AND a.size = ?", &[Value::Integer(size.unwrap_or(fp.size_bytes()) as i64)]);
                        for e in todo.into_iter().take(CANDIDATE_CAP) {
                            if self.fingerprint_now(&e).as_deref() == Some(hash) {
                                cands.push(Entry { fingerprint: Some(hash.to_owned()), ..e });
                            }
                        }
                    }
                }
                crate::doc::store::SourceFingerprintDecode::V1(fp) => {
                    let same_size = self.entries_where("a.state = 0 AND s.enabled = 1 AND s.available = 1 AND a.size = ?", &[Value::Integer(fp.size_bytes() as i64)]);
                    for e in same_size.into_iter().take(4) {
                        if self.same_file(&e, r) == Some(true) {
                            cands.push(e);
                        }
                    }
                }
                _ => {}
            }
            cands.dedup_by(|a, b| a.id == b.id);
            match cands.len() {
                1 => return Resolution::Strong { entry: cands.remove(0), why: "the same content".into() },
                n if n > 1 => return Resolution::Ambiguous(cands),
                _ => {}
            }
        }
        // 5. the same name and size, content unknown: a candidate for a person, never a link
        if r.content_hash.is_none() && !name.is_empty() {
            let mut cond = String::from("a.state = 0 AND s.enabled = 1 AND a.filename = ?");
            let mut args = vec![text(&name)];
            if let Some(size) = r.size {
                cond.push_str(" AND a.size = ?");
                args.push(Value::Integer(size as i64));
            }
            let cands = self.entries_where(&cond, &args);
            if !cands.is_empty() {
                return Resolution::Ambiguous(cands);
            }
        }
        match located {
            Some((s, _)) if !s.available => Resolution::SourceUnavailable { source: s.id, name: s.name },
            _ => Resolution::Missing,
        }
    }

    fn folder_moves_of(&self, source: &str) -> Vec<(String, String)> {
        let Ok(mut st) = self.db.prepare("SELECT f.old_prefix, f.new_prefix FROM folder_moves f JOIN sources s ON s.id = f.source_id WHERE s.uid = ?1 ORDER BY f.evidence DESC, f.at_revision DESC") else { return Vec::new() };
        let Ok(rows) = st.query_map([source], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))) else { return Vec::new() };
        rows.flatten().collect()
    }

    /// Resolves a batch of references (a work's media). The results are the same as one by one, plus: what one folder's
    /// files show about that folder's move is applied to the rest of the batch.
    pub(crate) fn resolve_many(&mut self, refs: &[SavedRef]) -> Vec<Resolution> {
        let mut out: Vec<Resolution> = refs.iter().map(|r| self.resolve_one(r)).collect();
        // what the found ones say about folders: (old directory, new directory) → how many showed it
        let mut folders: HashMap<(String, String), usize> = HashMap::new();
        for (r, res) in refs.iter().zip(&out) {
            if let Resolution::Strong { entry, .. } = res {
                let old = r.path.trim_start_matches('/');
                let new = entry.abs_path.trim_start_matches('/');
                if let Some(m) = folder_move(old, new) {
                    *folders.entry(m).or_default() += 1;
                }
            }
        }
        if folders.is_empty() {
            return out;
        }
        for i in 0..refs.len() {
            if !matches!(out[i], Resolution::Missing | Resolution::Ambiguous(_) | Resolution::SourceUnavailable { .. }) {
                continue;
            }
            let old = refs[i].path.trim_start_matches('/').to_owned();
            let mut best: Option<(usize, Entry, usize)> = None;
            for ((from, to), n) in &folders {
                let Some(new_rel) = rebase(&old, from, to) else { continue };
                let abs = format!("/{new_rel}");
                let Some((source, rel)) = self.source_of(&abs) else { continue };
                let Some(e) = self.present_at(&source.id, &rel) else { continue };
                let fits = match self.same_file(&e, &refs[i]) {
                    Some(v) => v,
                    None => refs[i].size.is_none_or(|s| s == e.size),
                };
                if fits && best.as_ref().is_none_or(|b| *n > b.2) {
                    best = Some((i, e, *n));
                }
            }
            if let Some((_, entry, n)) = best {
                out[i] = Resolution::Strong { entry, why: format!("its folder moved (seen with {n} other file(s))") };
            }
        }
        out
    }
}
