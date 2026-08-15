//! Stage/Timeline の C ABI。Host ABI は ffi。

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
use super::gpu_draw::*;
use super::gpu_ops::*;
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
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_stage_register(
    host_handle: u64,
    out_stage_handle: *mut u64,
    out: *mut u8,
    out_cap: usize,
) -> i64 {
    if out_stage_handle.is_null() {
        return -1;
    }
    unsafe {
        *out_stage_handle = 0;
    }
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
        let registered = with_registry(|registry| registry.register_stage(host_handle));
        match registered {
            Ok(stage_handle) => {
                let encoded = with_registry(|registry| {
                    let snapshot = registry.read_snapshot(host_handle)?;
                    encode_response(&accept(snapshot))
                });
                match encoded {
                    Ok(json) => {
                        let written = write_bytes(out, out_cap, &json);
                        if written <= 0 {
                            let _ = with_registry(|registry| registry.destroy_stage(stage_handle));
                            return written;
                        }
                        unsafe {
                            *out_stage_handle = stage_handle;
                        }
                        written
                    }
                    Err(_) => {
                        let _ = with_registry(|registry| registry.destroy_stage(stage_handle));
                        -1
                    }
                }
            }
            Err(error) => match map_host_lookup_error(&error) {
                Some(reason) => write_reject(out, out_cap, reason, Some(host_handle), None),
                None => -1,
            },
        }
    }))
    .unwrap_or(-1)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_stage_destroy(stage_handle: u64, out: *mut u8, out_cap: usize) -> i64 {
    if !output_usable(out, out_cap) {
        return -1;
    }
    catch_unwind(AssertUnwindSafe(|| {
        if stage_handle == 0 {
            return write_reject(
                out,
                out_cap,
                RnHostReasonCode::UnknownStageHandle,
                None,
                Some(0),
            );
        }
        let outcome = with_registry(|registry| {
            let lookup_error = {
                let host_handle = registry.hosts.values().find_map(|host| {
                    host.stages
                        .get(&stage_handle)
                        .map(|stage| stage.host_handle)
                });
                match host_handle {
                    Some(host_handle) => {
                        let Some(host) = registry.hosts.get(&host_handle) else {
                            return Ok(Err(RnHostError::UnknownHost(host_handle)));
                        };
                        match host.stages.get(&stage_handle) {
                            Some(stage) if stage.destroyed => {
                                Some(RnHostError::DestroyedStage(stage_handle))
                            }
                            Some(_) => None,
                            None => Some(RnHostError::UnknownStage(stage_handle)),
                        }
                    }
                    None => Some(if registry.destroyed_stages.contains(&stage_handle) {
                        RnHostError::DestroyedStage(stage_handle)
                    } else {
                        RnHostError::UnknownStage(stage_handle)
                    }),
                }
            };
            if let Some(error) = lookup_error {
                return Ok(Err(error));
            }
            let json = encode_response(&accept_no_snapshot())?;
            if json.len() > out_cap {
                return Err(RnHostError::PayloadTooLarge);
            }
            registry.destroy_stage(stage_handle)?;
            Ok(Ok(json))
        });
        match outcome {
            Ok(Ok(json)) => write_bytes(out, out_cap, &json),
            Ok(Err(error)) => match map_destroy_stage_error(&error) {
                Some(reason) => write_reject(out, out_cap, reason, None, Some(stage_handle)),
                None => -1,
            },
            Err(_) => -1,
        }
    }))
    .unwrap_or(-1)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_timeline_register(
    host_handle: u64,
    out_timeline_handle: *mut u64,
    out: *mut u8,
    out_cap: usize,
) -> i64 {
    if out_timeline_handle.is_null() {
        return -1;
    }
    unsafe {
        *out_timeline_handle = 0;
    }
    if !output_usable(out, out_cap) {
        return -1;
    }
    catch_unwind(AssertUnwindSafe(|| {
        if host_handle == 0 {
            return write_response(
                out,
                out_cap,
                &reject(
                    timeline_diagnostic(RnHostReasonCode::UnknownHostHandle, Some(0), None),
                    None,
                ),
            );
        }
        match with_registry(|registry| registry.register_timeline(host_handle)) {
            Ok(timeline_handle) => {
                let encoded = with_registry(|registry| {
                    let snapshot = registry.read_snapshot(host_handle)?;
                    encode_response(&accept(snapshot))
                });
                match encoded {
                    Ok(json) => {
                        let written = write_bytes(out, out_cap, &json);
                        if written <= 0 {
                            let _ = with_registry(|registry| {
                                registry.destroy_timeline(timeline_handle)
                            });
                            return written;
                        }
                        unsafe {
                            *out_timeline_handle = timeline_handle;
                        }
                        written
                    }
                    Err(_) => {
                        let _ =
                            with_registry(|registry| registry.destroy_timeline(timeline_handle));
                        -1
                    }
                }
            }
            Err(error) => match map_host_lookup_error(&error) {
                Some(reason) => write_response(
                    out,
                    out_cap,
                    &reject(timeline_diagnostic(reason, Some(host_handle), None), None),
                ),
                None => -1,
            },
        }
    }))
    .unwrap_or(-1)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_timeline_destroy(
    timeline_handle: u64,
    out: *mut u8,
    out_cap: usize,
) -> i64 {
    if !output_usable(out, out_cap) {
        return -1;
    }
    catch_unwind(AssertUnwindSafe(|| {
        if timeline_handle == 0 {
            return write_response(
                out,
                out_cap,
                &reject(
                    timeline_diagnostic(RnHostReasonCode::UnknownTimelineHandle, None, Some(0)),
                    None,
                ),
            );
        }
        let outcome = with_registry(|registry| {
            let lookup_error = registry.hosts.values().find_map(|host| {
                host.timelines.get(&timeline_handle).and_then(|timeline| {
                    timeline
                        .destroyed
                        .then_some(RnHostError::DestroyedTimeline(timeline_handle))
                })
            });
            if let Some(error) = lookup_error {
                return Ok(Err(error));
            }
            if !registry
                .hosts
                .values()
                .any(|host| host.timelines.contains_key(&timeline_handle))
            {
                return Ok(Err(
                    if registry.destroyed_timelines.contains(&timeline_handle) {
                        RnHostError::DestroyedTimeline(timeline_handle)
                    } else {
                        RnHostError::UnknownTimeline(timeline_handle)
                    },
                ));
            }
            let json = encode_response(&accept_no_snapshot())?;
            if json.len() > out_cap {
                return Err(RnHostError::PayloadTooLarge);
            }
            registry.destroy_timeline(timeline_handle)?;
            Ok(Ok(json))
        });
        match outcome {
            Ok(Ok(json)) => write_bytes(out, out_cap, &json),
            Ok(Err(error)) => match map_destroy_timeline_error(&error) {
                Some(reason) => write_response(
                    out,
                    out_cap,
                    &reject(
                        timeline_diagnostic(reason, None, Some(timeline_handle)),
                        None,
                    ),
                ),
                None => -1,
            },
            Err(_) => -1,
        }
    }))
    .unwrap_or(-1)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_stage_attach(
    host_handle: u64,
    stage_handle: u64,
    metal_layer: *mut core::ffi::c_void,
    out: *mut u8,
    out_cap: usize,
) -> i64 {
    let layer_ptr = metal_layer as usize;
    write_stage_gpu_op(
        out,
        out_cap,
        host_handle,
        stage_handle,
        move |host, stage| host.stage_attach_surface(stage, layer_ptr).map(|_| ()),
    )
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_stage_resize_physical(
    host_handle: u64,
    stage_handle: u64,
    width: u32,
    height: u32,
    out: *mut u8,
    out_cap: usize,
) -> i64 {
    write_stage_gpu_op(
        out,
        out_cap,
        host_handle,
        stage_handle,
        move |host, stage| host.stage_resize_physical(stage, width, height),
    )
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_stage_draw(
    host_handle: u64,
    stage_handle: u64,
    out: *mut u8,
    out_cap: usize,
) -> i64 {
    write_stage_gpu_op(out, out_cap, host_handle, stage_handle, |host, stage| {
        host.stage_draw(stage)
    })
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_stage_detach(
    host_handle: u64,
    stage_handle: u64,
    out: *mut u8,
    out_cap: usize,
) -> i64 {
    write_stage_gpu_op(out, out_cap, host_handle, stage_handle, |host, stage| {
        host.stage_detach_surface(stage).map(|_| ())
    })
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_timeline_attach(
    host_handle: u64,
    timeline_handle: u64,
    metal_layer: *mut core::ffi::c_void,
    out: *mut u8,
    out_cap: usize,
) -> i64 {
    let layer_ptr = metal_layer as usize;
    write_timeline_gpu_op(
        out,
        out_cap,
        host_handle,
        timeline_handle,
        move |host, timeline| {
            host.timeline_attach_surface(timeline, layer_ptr)
                .map(|_| ())
        },
    )
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_timeline_resize_physical(
    host_handle: u64,
    timeline_handle: u64,
    width: u32,
    height: u32,
    out: *mut u8,
    out_cap: usize,
) -> i64 {
    write_timeline_gpu_op(
        out,
        out_cap,
        host_handle,
        timeline_handle,
        move |host, timeline| host.timeline_resize_physical(timeline, width, height),
    )
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_timeline_draw(
    host_handle: u64,
    timeline_handle: u64,
    out: *mut u8,
    out_cap: usize,
) -> i64 {
    write_timeline_gpu_op(
        out,
        out_cap,
        host_handle,
        timeline_handle,
        |host, timeline| host.timeline_draw(timeline),
    )
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_timeline_detach(
    host_handle: u64,
    timeline_handle: u64,
    out: *mut u8,
    out_cap: usize,
) -> i64 {
    write_timeline_gpu_op(
        out,
        out_cap,
        host_handle,
        timeline_handle,
        |host, timeline| host.timeline_detach_surface(timeline).map(|_| ()),
    )
}
