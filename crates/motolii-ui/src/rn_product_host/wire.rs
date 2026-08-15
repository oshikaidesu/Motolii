//! RN wire 型と test DTO。snapshot/intent の形だけを持つ。

use std::collections::{HashMap, HashSet};
#[cfg(target_os = "macos")]
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use motolii_audio::{AudioProgram, PcmCache, CANONICAL_SAMPLE_RATE};
use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat, PixelSize, Quality, RationalTime};
use motolii_doc::{
    build_document_frame_graph, layer_names_for_item, prepare_set_transform_param_key_value,
    Affine2D, Clip, ClipSource, Command, DocParam, DocValue, EffectId, EvaluationTime,
    ItemEnvelope, KeyframeId, LayerId, ParentLocator, ResolvedLayerParams, ScalarPropertyId,
    TrackItem,
};
use motolii_eval::{DataTracks, Interp};
use motolii_export::{export_document_video, ExportJob, VideoSourceBinder};
use motolii_gpu::GpuCtx;
use motolii_plugins_firstparty::{first_party_catalog, first_party_runtime};
use motolii_render::{
    render_graph_cached, validate_render_graph_wiring, RenderGraphInputs, RenderSession,
};
use motolii_transport::PlaybackSession;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(target_os = "macos")]
use wgpu::{
    Color, CompositeAlphaMode, CurrentSurfaceTexture, LoadOp, Operations, PresentMode,
    RenderPassColorAttachment, RenderPassDescriptor, StoreOp, Surface, SurfaceConfiguration,
    SurfaceTargetUnsafe, TextureFormat, TextureUsages,
};

use crate::document_edit_runtime::{
    prepare_set_effect_param_command, prepare_set_source_param_command, AddPositionKeyRequest,
    AddTransformParamKeyRequest, AttachEffectRequest, DocumentEditQueue, DocumentEditRuntime,
    DocumentEditRuntimeError, PlaceEllipseRequest, PlaceMediaRequest, PlaceRectangleRequest,
    PlaceVismRequest, PublishedDocument, RemovePositionKeyRequest, SetEffectParamRequest,
    SetOpacityRequest, SetPositionConstRequest, SetPositionKeyInterpRequest,
    SetPositionKeyTimeRequest, SetPositionKeyValueRequest, SetSourceParamRequest,
};
use crate::media_library::{LibraryProjection, MediaLibrary};
use crate::shell::{open_project_runtime, ShellError};
use crate::stage_geometry_projection::{project_stage_geometry, StageLayerProjection};
use crate::stage_hit_test::{
    hit_test_projected_layers, view_local_in_stage, view_local_to_canonical, StageHit,
    StageHitTestReject,
};
use crate::stage_overlay_gpu::{overlay_dimensions_match, overlay_dirty, OverlayUploadKey};
use crate::stage_overlay_raster::{raster_selection_outline, StageOverlayRaster};
#[cfg(target_os = "macos")]
use crate::static_preview::{bootstrap_frame_desc, prepare_in_setup_worker, StaticPreview};
use crate::timeline_move_gesture::TimelineMoveRequest;
use crate::timeline_trim_gesture::TimelineTrimRequest;
use crate::{CommandId, DocumentCommandRequest, DomainIntent, InputPhase, RouterOutput};

