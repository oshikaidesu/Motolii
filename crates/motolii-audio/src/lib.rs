//! motolii-audio: D4 — 楽曲1本のSymphoniaデコード→PCM全展開キャッシュ+cpal出力+
//! SPSC リングバッファ/プロデューサスレッド(ミキサーなし)。
//!
//! docs/specs/M2-document-model.md「音声トランスポート設計」の4./5.をD4の範囲で実装する。
//! Documentスキーマには一切触れない独立レーン(docs/reviews/2026-07-12-m2-permanence-prevention.md
//! 「依存直列を飛ばさない」: D4は独立)。
//!
//! **D4の非ゴール**(仕様書D4行・#123): clock owner交代・映像frame drop・adaptive
//! resamplingはD5。export muxはD6。波形peakはM3/U3。ミキサー・clip内蔵音声・
//! 複数trackはこのクレートの範囲外(コンセプト決定: 音声はプロジェクト直下の楽曲1本)。

mod cache;
mod decode;
mod device;
mod error;
mod producer;
mod ring;

pub use cache::{PcmCache, PcmFormat};
pub use decode::{decode_file, decode_file_with_limits, decode_stream, decode_stream_with_limits};
pub use device::OutputStream;
pub use error::{AudioError, Result};
pub use producer::AudioProducer;
pub use ring::{channel, fill_or_silence, PlaybackCounters, RingConsumer, RingProducer};

// Symphoniaのpublicな`Hint`/`MediaSourceStream`型はdecode_streamの引数として
// 呼び出し側が組み立てる必要があるため再exportする。
pub use symphonia::core::formats::probe::Hint;
pub use symphonia::core::io::MediaSourceStream;
