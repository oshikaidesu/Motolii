#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use std::sync::Arc;

use cpal::traits::HostTrait;
use motolii_audio::{decode_file, simulate_playback_without_underrun, PcmCache, PlaybackHandle};
use motolii_testkit::tmp_dir;
use support::{fixture_sine_1s, write_pcm16_mono_wav};

#[test]
fn decode_wav_and_read_arbitrary_frames() {
    let dir = tmp_dir("d4-decode");
    let path = dir.join("sine.wav");
    let (rate, samples) = fixture_sine_1s();
    write_pcm16_mono_wav(&path, rate, &samples);

    let cache = decode_file(&path).expect("decode fixture wav");
    assert_eq!(cache.format().sample_rate, rate);
    assert_eq!(cache.format().channels, 1);
    assert_eq!(cache.frame_count(), rate as u64);

    let mid = rate as u64 / 2;
    let frame = cache.frame_at(0).expect("frame 0");
    assert_eq!(frame.len(), 1);
    assert!(frame[0].abs() < 0.01);

    let mid_frame = cache.frame_at(mid).expect("mid frame");
    let expected = samples[mid as usize] as f32 / 32768.0;
    assert!((mid_frame[0] - expected).abs() < 0.002);

    let tail = cache.read_frames(rate as u64 - 10, 10).expect("tail read");
    assert_eq!(tail.len(), 10);
}

#[test]
fn read_frames_at_multiple_offsets_match_direct_index() {
    let dir = tmp_dir("d4-offsets");
    let path = dir.join("sine.wav");
    let (rate, samples) = fixture_sine_1s();
    write_pcm16_mono_wav(&path, rate, &samples);
    let cache = decode_file(&path).unwrap();

    for offset in [0u64, 1, 100, 1_000, 10_000, rate as u64 - 1] {
        let direct = cache.frame_at(offset).unwrap()[0];
        let chunk = cache.read_frames(offset, 1).unwrap();
        assert!((direct - chunk[0]).abs() < 1e-6, "offset={offset}");
    }
}

#[test]
fn simulated_continuous_playback_has_zero_underruns() {
    let dir = tmp_dir("d4-sim");
    let path = dir.join("sine.wav");
    let (rate, samples) = fixture_sine_1s();
    write_pcm16_mono_wav(&path, rate, &samples);
    let cache = decode_file(&path).unwrap();

    let stats = simulate_playback_without_underrun(
        &cache,
        0,
        std::time::Duration::from_secs_f64(256.0 / rate as f64),
        256,
    )
    .unwrap();
    assert_eq!(stats.underrun_count, 0);
    assert_eq!(stats.frames_delivered, cache.frame_count());
}

#[test]
fn simulated_playback_from_mid_offset() {
    let dir = tmp_dir("d4-sim-mid");
    let path = dir.join("sine.wav");
    let (rate, samples) = fixture_sine_1s();
    write_pcm16_mono_wav(&path, rate, &samples);
    let cache = decode_file(&path).unwrap();
    let start = rate as u64 / 3;

    let stats = simulate_playback_without_underrun(
        &cache,
        start,
        std::time::Duration::from_secs_f64(128.0 / rate as f64),
        128,
    )
    .unwrap();
    assert_eq!(stats.underrun_count, 0);
    assert_eq!(stats.frames_delivered, cache.frame_count() - start);
}

/// 実デバイス向けの短い再生でアンダーラン0を確認する。
///
/// cpal出力はGPU/ffmpegと違いCI必須依存ではない(M2E-1の`MOTOLII_REQUIRE_GPU`対象外)。
/// 既定では無視し、ローカルで `cargo test -p motolii-audio -- --ignored` を走らせる。
#[test]
#[ignore = "optional cpal device; not covered by MOTOLII_REQUIRE_GPU"]
fn hardware_playback_without_underrun() {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("ignored hardware test requires a default output device");

    let rate = 48_000u32;
    let frames = rate as usize / 10; // 100ms
    let mut pcm = Vec::with_capacity(frames);
    for i in 0..frames {
        let t = i as f32 / rate as f32;
        pcm.push((t * 220.0 * std::f32::consts::TAU).sin());
    }
    let cache = Arc::new(
        PcmCache::from_interleaved(
            pcm,
            motolii_audio::PcmFormat {
                channels: 1,
                sample_rate: rate,
            },
        )
        .unwrap(),
    );

    let handle = PlaybackHandle::play_from_on_device(cache, &device, 0)
        .expect("start hardware playback");
    std::thread::sleep(std::time::Duration::from_millis(60));
    let stats = handle.stop();
    assert_eq!(stats.underrun_count, 0, "hardware underruns: {}", stats.underrun_count);
    assert!(stats.frames_delivered > 0);
}
