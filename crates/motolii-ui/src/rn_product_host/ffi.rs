//! RN 向け C ABI。handle と JSON だけを出し、意味は registry へ渡す。

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
use super::wire_io::*;
#[cfg(target_os = "macos")]
use super::MAX_PROJECT_PATH_BYTES;
use super::{
    HOST_TO_RN, MAX_DIAGNOSTICS, MAX_EFFECTS_PER_LAYER, MAX_JSON_BYTES, MAX_POSITION_KEYS,
    MAX_SNAPSHOT_JSON_BYTES, MAX_SOURCE_PARAMS_PER_LAYER, MAX_STAGE_BOUNDS, MAX_STAGE_SELECTION,
    PRODUCT_ROLE, RN_TO_HOST, WIRE_VERSION,
};
#[cfg(target_os = "macos")]
#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_host_create(
    path: *const u8,
    path_len: usize,
    out_host_handle: *mut u64,
    out: *mut u8,
    out_cap: usize,
) -> i64 {
    if out_host_handle.is_null() {
        return -1;
    }
    unsafe {
        *out_host_handle = 0;
    }
    if !output_usable(out, out_cap) {
        return -1;
    }
    catch_unwind(AssertUnwindSafe(|| {
        let project_path = match read_utf8(path, path_len, MAX_PROJECT_PATH_BYTES) {
            Ok(value) if !value.is_empty() => value,
            Ok(_) => {
                return write_reject(
                    out,
                    out_cap,
                    RnHostReasonCode::InvalidProjectPath,
                    None,
                    None,
                );
            }
            Err(error) => {
                return match map_create_error(&error) {
                    Some(reason) => write_reject(out, out_cap, reason, None, None),
                    None => -1,
                };
            }
        };

        let created = with_registry(|registry| registry.create_host(Path::new(&project_path)));
        match created {
            Ok(host_handle) => {
                let encoded = with_registry(|registry| {
                    let snapshot = registry.read_snapshot(host_handle)?;
                    encode_response(&accept(snapshot))
                });
                match encoded {
                    Ok(json) => {
                        let written = write_bytes(out, out_cap, &json);
                        if written <= 0 {
                            let _ = with_registry(|registry| registry.destroy_host(host_handle));
                            return written;
                        }
                        unsafe {
                            *out_host_handle = host_handle;
                        }
                        written
                    }
                    Err(_) => {
                        let _ = with_registry(|registry| registry.destroy_host(host_handle));
                        -1
                    }
                }
            }
            Err(error) => match map_create_error(&error) {
                Some(reason) => write_reject(out, out_cap, reason, None, None),
                None => -1,
            },
        }
    }))
    .unwrap_or(-1)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_host_destroy(host_handle: u64, out: *mut u8, out_cap: usize) -> i64 {
    if !output_usable(out, out_cap) {
        return -1;
    }
    catch_unwind(AssertUnwindSafe(|| {
        if host_handle == 0 {
            return write_reject(
                out,
                out_cap,
                RnHostReasonCode::UnknownHostHandle,
                Some(0),
                None,
            );
        }
        let outcome = with_registry(|registry| {
            if !registry.hosts.contains_key(&host_handle) {
                return if registry.destroyed_hosts.contains(&host_handle) {
                    Ok(Err(RnHostError::DestroyedHost(host_handle)))
                } else {
                    Ok(Err(RnHostError::UnknownHost(host_handle)))
                };
            }
            let json = encode_response(&accept_no_snapshot())?;
            if json.len() > out_cap {
                return Err(RnHostError::PayloadTooLarge);
            }
            registry.destroy_host(host_handle)?;
            Ok(Ok(json))
        });
        match outcome {
            Ok(Ok(json)) => write_bytes(out, out_cap, &json),
            Ok(Err(error)) => match map_destroy_host_error(&error) {
                Some(reason) => write_reject(out, out_cap, reason, Some(host_handle), None),
                None => -1,
            },
            Err(_) => -1,
        }
    }))
    .unwrap_or(-1)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_host_read_snapshot_json(
    host_handle: u64,
    out: *mut u8,
    out_cap: usize,
) -> i64 {
    if !output_usable(out, out_cap) {
        return -1;
    }
    catch_unwind(AssertUnwindSafe(|| {
        if host_handle == 0 {
            return write_reject(
                out,
                out_cap,
                RnHostReasonCode::UnknownHostHandle,
                Some(0),
                None,
            );
        }
        match with_registry(|registry| registry.read_snapshot(host_handle)) {
            Ok(snapshot) => match encode_snapshot_json(&snapshot) {
                Ok(json) => write_bytes(out, out_cap, &json),
                Err(_) => -1,
            },
            Err(error) => match map_host_lookup_error(&error) {
                Some(reason) => write_reject(out, out_cap, reason, Some(host_handle), None),
                None => -1,
            },
        }
    }))
    .unwrap_or(-1)
}

/// 軽量stamp: snapshot JSONが変わり得る変更で必ず revision か generation が動く。
/// serialize禁止。registry lock下で2つのu64を書くだけ(F9 / B7)。
#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_host_projection_stamp(
    handle: u64,
    out_revision: *mut u64,
    out_generation: *mut u64,
) -> bool {
    if out_revision.is_null() || out_generation.is_null() || handle == 0 {
        return false;
    }
    catch_unwind(AssertUnwindSafe(|| {
        let mut guard = lock_registry();
        let Some(host) = guard.hosts.get_mut(&handle) else {
            return false;
        };
        if host.destroyed {
            return false;
        }
        // Stage の毎frame stamp が再生時計。Transport は聴感時刻なので二重呼び出しでも進まない。
        host.pump_playback();
        // SAFETY: 呼び出し側がwritableな非nullポインタを渡す契約。
        unsafe {
            *out_revision = host.runtime.document_revision();
            *out_generation = host.projection_generation;
        }
        true
    }))
    .unwrap_or(false)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_host_dispatch_intent_json(
    host_handle: u64,
    intent_ptr: *const u8,
    intent_len: usize,
    out: *mut u8,
    out_cap: usize,
) -> i64 {
    if !output_usable(out, out_cap) {
        return -1;
    }
    catch_unwind(AssertUnwindSafe(|| {
        if host_handle == 0 {
            return write_reject(
                out,
                out_cap,
                RnHostReasonCode::UnknownHostHandle,
                Some(0),
                None,
            );
        }
        let intent_json = match read_utf8(intent_ptr, intent_len, MAX_JSON_BYTES) {
            Ok(value) => value,
            Err(RnHostError::InvalidUtf8) | Err(RnHostError::PayloadTooLarge) => {
                return write_reject(
                    out,
                    out_cap,
                    RnHostReasonCode::InvalidIntent,
                    Some(host_handle),
                    None,
                );
            }
            Err(_) => return -1,
        };
        match with_registry(|registry| registry.dispatch_intent_json(host_handle, &intent_json)) {
            Ok(response) => write_bytes(out, out_cap, &response),
            Err(error) => match map_host_lookup_error(&error) {
                Some(reason) => write_reject(out, out_cap, reason, Some(host_handle), None),
                None if matches!(error, RnHostError::Serialize(_)) => write_reject(
                    out,
                    out_cap,
                    RnHostReasonCode::InvalidIntent,
                    Some(host_handle),
                    None,
                ),
                None => -1,
            },
        }
    }))
    .unwrap_or(-1)
}

#[cfg(target_os = "macos")]
#[allow(dead_code)]
const _: fn() = || {
    let _ =
        motolii_rn_host_create as extern "C" fn(*const u8, usize, *mut u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_host_destroy as extern "C" fn(u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_stage_register as extern "C" fn(u64, *mut u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_stage_destroy as extern "C" fn(u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_timeline_register as extern "C" fn(u64, *mut u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_timeline_destroy as extern "C" fn(u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_host_read_snapshot_json as extern "C" fn(u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_host_projection_stamp as extern "C" fn(u64, *mut u64, *mut u64) -> bool;
    let _ = motolii_rn_host_dispatch_intent_json
        as extern "C" fn(u64, *const u8, usize, *mut u8, usize) -> i64;
    let _ = motolii_rn_stage_attach
        as extern "C" fn(u64, u64, *mut core::ffi::c_void, *mut u8, usize) -> i64;
    let _ = motolii_rn_stage_resize_physical
        as extern "C" fn(u64, u64, u32, u32, *mut u8, usize) -> i64;
    let _ = motolii_rn_stage_draw as extern "C" fn(u64, u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_stage_detach as extern "C" fn(u64, u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_timeline_attach
        as extern "C" fn(u64, u64, *mut core::ffi::c_void, *mut u8, usize) -> i64;
    let _ = motolii_rn_timeline_resize_physical
        as extern "C" fn(u64, u64, u32, u32, *mut u8, usize) -> i64;
    let _ = motolii_rn_timeline_draw as extern "C" fn(u64, u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_timeline_detach as extern "C" fn(u64, u64, *mut u8, usize) -> i64;
};
