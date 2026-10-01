
use crate::render::audio::error::{AudioError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcmFormat {
    pub channels: u16,
    pub sample_rate: u32,
}

/// 復号済みの音。16bit 整数で持つ(f32 の半分)。読む時に f32 へ。
#[derive(Debug, Clone)]
pub struct PcmCache {
    samples: Vec<i16>,
    format: PcmFormat,
    frame_count: u64,
}

const I16_SCALE: f32 = 32768.0;

#[inline]
pub fn i16_to_f32(sample: i16) -> f32 {
    sample as f32 / I16_SCALE
}

#[inline]
fn f32_to_i16(sample: f32) -> i16 {
    if sample.is_finite() {
        (sample.clamp(-1.0, 1.0) * I16_SCALE).round().clamp(i16::MIN as f32, i16::MAX as f32) as i16
    } else {
        0
    }
}

impl PcmCache {
    pub fn from_interleaved(samples: Vec<f32>, format: PcmFormat) -> Result<Self> {
        Self::from_interleaved_i16(samples.into_iter().map(f32_to_i16).collect(), format)
    }

    pub fn from_interleaved_i16(samples: Vec<i16>, format: PcmFormat) -> Result<Self> {
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

    /// 全サンプル(interleaved、i16)。波形や peak のように全体を 1 回なめる側の口。
    pub fn samples_i16(&self) -> &[i16] {
        &self.samples
    }

    pub fn read_frames(&self, start_frame: u64, frame_count: usize) -> Result<&[i16]> {
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

    /// stereo の 1 フレームを f32 で。混合の内側で毎サンプル呼ぶ。
    pub fn stereo_at(&self, frame_index: u64) -> Result<(f32, f32)> {
        if self.format.channels != 2 {
            return Err(AudioError::UnsupportedChannels { channels: self.format.channels });
        }
        let frame = self.read_frames(frame_index, 1)?;
        Ok((i16_to_f32(frame[0]), i16_to_f32(frame[1])))
    }
}
