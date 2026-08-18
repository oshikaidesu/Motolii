//! 最新要求だけを実行するprivate render worker境界。

use std::num::NonZeroU64;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};

use motolii_core::{CompCamera, FrameDesc, Quality};
use motolii_doc::{build_document_frame_graph, Command, CommandError, Document, EvaluationTime};
use motolii_eval::DataTracks;
use motolii_export::VideoSourceBinder;
use motolii_gpu::GpuCtx;
use motolii_plugins_firstparty::first_party_runtime;
use motolii_render::{render_graph_cached, RenderGraphInputs, RenderSession, RenderedFrame};

type RepaintSignal = Arc<dyn Fn() + Send + Sync>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct RenderGeneration(NonZeroU64);

impl RenderGeneration {
    pub(crate) fn new(value: u64) -> Option<Self> {
        NonZeroU64::new(value).map(Self)
    }

    pub(crate) fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RepaintSignalEpoch(NonZeroU64);

#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RepaintSignalRegistrationError {
    #[error("repaint signal epoch space is exhausted")]
    EpochExhausted,
}

struct RegisteredRepaintSignal {
    epoch: RepaintSignalEpoch,
    signal: RepaintSignal,
}

struct RepaintSignalState {
    next_epoch: Option<RepaintSignalEpoch>,
    current: Option<RegisteredRepaintSignal>,
    failed_epoch: Option<RepaintSignalEpoch>,
}

#[derive(Debug)]
struct StampedRequest<P> {
    generation: RenderGeneration,
    payload: P,
}

#[derive(Debug)]
pub(crate) struct StampedResult<R, E> {
    pub(crate) generation: RenderGeneration,
    pub(crate) result: Result<R, E>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RenderWorkerStatus {
    Running,
    Closing,
    Closed,
    WorkerPanicked {
        running_generation: RenderGeneration,
        abandoned_pending_generation: Option<RenderGeneration>,
    },
}

#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RenderSubmitError {
    #[error("render generation space is exhausted")]
    GenerationExhausted,
    #[error("render worker is closed")]
    Closed,
    #[error("render worker stopped after a panic")]
    WorkerStopped,
}

#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RenderJoinError {
    #[error(
        "render worker panicked at generation {running_generation:?}; abandoned pending generation: {abandoned_pending_generation:?}"
    )]
    WorkerPanicked {
        running_generation: RenderGeneration,
        abandoned_pending_generation: Option<RenderGeneration>,
    },
    #[error("render worker thread terminated outside the guarded executor")]
    ThreadPanicked,
}

#[derive(Debug)]
struct RequestState<P> {
    pending: Option<StampedRequest<P>>,
    next_generation: Option<RenderGeneration>,
    latest_accepted_generation: Option<RenderGeneration>,
    status: RenderWorkerStatus,
}

struct WorkerShared<P, R, E> {
    requests: Mutex<RequestState<P>>,
    request_ready: Condvar,
    result: Mutex<Option<StampedResult<R, E>>>,
    repaint_signal: Mutex<RepaintSignalState>,
}

