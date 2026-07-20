//! egui shell起動と既存wgpu device共有。

use std::sync::{Arc, Mutex};

use motolii_gpu::GpuCtx;

use crate::app::{LifecycleSmokeOutcome, MotoliiApp};
use crate::static_preview::{
    bootstrap_document, bootstrap_frame_desc, prepare_in_setup_worker, StaticPreviewError,
};

const LIFECYCLE_SMOKE_ENV: &str = "MOTOLII_TEST_U1A1_LIFECYCLE";

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
}

pub fn run_shell() -> Result<(), ShellError> {
    let lifecycle_smoke = std::env::var_os(LIFECYCLE_SMOKE_ENV).is_some();
    let (gpu, parts) = GpuCtx::new_for_ui()?;
    let gpu = Arc::new(gpu);
    let document = bootstrap_document()?;
    let desc = bootstrap_frame_desc()?;
    let preview = Arc::new(prepare_in_setup_worker(
        Arc::clone(&gpu),
        Arc::new(document),
        desc,
    )?);
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
    eframe::run_native(
        "Motolii",
        native_options,
        Box::new(move |cc| {
            MotoliiApp::new(
                cc,
                Arc::clone(&preview),
                lifecycle_smoke,
                Arc::clone(&app_outcome),
            )
            .map(|app| Box::new(app) as Box<dyn eframe::App>)
            .map_err(|error| -> Box<dyn std::error::Error + Send + Sync> { Box::new(error) })
        }),
    )
    .map_err(|error| match error {
        eframe::Error::AppCreation(source) => ShellError::AppConstruction(source),
        other => ShellError::Runtime(Box::new(other)),
    })?;

    let outcome = smoke_outcome
        .lock()
        .map_err(|_| ShellError::LifecycleOutcomeLockPoisoned)?
        .clone();
    match outcome {
        LifecycleSmokeOutcome::Failed(reason) => Err(ShellError::LifecycleSmokeFailed { reason }),
        LifecycleSmokeOutcome::NotRequested | LifecycleSmokeOutcome::Passed => Ok(()),
    }
}
