//! motolii-audio: D4 — 楽曲1本のSymphoniaデコード→PCM全展開キャッシュ+cpal出力+
//! SPSC リングバッファ/プロデューサスレッド(ミキサーなし)。
//! D4-FU — デバイス≠素材サンプルレート時の固定比リサンプル(プロデューサ側・リング書き込み前)。
//!
//! docs/specs/M2-document-model.md「音声トランスポート設計」の4./5./6.をD4/D4-FUの範囲で実装する。
//! Documentスキーマには一切触れない独立レーン(docs/reviews/2026-07-12-m2-permanence-prevention.md
//! 「依存直列を飛ばさない」: D4は独立)。
//!
//! **D4/D4-FUの非ゴール**(仕様書D4・D4-FU行・#123/#147): clock owner交代・映像frame drop・
//! 適応解像度/DRS・自動バリスピードはD5。export muxはD6。波形peakはM3/U3。ミキサー・
//! clip内蔵音声・複数trackはこのクレートの範囲外(コンセプト決定: 音声はプロジェクト直下の楽曲1本)。
//! Transport側でのリサンプラ遅延引き算もしない(遅延はproducerの先頭trimで閉じる)。

mod cache;
mod decode;
mod device;
mod error;
mod producer;
mod resample;
mod ring;

pub use cache::{PcmCache, PcmFormat};
pub use decode::{decode_file, decode_file_with_limits, decode_stream, decode_stream_with_limits};
pub use device::{negotiate_output, select_device_sample_rate, NegotiatedOutput, OutputStream};
pub use error::{AudioError, Result};
pub use producer::AudioProducer;
pub use resample::{source_frame_to_device, FixedRatioResampler};
pub use ring::{channel, fill_or_silence, PlaybackCounters, RingConsumer, RingProducer};

// Symphoniaのpublicな`Hint`/`MediaSourceStream`型はdecode_streamの引数として
// 呼び出し側が組み立てる必要があるため再exportする。
pub use symphonia::core::formats::probe::Hint;
pub use symphonia::core::io::MediaSourceStream;
