//! The JSON door to the Catalog, for the host: one request in, one reply out. The Catalog is the app's, not a document's
//! (one index for every window), so it is a process-wide value behind one lock, opened on first use in the state folder
//! (`MOTOLII_STATE_DIR` when set: the real-app tests never touch a person's own index).

use std::sync::{Mutex, OnceLock};

use rusqlite::types::Value;
use serde_json::{json, Value as J};

use super::{Catalog, Entry, MediaKind, Query, Resolution, SavedRef, Sort, Source};

static CATALOG: OnceLock<Mutex<Result<Catalog, String>>> = OnceLock::new();

fn shared() -> &'static Mutex<Result<Catalog, String>> {
    CATALOG.get_or_init(|| {
        let opened = crate::editor::history::default_file().map(|f| f.with_file_name("catalog.sqlite")).ok_or_else(|| "No state folder".to_owned()).and_then(|p| Catalog::open(&p));
        Mutex::new(opened)
    })
}

fn source_json(s: &Source) -> J {
    json!({"id": s.id, "name": s.name, "root": s.root.to_string_lossy(), "enabled": s.enabled, "available": s.available, "assets": s.assets})
}

fn entry_json(e: &Entry) -> J {
    json!({
        "id": e.id, "source": e.source, "sourceName": e.source_name, "path": e.abs_path, "rel": e.rel_path, "name": e.filename,
        "kind": e.kind.key(), "mime": e.media_type, "size": e.size, "mtimeNs": e.mtime_ns,
        "width": e.width, "height": e.height, "fingerprint": e.fingerprint, "faceKey": e.face_key,
        "missing": e.missing, "sourceAvailable": e.source_available,
    })
}

fn strings(j: &J) -> Option<Vec<String>> {
    j.as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_owned)).collect())
}

fn query_of(j: &J) -> Query {
    Query {
        sources: strings(&j["sources"]),
        folder: j["folder"]["source"].as_str().map(|s| (s.to_owned(), j["folder"]["prefix"].as_str().unwrap_or("").to_owned())),
        direct: j["direct"].as_bool().unwrap_or(false),
        kinds: strings(&j["kinds"]).map(|k| k.iter().filter_map(|k| MediaKind::parse(k)).collect()),
        text: j["text"].as_str().map(str::to_owned),
        include_missing: j["includeMissing"].as_bool().unwrap_or(false),
        sort: Sort::parse(j["sort"].as_str().unwrap_or("name")),
        descending: j["descending"].as_bool().unwrap_or(false),
        offset: j["offset"].as_u64().unwrap_or(0) as usize,
        limit: j["limit"].as_u64().unwrap_or(0) as usize,
    }
}

fn resolution_json(r: &Resolution) -> J {
    match r {
        Resolution::Exact(e) => json!({"result": "exact", "entry": entry_json(e)}),
        Resolution::Strong { entry, why } => json!({"result": "strong", "entry": entry_json(entry), "why": why}),
        Resolution::Ambiguous(v) => json!({"result": "ambiguous", "candidates": v.iter().map(entry_json).collect::<Vec<_>>()}),
        Resolution::SourceUnavailable { source, name } => json!({"result": "sourceUnavailable", "source": source, "name": name}),
        Resolution::Missing => json!({"result": "missing"}),
    }
}

fn entries_of(c: &Catalog, ids: &[String]) -> Vec<Entry> {
    let cond = format!("a.uid IN ({})", vec!["?"; ids.len().max(1)].join(","));
    let args: Vec<Value> = if ids.is_empty() { vec![Value::Text(String::new())] } else { ids.iter().cloned().map(Value::Text).collect() };
    c.entries_where(&cond, &args)
}

fn state_of(c: &Catalog) -> J {
    json!({"revision": c.revision(), "sources": c.sources().iter().map(source_json).collect::<Vec<_>>(), "pending": c.pending_enrich()})
}

