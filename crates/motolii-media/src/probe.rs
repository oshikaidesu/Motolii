use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use motolii_core::{ColorSpace, Fps, RationalTime};

use crate::{MediaError, Result};

/// probeで得る映像ストリーム情報(v:0のみ。後方互換ラッパ用)。
///
/// width/heightは**表示上の寸法**(回転メタデータ適用後)。デコード(autorotate)の
/// 出力寸法と一致する(レビュー指摘#4: スマホ縦動画対応)。
#[derive(Debug, Clone, PartialEq)]
pub struct MediaInfo {
    pub width: u32,
    pub height: u32,
    pub fps: Fps,
    /// フレームグリッドにスナップした**総尺**(M2E-17)。
    /// 区間は半開 `[0, duration)`。最終フレームのPTSではない。
    pub duration: Option<RationalTime>,
    pub nb_frames: Option<i64>,
    /// 素材のYUV色空間タグ(タグ欠落時はHD慣習でRec709Limited)
    pub color_space: ColorSpace,
    /// 回転メタデータ(度、反時計回り。ffprobe side_data準拠)
    pub rotation: i64,
}

/// container内の全stream列挙結果(AG-1)。
#[derive(Debug, Clone, PartialEq)]
pub struct ContainerInfo {
    pub video_streams: Vec<ProbedVideoStream>,
    pub audio_streams: Vec<ProbedAudioStream>,
}

/// kind内ordinal付きのvideo stream。
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

/// kind内ordinal付きのaudio stream。
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
    side_data_list: Vec<FfprobeSideData>,
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
}

/// ffprobeで先頭映像ストリームを解析する(後方互換)。
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

/// container内の全video/audio streamを列挙する(AG-1)。
///
/// ordinalは同じkindをcontainer順に0から数える。欠落時の自動fallbackはしない。
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
    let mut video_streams = Vec::new();
    let mut audio_streams = Vec::new();

    for stream in &parsed.streams {
        match stream.codec_type.as_deref() {
            Some("video") => {
                let ordinal = video_streams.len() as u32;
                video_streams.push(parse_video_stream(stream, ordinal, format_duration)?);
            }
            Some("audio") => {
                let ordinal = audio_streams.len() as u32;
                audio_streams.push(parse_audio_stream(stream, ordinal)?);
            }
            _ => {}
        }
    }

    Ok(ContainerInfo {
        video_streams,
        audio_streams,
    })
}

/// kind+ordinalでvideo streamを取得する。欠落はtyped error(別streamへfallbackしない)。
pub fn select_video_stream(
    info: &ContainerInfo,
    ordinal: u32,
) -> Result<&ProbedVideoStream> {
    info.video_streams
        .iter()
        .find(|s| s.ordinal == ordinal)
        .ok_or(MediaError::StreamNotFound {
            kind: MediaStreamKind::Video,
            ordinal,
        })
}

/// kind+ordinalでaudio streamを取得する。欠落はtyped error。
pub fn select_audio_stream(
    info: &ContainerInfo,
    ordinal: u32,
) -> Result<&ProbedAudioStream> {
    info.audio_streams
        .iter()
        .find(|s| s.ordinal == ordinal)
        .ok_or(MediaError::StreamNotFound {
            kind: MediaStreamKind::Audio,
            ordinal,
        })
}

