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

use motolii_core::RationalTime;
use motolii_doc::{EvaluationTime, LayerId};
use motolii_eval::DataTracks;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(target_os = "macos")]
use motolii_gpu::GpuCtx;
#[cfg(target_os = "macos")]
use wgpu::{
    Color, CompositeAlphaMode, CurrentSurfaceTexture, LoadOp, Operations, PresentMode,
    RenderPassColorAttachment, RenderPassDescriptor, StoreOp, Surface, SurfaceConfiguration,
    SurfaceTargetUnsafe, TextureFormat, TextureUsages,
};

use crate::document_edit_runtime::{
    DocumentEditQueue, DocumentEditRuntime, DocumentEditRuntimeError,
};
use crate::shell::{open_project_runtime, ShellError};
use crate::stage_geometry_projection::project_stage_geometry;
use crate::stage_hit_test::{
    hit_test_projected_layers, view_local_in_stage, view_local_to_canonical, StageHit,
    StageHitTestReject,
};
#[cfg(target_os = "macos")]
use crate::static_preview::{bootstrap_frame_desc, prepare_in_setup_worker, StaticPreview};

const WIRE_VERSION: u8 = 1;
const HOST_TO_RN: &str = "host-to-rn";
const RN_TO_HOST: &str = "rn-to-host";
const PRODUCT_ROLE: &str = "product-runtime-seat";
const MAX_STAGE_BOUNDS: usize = 16;
const MAX_STAGE_SELECTION: usize = 16;
const MAX_DIAGNOSTICS: usize = 8;
const MAX_JSON_BYTES: usize = 16_384;
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
    #[error("host handle {0} was already destroyed")]
    DestroyedHost(u64),
    #[error("stage handle {0} was already destroyed")]
    DestroyedStage(u64),
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
    DestroyedHostHandle,
    DestroyedStageHandle,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    diagnostics: Vec<RnHostDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct WireStageProjection {
    selection: Vec<WireStageSelection>,
    bounds: Vec<WireStageBound>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[doc(hidden)]
pub struct RnProductSnapshotForTest {
    pub revision: String,
    pub projection_generation: String,
    pub current_time: RationalTime,
    pub primary_layer_id: Option<String>,
    pub layer_ids: Vec<String>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    }

    fn validation_failed(&mut self) {
        self.poisoned = true;
        self.needs_reconfigure = false;
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

struct RnProductHost {
    runtime: DocumentEditRuntime,
    projection_generation: u64,
    /// Document 外の transient 評価時刻。初期値は ZERO。
    current_time: RationalTime,
    primary: Option<LayerId>,
    stages: HashMap<u64, RnStageSurface>,
    destroyed: bool,
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
                accept(self.snapshot_wire(host_handle))
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
            self.gpu = Some(HostGpuBundle {
                ctx,
                instance: parts.instance,
                adapter: parts.adapter,
                _preview: preview,
                preview_pipeline,
                preview_bind_group,
            });
        }
        Ok(self.gpu.as_mut().expect("gpu bundle initialized"))
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
        let gpu = self.gpu.as_ref().ok_or(RnHostReasonCode::InvalidIntent)?;
        let surface = self
            .stages
            .get(&stage_handle)
            .ok_or(RnHostReasonCode::UnknownStageHandle)?
            .gpu
            .surface
            .as_ref()
            .ok_or(RnHostReasonCode::InvalidIntent)?;
        match surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) => {
                draw_stage_preview(gpu, frame);
                self.stages
                    .get_mut(&stage_handle)
                    .ok_or(RnHostReasonCode::UnknownStageHandle)?
                    .gpu
                    .presented(false);
                Ok(())
            }
            CurrentSurfaceTexture::Suboptimal(frame) => {
                draw_stage_preview(gpu, frame);
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
    fn detach_all_stage_surfaces(&mut self) {
        let stage_handles = self.stages.keys().copied().collect::<Vec<_>>();
        for stage_handle in stage_handles {
            if let Some(stage) = self.stages.get_mut(&stage_handle) {
                stage.gpu_detach_surface();
            }
        }
    }

    fn destroy(&mut self, host_handle: u64) -> Result<(), RnHostError> {
        if self.destroyed {
            return Err(RnHostError::DestroyedHost(host_handle));
        }
        #[cfg(target_os = "macos")]
        self.detach_all_stage_surfaces();
        self.destroyed = true;
        self.stages.clear();
        #[cfg(target_os = "macos")]
        {
            self.gpu = None;
        }
        Ok(())
    }
}

#[cfg(target_os = "macos")]
fn draw_stage_preview(gpu: &HostGpuBundle, frame: wgpu::SurfaceTexture) {
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
    gpu.ctx.queue.submit(Some(encoder.finish()));
    frame.present();
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
        if self.gpu.is_attached() {
            self.gpu.surface = None;
            self.gpu.layer_ptr = 0;
            self.gpu.physical_width = 0;
            self.gpu.physical_height = 0;
            self.gpu.last_presented_epoch = None;
            self.gpu.needs_reconfigure = false;
            self.gpu.poisoned = false;
            self.gpu.surface_epoch = self.gpu.surface_epoch.saturating_add(1);
        }
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

struct RnHostRegistry {
    next_host_handle: u64,
    next_stage_handle: u64,
    hosts: HashMap<u64, RnProductHost>,
    destroyed_hosts: HashSet<u64>,
    destroyed_stages: HashSet<u64>,
}

impl Default for RnHostRegistry {
    fn default() -> Self {
        Self {
            next_host_handle: 1,
            next_stage_handle: 1,
            hosts: HashMap::new(),
            destroyed_hosts: HashSet::new(),
            destroyed_stages: HashSet::new(),
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
                destroyed: false,
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

    fn destroy_host(&mut self, host_handle: u64) -> Result<(), RnHostError> {
        let Some(host) = self.hosts.get_mut(&host_handle) else {
            return if self.destroyed_hosts.contains(&host_handle) {
                Err(RnHostError::DestroyedHost(host_handle))
            } else {
                Err(RnHostError::UnknownHost(host_handle))
            };
        };
        self.destroyed_stages.extend(host.stages.keys().copied());
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
        expected_projection_generation,
        actual_projection_generation,
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

pub fn host_read_snapshot_for_test(
    host_handle: u64,
) -> Result<RnProductSnapshotForTest, RnHostError> {
    with_registry(|registry| registry.read_snapshot(host_handle)).map(snapshot_for_test)
}

pub fn host_dispatch_intent_for_test(
    host_handle: u64,
    intent: RnHostTestIntent,
) -> Result<RnHostTestResponse, RnHostError> {
    let wire_intent = WireIntentEnvelope {
        version: WIRE_VERSION,
        direction: RN_TO_HOST.to_owned(),
        kind: intent.kind,
        host_handle: host_handle.to_string(),
        stage_handle: intent.stage_handle.map(|value| value.to_string()),
        projection_generation: intent.projection_generation,
        width: intent.width,
        height: intent.height,
        scale_factor: intent.scale_factor,
        focused: intent.focused,
        phase: None,
        view_local_x: None,
        view_local_y: None,
        sequence: None,
        frame: None,
    };
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
            Ok(snapshot) => match encode_json(&snapshot) {
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
#[allow(dead_code)]
const _: fn() = || {
    let _ =
        motolii_rn_host_create as extern "C" fn(*const u8, usize, *mut u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_host_destroy as extern "C" fn(u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_stage_register as extern "C" fn(u64, *mut u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_stage_destroy as extern "C" fn(u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_host_read_snapshot_json as extern "C" fn(u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_host_dispatch_intent_json
        as extern "C" fn(u64, *const u8, usize, *mut u8, usize) -> i64;
    let _ = motolii_rn_stage_attach
        as extern "C" fn(u64, u64, *mut core::ffi::c_void, *mut u8, usize) -> i64;
    let _ = motolii_rn_stage_resize_physical
        as extern "C" fn(u64, u64, u32, u32, *mut u8, usize) -> i64;
    let _ = motolii_rn_stage_draw as extern "C" fn(u64, u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_stage_detach as extern "C" fn(u64, u64, *mut u8, usize) -> i64;
};

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Mutex, MutexGuard};

    use motolii_core::{RationalTime, TimeMap};
    use motolii_doc::{
        Clip, ClipSource, CompCameraDoc, DocParam, Document, ItemEnvelope, LayerId, ProjectSession,
        ResourceLimits, SaveProjectOptions, Track, TrackItem, Transform2D, RECT_LAYER_SOURCE,
    };
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

    fn rect_params(center: [f64; 2], size: [f64; 2]) -> BTreeMap<String, DocParam> {
        BTreeMap::from([
            ("center".into(), DocParam::const_vec2(center)),
            ("size".into(), DocParam::const_vec2(size)),
            ("color".into(), DocParam::const_color([0.0, 1.0, 0.0, 1.0])),
        ])
    }

    fn push_rect_layer(
        document: &mut Document,
        name: &str,
        center: [f64; 2],
        size: [f64; 2],
        transform: Transform2D,
    ) -> LayerId {
        if document.tracks.is_empty() {
            let track = document.track_ids.allocate("V1").expect("track");
            document.tracks.push(Track {
                id: track,
                items: vec![],
            });
        }
        let layer = document.layers.allocate(name).expect("layer");
        let mut envelope = ItemEnvelope::new(layer);
        envelope.transform = transform;
        document.tracks[0].items.push(TrackItem::Clip(Clip {
            envelope,
            start: RationalTime::ZERO,
            duration: document.composition.duration,
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
        let mut document = Document::new_current();
        let layer = push_rect_layer(
            &mut document,
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
        document.composition.camera = CompCameraDoc::PlanarOrthographic {
            center: DocParam::const_vec2([0.03, -0.02]),
            roll_radians: DocParam::const_f64(0.2),
            height: DocParam::const_f64(1.0),
        };
        document.validate().expect("valid");
        let host = create_host_from_document("sel-rotated", &document);
        let stage = host_register_stage_for_test(host).expect("stage");
        mount_and_resize(host, stage, 1600, 900);
        let before_bytes = document_json_bytes(host);
        let before = read_snapshot(host);

        // 局所原点付近を camera∘world で正準へ写し、非対称な view-local へ戻す。
        let tracks = DataTracks::new();
        let proj =
            project_stage_geometry(&document, EvaluationTime::new(RationalTime::ZERO), &tracks)
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
        let mut document = Document::new_current();
        let layer = push_rect_layer(
            &mut document,
            "x-height",
            [0.0, 0.0],
            [0.7, 0.7],
            Transform2D {
                // 非 identity（平行移動）。逆写像を無視すると中心がずれる。
                position: DocParam::const_vec2([0.0, 0.05]),
                ..Transform2D::identity()
            },
        );
        document.validate().expect("valid");
        let host = create_host_from_document("sel-x-height", &document);
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
    fn stage_pointer_down_y_up_hits_upper_half_rect() {
        let _lock = test_lock();
        let mut document = Document::new_current();
        let layer = push_rect_layer(
            &mut document,
            "upper",
            [0.0, 0.25],
            [0.4, 0.3],
            Transform2D::identity(),
        );
        document.validate().expect("valid");
        let host = create_host_from_document("sel-y-up", &document);
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
        let mut document = Document::new_current();
        let layer = push_rect_layer(
            &mut document,
            "target",
            [0.0, 0.0],
            [0.4, 0.4],
            Transform2D {
                position: DocParam::const_vec2([-0.2, 0.15]),
                rotation: DocParam::const_f64(-0.3),
                ..Transform2D::identity()
            },
        );
        document.validate().expect("valid");
        let host = create_host_from_document("sel-clear", &document);
        let stage = host_register_stage_for_test(host).expect("stage");
        mount_and_resize(host, stage, 1600, 900);
        let before_bytes = document_json_bytes(host);

        let tracks = DataTracks::new();
        let proj =
            project_stage_geometry(&document, EvaluationTime::new(RationalTime::ZERO), &tracks)
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
        let mut document = Document::new_current();
        let back = push_rect_layer(
            &mut document,
            "back",
            [0.0, 0.0],
            [0.8, 0.8],
            Transform2D::identity(),
        );
        let front = push_rect_layer(
            &mut document,
            "front",
            [0.0, 0.0],
            [0.5, 0.5],
            Transform2D {
                position: DocParam::const_vec2([0.05, -0.04]),
                ..Transform2D::identity()
            },
        );
        document.validate().expect("valid");
        let host = create_host_from_document("sel-overlap", &document);
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
        let mut document = Document::new_current();
        let layer = push_rect_layer(
            &mut document,
            "same",
            [0.0, 0.0],
            [0.5, 0.5],
            Transform2D {
                position: DocParam::const_vec2([0.2, -0.1]),
                rotation: DocParam::const_f64(0.4),
                ..Transform2D::identity()
            },
        );
        document.validate().expect("valid");
        let host = create_host_from_document("sel-same-id", &document);
        let stage = host_register_stage_for_test(host).expect("stage");
        mount_and_resize(host, stage, 1600, 900);
        let tracks = DataTracks::new();
        let proj =
            project_stage_geometry(&document, EvaluationTime::new(RationalTime::ZERO), &tracks)
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
        let mut document = Document::new_current();
        let layer = push_rect_layer(
            &mut document,
            "keep",
            [0.0, 0.0],
            [0.4, 0.4],
            Transform2D {
                position: DocParam::const_vec2([0.1, 0.1]),
                ..Transform2D::identity()
            },
        );
        document.validate().expect("valid");
        let host = create_host_from_document("sel-phase-keep", &document);
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
        let mut document = Document::new_current();
        let layer = push_rect_layer(
            &mut document,
            "z",
            [0.0, 0.0],
            [0.4, 0.4],
            Transform2D::identity(),
        );
        document.validate().expect("valid");
        let host = create_host_from_document("sel-zero", &document);
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
    fn stage_pointer_down_geometry_error_keeps_primary() {
        let _lock = test_lock();
        let mut document = Document::new_current();
        let layer = push_rect_layer(
            &mut document,
            "singular",
            [0.0, 0.0],
            [0.5, 0.5],
            Transform2D {
                scale: DocParam::const_vec2([0.0, 1.0]),
                ..Transform2D::identity()
            },
        );
        document.validate().expect("valid");
        let host = create_host_from_document("sel-geom-err", &document);
        // 幾何は壊れていても ReplacePrimary は envelope 存在だけで受理できる。
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

        let rejected = dispatch_raw_json(host, &pointer_json(host, stage, "down", 800.0, 450.0, 1));
        assert!(!rejected.accepted);
        assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));
        let after = read_snapshot(host);
        assert_eq!(after.primary_layer_id, before.primary_layer_id);
        assert_eq!(after.projection_generation, before.projection_generation);
        assert_eq!(read_stage_pointer(stage).expect("pointer").sequence, 1);

        let _ = host_destroy_stage_for_test(stage);
        let _ = host_destroy_for_test(host);
    }

    #[test]
    fn stage_pointer_down_skips_degenerate_and_unavailable_layers() {
        let _lock = test_lock();
        let mut document = Document::new_current();
        let _degenerate = push_rect_layer(
            &mut document,
            "degen",
            [0.0, 0.0],
            [0.0, 0.5],
            Transform2D::identity(),
        );
        let group_id = document.layers.allocate("g").expect("group");
        document.tracks[0]
            .items
            .push(TrackItem::Group(motolii_doc::Group {
                envelope: ItemEnvelope::new(group_id),
                children: vec![],
            }));
        document.validate().expect("valid");
        let host = create_host_from_document("sel-skip", &document);
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
}