/// The faces the shelf's own code already knows how to draw, made from files the catalog names: the catalog does not
/// draw; it hands the existing thumbnail / waveform / facts routines the path (they cache under it).
fn faces_json(entries: Vec<Entry>) -> J {
    let mut out = serde_json::Map::new();
    for e in entries {
        if e.missing || !e.source_available {
            continue;
        }
        let mut face = serde_json::Map::new();
        match e.kind {
            MediaKind::Image | MediaKind::Environment => {
                if let Some(t) = crate::editor::thumbnail::image_data_uri(&e.abs_path) {
                    face.insert("thumbnail".into(), t.into());
                }
            }
            MediaKind::Video => {
                if let Some(t) = crate::editor::thumbnail::video_data_uri(&e.abs_path) {
                    face.insert("thumbnail".into(), t.into());
                }
            }
            MediaKind::Audio => {
                if let Some(p) = crate::editor::thumbnail::audio_peaks(&e.abs_path) {
                    face.insert("peaks".into(), json!(p));
                }
            }
            MediaKind::Model => {}
        }
        if let Some(f) = crate::editor::thumbnail::facts(&e.abs_path, &e.media_type) {
            face.insert("facts".into(), f);
        }
        out.insert(e.id, J::Object(face));
    }
    json!({ "faces": out })
}

/// The catalog for a caller inside the host (a work looking up its missing media), under the same lock the JSON door takes.
pub(crate) fn with<R>(f: impl FnOnce(&mut Catalog) -> R) -> Result<R, String> {
    let mut guard = shared().lock().map_err(|_| "catalog lock poisoned".to_owned())?;
    let c = guard.as_mut().map_err(|e| e.clone())?;
    Ok(f(c))
}

/// One request. Errors come back as `{"error": …}`; nothing here panics into the host.
pub(crate) fn request(text: &str) -> String {
    let reply = std::panic::catch_unwind(|| handle(text)).unwrap_or_else(|_| Err("catalog panic".into()));
    match reply {
        Ok(j) => j,
        Err(e) => json!({ "error": e }),
    }
    .to_string()
}

