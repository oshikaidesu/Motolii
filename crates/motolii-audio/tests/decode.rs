//! D4完了条件: 任意位置からのサンプル読み出しの決定的テスト、
//! format/channel/sample-rate境界の型付きerror。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use motolii_audio::{decode_file, AudioError};
use motolii_testkit::tmp_dir;
use support::{sine_wave_i16, write_pcm16_wav};

#[test]
fn decode_mono_wav_and_read_arbitrary_frames() {
    let dir = tmp_dir("motolii-audio-decode-mono");
    let path = dir.join("mono.wav");
    let rate = 48_000u32;
    let samples = sine_wave_i16(rate, rate as usize, 440.0, 1);
    write_pcm16_wav(&path, rate, 1, &samples);

    let cache = decode_file(&path).expect("decode fixture wav");
    assert_eq!(cache.format().sample_rate, rate);
    assert_eq!(cache.format().channels, 1);
    assert_eq!(cache.frame_count(), rate as u64);

    for offset in [0u64, 1, 1_000, rate as u64 / 2, rate as u64 - 1] {
        let decoded = cache.frame_at(offset).expect("in-range frame")[0];
        let expected = samples[offset as usize] as f32 / i16::MAX as f32;
        assert!(
            (decoded - expected).abs() < 2e-4,
            "offset={offset} decoded={decoded} expected={expected}"
        );
    }
}

#[test]
fn decode_stereo_wav_preserves_channel_interleaving() {
    let dir = tmp_dir("motolii-audio-decode-stereo");
    let path = dir.join("stereo.wav");
    let rate = 44_100u32;
    let frames = rate as usize / 4;
    let samples = sine_wave_i16(rate, frames, 220.0, 2);
    write_pcm16_wav(&path, rate, 2, &samples);

    let cache = decode_file(&path).expect("decode fixture wav");
    assert_eq!(cache.format().channels, 2);
    assert_eq!(cache.format().sample_rate, rate);
    assert_eq!(cache.frame_count(), frames as u64);

    let mid = frames as u64 / 2;
    let frame = cache.frame_at(mid).expect("mid frame");
    assert_eq!(frame.len(), 2);
    let expected_left = samples[mid as usize * 2] as f32 / i16::MAX as f32;
    let expected_right = samples[mid as usize * 2 + 1] as f32 / i16::MAX as f32;
    assert!((frame[0] - expected_left).abs() < 2e-4);
    assert!((frame[1] - expected_right).abs() < 2e-4);
    // 2チャンネルは振幅を変えて生成しているので同一値ではない = 混ざっていない証拠
    assert!((frame[0] - frame[1]).abs() > 1e-3);
}

#[test]
fn read_frames_at_many_offsets_matches_frame_at() {
    let dir = tmp_dir("motolii-audio-decode-offsets");
    let path = dir.join("mono.wav");
    let rate = 48_000u32;
    let samples = sine_wave_i16(rate, rate as usize, 440.0, 1);
    write_pcm16_wav(&path, rate, 1, &samples);
    let cache = decode_file(&path).expect("decode fixture wav");

    for offset in [0u64, 1, 100, 10_000, rate as u64 - 1] {
        let direct = cache.frame_at(offset).unwrap()[0];
        let chunk = cache.read_frames(offset, 1).unwrap();
        assert_eq!(direct, chunk[0], "offset={offset}");
    }
}

#[test]
fn out_of_range_read_is_typed_error() {
    let dir = tmp_dir("motolii-audio-decode-oor");
    let path = dir.join("mono.wav");
    let rate = 8_000u32;
    let samples = sine_wave_i16(rate, 100, 440.0, 1);
    write_pcm16_wav(&path, rate, 1, &samples);
    let cache = decode_file(&path).expect("decode fixture wav");

    let err = cache.read_frames(90, 20).unwrap_err();
    assert!(matches!(
        err,
        AudioError::OutOfRange {
            start: 90,
            requested: 20,
            total: 100,
        }
    ));

    let err = cache.frame_at(100).unwrap_err();
    assert!(matches!(err, AudioError::OutOfRange { start: 100, .. }));
}

#[test]
fn corrupt_file_decode_returns_typed_error_not_panic() {
    let dir = tmp_dir("motolii-audio-decode-corrupt");
    let path = dir.join("garbage.wav");
    std::fs::write(&path, b"not a real wav file, just some bytes\x00\x01\x02").unwrap();
    let result = decode_file(&path);
    assert!(result.is_err());
}

#[test]
fn zero_channel_pcm_cache_is_rejected() {
    use motolii_audio::{PcmCache, PcmFormat};
    let err = PcmCache::from_interleaved(
        vec![0.0; 4],
        PcmFormat {
            channels: 0,
            sample_rate: 48_000,
        },
    )
    .unwrap_err();
    assert!(matches!(
        err,
        AudioError::UnsupportedChannels { channels: 0 }
    ));
}

#[test]
fn zero_sample_rate_pcm_cache_is_rejected() {
    use motolii_audio::{PcmCache, PcmFormat};
    let err = PcmCache::from_interleaved(
        vec![0.0; 4],
        PcmFormat {
            channels: 2,
            sample_rate: 0,
        },
    )
    .unwrap_err();
    assert!(matches!(
        err,
        AudioError::UnsupportedSampleRate { sample_rate: 0 }
    ));
}
