//! Stage/Timeline surface と App 向け幾何型。GPU binding の状態機械を型と一緒に置く。

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
pub(super) struct HostGpuBundle {
    pub(super) ctx: Arc<GpuCtx>,
    pub(super) instance: wgpu::Instance,
    pub(super) adapter: wgpu::Adapter,
    pub(super) _preview: StaticPreview,
    pub(super) preview_pipeline: wgpu::RenderPipeline,
    pub(super) preview_bind_group: wgpu::BindGroup,
    pub(super) overlay_pipeline: wgpu::RenderPipeline,
    pub(super) overlay_bind_group_layout: wgpu::BindGroupLayout,
}

#[cfg(target_os = "macos")]
pub(super) struct StageOverlayGpu {
    pub(super) texture: wgpu::Texture,
    pub(super) _view: wgpu::TextureView,
    pub(super) _sampler: wgpu::Sampler,
    pub(super) bind_group: wgpu::BindGroup,
    pub(super) width: u32,
    pub(super) height: u32,
}

#[cfg(target_os = "macos")]
pub(super) struct StageGpuBinding {
    pub(super) surface_epoch: u64,
    pub(super) last_presented_epoch: Option<u64>,
    pub(super) physical_width: u32,
    pub(super) physical_height: u32,
    pub(super) layer_ptr: usize,
    pub(super) surface: Option<Surface<'static>>,
    pub(super) needs_reconfigure: bool,
    pub(super) poisoned: bool,
    pub(super) overlay: Option<StageOverlayGpu>,
    pub(super) overlay_upload_key: Option<OverlayUploadKey>,
}

#[cfg(target_os = "macos")]
impl StageGpuBinding {
    pub(super) fn detached() -> Self {
        Self {
            surface_epoch: 0,
            last_presented_epoch: None,
            physical_width: 0,
            physical_height: 0,
            layer_ptr: 0,
            surface: None,
            needs_reconfigure: false,
            poisoned: false,
            overlay: None,
            overlay_upload_key: None,
        }
    }

    pub(super) fn is_attached(&self) -> bool {
        self.layer_ptr != 0
    }

    pub(super) fn has_surface(&self) -> bool {
        self.surface.is_some()
    }

    pub(super) fn reject_if_poisoned(&self) -> Result<(), RnHostReasonCode> {
        if self.poisoned {
            Err(RnHostReasonCode::InvalidIntent)
        } else {
            Ok(())
        }
    }

    pub(super) fn validate_attach(&self, layer_ptr: usize) -> Result<(), RnHostReasonCode> {
        self.reject_if_poisoned()?;
        if layer_ptr == 0 || self.is_attached() {
            return Err(RnHostReasonCode::InvalidIntent);
        }
        Ok(())
    }

    pub(super) fn configured(&mut self, width: u32, height: u32) {
        if self.physical_width != width || self.physical_height != height {
            self.overlay_upload_key = None;
        }
        self.physical_width = width;
        self.physical_height = height;
        self.needs_reconfigure = false;
        self.surface_epoch = self.surface_epoch.saturating_add(1);
    }

    pub(super) fn presented(&mut self, suboptimal: bool) {
        self.last_presented_epoch = Some(self.surface_epoch);
        self.needs_reconfigure = suboptimal;
    }

    pub(super) fn outdated(&mut self) {
        self.needs_reconfigure = true;
    }

    pub(super) fn acquisition_deferred(&mut self) {}

    pub(super) fn lost(&mut self) {
        self.surface = None;
        self.layer_ptr = 0;
        self.physical_width = 0;
        self.physical_height = 0;
        self.needs_reconfigure = false;
        self.surface_epoch = self.surface_epoch.saturating_add(1);
        self.overlay = None;
        self.overlay_upload_key = None;
    }

    pub(super) fn validation_failed(&mut self) {
        self.poisoned = true;
        self.needs_reconfigure = false;
        self.surface_epoch = self.surface_epoch.saturating_add(1);
    }

    fn detach(&mut self) {
        if !self.is_attached() {
            return;
        }
        self.surface = None;
        self.layer_ptr = 0;
        self.physical_width = 0;
        self.physical_height = 0;
        self.last_presented_epoch = None;
        self.needs_reconfigure = false;
        self.poisoned = false;
        self.overlay = None;
        self.overlay_upload_key = None;
        self.surface_epoch = self.surface_epoch.saturating_add(1);
    }
}