/// AG-1で受理するaudio codec/layoutか。未対応はtyped error。
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
) -> Result<ProbedVideoStream> {
    let (mut width, mut height) = match (stream.width, stream.height) {
        (Some(w), Some(h)) if w > 0 && h > 0 => (w, h),
        _ => return Err(MediaError::Probe("missing dimensions".into())),
    };

    // 非正方ピクセル(アナモルフィック)はv1スコープ外として明確に拒否する
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

    let duration = stream
        .duration
        .as_deref()
        .or(format_duration)
        .and_then(|s| parse_duration_snapped(s, fps));
    let nb_frames = stream.nb_frames.as_deref().and_then(|s| s.parse().ok());

    let color_space = map_color_space(stream.color_space.as_deref(), stream.color_range.as_deref())
        .map_err(MediaError::Probe)?;

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

/// 4:2:0デコード前提のため偶数寸法のみ受理する。
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

/// r_frame_rate と avg_frame_rate が有意に食い違う場合はVFR疑いとして拒否する。
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

/// ffprobeの色タグ→FrameDescの色空間。タグ欠落時はHD慣習(BT.709 limited)。
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

/// "30000/1001" 形式を解析。
fn parse_fraction(s: &str) -> Option<Fps> {
    let (num, den) = s.split_once('/')?;
    let (num, den) = (num.parse::<i64>().ok()?, den.parse::<i64>().ok()?);
    if num <= 0 || den <= 0 {
        return None;
    }
    Fps::try_new(num, den).ok()
}

/// ffprobeの秒表記("2.000000")を、fpsグリッドにスナップしたRationalTimeへ。
fn parse_duration_snapped(s: &str, fps: Fps) -> Option<RationalTime> {
    let secs: f64 = s.parse().ok()?;
    let frames = (secs * fps.as_f64()).round() as i64;
    RationalTime::try_from_frame(frames, fps).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fraction() {
        assert_eq!(parse_fraction("30000/1001"), Fps::try_new(30000, 1001).ok());
        assert_eq!(parse_fraction("30/1"), Fps::try_new(30, 1).ok());
        assert_eq!(parse_fraction("0/0"), None);
        assert_eq!(parse_fraction("abc"), None);
    }

    #[test]
    fn duration_snaps_to_frame_grid() {
        let fps = Fps::try_new(30000, 1001).unwrap();
        let d = parse_duration_snapped("2.002000", fps).unwrap();
        assert_eq!(d, RationalTime::try_from_frame(60, fps).unwrap());
        assert!(d.den() <= 30000);
    }

    #[test]
    fn maps_color_tags() {
        assert_eq!(
            map_color_space(Some("bt709"), Some("tv")).unwrap(),
            ColorSpace::Rec709Limited
        );
        assert_eq!(
            map_color_space(Some("bt709"), Some("pc")).unwrap(),
            ColorSpace::Rec709Full
        );
        assert_eq!(
            map_color_space(Some("smpte170m"), None).unwrap(),
            ColorSpace::Rec601Limited
        );
        assert_eq!(
            map_color_space(None, None).unwrap(),
            ColorSpace::Rec709Limited
        );
    }

    #[test]
    fn rejects_unknown_and_hdr_color_tags() {
        assert!(map_color_space(Some("bt2020nc"), Some("tv")).is_err());
        assert!(map_color_space(Some("bt2020"), None).is_err());
        assert!(map_color_space(Some("unknown_tag"), None).is_err());
    }

    #[test]
    fn rejects_601_full_range() {
        assert!(map_color_space(Some("smpte170m"), Some("pc")).is_err());
        assert!(map_color_space(Some("bt470bg"), Some("jpeg")).is_err());
    }

    #[test]
    fn rejects_odd_dimensions() {
        assert!(validate_even_dimensions(641, 480).is_err());
        assert!(validate_even_dimensions(640, 481).is_err());
        assert!(validate_even_dimensions(640, 480).is_ok());
    }

    #[test]
    fn rejects_variable_frame_rate_when_rates_differ() {
        let cfr = Fps::try_new(30000, 1001).unwrap();
        assert!(!fps_differ_significantly(cfr, cfr));
        let vfr = Fps::try_new(24, 1).unwrap();
        assert!(fps_differ_significantly(cfr, vfr));
        assert!(reject_variable_frame_rate(Some(cfr), Some(vfr)).is_err());
        assert!(reject_variable_frame_rate(Some(cfr), None).is_ok());
    }

    #[test]
    fn supported_audio_accepts_common_codecs() {
        let ok = ProbedAudioStream {
            ordinal: 0,
            codec_name: "aac".into(),
            sample_rate: Some(48_000),
            channels: Some(2),
            channel_layout: Some("stereo".into()),
            language: None,
        };
        assert!(require_supported_audio(&ok).is_ok());
    }

    #[test]
    fn unsupported_audio_codec_is_typed() {
        let bad = ProbedAudioStream {
            ordinal: 1,
            codec_name: "cook".into(),
            sample_rate: Some(44_100),
            channels: Some(2),
            channel_layout: Some("stereo".into()),
            language: None,
        };
        assert!(matches!(
            require_supported_audio(&bad),
            Err(MediaError::UnsupportedAudioCodec { ordinal: 1, .. })
        ));
    }

    #[test]
    fn unsupported_layout_is_typed() {
        let bad = ProbedAudioStream {
            ordinal: 0,
            codec_name: "aac".into(),
            sample_rate: Some(48_000),
            channels: Some(6),
            channel_layout: Some("5.1".into()),
            language: None,
        };
        assert!(matches!(
            require_supported_audio(&bad),
            Err(MediaError::UnsupportedChannelLayout { .. })
        ));
    }
}
