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
    #[error("Could not read the frame to export. {0}")]
    Desc(String),
    #[error("Could not write the export file. {0}")]
    Write(String),
    #[error("Export cancelled. The partial file was removed.")]
    Cancelled,
    #[error("This document has no composition to export. Create a composition first.")]
    NoComposition,
    #[error("Could not export the still image. {0}")]
    Still(String),
    #[error("The encoder finished but its file is not the export: {0}. Nothing was written.")]
    Unverified(String),
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

    pub fn is_cancelled(&self) -> bool {
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

    // a cancel still stops ffmpeg while it flushes its tail
    match encoder.finish_unless(|| cancel.is_cancelled()) {
        Err(MediaError::Cancelled) => return Err(ExportError::Cancelled),
        other => other?,
    }
    // exit 0 does not prove the stream: the file must be the export before it replaces anything (GAP-26)
    verify_output(&output.path, &desc, fps, written, audio.is_some())?;
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

/// The encoded file is the export: one video stream at the frame size, an audio stream when audio was muxed, and as
/// long as the frames written (within a frame). Checked with ffprobe on the temporary file, before it is installed.
pub(crate) fn verify_output(path: &Path, desc: &FrameDesc, fps: crate::doc::core::Fps, frames: i64, audio: bool) -> Result<(), ExportError> {
    let info = crate::render::media::probe_container(path).map_err(|e| ExportError::Unverified(e.to_string()))?;
    let video = info.video_streams.first().ok_or_else(|| ExportError::Unverified("no video stream".into()))?;
    if (video.width, video.height) != (desc.width, desc.height) {
        return Err(ExportError::Unverified(format!("{}×{} instead of {}×{}", video.width, video.height, desc.width, desc.height)));
    }
    if audio && info.audio_streams.is_empty() {
        return Err(ExportError::Unverified("the audio stream is missing".into()));
    }
    if let Some(duration) = video.duration.or(info.duration) {
        let got = duration.try_to_frame_round(fps).map_err(|e| ExportError::Unverified(e.to_string()))?;
        if (got - frames).abs() > 1 {
            return Err(ExportError::Unverified(format!("{got} frames instead of {frames}")));
        }
    }
    Ok(())
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
                Err(error) => return Err(ExportError::Write(error.to_string())),
            }
        }
        Err(ExportError::Write(
            "Could not create the export file. Check that the destination folder is writable.".to_owned(),
        ))
    }

    fn commit(mut self) -> Result<(), ExportError> {
        std::fs::rename(&self.path, &self.destination)
            .map_err(|error| ExportError::Write(error.to_string()))?;
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
    let program = AudioProgram::from_view_blocking(view, &mut cache)?;
    if program.sources().is_empty() || range.end <= range.start {
        return Ok(None);
    }

    let start = RationalTime::try_from_frame(range.start, fps)
        .map_err(|error| ExportError::Write(error.to_string()))?;
    let end = RationalTime::try_from_frame(range.end, fps)
        .map_err(|error| ExportError::Write(error.to_string()))?;
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
            .map_err(|error| ExportError::Write(error.to_string()))?;
        cursor += count as u64;
    }
    file.flush()
        .and_then(|_| file.sync_all())
        .map_err(|error| ExportError::Write(error.to_string()))?;
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
            Err(error) => return Err(ExportError::Write(error.to_string())),
        }
    }
    Err(ExportError::Write(
        "Could not create the temporary audio file. Check free space in the system temporary folder.".to_owned(),
    ))
}

#[cfg(test)]
mod verify_tests {
    use super::*;
    use crate::doc::core::{ColorSpace, Fps};

    fn scratch(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("motolii-verify-{}-{name}", std::process::id()))
    }

    /// An encoded file passes only as the export it should be; a file an encoder left behind with exit 0 but the
    /// wrong frames, size or nothing at all does not (and so is never installed over the user's file).
    #[test]
    fn only_the_export_that_was_asked_for_is_accepted() {
        if !crate::render::media::tools_available() {
            eprintln!("skip: ffmpeg/ffprobe not available");
            return;
        }
        let desc = FrameDesc::try_packed(64, 32, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false).unwrap();
        let fps = Fps::try_new(30, 1).unwrap();
        let path = scratch("ok.mp4");
        let mut encoder = Encoder::open(&path, &desc, fps, false).unwrap();
        for i in 0..12u8 {
            encoder.write_frame(&vec![i * 20; desc.data_size()]).unwrap();
        }
        encoder.finish().unwrap();

        assert!(verify_output(&path, &desc, fps, 12, false).is_ok());
        assert!(matches!(verify_output(&path, &desc, fps, 40, false), Err(ExportError::Unverified(_))), "frames");
        let other = FrameDesc::try_packed(32, 32, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false).unwrap();
        assert!(matches!(verify_output(&path, &other, fps, 12, false), Err(ExportError::Unverified(_))), "size");
        assert!(matches!(verify_output(&path, &desc, fps, 12, true), Err(ExportError::Unverified(_))), "audio");

        let junk = scratch("junk.mp4");
        std::fs::write(&junk, b"not a movie").unwrap();
        assert!(matches!(verify_output(&junk, &desc, fps, 12, false), Err(ExportError::Unverified(_))), "junk");
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&junk);
    }
}
