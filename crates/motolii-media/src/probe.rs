use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use motolii_core::{ColorSpace, Fps, RationalTime};

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
    sample_aspect_ratio: Option<String>,
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

    // 非正方ピクセル(アナモルフィック)はv1スコープ外として明確に拒否する
    // (レビュー指摘#6: 黙ってアスペクト崩れさせない)
    if let Some(sar) = stream.sample_aspect_ratio.as_deref() {
        if sar != "1:1" && sar != "0:1" && !sar.is_empty() {
            return Err(MediaError::Probe(format!(
                "anamorphic footage (SAR {sar}) is not supported in v1; \
                 re-encode to square pixels first"
            )));
        }
    }

    // 回転メタデータ: 90/270度なら表示寸法はW/H入れ替え(デコードは明示transposeで
    // この寸法のフレームを出す)
    let rotation = stream
        .side_data_list
        .iter()
        .find_map(|sd| sd.rotation)
        .unwrap_or(0);
    if rotation.rem_euclid(180) == 90 {
        std::mem::swap(&mut width, &mut height);
    }

    validate_even_dimensions(width, height)?;

    let r_fps = stream
        .r_frame_rate
        .as_deref()
        .and_then(parse_fraction);
    let avg_fps = stream
        .avg_frame_rate
        .as_deref()
        .and_then(parse_fraction);
    reject_variable_frame_rate(r_fps, avg_fps)?;

    let fps = r_fps
        .or(avg_fps)
        .ok_or_else(|| MediaError::Probe("missing frame rate".into()))?;

    let duration = stream
        .duration
        .as_deref()
        .or(parsed.format.as_ref().and_then(|f| f.duration.as_deref()))
        .and_then(|s| parse_duration_snapped(s, fps));
    let nb_frames = stream.nb_frames.as_deref().and_then(|s| s.parse().ok());

    let color_space = map_color_space(stream.color_space.as_deref(), stream.color_range.as_deref())
        .map_err(MediaError::Probe)?;

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
            r.num, r.den, a.num, a.den
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
        // タグ欠落 → HD慣習
        assert_eq!(map_color_space(None, None).unwrap(), ColorSpace::Rec709Limited);
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
        let cfr = Fps::new(30000, 1001);
        assert!(!fps_differ_significantly(cfr, cfr));
        let vfr = Fps::new(24, 1);
        assert!(fps_differ_significantly(cfr, vfr));
        assert!(reject_variable_frame_rate(Some(cfr), Some(vfr)).is_err());
        assert!(reject_variable_frame_rate(Some(cfr), None).is_ok());
    }
}
