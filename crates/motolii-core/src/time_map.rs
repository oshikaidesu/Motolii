use serde::{Deserialize, Serialize};

use crate::RationalTime;

/// Timeline時刻からsource時刻への最小TimeMap。
///
/// M1では恒等・offset・定数速度だけを扱う。可変速やリタイム曲線はこの型の後方互換な拡張で足す。
///
/// 凍結範囲(2026-07-10): **報告口**(`try_map`でsource_timeを解決する契約)のみ。
/// 実デコード/シークの再写像はM2(未実証のため凍結しない)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimeMap {
    pub source_start: RationalTime,
    pub timeline_start: RationalTime,
    pub speed_num: i64,
    pub speed_den: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TimeMapError {
    #[error("TimeMap speed_den must not be zero")]
    ZeroSpeedDenominator,
}

impl TimeMap {
    pub const IDENTITY: Self = Self {
        source_start: RationalTime::ZERO,
        timeline_start: RationalTime::ZERO,
        speed_num: 1,
        speed_den: 1,
    };

    pub fn identity() -> Self {
        Self::IDENTITY
    }

    pub fn offset(source_start: RationalTime, timeline_start: RationalTime) -> Self {
        Self {
            source_start,
            timeline_start,
            speed_num: 1,
            speed_den: 1,
        }
    }

    pub fn constant_speed(
        source_start: RationalTime,
        timeline_start: RationalTime,
        speed_num: i64,
        speed_den: i64,
    ) -> Result<Self, TimeMapError> {
        let map = Self {
            source_start,
            timeline_start,
            speed_num,
            speed_den,
        };
        map.validate()?;
        Ok(map)
    }

    /// JSON等の未検証入力向け。拒否するかもしれない弱い約束。
    pub fn validate(&self) -> Result<(), TimeMapError> {
        if self.speed_den == 0 {
            Err(TimeMapError::ZeroSpeedDenominator)
        } else {
            Ok(())
        }
    }

    /// 未検証入力でもpanicしない写像。
    pub fn try_map(&self, timeline_time: RationalTime) -> Result<RationalTime, TimeMapError> {
        self.validate()?;
        Ok(self.source_start
            + (timeline_time - self.timeline_start)
                * self.speed_num
                * RationalTime::new(1, self.speed_den))
    }

    /// 恒等写像か。実デコードへの適用はM2まで未実装のため、export等は恒等のみ受理する。
    pub fn is_identity(&self) -> bool {
        *self == Self::IDENTITY
    }
}

impl Default for TimeMap {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_maps_same_time() {
        let t = RationalTime::new(1001, 30000);
        assert_eq!(TimeMap::identity().try_map(t).unwrap(), t);
    }

    #[test]
    fn is_identity_detects_non_identity_maps() {
        assert!(TimeMap::identity().is_identity());
        assert!(!TimeMap::offset(RationalTime::ZERO, RationalTime::from_seconds(1)).is_identity());
        assert!(
            !TimeMap::constant_speed(RationalTime::ZERO, RationalTime::ZERO, 2, 1)
                .unwrap()
                .is_identity()
        );
    }

    #[test]
    fn offset_maps_timeline_origin_to_source_start() {
        let map = TimeMap::offset(
            RationalTime::from_seconds(10),
            RationalTime::from_seconds(2),
        );
        assert_eq!(
            map.try_map(RationalTime::from_seconds(2)).unwrap(),
            RationalTime::from_seconds(10)
        );
        assert_eq!(
            map.try_map(RationalTime::from_seconds(3)).unwrap(),
            RationalTime::from_seconds(11)
        );
    }

    #[test]
    fn constant_speed_scales_delta() {
        let map = TimeMap::constant_speed(
            RationalTime::from_seconds(5),
            RationalTime::from_seconds(10),
            2,
            1,
        )
        .unwrap();
        assert_eq!(
            map.try_map(RationalTime::from_seconds(13)).unwrap(),
            RationalTime::from_seconds(11)
        );
    }

    #[test]
    fn rejects_zero_speed_denominator() {
        assert!(matches!(
            TimeMap::constant_speed(RationalTime::ZERO, RationalTime::ZERO, 1, 0),
            Err(TimeMapError::ZeroSpeedDenominator)
        ));
        let bad = TimeMap {
            source_start: RationalTime::ZERO,
            timeline_start: RationalTime::ZERO,
            speed_num: 1,
            speed_den: 0,
        };
        assert!(matches!(
            bad.try_map(RationalTime::ZERO),
            Err(TimeMapError::ZeroSpeedDenominator)
        ));
    }
}
