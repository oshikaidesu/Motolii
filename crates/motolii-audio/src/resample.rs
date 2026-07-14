//! 固定比リサンプル(D4-FU: デバイスレート≠素材レート時のみ)。
//!
//! アルゴリズム遅延は先頭 trim で吸収し、Transport 境界へ持ち出さない
//! (docs/specs/M2-document-model.md 音声トランスポート 6. / D4-FU行)。

use rubato::{FftFixedIn, Resampler};

use crate::error::{AudioError, Result};

/// プロデューサが1回に rubato へ渡す固定入力チャンク長。
const CHUNK_FRAMES: usize = 1024;

/// 素材レート→デバイスレートの固定比リサンプラ。
///
/// レート一致時は作らない(恒等パス)。構築後はリング書き込み前のプロデューサ専用。
pub struct FixedRatioResampler {
    inner: FftFixedIn<f32>,
    channels: usize,
    /// まだ捨てるべき出力フレーム数(`output_delay`の残り)。
    frames_to_trim: usize,
    planar_in: Vec<Vec<f32>>,
    planar_out: Vec<Vec<f32>>,
    interleaved: Vec<f32>,
}

impl FixedRatioResampler {
    /// `source_rate != device_rate` のときだけ成功する。一致時は呼び出し側が恒等パスを選ぶ。
    pub fn new(source_rate: u32, device_rate: u32, channels: u16) -> Result<Self> {
        if channels == 0 {
            return Err(AudioError::UnsupportedChannels { channels: 0 });
        }
        if source_rate == 0 {
            return Err(AudioError::UnsupportedSampleRate {
                sample_rate: source_rate,
            });
        }
        if device_rate == 0 {
            return Err(AudioError::UnsupportedSampleRate {
                sample_rate: device_rate,
            });
        }
        if source_rate == device_rate {
            return Err(AudioError::Resample {
                detail: "fixed-ratio resampler must not be built when rates already match",
            });
        }
        let ch = channels as usize;
        let inner = FftFixedIn::<f32>::new(
            source_rate as usize,
            device_rate as usize,
            CHUNK_FRAMES,
            2,
            ch,
        )
        .map_err(|_| AudioError::Resample {
            detail: "failed to construct fixed-ratio FFT resampler",
        })?;

        let delay = inner.output_delay();
        let planar_in = inner.input_buffer_allocate(true);
        let planar_out = inner.output_buffer_allocate(true);
        Ok(Self {
            inner,
            channels: ch,
            frames_to_trim: delay,
            planar_in,
            planar_out,
            interleaved: Vec::new(),
        })
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    pub fn input_frames_next(&self) -> usize {
        self.inner.input_frames_next()
    }

    pub fn output_delay(&self) -> usize {
        self.inner.output_delay()
    }

    /// シーク/再起動時: 内部状態と先頭 trim を初期化する。
    pub fn reset(&mut self) {
        self.inner.reset();
        self.frames_to_trim = self.inner.output_delay();
        self.interleaved.clear();
    }

    /// インターリーブ入力を1チャンク処理し、trim 後のインターリーブ出力を返す。
    ///
    /// `input` のフレーム数は `input_frames_next()` と一致している必要がある。
    pub fn process_interleaved(&mut self, input: &[f32]) -> Result<&[f32]> {
        let need = self.inner.input_frames_next();
        let frames = input.len() / self.channels;
        if input.len() % self.channels != 0 || frames != need {
            return Err(AudioError::Resample {
                detail: "input frame count must equal input_frames_next()",
            });
        }
        self.deinterleave_into(input, frames);
        self.inner
            .process_into_buffer(&self.planar_in, &mut self.planar_out, None)
            .map_err(|_| AudioError::Resample {
                detail: "rubato process_into_buffer failed",
            })?;
        self.interleave_trimmed()
    }

    /// トラック終端など、必要フレーム未満の残りを流し込む。
    pub fn process_partial_interleaved(&mut self, input: &[f32]) -> Result<&[f32]> {
        if input.len() % self.channels != 0 {
            return Err(AudioError::MisalignedSamples {
                len: input.len(),
                channels: self.channels as u16,
            });
        }
        let frames = input.len() / self.channels;
        self.deinterleave_into(input, frames);
        // process_partial は可変長入力を受け、内部バッファを吐き出す。
        let out = self
            .inner
            .process_partial(Some(&self.planar_in), None)
            .map_err(|_| AudioError::Resample {
                detail: "rubato process_partial failed",
            })?;
        self.planar_out = out;
        self.interleave_trimmed()
    }

    /// 無音/空入力で遅延分を吐き切る(終端フラッシュ)。
    pub fn flush_silence_chunk(&mut self) -> Result<&[f32]> {
        let empty: Option<&[Vec<f32>]> = None;
        let out = self
            .inner
            .process_partial(empty, None)
            .map_err(|_| AudioError::Resample {
                detail: "rubato flush process_partial failed",
            })?;
        self.planar_out = out;
        self.interleave_trimmed()
    }

    fn deinterleave_into(&mut self, input: &[f32], frames: usize) {
        for ch in 0..self.channels {
            self.planar_in[ch].resize(frames, 0.0);
            for f in 0..frames {
                self.planar_in[ch][f] = input[f * self.channels + ch];
            }
        }
    }

    fn interleave_trimmed(&mut self) -> Result<&[f32]> {
        let out_frames = self.planar_out.first().map(|c| c.len()).unwrap_or(0);
        for ch in &self.planar_out {
            if ch.len() != out_frames {
                return Err(AudioError::Resample {
                    detail: "planar output channel lengths diverged",
                });
            }
        }

        let skip = self.frames_to_trim.min(out_frames);
        self.frames_to_trim -= skip;
        let keep = out_frames - skip;
        self.interleaved.resize(keep * self.channels, 0.0);
        for f in 0..keep {
            let src_f = f + skip;
            for ch in 0..self.channels {
                self.interleaved[f * self.channels + ch] = self.planar_out[ch][src_f];
            }
        }
        Ok(&self.interleaved)
    }
}

/// ソースフレーム番号をデバイス側フレーム番号へ写す(整数比の切り捨て)。
///
/// 完了条件のインパルス対応検査で、期待出力位置の正本にする。
pub fn source_frame_to_device(source_frame: u64, source_rate: u32, device_rate: u32) -> u64 {
    if source_rate == 0 {
        return 0;
    }
    (source_frame as u128)
        .saturating_mul(device_rate as u128)
        .checked_div(source_rate as u128)
        .unwrap_or(0) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_matching_rates() {
        assert!(FixedRatioResampler::new(48_000, 48_000, 1).is_err());
    }

    #[test]
    fn impulse_at_origin_lands_on_expected_device_frame() {
        let src_rate = 44_100u32;
        let dst_rate = 48_000u32;
        let mut rs = FixedRatioResampler::new(src_rate, dst_rate, 1).unwrap();
        let impulse_at = 0u64;
        let total = 8_000usize;
        let mut src = vec![0.0f32; total];
        src[impulse_at as usize] = 1.0;

        let mut out = Vec::new();
        let mut pos = 0usize;
        while pos < total {
            let need = rs.input_frames_next();
            if pos + need <= total {
                let chunk = &src[pos..pos + need];
                pos += need;
                out.extend_from_slice(rs.process_interleaved(chunk).unwrap());
            } else {
                let chunk = &src[pos..];
                pos = total;
                out.extend_from_slice(rs.process_partial_interleaved(chunk).unwrap());
            }
        }
        for _ in 0..4 {
            out.extend_from_slice(rs.flush_silence_chunk().unwrap());
        }

        let (peak_idx, _) = out
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap();
        let expected = source_frame_to_device(impulse_at, src_rate, dst_rate) as usize;
        assert!(
            (peak_idx as isize - expected as isize).unsigned_abs() <= 1,
            "peak={peak_idx} expected≈{expected}"
        );
    }
}
