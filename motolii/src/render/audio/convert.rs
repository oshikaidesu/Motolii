
use crate::render::audio::cache::{PcmCache, PcmFormat};
use crate::render::audio::error::{AudioError, Result};
use crate::render::audio::resample::FixedRatioResampler;

pub const CANONICAL_SAMPLE_RATE: u32 = 48_000;
pub const CANONICAL_CHANNELS: u16 = 2;

pub fn canonical_format() -> PcmFormat {
    PcmFormat {
        channels: CANONICAL_CHANNELS,
        sample_rate: CANONICAL_SAMPLE_RATE,
    }
}

pub fn time_to_canonical_frames(t: crate::doc::core::RationalTime) -> u64 {
    if t <= crate::doc::core::RationalTime::ZERO {
        return 0;
    }
    let num = t.num().max(0) as u128;
    let den = t.den().max(1) as u128;
    ((num * u128::from(CANONICAL_SAMPLE_RATE)) / den) as u64
}

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
