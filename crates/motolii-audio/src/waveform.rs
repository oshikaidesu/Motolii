//! AG-3: PCMキャッシュからUI表示用の波形ピークを作る。

use crate::{AudioError, PcmCache};

/// PCM全体を等幅bucketへ分割し、各bucketの全channel絶対値peakを返す。
///
/// UI用の単一波形なので、L専用ではなくframe内全channelのmax(abs)を集約する。
pub fn waveform_peaks(pcm: &PcmCache, bucket_count: usize) -> Result<Vec<f32>, AudioError> {
    if bucket_count == 0 {
        return Ok(Vec::new());
    }

    let frames = pcm.frame_count();
    let channels = usize::from(pcm.format().channels);
    let mut peaks = Vec::with_capacity(bucket_count);
    for bucket in 0..bucket_count {
        let start = ((bucket as u128 * frames as u128) / bucket_count as u128) as u64;
        let end = (((bucket + 1) as u128 * frames as u128) / bucket_count as u128) as u64;
        let samples = pcm.read_frames(start, (end - start) as usize)?;
        let mut peak = 0.0f32;
        for frame in samples.chunks_exact(channels) {
            for sample in frame {
                peak = peak.max(sample.abs());
            }
        }
        peaks.push(peak);
    }
    Ok(peaks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PcmFormat;

    #[test]
    fn returns_peak_for_each_evenly_divided_bucket() {
        let pcm = PcmCache::from_interleaved(
            vec![0.25, -0.25, 0.5, -0.5, 1.0, -1.0, 0.75, -0.75],
            PcmFormat {
                channels: 2,
                sample_rate: 48_000,
            },
        )
        .expect("valid PCM");

        assert_eq!(waveform_peaks(&pcm, 2).expect("peaks"), [0.5, 1.0]);
    }

    #[test]
    fn right_only_stereo_is_visible() {
        // L=0 / R=peak — L専用読みだと無音波形になってしまう欠落の回帰防止。
        let pcm = PcmCache::from_interleaved(
            vec![0.0, 0.4, 0.0, 0.8, 0.0, 0.2, 0.0, 1.0],
            PcmFormat {
                channels: 2,
                sample_rate: 48_000,
            },
        )
        .expect("valid PCM");
        assert_eq!(waveform_peaks(&pcm, 2).expect("peaks"), [0.8, 1.0]);
    }

    #[test]
    fn mono_peaks_match_channel_abs() {
        let pcm = PcmCache::from_interleaved(
            vec![0.1, -0.5, 0.25, -0.9],
            PcmFormat {
                channels: 1,
                sample_rate: 48_000,
            },
        )
        .expect("valid PCM");
        assert_eq!(waveform_peaks(&pcm, 2).expect("peaks"), [0.5, 0.9]);
    }
}
