//! 事前準備済みの静止native textureだけを中央Stageへ投影する。

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use egui_tiles::{Behavior, EditAction, Tile, TileId, Tiles, UiResponse};
use motolii_gpu::GpuCtx;

use crate::command_registry::builtin_command_registry;
use crate::display_slot::DisplaySlotError;
use crate::document_edit_runtime::{
    DocumentEditActionKind, DocumentEditDispatchError, DocumentEditQueue, DocumentEditRuntime,
    DocumentEditRuntimeError, PublishedDocument,
};
use crate::input_router::{ImeGateState, InputPhase, InputRouter, NormalizedInput};
use crate::layout::{LayoutAction, LayoutConstraints, PanelRole, SeparatorAction};
use crate::layout_authority::{LayoutAuthority, RuntimeFrameEdit};
use crate::layout_runtime::{RuntimeLayout, RuntimeSeparator};
use crate::layout_runtime_adapter::{
    read_layout_cancel, read_safety_interrupt, read_separator_action,
};
use crate::render_worker::{
    RenderGeneration, RenderRequest, RenderSubmitError, RenderWorkerClient, RenderWorkerError,
    RepaintSignalEpoch, RepaintSignalRegistrationError,
};
use crate::static_preview::{StaticPreview, StaticPreviewEvidence};
use crate::{CommandId, DocumentCommandRequest};

const DEFAULT_STAGE_MIN_POINTS: f32 = 320.0;

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
enum PreviewProjectionFailure {
    #[error(transparent)]
    Worker(RenderWorkerError),
    #[error(transparent)]
    Display(DisplaySlotError),
    #[error(transparent)]
    RepaintSignal(RepaintSignalRegistrationError),
}

#[derive(Debug, Default)]
struct LatestResultProjection {
    last_displayed_generation: Option<RenderGeneration>,
}

impl LatestResultProjection {
    fn accepts(
        &self,
        result_generation: RenderGeneration,
        latest_accepted_generation: Option<RenderGeneration>,
    ) -> bool {
        Some(result_generation) == latest_accepted_generation
            && self
                .last_displayed_generation
                .is_none_or(|displayed| result_generation > displayed)
    }

    fn commit(&mut self, generation: RenderGeneration) {
        self.last_displayed_generation = Some(generation);
    }
}

pub(crate) struct MotoliiApp {
    preview: Arc<StaticPreview>,
    texture_id: egui::TextureId,
    projection: StaticViewportProjection,
    paint_count: u32,
    smoke: Option<LifecycleSmoke>,
    smoke_outcome: Arc<Mutex<LifecycleSmokeOutcome>>,
    layout_authority: LayoutAuthority,
    input_router: InputRouter,
    ime_gate: ImeGateState,
    layout_evidence_logged: bool,
    layout_failure: Option<String>,
    gpu: Arc<GpuCtx>,
    render_client: RenderWorkerClient,
    repaint_context: egui::Context,
    last_handled_signal_failure: Option<RepaintSignalEpoch>,
    latest_projection: LatestResultProjection,
    preview_failure: Option<PreviewProjectionFailure>,
    latest_smoke: Option<LatestPreviewSmoke>,
    document_runtime: DocumentEditRuntime,
    document_queue: DocumentEditQueue,
    current_document: Arc<motolii_doc::Document>,
    render_request_template: RenderRequest,
    document_failure: Option<DocumentEditFailure>,
    document_smoke: Option<DocumentEditSmoke>,
}

pub(crate) struct AppPreviewRuntime {
    pub(crate) preview: Arc<StaticPreview>,
    pub(crate) gpu: Arc<GpuCtx>,
    pub(crate) render_client: RenderWorkerClient,
    pub(crate) initial_request: RenderRequest,
    pub(crate) document_runtime: DocumentEditRuntime,
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
        let current_document = document_runtime.snapshot();
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
            current_document,
            render_request_template,
            document_failure: None,
            document_smoke: smoke
                .document_edit
                .map(|request| DocumentEditSmoke::new(evidence, request)),
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
        self.recover_repaint_signal();
        self.drain_latest_result();
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

