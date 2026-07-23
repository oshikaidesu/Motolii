//! 実デバイスでの短時間再生smoke(D4完了条件の注記: 「hardware smokeだけを完了証跡に
//! しない」— cpal出力・producer/consumer分離・アンダーラン契約の本証跡は
//! `tests/playback_simulation.rs`/`tests/decode.rs`/`src/ring.rs`のunit testが担う。
//! 本テストは実機での最終確認用の**補助**であり、既定では走らない。
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::time::{Duration, Instant};

use cpal::traits::HostTrait;
use motolii_audio::{
    canonical_format, channel, negotiate_output, AudioProducer, OutputStream, PcmCache,
    CANONICAL_CHANNELS, CANONICAL_SAMPLE_RATE,
};

/// ローカルで `cargo test -p motolii-audio -- --ignored device_smoke` を実行して確認する。
/// CIは音声出力デバイスの有無を保証しないため、`MOTOLII_REQUIRE_GPU`(≒環境依存テスト
/// 必須化フラグ)の対象にせず常時`#[ignore]`にする — 音声ハードウェアはGPU/ffmpegと違い
/// このプロジェクトの必須依存ではない。
#[test]
#[ignore = "requires a real default audio output device; not part of default evidence"]
fn hardware_playback_reports_zero_underrun() {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("ignored hardware test requires a default output device");

    let format = canonical_format();
    let frames = CANONICAL_SAMPLE_RATE as usize * 3;
    let mut pcm = Vec::with_capacity(frames * usize::from(CANONICAL_CHANNELS));
    for i in 0..frames {
        let t = i as f32 / CANONICAL_SAMPLE_RATE as f32;
        let audible_seconds = 1.0;
        let fade_seconds = 0.03;
        let envelope = if t < fade_seconds {
            t / fade_seconds
        } else if t < audible_seconds - fade_seconds {
            1.0
        } else if t < audible_seconds {
            (audible_seconds - t) / fade_seconds
        } else {
            0.0
        };
        let left = (t * 440.0 * std::f32::consts::TAU).sin() * 0.25 * envelope;
        let right = (t * 660.0 * std::f32::consts::TAU).sin() * 0.25 * envelope;
        pcm.extend([left, right]);
    }
    let cache = Arc::new(PcmCache::from_interleaved(pcm, format).unwrap());
    let negotiated = negotiate_output(&device, format).expect("negotiate hardware output");
    let device_rate = negotiated.device_sample_rate;
    let (ring_prod, ring_cons) = channel(CANONICAL_CHANNELS, device_rate as usize / 2).unwrap();

    let producer =
        AudioProducer::spawn_with_device_rate(Arc::clone(&cache), ring_prod, 0, device_rate)
            .unwrap();
    let prefill_target = device_rate as usize / 10;
    let prefill_deadline = Instant::now() + Duration::from_secs(1);
    while ring_cons.buffered_frames() < prefill_target && Instant::now() < prefill_deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(
        ring_cons.buffered_frames() >= prefill_target,
        "producer did not prefill 100ms before hardware playback"
    );

    let output = OutputStream::open_negotiated(&device, &negotiated, ring_cons)
        .expect("open hardware output stream");

    std::thread::sleep(Duration::from_millis(1_100));

    let counters = output.counters();
    producer.stop();
    drop(output);

    assert!(
        counters.frames_supplied() > 0,
        "expected some real playback"
    );
    assert_eq!(
        counters.underrun_events(),
        0,
        "unexpected hardware underrun"
    );
    assert_eq!(counters.silence_frames(), 0, "unexpected silence fill");
}
