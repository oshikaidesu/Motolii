
use rubato::{FftFixedIn, Resampler};

use crate::render::audio::error::{AudioError, Result};

const CHUNK_FRAMES: usize = 1024;

pub struct FixedRatioResampler {
    inner: FftFixedIn<f32>,
    channels: usize,
    frames_to_trim: usize,
    planar_in: Vec<Vec<f32>>,
    planar_out: Vec<Vec<f32>>,
    interleaved: Vec<f32>,
}

impl FixedRatioResampler {
    pub fn new(source_rate: u32, device_rate: u32, channels: u16) -> Result<Self> {
        if channels == 0 {
            return Err(AudioError::UnsupportedChannels { channels: 0 });
        }
        if source_rate == 0 {
            return Err(AudioError::UnsupportedSampleRate {
                sample_rate: source_rate,
            });
        }
        if device_rate == 0 {
            return Err(AudioError::UnsupportedSampleRate {
                sample_rate: device_rate,
            });
        }
        if source_rate == device_rate {
            return Err(AudioError::Resample {
                detail: "fixed-ratio resampler must not be built when rates already match",
            });
        }
        let ch = channels as usize;
        let inner = FftFixedIn::<f32>::new(
            source_rate as usize,
            device_rate as usize,
            CHUNK_FRAMES,
            2,
            ch,
        )
        .map_err(|_| AudioError::Resample {
            detail: "failed to construct fixed-ratio FFT resampler",
        })?;

        let delay = inner.output_delay();
        let planar_in = inner.input_buffer_allocate(true);
        let planar_out = inner.output_buffer_allocate(true);
        Ok(Self {
            inner,
            channels: ch,
            frames_to_trim: delay,
            planar_in,
            planar_out,
            interleaved: Vec::new(),
        })
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    pub fn input_frames_next(&self) -> usize {
        self.inner.input_frames_next()
    }

    pub fn output_delay(&self) -> usize {
        self.inner.output_delay()
    }

    pub fn reset(&mut self) {
        self.inner.reset();
        self.frames_to_trim = self.inner.output_delay();
        self.interleaved.clear();
    }

    pub fn process_interleaved(&mut self, input: &[f32]) -> Result<&[f32]> {
        let need = self.inner.input_frames_next();
        let frames = input.len() / self.channels;
        if !input.len().is_multiple_of(self.channels) || frames != need {
            return Err(AudioError::Resample {
                detail: "input frame count must equal input_frames_next()",
            });
        }
        self.deinterleave_into(input, frames);
        self.inner
            .process_into_buffer(&self.planar_in, &mut self.planar_out, None)
            .map_err(|_| AudioError::Resample {
                detail: "rubato process_into_buffer failed",
            })?;
        self.interleave_trimmed()
    }

    pub fn process_partial_interleaved(&mut self, input: &[f32]) -> Result<&[f32]> {
        if !input.len().is_multiple_of(self.channels) {
            return Err(AudioError::MisalignedSamples {
                len: input.len(),
                channels: self.channels as u16,
            });
        }
        let frames = input.len() / self.channels;
        self.deinterleave_into(input, frames);
        let out = self
            .inner
            .process_partial(Some(&self.planar_in), None)
            .map_err(|_| AudioError::Resample {
                detail: "rubato process_partial failed",
            })?;
        self.planar_out = out;
        self.interleave_trimmed()
    }

    pub fn flush_silence_chunk(&mut self) -> Result<&[f32]> {
        let empty: Option<&[Vec<f32>]> = None;
        let out = self
            .inner
            .process_partial(empty, None)
            .map_err(|_| AudioError::Resample {
                detail: "rubato flush process_partial failed",
            })?;
        self.planar_out = out;
        self.interleave_trimmed()
    }

    fn deinterleave_into(&mut self, input: &[f32], frames: usize) {
        for ch in 0..self.channels {
            self.planar_in[ch].resize(frames, 0.0);
            for f in 0..frames {
                self.planar_in[ch][f] = input[f * self.channels + ch];
            }
        }
    }

    fn interleave_trimmed(&mut self) -> Result<&[f32]> {
        let out_frames = self.planar_out.first().map(|c| c.len()).unwrap_or(0);
        for ch in &self.planar_out {
            if ch.len() != out_frames {
                return Err(AudioError::Resample {
                    detail: "planar output channel lengths diverged",
                });
            }
        }

        let skip = self.frames_to_trim.min(out_frames);
        self.frames_to_trim -= skip;
        let keep = out_frames - skip;
        self.interleaved.resize(keep * self.channels, 0.0);
        for f in 0..keep {
            let src_f = f + skip;
            for ch in 0..self.channels {
                self.interleaved[f * self.channels + ch] = self.planar_out[ch][src_f];
            }
        }
        Ok(&self.interleaved)
    }
}

pub fn source_frame_to_device(source_frame: u64, source_rate: u32, device_rate: u32) -> u64 {
    if source_rate == 0 {
        return 0;
    }
    (source_frame as u128)
        .saturating_mul(device_rate as u128)
        .checked_div(source_rate as u128)
        .unwrap_or(0) as u64
}
