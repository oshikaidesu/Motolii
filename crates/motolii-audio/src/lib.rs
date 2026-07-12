//! motolii-audio: 楽曲1本のPCM全展開・リングバッファ・cpal出力(D4)。
//!
//! ミキサーは存在しない。デコードとプロデューサはコールバック外で行い、
//! cpalコールバックはリングから読むだけ(音声トランスポート設計 §4)。

mod cache;
mod error;
mod playback;
mod ring;

pub use cache::{decode_file, PcmCache, PcmFormat};
pub use error::AudioError;
pub use playback::{simulate_playback_without_underrun, PlaybackHandle, PlaybackStats};
pub use ring::SampleRing;
