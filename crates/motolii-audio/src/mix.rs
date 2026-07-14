//! AG-2: 決定論的 `mix_audio`。preview/export 同一意味の正準PCM境界。

use std::sync::Arc;

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::AudioOutOfRange;

use crate::cache::PcmCache;
use crate::convert::{canonical_format, CANONICAL_CHANNELS, CANONICAL_SAMPLE_RATE};
use crate::error::{AudioError, Result};
use crate::meter::AudioMeter;

/// mixへ投入する1 source(正準48k stereo cache前提)。
#[derive(Debug, Clone)]
pub struct MixSource {
    pub pcm: Arc<PcmCache>,
    /// タイムライン上の開始。
    pub timeline_start: RationalTime,
    /// タイムライン上の尺(半開)。
    pub timeline_duration: RationalTime,
    /// clip_local → source 時刻(varispeed含む)。
    pub time_map: TimeMap,
    /// linear gain(有限・>=0)。DocParam評価後のスカラーを渡す。
    /// linear gain。Const / Keyframes(F64)をsample時刻で評価する。
    pub gain: motolii_doc::DocParam,
    pub out_of_range: AudioOutOfRange,
    pub enabled: bool,
}

impl MixSource {
    pub fn validate(&self) -> Result<()> {
        if self.pcm.format() != canonical_format() {
            return Err(AudioError::Resample {
                detail: "MixSource.pcm must be canonical 48kHz stereo",
            });
        }
        // 先頭時刻で型だけ検査(Keyframes空は拒否)。
        let _ = eval_gain_at(&self.gain, RationalTime::ZERO)?;
        if self.timeline_duration <= RationalTime::ZERO {
            return Err(AudioError::InvalidMixRange);
        }
        Ok(())
    }
}

/// `mix_audio` の結果メタ(正規silenceとunderflowの区別用)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MixReport {
    /// 出力フレーム数。
    pub frames: usize,
    /// sourceが無くgap/silenceで埋めたフレーム数(正規silence。underflowではない)。
    pub silence_frames: usize,
}

/// 正準フレーム範囲 `[start_frame, start_frame + frame_count)` を mix する。
///
/// - 評価順は呼び出し側が並べた `sources` 順(設計: Soundtrack→Track→item→component)
/// - 毎source clamp / normalize / limiter は行わない
/// - `meter` を渡してもPCM結果は変わらない
pub fn mix_audio(
    sources: &[MixSource],
    master_gain: f64,
    start_frame: u64,
    frame_count: usize,
    meter: Option<&AudioMeter>,
) -> Result<(Vec<f32>, MixReport)> {
    if !master_gain.is_finite() || master_gain < 0.0 {
        return Err(AudioError::InvalidGain { gain: master_gain });
    }
    for source in sources {
        source.validate()?;
    }

    let mut out = vec![0.0f32; frame_count.saturating_mul(CANONICAL_CHANNELS as usize)];
    if frame_count == 0 {
        return Ok((out, MixReport::default()));
    }

    let mut silence_frames = 0usize;
    for i in 0..frame_count {
        let frame_index = start_frame + i as u64;
        let timeline_t = frame_to_time(frame_index)?;
        let mut left = 0.0f64;
        let mut right = 0.0f64;
        let mut any = false;

        for source in sources {
            if !source.enabled {
                continue;
            }
            let Some((l, r)) = sample_source(source, timeline_t)? else {
                continue;
            };
            let gain = eval_gain_at(&source.gain, local_for_gain(source, timeline_t)?)?;
            any = true;
            left += l * gain;
            right += r * gain;
        }

        if !any {
            silence_frames += 1;
        }

        left *= master_gain;
        right *= master_gain;
        // 毎source / mix結果のclampはしない(AG-2 / metering契約)。
        let base = i * CANONICAL_CHANNELS as usize;
        out[base] = left as f32;
        out[base + 1] = right as f32;
    }

    if let Some(meter) = meter {
        meter.observe_interleaved_stereo(&out);
    }

    Ok((
        out,
        MixReport {
            frames: frame_count,
            silence_frames,
        },
    ))
}

fn frame_to_time(frame: u64) -> Result<RationalTime> {
    // frame / 48000。分母をレートに固定して肥大化を避ける。
    RationalTime::try_new(frame as i64, CANONICAL_SAMPLE_RATE as i64)
        .map_err(|_| AudioError::InvalidMixRange)
}

