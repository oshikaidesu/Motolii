//! D6: 映像mp4へ楽曲をmuxする。
//!
//! ミキシングバウンスはしない。コーデックがmp4互換かつ `master_gain == 1.0` なら
//! ストリームコピーを優先し、それ以外は AAC 再エンコードへ落ちる。
//! `start_offset` は音源側の開始点(ソースin点)として `-ss` で渡す。

use std::path::Path;
use std::process::Command;

use motolii_core::RationalTime;

use crate::{read_child_stderr, MediaError, Result};

/// 音声ストリームの最小情報(mux可否判定用)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioStreamInfo {
    pub codec_name: String,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioEncodeMode {
    /// コンテナ互換コーデックを無劣化コピー。
    StreamCopy,
    /// AACへ再エンコード(gain適用や非互換コーデック)。
    AacEncode,
}

/// 映像へ楽曲を載せる要求。Documentスキーマには触れない(Export層から渡す)。
#[derive(Debug, Clone)]
pub struct SoundtrackMuxRequest<'a> {
    pub video_path: &'a Path,
    pub audio_path: &'a Path,
    pub output_path: &'a Path,
    /// 音源の開始オフセット(ソースin点。タイムライン0に対応する音源時刻)。
    pub start_offset: RationalTime,
    /// [0, 1]。1.0以外はストリームコピー不可。
    pub master_gain: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundtrackMuxReport {
    pub encode_mode: AudioEncodeMode,
    pub audio_codec: String,
}

/// 先頭音声ストリームを解析する。無音声なら Err。
///
/// `a:0`だけを読む。`probe_container`へ委譲しない — album art(attached_pic)や
/// 未対応副videoの解析失敗でSoundtrack muxを巻き込まない(AG-1 review P2)。
pub fn probe_audio(path: impl AsRef<Path>) -> Result<AudioStreamInfo> {
    let out = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_entries",
            "stream=codec_name,sample_rate,channels",
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
    #[derive(serde::Deserialize)]
    struct Out {
        streams: Vec<Stream>,
    }
    #[derive(serde::Deserialize)]
    struct Stream {
        codec_name: Option<String>,
        sample_rate: Option<String>,
        channels: Option<u32>,
    }
    let parsed: Out = serde_json::from_slice(&out.stdout)
        .map_err(|e| MediaError::Probe(format!("json parse: {e}")))?;
    let stream = parsed
        .streams
        .first()
        .ok_or_else(|| MediaError::Probe("no audio stream".into()))?;
    let codec_name = stream
        .codec_name
        .clone()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| MediaError::Probe("missing audio codec_name".into()))?;
    let sample_rate = stream
        .sample_rate
        .as_deref()
        .and_then(|s| s.parse::<u32>().ok());
    Ok(AudioStreamInfo {
        codec_name,
        sample_rate,
        channels: stream.channels,
    })
}

/// mp4へ無劣化コピーできるコーデックか。
pub fn audio_codec_allows_stream_copy(codec: &str) -> bool {
    matches!(codec, "aac" | "mp3" | "ac3" | "eac3")
}

pub fn choose_audio_encode_mode(codec: &str, master_gain: f64) -> AudioEncodeMode {
    if master_gain == 1.0 && audio_codec_allows_stream_copy(codec) {
        AudioEncodeMode::StreamCopy
    } else {
        AudioEncodeMode::AacEncode
    }
}

