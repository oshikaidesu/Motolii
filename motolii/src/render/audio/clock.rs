
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use motolii_core::{Fps, RationalTime, RationalTimeError};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perceptual_subtracts_device_wait_only() {
        assert_eq!(perceptual_sample_frames(10_000, 480), 9_520);
        assert_eq!(perceptual_sample_frames(100, 200), 0);
    }

    #[test]
    fn sample_frames_to_time_matches_rational() {
        let t = sample_frames_to_time(48_000, 48_000).unwrap();
        assert_eq!(t, RationalTime::from_seconds(1));
    }

    #[test]
    fn drift_within_one_frame_at_same_floor() {
        let fps = Fps::try_new(30, 1).unwrap();
        let perceptual = RationalTime::try_new(11, 30).unwrap(); // frame 11 + 1/30
        assert!(drift_within_one_frame(11, perceptual, fps).unwrap());
        assert!(!drift_within_one_frame(9, perceptual, fps).unwrap());
    }

    #[test]
    fn full_block_without_underrun_only_advances_supplied() {
        let counters = PlaybackCounters::default();
        counters.record_block(480, 0);
        assert_eq!(counters.frames_supplied(), 480);
        assert_eq!(counters.silence_frames(), 0);
        assert_eq!(counters.underrun_events(), 0);
    }

    #[test]
    fn underrun_fills_silence_and_does_not_advance_logical_position() {
        let counters = PlaybackCounters::default();
        counters.record_block(1, 3);
        assert_eq!(
            counters.frames_supplied(),
            1,
            "無音補填分は論理sample位置(frames_supplied)に加算されない"
        );
        assert_eq!(counters.silence_frames(), 3);
        assert_eq!(counters.underrun_events(), 1);

        assert_eq!(perceptual_sample_frames(counters.frames_supplied(), 0), 1);
    }

    #[test]
    fn repeated_underruns_accumulate_independently_of_supplied() {
        let counters = PlaybackCounters::default();
        counters.record_block(10, 0);
        counters.record_block(0, 5);
        counters.record_block(2, 1);
        assert_eq!(counters.frames_supplied(), 12);
        assert_eq!(counters.silence_frames(), 6);
        assert_eq!(counters.underrun_events(), 2);
    }

    #[test]
    fn one_second_origin_is_exact_at_device_rates() {
        let origin = sample_frames_to_time(48_000, 48_000).unwrap();
        for sample_rate in [48_000, 44_100] {
            let elapsed = sample_frames_to_time(0, sample_rate).unwrap();
            assert_eq!(
                origin.try_add(elapsed).unwrap(),
                RationalTime::from_seconds(1)
            );
        }
    }

    #[test]
    fn device_wait_subtracts_from_elapsed_device_frames_only() {
        for (sample_rate, wait_frames) in [(48_000u32, 480u64), (44_100, 441)] {
            let counters = PlaybackCounters::default();
            let wait = DeviceWaitLatency::default();
            counters.advance_supplied_for_simulation(sample_rate as u64);
            wait.set_wait_frames(wait_frames);

            let elapsed_frames = sample_rate as u64 - wait_frames;
            assert_eq!(counters.frames_supplied(), sample_rate as u64);
            assert_eq!(
                perceptual_sample_frames(counters.frames_supplied(), wait.wait_frames()),
                elapsed_frames
            );
        }
    }

    #[test]
    fn update_from_output_callback_maps_playback_minus_callback_to_frames() {
        use cpal::{OutputCallbackInfo, OutputStreamTimestamp, StreamInstant};

        let latency = DeviceWaitLatency::default();
        let callback = StreamInstant::ZERO;
        let playback = StreamInstant::new(0, 10_000_000); // 10ms @48k ≈ 480 frames
        let info = OutputCallbackInfo::new(OutputStreamTimestamp { callback, playback });
        latency.update_from_output_callback(&info, 48_000);
        assert_eq!(latency.wait_frames(), 480);
    }

    #[test]
    fn update_from_output_callback_zero_wait_when_playback_equals_callback() {
        use cpal::{OutputCallbackInfo, OutputStreamTimestamp, StreamInstant};

        let latency = DeviceWaitLatency::default();
        let instant = StreamInstant::ZERO;
        let info = OutputCallbackInfo::new(OutputStreamTimestamp {
            callback: instant,
            playback: instant,
        });
        latency.update_from_output_callback(&info, 48_000);
        assert_eq!(latency.wait_frames(), 0);
    }

    fn fake_clock(
        sample_rate: u32,
    ) -> (PlaybackClock, Arc<PlaybackCounters>, Arc<DeviceWaitLatency>) {
        let counters = Arc::new(PlaybackCounters::default());
        let wait = Arc::new(DeviceWaitLatency::default());
        let clock =
            PlaybackClock::new(Arc::clone(&counters), Arc::clone(&wait), sample_rate).unwrap();
        (clock, counters, wait)
    }

    #[test]
    fn position_advances_monotonically_with_supply() {
        let (mut clock, counters, _wait) = fake_clock(48_000);
        clock.start(RationalTime::ZERO);
        assert_eq!(clock.position().unwrap(), RationalTime::ZERO);

        counters.advance_supplied_for_simulation(24_000); // +0.5s
        let half = clock.position().unwrap();
        assert_eq!(half, RationalTime::try_new(1, 2).unwrap());

        counters.advance_supplied_for_simulation(24_000); // +0.5s → 1.0s
        let one = clock.position().unwrap();
        assert_eq!(one, RationalTime::from_seconds(1));
        assert!(one > half, "供給が増えた分だけ単調に進む");
    }

    #[test]
    fn pause_freezes_position_even_if_counters_keep_advancing() {
        let (mut clock, counters, _wait) = fake_clock(48_000);
        clock.start(RationalTime::ZERO);
        counters.advance_supplied_for_simulation(48_000); // 1.0s
        clock.pause().unwrap();
        let frozen = clock.position().unwrap();
        assert_eq!(frozen, RationalTime::from_seconds(1));
        assert!(!clock.is_running());

        counters.advance_supplied_for_simulation(48_000); // さらに+1.0s供給されても…
        assert_eq!(
            clock.position().unwrap(),
            frozen,
            "停止中はcountersの続きを見ない"
        );

        clock.resume();
        assert!(clock.is_running());
        assert_eq!(clock.position().unwrap(), frozen);
        counters.advance_supplied_for_simulation(24_000); // +0.5s
        assert_eq!(
            clock.position().unwrap(),
            frozen
                .try_add(RationalTime::try_new(1, 2).unwrap())
                .unwrap()
        );
    }

    #[test]
    fn seek_jumps_position_immediately() {
        let (mut clock, counters, _wait) = fake_clock(48_000);
        clock.start(RationalTime::ZERO);
        counters.advance_supplied_for_simulation(48_000);
        assert_eq!(clock.position().unwrap(), RationalTime::from_seconds(1));

        let target = RationalTime::from_seconds(10);
        clock.seek(target);
        assert_eq!(clock.position().unwrap(), target, "seek直後は即座に目標値");

        counters.advance_supplied_for_simulation(48_000);
        assert_eq!(clock.position().unwrap(), RationalTime::from_seconds(11));

        clock.pause().unwrap();
        clock.seek(RationalTime::ZERO);
        assert_eq!(clock.position().unwrap(), RationalTime::ZERO);
        assert!(!clock.is_running(), "seekはrunning状態を変えない");
    }

    #[test]
    fn underrun_silence_does_not_advance_position() {
        let (mut clock, counters, _wait) = fake_clock(48_000);
        clock.start(RationalTime::ZERO);

        counters.record_block(24_000, 0); // 実供給0.5s
        let after_real_supply = clock.position().unwrap();
        assert_eq!(after_real_supply, RationalTime::try_new(1, 2).unwrap());

        counters.record_block(0, 24_000); // 丸ごとアンダーラン、無音で0.5s分埋める
        assert_eq!(
            clock.position().unwrap(),
            after_real_supply,
            "無音補填分は論理位置に一切現れない"
        );
        assert_eq!(counters.silence_frames(), 24_000);
    }

    #[test]
    fn zero_sample_rate_is_rejected_at_construction() {
        let counters = Arc::new(PlaybackCounters::default());
        let wait = Arc::new(DeviceWaitLatency::default());
        assert!(matches!(
            PlaybackClock::new(counters, wait, 0),
            Err(RationalTimeError::ZeroDenominator)
        ));
    }
}
