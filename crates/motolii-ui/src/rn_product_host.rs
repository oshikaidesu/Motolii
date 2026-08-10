//! Wave R0: product-private React Native Host seam.
//!
//! DocumentEditRuntime を単一 writer として保持し、revision 付き read-only snapshot と
//! lifecycle/read intent だけを RN へ投影する。

use std::collections::{HashMap, HashSet, VecDeque};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use motolii_doc::LayerId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::browser_host_runtime::BrowserHostRuntime;
use crate::document_edit_runtime::DocumentEditRuntime;
use crate::host_pointer_capture::{
    HostPointerCancel, HostPointerCandidate, HostPointerCaptureState,
};
use crate::shell::{open_project_runtime, ShellError};

const WIRE_VERSION: u8 = 1;
const HOST_TO_RN: &str = "host-to-rn";
const RN_TO_HOST: &str = "rn-to-host";
const PRODUCT_ROLE: &str = "product-runtime-seat";
const MAX_STAGE_BOUNDS: usize = 16;
const MAX_STAGE_SELECTION: usize = 16;
const MAX_DIAGNOSTICS: usize = 8;
const MAX_JSON_BYTES: usize = 16_384;
const MAX_PROJECT_PATH_BYTES: usize = 4_096;
const MAX_BROWSER_SOURCE_ID_BYTES: usize = 64;
const MAX_BROWSER_CAPTURE_CANDIDATES: usize = 32;

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
    #[error("browser source identity does not match the host-issued source")]
    InvalidBrowserSource,
    #[error("browser capture position must contain finite logical coordinates")]
    InvalidBrowserCapturePosition,
    #[error("browser capture generation space exhausted")]
    BrowserCaptureGenerationExhausted,
    #[error("browser capture is already active")]
    BrowserCaptureAlreadyActive,
    #[error("browser capture generation is inactive or stale")]
    BrowserCaptureGenerationMismatch,
    #[error("browser capture candidate queue is full")]
    BrowserCaptureQueueFull,
    #[error("stage layout must contain finite coordinates and positive finite dimensions")]
    InvalidStageLayout,
    #[error("stage layout is unavailable while the mounted stage is ambiguous")]
    AmbiguousMountedStage,
    #[error("stage layout epoch space exhausted")]
    StageLayoutEpochExhausted,
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
struct WireBrowserSource {
    scope_ref: String,
    item_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct WireBrowserProjection {
    rectangle_source: WireBrowserSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct WireProductSnapshot {
    version: u8,
    direction: String,
    role: String,
    host_handle: String,
    revision: String,
    projection_generation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    primary_layer_id: Option<String>,
    browser: WireBrowserProjection,
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
    pub primary_layer_id: Option<String>,
    pub browser_rectangle_scope_ref: String,
    pub browser_rectangle_item_id: String,
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

#[derive(Debug)]
struct RnStageSurface {
    host_handle: u64,
    mounted: bool,
    destroyed: bool,
    width: u32,
    height: u32,
    scale_factor: f64,
    focused: bool,
    layout: Option<RnStageLayout>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct RnStageLayout {
    rect: [f64; 4],
    epoch: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg(test)]
pub struct RnStageLayoutForTest {
    pub rect: [f64; 4],
    pub epoch: u64,
}

struct RnProductHost {
    runtime: DocumentEditRuntime,
    projection_generation: u64,
    primary: Option<LayerId>,
    stages: HashMap<u64, RnStageSurface>,
    stage_layout_epoch: u64,
    next_browser_capture_generation: u64,
    browser_capture: HostPointerCaptureState,
    browser_candidates: VecDeque<HostPointerCandidate>,
    destroyed: bool,
}

impl RnProductHost {
    fn validate_browser_source(
        &self,
        host_handle: u64,
        scope_ref: &str,
        item_id: &str,
    ) -> Result<(), RnHostError> {
        let expected = BrowserHostRuntime::built_in_rectangle_source(host_handle);
        if scope_ref != expected.scope_ref || item_id != expected.item_id {
            return Err(RnHostError::InvalidBrowserSource);
        }
        Ok(())
    }

    fn begin_browser_capture(
        &mut self,
        host_handle: u64,
        scope_ref: &str,
        item_id: &str,
        position: [f64; 2],
    ) -> Result<u64, RnHostError> {
        self.validate_browser_source(host_handle, scope_ref, item_id)?;
        validate_browser_capture_position(position)?;
        if self.browser_capture.is_active() {
            return Err(RnHostError::BrowserCaptureAlreadyActive);
        }
        if self.browser_candidates.len() > MAX_BROWSER_CAPTURE_CANDIDATES.saturating_sub(2) {
            return Err(RnHostError::BrowserCaptureQueueFull);
        }
        let generation = self.next_browser_capture_generation;
        self.next_browser_capture_generation = generation
            .checked_add(1)
            .ok_or(RnHostError::BrowserCaptureGenerationExhausted)?;
        if !self.browser_capture.arm(generation, 0) {
            return Err(RnHostError::BrowserCaptureAlreadyActive);
        }
        Ok(generation)
    }

    fn move_browser_capture(
        &mut self,
        generation: u64,
        position: [f64; 2],
    ) -> Result<(), RnHostError> {
        validate_browser_capture_position(position)?;
        let candidate = self
            .browser_capture
            .move_for_generation(generation, position)
            .ok_or(RnHostError::BrowserCaptureGenerationMismatch)?;
        // 高頻度moveは最新一点だけを保持し、terminal candidateを欠落させない。
        if matches!(self.browser_candidates.back(), Some(HostPointerCandidate::Moved { generation: queued, .. }) if *queued == generation)
        {
            self.browser_candidates.pop_back();
        }
        if self.browser_candidates.len() >= MAX_BROWSER_CAPTURE_CANDIDATES {
            return Err(RnHostError::BrowserCaptureQueueFull);
        }
        self.browser_candidates.push_back(candidate);
        Ok(())
    }

    fn release_browser_capture(
        &mut self,
        generation: u64,
        position: [f64; 2],
    ) -> Result<(), RnHostError> {
        validate_browser_capture_position(position)?;
        if self.browser_candidates.len() >= MAX_BROWSER_CAPTURE_CANDIDATES {
            return Err(RnHostError::BrowserCaptureQueueFull);
        }
        let candidate = self
            .browser_capture
            .release_for_generation(generation, position)
            .ok_or(RnHostError::BrowserCaptureGenerationMismatch)?;
        self.browser_candidates.push_back(candidate);
        Ok(())
    }

    fn cancel_browser_capture(
        &mut self,
        generation: u64,
        reason: HostPointerCancel,
    ) -> Result<(), RnHostError> {
        if self.browser_candidates.len() >= MAX_BROWSER_CAPTURE_CANDIDATES {
            return Err(RnHostError::BrowserCaptureQueueFull);
        }
        let candidate = self
            .browser_capture
            .cancel_for_generation(generation, reason)
            .ok_or(RnHostError::BrowserCaptureGenerationMismatch)?;
        self.browser_candidates.push_back(candidate);
        Ok(())
    }

    fn snapshot_wire(&self, host_handle: u64) -> WireProductSnapshot {
        let document = self.runtime.snapshot();
        let rectangle_source = BrowserHostRuntime::built_in_rectangle_source(host_handle);
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
            primary_layer_id: self.primary.map(|layer| layer.get().to_string()),
            browser: WireBrowserProjection {
                rectangle_source: WireBrowserSource {
                    scope_ref: rectangle_source.scope_ref,
                    item_id: rectangle_source.item_id,
                },
            },
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
            "stage_mount" | "stage_resize" | "stage_focus" | "stage_unmount" => {
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
                let Some(stage) = self.stages.get(&stage_handle) else {
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
                match intent.kind.as_str() {
                    "stage_mount" => {
                        if !stage.mounted
                            && self
                                .stages
                                .values()
                                .any(|other| other.mounted && !other.destroyed)
                        {
                            for surface in self.stages.values_mut() {
                                surface.layout = None;
                            }
                        }
                        let stage = self
                            .stages
                            .get_mut(&stage_handle)
                            .expect("validated stage handle");
                        stage.mounted = true;
                    }
                    "stage_resize" => {
                        let stage = self
                            .stages
                            .get_mut(&stage_handle)
                            .expect("validated stage handle");
                        stage.width = intent.width.expect("validated resize width");
                        stage.height = intent.height.expect("validated resize height");
                        stage.scale_factor =
                            intent.scale_factor.expect("validated resize scale factor");
                    }
                    "stage_focus" => {
                        let stage = self
                            .stages
                            .get_mut(&stage_handle)
                            .expect("validated stage handle");
                        stage.focused = intent.focused.expect("validated focus state");
                    }
                    "stage_unmount" => {
                        let stage = self
                            .stages
                            .get_mut(&stage_handle)
                            .expect("validated stage handle");
                        stage.mounted = false;
                        stage.layout = None;
                    }
                    _ => {}
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
                layout: None,
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
        stage.destroyed = true;
        stage.mounted = false;
        stage.layout = None;
        Ok(())
    }

    fn register_stage_layout(
        &mut self,
        host_handle: u64,
        stage_handle: u64,
        rect: [f64; 4],
    ) -> Result<u64, RnHostError> {
        validate_stage_layout(rect)?;
        if self.destroyed {
            return Err(RnHostError::DestroyedHost(host_handle));
        }
        let Some(stage) = self.stages.get(&stage_handle) else {
            return Err(RnHostError::UnknownStage(stage_handle));
        };
        if stage.destroyed {
            return Err(RnHostError::DestroyedStage(stage_handle));
        }
        if !stage.mounted {
            return Err(RnHostError::InvalidStageLayout);
        }
        if self
            .stages
            .values()
            .filter(|surface| surface.mounted && !surface.destroyed)
            .count()
            != 1
        {
            return Err(RnHostError::AmbiguousMountedStage);
        }
        if let Some(layout) = stage.layout {
            if layout.rect == rect {
                return Ok(layout.epoch);
            }
        }
        let epoch = self
            .stage_layout_epoch
            .checked_add(1)
            .ok_or(RnHostError::StageLayoutEpochExhausted)?;
        self.stage_layout_epoch = epoch;
        self.stages
            .get_mut(&stage_handle)
            .expect("validated stage handle")
            .layout = Some(RnStageLayout { rect, epoch });
        Ok(epoch)
    }

    fn destroy(&mut self, host_handle: u64) -> Result<(), RnHostError> {
        if self.destroyed {
            return Err(RnHostError::DestroyedHost(host_handle));
        }
        self.destroyed = true;
        self.stages.clear();
        Ok(())
    }
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
                primary: None,
                stages: HashMap::new(),
                stage_layout_epoch: 0,
                next_browser_capture_generation: 1,
                browser_capture: HostPointerCaptureState::default(),
                browser_candidates: VecDeque::new(),
                destroyed: false,
            },
        );
        Ok(handle)
    }

    fn register_stage(&mut self, host_handle: u64) -> Result<u64, RnHostError> {
        let host = self
            .hosts
            .get_mut(&host_handle)
            .ok_or(RnHostError::UnknownHost(host_handle))?;
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

    fn register_stage_layout(
        &mut self,
        host_handle: u64,
        stage_handle: u64,
        rect: [f64; 4],
    ) -> Result<u64, RnHostError> {
        let host = self
            .hosts
            .get_mut(&host_handle)
            .ok_or(RnHostError::UnknownHost(host_handle))?;
        host.register_stage_layout(host_handle, stage_handle, rect)
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

    fn read_snapshot_json(&self, host_handle: u64) -> Result<String, RnHostError> {
        encode_json(&self.read_snapshot(host_handle)?)
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
        let intent: WireIntentEnvelope = serde_json::from_str(intent_json)?;
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

    fn begin_browser_capture(
        &mut self,
        host_handle: u64,
        scope_ref: &str,
        item_id: &str,
        position: [f64; 2],
    ) -> Result<u64, RnHostError> {
        let host = self
            .hosts
            .get_mut(&host_handle)
            .ok_or(RnHostError::UnknownHost(host_handle))?;
        host.begin_browser_capture(host_handle, scope_ref, item_id, position)
    }

    fn move_browser_capture(
        &mut self,
        host_handle: u64,
        generation: u64,
        position: [f64; 2],
    ) -> Result<(), RnHostError> {
        let host = self
            .hosts
            .get_mut(&host_handle)
            .ok_or(RnHostError::UnknownHost(host_handle))?;
        host.move_browser_capture(generation, position)
    }

    fn release_browser_capture(
        &mut self,
        host_handle: u64,
        generation: u64,
        position: [f64; 2],
    ) -> Result<(), RnHostError> {
        let host = self
            .hosts
            .get_mut(&host_handle)
            .ok_or(RnHostError::UnknownHost(host_handle))?;
        host.release_browser_capture(generation, position)
    }

    fn cancel_browser_capture(
        &mut self,
        host_handle: u64,
        generation: u64,
        reason: HostPointerCancel,
    ) -> Result<(), RnHostError> {
        let host = self
            .hosts
            .get_mut(&host_handle)
            .ok_or(RnHostError::UnknownHost(host_handle))?;
        host.cancel_browser_capture(generation, reason)
    }
}

fn validate_browser_capture_position(position: [f64; 2]) -> Result<(), RnHostError> {
    if position.into_iter().all(f64::is_finite) {
        Ok(())
    } else {
        Err(RnHostError::InvalidBrowserCapturePosition)
    }
}

fn validate_stage_layout(rect: [f64; 4]) -> Result<(), RnHostError> {
    if rect.into_iter().all(f64::is_finite) && rect[2] > 0.0 && rect[3] > 0.0 {
        Ok(())
    } else {
        Err(RnHostError::InvalidStageLayout)
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

fn browser_capture_begin(
    host_handle: u64,
    scope_ref: &str,
    item_id: &str,
    position: [f64; 2],
) -> Result<u64, RnHostError> {
    with_registry(|registry| {
        registry.begin_browser_capture(host_handle, scope_ref, item_id, position)
    })
}

fn browser_capture_move(
    host_handle: u64,
    generation: u64,
    position: [f64; 2],
) -> Result<(), RnHostError> {
    with_registry(|registry| registry.move_browser_capture(host_handle, generation, position))
}

fn browser_capture_release(
    host_handle: u64,
    generation: u64,
    position: [f64; 2],
) -> Result<(), RnHostError> {
    with_registry(|registry| registry.release_browser_capture(host_handle, generation, position))
}

fn browser_capture_cancel(
    host_handle: u64,
    generation: u64,
    reason: HostPointerCancel,
) -> Result<(), RnHostError> {
    with_registry(|registry| registry.cancel_browser_capture(host_handle, generation, reason))
}

fn snapshot_for_test(snapshot: WireProductSnapshot) -> RnProductSnapshotForTest {
    RnProductSnapshotForTest {
        revision: snapshot.revision,
        projection_generation: snapshot.projection_generation,
        primary_layer_id: snapshot.primary_layer_id,
        browser_rectangle_scope_ref: snapshot.browser.rectangle_source.scope_ref,
        browser_rectangle_item_id: snapshot.browser.rectangle_source.item_id,
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

#[cfg(test)]
pub fn host_register_stage_layout_for_test(
    host_handle: u64,
    stage_handle: u64,
    rect: [f64; 4],
) -> Result<u64, RnHostError> {
    with_registry(|registry| registry.register_stage_layout(host_handle, stage_handle, rect))
}

#[cfg(test)]
pub fn host_stage_layout_for_test(
    host_handle: u64,
    stage_handle: u64,
) -> Result<Option<RnStageLayoutForTest>, RnHostError> {
    with_registry(|registry| {
        let host = registry
            .hosts
            .get(&host_handle)
            .ok_or(RnHostError::UnknownHost(host_handle))?;
        let stage = host
            .stages
            .get(&stage_handle)
            .ok_or(RnHostError::UnknownStage(stage_handle))?;
        Ok(stage.layout.map(|layout| RnStageLayoutForTest {
            rect: layout.rect,
            epoch: layout.epoch,
        }))
    })
}

pub fn host_destroy_for_test(host_handle: u64) -> Result<(), RnHostError> {
    with_registry(|registry| registry.destroy_host(host_handle))
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_host_create(
    project_path_ptr: *const u8,
    project_path_len: usize,
) -> u64 {
    catch_unwind(AssertUnwindSafe(|| {
        let project_path = read_utf8(project_path_ptr, project_path_len, MAX_PROJECT_PATH_BYTES)?;
        if project_path.is_empty() {
            return Err(RnHostError::EmptyProjectPath);
        }
        with_registry(|registry| registry.create_host(Path::new(&project_path)))
    }))
    .ok()
    .and_then(Result::ok)
    .unwrap_or(0)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_host_destroy(host_handle: u64) -> bool {
    if host_handle == 0 {
        return false;
    }
    catch_unwind(AssertUnwindSafe(|| {
        with_registry(|registry| registry.destroy_host(host_handle)).is_ok()
    }))
    .unwrap_or(false)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_stage_register(host_handle: u64) -> u64 {
    if host_handle == 0 {
        return 0;
    }
    catch_unwind(AssertUnwindSafe(|| {
        with_registry(|registry| registry.register_stage(host_handle))
    }))
    .ok()
    .and_then(Result::ok)
    .unwrap_or(0)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_stage_destroy(stage_handle: u64) -> bool {
    if stage_handle == 0 {
        return false;
    }
    catch_unwind(AssertUnwindSafe(|| {
        with_registry(|registry| registry.destroy_stage(stage_handle)).is_ok()
    }))
    .unwrap_or(false)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_stage_register_layout(
    host_handle: u64,
    stage_handle: u64,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> u64 {
    if host_handle == 0 || stage_handle == 0 {
        return 0;
    }
    catch_unwind(AssertUnwindSafe(|| {
        with_registry(|registry| {
            registry.register_stage_layout(host_handle, stage_handle, [x, y, width, height])
        })
    }))
    .ok()
    .and_then(Result::ok)
    .unwrap_or(0)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_browser_capture_begin(
    host_handle: u64,
    scope_ref_ptr: *const u8,
    scope_ref_len: usize,
    item_id_ptr: *const u8,
    item_id_len: usize,
    x: f64,
    y: f64,
) -> u64 {
    if host_handle == 0 {
        return 0;
    }
    catch_unwind(AssertUnwindSafe(|| {
        let scope_ref = read_utf8(scope_ref_ptr, scope_ref_len, MAX_BROWSER_SOURCE_ID_BYTES)?;
        let item_id = read_utf8(item_id_ptr, item_id_len, MAX_BROWSER_SOURCE_ID_BYTES)?;
        browser_capture_begin(host_handle, &scope_ref, &item_id, [x, y])
    }))
    .ok()
    .and_then(Result::ok)
    .unwrap_or(0)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_browser_capture_move(
    host_handle: u64,
    generation: u64,
    x: f64,
    y: f64,
) -> bool {
    if host_handle == 0 || generation == 0 {
        return false;
    }
    catch_unwind(AssertUnwindSafe(|| {
        browser_capture_move(host_handle, generation, [x, y]).is_ok()
    }))
    .unwrap_or(false)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_browser_capture_release(
    host_handle: u64,
    generation: u64,
    x: f64,
    y: f64,
) -> bool {
    if host_handle == 0 || generation == 0 {
        return false;
    }
    catch_unwind(AssertUnwindSafe(|| {
        browser_capture_release(host_handle, generation, [x, y]).is_ok()
    }))
    .unwrap_or(false)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_browser_capture_cancel(
    host_handle: u64,
    generation: u64,
    reason: u8,
) -> bool {
    if host_handle == 0 || generation == 0 {
        return false;
    }
    let reason = match reason {
        1 => HostPointerCancel::Escape,
        2 => HostPointerCancel::CaptureLost,
        _ => return false,
    };
    catch_unwind(AssertUnwindSafe(|| {
        browser_capture_cancel(host_handle, generation, reason).is_ok()
    }))
    .unwrap_or(false)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rn_host_read_snapshot_json(
    host_handle: u64,
    out: *mut u8,
    out_cap: usize,
) -> i64 {
    if host_handle == 0 {
        return -1;
    }
    catch_unwind(AssertUnwindSafe(|| {
        let json = with_registry(|registry| registry.read_snapshot_json(host_handle))?;
        Ok::<i64, RnHostError>(write_bytes(out, out_cap, &json))
    }))
    .ok()
    .and_then(Result::ok)
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
    if host_handle == 0 {
        return -1;
    }
    catch_unwind(AssertUnwindSafe(|| {
        let intent_json = read_utf8(intent_ptr, intent_len, MAX_JSON_BYTES)?;
        let response =
            with_registry(|registry| registry.dispatch_intent_json(host_handle, &intent_json))?;
        Ok::<i64, RnHostError>(write_bytes(out, out_cap, &response))
    }))
    .ok()
    .and_then(Result::ok)
    .unwrap_or(-1)
}

#[cfg(target_os = "macos")]
#[allow(dead_code)]
const _: fn() = || {
    let _ = motolii_rn_host_create as extern "C" fn(*const u8, usize) -> u64;
    let _ = motolii_rn_host_destroy as extern "C" fn(u64) -> bool;
    let _ = motolii_rn_stage_register as extern "C" fn(u64) -> u64;
    let _ = motolii_rn_stage_destroy as extern "C" fn(u64) -> bool;
    let _ = motolii_rn_stage_register_layout as extern "C" fn(u64, u64, f64, f64, f64, f64) -> u64;
    let _ = motolii_rn_browser_capture_begin
        as extern "C" fn(u64, *const u8, usize, *const u8, usize, f64, f64) -> u64;
    let _ = motolii_rn_browser_capture_move as extern "C" fn(u64, u64, f64, f64) -> bool;
    let _ = motolii_rn_browser_capture_release as extern "C" fn(u64, u64, f64, f64) -> bool;
    let _ = motolii_rn_browser_capture_cancel as extern "C" fn(u64, u64, u8) -> bool;
    let _ = motolii_rn_host_read_snapshot_json as extern "C" fn(u64, *mut u8, usize) -> i64;
    let _ = motolii_rn_host_dispatch_intent_json
        as extern "C" fn(u64, *const u8, usize, *mut u8, usize) -> i64;
};

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Mutex, MutexGuard};

    use motolii_core::{RationalTime, TimeMap};
    use motolii_doc::{
        Clip, ClipSource, DocParam, Document, ItemEnvelope, ProjectSession, ResourceLimits,
        SaveProjectOptions, Track, TrackItem, RECT_LAYER_SOURCE,
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

    #[test]
    fn snapshot_carries_revision_projection_generation_and_primary_layer_id() {
        let _lock = test_lock();
        let host = create_host("snapshot");
        let snapshot = read_snapshot(host);
        assert_eq!(snapshot.revision, "0");
        assert_eq!(snapshot.projection_generation, "0");
        assert!(snapshot.primary_layer_id.is_none());
        assert!(!snapshot.layer_ids.is_empty());
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
    fn stage_layout_registration_assigns_host_epoch_only_for_changed_valid_rects() {
        let _lock = test_lock();
        let host = create_host("stage-layout-epoch");
        let baseline = read_snapshot(host);
        let stage = host_register_stage_for_test(host).expect("stage");
        let mut mount = base_intent("stage_mount");
        mount.stage_handle = Some(stage);
        assert!(dispatch(host, mount).accepted);

        let first = host_register_stage_layout_for_test(host, stage, [120.0, 48.0, 640.0, 360.0])
            .expect("first layout");
        assert_eq!(first, 1);
        assert_eq!(
            host_stage_layout_for_test(host, stage).expect("layout"),
            Some(RnStageLayoutForTest {
                rect: [120.0, 48.0, 640.0, 360.0],
                epoch: first,
            })
        );
        assert_eq!(
            host_register_stage_layout_for_test(host, stage, [120.0, 48.0, 640.0, 360.0],)
                .expect("duplicate layout"),
            first
        );
        let changed = host_register_stage_layout_for_test(host, stage, [121.0, 48.0, 640.0, 360.0])
            .expect("changed layout");
        assert_eq!(changed, first + 1);

        let after = read_snapshot(host);
        assert_eq!(after, baseline);
        with_registry(|registry| {
            assert!(registry
                .hosts
                .get(&host)
                .expect("host")
                .browser_candidates
                .is_empty());
            Ok(())
        })
        .expect("registry");

        host_destroy_for_test(host).expect("destroy host");
    }

    #[test]
    fn stage_layout_rejects_invalid_unknown_unmounted_and_destroyed_without_write() {
        let _lock = test_lock();
        let host = create_host("stage-layout-reject");
        let baseline = read_snapshot(host);
        let stage = host_register_stage_for_test(host).expect("stage");

        assert!(matches!(
            host_register_stage_layout_for_test(host, stage, [0.0, 0.0, 10.0, 10.0]),
            Err(RnHostError::InvalidStageLayout)
        ));
        assert!(matches!(
            host_register_stage_layout_for_test(host, 99_999, [0.0, 0.0, 10.0, 10.0]),
            Err(RnHostError::UnknownStage(_))
        ));

        let mut mount = base_intent("stage_mount");
        mount.stage_handle = Some(stage);
        assert!(dispatch(host, mount).accepted);
        for invalid in [
            [f64::NAN, 0.0, 10.0, 10.0],
            [0.0, f64::INFINITY, 10.0, 10.0],
            [0.0, 0.0, 0.0, 10.0],
            [0.0, 0.0, 10.0, -1.0],
        ] {
            assert!(matches!(
                host_register_stage_layout_for_test(host, stage, invalid),
                Err(RnHostError::InvalidStageLayout)
            ));
        }
        assert!(host_stage_layout_for_test(host, stage)
            .expect("layout")
            .is_none());

        host_destroy_stage_for_test(stage).expect("destroy stage");
        assert!(matches!(
            host_register_stage_layout_for_test(host, stage, [0.0, 0.0, 10.0, 10.0]),
            Err(RnHostError::DestroyedStage(_))
        ));
        assert_eq!(read_snapshot(host), baseline);
        host_destroy_for_test(host).expect("destroy host");
    }

    #[test]
    fn stage_unmount_invalidates_layout_and_remount_requires_a_new_epoch() {
        let _lock = test_lock();
        let host = create_host("stage-layout-remount");
        let baseline = read_snapshot(host);
        let stage = host_register_stage_for_test(host).expect("stage");
        let mut mount = base_intent("stage_mount");
        mount.stage_handle = Some(stage);
        assert!(dispatch(host, mount.clone()).accepted);
        let first = host_register_stage_layout_for_test(host, stage, [1.0, 2.0, 3.0, 4.0])
            .expect("first layout");

        let mut unmount = base_intent("stage_unmount");
        unmount.stage_handle = Some(stage);
        assert!(dispatch(host, unmount).accepted);
        assert!(host_stage_layout_for_test(host, stage)
            .expect("invalidated layout")
            .is_none());
        assert!(matches!(
            host_register_stage_layout_for_test(host, stage, [1.0, 2.0, 3.0, 4.0]),
            Err(RnHostError::InvalidStageLayout)
        ));

        assert!(dispatch(host, mount).accepted);
        let remounted = host_register_stage_layout_for_test(host, stage, [1.0, 2.0, 3.0, 4.0])
            .expect("remounted layout");
        assert!(remounted > first);
        assert_eq!(read_snapshot(host), baseline);

        host_destroy_for_test(host).expect("destroy host");
    }

    #[test]
    fn multiple_mounted_stages_invalidate_and_reject_layout_as_ambiguous() {
        let _lock = test_lock();
        let host = create_host("stage-layout-ambiguous");
        let baseline = read_snapshot(host);
        let first_stage = host_register_stage_for_test(host).expect("first stage");
        let second_stage = host_register_stage_for_test(host).expect("second stage");
        let mut first_mount = base_intent("stage_mount");
        first_mount.stage_handle = Some(first_stage);
        assert!(dispatch(host, first_mount).accepted);
        host_register_stage_layout_for_test(host, first_stage, [1.0, 2.0, 3.0, 4.0])
            .expect("first layout");

        let mut second_mount = base_intent("stage_mount");
        second_mount.stage_handle = Some(second_stage);
        assert!(dispatch(host, second_mount).accepted);
        assert!(host_stage_layout_for_test(host, first_stage)
            .expect("invalidated first layout")
            .is_none());
        assert!(matches!(
            host_register_stage_layout_for_test(host, first_stage, [1.0, 2.0, 3.0, 4.0]),
            Err(RnHostError::AmbiguousMountedStage)
        ));
        assert!(matches!(
            host_register_stage_layout_for_test(host, second_stage, [5.0, 6.0, 7.0, 8.0]),
            Err(RnHostError::AmbiguousMountedStage)
        ));
        assert_eq!(read_snapshot(host), baseline);

        host_destroy_for_test(host).expect("destroy host");
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

    #[test]
    fn browser_capture_release_is_generation_bound_and_zero_write() {
        let _lock = test_lock();
        let path = fixture_path("browser-capture-release");
        let project_before = std::fs::read(&path).expect("project bytes");
        let host = host_create_for_test(&path).expect("host");
        let baseline = read_snapshot(host);
        let scope_ref = format!("builtin-{host}");

        let generation =
            browser_capture_begin(host, &scope_ref, "rectangle", [10.0, 20.0]).expect("arm");
        assert_eq!(generation, 1);
        assert!(matches!(
            browser_capture_begin(host, &scope_ref, "rectangle", [11.0, 21.0]),
            Err(RnHostError::BrowserCaptureAlreadyActive)
        ));
        browser_capture_move(host, generation, [12.0, 22.0]).expect("first move");
        browser_capture_move(host, generation, [13.0, 23.0]).expect("coalesced move");
        browser_capture_release(host, generation, [14.0, 24.0]).expect("release");
        assert!(matches!(
            browser_capture_release(host, generation, [15.0, 25.0]),
            Err(RnHostError::BrowserCaptureGenerationMismatch)
        ));
        assert!(matches!(
            browser_capture_move(host, generation, [16.0, 26.0]),
            Err(RnHostError::BrowserCaptureGenerationMismatch)
        ));

        let candidates = with_registry(|registry| {
            let host = registry.hosts.get_mut(&host).expect("host state");
            Ok(host.browser_candidates.drain(..).collect::<Vec<_>>())
        })
        .expect("candidate drain");
        assert_eq!(
            candidates,
            vec![
                HostPointerCandidate::Moved {
                    generation,
                    position: [13.0, 23.0],
                },
                HostPointerCandidate::Released {
                    generation,
                    position: [14.0, 24.0],
                },
            ]
        );
        assert_eq!(read_snapshot(host), baseline);
        host_destroy_for_test(host).expect("destroy host");
        assert_eq!(std::fs::read(path).expect("project after"), project_before);
    }

    #[test]
    fn browser_capture_rejects_wrong_identity_stale_and_late_events() {
        let _lock = test_lock();
        let path = fixture_path("browser-capture-cancel");
        let project_before = std::fs::read(&path).expect("project bytes");
        let host = host_create_for_test(&path).expect("host");
        let baseline = read_snapshot(host);
        let scope_ref = format!("builtin-{host}");

        assert!(matches!(
            browser_capture_begin(host, "builtin-999", "rectangle", [1.0, 2.0]),
            Err(RnHostError::InvalidBrowserSource)
        ));
        assert!(matches!(
            browser_capture_begin(host, &scope_ref, "ellipse", [1.0, 2.0]),
            Err(RnHostError::InvalidBrowserSource)
        ));
        assert!(matches!(
            browser_capture_begin(host, &scope_ref, "rectangle", [f64::NAN, 2.0]),
            Err(RnHostError::InvalidBrowserCapturePosition)
        ));
        assert!(matches!(
            browser_capture_begin(99_999, "builtin-99999", "rectangle", [1.0, 2.0]),
            Err(RnHostError::UnknownHost(99_999))
        ));

        let generation =
            browser_capture_begin(host, &scope_ref, "rectangle", [1.0, 2.0]).expect("arm");
        for result in [
            browser_capture_move(host, generation + 1, [2.0, 3.0]),
            browser_capture_release(host, generation + 1, [2.0, 3.0]),
            browser_capture_cancel(host, generation + 1, HostPointerCancel::Escape),
        ] {
            assert!(matches!(
                result,
                Err(RnHostError::BrowserCaptureGenerationMismatch)
            ));
        }
        browser_capture_cancel(host, generation, HostPointerCancel::Escape).expect("cancel");
        assert!(matches!(
            browser_capture_cancel(host, generation, HostPointerCancel::CaptureLost),
            Err(RnHostError::BrowserCaptureGenerationMismatch)
        ));

        let candidates = with_registry(|registry| {
            let host = registry.hosts.get_mut(&host).expect("host state");
            Ok(host.browser_candidates.drain(..).collect::<Vec<_>>())
        })
        .expect("candidate drain");
        assert_eq!(
            candidates,
            vec![HostPointerCandidate::Cancelled {
                generation,
                reason: HostPointerCancel::Escape,
            }]
        );
        assert_eq!(read_snapshot(host), baseline);
        host_destroy_for_test(host).expect("destroy host");
        assert!(matches!(
            browser_capture_move(host, generation, [3.0, 4.0]),
            Err(RnHostError::UnknownHost(_))
        ));
        assert_eq!(std::fs::read(path).expect("project after"), project_before);
    }

    #[test]
    fn browser_capture_fails_closed_before_candidate_capacity_is_exhausted() {
        let _lock = test_lock();
        let path = fixture_path("browser-capture-capacity");
        let project_before = std::fs::read(&path).expect("project bytes");
        let host = host_create_for_test(&path).expect("host");
        let baseline = read_snapshot(host);
        let scope_ref = format!("builtin-{host}");

        for index in 0..(MAX_BROWSER_CAPTURE_CANDIDATES / 2) {
            let generation =
                browser_capture_begin(host, &scope_ref, "rectangle", [index as f64, 1.0])
                    .expect("capacity-reserved arm");
            browser_capture_move(host, generation, [index as f64, 2.0]).expect("move");
            browser_capture_release(host, generation, [index as f64, 3.0]).expect("release");
        }
        assert!(matches!(
            browser_capture_begin(host, &scope_ref, "rectangle", [20.0, 20.0]),
            Err(RnHostError::BrowserCaptureQueueFull)
        ));
        let host_state_is_inactive = with_registry(|registry| {
            Ok(!registry
                .hosts
                .get(&host)
                .expect("host state")
                .browser_capture
                .is_active())
        })
        .expect("capture state");
        assert!(host_state_is_inactive);
        assert_eq!(read_snapshot(host), baseline);
        host_destroy_for_test(host).expect("destroy host");
        assert_eq!(std::fs::read(path).expect("project after"), project_before);
    }
}