        let mut requested_action = None;
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("View", |ui| {
                for role in PanelRole::AUXILIARY {
                    let visible = self.layout_authority.intent().is_visible(role);
                    let (_, action) = view_role_button(ui, role, visible);
                    if let Some(action) = action {
                        requested_action = Some(action);
                        ui.close();
                    }
                }
                if ui.button("Reset layout").clicked() {
                    requested_action = Some(LayoutAction::ResetPreset);
                    ui.close();
                }
            });
        });

        egui::Panel::bottom("motolii-status")
            .resizable(false)
            .show(ui, |ui| {
                ui.label("Status");
            });

        egui::CentralPanel::default().show(ui, |ui| {
            let constraints = layout_constraints(ui.available_width());
            if let Some(action) = requested_action.take() {
                if let Err(error) = self.layout_authority.apply(action, constraints) {
                    self.observe_layout_failure(error);
                }
            }

            let safety = read_safety_interrupt(ui);
            if let Some(safety) = safety {
                if let Err(error) = self
                    .input_router
                    .route(NormalizedInput::SafetyInterrupt(safety))
                {
                    self.observe_layout_failure_message(error.to_string());
                }
            }
            let cancel_runtime_frame = safety.is_some()
                || read_layout_cancel(ui, self.layout_authority.gesture_in_flight(), self.ime_gate);

            let mut behavior = PanelBehavior {
                preview: &self.preview,
                texture_id: self.texture_id,
                edits: Vec::new(),
                visibility_edited: false,
            };
            self.layout_authority
                .runtime_mut()
                .tree_mut()
                .ui(&mut behavior, ui);
            if !self.layout_evidence_logged {
                eprintln!(
                    "U1A2_LAYOUT signature={}",
                    self.layout_authority.intent().canonical_signature()
                );
                self.layout_evidence_logged = true;
            }

            let edits = behavior.edits;
            let runtime_edit = if edits
                .iter()
                .any(|edit| matches!(edit, EditAction::TileResized | EditAction::TileDragged))
            {
                RuntimeFrameEdit::Continuous
            } else if !edits.is_empty() || behavior.visibility_edited {
                RuntimeFrameEdit::Commit
            } else {
                RuntimeFrameEdit::None
            };
            let gesture_finished =
                edits.contains(&EditAction::TileDropped) || ui.ctx().drag_stopped_id().is_some();

            if cancel_runtime_frame {
                if let Err(error) = self.layout_authority.reconcile_runtime_frame(
                    true,
                    runtime_edit,
                    gesture_finished,
                    constraints,
                ) {
                    self.observe_layout_failure(error);
                }
                return;
            }

            let separator_actions =
                collect_separator_actions(ui, self.layout_authority.runtime(), self.ime_gate);
            let separator_consumed_runtime_edit = !separator_actions.is_empty();
            for (separator, action) in separator_actions {
                if action == SeparatorAction::Cancel {
                    if let Err(error) = self.layout_authority.reconcile_runtime_frame(
                        true,
                        RuntimeFrameEdit::None,
                        false,
                        constraints,
                    ) {
                        self.observe_layout_failure(error);
                    }
                    continue;
                }
                if let Err(error) = self.layout_authority.apply(
                    LayoutAction::Separator {
                        path: separator.path,
                        boundary: separator.boundary,
                        action,
                    },
                    constraints,
                ) {
                    self.observe_layout_failure(error);
                }
            }
            if let Err(error) = self.layout_authority.reconcile_runtime_frame(
                false,
                runtime_edit_after_separator_action(runtime_edit, separator_consumed_runtime_edit),
                gesture_finished,
                constraints,
            ) {
                self.observe_layout_failure(error);
            }
        });
    }
}

fn runtime_edit_after_separator_action(
    runtime_edit: RuntimeFrameEdit,
    separator_action_consumed: bool,
) -> RuntimeFrameEdit {
    if separator_action_consumed {
        RuntimeFrameEdit::None
    } else {
        runtime_edit
    }
}

