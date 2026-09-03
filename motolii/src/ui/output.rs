use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use dioxus_native::prelude::*;

use crate::render::export::{Cancel, ExportError, ExportJob, ExportProgress, ExportReport};
use crate::doc::store::Document;
use crate::ui::host::Poke;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ExportPhase {
    Idle,
    Preparing,
    Running,
    Cancelling,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ExportStatus {
    pub phase: ExportPhase,
    pub destination: Option<PathBuf>,
    pub frames_done: i64,
    pub frames_total: i64,
    pub detail: String,
}

impl Default for ExportStatus {
    fn default() -> Self {
        Self {
            phase: ExportPhase::Idle,
            destination: None,
            frames_done: 0,
            frames_total: 0,
            detail: String::new(),
        }
    }
}

#[derive(Default)]
struct ExportState {
    status: ExportStatus,
    cancel: Option<Cancel>,
    wake: Option<Poke>,
}

#[derive(Clone, Default)]
pub(super) struct ExportController(Arc<Mutex<ExportState>>);

impl PartialEq for ExportController {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl ExportController {
    pub(super) fn status(&self) -> ExportStatus {
        self.0.lock().unwrap().status.clone()
    }

    pub(super) fn is_active(&self) -> bool {
        matches!(
            self.0.lock().unwrap().status.phase,
            ExportPhase::Preparing | ExportPhase::Running | ExportPhase::Cancelling
        )
    }

    /// 終わった報せを畳む。次の書き出しまで status bar に残さない。
    pub(super) fn dismiss(&self) {
        let mut state = self.0.lock().unwrap();
        if matches!(state.status.phase, ExportPhase::Completed | ExportPhase::Cancelled | ExportPhase::Failed) {
            state.status.phase = ExportPhase::Idle;
        }
    }

    pub(super) fn cancel(&self) {
        let (cancel, wake) = {
            let mut state = self.0.lock().unwrap();
            // 支度の最中も止められる(長い書き出しで最初に押すのはここ)。
            if !matches!(state.status.phase, ExportPhase::Preparing | ExportPhase::Running) {
                return;
            }
            state.status.phase = ExportPhase::Cancelling;
            (state.cancel.clone(), state.wake.clone())
        };
        if let Some(cancel) = cancel {
            cancel.cancel();
        }
        if let Some(wake) = wake {
            wake.poke();
        }
    }

    pub(super) fn start(
        &self,
        doc: Arc<Mutex<Document>>,
        destination: PathBuf,
        poke: Poke,
    ) -> Result<(), String> {
        let cancel = Cancel::new();
        // 母数は最初から出す(音の支度の間 0 / 0 と出さない)。
        let frames_total = doc
            .lock()
            .unwrap()
            .view()
            .composition()
            .ok()
            .flatten()
            .map_or(0, |c| c.duration_frames);
        {
            let mut state = self.0.lock().unwrap();
            if matches!(
                state.status.phase,
                ExportPhase::Preparing | ExportPhase::Running | ExportPhase::Cancelling
            ) {
                return Err("Export is already running".to_owned());
            }
            state.status = ExportStatus {
                phase: ExportPhase::Preparing,
                destination: Some(destination.clone()),
                frames_done: 0,
                frames_total,
                detail: String::new(),
            };
            state.cancel = Some(cancel.clone());
            state.wake = Some(poke.clone());
        }
        poke.poke();

        let (temp_dir, snapshot) = match unique_snapshot() {
            Ok(paths) => paths,
            Err(error) => {
                self.fail(error.clone());
                poke.poke();
                return Err(error);
            }
        };
        if let Err(error) = doc.lock().unwrap().save(&snapshot) {
            let message = error.to_string();
            cleanup_snapshot(&temp_dir, &snapshot);
            self.fail(message.clone());
            poke.poke();
            return Err(message);
        }

        self.run(destination, cancel, snapshot, temp_dir, poke);
        Ok(())
    }

    fn run(
        &self,
        destination: PathBuf,
        cancel: Cancel,
        snapshot: PathBuf,
        temp_dir: PathBuf,
        poke: Poke,
    ) {
        self.running();
        poke.poke();
        let controller = self.clone();
        std::thread::spawn(move || {
            let done = (|| -> Result<ExportReport, ExportError> {
                let document = Document::load(&snapshot)
                    .map_err(|error| ExportError::Desc(error.to_string()))?;
                let mut engine = crate::render::engine::Engine::new()?;
                let job = ExportJob {
                    out_path: destination,
                    qp0: false,
                };
                crate::render::export::export_with_progress(
                    &mut engine,
                    &document.view(),
                    &job,
                    &cancel,
                    |progress| {
                        controller.progress(progress);
                        poke.poke();
                    },
                )
            })();

            cleanup_snapshot(&temp_dir, &snapshot);
            match done {
                Ok(report) => controller.complete(report),
                Err(ExportError::Cancelled) => controller.cancelled(),
                Err(error) => controller.fail(error.to_string()),
            }
            poke.poke();
        });
    }

    fn running(&self) {
        self.0.lock().unwrap().status.phase = ExportPhase::Running;
    }

    fn progress(&self, progress: ExportProgress) {
        let mut state = self.0.lock().unwrap();
        state.status.frames_done = progress.frames_done;
        state.status.frames_total = progress.frames_total;
    }

    fn complete(&self, report: ExportReport) {
        let mut state = self.0.lock().unwrap();
        state.status.phase = ExportPhase::Completed;
        state.status.destination = Some(report.out_path);
        state.status.frames_done = report.frames_written;
        state.status.frames_total = report.frames_written;
        state.status.detail.clear();
        state.cancel = None;
    }

    fn cancelled(&self) {
        let mut state = self.0.lock().unwrap();
        state.status.phase = ExportPhase::Cancelled;
        state.status.detail.clear();
        state.cancel = None;
    }

    fn fail(&self, detail: String) {
        let mut state = self.0.lock().unwrap();
        state.status.phase = ExportPhase::Failed;
        state.status.detail = detail;
        state.cancel = None;
    }
}

fn unique_snapshot() -> Result<(PathBuf, PathBuf), String> {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    for _ in 0..32 {
        let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("motolii-export-{}-{id}", std::process::id()));
        match std::fs::create_dir(&dir) {
            Ok(()) => return Ok((dir.clone(), dir.join("project.rrd"))),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("Could not reserve an export snapshot".to_owned())
}

fn cleanup_snapshot(dir: &Path, snapshot: &Path) {
    let _ = std::fs::remove_file(snapshot);
    let _ = std::fs::remove_dir(dir);
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum OutputSurface {
    Panel,
    StatusBar,
}

#[component]
pub(super) fn OutputStatus(
    controller: ExportController,
    surface: OutputSurface,
    generation: u32,
    /// 白紙(comp 無し)の時の一言。Panel だけが持つ。
    #[props(default)] idle_note: Option<String>,
) -> Element {
    let _ = generation;
    let status = controller.status();
    let destination = status
        .destination
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    let file_name = status
        .destination
        .as_ref()
        .and_then(|path| path.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let text = match status.phase {
        ExportPhase::Idle if surface == OutputSurface::Panel => idle_note.unwrap_or_else(|| "Ready to export".to_owned()),
        ExportPhase::Idle => String::new(),
        ExportPhase::Preparing => format!("Preparing export · {file_name}"),
        ExportPhase::Running => format!(
            "Exporting {} / {} · {file_name}",
            status.frames_done, status.frames_total
        ),
        ExportPhase::Cancelling => "Cancelling export…".to_owned(),
        ExportPhase::Completed if surface == OutputSurface::Panel => format!("Export finished · {destination}"),
        ExportPhase::Completed => format!("Export finished · {file_name}"),
        ExportPhase::Cancelled => format!("Export cancelled · {file_name}"),
        ExportPhase::Failed => format!("Export failed · {}", status.detail),
    };
    let class = if surface == OutputSurface::Panel {
        "output-status panel"
    } else {
        "output-status compact"
    };

    rsx!(
        if !text.is_empty() {
            div { class: class, role: "status", aria_live: "polite",
                span { "{text}" }
                if status.phase == ExportPhase::Running {
                    button {
                        class: "output-cancel",
                        onclick: {
                            let controller = controller.clone();
                            move |_| controller.cancel()
                        },
                        "Cancel"
                    }
                }
                if status.phase == ExportPhase::Completed {
                    if let Some(path) = status.destination.clone() {
                        button {
                            class: "output-cancel",
                            onclick: move |_| reveal_in_finder(&path),
                            "Show in Finder"
                        }
                    }
                }
                if matches!(status.phase, ExportPhase::Completed | ExportPhase::Cancelled | ExportPhase::Failed) {
                    button {
                        class: "output-cancel",
                        aria_label: "Dismiss",
                        onclick: {
                            let controller = controller.clone();
                            move |_| controller.dismiss()
                        },
                        "×"
                    }
                }
            }
        }
    )
}

/// 出来た物を Finder で指す(Premiere・Resolve の「書き出し先を開く」)。
pub(super) fn reveal_in_finder(path: &std::path::Path) {
    let _ = std::process::Command::new("open").arg("-R").arg(path).spawn();
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus_core::Properties;

    #[test]
    fn external_generation_prevents_output_status_memoization() {
        let controller = ExportController::default();
        let mut before = OutputStatusProps {
            controller: controller.clone(),
            surface: OutputSurface::Panel,
            generation: 1,
            idle_note: None,
        };
        let after = OutputStatusProps {
            controller,
            surface: OutputSurface::Panel,
            generation: 2,
            idle_note: None,
        };
        assert!(!before.memoize(&after));
    }

    #[test]
    fn progress_and_completion_are_one_state_machine() {
        let controller = ExportController::default();
        {
            let mut state = controller.0.lock().unwrap();
            state.status.destination = Some(PathBuf::from("movie.mp4"));
            state.cancel = Some(Cancel::new());
        }
        controller.running();
        controller.progress(ExportProgress {
            frames_done: 7,
            frames_total: 30,
        });
        assert_eq!(controller.status().phase, ExportPhase::Running);
        assert_eq!(controller.status().frames_done, 7);

        controller.complete(ExportReport {
            out_path: PathBuf::from("movie.mp4"),
            frames_written: 30,
        });
        assert_eq!(controller.status().phase, ExportPhase::Completed);
        assert_eq!(controller.status().frames_done, 30);
        assert!(controller.0.lock().unwrap().cancel.is_none());
    }

    #[test]
    fn cancel_is_visible_only_during_the_running_phase() {
        let controller = ExportController::default();
        let cancel = Cancel::new();
        {
            let mut state = controller.0.lock().unwrap();
            state.status.phase = ExportPhase::Running;
            state.cancel = Some(cancel);
        }
        controller.cancel();
        assert_eq!(controller.status().phase, ExportPhase::Cancelling);
        controller.cancelled();
        assert_eq!(controller.status().phase, ExportPhase::Cancelled);
        assert!(!controller.is_active());
    }

    #[test]
    fn snapshots_have_unique_owned_locations() {
        let (a_dir, a) = unique_snapshot().unwrap();
        let (b_dir, b) = unique_snapshot().unwrap();
        assert_ne!(a, b);
        cleanup_snapshot(&a_dir, &a);
        cleanup_snapshot(&b_dir, &b);
    }
}
