//! Reading a source's folder tree: what is on disk now. Read-only, one pass, never follows a folder link (so a link
//! loop cannot form) and never stops for one unreadable entry.

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// A supported file as the filesystem shows it right now.
#[derive(Clone, Debug)]
pub(crate) struct FsEntry {
    /// Path under the source root, `/`-separated.
    pub rel: String,
    pub filename: String,
    pub size: u64,
    pub mtime_ns: i64,
    /// The platform's file identity (device, inode); 0/0 when there is none worth trusting (a link, a non-Unix file).
    pub dev: u64,
    pub ino: u64,
    pub media_type: String,
}

pub(crate) struct Scan {
    pub entries: Vec<FsEntry>,
    /// Entries the walk could not read (permission, vanished mid-walk, broken link): counted, never fatal.
    pub errors: Vec<String>,
    /// The root itself was a readable folder.
    pub root_readable: bool,
}

fn hidden(name: &std::ffi::OsStr) -> bool {
    name.to_str().is_some_and(|n| n.starts_with('.'))
}

pub(crate) fn scan(root: &Path) -> Scan {
    let mut out = Scan { entries: Vec::new(), errors: Vec::new(), root_readable: root.is_dir() };
    if !out.root_readable {
        return out;
    }
    let walker = WalkDir::new(root).follow_links(false).sort_by_file_name().into_iter().filter_entry(|e| e.depth() == 0 || !hidden(e.file_name()));
    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                out.errors.push(e.to_string());
                continue;
            }
        };
        let path = entry.path();
        // the media type is the importer's own table, not a second list here
        let Some(media_type) = path.extension().and_then(|x| x.to_str()).and_then(crate::render::media::asset_type_for_extension) else { continue };
        let is_link = entry.path_is_symlink();
        // a link to a file is read through (once); a link to a folder is never entered
        let meta = if is_link { std::fs::metadata(path) } else { entry.metadata().map_err(std::io::Error::from) };
        let meta = match meta {
            Ok(m) if m.is_file() => m,
            Ok(_) => continue,
            Err(e) => {
                out.errors.push(format!("{}: {e}", path.display()));
                continue;
            }
        };
        let Ok(rel) = path.strip_prefix(root) else { continue };
        let rel = rel.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect::<Vec<_>>().join("/");
        let mtime_ns = meta.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_nanos() as i64).unwrap_or(0);
        let (dev, ino) = if is_link { (0, 0) } else { identity(&meta) };
        out.entries.push(FsEntry { filename: entry.file_name().to_string_lossy().into_owned(), rel, size: meta.len(), mtime_ns, dev, ino, media_type });
    }
    out
}

#[cfg(unix)]
fn identity(meta: &std::fs::Metadata) -> (u64, u64) {
    use std::os::unix::fs::MetadataExt;
    (meta.dev(), meta.ino())
}

#[cfg(not(unix))]
fn identity(_: &std::fs::Metadata) -> (u64, u64) {
    (0, 0)
}

/// `root` joined with a `/`-separated relative path.
pub(crate) fn join(root: &str, rel: &str) -> PathBuf {
    let mut p = PathBuf::from(root);
    for part in rel.split('/').filter(|s| !s.is_empty()) {
        p.push(part);
    }
    p
}
