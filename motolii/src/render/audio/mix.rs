
use std::sync::Arc;

use motolii_core::RationalTime;
use motolii_eval::{KeyframeTrack, Value};

use crate::render::audio::cache::PcmCache;
use crate::render::audio::convert::{canonical_format, CANONICAL_CHANNELS, CANONICAL_SAMPLE_RATE};
use crate::render::audio::error::{AudioError, Result};
use crate::render::audio::meter::AudioMeter;
use crate::render::audio::time_map::TimeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioOutOfRange {
    #[default]
    Silence,
    Loop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FadeCurve {
    Linear,
    #[default]
    EqualPower,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FadeSpec {
    pub fade_in: RationalTime,
    pub fade_out: RationalTime,
    pub curve: FadeCurve,
}

impl FadeSpec {
    pub const NONE: FadeSpec = FadeSpec {
        fade_in: RationalTime::ZERO,
        fade_out: RationalTime::ZERO,
        curve: FadeCurve::EqualPower,
    };
}

impl Default for FadeSpec {
    fn default() -> Self {
        Self::NONE
    }
}

#[derive(Debug, Clone)]
pub struct MixSource {
    pub pcm: Arc<PcmCache>,
    pub timeline_start: RationalTime,
    pub timeline_duration: RationalTime,
    pub time_map: TimeMap,
    pub gain: Option<KeyframeTrack>,
    pub pan: Option<KeyframeTrack>,
    pub fade: FadeSpec,
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
        let _ = eval_gain_at(&self.gain, RationalTime::ZERO)?;
        let _ = eval_pan_at(&self.pan, RationalTime::ZERO)?;
        if self.timeline_duration <= RationalTime::ZERO {
            return Err(AudioError::InvalidMixRange);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MixReport {
    pub frames: usize,
    pub silence_frames: usize,
}

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
            let local = local_for_gain(source, timeline_t)?;
            let gain = eval_gain_at(&source.gain, local)?;
            let envelope = fade_envelope(local, source.timeline_duration, &source.fade);
            let pan = eval_pan_at(&source.pan, local)?;
            any = true;
            let (pl, pr) = apply_pan_stereo(l * gain * envelope, r * gain * envelope, pan);
            left += pl;
            right += pr;
        }

        if !any {
            silence_frames += 1;
        }

        left *= master_gain;
        right *= master_gain;
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
    RationalTime::try_new(frame as i64, CANONICAL_SAMPLE_RATE as i64)
        .map_err(|_| AudioError::InvalidMixRange)
}

fn local_for_gain(source: &MixSource, timeline_t: RationalTime) -> Result<RationalTime> {
    timeline_t
        .try_sub(source.timeline_start)
        .map_err(|_| AudioError::InvalidMixRange)
}

fn eval_gain_at(gain: &Option<KeyframeTrack>, t: RationalTime) -> Result<f64> {
    let raw = match gain {
        None => 1.0,
        Some(track) => match track.eval(t) {
            Value::F64(v) => v,
            _ => return Err(AudioError::InvalidGain { gain: f64::NAN }),
        },
    };
    if raw.is_finite() && raw >= 0.0 {
        Ok(raw)
    } else {
        Err(AudioError::InvalidGain { gain: raw })
    }
}

fn eval_pan_at(pan: &Option<KeyframeTrack>, t: RationalTime) -> Result<f64> {
    let raw = match pan {
        None => 0.0,
        Some(track) => match track.eval(t) {
            Value::F64(v) => v,
            _ => return Err(AudioError::InvalidPan { pan: f64::NAN }),
        },
    };
    if raw.is_finite() {
        Ok(raw)
    } else {
        Err(AudioError::InvalidPan { pan: raw })
    }
}

fn apply_pan_stereo(left: f64, right: f64, pan: f64) -> (f64, f64) {
    let pan = pan.clamp(-1.0, 1.0);
    let x = if pan <= 0.0 { pan + 1.0 } else { pan };
    let gain_l = (x * std::f64::consts::FRAC_PI_2).cos();
    let gain_r = (x * std::f64::consts::FRAC_PI_2).sin();
    if pan <= 0.0 {
        (left + right * gain_l, right * gain_r)
    } else {
        (left * gain_l, right + left * gain_r)
    }
}

fn fade_envelope(local: RationalTime, duration: RationalTime, fade: &FadeSpec) -> f64 {
    let mut envelope = 1.0;

    if fade.fade_in > RationalTime::ZERO {
        let in_len = fade.fade_in.min(duration);
        if local < in_len {
            envelope *= curve_value(ratio_unit(local, in_len), fade.curve);
        }
    }

    if fade.fade_out > RationalTime::ZERO {
        let out_len = fade.fade_out.min(duration);
        if let Ok(out_start) = duration.try_sub(out_len) {
            if local > out_start {
                if let Ok(remaining) = duration.try_sub(local) {
                    envelope *= curve_value(ratio_unit(remaining, out_len), fade.curve);
                } else {
                    envelope = 0.0;
                }
            }
        }
    }

    envelope
}

fn ratio_unit(numerator: RationalTime, denominator: RationalTime) -> f64 {
    if denominator <= RationalTime::ZERO {
        return 1.0;
    }
    (numerator.as_seconds_f64() / denominator.as_seconds_f64()).clamp(0.0, 1.0)
}

fn curve_value(t: f64, curve: FadeCurve) -> f64 {
    match curve {
        FadeCurve::Linear => t,
        FadeCurve::EqualPower => (t * std::f64::consts::FRAC_PI_2).sin(),
    }
}

pub fn normalize_gain_for_peak(pcm: &PcmCache, target_peak: f64) -> Result<f64> {
    if !target_peak.is_finite() || target_peak < 0.0 {
        return Err(AudioError::InvalidGain { gain: target_peak });
    }
    let frame_count = pcm.frame_count();
    let mut peak = 0.0f64;
    if frame_count > 0 {
        let samples = pcm.read_frames(0, frame_count as usize)?;
        for &sample in samples {
            peak = peak.max(sample.abs() as f64);
        }
    }
    if peak <= 0.0 {
        Ok(1.0)
    } else {
        Ok(target_peak / peak)
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
    use crate::render::audio::convert::to_canonical;
    use crate::render::audio::meter::MeterSnapshot;
    use motolii_eval::{Interp, Keyframe};

    fn stereo_cache(samples: Vec<f32>) -> Arc<PcmCache> {
        Arc::new(PcmCache::from_interleaved(samples, canonical_format()).expect("valid"))
    }

    fn identity_source(pcm: Arc<PcmCache>, gain: f64) -> MixSource {
        MixSource {
            pcm,
            timeline_start: RationalTime::ZERO,
            timeline_duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: TimeMap::IDENTITY,
            gain: const_gain(gain),
            pan: None,
            fade: FadeSpec::NONE,
            out_of_range: AudioOutOfRange::Silence,
            enabled: true,
        }
    }

    fn const_gain(value: f64) -> Option<KeyframeTrack> {
        if value == 1.0 {
            return None;
        }
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe {
            t: RationalTime::ZERO,
            value: Value::F64(value),
            interp: Interp::Hold,
            spatial: None,
        });
        Some(track)
    }

    fn const_track(value: f64) -> Option<KeyframeTrack> {
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe {
            t: RationalTime::ZERO,
            value: Value::F64(value),
            interp: Interp::Hold,
            spatial: None,
        });
        Some(track)
    }

    #[test]
    fn two_sources_sum_deterministically() {
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
            crate::render::audio::cache::PcmFormat {
                channels: 1,
                sample_rate: 44_100,
            },
        )
        .unwrap();
        let stereo_48 = PcmCache::from_interleaved(
            [0.1, -0.1].repeat(480), // 0.01s @ 48000
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
                gain: None,
                pan: None,
                fade: FadeSpec::NONE,
                out_of_range: AudioOutOfRange::Silence,
                enabled: true,
            },
            MixSource {
                pcm: b,
                timeline_start: RationalTime::ZERO,
                timeline_duration: duration,
                time_map: TimeMap::IDENTITY,
                gain: None,
                pan: None,
                fade: FadeSpec::NONE,
                out_of_range: AudioOutOfRange::Silence,
                enabled: true,
            },
        ];
        let frames = 480; // 0.01s @ 48k
        let (out, _) = mix_audio(&sources, 1.0, 0, frames, None).unwrap();
        assert!(out[0] > 0.25 && out[0] < 0.35);
        assert!(out[1] > 0.05 && out[1] < 0.15);
    }

    #[test]
    fn varispeed_doubles_source_advance() {
        let pcm = stereo_cache(vec![0.0, 0.0, 0.25, 0.0, 0.5, 0.0, 0.75, 0.0]);
        let mut source = identity_source(pcm, 1.0);
        source.time_map = TimeMap::constant_speed(RationalTime::ZERO, 2, 1).unwrap();
        source.timeline_duration = RationalTime::try_new(2, CANONICAL_SAMPLE_RATE as i64).unwrap();
        let (out, _) = mix_audio(&[source], 1.0, 0, 2, None).unwrap();
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
            gain: None,
            pan: None,
            fade: FadeSpec::NONE,
            out_of_range: AudioOutOfRange::Silence,
            enabled: true,
        };
        let (out, _) = mix_audio(&[source], 1.0, ten_min - 32, 4, None).unwrap();
        assert_eq!(&out[0..2], &[0.0, 0.0]);
        assert_eq!(&out[2..4], &[0.01, -0.01]);
        assert_eq!(&out[4..6], &[0.02, -0.02]);
        assert_eq!(&out[6..8], &[0.03, -0.03]);
    }

    #[test]
    fn hold_gain_keyframes_follow_eval() {
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe {
            t: RationalTime::ZERO,
            value: Value::F64(0.5),
            interp: Interp::Hold,
            spatial: None,
        });
        track.insert(Keyframe {
            t: RationalTime::try_new(2, CANONICAL_SAMPLE_RATE as i64).unwrap(),
            value: Value::F64(2.0),
            interp: Interp::Hold,
            spatial: None,
        });
        let mid = RationalTime::try_new(1, CANONICAL_SAMPLE_RATE as i64).unwrap();
        let expected = match track.eval(mid) {
            Value::F64(v) => v,
            _ => panic!("expected f64"),
        };
        assert!((expected - 0.5).abs() < 1e-12);

        let source = MixSource {
            pcm: stereo_cache(vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0]),
            timeline_start: RationalTime::ZERO,
            timeline_duration: RationalTime::try_new(3, CANONICAL_SAMPLE_RATE as i64).unwrap(),
            time_map: TimeMap::IDENTITY,
            gain: Some(track),
            pan: None,
            fade: FadeSpec::NONE,
            out_of_range: AudioOutOfRange::Silence,
            enabled: true,
        };
        let (out, _) = mix_audio(&[source], 1.0, 1, 1, None).unwrap();
        assert!((out[0] as f64 - expected).abs() < 1e-6);
        assert!((out[1] as f64 - expected).abs() < 1e-6);
    }

    #[test]
    fn same_input_mixes_to_byte_identical_output() {
        let a = stereo_cache(
            (0..64)
                .flat_map(|i| [(i as f32 * 0.013).sin(), (i as f32 * 0.029).cos()])
                .collect(),
        );
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe {
            t: RationalTime::ZERO,
            value: Value::F64(0.2),
            interp: Interp::Linear,
            spatial: None,
        });
        track.insert(Keyframe {
            t: RationalTime::try_new(64, CANONICAL_SAMPLE_RATE as i64).unwrap(),
            value: Value::F64(0.9),
            interp: Interp::Bezier {
                x1: 0.25,
                y1: 0.1,
                x2: 0.75,
                y2: 0.9,
            },
            spatial: None,
        });
        let source = MixSource {
            pcm: a,
            timeline_start: RationalTime::ZERO,
            timeline_duration: RationalTime::try_new(64, CANONICAL_SAMPLE_RATE as i64).unwrap(),
            time_map: TimeMap::constant_speed(RationalTime::ZERO, 3, 2).unwrap(),
            gain: Some(track),
            pan: None,
            fade: FadeSpec::NONE,
            out_of_range: AudioOutOfRange::Loop,
            enabled: true,
        };
        let meter = AudioMeter::new();
        let (first, first_report) = mix_audio(&[source.clone()], 0.8, 0, 64, Some(&meter)).unwrap();
        let (second, second_report) = mix_audio(&[source], 0.8, 0, 64, Some(&meter)).unwrap();
        assert_eq!(
            first, second,
            "mix_audio は同一入力に対しbyte一致でなければならない"
        );
        assert_eq!(first_report, second_report);
    }

    #[test]
    fn gain_scales_output_linearly() {
        let a = stereo_cache(vec![0.4, -0.2]);
        let (base, _) = mix_audio(&[identity_source(a.clone(), 1.0)], 1.0, 0, 1, None).unwrap();
        let (scaled, _) = mix_audio(&[identity_source(a, 2.5)], 1.0, 0, 1, None).unwrap();
        assert!((scaled[0] - base[0] * 2.5).abs() < 1e-9);
        assert!((scaled[1] - base[1] * 2.5).abs() < 1e-9);
    }

    #[test]
    fn gain_zero_is_silence_without_affecting_other_sources() {
        let a = stereo_cache(vec![1.0, 1.0]);
        let b = stereo_cache(vec![0.5, 0.5]);
        let (out, _) = mix_audio(
            &[identity_source(a, 0.0), identity_source(b, 1.0)],
            1.0,
            0,
            1,
            None,
        )
        .unwrap();
        assert_eq!(out, vec![0.5, 0.5]);
    }

    #[test]
    fn pan_center_is_identity() {
        let (l, r) = apply_pan_stereo(0.6, 0.4, 0.0);
        assert!((l - 0.6).abs() < 1e-12);
        assert!((r - 0.4).abs() < 1e-12);
    }

    #[test]
    fn pan_hard_left_sums_both_channels_into_left() {
        let (l, r) = apply_pan_stereo(0.6, 0.4, -1.0);
        assert!((l - 1.0).abs() < 1e-12, "l={l}");
        assert!(r.abs() < 1e-12, "r={r}");
    }

    #[test]
    fn pan_hard_right_sums_both_channels_into_right() {
        let (l, r) = apply_pan_stereo(0.6, 0.4, 1.0);
        assert!(l.abs() < 1e-12, "l={l}");
        assert!((r - 1.0).abs() < 1e-12, "r={r}");
    }

    #[test]
    fn pan_out_of_range_values_clamp_instead_of_erroring() {
        let hard_right = apply_pan_stereo(0.6, 0.4, 1.0);
        let past_right = apply_pan_stereo(0.6, 0.4, 5.0);
        assert_eq!(hard_right, past_right);
        let hard_left = apply_pan_stereo(0.6, 0.4, -1.0);
        let past_left = apply_pan_stereo(0.6, 0.4, -5.0);
        assert_eq!(hard_left, past_left);
    }

    #[test]
    fn pan_field_routes_through_mix_audio() {
        let pcm = stereo_cache(vec![0.5, 0.5]); // 1 frame, L=R=0.5
        let mut source = identity_source(pcm, 1.0);
        source.pan = const_track(-1.0);
        let (out, _) = mix_audio(&[source], 1.0, 0, 1, None).unwrap();
        assert!((out[0] - 1.0).abs() < 1e-9, "L should sum both channels: {out:?}");
        assert!(out[1].abs() < 1e-9, "R should be silenced: {out:?}");
    }

    #[test]
    fn pan_none_track_defaults_to_center() {
        assert_eq!(eval_pan_at(&None, RationalTime::ZERO).unwrap(), 0.0);
    }

    #[test]
    fn linear_fade_in_ramps_from_zero_to_full() {
        let pcm = stereo_cache([1.0, 1.0].repeat(4));
        let mut source = identity_source(pcm, 1.0);
        let dur = RationalTime::try_new(4, CANONICAL_SAMPLE_RATE as i64).unwrap();
        source.timeline_duration = dur;
        source.fade = FadeSpec {
            fade_in: dur,
            fade_out: RationalTime::ZERO,
            curve: FadeCurve::Linear,
        };
        let (out, _) = mix_audio(&[source], 1.0, 0, 4, None).unwrap();
        assert!(out[0].abs() < 1e-6, "frame0 should be silent: {out:?}");
        assert!((out[2] - 0.25).abs() < 1e-5, "frame1: {out:?}");
        assert!((out[4] - 0.5).abs() < 1e-5, "frame2: {out:?}");
        assert!((out[6] - 0.75).abs() < 1e-5, "frame3: {out:?}");
    }

    #[test]
    fn linear_fade_out_ramps_from_full_to_zero() {
        let pcm = stereo_cache([1.0, 1.0].repeat(4));
        let mut source = identity_source(pcm, 1.0);
        let dur = RationalTime::try_new(4, CANONICAL_SAMPLE_RATE as i64).unwrap();
        source.timeline_duration = dur;
        source.fade = FadeSpec {
            fade_in: RationalTime::ZERO,
            fade_out: dur,
            curve: FadeCurve::Linear,
        };
        let (out, _) = mix_audio(&[source], 1.0, 0, 4, None).unwrap();
        assert!((out[0] - 1.0).abs() < 1e-5, "frame0: {out:?}");
        assert!((out[2] - 0.75).abs() < 1e-5, "frame1: {out:?}");
        assert!((out[4] - 0.5).abs() < 1e-5, "frame2: {out:?}");
        assert!((out[6] - 0.25).abs() < 1e-5, "frame3: {out:?}");
    }

    #[test]
    fn equal_power_fade_differs_from_linear_at_midpoint() {
        let linear = curve_value(0.5, FadeCurve::Linear);
        let equal_power = curve_value(0.5, FadeCurve::EqualPower);
        assert!((linear - 0.5).abs() < 1e-12);
        assert!((equal_power - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-9);
        assert!(equal_power > linear, "等パワー則は中間点でlinearより大きいはず");
    }

    #[test]
    fn overlapping_fade_in_and_out_multiply_without_leaving_unit_range() {
        let dur = RationalTime::try_new(4, CANONICAL_SAMPLE_RATE as i64).unwrap();
        let fade = FadeSpec {
            fade_in: dur,
            fade_out: dur,
            curve: FadeCurve::Linear,
        };
        for i in 0..4i64 {
            let local = RationalTime::try_new(i, CANONICAL_SAMPLE_RATE as i64).unwrap();
            let envelope = fade_envelope(local, dur, &fade);
            assert!(
                (0.0..=1.0).contains(&envelope),
                "envelope out of [0,1] at frame {i}: {envelope}"
            );
        }
        let start = fade_envelope(RationalTime::ZERO, dur, &fade);
        let mid = fade_envelope(
            RationalTime::try_new(2, CANONICAL_SAMPLE_RATE as i64).unwrap(),
            dur,
            &fade,
        );
        assert!(mid > start, "中央({mid})は端({start})より明るいはず");
    }

    #[test]
    fn fade_duration_exceeding_clip_length_is_clamped_not_rejected() {
        let pcm = stereo_cache([1.0, 1.0].repeat(2));
        let mut source = identity_source(pcm, 1.0);
        let dur = RationalTime::try_new(2, CANONICAL_SAMPLE_RATE as i64).unwrap();
        source.timeline_duration = dur;
        source.fade = FadeSpec {
            fade_in: RationalTime::try_new(100, CANONICAL_SAMPLE_RATE as i64).unwrap(),
            fade_out: RationalTime::ZERO,
            curve: FadeCurve::Linear,
        };
        let (out, _) = mix_audio(&[source], 1.0, 0, 2, None).unwrap();
        assert!(out.iter().all(|s| s.is_finite()));
        assert!(out[0].abs() < 1e-6, "先頭はほぼ無音のはず: {out:?}");
    }

    #[test]
    fn same_input_with_pan_and_fade_mixes_to_byte_identical_output() {
        let pcm = stereo_cache(
            (0..32)
                .flat_map(|i| [(i as f32 * 0.037).sin(), (i as f32 * 0.051).cos()])
                .collect(),
        );
        let dur = RationalTime::try_new(32, CANONICAL_SAMPLE_RATE as i64).unwrap();
        let mut source = MixSource {
            pcm,
            timeline_start: RationalTime::ZERO,
            timeline_duration: dur,
            time_map: TimeMap::IDENTITY,
            gain: const_track(0.7),
            pan: const_track(-0.3),
            fade: FadeSpec {
                fade_in: RationalTime::try_new(8, CANONICAL_SAMPLE_RATE as i64).unwrap(),
                fade_out: RationalTime::try_new(8, CANONICAL_SAMPLE_RATE as i64).unwrap(),
                curve: FadeCurve::EqualPower,
            },
            out_of_range: AudioOutOfRange::Silence,
            enabled: true,
        };
        let (first, first_report) = mix_audio(&[source.clone()], 1.0, 0, 32, None).unwrap();
        let (second, second_report) = mix_audio(&[source.clone()], 1.0, 0, 32, None).unwrap();
        assert_eq!(first, second, "pan/fade込みでも同一入力はbyte一致でなければならない");
        assert_eq!(first_report, second_report);

        source.enabled = false;
        let (silent, report) = mix_audio(&[source], 1.0, 0, 32, None).unwrap();
        assert!(silent.iter().all(|&s| s == 0.0));
        assert_eq!(report.silence_frames, 32);
    }

    #[test]
    fn normalize_gain_for_peak_computes_linear_scalar() {
        let pcm = stereo_cache(vec![0.5, -0.25, 0.4, -0.1]); // |peak| = 0.5
        let gain = normalize_gain_for_peak(&pcm, 1.0).unwrap();
        assert!((gain - 2.0).abs() < 1e-9, "gain={gain}");
    }

    #[test]
    fn normalize_gain_for_peak_silence_is_noop() {
        let pcm = stereo_cache(vec![0.0, 0.0, 0.0, 0.0]);
        assert_eq!(normalize_gain_for_peak(&pcm, 1.0).unwrap(), 1.0);
    }

    #[test]
    fn normalize_gain_for_peak_rejects_invalid_target() {
        let pcm = stereo_cache(vec![0.5, 0.5]);
        assert!(normalize_gain_for_peak(&pcm, -1.0).is_err());
        assert!(normalize_gain_for_peak(&pcm, f64::NAN).is_err());
        assert!(normalize_gain_for_peak(&pcm, f64::INFINITY).is_err());
    }

    #[test]
    fn normalize_gain_applied_through_existing_gain_path_hits_target_peak() {
        let target_peak = 0.8_f64;
        let pcm = stereo_cache(vec![0.25f32, -0.25, 0.5, -0.5]); // |peak| = 0.5
        let gain_value = normalize_gain_for_peak(&pcm, target_peak).unwrap();

        let mut source = identity_source(pcm, gain_value);
        source.timeline_duration = RationalTime::try_new(2, CANONICAL_SAMPLE_RATE as i64).unwrap();
        let (out, _) = mix_audio(&[source], 1.0, 0, 2, None).unwrap();
        let peak = out.iter().fold(0.0f32, |acc, &s| acc.max(s.abs()));
        assert!((peak as f64 - target_peak).abs() < 1e-6, "peak={peak}");
    }
}
