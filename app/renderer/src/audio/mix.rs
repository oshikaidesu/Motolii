
use std::sync::Arc;

use crate::doc::core::RationalTime;
use crate::doc::eval::{KeyframeTrack, Value};

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
        for &sample in pcm.samples_i16() {
            peak = peak.max(crate::render::audio::cache::i16_to_f32(sample).abs() as f64);
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
    let f0 = pcm.stereo_at(i0).expect("in-range");
    let f1 = pcm.stereo_at(i1).expect("in-range");
    let l = f0.0 as f64 * (1.0 - frac) + f1.0 as f64 * frac;
    let r = f0.1 as f64 * (1.0 - frac) + f1.1 as f64 * frac;
    (l, r)
}
