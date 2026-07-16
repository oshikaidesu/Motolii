//! D3変換層の評価時刻意味論(#55 / 監査T-9/LG-3)。
//!
//! M1では評価時刻=ソースPTS(タイムライン=ソースの縮退)だった。
//! D3は `timeline_time` を正本とし、クリップごとに
//! `clip_local = timeline_time - clip.start` → `TimeMap::try_map` → `source_time`
//! と明示的に写す。この再定義を暗黙に行わないための宣言モジュール。

use motolii_core::RationalTime;

/// 変換層が受け取る評価時刻。常にタイムライン時刻。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvaluationTime {
    pub timeline_time: RationalTime,
}

impl EvaluationTime {
    pub fn new(timeline_time: RationalTime) -> Self {
        Self { timeline_time }
    }
}

/// M1 export は timeline_time をソースPTSとして使っていた(恒等 TimeMap 縮退)。
pub const M1_SOURCE_PTS_EQUALS_TIMELINE: &str =
    "M1 export used timeline_time as source PTS (identity TimeMap degeneracy)";

/// D3: `(timeline_time - clip.start)` を TimeMap に通して source_time を得る。
pub const D3_CLIP_LOCAL_TO_SOURCE_VIA_TIMEMAP: &str =
    "D3 maps (timeline_time - clip.start) through clip TimeMap to source_time";
