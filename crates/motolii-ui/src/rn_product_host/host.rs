//! RnProductHost 本体と snapshot/playback/登録寿命。dispatch と GPU は隣module。

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

pub(super) struct RnProductHost {
    pub(super) runtime: DocumentEditRuntime,
    pub(super) projection_generation: u64,
    /// Document 外の transient 評価時刻。初期値は ZERO。
    pub(super) current_time: RationalTime,
    pub(super) primary: Option<LayerId>,
    pub(super) stages: HashMap<u64, RnStageSurface>,
    pub(super) timelines: HashMap<u64, RnTimelineSurface>,
    pub(super) destroyed: bool,
    /// Arc: GPU submitをregistry lock外で回すためcloneして持ち出す(F10)。
    pub(super) stage_frame_runtime: Option<Arc<motolii_plugin::PluginRuntime>>,
    /// Export と同じ FrameReader 束。GPU 初回 render で作る。
    pub(super) stage_video: Option<VideoSourceBinder>,
    /// dirty gate: 前回返した (revision, generation, time, preview_epoch)。
    pub(super) stage_frame_last: Option<(String, String, RationalTime, u64)>,
    pub(super) playback_session: Option<PlaybackSession>,
    pub(super) playback_caches: HashMap<(String, u32), Arc<PcmCache>>,
    pub(super) playback_at_revision: Option<u64>,
    pub(super) inspector_preview: Option<Command>,
    pub(super) inspector_preview_epoch: u64,
    pub(super) media_library: MediaLibrary,
    #[cfg(target_os = "macos")]
    pub(super) gpu: Option<HostGpuBundle>,
}

impl RnProductHost {
    pub(super) fn snapshot_wire(&self, host_handle: u64) -> WireProductSnapshot {
        self.snapshot_wire_with_runtime_diagnostic(host_handle, None)
    }

    pub(super) fn snapshot_wire_with_runtime_diagnostic(
        &self,
        host_handle: u64,
        runtime_diagnostic: Option<RnHostReasonCode>,
    ) -> WireProductSnapshot {
        let document = self.runtime.snapshot();
        let mut selection = Vec::new();
        if let Some(primary) = self.primary {
            selection.push(WireStageSelection {
                layer_id: primary.get().to_string(),
            });
        }
        selection.truncate(MAX_STAGE_SELECTION);

        let bounds = layers_in_track_order(document.as_ref())
            .into_iter()
            .take(MAX_STAGE_BOUNDS)
            .map(|(layer_id, name)| WireStageBound {
                layer_id: layer_id.get().to_string(),
                display_name: name,
            })
            .collect::<Vec<_>>();

        // stage seat と同じ評価文脈: current_time + 空 DataTracks。
        // Inspector preview は Document を書き換えず、同じ投影へ載せる。
        let previewed_document = self.inspector_preview.as_ref().and_then(|command| {
            let mut previewed = (*document).clone();
            command.apply(&mut previewed).ok()?;
            previewed.validate().ok()?;
            Some(previewed)
        });
        let params_document = previewed_document.as_ref().unwrap_or(document.as_ref());
        let stage_geometry = project_stage_geometry_wire(
            params_document,
            EvaluationTime::new(self.current_time),
            &DataTracks::new(),
        );
        let (timeline, truncated_total) = project_timeline(document.as_ref());

        WireProductSnapshot {
            version: WIRE_VERSION,
            direction: HOST_TO_RN.to_owned(),
            role: PRODUCT_ROLE.to_owned(),
            host_handle: host_handle.to_string(),
            revision: self.runtime.document_revision().to_string(),
            projection_generation: self.projection_generation.to_string(),
            current_time: self.current_time,
            primary_layer_id: self.primary.map(|layer| layer.get().to_string()),
            history: WireHistoryProjection {
                can_undo: self.runtime.can_undo(),
                can_redo: self.runtime.can_redo(),
            },
            truncated_total,
            stage: WireStageProjection { selection, bounds },
            stage_geometry,
            timeline,
            catalog: wire_catalog_projection(),
            library: wire_library_projection(self.media_library.project()),
            selected_doc_params: project_selected_doc_params(
                params_document,
                self.primary,
                self.current_time,
            ),
            diagnostics: runtime_diagnostic
                .or_else(|| {
                    self.runtime
                        .is_write_blocked()
                        .then_some(RnHostReasonCode::DocumentWriteBlocked)
                })
                .map(|reason| diagnostic(reason, Some(host_handle), None, None, None))
                .into_iter()
                .collect(),
            playback_state: if self.playback_session.is_some() {
                WirePlaybackState::Playing
            } else {
                WirePlaybackState::Idle
            },
        }
    }

    /// Document 変更を Host transient へ載せ、次の read_snapshot / dispatch が同じ live snapshot を返す。
    pub(super) fn adopt_published(&mut self, published: PublishedDocument) {
        self.primary = published.primary;
        self.projection_generation = published.projection_generation;
        self.stage_frame_last = None;
        #[cfg(target_os = "macos")]
        self.refresh_stage_overlays().ok();
    }

    pub(super) fn accept_live_snapshot(
        &mut self,
        host_handle: u64,
        published: Option<PublishedDocument>,
    ) -> WireIntentResponse {
        if let Some(published) = published {
            self.adopt_published(published);
        }
        accept(self.snapshot_wire(host_handle))
    }

    pub(super) fn bump_projection_generation(&mut self) -> bool {
        let Some(next) = self.projection_generation.checked_add(1) else {
            return false;
        };
        self.projection_generation = next;
        true
    }

