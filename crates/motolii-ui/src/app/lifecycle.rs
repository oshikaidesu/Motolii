//! shell寿命とlatest世代の不変条件を、描画やDocument編集から切り離して持つ。

use std::time::{Duration, Instant};

use crate::display_slot::DisplaySlotError;
use crate::render_worker::{RenderGeneration, RenderWorkerError, RepaintSignalRegistrationError};
use crate::static_preview::{StaticPreview, StaticPreviewEvidence};

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
        let current = preview.invariant_evidence();
        if current.document_json != self.baseline.document_json
            || current.slot.slot_id != self.baseline.slot.slot_id
            || current.slot.registration_count != self.baseline.slot.registration_count
            || current.render_count != self.baseline.render_count
        {
            return Err(LifecycleInvariantError);
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub(super) enum PreviewProjectionFailure {
    #[error(transparent)]
    Worker(RenderWorkerError),
    #[error(transparent)]
    Display(DisplaySlotError),
    #[error(transparent)]
    RepaintSignal(RepaintSignalRegistrationError),
}

#[derive(Debug, Default)]
pub(super) struct LatestResultProjection {
    pub(super) last_displayed_generation: Option<RenderGeneration>,
}

impl LatestResultProjection {
    pub(super) fn accepts(
        &self,
        result_generation: RenderGeneration,
        latest_accepted_generation: Option<RenderGeneration>,
    ) -> bool {
        Some(result_generation) == latest_accepted_generation
            && self
                .last_displayed_generation
                .is_none_or(|displayed| result_generation > displayed)
    }

    pub(super) fn commit(&mut self, generation: RenderGeneration) {
        self.last_displayed_generation = Some(generation);
    }
}

pub(super) struct LatestPreviewSmoke {
    pub(super) baseline: StaticPreviewEvidence,
    pub(super) expected_generation: RenderGeneration,
    pub(super) deadline: Instant,
}

impl LatestPreviewSmoke {
    pub(super) fn new(
        baseline: StaticPreviewEvidence,
        expected_generation: RenderGeneration,
    ) -> Self {
        Self {
            baseline,
            expected_generation,
            deadline: Instant::now() + Duration::from_secs(5),
        }
    }
}

pub(super) struct LifecycleSmoke {
    phase: SmokePhase,
    deadline: Instant,
    restore_paint_count: u32,
}

#[derive(Debug, Clone, Copy)]
enum SmokePhase {
    AwaitInitialPreview,
    Resize,
    Minimize,
    Restore,
    AwaitRestoredPaint,
}

impl LifecycleSmoke {
    pub(super) fn new() -> Self {
        Self {
            phase: SmokePhase::AwaitInitialPreview,
            deadline: Instant::now() + Duration::from_secs(5),
            restore_paint_count: 0,
        }
    }

    pub(super) fn advance(
        &mut self,
        ctx: &egui::Context,
        paint_count: u32,
        texture_id: egui::TextureId,
        initial_preview_ready: bool,
        projection: &mut StaticViewportProjection,
        preview: &StaticPreview,
    ) -> Result<Option<LifecycleSmokeOutcome>, String> {
        let now = Instant::now();
        if !matches!(self.phase, SmokePhase::AwaitInitialPreview) && now < self.deadline {
            ctx.request_repaint_after(self.deadline - now);
            return Ok(None);
        }
        let evidence = preview.invariant_evidence();
        match self.phase {
            SmokePhase::AwaitInitialPreview => {
                if initial_preview_ready {
                    self.phase = SmokePhase::Resize;
                    self.deadline = now;
                    ctx.request_repaint();
                } else if now >= self.deadline {
                    return Err("initial worker preview was not displayed".into());
                } else {
                    ctx.request_repaint_after(self.deadline - now);
                }
            }
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
    fn reversed_delivery_keeps_generation_two_after_one_copy() {
        let generation_one = RenderGeneration::new(1).unwrap();
        let generation_two = RenderGeneration::new(2).unwrap();
        let mut projection = LatestResultProjection::default();
        let latest = Some(generation_two);
        let mut copied = Vec::new();

        if projection.accepts(generation_two, latest) {
            copied.push(2);
            projection.commit(generation_two);
        }
        if projection.accepts(generation_one, latest) {
            copied.push(1);
            projection.commit(generation_one);
        }

        assert_eq!(copied, vec![2]);
        assert_eq!(projection.last_displayed_generation, Some(generation_two));
    }
}
