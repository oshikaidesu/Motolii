use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use oc_core::{Fps, RationalTime};

use crate::{MediaError, Result};

/// probeで得る映像ストリーム情報(v:0のみ。音声はM2で拡張)。
#[derive(Debug, Clone, PartialEq)]
pub struct MediaInfo {
    pub width: u32,
    pub height: u32,
    pub fps: Fps,
    pub duration: Option<RationalTime>,
    pub nb_frames: Option<i64>,
}

#[derive(Deserialize)]
struct FfprobeOut {
    streams: Vec<FfprobeStream>,
    format: Option<FfprobeFormat>,
}

#[derive(Deserialize)]
struct FfprobeStream {
    width: Option<u32>,
    height: Option<u32>,
    r_frame_rate: Option<String>,
    avg_frame_rate: Option<String>,
    nb_frames: Option<String>,
    duration: Option<String>,
}

#[derive(Deserialize)]
struct FfprobeFormat {
    duration: Option<String>,
}

/// ffprobeで先頭映像ストリームを解析する。
pub fn probe(path: impl AsRef<Path>) -> Result<MediaInfo> {
    let out = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
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
    let stream = parsed
        .streams
        .first()
        .ok_or_else(|| MediaError::Probe("no video stream".into()))?;

    let (width, height) = match (stream.width, stream.height) {
        (Some(w), Some(h)) if w > 0 && h > 0 => (w, h),
        _ => return Err(MediaError::Probe("missing dimensions".into())),
    };
    // r_frame_rateを優先(コンテナ宣言レート)。VFR素材ではavg_frame_rateと食い違う。
    // その扱い(CFR正規化)はM4のインポートパイプラインの責務。
    let fps = stream
        .r_frame_rate
        .as_deref()
        .and_then(parse_fraction)
        .or_else(|| stream.avg_frame_rate.as_deref().and_then(parse_fraction))
        .ok_or_else(|| MediaError::Probe("missing frame rate".into()))?;

    let duration = stream
        .duration
        .as_deref()
        .or(parsed.format.as_ref().and_then(|f| f.duration.as_deref()))
        .and_then(parse_seconds);
    let nb_frames = stream.nb_frames.as_deref().and_then(|s| s.parse().ok());

    Ok(MediaInfo {
        width,
        height,
        fps,
        duration,
        nb_frames,
    })
}

/// "30000/1001" 形式を解析。
fn parse_fraction(s: &str) -> Option<Fps> {
    let (num, den) = s.split_once('/')?;
    let (num, den) = (num.parse::<i64>().ok()?, den.parse::<i64>().ok()?);
    if num <= 0 || den <= 0 {
        return None;
    }
    Some(Fps::new(num, den))
}

/// ffprobeの秒表記("2.000000")をマイクロ秒精度のRationalTimeへ。
fn parse_seconds(s: &str) -> Option<RationalTime> {
    let secs: f64 = s.parse().ok()?;
    Some(RationalTime::new((secs * 1e6).round() as i64, 1_000_000))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fraction() {
        assert_eq!(parse_fraction("30000/1001"), Some(Fps::new(30000, 1001)));
        assert_eq!(parse_fraction("30/1"), Some(Fps::new(30, 1)));
        assert_eq!(parse_fraction("0/0"), None);
        assert_eq!(parse_fraction("abc"), None);
    }

    #[test]
    fn parses_seconds() {
        assert_eq!(parse_seconds("2.000000"), Some(RationalTime::new(2, 1)));
        assert_eq!(
            parse_seconds("0.033367"),
            Some(RationalTime::new(33367, 1_000_000))
        );
    }
}
