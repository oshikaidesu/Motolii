//! Host/Stage/Timeline handle 台帳。単一 Mutex で寿命を閉じる。

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

pub(super) struct RnHostRegistry {
    pub(super) next_host_handle: u64,
    pub(super) next_stage_handle: u64,
    pub(super) next_timeline_handle: u64,
    pub(super) hosts: HashMap<u64, RnProductHost>,
    pub(super) destroyed_hosts: HashSet<u64>,
    pub(super) destroyed_stages: HashSet<u64>,
    pub(super) destroyed_timelines: HashSet<u64>,
}

impl Default for RnHostRegistry {
    fn default() -> Self {
        Self {
            next_host_handle: 1,
            next_stage_handle: 1,
            next_timeline_handle: 1,
            hosts: HashMap::new(),
            destroyed_hosts: HashSet::new(),
            destroyed_stages: HashSet::new(),
            destroyed_timelines: HashSet::new(),
        }
    }
}

impl RnHostRegistry {
    pub(super) fn create_host(&mut self, project_path: &Path) -> Result<u64, RnHostError> {
        if !self.hosts.is_empty() {
            return Err(RnHostError::HostAlreadyExists);
        }
        let runtime = open_project_runtime(project_path).map_err(RnHostError::OpenProject)?;
        let handle = self.next_host_handle;
        self.next_host_handle = self
            .next_host_handle
            .checked_add(1)
            .ok_or(RnHostError::HostHandleExhausted)?;
        self.hosts.insert(
            handle,
            RnProductHost {
                runtime,
                projection_generation: 0,
                current_time: RationalTime::ZERO,
                primary: None,
                stages: HashMap::new(),
                timelines: HashMap::new(),
                destroyed: false,
                stage_frame_runtime: None,
                stage_video: None,
                stage_frame_last: None,
                playback_session: None,
                playback_caches: HashMap::new(),
                playback_at_revision: None,
                inspector_preview: None,
                inspector_preview_epoch: 0,
                media_library: MediaLibrary::with_default_root(),
                #[cfg(target_os = "macos")]
                gpu: None,
            },
        );
        // 開いた Document が read_snapshot / dispatch の正本。fixture clip はここで足さない。
        Ok(handle)
    }

    pub(super) fn register_stage(&mut self, host_handle: u64) -> Result<u64, RnHostError> {
        let Some(host) = self.hosts.get_mut(&host_handle) else {
            return if self.destroyed_hosts.contains(&host_handle) {
                Err(RnHostError::DestroyedHost(host_handle))
            } else {
                Err(RnHostError::UnknownHost(host_handle))
            };
        };
        let stage_handle = self.next_stage_handle;
        self.next_stage_handle = self
            .next_stage_handle
            .checked_add(1)
            .ok_or(RnHostError::StageHandleExhausted)?;
        host.register_stage(host_handle, stage_handle)?;
        Ok(stage_handle)
    }

    pub(super) fn destroy_stage(&mut self, stage_handle: u64) -> Result<(), RnHostError> {
        let host_handle = self.hosts.values().find_map(|host| {
            host.stages
                .get(&stage_handle)
                .map(|stage| stage.host_handle)
        });
        let Some(host_handle) = host_handle else {
            return if self.destroyed_stages.contains(&stage_handle) {
                Err(RnHostError::DestroyedStage(stage_handle))
            } else {
                Err(RnHostError::UnknownStage(stage_handle))
            };
        };
        let host = self
            .hosts
            .get_mut(&host_handle)
            .ok_or(RnHostError::UnknownHost(host_handle))?;
        host.destroy_stage(stage_handle)
    }

    pub(super) fn register_timeline(&mut self, host_handle: u64) -> Result<u64, RnHostError> {
        let Some(host) = self.hosts.get_mut(&host_handle) else {
            return if self.destroyed_hosts.contains(&host_handle) {
                Err(RnHostError::DestroyedHost(host_handle))
            } else {
                Err(RnHostError::UnknownHost(host_handle))
            };
        };
        let timeline_handle = self.next_timeline_handle;
        self.next_timeline_handle = self
            .next_timeline_handle
            .checked_add(1)
            .ok_or(RnHostError::TimelineHandleExhausted)?;
        host.register_timeline(host_handle, timeline_handle)?;
        Ok(timeline_handle)
    }

