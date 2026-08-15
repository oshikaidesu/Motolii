//! Host の playback session。snapshot は host。

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
use super::key_projection::*;
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
    pub(super) fn pause_playback(&mut self) {
        let Some(session) = self.playback_session.take() else {
            self.playback_at_revision = None;
            return;
        };
        self.playback_at_revision = None;
        let duration = self.runtime.snapshot().composition.duration;
        if let Ok(time) = session.transport().perceptual_time() {
            if time >= RationalTime::ZERO {
                self.current_time = if duration >= RationalTime::ZERO {
                    time.min(duration)
                } else {
                    time
                };
            }
        }
    }

    pub(super) fn begin_playback(&mut self) -> Result<(), RnHostReasonCode> {
        let document = self.runtime.snapshot();
        let start_frame = canonical_playback_start_frame(self.current_time)?;
        let mut caches = std::mem::take(&mut self.playback_caches);
        let program = AudioProgram::from_document(
            document.as_ref(),
            self.runtime.project_root().as_deref(),
            &mut caches,
        )
        .map_err(|_| RnHostReasonCode::InvalidIntent)?;
        self.playback_caches = caches;
        let fps = document.composition.fps;
        #[cfg(target_os = "macos")]
        let gpu = self.gpu.as_ref().map(|bundle| &*bundle.ctx);
        #[cfg(not(target_os = "macos"))]
        let gpu = None;
        let session =
            PlaybackSession::open_default(Arc::new(program), start_frame, fps, Quality::DRAFT, gpu)
                .map_err(|_| RnHostReasonCode::InvalidIntent)?;
        self.playback_session = Some(session);
        self.playback_at_revision = Some(self.runtime.document_revision());
        Ok(())
    }

    /// L=再生開始、K=停止。既にその状態なら no-op。新しい player は開かない。
    pub(super) fn ensure_playback(
        &mut self,
        host_handle: u64,
        want_playing: bool,
    ) -> WireIntentResponse {
        if self.playback_session.is_some() == want_playing {
            return accept(self.snapshot_wire(host_handle));
        }
        if want_playing {
            match self.begin_playback() {
                Ok(()) => {
                    if !self.bump_projection_generation() {
                        self.pause_playback();
                        return reject(
                            diagnostic(
                                RnHostReasonCode::InvalidIntent,
                                Some(host_handle),
                                None,
                                None,
                                None,
                            ),
                            Some(self.snapshot_wire(host_handle)),
                        );
                    }
                    accept(self.snapshot_wire(host_handle))
                }
                Err(reason) => reject(
                    diagnostic(reason, Some(host_handle), None, None, None),
                    Some(self.snapshot_wire(host_handle)),
                ),
            }
        } else {
            self.pause_playback();
            if !self.bump_projection_generation() {
                return reject(
                    diagnostic(
                        RnHostReasonCode::InvalidIntent,
                        Some(host_handle),
                        None,
                        None,
                        None,
                    ),
                    Some(self.snapshot_wire(host_handle)),
                );
            }
            accept(self.snapshot_wire(host_handle))
        }
    }

    pub(super) fn seek_to_frame(&mut self, host_handle: u64, frame: i64) -> WireIntentResponse {
        self.pause_playback();
        let (fps, duration) = {
            let snapshot = self.runtime.snapshot();
            (snapshot.composition.fps, snapshot.composition.duration)
        };
        let Ok(time) = RationalTime::try_from_frame(frame, fps) else {
            return self.invalid_intent(host_handle);
        };
        if time < RationalTime::ZERO || time > duration {
            return self.invalid_intent(host_handle);
        }
        if time == self.current_time {
            return accept(self.snapshot_wire(host_handle));
        }
        let Some(next_generation) = self.projection_generation.checked_add(1) else {
            return self.invalid_intent(host_handle);
        };
        self.current_time = time;
        self.projection_generation = next_generation;
        #[cfg(target_os = "macos")]
        self.refresh_stage_overlays().ok();
        accept(self.snapshot_wire(host_handle))
    }

    pub(super) fn shuttle_reverse(&mut self, host_handle: u64) -> WireIntentResponse {
        let was_playing = self.playback_session.is_some();
        self.pause_playback();
        let fps = self.runtime.snapshot().composition.fps;
        let Ok(frame) = self.current_time.try_to_frame_floor(fps) else {
            return self.invalid_intent(host_handle);
        };
        if frame <= 0 {
            if was_playing && !self.bump_projection_generation() {
                return reject(
                    diagnostic(
                        RnHostReasonCode::InvalidIntent,
                        Some(host_handle),
                        None,
                        None,
                        None,
                    ),
                    Some(self.snapshot_wire(host_handle)),
                );
            }
            return accept(self.snapshot_wire(host_handle));
        }
        self.seek_to_frame(host_handle, frame - 1)
    }

    pub(super) fn pump_playback(&mut self) {
        if self.playback_session.is_none() {
            return;
        }
        if self.playback_at_revision != Some(self.runtime.document_revision()) {
            self.pause_playback();
            return;
        }
        let duration = self.runtime.snapshot().composition.duration;
        let plan = match self
            .playback_session
            .as_mut()
            .expect("session checked")
            .transport_mut()
            .next_frame_plan()
        {
            Ok(plan) => plan,
            Err(_) => {
                self.playback_session = None;
                self.playback_at_revision = None;
                return;
            }
        };
        if duration >= RationalTime::ZERO && plan.timeline_time >= duration {
            self.current_time = duration;
            self.playback_session = None;
            self.playback_at_revision = None;
            let _ = self.bump_projection_generation();
            return;
        }
        if plan.timeline_time < RationalTime::ZERO {
            self.pause_playback();
            return;
        }
        if plan.timeline_time != self.current_time {
            self.current_time = plan.timeline_time;
            let _ = self.bump_projection_generation();
        }
    }
}
