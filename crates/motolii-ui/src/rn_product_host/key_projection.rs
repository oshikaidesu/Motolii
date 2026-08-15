//! Position/param key の wire 投影。幾何は stage_projection。

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
use super::gpu_draw::*;
use super::gpu_surface::*;
use super::host::*;
use super::registry::*;
use super::stage_projection::*;
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

pub(super) fn project_position_keys(
    document: &motolii_doc::Document,
    target: LayerId,
) -> (Vec<WireTimelinePositionKey>, bool, u32) {
    let Some(envelope) = find_envelope_in_document(document, target) else {
        return (Vec::new(), false, 0);
    };
    let DocParam::Keyframes(track) = &envelope.transform.position else {
        return (Vec::new(), false, 0);
    };
    let keys = track.keys();
    let hidden = keys.len().saturating_sub(MAX_POSITION_KEYS) as u32;
    let keys_truncated = hidden > 0;
    let position_keys = keys
        .iter()
        .take(MAX_POSITION_KEYS)
        .map(|key| WireTimelinePositionKey {
            key_id: key.id.get().to_string(),
            time: key.t,
            value: match key.value {
                DocValue::Vec2(value) => Some(value),
                _ => None,
            },
            interp: Some(key.interp),
        })
        .collect();
    (position_keys, keys_truncated, hidden)
}

pub(super) fn project_param_keys(
    document: &motolii_doc::Document,
    target: LayerId,
) -> Vec<WireTimelineParamKey> {
    let Some(envelope) = find_envelope_in_document(document, target) else {
        return Vec::new();
    };
    let mut keys = Vec::new();
    push_param_keys("scale", &envelope.transform.scale, &mut keys);
    push_param_keys("rotation", &envelope.transform.rotation, &mut keys);
    push_param_keys("opacity", &envelope.opacity, &mut keys);
    keys
}

pub(super) fn push_param_keys(
    property: &str,
    param: &DocParam,
    out: &mut Vec<WireTimelineParamKey>,
) {
    let DocParam::Keyframes(track) = param else {
        return;
    };
    for key in track.keys() {
        let (value, vec) = match key.value {
            DocValue::F64(value) if value.is_finite() => (Some(value), None),
            DocValue::Vec2(value) if value.iter().all(|component| component.is_finite()) => {
                (None, Some(value))
            }
            _ => (None, None),
        };
        out.push(WireTimelineParamKey {
            property: property.to_owned(),
            key_id: key.id.get().to_string(),
            time: key.t,
            value,
            vec,
        });
    }
}

pub(super) fn wire_transform_property(name: &str) -> Option<ScalarPropertyId> {
    match name {
        "scale" => Some(ScalarPropertyId::Scale),
        "rotation" => Some(ScalarPropertyId::Rotation),
        "opacity" => Some(ScalarPropertyId::Opacity),
        _ => None,
    }
}

pub(super) fn keyed_value_at(
    param: &DocParam,
    time: RationalTime,
) -> Option<(KeyframeId, DocValue)> {
    let DocParam::Keyframes(track) = param else {
        return None;
    };
    let key = track
        .keys()
        .iter()
        .find(|key| rational_time_eq(key.t, time))?;
    Some((key.id, key.value.clone()))
}

pub(super) fn keyed_transform_set(
    document: &motolii_doc::Document,
    target: LayerId,
    property: ScalarPropertyId,
    key: KeyframeId,
    new: DocValue,
) -> Result<Command, AppStageTransformError> {
    prepare_set_transform_param_key_value(document, target, property, key, new)
        .map_err(|_| AppStageTransformError::UnsupportedProperty)?
        .ok_or(AppStageTransformError::NoChange)
}

pub(super) fn position_key_at(
    document: &motolii_doc::Document,
    target: LayerId,
    time: RationalTime,
) -> Option<(KeyframeId, [f64; 2])> {
    let envelope = find_envelope_in_document(document, target)?;
    let DocParam::Keyframes(track) = &envelope.transform.position else {
        return None;
    };
    let keys = track.keys();
    if keys.is_empty()
        || track.validate().is_err()
        || keys.iter().any(|key| {
            !matches!(key.value, DocValue::Vec2(value) if value.iter().all(|value| value.is_finite()))
        })
    {
        return None;
    }
    let key = keys.iter().find(|key| rational_time_eq(key.t, time))?;
    let DocValue::Vec2(value) = key.value else {
        return None;
    };
    Some((key.id, value))
}

pub(super) fn position_key_time_at(
    document: &motolii_doc::Document,
    target: LayerId,
    key: KeyframeId,
) -> Option<RationalTime> {
    let envelope = find_envelope_in_document(document, target)?;
    let DocParam::Keyframes(track) = &envelope.transform.position else {
        return None;
    };
    Some(track.get_by_id(key)?.t)
}

pub(super) fn rational_time_eq(left: RationalTime, right: RationalTime) -> bool {
    let lhs = i128::from(left.num()).checked_mul(i128::from(right.den()));
    let rhs = i128::from(right.num()).checked_mul(i128::from(left.den()));
    match (lhs, rhs) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

pub(super) fn canonical_playback_start_frame(
    current: RationalTime,
) -> Result<u64, RnHostReasonCode> {
    if current < RationalTime::ZERO {
        return Err(RnHostReasonCode::InvalidIntent);
    }
    let canonical_fps = Fps::try_new(CANONICAL_SAMPLE_RATE as i64, 1)
        .map_err(|_| RnHostReasonCode::InvalidIntent)?;
    let frame = current
        .try_to_frame_floor(canonical_fps)
        .map_err(|_| RnHostReasonCode::InvalidIntent)?;
    u64::try_from(frame).map_err(|_| RnHostReasonCode::InvalidIntent)
}
