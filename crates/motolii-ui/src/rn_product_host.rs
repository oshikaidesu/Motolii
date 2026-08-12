//! Wave R0: product-private React Native Host seam.
//!
//! DocumentEditRuntime を単一 writer として保持し、revision 付き read-only snapshot と
//! lifecycle/read intent だけを RN へ投影する。

use std::collections::{HashMap, HashSet};
#[cfg(target_os = "macos")]
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;
#[cfg(target_os = "macos")]
use std::sync::Arc;
use std::sync::{Mutex, OnceLock};

use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat, PixelSize, Quality, RationalTime};
use motolii_doc::{
    build_document_frame_graph, layer_names_for_item, Affine2D, Clip, Command, DocParam, DocValue,
    EvaluationTime, ItemEnvelope, KeyframeId, LayerId, ParentLocator, TrackItem,
};
use motolii_eval::{DataTracks, Interp};
use motolii_gpu::GpuCtx;
use motolii_plugins_firstparty::first_party_runtime;
use motolii_render::{render_graph_cached, RenderGraphInputs, RenderSession};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(target_os = "macos")]
use wgpu::{
    Color, CompositeAlphaMode, CurrentSurfaceTexture, LoadOp, Operations, PresentMode,
    RenderPassColorAttachment, RenderPassDescriptor, StoreOp, Surface, SurfaceConfiguration,
    SurfaceTargetUnsafe, TextureFormat, TextureUsages,
};

use crate::document_edit_runtime::{
    AddPositionKeyRequest, DocumentEditQueue, DocumentEditRuntime, DocumentEditRuntimeError,
    PlaceRectangleRequest, RemovePositionKeyRequest, SetPositionConstRequest,
    SetPositionKeyInterpRequest, SetPositionKeyTimeRequest, SetPositionKeyValueRequest,
};
use crate::timeline_move_gesture::TimelineMoveRequest;
use crate::timeline_trim_gesture::TimelineTrimRequest;
use crate::shell::{open_project_runtime, ShellError};
use crate::stage_geometry_projection::{
    project_stage_geometry, StageLayerProjection,
};
use crate::stage_hit_test::{
    hit_test_projected_layers, view_local_in_stage, view_local_to_canonical, StageHit,
    StageHitTestReject,
};
use crate::stage_overlay_gpu::{overlay_dimensions_match, overlay_dirty, OverlayUploadKey};
use crate::stage_overlay_raster::{raster_selection_outline, StageOverlayRaster};
#[cfg(target_os = "macos")]
use crate::static_preview::{bootstrap_frame_desc, prepare_in_setup_worker, StaticPreview};
use crate::{CommandId, DocumentCommandRequest, DomainIntent, InputPhase, RouterOutput};

const WIRE_VERSION: u8 = 1;
const HOST_TO_RN: &str = "host-to-rn";
const RN_TO_HOST: &str = "rn-to-host";
const PRODUCT_ROLE: &str = "product-runtime-seat";
const MAX_STAGE_BOUNDS: usize = 16;
const MAX_STAGE_SELECTION: usize = 16;
const MAX_POSITION_KEYS: usize = 64;
const MAX_DIAGNOSTICS: usize = 8;
const MAX_JSON_BYTES: usize = 16_384;
const MAX_SNAPSHOT_JSON_BYTES: usize = 131_072;
#[cfg(target_os = "macos")]
const MAX_PROJECT_PATH_BYTES: usize = 4_096;

