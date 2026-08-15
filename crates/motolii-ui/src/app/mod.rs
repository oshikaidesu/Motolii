//! 事前準備済みの静止native textureだけを中央Stageへ投影する。
//!
//! 責任は各moduleが持ち、ここは組み立てだけを行う。

mod browser;
mod document;
mod layout;
mod lifecycle;
mod preview;
mod timeline;

use std::sync::{Arc, Mutex};

use motolii_gpu::GpuCtx;

use crate::browser_host::BrowserPlaceIntent;
use crate::browser_host_runtime::{BrowserHostRuntime, BrowserHostRuntimeError};
use crate::builtin_command_registry;
use crate::document_edit_runtime::{DocumentEditQueue, DocumentEditRuntime};
use crate::layout_authority::LayoutAuthority;
use crate::render_worker::{
    RenderRequest, RenderSubmitError, RenderWorkerClient, RepaintSignalRegistrationError,
};
use crate::static_preview::StaticPreview;
use crate::timeline_move_gesture::TimelineMoveGesture;
use crate::timeline_projection::TimelineProjection;
use crate::timeline_trim_gesture::TimelineTrimGesture;
use crate::{DocumentCommandRequest, ImeGateState, InputRouter};

use document::{DocumentEditFailure, DocumentEditSmoke};
use lifecycle::{
    LatestPreviewSmoke, LatestResultProjection, LifecycleSmoke, PreviewProjectionFailure,
};
use preview::register_repaint_signal;
use timeline::EguiKeyDrag;

pub(crate) use crate::canonical_drop::canonical_drop_from_ndc;
pub(crate) use lifecycle::{
    LifecycleInvariantError, LifecycleSmokeOutcome, ShellLifecycleInput, StaticViewportProjection,
};

