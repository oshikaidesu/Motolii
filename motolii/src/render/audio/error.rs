
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to demux/decode audio source: {0}")]
    Symphonia(#[from] symphonia::core::errors::Error),

    #[error("source has no supported audio track")]
    NoAudioTrack,

    #[error("unsupported channel count: {channels} (must be >= 1)")]
    UnsupportedChannels { channels: u16 },

    #[error("unsupported sample rate: {sample_rate} Hz (must be >= 1)")]
    UnsupportedSampleRate { sample_rate: u32 },

    #[error(
        "interleaved sample buffer length {len} is not a multiple of channel count {channels}"
    )]
    MisalignedSamples { len: usize, channels: u16 },

    #[error("requested {requested} frames starting at frame {start} are out of range; source has {total} frames")]
    OutOfRange {
        start: u64,
        requested: usize,
        total: u64,
    },

    #[error("decoded sample count {observed} exceeds limit {limit}")]
    SampleCountLimit { observed: u64, limit: u64 },

    #[error("fixed-ratio resample error: {detail}")]
    Resample { detail: &'static str },

    #[error("gain must be finite and >= 0, got {gain}")]
    InvalidGain { gain: f64 },

    #[error("pan must be finite, got {pan}")]
    InvalidPan { pan: f64 },

    #[error("fade duration must be finite and >= 0 seconds, got {fade}")]
    InvalidFade { fade: f64 },

    #[error("invalid mix timeline range or time map")]
    InvalidMixRange,

    #[error("audio stream ordinal {ordinal} not found in source")]
    StreamNotFound { ordinal: u32 },

    #[error("store read failed: {0}")]
    Store(#[from] crate::doc::store::StoreError),

    #[error("no default audio output device available")]
    NoOutputDevice,

    #[error(
        "no output stream config for {channels} ch @ {sample_rate} Hz on this device ({detail})"
    )]
    UnsupportedOutputConfig {
        channels: u16,
        sample_rate: u32,
        detail: &'static str,
    },

    #[error(transparent)]
    Cpal(#[from] cpal::Error),

    #[error("failed to spawn producer thread: {0}")]
    ProducerSpawn(std::io::Error),

    #[error(transparent)]
    Time(#[from] crate::doc::core::RationalTimeError),
}

pub type Result<T> = std::result::Result<T, AudioError>;
