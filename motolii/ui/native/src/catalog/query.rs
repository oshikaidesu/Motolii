//! Queries over the catalog: the catalog owner builds the result set, the view never receives (or filters) every asset.
//! A result set is presentation-neutral: List, Thumbnail and Explore all read the same one.

use rusqlite::types::Value;
use rusqlite::params_from_iter;

use super::{err, scan::join, Catalog, MediaKind};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Sort {
    #[default]
    Name,
    Kind,
    Size,
    Modified,
}

impl Sort {
    pub(crate) fn parse(key: &str) -> Self {
        match key {
            "kind" => Self::Kind,
            "size" => Self::Size,
            "modified" => Self::Modified,
            _ => Self::Name,
        }
    }
    fn order(self, descending: bool) -> String {
        let dir = if descending { "DESC" } else { "ASC" };
        match self {
            Self::Name => format!("a.filename COLLATE NOCASE {dir}, a.id"),
            Self::Kind => format!("a.kind {dir}, a.filename COLLATE NOCASE, a.id"),
            Self::Size => format!("a.size {dir}, a.filename COLLATE NOCASE, a.id"),
            Self::Modified => format!("a.mtime_ns {dir}, a.filename COLLATE NOCASE, a.id"),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Query {
    /// Which sources (ids); None = every enabled source.
    pub sources: Option<Vec<String>>,
    /// A folder inside one source: (source id, path under its root; "" = the whole source).
    pub folder: Option<(String, String)>,
    /// With a folder: only what stands directly in it (folder browsing), not everything below (the library view).
    pub direct: bool,
    /// Which kinds; None = all.
    pub kinds: Option<Vec<MediaKind>>,
    /// Words that must all appear in a name or path (any three characters or more, in the middle of a word too).
    pub text: Option<String>,
    /// Include the assets whose file is missing (a resolver's or a repair view's question; a library view says no).
    pub include_missing: bool,
    /// The order the owner returns them in (a list view's column headers ask; the view never re-sorts).
    pub sort: Sort,
    pub descending: bool,
    pub offset: usize,
    /// 0 = a page of 500.
    pub limit: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Entry {
    pub id: String,
    pub source: String,
    pub source_name: String,
    pub abs_path: String,
    pub rel_path: String,
    pub filename: String,
    pub kind: MediaKind,
    pub media_type: String,
    pub size: u64,
    pub mtime_ns: i64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fingerprint: Option<String>,
    /// Changes when the file does: a face (thumbnail, waveform) cached under it is still the file's own.
    pub face_key: String,
    pub missing: bool,
    /// The source's root was readable at the last look; false says "source unavailable", never "asset missing".
    pub source_available: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResultSet {
    /// The catalog's revision when this was made; the same number means the same answer.
    pub revision: i64,
    pub total: usize,
    pub offset: usize,
    pub entries: Vec<Entry>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FolderCount {
    pub name: String,
    /// Present assets in it and below.
    pub assets: u64,
}

pub(super) const SELECT_ENTRY: &str = "SELECT a.uid, s.uid, s.name, s.root, a.rel, a.filename, a.kind, a.media_type, a.size, a.mtime_ns, a.width, a.height, a.fp, a.state, s.available
                 FROM assets a JOIN sources s ON s.id = a.source_id";

pub(super) fn entry_from_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Entry> {
    let (uid, root, rel, size, mtime): (String, String, String, i64, i64) = (r.get(0)?, r.get(3)?, r.get(4)?, r.get(8)?, r.get(9)?);
    Ok(Entry {
        face_key: format!("{uid}:{size}:{mtime}"),
        abs_path: join(&root, &rel).to_string_lossy().into_owned(),
        id: uid,
        source: r.get(1)?,
        source_name: r.get(2)?,
        rel_path: rel,
        filename: r.get(5)?,
        kind: MediaKind::parse(&r.get::<_, String>(6)?).unwrap_or(MediaKind::Image),
        media_type: r.get(7)?,
        size: size as u64,
        mtime_ns: mtime,
        width: r.get::<_, Option<i64>>(10)?.map(|v| v as u32),
        height: r.get::<_, Option<i64>>(11)?.map(|v| v as u32),
        fingerprint: r.get(12)?,
        missing: r.get::<_, i64>(13)? != 0,
        source_available: r.get::<_, i64>(14)? != 0,
    })
}

fn like_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
}

impl Query {
    fn conditions(&self) -> (String, Vec<Value>) {
        let mut sql = String::from("s.enabled = 1");
        let mut args: Vec<Value> = Vec::new();
        if !self.include_missing {
            sql.push_str(" AND a.state = 0");
        }
        if let Some(ids) = &self.sources {
            if ids.is_empty() {
                sql.push_str(" AND 0");
            } else {
                sql.push_str(&format!(" AND s.uid IN ({})", vec!["?"; ids.len()].join(",")));
                args.extend(ids.iter().cloned().map(Value::Text));
            }
        }
        if let Some(kinds) = &self.kinds {
            if kinds.is_empty() {
                sql.push_str(" AND 0");
            } else {
                sql.push_str(&format!(" AND a.kind IN ({})", vec!["?"; kinds.len()].join(",")));
                args.extend(kinds.iter().map(|k| Value::Text(k.key().to_owned())));
            }
        }
        if let Some((source, prefix)) = &self.folder {
            sql.push_str(" AND s.uid = ?");
            args.push(Value::Text(source.clone()));
            let prefix = prefix.trim_matches('/');
            if !prefix.is_empty() {
                sql.push_str(" AND a.rel LIKE ? ESCAPE '\\'");
                args.push(Value::Text(format!("{}/%", like_escape(prefix))));
            }
            if self.direct {
                let skip = if prefix.is_empty() { 0 } else { prefix.chars().count() + 1 };
                sql.push_str(&format!(" AND instr(substr(a.rel, {}), '/') = 0", skip + 1));
            }
        }
        for word in self.text.as_deref().unwrap_or("").split_whitespace() {
            if word.chars().count() >= 3 {
                sql.push_str(" AND a.id IN (SELECT rowid FROM assets_fts WHERE assets_fts MATCH ?)");
                args.push(Value::Text(format!("\"{}\"", word.replace('"', "\"\""))));
            } else {
                sql.push_str(" AND (a.filename LIKE ? ESCAPE '\\' OR a.rel LIKE ? ESCAPE '\\')");
                let pat = format!("%{}%", like_escape(word));
                args.push(Value::Text(pat.clone()));
                args.push(Value::Text(pat));
            }
        }
        (sql, args)
    }
}

impl Catalog {
    /// Entries by a raw condition over `a` (assets) and `s` (sources): the resolver's lookups.
    /// One present asset by its id, when its source can be read now (what a work may take from the catalog).
    pub(crate) fn entry(&self, uid: &str) -> Option<Entry> {
        self.entries_where("a.uid = ? AND a.state = 0 AND s.available = 1", &[Value::Text(uid.to_owned())]).into_iter().next()
    }

    pub(super) fn entries_where(&self, cond: &str, args: &[Value]) -> Vec<Entry> {
        let Ok(mut st) = self.db.prepare(&format!("{SELECT_ENTRY} WHERE {cond} ORDER BY a.id")) else { return Vec::new() };
        let Ok(rows) = st.query_map(params_from_iter(args.iter()), entry_from_row) else { return Vec::new() };
        rows.flatten().collect()
    }

    pub(crate) fn query(&self, q: &Query) -> Result<ResultSet, String> {
        let (cond, args) = q.conditions();
        let total: i64 = self
            .db
            .query_row(&format!("SELECT COUNT(*) FROM assets a JOIN sources s ON s.id = a.source_id WHERE {cond}"), params_from_iter(args.iter()), |r| r.get(0))
            .map_err(err)?;
        let limit = if q.limit == 0 { 500 } else { q.limit };
        let mut all = args.clone();
        all.push(Value::Integer(limit as i64));
        all.push(Value::Integer(q.offset as i64));
        let mut st = self.db.prepare(&format!("{SELECT_ENTRY} WHERE {cond} ORDER BY {} LIMIT ? OFFSET ?", q.sort.order(q.descending))).map_err(err)?;
        let rows = st.query_map(params_from_iter(all.iter()), entry_from_row).map_err(err)?;
        Ok(ResultSet { revision: self.revision(), total: total as usize, offset: q.offset, entries: rows.flatten().collect() })
    }

    /// The sub-folders directly under `prefix` in one source, with how many present assets each holds (folder browsing;
    /// the same catalog as the library view, no second index).
    pub(crate) fn folders(&self, source: &str, prefix: &str) -> Result<Vec<FolderCount>, String> {
        let prefix = prefix.trim_matches('/');
        let skip = if prefix.is_empty() { 0 } else { prefix.chars().count() + 1 };
        let mut sql = String::from("SELECT substr(rest, 1, instr(rest, '/') - 1) AS dir, COUNT(*) FROM (SELECT substr(a.rel, ?1) AS rest FROM assets a JOIN sources s ON s.id = a.source_id WHERE s.uid = ?2 AND a.state = 0");
        let mut args: Vec<Value> = vec![Value::Integer(skip as i64 + 1), Value::Text(source.to_owned())];
        if !prefix.is_empty() {
            sql.push_str(" AND a.rel LIKE ?3 ESCAPE '\\'");
            args.push(Value::Text(format!("{}/%", like_escape(prefix))));
        }
        sql.push_str(") WHERE instr(rest, '/') > 0 GROUP BY dir ORDER BY dir COLLATE NOCASE");
        let mut st = self.db.prepare(&sql).map_err(err)?;
        let rows = st.query_map(params_from_iter(args.iter()), |r| Ok(FolderCount { name: r.get(0)?, assets: r.get::<_, i64>(1)? as u64 })).map_err(err)?;
        Ok(rows.flatten().collect())
    }
}