#[derive(Debug, thiserror::Error)]
pub(crate) enum AppConstructionError {
    #[error("wgpu render state is not available")]
    MissingWgpuRenderState,
    #[error(transparent)]
    CommandRegistry(#[from] crate::CommandRegistryError),
    #[error(transparent)]
    Layout(#[from] crate::layout::LayoutError),
    #[error(transparent)]
    Submit(#[from] RenderSubmitError),
    #[error(transparent)]
    RepaintSignal(#[from] RepaintSignalRegistrationError),
    #[error("native window is not available for the Browser Host")]
    MissingNativeWindow,
    #[error(transparent)]
    BrowserHost(#[from] BrowserHostRuntimeError),
}

pub(crate) struct MotoliiApp {
    pub(super) preview: Arc<StaticPreview>,
    pub(super) texture_id: egui::TextureId,
    pub(super) projection: StaticViewportProjection,
    pub(super) paint_count: u32,
    pub(super) smoke: Option<LifecycleSmoke>,
    pub(super) smoke_outcome: Arc<Mutex<LifecycleSmokeOutcome>>,
    pub(super) layout_authority: LayoutAuthority,
    pub(super) input_router: InputRouter,
    pub(super) ime_gate: ImeGateState,
    pub(super) layout_evidence_logged: bool,
    pub(super) layout_failure: Option<String>,
    pub(super) gpu: Arc<GpuCtx>,
    pub(super) render_client: RenderWorkerClient,
    pub(super) repaint_context: egui::Context,
    pub(super) last_handled_signal_failure: Option<crate::render_worker::RepaintSignalEpoch>,
    pub(super) latest_projection: LatestResultProjection,
    pub(super) preview_failure: Option<PreviewProjectionFailure>,
    pub(super) latest_smoke: Option<LatestPreviewSmoke>,
    pub(super) document_runtime: Option<DocumentEditRuntime>,
    pub(super) document_queue: DocumentEditQueue,
    pub(super) primary: Option<motolii_doc::LayerId>,
    pub(super) projection_generation: u64,
    pub(super) current_document: Arc<motolii_doc::Document>,
    pub(super) render_request_template: RenderRequest,
    pub(super) document_failure: Option<DocumentEditFailure>,
    pub(super) document_smoke: Option<DocumentEditSmoke>,
    pub(super) browser_host: Option<BrowserHostRuntime>,
    pub(super) browser_host_failure: Option<String>,
    pub(super) browser_place_generation: u64,
    pub(super) active_browser_place: Option<BrowserPlaceIntent>,
    pub(super) latest_camera: Option<motolii_core::CompCamera>,
    pub(super) timeline_move: Option<TimelineMoveGesture>,
    pub(super) timeline_trim: Option<TimelineTrimGesture>,
    pub(super) timeline_key_drag: Option<EguiKeyDrag>,
    // ドラッグ中は未確定previewを描き、Documentはreleaseまで触らない。
    pub(super) timeline_preview: Option<TimelineProjection>,
}

const INITIAL_PRIMARY: Option<motolii_doc::LayerId> = None;
const INITIAL_PROJECTION_GENERATION: u64 = 0;

pub(crate) struct AppPreviewRuntime {
    pub(crate) preview: Arc<StaticPreview>,
    pub(crate) gpu: Arc<GpuCtx>,
    pub(crate) render_client: RenderWorkerClient,
    pub(crate) initial_request: RenderRequest,
    pub(crate) document_runtime: Option<DocumentEditRuntime>,
}

pub(crate) struct AppSmokeConfig {
    pub(crate) lifecycle: bool,
    pub(crate) latest_preview: bool,
    pub(crate) document_edit: Option<DocumentCommandRequest>,
    pub(crate) outcome: Arc<Mutex<LifecycleSmokeOutcome>>,
}

impl MotoliiApp {
    pub(crate) fn new(
        cc: &eframe::CreationContext<'_>,
        runtime: AppPreviewRuntime,
        smoke: AppSmokeConfig,
    ) -> Result<Self, AppConstructionError> {
        let AppPreviewRuntime {
            preview,
            gpu,
            render_client,
            initial_request,
            document_runtime,
        } = runtime;
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
        let repaint_context = cc.egui_ctx.clone();
        register_repaint_signal(&render_client, &repaint_context)?;
        let render_request_template = initial_request.clone();
        let current_document = match &document_runtime {
            Some(runtime) => runtime.snapshot(),
            None => Arc::clone(&initial_request.document),
        };
        let browser_host = if document_runtime.is_some() {
            let window = cc
                .winit_window()
                .ok_or(AppConstructionError::MissingNativeWindow)?;
            let wake_context = repaint_context.clone();
            let instance_epoch = BrowserHostRuntime::fresh_instance_epoch()?;
            Some(BrowserHostRuntime::new(
                window,
                instance_epoch,
                BrowserHostRuntime::built_in_rectangle_source(instance_epoch),
                Arc::new(move || wake_context.request_repaint()),
                Arc::new(|_| {}),
            )?)
        } else {
            None
        };
        let initial_generation = render_client.submit(initial_request)?;
        let evidence = preview.invariant_evidence();
        eprintln!(
            "U1A1_REGISTER slot={} texture={texture_id:?} registrations={} copies={} renders={}",
            evidence.slot.slot_id,
            evidence.slot.registration_count,
            evidence.slot.copy_count,
            evidence.render_count
        );
        let projection = StaticViewportProjection::new(&preview);
        let layout_authority = LayoutAuthority::built_in()?;
        Ok(Self {
            preview,
            texture_id,
            projection,
            paint_count: 0,
            smoke: smoke.lifecycle.then(LifecycleSmoke::new),
            smoke_outcome: smoke.outcome,
            layout_authority,
            input_router: InputRouter::new(builtin_command_registry()?),
            ime_gate: ImeGateState::Inactive,
            layout_evidence_logged: false,
            layout_failure: None,
            gpu,
            render_client,
            repaint_context,
            last_handled_signal_failure: None,
            latest_projection: LatestResultProjection::default(),
            preview_failure: None,
            latest_smoke: smoke
                .latest_preview
                .then(|| LatestPreviewSmoke::new(evidence.clone(), initial_generation)),
            document_runtime,
            document_queue: DocumentEditQueue::default(),
            primary: INITIAL_PRIMARY,
            projection_generation: INITIAL_PROJECTION_GENERATION,
            current_document,
            render_request_template,
            document_failure: None,
            document_smoke: smoke
                .document_edit
                .map(|request| DocumentEditSmoke::new(evidence, request)),
            browser_host,
            browser_host_failure: None,
            browser_place_generation: 0,
            active_browser_place: None,
            latest_camera: None,
            timeline_move: None,
            timeline_trim: None,
            timeline_key_drag: None,
            timeline_preview: None,
        })
    }

    pub(super) fn record_smoke_failure(&self, reason: String) {
        if let Ok(mut outcome) = self.smoke_outcome.lock() {
            *outcome = LifecycleSmokeOutcome::Failed(reason);
        }
    }
}

impl eframe::App for MotoliiApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.recover_repaint_signal();
        self.drain_latest_result();
        self.begin_browser_place();
        self.poll_browser_place_pointer();
        if self.advance_latest_smoke(ctx) {
            return;
        }
        self.process_document_edit(ctx);
        if self.advance_document_smoke(ctx) {
            return;
        }
        let Some(smoke) = &mut self.smoke else {
            return;
        };
        match smoke.advance(
            ctx,
            self.paint_count,
            self.texture_id,
            self.latest_projection.last_displayed_generation
                == self.render_client.latest_accepted_generation(),
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
        self.paint_shell(ui);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_selection_fields_are_none_and_zero() {
        assert_eq!(INITIAL_PRIMARY, None);
        assert_eq!(INITIAL_PROJECTION_GENERATION, 0);
    }
}
