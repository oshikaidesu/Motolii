
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

#[cfg(test)]
mod tests {
    use super::*;

    fn stereo(frames: usize) -> PcmCache {
        let mut samples = Vec::with_capacity(frames * 2);
        for i in 0..frames {
            samples.push(i as f32);
            samples.push(-(i as f32));
        }
        PcmCache::from_interleaved(
            samples,
            PcmFormat {
                channels: 2,
                sample_rate: 48_000,
            },
        )
        .expect("valid cache")
    }

    #[test]
    fn rejects_zero_channels() {
        let err = PcmCache::from_interleaved(
            vec![0.0],
            PcmFormat {
                channels: 0,
                sample_rate: 48_000,
            },
        )
        .unwrap_err();
        assert!(matches!(
            err,
            AudioError::UnsupportedChannels { channels: 0 }
        ));
    }

    #[test]
    fn rejects_zero_sample_rate() {
        let err = PcmCache::from_interleaved(
            vec![0.0],
            PcmFormat {
                channels: 1,
                sample_rate: 0,
            },
        )
        .unwrap_err();
        assert!(matches!(
            err,
            AudioError::UnsupportedSampleRate { sample_rate: 0 }
        ));
    }

    #[test]
    fn rejects_misaligned_buffer() {
        let err = PcmCache::from_interleaved(
            vec![0.0, 1.0, 2.0],
            PcmFormat {
                channels: 2,
                sample_rate: 48_000,
            },
        )
        .unwrap_err();
        assert!(matches!(
            err,
            AudioError::MisalignedSamples {
                len: 3,
                channels: 2
            }
        ));
    }

    #[test]
    fn frame_at_arbitrary_positions_matches_direct_index() {
        let cache = stereo(1_000);
        for idx in [0u64, 1, 499, 500, 999] {
            let frame = cache.frame_at(idx).expect("in-range frame");
            assert_eq!(frame, [idx as f32, -(idx as f32)]);
        }
    }

    #[test]
    fn read_frames_returns_contiguous_slice() {
        let cache = stereo(10);
        let chunk = cache.read_frames(3, 4).expect("in-range chunk");
        assert_eq!(chunk, [3.0, -3.0, 4.0, -4.0, 5.0, -5.0, 6.0, -6.0]);
    }

    #[test]
    fn read_frames_out_of_range_is_typed_error_not_panic() {
        let cache = stereo(10);
        let err = cache.read_frames(8, 5).unwrap_err();
        match err {
            AudioError::OutOfRange {
                start: 8,
                requested: 5,
                total: 10,
            } => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn read_frames_at_exact_end_is_ok_when_empty() {
        let cache = stereo(10);
        assert_eq!(
            cache.read_frames(10, 0).expect("empty tail read"),
            &[] as &[f32]
        );
    }

    #[test]
    fn frame_at_end_is_out_of_range() {
        let cache = stereo(10);
        assert!(cache.frame_at(10).is_err());
    }
}
