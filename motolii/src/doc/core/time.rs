
use std::cmp::Ordering;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Copy, Serialize)]
pub struct RationalTime {
    num: i64,
    den: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RationalTimeError {
    #[error("RationalTime: denominator must not be zero")]
    ZeroDenominator,
    #[error("RationalTime: value overflows i64 after normalization")]
    Overflow,
}

#[derive(Deserialize)]
struct RawRationalTime {
    num: i64,
    den: i64,
}

impl<'de> Deserialize<'de> for RationalTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawRationalTime::deserialize(deserializer)?;
        RationalTime::try_new(raw.num, raw.den).map_err(serde::de::Error::custom)
    }
}

impl RationalTime {
    pub const ZERO: RationalTime = RationalTime { num: 0, den: 1 };

    pub fn try_new(num: i64, den: i64) -> Result<Self, RationalTimeError> {
        Self::try_reduce(num as i128, den as i128)
    }

    pub const fn from_seconds(secs: i64) -> Self {
        Self { num: secs, den: 1 }
    }

    pub const fn num(self) -> i64 {
        self.num
    }

    pub const fn den(self) -> i64 {
        self.den
    }

    pub fn as_seconds_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }

    pub fn try_from_frame(frame: i64, fps: Fps) -> Result<Self, RationalTimeError> {
        let num = (frame as i128)
            .checked_mul(fps.den() as i128)
            .ok_or(RationalTimeError::Overflow)?;
        Self::try_reduce(num, fps.num() as i128)
    }

    pub fn try_to_frame_floor(self, fps: Fps) -> Result<i64, RationalTimeError> {
        Ok(self.try_to_sample_index(fps)?.0)
    }

    pub fn try_to_frame_round(self, fps: Fps) -> Result<i64, RationalTimeError> {
        let num = (self.num as i128)
            .checked_mul(fps.num() as i128)
            .ok_or(RationalTimeError::Overflow)?;
        let den = (self.den as i128)
            .checked_mul(fps.den() as i128)
            .ok_or(RationalTimeError::Overflow)?;
        round_rational_to_i64(num, den)
    }

    pub fn try_from_decimal_str(s: &str) -> Result<Self, RationalTimeError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(RationalTimeError::Overflow);
        }
        let (sign, rest) = if let Some(r) = s.strip_prefix('-') {
            (-1i128, r)
        } else if let Some(r) = s.strip_prefix('+') {
            (1i128, r)
        } else {
            (1i128, s)
        };
        let (int_s, frac_s) = match rest.split_once('.') {
            Some((i, f)) => (i, f),
            None => (rest, ""),
        };
        let int_part: i128 = if int_s.is_empty() {
            0
        } else {
            int_s.parse().map_err(|_| RationalTimeError::Overflow)?
        };
        let frac_len = frac_s.len();
        let frac_part: i128 = if frac_s.is_empty() {
            0
        } else {
            frac_s.parse().map_err(|_| RationalTimeError::Overflow)?
        };
        let den_pow = if frac_len == 0 {
            1i128
        } else {
            10i128
                .checked_pow(frac_len as u32)
                .ok_or(RationalTimeError::Overflow)?
        };
        let unsigned = int_part
            .checked_mul(den_pow)
            .and_then(|v| v.checked_add(frac_part))
            .ok_or(RationalTimeError::Overflow)?;
        let num = sign
            .checked_mul(unsigned)
            .ok_or(RationalTimeError::Overflow)?;
        Self::try_reduce(num, den_pow)
    }

    pub fn try_to_sample_index(self, rate: Fps) -> Result<(i64, f64), RationalTimeError> {
        self.try_to_sample_index_since(Self::ZERO, rate)
    }

    pub fn try_to_sample_index_since(
        self,
        origin: Self,
        rate: Fps,
    ) -> Result<(i64, f64), RationalTimeError> {
        let tn = self.num as i128;
        let td = self.den as i128;
        let on = origin.num as i128;
        let od = origin.den as i128;
        let rn = rate.num() as i128;
        let rd = rate.den() as i128;
        if td <= 0 || od <= 0 || rd <= 0 || rn <= 0 {
            return Err(RationalTimeError::ZeroDenominator);
        }
        let left = tn.checked_mul(od).ok_or(RationalTimeError::Overflow)?;
        let right = on.checked_mul(td).ok_or(RationalTimeError::Overflow)?;
        let rel_num = left.checked_sub(right).ok_or(RationalTimeError::Overflow)?;

        let neg = rel_num < 0;
        let num_abs = rel_num.unsigned_abs();
        let (q_abs, rem, den) = crate::doc::core::wide_div::mul_div_floor_3den(
            num_abs, rn as u128, td as u128, od as u128, rd as u128,
        )?;
        let u = crate::doc::core::wide_div::rem_over_den_f64(rem, den);
        debug_assert!((0.0..1.0).contains(&u));

        if !neg {
            let q = i64::try_from(q_abs).map_err(|_| RationalTimeError::Overflow)?;
            Ok((q, u))
        } else if rem == crate::doc::core::wide_div::U256::ZERO {
            let q = i64::try_from(q_abs).map_err(|_| RationalTimeError::Overflow)?;
            Ok((-q, 0.0))
        } else {
            let q = i64::try_from(q_abs + 1).map_err(|_| RationalTimeError::Overflow)?;
            let frac = crate::doc::core::wide_div::complement_unit_interval(u);
            debug_assert!((0.0..1.0).contains(&frac));
            Ok((-q, frac))
        }
    }

    pub fn try_neg(self) -> Result<Self, RationalTimeError> {
        Self::try_reduce(-(self.num as i128), self.den as i128)
    }

    pub fn try_add(self, rhs: Self) -> Result<Self, RationalTimeError> {
        let left = (self.num as i128)
            .checked_mul(rhs.den as i128)
            .ok_or(RationalTimeError::Overflow)?;
        let right = (rhs.num as i128)
            .checked_mul(self.den as i128)
            .ok_or(RationalTimeError::Overflow)?;
        let num = left.checked_add(right).ok_or(RationalTimeError::Overflow)?;
        let den = (self.den as i128)
            .checked_mul(rhs.den as i128)
            .ok_or(RationalTimeError::Overflow)?;
        Self::try_reduce(num, den)
    }

    pub fn try_sub(self, rhs: Self) -> Result<Self, RationalTimeError> {
        let left = (self.num as i128)
            .checked_mul(rhs.den as i128)
            .ok_or(RationalTimeError::Overflow)?;
        let right = (rhs.num as i128)
            .checked_mul(self.den as i128)
            .ok_or(RationalTimeError::Overflow)?;
        let num = left.checked_sub(right).ok_or(RationalTimeError::Overflow)?;
        let den = (self.den as i128)
            .checked_mul(rhs.den as i128)
            .ok_or(RationalTimeError::Overflow)?;
        Self::try_reduce(num, den)
    }

    pub fn try_mul(self, rhs: Self) -> Result<Self, RationalTimeError> {
        let num = (self.num as i128)
            .checked_mul(rhs.num as i128)
            .ok_or(RationalTimeError::Overflow)?;
        let den = (self.den as i128)
            .checked_mul(rhs.den as i128)
            .ok_or(RationalTimeError::Overflow)?;
        Self::try_reduce(num, den)
    }

    pub fn try_mul_i64(self, rhs: i64) -> Result<Self, RationalTimeError> {
        let num = (self.num as i128)
            .checked_mul(rhs as i128)
            .ok_or(RationalTimeError::Overflow)?;
        Self::try_reduce(num, self.den as i128)
    }

    fn try_reduce(num: i128, den: i128) -> Result<Self, RationalTimeError> {
        if den == 0 {
            return Err(RationalTimeError::ZeroDenominator);
        }
        let (num, den) = if den < 0 { (-num, -den) } else { (num, den) };
        if num == 0 {
            return Ok(Self::ZERO);
        }
        let g = gcd(num.unsigned_abs(), den.unsigned_abs()).max(1);
        let num = num / g as i128;
        let den = den / g as i128;
        let num = i64::try_from(num).map_err(|_| RationalTimeError::Overflow)?;
        let den = i64::try_from(den).map_err(|_| RationalTimeError::Overflow)?;
        Ok(Self { num, den })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Fps {
    num: i64,
    den: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum FpsError {
    #[error("Fps: numerator and denominator must be positive")]
    NonPositive,
}

#[derive(Deserialize)]
struct RawFps {
    num: i64,
    den: i64,
}
impl<'de> Deserialize<'de> for Fps {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawFps::deserialize(deserializer)?;
        Fps::try_new(raw.num, raw.den).map_err(serde::de::Error::custom)
    }
}

impl Fps {
    pub const fn try_new(num: i64, den: i64) -> Result<Self, FpsError> {
        if num <= 0 || den <= 0 {
            return Err(FpsError::NonPositive);
        }
        let g = const_gcd_u64(num as u64, den as u64) as i64;
        Ok(Self {
            num: num / g,
            den: den / g,
        })
    }

    pub const fn num(self) -> i64 {
        self.num
    }

    pub const fn den(self) -> i64 {
        self.den
    }

    pub fn frame_duration(self) -> RationalTime {
        match RationalTime::try_new(self.den, self.num) {
            Ok(t) => t,
            Err(_) => unreachable!("Fps invariant: num and den are positive"),
        }
    }

    pub fn as_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }
}

