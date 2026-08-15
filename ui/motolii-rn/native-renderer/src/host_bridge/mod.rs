//! RN app ↔ RnProductHost 接続の単一owner。
//!
//! processに最大1 host。ObjC/RNは薄いcarrierとしてここへ委譲する。

mod dispatch;
mod ffi;
mod json_scan;
mod keymap;
mod lifecycle;
mod parse_catalog;
mod parse_wire;
mod projection;
mod slot;
mod stage;
mod terminal;
mod types;

#[cfg(all(test, target_os = "macos"))]
mod tests;

pub use ffi::{
    motolii_rnapp_commit_stage_transform, motolii_rnapp_host_dispatch_json,
    motolii_rnapp_host_snapshot_json,
};
pub use keymap::{motolii_rnapp_host_key_event, motolii_rnapp_host_keymap};
pub use lifecycle::{motolii_rnapp_host_ensure, motolii_rnapp_host_shutdown};
pub use stage::{
    motolii_rnapp_is_timeline_interacting, motolii_rnapp_stage_mount, motolii_rnapp_stage_pointer,
    motolii_rnapp_stage_resize, motolii_rnapp_stage_unmount,
};

pub(crate) use dispatch::{
    dispatch_commit_stage_transform, try_commit_stage_transform, try_dispatch_keymap,
    try_dispatch_move_layer_by, try_dispatch_remove_position_key, try_dispatch_set_time,
    try_dispatch_timeline_edit, try_dispatch_timeline_selection, try_preview_stage_transform,
    try_timeline_keymap_delete,
};
pub(crate) use parse_catalog::parse_catalog_projection;
pub(crate) use parse_wire::snapshot_layers_from_projection;
pub(crate) use projection::{
    frame_from_scrub_bar, playhead_from_current_time, rational_time_parts_from_bar,
    try_read_projection_stamp, try_read_timeline_projection,
};
pub(crate) use slot::{
    host_slot_present, is_timeline_interacting, set_timeline_interacting, try_host_handle,
};
pub(crate) use stage::{
    try_stage_logical_size, try_stage_mount, try_stage_pointer, try_stage_resize, try_stage_unmount,
};
pub(crate) use types::{
    HostCatalogEffect, HostCatalogProjection, HostCatalogSource, HostStageGeometry,
    HostStageGeometryLayer, HostTerminalDiagnostic, HostTerminalResult, HostTimelineEffect,
    HostTimelineEffectParam, HostTimelineKey, HostTimelineLayer, HostTimelineProjection,
    HostTimelineSourceParam,
};

#[cfg(test)]
pub(crate) use slot::{
    test_clear_host_slot, test_keymap_delete_layer_count, test_keymap_remove_position_key_count,
    test_move_layer_by_dispatch_count, test_reset_keymap_dispatch_counts,
    test_reset_move_layer_by_dispatch_count, test_reset_snapshot_read_count,
    test_reset_timeline_selection_dispatch_count, test_snapshot_read_count,
    test_timeline_selection_dispatch_count,
};
