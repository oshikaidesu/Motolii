//! Document JSON経由の書き出し(D3)。
use crate::CliError;
use motolii_doc::load_document;
use motolii_eval::DataTracks;
use motolii_export::{export_document_video, ExportJob, ExportReport};
use motolii_gpu::GpuCtx;
use std::path::Path;

pub fn export_document_file(
    gpu: &GpuCtx,
    doc_path: &Path,
    output_path: &Path,
    frame_count: Option<usize>,
    qp0: bool,
) -> Result<ExportReport, CliError> {
    let doc = load_document(doc_path).map_err(|e| CliError::Usage(e.to_string()))?;
    doc.validate().map_err(|e| CliError::Usage(e.to_string()))?;
    export_document_video(
        gpu,
        &ExportJob {
            doc: &doc,
            output_path,
            project_root: doc_path.parent(),
            frame_count,
            qp0,
            data_tracks: DataTracks::new(),
        },
    )
    .map_err(|e| CliError::Usage(e.to_string()))
}
