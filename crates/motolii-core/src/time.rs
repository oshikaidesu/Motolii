use std::cmp::Ordering;

use serde::{Deserialize, Deserializer, Serialize};

/// 有理数タイムスタンプ。秒 = num/den。
///
/// 浮動小数の秒を使うとフレーム境界の丸めが蓄積してドリフトする(落とし穴B-1)ため、
/// タイムライン上の時刻・長さは常にこの型で扱う。常に正規化(den > 0、既約)して保持する。
/// 演算は`try_add`等のResult経路のみ(公開APIは入力起因でpanicしない — M2E-16)。
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

    /// M2E-16: den==0拒否・符号正規化・既約化・0/x→0/1・オーバーフローはErr。
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

    /// 表示・デバッグ用途のみ。比較や演算にはf64を使わないこと。
    pub fn as_seconds_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }

    /// フレーム番号から時刻へ(frame / fps)。
    pub fn try_from_frame(frame: i64, fps: Fps) -> Result<Self, RationalTimeError> {
        let num = (frame as i128)
            .checked_mul(fps.den() as i128)
            .ok_or(RationalTimeError::Overflow)?;
        Self::try_reduce(num, fps.num() as i128)
    }

    /// 時刻が属するフレーム番号(床関数)。負の時刻でも数学的な床を返す。
    pub fn try_to_frame_floor(self, fps: Fps) -> Result<i64, RationalTimeError> {
        let n = (self.num as i128)
            .checked_mul(fps.num() as i128)
            .ok_or(RationalTimeError::Overflow)?;
        let d = (self.den as i128)
            .checked_mul(fps.den() as i128)
            .ok_or(RationalTimeError::Overflow)?;
        if d <= 0 {
            return Err(RationalTimeError::ZeroDenominator);
        }
        let q = n.div_euclid(d);
        i64::try_from(q).map_err(|_| RationalTimeError::Overflow)
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
        // 負の分母は符号を分子へ移す(負の時刻そのものは正当)
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

/// フレームレート(num/den フレーム毎秒)。例: 30fps = 30/1、29.97fps = 30000/1001。
/// 正値は型の不変条件(フィールドは非公開 — M2E-16)。
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
        Ok(Self { num, den })
    }

    pub const fn num(self) -> i64 {
        self.num
    }

    pub const fn den(self) -> i64 {
        self.den
    }

    /// 1フレームの長さ。Fpsの正値不変条件により常に成功する。
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

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

impl PartialEq for RationalTime {
    fn eq(&self, other: &Self) -> bool {
        // 常に既約・den>0で保持しているためフィールド比較でよい
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
        // 正規化済みi64同士の交差乗算はi128に収まる
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
