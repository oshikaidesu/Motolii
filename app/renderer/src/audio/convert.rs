
use crate::render::audio::cache::PcmFormat;

pub const CANONICAL_SAMPLE_RATE: u32 = 48_000;
pub const CANONICAL_CHANNELS: u16 = 2;

pub fn canonical_format() -> PcmFormat {
    PcmFormat {
        channels: CANONICAL_CHANNELS,
        sample_rate: CANONICAL_SAMPLE_RATE,
    }
}

pub fn time_to_canonical_frames(t: crate::doc::core::RationalTime) -> u64 {
    if t <= crate::doc::core::RationalTime::ZERO {
        return 0;
    }
    let num = t.num().max(0) as u128;
    let den = t.den().max(1) as u128;
    ((num * u128::from(CANONICAL_SAMPLE_RATE)) / den) as u64
}

/// 音の頭は素材の頭に揃う。符号化の準備区間(AAC の priming)や edit list で前後しない。
#[cfg(test)]
mod head_alignment {
    use crate::render::audio::decode::decode_file;

    fn ffmpeg_available() -> bool {
        crate::render::media::test_encoders_available(&["aac", "libmp3lame"])
    }

    /// 0.5 秒の無音のあと 1kHz が鳴る素材を、指定の codec で作る。
    fn silence_then_tone(dir: &std::path::Path, name: &str, codec_args: &[&str]) -> std::path::PathBuf {
        let out = dir.join(name);
        let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
            .args([
                "-v", "error", "-y",
                "-f", "lavfi", "-i", "sine=frequency=1000:sample_rate=48000:duration=1",
                "-af", "adelay=500:all=1",
            ])
            .args(codec_args)
            .arg(&out)
            .status()
            .expect("spawn ffmpeg");
        assert!(status.success(), "audio fixture failed: {name}");
        out
    }

    /// 最初に |sample| > 0.05 となる時刻(秒)。
    fn onset_seconds(path: &std::path::Path) -> f64 {
        let pcm = decode_file(path).unwrap();
        let first = pcm.samples_i16().iter().position(|s| (*s as i32).abs() > 1600).expect("tone never starts");
        (first / 2) as f64 / 48_000.0
    }

    fn assert_onset_at_half_second(path: &std::path::Path) {
        let onset = onset_seconds(path);
        let error_ms = (onset - 0.5) * 1000.0;
        assert!(
            error_ms.abs() <= 5.0,
            "{}: tone starts at {onset:.4}s, {error_ms:+.1} ms off",
            path.file_name().unwrap().to_string_lossy()
        );
    }

    #[test]
    fn wav_head_is_where_the_file_says() {
        if !ffmpeg_available() { eprintln!("skip: ffmpeg not on PATH"); return; }
        let dir = tempfile::tempdir().unwrap();
        assert_onset_at_half_second(&silence_then_tone(dir.path(), "tone.wav", &["-c:a", "pcm_s16le"]));
    }

    #[test]
    fn aac_head_is_where_the_file_says() {
        if !ffmpeg_available() { eprintln!("skip: ffmpeg not on PATH"); return; }
        let dir = tempfile::tempdir().unwrap();
        assert_onset_at_half_second(&silence_then_tone(dir.path(), "tone.m4a", &["-c:a", "aac", "-b:a", "128k"]));
    }

    #[test]
    fn mp3_head_is_where_the_file_says() {
        if !ffmpeg_available() { eprintln!("skip: ffmpeg not on PATH"); return; }
        let dir = tempfile::tempdir().unwrap();
        assert_onset_at_half_second(&silence_then_tone(dir.path(), "tone.mp3", &["-c:a", "libmp3lame", "-b:a", "128k"]));
    }
}

/// 数字を見る物(`cargo test -- --ignored head_numbers --nocapture`)。codec ごとの頭の誤差 ms。
#[cfg(test)]
mod head_numbers {
    #[test]
    #[ignore]
    fn head_error_per_codec() {
        if !crate::render::media::test_encoders_available(&["aac", "libmp3lame"]) {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        for (name, args) in [
            ("tone.wav", vec!["-c:a", "pcm_s16le"]),
            ("tone.m4a", vec!["-c:a", "aac", "-b:a", "128k"]),
            ("tone.mp3", vec!["-c:a", "libmp3lame", "-b:a", "128k"]),
            ("tone_44k.m4a", vec!["-ar", "44100", "-c:a", "aac", "-b:a", "128k"]),
        ] {
            let out = dir.path().join(name);
            let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
                .args(["-v", "error", "-y", "-f", "lavfi", "-i", "sine=frequency=1000:sample_rate=48000:duration=1", "-af", "adelay=500:all=1"])
                .args(&args)
                .arg(&out)
                .status()
                .unwrap();
            assert!(status.success());
            let pcm = crate::render::audio::decode::decode_file(&out).unwrap();
            let first = pcm.samples_i16().iter().position(|s| (*s as i32).abs() > 1600).unwrap();
            let onset = (first / 2) as f64 / 48_000.0;
            println!("PROBE room=audio verdict=head-error codec={name} error-ms={:+.2}", (onset - 0.5) * 1000.0);
        }
    }
}