    pub(super) fn destroy_timeline(&mut self, timeline_handle: u64) -> Result<(), RnHostError> {
        let host_handle = self.hosts.values().find_map(|host| {
            host.timelines
                .get(&timeline_handle)
                .map(|timeline| timeline.host_handle)
        });
        let Some(host_handle) = host_handle else {
            return if self.destroyed_timelines.contains(&timeline_handle) {
                Err(RnHostError::DestroyedTimeline(timeline_handle))
            } else {
                Err(RnHostError::UnknownTimeline(timeline_handle))
            };
        };
        let host = self
            .hosts
            .get_mut(&host_handle)
            .ok_or(RnHostError::UnknownHost(host_handle))?;
        host.destroy_timeline(timeline_handle)
    }

    pub(super) fn destroy_host(&mut self, host_handle: u64) -> Result<(), RnHostError> {
        let Some(host) = self.hosts.get_mut(&host_handle) else {
            return if self.destroyed_hosts.contains(&host_handle) {
                Err(RnHostError::DestroyedHost(host_handle))
            } else {
                Err(RnHostError::UnknownHost(host_handle))
            };
        };
        self.destroyed_stages.extend(host.stages.keys().copied());
        self.destroyed_timelines
            .extend(host.timelines.keys().copied());
        host.destroy(host_handle)?;
        self.hosts.remove(&host_handle);
        self.destroyed_hosts.insert(host_handle);
        Ok(())
    }

    pub(super) fn read_snapshot(
        &mut self,
        host_handle: u64,
    ) -> Result<WireProductSnapshot, RnHostError> {
        let Some(host) = self.hosts.get_mut(&host_handle) else {
            return if self.destroyed_hosts.contains(&host_handle) {
                Err(RnHostError::DestroyedHost(host_handle))
            } else {
                Err(RnHostError::UnknownHost(host_handle))
            };
        };
        if host.destroyed {
            return Err(RnHostError::DestroyedHost(host_handle));
        }
        let runtime_diagnostic = match host.runtime.reconcile_pending_commit() {
            Ok(Some(published)) => {
                host.adopt_published(published);
                None
            }
            Ok(None) => None,
            Err(error) => Some(document_runtime_reason(&error)),
        };
        Ok(host.snapshot_wire_with_runtime_diagnostic(host_handle, runtime_diagnostic))
    }

    pub(super) fn dispatch_intent_json(
        &mut self,
        host_handle: u64,
        intent_json: &str,
    ) -> Result<String, RnHostError> {
        let Some(host) = self.hosts.get_mut(&host_handle) else {
            return if self.destroyed_hosts.contains(&host_handle) {
                Err(RnHostError::DestroyedHost(host_handle))
            } else {
                Err(RnHostError::UnknownHost(host_handle))
            };
        };
        if host.destroyed {
            return Err(RnHostError::DestroyedHost(host_handle));
        }
        // set_time は frame index のみ。旧 time 秒 wire・非整数・欠落はここで typed 拒否する。
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(intent_json) {
            if value.get("kind").and_then(|kind| kind.as_str()) == Some("set_time") {
                let frame_is_i64 = value
                    .get("frame")
                    .map(|frame| frame.is_i64())
                    .unwrap_or(false);
                if value.get("time").is_some() || !frame_is_i64 {
                    return encode_snapshot_json(&reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    ));
                }
            }
        }
        let intent: WireIntentEnvelope = match serde_json::from_str(intent_json) {
            Ok(intent) => intent,
            Err(_) => {
                return encode_snapshot_json(&reject(
                    diagnostic(
                        RnHostReasonCode::InvalidIntent,
                        Some(host_handle),
                        None,
                        None,
                        None,
                    ),
                    None,
                ));
            }
        };
        if intent.version != WIRE_VERSION || intent.direction != RN_TO_HOST {
            let response = reject(
                diagnostic(
                    RnHostReasonCode::InvalidIntent,
                    Some(host_handle),
                    intent
                        .stage_handle
                        .as_ref()
                        .and_then(|value| value.parse().ok()),
                    None,
                    None,
                ),
                None,
            );
            return encode_snapshot_json(&response);
        }
        let response = host.dispatch_intent(host_handle, intent);
        encode_snapshot_json(&response)
    }
}

pub(super) fn registry() -> &'static Mutex<RnHostRegistry> {
    static REGISTRY: OnceLock<Mutex<RnHostRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(RnHostRegistry::default()))
}

pub(super) fn lock_registry() -> std::sync::MutexGuard<'static, RnHostRegistry> {
    registry()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub(super) fn with_registry<T>(
    f: impl FnOnce(&mut RnHostRegistry) -> Result<T, RnHostError>,
) -> Result<T, RnHostError> {
    let mut guard = lock_registry();
    f(&mut guard)
}