/// 映像mp4 + 楽曲ファイル → 音声付きmp4。
pub fn mux_soundtrack(req: &SoundtrackMuxRequest<'_>) -> Result<SoundtrackMuxReport> {
    if req.start_offset < RationalTime::ZERO {
        return Err(MediaError::InvalidStartOffset(req.start_offset));
    }
    if !(0.0..=1.0).contains(&req.master_gain) || !req.master_gain.is_finite() {
        return Err(MediaError::InvalidMasterGain(req.master_gain));
    }

    let audio = probe_audio(req.audio_path)?;
    let encode_mode = choose_audio_encode_mode(&audio.codec_name, req.master_gain);
    let offset = format_offset_secs(req.start_offset);

    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-v", "error", "-y"]);
    // 映像は先頭入力。音声は -ss 付きでソースin点から。
    cmd.arg("-i").arg(req.video_path);
    if req.start_offset > RationalTime::ZERO {
        cmd.args(["-ss", &offset]);
    }
    cmd.arg("-i").arg(req.audio_path);
    cmd.args(["-map", "0:v:0", "-map", "1:a:0", "-c:v", "copy"]);
    match encode_mode {
        AudioEncodeMode::StreamCopy => {
            cmd.args(["-c:a", "copy"]);
        }
        AudioEncodeMode::AacEncode => {
            cmd.args(["-c:a", "aac", "-b:a", "192k"]);
            if req.master_gain != 1.0 {
                // volumeフィルタは再エンコード時のみ。ストリームコピーと両立しない。
                cmd.args(["-af", &format!("volume={}", req.master_gain)]);
            }
        }
    }
    // 映像尺に合わせて音声を切る(MV最終書き出し: コンポ尺=映像)。
    cmd.args(["-shortest", "-movflags", "+faststart"]);
    cmd.arg(req.output_path);
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => MediaError::ToolNotFound("ffmpeg"),
        _ => MediaError::Io(e),
    })?;
    let mut err = String::new();
    if let Some(stderr) = child.stderr.as_mut() {
        err = read_child_stderr(stderr)?;
    }
    let status = child.wait()?;
    if !status.success() {
        return Err(MediaError::Ffmpeg(err));
    }
    Ok(SoundtrackMuxReport {
        encode_mode,
        audio_codec: audio.codec_name,
    })
}

