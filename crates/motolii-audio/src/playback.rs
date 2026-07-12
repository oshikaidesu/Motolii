use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};

use crate::cache::PcmCache;
use crate::error::AudioError;
use crate::ring::SampleRing;

/// 再生終了時の統計。アンダーランはコールバックが無音で埋めた回数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaybackStats {
    pub frames_delivered: u64,
    pub underrun_count: u64,
}

/// プロデューサスレッド + cpal出力の再生ハンドル。
pub struct PlaybackHandle {
    ring: Arc<SampleRing>,
    running: Arc<AtomicBool>,
    underruns: Arc<AtomicU64>,
    producer: Option<JoinHandle<()>>,
    stream: Option<Stream>,
}

impl PlaybackHandle {
    /// キャッシュ先頭から再生を開始する。
    pub fn play(cache: Arc<PcmCache>) -> Result<Self, AudioError> {
        Self::play_from(cache, 0)
    }

    /// 任意フレーム位置から再生を開始する。
    pub fn play_from(cache: Arc<PcmCache>, start_frame: u64) -> Result<Self, AudioError> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or_else(|| AudioError::NoOutputDevice {
            detail: "no default output device".into(),
        })?;
        Self::play_from_on_device(cache, &device, start_frame)
    }

    pub fn play_from_on_device(
        cache: Arc<PcmCache>,
        device: &cpal::Device,
        start_frame: u64,
    ) -> Result<Self, AudioError> {
        let format = cache.format();
        let (config, output_channels) =
            pick_output_config(device, format.sample_rate, format.channels)?;
        let input_channels = format.channels as usize;

        // 約200ms分を先読み — コールバックが飢えない余裕(D4完了条件)。
        let capacity_frames = (format.sample_rate as usize / 5).max(1024);
        let ring = Arc::new(SampleRing::new(output_channels, capacity_frames)?);
        let running = Arc::new(AtomicBool::new(true));
        let underruns = Arc::new(AtomicU64::new(0));

        let ring_cb = Arc::clone(&ring);
        let underruns_cb = Arc::clone(&underruns);
        let stream = match config.sample_format() {
            SampleFormat::F32 => device
                .build_output_stream(
                    &config.config(),
                    move |data: &mut [f32], _| {
                        let got = ring_cb.pop_samples(data);
                        if got < data.len() {
                            data[got..].fill(0.0);
                            underruns_cb.fetch_add(1, Ordering::Relaxed);
                        }
                    },
                    move |err| {
                        eprintln!("cpal stream error: {err}");
                    },
                    None,
                )
                .map_err(|e| AudioError::Cpal(e.to_string()))?,
            other => {
                return Err(AudioError::UnsupportedOutputConfig {
                    sample_rate: format.sample_rate,
                    channels: format.channels,
                    detail: format!("sample format {other:?} is not supported"),
                });
            }
        };

        let ring_prod = Arc::clone(&ring);
        let running_prod = Arc::clone(&running);
        let total_frames = cache.frame_count();
        let mut playhead = start_frame.min(total_frames);

        // play()前に先読み+プロデューサ起動 — コールバックがwriter不在のリングを枯らさない
        let prefill_target = capacity_frames.min((total_frames.saturating_sub(playhead)) as usize);
        while ring.buffered_frames() < prefill_target {
            let free = ring.free_frames();
            if free == 0 || playhead >= total_frames {
                break;
            }
            let chunk_frames = free.min((total_frames - playhead) as usize);
            let Ok(chunk) = cache.read_frames(playhead, chunk_frames) else {
                break;
            };
            let upmixed = upmix_to_output(chunk, input_channels, output_channels as usize);
            let pushed = ring.push_frames(&upmixed);
            if pushed == 0 {
                break;
            }
            playhead += pushed as u64;
        }

        let producer = thread::Builder::new()
            .name("motolii-audio-producer".into())
            .spawn(move || {
                while running_prod.load(Ordering::Acquire) {
                    if playhead >= total_frames {
                        break;
                    }
                    let free = ring_prod.free_frames();
                    if free == 0 {
                        thread::sleep(Duration::from_millis(1));
                        continue;
                    }
                    let chunk_frames = free.min((total_frames - playhead) as usize);
                    let Ok(chunk) = cache.read_frames(playhead, chunk_frames) else {
                        break;
                    };
                    let upmixed = upmix_to_output(chunk, input_channels, output_channels as usize);
                    let pushed = ring_prod.push_frames(&upmixed);
                    if pushed == 0 {
                        thread::sleep(Duration::from_millis(1));
                        continue;
                    }
                    playhead += pushed as u64;
                }
            })
            .map_err(|e| AudioError::Cpal(format!("failed to spawn producer thread: {e}")))?;

        stream
            .play()
            .map_err(|e| AudioError::Cpal(e.to_string()))?;

        Ok(Self {
            ring,
            running,
            underruns,
            producer: Some(producer),
            stream: Some(stream),
        })
    }

    pub fn stats(&self) -> PlaybackStats {
        PlaybackStats {
            frames_delivered: self.ring.frames_read(),
            underrun_count: self.underruns.load(Ordering::Acquire),
        }
    }

    pub fn stop(mut self) -> PlaybackStats {
        self.shutdown();
        self.stats()
    }

    fn shutdown(&mut self) {
        self.running.store(false, Ordering::Release);
        if let Some(producer) = self.producer.take() {
            let _ = producer.join();
        }
        // Stream dropがデバイス停止。プロデューサjoin後に落とす
        drop(self.stream.take());
    }
}

