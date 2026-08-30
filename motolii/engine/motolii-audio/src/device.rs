
use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, SupportedStreamConfig};

use crate::cache::PcmFormat;
use crate::clock::{DeviceWaitLatency, PlaybackCounters};
use crate::error::{AudioError, Result};
use crate::ring::fill_or_silence;

#[derive(Debug, Clone)]
pub struct NegotiatedOutput {
    pub source: PcmFormat,
    pub device_sample_rate: u32,
    pub(crate) config: SupportedStreamConfig,
}

impl NegotiatedOutput {
    pub fn device_format(&self) -> PcmFormat {
        PcmFormat {
            channels: self.source.channels,
            sample_rate: self.device_sample_rate,
        }
    }

    pub fn needs_resample(&self) -> bool {
        self.device_sample_rate != self.source.sample_rate
    }
}

pub struct OutputStream {
    stream: Stream,
}

impl OutputStream {
    pub fn open_default(
        source: PcmFormat,
        consumer: rtrb::Consumer<f32>,
        counters: Arc<PlaybackCounters>,
        device_wait: Arc<DeviceWaitLatency>,
    ) -> Result<(Self, NegotiatedOutput)> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(AudioError::NoOutputDevice)?;
        Self::open_on_device(&device, source, consumer, counters, device_wait)
    }

    pub fn open_on_device(
        device: &cpal::Device,
        source: PcmFormat,
        consumer: rtrb::Consumer<f32>,
        counters: Arc<PlaybackCounters>,
        device_wait: Arc<DeviceWaitLatency>,
    ) -> Result<(Self, NegotiatedOutput)> {
        let negotiated = negotiate_output(device, source)?;
        let stream =
            Self::open_negotiated_shared(device, &negotiated, consumer, counters, device_wait)?;
        Ok((stream, negotiated))
    }

    pub fn open_negotiated_shared(
        device: &cpal::Device,
        negotiated: &NegotiatedOutput,
        mut consumer: rtrb::Consumer<f32>,
        counters: Arc<PlaybackCounters>,
        device_wait: Arc<DeviceWaitLatency>,
    ) -> Result<Self> {
        let sample_rate = negotiated.device_sample_rate;
        let channels = negotiated.source.channels as usize;

        let config = negotiated.config.config();
        let stream = device.build_output_stream(
            config,
            move |data: &mut [f32], info: &cpal::OutputCallbackInfo| {
                device_wait.update_from_output_callback(info, sample_rate);
                fill_or_silence(&mut consumer, data, channels, &counters);
            },
            |err| {
                eprintln!("motolii-audio: output stream error: {err}");
            },
            None,
        )?;
        stream.play()?;

        Ok(Self { stream })
    }

    pub fn pause(&self) -> Result<()> {
        self.stream.pause().map_err(AudioError::from)
    }

    pub fn play(&self) -> Result<()> {
        self.stream.play().map_err(AudioError::from)
    }
}

pub fn negotiate_output(device: &cpal::Device, source: PcmFormat) -> Result<NegotiatedOutput> {
    if source.channels == 0 {
        return Err(AudioError::UnsupportedChannels {
            channels: source.channels,
        });
    }
    if source.sample_rate == 0 {
        return Err(AudioError::UnsupportedSampleRate {
            sample_rate: source.sample_rate,
        });
    }

    let ranges = collect_f32_channel_ranges(device, source.channels)?;
    let device_sample_rate = select_device_sample_rate(source.sample_rate, &ranges).ok_or(
        AudioError::UnsupportedOutputConfig {
            channels: source.channels,
            sample_rate: source.sample_rate,
            detail: "no f32 output config for the requested channel count",
        },
    )?;

    let config = pick_exact_output_config(device, device_sample_rate, source.channels)?;
    Ok(NegotiatedOutput {
        source,
        device_sample_rate,
        config,
    })
}

pub fn select_device_sample_rate(source_rate: u32, ranges: &[(u32, u32)]) -> Option<u32> {
    if source_rate == 0 || ranges.is_empty() {
        return None;
    }
    if ranges
        .iter()
        .any(|(lo, hi)| *lo <= source_rate && source_rate <= *hi)
    {
        return Some(source_rate);
    }

    const PREFERRED: &[u32] = &[
        48_000, 44_100, 96_000, 88_200, 32_000, 22_050, 16_000, 8_000,
    ];
    for &cand in PREFERRED {
        if ranges.iter().any(|(lo, hi)| *lo <= cand && cand <= *hi) {
            return Some(cand);
        }
    }

    let mut best: Option<u32> = None;
    let mut best_dist = u32::MAX;
    for &(lo, hi) in ranges {
        for cand in [lo, hi] {
            let dist = cand.abs_diff(source_rate);
            if dist < best_dist {
                best_dist = dist;
                best = Some(cand);
            }
        }
    }
    best
}

fn collect_f32_channel_ranges(device: &cpal::Device, channels: u16) -> Result<Vec<(u32, u32)>> {
    let configs = device.supported_output_configs()?;
    Ok(configs
        .filter(|c| c.channels() == channels && c.sample_format() == SampleFormat::F32)
        .map(|c| (c.min_sample_rate(), c.max_sample_rate()))
        .collect())
}

fn pick_exact_output_config(
    device: &cpal::Device,
    sample_rate: u32,
    channels: u16,
) -> Result<SupportedStreamConfig> {
    let configs = device.supported_output_configs()?;
    configs
        .filter(|c| c.channels() == channels && c.sample_format() == SampleFormat::F32)
        .find(|c| c.min_sample_rate() <= sample_rate && c.max_sample_rate() >= sample_rate)
        .map(|range| range.with_sample_rate(sample_rate))
        .ok_or(AudioError::UnsupportedOutputConfig {
            channels,
            sample_rate,
            detail: "no f32 output config spans the requested channels/sample-rate",
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_prefers_exact_source_rate() {
        let ranges = [(44_100, 48_000)];
        assert_eq!(select_device_sample_rate(48_000, &ranges), Some(48_000));
        assert_eq!(select_device_sample_rate(44_100, &ranges), Some(44_100));
    }

    #[test]
    fn select_falls_back_when_source_unsupported() {
        let ranges = [(48_000, 48_000)];
        assert_eq!(select_device_sample_rate(44_100, &ranges), Some(48_000));
    }

    #[test]
    fn select_returns_none_for_empty_ranges() {
        assert_eq!(select_device_sample_rate(48_000, &[]), None);
    }
}
