//! owns: 音声の意味核 — 決定論的mix・Documentからのprogram組み立て・音声クロックの骨。
//!
//! 音声束・第1切片(2026-08-20)。旧 `crates/motolii-audio`(4,286行)+
//! 旧 `crates/motolii-transport`(音声クロック部分のみ)からの移植であって
//! 再実装ではない。KNOWN.md「音声(2026-08-20解析済み)」が調達を確定済み:
//! rodio/kira は採らず cpal 直叩き(cpal自体はまだ足さない)、mux は
//! `motolii-media` が解決済み(この crate は PCM を作るだけでよい)。
//!
//! **第1切片の範囲(発注書): 「音を出さない」— 意味と時計だけ**:
//! - [`mix`] — 決定論的 `mix_audio`(preview/export同一意味の正準PCM境界)
//! - [`program`] — `motolii_store::StoreView` から `AudioProgram`(MixSource列)を組む
//! - [`clock`] — 音声クロックの骨(供給済みフレーム − デバイス待ち、wall-clock不使用)
//! - [`cache`]/[`convert`]/[`decode`]/[`resample`] — 上記2つが要る正準PCM表現一式
//!   (symphonia/rubato、KNOWN.md で調達確定済み)
//! - [`time_map`] — クリップローカル→ソース時刻の写像(`next/core/motolii-core` が
//!   落とした型をこの crate が持つ。詳細はモジュール doc)
//!
//! **持ってこなかったもの(cpal結線・device依存は第2切片)**:
//! - `device.rs`/`producer.rs`/`ring.rs`(rtrb)/`latency.rs` の cpal 依存部分 —
//!   実際にデバイスへ音を出す経路。`clock.rs` のカウンタ演算(骨)だけ移した
//! - 旧 `motolii-transport` の `Transport`/`DrsController`/`PreviewSimulator` —
//!   映像フレームドロップ・適応解像度は音声の意味核と別の関心事で、この束の対象外
//!
//! 旧クレートの内訳(KNOWN.md): wraps 743(symphonia/rubato/cpal)/ 必然scratch 1,451
//! (mix/producer/program)/ 再発明 367(`ring.rs` — 上流 `rtrb` が既にある)/
//! テスト 1,194。この束が移したのは wraps の一部(symphonia/rubato のみ、cpalは無し)
//! + 必然scratchのうち mix/program + audio-clock-master の骨。

mod cache;
mod clock;
mod convert;
mod decode;
mod error;
mod meter;
mod mix;
mod program;
mod resample;
mod time_map;

pub use cache::{PcmCache, PcmFormat};
pub use clock::{
    display_frame_without_latency_compensation, drift_within_one_frame, perceptual_sample_frames,
    sample_frames_to_time, synced_display_frame, DeviceWaitLatency, PlaybackCounters,
};
pub use convert::{canonical_format, to_canonical, CANONICAL_CHANNELS, CANONICAL_SAMPLE_RATE};
pub use decode::{decode_file, decode_file_audio_ordinal, decode_stream, MAX_SAMPLES};
pub use error::{AudioError, Result};
pub use meter::{AudioMeter, ClipLatch, MeterSnapshot, CLIP_THRESHOLD};
pub use mix::{mix_audio, AudioOutOfRange, MixReport, MixSource};
pub use program::{program_from_sources, AudioProgram};
pub use resample::{source_frame_to_device, FixedRatioResampler};
pub use time_map::{TimeMap, TimeMapError};
