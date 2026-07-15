//! 音声クロック: 供給済みサンプル数をタイムライン時刻へ写す。

use motolii_core::{Fps, RationalTime, RationalTimeError};

/// 聴感再生位置 = 供給済み − デバイス待ち(リング充填は引かない)。
#[inline]
pub fn perceptual_sample_frames(supplied_frames: u64, device_wait_frames: u64) -> u64 {
    supplied_frames.saturating_sub(device_wait_frames)
}

/// デバイスサンプルフレーム位置を`RationalTime`へ(浮動小数秒を使わない)。
pub fn sample_frames_to_time(
    frames: u64,
    sample_rate: u32,
) -> Result<RationalTime, RationalTimeError> {
    if sample_rate == 0 {
        return Err(RationalTimeError::ZeroDenominator);
    }
    RationalTime::try_new(frames as i64, sample_rate as i64)
}

/// 表示フレームPTS(床)と聴感時刻の差が1フレーム長以内か。
pub fn drift_within_one_frame(
    display_frame: i64,
    perceptual_time: RationalTime,
    fps: Fps,
) -> Result<bool, RationalTimeError> {
    let display_pts = RationalTime::try_from_frame(display_frame, fps)?;
    let frame_len = RationalTime::try_new(fps.den() as i64, fps.num() as i64)?;
    let diff = if display_pts >= perceptual_time {
        display_pts.try_sub(perceptual_time)?
    } else {
        perceptual_time.try_sub(display_pts)?
    };
    Ok(diff <= frame_len)
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
}
