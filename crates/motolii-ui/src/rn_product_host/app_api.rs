//! App/test 向け公開関数。C ABI は ffi。

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
use super::wire_io::*;
#[cfg(target_os = "macos")]
use super::MAX_PROJECT_PATH_BYTES;
use super::{
    HOST_TO_RN, MAX_DIAGNOSTICS, MAX_EFFECTS_PER_LAYER, MAX_JSON_BYTES, MAX_POSITION_KEYS,
    MAX_SNAPSHOT_JSON_BYTES, MAX_SOURCE_PARAMS_PER_LAYER, MAX_STAGE_BOUNDS, MAX_STAGE_SELECTION,
    PRODUCT_ROLE, RN_TO_HOST, WIRE_VERSION,
};

pub fn host_create_for_test(project_path: &Path) -> Result<u64, RnHostError> {
    with_registry(|registry| registry.create_host(project_path))
}

pub(super) fn restore_stage_video(host_handle: u64, binder: Option<VideoSourceBinder>) {
    let Some(binder) = binder else {
        return;
    };
    let mut guard = lock_registry();
    let Some(host) = guard.hosts.get_mut(&host_handle) else {
        return;
    };
    if !host.destroyed {
        host.stage_video = Some(binder);
    }
}

/// 評価済み Document の実フレームを Stage 合成へ渡す薄い seam。
/// dirty gate: 前回と同じ (revision, generation, time) なら再renderせず Unchanged。
///
/// F10: registry mutexは取り出し/書き戻しだけに閉じる。graph構築とGPU submitはlock外。
/// 並行renderは単一render thread前提のため二重render対策は持たない。
#[doc(hidden)]
pub fn host_render_frame_for_app(
    host_handle: u64,
    gpu: &GpuCtx,
    session: &mut RenderSession,
    out: &mut Option<AppStageFrame>,
) -> HostRenderFrameResult {
    // Unchanged / Failed では呼び手の既存frameへ触れない(Rendered時のみ上書き)。
    // 冒頭で無条件にNone化するとUnchanged tickごとに実フレームが消える。

    // 第一lock: destroyed確認・Unchanged判定・snapshot/runtime取り出し。ここまででguardを落とす。
    let (
        revision,
        generation,
        time,
        preview_epoch,
        document,
        runtime,
        preview_command,
        stage_video,
    ) = {
        let mut guard = lock_registry();
        let Some(host) = guard.hosts.get_mut(&host_handle) else {
            return HostRenderFrameResult::Failed;
        };
        if host.destroyed {
            return HostRenderFrameResult::Failed;
        }
        host.pump_playback();
        let revision = host.runtime.document_revision().to_string();
        let generation = host.projection_generation.to_string();
        let time = host.current_time;
        let preview_epoch = host.inspector_preview_epoch;
        // 呼び手がframe未保持なら(rev,gen,time,preview)一致でも再render(renderer再生成後の穴を塞ぐ)。
        if out.is_some() {
            if let Some((prev_rev, prev_gen, prev_time, prev_preview)) =
                host.stage_frame_last.as_ref()
            {
                if prev_rev == &revision
                    && prev_gen == &generation
                    && *prev_time == time
                    && *prev_preview == preview_epoch
                {
                    return HostRenderFrameResult::Unchanged;
                }
            }
        }

        let document = host.runtime.snapshot();
        if host.stage_frame_runtime.is_none() {
            let Ok(created) = first_party_runtime() else {
                return HostRenderFrameResult::Failed;
            };
            host.stage_frame_runtime = Some(Arc::new(created));
        }
        let runtime = host
            .stage_frame_runtime
            .as_ref()
            .expect("stage_frame_runtime initialized")
            .clone();
        let preview_command = host.inspector_preview.clone();
        let stage_video = host.stage_video.take();
        (
            revision,
            generation,
            time,
            preview_epoch,
            document,
            runtime,
            preview_command,
            stage_video,
        )
    };

    let Some(desc) = frame_desc_from_composition(document.as_ref()) else {
        restore_stage_video(host_handle, stage_video);
        return HostRenderFrameResult::Failed;
    };

    // product path / render_worker と同じ: 空 DataTracks、project_root=None、Quality::DRAFT。
    // video_sources は export と同じ FrameReader 束。lock外: build + GPU submit。
    let tracks = DataTracks::new();
    let eval = EvaluationTime::new(time);
    let request = crate::render_worker::RenderRequest {
        document: Arc::clone(&document),
        data_tracks: Arc::new(DataTracks::new()),
        evaluation_time: eval,
        desc,
        quality: Quality::DRAFT,
    };
    let previewed = preview_command.as_ref().and_then(|command| {
        crate::render_worker::prepare_preview_document(&request, Some(command)).ok()
    });
    let eval_document = previewed
        .as_ref()
        .map(|prepared| prepared.as_ref())
        .unwrap_or(document.as_ref());
    let built = match build_document_frame_graph(
        eval_document,
        eval,
        desc,
        &tracks,
        runtime.as_ref(),
        None,
    ) {
        Ok(built) => built,
        Err(_) => {
            restore_stage_video(host_handle, stage_video);
            return HostRenderFrameResult::Failed;
        }
    };
    let mut stage_video = stage_video.unwrap_or_else(|| VideoSourceBinder::new(gpu));
    let bound = match stage_video.bind(gpu, eval_document, None, &built.video_slots, desc) {
        Ok(bound) => bound,
        Err(_) => {
            restore_stage_video(host_handle, Some(stage_video));
            return HostRenderFrameResult::Failed;
        }
    };
    let video_inputs = bound.as_inputs();
    let rendered = match render_graph_cached(
        gpu,
        session,
        time,
        &built.graph,
        &RenderGraphInputs {
            camera: built.camera,
            video_sources: &video_inputs,
            source_time: Some(built.source_time),
            plugins: Some(runtime.executors()),
        },
        Quality::DRAFT,
    ) {
        Ok(frame) => frame,
        Err(_) => {
            restore_stage_video(host_handle, Some(stage_video));
            return HostRenderFrameResult::Failed;
        }
    };

    // 第二lock: host生存を再確認してstage_frame_lastを書き戻す。消えていたらFailed、outは触らない。
    // revisionが進んでいても書き戻してよい(次tickのUnchangedが正しく外れるだけ)。
    {
        let mut guard = lock_registry();
        let Some(host) = guard.hosts.get_mut(&host_handle) else {
            return HostRenderFrameResult::Failed;
        };
        if host.destroyed {
            return HostRenderFrameResult::Failed;
        }
        host.stage_video = Some(stage_video);
        host.stage_frame_last = Some((revision.clone(), generation.clone(), time, preview_epoch));
    }

    *out = Some(AppStageFrame {
        texture: rendered.texture,
        width: rendered.desc.width,
        height: rendered.desc.height,
        revision,
        generation,
        time,
    });
    HostRenderFrameResult::Rendered
}

