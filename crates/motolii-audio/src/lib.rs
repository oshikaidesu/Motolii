//! motolii-audio: D4 PCM cache + D4-FU resample + AG-2 deterministic mixer。
//!
//! docs/specs/M2-document-model.md「音声トランスポート設計」と
//! docs/reviews/2026-07-14-audio-generalization-design.md §6 を実装する。
//!
//! **AG-2**: `mix_audio` / `AudioProgram` で Soundtrack+Clip audio を正準48k stereoへ
//! 決定論的に加算する。Transport入力はこのmixed PCMへ一般化する(D5はクロック所有のみ)。
//! meteringはTransientのpeak/clip snapshot。自動normalize/limiterは行わない。

mod cache;
mod convert;
mod decode;
mod device;
mod error;
mod latency;
mod meter;
mod mix;
mod producer;
mod program;
mod resample;
mod ring;
mod waveform;

pub use cache::{PcmCache, PcmFormat};
pub use convert::{canonical_format, to_canonical, CANONICAL_CHANNELS, CANONICAL_SAMPLE_RATE};
pub use decode::{
    decode_file, decode_file_audio_ordinal, decode_file_audio_ordinal_with_limits,
    decode_file_with_limits, decode_stream, decode_stream_with_limits,
};
pub use device::{negotiate_output, select_device_sample_rate, NegotiatedOutput, OutputStream};
pub use latency::DeviceWaitLatency;
pub use error::{AudioError, Result};
pub use meter::{AudioMeter, ClipLatch, MeterSnapshot, CLIP_THRESHOLD};
pub use mix::{mix_audio, MixReport, MixSource};
pub use producer::{AudioProducer, MixProducer};
pub use program::{program_from_sources, AudioProgram};
pub use resample::{source_frame_to_device, FixedRatioResampler};
pub use ring::{channel, fill_or_silence, PlaybackCounters, RingConsumer, RingProducer};
pub use waveform::waveform_peaks;

// Symphoniaのpublicな`Hint`/`MediaSourceStream`型はdecode_streamの引数として
// 呼び出し側が組み立てる必要があるため再exportする。
pub use symphonia::core::formats::probe::Hint;
pub use symphonia::core::io::MediaSourceStream;
