//! Stage 幾何と transform の wire 投影。timeline 投影は projection。

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
use super::registry::*;
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

/// Available だけを corners に畳む。評価失敗は空投影（snapshot 自体は落とさない）。
pub(super) fn project_stage_geometry_wire(
    document: &motolii_doc::Document,
    eval: EvaluationTime,
    tracks: &DataTracks,
) -> WireStageGeometryProjection {
    let Ok(projection) = project_stage_geometry(document, eval, tracks) else {
        return WireStageGeometryProjection {
            layers: Vec::new(),
            layers_truncated: false,
        };
    };
    let mut layers = Vec::new();
    let mut available = 0usize;
    let mut layers_truncated = false;
    for (layer_id, layer) in projection.layers() {
        let StageLayerProjection::Available(geo) = layer else {
            continue;
        };
        available += 1;
        if layers.len() >= MAX_STAGE_BOUNDS {
            layers_truncated = true;
            continue;
        }
        let hw = geo.local_rect.size.width * 0.5;
        let hh = geo.local_rect.size.height * 0.5;
        let cx = geo.local_rect.center.x;
        let cy = geo.local_rect.center.y;
        // CCW・local 左下起点。v1 は world のみ（camera_view 不使用）。
        let local = [
            [cx - hw, cy - hh],
            [cx + hw, cy - hh],
            [cx + hw, cy + hh],
            [cx - hw, cy + hh],
        ];
        let corners = world_rect_corners(geo.world, local);
        let (position, rotation, scale) = world_layer_trs(geo.world, [cx, cy]);
        layers.push(WireStageGeometryLayer {
            layer_id: layer_id.get().to_string(),
            corners,
            position,
            rotation,
            scale,
        });
    }
    if available > MAX_STAGE_BOUNDS {
        layers_truncated = true;
    }
    WireStageGeometryProjection {
        layers,
        layers_truncated,
    }
}

pub(super) fn world_layer_trs(world: Affine2D, center: [f64; 2]) -> ([f64; 2], f64, [f64; 2]) {
    (
        world.transform_point(center[0], center[1]),
        world.m[3].atan2(world.m[0]),
        world.approx_scale(),
    )
}

pub(super) fn world_rect_corners(
    world: motolii_doc::Affine2D,
    local: [[f64; 2]; 4],
) -> [[f64; 2]; 4] {
    let mut corners = local.map(|[x, y]| {
        let p = world.transform_point(x, y);
        [p[0], p[1]]
    });
    // world determinant が負なら反転して CCW に揃える。
    if world.m[0] * world.m[4] - world.m[1] * world.m[3] < 0.0 {
        corners.reverse();
    }
    corners
}

/// world 空間 delta を position param の局所 delta へ写す（線形部のみ）。
/// `Affine2D::try_invert`（affine.rs）の逆行列で delta を変換する。
pub(super) fn world_delta_to_position_local(world: Affine2D, delta: [f64; 2]) -> Option<[f64; 2]> {
    let inv = world.try_invert()?;
    let m = inv.m;
    let local = [
        m[0] * delta[0] + m[1] * delta[1],
        m[3] * delta[0] + m[4] * delta[1],
    ];
    local.iter().all(|value| value.is_finite()).then_some(local)
}

