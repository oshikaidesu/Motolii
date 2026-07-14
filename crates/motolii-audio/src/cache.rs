//! 楽曲/stream 1本のインターリーブf32 PCM全展開キャッシュ(D4 + AG-2)。
//!
//! AG-2のmixerは本キャッシュをsourceとして読む。単一Soundtrack時代の
//! 「再生位置→サンプル添字」契約はsource単位で維持し、複数sourceの加算は`mix`へ。

use crate::error::{AudioError, Result};

/// PCMの形式(チャンネル数・サンプルレート)。stream単位で単一。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcmFormat {
    pub channels: u16,
    pub sample_rate: u32,
}

/// stream 1本のインターリーブf32 PCM。デコード時に全展開してRAM保持する
/// (docs/specs/M2-document-model.md「音声トランスポート設計」5.: 5分ステレオ48kHzで≈110MB)。
///
/// source単位の「再生位置→サンプル」は本構造体の添字計算。複数sourceの加算は`mix`。
#[derive(Debug, Clone)]
pub struct PcmCache {
    samples: Vec<f32>,
    format: PcmFormat,
    frame_count: u64,
}

impl PcmCache {
    /// インターリーブf32バッファから構築する。境界検査済みの`read_frames`/`frame_at`の
    /// 前提(フレーム整合・非ゼロchannels/sample_rate)をここで確定する。
    pub fn from_interleaved(samples: Vec<f32>, format: PcmFormat) -> Result<Self> {
        if format.channels == 0 {
            return Err(AudioError::UnsupportedChannels { channels: 0 });
        }
        if format.sample_rate == 0 {
            return Err(AudioError::UnsupportedSampleRate { sample_rate: 0 });
        }
        let channels = format.channels as usize;
        if !samples.len().is_multiple_of(channels) {
            return Err(AudioError::MisalignedSamples {
                len: samples.len(),
                channels: format.channels,
            });
        }
        let frame_count = (samples.len() / channels) as u64;
        Ok(Self {
            samples,
            format,
            frame_count,
        })
    }

    pub fn format(&self) -> PcmFormat {
        self.format
    }

    /// 楽曲全体のフレーム数(1フレーム=全チャンネル分のサンプル1組)。
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// `start_frame`から`frame_count`フレーム分のインターリーブサンプルを読む。
    /// 範囲外は`AudioError::OutOfRange`(D4完了条件: 境界検査付きread)。
    pub fn read_frames(&self, start_frame: u64, frame_count: usize) -> Result<&[f32]> {
        let end_frame = start_frame
            .checked_add(frame_count as u64)
            .filter(|end| *end <= self.frame_count)
            .ok_or(AudioError::OutOfRange {
                start: start_frame,
                requested: frame_count,
                total: self.frame_count,
            })?;
        let channels = self.format.channels as usize;
        let start = start_frame as usize * channels;
        let end = end_frame as usize * channels;
        Ok(&self.samples[start..end])
    }

    /// 単一フレーム(全チャンネルのサンプル)を読む。`read_frames(idx, 1)`の糖衣。
    pub fn frame_at(&self, frame_index: u64) -> Result<&[f32]> {
        self.read_frames(frame_index, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stereo(frames: usize) -> PcmCache {
        let mut samples = Vec::with_capacity(frames * 2);
        for i in 0..frames {
            samples.push(i as f32);
            samples.push(-(i as f32));
        }
        PcmCache::from_interleaved(
            samples,
            PcmFormat {
                channels: 2,
                sample_rate: 48_000,
            },
        )
        .expect("valid cache")
    }

    #[test]
    fn rejects_zero_channels() {
        let err = PcmCache::from_interleaved(
            vec![0.0],
            PcmFormat {
                channels: 0,
                sample_rate: 48_000,
            },
        )
        .unwrap_err();
        assert!(matches!(
            err,
            AudioError::UnsupportedChannels { channels: 0 }
        ));
    }

    #[test]
    fn rejects_zero_sample_rate() {
        let err = PcmCache::from_interleaved(
            vec![0.0],
            PcmFormat {
                channels: 1,
                sample_rate: 0,
            },
        )
        .unwrap_err();
        assert!(matches!(
            err,
            AudioError::UnsupportedSampleRate { sample_rate: 0 }
        ));
    }

    #[test]
    fn rejects_misaligned_buffer() {
        let err = PcmCache::from_interleaved(
            vec![0.0, 1.0, 2.0],
            PcmFormat {
                channels: 2,
                sample_rate: 48_000,
            },
        )
        .unwrap_err();
        assert!(matches!(
            err,
            AudioError::MisalignedSamples {
                len: 3,
                channels: 2
            }
        ));
    }

    #[test]
    fn frame_at_arbitrary_positions_matches_direct_index() {
        let cache = stereo(1_000);
        for idx in [0u64, 1, 499, 500, 999] {
            let frame = cache.frame_at(idx).expect("in-range frame");
            assert_eq!(frame, [idx as f32, -(idx as f32)]);
        }
    }

    #[test]
    fn read_frames_returns_contiguous_slice() {
        let cache = stereo(10);
        let chunk = cache.read_frames(3, 4).expect("in-range chunk");
        assert_eq!(chunk, [3.0, -3.0, 4.0, -4.0, 5.0, -5.0, 6.0, -6.0]);
    }

    #[test]
    fn read_frames_out_of_range_is_typed_error_not_panic() {
        let cache = stereo(10);
        let err = cache.read_frames(8, 5).unwrap_err();
        match err {
            AudioError::OutOfRange {
                start: 8,
                requested: 5,
                total: 10,
            } => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn read_frames_at_exact_end_is_ok_when_empty() {
        let cache = stereo(10);
        assert_eq!(
            cache.read_frames(10, 0).expect("empty tail read"),
            &[] as &[f32]
        );
    }

    #[test]
    fn frame_at_end_is_out_of_range() {
        let cache = stereo(10);
        assert!(cache.frame_at(10).is_err());
    }
}
