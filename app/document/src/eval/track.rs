use serde::{Deserialize, Serialize};

use crate::doc::core::RationalTime;

use crate::doc::eval::bezier::{cubic_bezier_ease, sample, solve_curve_x};
use crate::doc::eval::value::Value;

#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum TrackError {
    #[error("Bezier control point x1/x2 must be in [0,1], got x1={x1} x2={x2}")]
    InvalidBezier { x1: f64, x2: f64 },
    #[error(
        "Bezier split is unrepresentable: progress={progress}, x1={x1}, y1={y1}, x2={x2}, y2={y2}"
    )]
    UnrepresentableBezierSplit {
        progress: f64,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        curve_parameter: Option<f64>,
        split_value: Option<f64>,
        left_x_denominator: f64,
        right_x_denominator: f64,
        left_y_denominator: Option<f64>,
        right_y_denominator: Option<f64>,
    },
    #[error("keyframes must be sorted by strictly increasing time without duplicates")]
    UnsortedOrDuplicateKeys,
    #[error("補間パラメータが定義域の外: {0}")]
    InvalidInterp(String),
    #[error("{kind} 区間は分割できない — u→値の純関数を同型2本へ割る規則が無い")]
    UnsplittableInterp { kind: &'static str },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Interp {
    Hold,
    Linear,
    Bezier {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
    },
    /// 自己相似バウンド。`first_dip` は最初の谷の時刻、`dip` はその深さ。
    /// 振幅も持続も1バウンドごとに `1-dip` 倍(弾道則ではない)。
    Bounce { first_dip: f64, dip: f64 },
    /// `limit` は行き過ぎの天井、`period` は最初の谷の位置、`damp` は減衰。
    Elastic { limit: f64, period: f64, damp: f64 },
    /// 波を繰り返す。`peak` は頂点の位相、`linear` は cos↔三角の混ぜ、
    /// `envelope_end` は谷が乗る線の終点。
    Cyclic { period: f64, peak: f64, linear: f64, envelope_end: f64 },
    /// 揺らす。`grain` は粒の細かさ、`center_*` は揺れの中心と振幅、`bias` は帯の押し。
    Random { seed: f64, grain: f64, center_u: f64, center_v: f64, bias: f64 },
    /// 段。`width` は段幅(連続値)、`smooth` は段へ到着する平滑幅。
    Steps { width: f64, smooth: f64 },
    /// 弾む段。遷移は段時刻から**始まる**(Steps とは逆)。
    ElasticSteps { width: f64, elasticity: f64 },
}

impl Interp {
    pub const MAX_BOUNCES: u32 = 32;

    pub fn ease(&self, u: f64) -> f64 {
        match *self {
            Interp::Hold => 0.0,
            Interp::Linear => u.clamp(0.0, 1.0),
            Interp::Bezier { x1, y1, x2, y2 } => {
                cubic_bezier_ease(x1.clamp(0.0, 1.0), y1, x2.clamp(0.0, 1.0), y2, u)
            }
            Interp::Bounce { first_dip, dip } => bounce_ease(first_dip, dip, u),
            Interp::Elastic { limit, period, damp } => elastic_ease(limit, period, damp, u),
            Interp::Cyclic { period, peak, linear, envelope_end } => {
                cyclic_ease(period, peak, linear, envelope_end, u)
            }
            Interp::Random { seed, grain, center_u, center_v, bias } => {
                random_ease(seed, grain, center_u, center_v, bias, u)
            }
            Interp::Steps { width, smooth } => steps_ease(width, smooth, u),
            Interp::ElasticSteps { width, elasticity } => {
                elastic_steps_ease(width, elasticity, u)
            }
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Interp::Hold => "Hold",
            Interp::Linear => "Linear",
            Interp::Bezier { .. } => "Bezier",
            Interp::Bounce { .. } => "Bounce",
            Interp::Elastic { .. } => "Elastic",
            Interp::Cyclic { .. } => "Cyclic",
            Interp::Random { .. } => "Random",
            Interp::Steps { .. } => "Steps",
            Interp::ElasticSteps { .. } => "ElasticSteps",
        }
    }

