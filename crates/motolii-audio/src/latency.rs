//! cpalデバイス待ちレイテンシ(D5: Transportが聴感位置を求めるときだけ引く)。
//!
//! リング充填量は時計に使わない — 供給済みサンプル数との差分だけが補償対象。

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// デバイス出力コールバックで観測した待ち時間(サンプルフレーム)。
///
/// `Send + Sync`。Transportが`Arc`で共有し、deviceコールバックが更新する。
#[derive(Debug, Default)]
pub struct DeviceWaitLatency {
    wait_frames: AtomicU64,
}

impl DeviceWaitLatency {
    pub fn wait_frames(&self) -> u64 {
        self.wait_frames.load(Ordering::Acquire)
    }

    /// cpal `OutputCallbackInfo::timestamp()` の `playback − callback` を写す。
    pub fn update_from_output_callback(
        &self,
        info: &cpal::OutputCallbackInfo,
        sample_rate: u32,
    ) {
        if sample_rate == 0 {
            return;
        }
        let ts = info.timestamp();
        let wait = ts.playback.saturating_duration_since(ts.callback);
        let frames = duration_to_frames(wait, sample_rate);
        self.wait_frames.store(frames, Ordering::Release);
    }

    /// シミュレーション/テスト用。
    pub fn set_wait_frames(&self, frames: u64) {
        self.wait_frames.store(frames, Ordering::Release);
    }
}

fn duration_to_frames(duration: Duration, sample_rate: u32) -> u64 {
    let nanos = duration.as_nanos();
    let rate = sample_rate as u128;
    // 最近傍のサンプルフレームへ丸める。
    ((nanos * rate + 500_000_000) / 1_000_000_000) as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use cpal::{OutputCallbackInfo, OutputStreamTimestamp, StreamInstant};

    #[test]
    fn maps_playback_minus_callback_to_frames() {
        let latency = DeviceWaitLatency::default();
        let callback = StreamInstant::ZERO;
        let playback = StreamInstant::new(0, 10_000_000); // 10ms @48k ≈ 480 frames
        let info = OutputCallbackInfo::new(OutputStreamTimestamp {
            callback,
            playback,
        });
        latency.update_from_output_callback(&info, 48_000);
        assert_eq!(latency.wait_frames(), 480);
    }

    #[test]
    fn zero_wait_when_playback_equals_callback() {
        let latency = DeviceWaitLatency::default();
        let instant = StreamInstant::ZERO;
        let info = OutputCallbackInfo::new(OutputStreamTimestamp {
            callback: instant,
            playback: instant,
        });
        latency.update_from_output_callback(&info, 48_000);
        assert_eq!(latency.wait_frames(), 0);
    }
}
