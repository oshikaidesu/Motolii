//! owns: 音声クロック — 供給済みサンプル数をタイムライン時刻へ写す(D5)。
//!
//! 旧 `crates/motolii-transport/src/clock.rs`(pure 関数)+ 旧
//! `crates/motolii-audio/src/ring.rs` の `PlaybackCounters`(cpal/rtrb を持たない
//! カウンタ部分だけ)を移した。**「audio-clock-master の骨(カウンタ演算)」**が
//! 発注書の指示で、device 結線(cpal の `OutputCallbackInfo` / rtrb のリング)は
//! 第2切片に残す:
//!
//! - `DeviceWaitLatency::update_from_output_callback`(cpal 型を受ける)は持ってこない
//!   — `wait_frames`/`set_wait_frames` の pure な読み書きだけ移した
//! - `ring.rs` の `RingProducer`/`RingConsumer`/`fill_or_silence`(rtrb 依存)は
//!   持ってこない。`fill_or_silence` が更新していたカウンタの意味だけを
//!   [`PlaybackCounters::record_block`] として抜き出した — 第2切片で rtrb を足す日、
//!   producer 側は pop した後にこれを1回呼ぶだけでよい(カウンタの意味を2箇所で
//!   持たない)
//!
//! 論理サンプル位置の正本は**供給済みフレーム数のみ**(`frames_supplied`)。
//! 無音補填分(`silence_frames`)はこれを進めない — 発注書が指す
//! 「アンダーラン時に論理位置が進まない」契約はここで成立する。

use std::sync::atomic::{AtomicU64, Ordering};

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

/// 補償なし(供給済み直結)の表示フレーム床 — ドリフトテストの対照用。
pub fn display_frame_without_latency_compensation(
    supplied_frames: u64,
    sample_rate: u32,
    fps: Fps,
) -> Result<i64, RationalTimeError> {
    sample_frames_to_time(supplied_frames, sample_rate)?.try_to_frame_floor(fps)
}

/// 聴感時刻から独立に同期表示フレームを求める(`next_frame_plan`と同等の床)。
pub fn synced_display_frame(
    perceptual_time: RationalTime,
    fps: Fps,
) -> Result<i64, RationalTimeError> {
    perceptual_time.try_to_frame_floor(fps)
}

/// 表示フレームPTS(床)と聴感時刻の差が1フレーム長以内か。
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

/// cpalデバイス出力コールバックで観測した待ち時間(サンプルフレーム)の pure な置き場。
///
/// `Send + Sync`。実測値の書き込み(`update_from_output_callback` 相当)は cpal 結線
/// を持つ第2切片の仕事 — ここは `Transport` が読む口と、シミュレーション用の
/// 書き口だけを持つ。
#[derive(Debug, Default)]
pub struct DeviceWaitLatency {
    wait_frames: AtomicU64,
}

impl DeviceWaitLatency {
    pub fn wait_frames(&self) -> u64 {
        self.wait_frames.load(Ordering::Acquire)
    }

    /// シミュレーション/テスト用、および第2切片の device コールバックが直接書く口。
    pub fn set_wait_frames(&self, frames: u64) {
        self.wait_frames.store(frames, Ordering::Release);
    }
}

/// 実供給フレーム数とアンダーラン(無音補填)フレーム数を分離して数える監視口。
///
/// `Send + Sync`。D4契約の核心: 論理sample位置の正本は`frames_supplied`のみで、
/// 無音補填分(`silence_frames`)はこれを進めない(D5がクロックを組む土台)。
#[derive(Default)]
pub struct PlaybackCounters {
    frames_supplied: AtomicU64,
    silence_frames: AtomicU64,
    underrun_events: AtomicU64,
}

impl PlaybackCounters {
    /// 実PCMサンプルから供給できたフレーム数(=論理sample位置)。
    pub fn frames_supplied(&self) -> u64 {
        self.frames_supplied.load(Ordering::Acquire)
    }

    /// アンダーランで無音を充填したフレーム数(論理sample位置には加算されない)。
    pub fn silence_frames(&self) -> u64 {
        self.silence_frames.load(Ordering::Acquire)
    }

    /// アンダーランが発生したコールバック呼び出し回数。
    pub fn underrun_events(&self) -> u64 {
        self.underrun_events.load(Ordering::Acquire)
    }

    /// ヘッドレスTransportシミュレーション専用: 実PCMを伴わず供給済みだけ進める。
    #[doc(hidden)]
    pub fn advance_supplied_for_simulation(&self, frames: u64) {
        self.frames_supplied.fetch_add(frames, Ordering::Relaxed);
    }

    /// 1回のデバイスコールバック(または headless シミュレーション)分の結果を記録する。
    ///
    /// 旧 `ring.rs::fill_or_silence` のカウンタ更新をそのまま抜き出した pure 関数 —
    /// リングから実際に読めたフレーム数(`supplied_frames`)と、無音で埋めた分
    /// (`missing_frames`)を呼び出し側(第2切片の producer)が計算して渡す。
    /// **`missing_frames` は `frames_supplied` に加算しない** — これが D4 完了条件の核心。
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

    /// 発注書の契約: 「アンダーラン時に論理位置が進まない」。
    ///
    /// 旧 `ring.rs::underrun_fills_silence_and_does_not_advance_logical_position` の
    /// 移植 — リングを介さず、pop できたフレーム数と埋めた無音フレーム数を直接渡す形。
    #[test]
    fn underrun_fills_silence_and_does_not_advance_logical_position() {
        let counters = PlaybackCounters::default();
        // 4フレーム分の枠に対し実際は1フレームしか読めなかった、を模す。
        counters.record_block(1, 3);
        assert_eq!(
            counters.frames_supplied(),
            1,
            "無音補填分は論理sample位置(frames_supplied)に加算されない"
        );
        assert_eq!(counters.silence_frames(), 3);
        assert_eq!(counters.underrun_events(), 1);

        // 聴感位置(perceptual_sample_frames)はdevice_wait=0なら供給済みそのもの —
        // 無音補填でごまかした3フレーム分は聴感位置に一切現れない。
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
}
