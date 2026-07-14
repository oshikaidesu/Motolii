//! AG-2: mix結果のchannel別sample peak / clip状態(Transient・lock-free)。
//!
//! Documentへ永続化しない。callback内allocation/I/O/lock待ちは行わない。

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// `abs(sample) > 1.0` をclipとする(oversampling true-peakは非目標)。
pub const CLIP_THRESHOLD: f32 = 1.0;

/// UI/診断向けの最新meter snapshot。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeterSnapshot {
    /// L/R の絶対値ピーク(直近観測窓、またはリセット後の累積 — `AudioMeter`の契約)。
    pub peak_l: f32,
    pub peak_r: f32,
    pub clipped: bool,
}

impl MeterSnapshot {
    pub const SILENT: Self = Self {
        peak_l: 0.0,
        peak_r: 0.0,
        clipped: false,
    };
}

/// mix中に更新し、別スレッドが`snapshot`で読むだけのmeter。
///
/// 原子操作のみ。Mutex無し。peak更新はCASで欠落を防ぐ。
#[derive(Debug, Default)]
pub struct AudioMeter {
    peak_l_bits: AtomicU32,
    peak_r_bits: AtomicU32,
    clipped: AtomicBool,
}

impl AudioMeter {
    pub fn new() -> Self {
        Self::default()
    }

    /// mix結果ブロックを観測する。PCM値は変更しない(呼び出し側バッファは読み取り専用)。
    pub fn observe_interleaved_stereo(&self, samples: &[f32]) {
        debug_assert!(samples.len().is_multiple_of(2));
        let mut peak_l = 0.0f32;
        let mut peak_r = 0.0f32;
        let mut clipped = false;
        for frame in samples.chunks_exact(2) {
            let l = frame[0].abs();
            let r = frame[1].abs();
            if l > peak_l {
                peak_l = l;
            }
            if r > peak_r {
                peak_r = r;
            }
            if l > CLIP_THRESHOLD || r > CLIP_THRESHOLD {
                clipped = true;
            }
        }
        fetch_max_f32(&self.peak_l_bits, peak_l);
        fetch_max_f32(&self.peak_r_bits, peak_r);
        if clipped {
            self.clipped.store(true, Ordering::Relaxed);
        }
    }

    pub fn snapshot(&self) -> MeterSnapshot {
        MeterSnapshot {
            peak_l: f32::from_bits(self.peak_l_bits.load(Ordering::Relaxed)),
            peak_r: f32::from_bits(self.peak_r_bits.load(Ordering::Relaxed)),
            clipped: self.clipped.load(Ordering::Relaxed),
        }
    }

    /// UIの手動リセット用。ラッチ式CLIPも含めて消す。
    pub fn reset(&self) {
        self.peak_l_bits.store(0.0f32.to_bits(), Ordering::Relaxed);
        self.peak_r_bits.store(0.0f32.to_bits(), Ordering::Relaxed);
        self.clipped.store(false, Ordering::Relaxed);
    }
}

fn fetch_max_f32(slot: &AtomicU32, value: f32) {
    if value <= 0.0 {
        return;
    }
    let mut cur = slot.load(Ordering::Relaxed);
    loop {
        let cur_f = f32::from_bits(cur);
        if value <= cur_f {
            return;
        }
        match slot.compare_exchange_weak(cur, value.to_bits(), Ordering::Relaxed, Ordering::Relaxed)
        {
            Ok(_) => return,
            Err(observed) => cur = observed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_does_not_clip() {
        let meter = AudioMeter::new();
        meter.observe_interleaved_stereo(&[0.0, 0.0, 0.0, 0.0]);
        let s = meter.snapshot();
        assert_eq!(s.peak_l, 0.0);
        assert_eq!(s.peak_r, 0.0);
        assert!(!s.clipped);
    }

    #[test]
    fn peak_and_clip_match_known_samples() {
        let meter = AudioMeter::new();
        meter.observe_interleaved_stereo(&[0.5, -0.25, 1.5, -2.0]);
        let s = meter.snapshot();
        assert_eq!(s.peak_l, 1.5);
        assert_eq!(s.peak_r, 2.0);
        assert!(s.clipped);
    }

    #[test]
    fn reset_clears_latched_clip() {
        let meter = AudioMeter::new();
        meter.observe_interleaved_stereo(&[2.0, 0.0]);
        assert!(meter.snapshot().clipped);
        meter.reset();
        assert_eq!(meter.snapshot(), MeterSnapshot::SILENT);
    }
}
