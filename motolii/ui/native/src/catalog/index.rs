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

impl Catalog {
    /// Refreshes one source (or every enabled one): incremental, read-only towards the source.
    pub(crate) fn refresh(&mut self, only: Option<&str>) -> Result<Vec<RefreshReport>, String> {
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
        targets.into_iter().map(|(id, uid, root)| self.refresh_source(id, uid, root)).collect()
    }

    fn refresh_source(&mut self, id: i64, uid: String, root: String) -> Result<RefreshReport, String> {
        let mut report = RefreshReport { source: uid, ..Default::default() };
        let found = scan(std::path::Path::new(&root));
        report.errors = found.errors;
        let mut rows: HashMap<String, Row> = {
            let mut st = self.db.prepare("SELECT rel, id, size, mtime_ns, dev, ino, fp FROM assets WHERE source_id = ?1 AND state = 0").map_err(err)?;
            let it = st
                .query_map([id], |r| Ok((r.get::<_, String>(0)?, Row { id: r.get(1)?, size: r.get::<_, i64>(2)? as u64, mtime_ns: r.get(3)?, dev: r.get::<_, i64>(4)? as u64, ino: r.get::<_, i64>(5)? as u64, fp: r.get(6)? })))
                .map_err(err)?;
            it.flatten().collect()
        };
        // An unreadable root, or a readable one that shows nothing where thousands were indexed (an unmounted volume's
        // empty mount point), is not evidence that everything was deleted.
        if !found.root_readable || (found.entries.is_empty() && !rows.is_empty()) {
            self.db.execute("UPDATE sources SET available = 0 WHERE id = ?1", [id]).map_err(err)?;
            report.unavailable = true;
            return Ok(report);
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
        // 4. new files
        {
            let mut ins = tx
                .prepare_cached("INSERT INTO assets (uid, source_id, rel, filename, kind, media_type, size, mtime_ns, dev, ino) VALUES (lower(hex(randomblob(8))), ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)")
                .map_err(err)?;
            for (fi, f) in added.iter().enumerate() {
                if linked_fresh[fi] {
                    continue;
                }
                ins.execute(params![id, f.rel, f.filename, MediaKind::of(&f.media_type).key(), f.media_type, f.size as i64, f.mtime_ns, f.dev as i64, f.ino as i64]).map_err(err)?;
                report.added += 1;
            }
        }
        tx.execute("UPDATE sources SET available = 1 WHERE id = ?1", [id]).map_err(err)?;
        if report.added + report.moved + report.missing + report.changed > 0 || !refreshed_ids.is_empty() {
            Self::bump(&tx)?;
        }
        tx.commit().map_err(err)?;
        Ok(report)
    }

    /// Learns the slow things about entries, a bounded batch at a time, off the scan's path: the cheap fingerprint
    /// (what lets a copy or a move across volumes be recognised later) and an image's dimensions. Returns how many were
    /// filled; call again until 0. A file that changed since it was indexed is skipped (the next refresh sees it).
    pub(crate) fn enrich(&mut self, limit: usize) -> usize {
        let todo: Vec<(i64, String, String, String, u64)> = {
            let Ok(mut st) = self.db.prepare(
                "SELECT a.id, s.root, a.rel, a.kind, a.size FROM assets a JOIN sources s ON s.id = a.source_id
                 WHERE a.state = 0 AND s.enabled = 1 AND s.available = 1 AND a.fp IS NULL LIMIT ?1",
            ) else {
                return 0;
            };
            let Ok(rows) = st.query_map([limit as i64], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get::<_, i64>(4)? as u64))) else { return 0 };
            rows.flatten().collect()
        };
        let mut done = 0;
        for (id, root, rel, kind, size) in todo {
            let path = join(&root, &rel);
            if std::fs::metadata(&path).ok().map(|m| m.len()) != Some(size) {
                continue;
            }
            let Some(fp) = cheap_fp(&path) else { continue };
            let dims = matches!(MediaKind::parse(&kind), Some(MediaKind::Image | MediaKind::Environment)).then(|| image::ImageReader::open(&path).ok().and_then(|r| r.with_guessed_format().ok()).and_then(|r| r.into_dimensions().ok())).flatten();
            let (w, h) = dims.map(|(w, h)| (Some(w as i64), Some(h as i64))).unwrap_or((None, None));
            if self.db.execute("UPDATE assets SET fp = ?2, width = ?3, height = ?4 WHERE id = ?1", params![id, fp, w, h]).is_ok() {
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