    pub fn split_at(&self, progress: f64) -> Result<(Interp, Interp), TrackError> {
        if matches!(
            self,
            Interp::Bounce { .. }
                | Interp::Elastic { .. }
                | Interp::Cyclic { .. }
                | Interp::Random { .. }
                | Interp::Steps { .. }
                | Interp::ElasticSteps { .. }
        ) {
            return Err(TrackError::UnsplittableInterp { kind: self.kind() });
        }
        if !progress.is_finite() || !(0.0..=1.0).contains(&progress) {
            if let Interp::Bezier { x1, y1, x2, y2 } = *self {
                return Err(TrackError::UnrepresentableBezierSplit {
                    progress,
                    x1,
                    y1,
                    x2,
                    y2,
                    curve_parameter: None,
                    split_value: None,
                    left_x_denominator: progress,
                    right_x_denominator: 1.0 - progress,
                    left_y_denominator: None,
                    right_y_denominator: None,
                });
            }
            return Err(TrackError::UnrepresentableBezierSplit {
                progress,
                x1: 0.0,
                y1: 0.0,
                x2: 0.0,
                y2: 0.0,
                curve_parameter: None,
                split_value: None,
                left_x_denominator: progress,
                right_x_denominator: 1.0 - progress,
                left_y_denominator: None,
                right_y_denominator: None,
            });
        }

        match *self {
            Interp::Hold => Ok((Interp::Hold, Interp::Hold)),
            Interp::Linear => Ok((Interp::Linear, Interp::Linear)),
            Interp::Bounce { .. }
            | Interp::Elastic { .. }
            | Interp::Cyclic { .. }
            | Interp::Random { .. }
            | Interp::Steps { .. }
            | Interp::ElasticSteps { .. } => {
                Err(TrackError::UnsplittableInterp { kind: self.kind() })
            }
            Interp::Bezier { x1, y1, x2, y2 } => {
                let s = solve_curve_x(x1, x2, progress);
                if !s.is_finite() || !(0.0..=1.0).contains(&s) {
                    return Err(TrackError::UnrepresentableBezierSplit {
                        progress,
                        x1,
                        y1,
                        x2,
                        y2,
                        curve_parameter: Some(s),
                        split_value: None,
                        left_x_denominator: progress,
                        right_x_denominator: 1.0 - progress,
                        left_y_denominator: None,
                        right_y_denominator: None,
                    });
                }

                let v = sample(y1, y2, s);
                if !v.is_finite() {
                    return Err(TrackError::UnrepresentableBezierSplit {
                        progress,
                        x1,
                        y1,
                        x2,
                        y2,
                        curve_parameter: Some(s),
                        split_value: Some(v),
                        left_x_denominator: progress,
                        right_x_denominator: 1.0 - progress,
                        left_y_denominator: None,
                        right_y_denominator: None,
                    });
                }

                let inv = 1.0 - s;
                let left_x_denom = progress;
                let right_x_denom = 1.0 - progress;
                let left_y_denom = v;
                let right_y_denom = 1.0 - v;

                if !left_x_denom.is_finite()
                    || !right_x_denom.is_finite()
                    || !left_y_denom.is_finite()
                    || !right_y_denom.is_finite()
                    || left_x_denom == 0.0
                    || right_x_denom == 0.0
                    || left_y_denom == 0.0
                    || right_y_denom == 0.0
                {
                    return Err(TrackError::UnrepresentableBezierSplit {
                        progress,
                        x1,
                        y1,
                        x2,
                        y2,
                        curve_parameter: Some(s),
                        split_value: Some(v),
                        left_x_denominator: left_x_denom,
                        right_x_denominator: right_x_denom,
                        left_y_denominator: Some(left_y_denom),
                        right_y_denominator: Some(right_y_denom),
                    });
                }

                let l1 = (x1 * s, y1 * s);
                let l2 = (x1 * inv + x2 * s, y1 * inv + y2 * s);
                let l3 = (x2 * inv + s, y2 * inv + s);
                let l12 = (l1.0 * inv + l2.0 * s, l1.1 * inv + l2.1 * s);
                let l23 = (l2.0 * inv + l3.0 * s, l2.1 * inv + l3.1 * s);

                let left = Interp::Bezier {
                    x1: l1.0 / left_x_denom,
                    y1: l1.1 / left_y_denom,
                    x2: l12.0 / left_x_denom,
                    y2: l12.1 / left_y_denom,
                };
                let right = Interp::Bezier {
                    x1: (l23.0 - progress) / right_x_denom,
                    y1: (l23.1 - v) / right_y_denom,
                    x2: (l3.0 - progress) / right_x_denom,
                    y2: (l3.1 - v) / right_y_denom,
                };

                if !is_valid_bezier_control(left) || !is_valid_bezier_control(right) {
                    return Err(TrackError::UnrepresentableBezierSplit {
                        progress,
                        x1,
                        y1,
                        x2,
                        y2,
                        curve_parameter: Some(s),
                        split_value: Some(v),
                        left_x_denominator: left_x_denom,
                        right_x_denominator: right_x_denom,
                        left_y_denominator: Some(left_y_denom),
                        right_y_denominator: Some(right_y_denom),
                    });
                }

                Ok((left, right))
            }
        }
    }
}

