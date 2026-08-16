//! toolkit linkの照合と、現在も公開しているshell error型。

use crate::static_preview::StaticPreviewError;

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
    #[error("project session could not be opened")]
    ProjectSession(#[from] motolii_doc::SessionError),
}
