
mod encode;
mod mesh;
mod point_cloud;
mod probe;
mod spatial;

use std::io::Read;
use std::process::Command;

pub use encode::Encoder;
pub use mesh::{
    is_mesh_extension, is_mesh_path, load_mesh_bounds, MeshError, MESH_EXTENSIONS,
};
pub use crate::render::engine::decode_still_srgb;
pub use point_cloud::{
    is_point_cloud_extension, is_point_cloud_path, is_rerun_importable_extension,
    load_point_cloud, PointCloudData,
    PointCloudError, POINT_CLOUD_EXTENSIONS,
    is_still_image_path,
};
pub use spatial::{SpatialBounds, SpatialBoundsError};

/// 素材棚が受け入れる拡張子の唯一の分類口。
/// 映像・画像・3DはRerun importer、音声はpin済みSymphoniaのfeatureに合わせる。
pub const AUDIO_EXTENSIONS: &[&str] = &[
    "aac", "aif", "aiff", "flac", "m4a", "mp1", "mp2", "mp3", "oga", "ogg", "wav",
];
/// ISO BMFF の器。demux は re_video の mp4 reader 1 本で、mov/m4v も同じ箱(実測)。
pub const VIDEO_EXTENSIONS: &[&str] = &["mp4", "m4v", "mov"];
/// 空として置く画(1.0 超を持つ形式)。置いた時は最初から環境層(裁定 2026-09-07 利用者: 「入れたら背景にならなかった」)。
pub const ENVIRONMENT_EXTENSIONS: &[&str] = &["hdr", "exr"];

pub fn is_audio_path(path: impl AsRef<std::path::Path>) -> bool {
    path.as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .is_some_and(|e| AUDIO_EXTENSIONS.contains(&e.as_str()))
}

pub fn is_environment_image_path(path: impl AsRef<std::path::Path>) -> bool {
    path.as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .is_some_and(|e| ENVIRONMENT_EXTENSIONS.contains(&e.as_str()))
}

/// 門で受ける拡張子の全部。OS の選択窓・drop・Browser の門はこれ一本を読む。
/// ここに在る物は `asset_type_for_extension` が必ず種別を返す(test で縛る)。
pub fn import_extensions() -> Vec<&'static str> {
    AUDIO_EXTENSIONS
        .iter()
        .chain(VIDEO_EXTENSIONS)
        .copied()
        .chain(point_cloud::image_extensions())
        .chain(re_importer::SUPPORTED_POINT_CLOUD_EXTENSIONS.iter().copied())
        .chain(MESH_EXTENSIONS.iter().copied())
        .collect()
}

pub fn asset_type_for_extension(extension: &str) -> Option<String> {
    let extension = extension.to_ascii_lowercase();
    if AUDIO_EXTENSIONS.contains(&extension.as_str()) {
        Some(format!("audio/{extension}"))
    } else {
        point_cloud::non_audio_asset_type_for_extension(&extension)
    }
}
pub use probe::{
    probe, probe_container, require_supported_audio, select_audio_stream, select_video_stream,
    ContainerInfo, MediaInfo, MediaStreamKind, ProbedAudioStream, ProbedVideoStream,
};

#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    #[error("{0} was not found on PATH. Install ffmpeg, then export again.")]
    ToolNotFound(&'static str),
    #[error("I/O error. {0}")]
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
    #[error("ffmpeg reported an error. {0}")]
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

#[cfg(test)]
mod tests {
    /// 門の一覧と種別付けが食い違うと、選べるのに棚に入らない物ができる。
    #[test]
    fn every_import_extension_has_an_asset_type_and_the_door_matches_what_shows() {
        let all = super::import_extensions();
        for ext in &all {
            assert!(super::asset_type_for_extension(ext).is_some(), "{ext} は門に在るのに種別が無い");
        }
        for ext in ["jpg", "jpeg", "png", "gif", "webp", "tif", "tiff", "bmp", "hdr", "exr", "mp4", "mov", "m4v", "wav", "mp3", "glb"] {
            assert!(all.contains(&ext), "{ext} が門に無い");
        }
        for ext in ["heic", "txt", "md", "rrd", "mkv", "webm", "avif"] {
            assert!(!all.contains(&ext), "{ext} は絵が出ないのに門が通す");
            assert!(super::asset_type_for_extension(ext).is_none());
        }
    }

    #[test]
    fn the_media_gate_admits_the_audio_formats_our_decoder_owns() {
        assert_eq!(super::asset_type_for_extension("wav").as_deref(), Some("audio/wav"));
        assert_eq!(super::asset_type_for_extension("MP3").as_deref(), Some("audio/mp3"));
    }

    #[test]
    fn the_spatial_gate_matches_the_formats_the_product_can_render() {
        for extension in ["glb", "obj", "stl"] {
            assert_eq!(
                super::asset_type_for_extension(extension),
                Some(format!("model/{extension}"))
            );
        }
        assert_eq!(
            super::asset_type_for_extension("ply").as_deref(),
            Some("pointcloud.ply")
        );
        assert_eq!(super::asset_type_for_extension("gltf"), None);
        assert_eq!(super::asset_type_for_extension("dae"), None);
    }
}