/// 以下は Alight Motion 実機から起こした閉形式(2026-07-19 観察台帳)。
/// handle の位置がそのまま parameter になっている。

fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    if (edge1 - edge0).abs() < f64::EPSILON {
        return if x < edge0 { 0.0 } else { 1.0 };
    }
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn fract(x: f64) -> f64 {
    x - x.floor()
}

/// 自己相似バウンド。谷の頂点が handle。振幅も持続も1バウンドごとに `d` 倍。
fn bounce_ease(first_dip: f64, dip: f64, u: f64) -> f64 {
    if !u.is_finite() || u <= 0.0 {
        return 0.0;
    }
    if u >= 1.0 {
        return 1.0;
    }
    let d = (1.0 - dip).clamp(0.02, 1.0);
    let t = (first_dip / (1.0 + d)).max(0.02);
    if u <= t {
        return (u / t) * (u / t);
    }
    let mut cusp = t;
    let mut scale = d;
    for _ in 0..24 {
        let width = 2.0 * t * scale;
        if cusp + width > 1.0 {
            return 1.0;
        }
        if u <= cusp + width {
            let local = (u - cusp - t * scale) / (t * scale);
            return 1.0 - scale * (1.0 - local * local);
        }
        cusp += width;
        scale *= d;
    }
    1.0
}

/// `v = 1 − (A−1)(1−u)^n cos(2πu/p)`。クランプではなく振幅を縮める。
fn elastic_ease(limit: f64, period: f64, damp: f64, u: f64) -> f64 {
    if !u.is_finite() || u <= 0.0 {
        return 0.0;
    }
    if u >= 1.0 {
        return 1.0;
    }
    let period = if period.is_finite() && period > 0.0 { period } else { 0.3 };
    let n = if damp >= 0.97 { 400.0 } else { 1.0 / (1.0 - damp).powi(2) };
    let closed = (limit - 1.0) * (1.0 - u).powf(n);
    let envelope = mix(1.0, closed, smoothstep(0.0, 0.45 * period, u));
    1.0 - envelope * ((std::f64::consts::TAU * u) / period).cos()
}

fn cyclic_wave(peak: f64, linear: f64, phi: f64) -> f64 {
    let s = peak.clamp(0.02, 0.98);
    let tri = if phi < s { phi / s } else { (1.0 - phi) / (1.0 - s) };
    let smooth = if phi < s {
        (1.0 - ((std::f64::consts::PI * phi) / s).cos()) / 2.0
    } else {
        (1.0 + ((std::f64::consts::PI * (phi - s)) / (1.0 - s)).cos()) / 2.0
    };
    mix(smooth, tri, linear.clamp(0.0, 1.0))
}

