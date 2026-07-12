//! 有理数BPM(M2E-11④)。既約化して保持し、拍時刻をRationalTimeに畳む。

use serde::de::{self, Deserialize, Deserializer};
use serde::{Deserialize as DeserializeDerive, Serialize};

use motolii_core::{RationalTime, RationalTimeError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum BpmError {
    #[error("BPM numerator must be positive, got {0}")]
    NonPositiveNum(i64),
    #[error("BPM denominator must be positive, got {0}")]
    NonPositiveDen(i64),
    #[error(transparent)]
    Time(#[from] RationalTimeError),
}

/// プロジェクトBPM。`f64`禁止。常に正・既約で保持する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Bpm {
    num: i64,
    den: i64,
}

#[derive(DeserializeDerive)]
struct RawBpm {
    num: i64,
    den: i64,
}

impl<'de> Deserialize<'de> for Bpm {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawBpm::deserialize(deserializer)?;
        Bpm::try_new(raw.num, raw.den).map_err(de::Error::custom)
    }
}

impl Bpm {
    pub const DEFAULT: Self = Self { num: 120, den: 1 };

    pub fn try_new(num: i64, den: i64) -> Result<Self, BpmError> {
        if num <= 0 {
            return Err(BpmError::NonPositiveNum(num));
        }
        if den <= 0 {
            return Err(BpmError::NonPositiveDen(den));
        }
        let g = gcd(num as u64, den as u64).max(1);
        Ok(Self {
            num: num / g as i64,
            den: den / g as i64,
        })
    }

    pub fn num(self) -> i64 {
        self.num
    }

    pub fn den(self) -> i64 {
        self.den
    }

    /// 1拍の長さ(秒) = `60 / bpm` = `60 * den / num`。
    pub fn try_beat_duration(self) -> Result<RationalTime, BpmError> {
        let num = 60i64
            .checked_mul(self.den)
            .ok_or(RationalTimeError::Overflow)?;
        Ok(RationalTime::try_new(num, self.num)?)
    }
}

impl Default for Bpm {
    fn default() -> Self {
        Self::DEFAULT
    }
}

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beat_duration_folds_into_rational_time() {
        let bpm = Bpm::try_new(120, 1).unwrap();
        assert_eq!(bpm.try_beat_duration().unwrap(), RationalTime::try_new(1, 2).unwrap());
    }

    #[test]
    fn reduces_on_construct() {
        let a = Bpm::try_new(240, 2).unwrap();
        let b = Bpm::try_new(120, 1).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.num(), 120);
        assert_eq!(a.den(), 1);
    }

    #[test]
    fn fractional_bpm_stays_exact() {
        let bpm = Bpm::try_new(90, 1).unwrap();
        assert_eq!(bpm.try_beat_duration().unwrap(), RationalTime::try_new(2, 3).unwrap());
    }

    #[test]
    fn rejects_non_positive() {
        assert!(matches!(
            Bpm::try_new(0, 1),
            Err(BpmError::NonPositiveNum(0))
        ));
        assert!(matches!(
            Bpm::try_new(120, -1),
            Err(BpmError::NonPositiveDen(-1))
        ));
    }
}