impl MotoliiApp {
    fn process_document_edit(&mut self, ctx: &egui::Context) {
        match self.document_runtime.process_next(&mut self.document_queue) {
            Ok(Some(published)) => {
                if let Err(error) = self.publish_document_snapshot(published) {
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
        published: PublishedDocument,
    ) -> Result<(), DocumentEditFailure> {
        self.current_document = published.snapshot;
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

    fn advance_document_smoke(&mut self, ctx: &egui::Context) -> bool {
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
                        "U2B1_DOCUMENT passed slot={} registrations={} generation={} revisions=1,2,3",
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
            self.record_smoke_failure("U2b-1 edit/Undo/Redo snapshot was not displayed".into());
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return true;
        }
        ctx.request_repaint_after(smoke.deadline.saturating_duration_since(Instant::now()));
        true
    }

    fn recover_repaint_signal(&mut self) {
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

    fn drain_latest_result(&mut self) {
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
        if let Err(error) = self.preview.slot().copy(&self.gpu, &rendered) {
            self.record_preview_failure(PreviewProjectionFailure::Display(error));
            return;
        }
        self.latest_projection.commit(result.generation);
    }

    fn advance_latest_smoke(&mut self, ctx: &egui::Context) -> bool {
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

    fn observe_layout_failure(&mut self, error: crate::layout::LayoutError) {
        self.observe_layout_failure_message(error.to_string());
    }

    fn record_preview_failure(&mut self, error: PreviewProjectionFailure) {
        eprintln!("U1B2_PREVIEW_REJECT error={error}");
        self.preview_failure = Some(error);
    }

    fn record_document_failure(&mut self, error: DocumentEditFailure) {
        eprintln!("U2B1_DOCUMENT_REJECT error={error}");
        self.document_failure = Some(error);
    }

    fn observe_layout_failure_message(&mut self, message: String) {
        eprintln!("U1A2_LAYOUT_REJECT error={message}");
        self.layout_failure = Some(message);
    }
}

fn register_repaint_signal(
    client: &RenderWorkerClient,
    context: &egui::Context,
) -> Result<(), RepaintSignalRegistrationError> {
    let context = context.clone();
    client
        .register_repaint_signal(Arc::new(move || context.request_repaint()))
        .map(|_| ())
}

struct LatestPreviewSmoke {
    baseline: StaticPreviewEvidence,
    expected_generation: RenderGeneration,
    deadline: Instant,
}

#[derive(Debug, thiserror::Error)]
enum DocumentEditFailure {
    #[error(transparent)]
    Dispatch(DocumentEditDispatchError),
    #[error(transparent)]
    Runtime(DocumentEditRuntimeError),
    #[error(transparent)]
    Submit(#[from] RenderSubmitError),
    #[error(transparent)]
    Smoke(#[from] DocumentEditSmokeError),
}

struct DocumentEditSmoke {
    baseline: StaticPreviewEvidence,
    request: Option<DocumentCommandRequest>,
    dispatched: bool,
    applied_json: Option<Vec<u8>>,
    expected_redo_generation: Option<RenderGeneration>,
    deadline: Instant,
}

impl DocumentEditSmoke {
    fn new(baseline: StaticPreviewEvidence, request: DocumentCommandRequest) -> Self {
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
        queue: &mut DocumentEditQueue,
    ) -> Result<(), DocumentEditSmokeError> {
        let json = serde_json::to_vec(snapshot)?;
        match (kind, revision) {
            (DocumentEditActionKind::Apply, 1) => {
                if json == self.baseline.document_json.as_bytes() {
                    return Err(DocumentEditSmokeError::ApplyUnchanged);
                }
                self.applied_json = Some(json);
                queue.push_undo();
            }
            (DocumentEditActionKind::Undo, 2) => {
                if json != self.baseline.document_json.as_bytes() {
                    return Err(DocumentEditSmokeError::UndoMismatch);
                }
                queue.push_redo();
            }
            (DocumentEditActionKind::Redo, 3) => {
                if self.applied_json.as_deref() != Some(json.as_slice()) {
                    return Err(DocumentEditSmokeError::RedoMismatch);
                }
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
    #[error("U2b-1 Undo did not restore the initial Document")]
    UndoMismatch,
    #[error("U2b-1 Redo did not restore the applied Document")]
    RedoMismatch,
    #[error("U2b-1 action order or revision was unexpected")]
    UnexpectedOrder,
}

impl LatestPreviewSmoke {
    fn new(baseline: StaticPreviewEvidence, expected_generation: RenderGeneration) -> Self {
        Self {
            baseline,
            expected_generation,
            deadline: Instant::now() + Duration::from_secs(5),
        }
    }
}

struct PanelBehavior<'a> {
    preview: &'a StaticPreview,
    texture_id: egui::TextureId,
    edits: Vec<EditAction>,
    visibility_edited: bool,
}

impl Behavior<PanelRole> for PanelBehavior<'_> {
    fn pane_ui(&mut self, ui: &mut egui::Ui, _tile_id: TileId, pane: &mut PanelRole) -> UiResponse {
        match pane {
            PanelRole::Stage => paint_stage(ui, self.preview, self.texture_id),
            role => {
                let response = ui.add(egui::Label::new(role.title()).sense(egui::Sense::drag()));
                if response.drag_started() {
                    return UiResponse::DragStarted;
                }
            }
        }
        UiResponse::None
    }

    fn tab_title_for_pane(&mut self, pane: &PanelRole) -> egui::WidgetText {
        pane.title().into()
    }

    fn is_tab_closable(&self, tiles: &Tiles<PanelRole>, tile_id: TileId) -> bool {
        tiles
            .get_pane(&tile_id)
            .is_some_and(|role| role.is_auxiliary())
    }

    fn on_tab_close(&mut self, tiles: &mut Tiles<PanelRole>, tile_id: TileId) -> bool {
        if tiles
            .get_pane(&tile_id)
            .is_some_and(|role| role.is_auxiliary())
        {
            tiles.set_visible(tile_id, false);
            self.visibility_edited = true;
        }
        false
    }

    fn is_tile_draggable(&self, tiles: &Tiles<PanelRole>, tile_id: TileId) -> bool {
        matches!(
            tiles.get(tile_id),
            Some(Tile::Pane(role)) if role.is_auxiliary()
        )
    }

    fn on_edit(&mut self, edit_action: EditAction) {
        self.edits.push(edit_action);
    }
}

fn paint_stage(ui: &mut egui::Ui, preview: &StaticPreview, texture_id: egui::TextureId) {
    let desc = preview.slot().desc();
    let source_size = egui::vec2(desc.width as f32, desc.height as f32);
    let target_size = fit_inside(source_size, ui.available_size());
    ui.centered_and_justified(|ui| {
        ui.push_id("motolii-stage-viewport", |ui| {
            ui.add(
                egui::Image::from_texture((texture_id, source_size)).fit_to_exact_size(target_size),
            );
        });
    });
}

fn layout_constraints(viewport_width: f32) -> LayoutConstraints {
    let safe_width = viewport_width.max(2.0);
    LayoutConstraints {
        viewport_width: safe_width,
        stage_min_width: DEFAULT_STAGE_MIN_POINTS.min(safe_width * 0.75),
    }
}

fn view_role_button(
    ui: &mut egui::Ui,
    role: PanelRole,
    visible: bool,
) -> (egui::Response, Option<LayoutAction>) {
    let response = ui.button(if visible {
        format!("Hide {}", role.title())
    } else {
        format!("Restore {}", role.title())
    });
    let action = response.clicked().then_some(if visible {
        LayoutAction::Hide(role)
    } else {
        LayoutAction::Restore(role)
    });
    (response, action)
}

fn collect_separator_actions(
    ui: &mut egui::Ui,
    runtime: &RuntimeLayout,
    ime_gate: ImeGateState,
) -> Vec<(RuntimeSeparator, SeparatorAction)> {
    let mut actions = Vec::new();
    for separator in runtime.separators().iter().cloned() {
        let Some(response) = runtime.separator_response(ui, &separator) else {
            continue;
        };
        if let Some(action) = read_separator_action(ui, &response, separator.axis, ime_gate) {
            actions.push((separator, action));
        }
    }
    actions
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
    AwaitInitialPreview,
    Resize,
    Minimize,
    Restore,
    AwaitRestoredPaint,
}

impl LifecycleSmoke {
    fn new() -> Self {
        Self {
            phase: SmokePhase::AwaitInitialPreview,
            deadline: Instant::now() + Duration::from_secs(5),
            restore_paint_count: 0,
        }
    }

    fn advance(
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

    #[test]
    fn hidden_role_restores_through_the_product_view_button_with_enter() {
        let constraints = layout_constraints(1_000.0);
        let mut authority = LayoutAuthority::built_in().unwrap();
        authority
            .apply(LayoutAction::Hide(PanelRole::Browser), constraints)
            .unwrap();
        let context = egui::Context::default();
        let _ = context.run_ui(Default::default(), |ui| {
            let (response, action) = view_role_button(ui, PanelRole::Browser, false);
            assert!(action.is_none());
            response.request_focus();
        });
        let input = egui::RawInput {
            events: vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: Some(egui::Key::Enter),
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            ..Default::default()
        };
        let _ = context.run_ui(input, |ui| {
            let (_, action) = view_role_button(ui, PanelRole::Browser, false);
            authority.apply(action.unwrap(), constraints).unwrap();
        });
        assert!(authority.intent().is_visible(PanelRole::Browser));
    }

    #[test]
    fn native_double_click_reset_suppresses_tiles_mean_proposal() {
        assert_eq!(
            runtime_edit_after_separator_action(RuntimeFrameEdit::Continuous, true),
            RuntimeFrameEdit::None
        );
        assert_eq!(
            runtime_edit_after_separator_action(RuntimeFrameEdit::Continuous, false),
            RuntimeFrameEdit::Continuous
        );
    }

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
