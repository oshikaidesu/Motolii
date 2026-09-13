//! Freeze の裏仕事: 別 thread の engine が層のコマを順に焼き、書類の隣の cache へ置く(export と同じ型)。
//! 本番の engine は disk に増えたコマをそのまま読む。法は docs/freeze-and-flatten.md。
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use crate::doc::store::{Document, LayerId};
use crate::render::{engine::Engine, export::Cancel};
use serde_json::{json, Value};

struct State { phase: &'static str, layer: Option<u64>, done: i64, total: i64, error: Option<String> }
pub(crate) struct FreezeController { state: Arc<Mutex<State>>, cancel: Option<Cancel> }
impl Default for FreezeController {
    fn default() -> Self { Self { state: Arc::new(Mutex::new(State { phase: "idle", layer: None, done: 0, total: 0, error: None })), cancel: None } }
}
impl FreezeController {
    pub(crate) fn status(&self) -> Value {
        let s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        json!({ "phase": s.phase, "layer": s.layer, "done": s.done, "total": s.total, "error": s.error })
    }
    pub(crate) fn running(&self) -> Option<LayerId> {
        let s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        (s.phase == "running").then_some(s.layer.map(LayerId)).flatten()
    }
    pub(crate) fn cancel(&self) {
        if let Some(c) = &self.cancel { c.cancel(); let mut s = self.state.lock().unwrap_or_else(|e| e.into_inner()); if s.phase == "running" { s.phase = "cancelling" } }
    }
    /// 層の入点〜出点を順に焼く。既に走っていれば断る(1 本ずつ)。
    pub(crate) fn start(&mut self, document: &Document, layer: LayerId, root: Option<PathBuf>, start: i64, end: i64) -> Result<(), String> {
        if self.running().is_some() { return Err("A freeze is already running".into()); }
        if end <= start { return Err("The layer has no frames to freeze".into()); }
        let snapshot = document.flattened().map_err(|e| e.to_string())?;
        let cancel = Cancel::new(); self.cancel = Some(cancel.clone());
        { let mut s = self.state.lock().map_err(|e| e.to_string())?; *s = State { phase: "running", layer: Some(layer.0), done: 0, total: end - start, error: None }; }
        let state = self.state.clone();
        std::thread::Builder::new().name("motolii-port-freeze".into()).spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<bool, String> {
                let mut engine = Engine::new().map_err(|e| e.to_string())?;
                engine.set_cache_root(root);
                for frame in start..end {
                    if cancel.is_cancelled() { return Ok(false); }
                    engine.freeze_bake_frame(&snapshot.view(), layer, frame).map_err(|e| e.to_string())?;
                    let mut s = state.lock().unwrap_or_else(|e| e.into_inner()); s.done = frame - start + 1;
                }
                Ok(true)
            }));
            let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
            match result { Ok(Ok(true)) => s.phase = "complete", Ok(Ok(false)) => s.phase = "cancelled", Ok(Err(error)) => { s.phase = "failed"; s.error = Some(error) }, Err(_) => { s.phase = "failed"; s.error = Some("Freeze worker panicked".into()) } }
        }).map_err(|e| e.to_string())?;
        Ok(())
    }
}
impl Drop for FreezeController { fn drop(&mut self) { self.cancel(); } }
