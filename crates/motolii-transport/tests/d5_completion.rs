//! D5(#144)完了条件の名前付き自動検証(ヘッドレス)。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::time::Duration;

use motolii_audio::{PcmCache, PcmFormat, PlaybackCounters};
use motolii_core::{Fps, Quality};
use motolii_transport::{
    test_preview, test_transport_headless, DrsConfig, DrsController, DrsStage, FrameTiming,
    Transport,
};

fn sine_cache(seconds: u64, rate: u32) -> Arc<PcmCache> {
    let frames = (seconds * rate as u64) as usize;
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

#[test]
fn d5_drift_ten_minutes_within_one_frame() {
    let mut transport = test_transport_headless(48_000, false);
    let ten_min = 48_000u64 * 60 * 10;
    let chunk = 480u64;
    let budget = DrsConfig::from_fps(transport.fps()).frame_budget;
    let mut t = 0u64;
    while t < ten_min {
        transport.counters().advance_supplied_for_simulation(chunk);
        transport.device_wait().set_wait_frames(240);
        let plan = transport.next_frame_plan().unwrap();
        transport.record_render_timing(FrameTiming {
            gpu: budget / 2,
            cpu: Duration::ZERO,
            wall: budget / 2,
        });
        assert!(
            transport
                .display_drift_within_one_frame(plan.display_frame)
                .unwrap(),
            "drift at display frame {}",
            plan.display_frame
        );
        t += chunk;
    }
}

#[test]
fn d5_half_speed_pcm_bit_identical_video_drops() {
    let rate = 48_000u32;
    let cache = sine_cache(2, rate);
    let (mut sim, ring_prod) = test_preview(rate, 8_192, 480, true);

    let report = sim.run_half_speed_render(&cache, &ring_prod, 2, true);
    assert!(report.pcm_bit_identical, "PCM must stay bit-identical at source rate");
    assert_eq!(report.max_underrun_events, 0, "no audio underruns");
    assert!(report.frames_dropped > 0, "video must drop when render is 0.5x");
    assert!(
        report.frames_rendered < (rate as u64 * 2) / 30,
        "fewer renders than realtime 30fps"
    );
}

#[test]
fn d5_drs_no_pumping_near_threshold() {
    let fps = Fps::try_new(30, 1).unwrap();
    let config = DrsConfig::from_fps(fps);
    let mut drs = DrsController::new(true, config);
    let budget = config.frame_budget;
    let near = budget + Duration::from_micros(500);

    drs.record_frame(FrameTiming {
        gpu: near,
        cpu: Duration::from_micros(100),
        wall: near,
    });
    drs.record_frame(FrameTiming {
        gpu: near,
        cpu: Duration::from_micros(100),
        wall: near,
    });
    assert_eq!(drs.stage(), DrsStage::Quarter);

    for _ in 0..config.min_dwell_frames {
        drs.record_frame(FrameTiming {
            gpu: near,
            cpu: Duration::from_micros(100),
            wall: near,
        });
    }
    assert_eq!(
        drs.oscillations_in_dwell(),
        0,
        "no stage oscillation within min dwell"
    );
}

#[test]
fn d5_quality_switch_no_audio_glitch() {
    let rate = 48_000u32;
    let (mut sim, ring_prod) = test_preview(rate, 8_192, 480, true);
    sim.assert_quality_switch_glitch_free(&ring_prod)
        .expect("resolution switch must not glitch audio supply");
}

#[test]
fn d5_timestamp_query_unavailable_disables_auto_drs() {
    let transport = Transport::new(
        Arc::new(PlaybackCounters::default()),
        Arc::new(motolii_audio::DeviceWaitLatency::default()),
        Fps::try_new(30, 1).unwrap(),
        48_000,
        Quality::DRAFT,
        false,
    )
    .unwrap();
    assert!(!transport.drs().is_enabled());
    assert_eq!(
        transport.drs().effective_quality(Quality::DRAFT).resolution_scale,
        Quality::DRAFT.resolution_scale
    );
}

#[test]
fn d5_latency_compensation_uses_device_wait_only() {
    let counters = Arc::new(PlaybackCounters::default());
    let wait = Arc::new(motolii_audio::DeviceWaitLatency::default());
    counters.advance_supplied_for_simulation(48_000);
    wait.set_wait_frames(480);
    let transport = Transport::new(
        counters,
        wait,
        Fps::try_new(30, 1).unwrap(),
        48_000,
        Quality::DRAFT,
        false,
    )
    .unwrap();
    // 1秒供給 − 10ms待ち = 0.99秒聴感
    let perceptual = transport.perceptual_time().unwrap();
    assert_eq!(
        perceptual,
        motolii_core::RationalTime::try_new(47_520, 48_000).unwrap()
    );
}
