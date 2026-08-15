//! workerのlatest結果だけを表示slotへ載せ、古い世代を描き戻さない。

use std::sync::Arc;
use std::time::Instant;

use crate::render_worker::{RenderWorkerClient, RepaintSignalRegistrationError};

use super::lifecycle::{LifecycleSmokeOutcome, PreviewProjectionFailure};
use super::MotoliiApp;

impl MotoliiApp {
    pub(super) fn recover_repaint_signal(&mut self) {
        let Some(failed_epoch) = self.render_client.failed_repaint_signal_epoch() else {
            return;
        };
        if self.last_handled_signal_failure == Some(failed_epoch) {
            return;
        }
        match register_repaint_signal(&self.render_client, &self.repaint_context) {
            Ok(()) => self.last_handled_signal_failure = Some(failed_epoch),
            Err(error) => {
                self.record_preview_failure(PreviewProjectionFailure::RepaintSignal(error));
            }
        }
    }

    pub(super) fn drain_latest_result(&mut self) {
        let Some(result) = self.render_client.try_take_latest() else {
            return;
        };
        let latest = self.render_client.latest_accepted_generation();
        if !self.latest_projection.accepts(result.generation, latest) {
            return;
        }
        let rendered = match result.result {
            Ok(rendered) => rendered,
            Err(error) => {
                self.record_preview_failure(PreviewProjectionFailure::Worker(error));
                return;
            }
        };
        if let Err(error) = self.preview.slot().copy(&self.gpu, &rendered.frame) {
            self.record_preview_failure(PreviewProjectionFailure::Display(error));
            return;
        }
        self.latest_camera = Some(rendered.camera);
        self.latest_projection.commit(result.generation);
    }

    pub(super) fn advance_latest_smoke(&mut self, ctx: &egui::Context) -> bool {
        let Some(smoke) = &self.latest_smoke else {
            return false;
        };
        if self.latest_projection.last_displayed_generation == Some(smoke.expected_generation) {
            let evidence = self.preview.invariant_evidence();
            if evidence.slot.slot_id == smoke.baseline.slot.slot_id
                && evidence.slot.registration_count == smoke.baseline.slot.registration_count
                && evidence.slot.copy_count == smoke.baseline.slot.copy_count + 1
                && evidence.document_json == smoke.baseline.document_json
            {
                eprintln!(
                    "U1B2_LATEST passed slot={} registrations={} copies={} generation={}",
                    evidence.slot.slot_id,
                    evidence.slot.registration_count,
                    evidence.slot.copy_count,
                    smoke.expected_generation.get()
                );
                if let Ok(mut outcome) = self.smoke_outcome.lock() {
                    *outcome = LifecycleSmokeOutcome::Passed;
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return true;
            }
        }
        if Instant::now() >= smoke.deadline {
            self.record_smoke_failure("latest preview result was not projected".into());
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return true;
        }
        ctx.request_repaint_after(smoke.deadline.saturating_duration_since(Instant::now()));
        false
    }

    fn record_preview_failure(&mut self, error: PreviewProjectionFailure) {
        eprintln!("U1B2_PREVIEW_REJECT error={error}");
        self.preview_failure = Some(error);
    }
}

pub(super) fn register_repaint_signal(
    client: &RenderWorkerClient,
    context: &egui::Context,
) -> Result<(), RepaintSignalRegistrationError> {
    let context = context.clone();
    client
        .register_repaint_signal(Arc::new(move || context.request_repaint()))
        .map(|_| ())
}