fn handle(text: &str) -> Result<J, String> {
    let j: J = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if j["op"] == "watch" {
        // a signal for changes, not the truth: on, the sources are refreshed after they change; off, only when asked.
        // (Before the catalog lock is taken: starting to watch reads the sources through it.)
        let roots = if j["on"].as_bool().unwrap_or(true) { super::watch::start()? } else { super::watch::stop() };
        return Ok(json!({"watching": super::watch::watching(), "roots": roots}));
    }
    // The slow ops read files (a walk, fingerprints, thumbnails, decoded frames): none of them holds the catalog lock while
    // it does, so a query or a click never waits behind one. The lock is taken only to read what is needed and to write.
    match j["op"].as_str().unwrap_or("sources") {
        "refresh" => {
            let reports = super::index::refresh_unlocked(j["id"].as_str())?;
            let mut s = with(|c| state_of(c))?;
            s["reports"] = json!(reports.iter().map(|r| json!({"source": r.source, "scanned": r.scanned, "added": r.added, "changed": r.changed, "moved": r.moved, "missing": r.missing, "ambiguous": r.ambiguous, "errors": r.errors.len(), "unavailable": r.unavailable, "folderMoves": r.folder_moves})).collect::<Vec<_>>());
            return Ok(s);
        }
        "enrich" => {
            let done = super::index::enrich_unlocked(j["limit"].as_u64().unwrap_or(200) as usize)?;
            let mut s = with(|c| state_of(c))?;
            s["done"] = done.into();
            return Ok(s);
        }
        "faces" => {
            let ids = strings(&j["ids"]).unwrap_or_default();
            let entries = with(|c| entries_of(c, &ids))?;
            return Ok(faces_json(entries));
        }
        "picture" => {
            let e = with(|c| c.entries_where("a.uid = ? AND a.state = 0 AND s.available = 1", &[Value::Text(j["id"].as_str().unwrap_or("").to_owned())]).into_iter().next())?.ok_or("No such picture")?;
            if !matches!(e.kind, MediaKind::Image | MediaKind::Environment) {
                return Err("Not a still".into());
            }
            let edge = j["edge"].as_u64().unwrap_or(1024).clamp(64, 4096) as u32;
            return Ok(json!({"picture": crate::editor::thumbnail::image_data_uri_sized(&e.abs_path, edge)}));
        }
        "frame" => {
            // one frame of a clip at a time, for scrubbing in the preview; a person's file is only read
            let e = with(|c| c.entries_where("a.uid = ? AND a.state = 0 AND s.available = 1", &[Value::Text(j["id"].as_str().unwrap_or("").to_owned())]).into_iter().next())?.ok_or("No such clip")?;
            if e.kind != MediaKind::Video {
                return Err("Not a clip".into());
            }
            let at = j["at"].as_f64().unwrap_or(0.0);
            let edge = j["edge"].as_u64().unwrap_or(480).clamp(32, 1280) as u32;
            return Ok(json!({"frame": crate::editor::thumbnail::video_frame_at(&e.abs_path, at, edge)}));
        }
        _ => {}
    }
    let guard = &mut *shared().lock().map_err(|_| "catalog lock poisoned".to_owned())?;
    let c = guard.as_mut().map_err(|e| e.clone())?;
    let state = state_of;
    let id = || j["id"].as_str().ok_or_else(|| "Missing id".to_owned());
    match j["op"].as_str().unwrap_or("sources") {
        "sources" => Ok(state(c)),
        "addSource" => {
            c.add_source(std::path::Path::new(j["path"].as_str().ok_or("Missing path")?), j["name"].as_str())?;
            Ok(state(c))
        }
        "removeSource" => {
            c.remove_source(id()?)?;
            Ok(state(c))
        }
        "enableSource" => {
            c.set_enabled(id()?, j["enabled"].as_bool().unwrap_or(true))?;
            Ok(state(c))
        }
        "renameSource" => {
            c.rename_source(id()?, j["name"].as_str().ok_or("Missing name")?)?;
            Ok(state(c))
        }
        "relocateSource" => {
            c.relocate_source(id()?, std::path::Path::new(j["path"].as_str().ok_or("Missing path")?))?;
            Ok(state(c))
        }
        "forgetMissing" => {
            let n = c.forget_missing(id()?)?;
            let mut s = state(c);
            s["forgotten"] = n.into();
            Ok(s)
        }
        "query" => {
            let rs = c.query(&query_of(&j))?;
            Ok(json!({"revision": rs.revision, "total": rs.total, "offset": rs.offset, "entries": rs.entries.iter().map(entry_json).collect::<Vec<_>>()}))
        }
        "folders" => {
            let f = c.folders(id().or_else(|_| j["source"].as_str().ok_or("Missing source"))?, j["prefix"].as_str().unwrap_or(""))?;
            Ok(json!({"folders": f.iter().map(|f| json!({"name": f.name, "assets": f.assets})).collect::<Vec<_>>()}))
        }
        "resolve" => {
            let refs: Vec<SavedRef> = j["refs"]
                .as_array()
                .ok_or("Missing refs")?
                .iter()
                .map(|r| SavedRef { path: r["path"].as_str().unwrap_or("").to_owned(), size: r["size"].as_u64(), content_hash: r["contentHash"].as_str().map(str::to_owned), file_name: r["fileName"].as_str().map(str::to_owned) })
                .collect();
            Ok(json!({"results": c.resolve_many(&refs).iter().map(resolution_json).collect::<Vec<_>>()}))
        }
        other => Err(format!("Unknown catalog op {other}")),
    }
}

/// Tests share one process and so one catalog: it lives in a folder made once for them, never in a person's state.
#[cfg(test)]
pub(crate) fn use_test_state() {
    static DIR: OnceLock<tempfile::TempDir> = OnceLock::new();
    let dir = DIR.get_or_init(|| tempfile::tempdir().unwrap());
    std::env::set_var("MOTOLII_STATE_DIR", dir.path());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unknown_op_and_bad_json_answer_with_an_error_and_do_not_panic() {
        use_test_state();
        assert!(request("{not json").contains("error"));
        assert!(request(r#"{"op":"nope"}"#).contains("Unknown catalog op"));
        let listed: J = serde_json::from_str(&request(r#"{"op":"sources"}"#)).unwrap();
        assert!(listed["sources"].is_array());
    }
}
