//! The Catalog's tables. One SQLite file (WAL): the sources a person registered, one row per asset with where it is now,
//! where it has been, and what was learned about folders that moved. The files under a source are never written; this file
//! is a rebuildable index (deleting it forgets the index, never a source file).

pub(super) const VERSION: i32 = 1;

pub(super) const DDL: &str = "
CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value INTEGER NOT NULL);
INSERT OR IGNORE INTO meta (key, value) VALUES ('revision', 0);

CREATE TABLE IF NOT EXISTS sources (
  id INTEGER PRIMARY KEY,
  uid TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  root TEXT NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 1,
  available INTEGER NOT NULL DEFAULT 1
);

-- One row per asset identity. `state` 0 = present at (source, rel), 1 = missing (its last place is kept: a person is
-- shown 'missing', a resolver can still use where it was). Identity is `uid`; the path is only its current location.
CREATE TABLE IF NOT EXISTS assets (
  id INTEGER PRIMARY KEY,
  uid TEXT NOT NULL UNIQUE,
  source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
  rel TEXT NOT NULL,
  filename TEXT NOT NULL,
  kind TEXT NOT NULL,
  media_type TEXT NOT NULL,
  size INTEGER NOT NULL,
  mtime_ns INTEGER NOT NULL,
  dev INTEGER NOT NULL DEFAULT 0,
  ino INTEGER NOT NULL DEFAULT 0,
  fp TEXT,
  width INTEGER,
  height INTEGER,
  state INTEGER NOT NULL DEFAULT 0
);
CREATE UNIQUE INDEX IF NOT EXISTS assets_place ON assets(source_id, rel) WHERE state = 0;
CREATE INDEX IF NOT EXISTS assets_fp ON assets(fp) WHERE fp IS NOT NULL;
CREATE INDEX IF NOT EXISTS assets_size ON assets(size);
CREATE INDEX IF NOT EXISTS assets_kind ON assets(kind);

-- Where an asset used to be (each move or rename adds one). A saved reference to an old place finds its asset here.
CREATE TABLE IF NOT EXISTS past_places (
  asset_id INTEGER NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
  source_id INTEGER NOT NULL,
  rel TEXT NOT NULL,
  at_revision INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS past_places_rel ON past_places(source_id, rel);

-- A folder that was seen to move as a whole: everything under old_prefix is now under new_prefix.
CREATE TABLE IF NOT EXISTS folder_moves (
  source_id INTEGER NOT NULL,
  old_prefix TEXT NOT NULL,
  new_prefix TEXT NOT NULL,
  evidence INTEGER NOT NULL,
  at_revision INTEGER NOT NULL,
  PRIMARY KEY (source_id, old_prefix, new_prefix)
);

-- Substring search over names and paths (trigram: any 3+ characters in the middle of a word match).
CREATE VIRTUAL TABLE IF NOT EXISTS assets_fts USING fts5(filename, rel, content='assets', content_rowid='id', tokenize='trigram');
CREATE TRIGGER IF NOT EXISTS assets_ai AFTER INSERT ON assets BEGIN
  INSERT INTO assets_fts(rowid, filename, rel) VALUES (new.id, new.filename, new.rel);
END;
CREATE TRIGGER IF NOT EXISTS assets_ad AFTER DELETE ON assets BEGIN
  INSERT INTO assets_fts(assets_fts, rowid, filename, rel) VALUES ('delete', old.id, old.filename, old.rel);
END;
CREATE TRIGGER IF NOT EXISTS assets_au AFTER UPDATE OF filename, rel ON assets BEGIN
  INSERT INTO assets_fts(assets_fts, rowid, filename, rel) VALUES ('delete', old.id, old.filename, old.rel);
  INSERT INTO assets_fts(rowid, filename, rel) VALUES (new.id, new.filename, new.rel);
END;
";
