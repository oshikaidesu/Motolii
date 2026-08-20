//! D4契約(旧 motolii-audio から継承): 公開APIはpanicせず、入力起因の失敗を
//! 全て型付き`AudioError`で返す。
//!
//! 旧クレートにあった `InvalidRingConfig` / `NoOutputDevice` / `UnsupportedOutputConfig` /
//! `Cpal` / `ProducerSpawn` は落とした — 第1切片は device/ring を持ち込まない
//! (発注書の柵)ので、対応する経路がこの crate に存在しない。

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

    #[error("decoded sample count {observed} exceeds limit {limit}")]
    SampleCountLimit { observed: u64, limit: u64 },

    /// 固定比リサンプルの構築・処理失敗(D4-FU)。公開APIはpanicしない。
    #[error("fixed-ratio resample error: {detail}")]
    Resample { detail: &'static str },

    /// AG-2: gainが有限かつ0以上でない。
    #[error("gain must be finite and >= 0, got {gain}")]
    InvalidGain { gain: f64 },

    /// AG-2: mix範囲またはTimeMap写像が不正。
    #[error("invalid mix timeline range or time map")]
    InvalidMixRange,

    /// AG-2: 指定audio stream ordinalがコンテナに無い。
    #[error("audio stream ordinal {ordinal} not found in source")]
    StreamNotFound { ordinal: u32 },

    /// store 側の読み取り(track/meta/attrs/composition)が失敗した。
    #[error("store read failed: {0}")]
    Store(#[from] motolii_store::StoreError),
}

pub type Result<T> = std::result::Result<T, AudioError>;
