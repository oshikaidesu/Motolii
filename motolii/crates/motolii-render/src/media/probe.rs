use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use crate::doc::core::{ColorSpace, Fps, RationalTime};

use crate::render::media::{MediaError, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct MediaInfo {
    pub width: u32,
    pub height: u32,
    pub fps: Fps,
    pub duration: Option<RationalTime>,
    pub nb_frames: Option<i64>,
    pub color_space: ColorSpace,
    pub rotation: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContainerInfo {
    pub video_streams: Vec<ProbedVideoStream>,
    pub audio_streams: Vec<ProbedAudioStream>,
    pub duration: Option<RationalTime>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbedVideoStream {
    pub ordinal: u32,
    pub width: u32,
    pub height: u32,
    pub fps: Fps,
    pub duration: Option<RationalTime>,
    pub nb_frames: Option<i64>,
    pub color_space: ColorSpace,
    pub rotation: i64,
    pub codec_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbedAudioStream {
    pub ordinal: u32,
    pub codec_name: String,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    pub channel_layout: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaStreamKind {
    Video,
    Audio,
}

impl MediaStreamKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Video => "video",
            Self::Audio => "audio",
        }
    }
}

impl std::fmt::Display for MediaStreamKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Deserialize)]
struct FfprobeOut {
    streams: Vec<FfprobeStream>,
    format: Option<FfprobeFormat>,
}

#[derive(Deserialize)]
struct FfprobeStream {
    codec_type: Option<String>,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    r_frame_rate: Option<String>,
    avg_frame_rate: Option<String>,
    nb_frames: Option<String>,
    duration: Option<String>,
    sample_aspect_ratio: Option<String>,
    color_space: Option<String>,
    color_range: Option<String>,
    sample_rate: Option<String>,
    channels: Option<u32>,
    channel_layout: Option<String>,
    #[serde(default)]
    tags: FfprobeTags,
    #[serde(default)]
    disposition: FfprobeDisposition,
    #[serde(default)]
    side_data_list: Vec<FfprobeSideData>,
}

#[derive(Deserialize, Default)]
struct FfprobeDisposition {
    #[serde(default)]
    attached_pic: i64,
}

#[derive(Deserialize, Default)]
struct FfprobeTags {
    language: Option<String>,
}

#[derive(Deserialize)]
struct FfprobeSideData {
    rotation: Option<i64>,
}

#[derive(Deserialize)]
struct FfprobeFormat {
    duration: Option<String>,
    format_name: Option<String>,
}

fn is_still_image_format(format_name: Option<&str>) -> bool {
    let Some(names) = format_name else {
        return false;
    };
    names.split(',').any(|name| {
        matches!(
            name.trim(),
            "png_pipe"
                | "jpeg_pipe"
                | "mjpeg_pipe"
                | "webp_pipe"
                | "bmp_pipe"
                | "tiff_pipe"
                | "image2"
                | "image2pipe"
        )
    })
}

pub fn probe(path: impl AsRef<Path>) -> Result<MediaInfo> {
    let container = probe_container(path)?;
    let stream = container
        .video_streams
        .first()
        .ok_or_else(|| MediaError::Probe("no video stream".into()))?;
    Ok(MediaInfo {
        width: stream.width,
        height: stream.height,
        fps: stream.fps,
        duration: stream.duration,
        nb_frames: stream.nb_frames,
        color_space: stream.color_space,
        rotation: stream.rotation,
    })
}

pub fn probe_container(path: impl AsRef<Path>) -> Result<ContainerInfo> {
    let out = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_streams",
            "-show_format",
            "-print_format",
            "json",
        ])
        .arg(path.as_ref())
        .output()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => MediaError::ToolNotFound("ffprobe"),
            _ => MediaError::Io(e),
        })?;
    if !out.status.success() {
        return Err(MediaError::Probe(
            String::from_utf8_lossy(&out.stderr).into_owned(),
        ));
    }
    let parsed: FfprobeOut = serde_json::from_slice(&out.stdout)
        .map_err(|e| MediaError::Probe(format!("json parse: {e}")))?;

    let format_duration = parsed.format.as_ref().and_then(|f| f.duration.as_deref());
    let still_image = is_still_image_format(
        parsed
            .format
            .as_ref()
            .and_then(|f| f.format_name.as_deref()),
    );
    let mut video_streams = Vec::new();
    let mut audio_streams = Vec::new();

    for stream in &parsed.streams {
        match stream.codec_type.as_deref() {
            Some("video") => {
                if stream.disposition.attached_pic != 0 {
                    continue;
                }
                let ordinal = video_streams.len() as u32;
                video_streams.push(parse_video_stream(
                    stream,
                    ordinal,
                    format_duration,
                    still_image,
                )?);
            }
            Some("audio") => {
                let ordinal = audio_streams.len() as u32;
                audio_streams.push(parse_audio_stream(stream, ordinal)?);
            }
            _ => {}
        }
    }

    let duration = if still_image {
        None
    } else {
        format_duration.and_then(|s| RationalTime::try_from_decimal_str(s).ok())
    };

    Ok(ContainerInfo {
        video_streams,
        audio_streams,
        duration,
    })
}

