//! Host 上の Stage/Timeline surface attach/draw/detach。描画ヘルパは gpu_draw。

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

impl RnProductHost {
    #[cfg(target_os = "macos")]
    pub(super) fn timeline_frame_borrow(&self) -> Result<TimelineFrameBorrow, RnHostReasonCode> {
        let document = self.runtime.snapshot();
        let duration = document.composition.duration;
        let duration_seconds = duration.as_seconds_f64();
        let projection = crate::timeline_projection::project_timeline(
            document.as_ref(),
            &crate::timeline_projection::TimelineMetrics {
                band_height: 1.0,
                units_per_second: duration_seconds.recip(),
                key_half_extent: 1.0,
            },
            &crate::timeline_projection::TimelineViewport {
                start: RationalTime::ZERO,
                end: duration,
            },
        )
        .map_err(|_| RnHostReasonCode::InvalidIntent)?;
        Ok(TimelineFrameBorrow {
            revision: self.runtime.document_revision(),
            projection_generation: self.projection_generation,
            document,
            projection,
            primary: self.primary,
            playhead: self.current_time,
        })
    }

    #[cfg(target_os = "macos")]
    pub(super) fn ensure_gpu(&mut self) -> Result<&mut HostGpuBundle, RnHostReasonCode> {
        if self.gpu.is_none() {
            let (ctx, parts) = GpuCtx::new_for_ui().map_err(|_| RnHostReasonCode::InvalidIntent)?;
            let ctx = Arc::new(ctx);
            let preview = prepare_in_setup_worker(
                Arc::clone(&ctx),
                self.runtime.snapshot(),
                bootstrap_frame_desc().map_err(|_| RnHostReasonCode::InvalidIntent)?,
            )
            .map_err(|_| RnHostReasonCode::InvalidIntent)?;
            let (preview_pipeline, preview_bind_group) =
                crate::product_runtime::create_preview_pipeline(
                    &ctx.device,
                    TextureFormat::Bgra8Unorm,
                    preview.slot().view(),
                );
            let (overlay_pipeline, overlay_bind_group_layout) =
                create_overlay_pipeline(&ctx.device, TextureFormat::Bgra8Unorm);
            self.gpu = Some(HostGpuBundle {
                ctx,
                instance: parts.instance,
                adapter: parts.adapter,
                _preview: preview,
                preview_pipeline,
                preview_bind_group,
                overlay_pipeline,
                overlay_bind_group_layout,
            });
        }
        Ok(self.gpu.as_mut().expect("gpu bundle initialized"))
    }

