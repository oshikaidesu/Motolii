//! D4契約(旧 motolii-audio から継承): 公開APIはpanicせず、入力起因の失敗を
//! 全て型付き`AudioError`で返す。
//!
//! 旧クレートにあった `InvalidRingConfig` は落としたまま(`ring` モジュールが
//! 自前SPSCを持たない設計 — `ring.rs` doc 参照 — なので対応する経路が無い)。
//! `NoOutputDevice`/`UnsupportedOutputConfig`/`Cpal`/`ProducerSpawn` は第1切片
//! (A1)で落としていたが、実時間再生 第2切片(A2、device/producer 結線)で
//! 旧 `crates/motolii-audio/src/error.rs` から戻した — 定義は無改造の移植。

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

    /// B42: panが有限でない(NaN/inf)。範囲外(-1..1超)自体はエラーにせず
    /// clampする(W3C Web Audio API `StereoPannerNode` 仕様の挙動と同型 —
    /// `mix.rs::apply_pan_stereo` docコメント参照)。
    #[error("pan must be finite, got {pan}")]
    InvalidPan { pan: f64 },

    /// STORE3結線: `property::FADE_IN`/`FADE_OUT` の評価値が有限かつ0以上でない
    /// (負のフェード尺、または `Value::F64` でない値)。`FadeSpec::fade_in`/
    /// `fade_out` は負の尺を表現できない(`mix.rs::fade_envelope` の前提)。
    #[error("fade duration must be finite and >= 0 seconds, got {fade}")]
    InvalidFade { fade: f64 },

    /// AG-2: mix範囲またはTimeMap写像が不正。
    #[error("invalid mix timeline range or time map")]
    InvalidMixRange,

    /// AG-2: 指定audio stream ordinalがコンテナに無い。
    #[error("audio stream ordinal {ordinal} not found in source")]
    StreamNotFound { ordinal: u32 },

    /// store 側の読み取り(track/meta/attrs/composition)が失敗した。
    #[error("store read failed: {0}")]
    Store(#[from] motolii_store::StoreError),

    /// A2: デフォルト出力デバイスが無い(旧 `crates/motolii-audio` から無改造)。
    #[error("no default audio output device available")]
    NoOutputDevice,

    /// A2: 素材の channels/sample_rate に対応するデバイス出力 config が無い。
    #[error(
        "no output stream config for {channels} ch @ {sample_rate} Hz on this device ({detail})"
    )]
    UnsupportedOutputConfig {
        channels: u16,
        sample_rate: u32,
        detail: &'static str,
    },

    /// A2: cpal 側のエラー(device列挙・stream構築・play/pause 等)。
    #[error(transparent)]
    Cpal(#[from] cpal::Error),

    /// A2: producer スレッドの起動失敗。
    #[error("failed to spawn producer thread: {0}")]
    ProducerSpawn(std::io::Error),

    /// A2: `PlaybackClock`/`RationalTime`演算の失敗(旧
    /// `crates/motolii-transport::TransportError::Time`の移植)。
    #[error(transparent)]
    Time(#[from] motolii_core::RationalTimeError),
}

pub type Result<T> = std::result::Result<T, AudioError>;
