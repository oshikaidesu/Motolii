//! テスト用の最小WAV(PCM16)フィクスチャ生成。統合テスト間で共有する
//! (AGENTS.md「新規ヘルパーを書く前に既存を検索する」— 1箇所に集約し複製しない)。
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

/// `frames`フレーム分の`channels`チャンネル440Hz近傍サイン波(インターリーブPCM16)。
/// 全チャンネルへ同一波形+チャンネル番号だけ振幅を変えて、デコード後に
/// チャンネル配置が保たれているかを検証しやすくする。
pub fn sine_wave_i16(sample_rate: u32, frames: usize, freq_hz: f32, channels: u16) -> Vec<i16> {
    let mut samples = Vec::with_capacity(frames * channels as usize);
    for i in 0..frames {
        let t = i as f32 / sample_rate as f32;
        for ch in 0..channels {
            let amp = 0.5 - 0.1 * ch as f32;
            let v = (t * freq_hz * std::f32::consts::TAU).sin() * amp;
            samples.push((v * i16::MAX as f32) as i16);
        }
    }
    samples
}
