//! 適応解像度降格(DRS): Draft 1/2 ↔ 1/4、二重閾値+パニック+CPUバウンド除外。

use std::time::Duration;

use motolii_core::Quality;

/// DRS段階(Draft既定=1/2、降格=1/4)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrsStage {
    Half,
    Quarter,
}

impl DrsStage {
    pub const fn resolution_scale(self) -> u32 {
        match self {
            Self::Half => 2,
            Self::Quarter => 4,
        }
    }
}

/// 1フレームの計測結果(GPU timestamp query正本 + CPU壁時計)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameTiming {
    pub gpu: Duration,
    /// GPU timestamp query由来のときのみDRSが`gpu`を見る。falseなら壁時計でDRSしない。
    pub gpu_measured: bool,
    pub cpu: Duration,
    pub wall: Duration,
}

impl FrameTiming {
    /// timestamp query等で測ったGPU時間(DRS正本)。
    pub fn measured_gpu(gpu: Duration, cpu: Duration, wall: Duration) -> Self {
        Self {
            gpu,
            gpu_measured: true,
            cpu,
            wall,
        }
    }

    /// GPU計測なし(自動DRS更新は行わない)。
    pub fn unmeasured(cpu: Duration, wall: Duration) -> Self {
        Self {
            gpu: Duration::ZERO,
            gpu_measured: false,
            cpu,
            wall,
        }
    }
}

/// DRS制御則の運用調整値(永続スキーマに焼かない)。
#[derive(Debug, Clone, Copy)]
pub struct DrsConfig {
    /// 目標フレーム予算(通常 1/fps)。
    pub frame_budget: Duration,
    /// 連続超過で即時降格(UEパニック則の縮小版)。
    pub panic_consecutive_over: u32,
    /// 昇格に必要な連続余裕フレーム数。
    pub upgrade_sustain_frames: u32,
    /// 段階変更後の最小滞留(パンピング防止)。
    pub min_dwell_frames: u32,
    /// 昇格閾値 = budget × (1 − headroom)。
    pub upgrade_headroom: f64,
}

impl DrsConfig {
    pub fn from_fps(fps: motolii_core::Fps) -> Self {
        let budget_nanos = (1_000_000_000u128 * fps.den() as u128) / fps.num() as u128;
        Self {
            frame_budget: Duration::from_nanos(budget_nanos as u64),
            panic_consecutive_over: 2,
            upgrade_sustain_frames: 8,
            min_dwell_frames: 8,
            upgrade_headroom: 0.15,
        }
    }

    fn upgrade_threshold(&self) -> Duration {
        let factor = 1.0 - self.upgrade_headroom;
        Duration::from_secs_f64(self.frame_budget.as_secs_f64() * factor)
    }
}

/// 自動DRSコントローラ。
#[derive(Debug, Clone)]
pub struct DrsController {
    enabled: bool,
    config: DrsConfig,
    stage: DrsStage,
    consecutive_over: u32,
    consecutive_under: u32,
    frames_at_stage: u32,
    /// 最小滞留内の同一段階再復帰回数(テスト観測用)。
    oscillations_in_dwell: u32,
    last_change_frame: u64,
    global_frame: u64,
    stage_before_last_change: DrsStage,
}

impl DrsController {
    pub fn new(enabled: bool, config: DrsConfig) -> Self {
        Self {
            enabled,
            config,
            stage: DrsStage::Half,
            consecutive_over: 0,
            consecutive_under: 0,
            frames_at_stage: 0,
            oscillations_in_dwell: 0,
            last_change_frame: 0,
            global_frame: 0,
            stage_before_last_change: DrsStage::Half,
        }
    }