fn local_for_gain(source: &MixSource, timeline_t: RationalTime) -> Result<RationalTime> {
    timeline_t
        .try_sub(source.timeline_start)
        .map_err(|_| AudioError::InvalidMixRange)
}

fn eval_gain_at(param: &motolii_doc::DocParam, t: RationalTime) -> Result<f64> {
    use motolii_doc::{DocParam, DocValue};
    match param {
        DocParam::Const(DocValue::F64(v)) => {
            if v.is_finite() && *v >= 0.0 {
                Ok(*v)
            } else {
                Err(AudioError::InvalidGain { gain: *v })
            }
        }
        DocParam::Keyframes(track) => {
            let keys = track.keys();
            if keys.is_empty() {
                return Err(AudioError::InvalidGain { gain: f64::NAN });
            }
            if t <= keys[0].t {
                return gain_from_value(&keys[0].value);
            }
            if t >= keys[keys.len() - 1].t {
                return gain_from_value(&keys[keys.len() - 1].value);
            }
            for w in keys.windows(2) {
                if t >= w[0].t && t <= w[1].t {
                    let a = gain_from_value(&w[0].value)?;
                    let b = gain_from_value(&w[1].value)?;
                    let span = w[1]
                        .t
                        .try_sub(w[0].t)
                        .map_err(|_| AudioError::InvalidMixRange)?;
                    let delta = t.try_sub(w[0].t).map_err(|_| AudioError::InvalidMixRange)?;
                    let u = if span == RationalTime::ZERO {
                        0.0
                    } else {
                        delta.as_seconds_f64() / span.as_seconds_f64()
                    };
                    let g = a * (1.0 - u) + b * u;
                    if g.is_finite() && g >= 0.0 {
                        return Ok(g);
                    }
                    return Err(AudioError::InvalidGain { gain: g });
                }
            }
            gain_from_value(&keys[keys.len() - 1].value)
        }
        _ => Err(AudioError::InvalidGain { gain: f64::NAN }),
    }
}

fn gain_from_value(value: &motolii_doc::DocValue) -> Result<f64> {
    match value {
        motolii_doc::DocValue::F64(v) if v.is_finite() && *v >= 0.0 => Ok(*v),
        motolii_doc::DocValue::F64(v) => Err(AudioError::InvalidGain { gain: *v }),
        _ => Err(AudioError::InvalidGain { gain: f64::NAN }),
    }
}

fn sample_source(source: &MixSource, timeline_t: RationalTime) -> Result<Option<(f64, f64)>> {
    let local = match timeline_t.try_sub(source.timeline_start) {
        Ok(t) if t >= RationalTime::ZERO && t < source.timeline_duration => t,
        _ => return Ok(None),
    };
    let source_t = source
        .time_map
        .try_map(local)
        .map_err(|_| AudioError::InvalidMixRange)?;
    let src_frames = source.pcm.frame_count() as f64;
    if src_frames <= 0.0 {
        return Ok(None);
    }
    // seconds経由だと10分級でfloat丸めが乗るので、num*rate/denで直接フレーム位置へ。
    let mut pos =
        (source_t.num() as f64) * f64::from(CANONICAL_SAMPLE_RATE) / (source_t.den() as f64);
    if !(0.0..src_frames).contains(&pos) {
        match source.out_of_range {
            AudioOutOfRange::Silence => return Ok(None),
            AudioOutOfRange::Loop => {
                pos = pos.rem_euclid(src_frames);
            }
        }
    }
    Ok(Some(lerp_stereo(source.pcm.as_ref(), pos)))
}