pub fn select_video_stream(info: &ContainerInfo, ordinal: u32) -> Result<&ProbedVideoStream> {
    info.video_streams
        .iter()
        .find(|s| s.ordinal == ordinal)
        .ok_or(MediaError::StreamNotFound {
            kind: MediaStreamKind::Video,
            ordinal,
        })
}

pub fn select_audio_stream(info: &ContainerInfo, ordinal: u32) -> Result<&ProbedAudioStream> {
    info.audio_streams
        .iter()
        .find(|s| s.ordinal == ordinal)
        .ok_or(MediaError::StreamNotFound {
            kind: MediaStreamKind::Audio,
            ordinal,
        })
}

pub fn require_supported_audio(stream: &ProbedAudioStream) -> Result<()> {
    if !audio_codec_supported(&stream.codec_name) {
        return Err(MediaError::UnsupportedAudioCodec {
            ordinal: stream.ordinal,
            codec: stream.codec_name.clone(),
        });
    }
    if let Some(layout) = stream.channel_layout.as_deref() {
        if !channel_layout_supported(layout) {
            return Err(MediaError::UnsupportedChannelLayout {
                ordinal: stream.ordinal,
                layout: layout.to_string(),
            });
        }
    } else if let Some(ch) = stream.channels {
        if ch == 0 || ch > 2 {
            return Err(MediaError::UnsupportedChannelLayout {
                ordinal: stream.ordinal,
                layout: format!("{ch}ch"),
            });
        }
    }
    Ok(())
}

fn audio_codec_supported(codec: &str) -> bool {
    matches!(
        codec,
        "aac"
            | "mp3"
            | "ac3"
            | "eac3"
            | "flac"
            | "opus"
            | "vorbis"
            | "pcm_s16le"
            | "pcm_s24le"
            | "pcm_s32le"
            | "pcm_f32le"
            | "pcm_f64le"
            | "pcm_u8"
            | "pcm_s16be"
            | "pcm_s24be"
            | "pcm_s32be"
            | "pcm_f32be"
            | "pcm_f64be"
    )
}

fn channel_layout_supported(layout: &str) -> bool {
    matches!(
        layout,
        "mono" | "stereo" | "1 channels" | "2 channels" | "1.0" | "2.0"
    )
}

fn parse_video_stream(
    stream: &FfprobeStream,
    ordinal: u32,
    format_duration: Option<&str>,
    still_image: bool,
) -> Result<ProbedVideoStream> {
    let (mut width, mut height) = match (stream.width, stream.height) {
        (Some(w), Some(h)) if w > 0 && h > 0 => (w, h),
        _ => return Err(MediaError::Probe("missing dimensions".into())),
    };

    if let Some(sar) = stream.sample_aspect_ratio.as_deref() {
        if sar != "1:1" && sar != "0:1" && !sar.is_empty() {
            return Err(MediaError::Probe(format!(
                "anamorphic footage (SAR {sar}) is not supported in v1; \
                 re-encode to square pixels first"
            )));
        }
    }

    let rotation = stream
        .side_data_list
        .iter()
        .find_map(|sd| sd.rotation)
        .unwrap_or(0);
    if rotation.rem_euclid(180) == 90 {
        std::mem::swap(&mut width, &mut height);
    }

    validate_even_dimensions(width, height)?;

    let r_fps = stream.r_frame_rate.as_deref().and_then(parse_fraction);
    let avg_fps = stream.avg_frame_rate.as_deref().and_then(parse_fraction);
    reject_variable_frame_rate(r_fps, avg_fps)?;

    let fps = r_fps
        .or(avg_fps)
        .ok_or_else(|| MediaError::Probe("missing frame rate".into()))?;

    let (duration, nb_frames) = if still_image {
        (None, Some(1))
    } else {
        (
            stream
                .duration
                .as_deref()
                .or(format_duration)
                .and_then(|s| parse_duration_snapped(s, fps)),
            stream.nb_frames.as_deref().and_then(|s| s.parse().ok()),
        )
    };

    let color_space = if still_image {
        ColorSpace::Rec601Limited
    } else {
        map_color_space(stream.color_space.as_deref(), stream.color_range.as_deref())
            .map_err(MediaError::Probe)?
    };

    Ok(ProbedVideoStream {
        ordinal,
        width,
        height,
        fps,
        duration,
        nb_frames,
        color_space,
        rotation,
        codec_name: stream.codec_name.clone().filter(|s| !s.is_empty()),
    })
}

