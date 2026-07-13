//! 有界SPSCリングバッファ(D4契約: producer/consumer所有権分離、callbackは
//! allocate/block/decodeせずリングから読むだけ)。
//!
//! `RingProducer`/`RingConsumer`は`!Sync`にして、複数writer/複数readerを型で
//! 禁止する(F-2「単一writer」規律の音声版)。`PlaybackCounters`だけが
//! `Send + Sync`で、統計監視のために両側・監視スレッドから共有できる。

use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::error::{AudioError, Result};

struct Shared {
    storage: UnsafeCell<Box<[f32]>>,
    channels: usize,
    capacity_frames: usize,
    written: AtomicU64,
    read: AtomicU64,
}

// SAFETY: `storage`への書き込みは唯一の`RingProducer`、読み出しは唯一の
// `RingConsumer`のみが行う(両ハンドルとも!Syncで&selfの複数スレッド共有を
// 型で禁止)。`written`/`read`はAcquire/Releaseで刊行順序を保証する。
unsafe impl Send for Shared {}
unsafe impl Sync for Shared {}

impl Shared {
    fn buffered_frames(&self) -> usize {
        let w = self.written.load(Ordering::Acquire);
        let r = self.read.load(Ordering::Acquire);
        w.saturating_sub(r) as usize
    }

    fn free_frames(&self) -> usize {
        self.capacity_frames.saturating_sub(self.buffered_frames())
    }
}

/// 書き込み側ハンドル(プロデューサスレッドが単独所有)。
///
/// `!Sync`(型レベルで単独所有を強制)の構造的証明 — 参照を複数スレッドへ
/// 共有しようとするコードはコンパイルできない(D4完了条件: producer/consumer
/// 所有権分離の型レベル証明):
///
/// ```compile_fail
/// let (producer, _consumer) = motolii_audio::channel(2, 8).unwrap();
/// std::thread::scope(|s| {
///     s.spawn(|| { let _ = &producer; });
///     s.spawn(|| { let _ = &producer; }); // &RingProducerはSendではない
/// });
/// ```
pub struct RingProducer {
    shared: Arc<Shared>,
    _not_sync: PhantomData<std::cell::Cell<()>>,
}

/// 読み出し側ハンドル(音声コールバックが単独所有)。
pub struct RingConsumer {
    shared: Arc<Shared>,
    _not_sync: PhantomData<std::cell::Cell<()>>,
}

/// SPSCリングを1組のproducer/consumerへ分割する。
///
/// `channels == 0`または`capacity_frames == 0`は型付きエラー(D4契約: 境界検査)。
pub fn channel(channels: u16, capacity_frames: usize) -> Result<(RingProducer, RingConsumer)> {
    if channels == 0 || capacity_frames == 0 {
        return Err(AudioError::InvalidRingConfig {
            channels,
            capacity_frames,
        });
    }
    let ch = channels as usize;
    let len = capacity_frames * ch;
    let shared = Arc::new(Shared {
        storage: UnsafeCell::new(vec![0.0f32; len].into_boxed_slice()),
        channels: ch,
        capacity_frames,
        written: AtomicU64::new(0),
        read: AtomicU64::new(0),
    });
    Ok((
        RingProducer {
            shared: Arc::clone(&shared),
            _not_sync: PhantomData,
        },
        RingConsumer {
            shared,
            _not_sync: PhantomData,
        },
    ))
}

impl RingProducer {
    pub fn channels(&self) -> usize {
        self.shared.channels
    }

    pub fn free_frames(&self) -> usize {
        self.shared.free_frames()
    }

    /// インターリーブ`src`(長さはchannelsの倍数)を空き分だけ書く。
    /// 満杯なら0を返す(ブロックしない — プロデューサ側で待つかリトライする)。
    pub fn push_frames(&self, src: &[f32]) -> usize {
        let shared = &*self.shared;
        if shared.channels == 0 || !src.len().is_multiple_of(shared.channels) {
            return 0;
        }
        let frames_in = src.len() / shared.channels;
        let n = frames_in.min(shared.free_frames());
        if n == 0 {
            return 0;
        }
        let w = shared.written.load(Ordering::Relaxed);
        // SAFETY: 唯一のRingProducer(!Sync)のみがここに到達する。コンシューマは
        // `read`未満のスロットしか触らないので、書き込み範囲との重複はない。
        let storage = unsafe { &mut *shared.storage.get() };
        for i in 0..n {
            let slot = ((w as usize + i) % shared.capacity_frames) * shared.channels;
            let off = i * shared.channels;
            storage[slot..slot + shared.channels].copy_from_slice(&src[off..off + shared.channels]);
        }
        shared.written.fetch_add(n as u64, Ordering::Release);
        n
    }
}

impl RingConsumer {
    pub fn channels(&self) -> usize {
        self.shared.channels
    }

    pub fn buffered_frames(&self) -> usize {
        self.shared.buffered_frames()
    }

