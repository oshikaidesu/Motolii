//! macOS通常project sessionのdirect native Surface Host。

use std::borrow::Cow;
use std::sync::Arc;
use std::time::{Duration, Instant};

use motolii_core::{Quality, RationalTime};
use motolii_doc::EvaluationTime;
use motolii_eval::DataTracks;
use motolii_gpu::GpuCtx;
use winit::dpi::LogicalSize;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::window::Window;

use crate::app::canonical_drop_from_ndc;
use crate::browser_host::BrowserPlaceIntent;
use crate::browser_host_runtime::{
    BrowserFocusTarget, BrowserHostRuntime, BrowserHostRuntimeError, BrowserLifecycleEvent,
};
use crate::document_edit_runtime::{
    DocumentEditQueue, DocumentEditRuntime, DocumentEditRuntimeError, PlaceRectangleRequest,
};
use crate::host_pointer_capture::{HostPointerCancel, HostPointerCandidate};
use crate::native_host_layout::{NativeHostLayout, PhysicalRect};
use crate::render_worker::{
    RenderGeneration, RenderRequest, RenderWorker, RenderWorkerClient, RenderWorkerError,
};
use crate::static_preview::{bootstrap_frame_desc, prepare_in_setup_worker, StaticPreview};

#[derive(Debug, Clone, Copy)]
pub(crate) enum ProductEvent {
    Wake,
    BrowserLifecycle(BrowserLifecycleEvent),
}

pub(crate) fn run(document_runtime: DocumentEditRuntime) -> Result<(), ProductRuntimeError> {
    let document = document_runtime.snapshot();
    let (gpu, parts) = GpuCtx::new_for_ui()?;
    let parts = ProductGpuParts {
        instance: parts.instance,
        adapter: parts.adapter,
        device: parts.device,
    };
    let gpu = Arc::new(gpu);
    let preview = Arc::new(prepare_in_setup_worker(
        Arc::clone(&gpu),
        Arc::clone(&document),
        bootstrap_frame_desc()?,
    )?);
    let mut render_worker = RenderWorker::spawn(Arc::clone(&gpu))?;
    let render_client = render_worker.client();
    let event_loop = EventLoop::<ProductEvent>::with_user_event().build()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let proxy = event_loop.create_proxy();
    let wake_proxy = proxy.clone();
    render_client.register_repaint_signal(Arc::new(move || {
        let _ = wake_proxy.send_event(ProductEvent::Wake);
    }))?;
    let render_request_template = RenderRequest {
        document,
        data_tracks: Arc::new(DataTracks::new()),
        evaluation_time: EvaluationTime::new(RationalTime::ZERO),
        desc: bootstrap_frame_desc()?,
        quality: Quality::DRAFT,
    };
    let mut app = ProductApp::new(
        gpu,
        parts,
        preview,
        document_runtime,
        render_client,
        render_request_template,
        proxy,
    );
    let run_result = event_loop.run_app(&mut app);
    render_worker.close();
    let join_result = render_worker.join();
    run_result?;
    join_result?;
    if let Some(error) = app.failure {
        return Err(ProductRuntimeError::Runtime(error));
    }
    Ok(())
}

pub(crate) struct ProductApp {
    // surface → WebView → Windowの順にdropし、AppKit backingを先に失わない。
    gfx: Option<ProductSurface>,
    browser: Option<BrowserHostRuntime>,
    window: Option<Arc<Window>>,
    gpu: Arc<GpuCtx>,
    parts: Option<ProductGpuParts>,
    preview: Arc<StaticPreview>,
    render_client: RenderWorkerClient,
    render_request_template: RenderRequest,
    stage_projection: ProductStageProjection,
    displayed_camera: motolii_core::CompCamera,
    document_runtime: DocumentEditRuntime,
    document_queue: DocumentEditQueue,
    primary: Option<motolii_doc::LayerId>,
    projection_generation: u64,
    current_document: Arc<motolii_doc::Document>,
    proxy: EventLoopProxy<ProductEvent>,
    layout: Option<NativeHostLayout>,
    next_layout_epoch: u64,
    browser_source: Option<BrowserPlaceIntent>,
    browser_lifecycle: Option<BrowserLifecycleCoordinator>,
    browser_focus_target: BrowserFocusTarget,
    next_place_generation: u64,
    active_place: Option<BrowserPlaceIntent>,
    place_preview: PlacePreviewPhase,
    terminal_admission: PlaceTerminalAdmission,
    terminal_delivery: PlaceTerminalDelivery,
    candidate_terminal: Option<ClassifiedPlaceTerminal>,
    admitted_terminal: Option<ClassifiedPlaceTerminal>,
    pending_stage_drop: Option<PendingStageDrop>,
    surface_retry_at: Option<Instant>,
    failure: Option<String>,
}

#[derive(Debug, Clone)]
struct PendingStageDrop {
    source: BrowserPlaceIntent,
    generation: u64,
    layout_epoch: u64,
    ndc: [f64; 2],
}

#[derive(Debug, Default)]
struct ProductStageProjection {
    last_displayed_generation: Option<RenderGeneration>,
}

