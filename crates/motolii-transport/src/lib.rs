//! M2-D5 Transport: 音声クロック常時主 + 映像フレームドロップ + 適応解像度(DRS)。
//!
//! docs/specs/M2-document-model.md「音声トランスポート設計」正本。
//! 再生位置の正本 = デバイスへ供給済みサンプル数。映像は常に最新の聴感時刻のみレンダする。

mod clock;
mod drs;
mod playback;
mod simulate;

pub use clock::{
    display_frame_without_latency_compensation, drift_within_one_frame,
    perceptual_sample_frames, sample_frames_to_time, synced_display_frame,
};
pub use drs::{DrsConfig, DrsController, DrsStage, FrameTiming};
pub use playback::{PlaybackSession, PlaybackSessionError};
pub use simulate::{
    test_preview, test_transport_headless, HalfSpeedSimReport, PreviewSimulator,
};

use std::sync::Arc;

use motolii_audio::{DeviceWaitLatency, PlaybackCounters};
use motolii_core::{Fps, Quality, RationalTime, RationalTimeError};

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error(transparent)]
    Time(#[from] RationalTimeError),
    #[error("sample_rate must be positive")]
    InvalidSampleRate,
    #[error("pcm cache read failed")]
    CacheRead,
}

/// 映像レンダ1フレーム分の計画(常に最新の聴感時刻)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FramePlan {
    /// 聴感タイムライン時刻(供給済み−デバイス待ち)。
    pub timeline_time: RationalTime,
    /// 表示フレーム添字(床)。
    pub display_frame: i64,
    /// DRS適用後のプレビュー品質。
    pub quality: Quality,
    /// 前回レンダから飛ばしたフレーム数(ドロップ)。
    pub dropped_frames: u64,
}

/// 単一の再生ヘッド(Transport)。クロック所有者はここだけ。
pub struct Transport {
    counters: Arc<PlaybackCounters>,
    device_wait: Arc<DeviceWaitLatency>,
    drs: DrsController,
    fps: Fps,
    sample_rate: u32,
    base_quality: Quality,
    last_rendered_frame: Option<i64>,
    total_dropped: u64,
    renders: u64,
}

impl Transport {
    pub fn new(
        counters: Arc<PlaybackCounters>,
        device_wait: Arc<DeviceWaitLatency>,
        fps: Fps,
        sample_rate: u32,
        base_quality: Quality,
        drs_enabled: bool,
    ) -> Result<Self, TransportError> {
        if sample_rate == 0 {
            return Err(TransportError::InvalidSampleRate);
        }
        let config = DrsConfig::from_fps(fps);
        Ok(Self {
            counters,
            device_wait,
            drs: DrsController::new(drs_enabled, config),
            fps,
            sample_rate,
            base_quality,
            last_rendered_frame: None,
            total_dropped: 0,
            renders: 0,
        })
    }

    /// GPUのtimestamp query可否から自動DRSを初期化する。
    pub fn new_with_gpu(
        counters: Arc<PlaybackCounters>,
        device_wait: Arc<DeviceWaitLatency>,
        fps: Fps,
        sample_rate: u32,
        base_quality: Quality,
        gpu: &motolii_gpu::GpuCtx,
    ) -> Result<Self, TransportError> {
        let drs_enabled = motolii_gpu::drs_available(&gpu.device);
        Self::new(
            counters,
            device_wait,
            fps,
            sample_rate,
            base_quality,
            drs_enabled,
        )
    }

    pub fn counters(&self) -> &Arc<PlaybackCounters> {
        &self.counters
    }

    pub fn device_wait(&self) -> &Arc<DeviceWaitLatency> {
        &self.device_wait
    }

    pub fn drs(&self) -> &DrsController {
        &self.drs
    }

    pub fn drs_mut(&mut self) -> &mut DrsController {
        &mut self.drs
    }

    pub fn fps(&self) -> Fps {
        self.fps
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// クロック正本: デバイスへ供給済みサンプルフレーム数。
    pub fn supplied_frames(&self) -> u64 {
        self.counters.frames_supplied()
    }

    /// 聴感サンプルフレーム数(供給済み−デバイス待ちのみ)。
    pub fn perceptual_frames(&self) -> u64 {
        perceptual_sample_frames(self.supplied_frames(), self.device_wait.wait_frames())
    }

    /// 聴感タイムライン時刻。
    pub fn perceptual_time(&self) -> Result<RationalTime, TransportError> {
        Ok(sample_frames_to_time(self.perceptual_frames(), self.sample_rate)?)
    }

    /// 映像レンダ用: 常に最新の聴感時刻だけを返す(古い時刻は手掛けない=ドロップ)。
    pub fn next_frame_plan(&mut self) -> Result<FramePlan, TransportError> {
        let timeline_time = self.perceptual_time()?;
        let display_frame = timeline_time.try_to_frame_floor(self.fps)?;
        let dropped = match self.last_rendered_frame {
            Some(last) if display_frame > last => (display_frame - last - 1) as u64,
            _ => 0,
        };
        self.total_dropped += dropped;
        self.last_rendered_frame = Some(display_frame);
        self.renders += 1;

        Ok(FramePlan {
            timeline_time,
            display_frame,
            quality: self.drs.effective_quality(self.base_quality),
            dropped_frames: dropped,
        })
    }

    /// レンダ完了後のGPU/CPU計測をDRSへ渡す。
    pub fn record_render_timing(&mut self, timing: FrameTiming) {
        self.drs.record_frame(timing);
    }

    pub fn total_dropped_frames(&self) -> u64 {
        self.total_dropped
    }

    pub fn render_count(&self) -> u64 {
        self.renders
    }

    /// 表示PTSと聴感時刻の差が1フレーム長以内か(ドリフト自動判定)。
    pub fn display_drift_within_one_frame(
        &self,
        displayed_frame: i64,
    ) -> Result<bool, TransportError> {
        Ok(drift_within_one_frame(
            displayed_frame,
            self.perceptual_time()?,
            self.fps,
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perceptual_excludes_ring_not_device_wait() {
        let counters = Arc::new(PlaybackCounters::default());
        let wait = Arc::new(DeviceWaitLatency::default());
        counters.advance_supplied_for_simulation(10_000);
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
        assert_eq!(transport.perceptual_frames(), 9_520);
    }

    #[test]
    fn frame_plan_drops_skipped_indices() {
        let counters = Arc::new(PlaybackCounters::default());
        let wait = Arc::new(DeviceWaitLatency::default());
        let mut transport = Transport::new(
            Arc::clone(&counters),
            wait,
            Fps::try_new(30, 1).unwrap(),
            48_000,
            Quality::DRAFT,
            false,
        )
        .unwrap();

        counters.advance_supplied_for_simulation(48_000); // 1s → frame 30
        let first = transport.next_frame_plan().unwrap();
        assert_eq!(first.display_frame, 30);

        counters.advance_supplied_for_simulation(96_000); // +2s → frame 90
        let second = transport.next_frame_plan().unwrap();
        assert_eq!(second.display_frame, 90);
        assert_eq!(second.dropped_frames, 59); // skipped 31..=89
        assert_eq!(transport.total_dropped_frames(), 59);
    }
}
