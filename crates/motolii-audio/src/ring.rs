use std::cell::{Cell, UnsafeCell};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::error::AudioError;

/// SPSC共有状態。`RingProducer`/`RingConsumer`経由でのみ触る。
struct Shared {
    storage: UnsafeCell<Box<[f32]>>,
    channels: usize,
    capacity_frames: usize,
    written: AtomicU64,
    read: AtomicU64,
}

// SAFETY: storageへのアクセスは単一のRingProducer(writer)と単一のRingConsumer(reader)のみ。
// 両ハンドルは!Syncなので、&selfを複数スレッドへ共有できない。
unsafe impl Send for Shared {}
unsafe impl Sync for Shared {}

impl Shared {
    fn buffered_frames(&self) -> usize {
        let w = self.written.load(Ordering::Acquire);
        let r = self.read.load(Ordering::Acquire);
        (w.saturating_sub(r)) as usize
    }

    fn free_frames(&self) -> usize {
        self.capacity_frames.saturating_sub(self.buffered_frames())
    }
}

/// 書き込み側ハンドル。`Send`だが`!Sync` — 共有`&self`での複数writerを型で禁止する。
pub struct RingProducer {
    shared: Arc<Shared>,
    _not_sync: PhantomData<Cell<()>>,
}

/// 読み出し側ハンドル。`Send`だが`!Sync` — 共有`&self`での複数readerを型で禁止する。
pub struct RingConsumer {
    shared: Arc<Shared>,
    _not_sync: PhantomData<Cell<()>>,
}

/// 原子カウンタのみを読む監視口。`Sync`で安全(UnsafeCellに触らない)。
#[derive(Clone)]
pub struct RingStats {
    shared: Arc<Shared>,
}

/// SPSCリングを1組のプロデューサ/コンシューマに分割して返す。
pub fn split(
    channels: u16,
    capacity_frames: usize,
) -> Result<(RingProducer, RingConsumer), AudioError> {
    let channels = channels as usize;
    if channels == 0 {
        return Err(AudioError::UnsupportedChannels { channels: 0 });
    }
    let len = capacity_frames
        .checked_mul(channels)
        .ok_or_else(|| AudioError::Decode("ring buffer size overflow".into()))?;
    let shared = Arc::new(Shared {
        storage: UnsafeCell::new(vec![0.0; len].into_boxed_slice()),
        channels,
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

    pub fn capacity_frames(&self) -> usize {
        self.shared.capacity_frames
    }

    pub fn buffered_frames(&self) -> usize {
        self.shared.buffered_frames()
    }

    pub fn free_frames(&self) -> usize {
        self.shared.free_frames()
    }

    pub fn frames_written(&self) -> u64 {
        self.shared.written.load(Ordering::Acquire)
    }

    pub fn stats(&self) -> RingStats {
        RingStats {
            shared: Arc::clone(&self.shared),
        }
    }

    pub fn push_frames(&self, src: &[f32]) -> usize {
        let shared = &*self.shared;
        if !src.len().is_multiple_of(shared.channels) {
            return 0;
        }
        let frames_in = src.len() / shared.channels;
        let n = frames_in.min(shared.free_frames());
        if n == 0 {
            return 0;
        }
        let w = shared.written.load(Ordering::Relaxed);
        // SAFETY: このRingProducerは一意(!Sync)。コンシューマは未読スロットだけ読む。
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

    pub fn frames_read(&self) -> u64 {
        self.shared.read.load(Ordering::Acquire)
    }

    pub fn stats(&self) -> RingStats {
        RingStats {
            shared: Arc::clone(&self.shared),
        }
    }

    /// リクエストしたサンプル数(インターリーブ)を読む。不足時は読めた分だけ返す。
    pub fn pop_samples(&self, dst: &mut [f32]) -> usize {
        let shared = &*self.shared;
        if dst.is_empty() || !dst.len().is_multiple_of(shared.channels) {
            return 0;
        }
        let frames_req = dst.len() / shared.channels;
        let n = frames_req.min(shared.buffered_frames());
        if n == 0 {
            return 0;
        }
        let r = shared.read.load(Ordering::Relaxed);
        // SAFETY: このRingConsumerは一意(!Sync)。プロデューサは空きスロットだけ書く。
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

impl RingStats {
    pub fn frames_read(&self) -> u64 {
        self.shared.read.load(Ordering::Acquire)
    }

    pub fn frames_written(&self) -> u64 {
        self.shared.written.load(Ordering::Acquire)
    }

    pub fn buffered_frames(&self) -> usize {
        self.shared.buffered_frames()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{split, RingConsumer, RingProducer, RingStats};

    #[test]
    fn push_pop_roundtrip() {
        let (prod, cons) = split(2, 8).unwrap();
        let src = [1.0, -1.0, 2.0, -2.0];
        assert_eq!(prod.push_frames(&src), 2);
        let mut dst = [0.0; 4];
        assert_eq!(cons.pop_samples(&mut dst), 4);
        assert_eq!(dst, src);
    }

    #[test]
    fn underrun_returns_partial() {
        let (prod, cons) = split(1, 4).unwrap();
        prod.push_frames(&[0.5]);
        let mut dst = [0.0; 2];
        assert_eq!(cons.pop_samples(&mut dst), 1);
        assert_eq!(dst[0], 0.5);
        assert_eq!(dst[1], 0.0);
    }

    #[test]
    fn handles_are_send_and_stats_are_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<RingProducer>();
        assert_send::<RingConsumer>();
        assert_sync::<RingStats>();
    }
}
