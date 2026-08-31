
mod encode;
mod mesh;
mod point_cloud;
mod probe;

use std::io::Read;
use std::process::Command;

pub use encode::Encoder;
pub use mesh::{is_mesh_extension, is_mesh_path, load_mesh, MeshData, MeshError, MESH_EXTENSIONS};
pub use point_cloud::{
    asset_type_for_extension, is_point_cloud_extension, is_point_cloud_path, is_rerun_importable_extension,
    load_point_cloud, PointCloudData,
    PointCloudError, POINT_CLOUD_EXTENSIONS,
};
pub use probe::{
    probe, probe_container, require_supported_audio, select_audio_stream, select_video_stream,
    ContainerInfo, MediaInfo, MediaStreamKind, ProbedAudioStream, ProbedVideoStream,
};

#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    #[error("ffmpeg/ffprobe not found on PATH: {0}")]
    ToolNotFound(&'static str),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("probe failed: {0}")]
    Probe(String),
    #[error(transparent)]
    Fingerprint(#[from] crate::doc::store::SourceFingerprintError),
    #[error("media stream not found: kind={kind}, ordinal={ordinal}")]
    StreamNotFound { kind: MediaStreamKind, ordinal: u32 },
    #[error("unsupported audio codec `{codec}` (audio ordinal {ordinal})")]
    UnsupportedAudioCodec { ordinal: u32, codec: String },
    #[error("unsupported audio channel layout `{layout}` (audio ordinal {ordinal})")]
    UnsupportedChannelLayout { ordinal: u32, layout: String },
    #[error(transparent)]
    RationalTime(#[from] crate::doc::core::RationalTimeError),
    #[error("invalid start frame: {0}")]
    InvalidStartFrame(i64),
    #[error("soundtrack start_offset must be >= 0, got {0:?}")]
    InvalidStartOffset(crate::doc::core::RationalTime),
    #[error("soundtrack master_gain must be finite and in [0, 1], got {0}")]
    InvalidMasterGain(f64),
    #[error("encoder expects RGBA input, got {0:?}")]
    UnsupportedEncoderFormat(crate::doc::core::PixelFormat),
    #[error("frame size mismatch: expected {expected} bytes, got {got}")]
    FrameSizeMismatch { expected: usize, got: usize },
    #[error("ffmpeg failed: {0}")]
    Ffmpeg(String),
    #[error("frame read cancelled")]
    Cancelled,
}

pub type Result<T> = std::result::Result<T, MediaError>;

const MAX_STDERR_BYTES: usize = 64 * 1024;

pub(crate) fn read_child_stderr(stderr: &mut impl Read) -> std::io::Result<String> {
    let mut out = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        match stderr.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                if out.len() < MAX_STDERR_BYTES {
                    let take = (MAX_STDERR_BYTES - out.len()).min(n);
                    out.extend_from_slice(&chunk[..take]);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(String::from_utf8_lossy(&out).into_owned())
}

pub fn tools_available() -> bool {
    let ok = |bin: &str| {
        Command::new(bin)
            .arg("-version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    };
    ok("ffmpeg") && ok("ffprobe")
}

pub const MIN_FFMPEG_MAJOR: u32 = 6;

pub fn verify_tool_versions() -> Result<(u32, u32)> {
    let major = |bin: &'static str| -> Result<u32> {
        let out = Command::new(bin).arg("-version").output().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                MediaError::ToolNotFound(bin)
            } else {
                MediaError::Io(e)
            }
        })?;
        let text = String::from_utf8_lossy(&out.stdout);
        let tok = text
            .split_whitespace()
            .nth(2)
            .unwrap_or("")
            .trim_start_matches(['n', 'N']);
        let digits: String = tok.chars().take_while(|c| c.is_ascii_digit()).collect();
        match digits.parse::<u32>() {
            Ok(major) if major >= 1000 => {
                eprintln!(
                    "warning: {bin} version '{tok}' looks like a snapshot build; \
                     assuming >= {MIN_FFMPEG_MAJOR}"
                );
                Ok(0)
            }
            Ok(major) if major < MIN_FFMPEG_MAJOR => Err(MediaError::Probe(format!(
                "{bin} major version {major} < required {MIN_FFMPEG_MAJOR}"
            ))),
            Ok(major) => Ok(major),
            Err(_) => {
                eprintln!(
                    "warning: {bin} version '{tok}' is unparsable; \
                     assuming >= {MIN_FFMPEG_MAJOR}"
                );
                Ok(0)
            }
        }
    };
    Ok((major("ffmpeg")?, major("ffprobe")?))
}