fn lerp_stereo(pcm: &PcmCache, pos: f64) -> (f64, f64) {
    let max_index = pcm.frame_count().saturating_sub(1);
    if pcm.frame_count() == 0 {
        return (0.0, 0.0);
    }
    let i0 = (pos.floor() as u64).min(max_index);
    let i1 = (i0 + 1).min(max_index);
    let frac = (pos - i0 as f64).clamp(0.0, 1.0);
    let f0 = pcm.frame_at(i0).expect("in-range");
    let f1 = pcm.frame_at(i1).expect("in-range");
    let l = f0[0] as f64 * (1.0 - frac) + f1[0] as f64 * frac;
    let r = f0[1] as f64 * (1.0 - frac) + f1[1] as f64 * frac;
    (l, r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convert::to_canonical;
    use crate::meter::MeterSnapshot;

    fn stereo_cache(samples: Vec<f32>) -> Arc<PcmCache> {
        Arc::new(PcmCache::from_interleaved(samples, canonical_format()).expect("valid"))
    }

    fn identity_source(pcm: Arc<PcmCache>, gain: f64) -> MixSource {
        MixSource {
            pcm,
            timeline_start: RationalTime::ZERO,
            timeline_duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: TimeMap::IDENTITY,
            gain: motolii_doc::DocParam::const_f64(gain),
            out_of_range: AudioOutOfRange::Silence,
            enabled: true,
        }
    }

    #[test]
    fn two_sources_sum_deterministically() {
        // 1 frame: [0.25, 0.5] + [0.5, 0.25] = [0.75, 0.75]
        let a = stereo_cache(vec![0.25, 0.5]);
        let b = stereo_cache(vec![0.5, 0.25]);
        let (out, report) = mix_audio(
            &[identity_source(a, 1.0), identity_source(b, 1.0)],
            1.0,
            0,
            1,
            None,
        )
        .unwrap();
        assert_eq!(out, vec![0.75, 0.75]);
        assert_eq!(report.silence_frames, 0);
    }

    #[test]
    fn master_gain_applies_last_without_clamp() {
        let a = stereo_cache(vec![0.8, 0.8]);
        let (out, _) = mix_audio(&[identity_source(a, 1.0)], 2.0, 0, 1, None).unwrap();
        assert_eq!(out, vec![1.6, 1.6]);
    }

    #[test]
    fn gap_is_silence_not_underflow_counter() {
        let a = stereo_cache(vec![1.0, 1.0]);
        let mut source = identity_source(a, 1.0);
        source.timeline_start = RationalTime::try_new(1, CANONICAL_SAMPLE_RATE as i64).unwrap();
        source.timeline_duration = RationalTime::try_new(1, CANONICAL_SAMPLE_RATE as i64).unwrap();
        let (out, report) = mix_audio(&[source], 1.0, 0, 2, None).unwrap();
        assert_eq!(&out[..2], &[0.0, 0.0]);
        assert_eq!(&out[2..], &[1.0, 1.0]);
        assert_eq!(report.silence_frames, 1);
    }

    #[test]
    fn out_of_range_loop_wraps() {
        let a = stereo_cache(vec![0.1, 0.2, 0.3, 0.4]); // 2 frames
        let mut source = identity_source(a, 1.0);
        source.timeline_duration = RationalTime::try_new(4, CANONICAL_SAMPLE_RATE as i64).unwrap();
        source.out_of_range = AudioOutOfRange::Loop;
        // speed 1, source frames 0,1,0,1
        let (out, _) = mix_audio(&[source], 1.0, 0, 4, None).unwrap();
        assert_eq!(&out[0..2], &[0.1, 0.2]);
        assert_eq!(&out[2..4], &[0.3, 0.4]);
        assert_eq!(&out[4..6], &[0.1, 0.2]);
        assert_eq!(&out[6..8], &[0.3, 0.4]);
    }

    #[test]
    fn metering_does_not_change_pcm() {
        let a = stereo_cache(vec![1.5, -1.25]);
        let meter = AudioMeter::new();
        let (with_m, _) =
            mix_audio(&[identity_source(a.clone(), 1.0)], 1.0, 0, 1, Some(&meter)).unwrap();
        let (without_m, _) = mix_audio(&[identity_source(a, 1.0)], 1.0, 0, 1, None).unwrap();
        assert_eq!(with_m, without_m);
        assert_eq!(
            meter.snapshot(),
            MeterSnapshot {
                peak_l: 1.5,
                peak_r: 1.25,
                clipped: true,
            }
        );
    }

    #[test]
    fn mono_44100_and_stereo_48000_mix() {
        let mono_441 = PcmCache::from_interleaved(
            vec![0.2; 441], // 0.01s @ 44100
            crate::cache::PcmFormat {
                channels: 1,
                sample_rate: 44_100,
            },
        )
        .unwrap();
        let stereo_48 = PcmCache::from_interleaved(
            vec![0.1, -0.1].repeat(480), // 0.01s @ 48000
            canonical_format(),
        )
        .unwrap();
        let a = Arc::new(to_canonical(&mono_441).unwrap());
        let b = Arc::new(to_canonical(&stereo_48).unwrap());
        let duration = RationalTime::try_new(1, 100).unwrap(); // 0.01s
        let sources = [
            MixSource {
                pcm: a,
                timeline_start: RationalTime::ZERO,
                timeline_duration: duration,
                time_map: TimeMap::IDENTITY,
                gain: motolii_doc::DocParam::const_f64(1.0),
                out_of_range: AudioOutOfRange::Silence,
                enabled: true,
            },
            MixSource {
                pcm: b,
                timeline_start: RationalTime::ZERO,
                timeline_duration: duration,
                time_map: TimeMap::IDENTITY,
                gain: motolii_doc::DocParam::const_f64(1.0),
                out_of_range: AudioOutOfRange::Silence,
                enabled: true,
            },
        ];
        let frames = 480; // 0.01s @ 48k
        let (out, _) = mix_audio(&sources, 1.0, 0, frames, None).unwrap();
        // mono 0.2→L=R + stereo 0.1/-0.1 ≈ 0.3 / 0.1 (resample誤差あり)
        assert!(out[0] > 0.25 && out[0] < 0.35);
        assert!(out[1] > 0.05 && out[1] < 0.15);
    }

    #[test]
    fn varispeed_doubles_source_advance() {
        // 4 source frames of distinct L values.
        let pcm = stereo_cache(vec![0.0, 0.0, 0.25, 0.0, 0.5, 0.0, 0.75, 0.0]);
        let mut source = identity_source(pcm, 1.0);
        source.time_map = TimeMap::constant_speed(RationalTime::ZERO, 2, 1).unwrap();
        source.timeline_duration = RationalTime::try_new(2, CANONICAL_SAMPLE_RATE as i64).unwrap();
        let (out, _) = mix_audio(&[source], 1.0, 0, 2, None).unwrap();
        // t=0 → source 0, t=1/48000 → source 2/48000 (speed 2)
        assert_eq!(out[0], 0.0);
        assert_eq!(out[2], 0.5);
    }

    #[test]
    fn chunk_size_rebuild_matches_whole() {
        let a = stereo_cache(
            (0..20)
                .flat_map(|i| [i as f32 * 0.01, -(i as f32) * 0.01])
                .collect(),
        );
        let sources = [identity_source(a, 0.5)];
        let (whole, _) = mix_audio(&sources, 1.0, 0, 10, None).unwrap();
        let mut rebuilt = Vec::new();
        for (start, count) in [(0u64, 3usize), (3, 3), (6, 3), (9, 1)] {
            let (chunk, _) = mix_audio(&sources, 1.0, start, count, None).unwrap();
            rebuilt.extend_from_slice(&chunk);
        }
        assert_eq!(whole, rebuilt);
    }

    #[test]
    fn ten_minute_timeline_frame_maps_without_drift() {
        // 10分 = 28_800_000 frames @48k。全展開せず末尾付近の既知サンプル対応だけ審判する。
        let ten_min = 10u64 * 60 * u64::from(CANONICAL_SAMPLE_RATE);
        let pcm_frames = 64u64;
        let mut samples = Vec::with_capacity(pcm_frames as usize * 2);
        for i in 0..pcm_frames {
            samples.push(i as f32 * 0.01);
            samples.push(-(i as f32) * 0.01);
        }
        let source = MixSource {
            pcm: stereo_cache(samples),
            timeline_start: RationalTime::try_new(
                (ten_min - 32) as i64,
                CANONICAL_SAMPLE_RATE as i64,
            )
            .unwrap(),
            timeline_duration: RationalTime::try_new(
                pcm_frames as i64,
                CANONICAL_SAMPLE_RATE as i64,
            )
            .unwrap(),
            time_map: TimeMap::IDENTITY,
            gain: motolii_doc::DocParam::const_f64(1.0),
            out_of_range: AudioOutOfRange::Silence,
            enabled: true,
        };
        let (out, _) = mix_audio(&[source], 1.0, ten_min - 32, 4, None).unwrap();
        assert_eq!(&out[0..2], &[0.0, 0.0]);
        assert_eq!(&out[2..4], &[0.01, -0.01]);
        assert_eq!(&out[4..6], &[0.02, -0.02]);
        assert_eq!(&out[6..8], &[0.03, -0.03]);
    }
}