pub fn format_ffmpeg_seek_before_frame(frame: i64, fps: Fps) -> Result<String, RationalTimeError> {
    if frame <= 0 {
        return Err(RationalTimeError::Overflow);
    }
    let frame_time = RationalTime::try_from_frame(frame, fps)?;
    let half_frame = RationalTime::try_new(
        fps.den(),
        2i64.checked_mul(fps.num())
            .ok_or(RationalTimeError::Overflow)?,
    )?;
    let seek = frame_time.try_sub(half_frame)?;
    Ok(format!("{:.6}", seek.as_seconds_f64()))
}

fn round_rational_to_i64(num: i128, den: i128) -> Result<i64, RationalTimeError> {
    if den <= 0 {
        return Err(RationalTimeError::ZeroDenominator);
    }
    if num == 0 {
        return Ok(0);
    }
    let neg = num < 0;
    let num_abs = num.unsigned_abs();
    let den_u = den as u128;
    let floor = num_abs / den_u;
    let rem = num_abs % den_u;
    let twice_rem = rem.checked_mul(2).ok_or(RationalTimeError::Overflow)?;
    let rounded_abs = if twice_rem < den_u {
        floor
    } else {
        floor.checked_add(1).ok_or(RationalTimeError::Overflow)?
    };
    let signed = if neg {
        -(i64::try_from(rounded_abs).map_err(|_| RationalTimeError::Overflow)?)
    } else {
        i64::try_from(rounded_abs).map_err(|_| RationalTimeError::Overflow)?
    };
    Ok(signed)
}