use super::app_api::*;
use super::dispatch::*;
use super::error::*;
#[cfg(target_os = "macos")]
use super::ffi::*;
use super::ffi_surface::*;
use super::gpu_draw::*;
use super::gpu_ops::*;
use super::gpu_surface::*;
use super::host::*;
use super::key_projection::*;
use super::playback::*;
use super::projection::*;
use super::registry::*;
use super::stage_projection::*;
use super::surfaces::*;
use super::timeline_gpu::*;
use super::wire_io::*;
#[cfg(target_os = "macos")]
use super::MAX_PROJECT_PATH_BYTES;
use super::{
    HOST_TO_RN, MAX_DIAGNOSTICS, MAX_EFFECTS_PER_LAYER, MAX_JSON_BYTES, MAX_POSITION_KEYS,
    MAX_SNAPSHOT_JSON_BYTES, MAX_SOURCE_PARAMS_PER_LAYER, MAX_STAGE_BOUNDS, MAX_STAGE_SELECTION,
    PRODUCT_ROLE, RN_TO_HOST, WIRE_VERSION,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct RnHostDiagnostic {
    pub(super) reason: RnHostReasonCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) host_handle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) stage_handle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) timeline_handle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) expected_projection_generation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) actual_projection_generation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct WireStageBound {
    pub(super) layer_id: String,
    pub(super) display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct WireStageSelection {
    pub(super) layer_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct WireProductSnapshot {
    pub(super) version: u8,
    pub(super) direction: String,
    pub(super) role: String,
    pub(super) host_handle: String,
    pub(super) revision: String,
    pub(super) projection_generation: String,
    /// Host transient の現在評価時刻。Document / revision には載せない。
    pub(super) current_time: RationalTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) primary_layer_id: Option<String>,
    /// Undo/Redo入口の文脈無効化用。NothingToUndo/Redoの事前投影。
    #[serde(default)]
    pub(super) history: WireHistoryProjection,
    /// 各capで隠れた件数の合計。`(+)→(+N)`表示用。
    #[serde(default)]
    pub(super) truncated_total: u32,
    pub(super) stage: WireStageProjection,
    /// Available layer の world 適用済み canonical corners（v1・camera 不使用）。
    pub(super) stage_geometry: WireStageGeometryProjection,
    pub(super) timeline: WireTimelineProjection,
    /// session 不変の first-party effect catalog（reference を除く製品 plugin）。
    pub(super) catalog: WireCatalogProjection,
    /// Workspace media library。catalog / Document Asset とは別 owner。
    #[serde(default)]
    pub(super) library: WireLibraryProjection,
    /// 選択 layer の Document DocParams。未選択時は欠落。transform 数値は stage_geometry。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) selected_doc_params: Option<WireSelectedDocParams>,
    pub(super) diagnostics: Vec<RnHostDiagnostic>,
    /// idleは既存snapshot JSONを変えない。playingの時だけ出す。
    #[serde(default, skip_serializing_if = "playback_state_is_idle")]
    pub(super) playback_state: WirePlaybackState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub(super) enum WirePlaybackState {
    #[default]
    Idle,
    Playing,
}

pub(super) fn playback_state_is_idle(state: &WirePlaybackState) -> bool {
    *state == WirePlaybackState::Idle
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub(super) struct WireHistoryProjection {
    pub(super) can_undo: bool,
    pub(super) can_redo: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct WireCatalogProjection {
    pub(super) effects: Vec<WireCatalogEffect>,
    #[serde(default)]
    pub(super) sources: Vec<WireCatalogSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct WireCatalogEffect {
    pub(super) plugin_id: String,
    pub(super) name: String,
    pub(super) effect_version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct WireCatalogSource {
    pub(super) plugin_id: String,
    pub(super) name: String,
    pub(super) effect_version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub(super) struct WireLibraryProjection {
    pub(super) root: Option<WireLibraryRoot>,
    pub(super) directories: Vec<WireLibraryDirectory>,
    pub(super) tags: Vec<WireLibraryTag>,
    pub(super) items: Vec<WireLibraryItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct WireLibraryRoot {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct WireLibraryDirectory {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct WireLibraryTag {
    pub(super) id: String,
    pub(super) label: String,
    pub(super) count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct WireLibraryItem {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) kind: String,
    pub(super) directory: String,
    pub(super) tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct WireStageGeometryProjection {
    pub(super) layers: Vec<WireStageGeometryLayer>,
    pub(super) layers_truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct WireStageGeometryLayer {
    pub(super) layer_id: String,
    /// CCW・local rect 左下起点。world 適用済み canonical。
    pub(super) corners: [[f64; 2]; 4],
    /// Document world affine の並進（local center を写した点）。角平均ではない。
    pub(super) position: [f64; 2],
    /// Document world affine の回転（ラジアン）。
    pub(super) rotation: f64,
    /// Document world affine のスケール。
    pub(super) scale: [f64; 2],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct WireStageProjection {
    pub(super) selection: Vec<WireStageSelection>,
    pub(super) bounds: Vec<WireStageBound>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct WireTimelineProjection {
    pub(super) fps: Fps,
    pub(super) layers: Vec<WireTimelineLayer>,
    pub(super) duration: RationalTime,
    pub(super) layers_truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct WireTimelineLayer {
    pub(super) layer_id: String,
    pub(super) display_name: String,
    pub(super) start: RationalTime,
    pub(super) duration: RationalTime,
    pub(super) position_keys: Vec<WireTimelinePositionKey>,
    #[serde(default)]
    pub(super) param_keys: Vec<WireTimelineParamKey>,
    pub(super) keys_truncated: bool,
    /// layer の effect 使用列。f64 / Color Const。cap 超過は truncated。
    pub(super) effects: Vec<WireTimelineEffect>,
    pub(super) effects_truncated: bool,
    /// ClipSource::Plugin の Const params（f64 と Color）。cap 超過は truncated。
    #[serde(default)]
    pub(super) source_params: Vec<WireTimelineSourceParam>,
    #[serde(default)]
    pub(super) source_params_truncated: bool,
    #[serde(default = "wire_default_true")]
    pub(super) visible: bool,
    #[serde(default)]
    pub(super) solo: bool,
}

pub(super) fn wire_default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct WireTimelineEffect {
    pub(super) effect_use_id: String,
    pub(super) plugin_id: String,
    pub(super) params: Vec<WireTimelineEffectParam>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct WireTimelineEffectParam {
    pub(super) param_id: String,
    pub(super) value: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) color: Option<[f64; 4]>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct WireTimelineSourceParam {
    pub(super) param_id: String,
    pub(super) value: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) color: Option<[f64; 4]>,
}

/// 選択 layer の envelope / effect / source DocParams。平行 transform は持たない。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct WireSelectedDocParams {
    pub(super) layer_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) opacity: Option<f64>,
    #[serde(default)]
    pub(super) effects: Vec<WireTimelineEffect>,
    #[serde(default)]
    pub(super) source_params: Vec<WireTimelineSourceParam>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct WireTimelinePositionKey {
    pub(super) key_id: String,
    pub(super) time: RationalTime,
    /// DocValue::Vec2 のみ投影。他型は field 欠落(編集不可)。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) value: Option<[f64; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) interp: Option<Interp>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct WireTimelineParamKey {
    pub(super) property: String,
    pub(super) key_id: String,
    pub(super) time: RationalTime,
    /// rotation / opacity の F64。scale では欠落。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) value: Option<f64>,
    /// scale の Vec2。rotation / opacity では欠落。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) vec: Option<[f64; 2]>,
}

#[derive(Debug, Clone, PartialEq)]
#[doc(hidden)]
pub struct RnProductSnapshotForTest {
    pub revision: String,
    pub projection_generation: String,
    pub current_time: RationalTime,
    pub primary_layer_id: Option<String>,
    pub layer_ids: Vec<String>,
    pub timeline: RnTimelineProjectionForTest,
}

#[derive(Debug, Clone, PartialEq)]
#[doc(hidden)]
pub struct RnTimelineProjectionForTest {
    pub fps: Fps,
    pub layers: Vec<RnTimelineLayerForTest>,
    pub layers_truncated: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[doc(hidden)]
pub struct RnTimelineLayerForTest {
    pub layer_id: String,
    pub display_name: String,
    pub start: RationalTime,
    pub duration: RationalTime,
    pub position_keys: Vec<RnTimelinePositionKeyForTest>,
    pub param_keys: Vec<RnTimelineParamKeyForTest>,
    pub keys_truncated: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[doc(hidden)]
pub struct RnTimelinePositionKeyForTest {
    pub key_id: String,
    pub time: RationalTime,
    pub value: Option<[f64; 2]>,
    pub interp: Option<Interp>,
}

#[derive(Debug, Clone, PartialEq)]
#[doc(hidden)]
pub struct RnTimelineParamKeyForTest {
    pub property: String,
    pub key_id: String,
    pub time: RationalTime,
    pub value: Option<f64>,
    pub vec: Option<[f64; 2]>,
}

#[derive(Debug, Clone, PartialEq)]
#[doc(hidden)]
pub struct RnHostTestIntent {
    pub kind: String,
    pub stage_handle: Option<u64>,
    pub projection_generation: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub scale_factor: Option<f64>,
    pub focused: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
#[doc(hidden)]
pub struct RnHostTestResponse {
    pub accepted: bool,
    pub reason: Option<RnHostReasonCode>,
    pub snapshot: Option<RnProductSnapshotForTest>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct WireIntentEnvelope {
    pub(super) version: u8,
    pub(super) direction: String,
    pub(super) kind: String,
    pub(super) host_handle: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) stage_handle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) projection_generation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) scale_factor: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) focused: Option<bool>,
    /// stage_pointer: `down` | `drag` | `up` | `cancel`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) phase: Option<String>,
    /// stage_pointer: view-local logical X（physical / scale_factor と混同しない）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) view_local_x: Option<f64>,
    /// stage_pointer: view-local logical Y
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) view_local_y: Option<f64>,
    /// stage_pointer: 単調増加 sequence（二重配送検出用。本seamでは調停しない）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) sequence: Option<u64>,
    /// set_time: 評価 frame index。欠落・非整数は typed 拒否。暗黙 clamp しない。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) frame: Option<i64>,
    /// place_rectangle / place_ellipse: canonical document position
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) position: Option<[f64; 2]>,
    /// place_rectangle: document playhead time
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) playhead: Option<RationalTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) dest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) key_id: Option<String>,
    /// add_param_key / set_param_key_value: `scale` | `rotation` | `opacity`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) property: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) time: Option<RationalTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) new: Option<[f64; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) interp: Option<Interp>,
    /// move_layer_by: canonical world delta
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) delta: Option<[f64; 2]>,
    /// attach_effect: catalog plugin id
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) plugin_id: Option<String>,
    /// place_media: MediaLibrary item id（Document Asset ではない）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) item_id: Option<String>,
    /// set_effect_param: EffectUse id（u64 文字列）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) effect_use_id: Option<String>,
    /// set_effect_param: definition 上の param id
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) param_id: Option<String>,
    /// set_effect_param / set_opacity / set_source_param: f64 値
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) value: Option<f64>,
    /// set_source_param: Color。Some かつ finite なら value より優先。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) color: Option<[f64; 4]>,
    /// export_document: 出力ファイル path。保存先UIはRN側。ここは既存 ExportJob へ渡すだけ。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) output_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct WireIntentResponse {
    pub(super) version: u8,
    pub(super) accepted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) snapshot: Option<WireProductSnapshot>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) diagnostics: Vec<RnHostDiagnostic>,
    /// export 等の即時理由。永続意味ではない。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) message: Option<String>,
}
