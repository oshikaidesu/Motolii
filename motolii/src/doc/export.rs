use std::io::Write;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::doc::core::{FrameDesc, PixelFormat, RationalTime};
use crate::doc::store::StoreView;
use crate::render::audio::{time_to_canonical_frames, AudioError, AudioProgram, AudioProgramCache};
use crate::render::engine::{Engine, EngineError};
use crate::render::media::{Encoder, MediaError};

mod lottie;
pub use lottie::{export_lottie, LottieExport, LottieExportError, UnsupportedForLottie};

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error(transparent)]
    Engine(#[from] EngineError),
    #[error(transparent)]
    Media(#[from] MediaError),
    #[error(transparent)]
    Audio(#[from] AudioError),
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
        crate::doc::core::ColorSpace::Srgb,
        true,
    )
    .map_err(|e| ExportError::Desc(e.to_string()))?;

    let frames_total = (range.end - range.start).max(0);
    let output = TempOutput::reserve(&job.out_path)?;
    let audio = render_audio_input(view, &range, fps, cancel)?;
    let mut encoder = match audio.as_ref() {
        Some(audio) => Encoder::open_with_audio(&output.path, &desc, fps, job.qp0, &audio.path)?,
        None => Encoder::open(&output.path, &desc, fps, job.qp0)?,
    };
    let mut written = 0i64;

    for frame in range {
        if cancel.is_cancelled() {
            drop(encoder);
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
    output.commit()?;

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

    let t =
        RationalTime::try_from_frame(frame, fps).map_err(|e| ExportError::Desc(e.to_string()))?;
    let rgba = engine.render_frame(view, t)?;

    image::save_buffer(
        out_path,
        &rgba,
        comp.width,
        comp.height,
        image::ColorType::Rgba8,
    )
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

struct TempAudioInput {
    path: PathBuf,
}

impl Drop for TempAudioInput {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

struct TempOutput {
    path: PathBuf,
    destination: PathBuf,
    committed: bool,
}

impl TempOutput {
    fn reserve(destination: &Path) -> Result<Self, ExportError> {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let dir = destination
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let stem = destination
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("movie");
        let extension = destination
            .extension()
            .and_then(|extension| extension.to_str());

        for _ in 0..32 {
            let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let name = match extension {
                Some(extension) => {
                    format!(".{stem}.export-{}-{id}.{extension}", std::process::id())
                }
                None => format!(".{stem}.export-{}-{id}", std::process::id()),
            };
            let path = dir.join(name);
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(file) => {
                    drop(file);
                    return Ok(Self {
                        path,
                        destination: destination.to_path_buf(),
                        committed: false,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(ExportError::Desc(error.to_string())),
            }
        }
        Err(ExportError::Desc(
            "could not reserve an export output".to_owned(),
        ))
    }

    fn commit(mut self) -> Result<(), ExportError> {
        std::fs::rename(&self.path, &self.destination)
            .map_err(|error| ExportError::Desc(error.to_string()))?;
        self.committed = true;
        Ok(())
    }
}

impl Drop for TempOutput {
    fn drop(&mut self) {
        if !self.committed {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

fn render_audio_input(
    view: &StoreView<'_>,
    range: &Range<i64>,
    fps: crate::doc::core::Fps,
    cancel: &Cancel,
) -> Result<Option<TempAudioInput>, ExportError> {
    let mut cache = AudioProgramCache::default();
    let program = AudioProgram::from_view(view, &mut cache)?;
    if program.sources().is_empty() || range.end <= range.start {
        return Ok(None);
    }

    let start = RationalTime::try_from_frame(range.start, fps)
        .map_err(|error| ExportError::Desc(error.to_string()))?;
    let end = RationalTime::try_from_frame(range.end, fps)
        .map_err(|error| ExportError::Desc(error.to_string()))?;
    let mut cursor = time_to_canonical_frames(start);
    let end_frame = time_to_canonical_frames(end);
    if end_frame <= cursor {
        return Ok(None);
    }

    let (mut file, path) = reserve_audio_temp()?;
    let input = TempAudioInput { path };
    const CHUNK_FRAMES: usize = 48_000;
    while cursor < end_frame {
        if cancel.is_cancelled() {
            return Err(ExportError::Cancelled);
        }
        let count = (end_frame - cursor).min(CHUNK_FRAMES as u64) as usize;
        let (samples, _) = program.mix_audio(cursor, count, None)?;
        let mut bytes = Vec::with_capacity(samples.len() * std::mem::size_of::<f32>());
        for sample in samples {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
        file.write_all(&bytes)
            .map_err(|error| ExportError::Desc(error.to_string()))?;
        cursor += count as u64;
    }
    file.flush()
        .and_then(|_| file.sync_all())
        .map_err(|error| ExportError::Desc(error.to_string()))?;
    drop(file);
    Ok(Some(input))
}

fn reserve_audio_temp() -> Result<(std::fs::File, PathBuf), ExportError> {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    for _ in 0..32 {
        let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "motolii-export-audio-{}-{id}.f32le",
            std::process::id()
        ));
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(file) => return Ok((file, path)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(ExportError::Desc(error.to_string())),
        }
    }
    Err(ExportError::Desc(
        "could not reserve an audio export input".to_owned(),
    ))
}