    pub(super) fn invalid_intent(&self, host_handle: u64) -> WireIntentResponse {
        reject(
            diagnostic(
                RnHostReasonCode::InvalidIntent,
                Some(host_handle),
                None,
                None,
                None,
            ),
            None,
        )
    }

    pub(super) fn reject_document_runtime(
        &self,
        host_handle: u64,
        error: &DocumentEditRuntimeError,
    ) -> WireIntentResponse {
        with_message(
            reject(
                diagnostic(
                    document_runtime_reason(error),
                    Some(host_handle),
                    None,
                    None,
                    None,
                ),
                Some(self.snapshot_wire(host_handle)),
            ),
            error.to_string(),
        )
    }

    pub(super) fn commit_stage_property(
        &mut self,
        host_handle: u64,
        command: Command,
    ) -> WireIntentResponse {
        let mut queue = DocumentEditQueue::default();
        // Opacity の SetProperty は StageTransform が拒む。scale/rotation だけ通る。
        if !queue.push_stage_transform(command) {
            return self.invalid_intent(host_handle);
        }
        match self
            .runtime
            .process_next(&mut queue, self.primary, self.projection_generation)
        {
            Ok(Some(published)) => self.accept_live_snapshot(host_handle, Some(published)),
            Ok(None) => self.accept_live_snapshot(host_handle, None),
            Err(error) => self.reject_document_runtime(host_handle, &error),
        }
    }

    pub(super) fn try_set_inspector_preview(&mut self, command: Command) -> bool {
        let Some(next_generation) = self.projection_generation.checked_add(1) else {
            return false;
        };
        self.inspector_preview = Some(command);
        self.inspector_preview_epoch = self.inspector_preview_epoch.saturating_add(1);
        self.projection_generation = next_generation;
        true
    }

    pub(super) fn clear_inspector_preview(&mut self) {
        if self.inspector_preview.take().is_some() {
            self.inspector_preview_epoch = self.inspector_preview_epoch.saturating_add(1);
        }
    }

    pub(super) fn accept_inspector_preview_cancel(
        &mut self,
        host_handle: u64,
    ) -> WireIntentResponse {
        if self.inspector_preview.is_none() {
            return accept(self.snapshot_wire(host_handle));
        }
        let Some(next_generation) = self.projection_generation.checked_add(1) else {
            return reject(
                diagnostic(
                    RnHostReasonCode::ProjectionGenerationExhausted,
                    Some(host_handle),
                    None,
                    None,
                    None,
                ),
                None,
            );
        };
        self.clear_inspector_preview();
        self.projection_generation = next_generation;
        accept(self.snapshot_wire(host_handle))
    }
}

impl RnProductHost {
    pub(super) fn register_stage(
        &mut self,
        host_handle: u64,
        stage_handle: u64,
    ) -> Result<(), RnHostError> {
        if self.destroyed {
            return Err(RnHostError::DestroyedHost(host_handle));
        }
        self.stages.insert(
            stage_handle,
            RnStageSurface {
                host_handle,
                mounted: false,
                destroyed: false,
                width: 0,
                height: 0,
                scale_factor: 1.0,
                focused: false,
                pointer: None,
                #[cfg(target_os = "macos")]
                gpu: StageGpuBinding::detached(),
            },
        );
        Ok(())
    }

    pub(super) fn destroy_stage(&mut self, stage_handle: u64) -> Result<(), RnHostError> {
        let Some(stage) = self.stages.get_mut(&stage_handle) else {
            return Err(RnHostError::UnknownStage(stage_handle));
        };
        if stage.destroyed {
            return Err(RnHostError::DestroyedStage(stage_handle));
        }
        #[cfg(target_os = "macos")]
        stage.gpu_detach_surface();
        stage.destroyed = true;
        stage.mounted = false;
        Ok(())
    }

    pub(super) fn register_timeline(
        &mut self,
        host_handle: u64,
        timeline_handle: u64,
    ) -> Result<(), RnHostError> {
        if self.destroyed {
            return Err(RnHostError::DestroyedHost(host_handle));
        }
        self.timelines.insert(
            timeline_handle,
            RnTimelineSurface {
                host_handle,
                destroyed: false,
                #[cfg(target_os = "macos")]
                gpu: StageGpuBinding::detached(),
                #[cfg(target_os = "macos")]
                raster_key: None,
            },
        );
        Ok(())
    }

    pub(super) fn destroy_timeline(&mut self, timeline_handle: u64) -> Result<(), RnHostError> {
        let Some(timeline) = self.timelines.get_mut(&timeline_handle) else {
            return Err(RnHostError::UnknownTimeline(timeline_handle));
        };
        if timeline.destroyed {
            return Err(RnHostError::DestroyedTimeline(timeline_handle));
        }
        #[cfg(target_os = "macos")]
        timeline.gpu_detach_surface();
        timeline.destroyed = true;
        Ok(())
    }
}

impl RnProductHost {
    pub(super) fn destroy(&mut self, host_handle: u64) -> Result<(), RnHostError> {
        if self.destroyed {
            return Err(RnHostError::DestroyedHost(host_handle));
        }
        #[cfg(target_os = "macos")]
        self.detach_all_stage_surfaces();
        #[cfg(target_os = "macos")]
        self.detach_all_timeline_surfaces();
        self.destroyed = true;
        self.stages.clear();
        self.timelines.clear();
        #[cfg(target_os = "macos")]
        {
            self.gpu = None;
        }
        Ok(())
    }
}
