
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
