//! R9(B-4): 書き出しmp4と`render_frame`経路のピクセル一致検証。

use std::path::Path;

use motolii_gpu::GpuCtx;

use crate::project::{export_project_v1, prepare_project_export, PreparedProject, ProjectError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct B4FrameResult {
    pub export_index: usize,
    pub max_abs_diff: u32,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct B4VerifyReport {
    pub frames_checked: usize,
    pub frames_passed: usize,
    pub frame_results: Vec<B4FrameResult>,
    pub export_frames: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum B4VerifyError {
    #[error(transparent)]
    Project(#[from] ProjectError),
    #[error("no frames to verify")]
    EmptyExport,
    #[error("output file missing: {0} (run with --export or export-project first)")]
    OutputMissing(String),
    #[error(
        "B-4 verify failed: {failed}/{total} frames exceeded tolerance {tolerance}"
    )]
    Mismatch {
        report: B4VerifyReport,
        tolerance: u32,
        failed: usize,
        total: usize,
    },
}

/// 先頭・中間・末尾の3フレームで書き出しとレンダ経路を比較する。
pub fn verify_b4_project_v1(
    gpu: &GpuCtx,
    project_path: impl AsRef<Path>,
    tolerance: u32,
    do_export: bool,
) -> Result<B4VerifyReport, B4VerifyError> {
    let project_path = project_path.as_ref();
    if do_export {
        export_project_v1(gpu, project_path)?;
    } else {
        let prepared = prepare_project_export(project_path)?;
        if !prepared.output_path.is_file() {
            return Err(B4VerifyError::OutputMissing(
                prepared.output_path.display().to_string(),
            ));
        }
    }

    let prepared = prepare_project_export(project_path)?;
    verify_prepared_b4(gpu, &prepared, tolerance)
}

pub fn verify_prepared_b4(
    gpu: &GpuCtx,
    prepared: &PreparedProject,
    tolerance: u32,
) -> Result<B4VerifyReport, B4VerifyError> {
    if prepared.export_frames == 0 {
        return Err(B4VerifyError::EmptyExport);
    }

    let indices = sample_indices(prepared.export_frames);
    let mut session = motolii_render::RenderSession::new(gpu);
    let mut yuv = motolii_gpu::YuvToRgba::new(gpu);
    let mut downloader = motolii_gpu::RgbaDownloader::new();

    let mut frame_results = Vec::with_capacity(indices.len());
    for export_index in indices {
        let preview = prepared.render_export_frame_rgba(
            gpu,
            export_index,
            &mut session,
            &mut yuv,
            &mut downloader,
        )?;
        let exported = prepared.decode_exported_frame_rgba(
            gpu,
            export_index,
            &mut yuv,
            &mut downloader,
        )?;
        let max_abs_diff = max_rgba_diff(&preview, &exported);
        frame_results.push(B4FrameResult {
            export_index,
            max_abs_diff,
            passed: max_abs_diff <= tolerance,
        });
    }

    let frames_passed = frame_results.iter().filter(|r| r.passed).count();
    let report = B4VerifyReport {
        frames_checked: frame_results.len(),
        frames_passed,
        frame_results,
        export_frames: prepared.export_frames,
    };

    if frames_passed != report.frames_checked {
        return Err(B4VerifyError::Mismatch {
            failed: report.frames_checked - frames_passed,
            total: report.frames_checked,
            tolerance,
            report,
        });
    }

    Ok(report)
}

fn sample_indices(export_frames: usize) -> Vec<usize> {
    if export_frames == 1 {
        return vec![0];
    }
    let mid = export_frames / 2;
    let last = export_frames - 1;
    let mut indices = vec![0, mid, last];
    indices.sort_unstable();
    indices.dedup();
    indices
}

fn max_rgba_diff(a: &[u8], b: &[u8]) -> u32 {
    assert_eq!(a.len(), b.len());
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| x.abs_diff(*y) as u32)
        .max()
        .unwrap_or(0)
}
