
use motolii_core::{RationalTime, RationalTimeError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TimeMapError {
    #[error("TimeMap speed_den must be positive")]
    NonPositiveSpeedDenominator,
    #[error("TimeMap speed_num must be positive (reverse playback not represented by this type)")]
    NonPositiveSpeedNum,
    #[error(transparent)]
    RationalTime(#[from] RationalTimeError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimeMap {
    pub source_start: RationalTime,
    speed_num: i64,
    speed_den: i64,
}

impl TimeMap {
    pub const IDENTITY: Self = Self {
        source_start: RationalTime::ZERO,
        speed_num: 1,
        speed_den: 1,
    };

    pub fn identity() -> Self {
        Self::IDENTITY
    }

    pub fn offset(source_start: RationalTime) -> Self {
        Self {
            source_start,
            speed_num: 1,
            speed_den: 1,
        }
    }

    pub fn constant_speed(
        source_start: RationalTime,
        speed_num: i64,
        speed_den: i64,
    ) -> Result<Self, TimeMapError> {
        let (speed_num, speed_den) = reduce_positive_ratio(speed_num, speed_den)?;
        Ok(Self {
            source_start,
            speed_num,
            speed_den,
        })
    }

    pub const fn speed_num(self) -> i64 {
        self.speed_num
    }

    pub const fn speed_den(self) -> i64 {
        self.speed_den
    }

    pub fn validate(&self) -> Result<(), TimeMapError> {
        if self.speed_den <= 0 {
            return Err(TimeMapError::NonPositiveSpeedDenominator);
        }
        if self.speed_num <= 0 {
            return Err(TimeMapError::NonPositiveSpeedNum);
        }
        Ok(())
    }

    pub fn try_map(&self, clip_local_time: RationalTime) -> Result<RationalTime, TimeMapError> {
        self.validate()?;
        let scaled = clip_local_time.try_mul_i64(self.speed_num)?;
        let unit = RationalTime::try_new(1, self.speed_den)?;
        let mapped = scaled.try_mul(unit)?;
        Ok(self.source_start.try_add(mapped)?)
    }

    pub fn is_identity(&self) -> bool {
        self.source_start == RationalTime::ZERO && self.speed_num == 1 && self.speed_den == 1
    }
}

impl Default for TimeMap {
    fn default() -> Self {
        Self::IDENTITY
    }
}

fn reduce_positive_ratio(num: i64, den: i64) -> Result<(i64, i64), TimeMapError> {
    if den <= 0 {
        return Err(TimeMapError::NonPositiveSpeedDenominator);
    }
    if num <= 0 {
        return Err(TimeMapError::NonPositiveSpeedNum);
    }
    let g = gcd_u128(num as u128, den as u128);
    Ok((num / g as i64, den / g as i64))
}

fn gcd_u128(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rt(num: i64, den: i64) -> RationalTime {
        RationalTime::try_new(num, den).unwrap()
    }

    #[test]
    fn identity_maps_same_time() {
        let t = rt(1001, 30000);
        assert_eq!(TimeMap::identity().try_map(t).unwrap(), t);
    }

    #[test]
    fn is_identity_is_semantic() {
        assert!(TimeMap::identity().is_identity());
        let reduced = TimeMap::constant_speed(RationalTime::ZERO, 2, 2).unwrap();
        assert_eq!((reduced.speed_num(), reduced.speed_den()), (1, 1));
        assert!(reduced.is_identity());
        assert_eq!(reduced, TimeMap::identity());

        assert!(!TimeMap::offset(RationalTime::from_seconds(1)).is_identity());
        assert!(!TimeMap::constant_speed(RationalTime::ZERO, 2, 1)
            .unwrap()
            .is_identity());
    }

    #[test]
    fn offset_maps_local_zero_to_source_start() {
        let map = TimeMap::offset(RationalTime::from_seconds(10));
        assert_eq!(
            map.try_map(RationalTime::ZERO).unwrap(),
            RationalTime::from_seconds(10)
        );
        assert_eq!(
            map.try_map(RationalTime::from_seconds(1)).unwrap(),
            RationalTime::from_seconds(11)
        );
    }

    #[test]
    fn constant_speed_scales_clip_local() {
        let map = TimeMap::constant_speed(RationalTime::from_seconds(5), 2, 1).unwrap();
        assert_eq!(
            map.try_map(RationalTime::from_seconds(3)).unwrap(),
            RationalTime::from_seconds(11)
        );
    }

    #[test]
    fn rejects_non_positive_speed_denominator() {
        assert!(matches!(
            TimeMap::constant_speed(RationalTime::ZERO, 1, 0),
            Err(TimeMapError::NonPositiveSpeedDenominator)
        ));
        assert!(matches!(
            TimeMap::constant_speed(RationalTime::ZERO, 1, -1),
            Err(TimeMapError::NonPositiveSpeedDenominator)
        ));
    }

    #[test]
    fn rejects_non_positive_speed_num() {
        assert!(matches!(
            TimeMap::constant_speed(RationalTime::ZERO, 0, 1),
            Err(TimeMapError::NonPositiveSpeedNum)
        ));
        assert!(matches!(
            TimeMap::constant_speed(RationalTime::ZERO, -1, 1),
            Err(TimeMapError::NonPositiveSpeedNum)
        ));
    }
}