/// Clone the live Document, apply the same D2 command used at commit, and project Stage path
/// geometry. Does not mutate the single writer and does not render a second frame authority.
#[doc(hidden)]
pub fn host_preview_stage_transform_for_app(
    host_handle: u64,
    expected_revision: u64,
    target: u64,
    edit: AppStageTransformEdit,
) -> Result<AppStageTransformPreview, AppStageTransformError> {
    let (time, document, command) = {
        let mut guard = lock_registry();
        let host = guard
            .hosts
            .get_mut(&host_handle)
            .ok_or(AppStageTransformError::HostUnavailable)?;
        if host.destroyed {
            return Err(AppStageTransformError::HostUnavailable);
        }
        if host.runtime.document_revision() != expected_revision {
            return Err(AppStageTransformError::StaleDocument);
        }
        let document = host.runtime.snapshot();
        let time = host.current_time;
        let command = prepare_app_stage_transform_command(
            document.as_ref(),
            time,
            LayerId::from_raw(target),
            edit,
        )?;
        (time, document, command)
    };

    let desc = frame_desc_from_composition(document.as_ref())
        .ok_or_else(|| AppStageTransformError::Render("invalid composition dimensions".into()))?;
    let request = crate::render_worker::RenderRequest {
        document,
        data_tracks: Arc::new(DataTracks::new()),
        evaluation_time: EvaluationTime::new(time),
        desc,
        quality: Quality::DRAFT,
    };
    let prepared = crate::render_worker::prepare_preview_document(&request, Some(&command))
        .map_err(|error| AppStageTransformError::Preview(error.to_string()))?;
    Ok(AppStageTransformPreview {
        geometry: app_stage_geometry(prepared.as_ref(), time),
    })
}

