//! D4契約: 公開APIはpanicせず、入力起因の失敗を全て型付き`AudioError`で返す(AGENTS.md)。

/// motolii-audioの全公開APIが返すエラー。
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

    /// インターリーブサンプル数がチャンネル数の倍数でない(壊れたバッファ)。
    #[error(
        "interleaved sample buffer length {len} is not a multiple of channel count {channels}"
    )]
    MisalignedSamples { len: usize, channels: u16 },

    /// 任意位置readの境界外アクセス(D4完了条件: 境界検査)。
    #[error("requested {requested} frames starting at frame {start} are out of range; source has {total} frames")]
    OutOfRange {
        start: u64,
        requested: usize,
        total: u64,
    },

    #[error("ring buffer needs channels >= 1 and capacity_frames >= 1, got channels={channels} capacity_frames={capacity_frames}")]
    InvalidRingConfig {
        channels: u16,
        capacity_frames: usize,
    },

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

    #[error("decoded sample count {observed} exceeds limit {limit}")]
    SampleCountLimit { observed: u64, limit: u64 },

    /// 固定比リサンプルの構築・処理失敗(D4-FU)。公開APIはpanicしない。
    #[error("fixed-ratio resample error: {detail}")]
    Resample { detail: &'static str },
}

pub type Result<T> = std::result::Result<T, AudioError>;