impl<P, R, E> WorkerShared<P, R, E> {
    fn lock_requests(&self) -> MutexGuard<'_, RequestState<P>> {
        self.requests
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn lock_result(&self) -> MutexGuard<'_, Option<StampedResult<R, E>>> {
        self.result
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn lock_repaint_signal(&self) -> MutexGuard<'_, RepaintSignalState> {
        self.repaint_signal
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

struct LatestWorker<P, R, E> {
    shared: Arc<WorkerShared<P, R, E>>,
    join: Option<JoinHandle<()>>,
}

struct LatestWorkerClient<P, R, E> {
    shared: Arc<WorkerShared<P, R, E>>,
}

impl<P, R, E> Clone for LatestWorkerClient<P, R, E> {
    fn clone(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<P, R, E> LatestWorker<P, R, E>
where
    P: Send + 'static,
    R: Send + 'static,
    E: Send + 'static,
{
    fn spawn(
        name: &str,
        execute: impl FnMut(P) -> Result<R, E> + Send + 'static,
        panic_error: fn() -> E,
    ) -> Result<Self, std::io::Error> {
        Self::spawn_from_generation(name, 1, execute, panic_error, None)
    }

    fn spawn_from_generation(
        name: &str,
        first_generation: u64,
        mut execute: impl FnMut(P) -> Result<R, E> + Send + 'static,
        panic_error: fn() -> E,
        start_gate: Option<std::sync::mpsc::Receiver<()>>,
    ) -> Result<Self, std::io::Error> {
        let shared = Arc::new(WorkerShared {
            requests: Mutex::new(RequestState {
                pending: None,
                next_generation: RenderGeneration::new(first_generation),
                latest_accepted_generation: None,
                status: RenderWorkerStatus::Running,
            }),
            request_ready: Condvar::new(),
            result: Mutex::new(None),
            repaint_signal: Mutex::new(RepaintSignalState {
                next_epoch: Some(RepaintSignalEpoch(NonZeroU64::MIN)),
                current: None,
                failed_epoch: None,
            }),
        });
        let worker_shared = Arc::clone(&shared);
        let join = thread::Builder::new().name(name.into()).spawn(move || {
            if let Some(gate) = start_gate {
                let _ = gate.recv();
            }
            run_worker_loop(&worker_shared, &mut execute, panic_error);
        })?;
        Ok(Self {
            shared,
            join: Some(join),
        })
    }

    fn submit(&self, payload: P) -> Result<RenderGeneration, RenderSubmitError> {
        self.client().submit(payload)
    }

    fn latest_accepted_generation(&self) -> Option<RenderGeneration> {
        self.client().latest_accepted_generation()
    }

    fn status(&self) -> RenderWorkerStatus {
        self.client().status()
    }

    fn try_take_latest(&self) -> Option<StampedResult<R, E>> {
        self.client().try_take_latest()
    }

    fn close(&self) {
        let mut state = self.shared.lock_requests();
        if state.status == RenderWorkerStatus::Running {
            state.status = RenderWorkerStatus::Closing;
            self.shared.request_ready.notify_one();
        }
    }

    fn client(&self) -> LatestWorkerClient<P, R, E> {
        LatestWorkerClient {
            shared: Arc::clone(&self.shared),
        }
    }

    fn join(&mut self) -> Result<(), RenderJoinError> {
        self.close();
        if let Some(join) = self.join.take() {
            join.join().map_err(|_| RenderJoinError::ThreadPanicked)?;
        }
        match self.status() {
            RenderWorkerStatus::WorkerPanicked {
                running_generation,
                abandoned_pending_generation,
            } => Err(RenderJoinError::WorkerPanicked {
                running_generation,
                abandoned_pending_generation,
            }),
            RenderWorkerStatus::Running
            | RenderWorkerStatus::Closing
            | RenderWorkerStatus::Closed => Ok(()),
        }
    }

    #[cfg(test)]
    fn pending_generation(&self) -> Option<RenderGeneration> {
        self.shared
            .lock_requests()
            .pending
            .as_ref()
            .map(|request| request.generation)
    }
}

impl<P, R, E> LatestWorkerClient<P, R, E> {
    fn submit(&self, payload: P) -> Result<RenderGeneration, RenderSubmitError> {
        let mut state = self.shared.lock_requests();
        match state.status {
            RenderWorkerStatus::Running => {}
            RenderWorkerStatus::Closing | RenderWorkerStatus::Closed => {
                return Err(RenderSubmitError::Closed);
            }
            RenderWorkerStatus::WorkerPanicked { .. } => {
                return Err(RenderSubmitError::WorkerStopped);
            }
        }
        let generation = state
            .next_generation
            .ok_or(RenderSubmitError::GenerationExhausted)?;
        state.next_generation = generation
            .get()
            .checked_add(1)
            .and_then(RenderGeneration::new);
        state.latest_accepted_generation = Some(generation);
        state.pending = Some(StampedRequest {
            generation,
            payload,
        });
        self.shared.request_ready.notify_one();
        Ok(generation)
    }

    fn latest_accepted_generation(&self) -> Option<RenderGeneration> {
        self.shared.lock_requests().latest_accepted_generation
    }

    fn status(&self) -> RenderWorkerStatus {
        self.shared.lock_requests().status
    }

    fn try_take_latest(&self) -> Option<StampedResult<R, E>> {
        self.shared.lock_result().take()
    }

    fn register_repaint_signal(
        &self,
        signal: RepaintSignal,
    ) -> Result<RepaintSignalEpoch, RepaintSignalRegistrationError> {
        let epoch = {
            let mut state = self.shared.lock_repaint_signal();
            let epoch = state
                .next_epoch
                .ok_or(RepaintSignalRegistrationError::EpochExhausted)?;
            state.next_epoch = epoch
                .0
                .get()
                .checked_add(1)
                .and_then(NonZeroU64::new)
                .map(RepaintSignalEpoch);
            state.current = Some(RegisteredRepaintSignal {
                epoch,
                signal: Arc::clone(&signal),
            });
            epoch
        };
        if self.shared.lock_result().is_some() {
            invoke_repaint_signal(&self.shared, epoch, signal);
        }
        Ok(epoch)
    }

    fn failed_repaint_signal_epoch(&self) -> Option<RepaintSignalEpoch> {
        self.shared.lock_repaint_signal().failed_epoch
    }
}

impl<P, R, E> Drop for LatestWorker<P, R, E> {
    fn drop(&mut self) {
        let mut state = self.shared.lock_requests();
        if state.status == RenderWorkerStatus::Running {
            state.status = RenderWorkerStatus::Closing;
            self.shared.request_ready.notify_one();
        }
    }
}

fn run_worker_loop<P, R, E>(
    shared: &WorkerShared<P, R, E>,
    execute: &mut impl FnMut(P) -> Result<R, E>,
    panic_error: fn() -> E,
) {
    loop {
        let request = {
            let mut state = shared.lock_requests();
            loop {
                if let Some(request) = state.pending.take() {
                    break request;
                }
                match state.status {
                    RenderWorkerStatus::Running => {
                        state = shared
                            .request_ready
                            .wait(state)
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                    }
                    RenderWorkerStatus::Closing => {
                        state.status = RenderWorkerStatus::Closed;
                        return;
                    }
                    RenderWorkerStatus::Closed | RenderWorkerStatus::WorkerPanicked { .. } => {
                        return;
                    }
                }
            }
        };

        let generation = request.generation;
        match catch_unwind(AssertUnwindSafe(|| execute(request.payload))) {
            Ok(result) => {
                publish_result(shared, StampedResult { generation, result });
            }
            Err(_) => {
                publish_result(
                    shared,
                    StampedResult {
                        generation,
                        result: Err(panic_error()),
                    },
                );
                let mut state = shared.lock_requests();
                let abandoned_pending_generation =
                    state.pending.take().map(|pending| pending.generation);
                state.status = RenderWorkerStatus::WorkerPanicked {
                    running_generation: generation,
                    abandoned_pending_generation,
                };
                shared.request_ready.notify_all();
                return;
            }
        }
    }
}

fn publish_result<P, R, E>(shared: &WorkerShared<P, R, E>, result: StampedResult<R, E>) {
    *shared.lock_result() = Some(result);
    let registered = shared
        .lock_repaint_signal()
        .current
        .as_ref()
        .map(|registered| (registered.epoch, Arc::clone(&registered.signal)));
    if let Some((epoch, signal)) = registered {
        invoke_repaint_signal(shared, epoch, signal);
    }
}

fn invoke_repaint_signal<P, R, E>(
    shared: &WorkerShared<P, R, E>,
    epoch: RepaintSignalEpoch,
    signal: RepaintSignal,
) {
    if catch_unwind(AssertUnwindSafe(|| signal())).is_ok() {
        return;
    }
    let mut state = shared.lock_repaint_signal();
    state.failed_epoch = Some(epoch);
    if state
        .current
        .as_ref()
        .is_some_and(|registered| registered.epoch == epoch)
    {
        state.current = None;
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RenderRequest {
    pub(crate) document: Arc<Document>,
    pub(crate) data_tracks: Arc<DataTracks>,
    pub(crate) evaluation_time: EvaluationTime,
    pub(crate) desc: FrameDesc,
    pub(crate) quality: Quality,
}

#[derive(Debug, Clone)]
struct RenderWorkPayload {
    request: RenderRequest,
    preview_command: Option<Command>,
}

#[derive(Debug)]
pub(crate) enum PreparedPreviewDocument<'a> {
    Baseline(&'a Document),
    Preview(Box<Document>),
}

impl PreparedPreviewDocument<'_> {
    pub(crate) fn as_ref(&self) -> &Document {
        match self {
            Self::Baseline(document) => document,
            Self::Preview(document) => document,
        }
    }
}

pub(crate) fn prepare_preview_document<'a>(
    request: &'a RenderRequest,
    preview_command: Option<&Command>,
) -> Result<PreparedPreviewDocument<'a>, RenderWorkerError> {
    match preview_command {
        None => {
            request.document.validate()?;
            Ok(PreparedPreviewDocument::Baseline(request.document.as_ref()))
        }
        Some(command) => {
            let mut document = (*request.document).clone();
            command.apply(&mut document)?;
            document.validate()?;
            Ok(PreparedPreviewDocument::Preview(Box::new(document)))
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum RenderWorkerError {
    #[error(transparent)]
    Runtime(#[from] motolii_plugins_firstparty::FirstPartyError),
    #[error(transparent)]
    Command(#[from] CommandError),
    #[error(transparent)]
    Document(#[from] motolii_doc::DocumentError),
    #[error(transparent)]
    Graph(#[from] motolii_doc::GraphError),
    #[error(transparent)]
    Render(#[from] motolii_render::RenderError),
    #[error(transparent)]
    Gpu(#[from] motolii_gpu::GpuRuntimeError),
    #[error(transparent)]
    Export(#[from] motolii_export::ExportError),
    #[error("render worker executor panicked")]
    WorkerPanicked,
}

/// `pub`: `stage_frame_seat::StageFrameSeat::spawn` の戻り値として
/// 外部 crate(iced shell)まで名指しできる必要がある(`stage_frame_seat` 側の
/// `pub use` がここへ path を通している)。
#[derive(Debug, thiserror::Error)]
pub enum RenderWorkerStartError {
    #[error(transparent)]
    Runtime(#[from] motolii_plugins_firstparty::FirstPartyError),
    #[error("failed to spawn render worker: {0}")]
    Spawn(#[from] std::io::Error),
}

pub(crate) struct RenderWorker {
    inner: LatestWorker<RenderWorkPayload, RenderedPreview, RenderWorkerError>,
}

#[derive(Clone)]
pub(crate) struct RenderWorkerClient {
    inner: LatestWorkerClient<RenderWorkPayload, RenderedPreview, RenderWorkerError>,
}

#[derive(Debug)]
pub(crate) struct RenderedPreview {
    pub(crate) frame: RenderedFrame,
    pub(crate) camera: CompCamera,
}

impl RenderWorker {
    pub(crate) fn spawn(gpu: Arc<GpuCtx>) -> Result<Self, RenderWorkerStartError> {
        let runtime = first_party_runtime()?;
        let mut session = RenderSession::new(&gpu);
        let mut video_binder = VideoSourceBinder::new(&gpu);
        let execute_gpu = Arc::clone(&gpu);
        let execute = move |work: RenderWorkPayload| {
            let prepared = prepare_preview_document(&work.request, work.preview_command.as_ref())?;
            let document = prepared.as_ref();
            execute_gpu.check_health()?;
            let built = build_document_frame_graph(
                document,
                work.request.evaluation_time,
                work.request.desc,
                &work.request.data_tracks,
                &runtime,
                None,
            )?;
            let bound = video_binder.bind(
                &execute_gpu,
                document,
                None,
                &built.video_slots,
                work.request.desc,
            )?;
            let video_inputs = bound.as_inputs();
            let rendered = render_graph_cached(
                &execute_gpu,
                &mut session,
                work.request.evaluation_time.timeline_time,
                &built.graph,
                &RenderGraphInputs {
                    camera: built.camera,
                    video_sources: &video_inputs,
                    source_time: Some(built.source_time),
                    plugins: Some(runtime.executors()),
                },
                work.request.quality,
            )?;
            execute_gpu.check_health()?;
            Ok(RenderedPreview {
                frame: rendered,
                camera: built.camera,
            })
        };
        Ok(Self {
            inner: LatestWorker::spawn("motolii-u1b1-render-worker", execute, || {
                RenderWorkerError::WorkerPanicked
            })?,
        })
    }

    pub(crate) fn submit(
        &self,
        request: RenderRequest,
    ) -> Result<RenderGeneration, RenderSubmitError> {
        self.inner.submit(RenderWorkPayload {
            request,
            preview_command: None,
        })
    }

    pub(crate) fn client(&self) -> RenderWorkerClient {
        RenderWorkerClient {
            inner: self.inner.client(),
        }
    }

    pub(crate) fn latest_accepted_generation(&self) -> Option<RenderGeneration> {
        self.inner.latest_accepted_generation()
    }

    pub(crate) fn status(&self) -> RenderWorkerStatus {
        self.inner.status()
    }

    pub(crate) fn try_take_latest(
        &self,
    ) -> Option<StampedResult<RenderedPreview, RenderWorkerError>> {
        self.inner.try_take_latest()
    }

    pub(crate) fn close(&self) {
        self.inner.close();
    }

    pub(crate) fn join(&mut self) -> Result<(), RenderJoinError> {
        self.inner.join()
    }
}

impl RenderWorkerClient {
    pub(crate) fn submit(
        &self,
        request: RenderRequest,
    ) -> Result<RenderGeneration, RenderSubmitError> {
        self.inner.submit(RenderWorkPayload {
            request,
            preview_command: None,
        })
    }

    pub(crate) fn submit_preview(
        &self,
        request: RenderRequest,
        preview_command: Command,
    ) -> Result<RenderGeneration, RenderSubmitError> {
        self.inner.submit(RenderWorkPayload {
            request,
            preview_command: Some(preview_command),
        })
    }

    pub(crate) fn latest_accepted_generation(&self) -> Option<RenderGeneration> {
        self.inner.latest_accepted_generation()
    }

    pub(crate) fn try_take_latest(
        &self,
    ) -> Option<StampedResult<RenderedPreview, RenderWorkerError>> {
        self.inner.try_take_latest()
    }

    pub(crate) fn register_repaint_signal(
        &self,
        signal: RepaintSignal,
    ) -> Result<RepaintSignalEpoch, RepaintSignalRegistrationError> {
        self.inner.register_repaint_signal(signal)
    }

    pub(crate) fn failed_repaint_signal_epoch(&self) -> Option<RepaintSignalEpoch> {
        self.inner.failed_repaint_signal_epoch()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::mpsc;

    use motolii_testkit::unavailable_dep;

    use super::*;
    use crate::static_preview::{bootstrap_document, bootstrap_frame_desc};
    use crate::{
        adapt_input_router_error, CommandId, DiagnosticReasonCode, InputRouterError,
        InteractionState, InteractionStateMachine,
    };
    use motolii_doc::{
        Command, CommandError, DocParam, EffectDefinition, EffectDefinitionId, EffectId, EffectUse,
        LayerId, ScalarPropertyId, TrackItem,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestError {
        Job,
        Panicked,
    }

    fn assert_send_sync<T: Send + Sync>() {}

    fn document_with_opacity_effect() -> (Document, LayerId, EffectId, DocParam) {
        let mut document = bootstrap_document().expect("document");
        let effect = EffectId::from_raw(document.next_stable_id.allocate().expect("effect id"));
        let definition = EffectDefinitionId::from_raw(
            document.next_stable_id.allocate().expect("definition id"),
        );
        let TrackItem::Clip(clip) = &mut document.tracks[0].items[0] else {
            panic!("bootstrap fixture must contain a clip");
        };
        let layer = clip.envelope.layer_id;
        clip.envelope.effects.push(EffectUse {
            id: effect,
            definition_id: definition,
        });
        let amount = DocParam::const_f64(0.72);
        document.effect_definitions.push(EffectDefinition::new(
            definition,
            "core.filter.opacity",
            1,
            true,
            std::collections::BTreeMap::from([("amount".to_owned(), amount.clone())]),
            serde_json::Map::new(),
        ));
        document.validate().expect("effect document");
        (document, layer, effect, amount)
    }

    fn test_render_request(document: Arc<Document>) -> RenderRequest {
        RenderRequest {
            document,
            data_tracks: Arc::new(DataTracks::new()),
            evaluation_time: EvaluationTime::new(motolii_core::RationalTime::ZERO),
            desc: bootstrap_frame_desc().expect("frame desc"),
            quality: Quality::DRAFT,
        }
    }

    fn opacity_preview_command(
        target: LayerId,
        effect: EffectId,
        old_value: DocParam,
        new_value: DocParam,
    ) -> Command {
        Command::SetProperty {
            target,
            property: ScalarPropertyId::EffectParam(effect, "amount".to_owned()),
            old_value,
            new_value,
        }
    }

    #[test]
    fn worker_handles_are_send_sync() {
        assert_send_sync::<RenderRequest>();
        assert_send_sync::<RenderWorkPayload>();
        assert_send_sync::<StampedResult<RenderedPreview, RenderWorkerError>>();
        assert_send_sync::<Arc<WorkerShared<u64, u64, TestError>>>();
        assert_send_sync::<LatestWorkerClient<u64, u64, TestError>>();
    }

    #[test]
    fn hundred_submits_replace_pending_without_waiting_for_consumer() {
        let (release_tx, release_rx) = mpsc::channel();
        let mut worker = LatestWorker::spawn_from_generation(
            "u1b1-hundred-submit",
            1,
            Ok::<_, TestError>,
            || TestError::Panicked,
            Some(release_rx),
        )
        .expect("spawn");
        for value in 1..=100 {
            assert_eq!(worker.submit(value).expect("submit").get(), value);
        }
        assert_eq!(
            worker.pending_generation().map(RenderGeneration::get),
            Some(100)
        );
        release_tx.send(()).expect("release");
        worker.close();
        worker.join().expect("join");
        let result = worker.try_take_latest().expect("latest result");
        assert_eq!(result.generation.get(), 100);
        assert_eq!(result.result, Ok(100));
    }

    #[test]
    fn hundred_optional_preview_payloads_replace_pending_without_waiting() {
        let (release_tx, release_rx) = mpsc::channel();
        let last_had_preview = Arc::new(AtomicU32::new(0));
        let preview_flag = Arc::clone(&last_had_preview);
        let mut worker = LatestWorker::spawn_from_generation(
            "u205wb1-hundred-preview",
            1,
            move |payload: RenderWorkPayload| {
                preview_flag.store(
                    u32::from(payload.preview_command.is_some()),
                    Ordering::Relaxed,
                );
                Ok::<_, TestError>(())
            },
            || TestError::Panicked,
            Some(release_rx),
        )
        .expect("spawn");
        let (document, layer, effect, old_amount) = document_with_opacity_effect();
        let document = Arc::new(document);
        let preview = opacity_preview_command(layer, effect, old_amount, DocParam::const_f64(0.25));
        for value in 1..=100 {
            let preview_command = (value == 100).then(|| preview.clone());
            let generation = worker
                .submit(RenderWorkPayload {
                    request: test_render_request(Arc::clone(&document)),
                    preview_command,
                })
                .expect("submit");
            assert_eq!(generation.get(), value);
        }
        assert_eq!(
            worker.pending_generation().map(RenderGeneration::get),
            Some(100)
        );
        release_tx.send(()).expect("release");
        worker.close();
        worker.join().expect("join");
        assert_eq!(last_had_preview.load(Ordering::Relaxed), 1);
        let result = worker.try_take_latest().expect("latest result");
        assert_eq!(result.generation.get(), 100);
        assert_eq!(result.result, Ok(()));
    }

    #[test]
    fn preview_prepare_none_selects_baseline_document() {
        let document = Arc::new(bootstrap_document().expect("document"));
        let before = serde_json::to_string(&*document).expect("serialize");
        let request = test_render_request(document);
        let prepared = prepare_preview_document(&request, None).expect("baseline");
        match prepared {
            PreparedPreviewDocument::Baseline(baseline) => {
                assert!(std::ptr::eq(baseline, request.document.as_ref()));
                assert_eq!(serde_json::to_string(baseline).expect("serialize"), before);
            }
            PreparedPreviewDocument::Preview(_) => panic!("expected baseline borrow"),
        }
    }

    #[test]
    fn prepared_preview_document_keeps_the_owned_variant_indirect() {
        assert!(
            std::mem::size_of::<PreparedPreviewDocument<'static>>()
                <= 2 * std::mem::size_of::<usize>()
        );
    }

    #[test]
    fn preview_prepare_applies_command_to_clone_not_baseline() {
        let (document, layer, effect, old_amount) = document_with_opacity_effect();
        let document = Arc::new(document);
        let before = serde_json::to_string(&*document).expect("serialize");
        let command = opacity_preview_command(layer, effect, old_amount, DocParam::const_f64(0.25));
        let request = test_render_request(document);
        let prepared = prepare_preview_document(&request, Some(&command)).expect("preview apply");
        match prepared {
            PreparedPreviewDocument::Preview(preview) => {
                assert_ne!(
                    serde_json::to_string(&preview).expect("serialize preview"),
                    before
                );
            }
            PreparedPreviewDocument::Baseline(_) => panic!("expected owned preview"),
        }
        assert_eq!(
            serde_json::to_string(&request.document).expect("serialize baseline"),
            before
        );
    }

    #[test]
    fn preview_prepare_command_error_leaves_baseline_bytes_unchanged() {
        let (document, _, effect, _) = document_with_opacity_effect();
        let document = Arc::new(document);
        let before = serde_json::to_string(&*document).expect("serialize");
        let request = test_render_request(document);
        let command = opacity_preview_command(
            LayerId::from_raw(u64::MAX),
            effect,
            DocParam::const_f64(1.0),
            DocParam::const_f64(0.25),
        );
        let error = prepare_preview_document(&request, Some(&command)).expect_err("command");
        assert!(matches!(
            error,
            RenderWorkerError::Command(CommandError::LayerNotFound(_))
        ));
        assert_eq!(
            serde_json::to_string(&request.document).expect("serialize"),
            before
        );
    }

    #[test]
    fn preview_prepare_validation_error_leaves_baseline_bytes_unchanged() {
        let (document, layer, _, _) = document_with_opacity_effect();
        let document = Arc::new(document);
        let before = serde_json::to_string(&*document).expect("serialize");
        let request = test_render_request(document);
        let command = Command::SetProperty {
            target: layer,
            property: ScalarPropertyId::Opacity,
            old_value: DocParam::const_f64(1.0),
            new_value: DocParam::const_color([0.0, 0.0, 0.0, 1.0]),
        };
        let error = prepare_preview_document(&request, Some(&command)).expect_err("validate");
        assert!(matches!(error, RenderWorkerError::Document(_)));
        assert_eq!(
            serde_json::to_string(&request.document).expect("serialize"),
            before
        );
    }

    #[test]
    fn later_none_preview_payload_selects_pristine_baseline_after_apply() {
        let (document, layer, effect, old_amount) = document_with_opacity_effect();
        let document = Arc::new(document);
        let before = serde_json::to_string(&*document).expect("serialize");
        let request = test_render_request(Arc::clone(&document));
        let command = opacity_preview_command(layer, effect, old_amount, DocParam::const_f64(0.25));
        prepare_preview_document(&request, Some(&command)).expect("preview apply");
        let baseline = prepare_preview_document(&request, None).expect("baseline");
        match baseline {
            PreparedPreviewDocument::Baseline(baseline) => {
                assert!(std::ptr::eq(baseline, request.document.as_ref()));
                assert_eq!(serde_json::to_string(baseline).expect("serialize"), before);
            }
            PreparedPreviewDocument::Preview(_) => panic!("expected baseline borrow"),
        }
    }

    #[test]
    fn typed_preview_prepare_errors_publish_in_generation_order() {
        let (document, layer, effect, old_amount) = document_with_opacity_effect();
        let document = Arc::new(document);
        let before = serde_json::to_string(&*document).expect("serialize");
        let bad = opacity_preview_command(
            LayerId::from_raw(u64::MAX),
            effect,
            DocParam::const_f64(1.0),
            DocParam::const_f64(0.25),
        );
        let good = opacity_preview_command(layer, effect, old_amount, DocParam::const_f64(0.5));
        let mut worker = LatestWorker::spawn(
            "u205wb1-preview-error-order",
            |payload: RenderWorkPayload| {
                prepare_preview_document(&payload.request, payload.preview_command.as_ref())?;
                Ok::<_, RenderWorkerError>(())
            },
            || RenderWorkerError::WorkerPanicked,
        )
        .expect("spawn");
        worker
            .submit(RenderWorkPayload {
                request: test_render_request(Arc::clone(&document)),
                preview_command: Some(bad),
            })
            .expect("submit error");
        let first = loop {
            if let Some(result) = worker.try_take_latest() {
                break result;
            }
            thread::yield_now();
        };
        assert_eq!(first.generation.get(), 1);
        assert!(matches!(
            first.result,
            Err(RenderWorkerError::Command(CommandError::LayerNotFound(_)))
        ));
        worker
            .submit(RenderWorkPayload {
                request: test_render_request(Arc::clone(&document)),
                preview_command: Some(good),
            })
            .expect("submit recovery");
        worker.close();
        worker.join().expect("join");
        let result = worker.try_take_latest().expect("recovery result");
        assert_eq!(result.generation.get(), 2);
        assert!(result.result.is_ok());
        assert_eq!(serde_json::to_string(&document).expect("serialize"), before);
    }

    #[test]
    fn running_request_finishes_then_only_latest_pending_starts() {
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let mut first = true;
        let mut worker = LatestWorker::spawn(
            "u1b1-running-latest",
            move |value| {
                started_tx.send(value).expect("record start");
                if first {
                    first = false;
                    release_rx.recv().expect("release first");
                }
                Ok::<_, TestError>(value)
            },
            || TestError::Panicked,
        )
        .expect("spawn");

        worker.submit(1).expect("submit first");
        assert_eq!(started_rx.recv().expect("first start"), 1);
        for value in 2..=100 {
            worker.submit(value).expect("replace pending");
        }
        release_tx.send(()).expect("release first");
        worker.close();
        worker.join().expect("join");

        assert_eq!(started_rx.recv().expect("latest start"), 100);
        assert!(started_rx.try_recv().is_err());
        let result = worker.try_take_latest().expect("latest result");
        assert_eq!(result.generation.get(), 100);
    }

    #[test]
    fn generation_exhaustion_does_not_mutate_pending() {
        let (release_tx, release_rx) = mpsc::channel();
        let mut worker = LatestWorker::spawn_from_generation(
            "u1b1-generation-exhaustion",
            u64::MAX,
            Ok::<_, TestError>,
            || TestError::Panicked,
            Some(release_rx),
        )
        .expect("spawn");
        let last = worker.submit(7).expect("last generation");
        assert_eq!(last.get(), u64::MAX);
        assert_eq!(
            worker.submit(8),
            Err(RenderSubmitError::GenerationExhausted)
        );
        assert_eq!(worker.latest_accepted_generation(), Some(last));
        assert_eq!(worker.pending_generation(), Some(last));
        release_tx.send(()).expect("release");
        worker.close();
        worker.join().expect("join");
    }

    #[test]
    fn interaction_cancel_does_not_advance_real_worker_generation() {
        let mut worker = LatestWorker::spawn("u2c1-cancel-generation", Ok::<_, TestError>, || {
            TestError::Panicked
        })
        .expect("spawn");
        let baseline = worker.submit(1_u64).expect("baseline");
        for with_preview in [false, true] {
            let mut machine = InteractionStateMachine::new();
            machine.transition(InteractionState::Target).unwrap();
            if with_preview {
                machine.transition(InteractionState::Preview).unwrap();
            }
            machine.transition(InteractionState::Cancel).unwrap();
            machine.transition(InteractionState::Discover).unwrap();

            assert_eq!(worker.latest_accepted_generation(), Some(baseline));
        }
        let committed = worker.submit(2).expect("commit control");
        assert!(committed > baseline);
        assert_eq!(worker.latest_accepted_generation(), Some(committed));
        worker.close();
        worker.join().expect("join");
    }

    #[test]
    fn diagnostic_adaptation_does_not_advance_real_worker_generation() {
        let mut worker =
            LatestWorker::spawn("u2c4-diagnostic-generation", Ok::<_, TestError>, || {
                TestError::Panicked
            })
            .expect("spawn");
        let baseline = worker.submit(1_u64).expect("baseline");
        let missing = CommandId::try_new("motolii.missing.command").unwrap();

        let envelope =
            adapt_input_router_error(&InputRouterError::UnknownCommandId { id: missing });

        assert_eq!(envelope.reason(), DiagnosticReasonCode::UnknownCommand);
        assert_eq!(worker.latest_accepted_generation(), Some(baseline));
        let control = worker.submit(2).expect("control");
        assert!(control > baseline);
        assert_eq!(worker.latest_accepted_generation(), Some(control));
        worker.close();
        worker.join().expect("join");
    }

    #[test]
    fn panic_is_typed_and_records_abandoned_pending_generation() {
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let mut worker = LatestWorker::spawn(
            "u1b1-panic",
            move |value| -> Result<u64, TestError> {
                started_tx.send(value).expect("record start");
                release_rx.recv().expect("release panic");
                panic!("injected executor panic");
            },
            || TestError::Panicked,
        )
        .expect("spawn");
        let running = worker.submit(1).expect("running");
        assert_eq!(started_rx.recv().expect("started"), 1);
        let abandoned = worker.submit(2).expect("pending");
        release_tx.send(()).expect("release");

        assert_eq!(
            worker.join(),
            Err(RenderJoinError::WorkerPanicked {
                running_generation: running,
                abandoned_pending_generation: Some(abandoned),
            })
        );
        assert_eq!(
            worker.status(),
            RenderWorkerStatus::WorkerPanicked {
                running_generation: running,
                abandoned_pending_generation: Some(abandoned),
            }
        );
        assert_eq!(worker.submit(3), Err(RenderSubmitError::WorkerStopped));
        let result = worker.try_take_latest().expect("panic result");
        assert_eq!(result.generation, running);
        assert_eq!(result.result, Err(TestError::Panicked));
    }

    #[test]
    fn typed_job_error_does_not_stop_the_worker() {
        let mut worker = LatestWorker::spawn(
            "u1b1-job-error",
            |value| {
                if value == 1 {
                    Err(TestError::Job)
                } else {
                    Ok(value)
                }
            },
            || TestError::Panicked,
        )
        .expect("spawn");
        worker.submit(1).expect("submit error");
        while worker.try_take_latest().is_none() {
            thread::yield_now();
        }
        worker.submit(2).expect("submit recovery");
        worker.close();
        worker.join().expect("join");
        let result = worker.try_take_latest().expect("recovery result");
        assert_eq!(result.generation.get(), 2);
        assert_eq!(result.result, Ok(2));
    }

    #[test]
    fn real_document_uses_canonical_gpu_render_worker() {
        let Ok(gpu) = GpuCtx::new_headless() else {
            unavailable_dep("GPU adapter", "new_headless failed");
            return;
        };
        let document = Arc::new(bootstrap_document().expect("document"));
        let before = serde_json::to_string(&document).expect("serialize before");
        let desc = bootstrap_frame_desc().expect("frame desc");
        let mut worker = RenderWorker::spawn(Arc::new(gpu)).expect("spawn render worker");
        let generation = worker
            .submit(RenderRequest {
                document: Arc::clone(&document),
                data_tracks: Arc::new(DataTracks::new()),
                evaluation_time: EvaluationTime::new(motolii_core::RationalTime::ZERO),
                desc,
                quality: Quality::DRAFT,
            })
            .expect("submit");
        assert_eq!(worker.latest_accepted_generation(), Some(generation));
        worker.close();
        worker.join().expect("join");
        assert_eq!(worker.status(), RenderWorkerStatus::Closed);
        let result = worker.try_take_latest().expect("render result");
        assert_eq!(result.generation, generation);
        let rendered = result.result.expect("rendered frame");
        assert_eq!(rendered.frame.desc, Quality::DRAFT.render_desc(desc));
        assert_eq!(
            serde_json::to_string(&document).expect("serialize after"),
            before
        );
    }

    #[test]
    fn production_worker_has_no_ui_update_readback_or_direct_gpu_allocation() {
        let source = include_str!("render_worker.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source");
        for forbidden in [
            "egui::",
            "TextureId",
            "download_rgba",
            "device.poll",
            "create_texture",
            "create_buffer",
            "create_render_pipeline",
            "create_shader_module",
            "DisplaySlot",
            "video_sources: &[]",
        ] {
            assert!(
                !production.contains(forbidden),
                "production render worker contains forbidden token {forbidden}"
            );
        }
        assert!(
            production.contains("VideoSourceBinder"),
            "preview eval must pass export FrameReader textures into render_graph_cached"
        );
    }

    #[test]
    fn repaint_signal_covers_publish_before_and_after_registration() {
        let calls = Arc::new(AtomicU32::new(0));
        let mut worker = LatestWorker::spawn("u1b2-repaint-signal", Ok::<_, TestError>, || {
            TestError::Panicked
        })
        .expect("spawn");
        let client = worker.client();
        let first_calls = Arc::clone(&calls);
        client
            .register_repaint_signal(Arc::new(move || {
                first_calls.fetch_add(1, Ordering::Relaxed);
            }))
            .expect("register before publish");
        client.submit(1).expect("submit first");
        while calls.load(Ordering::Relaxed) == 0 {
            thread::yield_now();
        }
        client.try_take_latest().expect("first result");

        client.submit(2).expect("submit second");
        while client.shared.lock_result().is_none() {
            thread::yield_now();
        }
        let second_calls = Arc::clone(&calls);
        client
            .register_repaint_signal(Arc::new(move || {
                second_calls.fetch_add(1, Ordering::Relaxed);
            }))
            .expect("register with retained result");

        worker.close();
        worker.join().expect("join");
        assert!(calls.load(Ordering::Relaxed) >= 2);
    }

    #[test]
    fn panicking_repaint_signal_is_removed_and_fresh_epoch_recovers() {
        let panics = Arc::new(AtomicU32::new(0));
        let recovered = Arc::new(AtomicU32::new(0));
        let mut worker = LatestWorker::spawn("u1b2-repaint-panic", Ok::<_, TestError>, || {
            TestError::Panicked
        })
        .expect("spawn");
        let client = worker.client();
        let panic_calls = Arc::clone(&panics);
        let failed_epoch = client
            .register_repaint_signal(Arc::new(move || {
                panic_calls.fetch_add(1, Ordering::Relaxed);
                panic!("injected repaint panic");
            }))
            .expect("register panic");
        client.submit(1).expect("submit panic notification");
        while client.failed_repaint_signal_epoch() != Some(failed_epoch) {
            thread::yield_now();
        }
        assert_eq!(panics.load(Ordering::Relaxed), 1);

        let recovered_calls = Arc::clone(&recovered);
        client
            .register_repaint_signal(Arc::new(move || {
                recovered_calls.fetch_add(1, Ordering::Relaxed);
            }))
            .expect("register recovery");
        client.try_take_latest().expect("retained result");
        client.submit(2).expect("submit after recovery");
        worker.close();
        worker.join().expect("join");

        assert_eq!(panics.load(Ordering::Relaxed), 1);
        assert!(recovered.load(Ordering::Relaxed) >= 1);
        assert_eq!(
            client
                .try_take_latest()
                .expect("latest result")
                .generation
                .get(),
            2
        );
    }
}