/// `v = f + (1−f)·W(frac(u/T))`、`f = envelope_end·u`。谷は envelope 線に乗る。
fn cyclic_ease(period: f64, peak: f64, linear: f64, envelope_end: f64, u: f64) -> f64 {
    if !u.is_finite() {
        return 0.0;
    }
    let period = if period.is_finite() && period > 0.0 { period } else { 2.0 / 7.0 };
    if u >= 1.0 {
        return envelope_end + (1.0 - envelope_end) * cyclic_wave(peak, linear, fract(1.0 / period));
    }
    let floor = envelope_end * u;
    floor + (1.0 - floor) * cyclic_wave(peak, linear, fract(u / period))
}

fn lattice_noise(seed: f64, index: f64) -> f64 {
    let value = ((seed + index * 97.13) * 12.9898).sin() * 43758.5453;
    (value - value.floor()) * 2.0 - 1.0
}

fn value_noise(x: f64, seed: f64) -> f64 {
    let index = x.floor();
    let local = x - index;
    let a = lattice_noise(seed, index);
    let b = lattice_noise(seed, index + 1.0);
    mix(a, b, (1.0 - (std::f64::consts::PI * local).cos()) / 2.0)
}

/// 揺れの中心からの距離が振幅。0..1 の外へも出る(クランプしない)。
const RANDOM_NULL_LEVEL: f64 = 0.47;

fn random_ease(seed: f64, grain: f64, center_u: f64, center_v: f64, bias: f64, u: f64) -> f64 {
    if !u.is_finite() || u <= 0.0 {
        return 0.0;
    }
    if u >= 1.0 {
        return 1.0;
    }
    let seed = seed.round();
    let frequency = mix(40.0, 8.0, (grain / 0.5).clamp(0.0, 1.0));
    let noise = 0.8 * value_noise(u * frequency, seed)
        + 0.2 * value_noise(u * frequency * 2.7, seed + 37.0);
    let amplitude = (1.7 * (center_v - RANDOM_NULL_LEVEL).abs()).clamp(0.0, 0.9);
    let envelope = smoothstep(0.0, 1.0, (1.0 - (u - center_u).abs() / 0.55).clamp(0.0, 1.0));
    let fade = smoothstep(0.0, 0.05, u) * smoothstep(1.0, 0.95, u);
    let push = (0.5 - bias)
        * (1.0 - (-u / 0.15).exp())
        * (1.0 - (-(1.0 - u) / 0.15).exp());
    u + amplitude * envelope * fade * noise + push
}

/// 対角線の sample-and-hold。平滑ランプは段時刻に**到着**する。
fn steps_ease(width: f64, smooth: f64, u: f64) -> f64 {
    if !u.is_finite() || u <= 0.0 {
        return 0.0;
    }
    if u >= 1.0 {
        return 1.0;
    }
    let width = if width.is_finite() && width > 0.0 { width } else { 0.178 };
    let smooth = smooth.max(0.0);
    let index = (u / width).floor();
    let local = u - index * width;
    let ramp_start = width - smooth;
    let progress = if smooth <= 0.0001 {
        0.0
    } else {
        smoothstep(0.0, 1.0, ((local - ramp_start) / smooth).clamp(0.0, 1.0))
    };
    (width * (index + progress)).clamp(0.0, 1.0)
}

