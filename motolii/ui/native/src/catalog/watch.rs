//! Watching the sources: a signal that something under one changed, so the index catches up without being asked.
//! `notify` says a folder changed; it is never the truth. A change only marks its source dirty, and after a quiet moment
//! that source is refreshed the ordinary way (an incremental diff), so a lost or coalesced event, a volume that came and
//! went, or a network folder that sends nothing are all recovered by the next refresh (any explicit one, or the next event).

use std::collections::HashSet;
use std::sync::mpsc::{channel, RecvTimeoutError, Sender};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use super::bridge::with;

struct Running {
    _watcher: RecommendedWatcher,
    stop: Sender<()>,
}

static RUNNING: OnceLock<Mutex<Option<Running>>> = OnceLock::new();

/// How long the sources stay quiet before the changed ones are refreshed (a copy of many files is one refresh).
const QUIET: Duration = Duration::from_millis(400);

fn slot() -> &'static Mutex<Option<Running>> {
    RUNNING.get_or_init(|| Mutex::new(None))
}

pub(crate) fn watching() -> bool {
    slot().lock().map(|g| g.is_some()).unwrap_or(false)
}

/// Stops watching. Returns 0 (the roots now watched).
pub(crate) fn stop() -> usize {
    if let Ok(mut g) = slot().lock() {
        if let Some(r) = g.take() {
            let _ = r.stop.send(());
        }
    }
    0
}

/// (Re)starts watching every enabled source that can be read now. Returns how many roots are watched.
pub(crate) fn start() -> Result<usize, String> {
    stop();
    let roots: Vec<(String, std::path::PathBuf)> = with(|c| c.sources().into_iter().filter(|s| s.enabled && s.available).map(|s| (s.id, s.root)).collect())?;
    let (events, heard) = channel::<std::path::PathBuf>();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(e) = res {
            if !matches!(e.kind, EventKind::Access(_)) {
                for p in e.paths {
                    let _ = events.send(p);
                }
            }
        }
    })
    .map_err(|e| e.to_string())?;
    let mut watched = 0;
    for (_, root) in &roots {
        if watcher.watch(root, RecursiveMode::Recursive).is_ok() {
            watched += 1;
        }
    }
    let (stop_tx, stop_rx) = channel::<()>();
    thread::spawn(move || {
        let mut dirty: HashSet<String> = HashSet::new();
        loop {
            match heard.recv_timeout(if dirty.is_empty() { Duration::from_millis(500) } else { QUIET }) {
                Ok(path) => {
                    for (id, root) in &roots {
                        if path.starts_with(root) {
                            dirty.insert(id.clone());
                        }
                    }
                    continue;
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return,
            }
            if stop_rx.try_recv().is_ok() {
                return;
            }
            for id in dirty.drain() {
                let _ = super::index::refresh_unlocked(Some(&id));
            }
        }
    });
    *slot().lock().map_err(|_| "watch lock poisoned".to_owned())? = Some(Running { _watcher: watcher, stop: stop_tx });
    Ok(watched)
}