fn parse_audio_stream(stream: &FfprobeStream, ordinal: u32) -> Result<ProbedAudioStream> {
    let codec_name = stream
        .codec_name
        .clone()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| MediaError::Probe("missing audio codec_name".into()))?;
    let sample_rate = stream
        .sample_rate
        .as_deref()
        .and_then(|s| s.parse::<u32>().ok());
    Ok(ProbedAudioStream {
        ordinal,
        codec_name,
        sample_rate,
        channels: stream.channels,
        channel_layout: stream.channel_layout.clone().filter(|s| !s.is_empty()),
        language: stream.tags.language.clone().filter(|s| !s.is_empty()),
    })
}

fn validate_even_dimensions(width: u32, height: u32) -> Result<()> {
    if width.is_multiple_of(2) && height.is_multiple_of(2) {
        Ok(())
    } else {
        Err(MediaError::Probe(format!(
            "odd video dimensions ({width}x{height}) are not supported (4:2:0 requires even width and height); \
             re-encode with even dimensions, e.g. \
             ffmpeg -i input.mp4 -vf \"scale=trunc(iw/2)*2:trunc(ih/2)*2\" -c:v libx264 output.mp4"
        )))
    }
}

fn reject_variable_frame_rate(r_fps: Option<Fps>, avg_fps: Option<Fps>) -> Result<()> {
    let (Some(r), Some(a)) = (r_fps, avg_fps) else {
        return Ok(());
    };
    if fps_differ_significantly(r, a) {
        return Err(MediaError::Probe(format!(
            "variable frame rate (VFR) detected: r_frame_rate {}/{} != avg_frame_rate {}/{}; \
             re-encode to constant frame rate first, e.g. \
             ffmpeg -i input.mp4 -vf fps=30 -c:v libx264 output.mp4",
            r.num(),
            r.den(),
            a.num(),
            a.den()
        )));
    }
    Ok(())
}

fn fps_differ_significantly(a: Fps, b: Fps) -> bool {
    let a_f = a.as_f64();
    let b_f = b.as_f64();
    if a_f <= 0.0 || b_f <= 0.0 {
        return false;
    }
    (a_f - b_f).abs() / a_f.max(b_f) > 0.005
}

fn map_color_space(
    space: Option<&str>,
    range: Option<&str>,
) -> std::result::Result<ColorSpace, String> {
    let full = matches!(range, Some("pc") | Some("jpeg"));
    match space {
        Some("smpte170m") | Some("bt470bg") if full => Err(
            "BT.601 full range is not supported in v1; \
             re-encode to limited range or convert to BT.709 first"
                .to_string(),
        ),
        Some("smpte170m") | Some("bt470bg") => Ok(ColorSpace::Rec601Limited),
        Some("bt709") if full => Ok(ColorSpace::Rec709Full),
        Some("bt709") => Ok(ColorSpace::Rec709Limited),
        Some("bt2020nc") | Some("bt2020c") | Some("bt2020") => Err(
            "BT.2020/HDR color space is not supported in v1; \
             re-encode to BT.709 (SDR) first, e.g. \
             ffmpeg -i input.mp4 -vf zscale=transfer=linear,format=gbrpf32le,zscale=primaries=709,transfer=709,matrix=709,format=yuv420p -c:v libx264 output.mp4"
                .to_string(),
        ),
        Some(tag) => Err(format!(
            "unsupported color_space tag '{tag}'; \
             re-encode to BT.709 (SDR) or BT.601 limited first"
        )),
        None if full => Ok(ColorSpace::Rec709Full),
        None => Ok(ColorSpace::Rec709Limited),
    }
}

fn parse_fraction(s: &str) -> Option<Fps> {
    let (num, den) = s.split_once('/')?;
    let (num, den) = (num.parse::<i64>().ok()?, den.parse::<i64>().ok()?);
    if num <= 0 || den <= 0 {
        return None;
    }
    Fps::try_new(num, den).ok()
}

fn parse_duration_snapped(s: &str, fps: Fps) -> Option<RationalTime> {
    let t = RationalTime::try_from_decimal_str(s).ok()?;
    let frames = t.try_to_frame_round(fps).ok()?;
    RationalTime::try_from_frame(frames, fps).ok()
}