/// Commit one Stage transform through the existing DocumentEditRuntime single writer.
#[doc(hidden)]
pub fn host_commit_stage_transform_for_app(
    host_handle: u64,
    expected_revision: u64,
    target: u64,
    edit: AppStageTransformEdit,
) -> Result<(), AppStageTransformError> {
    let mut guard = lock_registry();
    let host = guard
        .hosts
        .get_mut(&host_handle)
        .ok_or(AppStageTransformError::HostUnavailable)?;
    if host.destroyed {
        return Err(AppStageTransformError::HostUnavailable);
    }
    if host.runtime.document_revision() != expected_revision {
        return Err(AppStageTransformError::StaleDocument);
    }
    let snapshot = host.runtime.snapshot();
    let command = prepare_app_stage_transform_command(
        snapshot.as_ref(),
        host.current_time,
        LayerId::from_raw(target),
        edit,
    )?;
    let mut queue = DocumentEditQueue::default();
    if !queue.push_stage_transform(command) {
        return Err(AppStageTransformError::UnsupportedProperty);
    }
    let published = host
        .runtime
        .process_next(&mut queue, host.primary, host.projection_generation)
        .map_err(|error| AppStageTransformError::Commit(error.to_string()))?
        .ok_or(AppStageTransformError::NoChange)?;
    host.adopt_published(published);
    Ok(())
}

/// composition アスペクトから bootstrap 系の FrameDesc を作る（高さ1080固定）。
pub(super) fn frame_desc_from_composition(document: &motolii_doc::Document) -> Option<FrameDesc> {
    const HEIGHT: u32 = 1080;
    let width = u64::from(HEIGHT)
        .checked_mul(document.composition.aspect_num() as u64)?
        .checked_div(document.composition.aspect_den() as u64)? as u32;
    if width == 0 {
        return None;
    }
    FrameDesc::try_packed(
        width,
        HEIGHT,
        PixelFormat::Rgba8Unorm,
        ColorSpace::Srgb,
        true,
    )
    .ok()
}

pub fn host_read_snapshot_for_test(
    host_handle: u64,
) -> Result<RnProductSnapshotForTest, RnHostError> {
    with_registry(|registry| registry.read_snapshot(host_handle)).map(snapshot_for_test)
}

impl From<RnHostTestIntent> for serde_json::Value {
    fn from(intent: RnHostTestIntent) -> Self {
        serde_json::json!({
            "version": WIRE_VERSION,
            "direction": RN_TO_HOST,
            "kind": intent.kind,
            "stage_handle": intent.stage_handle.map(|value| value.to_string()),
            "projection_generation": intent.projection_generation,
            "width": intent.width,
            "height": intent.height,
            "scale_factor": intent.scale_factor,
            "focused": intent.focused,
        })
    }
}

pub fn host_dispatch_intent_for_test<T: Into<serde_json::Value>>(
    host_handle: u64,
    intent: T,
) -> Result<RnHostTestResponse, RnHostError> {
    let mut wire_intent = intent.into();
    wire_intent["host_handle"] = serde_json::Value::String(host_handle.to_string());
    let json = with_registry(|registry| {
        registry.dispatch_intent_json(host_handle, &encode_json(&wire_intent)?)
    })?;
    serde_json::from_str::<WireIntentResponse>(&json)
        .map(response_for_test)
        .map_err(RnHostError::from)
}

pub fn host_register_stage_for_test(host_handle: u64) -> Result<u64, RnHostError> {
    with_registry(|registry| registry.register_stage(host_handle))
}

pub fn host_destroy_stage_for_test(stage_handle: u64) -> Result<(), RnHostError> {
    with_registry(|registry| registry.destroy_stage(stage_handle))
}

pub fn host_destroy_for_test(host_handle: u64) -> Result<(), RnHostError> {
    with_registry(|registry| registry.destroy_host(host_handle))
}

#[cfg(all(test, target_os = "macos"))]
pub(crate) fn stage_gpu_state_for_test(
    stage_handle: u64,
) -> Result<(u64, bool, u32, u32), RnHostError> {
    with_registry(|registry| {
        for host in registry.hosts.values() {
            if let Some(stage) = host.stages.get(&stage_handle) {
                return Ok((
                    stage.gpu.surface_epoch,
                    stage.gpu.is_attached(),
                    stage.gpu.physical_width,
                    stage.gpu.physical_height,
                ));
            }
        }
        Err(RnHostError::UnknownStage(stage_handle))
    })
}

#[cfg(all(test, target_os = "macos"))]
pub(crate) fn stage_gpu_mark_attached_for_test(
    stage_handle: u64,
    layer_ptr: usize,
) -> Result<u64, RnHostError> {
    with_registry(|registry| {
        for host in registry.hosts.values_mut() {
            if let Some(stage) = host.stages.get_mut(&stage_handle) {
                stage.gpu.surface_epoch = stage.gpu.surface_epoch.saturating_add(1);
                stage.gpu.layer_ptr = layer_ptr;
                stage.gpu.physical_width = 0;
                stage.gpu.physical_height = 0;
                return Ok(stage.gpu.surface_epoch);
            }
        }
        Err(RnHostError::UnknownStage(stage_handle))
    })
}
