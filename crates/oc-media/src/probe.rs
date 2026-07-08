use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use oc_core::{ColorSpace, Fps, RationalTime};

use crate::{MediaError, Result};

/// probeで得る映像ストリーム情報(v:0のみ。音声はM2で拡張)。
///
/// width/heightは**表示上の寸法**(回転メタデータ適用後)。デコード(autorotate)の
/// 出力寸法と一致する(レビュー指摘#4: スマホ縦動画対応)。
#[derive(Debug, Clone, PartialEq)]
pub struct MediaInfo {
    pub width: u32,
    pub height: u32,
    pub fps: Fps,
    /// フレームグリッドにスナップした長さ(レビュー指摘#7: μs分母を持ち込まない)
    pub duration: Option<RationalTime>,
    pub nb_frames: Option<i64>,
    /// 素材のYUV色空間タグ(タグ欠落時はHD慣習でRec709Limited)
    pub color_space: ColorSpace,
    /// 回転メタデータ(度、反時計回り。ffprobe side_data準拠)
    pub rotation: i64,
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
    color_space: Option<String>,
    color_range: Option<String>,
    #[serde(default)]
    side_data_list: Vec<FfprobeSideData>,
}

#[derive(Deserialize)]
struct FfprobeSideData {
    rotation: Option<i64>,
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

    let (mut width, mut height) = match (stream.width, stream.height) {
        (Some(w), Some(h)) if w > 0 && h > 0 => (w, h),
        _ => return Err(MediaError::Probe("missing dimensions".into())),
    };

    // 回転メタデータ: 90/270度なら表示寸法はW/H入れ替え(デコードはautorotateで
    // この寸法のフレームを出す)
    let rotation = stream
        .side_data_list
        .iter()
        .find_map(|sd| sd.rotation)
        .unwrap_or(0);
    if rotation.rem_euclid(180) == 90 {
        std::mem::swap(&mut width, &mut height);
    }

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
        .and_then(|s| parse_duration_snapped(s, fps));
    let nb_frames = stream.nb_frames.as_deref().and_then(|s| s.parse().ok());

    let color_space = map_color_space(stream.color_space.as_deref(), stream.color_range.as_deref());

    Ok(MediaInfo {
        width,
        height,
        fps,
        duration,
        nb_frames,
        color_space,
        rotation,
    })
}

/// ffprobeの色タグ→FrameDescの色空間。タグ欠落時はHD慣習(BT.709 limited)。
fn map_color_space(space: Option<&str>, range: Option<&str>) -> ColorSpace {
    let full = matches!(range, Some("pc") | Some("jpeg"));
    match space {
        Some("smpte170m") | Some("bt470bg") => ColorSpace::Rec601Limited,
        Some("bt709") if full => ColorSpace::Rec709Full,
        Some("bt709") => ColorSpace::Rec709Limited,
        _ if full => ColorSpace::Rec709Full,
        _ => ColorSpace::Rec709Limited,
    }
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

/// ffprobeの秒表記("2.000000")を、fpsグリッドにスナップしたRationalTimeへ。
/// μs分母(1_000_000)をタイムライン演算に持ち込まない(分母肥大化の回避)。
fn parse_duration_snapped(s: &str, fps: Fps) -> Option<RationalTime> {
    let secs: f64 = s.parse().ok()?;
    let frames = (secs * fps.as_f64()).round() as i64;
    Some(RationalTime::from_frame(frames, fps))
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
    fn duration_snaps_to_frame_grid() {
        let fps = Fps::new(30000, 1001);
        // 2.002秒 = ちょうど60フレーム(29.97fps)
        let d = parse_duration_snapped("2.002000", fps).unwrap();
        assert_eq!(d, RationalTime::from_frame(60, fps));
        // 分母がμsではなくfps由来であること
        assert!(d.den() <= 30000);
    }

    #[test]
    fn maps_color_tags() {
        assert_eq!(
            map_color_space(Some("bt709"), Some("tv")),
            ColorSpace::Rec709Limited
        );
        assert_eq!(
            map_color_space(Some("bt709"), Some("pc")),
            ColorSpace::Rec709Full
        );
        assert_eq!(
            map_color_space(Some("smpte170m"), None),
            ColorSpace::Rec601Limited
        );
        // タグ欠落 → HD慣習
        assert_eq!(map_color_space(None, None), ColorSpace::Rec709Limited);
    }
}