    /// リクエストしたサンプル数(インターリーブ)を読む。不足時は読めた分だけを
    /// 前方に詰めて返す(呼び出し側が末尾を無音で埋める — アロケーション・ブロック無し)。
    fn pop_samples(&self, dst: &mut [f32]) -> usize {
        let shared = &*self.shared;
        if shared.channels == 0 || dst.is_empty() || !dst.len().is_multiple_of(shared.channels) {
            return 0;
        }
        let frames_req = dst.len() / shared.channels;
        let n = frames_req.min(shared.buffered_frames());
        if n == 0 {
            return 0;
        }
        let r = shared.read.load(Ordering::Relaxed);
        // SAFETY: 唯一のRingConsumer(!Sync)のみがここに到達する。プロデューサは
        // `written`以上のスロットしか触らないので、読み出し範囲との重複はない。
        let storage = unsafe { &*shared.storage.get() };
        for i in 0..n {
            let slot = ((r as usize + i) % shared.capacity_frames) * shared.channels;
            let off = i * shared.channels;
            dst[off..off + shared.channels].copy_from_slice(&storage[slot..slot + shared.channels]);
        }
        shared.read.fetch_add(n as u64, Ordering::Release);
        n * shared.channels
    }
}

/// 実供給フレーム数とアンダーラン(無音補填)フレーム数を分離して数える監視口。
///
/// `Send + Sync`。D4契約の核心: 論理sample位置の正本は`frames_supplied`のみで、
/// 無音補填分(`silence_frames`)はこれを進めない([D5]がクロックを組む土台)。
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
}

/// コールバック本体(D4契約: allocate/block/decodeしない。リングから読むだけ)。
///
/// リングから読めた分は`dst`へコピーし、不足分は無音(0.0)で埋める。
/// アロケーション・ロック・sleepを一切行わない純関数 — cpalの音声コールバックと
/// ハードウェア無しの決定的シミュレーションの両方から同じ経路を通す
/// (プレビュー/書き出し同一関数と同型の規律)。
pub fn fill_or_silence(consumer: &RingConsumer, dst: &mut [f32], counters: &PlaybackCounters) {
    if dst.is_empty() {
        return;
    }
    let channels = consumer.channels().max(1);
    let popped = consumer.pop_samples(dst);
    if popped < dst.len() {
        dst[popped..].fill(0.0);
        let missing_frames = ((dst.len() - popped) / channels) as u64;
        counters
            .silence_frames
            .fetch_add(missing_frames, Ordering::Relaxed);
        counters.underrun_events.fetch_add(1, Ordering::Relaxed);
    }
    let supplied_frames = (popped / channels) as u64;
    counters
        .frames_supplied
        .fetch_add(supplied_frames, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_channels_or_capacity() {
        assert!(matches!(
            channel(0, 8),
            Err(AudioError::InvalidRingConfig {
                channels: 0,
                capacity_frames: 8
            })
        ));
        assert!(matches!(
            channel(2, 0),
            Err(AudioError::InvalidRingConfig {
                channels: 2,
                capacity_frames: 0
            })
        ));
    }

    #[test]
    fn push_pop_roundtrip() {
        let (prod, cons) = channel(2, 8).unwrap();
        let src = [1.0, -1.0, 2.0, -2.0];
        assert_eq!(prod.push_frames(&src), 2);
        let counters = PlaybackCounters::default();
        let mut dst = [0.0; 4];
        fill_or_silence(&cons, &mut dst, &counters);
        assert_eq!(dst, src);
        assert_eq!(counters.frames_supplied(), 2);
        assert_eq!(counters.silence_frames(), 0);
        assert_eq!(counters.underrun_events(), 0);
    }

    #[test]
    fn underrun_fills_silence_and_does_not_advance_logical_position() {
        let (prod, cons) = channel(1, 4).unwrap();
        prod.push_frames(&[0.5]);
        let counters = PlaybackCounters::default();
        let mut dst = [1.0, 1.0]; // 事前に非0を入れて「上書きされた」ことを検証
        fill_or_silence(&cons, &mut dst, &counters);
        assert_eq!(dst, [0.5, 0.0]);
        // 実供給は1フレームのみ。無音補填分は論理sample位置(frames_supplied)に
        // 加算されない — D4完了条件の核心。
        assert_eq!(counters.frames_supplied(), 1);
        assert_eq!(counters.silence_frames(), 1);
        assert_eq!(counters.underrun_events(), 1);
    }

    #[test]
    fn full_ring_rejects_extra_push_without_blocking() {
        let (prod, _cons) = channel(1, 2).unwrap();
        assert_eq!(prod.push_frames(&[1.0, 2.0]), 2);
        assert_eq!(prod.push_frames(&[3.0]), 0);
        assert_eq!(prod.free_frames(), 0);
    }

    /// producer/consumer所有権分離の構造的証明(D4完了条件)。
    ///
    /// `RingProducer`/`RingConsumer`は生スレッドへ移動(`Send`)できるが、
    /// `&self`を複数スレッドへ共有する経路(`Sync`)は`PhantomData<Cell<()>>`で
    /// 型レベルに塞いである。次の行は、その型が実際に`Send`であることを
    /// コンパイル時に検証する(型に`Sync`境界を要求する呼び出しをここに書けば、
    /// `!Sync`のため**コンパイルが失敗する**契約になっている — 意図的にコメントで
    /// その事実だけを記録し、`Sync`要求版は追加しない)。
    #[test]
    fn handles_are_send_and_counters_are_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<RingProducer>();
        assert_send::<RingConsumer>();
        assert_sync::<PlaybackCounters>();
        assert_send::<PlaybackCounters>();
    }
}
