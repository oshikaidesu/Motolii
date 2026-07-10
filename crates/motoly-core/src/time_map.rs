use serde::{Deserialize, Serialize};

use crate::RationalTime;

/// Timeline時刻からsource時刻への最小TimeMap。
///
/// M1では恒等・offset・定数速度だけを扱う。可変速やリタイム曲線はこの型の後方互換な拡張で足す。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimeMap {
    pub source_start: RationalTime,
    pub timeline_start: RationalTime,
    pub speed_num: i64,
    pub speed_den: i64,
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
    ) -> Self {
        assert!(speed_den != 0, "TimeMap speed denominator must not be zero");
        Self {
            source_start,
            timeline_start,
            speed_num,
            speed_den,
        }
    }

    pub fn map(&self, timeline_time: RationalTime) -> RationalTime {
        self.source_start
            + (timeline_time - self.timeline_start)
                * self.speed_num
                * RationalTime::new(1, self.speed_den)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_maps_same_time() {
        let t = RationalTime::new(1001, 30000);
        assert_eq!(TimeMap::identity().map(t), t);
    }

    #[test]
    fn offset_maps_timeline_origin_to_source_start() {
        let map = TimeMap::offset(
            RationalTime::from_seconds(10),
            RationalTime::from_seconds(2),
        );
        assert_eq!(
            map.map(RationalTime::from_seconds(2)),
            RationalTime::from_seconds(10)
        );
        assert_eq!(
            map.map(RationalTime::from_seconds(3)),
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
        );
        assert_eq!(
            map.map(RationalTime::from_seconds(13)),
            RationalTime::from_seconds(11)
        );
    }
}
