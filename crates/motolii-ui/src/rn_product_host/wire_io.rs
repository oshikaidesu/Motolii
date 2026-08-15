//! JSON 符号化と typed reject。registry/FFI が同じ口を使う。

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
use super::wire::*;
#[cfg(target_os = "macos")]
use super::MAX_PROJECT_PATH_BYTES;
use super::{
    HOST_TO_RN, MAX_DIAGNOSTICS, MAX_EFFECTS_PER_LAYER, MAX_JSON_BYTES, MAX_POSITION_KEYS,
    MAX_SNAPSHOT_JSON_BYTES, MAX_SOURCE_PARAMS_PER_LAYER, MAX_STAGE_BOUNDS, MAX_STAGE_SELECTION,
    PRODUCT_ROLE, RN_TO_HOST, WIRE_VERSION,
};

pub(super) fn encode_json<T: Serialize>(value: &T) -> Result<String, RnHostError> {
    let json = serde_json::to_string(value)?;
    if json.len() > MAX_JSON_BYTES {
        return Err(RnHostError::PayloadTooLarge);
    }
    Ok(json)
}

pub(super) fn encode_snapshot_json<T: Serialize>(snapshot: &T) -> Result<String, RnHostError> {
    let json = serde_json::to_string(snapshot)?;
    if json.len() > MAX_SNAPSHOT_JSON_BYTES {
        return Err(RnHostError::PayloadTooLarge);
    }
    Ok(json)
}

pub(super) fn diagnostic(
    reason: RnHostReasonCode,
    host_handle: Option<u64>,
    stage_handle: Option<u64>,
    expected_projection_generation: Option<String>,
    actual_projection_generation: Option<String>,
) -> RnHostDiagnostic {
    RnHostDiagnostic {
        reason,
        host_handle: host_handle.map(|value| value.to_string()),
        stage_handle: stage_handle.map(|value| value.to_string()),
        timeline_handle: None,
        expected_projection_generation,
        actual_projection_generation,
    }
}

pub(super) fn document_runtime_reason(error: &DocumentEditRuntimeError) -> RnHostReasonCode {
    match error {
        DocumentEditRuntimeError::ProjectionGenerationExhausted => {
            RnHostReasonCode::ProjectionGenerationExhausted
        }
        DocumentEditRuntimeError::NonFiniteDropPosition => RnHostReasonCode::NonFiniteDropPosition,
        DocumentEditRuntimeError::PlayheadOutsideComposition => {
            RnHostReasonCode::PlayheadOutsideComposition
        }
        DocumentEditRuntimeError::RemainingDurationBelowOneFrame => {
            RnHostReasonCode::RemainingDurationBelowOneFrame
        }
        DocumentEditRuntimeError::NoTrackForRectangle => RnHostReasonCode::NoTrackForRectangle,
        DocumentEditRuntimeError::LayerIdReservationChanged => {
            RnHostReasonCode::LayerIdReservationChanged
        }
        DocumentEditRuntimeError::LayerId(_) => RnHostReasonCode::LayerIdError,
        DocumentEditRuntimeError::RationalTime(_) => RnHostReasonCode::RationalTimeError,
        DocumentEditRuntimeError::Document(_) => RnHostReasonCode::DocumentError,
        DocumentEditRuntimeError::DocumentPlugin(_) => RnHostReasonCode::DocumentPluginError,
        DocumentEditRuntimeError::JournalCommit(_) => RnHostReasonCode::JournalCommit,
        DocumentEditRuntimeError::DocumentWriteBlocked { .. }
        | DocumentEditRuntimeError::CommitReceiptNotObserved { .. }
        | DocumentEditRuntimeError::ReconciledDocumentMismatch { .. } => {
            RnHostReasonCode::DocumentWriteBlocked
        }
        DocumentEditRuntimeError::JournalReconcile { .. } => RnHostReasonCode::JournalReconcile,
        DocumentEditRuntimeError::MissingJournalCommitReceipt => RnHostReasonCode::JournalCommit,
        DocumentEditRuntimeError::Command(_) => RnHostReasonCode::CommandError,
        DocumentEditRuntimeError::Undo(_) => RnHostReasonCode::CommandError,
        DocumentEditRuntimeError::SelectionTargetNotFound(_)
        | DocumentEditRuntimeError::NoPrimarySelection
        | DocumentEditRuntimeError::PrepareRejected
        | DocumentEditRuntimeError::HistoryProjectionMismatch
        | DocumentEditRuntimeError::MultiCommandActionRejected
        | DocumentEditRuntimeError::LibraryFileUnreadable
        | DocumentEditRuntimeError::AttachDefaultNotConst { .. }
        | DocumentEditRuntimeError::AttachPrepareCommandMismatch
        | DocumentEditRuntimeError::PositionKeyPrepareMismatch
        | DocumentEditRuntimeError::EffectPrepare(_)
        | DocumentEditRuntimeError::PositionKeyPrepare(_)
        | DocumentEditRuntimeError::TransformParamKeyPrepare(_)
        | DocumentEditRuntimeError::RemovePositionKeyPrepare(_)
        | DocumentEditRuntimeError::Duplicate(_)
        | DocumentEditRuntimeError::NothingToUndo
        | DocumentEditRuntimeError::NothingToRedo => RnHostReasonCode::InvalidIntent,
    }
}

pub(super) fn timeline_diagnostic(
    reason: RnHostReasonCode,
    host_handle: Option<u64>,
    timeline_handle: Option<u64>,
) -> RnHostDiagnostic {
    RnHostDiagnostic {
        reason,
        host_handle: host_handle.map(|value| value.to_string()),
        stage_handle: None,
        timeline_handle: timeline_handle.map(|value| value.to_string()),
        expected_projection_generation: None,
        actual_projection_generation: None,
    }
}

