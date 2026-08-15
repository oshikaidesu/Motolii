//! Stage/Timeline GPU op 入口。描画本体は gpu_draw。

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
use super::gpu_surface::*;
use super::host::*;
use super::projection::*;
use super::registry::*;
use super::surfaces::*;
use super::wire::*;
use super::wire_io::*;
#[cfg(target_os = "macos")]
use super::MAX_PROJECT_PATH_BYTES;
use super::{
    HOST_TO_RN, MAX_DIAGNOSTICS, MAX_EFFECTS_PER_LAYER, MAX_JSON_BYTES, MAX_POSITION_KEYS,
    MAX_SNAPSHOT_JSON_BYTES, MAX_SOURCE_PARAMS_PER_LAYER, MAX_STAGE_BOUNDS, MAX_STAGE_SELECTION,
    PRODUCT_ROLE, RN_TO_HOST, WIRE_VERSION,
};

#[cfg(target_os = "macos")]
pub(super) fn require_main_thread() -> Result<(), RnHostReasonCode> {
    if objc2::MainThreadMarker::new().is_none() {
        return Err(RnHostReasonCode::InvalidIntent);
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub(super) fn run_stage_gpu_op(
    host_handle: u64,
    stage_handle: u64,
    f: impl FnOnce(&mut RnProductHost, u64) -> Result<(), RnHostReasonCode>,
) -> Result<String, RnHostReasonCode> {
    let outcome = with_registry(|registry| {
        let host = registry.hosts.get_mut(&host_handle);
        let Some(host) = host else {
            return Ok(Err(if registry.destroyed_hosts.contains(&host_handle) {
                RnHostReasonCode::DestroyedHostHandle
            } else {
                RnHostReasonCode::UnknownHostHandle
            }));
        };
        if host.destroyed {
            return Ok(Err(RnHostReasonCode::DestroyedHostHandle));
        }
        let Some(stage) = host.stages.get(&stage_handle) else {
            return Ok(Err(if registry.destroyed_stages.contains(&stage_handle) {
                RnHostReasonCode::DestroyedStageHandle
            } else {
                RnHostReasonCode::UnknownStageHandle
            }));
        };
        if stage.host_handle != host_handle {
            return Ok(Err(RnHostReasonCode::UnknownStageHandle));
        }
        if let Err(reason) = f(host, stage_handle) {
            return Ok(Err(reason));
        }
        match encode_response(&accept_no_snapshot()) {
            Ok(json) => Ok(Ok(json)),
            Err(_) => Ok(Err(RnHostReasonCode::InvalidIntent)),
        }
    });
    match outcome {
        Ok(inner) => inner,
        Err(_) => Err(RnHostReasonCode::InvalidIntent),
    }
}

#[cfg(target_os = "macos")]
pub(super) fn write_stage_gpu_op(
    out: *mut u8,
    out_cap: usize,
    host_handle: u64,
    stage_handle: u64,
    f: impl FnOnce(&mut RnProductHost, u64) -> Result<(), RnHostReasonCode>,
) -> i64 {
    if !output_usable(out, out_cap) {
        return -1;
    }
    catch_unwind(AssertUnwindSafe(|| {
        if require_main_thread().is_err() {
            return write_reject(
                out,
                out_cap,
                RnHostReasonCode::InvalidIntent,
                Some(host_handle),
                Some(stage_handle),
            );
        }
        if host_handle == 0 {
            return write_reject(
                out,
                out_cap,
                RnHostReasonCode::UnknownHostHandle,
                Some(0),
                Some(stage_handle),
            );
        }
        if stage_handle == 0 {
            return write_reject(
                out,
                out_cap,
                RnHostReasonCode::UnknownStageHandle,
                Some(host_handle),
                Some(0),
            );
        }
        match run_stage_gpu_op(host_handle, stage_handle, f) {
            Ok(json) => write_bytes(out, out_cap, &json),
            Err(reason) => {
                write_reject(out, out_cap, reason, Some(host_handle), Some(stage_handle))
            }
        }
    }))
    .unwrap_or(-1)
}

#[cfg(target_os = "macos")]
pub(super) fn run_timeline_gpu_op(
    host_handle: u64,
    timeline_handle: u64,
    f: impl FnOnce(&mut RnProductHost, u64) -> Result<(), RnHostReasonCode>,
) -> Result<String, RnHostReasonCode> {
    let outcome = with_registry(|registry| {
        let Some(host) = registry.hosts.get_mut(&host_handle) else {
            return Ok(Err(if registry.destroyed_hosts.contains(&host_handle) {
                RnHostReasonCode::DestroyedHostHandle
            } else {
                RnHostReasonCode::UnknownHostHandle
            }));
        };
        if host.destroyed {
            return Ok(Err(RnHostReasonCode::DestroyedHostHandle));
        }
        let Some(timeline) = host.timelines.get(&timeline_handle) else {
            return Ok(Err(
                if registry.destroyed_timelines.contains(&timeline_handle) {
                    RnHostReasonCode::DestroyedTimelineHandle
                } else {
                    RnHostReasonCode::UnknownTimelineHandle
                },
            ));
        };
        if timeline.host_handle != host_handle {
            return Ok(Err(RnHostReasonCode::UnknownTimelineHandle));
        }
        if let Err(reason) = f(host, timeline_handle) {
            return Ok(Err(reason));
        }
        match encode_response(&accept_no_snapshot()) {
            Ok(json) => Ok(Ok(json)),
            Err(_) => Ok(Err(RnHostReasonCode::InvalidIntent)),
        }
    });
    match outcome {
        Ok(inner) => inner,
        Err(_) => Err(RnHostReasonCode::InvalidIntent),
    }
}

#[cfg(target_os = "macos")]
pub(super) fn write_timeline_gpu_op(
    out: *mut u8,
    out_cap: usize,
    host_handle: u64,
    timeline_handle: u64,
    f: impl FnOnce(&mut RnProductHost, u64) -> Result<(), RnHostReasonCode>,
) -> i64 {
    if !output_usable(out, out_cap) {
        return -1;
    }
    catch_unwind(AssertUnwindSafe(|| {
        if require_main_thread().is_err() {
            return write_response(
                out,
                out_cap,
                &reject(
                    timeline_diagnostic(
                        RnHostReasonCode::InvalidIntent,
                        Some(host_handle),
                        Some(timeline_handle),
                    ),
                    None,
                ),
            );
        }
        if host_handle == 0 || timeline_handle == 0 {
            return write_response(
                out,
                out_cap,
                &reject(
                    timeline_diagnostic(
                        if host_handle == 0 {
                            RnHostReasonCode::UnknownHostHandle
                        } else {
                            RnHostReasonCode::UnknownTimelineHandle
                        },
                        Some(host_handle),
                        Some(timeline_handle),
                    ),
                    None,
                ),
            );
        }
        match run_timeline_gpu_op(host_handle, timeline_handle, f) {
            Ok(json) => write_bytes(out, out_cap, &json),
            Err(reason) => write_response(
                out,
                out_cap,
                &reject(
                    timeline_diagnostic(reason, Some(host_handle), Some(timeline_handle)),
                    None,
                ),
            ),
        }
    }))
    .unwrap_or(-1)
}