    #[cfg(target_os = "macos")]
    pub(super) fn refresh_stage_overlays(&mut self) -> Result<(), RnHostReasonCode> {
        let Some(gpu) = self.gpu.as_ref() else {
            return Ok(());
        };
        let projection = project_stage_geometry(
            self.runtime.snapshot().as_ref(),
            EvaluationTime::new(self.current_time),
            &DataTracks::new(),
        )
        .map_err(|_| RnHostReasonCode::InvalidIntent)?;
        let device = &gpu.ctx.device;
        let queue = &gpu.ctx.queue;
        let layout = &gpu.overlay_bind_group_layout;
        for stage in self.stages.values_mut() {
            let next_key = OverlayUploadKey {
                selected: self.primary,
                projection_generation: self.projection_generation,
            };
            if !overlay_dirty(stage.gpu.overlay_upload_key, next_key) {
                continue;
            }
            stage.gpu.overlay_upload_key = Some(next_key);
            let Some(selected) = self.primary else {
                stage.gpu.overlay = None;
                continue;
            };
            let raster = raster_selection_outline(
                &projection,
                Some(selected),
                PixelSize {
                    width: stage.width as f64,
                    height: stage.height as f64,
                },
                stage.scale_factor,
            );
            if !overlay_dimensions_match(
                raster.width,
                raster.height,
                stage.gpu.physical_width,
                stage.gpu.physical_height,
            ) {
                stage.gpu.overlay = None;
                continue;
            }
            upload_stage_overlay(device, queue, layout, &mut stage.gpu.overlay, raster);
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    pub(super) fn stage_attach_surface(
        &mut self,
        stage_handle: u64,
        layer_ptr: usize,
    ) -> Result<u64, RnHostReasonCode> {
        require_main_thread()?;
        {
            let stage = self
                .stages
                .get(&stage_handle)
                .ok_or(RnHostReasonCode::UnknownStageHandle)?;
            if stage.destroyed {
                return Err(RnHostReasonCode::LateLifecycleEvent);
            }
            if !stage.mounted {
                return Err(RnHostReasonCode::InvalidIntent);
            }
            stage.gpu.validate_attach(layer_ptr)?;
        }
        let surface = {
            let gpu = self.ensure_gpu()?;
            unsafe {
                gpu.instance
                    .create_surface_unsafe(SurfaceTargetUnsafe::CoreAnimationLayer(
                        layer_ptr as *mut core::ffi::c_void,
                    ))
            }
            .map_err(|_| RnHostReasonCode::InvalidIntent)?
        };
        let stage = self
            .stages
            .get_mut(&stage_handle)
            .ok_or(RnHostReasonCode::UnknownStageHandle)?;
        stage.gpu.surface_epoch = stage.gpu.surface_epoch.saturating_add(1);
        stage.gpu.layer_ptr = layer_ptr;
        stage.gpu.surface = Some(surface);
        stage.gpu.physical_width = 0;
        stage.gpu.physical_height = 0;
        stage.gpu.last_presented_epoch = None;
        stage.gpu.needs_reconfigure = true;
        stage.gpu.overlay_upload_key = None;
        Ok(stage.gpu.surface_epoch)
    }

    #[cfg(target_os = "macos")]
    pub(super) fn configure_stage_surface(
        &mut self,
        stage_handle: u64,
        width: u32,
        height: u32,
    ) -> Result<(), RnHostReasonCode> {
        let config = {
            let gpu = self.gpu.as_ref().ok_or(RnHostReasonCode::InvalidIntent)?;
            let stage = self
                .stages
                .get(&stage_handle)
                .ok_or(RnHostReasonCode::UnknownStageHandle)?;
            stage.gpu.reject_if_poisoned()?;
            let surface = stage
                .gpu
                .surface
                .as_ref()
                .ok_or(RnHostReasonCode::InvalidIntent)?;
            supported_surface_config(surface, &gpu.adapter, width, height)?
        };
        let gpu = self.gpu.as_ref().ok_or(RnHostReasonCode::InvalidIntent)?;
        let stage = self
            .stages
            .get_mut(&stage_handle)
            .ok_or(RnHostReasonCode::UnknownStageHandle)?;
        let surface = stage
            .gpu
            .surface
            .as_ref()
            .ok_or(RnHostReasonCode::InvalidIntent)?;
        surface.configure(&gpu.ctx.device, &config);
        stage.gpu.configured(width, height);
        Ok(())
    }

    #[cfg(target_os = "macos")]
    pub(super) fn stage_resize_physical(
        &mut self,
        stage_handle: u64,
        width: u32,
        height: u32,
    ) -> Result<(), RnHostReasonCode> {
        require_main_thread()?;
        {
            let stage = self
                .stages
                .get(&stage_handle)
                .ok_or(RnHostReasonCode::UnknownStageHandle)?;
            if stage.destroyed {
                return Err(RnHostReasonCode::LateLifecycleEvent);
            }
            if !stage.gpu.is_attached() || !stage.gpu.has_surface() {
                return Err(RnHostReasonCode::InvalidIntent);
            }
            stage.gpu.reject_if_poisoned()?;
        }
        if width == 0 || height == 0 {
            let stage = self
                .stages
                .get_mut(&stage_handle)
                .ok_or(RnHostReasonCode::UnknownStageHandle)?;
            stage.gpu.physical_width = width;
            stage.gpu.physical_height = height;
            stage.gpu.needs_reconfigure = true;
            stage.gpu.overlay_upload_key = None;
            return Ok(());
        }
        self.configure_stage_surface(stage_handle, width, height)
    }

    #[cfg(target_os = "macos")]
    pub(super) fn stage_draw(&mut self, stage_handle: u64) -> Result<(), RnHostReasonCode> {
        require_main_thread()?;
        let (width, height, attached, needs_reconfigure) = {
            let stage = self
                .stages
                .get(&stage_handle)
                .ok_or(RnHostReasonCode::UnknownStageHandle)?;
            if stage.destroyed {
                return Err(RnHostReasonCode::LateLifecycleEvent);
            }
            stage.gpu.reject_if_poisoned()?;
            (
                stage.gpu.physical_width,
                stage.gpu.physical_height,
                stage.gpu.is_attached(),
                stage.gpu.needs_reconfigure,
            )
        };
        if !attached {
            return Err(RnHostReasonCode::InvalidIntent);
        }
        if width == 0 || height == 0 {
            return Ok(());
        }
        if needs_reconfigure {
            self.configure_stage_surface(stage_handle, width, height)?;
        }
        self.refresh_stage_overlays().ok();
        let gpu = self.gpu.as_ref().ok_or(RnHostReasonCode::InvalidIntent)?;
        let stage = self
            .stages
            .get(&stage_handle)
            .ok_or(RnHostReasonCode::UnknownStageHandle)?;
        let surface = stage
            .gpu
            .surface
            .as_ref()
            .ok_or(RnHostReasonCode::InvalidIntent)?;
        match surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) => {
                draw_stage_preview(gpu, frame, stage.gpu.overlay.as_ref());
                self.stages
                    .get_mut(&stage_handle)
                    .ok_or(RnHostReasonCode::UnknownStageHandle)?
                    .gpu
                    .presented(false);
                Ok(())
            }
            CurrentSurfaceTexture::Suboptimal(frame) => {
                draw_stage_preview(gpu, frame, stage.gpu.overlay.as_ref());
                self.stages
                    .get_mut(&stage_handle)
                    .ok_or(RnHostReasonCode::UnknownStageHandle)?
                    .gpu
                    .presented(true);
                Ok(())
            }
            CurrentSurfaceTexture::Timeout | CurrentSurfaceTexture::Occluded => {
                self.stages
                    .get_mut(&stage_handle)
                    .ok_or(RnHostReasonCode::UnknownStageHandle)?
                    .gpu
                    .acquisition_deferred();
                Ok(())
            }
            CurrentSurfaceTexture::Outdated => {
                self.stages
                    .get_mut(&stage_handle)
                    .ok_or(RnHostReasonCode::UnknownStageHandle)?
                    .gpu
                    .outdated();
                Ok(())
            }
            CurrentSurfaceTexture::Lost => {
                self.stages
                    .get_mut(&stage_handle)
                    .ok_or(RnHostReasonCode::UnknownStageHandle)?
                    .gpu
                    .lost();
                Err(RnHostReasonCode::InvalidIntent)
            }
            CurrentSurfaceTexture::Validation => {
                self.stages
                    .get_mut(&stage_handle)
                    .ok_or(RnHostReasonCode::UnknownStageHandle)?
                    .gpu
                    .validation_failed();
                Err(RnHostReasonCode::InvalidIntent)
            }
        }
    }

    #[cfg(target_os = "macos")]
    pub(super) fn stage_detach_surface(
        &mut self,
        stage_handle: u64,
    ) -> Result<u64, RnHostReasonCode> {
        require_main_thread()?;
        let stage = self
            .stages
            .get_mut(&stage_handle)
            .ok_or(RnHostReasonCode::UnknownStageHandle)?;
        if stage.destroyed {
            return Err(RnHostReasonCode::LateLifecycleEvent);
        }
        stage.gpu_detach_surface();
        Ok(stage.gpu.surface_epoch)
    }

    #[cfg(target_os = "macos")]
    pub(super) fn detach_all_stage_surfaces(&mut self) {
        let stage_handles = self.stages.keys().copied().collect::<Vec<_>>();
        for stage_handle in stage_handles {
            if let Some(stage) = self.stages.get_mut(&stage_handle) {
                stage.gpu_detach_surface();
            }
        }
    }
}
