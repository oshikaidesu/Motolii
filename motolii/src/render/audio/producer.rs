
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::render::audio::convert::{time_to_canonical_frames, CANONICAL_CHANNELS, CANONICAL_SAMPLE_RATE};
use crate::render::audio::error::{AudioError, Result};
use crate::render::audio::program::AudioProgram;
use crate::render::audio::resample::FixedRatioResampler;

const POLL_INTERVAL: Duration = Duration::from_millis(1);

const MIX_CHUNK_FRAMES: usize = 1_024;

const MAX_FLUSH_CHUNKS: usize = 8;

const NO_SEEK: u64 = u64::MAX;

pub struct MixProducer {
    running: Arc<AtomicBool>,
    seek_to: Arc<AtomicU64>,
    finished: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl MixProducer {
    pub fn spawn(
        program: Arc<AudioProgram>,
        producer: rtrb::Producer<f32>,
        start_frame: u64,
        device_sample_rate: u32,
    ) -> Result<Self> {
        if device_sample_rate == 0 {
            return Err(AudioError::UnsupportedSampleRate {
                sample_rate: device_sample_rate,
            });
        }

        let running = Arc::new(AtomicBool::new(true));
        let seek_to = Arc::new(AtomicU64::new(NO_SEEK));
        let finished = Arc::new(AtomicBool::new(false));
        let running_thread = Arc::clone(&running);
        let seek_thread = Arc::clone(&seek_to);
        let finished_thread = Arc::clone(&finished);

        let handle = thread::Builder::new()
            .name("motolii-audio-mix-producer".into())
            .spawn(move || {
                producer_loop(
                    &program,
                    producer,
                    start_frame,
                    device_sample_rate,
                    &running_thread,
                    &seek_thread,
                    &finished_thread,
                );
            })
            .map_err(AudioError::ProducerSpawn)?;

        Ok(Self {
            running,
            seek_to,
            finished,
            handle: Some(handle),
        })
    }

    pub fn seek(&self, frame: u64) {
        self.seek_to.store(frame.min(NO_SEEK - 1), Ordering::Release);
    }

    pub fn finished(&self) -> bool {
        self.finished.load(Ordering::Acquire)
    }

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

impl Drop for MixProducer {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn push_frames(producer: &mut rtrb::Producer<f32>, samples: &[f32], channels: usize) -> usize {
    if channels == 0 || samples.is_empty() {
        return 0;
    }
    let free_frames = producer.slots() / channels;
    let want_frames = samples.len() / channels;
    let frames = free_frames.min(want_frames);
    for &sample in &samples[..frames * channels] {
        let _ = producer.push(sample);
    }
    frames
}

fn program_end_frame(program: &AudioProgram) -> u64 {
    let mut end = time_to_canonical_frames(program.composition_duration());
    for source in program.sources() {
        let start = time_to_canonical_frames(source.timeline_start);
        let dur = time_to_canonical_frames(source.timeline_duration);
        end = end.max(start.saturating_add(dur));
    }
    end
}

#[allow(clippy::too_many_arguments)]
fn producer_loop(
    program: &AudioProgram,
    mut producer: rtrb::Producer<f32>,
    start_frame: u64,
    device_sample_rate: u32,
    running: &AtomicBool,
    seek_to: &AtomicU64,
    finished: &AtomicBool,
) {
    let channels = CANONICAL_CHANNELS as usize;
    let resampling = device_sample_rate != CANONICAL_SAMPLE_RATE;
    let mut resampler = if resampling {
        match FixedRatioResampler::new(CANONICAL_SAMPLE_RATE, device_sample_rate, CANONICAL_CHANNELS) {
            Ok(mut resampler) => {
                resampler.reset();
                Some(resampler)
            }
            Err(_) => None,
        }
    } else {
        None
    };

    let end_frame = program_end_frame(program);
    let mut playhead = start_frame.min(end_frame);
    let mut pending: Vec<f32> = Vec::new();
    let mut pending_off = 0usize;
    let mut flushing = playhead >= end_frame;
    let mut flush_chunks = 0usize;

    while running.load(Ordering::Acquire) {
        let seek = seek_to.swap(NO_SEEK, Ordering::AcqRel);
        if seek != NO_SEEK {
            playhead = seek.min(end_frame);
            pending.clear();
            pending_off = 0;
            flush_chunks = 0;
            flushing = playhead >= end_frame;
            finished.store(false, Ordering::Release);
            if let Some(resampler) = resampler.as_mut() {
                resampler.reset();
            }
        }

        if pending_off < pending.len() {
            let frames_left = (pending.len() - pending_off) / channels;
            if frames_left == 0 {
                pending.clear();
                pending_off = 0;
                continue;
            }
            let end = pending_off + frames_left * channels;
            let pushed = push_frames(&mut producer, &pending[pending_off..end], channels);
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
            let Some(resampler) = resampler.as_mut() else {
                finished.store(true, Ordering::Release);
                break;
            };
            if flush_chunks >= MAX_FLUSH_CHUNKS {
                finished.store(true, Ordering::Release);
                break;
            }
            let Ok(out) = resampler.flush_silence_chunk() else {
                finished.store(true, Ordering::Release);
                break;
            };
            flush_chunks += 1;
            if out.is_empty() {
                finished.store(true, Ordering::Release);
                break;
            }
            pending.extend_from_slice(out);
            continue;
        }

        if playhead >= end_frame {
            if resampler.is_some() {
                flushing = true;
                continue;
            }
            finished.store(true, Ordering::Release);
            break;
        }

        let need = resampler
            .as_ref()
            .map_or(MIX_CHUNK_FRAMES, FixedRatioResampler::input_frames_next)
            .min((end_frame - playhead) as usize);
        if need == 0 {
            flushing = true;
            continue;
        }
        let Ok((pcm, _report)) = program.mix_audio(playhead, need, None) else {
            finished.store(true, Ordering::Release);
            break;
        };
        playhead += need as u64;

        match resampler.as_mut() {
            Some(resampler) => match resampler.process_interleaved(&pcm) {
                Ok(out) => pending.extend_from_slice(out),
                Err(_) => {
                    finished.store(true, Ordering::Release);
                    break;
                }
            },
            None => pending.extend_from_slice(&pcm),
        }
        if playhead >= end_frame {
            flushing = true;
        }
    }
}