fn format_offset_secs(t: RationalTime) -> String {
    // 有理数を十分精度の秒文字列へ。比較・演算には使わずffmpeg引数専用。
    let s = format!("{:.9}", t.as_seconds_f64());
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat};

    use crate::Encoder;

    fn make_silent_video(path: &Path, frames: usize) {
        let desc = FrameDesc::packed(16, 8, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false);
        let mut enc = Encoder::open(path, &desc, Fps::try_new(12, 1).unwrap(), true).unwrap();
        let frame = vec![0u8; desc.data_size()];
        for _ in 0..frames {
            enc.write_frame(&frame).unwrap();
        }
        enc.finish().unwrap();
    }

    fn make_aac(path: &Path, seconds: f64) {
        let status = Command::new("ffmpeg")
            .args([
                "-v",
                "error",
                "-y",
                "-f",
                "lavfi",
                "-i",
                &format!("sine=frequency=440:sample_rate=48000:duration={seconds}"),
                "-c:a",
                "aac",
                "-b:a",
                "128k",
            ])
            .arg(path)
            .status()
            .expect("spawn ffmpeg");
        assert!(status.success(), "aac fixture failed");
    }

    fn extract_pcm(path: &Path, out: &Path, start: Option<&str>, duration: Option<&str>) {
        let mut cmd = Command::new("ffmpeg");
        cmd.args(["-v", "error", "-y"]);
        if let Some(ss) = start {
            cmd.args(["-ss", ss]);
        }
        cmd.arg("-i").arg(path);
        if let Some(t) = duration {
            cmd.args(["-t", t]);
        }
        cmd.args(["-vn", "-ac", "1", "-ar", "48000", "-f", "s16le"])
            .arg(out);
        let status = cmd.status().expect("spawn ffmpeg extract");
        assert!(status.success(), "pcm extract failed");
    }

    #[test]
    fn choose_mode_prefers_copy_for_aac_unity_gain() {
        assert_eq!(
            choose_audio_encode_mode("aac", 1.0),
            AudioEncodeMode::StreamCopy
        );
        assert_eq!(
            choose_audio_encode_mode("aac", 0.5),
            AudioEncodeMode::AacEncode
        );
        assert_eq!(
            choose_audio_encode_mode("pcm_s16le", 1.0),
            AudioEncodeMode::AacEncode
        );
    }

    #[test]
    fn mux_rejects_negative_offset() {
        let err = mux_soundtrack(&SoundtrackMuxRequest {
            video_path: Path::new("v.mp4"),
            audio_path: Path::new("a.m4a"),
            output_path: Path::new("o.mp4"),
            start_offset: RationalTime::try_new(-1, 10).unwrap(),
            master_gain: 1.0,
        })
        .unwrap_err();
        assert!(matches!(err, MediaError::InvalidStartOffset(_)));
    }

    #[test]
    fn mux_aac_stream_copy_matches_source_pcm() {
        if !motolii_testkit::ffmpeg_or_skip() {
            return;
        }
        let dir = motolii_testkit::tmp_dir("media-mux-copy");
        let video = dir.join("v.mp4");
        let audio = dir.join("a.m4a");
        let out = dir.join("out.mp4");
        make_silent_video(&video, 24); // 2s @12fps
        make_aac(&audio, 3.0);

        let report = mux_soundtrack(&SoundtrackMuxRequest {
            video_path: &video,
            audio_path: &audio,
            output_path: &out,
            start_offset: RationalTime::ZERO,
            master_gain: 1.0,
        })
        .unwrap();
        assert_eq!(report.encode_mode, AudioEncodeMode::StreamCopy);
        assert_eq!(report.audio_codec, "aac");

        let got = dir.join("got.pcm");
        let want = dir.join("want.pcm");
        // 映像尺(=shortest)に揃えた区間でPCM比較。
        extract_pcm(&out, &got, None, Some("2"));
        extract_pcm(&audio, &want, None, Some("2"));
        let got_bytes = std::fs::read(&got).unwrap();
        let want_bytes = std::fs::read(&want).unwrap();
        // -shortest は映像パケット境界で切るため、AACフレーム単位で数ms短くなり得る。
        // 重なる先頭はサンプル一致であることを完了条件の審判にする。
        let n = got_bytes.len().min(want_bytes.len());
        assert!(n > 48_000, "expected ~1s+ of pcm, got {n} bytes");
        assert_eq!(&got_bytes[..n], &want_bytes[..n]);
        assert!(
            (got_bytes.len() as i64 - want_bytes.len() as i64).abs() < 8_192,
            "duration skew too large: got={} want={}",
            got_bytes.len(),
            want_bytes.len()
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn mux_with_start_offset_matches_trimmed_source() {
        if !motolii_testkit::ffmpeg_or_skip() {
            return;
        }
        let dir = motolii_testkit::tmp_dir("media-mux-offset");
        let video = dir.join("v.mp4");
        let audio = dir.join("a.m4a");
        let out = dir.join("out.mp4");
        make_silent_video(&video, 12); // 1s
        make_aac(&audio, 3.0);
        let offset = RationalTime::try_new(1, 2).unwrap(); // 0.5s

        let report = mux_soundtrack(&SoundtrackMuxRequest {
            video_path: &video,
            audio_path: &audio,
            output_path: &out,
            start_offset: offset,
            master_gain: 1.0,
        })
        .unwrap();
        assert_eq!(report.encode_mode, AudioEncodeMode::StreamCopy);

        let got = dir.join("got.pcm");
        let want = dir.join("want.pcm");
        extract_pcm(&out, &got, None, Some("1"));
        extract_pcm(&audio, &want, Some("0.5"), Some("1"));
        let got_bytes = std::fs::read(&got).unwrap();
        let want_bytes = std::fs::read(&want).unwrap();
        let n = got_bytes.len().min(want_bytes.len());
        assert!(n > 24_000, "expected ~0.5s+ of pcm, got {n} bytes");
        assert_eq!(&got_bytes[..n], &want_bytes[..n]);
        assert!(
            (got_bytes.len() as i64 - want_bytes.len() as i64).abs() < 8_192,
            "duration skew too large: got={} want={}",
            got_bytes.len(),
            want_bytes.len()
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn probe_audio_reads_aac() {
        if !motolii_testkit::ffmpeg_or_skip() {
            return;
        }
        let dir = motolii_testkit::tmp_dir("media-probe-audio");
        let audio = dir.join("a.m4a");
        make_aac(&audio, 0.25);
        let info = probe_audio(&audio).unwrap();
        assert_eq!(info.codec_name, "aac");
        assert_eq!(info.sample_rate, Some(48000));
        std::fs::remove_dir_all(&dir).ok();
    }
}
