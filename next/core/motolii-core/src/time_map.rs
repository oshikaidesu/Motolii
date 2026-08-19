use serde::{Deserialize, Deserializer, Serialize};

use crate::{RationalTime, RationalTimeError};

/// クリップローカル時刻 → ソース時刻の TimeMap(D1g)。
///
/// `clip_local_time = timeline_time - clip.start` は呼び出し側(Clip)の責務。
/// TimeMap は素材尺を知らない純写像で、`overrun_mode` は保持のみ(適用は D3)。
///
/// `speed_num`/`speed_den` は構築時に既約化され、フィールド非公開で不変条件を型に載せる。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct TimeMap {
    pub source_start: RationalTime,
    speed_num: i64,
    speed_den: i64,
    #[serde(default, rename = "overrun_mode")]
    pub overrun_mode: OverrunMode,
}

/// ソース採取が素材 available 範囲を外れたときのモード。適用は D3。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OverrunMode {
    /// 近い側の端フレームへクランプ(既定)。
    #[default]
    Freeze,
    /// 非描画。
    Black,
    /// available 範囲で wrap。
    Loop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TimeMapError {
    #[error("TimeMap speed_den must be positive")]
    NonPositiveSpeedDenominator,
    #[error("TimeMap speed_num must be positive (reverse playback deferred)")]
    NonPositiveSpeedNum,
    #[error("TimeMap overrun mode {0:?} is not applied yet (D3); refuse silent Freeze fallback")]
    UnsupportedOverrunMode(OverrunMode),
    #[error(transparent)]
    RationalTime(#[from] RationalTimeError),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTimeMap {
    source_start: RationalTime,
    speed_num: i64,
    speed_den: i64,
    #[serde(default, rename = "overrun_mode")]
    overrun_mode: OverrunMode,
}

impl<'de> Deserialize<'de> for TimeMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawTimeMap::deserialize(deserializer)?;
        Self::try_new(
            raw.source_start,
            raw.speed_num,
            raw.speed_den,
            raw.overrun_mode,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl TimeMap {
    pub const IDENTITY: Self = Self {
        source_start: RationalTime::ZERO,
        speed_num: 1,
        speed_den: 1,
        overrun_mode: OverrunMode::Freeze,
    };

    pub fn identity() -> Self {
        Self::IDENTITY
    }

    /// ソース原点オフセットのみ(速度1・Freeze)。
    pub fn offset(source_start: RationalTime) -> Self {
        Self {
            source_start,
            speed_num: 1,
            speed_den: 1,
            overrun_mode: OverrunMode::Freeze,
        }
    }

    pub fn constant_speed(
        source_start: RationalTime,
        speed_num: i64,
        speed_den: i64,
    ) -> Result<Self, TimeMapError> {
        Self::try_new(source_start, speed_num, speed_den, OverrunMode::Freeze)
    }

    pub fn try_new(
        source_start: RationalTime,
        speed_num: i64,
        speed_den: i64,
        overrun_mode: OverrunMode,
    ) -> Result<Self, TimeMapError> {
        let (speed_num, speed_den) = reduce_positive_ratio(speed_num, speed_den)?;
        Ok(Self {
            source_start,
            speed_num,
            speed_den,
            overrun_mode,
        })
    }

    pub const fn speed_num(self) -> i64 {
        self.speed_num
    }

    pub const fn speed_den(self) -> i64 {
        self.speed_den
    }

    /// 構築不変条件の再確認。正準コンストラクタ経由なら常に Ok。
    pub fn validate(&self) -> Result<(), TimeMapError> {
        if self.speed_den <= 0 {
            return Err(TimeMapError::NonPositiveSpeedDenominator);
        }
        if self.speed_num <= 0 {
            return Err(TimeMapError::NonPositiveSpeedNum);
        }
        Ok(())
    }

    /// D3以前: Black/Loop を黙って Freeze 相当にしない。
    pub fn require_freeze_overrun(&self) -> Result<(), TimeMapError> {
        match self.overrun_mode {
            OverrunMode::Freeze => Ok(()),
            mode => Err(TimeMapError::UnsupportedOverrunMode(mode)),
        }
    }

    /// クリップローカル時刻 → ソース時刻。未検証入力でも panic しない。
    pub fn try_map(&self, clip_local_time: RationalTime) -> Result<RationalTime, TimeMapError> {
        self.validate()?;
        let scaled = clip_local_time.try_mul_i64(self.speed_num)?;
        let unit = RationalTime::try_new(1, self.speed_den)?;
        let mapped = scaled.try_mul(unit)?;
        Ok(self.source_start.try_add(mapped)?)
    }

    /// 意味的恒等: 正準アフィンが恒等かつ overrun_mode==Freeze。
    pub fn is_identity(&self) -> bool {
        self.source_start == RationalTime::ZERO
            && self.speed_num == 1
            && self.speed_den == 1
            && self.overrun_mode == OverrunMode::Freeze
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
    fn is_identity_is_semantic_and_requires_freeze() {
        assert!(TimeMap::identity().is_identity());
        let reduced = TimeMap::constant_speed(RationalTime::ZERO, 2, 2).unwrap();
        assert_eq!((reduced.speed_num(), reduced.speed_den()), (1, 1));
        assert!(reduced.is_identity());
        assert_eq!(reduced, TimeMap::identity());

        let black = TimeMap::try_new(RationalTime::ZERO, 1, 1, OverrunMode::Black).unwrap();
        assert!(!black.is_identity());

        assert!(!TimeMap::offset(RationalTime::from_seconds(1)).is_identity());
        assert!(!TimeMap::constant_speed(RationalTime::ZERO, 2, 1)
            .unwrap()
            .is_identity());
    }

    #[test]
    fn reduced_speed_eq_hash_match_canonical() {
        let a = TimeMap::constant_speed(RationalTime::ZERO, 2, 2).unwrap();
        let b = TimeMap::constant_speed(RationalTime::ZERO, 1, 1).unwrap();
        assert_eq!(a, b);
        let mut ha = std::collections::HashSet::new();
        ha.insert(a);
        assert!(ha.contains(&b));
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
    fn clip_move_preserves_resolve() {
        // resolve(start, map, timeline) = map.map(timeline - start)
        let map = TimeMap::constant_speed(RationalTime::from_seconds(2), 3, 2).unwrap();
        let start = RationalTime::from_seconds(5);
        let timeline = RationalTime::from_seconds(8);
        let delta = RationalTime::from_seconds(4);
        let orig = map.try_map(timeline.try_sub(start).unwrap()).unwrap();
        let moved = map
            .try_map(
                timeline
                    .try_add(delta)
                    .unwrap()
                    .try_sub(start.try_add(delta).unwrap())
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(moved, orig);
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

    #[test]
    fn serde_rejects_zero_and_negative_speed_num() {
        let zero = r#"{
            "source_start":{"num":0,"den":1},
            "speed_num":0,
            "speed_den":1
        }"#;
        assert!(serde_json::from_str::<TimeMap>(zero).is_err());
        let neg = r#"{
            "source_start":{"num":0,"den":1},
            "speed_num":-1,
            "speed_den":1
        }"#;
        assert!(serde_json::from_str::<TimeMap>(neg).is_err());
    }

    #[test]
    fn serde_rejects_zero_speed_denominator() {
        let json = r#"{
            "source_start":{"num":0,"den":1},
            "speed_num":1,
            "speed_den":0
        }"#;
        let err = serde_json::from_str::<TimeMap>(json).unwrap_err();
        assert!(err.to_string().contains("speed_den"), "{err}");
    }

    #[test]
    fn serde_rejects_legacy_timeline_start() {
        let json = r#"{
            "source_start":{"num":0,"den":1},
            "timeline_start":{"num":0,"den":1},
            "speed_num":1,
            "speed_den":1
        }"#;
        assert!(serde_json::from_str::<TimeMap>(json).is_err());
    }

    #[test]
    fn serde_rejects_legacy_overrun_key() {
        let json = r#"{
            "source_start":{"num":0,"den":1},
            "speed_num":1,
            "speed_den":1,
            "overrun":"freeze"
        }"#;
        assert!(serde_json::from_str::<TimeMap>(json).is_err());
    }

    #[test]
    fn overrun_mode_defaults_to_freeze_and_roundtrips() {
        let json = r#"{
            "source_start":{"num":0,"den":1},
            "speed_num":1,
            "speed_den":1
        }"#;
        let map: TimeMap = serde_json::from_str(json).unwrap();
        assert_eq!(map.overrun_mode, OverrunMode::Freeze);

        for mode in [OverrunMode::Freeze, OverrunMode::Black, OverrunMode::Loop] {
            let m = TimeMap::try_new(RationalTime::ZERO, 1, 1, mode).unwrap();
            let encoded = serde_json::to_string(&m).unwrap();
            assert!(encoded.contains("overrun_mode"), "{encoded}");
            assert!(!encoded.contains("\"overrun\""), "{encoded}");
            let again: TimeMap = serde_json::from_str(&encoded).unwrap();
            assert_eq!(again.overrun_mode, mode);
        }
    }

    #[test]
    fn require_freeze_rejects_black_and_loop() {
        let black = TimeMap::try_new(RationalTime::ZERO, 1, 1, OverrunMode::Black).unwrap();
        assert!(matches!(
            black.require_freeze_overrun(),
            Err(TimeMapError::UnsupportedOverrunMode(OverrunMode::Black))
        ));
        let loop_mode = TimeMap::try_new(RationalTime::ZERO, 1, 1, OverrunMode::Loop).unwrap();
        assert!(matches!(
            loop_mode.require_freeze_overrun(),
            Err(TimeMapError::UnsupportedOverrunMode(OverrunMode::Loop))
        ));
        assert!(TimeMap::identity().require_freeze_overrun().is_ok());
    }
}