impl Drop for PlaybackHandle {
    fn drop(&mut self) {
        // stop()を経由しない破棄でもゾンビプロデューサを残さない
        self.shutdown();
    }
}

fn pick_output_config(
    device: &cpal::Device,
    sample_rate: u32,
    channels: u16,
) -> Result<(cpal::SupportedStreamConfig, u16), AudioError> {
    if let Ok(cfg) = pick_exact(device, sample_rate, channels) {
        return Ok((cfg, channels));
    }
    // macOS等はmono f32出力を持たないことが多い — 同レートのstereoへフォールバック(複製のみ、ミキサーではない)
    if channels == 1 {
        if let Ok(cfg) = pick_exact(device, sample_rate, 2) {
            return Ok((cfg, 2));
        }
    }

    Err(AudioError::UnsupportedOutputConfig {
        sample_rate,
        channels,
        detail: "no f32 output config for requested or stereo fallback".into(),
    })
}

fn pick_exact(
    device: &cpal::Device,
    sample_rate: u32,
    channels: u16,
) -> Result<cpal::SupportedStreamConfig, AudioError> {
    let configs: Vec<_> = device
        .supported_output_configs()
        .map_err(|e| AudioError::Cpal(e.to_string()))?
        .filter(|c| c.channels() == channels && c.sample_format() == SampleFormat::F32)
        .collect();

    let Some(exact) = configs.iter().find(|c| {
        c.min_sample_rate().0 <= sample_rate && c.max_sample_rate().0 >= sample_rate
    }) else {
        return Err(AudioError::UnsupportedOutputConfig {
            sample_rate,
            channels,
            detail: if configs.is_empty() {
                "no f32 output configs for this channel count".into()
            } else {
                format!(
                    "no config spans {sample_rate} Hz (have {} candidates)",
                    configs.len()
                )
            },
        });
    };

    Ok(exact.with_sample_rate(cpal::SampleRate(sample_rate)))
}

fn upmix_to_output(src: &[f32], input_channels: usize, output_channels: usize) -> Vec<f32> {
    if input_channels == output_channels {
        return src.to_vec();
    }
    if input_channels == 1 && output_channels == 2 {
        let mut out = Vec::with_capacity(src.len() * 2);
        for &sample in src {
            out.push(sample);
            out.push(sample);
        }
        return out;
    }
    Vec::new()
}

/// ハードウェア無しでもアンダーラン検証できるシミュレーション。
///
/// 固定レートのコンシューマがリングを枯らす速度で、プロデューサが追いつけることを確認する。
pub fn simulate_playback_without_underrun(
    cache: &PcmCache,
    start_frame: u64,
    consumer_interval: Duration,
    frames_per_tick: usize,
) -> Result<PlaybackStats, AudioError> {
    let format = cache.format();
    let capacity = (format.sample_rate as usize / 5).max(1024);
    let ring = Arc::new(SampleRing::new(format.channels, capacity)?);
    let running = Arc::new(AtomicBool::new(true));
    let underruns = Arc::new(AtomicU64::new(0));
    let total = cache.frame_count();
    let mut playhead = start_frame.min(total);

    let ring_prod = Arc::clone(&ring);
    let running_prod = Arc::clone(&running);
    let cache = cache.clone();
    let producer = thread::spawn(move || {
        while running_prod.load(Ordering::Acquire) {
            if playhead >= total {
                break;
            }
            let free = ring_prod.free_frames();
            if free == 0 {
                thread::sleep(Duration::from_micros(200));
                continue;
            }
            let chunk = free.min((total - playhead) as usize);
            if let Ok(data) = cache.read_frames(playhead, chunk) {
                let pushed = ring_prod.push_frames(data);
                playhead += pushed as u64;
            } else {
                break;
            }
        }
    });

    let ch = format.channels as usize;
    let target = total.saturating_sub(start_frame);

    // 起動直後の競合アンダーランを避けるため、先読みが溜まるまで短く待つ
    let prefill = (format.sample_rate as usize / 20).max(256);
    while ring.buffered_frames() < prefill.min(target as usize) {
        thread::sleep(Duration::from_micros(200));
    }

    while ring.frames_read() < target {
        let remaining_frames = (target - ring.frames_read()) as usize;
        let tick_frames = frames_per_tick.min(remaining_frames);
        let mut buf = vec![0.0f32; tick_frames * ch];
        let got = ring.pop_samples(&mut buf);
        if got < buf.len() {
            underruns.fetch_add(1, Ordering::Relaxed);
            buf[got..].fill(0.0);
        }
        thread::sleep(consumer_interval);
    }

    running.store(false, Ordering::Release);
    let _ = producer.join();

    Ok(PlaybackStats {
        frames_delivered: ring.frames_read(),
        underrun_count: underruns.load(Ordering::Acquire),
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use crate::cache::{PcmCache, PcmFormat};

    use super::simulate_playback_without_underrun;

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
            .unwrap(),
        )
    }

    #[test]
    fn simulated_playback_has_no_underruns() {
        let cache = sine_cache(48_000, 48_000);
        let rate = cache.format().sample_rate;
        let frames_per_tick = 256;
        let interval = Duration::from_secs_f64(frames_per_tick as f64 / rate as f64);
        let stats =
            simulate_playback_without_underrun(&cache, 0, interval, frames_per_tick).unwrap();
        assert_eq!(stats.underrun_count, 0);
        assert_eq!(stats.frames_delivered, cache.frame_count());
    }
}
