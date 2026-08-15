//! Skia製Timelineのraster。
//!
//! 製品は host の Document 投影（layer / envelope区間 / position key / playhead）だけを描く。
//! `Default` は既存test用の probe fixture であり、製品初期状態には使わない。

mod draw;
mod geometry;
mod hit;
mod layout;
mod paint;
mod scene;
mod session;

pub(crate) use hit::{
    cursor_for_stage_hover, cursor_for_timeline_hover, hit_test, timeline_hover_hit, CursorKind,
    TimelineHoverHit,
};
pub(crate) use layout::{SECONDS_PER_BAR, SONG_BARS};
pub(crate) use paint::draw_timeline;
pub(crate) use scene::{
    restore_key_selection, selected_real_key, SnapshotKeyInput, SnapshotLayerInput, TimelineScene,
};
pub(crate) use session::{
    remove_position_key_commit, CursorDragKind, TimelineEditCommit, TimelinePointerOutcome,
    TimelinePointerPhase, TimelineSelectionCommit, TimelineSession,
};

#[cfg(test)]
pub(crate) use geometry::test_snap_bar;
#[cfg(test)]
pub(crate) use scene::test_select_first_real_key;

#[cfg(test)]
mod tests;
