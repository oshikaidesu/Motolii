//! 事前準備済みの静止native textureだけを中央Stageへ投影する。

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::static_preview::{StaticPreview, StaticPreviewEvidence};

#[derive(Debug, thiserror::Error)]
pub(crate) enum AppConstructionError {
    #[error("wgpu render state is not available")]
    MissingWgpuRenderState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LifecycleSmokeOutcome {
    NotRequested,
    Passed,
    Failed(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ShellLifecycleInput {
    Resized([f32; 2]),
    ScaleFactorChanged(f32),
    Minimized,
    Restored,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("static viewport evidence changed during shell lifecycle")]
pub(crate) struct LifecycleInvariantError;

#[derive(Debug)]
pub(crate) struct StaticViewportProjection {
    baseline: StaticPreviewEvidence,
    logical_size: [f32; 2],
    pixels_per_point: f32,
    minimized: bool,
}

impl StaticViewportProjection {
    pub(crate) fn new(preview: &StaticPreview) -> Self {
        Self {
            baseline: preview.invariant_evidence(),
            logical_size: [0.0, 0.0],
            pixels_per_point: 1.0,
            minimized: false,
        }
    }

    pub(crate) fn observe(
        &mut self,
        input: ShellLifecycleInput,
        preview: &StaticPreview,
    ) -> Result<(), LifecycleInvariantError> {
        match input {
            ShellLifecycleInput::Resized(logical_size) => self.logical_size = logical_size,
            ShellLifecycleInput::ScaleFactorChanged(pixels_per_point) => {
                self.pixels_per_point = pixels_per_point;
            }
            ShellLifecycleInput::Minimized => self.minimized = true,
            ShellLifecycleInput::Restored => self.minimized = false,
        }
        if preview.invariant_evidence() != self.baseline {
            return Err(LifecycleInvariantError);
        }
        Ok(())
    }
}

pub(crate) struct MotoliiApp {
    preview: Arc<StaticPreview>,
    texture_id: egui::TextureId,
    projection: StaticViewportProjection,
    paint_count: u32,
    smoke: Option<LifecycleSmoke>,
    smoke_outcome: Arc<Mutex<LifecycleSmokeOutcome>>,
}

impl MotoliiApp {
    pub(crate) fn new(
        cc: &eframe::CreationContext<'_>,
        preview: Arc<StaticPreview>,
        lifecycle_smoke: bool,
        smoke_outcome: Arc<Mutex<LifecycleSmokeOutcome>>,
    ) -> Result<Self, AppConstructionError> {
        let render_state = cc
            .wgpu_render_state
            .as_ref()
            .ok_or(AppConstructionError::MissingWgpuRenderState)?;
        let texture_id = {
            let mut renderer = render_state.renderer.write();
            preview
                .slot()
                .register_once(&render_state.device, &mut renderer)
        };
        let evidence = preview.invariant_evidence();
        eprintln!(
            "U1A1_REGISTER slot={} texture={texture_id:?} registrations={} copies={} renders={}",
            evidence.slot.slot_id,
            evidence.slot.registration_count,
            evidence.slot.copy_count,
            evidence.render_count
        );
        let projection = StaticViewportProjection::new(&preview);
        Ok(Self {
            preview,
            texture_id,
            projection,
            paint_count: 0,
            smoke: lifecycle_smoke.then(LifecycleSmoke::new),
            smoke_outcome,
        })
    }

    fn record_smoke_failure(&self, reason: String) {
        if let Ok(mut outcome) = self.smoke_outcome.lock() {
            *outcome = LifecycleSmokeOutcome::Failed(reason);
        }
    }
}

impl eframe::App for MotoliiApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Some(smoke) = &mut self.smoke else {
            return;
        };
        match smoke.advance(
            ctx,
            self.paint_count,
            self.texture_id,
            &mut self.projection,
            &self.preview,
        ) {
            Ok(Some(LifecycleSmokeOutcome::Passed)) => {
                if let Ok(mut outcome) = self.smoke_outcome.lock() {
                    *outcome = LifecycleSmokeOutcome::Passed;
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Ok(_) => {}
            Err(reason) => {
                self.record_smoke_failure(reason);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.paint_count = self.paint_count.saturating_add(1);
        let available = ui.available_size();
        for input in [
            ShellLifecycleInput::Resized([available.x, available.y]),
            ShellLifecycleInput::ScaleFactorChanged(ui.ctx().pixels_per_point()),
        ] {
            if self.projection.observe(input, &self.preview).is_err() {
                self.record_smoke_failure(LifecycleInvariantError.to_string());
            }
        }

        let desc = self.preview.slot().desc();
        let source_size = egui::vec2(desc.width as f32, desc.height as f32);
        let target_size = fit_inside(source_size, available);
        ui.centered_and_justified(|ui| {
            ui.push_id("motolii-stage-viewport", |ui| {
                ui.add(
                    egui::Image::from_texture((self.texture_id, source_size))
                        .fit_to_exact_size(target_size),
                );
            });
        });
    }
}

fn fit_inside(source: egui::Vec2, available: egui::Vec2) -> egui::Vec2 {
    if source.x <= 0.0 || source.y <= 0.0 || available.x <= 0.0 || available.y <= 0.0 {
        return egui::Vec2::ZERO;
    }
    let scale = (available.x / source.x).min(available.y / source.y);
    source * scale
}

struct LifecycleSmoke {
    phase: SmokePhase,
    deadline: Instant,
    restore_paint_count: u32,
}

#[derive(Debug, Clone, Copy)]
enum SmokePhase {
    Resize,
    Minimize,
    Restore,
    AwaitRestoredPaint,
}

impl LifecycleSmoke {
    fn new() -> Self {
        Self {
            phase: SmokePhase::Resize,
            deadline: Instant::now(),
            restore_paint_count: 0,
        }
    }

    fn advance(
        &mut self,
        ctx: &egui::Context,
        paint_count: u32,
        texture_id: egui::TextureId,
        projection: &mut StaticViewportProjection,
        preview: &StaticPreview,
    ) -> Result<Option<LifecycleSmokeOutcome>, String> {
        let now = Instant::now();
        if now < self.deadline {
            ctx.request_repaint_after(self.deadline - now);
            return Ok(None);
        }
        let evidence = preview.invariant_evidence();
        match self.phase {
            SmokePhase::Resize => {
                projection
                    .observe(ShellLifecycleInput::Resized([800.0, 520.0]), preview)
                    .map_err(|error| error.to_string())?;
                log_lifecycle("resize", texture_id, &evidence, None);
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(800.0, 520.0)));
                self.phase = SmokePhase::Minimize;
                self.deadline = now + Duration::from_millis(250);
                ctx.request_repaint_after(Duration::from_millis(250));
            }
            SmokePhase::Minimize => {
                projection
                    .observe(ShellLifecycleInput::Minimized, preview)
                    .map_err(|error| error.to_string())?;
                log_lifecycle("minimize", texture_id, &evidence, None);
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                self.phase = SmokePhase::Restore;
                self.deadline = now + Duration::from_millis(350);
                ctx.request_repaint_after(Duration::from_millis(350));
            }
            SmokePhase::Restore => {
                projection
                    .observe(ShellLifecycleInput::Restored, preview)
                    .map_err(|error| error.to_string())?;
                log_lifecycle("restore", texture_id, &evidence, None);
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(960.0, 640.0)));
                self.restore_paint_count = paint_count;
                self.phase = SmokePhase::AwaitRestoredPaint;
                self.deadline = now + Duration::from_secs(5);
                ctx.request_repaint();
            }
            SmokePhase::AwaitRestoredPaint => {
                if paint_count > self.restore_paint_count {
                    projection
                        .observe(ShellLifecycleInput::Resized([960.0, 640.0]), preview)
                        .map_err(|error| error.to_string())?;
                    log_lifecycle("passed", texture_id, &evidence, Some(paint_count));
                    return Ok(Some(LifecycleSmokeOutcome::Passed));
                }
                if now >= self.deadline {
                    return Err("no paint observed after restore".into());
                }
                ctx.request_repaint_after(Duration::from_millis(50));
            }
        }
        Ok(None)
    }
}

fn log_lifecycle(
    phase: &str,
    texture_id: egui::TextureId,
    evidence: &StaticPreviewEvidence,
    paint_count: Option<u32>,
) {
    eprint!(
        "U1A1_LIFECYCLE {phase} slot={} texture={texture_id:?} registrations={} copies={} renders={}",
        evidence.slot.slot_id,
        evidence.slot.registration_count,
        evidence.slot.copy_count,
        evidence.render_count
    );
    if let Some(paint_count) = paint_count {
        eprint!(" paint_count={paint_count}");
    }
    eprintln!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_inside_preserves_aspect_without_window_state() {
        assert_eq!(
            fit_inside(egui::vec2(16.0, 9.0), egui::vec2(800.0, 600.0)),
            egui::vec2(800.0, 450.0)
        );
        assert_eq!(
            fit_inside(egui::vec2(16.0, 9.0), egui::vec2(320.0, 100.0)),
            egui::vec2(1600.0 / 9.0, 100.0)
        );
    }
}