pub(super) fn prepare_app_stage_transform_command(
    document: &motolii_doc::Document,
    time: RationalTime,
    target: LayerId,
    edit: AppStageTransformEdit,
) -> Result<Command, AppStageTransformError> {
    let envelope = find_envelope_in_document(document, target)
        .ok_or(AppStageTransformError::TargetUnavailable)?;
    match edit {
        AppStageTransformEdit::TranslateWorld(delta) => {
            if !delta.iter().all(|value| value.is_finite()) {
                return Err(AppStageTransformError::NonFinite);
            }
            let tracks = DataTracks::new();
            let projection = project_stage_geometry(document, EvaluationTime::new(time), &tracks)
                .map_err(|_| AppStageTransformError::TransformUnavailable)?;
            let Some(StageLayerProjection::Available(geometry)) = projection.get(target).cloned()
            else {
                return Err(AppStageTransformError::TransformUnavailable);
            };
            let local_delta = world_delta_to_position_local(geometry.world, delta)
                .ok_or(AppStageTransformError::TransformUnavailable)?;
            match &envelope.transform.position {
                DocParam::Const(DocValue::Vec2(old)) => {
                    let new = [old[0] + local_delta[0], old[1] + local_delta[1]];
                    if !new.iter().all(|value| value.is_finite()) {
                        return Err(AppStageTransformError::NonFinite);
                    }
                    if new == *old {
                        return Err(AppStageTransformError::NoChange);
                    }
                    Ok(Command::SetProperty {
                        target,
                        property: ScalarPropertyId::Position,
                        old_value: DocParam::const_vec2(*old),
                        new_value: DocParam::const_vec2(new),
                    })
                }
                DocParam::Keyframes(_) => {
                    let (key, old) = position_key_at(document, target, time)
                        .ok_or(AppStageTransformError::OffKeyframe)?;
                    let new = [old[0] + local_delta[0], old[1] + local_delta[1]];
                    if !new.iter().all(|value| value.is_finite()) {
                        return Err(AppStageTransformError::NonFinite);
                    }
                    if new == old {
                        return Err(AppStageTransformError::NoChange);
                    }
                    Ok(Command::SetPositionKeyValue {
                        target,
                        key,
                        old,
                        new,
                    })
                }
                _ => Err(AppStageTransformError::UnsupportedProperty),
            }
        }
        AppStageTransformEdit::RotateZ(delta) => {
            if !delta.is_finite() {
                return Err(AppStageTransformError::NonFinite);
            }
            match &envelope.transform.rotation {
                DocParam::Const(DocValue::F64(old)) => {
                    let new = *old + delta;
                    if !new.is_finite() {
                        return Err(AppStageTransformError::NonFinite);
                    }
                    if new == *old {
                        return Err(AppStageTransformError::NoChange);
                    }
                    Ok(Command::SetProperty {
                        target,
                        property: ScalarPropertyId::Rotation,
                        old_value: DocParam::const_f64(*old),
                        new_value: DocParam::const_f64(new),
                    })
                }
                DocParam::Keyframes(_) => {
                    let (key, old) = match keyed_value_at(&envelope.transform.rotation, time) {
                        Some((key, DocValue::F64(old))) => (key, old),
                        Some(_) => return Err(AppStageTransformError::UnsupportedProperty),
                        None => return Err(AppStageTransformError::OffKeyframe),
                    };
                    let new = old + delta;
                    if !new.is_finite() {
                        return Err(AppStageTransformError::NonFinite);
                    }
                    if new == old {
                        return Err(AppStageTransformError::NoChange);
                    }
                    keyed_transform_set(
                        document,
                        target,
                        ScalarPropertyId::Rotation,
                        key,
                        DocValue::F64(new),
                    )
                }
                _ => Err(AppStageTransformError::UnsupportedProperty),
            }
        }
        AppStageTransformEdit::Scale(factor) => {
            if !factor.iter().all(|value| value.is_finite()) {
                return Err(AppStageTransformError::NonFinite);
            }
            match &envelope.transform.scale {
                DocParam::Const(DocValue::Vec2(old)) => {
                    let new = [old[0] * factor[0], old[1] * factor[1]];
                    if !new.iter().all(|value| value.is_finite()) {
                        return Err(AppStageTransformError::NonFinite);
                    }
                    if new == *old {
                        return Err(AppStageTransformError::NoChange);
                    }
                    Ok(Command::SetProperty {
                        target,
                        property: ScalarPropertyId::Scale,
                        old_value: DocParam::const_vec2(*old),
                        new_value: DocParam::const_vec2(new),
                    })
                }
                DocParam::Keyframes(_) => {
                    let (key, old) = match keyed_value_at(&envelope.transform.scale, time) {
                        Some((key, DocValue::Vec2(old))) => (key, old),
                        Some(_) => return Err(AppStageTransformError::UnsupportedProperty),
                        None => return Err(AppStageTransformError::OffKeyframe),
                    };
                    let new = [old[0] * factor[0], old[1] * factor[1]];
                    if !new.iter().all(|value| value.is_finite()) {
                        return Err(AppStageTransformError::NonFinite);
                    }
                    if new == old {
                        return Err(AppStageTransformError::NoChange);
                    }
                    keyed_transform_set(
                        document,
                        target,
                        ScalarPropertyId::Scale,
                        key,
                        DocValue::Vec2(new),
                    )
                }
                _ => Err(AppStageTransformError::UnsupportedProperty),
            }
        }
    }
}

