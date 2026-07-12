use serde::{Deserialize, Deserializer, Serialize};

use crate::RationalTime;

/// Timeline時刻からsource時刻への最小TimeMap。
///
/// M1では恒等・offset・定数速度だけを扱う。可変速やリタイム曲線はこの型の後方互換な拡張で足す。
///
/// 凍結範囲(2026-07-10): **報告口**(`try_map`でsource_timeを解決する契約)のみ。
/// 実デコード/シークの再写像はM2(未実証のため凍結しない)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct TimeMap {
    pub source_start: RationalTime,
    pub timeline_start: RationalTime,
    pub speed_num: i64,
    pub speed_den: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TimeMapError {
    #[error("TimeMap speed_den must be positive")]
    NonPositiveSpeedDenominator,
    #[error("TimeMap speed_num must be positive (reverse playback deferred)")]
    NonPositiveSpeedNum,
}

#[derive(Deserialize)]
struct RawTimeMap {
    source_start: RationalTime,
    timeline_start: RationalTime,
    speed_num: i64,
    speed_den: i64,
}

impl<'de> Deserialize<'de> for TimeMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawTimeMap::deserialize(deserializer)?;
        let map = Self {
            source_start: raw.source_start,
            timeline_start: raw.timeline_start,
            speed_num: raw.speed_num,
            speed_den: raw.speed_den,
        };
        map.validate().map_err(serde::de::Error::custom)?;
        Ok(map)
    }
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

    /// JSON等の未検証入力向け。M2では`speed_num > 0`かつ`speed_den > 0`のみ。
    pub fn validate(&self) -> Result<(), TimeMapError> {
        if self.speed_den <= 0 {
            return Err(TimeMapError::NonPositiveSpeedDenominator);
        }
        if self.speed_num <= 0 {
            return Err(TimeMapError::NonPositiveSpeedNum);
        }
        Ok(())
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
    fn rejects_non_positive_speed_denominator() {
        assert!(matches!(
            TimeMap::constant_speed(RationalTime::ZERO, RationalTime::ZERO, 1, 0),
            Err(TimeMapError::NonPositiveSpeedDenominator)
        ));
        assert!(matches!(
            TimeMap::constant_speed(RationalTime::ZERO, RationalTime::ZERO, 1, -1),
            Err(TimeMapError::NonPositiveSpeedDenominator)
        ));
    }

    #[test]
    fn rejects_non_positive_speed_num() {
        assert!(matches!(
            TimeMap::constant_speed(RationalTime::ZERO, RationalTime::ZERO, 0, 1),
            Err(TimeMapError::NonPositiveSpeedNum)
        ));
        assert!(matches!(
            TimeMap::constant_speed(RationalTime::ZERO, RationalTime::ZERO, -1, 1),
            Err(TimeMapError::NonPositiveSpeedNum)
        ));
    }

    #[test]
    fn serde_rejects_zero_and_negative_speed_num() {
        let zero = r#"{
            "source_start":{"num":0,"den":1},
            "timeline_start":{"num":0,"den":1},
            "speed_num":0,
            "speed_den":1
        }"#;
        assert!(serde_json::from_str::<TimeMap>(zero).is_err());
        let neg = r#"{
            "source_start":{"num":0,"den":1},
            "timeline_start":{"num":0,"den":1},
            "speed_num":-1,
            "speed_den":1
        }"#;
        assert!(serde_json::from_str::<TimeMap>(neg).is_err());
    }

    #[test]
    fn serde_rejects_zero_speed_denominator() {
        let json = r#"{
            "source_start":{"num":0,"den":1},
            "timeline_start":{"num":0,"den":1},
            "speed_num":1,
            "speed_den":0
        }"#;
        let err = serde_json::from_str::<TimeMap>(json).unwrap_err();
        assert!(err.to_string().contains("speed_den"), "{err}");
    }
}
