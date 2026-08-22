//! owns: 音声の意味核 — 決定論的mix・Documentからのprogram組み立て・音声クロック・
//! 実デバイス再生。
//!
//! 音声束・第1切片(2026-08-20)+ 実時間再生 第1切片 A1(2026-08-21、音声クロック核の
//! 移植)+ 実時間再生 第2切片 A2(2026-08-21、device/producer 結線)。旧
//! `crates/motolii-audio`(4,286行)+ 旧 `crates/motolii-transport`(音声クロック+
//! 再生セッション部分)からの移植であって再実装ではない。KNOWN.md
//! 「音声(2026-08-20解析済み)」が調達を確定済み: rodio/kira は採らず cpal 直叩き、
//! mux は `motolii-media` が解決済み(この crate は PCM を作るだけでよい)。
//!
//! **範囲**:
//! - [`mix`] — 決定論的 `mix_audio`(preview/export同一意味の正準PCM境界)。
//!   B42(音声内容整形束、2026-08-22)でレイヤー単位の pan([`FadeSpec`]と対の
//!   `MixSource::pan`)・fade in/out([`FadeSpec`]/[`FadeCurve`])・
//!   正規化([`normalize_gain_for_peak`])を追加(gain/muteは既存の
//!   `MixSource::gain`/`enabled` がそのままカバー、詳細は `mix.rs` モジュールdoc)
//! - [`program`] — `motolii_store::StoreView` から `AudioProgram`(MixSource列)を組む
//! - [`clock`] — 音声クロック([`PlaybackCounters`]/[`DeviceWaitLatency`]の骨 +
//!   start/pause/seekの状態機械 [`PlaybackClock`]。供給済みフレーム − デバイス待ち、
//!   wall-clock不使用)
//! - [`ring`] — `rtrb`ベースのコールバック充填([`fill_or_silence`]。旧`ring.rs`の
//!   自前SPSC367行は再発明としてKNOWN.mdで不採用済み、フレーム充填/無音補填の
//!   ロジックだけを`rtrb::Consumer`版として移した)
//! - [`device`](A2) — cpal 出力の実結線(`NegotiatedOutput`/`OutputStream`/
//!   `negotiate_output`)。**このモジュールだけがハードウェアに触る**(D4契約)
//! - [`producer`](A2) — `AudioProgram::mix_audio` をバックグラウンドスレッドで
//!   実行し `rtrb` リングへ供給する `MixProducer`(再生中の seek 対応込み)
//! - [`session`](A2) — [`device`]+[`producer`]+[`PlaybackClock`] を束ねる
//!   [`PlaybackSession`](旧 `crates/motolii-transport::PlaybackSession` の形の移植)
//! - [`cache`]/[`convert`]/[`decode`]/[`resample`] — 上記が要る正準PCM表現一式
//!   (symphonia/rubato、KNOWN.md で調達確定済み)
//! - [`time_map`] — クリップローカル→ソース時刻の写像(`next/core/motolii-core` が
//!   落とした型をこの crate が持つ。詳細はモジュール doc)
//!
//! **持ってこなかったもの**: 旧 `motolii-transport` の `Transport`/
//! `DrsController`/`PreviewSimulator` — 映像フレームドロップ・適応解像度・
//! `FramePlan`は音声クロックと別の関心事で、この束の対象外(`PlaybackClock`は
//! Transportの音声クロック部分だけを継承する)。
//!
//! 旧クレートの内訳(KNOWN.md): wraps 743(symphonia/rubato/cpal)/ 必然scratch 1,451
//! (mix/producer/program)/ 再発明 367(`ring.rs` — 上流 `rtrb` が既にある)/
//! テスト 1,194。この束が移したのは wraps の一部(symphonia/rubato/cpal)+
//! 必然scratchのうち mix/program/producer/device + audio-clock-master の骨。

mod cache;
mod clock;
mod convert;
mod decode;
mod device;
mod error;
mod meter;
mod mix;
mod producer;
mod program;
mod resample;
mod ring;
mod session;
mod time_map;

pub use cache::{PcmCache, PcmFormat};
pub use clock::{
    display_frame_without_latency_compensation, drift_within_one_frame, perceptual_sample_frames,
    sample_frames_to_time, synced_display_frame, DeviceWaitLatency, PlaybackClock,
    PlaybackCounters,
};
pub use convert::{
    canonical_format, time_to_canonical_frames, to_canonical, CANONICAL_CHANNELS,
    CANONICAL_SAMPLE_RATE,
};
pub use decode::{decode_file, decode_file_audio_ordinal, decode_stream, MAX_SAMPLES};
pub use device::{negotiate_output, select_device_sample_rate, NegotiatedOutput, OutputStream};
pub use error::{AudioError, Result};
pub use meter::{AudioMeter, ClipLatch, MeterSnapshot, CLIP_THRESHOLD};
pub use mix::{
    mix_audio, normalize_gain_for_peak, AudioOutOfRange, FadeCurve, FadeSpec, MixReport, MixSource,
};
pub use producer::MixProducer;
pub use program::{program_from_sources, AudioProgram};
pub use resample::{source_frame_to_device, FixedRatioResampler};
pub use ring::fill_or_silence;
pub use session::PlaybackSession;
pub use time_map::{TimeMap, TimeMapError};