/// 弾む段。遷移は段時刻 kP から**始まる**(Steps とは逆向き)。
fn elastic_steps_ease(width: f64, elasticity: f64, u: f64) -> f64 {
    if !u.is_finite() || u <= 0.0 {
        return 0.0;
    }
    if u >= 1.0 {
        return 1.0;
    }
    let width = if width.is_finite() && width > 0.0 { width } else { 0.2 };
    let e = elasticity.clamp(0.0, 1.0);
    if u < width {
        return 0.0;
    }
    let index = (u / width).floor() - 1.0;
    let tau = (u - (index + 1.0) * width) / width;
    let rise = mix(0.45, 0.03, e);
    let mut response = smoothstep(0.0, rise, tau);
    if tau > rise && e > 0.01 {
        let ring_t = tau - rise;
        let ring_amp = 0.36 * e.powf(3.2);
        response += ring_amp * (-5.9 * ring_t).exp() * ((std::f64::consts::TAU * ring_t) / 0.112).sin();
    }
    let predip = 0.045 * (std::f64::consts::PI * e).sin();
    response -= predip * smoothstep(0.78, 0.99, tau);
    (width * (index + response)).clamp(-0.2, 1.35)
}

fn is_valid_bezier_control(interp: Interp) -> bool {
    if let Interp::Bezier { x1, y1, x2, y2 } = interp {
        [x1, y1, x2, y2].iter().all(|v| v.is_finite())
            && (0.0..=1.0).contains(&x1)
            && (0.0..=1.0).contains(&x2)
    } else {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpatialTangent {
    pub out_tangent: [f64; 2],
    pub in_tangent: [f64; 2],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keyframe {
    pub t: RationalTime,
    pub value: Value,
    pub interp: Interp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spatial: Option<SpatialTangent>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "KeyframeTrackDe")]
pub struct KeyframeTrack {
    keys: Vec<Keyframe>,
}

#[derive(Deserialize)]
struct KeyframeTrackDe {
    keys: Vec<Keyframe>,
}

impl TryFrom<KeyframeTrackDe> for KeyframeTrack {
    type Error = TrackError;

    fn try_from(value: KeyframeTrackDe) -> Result<Self, Self::Error> {
        let track = Self { keys: value.keys };
        track.validate()?;
        Ok(track)
    }
}

impl KeyframeTrack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn try_from_keys(keys: Vec<Keyframe>) -> Result<Self, TrackError> {
        let track = Self { keys };
        track.validate()?;
        Ok(track)
    }

    pub fn insert(&mut self, key: Keyframe) {
        match self.keys.binary_search_by(|k| k.t.cmp(&key.t)) {
            Ok(i) => self.keys[i] = key,
            Err(i) => self.keys.insert(i, key),
        }
    }

    pub fn keys(&self) -> &[Keyframe] {
        &self.keys
    }

    pub fn validate(&self) -> Result<(), TrackError> {
        for window in self.keys.windows(2) {
            if window[0].t >= window[1].t {
                return Err(TrackError::UnsortedOrDuplicateKeys);
            }
        }
        for key in &self.keys {
            match key.interp {
                Interp::Bezier { x1, y1, x2, y2 } => {
                    if ![x1, y1, x2, y2].iter().all(|v| v.is_finite()) {
                        return Err(TrackError::InvalidBezier { x1, x2 });
                    }
                    if !(0.0..=1.0).contains(&x1) || !(0.0..=1.0).contains(&x2) {
                        return Err(TrackError::InvalidBezier { x1, x2 });
                    }
                }
                Interp::Bounce { first_dip, dip } => {
                    if !first_dip.is_finite() || first_dip <= 0.0 || !dip.is_finite() {
                        return Err(TrackError::InvalidInterp(format!(
                            "Bounce{{first_dip: {first_dip}, dip: {dip}}} — first_dip は正、\
                             どちらも有限"
                        )));
                    }
                }
                Interp::Elastic { limit, period, damp } => {
                    if !limit.is_finite() || !period.is_finite() || period <= 0.0 || !damp.is_finite()
                    {
                        return Err(TrackError::InvalidInterp(format!(
                            "Elastic{{limit: {limit}, period: {period}, damp: {damp}}} — \
                             period は正、どれも有限"
                        )));
                    }
                }
                Interp::Cyclic { period, peak, linear, envelope_end } => {
                    if !period.is_finite()
                        || period <= 0.0
                        || !peak.is_finite()
                        || !linear.is_finite()
                        || !envelope_end.is_finite()
                    {
                        return Err(TrackError::InvalidInterp(format!(
                            "Cyclic{{period: {period}, peak: {peak}, linear: {linear}, \
                             envelope_end: {envelope_end}}} — period は正、どれも有限"
                        )));
                    }
                }
                Interp::Random { seed, grain, center_u, center_v, bias } => {
                    if !seed.is_finite()
                        || !grain.is_finite()
                        || !center_u.is_finite()
                        || !center_v.is_finite()
                        || !bias.is_finite()
                    {
                        return Err(TrackError::InvalidInterp(format!(
                            "Random{{seed: {seed}, grain: {grain}, center_u: {center_u}, \
                             center_v: {center_v}, bias: {bias}}} — どれも有限"
                        )));
                    }
                }
                Interp::Steps { width, smooth } => {
                    if !width.is_finite() || width <= 0.0 || !smooth.is_finite() || smooth < 0.0 {
                        return Err(TrackError::InvalidInterp(format!(
                            "Steps{{width: {width}, smooth: {smooth}}} — width は正、\
                             smooth は 0 以上"
                        )));
                    }
                }
                Interp::ElasticSteps { width, elasticity } => {
                    if !width.is_finite() || width <= 0.0 || !elasticity.is_finite() {
                        return Err(TrackError::InvalidInterp(format!(
                            "ElasticSteps{{width: {width}, elasticity: {elasticity}}} — \
                             width は正、どちらも有限"
                        )));
                    }
                }
                Interp::Hold | Interp::Linear => {}
            }
        }
        Ok(())
    }

    pub fn eval(&self, t: RationalTime) -> Value {
        let keys = &self.keys;
        if keys.is_empty() {
            return Value::F64(0.0);
        }
        if t <= keys[0].t {
            return keys[0].value.clone();
        }
        let last = keys.len() - 1;
        if t >= keys[last].t {
            return keys[last].value.clone();
        }
        let i = match keys.binary_search_by(|k| k.t.cmp(&t)) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        let (a, b) = (&keys[i], &keys[i + 1]);
        if let Interp::Hold = a.interp {
            return a.value.clone();
        }
        let u = a.interp.ease(segment_u(a.t, b.t, t));
        interpolate_value(a, b, u)
    }
}

fn interpolate_value(a: &Keyframe, b: &Keyframe, u: f64) -> Value {
    if let (Value::Vec2(p0), Value::Vec2(p3)) = (&a.value, &b.value) {
        if a.spatial.is_some() || b.spatial.is_some() {
            let p1 = match &a.spatial {
                Some(s) => [p0[0] + s.out_tangent[0], p0[1] + s.out_tangent[1]],
                None => *p0,
            };
            let p2 = match &b.spatial {
                Some(s) => [p3[0] + s.in_tangent[0], p3[1] + s.in_tangent[1]],
                None => *p3,
            };
            return Value::Vec2(cubic_bezier_point(*p0, p1, p2, *p3, u));
        }
    }
    Value::lerp(&a.value, &b.value, u)
}

fn cubic_bezier_point(p0: [f64; 2], p1: [f64; 2], p2: [f64; 2], p3: [f64; 2], u: f64) -> [f64; 2] {
    let inv = 1.0 - u;
    std::array::from_fn(|i| {
        inv * inv * inv * p0[i]
            + 3.0 * inv * inv * u * p1[i]
            + 3.0 * inv * u * u * p2[i]
            + u * u * u * p3[i]
    })
}

fn segment_u(a: RationalTime, b: RationalTime, t: RationalTime) -> f64 {
    let den = seconds_since(b, a);
    if den == 0.0 {
        return 0.0;
    }
    seconds_since(t, a) / den
}

fn seconds_since(t: RationalTime, origin: RationalTime) -> f64 {
    match t.try_sub(origin) {
        Ok(rel) => rel.as_seconds_f64(),
        Err(_) => t.as_seconds_f64() - origin.as_seconds_f64(),
    }
}
