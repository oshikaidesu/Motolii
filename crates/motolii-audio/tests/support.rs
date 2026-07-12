//! テスト用の最小WAV(PCM16 mono)を生成する。
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub fn write_pcm16_mono_wav(path: &std::path::Path, sample_rate: u32, samples: &[i16]) {
    let data_bytes = (samples.len() * 2) as u32;
    let byte_rate = sample_rate * 2;
    let block_align = 2u16;
    let bits = 16u16;

    let mut wav = Vec::new();
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_bytes).to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // mono
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

/// 440Hzの短いサイン波(1秒・48kHz mono)。
pub fn fixture_sine_1s() -> (u32, Vec<i16>) {
    let rate = 48_000u32;
    let frames = rate as usize;
    let mut samples = Vec::with_capacity(frames);
    for i in 0..frames {
        let t = i as f32 / rate as f32;
        let v = (t * 440.0 * std::f32::consts::TAU).sin();
        samples.push((v * i16::MAX as f32 * 0.5) as i16);
    }
    (rate, samples)
}