pub(super) fn accept(snapshot: WireProductSnapshot) -> WireIntentResponse {
    WireIntentResponse {
        version: WIRE_VERSION,
        accepted: true,
        snapshot: Some(snapshot),
        diagnostics: Vec::new(),
        message: None,
    }
}

pub(super) fn reject(
    diagnostic: RnHostDiagnostic,
    snapshot: Option<WireProductSnapshot>,
) -> WireIntentResponse {
    let mut diagnostics = vec![diagnostic];
    diagnostics.truncate(MAX_DIAGNOSTICS);
    WireIntentResponse {
        version: WIRE_VERSION,
        accepted: false,
        snapshot,
        diagnostics,
        message: None,
    }
}

pub(super) fn with_message(
    mut response: WireIntentResponse,
    message: impl Into<String>,
) -> WireIntentResponse {
    response.message = Some(message.into());
    response
}

#[cfg(target_os = "macos")]
pub(super) fn write_bytes(out: *mut u8, out_cap: usize, payload: &str) -> i64 {
    if out.is_null() || out_cap == 0 {
        return -1;
    }
    let bytes = payload.as_bytes();
    if bytes.len() > out_cap {
        return -(bytes.len() as i64);
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, bytes.len());
    }
    bytes.len() as i64
}

#[cfg(target_os = "macos")]
pub(super) fn output_usable(out: *mut u8, out_cap: usize) -> bool {
    !out.is_null() && out_cap > 0
}

#[cfg(target_os = "macos")]
pub(super) fn accept_no_snapshot() -> WireIntentResponse {
    WireIntentResponse {
        version: WIRE_VERSION,
        accepted: true,
        snapshot: None,
        diagnostics: Vec::new(),
        message: None,
    }
}

#[cfg(target_os = "macos")]
pub(super) fn encode_response(response: &WireIntentResponse) -> Result<String, RnHostError> {
    encode_snapshot_json(response)
}

#[cfg(target_os = "macos")]
pub(super) fn write_response(out: *mut u8, out_cap: usize, response: &WireIntentResponse) -> i64 {
    match encode_response(response) {
        Ok(json) => write_bytes(out, out_cap, &json),
        Err(_) => -1,
    }
}

#[cfg(target_os = "macos")]
pub(super) fn write_reject(
    out: *mut u8,
    out_cap: usize,
    reason: RnHostReasonCode,
    host_handle: Option<u64>,
    stage_handle: Option<u64>,
) -> i64 {
    write_response(
        out,
        out_cap,
        &reject(
            diagnostic(reason, host_handle, stage_handle, None, None),
            None,
        ),
    )
}

#[cfg(target_os = "macos")]
pub(super) fn map_create_error(error: &RnHostError) -> Option<RnHostReasonCode> {
    match error {
        RnHostError::HostAlreadyExists => Some(RnHostReasonCode::HostAlreadyExists),
        RnHostError::OpenProject(ShellError::ProjectSession(
            motolii_doc::SessionError::ProjectAlreadyOpen,
        )) => Some(RnHostReasonCode::ProjectAlreadyOpen),
        RnHostError::EmptyProjectPath | RnHostError::InvalidUtf8 | RnHostError::OpenProject(_) => {
            Some(RnHostReasonCode::InvalidProjectPath)
        }
        _ => None,
    }
}

#[cfg(target_os = "macos")]
pub(super) fn map_host_lookup_error(error: &RnHostError) -> Option<RnHostReasonCode> {
    match error {
        RnHostError::UnknownHost(_) => Some(RnHostReasonCode::UnknownHostHandle),
        RnHostError::DestroyedHost(_) => Some(RnHostReasonCode::DestroyedHostHandle),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
pub(super) fn map_destroy_host_error(error: &RnHostError) -> Option<RnHostReasonCode> {
    match error {
        RnHostError::UnknownHost(_) => Some(RnHostReasonCode::UnknownHostHandle),
        RnHostError::DestroyedHost(_) => Some(RnHostReasonCode::DoubleDestroy),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
pub(super) fn map_destroy_stage_error(error: &RnHostError) -> Option<RnHostReasonCode> {
    match error {
        RnHostError::UnknownStage(_) => Some(RnHostReasonCode::UnknownStageHandle),
        RnHostError::DestroyedStage(_) => Some(RnHostReasonCode::DoubleDestroy),
        RnHostError::UnknownHost(_) => Some(RnHostReasonCode::UnknownHostHandle),
        RnHostError::DestroyedHost(_) => Some(RnHostReasonCode::DestroyedHostHandle),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
pub(super) fn map_destroy_timeline_error(error: &RnHostError) -> Option<RnHostReasonCode> {
    match error {
        RnHostError::UnknownTimeline(_) => Some(RnHostReasonCode::UnknownTimelineHandle),
        RnHostError::DestroyedTimeline(_) => Some(RnHostReasonCode::DoubleDestroy),
        RnHostError::UnknownHost(_) => Some(RnHostReasonCode::UnknownHostHandle),
        RnHostError::DestroyedHost(_) => Some(RnHostReasonCode::DestroyedHostHandle),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
pub(super) fn read_utf8(ptr: *const u8, len: usize, max_len: usize) -> Result<String, RnHostError> {
    if ptr.is_null() || len == 0 {
        return Err(RnHostError::InvalidUtf8);
    }
    if len > max_len {
        return Err(RnHostError::PayloadTooLarge);
    }
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    std::str::from_utf8(slice)
        .map(ToOwned::to_owned)
        .map_err(|_| RnHostError::InvalidUtf8)
}
