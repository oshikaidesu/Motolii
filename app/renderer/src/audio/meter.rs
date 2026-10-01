
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub const CLIP_THRESHOLD: f32 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeterSnapshot {
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

#[derive(Debug, Default)]
pub struct AudioMeter {
    peak_l_bits: AtomicU32,
    peak_r_bits: AtomicU32,
    clipped: AtomicBool,
}

#[derive(Debug, Default)]
pub struct ClipLatch {
    latched: AtomicBool,
}

impl ClipLatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn observe(&self, snapshot: MeterSnapshot) {
        if snapshot.clipped {
            self.latched.store(true, Ordering::Relaxed);
        }
    }

    pub fn reset(&self) {
        self.latched.store(false, Ordering::Relaxed);
    }

    pub fn is_latched(&self) -> bool {
        self.latched.load(Ordering::Relaxed)
    }
}

impl AudioMeter {
    pub fn new() -> Self {
        Self::default()
    }

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
