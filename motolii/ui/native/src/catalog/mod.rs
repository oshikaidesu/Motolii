//! The Media Catalog: the assets of every registered source folder, one index, queried as one library.
//!
//!   registered source roots  →  recursive index  →  catalog  →  query (source / folder / type / search)  →  result set
//!
//! Where (a folder), What (a type), Which (search) and How (a view) are separate axes over one catalog; the same file is
//! the same asset whichever way it is reached. Identity is the asset's `id`; a path is only where it is now, and where it
//! has been is kept, so a saved reference survives a move, a rename and a folder that was carried elsewhere.
//!
//! Sources are read-only: nothing here moves, renames, deletes or writes a source file. The index is a rebuildable file
//! (`catalog.sqlite` in the state folder); removing a source or the index never touches the files under it.

mod index;
mod matching;
mod query;
mod resolve;
mod scan;
mod schema;
#[cfg(test)]
mod bench;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};

pub(crate) use query::{Entry, FolderCount, Query, ResultSet};
pub(crate) use resolve::{Resolution, SavedRef};

/// What a file is, as the importer's own table says (`asset_type_for_extension`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MediaKind {
    Image,
    Video,
    Audio,
    Model,
    Environment,
}

impl MediaKind {
    pub(crate) fn of(media_type: &str) -> Self {
        match media_type {
            t if t.starts_with("video/") => Self::Video,
            t if t.starts_with("audio/") => Self::Audio,
            t if t.starts_with("model/") || t.starts_with("pointcloud") => Self::Model,
            "image/hdr" | "image/exr" => Self::Environment,
            _ => Self::Image,
        }
    }
    pub(crate) fn key(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Model => "model",
            Self::Environment => "environment",
        }
    }
    pub(crate) fn parse(key: &str) -> Option<Self> {
        [Self::Image, Self::Video, Self::Audio, Self::Model, Self::Environment].into_iter().find(|k| k.key() == key)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Source {
    /// Stable across sessions and moves of the root.
    pub id: String,
    pub name: String,
    pub root: PathBuf,
    pub enabled: bool,
    /// The root was readable at the last look. An unavailable source is not a pile of missing assets.
    pub available: bool,
    /// Present assets indexed under it.
    pub assets: u64,
}

pub(crate) struct Catalog {
    db: Connection,
}

pub(super) fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

impl Catalog {
    pub(crate) fn open(path: &Path) -> Result<Self, String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(err)?;
        }
        Self::init(Connection::open(path).map_err(err)?)
    }

    #[cfg(test)]
    pub(crate) fn open_memory() -> Result<Self, String> {
        Self::init(Connection::open_in_memory().map_err(err)?)
    }

    fn init(db: Connection) -> Result<Self, String> {
        db.pragma_update(None, "journal_mode", "WAL").map_err(err)?;
        db.pragma_update(None, "synchronous", "NORMAL").map_err(err)?;
        db.pragma_update(None, "foreign_keys", "ON").map_err(err)?;
        db.execute_batch(schema::DDL).map_err(err)?;
        db.pragma_update(None, "user_version", schema::VERSION).map_err(err)?;
        Ok(Self { db })
    }

    /// Counts every change to what a query can return; a view that holds the number it last drew can tell when to ask again.
    pub(crate) fn revision(&self) -> i64 {
        self.db.query_row("SELECT value FROM meta WHERE key='revision'", [], |r| r.get(0)).unwrap_or(0)
    }

    fn bump(tx: &rusqlite::Transaction<'_>) -> Result<i64, String> {
        tx.execute("UPDATE meta SET value = value + 1 WHERE key='revision'", []).map_err(err)?;
        tx.query_row("SELECT value FROM meta WHERE key='revision'", [], |r| r.get(0)).map_err(err)
    }

    // ---- sources ---------------------------------------------------------------------------------------------------

    pub(crate) fn sources(&self) -> Vec<Source> {
        let mut st = self
            .db
            .prepare("SELECT s.uid, s.name, s.root, s.enabled, s.available, (SELECT COUNT(*) FROM assets a WHERE a.source_id = s.id AND a.state = 0) FROM sources s ORDER BY s.id")
            .expect("sources query");
        st.query_map([], |r| {
            Ok(Source { id: r.get(0)?, name: r.get(1)?, root: PathBuf::from(r.get::<_, String>(2)?), enabled: r.get::<_, i64>(3)? != 0, available: r.get::<_, i64>(4)? != 0, assets: r.get::<_, i64>(5)? as u64 })
        })
        .expect("sources rows")
        .flatten()
        .collect()
    }

    pub(super) fn source_row(&self, uid: &str) -> Result<(i64, String), String> {
        self.db.query_row("SELECT id, root FROM sources WHERE uid = ?1", [uid], |r| Ok((r.get(0)?, r.get(1)?))).optional().map_err(err)?.ok_or_else(|| format!("No source {uid}"))
    }

    /// Registers a folder. A folder inside another source (or one that contains one) is refused: one file would be
    /// indexed twice and become two assets.
    pub(crate) fn add_source(&mut self, root: &Path, name: Option<&str>) -> Result<Source, String> {
        let root = std::fs::canonicalize(root).map_err(|e| format!("{}: {e}", root.display()))?;
        if !root.is_dir() {
            return Err(format!("{} is not a folder", root.display()));
        }
        for s in self.sources() {
            if root.starts_with(&s.root) || s.root.starts_with(&root) {
                return Err(format!("{} overlaps the source {}", root.display(), s.name));
            }
        }
        let name = name.map(str::to_owned).unwrap_or_else(|| root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| root.display().to_string()));
        let tx = self.db.transaction().map_err(err)?;
        tx.execute("INSERT INTO sources (uid, name, root) VALUES (lower(hex(randomblob(8))), ?1, ?2)", params![name, root.to_string_lossy()]).map_err(err)?;
        Self::bump(&tx)?;
        tx.commit().map_err(err)?;
        self.sources().into_iter().last().ok_or_else(|| "source not saved".to_owned())
    }

    /// Forgets a source and its index rows. The files under it are not touched.
    pub(crate) fn remove_source(&mut self, uid: &str) -> Result<(), String> {
        let (id, _) = self.source_row(uid)?;
        let tx = self.db.transaction().map_err(err)?;
        tx.execute("DELETE FROM folder_moves WHERE source_id = ?1", [id]).map_err(err)?;
        tx.execute("DELETE FROM past_places WHERE source_id = ?1", [id]).map_err(err)?;
        tx.execute("DELETE FROM sources WHERE id = ?1", [id]).map_err(err)?;
        Self::bump(&tx)?;
        tx.commit().map_err(err)
    }

    pub(crate) fn set_enabled(&mut self, uid: &str, enabled: bool) -> Result<(), String> {
        let (id, _) = self.source_row(uid)?;
        let tx = self.db.transaction().map_err(err)?;
        tx.execute("UPDATE sources SET enabled = ?2 WHERE id = ?1", params![id, enabled as i64]).map_err(err)?;
        Self::bump(&tx)?;
        tx.commit().map_err(err)
    }

    pub(crate) fn rename_source(&mut self, uid: &str, name: &str) -> Result<(), String> {
        let (id, _) = self.source_row(uid)?;
        let tx = self.db.transaction().map_err(err)?;
        tx.execute("UPDATE sources SET name = ?2 WHERE id = ?1", params![id, name]).map_err(err)?;
        Self::bump(&tx)?;
        tx.commit().map_err(err)
    }

    /// Points a source at a new root (an external drive mounted elsewhere, a renamed folder). The index carries over when
    /// the relative paths still hold: a sample of the indexed files must be found there with the same size, so the wrong
    /// folder is not adopted by accident.
    pub(crate) fn relocate_source(&mut self, uid: &str, new_root: &Path) -> Result<(), String> {
        let (id, _) = self.source_row(uid)?;
        let new_root = std::fs::canonicalize(new_root).map_err(|e| format!("{}: {e}", new_root.display()))?;
        if !new_root.is_dir() {
            return Err(format!("{} is not a folder", new_root.display()));
        }
        for s in self.sources() {
            if s.id != uid && (new_root.starts_with(&s.root) || s.root.starts_with(&new_root)) {
                return Err(format!("{} overlaps the source {}", new_root.display(), s.name));
            }
        }
        let sample: Vec<(String, i64)> = {
            let mut st = self.db.prepare("SELECT rel, size FROM assets WHERE source_id = ?1 AND state = 0 ORDER BY id LIMIT 200").map_err(err)?;
            let rows = st.query_map([id], |r| Ok((r.get(0)?, r.get(1)?))).map_err(err)?;
            rows.flatten().collect()
        };
        if !sample.is_empty() {
            let found = sample.iter().filter(|(rel, size)| std::fs::metadata(scan::join(&new_root.to_string_lossy(), rel)).is_ok_and(|m| m.len() as i64 == *size)).count();
            if found * 10 < sample.len() * 8 {
                return Err(format!("Only {found} of {} indexed files are at {}: not the same folder", sample.len(), new_root.display()));
            }
        }
        let tx = self.db.transaction().map_err(err)?;
        tx.execute("UPDATE sources SET root = ?2, available = 1 WHERE id = ?1", params![id, new_root.to_string_lossy()]).map_err(err)?;
        Self::bump(&tx)?;
        tx.commit().map_err(err)
    }
}