/// Host 内 transient の最新 pointer。Document / revision / primary には載せない。
#[derive(Debug, Clone, PartialEq)]
pub(super) struct StagePointerTransient {
    pub(super) phase: String,
    pub(super) view_local_x: f64,
    pub(super) view_local_y: f64,
    pub(super) sequence: u64,
}

pub(super) struct RnStageSurface {
    pub(super) host_handle: u64,
    pub(super) mounted: bool,
    pub(super) destroyed: bool,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) scale_factor: f64,
    pub(super) focused: bool,
    pub(super) pointer: Option<StagePointerTransient>,
    #[cfg(target_os = "macos")]
    pub(super) gpu: StageGpuBinding,
}

pub(super) struct RnTimelineSurface {
    pub(super) host_handle: u64,
    pub(super) destroyed: bool,
    #[cfg(target_os = "macos")]
    pub(super) gpu: StageGpuBinding,
    #[cfg(target_os = "macos")]
    pub(super) raster_key: Option<TimelineRasterKey>,
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TimelineRasterKey {
    pub(super) revision: u64,
    pub(super) projection_generation: u64,
    pub(super) primary: Option<LayerId>,
    pub(super) playhead: RationalTime,
    pub(super) width: u32,
    pub(super) height: u32,
}

#[cfg(target_os = "macos")]
pub(crate) struct TimelineFrameBorrow {
    pub(crate) revision: u64,
    pub(crate) projection_generation: u64,
    pub(crate) document: Arc<motolii_doc::Document>,
    pub(crate) projection: crate::timeline_projection::TimelineProjection,
    pub(crate) primary: Option<LayerId>,
    pub(crate) playhead: RationalTime,
}

/// Stage実フレーム。native合成が同一device上でtextureを読む。
#[derive(Debug)]
#[doc(hidden)]
pub struct AppStageFrame {
    pub texture: wgpu::Texture,
    pub width: u32,
    pub height: u32,
    pub revision: String,
    pub generation: String,
    pub time: RationalTime,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[doc(hidden)]
pub enum AppStageTransformEdit {
    TranslateWorld([f64; 2]),
    RotateZ(f64),
    Scale([f64; 2]),
}

#[derive(Debug, Clone, PartialEq)]
#[doc(hidden)]
pub struct AppStageGeometryLayer {
    pub layer_id: String,
    pub corners: [[f64; 2]; 4],
    pub position: [f64; 2],
    pub rotation: f64,
    pub scale: [f64; 2],
}

#[derive(Debug, Clone, PartialEq)]
#[doc(hidden)]
pub struct AppStageGeometry {
    pub layers: Vec<AppStageGeometryLayer>,
    pub layers_truncated: bool,
}

#[derive(Debug)]
#[doc(hidden)]
pub struct AppStageTransformPreview {
    pub geometry: AppStageGeometry,
}

#[derive(Debug, Error)]
#[doc(hidden)]
pub enum AppStageTransformError {
    #[error("Stage host is unavailable")]
    HostUnavailable,
    #[error("The Document changed during the gesture; try again")]
    StaleDocument,
    #[error("The selected layer is no longer available")]
    TargetUnavailable,
    #[error("The selected layer has no editable Stage transform")]
    TransformUnavailable,
    #[error("This animated transform can only be edited on an existing keyframe")]
    OffKeyframe,
    #[error("This transform value is not editable")]
    UnsupportedProperty,
    #[error("The transform result is not finite")]
    NonFinite,
    #[error("The transform did not change")]
    NoChange,
    #[error("Document preview failed: {0}")]
    Preview(String),
    #[error("Stage preview render failed: {0}")]
    Render(String),
    #[error("Document commit failed: {0}")]
    Commit(String),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[doc(hidden)]
pub enum HostRenderFrameResult {
    Unchanged,
    Rendered,
    Failed,
}

impl RnStageSurface {
    pub(super) fn gpu_detach_surface(&mut self) {
        self.gpu.detach();
    }
}

#[cfg(target_os = "macos")]
impl RnTimelineSurface {
    pub(super) fn gpu_detach_surface(&mut self) {
        self.gpu.detach();
        self.raster_key = None;
    }
}
