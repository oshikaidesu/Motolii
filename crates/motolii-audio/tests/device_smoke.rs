//! 実デバイスでの短時間再生smoke(D4完了条件の注記: 「hardware smokeだけを完了証跡に
//! しない」— cpal出力・producer/consumer分離・アンダーラン契約の本証跡は
//! `tests/playback_simulation.rs`/`tests/decode.rs`/`src/ring.rs`のunit testが担う。
//! 本テストは実機での最終確認用の**補助**であり、既定では走らない。
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::time::Duration;

use cpal::traits::HostTrait;
use motolii_audio::{channel, AudioProducer, OutputStream, PcmCache, PcmFormat};

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

    let rate = 48_000u32;
    let frames = rate as usize / 10; // 100ms
    let mut pcm = Vec::with_capacity(frames);
    for i in 0..frames {
        let t = i as f32 / rate as f32;
        pcm.push((t * 220.0 * std::f32::consts::TAU).sin() * 0.2);
    }
    let format = PcmFormat {
        channels: 1,
        sample_rate: rate,
    };
    let cache = Arc::new(PcmCache::from_interleaved(pcm, format).unwrap());
    let (ring_prod, ring_cons) = channel(1, rate as usize / 5).unwrap();

    let producer = AudioProducer::spawn(Arc::clone(&cache), ring_prod, 0).unwrap();
    let output = OutputStream::open_on_device(&device, format, ring_cons)
        .expect("open hardware output stream");

    std::thread::sleep(Duration::from_millis(200));

    let counters = output.counters();
    producer.stop();
    drop(output);

    assert!(
        counters.frames_supplied() > 0,
        "expected some real playback"
    );
}
