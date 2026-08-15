//! Document編集の適用とU2b-1 smoke。表示slotやlayoutには触れない。

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::document_edit_runtime::{
    DocumentEditActionKind, DocumentEditDispatchError, DocumentEditQueue, DocumentEditRuntimeError,
    PublishedDocument,
};
use crate::render_worker::{RenderGeneration, RenderRequest, RenderSubmitError};
use crate::static_preview::StaticPreviewEvidence;
use crate::{CommandId, DocumentCommandRequest, InputPhase, NormalizedInput};

use super::lifecycle::LifecycleSmokeOutcome;
use super::MotoliiApp;

impl MotoliiApp {
    pub(super) fn process_document_edit(&mut self, ctx: &egui::Context) {
        let Some(document_runtime) = &mut self.document_runtime else {
            return;
        };
        match document_runtime.process_next(
            &mut self.document_queue,
            self.primary,
            self.projection_generation,
        ) {
            Ok(Some(published)) => {
                if let Err(error) = self.publish_document_snapshot(&published) {
                    self.record_smoke_failure(error.to_string());
                    self.record_document_failure(error);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                ctx.request_repaint();
            }
            Ok(None) => {}
            Err(error) => {
                let reason = error.to_string();
                self.record_document_failure(DocumentEditFailure::Runtime(error));
                if self.document_smoke.is_some() {
                    self.record_smoke_failure(reason);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }

    fn publish_document_snapshot(
        &mut self,
        published: &PublishedDocument,
    ) -> Result<(), DocumentEditFailure> {
        adopt_published_document(
            &mut self.current_document,
            &mut self.primary,
            &mut self.projection_generation,
            published,
        );
        let generation = self.render_client.submit(RenderRequest {
            document: Arc::clone(&self.current_document),
            data_tracks: Arc::clone(&self.render_request_template.data_tracks),
            evaluation_time: self.render_request_template.evaluation_time,
            desc: self.render_request_template.desc,
            quality: self.render_request_template.quality,
        })?;
        if let Some(smoke) = &mut self.document_smoke {
            smoke
                .observe(
                    published.kind,
                    published.revision,
                    &self.current_document,
                    generation,
                    &mut self.document_queue,
                )
                .map_err(DocumentEditFailure::Smoke)?;
        }
        Ok(())
    }

    pub(super) fn advance_document_smoke(&mut self, ctx: &egui::Context) -> bool {
        let Some(smoke) = &mut self.document_smoke else {
            return false;
        };
        if !smoke.dispatched {
            let initial_ready = self.latest_projection.last_displayed_generation
                == self.render_client.latest_accepted_generation();
            if initial_ready {
                let output = self
                    .input_router
                    .route(NormalizedInput::Command {
                        phase: InputPhase::Click,
                        id: CommandId::try_new("motolii.edit.delete_targeted_items")
                            .expect("built-in command ID"),
                    })
                    .expect("built-in command registry");
                let request = smoke.request.take();
                if let Err(error) = self.document_queue.push_prepared(output, request) {
                    self.record_document_failure(DocumentEditFailure::Dispatch(error));
                    self.record_smoke_failure(error.to_string());
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    return true;
                }
                smoke.dispatched = true;
                ctx.request_repaint();
            } else if Instant::now() >= smoke.deadline {
                self.record_smoke_failure(
                    "initial preview was not displayed before U2b-1 dispatch".into(),
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else {
                ctx.request_repaint_after(smoke.deadline.saturating_duration_since(Instant::now()));
            }
            return true;
        }

        if let Some(expected_generation) = smoke.expected_redo_generation {
            if self.latest_projection.last_displayed_generation == Some(expected_generation) {
                let evidence = self.preview.invariant_evidence();
                if evidence.slot.slot_id == smoke.baseline.slot.slot_id
                    && evidence.slot.registration_count == smoke.baseline.slot.registration_count
                {
                    eprintln!(
                        "U2B1_DOCUMENT passed slot={} registrations={} generation={} revisions=1",
                        evidence.slot.slot_id,
                        evidence.slot.registration_count,
                        expected_generation.get()
                    );
                    if let Ok(mut outcome) = self.smoke_outcome.lock() {
                        *outcome = LifecycleSmokeOutcome::Passed;
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    return true;
                }
            }
        }
        if Instant::now() >= smoke.deadline {
            self.record_smoke_failure("U2b-1 apply snapshot was not displayed".into());
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return true;
        }
        ctx.request_repaint_after(smoke.deadline.saturating_duration_since(Instant::now()));
        true
    }

    fn record_document_failure(&mut self, error: DocumentEditFailure) {
        eprintln!("U2B1_DOCUMENT_REJECT error={error}");
        self.document_failure = Some(error);
    }
}

fn adopt_published_document(
    current_document: &mut Arc<motolii_doc::Document>,
    primary: &mut Option<motolii_doc::LayerId>,
    projection_generation: &mut u64,
    published: &PublishedDocument,
) {
    *current_document = Arc::clone(&published.snapshot);
    *primary = published.primary;
    *projection_generation = published.projection_generation;
}

#[derive(Debug, thiserror::Error)]
pub(super) enum DocumentEditFailure {
    #[error(transparent)]
    Dispatch(DocumentEditDispatchError),
    #[error(transparent)]
    Runtime(DocumentEditRuntimeError),
    #[error(transparent)]
    Submit(#[from] RenderSubmitError),
    #[error(transparent)]
    Smoke(#[from] DocumentEditSmokeError),
}

pub(super) struct DocumentEditSmoke {
    baseline: StaticPreviewEvidence,
    request: Option<DocumentCommandRequest>,
    dispatched: bool,
    applied_json: Option<Vec<u8>>,
    expected_redo_generation: Option<RenderGeneration>,
    deadline: Instant,
}

impl DocumentEditSmoke {
    pub(super) fn new(baseline: StaticPreviewEvidence, request: DocumentCommandRequest) -> Self {
        Self {
            baseline,
            request: Some(request),
            dispatched: false,
            applied_json: None,
            expected_redo_generation: None,
            deadline: Instant::now() + Duration::from_secs(5),
        }
    }

    fn observe(
        &mut self,
        kind: DocumentEditActionKind,
        revision: u64,
        snapshot: &motolii_doc::Document,
        generation: RenderGeneration,
        _queue: &mut DocumentEditQueue,
    ) -> Result<(), DocumentEditSmokeError> {
        let json = serde_json::to_vec(snapshot)?;
        match (kind, revision) {
            (DocumentEditActionKind::Apply, 1) => {
                if json == self.baseline.document_json.as_bytes() {
                    return Err(DocumentEditSmokeError::ApplyUnchanged);
                }
                self.applied_json = Some(json);
                self.expected_redo_generation = Some(generation);
            }
            _ => return Err(DocumentEditSmokeError::UnexpectedOrder),
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
enum DocumentEditSmokeError {
    #[error(transparent)]
    Serialize(#[from] serde_json::Error),
    #[error("U2b-1 apply did not change Document")]
    ApplyUnchanged,
    #[error("U2b-1 action order or revision was unexpected")]
    UnexpectedOrder,
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_doc::Document;

    #[test]
    fn adoption_replaces_document_primary_and_generation_together() {
        let mut published_document = Document::new_current();
        let published_layer = published_document.layers.allocate("published").unwrap();
        let mut current_document = Arc::new(Document::new_current());
        let mut primary = None;
        let mut projection_generation = 0;
        let published = PublishedDocument {
            kind: DocumentEditActionKind::Apply,
            revision: 7,
            snapshot: Arc::new(published_document),
            primary: Some(published_layer),
            projection_generation: 9,
            created_effect_use: None,
        };

        adopt_published_document(
            &mut current_document,
            &mut primary,
            &mut projection_generation,
            &published,
        );

        assert!(Arc::ptr_eq(&current_document, &published.snapshot));
        assert_eq!(primary, Some(published_layer));
        assert_eq!(projection_generation, 9);
    }
}
