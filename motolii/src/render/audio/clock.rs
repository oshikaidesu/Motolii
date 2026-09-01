
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::doc::core::{Fps, RationalTime, RationalTimeError};

#[inline]
pub fn perceptual_sample_frames(supplied_frames: u64, device_wait_frames: u64) -> u64 {
    supplied_frames.saturating_sub(device_wait_frames)
}

pub fn sample_frames_to_time(
    frames: u64,
    sample_rate: u32,
) -> Result<RationalTime, RationalTimeError> {
    if sample_rate == 0 {
        return Err(RationalTimeError::ZeroDenominator);
    }
    RationalTime::try_new(frames as i64, sample_rate as i64)
}

pub fn display_frame_without_latency_compensation(
    supplied_frames: u64,
    sample_rate: u32,
    fps: Fps,
) -> Result<i64, RationalTimeError> {
    sample_frames_to_time(supplied_frames, sample_rate)?.try_to_frame_floor(fps)
}

pub fn synced_display_frame(
    perceptual_time: RationalTime,
    fps: Fps,
) -> Result<i64, RationalTimeError> {
    perceptual_time.try_to_frame_floor(fps)
}

pub fn drift_within_one_frame(
    display_frame: i64,
    perceptual_time: RationalTime,
    fps: Fps,
) -> Result<bool, RationalTimeError> {
    let display_pts = RationalTime::try_from_frame(display_frame, fps)?;
    let frame_len = RationalTime::try_new(fps.den(), fps.num())?;
    let diff = if display_pts >= perceptual_time {
        display_pts.try_sub(perceptual_time)?
    } else {
        perceptual_time.try_sub(display_pts)?
    };
    Ok(diff <= frame_len)
}

#[derive(Debug, Default)]
pub struct DeviceWaitLatency {
    wait_frames: AtomicU64,
}

impl DeviceWaitLatency {
    pub fn wait_frames(&self) -> u64 {
        self.wait_frames.load(Ordering::Acquire)
    }

    pub fn set_wait_frames(&self, frames: u64) {
        self.wait_frames.store(frames, Ordering::Release);
    }

    pub fn update_from_output_callback(&self, info: &cpal::OutputCallbackInfo, sample_rate: u32) {
        if sample_rate == 0 {
            return;
        }
        let ts = info.timestamp();
        let wait = ts.playback.saturating_duration_since(ts.callback);
        self.set_wait_frames(duration_to_frames(wait, sample_rate));
    }
}

fn duration_to_frames(duration: std::time::Duration, sample_rate: u32) -> u64 {
    let nanos = duration.as_nanos();
    let rate = sample_rate as u128;
    ((nanos * rate + 500_000_000) / 1_000_000_000) as u64
}

#[derive(Default)]
pub struct PlaybackCounters {
    frames_supplied: AtomicU64,
    silence_frames: AtomicU64,
    underrun_events: AtomicU64,
}

impl PlaybackCounters {
    pub fn frames_supplied(&self) -> u64 {
        self.frames_supplied.load(Ordering::Acquire)
    }

    pub fn silence_frames(&self) -> u64 {
        self.silence_frames.load(Ordering::Acquire)
    }

    pub fn underrun_events(&self) -> u64 {
        self.underrun_events.load(Ordering::Acquire)
    }

    #[doc(hidden)]
    pub fn advance_supplied_for_simulation(&self, frames: u64) {
        self.frames_supplied.fetch_add(frames, Ordering::Relaxed);
    }

    pub fn record_block(&self, supplied_frames: u64, missing_frames: u64) {
        self.frames_supplied
            .fetch_add(supplied_frames, Ordering::Relaxed);
        if missing_frames > 0 {
            self.silence_frames
                .fetch_add(missing_frames, Ordering::Relaxed);
            self.underrun_events.fetch_add(1, Ordering::Relaxed);
        }
    }
}

pub struct PlaybackClock {
    counters: Arc<PlaybackCounters>,
    device_wait: Arc<DeviceWaitLatency>,
    sample_rate: u32,
    origin_time: RationalTime,
    origin_perceptual_frames: u64,
    running: bool,
}

impl PlaybackClock {
    pub fn new(
        counters: Arc<PlaybackCounters>,
        device_wait: Arc<DeviceWaitLatency>,
        sample_rate: u32,
    ) -> Result<Self, RationalTimeError> {
        if sample_rate == 0 {
            return Err(RationalTimeError::ZeroDenominator);
        }
        Ok(Self {
            counters,
            device_wait,
            sample_rate,
            origin_time: RationalTime::ZERO,
            origin_perceptual_frames: 0,
            running: false,
        })
    }

    pub fn start(&mut self, at: RationalTime) {
        self.rebase(at);
        self.running = true;
    }

    pub fn pause(&mut self) -> Result<(), RationalTimeError> {
        if self.running {
            let frozen = self.position()?;
            self.origin_time = frozen;
            self.origin_perceptual_frames = self.current_perceptual_frames();
            self.running = false;
        }
        Ok(())
    }

    pub fn resume(&mut self) {
        if !self.running {
            self.origin_perceptual_frames = self.current_perceptual_frames();
            self.running = true;
        }
    }

    pub fn seek(&mut self, to: RationalTime) {
        self.rebase(to);
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn position(&self) -> Result<RationalTime, RationalTimeError> {
        if !self.running {
            return Ok(self.origin_time);
        }
        let now = self.current_perceptual_frames();
        let advanced = now.saturating_sub(self.origin_perceptual_frames);
        let elapsed = sample_frames_to_time(advanced, self.sample_rate)?;
        self.origin_time.try_add(elapsed)
    }

    fn rebase(&mut self, at: RationalTime) {
        self.origin_time = at;
        self.origin_perceptual_frames = self.current_perceptual_frames();
    }

    fn current_perceptual_frames(&self) -> u64 {
        perceptual_sample_frames(
            self.counters.frames_supplied(),
            self.device_wait.wait_frames(),
        )
    }
}