#[derive(Debug, Error)]
#[doc(hidden)]
pub enum RnHostError {
    #[error("failed to open project runtime")]
    OpenProject(#[source] ShellError),
    #[error("a product host is already active")]
    HostAlreadyExists,
    #[error("host handle space exhausted")]
    HostHandleExhausted,
    #[error("stage handle space exhausted")]
    StageHandleExhausted,
    #[error("timeline handle space exhausted")]
    TimelineHandleExhausted,
    #[error("host registry lock was poisoned")]
    RegistryLockPoisoned,
    #[error(transparent)]
    Serialize(#[from] serde_json::Error),
    #[error("json payload exceeds {MAX_JSON_BYTES} bytes")]
    PayloadTooLarge,
    #[error("project path is empty")]
    EmptyProjectPath,
    #[error("host handle {0} is unknown")]
    UnknownHost(u64),
    #[error("stage handle {0} is unknown")]
    UnknownStage(u64),
    #[error("timeline handle {0} is unknown")]
    UnknownTimeline(u64),
    #[error("host handle {0} was already destroyed")]
    DestroyedHost(u64),
    #[error("stage handle {0} was already destroyed")]
    DestroyedStage(u64),
    #[error("timeline handle {0} was already destroyed")]
    DestroyedTimeline(u64),
    #[error("invalid utf-8 in wire payload")]
    InvalidUtf8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[doc(hidden)]
pub enum RnHostReasonCode {
    HostAlreadyExists,
    InvalidProjectPath,
    UnknownHostHandle,
    UnknownStageHandle,
    UnknownTimelineHandle,
    DestroyedHostHandle,
    DestroyedStageHandle,
    DestroyedTimelineHandle,
    InvalidIntent,
    StaleProjectionGeneration,
    LateLifecycleEvent,
    DoubleDestroy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RnHostDiagnostic {
    reason: RnHostReasonCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    host_handle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stage_handle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeline_handle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_projection_generation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actual_projection_generation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct WireStageBound {
    layer_id: String,
    display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct WireStageSelection {
    layer_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct WireProductSnapshot {
    version: u8,
    direction: String,
    role: String,
    host_handle: String,
    revision: String,
    projection_generation: String,
    /// Host transient の現在評価時刻。Document / revision には載せない。
    current_time: RationalTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    primary_layer_id: Option<String>,
    stage: WireStageProjection,
    /// Available layer の world 適用済み canonical corners（v1・camera 不使用）。
    stage_geometry: WireStageGeometryProjection,
    timeline: WireTimelineProjection,
    diagnostics: Vec<RnHostDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct WireStageGeometryProjection {
    layers: Vec<WireStageGeometryLayer>,
    layers_truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct WireStageGeometryLayer {
    layer_id: String,
    /// CCW・local rect 左下起点。world 適用済み canonical。
    corners: [[f64; 2]; 4],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct WireStageProjection {
    selection: Vec<WireStageSelection>,
    bounds: Vec<WireStageBound>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct WireTimelineProjection {
    fps: Fps,
    layers: Vec<WireTimelineLayer>,
    layers_truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct WireTimelineLayer {
    layer_id: String,
    display_name: String,
    start: RationalTime,
    duration: RationalTime,
    position_keys: Vec<WireTimelinePositionKey>,
    keys_truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct WireTimelinePositionKey {
    key_id: String,
    time: RationalTime,
    /// DocValue::Vec2 のみ投影。他型は field 欠落(編集不可)。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    value: Option<[f64; 2]>,
}

#[derive(Debug, Clone, PartialEq)]
#[doc(hidden)]
pub struct RnProductSnapshotForTest {
    pub revision: String,
    pub projection_generation: String,
    pub current_time: RationalTime,
    pub primary_layer_id: Option<String>,
    pub layer_ids: Vec<String>,
    pub timeline: RnTimelineProjectionForTest,
}

#[derive(Debug, Clone, PartialEq)]
#[doc(hidden)]
pub struct RnTimelineProjectionForTest {
    pub fps: Fps,
    pub layers: Vec<RnTimelineLayerForTest>,
    pub layers_truncated: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[doc(hidden)]
pub struct RnTimelineLayerForTest {
    pub layer_id: String,
    pub display_name: String,
    pub start: RationalTime,
    pub duration: RationalTime,
    pub position_keys: Vec<RnTimelinePositionKeyForTest>,
    pub keys_truncated: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[doc(hidden)]
pub struct RnTimelinePositionKeyForTest {
    pub key_id: String,
    pub time: RationalTime,
    pub value: Option<[f64; 2]>,
}

#[derive(Debug, Clone, PartialEq)]
#[doc(hidden)]
pub struct RnHostTestIntent {
    pub kind: String,
    pub stage_handle: Option<u64>,
    pub projection_generation: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub scale_factor: Option<f64>,
    pub focused: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
#[doc(hidden)]
pub struct RnHostTestResponse {
    pub accepted: bool,
    pub reason: Option<RnHostReasonCode>,
    pub snapshot: Option<RnProductSnapshotForTest>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct WireIntentEnvelope {
    version: u8,
    direction: String,
    kind: String,
    host_handle: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    stage_handle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    projection_generation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    scale_factor: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    focused: Option<bool>,
    /// stage_pointer: `down` | `drag` | `up` | `cancel`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    phase: Option<String>,
    /// stage_pointer: view-local logical X（physical / scale_factor と混同しない）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    view_local_x: Option<f64>,
    /// stage_pointer: view-local logical Y
    #[serde(default, skip_serializing_if = "Option::is_none")]
    view_local_y: Option<f64>,
    /// stage_pointer: 単調増加 sequence（二重配送検出用。本seamでは調停しない）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sequence: Option<u64>,
    /// set_time: 評価 frame index。欠落・非整数は typed 拒否。暗黙 clamp しない。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    frame: Option<i64>,
    /// place_rectangle: canonical document position
    #[serde(default, skip_serializing_if = "Option::is_none")]
    position: Option<[f64; 2]>,
    /// place_rectangle: document playhead time
    #[serde(default, skip_serializing_if = "Option::is_none")]
    playhead: Option<RationalTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    key_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    time: Option<RationalTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    new: Option<[f64; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    interp: Option<Interp>,
    /// move_layer_by: canonical world delta
    #[serde(default, skip_serializing_if = "Option::is_none")]
    delta: Option<[f64; 2]>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct WireIntentResponse {
    version: u8,
    accepted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    snapshot: Option<WireProductSnapshot>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    diagnostics: Vec<RnHostDiagnostic>,
}

#[cfg(target_os = "macos")]
struct HostGpuBundle {
    ctx: Arc<GpuCtx>,
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    _preview: StaticPreview,
    preview_pipeline: wgpu::RenderPipeline,
    preview_bind_group: wgpu::BindGroup,
    overlay_pipeline: wgpu::RenderPipeline,
    overlay_bind_group_layout: wgpu::BindGroupLayout,
}

#[cfg(target_os = "macos")]
struct StageOverlayGpu {
    texture: wgpu::Texture,
    _view: wgpu::TextureView,
    _sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
}

#[cfg(target_os = "macos")]
struct StageGpuBinding {
    surface_epoch: u64,
    last_presented_epoch: Option<u64>,
    physical_width: u32,
    physical_height: u32,
    layer_ptr: usize,
    surface: Option<Surface<'static>>,
    needs_reconfigure: bool,
    poisoned: bool,
    overlay: Option<StageOverlayGpu>,
    overlay_upload_key: Option<OverlayUploadKey>,
}

#[cfg(target_os = "macos")]
impl StageGpuBinding {
    fn detached() -> Self {
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

    fn is_attached(&self) -> bool {
        self.layer_ptr != 0
    }

    fn has_surface(&self) -> bool {
        self.surface.is_some()
    }

    fn reject_if_poisoned(&self) -> Result<(), RnHostReasonCode> {
        if self.poisoned {
            Err(RnHostReasonCode::InvalidIntent)
        } else {
            Ok(())
        }
    }

    fn validate_attach(&self, layer_ptr: usize) -> Result<(), RnHostReasonCode> {
        self.reject_if_poisoned()?;
        if layer_ptr == 0 || self.is_attached() {
            return Err(RnHostReasonCode::InvalidIntent);
        }
        Ok(())
    }

    fn configured(&mut self, width: u32, height: u32) {
        if self.physical_width != width || self.physical_height != height {
            self.overlay_upload_key = None;
        }
        self.physical_width = width;
        self.physical_height = height;
        self.needs_reconfigure = false;
        self.surface_epoch = self.surface_epoch.saturating_add(1);
    }

    fn presented(&mut self, suboptimal: bool) {
        self.last_presented_epoch = Some(self.surface_epoch);
        self.needs_reconfigure = suboptimal;
    }

    fn outdated(&mut self) {
        self.needs_reconfigure = true;
    }

    fn acquisition_deferred(&mut self) {}

    fn lost(&mut self) {
        self.surface = None;
        self.layer_ptr = 0;
        self.physical_width = 0;
        self.physical_height = 0;
        self.needs_reconfigure = false;
        self.surface_epoch = self.surface_epoch.saturating_add(1);
        self.overlay = None;
        self.overlay_upload_key = None;
    }

    fn validation_failed(&mut self) {
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
struct StagePointerTransient {
    phase: String,
    view_local_x: f64,
    view_local_y: f64,
    sequence: u64,
}

struct RnStageSurface {
    host_handle: u64,
    mounted: bool,
    destroyed: bool,
    width: u32,
    height: u32,
    scale_factor: f64,
    focused: bool,
    pointer: Option<StagePointerTransient>,
    #[cfg(target_os = "macos")]
    gpu: StageGpuBinding,
}

struct RnTimelineSurface {
    host_handle: u64,
    destroyed: bool,
    #[cfg(target_os = "macos")]
    gpu: StageGpuBinding,
    #[cfg(target_os = "macos")]
    raster_key: Option<TimelineRasterKey>,
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TimelineRasterKey {
    revision: u64,
    projection_generation: u64,
    primary: Option<LayerId>,
    playhead: RationalTime,
    width: u32,
    height: u32,
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[doc(hidden)]
pub enum HostRenderFrameResult {
    Unchanged,
    Rendered,
    Failed,
}

struct RnProductHost {
    runtime: DocumentEditRuntime,
    projection_generation: u64,
    /// Document 外の transient 評価時刻。初期値は ZERO。
    current_time: RationalTime,
    primary: Option<LayerId>,
    stages: HashMap<u64, RnStageSurface>,
    timelines: HashMap<u64, RnTimelineSurface>,
    destroyed: bool,
    stage_frame_runtime: Option<motolii_plugin::PluginRuntime>,
    /// dirty gate: 前回返した (revision, generation, time)。
    stage_frame_last: Option<(String, String, RationalTime)>,
    #[cfg(target_os = "macos")]
    gpu: Option<HostGpuBundle>,
}

impl RnProductHost {
    fn snapshot_wire(&self, host_handle: u64) -> WireProductSnapshot {
        let document = self.runtime.snapshot();
        let mut selection = Vec::new();
        if let Some(primary) = self.primary {
            selection.push(WireStageSelection {
                layer_id: primary.get().to_string(),
            });
        }
        selection.truncate(MAX_STAGE_SELECTION);

        let bounds = document
            .layers
            .iter()
            .take(MAX_STAGE_BOUNDS)
            .map(|(layer_id, name)| WireStageBound {
                layer_id: layer_id.get().to_string(),
                display_name: name.to_owned(),
            })
            .collect::<Vec<_>>();

        // stage seat と同じ評価文脈: current_time + 空 DataTracks。
        let stage_geometry = project_stage_geometry_wire(
            document.as_ref(),
            EvaluationTime::new(self.current_time),
            &DataTracks::new(),
        );

        WireProductSnapshot {
            version: WIRE_VERSION,
            direction: HOST_TO_RN.to_owned(),
            role: PRODUCT_ROLE.to_owned(),
            host_handle: host_handle.to_string(),
            revision: self.runtime.document_revision().to_string(),
            projection_generation: self.projection_generation.to_string(),
            current_time: self.current_time,
            primary_layer_id: self.primary.map(|layer| layer.get().to_string()),
            stage: WireStageProjection { selection, bounds },
            stage_geometry,
            timeline: project_timeline(document.as_ref()),
            diagnostics: Vec::new(),
        }
    }

    fn dispatch_intent(
        &mut self,
        host_handle: u64,
        intent: WireIntentEnvelope,
    ) -> WireIntentResponse {
        if self.destroyed {
            return reject(
                diagnostic(
                    RnHostReasonCode::DestroyedHostHandle,
                    Some(host_handle),
                    None,
                    None,
                    None,
                ),
                None,
            );
        }

        if intent.host_handle != host_handle.to_string() {
            return reject(
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
        }

        if let Some(expected) = intent.projection_generation.as_deref() {
            if expected != self.projection_generation.to_string() {
                return reject(
                    diagnostic(
                        RnHostReasonCode::StaleProjectionGeneration,
                        Some(host_handle),
                        intent
                            .stage_handle
                            .as_ref()
                            .and_then(|value| value.parse().ok()),
                        Some(self.projection_generation.to_string()),
                        Some(expected.to_owned()),
                    ),
                    None,
                );
            }
        }

        match intent.kind.as_str() {
            "read_snapshot" => accept(self.snapshot_wire(host_handle)),
            "set_time" => {
                // frame index だけを受け、Composition.fps で RationalTime へ解決する。
                // 負・duration 超過・try_from_frame 失敗は暗黙 clamp せず typed 拒否。
                let Some(frame) = intent.frame else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let (fps, duration) = {
                    let snapshot = self.runtime.snapshot();
                    (snapshot.composition.fps, snapshot.composition.duration)
                };
                let Ok(time) = RationalTime::try_from_frame(frame, fps) else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                if time < RationalTime::ZERO || time > duration {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                }
                if time == self.current_time {
                    // 同一時刻の再設定は no-op。generation は進めない。
                    return accept(self.snapshot_wire(host_handle));
                }
                // Host 内に CU-104E 枯渇 preflight は無い。飽和・wrap せず typed 拒否する。
                let Some(next_generation) = self.projection_generation.checked_add(1) else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                self.current_time = time;
                self.projection_generation = next_generation;
                self.refresh_stage_overlays().ok();
                accept(self.snapshot_wire(host_handle))
            }
            "place_rectangle" => {
                let Some(position) = intent.position else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(playhead) = intent.playhead else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                if !position.iter().all(|value| value.is_finite()) {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                }
                let mut queue = DocumentEditQueue::default();
                queue.push_place_rectangle(PlaceRectangleRequest { position, playhead });
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => {
                        self.primary = published.primary;
                        self.projection_generation = published.projection_generation;
                        self.refresh_stage_overlays().ok();
                        accept(self.snapshot_wire(host_handle))
                    }
                    Ok(None) => accept(self.snapshot_wire(host_handle)),
                    Err(_) => reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    ),
                }
            }
            "add_position_key" | "set_position_key_value" | "set_position_key_interp" => {
                let Some(target) = intent
                    .target
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(LayerId::from_raw)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(time) = intent.time else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let new = match intent.kind.as_str() {
                    "set_position_key_value" => {
                        let Some(new) = intent.new else {
                            return reject(
                                diagnostic(
                                    RnHostReasonCode::InvalidIntent,
                                    Some(host_handle),
                                    None,
                                    None,
                                    None,
                                ),
                                None,
                            );
                        };
                        if !new.iter().all(|value| value.is_finite()) {
                            return reject(
                                diagnostic(
                                    RnHostReasonCode::InvalidIntent,
                                    Some(host_handle),
                                    None,
                                    None,
                                    None,
                                ),
                                None,
                            );
                        }
                        Some(new)
                    }
                    _ => None,
                };
                let interp = if intent.kind == "set_position_key_interp" {
                    let Some(interp) = intent.interp else {
                        return reject(
                            diagnostic(
                                RnHostReasonCode::InvalidIntent,
                                Some(host_handle),
                                None,
                                None,
                                None,
                            ),
                            None,
                        );
                    };
                    Some(interp)
                } else {
                    None
                };

                if self.primary != Some(target) {
                    return accept(self.snapshot_wire(host_handle));
                }

                let mut queue = DocumentEditQueue::default();
                match intent.kind.as_str() {
                    "add_position_key" => {
                        queue.push_add_position_key(AddPositionKeyRequest { target, time });
                    }
                    "set_position_key_value" => {
                        let Some((key, old)) =
                            position_key_at(self.runtime.snapshot().as_ref(), target, time)
                        else {
                            return reject(
                                diagnostic(
                                    RnHostReasonCode::InvalidIntent,
                                    Some(host_handle),
                                    None,
                                    None,
                                    None,
                                ),
                                None,
                            );
                        };
                        queue.push_set_position_key_value(SetPositionKeyValueRequest {
                            target,
                            key,
                            old,
                            new: new.expect("validated position value"),
                        });
                    }
                    "set_position_key_interp" => {
                        let Some((key, _)) =
                            position_key_at(self.runtime.snapshot().as_ref(), target, time)
                        else {
                            return reject(
                                diagnostic(
                                    RnHostReasonCode::InvalidIntent,
                                    Some(host_handle),
                                    None,
                                    None,
                                    None,
                                ),
                                None,
                            );
                        };
                        queue.push_set_position_key_interp(SetPositionKeyInterpRequest {
                            target,
                            key,
                            interp: interp.expect("validated interpolation"),
                        });
                    }
                    _ => unreachable!("matched position key intent"),
                }
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => {
                        self.primary = published.primary;
                        self.projection_generation = published.projection_generation;
                        accept(self.snapshot_wire(host_handle))
                    }
                    Ok(None) => accept(self.snapshot_wire(host_handle)),
                    Err(_) => reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    ),
                }
            }
            "set_position_key_time" => {
                let Some(target) = intent
                    .target
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(LayerId::from_raw)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(key) = intent
                    .key_id
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(KeyframeId::from_raw)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(new) = intent.time else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(old) =
                    position_key_time_at(self.runtime.snapshot().as_ref(), target, key)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let mut queue = DocumentEditQueue::default();
                queue.push_set_position_key_time(SetPositionKeyTimeRequest {
                    target,
                    key,
                    old,
                    new,
                });
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => {
                        self.primary = published.primary;
                        self.projection_generation = published.projection_generation;
                        accept(self.snapshot_wire(host_handle))
                    }
                    Ok(None) => accept(self.snapshot_wire(host_handle)),
                    Err(_) => reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    ),
                }
            }
            "remove_position_key" => {
                let Some(target) = intent
                    .target
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(LayerId::from_raw)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(key) = intent
                    .key_id
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(KeyframeId::from_raw)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let mut queue = DocumentEditQueue::default();
                queue.push_remove_position_key(RemovePositionKeyRequest { target, key });
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => {
                        self.primary = published.primary;
                        self.projection_generation = published.projection_generation;
                        accept(self.snapshot_wire(host_handle))
                    }
                    Ok(None) => accept(self.snapshot_wire(host_handle)),
                    Err(_) => reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    ),
                }
            }
            "set_clip_start" | "trim_clip_in" | "trim_clip_out" => {
                let Some(target) = intent
                    .target
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(LayerId::from_raw)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(time) = intent.time else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let mut queue = DocumentEditQueue::default();
                match intent.kind.as_str() {
                    "set_clip_start" => {
                        queue.push_move_clip(TimelineMoveRequest {
                            layer: target,
                            new_start: time,
                        });
                    }
                    "trim_clip_in" => {
                        queue.push_trim_clip(TimelineTrimRequest::In {
                            layer: target,
                            new_start: time,
                        });
                    }
                    "trim_clip_out" => {
                        queue.push_trim_clip(TimelineTrimRequest::Out {
                            layer: target,
                            new_end: time,
                        });
                    }
                    _ => unreachable!("matched clip edit intent"),
                }
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => {
                        self.primary = published.primary;
                        self.projection_generation = published.projection_generation;
                        accept(self.snapshot_wire(host_handle))
                    }
                    Ok(None) => accept(self.snapshot_wire(host_handle)),
                    Err(_) => reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    ),
                }
            }
            "select_layer" => {
                let Some(target) = intent
                    .target
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(LayerId::from_raw)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let mut queue = DocumentEditQueue::default();
                queue.push_replace_primary(target);
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(None) => accept(self.snapshot_wire(host_handle)),
                    Ok(Some(published)) => {
                        self.primary = published.primary;
                        self.projection_generation = published.projection_generation;
                        self.refresh_stage_overlays().ok();
                        accept(self.snapshot_wire(host_handle))
                    }
                    Err(DocumentEditRuntimeError::SelectionTargetNotFound(_))
                    | Err(DocumentEditRuntimeError::ProjectionGenerationExhausted) => reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    ),
                    Err(_) => reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    ),
                }
            }
            "clear_selection" => {
                let mut queue = DocumentEditQueue::default();
                queue.push_clear_primary();
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(None) => accept(self.snapshot_wire(host_handle)),
                    Ok(Some(published)) => {
                        self.primary = published.primary;
                        self.projection_generation = published.projection_generation;
                        self.refresh_stage_overlays().ok();
                        accept(self.snapshot_wire(host_handle))
                    }
                    Err(_) => reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    ),
                }
            }
            "delete_layer" => {
                let Some(target) = intent
                    .target
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(LayerId::from_raw)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let document = self.runtime.snapshot();
                let Some((parent, index, item)) =
                    find_track_item_location(document.as_ref(), target)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Ok(layer_names) = layer_names_for_item(document.as_ref(), &item) else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Ok(request) = DocumentCommandRequest::try_new(
                    DomainIntent::DeleteTargetedItems,
                    vec![Command::RemoveTrackItem {
                        parent,
                        index,
                        layer_names,
                        item,
                    }],
                ) else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let output = RouterOutput::Intent {
                    phase: InputPhase::Click,
                    id: CommandId::try_new("motolii.rn.delete_layer")
                        .expect("static command id"),
                    intent: DomainIntent::DeleteTargetedItems,
                };
                let mut queue = DocumentEditQueue::default();
                if queue.push_prepared(output, Some(request)).is_err() {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                }
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => {
                        self.primary = published.primary;
                        self.projection_generation = published.projection_generation;
                        self.refresh_stage_overlays().ok();
                        accept(self.snapshot_wire(host_handle))
                    }
                    Ok(None) => accept(self.snapshot_wire(host_handle)),
                    Err(_) => reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    ),
                }
            }
            "undo" | "redo" => {
                let (intent, id) = if intent.kind == "undo" {
                    (DomainIntent::Undo, "motolii.rn.undo")
                } else {
                    (DomainIntent::Redo, "motolii.rn.redo")
                };
                let output = RouterOutput::Intent {
                    phase: InputPhase::Press,
                    id: CommandId::try_new(id).expect("static command id"),
                    intent,
                };
                let mut queue = DocumentEditQueue::default();
                if queue.push_prepared(output, None).is_err() {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                }
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => {
                        self.primary = published.primary;
                        self.projection_generation = published.projection_generation;
                        self.refresh_stage_overlays().ok();
                        accept(self.snapshot_wire(host_handle))
                    }
                    Ok(None) => accept(self.snapshot_wire(host_handle)),
                    Err(DocumentEditRuntimeError::NothingToUndo)
                    | Err(DocumentEditRuntimeError::NothingToRedo) => reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    ),
                    Err(_) => reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    ),
                }
            }
            "move_layer_by" => {
                let Some(target) = intent
                    .target
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(LayerId::from_raw)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(delta) = intent.delta else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                if !delta.iter().all(|value| value.is_finite()) {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                }
                let snapshot = self.runtime.snapshot();
                let Some(envelope) = find_envelope_in_document(snapshot.as_ref(), target) else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let tracks = DataTracks::new();
                let projection = match project_stage_geometry(
                    snapshot.as_ref(),
                    EvaluationTime::new(self.current_time),
                    &tracks,
                ) {
                    Ok(projection) => projection,
                    Err(_) => {
                        return reject(
                            diagnostic(
                                RnHostReasonCode::InvalidIntent,
                                Some(host_handle),
                                None,
                                None,
                                None,
                            ),
                            None,
                        );
                    }
                };
                let Some(StageLayerProjection::Available(geo)) = projection.get(target).cloned()
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(local_delta) = world_delta_to_position_local(geo.world, delta) else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let mut queue = DocumentEditQueue::default();
                match &envelope.transform.position {
                    DocParam::Const(DocValue::Vec2(old)) => {
                        let new = [old[0] + local_delta[0], old[1] + local_delta[1]];
                        if !new.iter().all(|value| value.is_finite()) {
                            return reject(
                                diagnostic(
                                    RnHostReasonCode::InvalidIntent,
                                    Some(host_handle),
                                    None,
                                    None,
                                    None,
                                ),
                                None,
                            );
                        }
                        queue.push_set_position_const(SetPositionConstRequest {
                            target,
                            old: *old,
                            new,
                        });
                    }
                    DocParam::Keyframes(_) => {
                        let Some((key, old)) =
                            position_key_at(snapshot.as_ref(), target, self.current_time)
                        else {
                            // U4b-0V: off-key は Auto Key せず typed 拒否。
                            return reject(
                                diagnostic(
                                    RnHostReasonCode::InvalidIntent,
                                    Some(host_handle),
                                    None,
                                    None,
                                    None,
                                ),
                                None,
                            );
                        };
                        let new = [old[0] + local_delta[0], old[1] + local_delta[1]];
                        if !new.iter().all(|value| value.is_finite()) {
                            return reject(
                                diagnostic(
                                    RnHostReasonCode::InvalidIntent,
                                    Some(host_handle),
                                    None,
                                    None,
                                    None,
                                ),
                                None,
                            );
                        }
                        queue.push_set_position_key_value(SetPositionKeyValueRequest {
                            target,
                            key,
                            old,
                            new,
                        });
                    }
                    _ => {
                        return reject(
                            diagnostic(
                                RnHostReasonCode::InvalidIntent,
                                Some(host_handle),
                                None,
                                None,
                                None,
                            ),
                            None,
                        );
                    }
                }
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => {
                        self.primary = published.primary;
                        self.projection_generation = published.projection_generation;
                        self.refresh_stage_overlays().ok();
                        accept(self.snapshot_wire(host_handle))
                    }
                    Ok(None) => accept(self.snapshot_wire(host_handle)),
                    Err(_) => reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    ),
                }
            }
            "stage_mount" | "stage_resize" | "stage_focus" | "stage_unmount" | "stage_pointer" => {
                let Some(stage_handle) = intent
                    .stage_handle
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(stage) = self.stages.get_mut(&stage_handle) else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::UnknownStageHandle,
                            Some(host_handle),
                            Some(stage_handle),
                            None,
                            None,
                        ),
                        None,
                    );
                };
                if stage.destroyed {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::LateLifecycleEvent,
                            Some(host_handle),
                            Some(stage_handle),
                            None,
                            None,
                        ),
                        None,
                    );
                }
                // unmount 後の late pointer は lifecycle と同じ late route で拒否する。
                if intent.kind == "stage_pointer" && !stage.mounted {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::LateLifecycleEvent,
                            Some(host_handle),
                            Some(stage_handle),
                            None,
                            None,
                        ),
                        None,
                    );
                }
                let payload_is_valid = match intent.kind.as_str() {
                    "stage_resize" => matches!(
                        (intent.width, intent.height, intent.scale_factor),
                        (Some(width), Some(height), Some(scale_factor))
                            if width > 0
                                && height > 0
                                && scale_factor.is_finite()
                                && scale_factor > 0.0
                    ),
                    "stage_focus" => intent.focused.is_some(),
                    "stage_mount" | "stage_unmount" => true,
                    "stage_pointer" => {
                        let phase_ok = matches!(
                            intent.phase.as_deref(),
                            Some("down" | "drag" | "up" | "cancel")
                        );
                        let coords_ok = matches!(
                            (intent.view_local_x, intent.view_local_y),
                            (Some(x), Some(y)) if x.is_finite() && y.is_finite()
                        );
                        phase_ok && coords_ok && intent.sequence.is_some()
                    }
                    _ => false,
                };
                if !payload_is_valid {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            Some(stage_handle),
                            None,
                            None,
                        ),
                        None,
                    );
                }
                // pointer の selection は stage borrow 解放後に行う。
                let pointer_down = match intent.kind.as_str() {
                    "stage_mount" => {
                        stage.mounted = true;
                        None
                    }
                    "stage_resize" => {
                        stage.width = intent.width.expect("validated resize width");
                        stage.height = intent.height.expect("validated resize height");
                        stage.scale_factor =
                            intent.scale_factor.expect("validated resize scale factor");
                        #[cfg(target_os = "macos")]
                        {
                            stage.gpu.overlay_upload_key = None;
                        }
                        None
                    }
                    "stage_focus" => {
                        stage.focused = intent.focused.expect("validated focus state");
                        None
                    }
                    "stage_unmount" => {
                        #[cfg(target_os = "macos")]
                        stage.gpu_detach_surface();
                        stage.mounted = false;
                        None
                    }
                    "stage_pointer" => {
                        // selection 成否と独立に transient を先に記録する（grain 2）。
                        let phase = intent.phase.expect("validated pointer phase");
                        let view_local_x = intent.view_local_x.expect("validated view_local_x");
                        let view_local_y = intent.view_local_y.expect("validated view_local_y");
                        let sequence = intent.sequence.expect("validated pointer sequence");
                        let width = stage.width;
                        let height = stage.height;
                        stage.pointer = Some(StagePointerTransient {
                            phase: phase.clone(),
                            view_local_x,
                            view_local_y,
                            sequence,
                        });
                        if phase == "down" {
                            Some((view_local_x, view_local_y, width, height))
                        } else {
                            // drag / up / cancel は selection を変更しない。
                            None
                        }
                    }
                    _ => None,
                };
                if let Some((view_local_x, view_local_y, width, height)) = pointer_down {
                    if let Some(response) = self.apply_stage_pointer_selection(
                        host_handle,
                        stage_handle,
                        view_local_x,
                        view_local_y,
                        width,
                        height,
                    ) {
                        return response;
                    }
                }
                accept(self.snapshot_wire(host_handle))
            }
            _ => reject(
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
            ),
        }
    }

    /// `stage_pointer` down の hit-test → 既存 selection writer。
    /// typed 拒否時だけ `Some(reject)`。受理・no-op は `None`（呼び出し側が snapshot を返す）。
    fn apply_stage_pointer_selection(
        &mut self,
        host_handle: u64,
        stage_handle: u64,
        view_local_x: f64,
        view_local_y: f64,
        width: u32,
        height: u32,
    ) -> Option<WireIntentResponse> {
        let canonical = match view_local_to_canonical(view_local_x, view_local_y, width, height) {
            Ok(point) => point,
            Err(StageHitTestReject::ZeroStageExtent) => {
                return Some(reject(
                    diagnostic(
                        RnHostReasonCode::InvalidIntent,
                        Some(host_handle),
                        Some(stage_handle),
                        None,
                        None,
                    ),
                    None,
                ));
            }
        };

        // product path と同じ空 DataTracks。runtime に格納口は無い。
        let tracks = DataTracks::new();
        let document = self.runtime.snapshot();
        let projection = match project_stage_geometry(
            document.as_ref(),
            EvaluationTime::new(self.current_time),
            &tracks,
        ) {
            Ok(projection) => projection,
            Err(_) => {
                // 幾何失敗を選択解除の意思に読み替えない。
                return Some(reject(
                    diagnostic(
                        RnHostReasonCode::InvalidIntent,
                        Some(host_handle),
                        Some(stage_handle),
                        None,
                        None,
                    ),
                    None,
                ));
            }
        };

        let hit = if view_local_in_stage(view_local_x, view_local_y, width, height) {
            hit_test_projected_layers(canonical, &projection)
        } else {
            StageHit::Miss
        };

        let mut queue = DocumentEditQueue::default();
        match hit {
            StageHit::Layer(layer) => queue.push_replace_primary(layer),
            StageHit::Miss => queue.push_clear_primary(),
        }

        match self
            .runtime
            .process_next(&mut queue, self.primary, self.projection_generation)
        {
            Ok(None) => {
                // 存在拒否以外の same-id / already-clear no-op。generation は進めない。
                None
            }
            Ok(Some(published)) => {
                // accepted 変更だけを Host transient へ反映する（直接代入で意図を捏造しない）。
                self.primary = published.primary;
                self.projection_generation = published.projection_generation;
                self.refresh_stage_overlays().ok();
                None
            }
            Err(DocumentEditRuntimeError::SelectionTargetNotFound(_))
            | Err(DocumentEditRuntimeError::ProjectionGenerationExhausted) => Some(reject(
                diagnostic(
                    RnHostReasonCode::InvalidIntent,
                    Some(host_handle),
                    Some(stage_handle),
                    None,
                    None,
                ),
                None,
            )),
            Err(_) => Some(reject(
                diagnostic(
                    RnHostReasonCode::InvalidIntent,
                    Some(host_handle),
                    Some(stage_handle),
                    None,
                    None,
                ),
                None,
            )),
        }
    }

    fn register_stage(&mut self, host_handle: u64, stage_handle: u64) -> Result<(), RnHostError> {
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

    fn destroy_stage(&mut self, stage_handle: u64) -> Result<(), RnHostError> {
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

    fn register_timeline(
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

    fn destroy_timeline(&mut self, timeline_handle: u64) -> Result<(), RnHostError> {
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

    #[cfg(target_os = "macos")]
    fn timeline_frame_borrow(&self) -> Result<TimelineFrameBorrow, RnHostReasonCode> {
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
    fn ensure_gpu(&mut self) -> Result<&mut HostGpuBundle, RnHostReasonCode> {
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
    fn refresh_stage_overlays(&mut self) -> Result<(), RnHostReasonCode> {
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
    fn stage_attach_surface(
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
    fn configure_stage_surface(
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
    fn stage_resize_physical(
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
    fn stage_draw(&mut self, stage_handle: u64) -> Result<(), RnHostReasonCode> {
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
    fn stage_detach_surface(&mut self, stage_handle: u64) -> Result<u64, RnHostReasonCode> {
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
    fn timeline_attach_surface(
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
    fn configure_timeline_surface(
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
    fn timeline_resize_physical(
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
    fn refresh_timeline_raster(
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
    fn timeline_draw(&mut self, timeline_handle: u64) -> Result<(), RnHostReasonCode> {
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
    fn timeline_detach_surface(&mut self, timeline_handle: u64) -> Result<u64, RnHostReasonCode> {
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
    fn detach_all_stage_surfaces(&mut self) {
        let stage_handles = self.stages.keys().copied().collect::<Vec<_>>();
        for stage_handle in stage_handles {
            if let Some(stage) = self.stages.get_mut(&stage_handle) {
                stage.gpu_detach_surface();
            }
        }
    }

    #[cfg(target_os = "macos")]
    fn detach_all_timeline_surfaces(&mut self) {
        for timeline in self.timelines.values_mut() {
            timeline.gpu_detach_surface();
        }
    }

    fn destroy(&mut self, host_handle: u64) -> Result<(), RnHostError> {
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

#[cfg(target_os = "macos")]
fn draw_stage_preview(
    gpu: &HostGpuBundle,
    frame: wgpu::SurfaceTexture,
    overlay: Option<&StageOverlayGpu>,
) {
    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = gpu
        .ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("motolii-rn-stage-preview"),
        });
    {
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("motolii-rn-stage-preview-pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&gpu.preview_pipeline);
        pass.set_bind_group(0, &gpu.preview_bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
    if let Some(overlay) = overlay {
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("motolii-rn-stage-overlay-pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&gpu.overlay_pipeline);
        pass.set_bind_group(0, &overlay.bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
    gpu.ctx.queue.submit(Some(encoder.finish()));
    frame.present();
}

#[cfg(target_os = "macos")]
fn draw_timeline_seat(
    gpu: &HostGpuBundle,
    frame: wgpu::SurfaceTexture,
    raster: Option<&StageOverlayGpu>,
) {
    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = gpu
        .ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("motolii-rn-timeline-seat"),
        });
    {
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("motolii-rn-timeline-seat-pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.02,
                        g: 0.02,
                        b: 0.025,
                        a: 1.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        if let Some(raster) = raster {
            pass.set_pipeline(&gpu.overlay_pipeline);
            pass.set_bind_group(0, &raster.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
    }
    gpu.ctx.queue.submit(Some(encoder.finish()));
    frame.present();
}

#[cfg(target_os = "macos")]
fn upload_stage_overlay(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    current: &mut Option<StageOverlayGpu>,
    raster: StageOverlayRaster,
) {
    let recreate = current
        .as_ref()
        .is_none_or(|overlay| overlay.width != raster.width || overlay.height != raster.height);
    if recreate {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("motolii-rn-stage-overlay-texture"),
            size: wgpu::Extent3d {
                width: raster.width,
                height: raster.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("motolii-rn-stage-overlay-sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-rn-stage-overlay-bind-group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        *current = Some(StageOverlayGpu {
            texture,
            _view: view,
            _sampler: sampler,
            bind_group,
            width: raster.width,
            height: raster.height,
        });
    }
    let overlay = current.as_ref().expect("overlay texture initialized");
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &overlay.texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &raster.pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(raster.width * 4),
            rows_per_image: Some(raster.height),
        },
        wgpu::Extent3d {
            width: raster.width,
            height: raster.height,
            depth_or_array_layers: 1,
        },
    );
}

#[cfg(target_os = "macos")]
fn create_overlay_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("motolii-rn-stage-overlay-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("motolii-rn-stage-overlay-shader"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
            "struct V { @builtin(position) p: vec4<f32>, @location(0) uv: vec2<f32> }\n@vertex fn vs(@builtin(vertex_index) i: u32) -> V { var p = array<vec2<f32>, 3>(vec2(-1., -1.), vec2(3., -1.), vec2(-1., 3.)); var u = array<vec2<f32>, 3>(vec2(0., 1.), vec2(2., 1.), vec2(0., -1.)); var o: V; o.p = vec4(p[i], 0., 1.); o.uv = u[i]; return o; }\n@group(0) @binding(0) var t: texture_2d<f32>;\n@group(0) @binding(1) var s: sampler;\n@fragment fn fs(i: V) -> @location(0) vec4<f32> { return textureSample(t, s, i.uv); }\n",
        )),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("motolii-rn-stage-overlay-pipeline-layout"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("motolii-rn-stage-overlay-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });
    (pipeline, layout)
}

#[cfg(target_os = "macos")]
fn supported_surface_config(
    surface: &Surface<'static>,
    adapter: &wgpu::Adapter,
    width: u32,
    height: u32,
) -> Result<SurfaceConfiguration, RnHostReasonCode> {
    let capabilities = surface.get_capabilities(adapter);
    if !capabilities.formats.contains(&TextureFormat::Bgra8Unorm)
        || !capabilities.present_modes.contains(&PresentMode::Fifo)
        || !capabilities.alpha_modes.contains(&CompositeAlphaMode::Auto)
    {
        return Err(RnHostReasonCode::InvalidIntent);
    }
    Ok(SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: TextureFormat::Bgra8Unorm,
        width,
        height,
        present_mode: PresentMode::Fifo,
        desired_maximum_frame_latency: 2,
        alpha_mode: CompositeAlphaMode::Auto,
        view_formats: Vec::new(),
    })
}

#[cfg(target_os = "macos")]
impl RnStageSurface {
    fn gpu_detach_surface(&mut self) {
        self.gpu.detach();
    }
}

#[cfg(target_os = "macos")]
impl RnTimelineSurface {
    fn gpu_detach_surface(&mut self) {
        self.gpu.detach();
        self.raster_key = None;
    }
}

#[cfg(target_os = "macos")]
fn require_main_thread() -> Result<(), RnHostReasonCode> {
    if objc2::MainThreadMarker::new().is_none() {
        return Err(RnHostReasonCode::InvalidIntent);
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn run_stage_gpu_op(
    host_handle: u64,
    stage_handle: u64,
    f: impl FnOnce(&mut RnProductHost, u64) -> Result<(), RnHostReasonCode>,
) -> Result<String, RnHostReasonCode> {
    let outcome = with_registry(|registry| {
        let host = registry.hosts.get_mut(&host_handle);
        let Some(host) = host else {
            return Ok(Err(if registry.destroyed_hosts.contains(&host_handle) {
                RnHostReasonCode::DestroyedHostHandle
            } else {
                RnHostReasonCode::UnknownHostHandle
            }));
        };
        if host.destroyed {
            return Ok(Err(RnHostReasonCode::DestroyedHostHandle));
        }
        let Some(stage) = host.stages.get(&stage_handle) else {
            return Ok(Err(if registry.destroyed_stages.contains(&stage_handle) {
                RnHostReasonCode::DestroyedStageHandle
            } else {
                RnHostReasonCode::UnknownStageHandle
            }));
        };
        if stage.host_handle != host_handle {
            return Ok(Err(RnHostReasonCode::UnknownStageHandle));
        }
        if let Err(reason) = f(host, stage_handle) {
            return Ok(Err(reason));
        }
        match encode_response(&accept_no_snapshot()) {
            Ok(json) => Ok(Ok(json)),
            Err(_) => Ok(Err(RnHostReasonCode::InvalidIntent)),
        }
    });
    match outcome {
        Ok(inner) => inner,
        Err(_) => Err(RnHostReasonCode::InvalidIntent),
    }
}

#[cfg(target_os = "macos")]
fn write_stage_gpu_op(
    out: *mut u8,
    out_cap: usize,
    host_handle: u64,
    stage_handle: u64,
    f: impl FnOnce(&mut RnProductHost, u64) -> Result<(), RnHostReasonCode>,
) -> i64 {
    if !output_usable(out, out_cap) {
        return -1;
    }
    catch_unwind(AssertUnwindSafe(|| {
        if require_main_thread().is_err() {
            return write_reject(
                out,
                out_cap,
                RnHostReasonCode::InvalidIntent,
                Some(host_handle),
                Some(stage_handle),
            );
        }
        if host_handle == 0 {
            return write_reject(
                out,
                out_cap,
                RnHostReasonCode::UnknownHostHandle,
                Some(0),
                Some(stage_handle),
            );
        }
        if stage_handle == 0 {
            return write_reject(
                out,
                out_cap,
                RnHostReasonCode::UnknownStageHandle,
                Some(host_handle),
                Some(0),
            );
        }
        match run_stage_gpu_op(host_handle, stage_handle, f) {
            Ok(json) => write_bytes(out, out_cap, &json),
            Err(reason) => {
                write_reject(out, out_cap, reason, Some(host_handle), Some(stage_handle))
            }
        }
    }))
    .unwrap_or(-1)
}

#[cfg(target_os = "macos")]
fn run_timeline_gpu_op(
    host_handle: u64,
    timeline_handle: u64,
    f: impl FnOnce(&mut RnProductHost, u64) -> Result<(), RnHostReasonCode>,
) -> Result<String, RnHostReasonCode> {
    let outcome = with_registry(|registry| {
        let Some(host) = registry.hosts.get_mut(&host_handle) else {
            return Ok(Err(if registry.destroyed_hosts.contains(&host_handle) {
                RnHostReasonCode::DestroyedHostHandle
            } else {
                RnHostReasonCode::UnknownHostHandle
            }));
        };
        if host.destroyed {
            return Ok(Err(RnHostReasonCode::DestroyedHostHandle));
        }
        let Some(timeline) = host.timelines.get(&timeline_handle) else {
            return Ok(Err(
                if registry.destroyed_timelines.contains(&timeline_handle) {
                    RnHostReasonCode::DestroyedTimelineHandle
                } else {
                    RnHostReasonCode::UnknownTimelineHandle
                },
            ));
        };
        if timeline.host_handle != host_handle {
            return Ok(Err(RnHostReasonCode::UnknownTimelineHandle));
        }
        if let Err(reason) = f(host, timeline_handle) {
            return Ok(Err(reason));
        }
        match encode_response(&accept_no_snapshot()) {
            Ok(json) => Ok(Ok(json)),
            Err(_) => Ok(Err(RnHostReasonCode::InvalidIntent)),
        }
    });
    match outcome {
        Ok(inner) => inner,
        Err(_) => Err(RnHostReasonCode::InvalidIntent),
    }
}

#[cfg(target_os = "macos")]
fn write_timeline_gpu_op(
    out: *mut u8,
    out_cap: usize,
    host_handle: u64,
    timeline_handle: u64,
    f: impl FnOnce(&mut RnProductHost, u64) -> Result<(), RnHostReasonCode>,
) -> i64 {
    if !output_usable(out, out_cap) {
        return -1;
    }
    catch_unwind(AssertUnwindSafe(|| {
        if require_main_thread().is_err() {
            return write_response(
                out,
                out_cap,
                &reject(
                    timeline_diagnostic(
                        RnHostReasonCode::InvalidIntent,
                        Some(host_handle),
                        Some(timeline_handle),
                    ),
                    None,
                ),
            );
        }
        if host_handle == 0 || timeline_handle == 0 {
            return write_response(
                out,
                out_cap,
                &reject(
                    timeline_diagnostic(
                        if host_handle == 0 {
                            RnHostReasonCode::UnknownHostHandle
                        } else {
                            RnHostReasonCode::UnknownTimelineHandle
                        },
                        Some(host_handle),
                        Some(timeline_handle),
                    ),
                    None,
                ),
            );
        }
        match run_timeline_gpu_op(host_handle, timeline_handle, f) {
            Ok(json) => write_bytes(out, out_cap, &json),
            Err(reason) => write_response(
                out,
                out_cap,
                &reject(
                    timeline_diagnostic(reason, Some(host_handle), Some(timeline_handle)),
                    None,
                ),
            ),
        }
    }))
    .unwrap_or(-1)
}

struct RnHostRegistry {
    next_host_handle: u64,
    next_stage_handle: u64,
    next_timeline_handle: u64,
    hosts: HashMap<u64, RnProductHost>,
    destroyed_hosts: HashSet<u64>,
    destroyed_stages: HashSet<u64>,
    destroyed_timelines: HashSet<u64>,
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
    fn create_host(&mut self, project_path: &Path) -> Result<u64, RnHostError> {
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
            stage_frame_last: None,
            #[cfg(target_os = "macos")]
            gpu: None,
            },
        );
        Ok(handle)
    }

    fn register_stage(&mut self, host_handle: u64) -> Result<u64, RnHostError> {
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

    fn destroy_stage(&mut self, stage_handle: u64) -> Result<(), RnHostError> {
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

    fn register_timeline(&mut self, host_handle: u64) -> Result<u64, RnHostError> {
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

    fn destroy_timeline(&mut self, timeline_handle: u64) -> Result<(), RnHostError> {
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

    fn destroy_host(&mut self, host_handle: u64) -> Result<(), RnHostError> {
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

    fn read_snapshot(&self, host_handle: u64) -> Result<WireProductSnapshot, RnHostError> {
        let Some(host) = self.hosts.get(&host_handle) else {
            return if self.destroyed_hosts.contains(&host_handle) {
                Err(RnHostError::DestroyedHost(host_handle))
            } else {
                Err(RnHostError::UnknownHost(host_handle))
            };
        };
        if host.destroyed {
            return Err(RnHostError::DestroyedHost(host_handle));
        }
        Ok(host.snapshot_wire(host_handle))
    }

    fn dispatch_intent_json(
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
                    return encode_json(&reject(
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
                return encode_json(&reject(
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
            return encode_json(&response);
        }
        let response = host.dispatch_intent(host_handle, intent);
        encode_json(&response)
    }
}

fn registry() -> &'static Mutex<RnHostRegistry> {
    static REGISTRY: OnceLock<Mutex<RnHostRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(RnHostRegistry::default()))
}

fn with_registry<T>(
    f: impl FnOnce(&mut RnHostRegistry) -> Result<T, RnHostError>,
) -> Result<T, RnHostError> {
    let mut guard = registry()
        .lock()
        .map_err(|_| RnHostError::RegistryLockPoisoned)?;
    f(&mut guard)
}

fn encode_json<T: Serialize>(value: &T) -> Result<String, RnHostError> {
    let json = serde_json::to_string(value)?;
    if json.len() > MAX_JSON_BYTES {
        return Err(RnHostError::PayloadTooLarge);
    }
    Ok(json)
}

fn encode_snapshot_json<T: Serialize>(snapshot: &T) -> Result<String, RnHostError> {
    let json = serde_json::to_string(snapshot)?;
    if json.len() > MAX_SNAPSHOT_JSON_BYTES {
        return Err(RnHostError::PayloadTooLarge);
    }
    Ok(json)
}

fn diagnostic(
    reason: RnHostReasonCode,
    host_handle: Option<u64>,
    stage_handle: Option<u64>,
    expected_projection_generation: Option<String>,
    actual_projection_generation: Option<String>,
) -> RnHostDiagnostic {
    RnHostDiagnostic {
        reason,
        host_handle: host_handle.map(|value| value.to_string()),
        stage_handle: stage_handle.map(|value| value.to_string()),
        timeline_handle: None,
        expected_projection_generation,
        actual_projection_generation,
    }
}

fn timeline_diagnostic(
    reason: RnHostReasonCode,
    host_handle: Option<u64>,
    timeline_handle: Option<u64>,
) -> RnHostDiagnostic {
    RnHostDiagnostic {
        reason,
        host_handle: host_handle.map(|value| value.to_string()),
        stage_handle: None,
        timeline_handle: timeline_handle.map(|value| value.to_string()),
        expected_projection_generation: None,
        actual_projection_generation: None,
    }
}

fn accept(snapshot: WireProductSnapshot) -> WireIntentResponse {
    WireIntentResponse {
        version: WIRE_VERSION,
        accepted: true,
        snapshot: Some(snapshot),
        diagnostics: Vec::new(),
    }
}

fn reject(
    diagnostic: RnHostDiagnostic,
    snapshot: Option<WireProductSnapshot>,
) -> WireIntentResponse {
    let mut diagnostics = vec![diagnostic];
    diagnostics.truncate(MAX_DIAGNOSTICS);
    WireIntentResponse {
        version: WIRE_VERSION,
        accepted: false,
        snapshot,
        diagnostics,
    }
}

#[cfg(target_os = "macos")]
fn write_bytes(out: *mut u8, out_cap: usize, payload: &str) -> i64 {
    if out.is_null() || out_cap == 0 {
        return -1;
    }
    let bytes = payload.as_bytes();
    if bytes.len() > out_cap {
        return -(bytes.len() as i64);
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, bytes.len());
    }
    bytes.len() as i64
}

#[cfg(target_os = "macos")]
fn output_usable(out: *mut u8, out_cap: usize) -> bool {
    !out.is_null() && out_cap > 0
}

#[cfg(target_os = "macos")]
fn accept_no_snapshot() -> WireIntentResponse {
    WireIntentResponse {
        version: WIRE_VERSION,
        accepted: true,
        snapshot: None,
        diagnostics: Vec::new(),
    }
}

#[cfg(target_os = "macos")]
fn encode_response(response: &WireIntentResponse) -> Result<String, RnHostError> {
    encode_json(response)
}

#[cfg(target_os = "macos")]
fn write_response(out: *mut u8, out_cap: usize, response: &WireIntentResponse) -> i64 {
    match encode_response(response) {
        Ok(json) => write_bytes(out, out_cap, &json),
        Err(_) => -1,
    }
}

#[cfg(target_os = "macos")]
fn write_reject(
    out: *mut u8,
    out_cap: usize,
    reason: RnHostReasonCode,
    host_handle: Option<u64>,
    stage_handle: Option<u64>,
) -> i64 {
    write_response(
        out,
        out_cap,
        &reject(
            diagnostic(reason, host_handle, stage_handle, None, None),
            None,
        ),
    )
}

#[cfg(target_os = "macos")]
fn map_create_error(error: &RnHostError) -> Option<RnHostReasonCode> {
    match error {
        RnHostError::HostAlreadyExists => Some(RnHostReasonCode::HostAlreadyExists),
        RnHostError::EmptyProjectPath | RnHostError::InvalidUtf8 | RnHostError::OpenProject(_) => {
            Some(RnHostReasonCode::InvalidProjectPath)
        }
        _ => None,
    }
}

#[cfg(target_os = "macos")]
fn map_host_lookup_error(error: &RnHostError) -> Option<RnHostReasonCode> {
    match error {
        RnHostError::UnknownHost(_) => Some(RnHostReasonCode::UnknownHostHandle),
        RnHostError::DestroyedHost(_) => Some(RnHostReasonCode::DestroyedHostHandle),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
fn map_destroy_host_error(error: &RnHostError) -> Option<RnHostReasonCode> {
    match error {
        RnHostError::UnknownHost(_) => Some(RnHostReasonCode::UnknownHostHandle),
        RnHostError::DestroyedHost(_) => Some(RnHostReasonCode::DoubleDestroy),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
fn map_destroy_stage_error(error: &RnHostError) -> Option<RnHostReasonCode> {
    match error {
        RnHostError::UnknownStage(_) => Some(RnHostReasonCode::UnknownStageHandle),
        RnHostError::DestroyedStage(_) => Some(RnHostReasonCode::DoubleDestroy),
        RnHostError::UnknownHost(_) => Some(RnHostReasonCode::UnknownHostHandle),
        RnHostError::DestroyedHost(_) => Some(RnHostReasonCode::DestroyedHostHandle),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
fn map_destroy_timeline_error(error: &RnHostError) -> Option<RnHostReasonCode> {
    match error {
        RnHostError::UnknownTimeline(_) => Some(RnHostReasonCode::UnknownTimelineHandle),
        RnHostError::DestroyedTimeline(_) => Some(RnHostReasonCode::DoubleDestroy),
        RnHostError::UnknownHost(_) => Some(RnHostReasonCode::UnknownHostHandle),
        RnHostError::DestroyedHost(_) => Some(RnHostReasonCode::DestroyedHostHandle),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
fn read_utf8(ptr: *const u8, len: usize, max_len: usize) -> Result<String, RnHostError> {
    if ptr.is_null() || len == 0 {
        return Err(RnHostError::InvalidUtf8);
    }
    if len > max_len {
        return Err(RnHostError::PayloadTooLarge);
    }
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    std::str::from_utf8(slice)
        .map(ToOwned::to_owned)
        .map_err(|_| RnHostError::InvalidUtf8)
}

fn snapshot_for_test(snapshot: WireProductSnapshot) -> RnProductSnapshotForTest {
    RnProductSnapshotForTest {
        revision: snapshot.revision,
        projection_generation: snapshot.projection_generation,
        current_time: snapshot.current_time,
        primary_layer_id: snapshot.primary_layer_id,
        layer_ids: snapshot
            .stage
            .bounds
            .into_iter()
            .map(|bound| bound.layer_id)
            .collect(),
        timeline: RnTimelineProjectionForTest {
            fps: snapshot.timeline.fps,
            layers: snapshot
                .timeline
                .layers
                .into_iter()
                .map(|layer| RnTimelineLayerForTest {
                    layer_id: layer.layer_id,
                    display_name: layer.display_name,
                    start: layer.start,
                    duration: layer.duration,
                    position_keys: layer
                        .position_keys
                        .into_iter()
                        .map(|key| RnTimelinePositionKeyForTest {
                            key_id: key.key_id,
                            time: key.time,
                            value: key.value,
                        })
                        .collect(),
                    keys_truncated: layer.keys_truncated,
                })
                .collect(),
            layers_truncated: snapshot.timeline.layers_truncated,
        },
    }
}

fn project_timeline(document: &motolii_doc::Document) -> WireTimelineProjection {
    let layers_truncated = document.layers.len() > MAX_STAGE_BOUNDS;
    let layers = document
        .layers
        .iter()
        .take(MAX_STAGE_BOUNDS)
        .map(|(layer_id, name)| {
            let (start, duration) = find_first_clip(document, layer_id)
                .map(|clip| (clip.start, clip.duration))
                .unwrap_or((RationalTime::ZERO, RationalTime::ZERO));
            let (position_keys, keys_truncated) = project_position_keys(document, layer_id);
            WireTimelineLayer {
                layer_id: layer_id.get().to_string(),
                display_name: name.to_owned(),
                start,
                duration,
                position_keys,
                keys_truncated,
            }
        })
        .collect();
    WireTimelineProjection {
        fps: document.composition.fps,
        layers,
        layers_truncated,
    }
}

/// Available だけを corners に畳む。評価失敗は空投影（snapshot 自体は落とさない）。
fn project_stage_geometry_wire(
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
        layers.push(WireStageGeometryLayer {
            layer_id: layer_id.get().to_string(),
            corners,
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

fn world_rect_corners(world: motolii_doc::Affine2D, local: [[f64; 2]; 4]) -> [[f64; 2]; 4] {
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
fn world_delta_to_position_local(world: Affine2D, delta: [f64; 2]) -> Option<[f64; 2]> {
    let inv = world.try_invert()?;
    let m = inv.m;
    let local = [
        m[0] * delta[0] + m[1] * delta[1],
        m[3] * delta[0] + m[4] * delta[1],
    ];
    local.iter().all(|value| value.is_finite()).then_some(local)
}

fn find_first_clip(document: &motolii_doc::Document, target: LayerId) -> Option<&Clip> {
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

fn find_envelope_in_document(
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
fn find_track_item_location(
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

fn project_position_keys(
    document: &motolii_doc::Document,
    target: LayerId,
) -> (Vec<WireTimelinePositionKey>, bool) {
    let Some(envelope) = find_envelope_in_document(document, target) else {
        return (Vec::new(), false);
    };
    let DocParam::Keyframes(track) = &envelope.transform.position else {
        return (Vec::new(), false);
    };
    let keys = track.keys();
    let keys_truncated = keys.len() > MAX_POSITION_KEYS;
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
        })
        .collect();
    (position_keys, keys_truncated)
}

fn position_key_at(
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

fn position_key_time_at(
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

fn rational_time_eq(left: RationalTime, right: RationalTime) -> bool {
    let lhs = i128::from(left.num()).checked_mul(i128::from(right.den()));
    let rhs = i128::from(right.num()).checked_mul(i128::from(left.den()));
    match (lhs, rhs) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

fn response_for_test(response: WireIntentResponse) -> RnHostTestResponse {
    RnHostTestResponse {
        accepted: response.accepted,
        reason: response
            .diagnostics
            .first()
            .map(|diagnostic| diagnostic.reason),
        snapshot: response.snapshot.map(snapshot_for_test),
    }
}

pub fn host_create_for_test(project_path: &Path) -> Result<u64, RnHostError> {
    with_registry(|registry| registry.create_host(project_path))
}

/// 評価済み Document の実フレームを Stage 合成へ渡す薄い seam。
/// dirty gate: 前回と同じ (revision, generation, time) なら再renderせず Unchanged。
#[doc(hidden)]
pub fn host_render_frame_for_app(
    host_handle: u64,
    gpu: &GpuCtx,
    session: &mut RenderSession,
    out: &mut Option<AppStageFrame>,
) -> HostRenderFrameResult {
    // Unchanged / Failed では呼び手の既存frameへ触れない(Rendered時のみ上書き)。
    // 冒頭で無条件にNone化するとUnchanged tickごとに実フレームが消える。
    let Ok(mut guard) = registry().lock() else {
        return HostRenderFrameResult::Failed;
    };
    let Some(host) = guard.hosts.get_mut(&host_handle) else {
        return HostRenderFrameResult::Failed;
    };
    if host.destroyed {
        return HostRenderFrameResult::Failed;
    }

    let revision = host.runtime.document_revision().to_string();
    let generation = host.projection_generation.to_string();
    let time = host.current_time;
    if let Some((prev_rev, prev_gen, prev_time)) = host.stage_frame_last.as_ref() {
        if prev_rev == &revision && prev_gen == &generation && *prev_time == time {
            return HostRenderFrameResult::Unchanged;
        }
    }

    let document = host.runtime.snapshot();
    let Some(desc) = frame_desc_from_composition(document.as_ref()) else {
        return HostRenderFrameResult::Failed;
    };

    if host.stage_frame_runtime.is_none() {
        let Ok(runtime) = first_party_runtime() else {
            return HostRenderFrameResult::Failed;
        };
        host.stage_frame_runtime = Some(runtime);
    }

    // product path / render_worker と同じ: 空 DataTracks、project_root=None、Quality::DRAFT。
    let tracks = DataTracks::new();
    let eval = EvaluationTime::new(time);
    let built = {
        let runtime = host
            .stage_frame_runtime
            .as_ref()
            .expect("stage_frame_runtime initialized");
        match build_document_frame_graph(
            document.as_ref(),
            eval,
            desc,
            &tracks,
            runtime,
            None,
        ) {
            Ok(built) => built,
            Err(_) => return HostRenderFrameResult::Failed,
        }
    };
    let rendered = {
        let runtime = host
            .stage_frame_runtime
            .as_ref()
            .expect("stage_frame_runtime initialized");
        match render_graph_cached(
            gpu,
            session,
            time,
            &built.graph,
            &RenderGraphInputs {
                camera: built.camera,
                video_sources: &[],
                source_time: Some(built.source_time),
                plugins: Some(runtime.executors()),
            },
            Quality::DRAFT,
        ) {
            Ok(frame) => frame,
            Err(_) => return HostRenderFrameResult::Failed,
        }
    };

    host.stage_frame_last = Some((revision.clone(), generation.clone(), time));
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

/// composition アスペクトから bootstrap 系の FrameDesc を作る（高さ1080固定）。
fn frame_desc_from_composition(document: &motolii_doc::Document) -> Option<FrameDesc> {
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

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_host_create(
    path: *const u8,
    path_len: usize,
    out_host_handle: *mut u64,
    out: *mut u8,
    out_cap: usize,
) -> i64 {
    if out_host_handle.is_null() {
        return -1;
    }
    unsafe {
        *out_host_handle = 0;
    }
    if !output_usable(out, out_cap) {
        return -1;
    }
    catch_unwind(AssertUnwindSafe(|| {
        let project_path = match read_utf8(path, path_len, MAX_PROJECT_PATH_BYTES) {
            Ok(value) if !value.is_empty() => value,
            Ok(_) => {
                return write_reject(
                    out,
                    out_cap,
                    RnHostReasonCode::InvalidProjectPath,
                    None,
                    None,
                );
            }
            Err(error) => {
                return match map_create_error(&error) {
                    Some(reason) => write_reject(out, out_cap, reason, None, None),
                    None => -1,
                };
            }
        };

        let created = with_registry(|registry| registry.create_host(Path::new(&project_path)));
        match created {
            Ok(host_handle) => {
                let encoded = with_registry(|registry| {
                    let snapshot = registry.read_snapshot(host_handle)?;
                    encode_response(&accept(snapshot))
                });
                match encoded {
                    Ok(json) => {
                        let written = write_bytes(out, out_cap, &json);
                        if written <= 0 {
                            let _ = with_registry(|registry| registry.destroy_host(host_handle));
                            return written;
                        }
                        unsafe {
                            *out_host_handle = host_handle;
                        }
                        written
                    }
                    Err(_) => {
                        let _ = with_registry(|registry| registry.destroy_host(host_handle));
                        -1
                    }
                }
            }
            Err(error) => match map_create_error(&error) {
                Some(reason) => write_reject(out, out_cap, reason, None, None),
                None => -1,
            },
        }
    }))
    .unwrap_or(-1)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_host_destroy(host_handle: u64, out: *mut u8, out_cap: usize) -> i64 {
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
        let outcome = with_registry(|registry| {
            if !registry.hosts.contains_key(&host_handle) {
                return if registry.destroyed_hosts.contains(&host_handle) {
                    Ok(Err(RnHostError::DestroyedHost(host_handle)))
                } else {
                    Ok(Err(RnHostError::UnknownHost(host_handle)))
                };
            }
            let json = encode_response(&accept_no_snapshot())?;
            if json.len() > out_cap {
                return Err(RnHostError::PayloadTooLarge);
            }
            registry.destroy_host(host_handle)?;
            Ok(Ok(json))
        });
        match outcome {
            Ok(Ok(json)) => write_bytes(out, out_cap, &json),
            Ok(Err(error)) => match map_destroy_host_error(&error) {
                Some(reason) => write_reject(out, out_cap, reason, Some(host_handle), None),
                None => -1,
            },
            Err(_) => -1,
        }
    }))
    .unwrap_or(-1)
}

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
pub extern "C" fn motolii_rn_host_read_snapshot_json(
    host_handle: u64,
    out: *mut u8,
    out_cap: usize,
) -> i64 {
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
        match with_registry(|registry| registry.read_snapshot(host_handle)) {
            Ok(snapshot) => match encode_snapshot_json(&snapshot) {
                Ok(json) => write_bytes(out, out_cap, &json),
                Err(_) => -1,
            },
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
pub extern "C" fn motolii_rn_host_dispatch_intent_json(
    host_handle: u64,
    intent_ptr: *const u8,
    intent_len: usize,
    out: *mut u8,
    out_cap: usize,
) -> i64 {
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
        let intent_json = match read_utf8(intent_ptr, intent_len, MAX_JSON_BYTES) {
            Ok(value) => value,
            Err(RnHostError::InvalidUtf8) | Err(RnHostError::PayloadTooLarge) => {
                return write_reject(
                    out,
                    out_cap,
                    RnHostReasonCode::InvalidIntent,
                    Some(host_handle),
                    None,
                );
            }
            Err(_) => return -1,
        };
        match with_registry(|registry| registry.dispatch_intent_json(host_handle, &intent_json)) {
            Ok(response) => write_bytes(out, out_cap, &response),
            Err(error) => match map_host_lookup_error(&error) {
                Some(reason) => write_reject(out, out_cap, reason, Some(host_handle), None),
                None if matches!(error, RnHostError::Serialize(_)) => write_reject(
                    out,
                    out_cap,
                    RnHostReasonCode::InvalidIntent,
                    Some(host_handle),
                    None,
                ),
                None => -1,
            },
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

#[cfg(target_os = "macos")]
#[allow(dead_code)]
const _: fn() = || {
    let _ =
        motolii_rn_host_create as extern "C" fn(*const u8, usize, *mut u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_host_destroy as extern "C" fn(u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_stage_register as extern "C" fn(u64, *mut u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_stage_destroy as extern "C" fn(u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_timeline_register as extern "C" fn(u64, *mut u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_timeline_destroy as extern "C" fn(u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_host_read_snapshot_json as extern "C" fn(u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_host_dispatch_intent_json
        as extern "C" fn(u64, *const u8, usize, *mut u8, usize) -> i64;
    let _ = motolii_rn_stage_attach
        as extern "C" fn(u64, u64, *mut core::ffi::c_void, *mut u8, usize) -> i64;
    let _ = motolii_rn_stage_resize_physical
        as extern "C" fn(u64, u64, u32, u32, *mut u8, usize) -> i64;
    let _ = motolii_rn_stage_draw as extern "C" fn(u64, u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_stage_detach as extern "C" fn(u64, u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_timeline_attach
        as extern "C" fn(u64, u64, *mut core::ffi::c_void, *mut u8, usize) -> i64;
    let _ = motolii_rn_timeline_resize_physical
        as extern "C" fn(u64, u64, u32, u32, *mut u8, usize) -> i64;
    let _ = motolii_rn_timeline_draw as extern "C" fn(u64, u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_timeline_detach as extern "C" fn(u64, u64, *mut u8, usize) -> i64;
};

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Mutex, MutexGuard};

    use motolii_core::{RationalTime, TimeMap};
    use motolii_doc::{
        Clip, ClipSource, CompCameraDoc, DocKeyframe, DocKeyframeTrack, DocParam, DocValue,
        Document, ItemEnvelope, KeyframeId, LayerId, ProjectSession, ResourceLimits,
        SaveProjectOptions, Track, TrackItem, Transform2D, RECT_LAYER_SOURCE,
    };
    use motolii_gpu::download_rgba;
    use motolii_render::RenderSession;
    use motolii_testkit::tmp_dir;

    use super::*;

    static TEST_LOCK: Mutex<()> = Mutex::new(());
    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_lock() -> MutexGuard<'static, ()> {
        TEST_LOCK.lock().expect("test host registry lock")
    }

    fn fixture_path(tag: &str) -> std::path::PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = tmp_dir(&format!("rn-product-host-{tag}-{id}")).join("project.json");
        let mut document = Document::new_current();
        let layer = document.layers.allocate("r0-layer").expect("layer");
        let track = document.track_ids.allocate("r0-track").expect("track");
        document.tracks.push(Track {
            id: track,
            items: vec![TrackItem::Clip(Clip {
                envelope: ItemEnvelope::new(layer),
                start: RationalTime::ZERO,
                duration: document.composition.duration,
                time_map: TimeMap::identity(),
                source: ClipSource::Plugin {
                    plugin_id: RECT_LAYER_SOURCE.into(),
                    effect_version: 1,
                    params: BTreeMap::from([
                        ("center".into(), DocParam::const_vec2([0.0, 0.0])),
                        ("size".into(), DocParam::const_vec2([1.0, 1.0])),
                        ("color".into(), DocParam::const_color([0.0, 1.0, 0.0, 1.0])),
                    ]),
                    extra: Default::default(),
                },
            })],
        });
        document.validate().expect("valid fixture document");
        let limits = ResourceLimits::production();
        let mut session = ProjectSession::acquire(&path, &limits).expect("acquire fixture");
        session
            .save_with_journal(
                &document,
                &SaveProjectOptions {
                    limits,
                    checkpoint: true,
                    ..SaveProjectOptions::default()
                },
            )
            .expect("save fixture");
        path
    }

    fn pixel_at(bytes: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
        let base = ((y * width + x) * 4) as usize;
        [bytes[base], bytes[base + 1], bytes[base + 2], bytes[base + 3]]
    }

    fn has_non_background_pixel(
        bytes: &[u8],
        width: u32,
        height: u32,
        background: [u8; 4],
    ) -> bool {
        for y in 0..height {
            for x in 0..width {
                if pixel_at(bytes, width, x, y) != background {
                    return true;
                }
            }
        }
        false
    }

    fn create_host(tag: &str) -> u64 {
        let path = fixture_path(tag);
        host_create_for_test(&path).expect("host")
    }

    fn read_snapshot(host: u64) -> RnProductSnapshotForTest {
        host_read_snapshot_for_test(host).expect("snapshot")
    }

    fn dispatch(host: u64, intent: RnHostTestIntent) -> RnHostTestResponse {
        host_dispatch_intent_for_test(host, intent).expect("dispatch")
    }

    fn base_intent(kind: &str) -> RnHostTestIntent {
        RnHostTestIntent {
            kind: kind.to_owned(),
            stage_handle: None,
            projection_generation: None,
            width: None,
            height: None,
            scale_factor: None,
            focused: None,
        }
    }

    fn pointer_intent(
        stage: u64,
        phase: &str,
        view_local_x: f64,
        view_local_y: f64,
        sequence: u64,
    ) -> WireIntentEnvelope {
        WireIntentEnvelope {
            version: WIRE_VERSION,
            direction: RN_TO_HOST.to_owned(),
            kind: "stage_pointer".to_owned(),
            host_handle: String::new(),
            stage_handle: Some(stage.to_string()),
            projection_generation: None,
            width: None,
            height: None,
            scale_factor: None,
            focused: None,
            phase: Some(phase.to_owned()),
            view_local_x: Some(view_local_x),
            view_local_y: Some(view_local_y),
            sequence: Some(sequence),
            frame: None,
            position: None,
            playhead: None,
            target: None,
            key_id: None,
            time: None,
            new: None,
            interp: None,
            delta: None,
        }
    }

    fn set_time_json(host: u64, frame_json: &str) -> String {
        format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"set_time","#,
                r#""host_handle":"{host}","frame":{frame}}}"#
            ),
            host = host,
            frame = frame_json
        )
    }

    fn dispatch_raw_json(host: u64, intent_json: &str) -> RnHostTestResponse {
        #[cfg(target_os = "macos")]
        {
            let mut out = vec![0u8; MAX_JSON_BYTES];
            let written = motolii_rn_host_dispatch_intent_json(
                host,
                intent_json.as_ptr(),
                intent_json.len(),
                out.as_mut_ptr(),
                out.len(),
            );
            assert!(
                written > 0,
                "motolii_rn_host_dispatch_intent_json failed: {written}"
            );
            let response: WireIntentResponse =
                serde_json::from_slice(&out[..written as usize]).expect("response json");
            response_for_test(response)
        }
        #[cfg(not(target_os = "macos"))]
        {
            with_registry(|registry| {
                let out = registry.dispatch_intent_json(host, intent_json)?;
                let response: WireIntentResponse =
                    serde_json::from_str(&out).map_err(RnHostError::from)?;
                Ok(response_for_test(response))
            })
            .expect("dispatch raw json")
        }
    }

    fn fixture_path_with_fps(tag: &str, fps: motolii_core::Fps) -> std::path::PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = tmp_dir(&format!("rn-product-host-{tag}-{id}")).join("project.json");
        let mut document = Document::new_current();
        document.composition.fps = fps;
        let layer = document.layers.allocate("r0-layer").expect("layer");
        let track = document.track_ids.allocate("r0-track").expect("track");
        document.tracks.push(Track {
            id: track,
            items: vec![TrackItem::Clip(Clip {
                envelope: ItemEnvelope::new(layer),
                start: RationalTime::ZERO,
                duration: document.composition.duration,
                time_map: TimeMap::identity(),
                source: ClipSource::Plugin {
                    plugin_id: RECT_LAYER_SOURCE.into(),
                    effect_version: 1,
                    params: BTreeMap::from([
                        ("center".into(), DocParam::const_vec2([0.0, 0.0])),
                        ("size".into(), DocParam::const_vec2([1.0, 1.0])),
                        ("color".into(), DocParam::const_color([0.0, 1.0, 0.0, 1.0])),
                    ]),
                    extra: Default::default(),
                },
            })],
        });
        document.validate().expect("valid fixture document");
        let limits = ResourceLimits::production();
        let mut session = ProjectSession::acquire(&path, &limits).expect("acquire fixture");
        session
            .save_with_journal(
                &document,
                &SaveProjectOptions {
                    limits,
                    checkpoint: true,
                    ..SaveProjectOptions::default()
                },
            )
            .expect("save fixture");
        path
    }

    fn create_host_with_fps(tag: &str, fps: motolii_core::Fps) -> u64 {
        let path = fixture_path_with_fps(tag, fps);
        host_create_for_test(&path).expect("host")
    }

    fn save_document_fixture(tag: &str, document: &Document) -> std::path::PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = tmp_dir(&format!("rn-product-host-{tag}-{id}")).join("project.json");
        let limits = ResourceLimits::production();
        let mut session = ProjectSession::acquire(&path, &limits).expect("acquire fixture");
        session
            .save_with_journal(
                document,
                &SaveProjectOptions {
                    limits,
                    checkpoint: true,
                    ..SaveProjectOptions::default()
                },
            )
            .expect("save fixture");
        path
    }

    fn create_host_from_document(tag: &str, document: &Document) -> u64 {
        let path = save_document_fixture(tag, document);
        host_create_for_test(&path).expect("host")
    }

    struct Fixture {
        document: Document,
    }

    impl Fixture {
        fn new() -> Self {
            Self {
                document: Document::new_current(),
            }
        }

        fn push_rect_layer(
            &mut self,
            name: &str,
            center: [f64; 2],
            size: [f64; 2],
            transform: Transform2D,
        ) -> LayerId {
            if self.document.tracks.is_empty() {
                let track = self.document.track_ids.allocate("V1").expect("track");
                self.document.tracks.push(Track {
                    id: track,
                    items: vec![],
                });
            }
            let layer = self.document.layers.allocate(name).expect("layer");
            let mut envelope = ItemEnvelope::new(layer);
            envelope.transform = transform;
            self.document.tracks[0].items.push(TrackItem::Clip(Clip {
                envelope,
                start: RationalTime::ZERO,
                duration: self.document.composition.duration,
                time_map: TimeMap::identity(),
                source: ClipSource::Plugin {
                    plugin_id: RECT_LAYER_SOURCE.into(),
                    effect_version: 1,
                    params: rect_params(center, size),
                    extra: Default::default(),
                },
            }));
            layer
        }
    }

    fn rect_params(center: [f64; 2], size: [f64; 2]) -> BTreeMap<String, DocParam> {
        BTreeMap::from([
            ("center".into(), DocParam::const_vec2(center)),
            ("size".into(), DocParam::const_vec2(size)),
            ("color".into(), DocParam::const_color([0.0, 1.0, 0.0, 1.0])),
        ])
    }

    fn mount_and_resize(host: u64, stage: u64, width: u32, height: u32) {
        let mut mount = base_intent("stage_mount");
        mount.stage_handle = Some(stage);
        assert!(dispatch(host, mount).accepted);
        let mut resize = base_intent("stage_resize");
        resize.stage_handle = Some(stage);
        resize.width = Some(width);
        resize.height = Some(height);
        resize.scale_factor = Some(1.0);
        assert!(dispatch(host, resize).accepted);
    }

    fn pointer_json(
        host: u64,
        stage: u64,
        phase: &str,
        view_local_x: f64,
        view_local_y: f64,
        sequence: u64,
    ) -> String {
        format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"stage_pointer","#,
                r#""host_handle":"{host}","stage_handle":"{stage}","phase":"{phase}","#,
                r#""view_local_x":{x},"view_local_y":{y},"sequence":{sequence}}}"#
            ),
            host = host,
            stage = stage,
            phase = phase,
            x = view_local_x,
            y = view_local_y,
            sequence = sequence
        )
    }

    fn canonical_to_view_local(
        canonical_x: f64,
        canonical_y: f64,
        width: u32,
        height: u32,
    ) -> (f64, f64) {
        let w = f64::from(width);
        let h = f64::from(height);
        (w * 0.5 + canonical_x * h, h * 0.5 + canonical_y * h)
    }

    fn document_json_bytes(host: u64) -> Vec<u8> {
        with_registry(|registry| {
            let product = registry
                .hosts
                .get(&host)
                .ok_or(RnHostError::UnknownHost(host))?;
            Ok(serde_json::to_vec(product.runtime.snapshot().as_ref()).expect("document json"))
        })
        .expect("document bytes")
    }

    fn dispatch_wire(host: u64, mut intent: WireIntentEnvelope) -> RnHostTestResponse {
        intent.host_handle = host.to_string();
        // JSON 経由だと非有限 f64 を運べないため、受理検証は envelope 直送で行う。
        with_registry(|registry| {
            let product = registry
                .hosts
                .get_mut(&host)
                .ok_or(RnHostError::UnknownHost(host))?;
            Ok(response_for_test(product.dispatch_intent(host, intent)))
        })
        .expect("dispatch wire")
    }

    fn read_stage_pointer(stage: u64) -> Option<StagePointerTransient> {
        with_registry(|registry| {
            for host in registry.hosts.values() {
                if let Some(surface) = host.stages.get(&stage) {
                    return Ok(surface.pointer.clone());
                }
            }
            Err(RnHostError::UnknownStage(stage))
        })
        .ok()
        .flatten()
    }

    fn make_16_layers_64_keys_document() -> Document {
        let mut document = Document::new_current();
        let track = document.track_ids.allocate("stress").expect("track");
        document.tracks.push(Track {
            id: track,
            items: vec![],
        });
        for layer_idx in 0_u64..16 {
            let layer = document
                .layers
                .allocate(&format!("layer-{layer_idx}"))
                .expect("layer");
            let mut keyframes = DocKeyframeTrack::new();
            for key_idx in 0_u64..64 {
                let key_id = document.next_stable_id.allocate().expect("key id");
                keyframes.insert(DocKeyframe {
                    id: KeyframeId::from_raw(key_id),
                    t: RationalTime::try_new(key_idx as i64, 1).expect("key time"),
                    value: DocValue::Vec2([0.0, key_idx as f64]),
                    interp: Interp::Linear,
                });
            }
            let mut envelope = ItemEnvelope::new(layer);
            envelope.transform.position = DocParam::Keyframes(keyframes);
            document.tracks[0].items.push(TrackItem::Clip(Clip {
                envelope,
                start: RationalTime::ZERO,
                duration: document.composition.duration,
                time_map: TimeMap::identity(),
                source: ClipSource::Plugin {
                    plugin_id: RECT_LAYER_SOURCE.into(),
                    effect_version: 1,
                    params: rect_params([0.0, 0.0], [1.0, 1.0]),
                    extra: Default::default(),
                },
            }));
        }
        document.validate().expect("valid");
        document
    }

    #[test]
    fn snapshot_carries_revision_projection_generation_and_primary_layer_id() {
        let _lock = test_lock();
        let host = create_host("snapshot");
        let snapshot = read_snapshot(host);
        assert_eq!(snapshot.revision, "0");
        assert_eq!(snapshot.projection_generation, "0");
        assert_eq!(snapshot.current_time, RationalTime::ZERO);
        assert!(snapshot.primary_layer_id.is_none());
        assert!(!snapshot.layer_ids.is_empty());
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn set_time_accepts_valid_frame_and_advances_projection_generation() {
        let _lock = test_lock();
        let host = create_host("set-time-accept");
        let baseline = read_snapshot(host);
        assert_eq!(baseline.current_time, RationalTime::ZERO);
        assert_eq!(baseline.projection_generation, "0");

        // 既定 Composition は 30fps・duration 10s。frame 45 → 45/30 = 3/2。
        let response = dispatch_raw_json(host, &set_time_json(host, "45"));
        assert!(response.accepted);
        let snap = response.snapshot.expect("snapshot");
        assert_eq!(snap.current_time, RationalTime::try_new(3, 2).expect("3/2"));
        assert_eq!(snap.projection_generation, "1");
        assert_eq!(snap.revision, baseline.revision);
        assert_eq!(snap.primary_layer_id, baseline.primary_layer_id);

        let after = read_snapshot(host);
        assert_eq!(after.current_time, snap.current_time);
        assert_eq!(after.projection_generation, "1");
        assert_eq!(after.revision, baseline.revision);
        assert_eq!(after.primary_layer_id, baseline.primary_layer_id);

        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn set_time_frame_zero_resolves_to_rational_time_zero_via_ffi_json() {
        let _lock = test_lock();
        let host = create_host("set-time-zero");
        // いったん非 ZERO にしてから frame 0 へ戻し、解決結果を観測する。
        assert!(dispatch_raw_json(host, &set_time_json(host, "30")).accepted);
        let response = dispatch_raw_json(host, &set_time_json(host, "0"));
        assert!(response.accepted);
        let snap = response.snapshot.expect("snapshot");
        assert_eq!(snap.current_time, RationalTime::ZERO);
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn set_time_ntsc_frame_is_exact_fraction_via_ffi_json() {
        let _lock = test_lock();
        let fps = motolii_core::Fps::try_new(30_000, 1_001).expect("29.97");
        let host = create_host_with_fps("set-time-ntsc", fps);
        // duration 10s 内に収まる N。N*1001/30000 を十進近似なしで観測する。
        let frame = 100i64;
        let response = dispatch_raw_json(host, &set_time_json(host, &frame.to_string()));
        assert!(response.accepted);
        let snap = response.snapshot.expect("snapshot");
        assert_eq!(
            snap.current_time,
            RationalTime::try_new(frame * 1_001, 30_000).expect("exact ntsc")
        );
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn set_time_film_24_frame_is_exact_fraction_via_ffi_json() {
        let _lock = test_lock();
        let fps = motolii_core::Fps::try_new(24, 1).expect("24");
        let host = create_host_with_fps("set-time-24", fps);
        let frame = 48i64;
        let response = dispatch_raw_json(host, &set_time_json(host, &frame.to_string()));
        assert!(response.accepted);
        let snap = response.snapshot.expect("snapshot");
        assert_eq!(
            snap.current_time,
            RationalTime::try_new(frame, 24).expect("exact 24")
        );
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn set_time_same_frame_is_noop_without_generation_advance() {
        let _lock = test_lock();
        let host = create_host("set-time-noop");
        assert!(dispatch_raw_json(host, &set_time_json(host, "60")).accepted);
        let after_first = read_snapshot(host);
        assert_eq!(after_first.projection_generation, "1");

        let noop = dispatch_raw_json(host, &set_time_json(host, "60"));
        assert!(noop.accepted);
        let snap = noop.snapshot.expect("snapshot");
        assert_eq!(snap.current_time, after_first.current_time);
        assert_eq!(
            snap.projection_generation,
            after_first.projection_generation
        );
        assert_eq!(snap.revision, after_first.revision);
        assert_eq!(snap.primary_layer_id, after_first.primary_layer_id);

        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn set_time_rejects_out_of_bounds_and_bad_wire_without_clamp_or_document_write() {
        let _lock = test_lock();
        let host = create_host("set-time-reject");
        let baseline = read_snapshot(host);

        let negative = dispatch_raw_json(host, &set_time_json(host, "-1"));
        assert!(!negative.accepted);
        assert_eq!(negative.reason, Some(RnHostReasonCode::InvalidIntent));

        // duration 10s / 30fps → frame 300 が境界。301 は超過。
        let over = dispatch_raw_json(host, &set_time_json(host, "301"));
        assert!(!over.accepted);
        assert_eq!(over.reason, Some(RnHostReasonCode::InvalidIntent));

        let missing = dispatch_raw_json(
            host,
            &format!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"set_time","host_handle":"{host}"}}"#
            ),
        );
        assert!(!missing.accepted);
        assert_eq!(missing.reason, Some(RnHostReasonCode::InvalidIntent));

        let non_integer = dispatch_raw_json(host, &set_time_json(host, "1.5"));
        assert!(!non_integer.accepted);
        assert_eq!(non_integer.reason, Some(RnHostReasonCode::InvalidIntent));

        let legacy_time = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"set_time","#,
                    r#""host_handle":"{host}","time":1.5}}"#
                ),
                host = host
            ),
        );
        assert!(!legacy_time.accepted);
        assert_eq!(legacy_time.reason, Some(RnHostReasonCode::InvalidIntent));

        let legacy_time_with_frame = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"set_time","#,
                    r#""host_handle":"{host}","frame":1,"time":1.5}}"#
                ),
                host = host
            ),
        );
        assert!(!legacy_time_with_frame.accepted);
        assert_eq!(
            legacy_time_with_frame.reason,
            Some(RnHostReasonCode::InvalidIntent)
        );

        let after = read_snapshot(host);
        assert_eq!(after.current_time, RationalTime::ZERO);
        assert_eq!(after.revision, baseline.revision);
        assert_eq!(after.projection_generation, baseline.projection_generation);
        assert_eq!(after.primary_layer_id, baseline.primary_layer_id);
        assert_eq!(after.layer_ids, baseline.layer_ids);

        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn set_time_rejects_try_from_frame_overflow_without_panic() {
        let _lock = test_lock();
        // fps=1/2 かつ frame=i64::MAX だと try_from_frame が Overflow になる。
        let fps = motolii_core::Fps::try_new(1, 2).expect("1/2");
        let host = create_host_with_fps("set-time-overflow", fps);
        let baseline = read_snapshot(host);
        let rejected = dispatch_raw_json(host, &set_time_json(host, &i64::MAX.to_string()));
        assert!(!rejected.accepted);
        assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));
        let after = read_snapshot(host);
        assert_eq!(after.current_time, baseline.current_time);
        assert_eq!(after.projection_generation, baseline.projection_generation);
        assert_eq!(after.revision, baseline.revision);
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn set_time_rejects_projection_generation_exhaustion_without_saturation() {
        let _lock = test_lock();
        let host = create_host("set-time-gen-exhaust");
        with_registry(|registry| {
            let product = registry
                .hosts
                .get_mut(&host)
                .ok_or(RnHostError::UnknownHost(host))?;
            product.projection_generation = u64::MAX;
            Ok(())
        })
        .expect("force exhaustion");

        // 同一 ZERO は no-op で受理し、枯渇でも generation を触らない。
        let noop = dispatch_raw_json(host, &set_time_json(host, "0"));
        assert!(noop.accepted);
        assert_eq!(
            noop.snapshot.expect("snapshot").projection_generation,
            u64::MAX.to_string()
        );

        // 異なる frame は前進不能なので typed 拒否。飽和させない。
        let rejected = dispatch_raw_json(host, &set_time_json(host, "1"));
        assert!(!rejected.accepted);
        assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));
        let after = read_snapshot(host);
        assert_eq!(after.current_time, RationalTime::ZERO);
        assert_eq!(after.projection_generation, u64::MAX.to_string());

        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn lifecycle_sequence_preserves_revision_and_projection_generation() {
        let _lock = test_lock();
        let host = create_host("lifecycle");
        let baseline = read_snapshot(host);
        let stage = host_register_stage_for_test(host).expect("stage");

        let mut intent = base_intent("stage_mount");
        intent.stage_handle = Some(stage);
        let mounted = dispatch(host, intent);
        assert!(mounted.accepted);

        let mut resize = base_intent("stage_resize");
        resize.stage_handle = Some(stage);
        resize.width = Some(1280);
        resize.height = Some(720);
        resize.scale_factor = Some(2.0);
        let resized = dispatch(host, resize);
        assert!(resized.accepted);

        let mut focus = base_intent("stage_focus");
        focus.stage_handle = Some(stage);
        focus.focused = Some(true);
        let focused = dispatch(host, focus);
        assert!(focused.accepted);

        let mut unmount = base_intent("stage_unmount");
        unmount.stage_handle = Some(stage);
        let unmounted = dispatch(host, unmount);
        assert!(unmounted.accepted);

        let mut remount = base_intent("stage_mount");
        remount.stage_handle = Some(stage);
        let remounted = dispatch(host, remount);
        assert!(remounted.accepted);

        let after = read_snapshot(host);
        assert_eq!(after.revision, baseline.revision);
        assert_eq!(after.projection_generation, baseline.projection_generation);
        assert_eq!(after.primary_layer_id, baseline.primary_layer_id);
        assert_eq!(after.layer_ids, baseline.layer_ids);

        let _ = host_destroy_stage_for_test(stage);
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn stage_pointer_phases_record_transient_without_document_write() {
        let _lock = test_lock();
        let host = create_host("pointer-accept");
        let baseline = read_snapshot(host);
        let stage = host_register_stage_for_test(host).expect("stage");
        // resize 前は width/height 0 で selection が typed 拒否されるため、先に非正方形へ拡げる。
        mount_and_resize(host, stage, 1600, 900);
        let before_bytes = document_json_bytes(host);

        // 既定 Rect(半幅0.5)の外。down は Miss→clear no-op で generation / Document 不変。
        let phases = [
            ("down", 12.5, 34.0, 1_u64),
            ("drag", 18.0, 40.25, 2),
            ("up", 20.0, 41.0, 3),
            ("cancel", 1.0, 1.0, 4),
        ];
        for (phase, x, y, sequence) in phases {
            let response = dispatch_wire(host, pointer_intent(stage, phase, x, y, sequence));
            assert!(response.accepted, "phase {phase} should accept");
            let recorded = read_stage_pointer(stage).expect("transient pointer");
            assert_eq!(recorded.phase, phase);
            assert_eq!(recorded.view_local_x, x);
            assert_eq!(recorded.view_local_y, y);
            assert_eq!(recorded.sequence, sequence);
            let snap = response.snapshot.expect("snapshot");
            assert_eq!(snap.revision, baseline.revision);
            assert_eq!(snap.projection_generation, baseline.projection_generation);
            assert_eq!(snap.primary_layer_id, baseline.primary_layer_id);
        }

        let after = read_snapshot(host);
        assert_eq!(after.revision, baseline.revision);
        assert_eq!(after.projection_generation, baseline.projection_generation);
        assert_eq!(after.primary_layer_id, baseline.primary_layer_id);
        assert_eq!(after.layer_ids, baseline.layer_ids);
        assert_eq!(document_json_bytes(host), before_bytes);

        let _ = host_destroy_stage_for_test(stage);
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn stage_pointer_rejects_invalid_payload_and_late_events() {
        let _lock = test_lock();
        let host = create_host("pointer-reject");
        let baseline = read_snapshot(host);
        let stage = host_register_stage_for_test(host).expect("stage");

        let mut mount = base_intent("stage_mount");
        mount.stage_handle = Some(stage);
        assert!(dispatch(host, mount).accepted);

        let mut unknown_phase = pointer_intent(stage, "move", 1.0, 2.0, 1);
        unknown_phase.phase = Some("move".to_owned());
        let rejected = dispatch_wire(host, unknown_phase);
        assert!(!rejected.accepted);
        assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));

        let mut non_finite = pointer_intent(stage, "down", f64::NAN, 2.0, 2);
        let rejected = dispatch_wire(host, non_finite);
        assert!(!rejected.accepted);
        assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));

        non_finite = pointer_intent(stage, "down", 1.0, f64::INFINITY, 3);
        let rejected = dispatch_wire(host, non_finite);
        assert!(!rejected.accepted);
        assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));

        let mut missing_sequence = pointer_intent(stage, "down", 1.0, 2.0, 4);
        missing_sequence.sequence = None;
        let rejected = dispatch_wire(host, missing_sequence);
        assert!(!rejected.accepted);
        assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));

        let unknown_stage = dispatch_wire(host, pointer_intent(99_999, "down", 1.0, 2.0, 5));
        assert!(!unknown_stage.accepted);
        assert_eq!(
            unknown_stage.reason,
            Some(RnHostReasonCode::UnknownStageHandle)
        );

        let mut unmount = base_intent("stage_unmount");
        unmount.stage_handle = Some(stage);
        assert!(dispatch(host, unmount).accepted);
        let late = dispatch_wire(host, pointer_intent(stage, "up", 1.0, 2.0, 6));
        assert!(!late.accepted);
        assert_eq!(late.reason, Some(RnHostReasonCode::LateLifecycleEvent));

        let after = read_snapshot(host);
        assert_eq!(after.revision, baseline.revision);
        assert_eq!(after.projection_generation, baseline.projection_generation);
        assert_eq!(after.primary_layer_id, baseline.primary_layer_id);

        let _ = host_destroy_stage_for_test(stage);
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn stale_projection_generation_is_zero_write() {
        let _lock = test_lock();
        let host = create_host("stale");
        let before = read_snapshot(host);
        let mut intent = base_intent("read_snapshot");
        intent.projection_generation = Some("99".to_owned());
        let response = dispatch(host, intent);
        assert!(!response.accepted);
        assert_eq!(
            response.reason,
            Some(RnHostReasonCode::StaleProjectionGeneration)
        );
        let after = read_snapshot(host);
        assert_eq!(after.revision, before.revision);
        assert_eq!(after.projection_generation, before.projection_generation);
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn stage_pointer_down_selects_rotated_rect_via_json_snapshot() {
        let _lock = test_lock();
        let mut fixture = Fixture::new();
        let layer = fixture.push_rect_layer(
            "rotated",
            [0.05, -0.08],
            [0.35, 0.22],
            Transform2D {
                position: DocParam::const_vec2([0.18, 0.12]),
                rotation: DocParam::const_f64(0.55),
                scale: DocParam::const_vec2([1.15, 0.9]),
                ..Transform2D::identity()
            },
        );
        fixture.document.composition.camera = CompCameraDoc::PlanarOrthographic {
            center: DocParam::const_vec2([0.03, -0.02]),
            roll_radians: DocParam::const_f64(0.2),
            height: DocParam::const_f64(1.0),
        };
        fixture.document.validate().expect("valid");
        let host = create_host_from_document("sel-rotated", &fixture.document);
        let stage = host_register_stage_for_test(host).expect("stage");
        mount_and_resize(host, stage, 1600, 900);
        let before_bytes = document_json_bytes(host);
        let before = read_snapshot(host);

        // 局所原点付近を camera∘world で正準へ写し、非対称な view-local へ戻す。
        let tracks = DataTracks::new();
        let proj = project_stage_geometry(
            &fixture.document,
            EvaluationTime::new(RationalTime::ZERO),
            &tracks,
        )
        .expect("geometry");
        let crate::stage_geometry_projection::StageLayerProjection::Available(geo) =
            proj.get(layer).expect("layer")
        else {
            panic!("available");
        };
        let composed = geo.camera_view * geo.world;
        let [cx, cy] = composed.transform_point(geo.local_rect.center.x, geo.local_rect.center.y);
        let (vx, vy) = canonical_to_view_local(cx, cy, 1600, 900);

        let response = dispatch_raw_json(host, &pointer_json(host, stage, "down", vx, vy, 1));
        assert!(response.accepted);
        let snap = response.snapshot.expect("snapshot");
        assert_eq!(snap.primary_layer_id, Some(layer.get().to_string()));
        assert_eq!(snap.projection_generation, "1");
        assert_eq!(snap.revision, before.revision);
        assert_eq!(document_json_bytes(host), before_bytes);

        let _ = host_destroy_stage_for_test(stage);
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn stage_pointer_down_x_uses_height_denominator_on_portrait_stage() {
        let _lock = test_lock();
        // h>w でないと /height hit かつ /width miss が作れない。
        let mut fixture = Fixture::new();
        let layer = fixture.push_rect_layer(
            "x-height",
            [0.0, 0.0],
            [0.7, 0.7],
            Transform2D {
                // 非 identity（平行移動）。逆写像を無視すると中心がずれる。
                position: DocParam::const_vec2([0.0, 0.05]),
                ..Transform2D::identity()
            },
        );
        fixture.document.validate().expect("valid");
        let host = create_host_from_document("sel-x-height", &fixture.document);
        let stage = host_register_stage_for_test(host).expect("stage");
        let (width, height) = (900_u32, 1600_u32);
        mount_and_resize(host, stage, width, height);

        // local x=0.25 → 正準 x=0.25。/height は半幅0.35内、/width なら ≈0.444 で外れ。
        let (vx, vy) = canonical_to_view_local(0.25, 0.05, width, height);
        assert!(vx >= 0.0 && vx <= f64::from(width));
        let wrong_x = (vx - f64::from(width) * 0.5) / f64::from(width);
        assert!(
            wrong_x.abs() > 0.35,
            "oracle requires /width miss: wrong_x={wrong_x}"
        );

        let response = dispatch_raw_json(host, &pointer_json(host, stage, "down", vx, vy, 1));
        assert!(response.accepted);
        assert_eq!(
            response.snapshot.expect("snapshot").primary_layer_id,
            Some(layer.get().to_string())
        );

        let _ = host_destroy_stage_for_test(stage);
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn stage_pointer_down_scale_factor_oracle_preserves_logical_hit_target() {
        let _lock = test_lock();
        let mut fixture = Fixture::new();
        let layer =
            fixture.push_rect_layer("base", [0.0, 0.0], [0.5, 0.5], Transform2D::identity());
        fixture.document.validate().expect("valid");
        let host = create_host_from_document("sel-scale-factor", &fixture.document);
        let stage = host_register_stage_for_test(host).expect("stage");

        let mut mount = base_intent("stage_mount");
        mount.stage_handle = Some(stage);
        assert!(dispatch(host, mount).accepted);

        let mut resize = base_intent("stage_resize");
        resize.stage_handle = Some(stage);
        resize.width = Some(1600);
        resize.height = Some(900);
        resize.scale_factor = Some(1.0);
        assert!(dispatch(host, resize).accepted);

        let tracks = DataTracks::new();
        let proj = project_stage_geometry(
            &fixture.document,
            EvaluationTime::new(RationalTime::ZERO),
            &tracks,
        )
        .expect("geometry");
        let crate::stage_geometry_projection::StageLayerProjection::Available(geo) =
            proj.get(layer).expect("layer")
        else {
            panic!("available");
        };
        let [cx, cy] = (geo.camera_view * geo.world)
            .transform_point(geo.local_rect.center.x, geo.local_rect.center.y);
        let (vx, vy) = canonical_to_view_local(cx, cy, 1600, 900);

        let selected_once = dispatch_raw_json(host, &pointer_json(host, stage, "down", vx, vy, 1));
        assert!(selected_once.accepted);
        let primary_once = selected_once
            .snapshot
            .expect("snapshot")
            .primary_layer_id
            .expect("primary");
        assert_eq!(primary_once, layer.get().to_string());

        let mut resize_scaled = base_intent("stage_resize");
        resize_scaled.stage_handle = Some(stage);
        resize_scaled.width = Some(1600);
        resize_scaled.height = Some(900);
        resize_scaled.scale_factor = Some(2.0);
        assert!(dispatch(host, resize_scaled).accepted);

        let selected_twice = dispatch_raw_json(host, &pointer_json(host, stage, "down", vx, vy, 2));
        assert_eq!(
            selected_twice
                .snapshot
                .expect("snapshot")
                .primary_layer_id
                .expect("primary"),
            layer.get().to_string()
        );

        let _ = host_destroy_stage_for_test(stage);
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn stage_pointer_down_y_up_hits_upper_half_rect() {
        let _lock = test_lock();
        let mut fixture = Fixture::new();
        let layer =
            fixture.push_rect_layer("upper", [0.0, 0.25], [0.4, 0.3], Transform2D::identity());
        fixture.document.validate().expect("valid");
        let host = create_host_from_document("sel-y-up", &fixture.document);
        let stage = host_register_stage_for_test(host).expect("stage");
        mount_and_resize(host, stage, 1600, 900);
        // view-local の大きい y（上方向）。Y 反転なら負の正準 y になり外れる。
        let (vx, vy) = canonical_to_view_local(0.0, 0.25, 1600, 900);
        assert!(vy > 450.0);

        let response = dispatch_raw_json(host, &pointer_json(host, stage, "down", vx, vy, 1));
        assert!(response.accepted);
        assert_eq!(
            response.snapshot.expect("snapshot").primary_layer_id,
            Some(layer.get().to_string())
        );

        let _ = host_destroy_stage_for_test(stage);
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn stage_pointer_down_clear_requires_prior_primary() {
        let _lock = test_lock();
        let mut fixture = Fixture::new();
        let layer = fixture.push_rect_layer(
            "target",
            [0.0, 0.0],
            [0.4, 0.4],
            Transform2D {
                position: DocParam::const_vec2([-0.2, 0.15]),
                rotation: DocParam::const_f64(-0.3),
                ..Transform2D::identity()
            },
        );
        fixture.document.validate().expect("valid");
        let host = create_host_from_document("sel-clear", &fixture.document);
        let stage = host_register_stage_for_test(host).expect("stage");
        mount_and_resize(host, stage, 1600, 900);
        let before_bytes = document_json_bytes(host);

        let tracks = DataTracks::new();
        let proj = project_stage_geometry(
            &fixture.document,
            EvaluationTime::new(RationalTime::ZERO),
            &tracks,
        )
        .expect("geometry");
        let crate::stage_geometry_projection::StageLayerProjection::Available(geo) =
            proj.get(layer).expect("layer")
        else {
            panic!("available");
        };
        let [cx, cy] = (geo.camera_view * geo.world)
            .transform_point(geo.local_rect.center.x, geo.local_rect.center.y);
        let (vx, vy) = canonical_to_view_local(cx, cy, 1600, 900);
        let selected = dispatch_raw_json(host, &pointer_json(host, stage, "down", vx, vy, 1));
        assert_eq!(
            selected.snapshot.expect("snapshot").primary_layer_id,
            Some(layer.get().to_string())
        );

        let cleared = dispatch_raw_json(host, &pointer_json(host, stage, "down", 10.0, 10.0, 2));
        assert!(cleared.accepted);
        let snap = cleared.snapshot.expect("snapshot");
        assert!(snap.primary_layer_id.is_none());
        assert_eq!(snap.projection_generation, "2");
        assert_eq!(document_json_bytes(host), before_bytes);

        let _ = host_destroy_stage_for_test(stage);
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn stage_pointer_down_overlap_prefers_later_projection_layer() {
        let _lock = test_lock();
        let mut fixture = Fixture::new();
        let back = fixture.push_rect_layer("back", [0.0, 0.0], [0.8, 0.8], Transform2D::identity());
        let front = fixture.push_rect_layer(
            "front",
            [0.0, 0.0],
            [0.5, 0.5],
            Transform2D {
                position: DocParam::const_vec2([0.05, -0.04]),
                ..Transform2D::identity()
            },
        );
        fixture.document.validate().expect("valid");
        let host = create_host_from_document("sel-overlap", &fixture.document);
        let stage = host_register_stage_for_test(host).expect("stage");
        mount_and_resize(host, stage, 1600, 900);
        let (vx, vy) = canonical_to_view_local(0.05, -0.04, 1600, 900);
        let response = dispatch_raw_json(host, &pointer_json(host, stage, "down", vx, vy, 1));
        assert_eq!(
            response.snapshot.expect("snapshot").primary_layer_id,
            Some(front.get().to_string())
        );
        assert_ne!(front, back);

        let _ = host_destroy_stage_for_test(stage);
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn stage_pointer_same_id_down_is_noop_for_generation() {
        let _lock = test_lock();
        let mut fixture = Fixture::new();
        let layer = fixture.push_rect_layer(
            "same",
            [0.0, 0.0],
            [0.5, 0.5],
            Transform2D {
                position: DocParam::const_vec2([0.2, -0.1]),
                rotation: DocParam::const_f64(0.4),
                ..Transform2D::identity()
            },
        );
        fixture.document.validate().expect("valid");
        let host = create_host_from_document("sel-same-id", &fixture.document);
        let stage = host_register_stage_for_test(host).expect("stage");
        mount_and_resize(host, stage, 1600, 900);
        let tracks = DataTracks::new();
        let proj = project_stage_geometry(
            &fixture.document,
            EvaluationTime::new(RationalTime::ZERO),
            &tracks,
        )
        .expect("geometry");
        let crate::stage_geometry_projection::StageLayerProjection::Available(geo) =
            proj.get(layer).expect("layer")
        else {
            panic!("available");
        };
        let [cx, cy] = (geo.camera_view * geo.world)
            .transform_point(geo.local_rect.center.x, geo.local_rect.center.y);
        let (vx, vy) = canonical_to_view_local(cx, cy, 1600, 900);

        let first = dispatch_raw_json(host, &pointer_json(host, stage, "down", vx, vy, 1));
        assert_eq!(first.snapshot.expect("snapshot").projection_generation, "1");
        let second = dispatch_raw_json(host, &pointer_json(host, stage, "down", vx, vy, 2));
        let snap = second.snapshot.expect("snapshot");
        assert_eq!(snap.projection_generation, "1");
        assert_eq!(snap.primary_layer_id, Some(layer.get().to_string()));

        let _ = host_destroy_stage_for_test(stage);
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn stage_pointer_drag_up_cancel_keep_prior_primary() {
        let _lock = test_lock();
        let mut fixture = Fixture::new();
        let layer = fixture.push_rect_layer(
            "keep",
            [0.0, 0.0],
            [0.4, 0.4],
            Transform2D {
                position: DocParam::const_vec2([0.1, 0.1]),
                ..Transform2D::identity()
            },
        );
        fixture.document.validate().expect("valid");
        let host = create_host_from_document("sel-phase-keep", &fixture.document);
        let stage = host_register_stage_for_test(host).expect("stage");
        mount_and_resize(host, stage, 1600, 900);
        let (vx, vy) = canonical_to_view_local(0.1, 0.1, 1600, 900);
        let selected = dispatch_raw_json(host, &pointer_json(host, stage, "down", vx, vy, 1));
        assert_eq!(
            selected.snapshot.expect("snapshot").primary_layer_id,
            Some(layer.get().to_string())
        );
        let gen = read_snapshot(host).projection_generation;

        for (phase, seq) in [("drag", 2_u64), ("up", 3), ("cancel", 4)] {
            // 空領域へ送っても selection は変えない。
            let response =
                dispatch_raw_json(host, &pointer_json(host, stage, phase, 10.0, 10.0, seq));
            assert!(response.accepted, "{phase}");
            let snap = response.snapshot.expect("snapshot");
            assert_eq!(snap.primary_layer_id, Some(layer.get().to_string()));
            assert_eq!(snap.projection_generation, gen);
        }

        let _ = host_destroy_stage_for_test(stage);
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn stage_pointer_down_zero_extent_rejects_without_changing_primary() {
        let _lock = test_lock();
        let mut fixture = Fixture::new();
        let layer = fixture.push_rect_layer("z", [0.0, 0.0], [0.4, 0.4], Transform2D::identity());
        fixture.document.validate().expect("valid");
        let host = create_host_from_document("sel-zero", &fixture.document);
        let stage = host_register_stage_for_test(host).expect("stage");
        // いったん選択してから width=0 相当（resize 無し）で down する。
        mount_and_resize(host, stage, 1600, 900);
        let (vx, vy) = canonical_to_view_local(0.0, 0.0, 1600, 900);
        assert!(dispatch_raw_json(host, &pointer_json(host, stage, "down", vx, vy, 1)).accepted);
        assert_eq!(
            read_snapshot(host).primary_layer_id,
            Some(layer.get().to_string())
        );

        // resize で正のサイズは必須なので、内部 state を 0 にして zero-extent を再現する。
        with_registry(|registry| {
            let product = registry
                .hosts
                .get_mut(&host)
                .ok_or(RnHostError::UnknownHost(host))?;
            let surface = product
                .stages
                .get_mut(&stage)
                .ok_or(RnHostError::UnknownStage(stage))?;
            surface.width = 0;
            surface.height = 900;
            Ok(())
        })
        .expect("zero width");

        let before = read_snapshot(host);
        let rejected = dispatch_wire(host, pointer_intent(stage, "down", 100.0, 100.0, 2));
        assert!(!rejected.accepted);
        assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));
        let recorded = read_stage_pointer(stage).expect("pointer recorded");
        assert_eq!(recorded.phase, "down");
        assert_eq!(recorded.sequence, 2);
        let after = read_snapshot(host);
        assert_eq!(after.primary_layer_id, before.primary_layer_id);
        assert_eq!(after.projection_generation, before.projection_generation);

        let _ = host_destroy_stage_for_test(stage);
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn stage_pointer_down_on_singular_layer_clears_primary() {
        let _lock = test_lock();
        let mut fixture = Fixture::new();
        let layer = fixture.push_rect_layer(
            "singular",
            [0.0, 0.0],
            [0.5, 0.5],
            Transform2D {
                scale: DocParam::const_vec2([0.0, 1.0]),
                ..Transform2D::identity()
            },
        );
        fixture.document.validate().expect("valid");
        let host = create_host_from_document("sel-geom-err", &fixture.document);
        // 幾何は壊れていても ReplacePrimary は envelope 存在だけで受理できる。
        // layer 単位の特異は projection 全体を落とさず Unavailable になるため、
        // hit は Miss へ落ちて primary が clear される。
        with_registry(|registry| {
            let product = registry
                .hosts
                .get_mut(&host)
                .ok_or(RnHostError::UnknownHost(host))?;
            let mut queue = DocumentEditQueue::default();
            queue.push_replace_primary(layer);
            let published = product
                .runtime
                .process_next(&mut queue, product.primary, product.projection_generation)
                .expect("process")
                .expect("published");
            product.primary = published.primary;
            product.projection_generation = published.projection_generation;
            Ok(())
        })
        .expect("seed primary");
        let stage = host_register_stage_for_test(host).expect("stage");
        mount_and_resize(host, stage, 1600, 900);
        let before = read_snapshot(host);
        assert_eq!(before.primary_layer_id, Some(layer.get().to_string()));

        let selected = dispatch_raw_json(host, &pointer_json(host, stage, "down", 800.0, 450.0, 1));
        assert!(selected.accepted);
        assert_eq!(selected.snapshot.expect("snapshot").primary_layer_id, None);
        let after = read_snapshot(host);
        assert_eq!(after.primary_layer_id, None);
        assert_eq!(read_stage_pointer(stage).expect("pointer").sequence, 1);

        let _ = host_destroy_stage_for_test(stage);
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn stage_pointer_down_miss_clears_primary() {
        let _lock = test_lock();
        let mut fixture = Fixture::new();
        let layer =
            fixture.push_rect_layer("healthy", [0.0, 0.0], [0.2, 0.2], Transform2D::identity());
        fixture.document.validate().expect("valid");
        let host = create_host_from_document("sel-miss-clear", &fixture.document);
        with_registry(|registry| {
            let product = registry
                .hosts
                .get_mut(&host)
                .ok_or(RnHostError::UnknownHost(host))?;
            let mut queue = DocumentEditQueue::default();
            queue.push_replace_primary(layer);
            let published = product
                .runtime
                .process_next(&mut queue, product.primary, product.projection_generation)
                .expect("process")
                .expect("published");
            product.primary = published.primary;
            product.projection_generation = published.projection_generation;
            Ok(())
        })
        .expect("seed primary");
        let stage = host_register_stage_for_test(host).expect("stage");
        mount_and_resize(host, stage, 1600, 900);
        let before = read_snapshot(host);
        assert_eq!(before.primary_layer_id, Some(layer.get().to_string()));

        // 健全な rect の外側を押す。空き領域の click は選択解除である。
        let missed = dispatch_raw_json(host, &pointer_json(host, stage, "down", 20.0, 20.0, 1));
        assert!(missed.accepted);
        assert_eq!(missed.snapshot.expect("snapshot").primary_layer_id, None);
        assert_eq!(read_snapshot(host).primary_layer_id, None);

        let _ = host_destroy_stage_for_test(stage);
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn stage_pointer_down_skips_degenerate_and_unavailable_layers() {
        let _lock = test_lock();
        let mut fixture = Fixture::new();
        let _degenerate =
            fixture.push_rect_layer("degen", [0.0, 0.0], [0.0, 0.5], Transform2D::identity());
        let group_id = fixture.document.layers.allocate("g").expect("group");
        fixture.document.tracks[0]
            .items
            .push(TrackItem::Group(motolii_doc::Group {
                envelope: ItemEnvelope::new(group_id),
                children: vec![],
            }));
        fixture.document.validate().expect("valid");
        let host = create_host_from_document("sel-skip", &fixture.document);
        let stage = host_register_stage_for_test(host).expect("stage");
        mount_and_resize(host, stage, 1600, 900);
        // 事前 primary を group 以外の存在 layer で持てないので、先に Replace で degenerate を primary にしない。
        // Miss（退化・Unavailable 除外）→ clear no-op（primary None のまま）。
        let response = dispatch_raw_json(host, &pointer_json(host, stage, "down", 800.0, 450.0, 1));
        assert!(response.accepted);
        assert!(response
            .snapshot
            .expect("snapshot")
            .primary_layer_id
            .is_none());

        let _ = host_destroy_stage_for_test(stage);
        let _ = host_destroy_for_test(host);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn timeline_registration_borrows_revisioned_document_projection() {
        let _lock = test_lock();
        let mut fixture = Fixture::new();
        let layer = fixture.push_rect_layer(
            "timeline-seat",
            [0.0, 0.0],
            [0.25, 0.25],
            Transform2D::identity(),
        );
        fixture.document.validate().expect("valid");
        let host = create_host_from_document("timeline-seat", &fixture.document);
        let timeline =
            with_registry(|registry| registry.register_timeline(host)).expect("timeline");

        let frame = with_registry(|registry| {
            let product = registry
                .hosts
                .get(&host)
                .ok_or(RnHostError::UnknownHost(host))?;
            product
                .timeline_frame_borrow()
                .map_err(|_| RnHostError::UnknownTimeline(timeline))
        })
        .expect("frame borrow");
        assert_eq!(frame.revision, 0);
        assert_eq!(frame.projection_generation, 0);
        assert_eq!(
            frame.document.layers.display_name(layer),
            Some("timeline-seat")
        );
        assert_eq!(frame.projection.bars().len(), 1);
        assert_eq!(frame.projection.bars()[0].layer, layer);
        assert_eq!(frame.primary, None);
        assert_eq!(frame.playhead, RationalTime::ZERO);

        with_registry(|registry| registry.destroy_timeline(timeline)).expect("destroy timeline");
        let double = with_registry(|registry| registry.destroy_timeline(timeline)).unwrap_err();
        assert!(matches!(double, RnHostError::DestroyedTimeline(_)));
        host_destroy_for_test(host).expect("destroy host");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn timeline_detach_reuses_surface_binding_lifecycle() {
        let mut timeline = RnTimelineSurface {
            host_handle: 1,
            destroyed: false,
            gpu: StageGpuBinding::detached(),
            raster_key: None,
        };
        timeline.gpu.layer_ptr = 7;
        timeline.gpu.physical_width = 640;
        timeline.gpu.physical_height = 240;
        timeline.gpu_detach_surface();
        assert!(!timeline.gpu.is_attached());
        assert_eq!(timeline.gpu.physical_width, 0);
        assert_eq!(timeline.gpu.physical_height, 0);
        assert_eq!(timeline.gpu.surface_epoch, 1);
    }

    #[test]
    fn unknown_and_destroyed_handles_are_rejected_safely() {
        let _lock = test_lock();
        let host = create_host("handles");
        let err = host_read_snapshot_for_test(9_999).unwrap_err();
        assert!(matches!(err, RnHostError::UnknownHost(9_999)));

        let stage = host_register_stage_for_test(host).expect("stage");
        host_destroy_stage_for_test(stage).expect("destroy");
        let err = host_destroy_stage_for_test(stage).unwrap_err();
        assert!(matches!(err, RnHostError::DestroyedStage(_)));

        host_destroy_for_test(host).expect("destroy host");
        let err = host_destroy_for_test(host).unwrap_err();
        assert!(matches!(err, RnHostError::DestroyedHost(_)));

        let late = base_intent("stage_mount");
        assert!(matches!(
            host_dispatch_intent_for_test(host, late),
            Err(RnHostError::DestroyedHost(_))
        ));
    }

    #[test]
    fn late_lifecycle_event_after_stage_destroy_is_rejected() {
        let _lock = test_lock();
        let host = create_host("late");
        let stage = host_register_stage_for_test(host).expect("stage");
        host_destroy_stage_for_test(stage).expect("destroy");

        let mut intent = base_intent("stage_resize");
        intent.stage_handle = Some(stage);
        intent.width = Some(640);
        intent.height = Some(480);
        let response = dispatch(host, intent);
        assert!(!response.accepted);
        assert_eq!(response.reason, Some(RnHostReasonCode::LateLifecycleEvent));
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn second_host_and_invalid_path_are_rejected_without_replacing_active_host() {
        let _lock = test_lock();
        let host = create_host("single");
        let second_path = fixture_path("second");
        assert!(matches!(
            host_create_for_test(&second_path),
            Err(RnHostError::HostAlreadyExists)
        ));

        let missing_path = tmp_dir("rn-product-host-missing").join("missing.json");
        assert!(matches!(
            host_create_for_test(&missing_path),
            Err(RnHostError::HostAlreadyExists)
        ));
        assert!(host_read_snapshot_for_test(host).is_ok());
        host_destroy_for_test(host).expect("destroy host");
    }

    #[cfg(target_os = "macos")]
    fn parse_wire_response(buf: &[u8], len: i64) -> WireIntentResponse {
        assert!(len > 0);
        let json = std::str::from_utf8(&buf[..len as usize]).expect("utf8");
        serde_json::from_str(json).expect("wire response")
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn ffi_create_register_read_destroy_emit_typed_envelopes() {
        let _lock = test_lock();
        let path = fixture_path("ffi-create");
        let path_bytes = path.to_string_lossy();
        let mut host_handle = 0u64;
        let mut out = [0u8; MAX_JSON_BYTES];
        let created = motolii_rn_host_create(
            path_bytes.as_bytes().as_ptr(),
            path_bytes.len(),
            &mut host_handle,
            out.as_mut_ptr(),
            out.len(),
        );
        assert!(created > 0);
        assert_ne!(host_handle, 0);
        let created_response = parse_wire_response(&out, created);
        assert!(created_response.accepted);
        let snapshot = created_response.snapshot.expect("create snapshot");
        assert_eq!(snapshot.host_handle, host_handle.to_string());
        assert_eq!(snapshot.revision, "0");
        assert_eq!(snapshot.projection_generation, "0");

        let mut stage_handle = 0u64;
        let registered =
            motolii_rn_stage_register(host_handle, &mut stage_handle, out.as_mut_ptr(), out.len());
        assert!(registered > 0);
        assert_ne!(stage_handle, 0);
        let registered_response = parse_wire_response(&out, registered);
        assert!(registered_response.accepted);
        assert_eq!(
            registered_response
                .snapshot
                .as_ref()
                .map(|snapshot| snapshot.revision.as_str()),
            Some("0")
        );

        let read = motolii_rn_host_read_snapshot_json(host_handle, out.as_mut_ptr(), out.len());
        assert!(read > 0);
        let read_snapshot: WireProductSnapshot =
            serde_json::from_slice(&out[..read as usize]).expect("read snapshot");
        assert_eq!(read_snapshot.revision, snapshot.revision);
        assert_eq!(
            read_snapshot.projection_generation,
            snapshot.projection_generation
        );

        let destroyed_stage = motolii_rn_stage_destroy(stage_handle, out.as_mut_ptr(), out.len());
        assert!(destroyed_stage > 0);
        let stage_destroy_response = parse_wire_response(&out, destroyed_stage);
        assert!(stage_destroy_response.accepted);
        assert!(stage_destroy_response.snapshot.is_none());

        let destroyed_host = motolii_rn_host_destroy(host_handle, out.as_mut_ptr(), out.len());
        assert!(destroyed_host > 0);
        let host_destroy_response = parse_wire_response(&out, destroyed_host);
        assert!(host_destroy_response.accepted);
        assert!(host_destroy_response.snapshot.is_none());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn ffi_rejects_preserve_typed_reasons_and_skip_registry_mutation_on_bad_out() {
        let _lock = test_lock();
        let path = fixture_path("ffi-reject");
        let path_bytes = path.to_string_lossy();
        let mut host_handle = 0u64;
        let mut out = [0u8; MAX_JSON_BYTES];

        let missing = tmp_dir("rn-product-host-ffi-missing").join("missing.json");
        let missing_bytes = missing.to_string_lossy();
        let mut missing_handle = 1u64;
        let missing_result = motolii_rn_host_create(
            missing_bytes.as_bytes().as_ptr(),
            missing_bytes.len(),
            &mut missing_handle,
            out.as_mut_ptr(),
            out.len(),
        );
        assert!(missing_result > 0);
        assert_eq!(missing_handle, 0);
        let missing_response = parse_wire_response(&out, missing_result);
        assert!(!missing_response.accepted);
        assert_eq!(
            missing_response.diagnostics[0].reason,
            RnHostReasonCode::InvalidProjectPath
        );

        let created = motolii_rn_host_create(
            path_bytes.as_bytes().as_ptr(),
            path_bytes.len(),
            &mut host_handle,
            out.as_mut_ptr(),
            out.len(),
        );
        assert!(created > 0);
        assert_ne!(host_handle, 0);

        let mut second_handle = 1u64;
        let second = motolii_rn_host_create(
            path_bytes.as_bytes().as_ptr(),
            path_bytes.len(),
            &mut second_handle,
            out.as_mut_ptr(),
            out.len(),
        );
        assert!(second > 0);
        assert_eq!(second_handle, 0);
        let second_response = parse_wire_response(&out, second);
        assert!(!second_response.accepted);
        assert_eq!(
            second_response.diagnostics[0].reason,
            RnHostReasonCode::HostAlreadyExists
        );
        assert!(motolii_rn_host_read_snapshot_json(host_handle, out.as_mut_ptr(), out.len()) > 0);

        let unknown_read = motolii_rn_host_read_snapshot_json(9_999, out.as_mut_ptr(), out.len());
        assert!(unknown_read > 0);
        let unknown_response = parse_wire_response(&out, unknown_read);
        assert!(!unknown_response.accepted);
        assert_eq!(
            unknown_response.diagnostics[0].reason,
            RnHostReasonCode::UnknownHostHandle
        );

        let mut stage_handle = 0u64;
        let registered =
            motolii_rn_stage_register(host_handle, &mut stage_handle, out.as_mut_ptr(), out.len());
        assert!(registered > 0);
        assert_ne!(stage_handle, 0);
        assert!(motolii_rn_stage_destroy(stage_handle, out.as_mut_ptr(), out.len()) > 0);
        let double_stage = motolii_rn_stage_destroy(stage_handle, out.as_mut_ptr(), out.len());
        assert!(double_stage > 0);
        let double_stage_response = parse_wire_response(&out, double_stage);
        assert!(!double_stage_response.accepted);
        assert_eq!(
            double_stage_response.diagnostics[0].reason,
            RnHostReasonCode::DoubleDestroy
        );

        let unknown_stage = motolii_rn_stage_destroy(42_042, out.as_mut_ptr(), out.len());
        assert!(unknown_stage > 0);
        let unknown_stage_response = parse_wire_response(&out, unknown_stage);
        assert!(!unknown_stage_response.accepted);
        assert_eq!(
            unknown_stage_response.diagnostics[0].reason,
            RnHostReasonCode::UnknownStageHandle
        );

        let null_create = motolii_rn_host_create(
            path_bytes.as_bytes().as_ptr(),
            path_bytes.len(),
            std::ptr::null_mut(),
            out.as_mut_ptr(),
            out.len(),
        );
        assert_eq!(null_create, -1);

        let undersized = motolii_rn_host_create(
            path_bytes.as_bytes().as_ptr(),
            path_bytes.len(),
            &mut second_handle,
            out.as_mut_ptr(),
            1,
        );
        assert!(undersized < 0);
        assert_eq!(second_handle, 0);
        assert!(motolii_rn_host_read_snapshot_json(host_handle, out.as_mut_ptr(), out.len()) > 0);

        assert!(motolii_rn_host_destroy(host_handle, out.as_mut_ptr(), out.len()) > 0);
        let destroyed_read =
            motolii_rn_host_read_snapshot_json(host_handle, out.as_mut_ptr(), out.len());
        assert!(destroyed_read > 0);
        let destroyed_response = parse_wire_response(&out, destroyed_read);
        assert!(!destroyed_response.accepted);
        assert_eq!(
            destroyed_response.diagnostics[0].reason,
            RnHostReasonCode::DestroyedHostHandle
        );
        let double_host = motolii_rn_host_destroy(host_handle, out.as_mut_ptr(), out.len());
        assert!(double_host > 0);
        let double_host_response = parse_wire_response(&out, double_host);
        assert!(!double_host_response.accepted);
        assert_eq!(
            double_host_response.diagnostics[0].reason,
            RnHostReasonCode::DoubleDestroy
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn stage_gpu_binding_starts_detached_with_zero_epoch() {
        let binding = StageGpuBinding::detached();
        assert_eq!(binding.surface_epoch, 0);
        assert!(!binding.is_attached());
        assert_eq!(binding.physical_width, 0);
        assert_eq!(binding.physical_height, 0);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn stage_gpu_detach_increments_epoch_and_clears_binding_markers() {
        let mut stage = RnStageSurface {
            host_handle: 1,
            mounted: true,
            destroyed: false,
            width: 100,
            height: 50,
            scale_factor: 2.0,
            focused: false,
            pointer: None,
            gpu: StageGpuBinding {
                surface_epoch: 2,
                last_presented_epoch: Some(2),
                physical_width: 200,
                physical_height: 100,
                layer_ptr: 0xdead_beef,
                surface: None,
                needs_reconfigure: false,
                poisoned: false,
                overlay: None,
                overlay_upload_key: None,
            },
        };
        assert!(stage.gpu.is_attached());
        stage.gpu_detach_surface();
        assert!(!stage.gpu.is_attached());
        assert_eq!(stage.gpu.surface_epoch, 3);
        assert_eq!(stage.gpu.physical_width, 0);
        assert_eq!(stage.gpu.physical_height, 0);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn stage_gpu_resize_unknown_stage_is_rejected_without_registry_mutation() {
        let _lock = test_lock();
        let host = create_host("gpu-unknown-stage");
        let before = read_snapshot(host);
        let outcome = run_stage_gpu_op(host, 99_999, |_, _| Ok(()));
        assert_eq!(outcome, Err(RnHostReasonCode::UnknownStageHandle));
        let after = read_snapshot(host);
        assert_eq!(after.revision, before.revision);
        assert_eq!(after.projection_generation, before.projection_generation);
        let _ = host_destroy_for_test(host);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn stage_gpu_attach_validation_rejects_null_layer_without_state_change() {
        let binding = StageGpuBinding::detached();
        assert_eq!(
            binding.validate_attach(0),
            Err(RnHostReasonCode::InvalidIntent)
        );
        assert_eq!(binding.surface_epoch, 0);
        assert!(!binding.is_attached());
        assert!(!binding.has_surface());
        assert!(!binding.needs_reconfigure);
        assert!(!binding.poisoned);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn stage_unmount_detaches_gpu_binding_markers_without_real_layer() {
        let _lock = test_lock();
        let host = create_host("gpu-unmount-detach");
        let stage = host_register_stage_for_test(host).expect("stage");
        stage_gpu_mark_attached_for_test(stage, 0xfeed_face).expect("mark attached");
        let (epoch, attached, _, _) = stage_gpu_state_for_test(stage).expect("state");
        assert!(attached);
        assert_eq!(epoch, 1);

        let mut mount = base_intent("stage_mount");
        mount.stage_handle = Some(stage);
        assert!(dispatch(host, mount).accepted);

        let mut unmount = base_intent("stage_unmount");
        unmount.stage_handle = Some(stage);
        assert!(dispatch(host, unmount).accepted);

        let (epoch_after, attached_after, width, height) =
            stage_gpu_state_for_test(stage).expect("state");
        assert!(!attached_after);
        assert_eq!(epoch_after, epoch + 1);
        assert_eq!(width, 0);
        assert_eq!(height, 0);

        let _ = host_destroy_stage_for_test(stage);
        let _ = host_destroy_for_test(host);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn stage_gpu_duplicate_attach_is_rejected_without_replacing_binding() {
        let mut binding = StageGpuBinding::detached();
        binding.layer_ptr = 0xfeed_face;
        binding.surface_epoch = 4;
        assert_eq!(
            binding.validate_attach(0xdead_beef),
            Err(RnHostReasonCode::InvalidIntent)
        );
        assert_eq!(binding.layer_ptr, 0xfeed_face);
        assert_eq!(binding.surface_epoch, 4);
        assert!(!binding.has_surface());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn stage_gpu_surface_state_transitions_are_epoch_bounded() {
        let mut binding = StageGpuBinding::detached();
        binding.layer_ptr = 1;
        binding.needs_reconfigure = true;

        binding.configured(640, 360);
        assert_eq!(binding.surface_epoch, 1);
        assert!(!binding.needs_reconfigure);
        assert_eq!(binding.last_presented_epoch, None);

        binding.presented(false);
        assert_eq!(binding.last_presented_epoch, Some(1));
        assert!(!binding.needs_reconfigure);

        binding.acquisition_deferred();
        assert_eq!(binding.surface_epoch, 1);
        assert_eq!(binding.last_presented_epoch, Some(1));
        assert!(!binding.needs_reconfigure);

        binding.presented(true);
        assert_eq!(binding.last_presented_epoch, Some(1));
        assert!(binding.needs_reconfigure);
        binding.configured(640, 360);
        assert_eq!(binding.surface_epoch, 2);
        assert!(!binding.needs_reconfigure);

        binding.outdated();
        assert!(binding.needs_reconfigure);
        binding.configured(640, 360);
        assert_eq!(binding.surface_epoch, 3);
        assert!(!binding.needs_reconfigure);

        binding.lost();
        assert_eq!(binding.surface_epoch, 4);
        assert!(!binding.is_attached());
        assert!(!binding.has_surface());
        assert!(!binding.needs_reconfigure);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn stage_gpu_size_change_invalidates_overlay_upload() {
        let mut binding = StageGpuBinding::detached();
        binding.configured(640, 360);
        let key = OverlayUploadKey {
            selected: Some(LayerId::from_raw(1)),
            projection_generation: 2,
        };
        binding.overlay_upload_key = Some(key);

        binding.configured(640, 360);
        assert_eq!(binding.overlay_upload_key, Some(key));

        binding.configured(1280, 720);
        assert_eq!(binding.overlay_upload_key, None);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn stage_gpu_validation_poison_recovers_only_through_detach() {
        let mut stage = RnStageSurface {
            host_handle: 1,
            mounted: true,
            destroyed: false,
            width: 100,
            height: 50,
            scale_factor: 2.0,
            focused: false,
            pointer: None,
            gpu: StageGpuBinding {
                surface_epoch: 7,
                last_presented_epoch: Some(7),
                physical_width: 200,
                physical_height: 100,
                layer_ptr: 1,
                surface: None,
                needs_reconfigure: false,
                poisoned: false,
                overlay: None,
                overlay_upload_key: None,
            },
        };

        stage.gpu.validation_failed();
        assert_eq!(stage.gpu.surface_epoch, 8);
        assert!(stage.gpu.poisoned);
        // draw／resizeは同じpoison gateを通り、attachは重複bindingも併せて拒否する。
        assert_eq!(
            stage.gpu.reject_if_poisoned(),
            Err(RnHostReasonCode::InvalidIntent)
        );
        assert_eq!(
            stage.gpu.validate_attach(2),
            Err(RnHostReasonCode::InvalidIntent)
        );

        stage.gpu_detach_surface();
        assert_eq!(stage.gpu.surface_epoch, 9);
        assert!(!stage.gpu.is_attached());
        assert!(!stage.gpu.has_surface());
        assert!(!stage.gpu.poisoned);
        assert_eq!(stage.gpu.last_presented_epoch, None);
        assert_eq!(stage.gpu.physical_width, 0);
        assert_eq!(stage.gpu.physical_height, 0);
        assert_eq!(stage.gpu.validate_attach(2), Ok(()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn stage_gpu_host_stage_pair_mismatch_is_rejected_without_snapshot_write() {
        let _lock = test_lock();
        let host = create_host("gpu-pair-mismatch");
        let stage = host_register_stage_for_test(host).expect("stage");
        let before = read_snapshot(host);
        let outcome = run_stage_gpu_op(host + 100, stage, |_, _| Ok(()));
        assert_eq!(outcome, Err(RnHostReasonCode::UnknownHostHandle));
        let after = read_snapshot(host);
        assert_eq!(after.revision, before.revision);
        assert_eq!(after.projection_generation, before.projection_generation);
        let _ = host_destroy_stage_for_test(stage);
        let _ = host_destroy_for_test(host);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn stage_gpu_abi_rejects_off_main_before_zero_handle_validation() {
        let (written, output) = std::thread::spawn(|| {
            let mut output = vec![0_u8; MAX_JSON_BYTES];
            let written = motolii_rn_stage_draw(0, 0, output.as_mut_ptr(), output.len());
            (written, output)
        })
        .join()
        .expect("off-main gpu call");
        assert!(written > 0);
        let response = parse_wire_response(&output, written);
        assert!(!response.accepted);
        assert_eq!(
            response.diagnostics[0].reason,
            RnHostReasonCode::InvalidIntent
        );
    }

    #[test]
    fn seed_snapshot_projects_timeline_layer_interval_without_keys() {
        let _lock = test_lock();
        let host = create_host("timeline-seed");
        let snap = read_snapshot(host);
        assert_eq!(snap.timeline.layers.len(), 1);
        let layer = &snap.timeline.layers[0];
        assert_eq!(layer.layer_id, snap.layer_ids[0]);
        assert_eq!(layer.start, RationalTime::ZERO);
        assert_eq!(
            layer.duration,
            RationalTime::try_new(10, 1).expect("composition duration")
        );
        assert!(layer.position_keys.is_empty());
        assert!(!layer.keys_truncated);
        assert!(!snap.timeline.layers_truncated);
        assert_eq!(snap.timeline.fps.num(), 30);
        assert_eq!(snap.timeline.fps.den(), 1);
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn seed_snapshot_projects_stage_geometry_corners_for_unit_rect() {
        let _lock = test_lock();
        let host = create_host("stage-geom-seed");
        let wire = with_registry(|registry| registry.read_snapshot(host)).expect("wire");
        assert_eq!(wire.stage_geometry.layers.len(), 1);
        assert!(!wire.stage_geometry.layers_truncated);
        // seed: center(0,0) size(1,1) · identity world → CCW 左下起点
        assert_eq!(
            wire.stage_geometry.layers[0].corners,
            [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]]
        );
        let _ = host_destroy_for_test(host);
    }

    fn mirror_signed_area(corners: &[[f64; 2]; 4]) -> f64 {
        let p0 = corners[0];
        let p1 = corners[1];
        let p2 = corners[2];
        let p3 = corners[3];
        0.5
            * ((p0[0] * p1[1] - p1[0] * p0[1])
                + (p1[0] * p2[1] - p2[0] * p1[1])
                + (p2[0] * p3[1] - p3[0] * p2[1])
                + (p3[0] * p0[1] - p0[0] * p3[1]))
    }

    #[test]
    fn mirrored_world_geometry_is_forced_to_ccw() {
        let corners = world_rect_corners(
            motolii_doc::Affine2D::scale(-1.0, 1.0),
            [
                [-0.5, -0.5],
                [0.5, -0.5],
                [0.5, 0.5],
                [-0.5, 0.5],
            ],
        );
        assert!(mirror_signed_area(&corners) > 0.0);
        assert_eq!(
            corners,
            [
                [0.5, 0.5],
                [-0.5, 0.5],
                [-0.5, -0.5],
                [0.5, -0.5]
            ]
        );
    }

    #[test]
    fn place_rectangle_adds_stage_geometry_layer_at_drop_position() {
        let _lock = test_lock();
        let host = create_host("stage-geom-place");
        let seed = with_registry(|registry| registry.read_snapshot(host)).expect("seed");
        let seed_layer = seed.stage_geometry.layers[0].layer_id.clone();
        let response = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"place_rectangle","#,
                    r#""host_handle":"{host}","position":[0.25,-0.125],"playhead":{{"num":0,"den":1}}}}"#
                ),
                host = host,
            ),
        );
        assert!(response.accepted);
        let wire = with_registry(|registry| registry.read_snapshot(host)).expect("placed");
        assert_eq!(wire.stage_geometry.layers.len(), 2);
        let placed = wire
            .stage_geometry
            .layers
            .iter()
            .find(|layer| layer.layer_id != seed_layer)
            .expect("placed layer");
        // place Vector rect 0.2×0.2 at transform.position — world 適用済み corners
        let expected = [
            [0.15, -0.225],
            [0.35, -0.225],
            [0.35, -0.025],
            [0.15, -0.025],
        ];
        for (got, want) in placed.corners.iter().zip(expected.iter()) {
            assert!(
                (got[0] - want[0]).abs() < 1e-12 && (got[1] - want[1]).abs() < 1e-12,
                "corners {got:?} vs {want:?}"
            );
        }
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn add_position_key_appears_in_timeline_projection() {
        let _lock = test_lock();
        let host = create_host("timeline-add-key");
        let baseline = read_snapshot(host);
        let layer_id = baseline.layer_ids[0].parse::<u64>().expect("layer id");
        let target = LayerId::from_raw(layer_id);
        with_registry(|registry| {
            let product = registry
                .hosts
                .get_mut(&host)
                .ok_or(RnHostError::UnknownHost(host))?;
            let mut queue = DocumentEditQueue::default();
            queue.push_replace_primary(target);
            let published = product
                .runtime
                .process_next(&mut queue, product.primary, product.projection_generation)
                .expect("process")
                .expect("published");
            product.primary = published.primary;
            product.projection_generation = published.projection_generation;
            Ok(())
        })
        .expect("seed primary");

        let time = RationalTime::try_new(1, 1).expect("1s");
        let response = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"add_position_key","#,
                    r#""host_handle":"{host}","target":"{layer}","time":{{"num":1,"den":1}}}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        );
        assert!(response.accepted);
        let snap = response.snapshot.expect("snapshot");
        let layer = &snap.timeline.layers[0];
        assert_eq!(layer.position_keys.len(), 1);
        assert!(!layer.position_keys[0].key_id.is_empty());
        assert_eq!(layer.position_keys[0].time, time);
        assert!(!layer.keys_truncated);
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn add_position_key_snapshot_carries_document_vec2_value() {
        let _lock = test_lock();
        let host = create_host("timeline-add-key-value");
        let baseline = read_snapshot(host);
        let layer_id = baseline.layer_ids[0].parse::<u64>().expect("layer id");
        let target = LayerId::from_raw(layer_id);
        with_registry(|registry| {
            let product = registry
                .hosts
                .get_mut(&host)
                .ok_or(RnHostError::UnknownHost(host))?;
            let mut queue = DocumentEditQueue::default();
            queue.push_replace_primary(target);
            let published = product
                .runtime
                .process_next(&mut queue, product.primary, product.projection_generation)
                .expect("process")
                .expect("published");
            product.primary = published.primary;
            product.projection_generation = published.projection_generation;
            Ok(())
        })
        .expect("seed primary");

        let time = RationalTime::try_new(1, 1).expect("1s");
        let response = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"add_position_key","#,
                    r#""host_handle":"{host}","target":"{layer}","time":{{"num":1,"den":1}}}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        );
        assert!(response.accepted);
        let snap = response.snapshot.expect("snapshot");
        let key = &snap.timeline.layers[0].position_keys[0];
        assert_eq!(key.time, time);
        assert_eq!(key.value, Some([0.0, 0.0]));
        let (doc_key, doc_value) = with_registry(|registry| {
            let product = registry
                .hosts
                .get(&host)
                .ok_or(RnHostError::UnknownHost(host))?;
            Ok(position_key_at(
                product.runtime.snapshot().as_ref(),
                target,
                time,
            ))
        })
        .expect("doc lookup")
        .expect("doc key");
        assert_eq!(key.key_id, doc_key.get().to_string());
        assert_eq!(key.value, Some(doc_value));
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn set_position_key_value_updates_wire_value_and_preserves_identity_other_keys() {
        let _lock = test_lock();
        let host = create_host("timeline-set-key-value");
        let baseline = read_snapshot(host);
        let layer_id = baseline.layer_ids[0].parse::<u64>().expect("layer id");
        let target = LayerId::from_raw(layer_id);
        with_registry(|registry| {
            let product = registry
                .hosts
                .get_mut(&host)
                .ok_or(RnHostError::UnknownHost(host))?;
            let mut queue = DocumentEditQueue::default();
            queue.push_replace_primary(target);
            let published = product
                .runtime
                .process_next(&mut queue, product.primary, product.projection_generation)
                .expect("process")
                .expect("published");
            product.primary = published.primary;
            product.projection_generation = published.projection_generation;
            Ok(())
        })
        .expect("seed primary");

        let add = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"add_position_key","#,
                    r#""host_handle":"{host}","target":"{layer}","time":{{"num":1,"den":2}}}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        );
        assert!(add.accepted);
        let before = add.snapshot.expect("before");
        let before_key = before.timeline.layers[0].position_keys[0].clone();
        assert_eq!(before_key.value, Some([0.0, 0.0]));

        let second = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"add_position_key","#,
                    r#""host_handle":"{host}","target":"{layer}","time":{{"num":2,"den":1}}}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        );
        assert!(second.accepted);
        let before_second_key = second
            .snapshot
            .expect("before second")
            .timeline
            .layers[0]
            .position_keys
            .iter()
            .find(|key| key.key_id != before_key.key_id)
            .cloned()
            .expect("other key");

        let response = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"set_position_key_value","#,
                    r#""host_handle":"{host}","target":"{layer}","time":{{"num":2,"den":4}},"#,
                    r#""new":[0.25,-0.5]}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        );
        assert!(response.accepted);
        let after = response.snapshot.expect("after");
        assert_eq!(after.timeline.layers[0].position_keys.len(), 2);
        let after_key = after
            .timeline
            .layers[0]
            .position_keys
            .iter()
            .find(|key| key.key_id == before_key.key_id)
            .expect("target key");
        let after_other_key = after
            .timeline
            .layers[0]
            .position_keys
            .iter()
            .find(|key| key.key_id == before_second_key.key_id)
            .expect("other key");
        assert_eq!(after_key.key_id, before_key.key_id);
        assert_eq!(after_key.time, before_key.time);
        assert_eq!(after_key.value, Some([0.25, -0.5]));
        assert_eq!(after_other_key.key_id, before_second_key.key_id);
        assert_eq!(after_other_key.time, before_second_key.time);
        assert_eq!(after_other_key.value, Some([0.0, 0.0]));
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn timeline_position_keys_cap_at_64_and_mark_truncated() {
        let _lock = test_lock();
        let mut document = Document::new_current();
        let layer = document.layers.allocate("keyed").expect("layer");
        let track = document.track_ids.allocate("track").expect("track");
        let mut keyframes = DocKeyframeTrack::new();
        for i in 0..65 {
            let id = document.next_stable_id.allocate().expect("key id");
            keyframes.insert(DocKeyframe {
                id: KeyframeId::from_raw(id),
                t: RationalTime::try_new(i, 10).expect("key time"),
                value: DocValue::Vec2([0.0, 0.0]),
                interp: Interp::Linear,
            });
        }
        let mut envelope = ItemEnvelope::new(layer);
        envelope.transform.position = DocParam::Keyframes(keyframes);
        document.tracks.push(Track {
            id: track,
            items: vec![TrackItem::Clip(Clip {
                envelope,
                start: RationalTime::ZERO,
                duration: document.composition.duration,
                time_map: TimeMap::identity(),
                source: ClipSource::Plugin {
                    plugin_id: RECT_LAYER_SOURCE.into(),
                    effect_version: 1,
                    params: rect_params([0.0, 0.0], [1.0, 1.0]),
                    extra: Default::default(),
                },
            })],
        });
        document.validate().expect("valid keyed document");
        let host = create_host_from_document("timeline-keys-cap", &document);
        let snap = read_snapshot(host);
        let layer = &snap.timeline.layers[0];
        assert_eq!(layer.position_keys.len(), 64);
        assert!(layer.keys_truncated);
        assert_eq!(
            layer.position_keys[0].time,
            RationalTime::try_new(0, 10).expect("first")
        );
        assert_eq!(
            layer.position_keys[63].time,
            RationalTime::try_new(63, 10).expect("64th")
        );
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn snapshot_stage_bounds_and_timeline_layers_share_layer_ids_in_order() {
        let _lock = test_lock();
        let host = create_host("bounds-timeline-alignment");
        let snap = read_snapshot(host);
        let timeline_ids: Vec<String> = snap
            .timeline
            .layers
            .iter()
            .map(|layer| layer.layer_id.clone())
            .collect();
        assert_eq!(snap.layer_ids, timeline_ids);
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn snapshot_json_of_16_layers_64_keys_stays_under_the_snapshot_cap_and_untruncated() {
        let _lock = test_lock();
        let document = make_16_layers_64_keys_document();
        let host = create_host_from_document("snapshot-16x64", &document);
        let mut out = vec![0_u8; MAX_SNAPSHOT_JSON_BYTES];
        let written = motolii_rn_host_read_snapshot_json(
            host,
            out.as_mut_ptr(),
            out.len(),
        );
        assert!(written > 0);
        assert!((written as usize) < MAX_SNAPSHOT_JSON_BYTES);

        let snapshot: WireProductSnapshot =
            serde_json::from_slice(&out[..written as usize]).expect("snapshot json parse");
        assert_eq!(snapshot.timeline.layers.len(), 16);
        for layer in snapshot.timeline.layers.iter() {
            assert_eq!(layer.position_keys.len(), 64);
            assert!(!layer.keys_truncated);
        }
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn set_position_key_time_and_clip_edits_update_timeline_projection_and_undo() {
        let _lock = test_lock();
        let host = create_host("timeline-edit-intents");
        let baseline = read_snapshot(host);
        let layer_id = baseline.layer_ids[0].clone();
        let target = LayerId::from_raw(layer_id.parse::<u64>().expect("layer id"));
        with_registry(|registry| {
            let product = registry
                .hosts
                .get_mut(&host)
                .ok_or(RnHostError::UnknownHost(host))?;
            let mut queue = DocumentEditQueue::default();
            queue.push_replace_primary(target);
            let published = product
                .runtime
                .process_next(&mut queue, product.primary, product.projection_generation)
                .expect("process")
                .expect("published");
            product.primary = published.primary;
            product.projection_generation = published.projection_generation;
            Ok(())
        })
        .expect("seed primary");

        let before = read_snapshot(host);
        let before_layer = before
            .timeline
            .layers
            .iter()
            .find(|layer| layer.layer_id == layer_id)
            .expect("layer");
        let before_start = before_layer.start;
        let before_duration = before_layer.duration;

        let add = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"add_position_key","#,
                    r#""host_handle":"{host}","target":"{layer}","time":{{"num":1,"den":1}}}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        );
        assert!(add.accepted);
        let key_id = add
            .snapshot
            .expect("keyed")
            .timeline
            .layers
            .iter()
            .find(|layer| layer.layer_id == layer_id)
            .expect("layer")
            .position_keys[0]
            .key_id
            .clone();

        let moved_key = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"set_position_key_time","#,
                    r#""host_handle":"{host}","target":"{layer}","key_id":"{key}","#,
                    r#""time":{{"num":2,"den":1}}}}"#
                ),
                host = host,
                layer = layer_id,
                key = key_id,
            ),
        );
        assert!(moved_key.accepted);
        let after_key_layer = moved_key
            .snapshot
            .expect("key moved")
            .timeline
            .layers
            .into_iter()
            .find(|layer| layer.layer_id == layer_id)
            .expect("layer");
        assert_eq!(
            after_key_layer.position_keys[0].time,
            RationalTime::from_seconds(2)
        );
        assert_eq!(after_key_layer.position_keys[0].key_id, key_id);

        // 先に右edgeを短くしてから start を動かす(compositionはみ出しを避ける)。
        let trimmed = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"trim_clip_out","#,
                    r#""host_handle":"{host}","target":"{layer}","time":{{"num":3,"den":1}}}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        );
        assert!(trimmed.accepted);
        let after_trim_layer = trimmed
            .snapshot
            .expect("trimmed")
            .timeline
            .layers
            .into_iter()
            .find(|layer| layer.layer_id == layer_id)
            .expect("layer");
        assert_eq!(after_trim_layer.start, before_start);
        assert_eq!(
            after_trim_layer.duration,
            RationalTime::from_seconds(3)
        );

        let moved_clip = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"set_clip_start","#,
                    r#""host_handle":"{host}","target":"{layer}","time":{{"num":1,"den":2}}}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        );
        assert!(moved_clip.accepted);
        let after_move_layer = moved_clip
            .snapshot
            .expect("clip moved")
            .timeline
            .layers
            .into_iter()
            .find(|layer| layer.layer_id == layer_id)
            .expect("layer");
        assert_eq!(
            after_move_layer.start,
            RationalTime::try_new(1, 2).unwrap()
        );
        assert_eq!(after_move_layer.duration, RationalTime::from_seconds(3));

        let trimmed_in = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"trim_clip_in","#,
                    r#""host_handle":"{host}","target":"{layer}","time":{{"num":1,"den":1}}}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        );
        assert!(trimmed_in.accepted);
        let after_in_layer = trimmed_in
            .snapshot
            .expect("in")
            .timeline
            .layers
            .into_iter()
            .find(|layer| layer.layer_id == layer_id)
            .expect("layer");
        assert_eq!(after_in_layer.start, RationalTime::from_seconds(1));
        // start 1..3.5 → duration 2.5 after left trim from 0.5
        assert_eq!(
            after_in_layer.duration,
            RationalTime::try_new(5, 2).unwrap()
        );

        for _ in 0..5 {
            assert!(dispatch_raw_json(
                host,
                &format!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"undo","host_handle":"{host}"}}"#
                ),
            )
            .accepted);
        }
        let restored_layer = read_snapshot(host)
            .timeline
            .layers
            .into_iter()
            .find(|layer| layer.layer_id == layer_id)
            .expect("layer");
        assert_eq!(restored_layer.start, before_start);
        assert_eq!(restored_layer.duration, before_duration);
        assert!(restored_layer.position_keys.is_empty());
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn select_layer_and_clear_selection_update_primary_layer_id() {
        let _lock = test_lock();
        let host = create_host("select-clear");
        let baseline = read_snapshot(host);
        let layer_id = baseline.layer_ids[0].clone();
        assert!(baseline.primary_layer_id.is_none());

        let selected = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"select_layer","#,
                    r#""host_handle":"{host}","target":"{layer}"}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        );
        assert!(selected.accepted);
        assert_eq!(
            selected.snapshot.expect("selected").primary_layer_id,
            Some(layer_id.clone())
        );
        assert_eq!(read_snapshot(host).primary_layer_id, Some(layer_id));

        let cleared = dispatch_raw_json(
            host,
            &format!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"clear_selection","host_handle":"{host}"}}"#
            ),
        );
        assert!(cleared.accepted);
        assert!(cleared
            .snapshot
            .expect("cleared")
            .primary_layer_id
            .is_none());
        assert!(read_snapshot(host).primary_layer_id.is_none());
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn remove_position_key_clears_timeline_projection_and_undo_restores() {
        let _lock = test_lock();
        let host = create_host("remove-position-key");
        let baseline = read_snapshot(host);
        let layer_id = baseline.layer_ids[0].clone();
        let target = LayerId::from_raw(layer_id.parse::<u64>().expect("layer id"));
        with_registry(|registry| {
            let product = registry
                .hosts
                .get_mut(&host)
                .ok_or(RnHostError::UnknownHost(host))?;
            let mut queue = DocumentEditQueue::default();
            queue.push_replace_primary(target);
            let published = product
                .runtime
                .process_next(&mut queue, product.primary, product.projection_generation)
                .expect("process")
                .expect("published");
            product.primary = published.primary;
            product.projection_generation = published.projection_generation;
            Ok(())
        })
        .expect("seed primary");

        let add = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"add_position_key","#,
                    r#""host_handle":"{host}","target":"{layer}","time":{{"num":1,"den":1}}}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        );
        assert!(add.accepted);
        let key_id = add
            .snapshot
            .expect("keyed")
            .timeline
            .layers
            .iter()
            .find(|layer| layer.layer_id == layer_id)
            .expect("layer")
            .position_keys[0]
            .key_id
            .clone();

        let removed = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"remove_position_key","#,
                    r#""host_handle":"{host}","target":"{layer}","key_id":"{key}"}}"#
                ),
                host = host,
                layer = layer_id,
                key = key_id,
            ),
        );
        assert!(removed.accepted);
        let after_remove = removed.snapshot.expect("removed");
        let after_layer = after_remove
            .timeline
            .layers
            .iter()
            .find(|layer| layer.layer_id == layer_id)
            .expect("layer");
        assert!(after_layer.position_keys.is_empty());

        let undone = dispatch_raw_json(
            host,
            &format!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"undo","host_handle":"{host}"}}"#
            ),
        );
        assert!(undone.accepted);
        let restored = undone
            .snapshot
            .expect("restored")
            .timeline
            .layers
            .into_iter()
            .find(|layer| layer.layer_id == layer_id)
            .expect("layer");
        assert_eq!(restored.position_keys.len(), 1);
        assert_eq!(restored.position_keys[0].key_id, key_id);
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn delete_layer_removes_timeline_row_and_undo_restores_id_and_name() {
        let _lock = test_lock();
        let host = create_host("delete-layer");
        let before = read_snapshot(host);
        let layer_id = before.layer_ids[0].clone();
        let display_name = before
            .timeline
            .layers
            .iter()
            .find(|layer| layer.layer_id == layer_id)
            .expect("layer")
            .display_name
            .clone();

        let deleted = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"delete_layer","#,
                    r#""host_handle":"{host}","target":"{layer}"}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        );
        assert!(deleted.accepted);
        let after = deleted.snapshot.expect("deleted");
        assert!(!after
            .timeline
            .layers
            .iter()
            .any(|layer| layer.layer_id == layer_id));
        assert!(!after.layer_ids.iter().any(|id| id == &layer_id));

        let undone = dispatch_raw_json(
            host,
            &format!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"undo","host_handle":"{host}"}}"#
            ),
        );
        assert!(undone.accepted);
        let restored = undone.snapshot.expect("restored");
        let layer = restored
            .timeline
            .layers
            .iter()
            .find(|layer| layer.layer_id == layer_id)
            .expect("restored layer");
        assert_eq!(layer.display_name, display_name);
        assert!(restored.layer_ids.iter().any(|id| id == &layer_id));
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn missing_target_selection_and_delete_intents_reject_without_document_mutation() {
        let _lock = test_lock();
        let host = create_host("missing-target-reject");
        let baseline = read_snapshot(host);
        let missing = "999999";

        for kind in ["select_layer", "delete_layer"] {
            let rejected = dispatch_raw_json(
                host,
                &format!(
                    concat!(
                        r#"{{"version":1,"direction":"rn-to-host","kind":"{kind}","#,
                        r#""host_handle":"{host}","target":"{missing}"}}"#
                    ),
                    kind = kind,
                    host = host,
                    missing = missing,
                ),
            );
            assert!(!rejected.accepted, "{kind} must reject missing target");
            assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));
            let after = read_snapshot(host);
            assert_eq!(after.revision, baseline.revision);
            assert_eq!(after.projection_generation, baseline.projection_generation);
            assert_eq!(after.primary_layer_id, baseline.primary_layer_id);
            assert_eq!(after.layer_ids, baseline.layer_ids);
        }
        let _ = host_destroy_for_test(host);
    }

    fn position_const_at(document: &Document, target: LayerId) -> Option<[f64; 2]> {
        let envelope = find_envelope_in_document(document, target)?;
        match &envelope.transform.position {
            DocParam::Const(DocValue::Vec2(value)) => Some(*value),
            _ => None,
        }
    }

    #[test]
    fn move_layer_by_const_updates_position_and_undo_restores() {
        let _lock = test_lock();
        let host = create_host("move-const");
        let baseline = read_snapshot(host);
        let layer_id = baseline.layer_ids[0].parse::<u64>().expect("layer id");
        let target = LayerId::from_raw(layer_id);
        let before = with_registry(|registry| {
            let product = registry
                .hosts
                .get(&host)
                .ok_or(RnHostError::UnknownHost(host))?;
            Ok(position_const_at(
                product.runtime.snapshot().as_ref(),
                target,
            ))
        })
        .expect("lookup")
        .expect("const position");

        let delta = [0.1, -0.05];
        let response = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"move_layer_by","#,
                    r#""host_handle":"{host}","target":"{layer}","delta":[0.1,-0.05]}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        );
        assert!(response.accepted);
        let after = with_registry(|registry| {
            let product = registry
                .hosts
                .get(&host)
                .ok_or(RnHostError::UnknownHost(host))?;
            Ok(position_const_at(
                product.runtime.snapshot().as_ref(),
                target,
            ))
        })
        .expect("lookup")
        .expect("const position");
        assert!((after[0] - (before[0] + delta[0])).abs() < 1e-12);
        assert!((after[1] - (before[1] + delta[1])).abs() < 1e-12);

        let undone = dispatch_raw_json(
            host,
            &format!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"undo","host_handle":"{host}"}}"#
            ),
        );
        assert!(undone.accepted);
        let restored = with_registry(|registry| {
            let product = registry
                .hosts
                .get(&host)
                .ok_or(RnHostError::UnknownHost(host))?;
            Ok(position_const_at(
                product.runtime.snapshot().as_ref(),
                target,
            ))
        })
        .expect("lookup")
        .expect("const position");
        assert_eq!(restored, before);
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn move_layer_by_exact_on_key_updates_only_that_key_value_and_off_key_rejects() {
        let _lock = test_lock();
        let host = create_host("move-on-key");
        let baseline = read_snapshot(host);
        let layer_id = baseline.layer_ids[0].parse::<u64>().expect("layer id");
        let target = LayerId::from_raw(layer_id);
        with_registry(|registry| {
            let product = registry
                .hosts
                .get_mut(&host)
                .ok_or(RnHostError::UnknownHost(host))?;
            let mut queue = DocumentEditQueue::default();
            queue.push_replace_primary(target);
            let published = product
                .runtime
                .process_next(&mut queue, product.primary, product.projection_generation)
                .expect("process")
                .expect("published");
            product.primary = published.primary;
            product.projection_generation = published.projection_generation;
            Ok(())
        })
        .expect("seed primary");

        assert!(dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"add_position_key","#,
                    r#""host_handle":"{host}","target":"{layer}","time":{{"num":0,"den":1}}}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        )
        .accepted);
        assert!(dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"add_position_key","#,
                    r#""host_handle":"{host}","target":"{layer}","time":{{"num":2,"den":1}}}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        )
        .accepted);
        let before = read_snapshot(host);
        let before_keys = before.timeline.layers[0].position_keys.clone();
        assert_eq!(before_keys.len(), 2);
        let on_key_id = before_keys[0].key_id.clone();
        let before_doc = document_json_bytes(host);

        // current_time は seed で 0。exact-on-key。
        let moved = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"move_layer_by","#,
                    r#""host_handle":"{host}","target":"{layer}","delta":[0.2,0.1]}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        );
        assert!(moved.accepted);
        let after = moved.snapshot.expect("after");
        let after_keys = &after.timeline.layers[0].position_keys;
        assert_eq!(after_keys.len(), 2);
        let mut before_keys_sorted = before_keys.to_vec();
        before_keys_sorted.sort_by_key(|key| key.key_id.clone());
        let mut after_keys = after_keys.to_vec();
        after_keys.sort_by_key(|key| key.key_id.clone());
        for before_key in before_keys_sorted {
            let after_key = after_keys
                .iter()
                .find(|key| key.key_id == before_key.key_id)
                .expect("all keys preserved");
            if before_key.key_id == on_key_id {
                assert_eq!(after_key.key_id, before_key.key_id);
                assert_eq!(after_key.time, before_key.time);
                assert_eq!(after_key.value, Some([0.2, 0.1]));
            } else {
                assert_eq!(after_key, &before_key);
            }
        }
        let after_on = after_keys
            .iter()
            .find(|key| key.key_id == on_key_id)
            .expect("on key");
        assert_eq!(after_on.key_id, on_key_id);
        assert_eq!(after_on.value, Some([0.2, 0.1]));

        // off-key: frame へ進めて拒否。Document 不変。
        assert!(dispatch_raw_json(host, &set_time_json(host, "15")).accepted);
        let before_off = document_json_bytes(host);
        let rejected = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"move_layer_by","#,
                    r#""host_handle":"{host}","target":"{layer}","delta":[0.05,0.0]}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        );
        assert!(!rejected.accepted);
        assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));
        assert_eq!(document_json_bytes(host), before_off);
        let _ = before_doc;
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn move_layer_by_rotated_scaled_layer_uses_world_inverse_delta() {
        let _lock = test_lock();
        let mut fixture = Fixture::new();
        let layer = fixture.push_rect_layer(
            "rotScaled",
            [0.0, 0.0],
            [0.4, 0.4],
            Transform2D {
                position: DocParam::const_vec2([0.1, -0.08]),
                rotation: DocParam::const_f64(0.55),
                scale: DocParam::const_vec2([1.25, 0.8]),
                ..Transform2D::identity()
            },
        );
        fixture.document.validate().expect("valid");
        let host = create_host_from_document("move-rot-scale", &fixture.document);
        let layer_id = layer.get().to_string();

        let tracks = DataTracks::new();
        let projection = project_stage_geometry(
            &fixture.document,
            EvaluationTime::new(RationalTime::ZERO),
            &tracks,
        )
        .expect("geometry");
        let crate::stage_geometry_projection::StageLayerProjection::Available(geo) = projection
            .get(layer)
            .expect("layer")
        else {
            panic!("available");
        };

        let delta = [0.18, -0.12];
        let expected_local = world_delta_to_position_local(geo.world, delta)
            .expect("local inverse exists");
        let before = with_registry(|registry| {
            let product = registry
                .hosts
                .get(&host)
                .ok_or(RnHostError::UnknownHost(host))?;
            Ok(position_const_at(
                product.runtime.snapshot().as_ref(),
                LayerId::from_raw(layer_id.parse::<u64>().expect("id")),
            ))
        })
        .expect("lookup")
        .expect("const position");

        let moved = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"move_layer_by","#,
                    r#""host_handle":"{host}","target":"{layer}","delta":[{dx},{dy}]}}"#
                ),
                host = host,
                layer = layer_id,
                dx = delta[0],
                dy = delta[1],
            ),
        );
        assert!(moved.accepted);
        let after = with_registry(|registry| {
            let product = registry
                .hosts
                .get(&host)
                .ok_or(RnHostError::UnknownHost(host))?;
            Ok(position_const_at(
                product.runtime.snapshot().as_ref(),
                LayerId::from_raw(layer_id.parse::<u64>().expect("id")),
            ))
        })
        .expect("lookup")
        .expect("const position");
        assert!((after[0] - (before[0] + expected_local[0])).abs() < 1e-12);
        assert!((after[1] - (before[1] + expected_local[1])).abs() < 1e-12);

        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn move_layer_by_rejects_non_finite_delta_and_missing_target() {
        let _lock = test_lock();
        let host = create_host("move-reject");
        let baseline = read_snapshot(host);
        let layer_id = baseline.layer_ids[0].clone();
        let before = document_json_bytes(host);

        let missing = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"move_layer_by","#,
                    r#""host_handle":"{host}","target":"999999","delta":[0.1,0.0]}}"#
                ),
                host = host,
            ),
        );
        assert!(!missing.accepted);
        assert_eq!(missing.reason, Some(RnHostReasonCode::InvalidIntent));
        assert_eq!(document_json_bytes(host), before);

        let mut intent = WireIntentEnvelope {
            version: WIRE_VERSION,
            direction: RN_TO_HOST.to_owned(),
            kind: "move_layer_by".to_owned(),
            host_handle: String::new(),
            stage_handle: None,
            projection_generation: None,
            width: None,
            height: None,
            scale_factor: None,
            focused: None,
            phase: None,
            view_local_x: None,
            view_local_y: None,
            sequence: None,
            frame: None,
            position: None,
            playhead: None,
            target: Some(layer_id),
            key_id: None,
            time: None,
            new: None,
            interp: None,
            delta: Some([f64::INFINITY, 0.0]),
        };
        let non_finite = dispatch_wire(host, intent.clone());
        assert!(!non_finite.accepted);
        assert_eq!(non_finite.reason, Some(RnHostReasonCode::InvalidIntent));
        assert_eq!(document_json_bytes(host), before);

        intent.delta = Some([0.0, f64::NAN]);
        let nan = dispatch_wire(host, intent);
        assert!(!nan.accepted);
        assert_eq!(document_json_bytes(host), before);
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn host_render_frame_returns_texture_and_dirty_gates() {
        let _lock = test_lock();
        let Some(gpu) = motolii_testkit::gpu_or_skip() else {
            // BLOCKED(GPU): sandboxにadapterが無い場合はsupervisorが実機で回す。
            return;
        };
        let mut session = RenderSession::new(&gpu);
        let host = create_host("stage-frame-dirty");
        assert!(dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"place_rectangle","#,
                    r#""host_handle":"{host}","position":[0.0,0.0],"playhead":{{"num":0,"den":1}}}}"#
                ),
                host = host,
            ),
        )
        .accepted);

        let mut frame = None;
        assert_eq!(
            host_render_frame_for_app(host, &gpu, &mut session, &mut frame),
            HostRenderFrameResult::Rendered
        );
        let first = frame.take().expect("frame");
        let draft = Quality::DRAFT
            .render_desc(frame_desc_from_composition(&Document::new_current()).expect("desc"));
        assert_eq!((first.width, first.height), (draft.width, draft.height));
        assert_eq!(
            host_render_frame_for_app(host, &gpu, &mut session, &mut frame),
            HostRenderFrameResult::Unchanged
        );
        assert!(frame.is_none());

        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn host_render_frame_rerenders_on_revision_and_time() {
        let _lock = test_lock();
        let Some(gpu) = motolii_testkit::gpu_or_skip() else {
            return;
        };
        let mut session = RenderSession::new(&gpu);
        let host = create_host("stage-frame-rerender");
        assert!(dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"place_rectangle","#,
                    r#""host_handle":"{host}","position":[0.1,0.0],"playhead":{{"num":0,"den":1}}}}"#
                ),
                host = host,
            ),
        )
        .accepted);

        let mut frame = None;
        assert_eq!(
            host_render_frame_for_app(host, &gpu, &mut session, &mut frame),
            HostRenderFrameResult::Rendered
        );
        assert_eq!(
            host_render_frame_for_app(host, &gpu, &mut session, &mut frame),
            HostRenderFrameResult::Unchanged
        );
        let rev1 = frame.as_ref().expect("f1").revision.clone();

        assert!(dispatch_raw_json(
            host,
            &format!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"undo","host_handle":"{host}"}}"#
            ),
        )
        .accepted);
        assert_eq!(
            host_render_frame_for_app(host, &gpu, &mut session, &mut frame),
            HostRenderFrameResult::Rendered
        );
        let after_undo = frame.as_ref().expect("undo frame");
        assert_ne!(after_undo.revision, rev1);

        assert!(dispatch_raw_json(host, &set_time_json(host, "30")).accepted);
        assert_eq!(
            host_render_frame_for_app(host, &gpu, &mut session, &mut frame),
            HostRenderFrameResult::Rendered
        );
        let after_time = frame.as_ref().expect("time frame");
        assert_eq!(after_time.time, RationalTime::try_new(1, 1).expect("1/1"));
        assert_eq!(
            host_render_frame_for_app(host, &gpu, &mut session, &mut frame),
            HostRenderFrameResult::Unchanged
        );

        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn host_render_frame_unknown_handle_is_false() {
        let _lock = test_lock();
        let Some(gpu) = motolii_testkit::gpu_or_skip() else {
            return;
        };
        let mut session = RenderSession::new(&gpu);
        let mut frame = None;
        assert_eq!(
            host_render_frame_for_app(9_999, &gpu, &mut session, &mut frame),
            HostRenderFrameResult::Failed
        );
        assert!(frame.is_none());
    }

    #[test]
    fn host_render_frame_after_seed_place_has_non_uniform_pixels() {
        let _lock = test_lock();
        let Some(gpu) = motolii_testkit::gpu_or_skip() else {
            return;
        };
        let mut session = RenderSession::new(&gpu);
        let host = create_host("stage-frame-readback");
        assert!(dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"place_rectangle","#,
                    r#""host_handle":"{host}","position":[0.0,0.0],"playhead":{{"num":0,"den":1}}}}"#
                ),
                host = host,
            ),
        )
        .accepted);

        let mut frame = None;
        assert_eq!(
            host_render_frame_for_app(host, &gpu, &mut session, &mut frame),
            HostRenderFrameResult::Rendered
        );
        let first = frame.take().expect("frame");
        let draft = Quality::DRAFT
            .render_desc(frame_desc_from_composition(&Document::new_current()).expect("desc"));
        assert_eq!((first.width, first.height), (draft.width, draft.height));

        let bytes = download_rgba(&gpu, &first.texture).expect("frame readback");
        assert_eq!(bytes.len(), (first.width as usize) * (first.height as usize) * 4);
        let center = pixel_at(&bytes, first.width, first.width / 2, first.height / 2);
        let background = pixel_at(&bytes, first.width, 0, 0);
        assert_ne!(center, background);
        assert!(has_non_background_pixel(&bytes, first.width, first.height, background));

        let _ = host_destroy_for_test(host);
    }
}
