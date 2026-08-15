//! Timeline surface の attach/draw/detach。Stage GPU は gpu_surface。

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
use super::gpu_ops::*;
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

impl RnProductHost {
    #[cfg(target_os = "macos")]
    pub(super) fn timeline_attach_surface(
        &mut self,
        timeline_handle: u64,
        layer_ptr: usize,
    ) -> Result<u64, RnHostReasonCode> {
        require_main_thread()?;
        {
            let timeline = self
                .timelines
                .get(&timeline_handle)
                .ok_or(RnHostReasonCode::UnknownTimelineHandle)?;
            if timeline.destroyed {
                return Err(RnHostReasonCode::LateLifecycleEvent);
            }
            timeline.gpu.validate_attach(layer_ptr)?;
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
        let timeline = self
            .timelines
            .get_mut(&timeline_handle)
            .ok_or(RnHostReasonCode::UnknownTimelineHandle)?;
        timeline.gpu.surface_epoch = timeline.gpu.surface_epoch.saturating_add(1);
        timeline.gpu.layer_ptr = layer_ptr;
        timeline.gpu.surface = Some(surface);
        timeline.gpu.physical_width = 0;
        timeline.gpu.physical_height = 0;
        timeline.gpu.last_presented_epoch = None;
        timeline.gpu.needs_reconfigure = true;
        timeline.raster_key = None;
        Ok(timeline.gpu.surface_epoch)
    }

    #[cfg(target_os = "macos")]
    pub(super) fn configure_timeline_surface(
        &mut self,
        timeline_handle: u64,
        width: u32,
        height: u32,
    ) -> Result<(), RnHostReasonCode> {
        let config = {
            let gpu = self.gpu.as_ref().ok_or(RnHostReasonCode::InvalidIntent)?;
            let timeline = self
                .timelines
                .get(&timeline_handle)
                .ok_or(RnHostReasonCode::UnknownTimelineHandle)?;
            timeline.gpu.reject_if_poisoned()?;
            let surface = timeline
                .gpu
                .surface
                .as_ref()
                .ok_or(RnHostReasonCode::InvalidIntent)?;
            supported_surface_config(surface, &gpu.adapter, width, height)?
        };
        let gpu = self.gpu.as_ref().ok_or(RnHostReasonCode::InvalidIntent)?;
        let timeline = self
            .timelines
            .get_mut(&timeline_handle)
            .ok_or(RnHostReasonCode::UnknownTimelineHandle)?;
        if timeline.gpu.physical_width != width || timeline.gpu.physical_height != height {
            timeline.raster_key = None;
        }
        let surface = timeline
            .gpu
            .surface
            .as_ref()
            .ok_or(RnHostReasonCode::InvalidIntent)?;
        surface.configure(&gpu.ctx.device, &config);
        timeline.gpu.configured(width, height);
        Ok(())
    }

    #[cfg(target_os = "macos")]
    pub(super) fn timeline_resize_physical(
        &mut self,
        timeline_handle: u64,
        width: u32,
        height: u32,
    ) -> Result<(), RnHostReasonCode> {
        require_main_thread()?;
        {
            let timeline = self
                .timelines
                .get(&timeline_handle)
                .ok_or(RnHostReasonCode::UnknownTimelineHandle)?;
            if timeline.destroyed {
                return Err(RnHostReasonCode::LateLifecycleEvent);
            }
            if !timeline.gpu.is_attached() || !timeline.gpu.has_surface() {
                return Err(RnHostReasonCode::InvalidIntent);
            }
            timeline.gpu.reject_if_poisoned()?;
        }
        if width == 0 || height == 0 {
            let timeline = self
                .timelines
                .get_mut(&timeline_handle)
                .ok_or(RnHostReasonCode::UnknownTimelineHandle)?;
            timeline.gpu.physical_width = width;
            timeline.gpu.physical_height = height;
            timeline.gpu.needs_reconfigure = true;
            return Ok(());
        }
        self.configure_timeline_surface(timeline_handle, width, height)
    }

    #[cfg(target_os = "macos")]
    pub(super) fn refresh_timeline_raster(
        &mut self,
        timeline_handle: u64,
        frame: &TimelineFrameBorrow,
    ) -> Result<(), RnHostReasonCode> {
        let (width, height, current_key) = {
            let timeline = self
                .timelines
                .get(&timeline_handle)
                .ok_or(RnHostReasonCode::UnknownTimelineHandle)?;
            (
                timeline.gpu.physical_width,
                timeline.gpu.physical_height,
                timeline.raster_key,
            )
        };
        let next_key = TimelineRasterKey {
            revision: frame.revision,
            projection_generation: frame.projection_generation,
            primary: frame.primary,
            playhead: frame.playhead,
            width,
            height,
        };
        if current_key == Some(next_key) {
            return Ok(());
        }
        let raster = crate::timeline_skia_raster::raster_timeline(frame, width, height)
            .ok_or(RnHostReasonCode::InvalidIntent)?;
        let _stats = raster.stats;
        let gpu = self.gpu.as_ref().ok_or(RnHostReasonCode::InvalidIntent)?;
        let timeline = self
            .timelines
            .get_mut(&timeline_handle)
            .ok_or(RnHostReasonCode::UnknownTimelineHandle)?;
        upload_stage_overlay(
            &gpu.ctx.device,
            &gpu.ctx.queue,
            &gpu.overlay_bind_group_layout,
            &mut timeline.gpu.overlay,
            StageOverlayRaster {
                pixels: raster.pixels,
                width: raster.width,
                height: raster.height,
            },
        );
        timeline.raster_key = Some(next_key);
        Ok(())
    }

    #[cfg(target_os = "macos")]
    pub(super) fn timeline_draw(&mut self, timeline_handle: u64) -> Result<(), RnHostReasonCode> {
        require_main_thread()?;
        let (width, height, attached, needs_reconfigure) = {
            let timeline = self
                .timelines
                .get(&timeline_handle)
                .ok_or(RnHostReasonCode::UnknownTimelineHandle)?;
            if timeline.destroyed {
                return Err(RnHostReasonCode::LateLifecycleEvent);
            }
            timeline.gpu.reject_if_poisoned()?;
            (
                timeline.gpu.physical_width,
                timeline.gpu.physical_height,
                timeline.gpu.is_attached(),
                timeline.gpu.needs_reconfigure,
            )
        };
        if !attached {
            return Err(RnHostReasonCode::InvalidIntent);
        }
        if width == 0 || height == 0 {
            return Ok(());
        }
        if needs_reconfigure {
            self.configure_timeline_surface(timeline_handle, width, height)?;
        }
        let frame_borrow = self.timeline_frame_borrow()?;
        self.refresh_timeline_raster(timeline_handle, &frame_borrow)?;
        let gpu = self.gpu.as_ref().ok_or(RnHostReasonCode::InvalidIntent)?;
        let timeline = self
            .timelines
            .get(&timeline_handle)
            .ok_or(RnHostReasonCode::UnknownTimelineHandle)?;
        let surface = timeline
            .gpu
            .surface
            .as_ref()
            .ok_or(RnHostReasonCode::InvalidIntent)?;
        match surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) => {
                draw_timeline_seat(gpu, frame, timeline.gpu.overlay.as_ref());
                self.timelines
                    .get_mut(&timeline_handle)
                    .ok_or(RnHostReasonCode::UnknownTimelineHandle)?
                    .gpu
                    .presented(false);
                Ok(())
            }
            CurrentSurfaceTexture::Suboptimal(frame) => {
                draw_timeline_seat(gpu, frame, timeline.gpu.overlay.as_ref());
                self.timelines
                    .get_mut(&timeline_handle)
                    .ok_or(RnHostReasonCode::UnknownTimelineHandle)?
                    .gpu
                    .presented(true);
                Ok(())
            }
            CurrentSurfaceTexture::Timeout | CurrentSurfaceTexture::Occluded => Ok(()),
            CurrentSurfaceTexture::Outdated => {
                self.timelines
                    .get_mut(&timeline_handle)
                    .ok_or(RnHostReasonCode::UnknownTimelineHandle)?
                    .gpu
                    .outdated();
                Ok(())
            }
            CurrentSurfaceTexture::Lost => {
                self.timelines
                    .get_mut(&timeline_handle)
                    .ok_or(RnHostReasonCode::UnknownTimelineHandle)?
                    .gpu
                    .lost();
                Err(RnHostReasonCode::InvalidIntent)
            }
            CurrentSurfaceTexture::Validation => {
                self.timelines
                    .get_mut(&timeline_handle)
                    .ok_or(RnHostReasonCode::UnknownTimelineHandle)?
                    .gpu
                    .validation_failed();
                Err(RnHostReasonCode::InvalidIntent)
            }
        }
    }

    #[cfg(target_os = "macos")]
    pub(super) fn timeline_detach_surface(
        &mut self,
        timeline_handle: u64,
    ) -> Result<u64, RnHostReasonCode> {
        require_main_thread()?;
        let timeline = self
            .timelines
            .get_mut(&timeline_handle)
            .ok_or(RnHostReasonCode::UnknownTimelineHandle)?;
        if timeline.destroyed {
            return Err(RnHostReasonCode::LateLifecycleEvent);
        }
        timeline.gpu_detach_surface();
        Ok(timeline.gpu.surface_epoch)
    }

    #[cfg(target_os = "macos")]
    pub(super) fn detach_all_timeline_surfaces(&mut self) {
        for timeline in self.timelines.values_mut() {
            timeline.gpu_detach_surface();
        }
    }
}
