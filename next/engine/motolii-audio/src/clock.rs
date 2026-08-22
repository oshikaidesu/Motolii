//! owns: 音声クロック — 供給済みサンプル数をタイムライン時刻へ写す(D5)。
//!
//! OWNS-JUSTIFICATION(A): 意見17と同じ `next/reference/KNOWN.md`「音声」D4/D5契約
//! — 新規部分(`PlaybackClock` の状態機械)と移植部分(旧
//! `motolii-transport`/`motolii-audio` の演算)を切り分けた上で、ここに置く
//! 必然性を doc 自身が明記している(裁定215 棚卸し 2026-08-23 #6)。
//!
//! 旧 `crates/motolii-transport/src/clock.rs`(pure 関数)+ 旧
//! `crates/motolii-audio/src/ring.rs` の `PlaybackCounters` + 旧
//! `crates/motolii-audio/src/latency.rs` の `DeviceWaitLatency::update_from_output_callback`
//! (cpal `OutputCallbackInfo` を読む pure 関数)を移した。**「audio-clock-master の骨
//! (カウンタ演算)」**が発注書の指示で、実デバイスへ音を出す経路(`device.rs`の
//! `cpal::Stream`結線・`producer.rs`のバックグラウンドスレッド)は shell 結線と
//! 併せて第2切片(A2)に残す:
//!
//! - `ring.rs` の `RingProducer`/`RingConsumer` 自体(自前SPSC、367行、KNOWN.md
//!   「音声」節で再発明と判定済み)は持ってこない。同じ役割を上流 `rtrb` に譲り、
//!   フレーム充填/無音補填ロジックだけ [`crate::ring::fill_or_silence`] として
//!   `rtrb::Consumer` 版に移した(このモジュールの [`PlaybackCounters::record_block`]
//!   を呼ぶ側)
//! - `cpal::Device`/`cpal::Stream` を開く経路(`OutputStream::open_*`)は持ってこない
//!   — `update_from_output_callback` は cpal の**型**(`OutputCallbackInfo`)だけを
//!   読む pure 関数なので、デバイスを開かずに移せる
//!
//! 論理サンプル位置の正本は**供給済みフレーム数のみ**(`frames_supplied`)。
//! 無音補填分(`silence_frames`)はこれを進めない — 発注書が指す
//! 「アンダーラン時に論理位置が進まない」契約はここで成立する。
//!
//! [`PlaybackClock`] は旧コードに直接の対応物が無い**新規の口**(旧
//! `motolii-transport::Transport` は DRS・映像フレームドロップ・`FramePlan` まで
//! 一体だったが、今回の発注は「クロックが進む・止まる・seekする」だけ)。中身は
//! 上記の移植済みの骨(`PlaybackCounters`/`DeviceWaitLatency`/`perceptual_sample_frames`/
//! `sample_frames_to_time`)を start/pause/seek の状態機械で束ねただけで、新しい
//! 演算は無い。旧 `Transport` に「seek」は無く(新しい `Transport` を作り直す設計
//! だった)、`pause`も無かった(cpalストリームの`pause()`任せ)— この2つは今回の
//! 発注が求める新しい状態管理として実装した。

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

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

/// cpalデバイス出力コールバックで観測した待ち時間(サンプルフレーム)の置き場。
///
/// `Send + Sync`。実測値の書き込みは [`Self::update_from_output_callback`](cpal
/// コールバックが直接呼ぶ)。実際に`cpal::Stream`を開いてこのコールバックへ
/// 配線する経路(`build_output_stream`)は第2切片(A2、shell結線)の仕事 —
/// ここは`OutputCallbackInfo`という**値**を受けるだけで、デバイスは持たない。
#[derive(Debug, Default)]
pub struct DeviceWaitLatency {
    wait_frames: AtomicU64,
}

impl DeviceWaitLatency {
    pub fn wait_frames(&self) -> u64 {
        self.wait_frames.load(Ordering::Acquire)
    }

    /// シミュレーション/テスト用、および実デバイス結線が直接書く口。
    pub fn set_wait_frames(&self, frames: u64) {
        self.wait_frames.store(frames, Ordering::Release);
    }

