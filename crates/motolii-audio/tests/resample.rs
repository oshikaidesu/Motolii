//! D4-FU完了条件: 固定比リサンプル(デバイス≠素材レート)、恒等パス、
//! 変換前PCMビット同一、インパルス時刻対応(開始・シーク)、変換後の正速・無欠落。
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::time::{Duration, Instant};

use motolii_audio::{
    channel, fill_or_silence, select_device_sample_rate, source_frame_to_device, AudioProducer,
    FixedRatioResampler, PcmCache, PcmFormat, PlaybackCounters,
};

fn impulse_cache(frames: usize, rate: u32, impulse_at: usize) -> Arc<PcmCache> {
    let mut samples = vec![0.0f32; frames];
    samples[impulse_at] = 1.0;
    Arc::new(
        PcmCache::from_interleaved(
            samples,
            PcmFormat {
                channels: 1,
                sample_rate: rate,
            },
        )
        .unwrap(),
    )
}

fn sine_cache(frames: usize, rate: u32) -> Arc<PcmCache> {
    let mut samples = Vec::with_capacity(frames);
    for i in 0..frames {
        let t = i as f32 / rate as f32;
        samples.push((t * 440.0 * std::f32::consts::TAU).sin());
    }
    Arc::new(
        PcmCache::from_interleaved(
            samples,
            PcmFormat {
                channels: 1,
                sample_rate: rate,
            },
        )
        .unwrap(),
    )
}