pub(crate) fn app_stage_geometry(
    document: &motolii_doc::Document,
    time: RationalTime,
) -> AppStageGeometry {
    let wire = project_stage_geometry_wire(document, EvaluationTime::new(time), &DataTracks::new());
    AppStageGeometry {
        layers: wire
            .layers
            .into_iter()
            .map(|layer| AppStageGeometryLayer {
                layer_id: layer.layer_id,
                corners: layer.corners,
                position: layer.position,
                rotation: layer.rotation,
                scale: layer.scale,
            })
            .collect(),
        layers_truncated: wire.layers_truncated,
    }
}

pub(super) fn find_first_clip(document: &motolii_doc::Document, target: LayerId) -> Option<&Clip> {
    fn walk<'a>(items: &'a [TrackItem], target: LayerId) -> Option<&'a Clip> {
        for item in items {
            match item {
                TrackItem::Clip(clip) if clip.envelope.layer_id == target => {
                    return Some(clip);
                }
                TrackItem::Group(group) => {
                    if let Some(clip) = walk(&group.children, target) {
                        return Some(clip);
                    }
                }
                TrackItem::Clip(_) => {}
            }
        }
        None
    }
    document
        .tracks
        .iter()
        .find_map(|track| walk(&track.items, target))
}

pub(super) fn find_envelope_in_document(
    document: &motolii_doc::Document,
    target: LayerId,
) -> Option<&ItemEnvelope> {
    fn walk<'a>(items: &'a [TrackItem], target: LayerId) -> Option<&'a ItemEnvelope> {
        for item in items {
            match item {
                TrackItem::Clip(clip) if clip.envelope.layer_id == target => {
                    return Some(&clip.envelope);
                }
                TrackItem::Group(group) if group.envelope.layer_id == target => {
                    return Some(&group.envelope);
                }
                TrackItem::Group(group) => {
                    if let Some(envelope) = walk(&group.children, target) {
                        return Some(envelope);
                    }
                }
                TrackItem::Clip(_) => {}
            }
        }
        None
    }
    document
        .tracks
        .iter()
        .find_map(|track| walk(&track.items, target))
}

/// DeleteTargetedItems用: target層の(parent, index, item)を現Documentから拾う。
pub(super) fn find_track_item_location(
    document: &motolii_doc::Document,
    target: LayerId,
) -> Option<(ParentLocator, usize, TrackItem)> {
    fn envelope_layer(item: &TrackItem) -> LayerId {
        match item {
            TrackItem::Clip(clip) => clip.envelope.layer_id,
            TrackItem::Group(group) => group.envelope.layer_id,
        }
    }
    fn walk_groups(
        items: &[TrackItem],
        target: LayerId,
    ) -> Option<(ParentLocator, usize, TrackItem)> {
        for item in items {
            if let TrackItem::Group(group) = item {
                if let Some((idx, child)) = group
                    .children
                    .iter()
                    .enumerate()
                    .find(|(_, child)| envelope_layer(child) == target)
                {
                    return Some((
                        ParentLocator::Group(group.envelope.layer_id),
                        idx,
                        child.clone(),
                    ));
                }
                if let Some(found) = walk_groups(&group.children, target) {
                    return Some(found);
                }
            }
        }
        None
    }
    for track in &document.tracks {
        if let Some((idx, item)) = track
            .items
            .iter()
            .enumerate()
            .find(|(_, item)| envelope_layer(item) == target)
        {
            return Some((ParentLocator::Track(track.id), idx, item.clone()));
        }
        if let Some(found) = walk_groups(&track.items, target) {
            return Some(found);
        }
    }
    None
}
