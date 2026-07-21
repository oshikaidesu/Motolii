//! egui shell起動と既存wgpu device共有。

use std::sync::{Arc, Mutex};

use motolii_core::{Quality, RationalTime};
use motolii_doc::{
    layer_names_for_item, Command, Document, DocumentWriter, EvaluationTime, ParentLocator,
};
use motolii_eval::DataTracks;
use motolii_gpu::GpuCtx;
use motolii_plugins_firstparty::first_party_catalog;

use crate::app::{AppPreviewRuntime, AppSmokeConfig, LifecycleSmokeOutcome, MotoliiApp};
use crate::document_edit_runtime::DocumentEditRuntime;
use crate::render_worker::{RenderJoinError, RenderRequest, RenderWorker};
use crate::static_preview::{
    bootstrap_document, bootstrap_document_for_edit_smoke, bootstrap_frame_desc,
    prepare_in_setup_worker, StaticPreviewError,
};
use crate::{DocumentCommandRequest, DomainIntent};

const LIFECYCLE_SMOKE_ENV: &str = "MOTOLII_TEST_U1A1_LIFECYCLE";
const LATEST_PREVIEW_SMOKE_ENV: &str = "MOTOLII_TEST_U1B2_LATEST";
const DOCUMENT_EDIT_SMOKE_ENV: &str = "MOTOLII_TEST_U2B1_DOCUMENT";

pub(crate) fn toolkit_linked() -> bool {
    std::mem::size_of::<egui::Context>() > 0
}