const fn const_gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

impl PartialEq for RationalTime {
    fn eq(&self, other: &Self) -> bool {
        self.num == other.num && self.den == other.den
    }
}

impl Eq for RationalTime {}

impl std::hash::Hash for RationalTime {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.num.hash(state);
        self.den.hash(state);
    }
}

impl PartialOrd for RationalTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RationalTime {
    fn cmp(&self, other: &Self) -> Ordering {
        let lhs = self.num as i128 * other.den as i128;
        let rhs = other.num as i128 * self.den as i128;
        lhs.cmp(&rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rt(num: i64, den: i64) -> RationalTime {
        RationalTime::try_new(num, den).unwrap()
    }

    fn fps(num: i64, den: i64) -> Fps {
        Fps::try_new(num, den).unwrap()
    }

    #[test]
    fn normalizes_sign_and_reduces() {
        let t = rt(2, -4);
        assert_eq!(t.num(), -1);
        assert_eq!(t.den(), 2);
        assert_eq!(rt(30, 30), rt(1, 1));
    }

    #[test]
    fn arithmetic() {
        let a = rt(1, 3);
        let b = rt(1, 6);
        assert_eq!(a.try_add(b).unwrap(), rt(1, 2));
        assert_eq!(a.try_sub(b).unwrap(), rt(1, 6));
        assert_eq!(b.try_mul_i64(3).unwrap(), rt(1, 2));
    }

    #[test]
    fn ordering_across_denominators() {
        let a = rt(1001, 30000); // 29.97fpsの1フレーム
        let b = rt(1, 30); // 30fpsの1フレーム
        assert!(a > b);
        assert!(rt(-1, 30) < RationalTime::ZERO);
    }

    #[test]
    fn frame_conversion_exact_ntsc() {
        let rate = fps(30000, 1001);
        for frame in [0i64, 1, 29, 30, 1799, 1800, 123_456] {
            let t = RationalTime::try_from_frame(frame, rate).unwrap();
            assert_eq!(t.try_to_frame_floor(rate).unwrap(), frame, "frame {frame}");
        }
    }

    #[test]
    fn sample_index_matches_frame_on_lattice() {
        let rate = fps(30000, 1001);
        for frame in [0i64, 1, 15, 29, 30, 1799] {
            let t = RationalTime::try_from_frame(frame, rate).unwrap();
            let (idx, u) = t.try_to_sample_index(rate).unwrap();
            assert_eq!(idx, frame, "sample index {frame}");
            assert_eq!(u, 0.0, "exact frame must have zero fraction");
        }
    }

    #[test]
    fn sample_index_fraction_is_rational_remainder() {
        let rate = fps(10, 1);
        let (idx, u) = rt(55, 100).try_to_sample_index(rate).unwrap();
        assert_eq!(idx, 5);
        assert!((u - 0.5).abs() < 1e-15);
    }

    #[test]
    fn sample_index_negative_fraction_stays_in_unit_interval() {
        let rate = fps(10, 1);
        let (idx, u) = rt(-5, 100).try_to_sample_index(rate).unwrap();
        assert_eq!(idx, -1);
        assert!((u - 0.5).abs() < 1e-15);
        assert!((0.0..1.0).contains(&u));
    }

    #[test]
    fn sample_index_avoids_f64_underflow_at_ntsc_frame() {
        let rate = fps(30000, 1001);
        let frame = 15i64;
        let t = RationalTime::try_from_frame(frame, rate).unwrap();
        let f64_floor = (t.as_seconds_f64() * rate.as_f64()).floor() as i64;
        assert_eq!(
            f64_floor,
            frame - 1,
            "precondition: f64 path must underfloor at frame {frame}"
        );
        let (idx, u) = t.try_to_sample_index(rate).unwrap();
        assert_eq!(idx, frame);
        assert_eq!(u, 0.0);
    }

    #[test]
    fn sample_index_since_survives_i64_span() {
        let start = RationalTime::from_seconds(i64::MIN);
        let t = RationalTime::from_seconds(i64::MAX);
        assert!(t.try_sub(start).is_err(), "precondition: try_sub overflows");
        let rate = fps(1, 1_000_000_000);
        let (idx, u) = t.try_to_sample_index_since(start, rate).unwrap();
        let expected = ((i64::MAX as i128 - i64::MIN as i128) / 1_000_000_000) as i64;
        assert_eq!(idx, expected);
        assert!((0.0..1.0).contains(&u));
    }

    #[test]
    fn sample_index_since_near_zero_despite_i128_overflow() {
        let d = i64::MAX;
        let t = rt(1000, d);
        let start = rt(1, d - 2);
        let rate = fps(d - 1, d);
        let tn = t.num() as i128;
        let td = t.den() as i128;
        let on = start.num() as i128;
        let od = start.den() as i128;
        let rn = rate.num() as i128;
        let rel_num = tn * od - on * td;
        assert!(
            rel_num.checked_mul(rn).is_none(),
            "precondition: rel_num * rate_num must overflow i128"
        );
        let (idx, u) = t.try_to_sample_index_since(start, rate).unwrap();
        assert_eq!(idx, 0);
        assert!((0.0..1.0).contains(&u));
        assert!(u < 1e-10, "position should be tiny, got {u}");
    }

    #[test]
    fn frame_round_nearest() {
        let rate = fps(30, 1);
        assert_eq!(rt(1, 30).try_to_frame_round(rate).unwrap(), 1);
        assert_eq!(rt(49, 3000).try_to_frame_round(rate).unwrap(), 0);
        assert_eq!(rt(51, 3000).try_to_frame_round(rate).unwrap(), 1);
        assert_eq!(rt(-49, 3000).try_to_frame_round(rate).unwrap(), 0);
        assert_eq!(rt(-51, 3000).try_to_frame_round(rate).unwrap(), -1);
        assert_eq!(rt(1, 60).try_to_frame_round(rate).unwrap(), 1);
        assert_eq!(rt(-1, 60).try_to_frame_round(rate).unwrap(), -1);
    }

    #[test]
    fn frame_round_ntsc_lattice() {
        let rate = fps(30000, 1001);
        for frame in [0i64, 1, 29, 30, 1799, 1800] {
            let t = RationalTime::try_from_frame(frame, rate).unwrap();
            assert_eq!(t.try_to_frame_round(rate).unwrap(), frame, "frame {frame}");
        }
    }

    #[test]
    fn decimal_str_parses_ffprobe_style() {
        let t = RationalTime::try_from_decimal_str("2.002000").unwrap();
        assert_eq!(t, rt(2002000, 1_000_000));
        assert_eq!(RationalTime::try_from_decimal_str(".5").unwrap(), rt(1, 2));
        assert_eq!(
            RationalTime::try_from_decimal_str("-1.25").unwrap(),
            rt(-5, 4)
        );
    }

    #[test]
    fn ffmpeg_seek_before_frame_matches_half_frame_offset() {
        let rate = fps(30, 1);
        assert_eq!(
            format_ffmpeg_seek_before_frame(1, rate).unwrap(),
            "0.016667"
        );
        assert_eq!(
            format_ffmpeg_seek_before_frame(30, rate).unwrap(),
            "0.983333"
        );
        let ntsc = fps(30000, 1001);
        let legacy = {
            let target = (15f64 - 0.5) * ntsc.den() as f64 / ntsc.num() as f64;
            format!("{target:.6}")
        };
        assert_eq!(format_ffmpeg_seek_before_frame(15, ntsc).unwrap(), legacy);
    }

    #[test]
    fn frame_floor_boundaries() {
        let rate = fps(30, 1);
        assert_eq!(rt(1, 30).try_to_frame_floor(rate).unwrap(), 1);
        assert_eq!(rt(999, 30000).try_to_frame_floor(rate).unwrap(), 0);
        assert_eq!(rt(-1, 60).try_to_frame_floor(rate).unwrap(), -1);
    }

    #[test]
    fn mixed_fps_clips_align() {
        let len = RationalTime::try_from_frame(48, fps(24, 1)).unwrap();
        assert_eq!(len, RationalTime::from_seconds(2));
        assert_eq!(len.try_to_frame_floor(fps(30, 1)).unwrap(), 60);
    }

    #[test]
    fn no_float_drift_accumulation() {
        let rate = fps(30000, 1001);
        let mut t = RationalTime::ZERO;
        let n = 30 * 60 * 60 * 10;
        for _ in 0..n {
            t = t.try_add(rate.frame_duration()).unwrap();
        }
        assert_eq!(t, RationalTime::try_from_frame(n, rate).unwrap());
    }

    #[test]
    fn try_new_rejects_zero_denominator() {
        assert_eq!(
            RationalTime::try_new(1, 0),
            Err(RationalTimeError::ZeroDenominator)
        );
    }

    #[test]
    fn try_new_normalizes_negative_denominator() {
        let t = RationalTime::try_new(3, -6).unwrap();
        assert_eq!(t.num(), -1);
        assert_eq!(t.den(), 2);
    }

    #[test]
    fn try_new_reduces_by_gcd() {
        let t = RationalTime::try_new(6, 9).unwrap();
        assert_eq!(t, rt(2, 3));
    }

    #[test]
    fn try_new_zero_over_any_is_zero_one() {
        assert_eq!(RationalTime::try_new(0, 42).unwrap(), RationalTime::ZERO);
        assert_eq!(RationalTime::try_new(0, -7).unwrap(), RationalTime::ZERO);
    }

    #[test]
    fn try_neg_i64_min_overflows() {
        let t = RationalTime::try_new(i64::MIN, 1).unwrap();
        assert_eq!(t.try_neg(), Err(RationalTimeError::Overflow));
    }

    #[test]
    fn try_new_i64_min_with_negative_den_overflows() {
        assert_eq!(
            RationalTime::try_new(i64::MIN, -1),
            Err(RationalTimeError::Overflow)
        );
    }

    #[test]
    fn serde_rejects_zero_denominator() {
        let err = serde_json::from_str::<RationalTime>(r#"{"num":1,"den":0}"#).unwrap_err();
        assert!(err.to_string().contains("denominator"), "{err}");
    }

    #[test]
    fn serde_normalizes_on_load() {
        let t: RationalTime = serde_json::from_str(r#"{"num":2,"den":-4}"#).unwrap();
        assert_eq!(t.num(), -1);
        assert_eq!(t.den(), 2);
        let z: RationalTime = serde_json::from_str(r#"{"num":0,"den":5}"#).unwrap();
        assert_eq!(z, RationalTime::ZERO);
    }

    #[test]
    fn fps_try_new_rejects_non_positive() {
        assert_eq!(Fps::try_new(0, 1), Err(FpsError::NonPositive));
        assert_eq!(Fps::try_new(30, -1), Err(FpsError::NonPositive));
        assert_eq!(Fps::try_new(-30, 1), Err(FpsError::NonPositive));
    }

    #[test]
    fn fps_try_new_reduces_by_gcd() {
        let f = Fps::try_new(60, 2).unwrap();
        assert_eq!(f.num(), 30);
        assert_eq!(f.den(), 1);
        assert_eq!(Fps::try_new(60, 2).unwrap(), Fps::try_new(30, 1).unwrap());
    }

    #[test]
    fn fps_serde_rejects_non_positive() {
        let err = serde_json::from_str::<Fps>(r#"{"num":0,"den":1}"#).unwrap_err();
        assert!(err.to_string().contains("positive"), "{err}");
    }

    #[test]
    fn fps_fields_are_encapsulated() {
        let rate = fps(30, 1);
        assert_eq!(rate.num(), 30);
        assert_eq!(rate.den(), 1);
    }
}
