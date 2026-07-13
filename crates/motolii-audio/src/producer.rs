//! プロデューサスレッド(D4契約: デコード済みPCMをキャッシュから読み、リングへ供給する。
//! 音声コールバックは絶対にこのスレッドをブロックしない — 両者はリング経由でのみ結合する)。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::cache::PcmCache;
use crate::error::{AudioError, Result};
use crate::ring::RingProducer;

/// リングが満杯、または供給側が追いつけない時の再試行間隔。
const POLL_INTERVAL: Duration = Duration::from_millis(1);

/// `PcmCache`からリングへ供給し続けるバックグラウンドスレッドの所有ハンドル。
///
/// **D4のスコープ境界**: ここは「どのフレームから読み始めるか」しか知らない。
/// 曲の終端に達したら供給を止めて終了する。ループ再生・クロック所有・映像との
/// 同期は[D5](../../../docs/specs/M2-document-model.md)のTransportの責務であり、
/// このスレッドはそれに関与しない(旧PR#90が閉じられた理由の再発防止: device出力と
/// クロック所有を混ぜない)。
pub struct AudioProducer {
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl AudioProducer {
    /// `start_frame`からキャッシュを読み`ring`へ供給するスレッドを起動する。
    pub fn spawn(cache: Arc<PcmCache>, ring: RingProducer, start_frame: u64) -> Result<Self> {
        let running = Arc::new(AtomicBool::new(true));
        let running_thread = Arc::clone(&running);
        let handle = thread::Builder::new()
            .name("motolii-audio-producer".into())
            .spawn(move || producer_loop(&cache, &ring, start_frame, &running_thread))
            .map_err(AudioError::ProducerSpawn)?;
        Ok(Self {
            running,
            handle: Some(handle),
        })
    }

    /// 供給を止めてスレッドをjoinする(`?`早期returnで飛ばされないよう、
    /// Dropからも同じ経路を呼ぶ — AGENTS.md「後始末を飛ばさない」)。
    pub fn stop(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        self.running.store(false, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for AudioProducer {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn producer_loop(cache: &PcmCache, ring: &RingProducer, start_frame: u64, running: &AtomicBool) {
    let total = cache.frame_count();
    let mut playhead = start_frame.min(total);
    while running.load(Ordering::Acquire) {
        if playhead >= total {
            break;
        }
        let free = ring.free_frames();
        if free == 0 {
            thread::sleep(POLL_INTERVAL);
            continue;
        }
        let chunk_frames = free.min((total - playhead) as usize);
        let Ok(chunk) = cache.read_frames(playhead, chunk_frames) else {
            // playhead < total かつ chunk_frames <= total - playhead なので理論上到達しない。
            // 防御的に停止する(境界検査を信用しすぎない)。
            break;
        };
        let pushed = ring.push_frames(chunk);
        if pushed == 0 {
            thread::sleep(POLL_INTERVAL);
            continue;
        }
        playhead += pushed as u64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::PcmFormat;
    use crate::ring;

    fn sine_cache(frames: usize, rate: u32) -> Arc<PcmCache> {
        let mut samples = Vec::with_capacity(frames);
        for i in 0..frames {
            let t = i as f32 / rate as f32;
            samples.push((t * 440.0 * std::f32::consts::TAU).sin());
        }
        Arc::new(
            PcmCache::from_interleaved(
                samples,
                PcmFormat {
                    channels: 1,
                    sample_rate: rate,
                },
            )
            .expect("valid cache"),
        )
    }

    #[test]
    fn producer_thread_fills_ring_and_stops_at_track_end() {
        let cache = sine_cache(2_000, 48_000);
        let (ring_prod, ring_cons) = ring::channel(1, 4_096).expect("ring channel");
        let producer =
            AudioProducer::spawn(Arc::clone(&cache), ring_prod, 0).expect("spawn producer");

        // リング容量(4096) > 総フレーム数(2000)なので、供給完了までポーリングで待てる。
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while ring_cons.buffered_frames() < cache.frame_count() as usize {
            assert!(
                std::time::Instant::now() < deadline,
                "producer did not fill ring in time"
            );
            thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(ring_cons.buffered_frames(), cache.frame_count() as usize);
        producer.stop();
    }

    #[test]
    fn producer_starts_from_arbitrary_offset() {
        let cache = sine_cache(1_000, 48_000);
        let start = 300u64;
        let (ring_prod, ring_cons) = ring::channel(1, 4_096).expect("ring channel");
        let producer =
            AudioProducer::spawn(Arc::clone(&cache), ring_prod, start).expect("spawn producer");

        let expected = (cache.frame_count() - start) as usize;
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while ring_cons.buffered_frames() < expected {
            assert!(
                std::time::Instant::now() < deadline,
                "producer did not fill ring in time"
            );
            thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(ring_cons.buffered_frames(), expected);
        producer.stop();
    }
}
