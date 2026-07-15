//! ヘッドレス向けプレビュー/再生シミュレータ(D5骨格検証)。

use std::sync::Arc;
use std::time::Duration;

use motolii_audio::{channel, fill_or_silence, PcmCache, PlaybackCounters, RingConsumer};
use motolii_core::{Fps, Quality};

use crate::drs::{DrsConfig, FrameTiming};
use crate::{FramePlan, Transport, TransportError};

/// 0.5x律速シミュレーションの観測結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HalfSpeedSimReport {
    pub frames_rendered: u64,
    pub frames_dropped: u64,
    pub pcm_bit_identical: bool,
    pub drs_downgraded: bool,
    pub max_underrun_events: u64,
}

/// 人工クロックで音声コールバックと低速レンダを同期するシミュレータ。
pub struct PreviewSimulator {
    transport: Transport,
    ring_cons: RingConsumer,
    counters: Arc<PlaybackCounters>,
    fps: Fps,
    sample_rate: u32,
    chunk_frames: usize,
    supplied_trace: Vec<u64>,
    last_callback_pcm: Vec<f32>,
    source_playhead: u64,
}

impl PreviewSimulator {
    pub fn new(
        transport: Transport,
        ring_cons: RingConsumer,
        counters: Arc<PlaybackCounters>,
        sample_rate: u32,
        chunk_frames: usize,
    ) -> Self {
        let fps = transport.fps();
        Self {
            transport,
            ring_cons,
            counters,
            fps,
            sample_rate,
            chunk_frames,
            supplied_trace: Vec::new(),
            last_callback_pcm: Vec::new(),
            source_playhead: 0,
        }
    }

    pub fn transport(&self) -> &Transport {
        &self.transport
    }

    pub fn transport_mut(&mut self) -> &mut Transport {
        &mut self.transport
    }

    pub fn ring_buffered_frames(&self) -> usize {
        self.ring_cons.buffered_frames()
    }

    pub fn last_callback_pcm(&self) -> &[f32] {
        &self.last_callback_pcm
    }

    /// 1音声コールバック分を進め、供給済み列とPCM出力を記録する。
    pub fn tick_audio_callback(&mut self, device_wait_frames: u64) -> Result<(), TransportError> {
        self.transport
            .device_wait()
            .set_wait_frames(device_wait_frames);
        self.last_callback_pcm.resize(self.chunk_frames, 0.0);
        fill_or_silence(&self.ring_cons, &mut self.last_callback_pcm, &self.counters);
        self.supplied_trace.push(self.counters.frames_supplied());
        Ok(())
    }

    /// レンダ1回(遅延`render_cost`をDRSへ記録)。
    pub fn tick_render(&mut self, render_cost: Duration) -> Result<FramePlan, TransportError> {
        let plan = self.transport.next_frame_plan()?;
        self.transport
            .record_render_timing(FrameTiming::measured_gpu(
                render_cost,
                Duration::ZERO,
                render_cost,
            ));
        Ok(plan)
    }

    /// 0.5x律速: レンダが2×遅い間もPCM供給は正速・無欠落、映像はドロップ+DRS可。
    pub fn run_half_speed_render(
        &mut self,
        cache: &PcmCache,
        ring_prod: &motolii_audio::RingProducer,
        duration_secs: u64,
        drs_enabled: bool,
    ) -> Result<HalfSpeedSimReport, TransportError> {
        self.transport.drs_mut().set_enabled(drs_enabled);
        let total_frames = (self.sample_rate as u64) * duration_secs;
        let frame_budget = DrsConfig::from_fps(self.fps).frame_budget;
        let slow_cost = frame_budget * 2;
        let samples_per_video_frame =
            (self.sample_rate as u64 * self.fps.den() as u64) / self.fps.num() as u64;
        let mut renders = 0u64;
        let mut pcm_ok = true;
        let underrun_before = self.counters.underrun_events();
        let mut samples_since_opportunity = 0u64;
        let mut render_opportunity = 0u64;

        let mut elapsed = 0u64;
        while elapsed < total_frames {
            let expected = cache
                .read_frames(self.source_playhead, self.chunk_frames)
                .map_err(|_| TransportError::CacheRead)?;
            while ring_prod.push_frames(expected) == 0 {
                std::hint::spin_loop();
            }

            self.tick_audio_callback(256)?;

            if self.last_callback_pcm[..] != expected[..] {
                pcm_ok = false;
            }
            self.source_playhead += self.chunk_frames as u64;

            elapsed += self.chunk_frames as u64;
            samples_since_opportunity += self.chunk_frames as u64;

            if samples_since_opportunity >= samples_per_video_frame {
                samples_since_opportunity = 0;
                render_opportunity += 1;
                if render_opportunity.is_multiple_of(2) {
                    self.tick_render(slow_cost)?;
                    renders += 1;
                }
            }
        }

        Ok(HalfSpeedSimReport {
            frames_rendered: renders,
            frames_dropped: self.transport.total_dropped_frames(),
            pcm_bit_identical: pcm_ok,
            drs_downgraded: self.transport.drs().stage() == crate::DrsStage::Quarter,
            max_underrun_events: self.counters.underrun_events() - underrun_before,
        })
    }

