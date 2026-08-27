//! Timeline の投影(Document → 行・キー・目盛り)と積み替えの計算。
//!
//! **絵を持たない。** ここにあるのは「どの行がどこに、どの刻みで並ぶか」を
//! 決める数だけで、それを描く仕事は front(Makepad)の側にある。切り出した
//! 理由もそこにあって、`motolii-timeline-pane` に同居していた頃はこの計算を
//! 読むためだけに iced の view 層一式が依存へ付いてきていた。
//!
//! 呼び手は2人いる: Makepad front(`r7-makepad-panel`)と、凍結された iced
//! assembler。**どちらの柱にも寄らない**ことがこの crate の契約である。

pub mod projection;
pub mod stacking;

pub use projection::{
    audio_rows, frame_at_x, frame_to_x, key_order, layer_row_top, property_rows, rows,
    selected_row_index, tick_steps, tick_steps_with_target, time_band_segment_frames,
    time_band_segment_frames_with_target, waveform_bucket_range, AudioRowProjection,
    KeySelectionOp, KeySelector, PropertyKeyProjection, PropertyRowProjection, RowProjection,
    WAVEFORM_BUCKETS,
};
pub use stacking::StackDirection;
