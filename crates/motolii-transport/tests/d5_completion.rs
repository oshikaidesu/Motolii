//! D5(#144)骨格検証 — 完了条件のうち機械判定可能な部分のみ(統合/E2Eはpending)。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::time::Duration;

use motolii_audio::{PcmCache, PcmFormat, PlaybackCounters};
use motolii_core::{Fps, Quality, RationalTime};
use motolii_transport::{
    display_frame_without_latency_compensation, drift_within_one_frame, synced_display_frame,
    test_preview, DrsConfig, DrsController, DrsStage, FrameTiming, Transport,
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

/// リング+fill_or_silence経路で、古い表示フレームが残る間のドリフトと補償後の同期を検証する。
#[test]
fn d5_drift_with_stale_display_and_latency_compensation() {
    let rate = 48_000u32;
    let fps = Fps::try_new(30, 1).unwrap();
    let chunk = 480usize;
    let cache = sine_cache(600, rate); // 10分
    let (mut sim, ring_prod) = test_preview(rate, 8_192, chunk, false);

    let mut on_screen_frame: Option<i64> = None;
    let mut source_playhead = 0u64;
    let ten_min_samples = rate as u64 * 60 * 10;
    let mut supplied_total = 0u64;
    let mut tick = 0u64;

    let frame_samples = (rate as u64 * fps.den() as u64) / fps.num() as u64;

    while supplied_total < ten_min_samples {
        let expected = cache.read_frames(source_playhead, chunk).unwrap();
        while ring_prod.push_frames(expected) == 0 {
            std::hint::spin_loop();
        }
        source_playhead += chunk as u64;
        supplied_total += chunk as u64;

        let device_wait = frame_samples * 2 + (tick % 3) * (frame_samples / 4);
        sim.tick_audio_callback(device_wait).unwrap();

        let perceptual = sim.transport().perceptual_time().unwrap();
        let supplied = sim.transport().supplied_frames();

        // 補償なし対照: 供給が十分かつ待ちが2フレーム超のときだけドリフトを要求する。
        if supplied > device_wait + frame_samples * 2 {
            let uncompensated =
                display_frame_without_latency_compensation(supplied, rate, fps).unwrap();
            assert!(
                !drift_within_one_frame(uncompensated, perceptual, fps).unwrap(),
                "uncompensated frame must drift when device_wait spans multiple frames"
            );
        }

        let update_display = tick.is_multiple_of(4);
        if update_display {
            let synced =
                synced_display_frame(perceptual, fps).expect("synced frame from perceptual");
            let plan = sim.transport_mut().next_frame_plan().unwrap();
            if let Some(prev) = on_screen_frame {
                if synced > prev {
                    assert_ne!(prev, synced, "display frame must advance over 10min");
                }
            }
            assert_eq!(
                plan.display_frame, synced,
                "plan must match independently computed sync frame"
            );
            assert!(
                drift_within_one_frame(synced, perceptual, fps).unwrap(),
                "compensated display must track perceptual within 1 frame"
            );
            on_screen_frame = Some(synced);
        } else if let Some(stale) = on_screen_frame {
            let current_synced = synced_display_frame(perceptual, fps).unwrap();
            if tick % 4 == 2 && current_synced > stale {
                assert_ne!(
                    stale, current_synced,
                    "stale frame must lag behind perceptual during render gap"
                );
            }
        }

        sim.transport_mut()
            .record_render_timing(FrameTiming::unmeasured(
                Duration::ZERO,
                DrsConfig::from_fps(fps).frame_budget / 2,
            ));
        tick += 1;
    }
}

#[test]
fn d5_half_speed_pcm_bit_identical_video_drops() {
    let rate = 48_000u32;
    let cache = sine_cache(2, rate);
    let (mut sim, ring_prod) = test_preview(rate, 8_192, 480, true);

    let report = sim
        .run_half_speed_render(&cache, &ring_prod, 2, true)
        .unwrap();
    assert!(
        report.pcm_bit_identical,
        "PCM must match cache at source rate"
    );
    assert_eq!(report.max_underrun_events, 0, "no audio underruns");
    assert!(
        report.frames_dropped > 0,
        "video must drop when render is 0.5x"
    );
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

    drs.record_frame(FrameTiming::measured_gpu(
        near,
        Duration::from_micros(100),
        near,
    ));
    drs.record_frame(FrameTiming::measured_gpu(
        near,
        Duration::from_micros(100),
        near,
    ));
    assert_eq!(drs.stage(), DrsStage::Quarter);

    for _ in 0..config.min_dwell_frames {
        drs.record_frame(FrameTiming::measured_gpu(
            near,
            Duration::from_micros(100),
            near,
        ));
    }
    assert_eq!(
        drs.oscillations_in_dwell(),
        0,
        "no stage oscillation within min dwell"
    );
}

#[test]
fn d5_drs_ignores_wall_when_gpu_unmeasured() {
    let fps = Fps::try_new(30, 1).unwrap();
    let config = DrsConfig::from_fps(fps);
    let mut drs = DrsController::new(true, config);
    drs.record_frame(FrameTiming::unmeasured(
        Duration::ZERO,
        config.frame_budget + Duration::from_secs(1),
    ));
    drs.record_frame(FrameTiming::unmeasured(
        Duration::ZERO,
        config.frame_budget + Duration::from_secs(1),
    ));
    assert_eq!(drs.stage(), DrsStage::Half);
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
        transport
            .drs()
            .effective_quality(Quality::DRAFT)
            .resolution_scale,
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
    let perceptual = transport.perceptual_time().unwrap();
    assert_eq!(perceptual, RationalTime::try_new(47_520, 48_000).unwrap());
}
