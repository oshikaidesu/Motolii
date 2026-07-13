//! cpal出力(D4契約: このモジュールだけがハードウェアに触る)。
//!
//! **責務の境界(旧PR#90の再発防止)**: ここは`RingConsumer`を消費してデバイスへ
//! 渡すだけで、再生位置(Transport)の所有権を持たない。曲のどこから鳴らすかは
//! [`crate::producer::AudioProducer`]の起動時`start_frame`が決め、このモジュールは
//! それを一切知らない。clock owner交代・映像フレームドロップ・適応リサンプリングは
//! D5のTransportの責務(D4の非ゴール)。

use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};

use crate::cache::PcmFormat;
use crate::error::{AudioError, Result};
use crate::ring::{fill_or_silence, PlaybackCounters, RingConsumer};

/// 再生中のcpal出力ストリーム。Dropでストリームを止める。
pub struct OutputStream {
    stream: Stream,
    counters: Arc<PlaybackCounters>,
}

impl OutputStream {
    /// デフォルト出力デバイスへ`consumer`を繋いで再生を開始する。
    pub fn open_default(format: PcmFormat, consumer: RingConsumer) -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(AudioError::NoOutputDevice)?;
        Self::open_on_device(&device, format, consumer)
    }

    /// 指定デバイスへ`consumer`を繋いで再生を開始する(テスト/明示選択用)。
    pub fn open_on_device(
        device: &cpal::Device,
        format: PcmFormat,
        consumer: RingConsumer,
    ) -> Result<Self> {
        let supported = pick_output_config(device, format.sample_rate, format.channels)?;
        let counters = Arc::new(PlaybackCounters::default());
        let counters_cb = Arc::clone(&counters);

        let config = supported.config();
        let stream = device.build_output_stream(
            config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // D4契約: allocate/block/decodeしない。リングから読むだけ。
                fill_or_silence(&consumer, data, &counters_cb);
            },
            |err| {
                eprintln!("motolii-audio: output stream error: {err}");
            },
            None,
        )?;
        stream.play()?;

        Ok(Self { stream, counters })
    }

    /// 実供給フレーム数とアンダーラン(無音補填)フレーム数の監視口を返す。
    pub fn counters(&self) -> Arc<PlaybackCounters> {
        Arc::clone(&self.counters)
    }

    pub fn pause(&self) -> Result<()> {
        self.stream.pause().map_err(AudioError::from)
    }

    pub fn play(&self) -> Result<()> {
        self.stream.play().map_err(AudioError::from)
    }
}

fn pick_output_config(
    device: &cpal::Device,
    sample_rate: u32,
    channels: u16,
) -> Result<cpal::SupportedStreamConfig> {
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
