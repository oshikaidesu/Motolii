//! 再生開始経路: Transport + 共有`PlaybackCounters`/`DeviceWaitLatency` + cpal出力。

use std::sync::Arc;

use cpal::traits::HostTrait;
use motolii_audio::{
    channel, negotiate_output, AudioProducer, DeviceWaitLatency, NegotiatedOutput, OutputStream,
    PcmCache, PlaybackCounters,
};
use motolii_core::{Fps, Quality};

use crate::{Transport, TransportError};

/// Transportと音声出力を共有状態で束ねる再生セッション。
pub struct PlaybackSession {
    transport: Transport,
    counters: Arc<PlaybackCounters>,
    device_wait: Arc<DeviceWaitLatency>,
    negotiated: NegotiatedOutput,
    _output: OutputStream,
    _producer: AudioProducer,
}

impl PlaybackSession {
    /// デフォルト出力デバイスで再生を開始する。
    pub fn open_default(
        cache: Arc<PcmCache>,
        start_frame: u64,
        fps: Fps,
        base_quality: Quality,
        gpu: Option<&motolii_gpu::GpuCtx>,
    ) -> Result<Self, PlaybackSessionError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(PlaybackSessionError::NoOutputDevice)?;
        Self::open_on_device(cache, start_frame, fps, base_quality, gpu, &device)
    }

    /// 指定デバイスで再生を開始する。
    pub fn open_on_device(
        cache: Arc<PcmCache>,
        start_frame: u64,
        fps: Fps,
        base_quality: Quality,
        gpu: Option<&motolii_gpu::GpuCtx>,
        device: &cpal::Device,
    ) -> Result<Self, PlaybackSessionError> {
        let format = cache.format();
        let counters = Arc::new(PlaybackCounters::default());
        let device_wait = Arc::new(DeviceWaitLatency::default());
        let (ring_prod, ring_cons) = channel(format.channels, 4_096)
            .map_err(PlaybackSessionError::Audio)?;

        let negotiated = negotiate_output(device, format).map_err(PlaybackSessionError::Audio)?;
        let output = OutputStream::open_negotiated_shared(
            device,
            &negotiated,
            ring_cons,
            Arc::clone(&counters),
            Some(Arc::clone(&device_wait)),
        )
        .map_err(PlaybackSessionError::Audio)?;

        let producer = AudioProducer::spawn_with_device_rate(
            Arc::clone(&cache),
            ring_prod,
            start_frame,
            negotiated.device_sample_rate,
        )
        .map_err(PlaybackSessionError::Audio)?;

        let transport = if let Some(gpu) = gpu {
            Transport::new_with_gpu(
                Arc::clone(&counters),
                Arc::clone(&device_wait),
                fps,
                negotiated.device_sample_rate,
                base_quality,
                gpu,
            )
            .map_err(PlaybackSessionError::Transport)?
        } else {
            Transport::new(
                Arc::clone(&counters),
                Arc::clone(&device_wait),
                fps,
                negotiated.device_sample_rate,
                base_quality,
                false,
            )
            .map_err(PlaybackSessionError::Transport)?
        };

        Ok(Self {
            transport,
            counters,
            device_wait,
            negotiated,
            _output: output,
            _producer: producer,
        })
    }

    pub fn transport(&self) -> &Transport {
        &self.transport
    }

    pub fn transport_mut(&mut self) -> &mut Transport {
        &mut self.transport
    }

    pub fn counters(&self) -> &Arc<PlaybackCounters> {
        &self.counters
    }

    pub fn device_wait(&self) -> &Arc<DeviceWaitLatency> {
        &self.device_wait
    }

    pub fn negotiated(&self) -> &NegotiatedOutput {
        &self.negotiated
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PlaybackSessionError {
    #[error("no default output audio device")]
    NoOutputDevice,
    #[error(transparent)]
    Audio(#[from] motolii_audio::AudioError),
    #[error(transparent)]
    Transport(#[from] TransportError),
}
