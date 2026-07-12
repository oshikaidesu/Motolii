use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::AudioError;

/// SPSCリングバッファ(フレーム単位・インターリーブf32)。
///
/// 書き込みはプロデューサスレッド、読み出しはcpalコールバックのみ —
/// コールバックがブロックしないための境界(D4)。
pub struct SampleRing {
    storage: UnsafeCell<Box<[f32]>>,
    channels: usize,
    capacity_frames: usize,
    written: AtomicU64,
    read: AtomicU64,
}

// SAFETY: 単一writer(プロデューサ)と単一reader(コールバック)のみが
// storageの同一スロットへ触る。インデックスはwritten/readのAcquire/Releaseで同期。
unsafe impl Sync for SampleRing {}

impl SampleRing {
    pub fn new(channels: u16, capacity_frames: usize) -> Result<Self, AudioError> {
        let channels = channels as usize;
        if channels == 0 {
            return Err(AudioError::UnsupportedChannels { channels: 0 });
        }
        let len = capacity_frames
            .checked_mul(channels)
            .ok_or_else(|| AudioError::Decode("ring buffer size overflow".into()))?;
        Ok(Self {
            storage: UnsafeCell::new(vec![0.0; len].into_boxed_slice()),
            channels,
            capacity_frames,
            written: AtomicU64::new(0),
            read: AtomicU64::new(0),
        })
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    pub fn capacity_frames(&self) -> usize {
        self.capacity_frames
    }

    pub fn buffered_frames(&self) -> usize {
        let w = self.written.load(Ordering::Acquire);
        let r = self.read.load(Ordering::Acquire);
        (w.saturating_sub(r)) as usize
    }

    pub fn free_frames(&self) -> usize {
        self.capacity_frames.saturating_sub(self.buffered_frames())
    }

    pub fn frames_written(&self) -> u64 {
        self.written.load(Ordering::Acquire)
    }

    pub fn frames_read(&self) -> u64 {
        self.read.load(Ordering::Acquire)
    }

    pub fn push_frames(&self, src: &[f32]) -> usize {
        if !src.len().is_multiple_of(self.channels) {
            return 0;
        }
        let frames_in = src.len() / self.channels;
        let n = frames_in.min(self.free_frames());
        if n == 0 {
            return 0;
        }
        let w = self.written.load(Ordering::Relaxed);
        let storage = unsafe { &mut *self.storage.get() };
        for i in 0..n {
            let slot = ((w as usize + i) % self.capacity_frames) * self.channels;
            let off = i * self.channels;
            storage[slot..slot + self.channels].copy_from_slice(&src[off..off + self.channels]);
        }
        self.written.fetch_add(n as u64, Ordering::Release);
        n
    }

    /// リクエストしたサンプル数(インターリーブ)を読む。不足分は0で埋め、戻り値は読めたサンプル数。
    pub fn pop_samples(&self, dst: &mut [f32]) -> usize {
        if dst.is_empty() || !dst.len().is_multiple_of(self.channels) {
            return 0;
        }
        let frames_req = dst.len() / self.channels;
        let n = frames_req.min(self.buffered_frames());
        if n == 0 {
            return 0;
        }
        let r = self.read.load(Ordering::Relaxed);
        let storage = unsafe { &*self.storage.get() };
        for i in 0..n {
            let slot = ((r as usize + i) % self.capacity_frames) * self.channels;
            let off = i * self.channels;
            dst[off..off + self.channels].copy_from_slice(&storage[slot..slot + self.channels]);
        }
        self.read.fetch_add(n as u64, Ordering::Release);
        n * self.channels
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::SampleRing;

    #[test]
    fn push_pop_roundtrip() {
        let ring = SampleRing::new(2, 8).unwrap();
        let src = [1.0, -1.0, 2.0, -2.0];
        assert_eq!(ring.push_frames(&src), 2);
        let mut dst = [0.0; 4];
        assert_eq!(ring.pop_samples(&mut dst), 4);
        assert_eq!(dst, src);
    }

    #[test]
    fn underrun_returns_partial() {
        let ring = SampleRing::new(1, 4).unwrap();
        ring.push_frames(&[0.5]);
        let mut dst = [0.0; 2];
        assert_eq!(ring.pop_samples(&mut dst), 1);
        assert_eq!(dst[0], 0.5);
        assert_eq!(dst[1], 0.0);
    }
}
