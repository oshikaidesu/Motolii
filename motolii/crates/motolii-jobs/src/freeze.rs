//! Freeze の裏仕事: 別 thread の engine が層のコマを順に焼き、書類の隣の cache へ置く(export と同じ型)。
//! 本番の engine は disk に増えたコマをそのまま読む。法は docs/freeze-and-flatten.md。
use crate::doc::store::{Recording, LayerId};
use crate::render::{engine::Engine, export::Cancel};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

fn bake_frames(
    frames: std::ops::Range<i64>,
    cancel: &Cancel,
    mut bake: impl FnMut(i64) -> Result<bool, String>,
    mut progress: impl FnMut(i64),
) -> Result<bool, String> {
    let start = frames.start;
    let mut baked = 0;
    for frame in frames {
        if cancel.is_cancelled() { return Ok(false); }
        if bake(frame)? { baked += 1; }
        progress(frame - start + 1);
    }
    if baked == 0 { return Err("No frames could be frozen for this layer".into()); }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_bake_is_not_a_success_and_cancellation_stays_distinct() {
        let cancel = Cancel::new();
        assert!(bake_frames(0..3, &cancel, |_| Ok(false), |_| {}).is_err());
        let mut done = 0;
        assert_eq!(bake_frames(0..3, &cancel, |frame| Ok(frame == 1), |n| done = n), Ok(true));
        assert_eq!(done, 3);
        assert_eq!(bake_frames(0..3, &cancel, |_| Err("render failed".into()), |_| {}), Err("render failed".into()));
        cancel.cancel();
        assert_eq!(bake_frames(0..3, &cancel, |_| panic!("cancelled work rendered"), |_| {}), Ok(false));
    }

    #[test]
    fn rejected_freeze_does_not_copy_the_document() {
        let mut job = FreezeController::default();
        assert!(job.start(|| panic!("invalid range copied"), LayerId(1), None, 0, 0).is_err());
        { let mut state = job.state.lock().unwrap(); state.phase = "running"; state.layer = Some(1); }
        assert!(job.start(|| panic!("busy job copied"), LayerId(1), None, 0, 1).is_err());
    }

    #[test]
    fn snapshot_failure_does_not_start_freeze() {
        let mut job = FreezeController::default();
        assert_eq!(job.start(|| Err("snapshot failed".into()), LayerId(1), None, 0, 1), Err("snapshot failed".into()));
        assert_eq!(job.status()["phase"], "idle");
    }
}

struct State {
    phase: &'static str,
    layer: Option<u64>,
    done: i64,
    total: i64,
    error: Option<String>,
}
pub struct FreezeController {
    state: Arc<Mutex<State>>,
    cancel: Option<Cancel>,
}
impl Default for FreezeController {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                phase: "idle",
                layer: None,
                done: 0,
                total: 0,
                error: None,
            })),
            cancel: None,
        }
    }
}
impl FreezeController {
    pub fn status(&self) -> Value {
        let s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        json!({ "phase": s.phase, "layer": s.layer, "done": s.done, "total": s.total, "error": s.error })
    }
    pub fn running(&self) -> Option<LayerId> {
        let s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        (s.phase == "running")
            .then_some(s.layer.map(LayerId))
            .flatten()
    }
    pub fn cancel(&self) {
        if let Some(c) = &self.cancel {
            c.cancel();
            let mut s = self.state.lock().unwrap_or_else(|e| e.into_inner());
            if s.phase == "running" {
                s.phase = "cancelling"
            }
        }
    }
    /// 層の入点〜出点を順に焼く。既に走っていれば断る(1 本ずつ)。
    pub fn start(
        &mut self,
        snapshot: impl FnOnce() -> Result<Recording, String>,
        layer: LayerId,
        root: Option<PathBuf>,
        start: i64,
        end: i64,
    ) -> Result<(), String> {
        if self.running().is_some() {
            return Err("A freeze is already running".into());
        }
        if end <= start {
            return Err("The layer has no frames to freeze".into());
        }
        let snapshot = snapshot()?;
        let cancel = Cancel::new();
        self.cancel = Some(cancel.clone());
        {
            let mut s = self.state.lock().map_err(|e| e.to_string())?;
            *s = State {
                phase: "running",
                layer: Some(layer.0),
                done: 0,
                total: end - start,
                error: None,
            };
        }
        let state = self.state.clone();
        std::thread::Builder::new()
            .name("motolii-port-freeze".into())
            .spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                    || -> Result<bool, String> {
                        let mut engine = Engine::new().map_err(|e| e.to_string())?;
                        engine.set_cache_root(root);
                        bake_frames(start..end, &cancel, |frame| engine
                                .freeze_bake_frame(&snapshot.view(), layer, frame)
                                .map_err(|e| e.to_string()), |done| {
                            let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                            s.done = done;
                        })
                    },
                ));
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                match result {
                    Ok(Ok(true)) => s.phase = "complete",
                    Ok(Ok(false)) => s.phase = "cancelled",
                    Ok(Err(error)) => {
                        s.phase = "failed";
                        s.error = Some(error)
                    }
                    Err(_) => {
                        s.phase = "failed";
                        s.error = Some("Freeze worker panicked".into())
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
impl Drop for FreezeController {
    fn drop(&mut self) {
        self.cancel();
    }
}