impl ProductStageProjection {
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

#[derive(Debug, Default)]
struct PlaceTerminalDelivery {
    delivered_high_water: Option<u64>,
}

impl PlaceTerminalDelivery {
    fn deliver(&mut self, terminal: &ClassifiedPlaceTerminal) -> Option<PendingStageDrop> {
        if terminal.cause != PlaceTerminalCause::NoNonCommitCause
            || self
                .delivered_high_water
                .is_some_and(|high_water| terminal.generation <= high_water)
        {
            return None;
        }
        let (Some(layout_epoch), Some(ndc)) = (terminal.layout_epoch, terminal.stage_ndc) else {
            return None;
        };
        self.delivered_high_water = Some(terminal.generation);
        Some(PendingStageDrop {
            source: terminal.source.clone(),
            generation: terminal.generation,
            layout_epoch,
            ndc,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlaceTerminalCause {
    Escape,
    OutsideStage,
    CaptureLoss,
    NoNonCommitCause,
}

#[derive(Debug, Clone, PartialEq)]
struct ClassifiedPlaceTerminal {
    source: BrowserPlaceIntent,
    generation: u64,
    cause: PlaceTerminalCause,
    layout_epoch: Option<u64>,
    stage_ndc: Option<[f64; 2]>,
}

impl ClassifiedPlaceTerminal {
    fn released(
        source: BrowserPlaceIntent,
        generation: u64,
        position: [f64; 2],
        layout: NativeHostLayout,
    ) -> Self {
        let stage_ndc = layout.stage_ndc(position);
        Self {
            source,
            generation,
            cause: if stage_ndc.is_some() {
                PlaceTerminalCause::NoNonCommitCause
            } else {
                PlaceTerminalCause::OutsideStage
            },
            layout_epoch: Some(layout.epoch),
            stage_ndc,
        }
    }

    fn cancelled(source: BrowserPlaceIntent, generation: u64, reason: HostPointerCancel) -> Self {
        Self {
            source,
            generation,
            cause: match reason {
                HostPointerCancel::Escape => PlaceTerminalCause::Escape,
                HostPointerCancel::CaptureLost => PlaceTerminalCause::CaptureLoss,
            },
            layout_epoch: None,
            stage_ndc: None,
        }
    }
}

#[derive(Debug, Default)]
struct PlaceTerminalAdmission {
    active_generation: Option<u64>,
    retired_high_water: Option<u64>,
}

impl PlaceTerminalAdmission {
    fn begin(&mut self, generation: u64) -> bool {
        if self.active_generation.is_some()
            || self
                .retired_high_water
                .is_some_and(|high_water| generation <= high_water)
        {
            return false;
        }
        self.active_generation = Some(generation);
        true
    }

    fn admit(&mut self, terminal: &ClassifiedPlaceTerminal) -> bool {
        if self.active_generation != Some(terminal.generation) {
            return false;
        }
        self.active_generation = None;
        self.retired_high_water = Some(
            self.retired_high_water
                .map_or(terminal.generation, |high_water| {
                    high_water.max(terminal.generation)
                }),
        );
        terminal.cause == PlaceTerminalCause::NoNonCommitCause
    }

    fn retire_active(&mut self) {
        if let Some(generation) = self.active_generation.take() {
            self.retired_high_water = Some(
                self.retired_high_water
                    .map_or(generation, |high_water| high_water.max(generation)),
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct PlacePreviewProgress {
    source: BrowserPlaceIntent,
    generation: u64,
    layout_epoch: u64,
    stage_ndc: Option<[f64; 2]>,
}

#[derive(Debug, Default)]
struct PlacePreviewPhase {
    latest: Option<PlacePreviewProgress>,
}

impl PlacePreviewPhase {
    fn deliver(
        &mut self,
        source: &BrowserPlaceIntent,
        generation: u64,
        position: [f64; 2],
        layout: NativeHostLayout,
    ) {
        self.latest = Some(PlacePreviewProgress {
            source: source.clone(),
            generation,
            layout_epoch: layout.epoch,
            stage_ndc: layout.stage_ndc(position),
        });
    }

    fn clear(&mut self) {
        self.latest = None;
    }
}

impl ProductApp {
    fn new(
        gpu: Arc<GpuCtx>,
        parts: ProductGpuParts,
        preview: Arc<StaticPreview>,
        document_runtime: DocumentEditRuntime,
        render_client: RenderWorkerClient,
        render_request_template: RenderRequest,
        proxy: EventLoopProxy<ProductEvent>,
    ) -> Self {
        let displayed_camera = preview.camera();
        Self {
            gfx: None,
            browser: None,
            window: None,
            gpu,
            parts: Some(parts),
            preview,
            render_client,
            render_request_template,
            stage_projection: ProductStageProjection::default(),
            displayed_camera,
            current_document: document_runtime.snapshot(),
            document_runtime,
            document_queue: DocumentEditQueue::default(),
            primary: None,
            projection_generation: 0,
            proxy,
            layout: None,
            next_layout_epoch: 1,
            browser_source: None,
            browser_lifecycle: None,
            browser_focus_target: BrowserFocusTarget::Browser,
            next_place_generation: 1,
            active_place: None,
            place_preview: PlacePreviewPhase::default(),
            terminal_admission: PlaceTerminalAdmission::default(),
            terminal_delivery: PlaceTerminalDelivery::default(),
            candidate_terminal: None,
            admitted_terminal: None,
            pending_stage_drop: None,
            surface_retry_at: None,
            failure: None,
        }
    }

    pub(crate) fn initialize(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), ProductRuntimeError> {
        let window = Arc::new(
            event_loop.create_window(
                Window::default_attributes()
                    .with_title("Motolii")
                    .with_inner_size(LogicalSize::new(1200.0, 800.0))
                    .with_visible(false),
            )?,
        );
        // Metal sublayerをon-screenのcontent viewへ結び付けてからSurfaceを作る。
        window.set_visible(true);
        let parts = self
            .parts
            .take()
            .ok_or(ProductRuntimeError::AlreadyInitialized)?;
        let gfx = ProductSurface::new(&window, parts, &self.gpu, &self.preview)?;
        let initial_instance_epoch = BrowserHostRuntime::fresh_instance_epoch()?;
        let browser_source = BrowserHostRuntime::built_in_rectangle_source(initial_instance_epoch);
        let browser = build_browser_runtime(
            &window,
            initial_instance_epoch,
            browser_source.clone(),
            self.proxy.clone(),
        )?;
        self.browser_lifecycle = Some(BrowserLifecycleCoordinator::new(initial_instance_epoch)?);
        self.browser_source = Some(browser_source);
        self.window = Some(window);
        self.browser = Some(browser);
        self.gfx = Some(gfx);
        self.update_layout()?;
        if let Some(window) = &self.window {
            window.request_redraw();
        }
        Ok(())
    }

    pub(crate) fn update_layout(&mut self) -> Result<(), ProductRuntimeError> {
        let Some(window) = &self.window else {
            return Ok(());
        };
        let size = window.inner_size();
        let Some(layout) = NativeHostLayout::try_new(
            self.next_layout_epoch,
            size.width,
            size.height,
            window.scale_factor(),
            self.preview.slot().desc(),
        ) else {
            self.layout = None;
            return Ok(());
        };
        self.next_layout_epoch = self
            .next_layout_epoch
            .checked_add(1)
            .ok_or(ProductRuntimeError::LayoutEpochExhausted)?;
        if let Some(browser) = &self.browser {
            browser.set_bounds(layout.epoch, layout.browser)?;
        }
        self.layout = Some(layout);
        Ok(())
    }

    pub(crate) fn poll_browser(&mut self, event_loop: &ActiveEventLoop) {
        if self
            .surface_retry_at
            .is_some_and(|retry_at| Instant::now() >= retry_at)
        {
            self.surface_retry_at = None;
            self.request_redraw();
        }
        let Some(browser) = &self.browser else {
            return;
        };
        if let Err(error) = browser.ensure_focus(self.browser_focus_target) {
            return self.fail(event_loop, error);
        }
        if self.active_place.is_none() {
            let generation = self.next_place_generation;
            let Some(next_generation) = generation.checked_add(1) else {
                return self.fail(event_loop, ProductRuntimeError::PlaceGenerationExhausted);
            };
            match browser.take_place_intent(generation) {
                Ok(Some((intent, generation))) => {
                    self.next_place_generation = next_generation;
                    self.browser_focus_target = BrowserFocusTarget::Parent;
                    if let Err(error) = browser.ensure_focus(self.browser_focus_target) {
                        return self.fail(event_loop, error);
                    }
                    if !self.terminal_admission.begin(generation) {
                        return self.fail(
                            event_loop,
                            ProductRuntimeError::PlaceAdmissionGenerationRejected(generation),
                        );
                    }
                    self.place_preview.clear();
                    self.candidate_terminal = None;
                    self.admitted_terminal = None;
                    self.pending_stage_drop = None;
                    self.active_place = Some(intent);
                }
                Ok(None) => {}
                Err(error) => return self.fail(event_loop, error),
            }
        }
        if self.active_place.is_none() {
            self.set_idle_control_flow(event_loop);
            return;
        }
        match browser.poll_pointer_candidate() {
            Ok(Some(HostPointerCandidate::Moved {
                generation,
                position,
            })) => {
                if let (Some(source), Some(layout)) = (&self.active_place, self.layout) {
                    self.place_preview
                        .deliver(source, generation, position, layout);
                    self.request_redraw();
                }
                event_loop.set_control_flow(ControlFlow::Poll);
            }
            Ok(Some(HostPointerCandidate::Released {
                generation,
                position,
            })) => {
                self.place_preview.clear();
                let Some(source) = self.active_place.take() else {
                    self.set_idle_control_flow(event_loop);
                    return;
                };
                let Some(layout) = self.layout else {
                    return self.fail(
                        event_loop,
                        ProductRuntimeError::PlaceTerminalLayoutUnavailable,
                    );
                };
                let terminal =
                    ClassifiedPlaceTerminal::released(source, generation, position, layout);
                if self.terminal_admission.admit(&terminal) {
                    self.pending_stage_drop = self.terminal_delivery.deliver(&terminal);
                    self.admitted_terminal = Some(terminal.clone());
                }
                self.candidate_terminal = Some(terminal);
                self.set_idle_control_flow(event_loop);
            }
            Ok(Some(HostPointerCandidate::Cancelled { generation, reason })) => {
                self.place_preview.clear();
                if let Some(source) = self.active_place.take() {
                    let terminal = ClassifiedPlaceTerminal::cancelled(source, generation, reason);
                    if self.terminal_admission.admit(&terminal) {
                        self.admitted_terminal = Some(terminal.clone());
                    }
                    self.candidate_terminal = Some(terminal);
                }
                self.set_idle_control_flow(event_loop);
            }
            Ok(None) => match browser.pointer_capture_is_active() {
                Ok(true) => event_loop.set_control_flow(ControlFlow::Poll),
                Ok(false) => self.set_idle_control_flow(event_loop),
                Err(error) => self.fail(event_loop, error),
            },
            Err(error) => self.fail(event_loop, error),
        }
        if let Some(drop) = &self.pending_stage_drop {
            let _ = (&drop.source, drop.generation, drop.layout_epoch, drop.ndc);
        }
        if let Some(terminal) = &self.candidate_terminal {
            let _ = (
                &terminal.source,
                terminal.generation,
                terminal.cause,
                terminal.layout_epoch,
                terminal.stage_ndc,
            );
        }
        if let Some(terminal) = &self.admitted_terminal {
            let _ = (
                &terminal.source,
                terminal.generation,
                terminal.layout_epoch,
                terminal.stage_ndc,
            );
        }
        if let Some(drop) = self.pending_stage_drop.take() {
            let Some(position) = canonical_drop_from_ndc(self.displayed_camera, drop.ndc) else {
                return self.fail(event_loop, ProductRuntimeError::PlaceCanonicalConversion);
            };
            self.document_queue
                .push_place_rectangle(PlaceRectangleRequest {
                    position,
                    playhead: RationalTime::ZERO,
                });
            match self.document_runtime.process_next(
                &mut self.document_queue,
                self.primary,
                self.projection_generation,
            ) {
                Ok(Some(published)) => {
                    self.current_document = published.snapshot;
                    self.primary = published.primary;
                    self.projection_generation = published.projection_generation;
                    if let Err(error) = self.submit_stage_projection() {
                        return self.fail(event_loop, error);
                    }
                    self.request_redraw();
                }
                Ok(None) => {}
                Err(error) => self.fail(event_loop, error),
            }
        }
    }

    pub(crate) fn handle_product_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        event: ProductEvent,
    ) {
        match event {
            ProductEvent::Wake => {
                if let Err(error) = self.drain_stage_projection() {
                    self.fail(event_loop, error);
                    return;
                }
                self.request_redraw();
            }
            ProductEvent::BrowserLifecycle(event) => {
                if let Err(error) = self.handle_browser_lifecycle(event) {
                    self.fail(event_loop, error);
                }
            }
        }
    }

    fn handle_browser_lifecycle(
        &mut self,
        event: BrowserLifecycleEvent,
    ) -> Result<(), ProductRuntimeError> {
        let Some(active_epoch) = self
            .browser
            .as_ref()
            .map(BrowserHostRuntime::instance_epoch)
            .transpose()?
        else {
            return Ok(());
        };
        let decision = self
            .browser_lifecycle
            .as_mut()
            .ok_or(ProductRuntimeError::BrowserLifecycleUnavailable)?
            .observe(active_epoch, event)?;
        match decision {
            BrowserRecoveryDecision::Ignore => Ok(()),
            BrowserRecoveryDecision::Replace { instance_epoch } => {
                self.replace_browser(instance_epoch)
            }
            BrowserRecoveryDecision::Degrade => {
                self.active_place = None;
                self.place_preview.clear();
                self.terminal_admission.retire_active();
                self.candidate_terminal = None;
                self.admitted_terminal = None;
                self.browser.take();
                Ok(())
            }
        }
    }

    fn replace_browser(&mut self, instance_epoch: u64) -> Result<(), ProductRuntimeError> {
        let window = self
            .window
            .as_ref()
            .ok_or(ProductRuntimeError::BrowserLifecycleUnavailable)?;
        let source = self
            .browser_source
            .clone()
            .ok_or(ProductRuntimeError::BrowserLifecycleUnavailable)?;
        self.active_place = None;
        self.place_preview.clear();
        self.terminal_admission.retire_active();
        self.candidate_terminal = None;
        self.admitted_terminal = None;
        self.browser.take();
        let browser = build_browser_runtime(window, instance_epoch, source, self.proxy.clone())?;
        if let Some(layout) = self.layout {
            browser.set_bounds(layout.epoch, layout.browser)?;
        }
        self.browser = Some(browser);
        self.request_redraw();
        Ok(())
    }

    fn set_idle_control_flow(&self, event_loop: &ActiveEventLoop) {
        match self.surface_retry_at {
            Some(retry_at) => event_loop.set_control_flow(ControlFlow::WaitUntil(retry_at)),
            None => event_loop.set_control_flow(ControlFlow::Wait),
        }
    }

    fn submit_stage_projection(&self) -> Result<RenderGeneration, ProductRuntimeError> {
        Ok(self.render_client.submit(RenderRequest {
            document: Arc::clone(&self.current_document),
            data_tracks: Arc::clone(&self.render_request_template.data_tracks),
            evaluation_time: self.render_request_template.evaluation_time,
            desc: self.render_request_template.desc,
            quality: self.render_request_template.quality,
        })?)
    }

    fn drain_stage_projection(&mut self) -> Result<(), ProductRuntimeError> {
        let Some(result) = self.render_client.try_take_latest() else {
            return Ok(());
        };
        if !self.stage_projection.accepts(
            result.generation,
            self.render_client.latest_accepted_generation(),
        ) {
            return Ok(());
        }
        let rendered = result.result?;
        self.preview.slot().copy(&self.gpu, &rendered.frame)?;
        self.displayed_camera = rendered.camera;
        self.stage_projection.commit(result.generation);
        Ok(())
    }

    pub(crate) fn fail(&mut self, event_loop: &ActiveEventLoop, error: impl std::fmt::Display) {
        self.failure = Some(error.to_string());
        event_loop.exit();
    }

    pub(crate) fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    pub(crate) fn resize(&mut self, event_loop: &ActiveEventLoop, width: u32, height: u32) {
        if let Some(gfx) = &mut self.gfx {
            gfx.configure(width, height);
        }
        if let Err(error) = self.update_layout() {
            self.fail(event_loop, error);
            return;
        }
        if width > 0 && height > 0 {
            self.render(event_loop);
        }
    }

    pub(crate) fn scale_factor_changed(&mut self, event_loop: &ActiveEventLoop) {
        let Some(window) = &self.window else {
            return;
        };
        let size = window.inner_size();
        self.resize(event_loop, size.width, size.height);
    }

    pub(crate) fn set_occluded(&mut self, occluded: bool) {
        if let Some(gfx) = &mut self.gfx {
            gfx.occluded = occluded;
        }
        if !occluded {
            self.surface_retry_at = None;
            self.request_redraw();
        }
    }

    pub(crate) fn render(&mut self, event_loop: &ActiveEventLoop) {
        let (Some(gfx), Some(layout), Some(window)) = (&mut self.gfx, self.layout, &self.window)
        else {
            return;
        };
        match gfx.render(layout, window) {
            Ok(()) => {}
            Err(ProductSurfaceError::Recover) => {
                gfx.reconfigure();
                window.request_redraw();
            }
            Err(ProductSurfaceError::Retry) => {
                let retry_at = Instant::now() + Duration::from_millis(50);
                self.surface_retry_at = Some(retry_at);
                event_loop.set_control_flow(ControlFlow::WaitUntil(retry_at));
            }
            Err(ProductSurfaceError::Skip) => {}
            Err(ProductSurfaceError::Fatal(reason)) => self.fail(event_loop, reason),
        }
    }
}

fn build_browser_runtime(
    window: &Window,
    instance_epoch: u64,
    source: BrowserPlaceIntent,
    proxy: EventLoopProxy<ProductEvent>,
) -> Result<BrowserHostRuntime, BrowserHostRuntimeError> {
    let wake_proxy = proxy.clone();
    let lifecycle_proxy = proxy;
    BrowserHostRuntime::new(
        window,
        instance_epoch,
        source,
        Arc::new(move || {
            let _ = wake_proxy.send_event(ProductEvent::Wake);
        }),
        Arc::new(move |event| {
            let _ = lifecycle_proxy.send_event(ProductEvent::BrowserLifecycle(event));
        }),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BrowserRecoveryDecision {
    Ignore,
    Replace { instance_epoch: u64 },
    Degrade,
}

#[derive(Debug)]
struct BrowserLifecycleCoordinator {
    next_instance_epoch: u64,
    automatic_process_recovery_used: bool,
    degraded: bool,
}

impl BrowserLifecycleCoordinator {
    fn new(initial_instance_epoch: u64) -> Result<Self, ProductRuntimeError> {
        Ok(Self {
            next_instance_epoch: initial_instance_epoch
                .checked_add(1)
                .ok_or(ProductRuntimeError::BrowserInstanceEpochExhausted)?,
            automatic_process_recovery_used: false,
            degraded: false,
        })
    }

    fn observe(
        &mut self,
        active_epoch: u64,
        event: BrowserLifecycleEvent,
    ) -> Result<BrowserRecoveryDecision, ProductRuntimeError> {
        if self.degraded || event.instance_epoch() != active_epoch {
            return Ok(BrowserRecoveryDecision::Ignore);
        }
        if matches!(event, BrowserLifecycleEvent::ProcessTerminated { .. })
            && self.automatic_process_recovery_used
        {
            self.degraded = true;
            return Ok(BrowserRecoveryDecision::Degrade);
        }
        if matches!(event, BrowserLifecycleEvent::ProcessTerminated { .. }) {
            self.automatic_process_recovery_used = true;
        }
        let instance_epoch = self.next_instance_epoch;
        self.next_instance_epoch = self
            .next_instance_epoch
            .checked_add(1)
            .ok_or(ProductRuntimeError::BrowserInstanceEpochExhausted)?;
        Ok(BrowserRecoveryDecision::Replace { instance_epoch })
    }
}

struct ProductSurface {
    surface: wgpu::Surface<'static>,
    gpu: Arc<GpuCtx>,
    config: wgpu::SurfaceConfiguration,
    preview_pipeline: wgpu::RenderPipeline,
    preview_bind_group: wgpu::BindGroup,
    timeline_pipeline: wgpu::RenderPipeline,
    occluded: bool,
}

struct ProductGpuParts {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
}

impl ProductSurface {
    fn new(
        window: &Arc<Window>,
        parts: ProductGpuParts,
        gpu: &Arc<GpuCtx>,
        preview: &StaticPreview,
    ) -> Result<Self, ProductRuntimeError> {
        let surface = parts.instance.create_surface(Arc::clone(window))?;
        if !parts.adapter.is_surface_supported(&surface) {
            return Err(ProductRuntimeError::SurfaceUnsupported);
        }
        let size = window.inner_size();
        let capabilities = surface.get_capabilities(&parts.adapter);
        let format = capabilities
            .formats
            .first()
            .copied()
            .ok_or(ProductRuntimeError::SurfaceUnsupported)?;
        let alpha_mode = capabilities
            .alpha_modes
            .first()
            .copied()
            .ok_or(ProductRuntimeError::SurfaceUnsupported)?;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats: vec![],
        };
        surface.configure(&parts.device, &config);
        let (preview_pipeline, preview_bind_group) =
            create_preview_pipeline(&parts.device, format, preview.slot().view());
        let timeline_pipeline = create_solid_pipeline(&parts.device, format);
        Ok(Self {
            surface,
            gpu: Arc::clone(gpu),
            config,
            preview_pipeline,
            preview_bind_group,
            timeline_pipeline,
            occluded: false,
        })
    }

    fn configure(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.reconfigure();
    }

    fn reconfigure(&self) {
        self.surface.configure(&self.gpu.device, &self.config);
    }

    fn render(
        &mut self,
        layout: NativeHostLayout,
        window: &Window,
    ) -> Result<(), ProductSurfaceError> {
        if self.occluded || self.config.width == 0 || self.config.height == 0 {
            return Err(ProductSurfaceError::Skip);
        }
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Timeout => {
                return Err(ProductSurfaceError::Retry);
            }
            wgpu::CurrentSurfaceTexture::Occluded => {
                return Err(ProductSurfaceError::Retry);
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                return Err(ProductSurfaceError::Recover);
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(ProductSurfaceError::Fatal(
                    "native product Surface validation failed".to_owned(),
                ));
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("motolii-product-native-frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("motolii-product-stage-timeline"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.018,
                            g: 0.020,
                            b: 0.024,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            draw_rect(
                &mut pass,
                layout.timeline_physical,
                &self.timeline_pipeline,
                None,
            );
            draw_rect(
                &mut pass,
                layout.stage_physical,
                &self.preview_pipeline,
                Some(&self.preview_bind_group),
            );
        }
        self.gpu.queue.submit([encoder.finish()]);
        window.pre_present_notify();
        frame.present();
        Ok(())
    }
}

fn draw_rect<'a>(
    pass: &mut wgpu::RenderPass<'a>,
    rect: PhysicalRect,
    pipeline: &'a wgpu::RenderPipeline,
    bind_group: Option<&'a wgpu::BindGroup>,
) {
    if rect.width == 0 || rect.height == 0 {
        return;
    }
    pass.set_pipeline(pipeline);
    if let Some(bind_group) = bind_group {
        pass.set_bind_group(0, bind_group, &[]);
    }
    pass.set_viewport(
        rect.x as f32,
        rect.y as f32,
        rect.width as f32,
        rect.height as f32,
        0.0,
        1.0,
    );
    pass.set_scissor_rect(rect.x, rect.y, rect.width, rect.height);
    pass.draw(0..3, 0..1);
}

fn create_preview_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    view: &wgpu::TextureView,
) -> (wgpu::RenderPipeline, wgpu::BindGroup) {
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("motolii-product-preview-sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("motolii-product-preview-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("motolii-product-preview-bind-group"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    (
        create_pipeline(device, format, Some(&layout), PREVIEW_SHADER),
        bind_group,
    )
}

fn create_solid_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    create_pipeline(device, format, None, TIMELINE_SHADER)
}

fn create_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    bind_group_layout: Option<&wgpu::BindGroupLayout>,
    source: &'static str,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("motolii-product-native-shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
    });
    let layouts: Vec<_> = bind_group_layout.into_iter().map(Some).collect();
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("motolii-product-native-pipeline-layout"),
        bind_group_layouts: &layouts,
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("motolii-product-native-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(format.into())],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

const PREVIEW_SHADER: &str = r#"
struct VertexOut { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> }
@vertex fn vs_main(@builtin(vertex_index) index: u32) -> VertexOut {
    var positions = array<vec2<f32>, 3>(vec2(-1.0,-1.0), vec2(3.0,-1.0), vec2(-1.0,3.0));
    var uvs = array<vec2<f32>, 3>(vec2(0.0,1.0), vec2(2.0,1.0), vec2(0.0,-1.0));
    var out: VertexOut; out.position = vec4(positions[index],0.0,1.0); out.uv = uvs[index]; return out;
}
@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@fragment fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(source_texture, source_sampler, in.uv);
}
"#;
const TIMELINE_SHADER: &str = r#"
struct VertexOut { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> }
@vertex fn vs_main(@builtin(vertex_index) index: u32) -> VertexOut {
    var positions = array<vec2<f32>, 3>(vec2(-1.0,-1.0), vec2(3.0,-1.0), vec2(-1.0,3.0));
    var uvs = array<vec2<f32>, 3>(vec2(0.0,1.0), vec2(2.0,1.0), vec2(0.0,-1.0));
    var out: VertexOut; out.position = vec4(positions[index],0.0,1.0); out.uv = uvs[index]; return out;
}
@fragment fn fs_main(_in: VertexOut) -> @location(0) vec4<f32> {
    return vec4(0.055, 0.058, 0.066, 1.0);
}
"#;

enum ProductSurfaceError {
    Recover,
    Retry,
    Skip,
    Fatal(String),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ProductRuntimeError {
    #[error(transparent)]
    Gpu(#[from] motolii_gpu::GpuError),
    #[error(transparent)]
    Preview(#[from] crate::static_preview::StaticPreviewError),
    #[error(transparent)]
    RenderWorkerStart(#[from] crate::render_worker::RenderWorkerStartError),
    #[error(transparent)]
    RenderWorkerJoin(#[from] crate::render_worker::RenderJoinError),
    #[error(transparent)]
    RenderSubmit(#[from] crate::render_worker::RenderSubmitError),
    #[error(transparent)]
    RenderWorker(#[from] RenderWorkerError),
    #[error(transparent)]
    RepaintSignal(#[from] crate::render_worker::RepaintSignalRegistrationError),
    #[error(transparent)]
    Display(#[from] crate::display_slot::DisplaySlotError),
    #[error(transparent)]
    EventLoop(#[from] winit::error::EventLoopError),
    #[error(transparent)]
    Os(#[from] winit::error::OsError),
    #[error(transparent)]
    Surface(#[from] wgpu::CreateSurfaceError),
    #[error(transparent)]
    Browser(#[from] BrowserHostRuntimeError),
    #[error("native product Surface is unsupported by the selected adapter")]
    SurfaceUnsupported,
    #[error("native product Host was initialized twice")]
    AlreadyInitialized,
    #[error("native product layout epoch is exhausted")]
    LayoutEpochExhausted,
    #[error("Browser instance epoch is exhausted")]
    BrowserInstanceEpochExhausted,
    #[error("Browser lifecycle coordinator is unavailable")]
    BrowserLifecycleUnavailable,
    #[error("Place terminal cannot be classified without a native Host layout")]
    PlaceTerminalLayoutUnavailable,
    #[error("Place admission rejected capture generation {0}")]
    PlaceAdmissionGenerationRejected(u64),
    #[error("Place capture generation is exhausted")]
    PlaceGenerationExhausted,
    #[error("Place Stage NDC could not be converted through the displayed camera")]
    PlaceCanonicalConversion,
    #[error(transparent)]
    DocumentEdit(#[from] DocumentEditRuntimeError),
    #[error("native product runtime failed: {0}")]
    Runtime(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_core::{ColorSpace, FrameDesc, PixelFormat};

    fn test_layout(epoch: u64) -> NativeHostLayout {
        let frame =
            FrameDesc::try_packed(1920, 1080, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true)
                .unwrap();
        NativeHostLayout::try_new(epoch, 1000, 800, 1.0, frame).unwrap()
    }

    fn test_source() -> BrowserPlaceIntent {
        BrowserPlaceIntent {
            scope_ref: "builtin-stable".to_owned(),
            item_id: "rectangle".to_owned(),
        }
    }

    #[test]
    fn stage_projection_accepts_only_the_latest_new_generation() {
        let one = RenderGeneration::new(1).unwrap();
        let two = RenderGeneration::new(2).unwrap();
        let mut projection = ProductStageProjection::default();

        assert!(!projection.accepts(one, Some(two)));
        assert!(projection.accepts(two, Some(two)));
        projection.commit(two);
        assert!(!projection.accepts(two, Some(two)));
        assert!(!projection.accepts(one, Some(one)));
    }

    #[test]
    fn moved_progress_creates_a_nonterminal_preview_phase() {
        let layout = test_layout(9);
        let center = [
            layout.stage.x + layout.stage.width / 2.0,
            layout.stage.y + layout.stage.height / 2.0,
        ];
        let source = test_source();
        let mut phase = PlacePreviewPhase::default();

        phase.deliver(&source, 4, center, layout);

        assert_eq!(
            phase.latest,
            Some(PlacePreviewProgress {
                source,
                generation: 4,
                layout_epoch: 9,
                stage_ndc: Some([0.0, 0.0]),
            })
        );
    }

    #[test]
    fn outside_progress_updates_preview_without_becoming_a_terminal() {
        let layout = test_layout(9);
        let source = test_source();
        let mut phase = PlacePreviewPhase::default();
        phase.deliver(&source, 4, [10.0, 10.0], layout);

        assert_eq!(
            phase.latest,
            Some(PlacePreviewProgress {
                source,
                generation: 4,
                layout_epoch: 9,
                stage_ndc: None,
            })
        );
        phase.clear();
        assert_eq!(phase.latest, None);
    }

    #[test]
    fn release_inside_stage_has_no_noncommit_cause() {
        let layout = test_layout(9);
        let position = [
            layout.stage.x + layout.stage.width / 2.0,
            layout.stage.y + layout.stage.height / 2.0,
        ];

        assert_eq!(
            ClassifiedPlaceTerminal::released(test_source(), 4, position, layout),
            ClassifiedPlaceTerminal {
                source: test_source(),
                generation: 4,
                cause: PlaceTerminalCause::NoNonCommitCause,
                layout_epoch: Some(9),
                stage_ndc: Some([0.0, 0.0]),
            }
        );
    }

    #[test]
    fn release_outside_stage_has_outside_cause() {
        let layout = test_layout(9);

        assert_eq!(
            ClassifiedPlaceTerminal::released(test_source(), 4, [10.0, 10.0], layout),
            ClassifiedPlaceTerminal {
                source: test_source(),
                generation: 4,
                cause: PlaceTerminalCause::OutsideStage,
                layout_epoch: Some(9),
                stage_ndc: None,
            }
        );
    }

    #[test]
    fn cancellation_reason_maps_exhaustively_to_noncommit_cause() {
        for (reason, cause) in [
            (HostPointerCancel::Escape, PlaceTerminalCause::Escape),
            (
                HostPointerCancel::CaptureLost,
                PlaceTerminalCause::CaptureLoss,
            ),
        ] {
            assert_eq!(
                ClassifiedPlaceTerminal::cancelled(test_source(), 4, reason),
                ClassifiedPlaceTerminal {
                    source: test_source(),
                    generation: 4,
                    cause,
                    layout_epoch: None,
                    stage_ndc: None,
                }
            );
        }
    }

    #[test]
    fn admission_accepts_at_most_one_matching_commit_candidate() {
        let layout = test_layout(9);
        let position = [
            layout.stage.x + layout.stage.width / 2.0,
            layout.stage.y + layout.stage.height / 2.0,
        ];
        let terminal = ClassifiedPlaceTerminal::released(test_source(), 4, position, layout);
        let mut admission = PlaceTerminalAdmission::default();

        assert!(admission.begin(4));
        assert!(admission.admit(&terminal));
        assert!(!admission.admit(&terminal));
        assert!(!admission.begin(4));
    }

    #[test]
    fn noncommit_terminal_retires_generation_without_admission() {
        let terminal =
            ClassifiedPlaceTerminal::cancelled(test_source(), 4, HostPointerCancel::CaptureLost);
        let mut admission = PlaceTerminalAdmission::default();

        assert!(admission.begin(4));
        assert!(!admission.admit(&terminal));
        assert!(!admission.admit(&terminal));
        assert!(!admission.begin(4));
    }

    #[test]
    fn stale_terminal_does_not_retire_the_current_drag() {
        let layout = test_layout(9);
        let position = [
            layout.stage.x + layout.stage.width / 2.0,
            layout.stage.y + layout.stage.height / 2.0,
        ];
        let stale = ClassifiedPlaceTerminal::released(test_source(), 4, position, layout);
        let current = ClassifiedPlaceTerminal::released(test_source(), 5, position, layout);
        let mut admission = PlaceTerminalAdmission::default();

        assert!(admission.begin(4));
        assert!(admission.admit(&stale));
        assert!(admission.begin(5));
        assert!(!admission.admit(&stale));
        assert!(admission.admit(&current));
    }

    #[test]
    fn retained_high_water_rejects_replay_after_detail_eviction() {
        let layout = test_layout(9);
        let position = [
            layout.stage.x + layout.stage.width / 2.0,
            layout.stage.y + layout.stage.height / 2.0,
        ];
        let terminal = ClassifiedPlaceTerminal::released(test_source(), 8, position, layout);
        let mut admission = PlaceTerminalAdmission::default();

        assert!(admission.begin(8));
        assert!(admission.admit(&terminal));
        assert!(!admission.begin(7));
        assert!(!admission.begin(8));
        assert!(admission.begin(9));
    }

    #[test]
    fn admitted_terminal_delivers_once_to_the_single_pending_boundary() {
        let layout = test_layout(9);
        let position = [
            layout.stage.x + layout.stage.width / 2.0,
            layout.stage.y + layout.stage.height / 2.0,
        ];
        let terminal = ClassifiedPlaceTerminal::released(test_source(), 4, position, layout);
        let mut delivery = PlaceTerminalDelivery::default();

        let delivered = delivery.deliver(&terminal).unwrap();
        assert_eq!(delivered.generation, 4);
        assert_eq!(delivered.layout_epoch, 9);
        assert_eq!(delivered.ndc, [0.0, 0.0]);
        assert!(delivery.deliver(&terminal).is_none());
    }

    #[test]
    fn unadmitted_causes_cannot_enter_the_delivery_boundary() {
        let mut delivery = PlaceTerminalDelivery::default();
        for reason in [HostPointerCancel::Escape, HostPointerCancel::CaptureLost] {
            let terminal = ClassifiedPlaceTerminal::cancelled(test_source(), 4, reason);
            assert!(delivery.deliver(&terminal).is_none());
        }
    }

    #[test]
    fn lifecycle_replaces_with_new_epochs_and_ignores_old_callbacks() {
        let mut lifecycle = BrowserLifecycleCoordinator::new(7).unwrap();

        assert_eq!(
            lifecycle
                .observe(
                    7,
                    BrowserLifecycleEvent::ReloadStarted { instance_epoch: 7 }
                )
                .unwrap(),
            BrowserRecoveryDecision::Replace { instance_epoch: 8 }
        );
        assert_eq!(
            lifecycle
                .observe(
                    8,
                    BrowserLifecycleEvent::ReloadStarted { instance_epoch: 7 }
                )
                .unwrap(),
            BrowserRecoveryDecision::Ignore
        );
        assert_eq!(
            lifecycle
                .observe(
                    8,
                    BrowserLifecycleEvent::ReloadStarted { instance_epoch: 8 }
                )
                .unwrap(),
            BrowserRecoveryDecision::Replace { instance_epoch: 9 }
        );
    }

    #[test]
    fn automatic_process_recovery_is_bounded_to_one_replacement() {
        let mut lifecycle = BrowserLifecycleCoordinator::new(10).unwrap();

        assert_eq!(
            lifecycle
                .observe(
                    10,
                    BrowserLifecycleEvent::ProcessTerminated { instance_epoch: 10 }
                )
                .unwrap(),
            BrowserRecoveryDecision::Replace { instance_epoch: 11 }
        );
        assert_eq!(
            lifecycle
                .observe(
                    11,
                    BrowserLifecycleEvent::ProcessTerminated { instance_epoch: 11 }
                )
                .unwrap(),
            BrowserRecoveryDecision::Degrade
        );
        assert_eq!(
            lifecycle
                .observe(
                    11,
                    BrowserLifecycleEvent::ReloadStarted { instance_epoch: 11 }
                )
                .unwrap(),
            BrowserRecoveryDecision::Ignore
        );
    }

    #[test]
    fn lifecycle_epoch_exhaustion_is_typed() {
        assert!(matches!(
            BrowserLifecycleCoordinator::new(u64::MAX),
            Err(ProductRuntimeError::BrowserInstanceEpochExhausted)
        ));
    }
}