    /// 解像度切替前後で供給サンプル列に不連続が無いか + アンダーラン増加0。
    pub fn assert_quality_switch_glitch_free(
        &mut self,
        ring_prod: &motolii_audio::RingProducer,
    ) -> Result<(), String> {
        let underrun_before = self.counters.underrun_events();
        self.supplied_trace.clear();
        let frame_budget = DrsConfig::from_fps(self.fps).frame_budget;

        for i in 0..24 {
            while ring_prod.free_frames() > 0 {
                let chunk = vec![0.0f32; self.chunk_frames];
                if ring_prod.push_frames(&chunk) == 0 {
                    break;
                }
                let _ = i;
            }
            self.tick_audio_callback(128).map_err(|e| e.to_string())?;
            let over = i < 4;
            let cost = if over {
                frame_budget * 2
            } else {
                frame_budget / 3
            };
            self.tick_render(cost).map_err(|e| e.to_string())?;
        }

        if self.counters.underrun_events() != underrun_before {
            return Err(format!(
                "underrun increased: before={underrun_before} after={}",
                self.counters.underrun_events()
            ));
        }

        for w in self.supplied_trace.windows(2) {
            if w[1] < w[0] {
                return Err(format!("supplied frames decreased: {w:?}"));
            }
            let delta = w[1] - w[0];
            if delta == 0 || delta > self.chunk_frames as u64 {
                return Err(format!("supplied delta not contiguous: {w:?}"));
            }
        }
        Ok(())
    }

    /// 閾値近傍負荷で最小滞留内の同一段階再復帰振動が0回。
    pub fn assert_no_pumping_near_threshold(&mut self) -> Result<(), String> {
        let config = DrsConfig::from_fps(self.fps);
        let mut drs = crate::DrsController::new(true, config);
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
        if drs.stage() != crate::DrsStage::Quarter {
            return Err("expected downgrade to quarter before dwell test".into());
        }

        for _ in 0..config.min_dwell_frames {
            drs.record_frame(FrameTiming::measured_gpu(
                near,
                Duration::from_micros(100),
                near,
            ));
        }
        if drs.oscillations_in_dwell() != 0 {
            return Err(format!(
                "pumping detected: oscillations={}",
                drs.oscillations_in_dwell()
            ));
        }
        Ok(())
    }
}

/// リング+Transportを束ねたテスト用ビルダ。
pub fn test_preview(
    sample_rate: u32,
    ring_capacity: usize,
    chunk_frames: usize,
    drs_enabled: bool,
) -> (PreviewSimulator, motolii_audio::RingProducer) {
    let counters = Arc::new(PlaybackCounters::default());
    let wait = Arc::new(motolii_audio::DeviceWaitLatency::default());
    let (prod, cons) = channel(1, ring_capacity).unwrap();
    let transport = Transport::new(
        Arc::clone(&counters),
        wait,
        Fps::try_new(30, 1).unwrap(),
        sample_rate,
        Quality::DRAFT,
        drs_enabled,
    )
    .unwrap();
    let sim = PreviewSimulator::new(transport, cons, counters, sample_rate, chunk_frames);
    (sim, prod)
}

/// 10分シミュレーション用の軽量Transport(リング無し)。
pub fn test_transport_headless(sample_rate: u32, drs_enabled: bool) -> Transport {
    Transport::new(
        Arc::new(PlaybackCounters::default()),
        Arc::new(motolii_audio::DeviceWaitLatency::default()),
        Fps::try_new(30, 1).unwrap(),
        sample_rate,
        Quality::DRAFT,
        drs_enabled,
    )
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use motolii_audio::PcmFormat;

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
    fn half_speed_render_drops_video_keeps_audio_clock() {
        let rate = 48_000u32;
        let cache = sine_cache(2, rate);
        let (mut sim, ring_prod) = test_preview(rate, 8_192, 480, true);

        let report = sim
            .run_half_speed_render(&cache, &ring_prod, 2, true)
            .unwrap();
        assert!(report.frames_dropped > 0, "must drop frames at 0.5x render");
        assert!(report.frames_rendered < rate as u64 * 2 / 30);
        assert_eq!(report.max_underrun_events, 0);
        assert!(report.pcm_bit_identical);
    }

    #[test]
    fn no_pumping_near_threshold() {
        let mut sim = PreviewSimulator::new(
            test_transport_headless(48_000, true),
            channel(1, 1024).unwrap().1,
            Arc::new(PlaybackCounters::default()),
            48_000,
            480,
        );
        sim.assert_no_pumping_near_threshold().unwrap();
    }

    #[test]
    fn quality_switch_does_not_glitch_audio_supply() {
        let rate = 48_000u32;
        let (mut sim, ring_prod) = test_preview(rate, 8_192, 480, true);
        sim.assert_quality_switch_glitch_free(&ring_prod).unwrap();
    }
}
