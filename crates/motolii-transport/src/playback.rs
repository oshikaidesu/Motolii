//! 再生開始経路: Transport + 共有`PlaybackCounters`/`DeviceWaitLatency` + cpal出力。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use cpal::traits::HostTrait;
use motolii_audio::{
    channel, negotiate_output, source_frame_to_device, AudioProducer, AudioProgram,
    DeviceWaitLatency, MixProducer, NegotiatedOutput, OutputStream, PcmCache, PcmFormat,
    PlaybackCounters, RingProducer,
};
use motolii_core::{Fps, Quality};

use crate::{Transport, TransportError};

/// Transportと音声出力を共有状態で束ねる再生セッション。
pub struct PlaybackSession {
    transport: Transport,
    counters: Arc<PlaybackCounters>,
    device_wait: Arc<DeviceWaitLatency>,
    negotiated: NegotiatedOutput,
    source_start_frame: u64,
    source_sample_rate: u32,
    _output: OutputStream,
    _producer: PlaybackProducer,
    playing: Arc<AtomicBool>,
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
        Self::open_on_device_with(
            format,
            start_frame,
            fps,
            base_quality,
            gpu,
            device,
            |ring, rate| {
                AudioProducer::spawn_with_device_rate(Arc::clone(&cache), ring, start_frame, rate)
                    .map(PlaybackProducer::Pcm)
            },
        )
    }

    /// Document由来の決定論的mixed PCMをデフォルト出力へ接続する。
    pub fn open_default_program(
        program: Arc<AudioProgram>,
        start_frame: u64,
        fps: Fps,
        base_quality: Quality,
        gpu: Option<&motolii_gpu::GpuCtx>,
    ) -> Result<Self, PlaybackSessionError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(PlaybackSessionError::NoOutputDevice)?;
        Self::open_on_device_program(program, start_frame, fps, base_quality, gpu, &device)
    }

    /// 指定デバイスへDocument由来のmixed PCMを接続する。
    pub fn open_on_device_program(
        program: Arc<AudioProgram>,
        start_frame: u64,
        fps: Fps,
        base_quality: Quality,
        gpu: Option<&motolii_gpu::GpuCtx>,
        device: &cpal::Device,
    ) -> Result<Self, PlaybackSessionError> {
        let format = PcmFormat {
            channels: motolii_audio::CANONICAL_CHANNELS,
            sample_rate: motolii_audio::CANONICAL_SAMPLE_RATE,
        };
        Self::open_on_device_with(
            format,
            start_frame,
            fps,
            base_quality,
            gpu,
            device,
            |ring, rate| {
                MixProducer::spawn_with_device_rate(
                    Arc::clone(&program),
                    ring,
                    start_frame,
                    rate,
                    None,
                )
                .map(PlaybackProducer::Mixed)
            },
        )
    }

    fn open_on_device_with(
        format: PcmFormat,
        start_frame: u64,
        fps: Fps,
        base_quality: Quality,
        gpu: Option<&motolii_gpu::GpuCtx>,
        device: &cpal::Device,
        producer_factory: impl FnOnce(
            RingProducer,
            u32,
        ) -> Result<PlaybackProducer, motolii_audio::AudioError>,
    ) -> Result<Self, PlaybackSessionError> {
        let counters = Arc::new(PlaybackCounters::default());
        let device_wait = Arc::new(DeviceWaitLatency::default());
        let (ring_prod, ring_cons) =
            channel(format.channels, 4_096).map_err(PlaybackSessionError::Audio)?;

        let negotiated = negotiate_output(device, format).map_err(PlaybackSessionError::Audio)?;
        let output = OutputStream::open_negotiated_shared(
            device,
            &negotiated,
            ring_cons,
            Arc::clone(&counters),
            Some(Arc::clone(&device_wait)),
        )
        .map_err(PlaybackSessionError::Audio)?;

        let producer = producer_factory(ring_prod, negotiated.device_sample_rate)
            .map_err(PlaybackSessionError::Audio)?;

        // Transportのoriginは供給済みデバイスフレームと同じレートへ変換する。
        let transport_origin_frame = source_frame_to_device(
            start_frame,
            format.sample_rate,
            negotiated.device_sample_rate,
        );
        let transport = if let Some(gpu) = gpu {
            Transport::new_with_gpu_origin(
                Arc::clone(&counters),
                Arc::clone(&device_wait),
                fps,
                negotiated.device_sample_rate,
                base_quality,
                gpu,
                transport_origin_frame,
            )
            .map_err(PlaybackSessionError::Transport)?
        } else {
            Transport::new_with_origin(
                Arc::clone(&counters),
                Arc::clone(&device_wait),
                fps,
                negotiated.device_sample_rate,
                base_quality,
                false,
                transport_origin_frame,
            )
            .map_err(PlaybackSessionError::Transport)?
        };
        let playing = Arc::new(AtomicBool::new(true));

        Ok(Self {
            transport,
            counters,
            device_wait,
            negotiated,
            source_start_frame: start_frame,
            source_sample_rate: format.sample_rate,
            _output: output,
            _producer: producer,
            playing,
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

    pub fn source_start_frame(&self) -> u64 {
        self.source_start_frame
    }

    pub fn source_sample_rate(&self) -> u32 {
        self.source_sample_rate
    }

    pub fn pause(&self) -> Result<(), PlaybackSessionError> {
        self._output.pause().map_err(PlaybackSessionError::Audio)?;
        self.playing.store(false, Ordering::Release);
        Ok(())
    }

    pub fn play(&self) -> Result<(), PlaybackSessionError> {
        self._output.play().map_err(PlaybackSessionError::Audio)?;
        self.playing.store(true, Ordering::Release);
        Ok(())
    }

    pub fn is_playing(&self) -> bool {
        self.playing.load(Ordering::Acquire)
    }
}

// ProducerはPlaybackSessionの生存期間を音声workerへ伝えるためだけに保持する。
#[allow(dead_code)]
enum PlaybackProducer {
    Pcm(AudioProducer),
    Mixed(MixProducer),
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
