//! AG-2: 内部mixの正準形式(48,000 Hz / stereo / interleaved f32)への変換。

use crate::cache::{PcmCache, PcmFormat};
use crate::error::{AudioError, Result};
use crate::resample::FixedRatioResampler;

/// 内部mixの正準サンプルレート。
pub const CANONICAL_SAMPLE_RATE: u32 = 48_000;
/// 内部mixの正準チャンネル数(stereo)。
pub const CANONICAL_CHANNELS: u16 = 2;

pub fn canonical_format() -> PcmFormat {
    PcmFormat {
        channels: CANONICAL_CHANNELS,
        sample_rate: CANONICAL_SAMPLE_RATE,
    }
}

/// 任意形式のPCMを正準48k stereoへ変換する(channel-map → 必要なら固定比resample)。
///
/// pan未対応のため mono→stereo は L=R 複製。3ch以上は拒否。
pub fn to_canonical(cache: &PcmCache) -> Result<PcmCache> {
    let stereo = map_channels_to_stereo(cache)?;
    if stereo.format().sample_rate == CANONICAL_SAMPLE_RATE {
        return Ok(stereo);
    }
    resample_whole(&stereo, CANONICAL_SAMPLE_RATE)
}

fn map_channels_to_stereo(cache: &PcmCache) -> Result<PcmCache> {
    let fmt = cache.format();
    match fmt.channels {
        2 => Ok(cache.clone()),
        1 => {
            let frames = cache.frame_count() as usize;
            let mut out = Vec::with_capacity(frames * 2);
            let src = cache
                .read_frames(0, frames)
                .expect("full cache read must succeed");
            for &s in src {
                out.push(s);
                out.push(s);
            }
            PcmCache::from_interleaved(
                out,
                PcmFormat {
                    channels: 2,
                    sample_rate: fmt.sample_rate,
                },
            )
        }
        other => Err(AudioError::UnsupportedChannels { channels: other }),
    }
}

fn resample_whole(cache: &PcmCache, device_rate: u32) -> Result<PcmCache> {
    let source_rate = cache.format().sample_rate;
    let channels = cache.format().channels;
    let mut resampler = FixedRatioResampler::new(source_rate, device_rate, channels)?;
    resampler.reset();

    let total = cache.frame_count() as usize;
    let mut out = Vec::new();
    let mut cursor = 0usize;
    while cursor < total {
        let need = resampler.input_frames_next();
        let remain = total - cursor;
        if remain >= need {
            let chunk = cache.read_frames(cursor as u64, need)?;
            out.extend_from_slice(resampler.process_interleaved(chunk)?);
            cursor += need;
        } else {
            let chunk = cache.read_frames(cursor as u64, remain)?;
            out.extend_from_slice(resampler.process_partial_interleaved(chunk)?);
            cursor = total;
        }
    }
    for _ in 0..8 {
        let flushed = resampler.flush_silence_chunk()?;
        if flushed.is_empty() {
            break;
        }
        out.extend_from_slice(flushed);
    }

    PcmCache::from_interleaved(
        out,
        PcmFormat {
            channels,
            sample_rate: device_rate,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mono_to_stereo_duplicates_channels() {
        let mono = PcmCache::from_interleaved(
            vec![0.5, -0.25, 0.0],
            PcmFormat {
                channels: 1,
                sample_rate: 48_000,
            },
        )
        .unwrap();
        let stereo = to_canonical(&mono).unwrap();
        assert_eq!(stereo.format(), canonical_format());
        assert_eq!(
            stereo.read_frames(0, 3).unwrap(),
            &[0.5, 0.5, -0.25, -0.25, 0.0, 0.0]
        );
    }

    #[test]
    fn rejects_more_than_two_channels() {
        let bad = PcmCache::from_interleaved(
            vec![0.0; 6],
            PcmFormat {
                channels: 3,
                sample_rate: 48_000,
            },
        )
        .unwrap();
        assert!(matches!(
            to_canonical(&bad),
            Err(AudioError::UnsupportedChannels { channels: 3 })
        ));
    }
}
