
use crate::render::audio::error::{AudioError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcmFormat {
    pub channels: u16,
    pub sample_rate: u32,
}

#[derive(Debug, Clone)]
pub struct PcmCache {
    samples: Vec<f32>,
    format: PcmFormat,
    frame_count: u64,
}

impl PcmCache {
    pub fn from_interleaved(samples: Vec<f32>, format: PcmFormat) -> Result<Self> {
        if format.channels == 0 {
            return Err(AudioError::UnsupportedChannels { channels: 0 });
        }
        if format.sample_rate == 0 {
            return Err(AudioError::UnsupportedSampleRate { sample_rate: 0 });
        }
        let channels = format.channels as usize;
        if !samples.len().is_multiple_of(channels) {
            return Err(AudioError::MisalignedSamples {
                len: samples.len(),
                channels: format.channels,
            });
        }
        let frame_count = (samples.len() / channels) as u64;
        Ok(Self {
            samples,
            format,
            frame_count,
        })
    }

    pub fn format(&self) -> PcmFormat {
        self.format
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn read_frames(&self, start_frame: u64, frame_count: usize) -> Result<&[f32]> {
        let end_frame = start_frame
            .checked_add(frame_count as u64)
            .filter(|end| *end <= self.frame_count)
            .ok_or(AudioError::OutOfRange {
                start: start_frame,
                requested: frame_count,
                total: self.frame_count,
            })?;
        let channels = self.format.channels as usize;
        let start = start_frame as usize * channels;
        let end = end_frame as usize * channels;
        Ok(&self.samples[start..end])
    }

    pub fn frame_at(&self, frame_index: u64) -> Result<&[f32]> {
        self.read_frames(frame_index, 1)
    }
}
