//! Wave R0: product-private React Native Host seam.
//!
//! DocumentEditRuntime を単一 writer として保持し、revision 付き read-only snapshot と
//! lifecycle/read intent だけを RN へ投影する。
//! 責任は各moduleが持ち、ここは組み立てと定数だけを行う。

mod app_api;
mod dispatch;
mod error;
mod ffi;
mod ffi_surface;
mod gpu_draw;
mod gpu_ops;
mod gpu_surface;
mod host;
mod key_projection;
mod playback;
mod projection;
mod registry;
mod stage_projection;
mod surfaces;
mod timeline_gpu;
mod wire;
mod wire_io;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_clip;
#[cfg(test)]
mod tests_effects_catalog;
#[cfg(test)]
mod tests_effects_write;
#[cfg(test)]
mod tests_ffi;
#[cfg(test)]
mod tests_gpu;
#[cfg(test)]
mod tests_keys;
#[cfg(test)]
mod tests_layer;
#[cfg(test)]
mod tests_move;
#[cfg(test)]
mod tests_place;
#[cfg(test)]
mod tests_playback;
#[cfg(test)]
mod tests_pointer;
#[cfg(test)]
mod tests_render;
#[cfg(test)]
mod tests_time;
#[cfg(test)]
mod tests_vism_param;
#[cfg(test)]
mod tests_vism_place;

const WIRE_VERSION: u8 = 1;
const HOST_TO_RN: &str = "host-to-rn";
const RN_TO_HOST: &str = "rn-to-host";
const PRODUCT_ROLE: &str = "product-runtime-seat";
// ResourceLimits.production().max_layers は 100_000。ここは wire/transport truncate であり Document 天井ではない。
const MAX_STAGE_BOUNDS: usize = 64;
const MAX_STAGE_SELECTION: usize = 16;
const MAX_POSITION_KEYS: usize = 64;
const MAX_EFFECTS_PER_LAYER: usize = 8;
// Color+scalar を落とさない。f64 専用の意味天井ではない。
const MAX_SOURCE_PARAMS_PER_LAYER: usize = 16;
const MAX_DIAGNOSTICS: usize = 8;
const MAX_JSON_BYTES: usize = 16_384;
const MAX_SNAPSHOT_JSON_BYTES: usize = 131_072;
#[cfg(target_os = "macos")]
const MAX_PROJECT_PATH_BYTES: usize = 4_096;

pub use app_api::{
    host_commit_stage_transform_for_app, host_create_for_test, host_destroy_for_test,
    host_destroy_stage_for_test, host_dispatch_intent_for_test,
    host_preview_stage_transform_for_app, host_read_snapshot_for_test,
    host_register_stage_for_test, host_render_frame_for_app,
};
#[cfg(all(test, target_os = "macos"))]
pub(crate) use app_api::{stage_gpu_mark_attached_for_test, stage_gpu_state_for_test};
pub use error::{RnHostError, RnHostReasonCode};
#[cfg(target_os = "macos")]
pub use ffi::{
    motolii_rn_host_create, motolii_rn_host_destroy, motolii_rn_host_dispatch_intent_json,
    motolii_rn_host_projection_stamp, motolii_rn_host_read_snapshot_json,
};
#[cfg(target_os = "macos")]
pub use ffi_surface::{
    motolii_rn_stage_attach, motolii_rn_stage_destroy, motolii_rn_stage_detach,
    motolii_rn_stage_draw, motolii_rn_stage_register, motolii_rn_stage_resize_physical,
    motolii_rn_timeline_attach, motolii_rn_timeline_destroy, motolii_rn_timeline_detach,
    motolii_rn_timeline_draw, motolii_rn_timeline_register, motolii_rn_timeline_resize_physical,
};
#[cfg(target_os = "macos")]
pub(crate) use surfaces::TimelineFrameBorrow;
pub use surfaces::{
    AppStageFrame, AppStageGeometry, AppStageGeometryLayer, AppStageTransformEdit,
    AppStageTransformError, AppStageTransformPreview, HostRenderFrameResult,
};
pub use wire::{
    RnHostTestIntent, RnHostTestResponse, RnProductSnapshotForTest, RnTimelineLayerForTest,
    RnTimelineParamKeyForTest, RnTimelinePositionKeyForTest, RnTimelineProjectionForTest,
};
pub(crate) use wire::{WireIntentEnvelope, WireIntentResponse, WireProductSnapshot};
