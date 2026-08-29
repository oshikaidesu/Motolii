use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Instant;

/// Timeline/Stage両widgetが同じ時刻を読むための共有クロック。ループ再生。
pub struct Clock {
    playing: AtomicBool,
    anchor: Mutex<(Instant, f64)>,
    pub duration: f64,
}

impl Clock {
    pub fn new(duration: f64) -> Self {
        Self {
            playing: AtomicBool::new(false),
            anchor: Mutex::new((Instant::now(), 0.0)),
            duration,
        }
    }

    pub fn playing(&self) -> bool {
        self.playing.load(Ordering::Relaxed)
    }

    pub fn toggle(&self) {
        let mut anchor = self.anchor.lock().unwrap();
        let now = Instant::now();
        if self.playing.swap(false, Ordering::Relaxed) {
            let pos = (anchor.1 + now.duration_since(anchor.0).as_secs_f64()) % self.duration;
            *anchor = (now, pos);
        } else {
            anchor.0 = now;
            self.playing.store(true, Ordering::Relaxed);
        }
    }

    pub fn now_sec(&self) -> f64 {
        let anchor = self.anchor.lock().unwrap();
        if self.playing.load(Ordering::Relaxed) {
            (anchor.1 + anchor.0.elapsed().as_secs_f64()) % self.duration
        } else {
            anchor.1
        }
    }
}
