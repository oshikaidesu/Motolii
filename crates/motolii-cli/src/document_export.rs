//! Document JSON経由の書き出し(D3)。
use crate::CliError;
use motolii_doc::{open_project_resolved, ResourceLimits};
use motolii_eval::DataTracks;
use motolii_export::{export_document_video, ExportJob, ExportReport};
use motolii_gpu::GpuCtx;
use motolii_plugins_firstparty::first_party_runtime;
use std::path::Path;

pub fn export_document_file(
    gpu: &GpuCtx,
    doc_path: &Path,
    output_path: &Path,
    frame_count: Option<usize>,
    qp0: bool,
) -> Result<ExportReport, CliError> {
    let runtime = first_party_runtime()?;
    let opened = open_project_resolved(doc_path, &ResourceLimits::production(), runtime.catalog())
        .map_err(|e| CliError::Usage(e.to_string()))?;
    let doc = opened.recovered.document;
    export_document_video(
        gpu,
        &ExportJob {
            doc: &doc,
            runtime: &runtime,
            output_path,
            project_root: doc_path.parent(),
            frame_count,
            qp0,
            data_tracks: DataTracks::new(),
        },
    )
    .map_err(|e| CliError::Usage(e.to_string()))
}
