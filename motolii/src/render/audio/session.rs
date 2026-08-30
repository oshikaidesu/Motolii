
use std::sync::Arc;

use cpal::traits::HostTrait;

use motolii_core::RationalTime;

use crate::render::audio::clock::{DeviceWaitLatency, PlaybackClock, PlaybackCounters};
use crate::render::audio::convert::{canonical_format, time_to_canonical_frames};
use crate::render::audio::device::{negotiate_output, NegotiatedOutput, OutputStream};
use crate::render::audio::error::{AudioError, Result};
use crate::render::audio::producer::MixProducer;
use crate::render::audio::program::AudioProgram;

const RING_CAPACITY_FRAMES: usize = 4_096;

pub struct PlaybackSession {
    clock: PlaybackClock,
    negotiated: Option<NegotiatedOutput>,
    _stream: Option<OutputStream>,
    _producer: Option<MixProducer>,
}

impl PlaybackSession {
    pub fn open_default(program: Arc<AudioProgram>, at: RationalTime) -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(AudioError::NoOutputDevice)?;
        Self::open_on_device(program, at, &device)
    }

    pub fn open_on_device(
        program: Arc<AudioProgram>,
        at: RationalTime,
        device: &cpal::Device,
    ) -> Result<Self> {
        let format = canonical_format();
        let negotiated = negotiate_output(device, format)?;

        let counters = Arc::new(PlaybackCounters::default());
        let device_wait = Arc::new(DeviceWaitLatency::default());
        let (ring_prod, ring_cons) =
            rtrb::RingBuffer::<f32>::new(RING_CAPACITY_FRAMES * format.channels as usize);

        let stream = OutputStream::open_negotiated_shared(
            device,
            &negotiated,
            ring_cons,
            Arc::clone(&counters),
            Arc::clone(&device_wait),
        )?;

        let start_frame = time_to_canonical_frames(at);
        let producer = MixProducer::spawn(
            program,
            ring_prod,
            start_frame,
            negotiated.device_sample_rate,
        )?;

        let mut clock =
            PlaybackClock::new(Arc::clone(&counters), Arc::clone(&device_wait), negotiated.device_sample_rate)?;
        clock.start(at);

        Ok(Self {
            clock,
            negotiated: Some(negotiated),
            _stream: Some(stream),
            _producer: Some(producer),
        })
    }

    pub fn clock(&self) -> &PlaybackClock {
        &self.clock
    }

    pub fn clock_mut(&mut self) -> &mut PlaybackClock {
        &mut self.clock
    }

    pub fn negotiated(&self) -> Option<&NegotiatedOutput> {
        self.negotiated.as_ref()
    }

    pub fn seek(&mut self, at: RationalTime) {
        self.clock.seek(at);
        if let Some(producer) = self._producer.as_ref() {
            producer.seek(time_to_canonical_frames(at));
        }
    }

    #[doc(hidden)]
    pub fn for_simulation(clock: PlaybackClock) -> Self {
        Self {
            clock,
            negotiated: None,
            _stream: None,
            _producer: None,
        }
    }
}
