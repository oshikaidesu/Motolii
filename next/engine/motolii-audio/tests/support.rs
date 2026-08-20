//! テスト用の最小WAV(PCM16)フィクスチャ生成。統合テスト間で共有する。
//! 旧 `crates/motolii-audio/tests/support.rs` から無改造で移植。
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;

/// `channels`チャンネル・`sample_rate`のPCM16 WAVを書く。`samples`はインターリーブ済み。
pub fn write_pcm16_wav(path: &Path, sample_rate: u32, channels: u16, samples: &[i16]) {
    let data_bytes = (samples.len() * 2) as u32;
    let block_align = 2u16 * channels;
    let byte_rate = sample_rate * block_align as u32;
    let bits = 16u16;

    let mut wav = Vec::new();
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_bytes).to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_bytes.to_le_bytes());
    for s in samples {
        wav.extend_from_slice(&s.to_le_bytes());
    }
    std::fs::write(path, wav).expect("write test wav");
}