/// プロデューサが自然終了するまでリングを消費し、デバイス側PCMを回収する。
///
/// `stop` は `running=false` で打ち切るため、総フレーム回帰では期待尺到達後も
/// しばらく無供給が続くのを待ってから join する(過剰 flush を検出するため)。
fn drain_producer(
    cache: Arc<PcmCache>,
    start_frame: u64,
    device_rate: u32,
    ring_capacity: usize,
) -> (Vec<f32>, bool) {
    let channels = cache.format().channels as usize;
    let source_rate = cache.format().sample_rate;
    let start = start_frame.min(cache.frame_count());
    let expected_frames = source_frame_to_device(
        cache.frame_count().saturating_sub(start),
        source_rate,
        device_rate,
    ) as usize;

    let (prod, cons) = channel(cache.format().channels, ring_capacity).unwrap();
    let producer =
        AudioProducer::spawn_with_device_rate(Arc::clone(&cache), prod, start_frame, device_rate)
            .unwrap();
    let resampling = producer.is_resampling();

    let mut out = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut quiet_after_expected = 0u32;
    while Instant::now() < deadline {
        let n = cons.buffered_frames();
        if n > 0 {
            let mut buf = vec![0.0f32; n * channels];
            let local = PlaybackCounters::default();
            fill_or_silence(&cons, &mut buf, &local);
            let supplied = local.frames_supplied() as usize;
            out.extend_from_slice(&buf[..supplied * channels]);
            quiet_after_expected = 0;
        } else {
            let out_frames = out.len() / channels;
            if out_frames >= expected_frames {
                quiet_after_expected += 1;
                // 期待尺到達後 100ms 無供給なら自然終了とみなす。
                // 過剰 flush があるとここに到達する前に out が伸び続ける。
                if quiet_after_expected >= 100 {
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    producer.stop();
    loop {
        let n = cons.buffered_frames();
        if n == 0 {
            break;
        }
        let mut buf = vec![0.0f32; n * channels];
        let local = PlaybackCounters::default();
        fill_or_silence(&cons, &mut buf, &local);
        let supplied = local.frames_supplied() as usize;
        out.extend_from_slice(&buf[..supplied * channels]);
    }
    (out, resampling)
}

#[test]
fn matching_rates_use_identity_path() {
    let cache = sine_cache(1_024, 48_000);
    let (out, resampling) = drain_producer(Arc::clone(&cache), 0, 48_000, 4_096);
    assert!(!resampling);
    let expected = cache.read_frames(0, cache.frame_count() as usize).unwrap();
    assert_eq!(out.as_slice(), expected);
}

#[test]
fn pre_conversion_pcm_remains_bit_identical_while_resampling() {
    let cache = sine_cache(2_048, 44_100);
    let before: Vec<f32> = cache
        .read_frames(0, cache.frame_count() as usize)
        .unwrap()
        .to_vec();
    let (_out, resampling) = drain_producer(Arc::clone(&cache), 0, 48_000, 8_192);
    assert!(resampling);
    let after = cache.read_frames(0, cache.frame_count() as usize).unwrap();
    assert_eq!(before.as_slice(), after);
}

#[test]
fn impulse_from_start_maps_to_expected_device_frame() {
    let src_rate = 44_100u32;
    let dst_rate = 48_000u32;
    let impulse_at = 200usize;
    let cache = impulse_cache(8_000, src_rate, impulse_at);
    let (out, resampling) = drain_producer(Arc::clone(&cache), 0, dst_rate, 16_384);
    assert!(resampling);

    let (peak_idx, peak_val) = out
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap();
    assert!(*peak_val > 0.1, "impulse peak too small: {peak_val}");
    let expected = source_frame_to_device(impulse_at as u64, src_rate, dst_rate) as usize;
    assert!(
        (peak_idx as isize - expected as isize).unsigned_abs() <= 2,
        "peak={peak_idx} expected≈{expected} val={peak_val}"
    );
}

#[test]
fn impulse_after_seek_maps_to_output_origin() {
    let src_rate = 44_100u32;
    let dst_rate = 48_000u32;
    let impulse_at = 1_500usize;
    let cache = impulse_cache(8_000, src_rate, impulse_at);
    let (out, resampling) = drain_producer(Arc::clone(&cache), impulse_at as u64, dst_rate, 16_384);
    assert!(resampling);

    let (peak_idx, peak_val) = out
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap();
    assert!(*peak_val > 0.1, "impulse peak too small: {peak_val}");
    assert!(
        peak_idx <= 2,
        "seeked impulse should land near device frame 0, got {peak_idx} (val={peak_val})"
    );
}

#[test]
fn resampled_supply_is_gapless_with_separated_underrun_counters() {
    let src_rate = 44_100u32;
    let dst_rate = 48_000u32;
    let cache = sine_cache(src_rate as usize / 2, src_rate); // 0.5s
    let (prod, cons) = channel(1, 8_192).unwrap();
    let producer =
        AudioProducer::spawn_with_device_rate(Arc::clone(&cache), prod, 0, dst_rate).unwrap();
    assert!(producer.is_resampling());

    let counters = PlaybackCounters::default();
    let chunk = 256usize;
    let prefill = Instant::now() + Duration::from_secs(5);
    while cons.buffered_frames() < chunk * 4 {
        assert!(Instant::now() < prefill, "prefill timeout");
        std::thread::sleep(Duration::from_millis(1));
    }

    let expected_device_frames =
        source_frame_to_device(cache.frame_count(), src_rate, dst_rate) as usize;
    let mut prev_supplied = 0u64;
    let deadline = Instant::now() + Duration::from_secs(15);
    // 空きを待ってから読む(人工 underrun を作らない)。
    while (counters.frames_supplied() as usize) < expected_device_frames {
        assert!(Instant::now() < deadline, "playback timeout");
        let available = cons.buffered_frames();
        if available == 0 {
            std::thread::sleep(Duration::from_millis(1));
            continue;
        }
        let n = available.min(chunk);
        let mut buf = vec![0.0f32; n];
        fill_or_silence(&cons, &mut buf, &counters);
        let supplied = counters.frames_supplied();
        assert!(supplied > prev_supplied);
        prev_supplied = supplied;
    }
    // 期待尺到達後も過剰 flush が無いか短時間監視する。
    let quiet_deadline = Instant::now() + Duration::from_millis(100);
    while Instant::now() < quiet_deadline {
        let available = cons.buffered_frames();
        if available > 0 {
            let mut buf = vec![0.0f32; available];
            fill_or_silence(&cons, &mut buf, &counters);
        } else {
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    producer.stop();
    loop {
        let n = cons.buffered_frames();
        if n == 0 {
            break;
        }
        let mut buf = vec![0.0f32; n];
        fill_or_silence(&cons, &mut buf, &counters);
    }

    assert_eq!(counters.underrun_events(), 0, "unexpected underrun");
    assert_eq!(counters.silence_frames(), 0);
    assert_eq!(
        counters.frames_supplied() as usize,
        expected_device_frames,
        "flush must not supply past expected device frames"
    );
}

#[test]
fn resampled_total_frames_match_expected_from_start() {
    let src_rate = 44_100u32;
    let dst_rate = 48_000u32;
    let src_frames = (src_rate / 2) as usize; // 0.5s → 期待 24_000 device frames
    let cache = sine_cache(src_frames, src_rate);
    let (out, resampling) = drain_producer(Arc::clone(&cache), 0, dst_rate, 16_384);
    assert!(resampling);
    let expected = source_frame_to_device(src_frames as u64, src_rate, dst_rate) as usize;
    assert_eq!(
        out.len(),
        expected,
        "start=0: got {} frames, expected {expected} (bug was 36800 via over-flush)",
        out.len()
    );
}

#[test]
fn resampled_total_frames_match_expected_after_seek() {
    let src_rate = 44_100u32;
    let dst_rate = 48_000u32;
    let src_frames = (src_rate / 2) as usize;
    let start = 4_410u64; // 0.1s seek
    let cache = sine_cache(src_frames, src_rate);
    let (out, resampling) = drain_producer(Arc::clone(&cache), start, dst_rate, 16_384);
    assert!(resampling);
    let remaining = src_frames as u64 - start;
    let expected = source_frame_to_device(remaining, src_rate, dst_rate) as usize;
    assert_eq!(
        out.len(),
        expected,
        "seek: got {} frames, expected {expected}",
        out.len()
    );
}

#[test]
fn unsupported_source_rate_selects_fallback_device_rate() {
    let ranges = [(48_000, 48_000)];
    assert_eq!(select_device_sample_rate(44_100, &ranges), Some(48_000));
    let cache = sine_cache(2_048, 44_100);
    let device_rate = select_device_sample_rate(44_100, &ranges).unwrap();
    let (out, resampling) = drain_producer(cache, 0, device_rate, 8_192);
    assert!(resampling);
    assert!(!out.is_empty());
}

#[test]
fn leading_trim_absorbs_resampler_delay() {
    let mut rs = FixedRatioResampler::new(44_100, 48_000, 1).unwrap();
    let delay = rs.output_delay();
    assert!(delay > 0);
    let need = rs.input_frames_next();
    let zeros = vec![0.0f32; need];
    let first = rs.process_interleaved(&zeros).unwrap().to_vec();
    let second = rs.process_interleaved(&zeros).unwrap().to_vec();
    // 先頭チャンクは delay trim 中なので、定常チャンクより長くならない。
    assert!(
        first.len() <= second.len(),
        "first={} second={} delay={delay}",
        first.len(),
        second.len()
    );
}
