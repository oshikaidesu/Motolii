//! owns: cpal出力(A2: 実デバイスへ音を出す経路)。D4契約(旧 crate から継承):
//! ここだけがハードウェアに触る。
//!
//! 旧 `crates/motolii-audio/src/device.rs` からの移植 — **型の読み替え**:
//! - `RingConsumer`(自前SPSC、KNOWN.md「音声」節で再発明と判定済み)→
//!   `rtrb::Consumer<f32>`(A1 の `ring.rs` doc 参照、上流 `rtrb` を直接使う)
//! - `crate::ring::{fill_or_silence, PlaybackCounters}` → `crate::clock::
//!   PlaybackCounters` + `crate::ring::fill_or_silence`(A1 で `clock.rs`/
//!   `ring.rs` へ分離済み、`fill_or_silence` は `channels` を明示引数に取る形へ
//!   変わっている — 呼び出し側でその差を埋める)
//! - `crate::latency::DeviceWaitLatency` → `crate::clock::DeviceWaitLatency`
//!
//! **持ってこなかったもの**: `open_negotiated_with_latency`(D5 以前の過渡API、
//! `open_negotiated_shared` 1本に統合済み)。

use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, SupportedStreamConfig};

use crate::cache::PcmFormat;
use crate::clock::{DeviceWaitLatency, PlaybackCounters};
use crate::error::{AudioError, Result};
use crate::ring::fill_or_silence;

/// 素材形式に対して選んだデバイス出力形式(D4-FU、旧版から無改造)。
#[derive(Debug, Clone)]
pub struct NegotiatedOutput {
    /// 素材(変換前PCM)の形式。
    pub source: PcmFormat,
    /// デバイスへ開くサンプルレート。
    pub device_sample_rate: u32,
    /// cpalが受理した具体config。
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

/// 再生中のcpal出力ストリーム。Dropでストリームを止める。
pub struct OutputStream {
    stream: Stream,
}

impl OutputStream {
    /// デフォルト出力デバイスへ`consumer`を繋いで再生を開始する。
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

    /// 指定デバイスへ`consumer`を繋いで再生を開始する(テスト/明示選択用)。
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

    /// `PlaybackCounters`/`DeviceWaitLatency`(A1の`PlaybackClock`と共有する)で
    /// コールバックを組んで開く(A2本番経路)。callback内は
    /// `update_from_output_callback`(型を読むだけ)+`fill_or_silence`
    /// (`&mut Consumer`から読むだけ)のみ — alloc しない(KNOWN.md「音声」節)。
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

/// デバイスのサポート範囲から出力レートを交渉する(D4-FU、旧版から無改造)。
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

/// サポート範囲リストからデバイス出力レートを選ぶ(ハードウェア無しの単体テスト用、
/// 旧版から無改造 — この関数だけが「実デバイス経路はテスト不能でよい」の例外
/// (pure、cpal の型だけを読む))。
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

    // 素材非対応でも再生開始できるよう、定番レートを優先する。
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
        // 素材44.1kのみ拒否、48kだけ持つデバイス。
        let ranges = [(48_000, 48_000)];
        assert_eq!(select_device_sample_rate(44_100, &ranges), Some(48_000));
    }

    #[test]
    fn select_returns_none_for_empty_ranges() {
        assert_eq!(select_device_sample_rate(48_000, &[]), None);
    }
}
