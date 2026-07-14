//! プロデューサスレッド(D4契約: デコード済みPCMをキャッシュから読み、リングへ供給する。
//! 音声コールバックは絶対にこのスレッドをブロックしない — 両者はリング経由でのみ結合する)。
//!
//! **D4-FU**: デバイスレート≠素材レートのときだけ固定比リサンプルをリング書き込み前に挿入する。
//! レート一致時はリサンプラを作らない(恒等パス)。アルゴリズム遅延はリサンプラ内の先頭 trim で吸収する。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::cache::PcmCache;
use crate::error::{AudioError, Result};
use crate::resample::FixedRatioResampler;
use crate::ring::RingProducer;

/// リングが満杯、または供給側が追いつけない時の再試行間隔。
const POLL_INTERVAL: Duration = Duration::from_millis(1);

/// 終端フラッシュで無音チャンクを流す上限(遅延吐き出し用)。
const MAX_FLUSH_CHUNKS: usize = 8;

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
    /// リング書き込み前に固定比リサンプラを通すか(D4-FU完了条件: レート一致で非挿入)。
    resampling: bool,
}

impl AudioProducer {
    /// `start_frame`からキャッシュを読み、素材レートのまま`ring`へ供給する(恒等パス)。
    pub fn spawn(cache: Arc<PcmCache>, ring: RingProducer, start_frame: u64) -> Result<Self> {
        let rate = cache.format().sample_rate;
        Self::spawn_with_device_rate(cache, ring, start_frame, rate)
    }

    /// デバイス出力レート向けに供給する。レート不一致時のみリサンプラを挿入する。
    pub fn spawn_with_device_rate(
        cache: Arc<PcmCache>,
        ring: RingProducer,
        start_frame: u64,
        device_sample_rate: u32,
    ) -> Result<Self> {
        if device_sample_rate == 0 {
            return Err(AudioError::UnsupportedSampleRate {
                sample_rate: device_sample_rate,
            });
        }
        if ring.channels() != cache.format().channels as usize {
            return Err(AudioError::Resample {
                detail: "ring channel count must match PCM cache",
            });
        }

        let source_rate = cache.format().sample_rate;
        let resampling = source_rate != device_sample_rate;
        let running = Arc::new(AtomicBool::new(true));
        let running_thread = Arc::clone(&running);

        let handle = if resampling {
            let mut resampler = FixedRatioResampler::new(
                source_rate,
                device_sample_rate,
                cache.format().channels,
            )?;
            // 開始位置からの供給に合わせ、遅延 trim を初期状態にする。
            resampler.reset();
            thread::Builder::new()
                .name("motolii-audio-producer".into())
                .spawn(move || {
                    producer_loop_resample(
                        &cache,
                        &ring,
                        start_frame,
                        &mut resampler,
                        &running_thread,
                    )
                })
                .map_err(AudioError::ProducerSpawn)?
        } else {
            thread::Builder::new()
                .name("motolii-audio-producer".into())
                .spawn(move || producer_loop_identity(&cache, &ring, start_frame, &running_thread))
                .map_err(AudioError::ProducerSpawn)?
        };

        Ok(Self {
            running,
            handle: Some(handle),
            resampling,
        })
    }

    /// 固定比リサンプラを挿入しているか。
    pub fn is_resampling(&self) -> bool {
        self.resampling
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

fn producer_loop_identity(
    cache: &PcmCache,
    ring: &RingProducer,
    start_frame: u64,
    running: &AtomicBool,
) {
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

fn producer_loop_resample(
    cache: &PcmCache,
    ring: &RingProducer,
    start_frame: u64,
    resampler: &mut FixedRatioResampler,
    running: &AtomicBool,
) {
    let total = cache.frame_count();
    let mut playhead = start_frame.min(total);
    let mut pending: Vec<f32> = Vec::new();
    let mut pending_off = 0usize; // サンプル単位
    let mut flushing = playhead >= total;
    let mut flush_chunks = 0usize;

    while running.load(Ordering::Acquire) {
        // リングへ未送出分を先に出す(部分pushに耐える)。
        if pending_off < pending.len() {
            let channels = resampler.channels();
            let frames_left = (pending.len() - pending_off) / channels;
            if frames_left == 0 {
                pending.clear();
                pending_off = 0;
                continue;
            }
            let pushed = ring.push_frames(&pending[pending_off..]);
            if pushed == 0 {
                thread::sleep(POLL_INTERVAL);
                continue;
            }
            pending_off += pushed * channels;
            if pending_off >= pending.len() {
                pending.clear();
                pending_off = 0;
            }
            continue;
        }

        if flushing {
            if flush_chunks >= MAX_FLUSH_CHUNKS {
                break;
            }
            let Ok(out) = resampler.flush_silence_chunk() else {
                break;
            };
            flush_chunks += 1;
            if out.is_empty() {
                // trim 中の無音だけならもう一度。出力がずっと空なら打ち切る。
                if flush_chunks >= MAX_FLUSH_CHUNKS {
                    break;
                }
                continue;
            }
            pending.extend_from_slice(out);
            continue;
        }

        if playhead >= total {
            flushing = true;
            continue;
        }

        let need = resampler.input_frames_next();
        let remaining = (total - playhead) as usize;
        if remaining >= need {
            let Ok(chunk) = cache.read_frames(playhead, need) else {
                break;
            };
            playhead += need as u64;
            let Ok(out) = resampler.process_interleaved(chunk) else {
                break;
            };
            if !out.is_empty() {
                pending.extend_from_slice(out);
            }
        } else {
            let Ok(chunk) = cache.read_frames(playhead, remaining) else {
                break;
            };
            playhead = total;
            let Ok(out) = resampler.process_partial_interleaved(chunk) else {
                break;
            };
            if !out.is_empty() {
                pending.extend_from_slice(out);
            }
            flushing = true;
        }
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
        assert!(!producer.is_resampling());

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

    #[test]
    fn matching_device_rate_does_not_insert_resampler() {
        let cache = sine_cache(512, 48_000);
        let (ring_prod, _ring_cons) = ring::channel(1, 1_024).expect("ring channel");
        let producer =
            AudioProducer::spawn_with_device_rate(Arc::clone(&cache), ring_prod, 0, 48_000)
                .expect("spawn");
        assert!(!producer.is_resampling());
        producer.stop();
    }

    #[test]
    fn mismatched_device_rate_inserts_resampler() {
        let cache = sine_cache(512, 44_100);
        let (ring_prod, _ring_cons) = ring::channel(1, 2_048).expect("ring channel");
        let producer =
            AudioProducer::spawn_with_device_rate(Arc::clone(&cache), ring_prod, 0, 48_000)
                .expect("spawn");
        assert!(producer.is_resampling());
        producer.stop();
    }
}
