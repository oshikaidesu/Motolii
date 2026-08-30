
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use motolii_core::{FrameDesc, PixelFormat, RationalTime};
use crate::render::engine::{Engine, EngineError};
use crate::render::media::{Encoder, MediaError};
use motolii_store::StoreView;

mod lottie;
pub use lottie::{export_lottie, LottieExport, LottieExportError, UnsupportedForLottie};

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error(transparent)]
    Engine(#[from] EngineError),
    #[error(transparent)]
    Media(#[from] MediaError),
    #[error("frame 記述を作れない: {0}")]
    Desc(String),
    #[error("中断された(残骸は消してある)")]
    Cancelled,
    #[error("comp の設定が Document に無い")]
    NoComposition,
    #[error("静止画の書き出しに失敗: {0}")]
    Still(String),
}

pub struct ExportJob {
    pub out_path: PathBuf,
    pub qp0: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportReport {
    pub out_path: PathBuf,
    pub frames_written: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportProgress {
    pub frames_done: i64,
    pub frames_total: i64,
}

#[derive(Clone, Default)]
pub struct Cancel(Arc<AtomicBool>);

impl Cancel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

pub fn export(
    engine: &mut Engine,
    view: &StoreView<'_>,
    job: &ExportJob,
) -> Result<ExportReport, ExportError> {
    export_with_cancel(engine, view, job, &Cancel::new())
}

pub fn export_with_cancel(
    engine: &mut Engine,
    view: &StoreView<'_>,
    job: &ExportJob,
    cancel: &Cancel,
) -> Result<ExportReport, ExportError> {
    let duration_frames = composition_duration_frames(view)?;
    export_range_with_cancel(engine, view, job, 0..duration_frames, cancel)
}

pub fn export_range(
    engine: &mut Engine,
    view: &StoreView<'_>,
    job: &ExportJob,
    range: Range<i64>,
) -> Result<ExportReport, ExportError> {
    export_range_with_cancel(engine, view, job, range, &Cancel::new())
}

pub fn export_range_with_cancel(
    engine: &mut Engine,
    view: &StoreView<'_>,
    job: &ExportJob,
    range: Range<i64>,
    cancel: &Cancel,
) -> Result<ExportReport, ExportError> {
    export_range_with_progress(engine, view, job, range, cancel, |_| {})
}

pub fn export_with_progress(
    engine: &mut Engine,
    view: &StoreView<'_>,
    job: &ExportJob,
    cancel: &Cancel,
    on_progress: impl FnMut(ExportProgress),
) -> Result<ExportReport, ExportError> {
    let duration_frames = composition_duration_frames(view)?;
    export_range_with_progress(engine, view, job, 0..duration_frames, cancel, on_progress)
}

pub fn export_range_with_progress(
    engine: &mut Engine,
    view: &StoreView<'_>,
    job: &ExportJob,
    range: Range<i64>,
    cancel: &Cancel,
    mut on_progress: impl FnMut(ExportProgress),
) -> Result<ExportReport, ExportError> {
    let composition = view
        .composition()
        .map_err(|e| ExportError::Desc(e.to_string()))?
        .ok_or(ExportError::NoComposition)?;
    let comp = composition.spec();
    let fps = composition.fps;

    let desc = FrameDesc::try_packed(
        comp.width,
        comp.height,
        PixelFormat::Rgba8Unorm,
        motolii_core::ColorSpace::Srgb,
        true,
    )
    .map_err(|e| ExportError::Desc(e.to_string()))?;

    let frames_total = (range.end - range.start).max(0);
    let mut encoder = Encoder::open(&job.out_path, &desc, fps, job.qp0)?;
    let mut written = 0i64;

    for frame in range {
        if cancel.is_cancelled() {
            drop(encoder);
            remove_partial(&job.out_path);
            return Err(ExportError::Cancelled);
        }

        let t = RationalTime::try_from_frame(frame, fps)
            .map_err(|e| ExportError::Desc(e.to_string()))?;
        let rgba = engine.render_frame(view, t)?;
        encoder.write_frame(&rgba)?;
        written += 1;
        on_progress(ExportProgress {
            frames_done: written,
            frames_total,
        });
    }

    encoder.finish()?;

    Ok(ExportReport {
        out_path: job.out_path.clone(),
        frames_written: written,
    })
}

pub fn export_still(
    engine: &mut Engine,
    view: &StoreView<'_>,
    frame: i64,
    out_path: &Path,
) -> Result<ExportReport, ExportError> {
    let composition = view
        .composition()
        .map_err(|e| ExportError::Desc(e.to_string()))?
        .ok_or(ExportError::NoComposition)?;
    let comp = composition.spec();
    let fps = composition.fps;

    let t = RationalTime::try_from_frame(frame, fps)
        .map_err(|e| ExportError::Desc(e.to_string()))?;
    let rgba = engine.render_frame(view, t)?;

    image::save_buffer(out_path, &rgba, comp.width, comp.height, image::ColorType::Rgba8)
        .map_err(|e| ExportError::Still(e.to_string()))?;

    Ok(ExportReport {
        out_path: out_path.to_path_buf(),
        frames_written: 1,
    })
}

fn composition_duration_frames(view: &StoreView<'_>) -> Result<i64, ExportError> {
    let composition = view
        .composition()
        .map_err(|e| ExportError::Desc(e.to_string()))?
        .ok_or(ExportError::NoComposition)?;
    Ok(composition.duration_frames)
}

fn remove_partial(path: &Path) {
    let _ = std::fs::remove_file(path);
}
