//! Indexing: the first recursive scan, and every refresh after it (a diff against what the catalog already knows, so only
//! the entries that changed are written). A file that moved keeps its asset; one that vanished is kept as missing (its last
//! place stays for a resolver) and never purged; a source that cannot be read at all is 'unavailable', not a pile of
//! missing assets.

use std::collections::HashMap;

use rusqlite::{params, OptionalExtension};

use super::matching::{folder_move, pair, Gone};
use super::scan::{join, scan, FsEntry};
use super::{err, Catalog, MediaKind};

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct RefreshReport {
    pub source: String,
    pub scanned: usize,
    pub added: usize,
    pub changed: usize,
    pub moved: usize,
    pub missing: usize,
    /// Pairs left unlinked because several fit equally.
    pub ambiguous: usize,
    pub errors: Vec<String>,
    /// The root could not be read (or was suspiciously empty): nothing was changed.
    pub unavailable: bool,
    pub folder_moves: Vec<(String, String)>,
}

struct Row {
    id: i64,
    size: u64,
    mtime_ns: i64,
    dev: u64,
    ino: u64,
    fp: Option<String>,
}

fn cheap_fp(path: &std::path::Path) -> Option<String> {
    crate::doc::store::SourceFingerprintV1::from_edges(path).ok().map(|f| f.content_hash())
}

/// What one source looked like in the catalog when a refresh began (read under the lock, quickly).
pub(crate) struct RefreshInput {
    id: i64,
    uid: String,
    root: String,
    rows: HashMap<String, Row>,
    revision: i64,
}

/// A refresh worked out away from the catalog lock: the filesystem walk and every file read happen here, and what is left
/// for the lock is one short write of the difference.
pub(crate) enum Prepared {
    Unavailable { id: i64, report: RefreshReport },
    Plan(Box<Plan>),
}

pub(crate) struct Plan {
    id: i64,
    revision: i64,
    report: RefreshReport,
    retouched: Vec<(i64, FsEntry, bool)>,
    refreshed_ids: Vec<(i64, u64, u64)>,
    added: Vec<FsEntry>,
    gone: Vec<Gone>,
    paired: super::matching::Pairing,
}

/// The walk and the classification: no catalog, no lock; only the filesystem is read (never written).
pub(crate) fn prepare(input: RefreshInput) -> Prepared {
    let RefreshInput { id, uid, root, mut rows, revision } = input;
    let mut report = RefreshReport { source: uid, ..Default::default() };
    let found = scan(std::path::Path::new(&root));
    report.errors = found.errors;
    // An unreadable root, or a readable one that shows nothing where thousands were indexed (an unmounted volume's
    // empty mount point), is not evidence that everything was deleted.
    if !found.root_readable || (found.entries.is_empty() && !rows.is_empty()) {
        report.unavailable = true;
        return Prepared::Unavailable { id, report };
    }
    report.scanned = found.entries.len();

    let mut retouched: Vec<(i64, FsEntry, bool)> = Vec::new(); // (row, new state, content dropped)
    let mut refreshed_ids: Vec<(i64, u64, u64)> = Vec::new(); // same file, new platform id
    let mut added: Vec<FsEntry> = Vec::new();
    let mut gone: Vec<Gone> = Vec::new();
    for e in found.entries {
        match rows.remove(&e.rel) {
            None => added.push(e),
            Some(row) if row.size == e.size && row.mtime_ns == e.mtime_ns => {
                if (row.dev, row.ino) != (e.dev, e.ino) {
                    refreshed_ids.push((row.id, e.dev, e.ino));
                }
            }
            Some(row) => {
                let same_file = (row.dev, row.ino) == (e.dev, e.ino);
                if same_file {
                    // edited in place: still the same asset, what was learned about its content is stale
                    retouched.push((row.id, e, true));
                } else if row.fp.is_some() && row.fp == cheap_fp(&join(&root, &e.rel)) {
                    // replaced by an identical copy (a save that swaps the file): the same asset
                    retouched.push((row.id, e, false));
                } else {
                    // another file now stands at this path: the old asset lost its place, the new one is new
                    gone.push(Gone { id: row.id, rel: e.rel.clone(), size: row.size, mtime_ns: row.mtime_ns, dev: row.dev, ino: row.ino, fp: row.fp });
                    added.push(e);
                }
            }
        }
    }
    for (rel, row) in rows {
        gone.push(Gone { id: row.id, rel, size: row.size, mtime_ns: row.mtime_ns, dev: row.dev, ino: row.ino, fp: row.fp });
    }
    gone.sort_by(|a, b| a.rel.cmp(&b.rel));
    let paired = pair(&gone, &added, &mut |f| cheap_fp(&join(&root, &f.rel)));
    report.ambiguous = paired.ambiguous;
    Prepared::Plan(Box::new(Plan { id, revision, report, retouched, refreshed_ids, added, gone, paired }))
}