#[derive(Debug, thiserror::Error)]
pub enum ShellError {
    #[error(transparent)]
    Gpu(#[from] motolii_gpu::GpuError),
    #[error(transparent)]
    Preview(#[from] StaticPreviewError),
    #[error("app construction failed")]
    AppConstruction(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("eframe runtime failed")]
    Runtime(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("U1a-1 lifecycle outcome lock was poisoned")]
    LifecycleOutcomeLockPoisoned,
    #[error("U1a-1 lifecycle smoke failed: {reason}")]
    LifecycleSmokeFailed { reason: String },
    #[error("U2b-1 bootstrap fixture has no removable track item")]
    MissingDocumentEditFixture,
}

pub fn run_shell() -> Result<(), ShellError> {
    let lifecycle_smoke = std::env::var_os(LIFECYCLE_SMOKE_ENV).is_some();
    let latest_smoke = std::env::var_os(LATEST_PREVIEW_SMOKE_ENV).is_some();
    let document_edit_smoke = std::env::var_os(DOCUMENT_EDIT_SMOKE_ENV).is_some();
    let (gpu, parts) = GpuCtx::new_for_ui()?;
    let gpu = Arc::new(gpu);
    let document = if document_edit_smoke {
        bootstrap_document_for_edit_smoke()?
    } else {
        bootstrap_document()?
    };
    let document_edit_request = document_edit_smoke
        .then(|| bootstrap_delete_request(&document))
        .transpose()?;
    let catalog = first_party_catalog().map_err(|error| ShellError::Runtime(Box::new(error)))?;
    let writer = DocumentWriter::new(document, Arc::new(catalog))
        .map_err(|error| ShellError::Runtime(Box::new(error)))?;
    let document_runtime = DocumentEditRuntime::new(writer);
    let document = document_runtime.snapshot();
    let desc = bootstrap_frame_desc()?;
    let preview = Arc::new(prepare_in_setup_worker(
        Arc::clone(&gpu),
        Arc::clone(&document),
        desc,
    )?);
    let mut render_worker = RenderWorker::spawn(Arc::clone(&gpu))
        .map_err(|error| ShellError::Runtime(Box::new(error)))?;
    let render_client = render_worker.client();
    let initial_request = RenderRequest {
        document,
        data_tracks: Arc::new(DataTracks::new()),
        evaluation_time: EvaluationTime::new(RationalTime::ZERO),
        desc,
        quality: Quality::DRAFT,
    };
    let smoke_outcome = Arc::new(Mutex::new(LifecycleSmokeOutcome::NotRequested));

    let mut native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Motolii")
            .with_inner_size([960.0, 640.0]),
        ..Default::default()
    };
    native_options.wgpu_options.wgpu_setup =
        egui_wgpu::WgpuSetup::Existing(egui_wgpu::WgpuSetupExisting {
            instance: parts.instance,
            adapter: parts.adapter,
            device: parts.device,
            queue: parts.queue,
        });

    let app_outcome = Arc::clone(&smoke_outcome);
    let run_result = eframe::run_native(
        "Motolii",
        native_options,
        Box::new(move |cc| {
            MotoliiApp::new(
                cc,
                AppPreviewRuntime {
                    preview: Arc::clone(&preview),
                    gpu: Arc::clone(&gpu),
                    render_client: render_client.clone(),
                    document_runtime,
                    initial_request: RenderRequest {
                        document: Arc::clone(&initial_request.document),
                        data_tracks: Arc::clone(&initial_request.data_tracks),
                        evaluation_time: initial_request.evaluation_time,
                        desc: initial_request.desc,
                        quality: initial_request.quality,
                    },
                },
                AppSmokeConfig {
                    lifecycle: lifecycle_smoke,
                    latest_preview: latest_smoke,
                    document_edit: document_edit_request,
                    outcome: Arc::clone(&app_outcome),
                },
            )
            .map(|app| Box::new(app) as Box<dyn eframe::App>)
            .map_err(|error| -> Box<dyn std::error::Error + Send + Sync> { Box::new(error) })
        }),
    );
    render_worker.close();
    let join_result = render_worker.join();
    match (run_result, join_result) {
        (Ok(()), Ok(())) => {
            eprintln!("U1B2_JOIN passed after_run_native=true");
        }
        (Err(runtime), Ok(())) => return Err(map_eframe_error(runtime)),
        (Ok(()), Err(join)) => return Err(ShellError::Runtime(Box::new(join))),
        (Err(runtime), Err(join)) => {
            return Err(ShellError::Runtime(Box::new(CombinedRuntimeError {
                runtime: Box::new(runtime),
                join,
            })));
        }
    }

    let outcome = smoke_outcome
        .lock()
        .map_err(|_| ShellError::LifecycleOutcomeLockPoisoned)?
        .clone();
    match outcome {
        LifecycleSmokeOutcome::Failed(reason) => Err(ShellError::LifecycleSmokeFailed { reason }),
        LifecycleSmokeOutcome::NotRequested | LifecycleSmokeOutcome::Passed => Ok(()),
    }
}

fn bootstrap_delete_request(document: &Document) -> Result<DocumentCommandRequest, ShellError> {
    let track = document
        .tracks
        .first()
        .ok_or(ShellError::MissingDocumentEditFixture)?;
    let item = track
        .items
        .first()
        .cloned()
        .ok_or(ShellError::MissingDocumentEditFixture)?;
    let layer_names = layer_names_for_item(document, &item)
        .map_err(|error| ShellError::Runtime(Box::new(error)))?;
    DocumentCommandRequest::try_new(
        DomainIntent::DeleteTargetedItems,
        vec![Command::RemoveTrackItem {
            parent: ParentLocator::Track(track.id),
            index: 0,
            item,
            layer_names,
        }],
    )
    .map_err(|error| ShellError::Runtime(Box::new(error)))
}

fn map_eframe_error(error: eframe::Error) -> ShellError {
    match error {
        eframe::Error::AppCreation(source) => ShellError::AppConstruction(source),
        other => ShellError::Runtime(Box::new(other)),
    }
}

#[derive(Debug, thiserror::Error)]
#[error("eframe runtime and render worker shutdown both failed: {join}")]
struct CombinedRuntimeError {
    #[source]
    runtime: Box<eframe::Error>,
    join: RenderJoinError,
}
