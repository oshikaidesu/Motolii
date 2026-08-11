//! Host非依存の外部メディアdrop検証。

use std::fs::File;
use std::path::{Path, PathBuf};

use motolii_core::RationalTime;
use motolii_doc::{Asset, AssetDraft, SourceFingerprintError, SourceFingerprintV1};
use motolii_media::ContainerInfo;

#[derive(Debug)]
pub(crate) struct PreparedMediaDrop {
    pub(crate) asset: AssetDraft,
    pub(crate) duration: RationalTime,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum MediaDropReject {
    #[error("a media drop must contain exactly one file, got {0}")]
    FileCount(usize),
    #[error("media probe rejected the file")]
    Probe(#[source] motolii_media::MediaError),
    #[error("the container has no ordinal-0 video stream")]
    NoVideoStream,
    #[error("the ordinal-0 video stream has no duration")]
    DurationUnavailable,
    #[error("the dropped file has no extension")]
    MissingExtension,
    #[error("the dropped file extension is not UTF-8")]
    NonUtf8Extension,
    #[error("the dropped file path is not UTF-8")]
    NonUtf8Path,
    #[error("the dropped file has no file name")]
    MissingFileName,
    #[error("the dropped file name is not UTF-8")]
    NonUtf8FileName,
    #[error("the dropped file has no stem")]
    MissingFileStem,
    #[error("the dropped file stem is not UTF-8")]
    NonUtf8FileStem,
    #[error("failed to open the dropped file")]
    Open(#[source] std::io::Error),
    #[error("failed to fingerprint the dropped file")]
    Fingerprint(#[source] SourceFingerprintError),
}

impl MediaDropReject {
    pub(crate) const fn reason(&self) -> &'static str {
        match self {
            Self::FileCount(_) => "file-count",
            Self::Probe(_) => "probe",
            Self::NoVideoStream => "no-video-stream",
            Self::DurationUnavailable => "duration-unavailable",
            Self::MissingExtension => "missing-extension",
            Self::NonUtf8Extension => "non-utf8-extension",
            Self::NonUtf8Path => "non-utf8-path",
            Self::MissingFileName => "missing-file-name",
            Self::NonUtf8FileName => "non-utf8-file-name",
            Self::MissingFileStem => "missing-file-stem",
            Self::NonUtf8FileStem => "non-utf8-file-stem",
            Self::Open(_) => "open",
            Self::Fingerprint(_) => "fingerprint",
        }
    }
}

pub(crate) fn prepare_media_drop(paths: &[PathBuf]) -> Result<PreparedMediaDrop, MediaDropReject> {
    let [path] = paths else {
        return Err(MediaDropReject::FileCount(paths.len()));
    };
    let container = motolii_media::probe_container(path).map_err(MediaDropReject::Probe)?;
    let duration = ordinal_zero_duration(&container)?;
    prepare_asset(path, duration)
}

fn ordinal_zero_duration(container: &ContainerInfo) -> Result<RationalTime, MediaDropReject> {
    container
        .video_streams
        .iter()
        .find(|stream| stream.ordinal == 0)
        .ok_or(MediaDropReject::NoVideoStream)?
        .duration
        .ok_or(MediaDropReject::DurationUnavailable)
}

fn prepare_asset(
    path: &Path,
    duration: RationalTime,
) -> Result<PreparedMediaDrop, MediaDropReject> {
    let normalized = Asset::normalize_path(path.to_str().ok_or(MediaDropReject::NonUtf8Path)?);
    let normalized_path = Path::new(&normalized);
    let extension = normalized_path
        .extension()
        .ok_or(MediaDropReject::MissingExtension)?
        .to_str()
        .ok_or(MediaDropReject::NonUtf8Extension)?
        .to_lowercase();
    if extension.is_empty() {
        return Err(MediaDropReject::MissingExtension);
    }
    let file_name = normalized_path
        .file_name()
        .ok_or(MediaDropReject::MissingFileName)?
        .to_str()
        .ok_or(MediaDropReject::NonUtf8FileName)?
        .to_owned();
    let name = normalized_path
        .file_stem()
        .ok_or(MediaDropReject::MissingFileStem)?
        .to_str()
        .ok_or(MediaDropReject::NonUtf8FileStem)?
        .to_owned();
    let fingerprint =
        SourceFingerprintV1::from_reader(File::open(path).map_err(MediaDropReject::Open)?)
            .map_err(MediaDropReject::Fingerprint)?;

    Ok(PreparedMediaDrop {
        asset: AssetDraft {
            name,
            asset_type: format!("video/{extension}"),
            content_hash: fingerprint.content_hash(),
            path_absolute: Some(normalized),
            path_project_relative: None,
            file_name: Some(file_name),
            size_bytes: Some(fingerprint.size_bytes()),
            head_hash: None,
            tail_hash: None,
        },
        duration,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_core::{ColorSpace, Fps};
    use motolii_media::ProbedVideoStream;

    #[test]
    fn multiple_files_are_rejected_before_probe() {
        let paths = [PathBuf::from("one.mp4"), PathBuf::from("two.mp4")];
        assert!(matches!(
            prepare_media_drop(&paths),
            Err(MediaDropReject::FileCount(2))
        ));
    }

    #[test]
    fn text_file_is_a_typed_probe_reject() {
        if !motolii_testkit::ffmpeg_or_skip() {
            return;
        }
        let dir = motolii_testkit::tmp_dir("media-drop-text");
        let path = dir.join("not-media.txt");
        std::fs::write(&path, b"not media").unwrap();
        assert!(matches!(
            prepare_media_drop(&[path]),
            Err(MediaDropReject::Probe(_))
        ));
    }

    #[test]
    fn durationless_video_is_a_typed_reject() {
        let fps = Fps::try_new(30, 1).unwrap();
        let container = ContainerInfo {
            video_streams: vec![ProbedVideoStream {
                ordinal: 0,
                width: 64,
                height: 48,
                fps,
                duration: None,
                nb_frames: None,
                color_space: ColorSpace::Rec709Limited,
                rotation: 0,
                codec_name: Some("h264".into()),
            }],
            audio_streams: Vec::new(),
        };
        assert!(matches!(
            ordinal_zero_duration(&container),
            Err(MediaDropReject::DurationUnavailable)
        ));
    }
}
