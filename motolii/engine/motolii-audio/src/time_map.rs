//! owns: クリップローカル時刻 → ソース時刻の写像(D1g)。
//!
//! 旧 `crates/motolii-core/src/time_map.rs` からの移植。**旧 `motolii-core` の型では
//! ない** — `next/core/motolii-core` はこの型を持っていない(2026-08-20 リセットで
//! 落とされた)。store 側にも代わりが無い: `motolii_store::LayerTiming`/`Speed` は
//! **comp フレーム単位の整数写像**(裁定63)であって、mix が要る「48kHz サンプル
//! グリッド上の有理数写像」を表せない。よってこの crate 自身が持つ(`program.rs` が
//! `LayerTiming` から comp の fps を通してこの型を組み立てる)。
//!
//! OWNS-JUSTIFICATION(B): 探索対象=`motolii_store::LayerTiming`/`Speed` — 裁定63で
//! compフレーム単位の整数写像と決まっていることを確認した上で、mixが要る
//! 「48kHzサンプルグリッド上の有理数写像」を店側が表現できないことを具体的に
//! 確かめた(裁定215 棚卸し 2026-08-23 #11、「上流に無い」を裁定番号付きで
//! 実証した最良の例の1つ)。
//!
//! **落としたもの**: 旧型が持っていた `overrun_mode`(`OverrunMode` — Freeze/Black/
//! Loop、D3 未着手のため常に Freeze 相当でしか使われていなかった)。`mix.rs` の
//! 範囲外挙動は `AudioOutOfRange`(Silence/Loop)が別に持っており、`try_map` 自体は
//! 呼び出し側を見ていない — 旧版でも `mix.rs` は `overrun_mode` を一度も読んでいない
//! (使われていない状態を移植しても保守が増えるだけなので、ここで落とす)。

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

/// クリップローカル時刻 → ソース時刻の写像。
///
/// `clip_local_time = timeline_time - clip.start` は呼び出し側の責務。この型自身は
/// 素材尺を知らない純写像。
///
/// `speed_num`/`speed_den` は構築時に既約化され、フィールド非公開で不変条件を型に
/// 載せる。**正の速度のみ**(逆再生は未表現 — 旧型も同じ制約だった)。
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

    /// ソース原点オフセットのみ(速度1)。
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

    /// クリップローカル時刻 → ソース時刻。未検証入力でも panic しない。
    pub fn try_map(&self, clip_local_time: RationalTime) -> Result<RationalTime, TimeMapError> {
        self.validate()?;
        let scaled = clip_local_time.try_mul_i64(self.speed_num)?;
        let unit = RationalTime::try_new(1, self.speed_den)?;
        let mapped = scaled.try_mul(unit)?;
        Ok(self.source_start.try_add(mapped)?)
    }

    /// 意味的恒等: 正準アフィンが恒等か。
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
