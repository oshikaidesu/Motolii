//! Wave R0: product-private React Native Host seam.
//!
//! DocumentEditRuntime を単一 writer として保持し、revision 付き read-only snapshot と
//! lifecycle/read intent だけを RN へ投影する。

use std::collections::{HashMap, HashSet};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use motolii_doc::LayerId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::document_edit_runtime::DocumentEditRuntime;
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
}

struct RnProductHost {
    runtime: DocumentEditRuntime,
    projection_generation: u64,
    primary: Option<LayerId>,
    stages: HashMap<u64, RnStageSurface>,
    destroyed: bool,
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
                        stage.mounted = true;
                    }
                    "stage_resize" => {
                        stage.width = intent.width.expect("validated resize width");
                        stage.height = intent.height.expect("validated resize height");
                        stage.scale_factor =
                            intent.scale_factor.expect("validated resize scale factor");
                    }
                    "stage_focus" => {
                        stage.focused = intent.focused.expect("validated focus state");
                    }
                    "stage_unmount" => {
                        stage.mounted = false;
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
        Ok(())
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

fn snapshot_for_test(snapshot: WireProductSnapshot) -> RnProductSnapshotForTest {
    RnProductSnapshotForTest {
        revision: snapshot.revision,
        projection_generation: snapshot.projection_generation,
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
}