    pub fn disabled(config: DrsConfig) -> Self {
        Self::new(false, config)
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn stage(&self) -> DrsStage {
        self.stage
    }

    pub fn oscillations_in_dwell(&self) -> u32 {
        self.oscillations_in_dwell
    }

    /// timestamp query非対応等で自動DRSを切る。
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.stage = DrsStage::Half;
            self.reset_counters();
        }
    }

    pub fn effective_quality(&self, base: Quality) -> Quality {
        if !self.enabled {
            return base;
        }
        let scale = self.stage.resolution_scale().max(base.resolution_scale);
        Quality {
            resolution_scale: scale,
            ..base
        }
    }

    /// フレーム完了後に計測を渡し、段階を更新する。
    pub fn record_frame(&mut self, timing: FrameTiming) {
        self.global_frame += 1;
        self.frames_at_stage += 1;

        if !self.enabled {
            return;
        }

        if !timing.gpu_measured {
            return;
        }

        let cpu_bound = timing.cpu >= timing.gpu;
        let over = !cpu_bound && timing.gpu > self.config.frame_budget;
        let under = timing.gpu < self.config.upgrade_threshold();

        if over {
            self.consecutive_over += 1;
            self.consecutive_under = 0;
            if self.consecutive_over >= self.config.panic_consecutive_over {
                self.try_downgrade();
                self.consecutive_over = 0;
            }
        } else if under {
            self.consecutive_under += 1;
            self.consecutive_over = 0;
            if self.consecutive_under >= self.config.upgrade_sustain_frames
                && self.frames_at_stage >= self.config.min_dwell_frames
            {
                self.try_upgrade();
                self.consecutive_under = 0;
            }
        } else {
            self.consecutive_over = 0;
            self.consecutive_under = 0;
        }
    }

    fn try_downgrade(&mut self) {
        if self.stage == DrsStage::Quarter {
            return;
        }
        let prev = self.stage;
        self.stage = DrsStage::Quarter;
        self.on_stage_change(prev);
    }

    fn try_upgrade(&mut self) {
        if self.stage == DrsStage::Half {
            return;
        }
        let prev = self.stage;
        self.stage = DrsStage::Half;
        self.on_stage_change(prev);
    }

    fn on_stage_change(&mut self, from: DrsStage) {
        let since = self.global_frame.saturating_sub(self.last_change_frame);
        if since < self.config.min_dwell_frames as u64
            && self.stage == self.stage_before_last_change
        {
            self.oscillations_in_dwell += 1;
        }
        self.stage_before_last_change = from;
        self.last_change_frame = self.global_frame;
        self.frames_at_stage = 0;
    }

    fn reset_counters(&mut self) {
        self.consecutive_over = 0;
        self.consecutive_under = 0;
        self.frames_at_stage = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn over_budget(config: &DrsConfig) -> FrameTiming {
        FrameTiming::measured_gpu(
            config.frame_budget + Duration::from_millis(2),
            Duration::from_millis(1),
            config.frame_budget + Duration::from_millis(3),
        )
    }

    fn under_budget(config: &DrsConfig) -> FrameTiming {
        FrameTiming::measured_gpu(
            config.upgrade_threshold() / 2,
            Duration::from_millis(1),
            config.upgrade_threshold() / 2,
        )
    }

    #[test]
    fn panic_downgrade_after_two_consecutive_overs() {
        let fps = motolii_core::Fps::try_new(30, 1).unwrap();
        let config = DrsConfig::from_fps(fps);
        let mut drs = DrsController::new(true, config);
        assert_eq!(drs.stage(), DrsStage::Half);

        drs.record_frame(over_budget(&config));
        assert_eq!(drs.stage(), DrsStage::Half);
        drs.record_frame(over_budget(&config));
        assert_eq!(drs.stage(), DrsStage::Quarter);
    }

    #[test]
    fn cpu_bound_over_budget_does_not_downgrade() {
        let fps = motolii_core::Fps::try_new(30, 1).unwrap();
        let config = DrsConfig::from_fps(fps);
        let mut drs = DrsController::new(true, config);
        let cpu_bound = FrameTiming::measured_gpu(
            Duration::from_millis(1),
            config.frame_budget + Duration::from_millis(5),
            config.frame_budget + Duration::from_millis(5),
        );
        for _ in 0..4 {
            drs.record_frame(cpu_bound);
        }
        assert_eq!(drs.stage(), DrsStage::Half);
    }

    #[test]
    fn disabled_keeps_base_quality() {
        let fps = motolii_core::Fps::try_new(30, 1).unwrap();
        let config = DrsConfig::from_fps(fps);
        let drs = DrsController::disabled(config);
        assert_eq!(
            drs.effective_quality(Quality::DRAFT).resolution_scale,
            Quality::DRAFT.resolution_scale
        );
    }

    #[test]
    fn upgrade_requires_sustain_and_dwell() {
        let fps = motolii_core::Fps::try_new(30, 1).unwrap();
        let config = DrsConfig::from_fps(fps);
        let mut drs = DrsController::new(true, config);
        drs.stage = DrsStage::Quarter;
        drs.frames_at_stage = config.min_dwell_frames;

        for _ in 0..(config.upgrade_sustain_frames - 1) {
            drs.record_frame(under_budget(&config));
        }
        assert_eq!(drs.stage(), DrsStage::Quarter);
        drs.record_frame(under_budget(&config));
        assert_eq!(drs.stage(), DrsStage::Half);
    }

    #[test]
    fn wall_over_budget_without_gpu_measurement_does_not_downgrade() {
        let fps = motolii_core::Fps::try_new(30, 1).unwrap();
        let config = DrsConfig::from_fps(fps);
        let mut drs = DrsController::new(true, config);
        let wall_only = FrameTiming::unmeasured(
            Duration::from_millis(1),
            config.frame_budget + Duration::from_millis(10),
        );
        for _ in 0..4 {
            drs.record_frame(wall_only);
        }
        assert_eq!(drs.stage(), DrsStage::Half);
    }
}