/// Refreshes one source (or every enabled one) with the lock held only for the reads before and the write after.
/// A write that finds the catalog changed under it (another refresh, a source edit) starts that source over.
pub(crate) fn refresh_unlocked(only: Option<&str>) -> Result<Vec<RefreshReport>, String> {
    // one refresh at a time (the watcher and an asked one do not interleave their batches); this is not the catalog lock
    static ONE_AT_A_TIME: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let uids = super::bridge::with(|c| c.refresh_uids(only))??;
    let mut reports = Vec::new();
    for uid in uids {
        for _ in 0..4 {
            // each attempt reads the source afresh, so a write by someone else in between only costs a second walk
            let Some(input) = super::bridge::with(|c| c.refresh_inputs(Some(&uid)))?.ok().and_then(|mut v| v.pop()) else { break };
            let prepared = prepare(input);
            if let Some((mut r, id, fresh)) = super::bridge::with(|c| c.apply_head(prepared))?? {
                for batch in fresh.chunks(400) {
                    r.added += super::bridge::with(|c| c.insert_new(id, batch))??;
                    // a std Mutex is not fair: without a breath the next batch takes the lock again before a waiting query can
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
                reports.push(r);
                break;
            }
        }
    }
    Ok(reports)
}

/// Reads the files of one batch (fingerprints, image dimensions): no catalog, no lock.
pub(crate) fn enrich_read(todo: Vec<(i64, String, String, String, u64)>) -> Vec<(i64, u64, String, Option<i64>, Option<i64>)> {
    let mut out = Vec::new();
    for (id, root, rel, kind, size) in todo {
        let path = join(&root, &rel);
        if std::fs::metadata(&path).ok().map(|m| m.len()) != Some(size) {
            continue;
        }
        let Some(fp) = cheap_fp(&path) else { continue };
        let dims = matches!(MediaKind::parse(&kind), Some(MediaKind::Image | MediaKind::Environment)).then(|| image::ImageReader::open(&path).ok().and_then(|r| r.with_guessed_format().ok()).and_then(|r| r.into_dimensions().ok())).flatten();
        let (w, h) = dims.map(|(w, h)| (Some(w as i64), Some(h as i64))).unwrap_or((None, None));
        out.push((id, size, fp, w, h));
    }
    out
}

/// One enrich batch with the lock held only to pick the batch and to store the result.
pub(crate) fn enrich_unlocked(limit: usize) -> Result<usize, String> {
    let todo = super::bridge::with(|c| c.enrich_todo(limit))?;
    let learned = enrich_read(todo);
    super::bridge::with(|c| c.enrich_write(learned))
}

impl Catalog {
    /// Refreshes one source (or every enabled one): incremental, read-only towards the source.
    pub(crate) fn refresh(&mut self, only: Option<&str>) -> Result<Vec<RefreshReport>, String> {
        let mut out = Vec::new();
        for uid in self.refresh_uids(only)? {
            for i in self.refresh_inputs(Some(&uid))? {
                out.push(self.apply(prepare(i))?.ok_or_else(|| "catalog changed during refresh".to_owned())?);
            }
        }
        Ok(out)
    }

    /// The enabled sources a refresh would visit.
    pub(crate) fn refresh_uids(&self, only: Option<&str>) -> Result<Vec<String>, String> {
        Ok(self.refresh_inputs_meta(only)?.into_iter().map(|(_, uid, _)| uid).collect())
    }

    /// What the enabled sources hold now, as far as the catalog knows (a quick read; no file is touched).
    fn refresh_inputs_meta(&self, only: Option<&str>) -> Result<Vec<(i64, String, String)>, String> {
        let targets: Vec<(i64, String, String)> = {
            let mut st = self.db.prepare("SELECT id, uid, root FROM sources WHERE enabled = 1 ORDER BY id").map_err(err)?;
            let rows = st.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).map_err(err)?;
            rows.flatten().filter(|(_, uid, _)| only.is_none_or(|o| o == uid)).collect()
        };
        if let Some(o) = only {
            if targets.is_empty() {
                return Err(format!("No enabled source {o}"));
            }
        }
        Ok(targets)
    }

    pub(crate) fn refresh_inputs(&self, only: Option<&str>) -> Result<Vec<RefreshInput>, String> {
        let targets = self.refresh_inputs_meta(only)?;
        let revision = self.revision();
        let mut out = Vec::new();
        for (id, uid, root) in targets {
            let mut st = self.db.prepare("SELECT rel, id, size, mtime_ns, dev, ino, fp FROM assets WHERE source_id = ?1 AND state = 0").map_err(err)?;
            let it = st
                .query_map([id], |r| Ok((r.get::<_, String>(0)?, Row { id: r.get(1)?, size: r.get::<_, i64>(2)? as u64, mtime_ns: r.get(3)?, dev: r.get::<_, i64>(4)? as u64, ino: r.get::<_, i64>(5)? as u64, fp: r.get(6)? })))
                .map_err(err)?;
            out.push(RefreshInput { id, uid, root, rows: it.flatten().collect(), revision });
        }
        Ok(out)
    }

    /// Writes what [`prepare`] found. `None`: the catalog changed since the inputs were read, so nothing was written.
    pub(crate) fn apply_head(&mut self, prepared: Prepared) -> Result<Option<(RefreshReport, i64, Vec<FsEntry>)>, String> {
        let Plan { id, revision, mut report, retouched, refreshed_ids, added, gone, paired } = match prepared {
            Prepared::Unavailable { id, report } => {
                self.db.execute("UPDATE sources SET available = 0 WHERE id = ?1", [id]).map_err(err)?;
                return Ok(Some((report, id, Vec::new())));
            }
            Prepared::Plan(p) => *p,
        };
        if self.revision() != revision {
            return Ok(None);
        }
        let mut linked_gone = vec![false; gone.len()];
        let mut linked_fresh = vec![false; added.len()];
        for (gi, fi, _) in &paired.linked {
            linked_gone[*gi] = true;
            linked_fresh[*fi] = true;
        }

        let tx = self.db.transaction().map_err(err)?;
        // 1. the ones that lost their place first (the place may be taken again below)
        for (gi, g) in gone.iter().enumerate() {
            if !linked_gone[gi] {
                tx.execute("UPDATE assets SET state = 1 WHERE id = ?1", [g.id]).map_err(err)?;
                report.missing += 1;
            }
        }
        // 2. moves and renames: same asset, new place, the old place remembered
        let revision_next: i64 = tx.query_row("SELECT value + 1 FROM meta WHERE key='revision'", [], |r| r.get(0)).map_err(err)?;
        let mut moves: HashMap<(String, String), i64> = HashMap::new();
        for (gi, fi, _evidence) in &paired.linked {
            let (g, f) = (&gone[*gi], &added[*fi]);
            tx.execute("INSERT INTO past_places (asset_id, source_id, rel, at_revision) VALUES (?1, ?2, ?3, ?4)", params![g.id, id, g.rel, revision_next]).map_err(err)?;
            // the gone row may still hold its old place under the unique index while it moves: free it first
            tx.execute("UPDATE assets SET state = 1 WHERE id = ?1", [g.id]).map_err(err)?;
            tx.execute(
                "UPDATE assets SET rel = ?2, filename = ?3, kind = ?4, media_type = ?5, size = ?6, mtime_ns = ?7, dev = ?8, ino = ?9, state = 0 WHERE id = ?1",
                params![g.id, f.rel, f.filename, MediaKind::of(&f.media_type).key(), f.media_type, f.size as i64, f.mtime_ns, f.dev as i64, f.ino as i64],
            )
            .map_err(err)?;
            if let Some(m) = folder_move(&g.rel, &f.rel) {
                *moves.entry(m).or_default() += 1;
            }
            report.moved += 1;
        }
        for ((old, new), n) in &moves {
            tx.execute(
                "INSERT INTO folder_moves (source_id, old_prefix, new_prefix, evidence, at_revision) VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(source_id, old_prefix, new_prefix) DO UPDATE SET evidence = evidence + excluded.evidence, at_revision = excluded.at_revision",
                params![id, old, new, n, revision_next],
            )
            .map_err(err)?;
            report.folder_moves.push((old.clone(), new.clone()));
        }
        // 3. edits in place
        for (row, e, drop_content) in &retouched {
            if *drop_content {
                tx.execute("UPDATE assets SET size = ?2, mtime_ns = ?3, dev = ?4, ino = ?5, fp = NULL, width = NULL, height = NULL WHERE id = ?1", params![row, e.size as i64, e.mtime_ns, e.dev as i64, e.ino as i64]).map_err(err)?;
            } else {
                tx.execute("UPDATE assets SET size = ?2, mtime_ns = ?3, dev = ?4, ino = ?5 WHERE id = ?1", params![row, e.size as i64, e.mtime_ns, e.dev as i64, e.ino as i64]).map_err(err)?;
            }
            report.changed += 1;
        }
        for (row, dev, ino) in &refreshed_ids {
            tx.execute("UPDATE assets SET dev = ?2, ino = ?3 WHERE id = ?1", params![row, *dev as i64, *ino as i64]).map_err(err)?;
        }
        tx.execute("UPDATE sources SET available = 1 WHERE id = ?1", [id]).map_err(err)?;
        if report.moved + report.missing + report.changed > 0 || !refreshed_ids.is_empty() {
            Self::bump(&tx)?;
        }
        tx.commit().map_err(err)?;
        // 4. new files are written by the caller, in batches (a first index of thousands is not one long hold of the lock)
        let fresh: Vec<FsEntry> = added.into_iter().enumerate().filter(|(fi, _)| !linked_fresh[*fi]).map(|(_, f)| f).collect();
        Ok(Some((report, id, fresh)))
    }

    /// Writes one batch of new files (one short transaction).
    pub(crate) fn insert_new(&mut self, id: i64, fresh: &[FsEntry]) -> Result<usize, String> {
        let tx = self.db.transaction().map_err(err)?;
        {
            let mut ins = tx
                .prepare_cached("INSERT INTO assets (uid, source_id, rel, filename, kind, media_type, size, mtime_ns, dev, ino) VALUES (lower(hex(randomblob(8))), ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)")
                .map_err(err)?;
            for f in fresh {
                ins.execute(params![id, f.rel, f.filename, MediaKind::of(&f.media_type).key(), f.media_type, f.size as i64, f.mtime_ns, f.dev as i64, f.ino as i64]).map_err(err)?;
            }
        }
        if !fresh.is_empty() {
            Self::bump(&tx)?;
        }
        tx.commit().map_err(err)?;
        Ok(fresh.len())
    }

    /// Everything of [`apply_head`] and then every new file.
    pub(crate) fn apply(&mut self, prepared: Prepared) -> Result<Option<RefreshReport>, String> {
        let Some((mut report, id, fresh)) = self.apply_head(prepared)? else { return Ok(None) };
        report.added = self.insert_new(id, &fresh)?;
        Ok(Some(report))
    }

    /// Learns the slow things about entries, a bounded batch at a time, off the scan's path: the cheap fingerprint
    /// (what lets a copy or a move across volumes be recognised later) and an image's dimensions. Returns how many were
    /// filled; call again until 0. A file that changed since it was indexed is skipped (the next refresh sees it).
    pub(crate) fn enrich(&mut self, limit: usize) -> usize {
        let learned = enrich_read(self.enrich_todo(limit));
        self.enrich_write(learned)
    }

    /// The next batch of entries with nothing learned yet (a quick read; no file is touched).
    pub(crate) fn enrich_todo(&self, limit: usize) -> Vec<(i64, String, String, String, u64)> {
        let Ok(mut st) = self.db.prepare(
            "SELECT a.id, s.root, a.rel, a.kind, a.size FROM assets a JOIN sources s ON s.id = a.source_id
             WHERE a.state = 0 AND s.enabled = 1 AND s.available = 1 AND a.fp IS NULL LIMIT ?1",
        ) else {
            return Vec::new();
        };
        let Ok(rows) = st.query_map([limit as i64], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get::<_, i64>(4)? as u64))) else { return Vec::new() };
        rows.flatten().collect()
    }

    /// Stores what [`enrich_read`] learned; an entry whose size changed meanwhile keeps waiting.
    pub(crate) fn enrich_write(&mut self, learned: Vec<(i64, u64, String, Option<i64>, Option<i64>)>) -> usize {
        let mut done = 0;
        for (id, size, fp, w, h) in learned {
            if self.db.execute("UPDATE assets SET fp = ?2, width = ?3, height = ?4 WHERE id = ?1 AND size = ?5 AND fp IS NULL", params![id, fp, w, h, size as i64]).is_ok_and(|n| n > 0) {
                done += 1;
            }
        }
        done
    }

    /// Entries still waiting for [`enrich`].
    pub(crate) fn pending_enrich(&self) -> u64 {
        self.db.query_row("SELECT COUNT(*) FROM assets a JOIN sources s ON s.id = a.source_id WHERE a.state = 0 AND s.enabled = 1 AND s.available = 1 AND a.fp IS NULL", [], |r| r.get::<_, i64>(0)).optional().ok().flatten().unwrap_or(0) as u64
    }

    /// Forgets the assets that are missing (a person's decision; nothing calls it by itself). Source files are not touched.
    pub(crate) fn forget_missing(&mut self, source: &str) -> Result<usize, String> {
        let (id, _) = self.source_row(source)?;
        let tx = self.db.transaction().map_err(err)?;
        let n = tx.execute("DELETE FROM assets WHERE source_id = ?1 AND state = 1", [id]).map_err(err)?;
        if n > 0 {
            Self::bump(&tx)?;
        }
        tx.commit().map_err(err)?;
        Ok(n)
    }
}
