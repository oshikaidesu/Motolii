//! D3変換層の評価時刻意味論(#55)。
use motolii_core::RationalTime;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvaluationTime { pub timeline_time: RationalTime }
impl EvaluationTime { pub fn new(timeline_time: RationalTime) -> Self { Self { timeline_time } } }
pub const M1_SOURCE_PTS_EQUALS_TIMELINE: &str = "M1 export used timeline_time as source PTS (identity TimeMap degeneracy)";
pub const D3_CLIP_LOCAL_TO_SOURCE_VIA_TIMEMAP: &str = "D3 maps (timeline_time - clip.start) through clip TimeMap to source_time";
