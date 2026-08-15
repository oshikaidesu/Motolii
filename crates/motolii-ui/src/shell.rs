//! プロジェクトを開いて編集ランタイムを作る。
//!
//! **旧 egui アプリの起動口(`run_shell` / `run_shell_with_project` / `run_shell_inner`)は
//! 畳んだ。** 製品 Timeline は Skia(`timeline_skia_raster`)で、旧 egui アプリは
//! 移行の途中で止まった残骸だった(2026-08-16 利用者裁定)。
//! 残っているのは製品(`rn_product_host`)が呼ぶ `open_project_runtime` と `ShellError`。

use std::path::Path;
use std::sync::Arc;

use motolii_doc::{DocumentWriter, ProjectSession, ResourceLimits};
use motolii_plugins_firstparty::first_party_catalog;

use crate::document_edit_runtime::DocumentEditRuntime;
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

pub(crate) fn open_project_runtime(project_path: &Path) -> Result<DocumentEditRuntime, ShellError> {
    let limits = ResourceLimits::production();
    let (session, opened) = ProjectSession::open(project_path, &limits)?;
    let catalog =
        Arc::new(first_party_catalog().map_err(|error| ShellError::Runtime(Box::new(error)))?);
    let writer = DocumentWriter::new(opened.document, Arc::clone(&catalog))
        .map_err(|error| ShellError::Runtime(Box::new(error)))?;
    Ok(DocumentEditRuntime::new(session, writer, catalog))
}