    /// cpal `OutputCallbackInfo::timestamp()` の `playback − callback` を写す
    /// (旧 `crates/motolii-audio/src/latency.rs::update_from_output_callback` の移植)。
    /// デバイスを開かずに呼べる — `OutputCallbackInfo`/`StreamInstant` は値型。
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
    // 最近傍のサンプルフレームへ丸める(旧 latency.rs と同じ丸め)。
    ((nanos * rate + 500_000_000) / 1_000_000_000) as u64
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

/// 再生クロック: start/pause/seek と現在の論理再生位置(`RationalTime`)を持つ口。
///
/// `PlaybackCounters`/`DeviceWaitLatency` を**読むだけ**(誰がそれらを進めるかは
/// 知らない — 実デバイス経路(cpal callback → [`crate::ring::fill_or_silence`] →
/// `record_block`)でもヘッドレスシミュレーション(`advance_supplied_for_simulation`)
/// でも同じ口が動く。「device 抽象の裏」という発注書の指示はこの分離のこと)。
///
/// # 状態機械
/// - `start(at)`: `at` を新しい原点にして走行開始(既に走行中でも安全 — 原点を
///   置き直すだけ)
/// - `pause()`: 呼んだ瞬間の論理位置を凍結する。凍結後にカウンタがさらに進んでも
///   (実デバイスの`pause()`が非同期で数コールバック遅れて効く競合を想定)
///   `position()`は凍結値を返し続ける — カウンタの継続に賭けない
/// - `resume()`: 凍結位置から走行を再開する(pause中に競合で進んだ分は捨てる)
/// - `seek(to)`: 走行状態を変えずに原点を`to`へ置き直す(旧`Transport`は
///   seekの度に新しいセッションを作る設計だったが、ここは原点の付け替えで表現する)
pub struct PlaybackClock {
    counters: Arc<PlaybackCounters>,
    device_wait: Arc<DeviceWaitLatency>,
    sample_rate: u32,
    origin_time: RationalTime,
    origin_perceptual_frames: u64,
    running: bool,
}

impl PlaybackClock {
    /// 停止状態(原点=ZERO)で構築する。`start`を呼ぶまで`position()`は`ZERO`のまま。
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

    /// `at`を新しい原点として走行を開始する。
    pub fn start(&mut self, at: RationalTime) {
        self.rebase(at);
        self.running = true;
    }

    /// 現在位置を凍結して停止する(冪等 — 既に停止中なら何もしない)。
    pub fn pause(&mut self) -> Result<(), RationalTimeError> {
        if self.running {
            let frozen = self.position()?;
            self.origin_time = frozen;
            self.origin_perceptual_frames = self.current_perceptual_frames();
            self.running = false;
        }
        Ok(())
    }

    /// 凍結位置から走行を再開する(冪等 — 既に走行中なら何もしない)。
    pub fn resume(&mut self) {
        if !self.running {
            self.origin_perceptual_frames = self.current_perceptual_frames();
            self.running = true;
        }
    }

    /// 走行状態を変えずに`to`へ跳ぶ。
    pub fn seek(&mut self, to: RationalTime) {
        self.rebase(to);
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// 現在の論理再生位置。停止中は凍結値(カウンタの続きを見ない)。
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

    /// ORACLE(a): 論理位置は供給に応じて単調に進む(device抽象=フェイクのcountersを
    /// 直接進める。実デバイス・実cpalは要らない)。
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

    /// ORACLE(b): pauseで止まる — 凍結後にcountersがさらに進んでも(実デバイスの
    /// 非同期pause競合を模して)position()は動かない。
    #[test]
    fn pause_freezes_position_even_if_counters_keep_advancing() {
        let (mut clock, counters, _wait) = fake_clock(48_000);
        clock.start(RationalTime::ZERO);
        counters.advance_supplied_for_simulation(48_000); // 1.0s
        clock.pause().unwrap();
        let frozen = clock.position().unwrap();
        assert_eq!(frozen, RationalTime::from_seconds(1));
        assert!(!clock.is_running());

        // pause直後もコールバックが数回追加で鳴った、という競合を模す。
        counters.advance_supplied_for_simulation(48_000); // さらに+1.0s供給されても…
        assert_eq!(
            clock.position().unwrap(),
            frozen,
            "停止中はcountersの続きを見ない"
        );

        clock.resume();
        assert!(clock.is_running());
        // resume直後は凍結点から。resume前に競合で進んだ分は数えない。
        assert_eq!(clock.position().unwrap(), frozen);
        counters.advance_supplied_for_simulation(24_000); // +0.5s
        assert_eq!(
            clock.position().unwrap(),
            frozen
                .try_add(RationalTime::try_new(1, 2).unwrap())
                .unwrap()
        );
    }

    /// ORACLE(c): seekで跳ぶ — 走行中でも停止中でも即座にpositionが目標値になる。
    #[test]
    fn seek_jumps_position_immediately() {
        let (mut clock, counters, _wait) = fake_clock(48_000);
        clock.start(RationalTime::ZERO);
        counters.advance_supplied_for_simulation(48_000);
        assert_eq!(clock.position().unwrap(), RationalTime::from_seconds(1));

        let target = RationalTime::from_seconds(10);
        clock.seek(target);
        assert_eq!(clock.position().unwrap(), target, "seek直後は即座に目標値");

        // seek後も供給に応じてそこから進む。
        counters.advance_supplied_for_simulation(48_000);
        assert_eq!(clock.position().unwrap(), RationalTime::from_seconds(11));

        // 停止中のseekも同様に即座反映(走行状態は変えない)。
        clock.pause().unwrap();
        clock.seek(RationalTime::ZERO);
        assert_eq!(clock.position().unwrap(), RationalTime::ZERO);
        assert!(!clock.is_running(), "seekはrunning状態を変えない");
    }

    /// ORACLE(d): 無音補填(underrun)ではposition()が進まない —
    /// `record_block`のmissing_framesはfrmes_suppliedに乗らない契約をPlaybackClock
    /// 経由でも確認する。
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
