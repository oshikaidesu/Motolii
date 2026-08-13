//! RN app ↔ RnProductHost 接続の単一owner。
//!
//! processに最大1 host。ObjC/RNは薄いcarrierとしてここへ委譲する。

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

const MAX_JSON_BYTES: usize = 16_384;
const MAX_SNAPSHOT_JSON_BYTES: usize = 131_072;

#[cfg(test)]
use std::sync::atomic::AtomicU64;

#[cfg(test)]
static TEST_SELECTION_DISPATCH_COUNT: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
static TEST_KEYMAP_REMOVE_POSITION_KEY_COUNT: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
static TEST_KEYMAP_DELETE_LAYER_COUNT: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
static TEST_MOVE_LAYER_BY_DISPATCH_COUNT: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
static TEST_SNAPSHOT_READ_COUNT: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
static TEST_SET_POSITION_KEY_TIME_DISPATCH_COUNT: AtomicU64 = AtomicU64::new(0);

// extern importではなくRust経由で呼ぶ。externで宣言すると同一crate graph内でも
// motolii-uiの該当objectがarchiveから引かれず、appのlinkで未解決symbolになる(実測)。
#[cfg(target_os = "macos")]
use motolii_ui::{
    motolii_rn_host_create, motolii_rn_host_dispatch_intent_json, motolii_rn_host_projection_stamp,
    motolii_rn_host_read_snapshot_json, motolii_rn_stage_register,
};

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn motolii_rn_host_destroy(host_handle: u64, out: *mut u8, out_cap: usize) -> i64;
}

use motolii_ui::{
    AppStageGeometry, AppStageTransformEdit, AsciiKey, EffectiveTrigger, InputPhase, KeyToken,
    Modifier, Modifiers, PlatformCommandModifier, ProductAction, builtin_command_registry,
    default_user_keymap_override_path, host_commit_stage_transform_for_app,
    host_preview_stage_transform_for_app, load_user_keymap_override, product_action_host_kind,
    product_builtin_keymap, resolve_product_action,
};

struct HostSlot {
    handle: u64,
    /// 同じlive Document writerへ再接続できるproject identity。
    project_path: PathBuf,
    /// processに1つだけのStage seat。register後に埋まる。
    stage_handle: Option<u64>,
    /// stage_pointer がdown状態かどうか（Rust内状態機械）。
    stage_pointer_active: bool,
    /// stage seat が現在mount状態か。
    stage_mounted: bool,
    /// stage_pointer の単調 sequence（bridge内部採番）。
    pointer_sequence: u64,
    /// 直近の mount/resize 論理寸法（move閾値の logical px 換算用）。
    stage_logical_width: f64,
    stage_logical_height: f64,
}

static TIMELINE_INTERACTING: AtomicBool = AtomicBool::new(false);
static IME_PREEDIT: AtomicBool = AtomicBool::new(false);

fn host_slot() -> &'static Mutex<Option<HostSlot>> {
    static SLOT: OnceLock<Mutex<Option<HostSlot>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

/// Host生成に失敗したprocessでも、RN操作へcoreのtyped rejectを返す。
#[cfg(target_os = "macos")]
fn host_startup_reject() -> &'static Mutex<Option<String>> {
    static REJECT: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    REJECT.get_or_init(|| Mutex::new(None))
}

pub(crate) fn is_timeline_interacting() -> bool {
    TIMELINE_INTERACTING.load(Ordering::Acquire)
}

pub(crate) fn set_timeline_interacting(interacting: bool) {
    TIMELINE_INTERACTING.store(interacting, Ordering::Release);
}

/// Host投影。revision変化時だけTimelineへ適用する。
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostTimelineProjection {
    pub revision: String,
    pub projection_generation: String,
    pub primary_layer_id: Option<String>,
    /// wire `current_time` の {num,den}。欠落時は0/1。
    pub current_time: (i64, i64),
    /// wire `timeline.duration` の {num,den}。
    pub timeline_duration: Option<(i64, i64)>,
    /// wire `timeline.fps`。timeline欠落時はNone。
    pub fps: Option<(i64, i64)>,
    pub bounds: Vec<(String, String)>,
    /// wire `timeline` がある時だけ。欠落時は旧host互換fallback。
    pub timeline_layers: Option<Vec<HostTimelineLayer>>,
    /// wire `stage_geometry`。欠落・壊れている時はNone（timeline投影は落とさない）。
    pub stage_geometry: Option<HostStageGeometry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HostTerminalDiagnostic {
    pub reason: String,
    pub host_handle: Option<String>,
    pub stage_handle: Option<String>,
    pub timeline_handle: Option<String>,
    pub expected_projection_generation: Option<String>,
    pub actual_projection_generation: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostTerminalResult {
    pub accepted: bool,
    pub diagnostics: Vec<HostTerminalDiagnostic>,
    pub message: Option<String>,
    pub projection: Option<HostTimelineProjection>,
}

impl HostTerminalResult {
    pub(crate) fn stamp(&self) -> Option<(u64, u64)> {
        let projection = self.projection.as_ref()?;
        Some((
            projection.revision.parse().ok()?,
            projection.projection_generation.parse().ok()?,
        ))
    }

    pub(crate) fn feedback(&self) -> Option<&str> {
        self.message.as_deref().or_else(|| {
            self.diagnostics
                .first()
                .map(|diagnostic| diagnostic.reason.as_str())
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostCatalogProjection {
    pub effects: Vec<HostCatalogEffect>,
    pub sources: Vec<HostCatalogSource>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostCatalogEffect {
    pub plugin_id: String,
    pub name: String,
    pub effect_version: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostCatalogSource {
    pub plugin_id: String,
    pub name: String,
    pub effect_version: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostStageGeometry {
    pub layers: Vec<HostStageGeometryLayer>,
    pub layers_truncated: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostStageGeometryLayer {
    pub layer_id: String,
    pub corners: [[f64; 2]; 4],
    pub position: [f64; 2],
    pub rotation: f64,
    pub scale: [f64; 2],
}

impl HostStageGeometryLayer {
    pub(crate) fn from_corners(layer_id: impl Into<String>, corners: [[f64; 2]; 4]) -> Self {
        Self {
            layer_id: layer_id.into(),
            corners,
            position: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
        }
    }
}

impl From<AppStageGeometry> for HostStageGeometry {
    fn from(geometry: AppStageGeometry) -> Self {
        Self {
            layers: geometry
                .layers
                .into_iter()
                .map(|layer| HostStageGeometryLayer {
                    layer_id: layer.layer_id,
                    corners: layer.corners,
                    position: layer.position,
                    rotation: layer.rotation,
                    scale: layer.scale,
                })
                .collect(),
            layers_truncated: geometry.layers_truncated,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostTimelineLayer {
    pub layer_id: String,
    pub display_name: String,
    pub start_secs: f64,
    pub duration_secs: f64,
    pub position_keys: Vec<HostTimelineKey>,
    /// wire `param_keys`。欠落は空。scene keys へ position_keys と union。
    pub param_keys: Vec<HostTimelineKey>,
    pub effects: Vec<HostTimelineEffect>,
    pub effects_truncated: bool,
    pub source_params: Vec<HostTimelineSourceParam>,
    pub source_params_truncated: bool,
    pub visible: bool,
    pub solo: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostTimelineEffect {
    pub effect_use_id: String,
    pub plugin_id: String,
    pub params: Vec<HostTimelineEffectParam>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostTimelineEffectParam {
    pub param_id: String,
    pub value: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostTimelineSourceParam {
    pub param_id: String,
    pub value: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostTimelineKey {
    pub key_id: u64,
    pub time_secs: f64,
    /// wire `value`。[f64;2] がある時だけ。sceneには載せない。
    pub value: Option<[f64; 2]>,
}

/// 欠落documentを開ける最小projectでseedする。
/// `Document::new_current()` だけだと place_rectangle が process_next で落ちるため、
/// 空track 1本のseedを置く。
fn ensure_project_document(path: &Path) -> bool {
    if path.exists() {
        return true;
    }
    if let Some(parent) = path.parent() {
        if std::fs::create_dir_all(parent).is_err() {
            return false;
        }
    }
    use motolii_doc::{Document, ProjectSession, ResourceLimits, SaveProjectOptions, Track};

    let mut document = Document::new_current();
    let Ok(track) = document.track_ids.allocate("seed-track") else {
        return false;
    };
    document.tracks.push(Track {
        id: track,
        items: vec![],
    });
    if document.validate().is_err() {
        return false;
    }
    let limits = ResourceLimits::production();
    let Ok(mut session) = ProjectSession::acquire(path, &limits) else {
        return false;
    };
    session
        .save_with_journal(
            &document,
            &SaveProjectOptions {
                limits,
                checkpoint: true,
                ..SaveProjectOptions::default()
            },
        )
        .is_ok()
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn motolii_rnapp_host_ensure(path_utf8: *const u8, path_len: usize) -> bool {
    if path_utf8.is_null() || path_len == 0 || path_len > 4096 {
        return false;
    }
    let Ok(path) = std::str::from_utf8(unsafe { std::slice::from_raw_parts(path_utf8, path_len) })
    else {
        return false;
    };
    let project_path = Path::new(path);
    let Ok(mut guard) = host_slot().lock() else {
        return false;
    };
    if let Some(slot) = guard.as_ref() {
        return slot.project_path == project_path;
    }
    if !ensure_project_document(project_path) {
        return false;
    }
    let path_bytes = path.as_bytes();
    let mut host_handle = 0u64;
    let mut out = [0u8; MAX_SNAPSHOT_JSON_BYTES];
    let written = unsafe {
        motolii_rn_host_create(
            path_bytes.as_ptr(),
            path_bytes.len(),
            &mut host_handle,
            out.as_mut_ptr(),
            out.len(),
        )
    };
    if written <= 0 || host_handle == 0 {
        if let Some(response) = slice_from_written(&out, written)
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
            .filter(|response| !response_is_accepted(response))
        {
            if let Ok(mut reject) = host_startup_reject().lock() {
                *reject = Some(response.to_owned());
            }
        }
        return false;
    }
    if let Ok(mut reject) = host_startup_reject().lock() {
        *reject = None;
    }
    *guard = Some(HostSlot {
        handle: host_handle,
        project_path: project_path.to_owned(),
        stage_handle: None,
        stage_pointer_active: false,
        stage_mounted: false,
        pointer_sequence: 0,
        stage_logical_width: 0.0,
        stage_logical_height: 0.0,
    });
    true
}

#[cfg(target_os = "macos")]
fn try_host_shutdown() -> bool {
    let Ok(mut guard) = host_slot().lock() else {
        return false;
    };
    let Some(slot) = guard.as_ref() else {
        return true;
    };
    let mut out = [0u8; MAX_SNAPSHOT_JSON_BYTES];
    let written = unsafe { motolii_rn_host_destroy(slot.handle, out.as_mut_ptr(), out.len()) };
    let Some(response) =
        slice_from_written(&out, written).and_then(|bytes| std::str::from_utf8(bytes).ok())
    else {
        return false;
    };
    if !response_is_accepted(response) {
        return false;
    }
    *guard = None;
    true
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_rnapp_host_shutdown() -> bool {
    try_host_shutdown()
}

/// Host投影のStage seatを1つだけregisterする。既登録はreuse。
#[cfg(target_os = "macos")]
fn ensure_stage_registered(slot: &mut HostSlot) -> bool {
    if slot.stage_handle.is_some() {
        return true;
    }
    let mut stage_handle = 0u64;
    let mut out = [0u8; MAX_SNAPSHOT_JSON_BYTES];
    let written = unsafe {
        motolii_rn_stage_register(slot.handle, &mut stage_handle, out.as_mut_ptr(), out.len())
    };
    if written <= 0 || stage_handle == 0 {
        return false;
    }
    let Some(response_bytes) = slice_from_written(&out, written) else {
        return false;
    };
    let Ok(response) = std::str::from_utf8(response_bytes) else {
        return false;
    };
    if !response_is_accepted(response) {
        return false;
    }
    slot.stage_handle = Some(stage_handle);
    true
}

#[cfg(target_os = "macos")]
fn dispatch_stage_intent(slot: &HostSlot, kind: &str, extra: &str) -> bool {
    let Some(stage) = slot.stage_handle else {
        return false;
    };
    let intent = format!(
        r#"{{"version":1,"direction":"rn-to-host","kind":"{kind}","host_handle":"{}","stage_handle":"{stage}"{extra}}}"#,
        slot.handle,
    );
    if intent.len() > MAX_JSON_BYTES {
        return false;
    }
    let mut out = [0u8; MAX_SNAPSHOT_JSON_BYTES];
    let written = unsafe {
        motolii_rn_host_dispatch_intent_json(
            slot.handle,
            intent.as_ptr(),
            intent.len(),
            out.as_mut_ptr(),
            out.len(),
        )
    };
    if written <= 0 {
        return false;
    }
    let Some(response_bytes) = slice_from_written(&out, written) else {
        return false;
    };
    let Ok(response) = std::str::from_utf8(response_bytes) else {
        return false;
    };
    response_is_accepted(response)
}

/// logical viewport 寸法で mount→resize。host不在・拒否はfalse。
#[cfg(target_os = "macos")]
pub(crate) fn try_stage_mount(width: f64, height: f64, scale_factor: f64) -> bool {
    if !(width > 0.0 && height > 0.0 && scale_factor.is_finite() && scale_factor > 0.0) {
        return false;
    }
    let Ok(mut guard) = host_slot().lock() else {
        return false;
    };
    let Some(slot) = guard.as_mut() else {
        return false;
    };
    if !ensure_stage_registered(slot) {
        return false;
    }
    if !dispatch_stage_intent(slot, "stage_mount", "") {
        return false;
    }
    slot.stage_mounted = true;
    slot.stage_pointer_active = false;
    slot.stage_logical_width = width;
    slot.stage_logical_height = height;
    let w = width.round() as u32;
    let h = height.round() as u32;
    if w == 0 || h == 0 {
        return false;
    }
    let extra = format!(r#","width":{w},"height":{h},"scale_factor":{scale_factor}"#);
    dispatch_stage_intent(slot, "stage_resize", &extra)
}

#[cfg(target_os = "macos")]
pub(crate) fn try_stage_resize(width: f64, height: f64, scale_factor: f64) -> bool {
    if !(width > 0.0 && height > 0.0 && scale_factor.is_finite() && scale_factor > 0.0) {
        return false;
    }
    let Ok(mut guard) = host_slot().lock() else {
        return false;
    };
    let Some(slot) = guard.as_mut() else {
        return false;
    };
    if slot.stage_handle.is_none() {
        if !ensure_stage_registered(slot) {
            return false;
        }
    }
    if !slot.stage_mounted {
        if !dispatch_stage_intent(slot, "stage_mount", "") {
            return false;
        }
        slot.stage_mounted = true;
        slot.stage_pointer_active = false;
    }
    slot.stage_logical_width = width;
    slot.stage_logical_height = height;
    let w = width.round() as u32;
    let h = height.round() as u32;
    if w == 0 || h == 0 {
        return false;
    }
    let extra = format!(r#","width":{w},"height":{h},"scale_factor":{scale_factor}"#);
    dispatch_stage_intent(slot, "stage_resize", &extra)
}

#[cfg(target_os = "macos")]
pub(crate) fn try_stage_unmount() -> bool {
    let Ok(mut guard) = host_slot().lock() else {
        return false;
    };
    let Some(slot) = guard.as_mut() else {
        return false;
    };
    if slot.stage_handle.is_none() {
        return false;
    }
    if !dispatch_stage_intent(slot, "stage_unmount", "") {
        return false;
    }
    slot.stage_mounted = false;
    slot.stage_pointer_active = false;
    true
}

/// view-local logical 座標で stage_pointer。accepted:true のみ成功。
#[cfg(target_os = "macos")]
pub(crate) fn try_stage_pointer(phase: &str, view_local_x: f64, view_local_y: f64) -> bool {
    if !matches!(phase, "down" | "drag" | "up" | "cancel") {
        return false;
    }
    if !(view_local_x.is_finite() && view_local_y.is_finite()) {
        return false;
    }
    let Ok(mut guard) = host_slot().lock() else {
        return false;
    };
    let Some(slot) = guard.as_mut() else {
        return false;
    };
    if slot.stage_handle.is_none() {
        return false;
    }
    match phase {
        "down" => {
            if slot.stage_pointer_active {
                return false;
            }
            slot.stage_pointer_active = true;
        }
        "drag" | "up" | "cancel" => {
            if !slot.stage_pointer_active {
                return false;
            }
        }
        _ => {
            return false;
        }
    }
    slot.pointer_sequence = slot.pointer_sequence.saturating_add(1);
    let sequence = slot.pointer_sequence;
    let extra = format!(
        r#","phase":"{phase}","view_local_x":{view_local_x},"view_local_y":{view_local_y},"sequence":{sequence}"#
    );
    let accepted = dispatch_stage_intent(slot, "stage_pointer", &extra);
    if !accepted && phase == "down" {
        slot.stage_pointer_active = false;
    }
    if accepted && (phase == "up" || phase == "cancel") {
        slot.stage_pointer_active = false;
    }
    accepted
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
/// # Safety
/// No pointer input required; returns current timeline scrub/interact status.
pub unsafe extern "C" fn motolii_rnapp_is_timeline_interacting() -> bool {
    is_timeline_interacting()
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn motolii_rnapp_stage_mount(
    width: f64,
    height: f64,
    scale_factor: f64,
) -> bool {
    try_stage_mount(width, height, scale_factor)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn motolii_rnapp_stage_resize(
    width: f64,
    height: f64,
    scale_factor: f64,
) -> bool {
    try_stage_resize(width, height, scale_factor)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn motolii_rnapp_stage_unmount() -> bool {
    try_stage_unmount()
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
/// # Safety
/// `phase` は NUL 終端 UTF-8（"down"|"drag"|"up"|"cancel"）。
pub unsafe extern "C" fn motolii_rnapp_stage_pointer(
    phase: *const std::ffi::c_char,
    view_local_x: f64,
    view_local_y: f64,
) -> bool {
    if phase.is_null() {
        return false;
    }
    let Ok(phase) = (unsafe { std::ffi::CStr::from_ptr(phase) }).to_str() else {
        return false;
    };
    try_stage_pointer(phase, view_local_x, view_local_y)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
/// # Safety
/// `kind_utf8` must point to `kind_len` UTF-8 bytes naming undo/redo/delete_layer/toggle_playback.
pub unsafe extern "C" fn motolii_rnapp_host_keymap(kind_utf8: *const u8, kind_len: usize) -> bool {
    if kind_utf8.is_null() || kind_len == 0 || kind_len > 64 {
        return false;
    }
    let Ok(kind) = std::str::from_utf8(unsafe { std::slice::from_raw_parts(kind_utf8, kind_len) })
    else {
        return false;
    };
    try_dispatch_keymap(kind).is_some_and(|result| result.accepted)
}

const MOD_SHIFT: u32 = 1;
const MOD_CONTROL: u32 = 2;
const MOD_ALT: u32 = 4;
const MOD_META: u32 = 8;

fn mac_key_token(key_code: u16, chars: &str) -> Option<KeyToken> {
    match key_code {
        49 => Some(KeyToken::Space),
        36 => Some(KeyToken::Enter),
        53 => Some(KeyToken::Escape),
        117 => Some(KeyToken::Delete),
        51 => Some(KeyToken::Backspace),
        48 => Some(KeyToken::Tab),
        126 => Some(KeyToken::ArrowUp),
        125 => Some(KeyToken::ArrowDown),
        123 => Some(KeyToken::ArrowLeft),
        124 => Some(KeyToken::ArrowRight),
        115 => Some(KeyToken::Home),
        119 => Some(KeyToken::End),
        116 => Some(KeyToken::PageUp),
        121 => Some(KeyToken::PageDown),
        _ => {
            let value = chars.chars().next()?.to_ascii_lowercase();
            AsciiKey::try_new(value).ok().map(KeyToken::Ascii)
        }
    }
}

fn mac_modifiers(bits: u32) -> Option<Modifiers> {
    let mut modifiers = Vec::new();
    if bits & MOD_SHIFT != 0 {
        modifiers.push(Modifier::Shift);
    }
    if bits & MOD_CONTROL != 0 {
        modifiers.push(Modifier::Control);
    }
    if bits & MOD_ALT != 0 {
        modifiers.push(Modifier::Alt);
    }
    if bits & MOD_META != 0 {
        modifiers.push(Modifier::Meta);
    }
    Modifiers::try_new(modifiers).ok()
}

fn resolve_mac_key_action(key_code: u16, modifier_bits: u32, chars: &str) -> Option<ProductAction> {
    let key = mac_key_token(key_code, chars)?;
    let modifiers = mac_modifiers(modifier_bits)?;
    let trigger = EffectiveTrigger::Keyboard {
        key,
        modifiers,
        phase: InputPhase::Press,
    };
    let registry = builtin_command_registry().ok()?;
    let base = product_builtin_keymap();
    let delta = load_user_keymap_override(default_user_keymap_override_path().as_deref(), &base);
    resolve_product_action(&trigger, &registry, &delta, PlatformCommandModifier::Meta)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
/// # Safety
/// `chars_utf8` may be null when `chars_len` is 0.
/// 戻り: 0=未束縛, 1=消費, 2=timeline既存deleteへ。
pub unsafe extern "C" fn motolii_rnapp_host_key_event(
    key_code: u16,
    modifier_bits: u32,
    chars_utf8: *const u8,
    chars_len: usize,
    is_repeat: bool,
    timeline_focused: bool,
) -> i32 {
    let chars = if chars_utf8.is_null() || chars_len == 0 {
        ""
    } else if chars_len > 16 {
        return 0;
    } else {
        match std::str::from_utf8(unsafe { std::slice::from_raw_parts(chars_utf8, chars_len) }) {
            Ok(value) => value,
            Err(_) => return 0,
        }
    };
    let Some(action) = resolve_mac_key_action(key_code, modifier_bits, chars) else {
        return 0;
    };
    let Some(kind) = product_action_host_kind(&action) else {
        return i32::from(matches!(action, ProductAction::Unwired(_)));
    };
    if is_repeat
        && matches!(
            kind,
            "toggle_playback"
                | "shuttle_forward"
                | "shuttle_stop"
                | "trim_clip_in"
                | "trim_clip_out"
        )
    {
        return 1;
    }
    if kind == "delete_layer" && timeline_focused {
        return 2;
    }
    i32::from(try_dispatch_keymap(kind).is_some_and(|result| result.accepted))
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn motolii_rnapp_host_dispatch_json(
    in_utf8: *const u8,
    in_len: usize,
    out: *mut u8,
    out_cap: usize,
) -> i64 {
    if in_utf8.is_null() || out.is_null() || out_cap == 0 || in_len > MAX_JSON_BYTES {
        return -1;
    }
    let Ok(intent) = std::str::from_utf8(unsafe { std::slice::from_raw_parts(in_utf8, in_len) })
    else {
        return -1;
    };
    let Ok(guard) = host_slot().lock() else {
        return -1;
    };
    let Some(slot) = guard.as_ref() else {
        let Ok(reject) = host_startup_reject().lock() else {
            return -1;
        };
        let Some(reject) = reject.as_deref() else {
            return -1;
        };
        if reject.len() > out_cap {
            return -(reject.len() as i64);
        }
        unsafe {
            std::ptr::copy_nonoverlapping(reject.as_ptr(), out, reject.len());
        }
        return reject.len() as i64;
    };
    let Ok(patched) = inject_host_handle(intent, slot.handle) else {
        return -1;
    };
    if patched.len() > MAX_JSON_BYTES {
        return -1;
    }
    unsafe {
        motolii_rn_host_dispatch_intent_json(
            slot.handle,
            patched.as_ptr(),
            patched.len(),
            out,
            out_cap,
        )
    }
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn motolii_rnapp_host_snapshot_json(out: *mut u8, out_cap: usize) -> i64 {
    if out.is_null() || out_cap == 0 {
        return -1;
    }
    let Ok(guard) = host_slot().lock() else {
        return -1;
    };
    let Some(slot) = guard.as_ref() else {
        return -1;
    };
    unsafe { motolii_rn_host_read_snapshot_json(slot.handle, out, out_cap) }
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn motolii_rnapp_commit_stage_transform(
    target_utf8: *const u8,
    target_len: usize,
    revision_utf8: *const u8,
    revision_len: usize,
    kind: i32,
    a: f64,
    b: f64,
) -> bool {
    if target_utf8.is_null() || revision_utf8.is_null() {
        return false;
    }
    let Ok(target) =
        std::str::from_utf8(unsafe { std::slice::from_raw_parts(target_utf8, target_len) })
    else {
        return false;
    };
    let Ok(revision) =
        std::str::from_utf8(unsafe { std::slice::from_raw_parts(revision_utf8, revision_len) })
    else {
        return false;
    };
    let Ok(expected_revision) = revision.parse::<u64>() else {
        return false;
    };
    let edit = match kind {
        1 => AppStageTransformEdit::RotateZ(a),
        2 => AppStageTransformEdit::Scale([a, b]),
        _ => return false,
    };
    try_commit_stage_transform(expected_revision, target, edit).is_ok()
}

/// Hostが在る時だけsnapshotを読む。不在・失敗はNone。
pub(crate) fn try_read_timeline_projection() -> Option<HostTimelineProjection> {
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
    #[cfg(target_os = "macos")]
    {
        #[cfg(test)]
        TEST_SNAPSHOT_READ_COUNT.fetch_add(1, Ordering::SeqCst);
        let Ok(guard) = host_slot().lock() else {
            return None;
        };
        let Some(slot) = guard.as_ref() else {
            return None;
        };
        let mut out = [0u8; MAX_SNAPSHOT_JSON_BYTES];
        let written =
            unsafe { motolii_rn_host_read_snapshot_json(slot.handle, out.as_mut_ptr(), out.len()) };
        if written <= 0 {
            return None;
        }
        let Some(json_bytes) = slice_from_written(&out, written) else {
            return None;
        };
        let Ok(json) = std::str::from_utf8(json_bytes) else {
            return None;
        };
        parse_timeline_projection(json)
    }
}

/// 軽量stamp。(revision, generation)。不在・失敗はNone。serializeしない。
pub(crate) fn try_read_projection_stamp() -> Option<(u64, u64)> {
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
    #[cfg(target_os = "macos")]
    {
        let Ok(guard) = host_slot().lock() else {
            return None;
        };
        let Some(slot) = guard.as_ref() else {
            return None;
        };
        let mut revision = 0u64;
        let mut generation = 0u64;
        if !unsafe { motolii_rn_host_projection_stamp(slot.handle, &mut revision, &mut generation) }
        {
            return None;
        }
        Some((revision, generation))
    }
}

/// scrub 秒 → set_time frame。`bar` は Skia 凍結名で中身は秒。
/// `frame = round(secs * fps.num / fps.den)`。
pub(crate) fn frame_from_scrub_bar(bar: f64, fps_num: i64, fps_den: i64) -> i64 {
    if fps_num <= 0 || fps_den <= 0 {
        return 0;
    }
    const SCALE: i128 = 1_000_000;
    // なぜ: 秒を先にµs丸めすると 24fps の 0.5 frame が 0 になる
    let frames_fixed =
        (bar * crate::timeline_skia::SECONDS_PER_BAR * (fps_num as f64) * (SCALE as f64)).round()
            as i128;
    let den = (fps_den as i128) * SCALE;
    let half = den / 2;
    let signed_half = if frames_fixed.is_negative() {
        -half
    } else {
        half
    };
    ((frames_fixed + signed_half) / den) as i64
}

/// 秒 → RationalTime wire `{num,den}`。`bar` は Skia 凍結名で中身は秒。
/// SCALE固定小数でf64連鎖丸めを避ける。
pub(crate) fn rational_time_parts_from_bar(bar: f64) -> (i64, i64) {
    const SCALE: i128 = 1_000_000;
    let s_fixed = (bar * crate::timeline_skia::SECONDS_PER_BAR * (SCALE as f64)).round() as i128;
    (s_fixed as i64, SCALE as i64)
}

/// host `current_time`(秒) → playhead(0..1)。
/// `song_bars` は Skia 凍結名。SECONDS_PER_BAR=1 なので曲長秒。
pub(crate) fn playhead_from_current_time(num: i64, den: i64, song_bars: f32) -> f64 {
    if den == 0 || song_bars <= 0.0 {
        return 0.0;
    }
    let secs = num as f64 / den as f64;
    let scene_secs = secs / crate::timeline_skia::SECONDS_PER_BAR;
    (scene_secs / f64::from(song_bars)).clamp(0.0, 1.0)
}

/// process host slot経由でset_timeを送り、同じHost応答を返す。host不在はNone。
pub(crate) fn try_dispatch_set_time(frame: i64) -> Option<HostTerminalResult> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = frame;
        None
    }
    #[cfg(target_os = "macos")]
    {
        let Ok(guard) = host_slot().lock() else {
            return None;
        };
        let Some(slot) = guard.as_ref() else {
            return None;
        };
        let intent = format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"set_time","host_handle":"{}","frame":{}}}"#,
            slot.handle, frame
        );
        dispatch_intent_json_terminal(slot.handle, &intent)
    }
}

pub(crate) fn try_dispatch_timeline_edit(
    commit: &crate::timeline_skia::TimelineEditCommit,
) -> Option<HostTerminalResult> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = commit;
        None
    }
    #[cfg(target_os = "macos")]
    {
        use crate::timeline_skia::TimelineEditCommit;
        let Ok(guard) = host_slot().lock() else {
            return None;
        };
        let Some(slot) = guard.as_ref() else {
            return None;
        };
        let intent = match commit {
            TimelineEditCommit::SetClipStart { layer_id, bar } => {
                let (num, den) = rational_time_parts_from_bar(f64::from(*bar));
                format!(
                    concat!(
                        r#"{{"version":1,"direction":"rn-to-host","kind":"set_clip_start","#,
                        r#""host_handle":"{}","target":"{}","time":{{"num":{},"den":{}}}}}"#
                    ),
                    slot.handle, layer_id, num, den
                )
            }
            TimelineEditCommit::TrimClipIn { layer_id, bar } => {
                let (num, den) = rational_time_parts_from_bar(f64::from(*bar));
                format!(
                    concat!(
                        r#"{{"version":1,"direction":"rn-to-host","kind":"trim_clip_in","#,
                        r#""host_handle":"{}","target":"{}","time":{{"num":{},"den":{}}}}}"#
                    ),
                    slot.handle, layer_id, num, den
                )
            }
            TimelineEditCommit::TrimClipOut { layer_id, bar } => {
                let (num, den) = rational_time_parts_from_bar(f64::from(*bar));
                format!(
                    concat!(
                        r#"{{"version":1,"direction":"rn-to-host","kind":"trim_clip_out","#,
                        r#""host_handle":"{}","target":"{}","time":{{"num":{},"den":{}}}}}"#
                    ),
                    slot.handle, layer_id, num, den
                )
            }
            TimelineEditCommit::SetPositionKeyTime {
                layer_id,
                key_id,
                bar,
            } => {
                // param_keys は diamond だけ。position_keys に無い id は commit しない。
                if !snapshot_has_position_key(slot.handle, layer_id, *key_id) {
                    return None;
                }
                #[cfg(test)]
                TEST_SET_POSITION_KEY_TIME_DISPATCH_COUNT.fetch_add(1, Ordering::SeqCst);
                let (num, den) = rational_time_parts_from_bar(f64::from(*bar));
                format!(
                    concat!(
                        r#"{{"version":1,"direction":"rn-to-host","kind":"set_position_key_time","#,
                        r#""host_handle":"{}","target":"{}","key_id":"{}","time":{{"num":{},"den":{}}}}}"#
                    ),
                    slot.handle, layer_id, key_id, num, den
                )
            }
            TimelineEditCommit::ReparentClip {
                layer_id,
                dest_layer_id,
                bar,
            } => {
                let (num, den) = rational_time_parts_from_bar(f64::from(*bar));
                format!(
                    concat!(
                        r#"{{"version":1,"direction":"rn-to-host","kind":"reparent_clip","#,
                        r#""host_handle":"{}","target":"{}","dest":"{}","time":{{"num":{},"den":{}}}}}"#
                    ),
                    slot.handle, layer_id, dest_layer_id, num, den
                )
            }
            TimelineEditCommit::ToggleMute { layer_id } => format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"mute","#,
                    r#""host_handle":"{}","target":"{}"}}"#
                ),
                slot.handle, layer_id
            ),
            TimelineEditCommit::ToggleSolo { layer_id } => format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"solo","#,
                    r#""host_handle":"{}","target":"{}"}}"#
                ),
                slot.handle, layer_id
            ),
            TimelineEditCommit::RemovePositionKey { layer_id, key_id } => {
                // param_keys diamond の Delete を Position 削除へ流さない。
                if !snapshot_has_position_key(slot.handle, layer_id, *key_id) {
                    return None;
                }
                format!(
                    concat!(
                        r#"{{"version":1,"direction":"rn-to-host","kind":"remove_position_key","#,
                        r#""host_handle":"{}","target":"{}","key_id":"{}"}}"#
                    ),
                    slot.handle, layer_id, key_id
                )
            }
        };
        dispatch_intent_json_terminal(slot.handle, &intent)
    }
}

pub(crate) fn try_dispatch_remove_position_key(
    layer_id: &str,
    key_id: u64,
) -> Option<HostTerminalResult> {
    #[cfg(test)]
    TEST_KEYMAP_REMOVE_POSITION_KEY_COUNT.fetch_add(1, Ordering::SeqCst);
    try_dispatch_timeline_edit(
        &crate::timeline_skia::TimelineEditCommit::RemovePositionKey {
            layer_id: layer_id.to_string(),
            key_id,
        },
    )
}

pub(crate) fn try_dispatch_timeline_selection(
    commit: &crate::timeline_skia::TimelineSelectionCommit,
) -> Option<HostTerminalResult> {
    #[cfg(test)]
    TEST_SELECTION_DISPATCH_COUNT.fetch_add(1, Ordering::SeqCst);
    #[cfg(not(target_os = "macos"))]
    {
        let _ = commit;
        None
    }
    #[cfg(target_os = "macos")]
    {
        use crate::timeline_skia::TimelineSelectionCommit;
        let Ok(guard) = host_slot().lock() else {
            return None;
        };
        let Some(slot) = guard.as_ref() else {
            return None;
        };
        let intent = match commit {
            TimelineSelectionCommit::SelectLayer { layer_id } => format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"select_layer","#,
                    r#""host_handle":"{}","target":"{}"}}"#
                ),
                slot.handle, layer_id
            ),
            TimelineSelectionCommit::ClearSelection => format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"clear_selection","#,
                    r#""host_handle":"{}"}}"#
                ),
                slot.handle
            ),
        };
        dispatch_intent_json_terminal(slot.handle, &intent)
    }
}

pub(crate) fn try_dispatch_move_layer_by(
    target: &str,
    delta: [f64; 2],
) -> Option<HostTerminalResult> {
    #[cfg(test)]
    TEST_MOVE_LAYER_BY_DISPATCH_COUNT.fetch_add(1, Ordering::SeqCst);
    if !delta.iter().all(|value| value.is_finite()) {
        return None;
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = target;
        None
    }
    #[cfg(target_os = "macos")]
    {
        let Ok(guard) = host_slot().lock() else {
            return None;
        };
        let Some(slot) = guard.as_ref() else {
            return None;
        };
        let intent = format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"move_layer_by","#,
                r#""host_handle":"{}","target":"{}","delta":[{},{}]}}"#
            ),
            slot.handle, target, delta[0], delta[1]
        );
        dispatch_intent_json_terminal(slot.handle, &intent)
    }
}

/// mount/resize 済みの論理 viewport。未設定は None。
pub(crate) fn try_stage_logical_size() -> Option<(f64, f64)> {
    let Ok(guard) = host_slot().lock() else {
        return None;
    };
    let slot = guard.as_ref()?;
    if slot.stage_logical_width > 0.0 && slot.stage_logical_height > 0.0 {
        Some((slot.stage_logical_width, slot.stage_logical_height))
    } else {
        None
    }
}

pub(crate) fn try_preview_stage_transform(
    expected_revision: u64,
    target: &str,
    edit: AppStageTransformEdit,
) -> Result<HostStageGeometry, String> {
    let target = target
        .parse::<u64>()
        .map_err(|_| "The selected layer identity is invalid".to_owned())?;
    let handle = host_slot()
        .lock()
        .map_err(|_| "Stage host is unavailable".to_owned())?
        .as_ref()
        .map(|slot| slot.handle)
        .ok_or_else(|| "Stage host is unavailable".to_owned())?;
    let preview = host_preview_stage_transform_for_app(handle, expected_revision, target, edit)
        .map_err(|error| error.to_string())?;
    Ok(preview.geometry.into())
}

pub(crate) fn try_commit_stage_transform(
    expected_revision: u64,
    target: &str,
    edit: AppStageTransformEdit,
) -> Result<(), String> {
    let result = dispatch_commit_stage_transform(expected_revision, target, edit);
    if result.accepted {
        Ok(())
    } else {
        Err(result
            .feedback()
            .unwrap_or("Stage transform rejected")
            .to_owned())
    }
}

pub(crate) fn dispatch_commit_stage_transform(
    expected_revision: u64,
    target: &str,
    edit: AppStageTransformEdit,
) -> HostTerminalResult {
    let Ok(target) = target.parse::<u64>() else {
        return rejected_terminal_result(
            "invalid_layer_identity",
            "The selected layer identity is invalid".to_owned(),
        );
    };
    let Some(handle) = host_slot()
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().map(|slot| slot.handle))
    else {
        return rejected_terminal_result(
            "host_unavailable",
            "Stage host is unavailable".to_owned(),
        );
    };
    let result = host_commit_stage_transform_for_app(handle, expected_revision, target, edit);
    let diagnostic = result.as_ref().err().map(|error| HostTerminalDiagnostic {
        reason: stage_transform_reason(error).to_owned(),
        host_handle: Some(handle.to_string()),
        stage_handle: None,
        timeline_handle: None,
        expected_projection_generation: None,
        actual_projection_generation: None,
    });
    HostTerminalResult {
        accepted: result.is_ok(),
        diagnostics: diagnostic.into_iter().collect(),
        message: result.as_ref().err().map(ToString::to_string),
        projection: try_read_timeline_projection(),
    }
}

fn rejected_terminal_result(reason: &str, message: String) -> HostTerminalResult {
    HostTerminalResult {
        accepted: false,
        diagnostics: vec![HostTerminalDiagnostic {
            reason: reason.to_owned(),
            host_handle: None,
            stage_handle: None,
            timeline_handle: None,
            expected_projection_generation: None,
            actual_projection_generation: None,
        }],
        message: Some(message),
        projection: try_read_timeline_projection(),
    }
}

fn stage_transform_reason(error: &motolii_ui::AppStageTransformError) -> &'static str {
    use motolii_ui::AppStageTransformError;
    match error {
        AppStageTransformError::HostUnavailable => "host_unavailable",
        AppStageTransformError::StaleDocument => "stale_document",
        AppStageTransformError::TargetUnavailable => "target_unavailable",
        AppStageTransformError::TransformUnavailable => "transform_unavailable",
        AppStageTransformError::OffKeyframe => "off_keyframe",
        AppStageTransformError::UnsupportedProperty => "unsupported_property",
        AppStageTransformError::NonFinite => "non_finite",
        AppStageTransformError::NoChange => "no_change",
        AppStageTransformError::Preview(_) => "preview",
        AppStageTransformError::Render(_) => "render",
        AppStageTransformError::Commit(_) => "commit",
    }
}

pub(crate) fn try_host_handle() -> Option<u64> {
    host_slot().lock().ok()?.as_ref().map(|slot| slot.handle)
}

pub(crate) fn host_slot_present() -> bool {
    try_host_handle().is_some()
}

#[cfg(test)]
pub(crate) fn test_reset_timeline_selection_dispatch_count() {
    TEST_SELECTION_DISPATCH_COUNT.store(0, Ordering::SeqCst);
}

#[cfg(test)]
pub(crate) fn test_timeline_selection_dispatch_count() -> u64 {
    TEST_SELECTION_DISPATCH_COUNT.load(Ordering::SeqCst)
}

#[cfg(test)]
pub(crate) fn test_reset_move_layer_by_dispatch_count() {
    TEST_MOVE_LAYER_BY_DISPATCH_COUNT.store(0, Ordering::SeqCst);
}

#[cfg(test)]
pub(crate) fn test_move_layer_by_dispatch_count() -> u64 {
    TEST_MOVE_LAYER_BY_DISPATCH_COUNT.load(Ordering::SeqCst)
}

#[cfg(test)]
pub(crate) fn test_reset_snapshot_read_count() {
    TEST_SNAPSHOT_READ_COUNT.store(0, Ordering::SeqCst);
}

#[cfg(test)]
pub(crate) fn test_snapshot_read_count() -> u64 {
    TEST_SNAPSHOT_READ_COUNT.load(Ordering::SeqCst)
}

#[cfg(test)]
pub(crate) fn test_reset_keymap_dispatch_counts() {
    TEST_KEYMAP_REMOVE_POSITION_KEY_COUNT.store(0, Ordering::SeqCst);
    TEST_KEYMAP_DELETE_LAYER_COUNT.store(0, Ordering::SeqCst);
}

#[cfg(test)]
pub(crate) fn test_keymap_remove_position_key_count() -> u64 {
    TEST_KEYMAP_REMOVE_POSITION_KEY_COUNT.load(Ordering::SeqCst)
}

#[cfg(test)]
pub(crate) fn test_keymap_delete_layer_count() -> u64 {
    TEST_KEYMAP_DELETE_LAYER_COUNT.load(Ordering::SeqCst)
}

#[cfg(test)]
pub(crate) fn test_clear_host_slot() {
    if let Ok(mut guard) = host_slot().lock() {
        *guard = None;
    }
}

/// Timeline Delete: real key選択中なら remove_position_key、否則 delete_layer。
pub(crate) fn try_timeline_keymap_delete(
    scene: &crate::timeline_skia::TimelineScene,
) -> Option<HostTerminalResult> {
    if let Some(crate::timeline_skia::TimelineEditCommit::RemovePositionKey { layer_id, key_id }) =
        crate::timeline_skia::remove_position_key_commit(scene)
    {
        return try_dispatch_remove_position_key(&layer_id, key_id);
    }
    #[cfg(test)]
    TEST_KEYMAP_DELETE_LAYER_COUNT.fetch_add(1, Ordering::SeqCst);
    try_dispatch_keymap("delete_layer")
}

/// keymap: undo / redo / delete_layer(現primary=RemoveTrackItem) / duplicate(現primary) / toggle_playback。primaryなしのdelete/duplicateは何もしない。
pub(crate) fn try_dispatch_keymap(kind: &str) -> Option<HostTerminalResult> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = kind;
        None
    }
    #[cfg(target_os = "macos")]
    {
        let Ok(guard) = host_slot().lock() else {
            return None;
        };
        let Some(slot) = guard.as_ref() else {
            return None;
        };
        let intent = match kind {
            "undo" | "redo" | "toggle_playback" | "shuttle_forward" | "shuttle_reverse"
            | "shuttle_stop" => format!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"{kind}","host_handle":"{}"}}"#,
                slot.handle
            ),
            "delete_layer" | "duplicate" | "split" | "mute" | "solo" | "trim_clip_in"
            | "trim_clip_out" => {
                drop(guard);
                let Some(projection) = try_read_timeline_projection() else {
                    return None;
                };
                let Some(target) = projection.primary_layer_id else {
                    return Some(HostTerminalResult {
                        accepted: true,
                        diagnostics: Vec::new(),
                        message: None,
                        projection: Some(projection),
                    });
                };
                let Ok(guard) = host_slot().lock() else {
                    return None;
                };
                let Some(slot) = guard.as_ref() else {
                    return None;
                };
                let intent = if kind == "split" || kind == "trim_clip_in" || kind == "trim_clip_out"
                {
                    let (num, den) = projection.current_time;
                    format!(
                        concat!(
                            r#"{{"version":1,"direction":"rn-to-host","kind":"{}","#,
                            r#""host_handle":"{}","target":"{}","time":{{"num":{},"den":{}}}}}"#
                        ),
                        kind, slot.handle, target, num, den
                    )
                } else {
                    format!(
                        concat!(
                            r#"{{"version":1,"direction":"rn-to-host","kind":"{}","#,
                            r#""host_handle":"{}","target":"{}"}}"#
                        ),
                        kind, slot.handle, target
                    )
                };
                return dispatch_intent_json_terminal(slot.handle, &intent);
            }
            _ => return None,
        };
        dispatch_intent_json_terminal(slot.handle, &intent)
    }
}

#[cfg(target_os = "macos")]
fn dispatch_intent_json_terminal(handle: u64, intent: &str) -> Option<HostTerminalResult> {
    let intent = intent_with_projection_generation(handle, intent);
    if intent.len() > MAX_JSON_BYTES {
        return None;
    }
    let mut out = [0u8; MAX_SNAPSHOT_JSON_BYTES];
    let written = unsafe {
        motolii_rn_host_dispatch_intent_json(
            handle,
            intent.as_ptr(),
            intent.len(),
            out.as_mut_ptr(),
            out.len(),
        )
    };
    if written <= 0 {
        return None;
    }
    let Some(response_bytes) = slice_from_written(&out, written) else {
        return None;
    };
    let Ok(response) = std::str::from_utf8(response_bytes) else {
        return None;
    };
    parse_terminal_result(response)
}

#[cfg(target_os = "macos")]
fn intent_with_projection_generation(handle: u64, intent: &str) -> String {
    if intent.contains("\"projection_generation\"") {
        return intent.to_owned();
    }
    let mut _revision = 0u64;
    let mut generation = 0u64;
    if !unsafe { motolii_rn_host_projection_stamp(handle, &mut _revision, &mut generation) } {
        return intent.to_owned();
    }
    let Some(body) = intent.strip_suffix('}') else {
        return intent.to_owned();
    };
    format!(r#"{body},"projection_generation":"{generation}"}}"#)
}

fn parse_terminal_result(response: &str) -> Option<HostTerminalResult> {
    let accepted = json_bool_value(response, "accepted")?;
    let projection = find_key_object(response, "snapshot").and_then(parse_timeline_projection);
    let diagnostics = parse_terminal_diagnostics(response)?;
    Some(HostTerminalResult {
        accepted,
        diagnostics,
        message: json_string_value(response, "message"),
        projection,
    })
}

fn parse_terminal_diagnostics(response: &str) -> Option<Vec<HostTerminalDiagnostic>> {
    let Some(array) = find_root_key_array(response, "diagnostics") else {
        return Some(Vec::new());
    };
    let mut diagnostics = Vec::new();
    let mut rest = &array[1..array.len() - 1];
    loop {
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        if let Some(next) = rest.strip_prefix(',') {
            rest = next;
            continue;
        }
        if !rest.starts_with('{') {
            return None;
        }
        let end = find_matching_brace(rest)?;
        let diagnostic = &rest[..=end];
        diagnostics.push(HostTerminalDiagnostic {
            reason: json_string_value(diagnostic, "reason")?,
            host_handle: json_string_value(diagnostic, "host_handle"),
            stage_handle: json_string_value(diagnostic, "stage_handle"),
            timeline_handle: json_string_value(diagnostic, "timeline_handle"),
            expected_projection_generation: json_string_value(
                diagnostic,
                "expected_projection_generation",
            ),
            actual_projection_generation: json_string_value(
                diagnostic,
                "actual_projection_generation",
            ),
        });
        rest = &rest[end + 1..];
    }
    Some(diagnostics)
}

fn response_is_accepted(response: &str) -> bool {
    let Some(pos) = response.find("\"accepted\"") else {
        return false;
    };
    let Some(colon_pos) = response[pos..].find(':') else {
        return false;
    };
    response[pos + colon_pos + 1..]
        .trim_start()
        .starts_with("true")
}

fn slice_from_written<'a>(out: &'a [u8], written: i64) -> Option<&'a [u8]> {
    let written = usize::try_from(written).ok()?;
    (written <= out.len()).then_some(&out[..written])
}

fn inject_host_handle(intent: &str, handle: u64) -> Result<String, ()> {
    const KEY: &str = "\"host_handle\"";
    // top-level fieldのみ。brace depthで nested `"host_handle"` への誤爆を防ぐ。
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;
    let bytes = intent.as_bytes();
    let mut key_at = None;
    let mut root_open = None;
    let mut root_end = None;
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match b {
            b'"' => {
                if depth == 1 && intent[i..].starts_with(KEY) {
                    let after = &intent[i + KEY.len()..];
                    if after.trim_start().starts_with(':') {
                        key_at = Some(i);
                        break;
                    }
                }
                in_string = true;
                i += 1;
            }
            b'{' => {
                if depth == 0 {
                    root_open = Some(i);
                }
                depth += 1;
                i += 1;
            }
            b'}' => {
                if depth == 1 {
                    root_end = Some(i);
                    break;
                }
                depth -= 1;
                i += 1;
            }
            _ => i += 1,
        }
    }
    let Some(key_at) = key_at else {
        let root_open = root_open.ok_or(())?;
        let root_end = root_end.ok_or(())?;
        let separator = if intent[root_open + 1..root_end].trim().is_empty() {
            ""
        } else {
            ","
        };
        let mut out = String::with_capacity(intent.len() + 40);
        out.push_str(&intent[..root_end]);
        out.push_str(separator);
        out.push_str(&format!(r#""host_handle":"{handle}""#));
        out.push_str(&intent[root_end..]);
        return Ok(out);
    };
    let after_key = &intent[key_at + KEY.len()..];
    let colon = after_key.find(':').ok_or(())?;
    let after_colon = &after_key[colon + 1..];
    let value_start_rel = after_colon.find('"').ok_or(())?;
    let value_body = &after_colon[value_start_rel + 1..];
    let value_end_rel = value_body.find('"').ok_or(())?;
    let abs_value_start = key_at + KEY.len() + colon + 1 + value_start_rel + 1;
    let abs_value_end = abs_value_start + value_end_rel;
    let mut out = String::with_capacity(intent.len() + 24);
    out.push_str(&intent[..abs_value_start]);
    out.push_str(&handle.to_string());
    out.push_str(&intent[abs_value_end..]);
    Ok(out)
}

fn parse_timeline_projection(json: &str) -> Option<HostTimelineProjection> {
    let revision = json_string_value(json, "revision")?;
    let projection_generation =
        json_string_value(json, "projection_generation").unwrap_or_else(|| "0".into());
    let primary_layer_id = json_string_value(json, "primary_layer_id");
    let current_time = json_rational(json, "current_time").unwrap_or((0, 1));
    let timeline_duration =
        find_key_object(json, "timeline").and_then(|timeline| json_rational(timeline, "duration"));
    let fps = find_key_object(json, "timeline").and_then(|timeline| json_rational(timeline, "fps"));
    let bounds = parse_bounds(json)?;
    let timeline_layers = parse_timeline_layers(json);
    // 壊れていたら stage_geometry 全体を None へ（timeline は維持）。
    let stage_geometry = parse_stage_geometry(json);
    Some(HostTimelineProjection {
        revision,
        projection_generation,
        primary_layer_id,
        current_time,
        timeline_duration,
        fps,
        bounds,
        timeline_layers,
        stage_geometry,
    })
}

/// wire `catalog`。欠落・壊れは None。
pub(crate) fn parse_catalog_projection(json: &str) -> Option<HostCatalogProjection> {
    parse_catalog(json)
}

pub(crate) fn snapshot_layers_from_projection(
    projection: &HostTimelineProjection,
) -> Vec<crate::timeline_skia::SnapshotLayerInput> {
    use crate::timeline_skia::{SnapshotKeyInput, SnapshotLayerInput};
    if let Some(layers) = &projection.timeline_layers {
        return layers
            .iter()
            .map(|layer| SnapshotLayerInput {
                layer_id: layer.layer_id.clone(),
                display_name: layer.display_name.clone(),
                interval_secs: Some((layer.start_secs, layer.duration_secs)),
                keys: layer
                    .position_keys
                    .iter()
                    .chain(layer.param_keys.iter())
                    .map(|key| SnapshotKeyInput {
                        key_id: key.key_id,
                        time_secs: key.time_secs,
                    })
                    .collect(),
            })
            .collect();
    }
    projection
        .bounds
        .iter()
        .map(|(layer_id, display_name)| SnapshotLayerInput {
            layer_id: layer_id.clone(),
            display_name: display_name.clone(),
            interval_secs: None,
            keys: Vec::new(),
        })
        .collect()
}

fn json_string_value(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let mut search = json;
    let mut abs = 0usize;
    while let Some(at) = search.find(&needle) {
        abs += at;
        let after = &json[abs + needle.len()..];
        let trimmed = after.trim_start();
        if let Some(rest) = trimmed.strip_prefix(':') {
            let rest = rest.trim_start();
            if rest.starts_with("null") {
                return None;
            }
            if let Some(body) = rest.strip_prefix('"') {
                let (decoded, _) = scan_json_string(body)?;
                return Some(decoded);
            }
        }
        abs += needle.len();
        search = &json[abs..];
    }
    None
}

fn scan_json_string(input: &str) -> Option<(String, usize)> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut out = String::new();
    let mut i = 0usize;
    while i < len {
        let b = bytes[i];
        if b == b'\\' {
            if i + 1 >= len {
                return None;
            }
            let esc = bytes[i + 1];
            match esc {
                b'"' => {
                    out.push('"');
                    i += 2;
                }
                b'\\' => {
                    out.push('\\');
                    i += 2;
                }
                b'/' => {
                    out.push('/');
                    i += 2;
                }
                b'b' => {
                    out.push('\u{0008}');
                    i += 2;
                }
                b'f' => {
                    out.push('\u{000C}');
                    i += 2;
                }
                b'n' => {
                    out.push('\n');
                    i += 2;
                }
                b'r' => {
                    out.push('\r');
                    i += 2;
                }
                b't' => {
                    out.push('\t');
                    i += 2;
                }
                b'u' => {
                    if i + 6 > len {
                        return None;
                    }
                    let mut codepoint = parse_hex_u16(input, i + 2)? as u32;
                    i += 6;
                    if (0xD800..=0xDBFF).contains(&(codepoint as u16)) {
                        if i + 6 > len || bytes[i] != b'\\' || bytes[i + 1] != b'u' {
                            return None;
                        }
                        let next_codepoint = parse_hex_u16(input, i + 2)?;
                        if !(0xDC00..=0xDFFF).contains(&next_codepoint) {
                            return None;
                        }
                        let high = codepoint as u32 - 0xD800;
                        let low = next_codepoint as u32 - 0xDC00;
                        codepoint = 0x10000 + ((high << 10) | low);
                        i += 6;
                    }
                    let Some(ch) = std::char::from_u32(codepoint) else {
                        return None;
                    };
                    out.push(ch);
                }
                _ => return None,
            }
            continue;
        }
        if b == b'"' {
            return Some((out, i));
        }
        let next = input[i..].chars().next()?;
        out.push(next);
        i += next.len_utf8();
    }
    None
}

fn parse_hex_u16(input: &str, start: usize) -> Option<u16> {
    let end = start + 4;
    if start >= input.len() || end > input.len() {
        return None;
    }
    u16::from_str_radix(&input[start..end], 16).ok()
}

fn parse_bounds(json: &str) -> Option<Vec<(String, String)>> {
    let marker = "\"bounds\"";
    let at = json.find(marker)?;
    let after = json[at + marker.len()..].trim_start().strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('[')?;
    let mut bounds = Vec::new();
    let mut rest = after;
    loop {
        rest = rest.trim_start();
        if rest.starts_with(']') {
            break;
        }
        if rest.starts_with(',') {
            rest = rest[1..].trim_start();
            continue;
        }
        if !rest.starts_with('{') {
            return None;
        }
        let end = find_matching_brace(rest)?;
        let obj = &rest[..=end];
        let layer_id = json_string_value(obj, "layer_id")?.to_owned();
        let display_name = json_string_value(obj, "display_name")?.to_owned();
        bounds.push((layer_id, display_name));
        rest = &rest[end + 1..];
    }
    Some(bounds)
}

fn parse_timeline_layers(json: &str) -> Option<Vec<HostTimelineLayer>> {
    let timeline = find_key_object(json, "timeline")?;
    let marker = "\"layers\"";
    let at = timeline.find(marker)?;
    let after = timeline[at + marker.len()..]
        .trim_start()
        .strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('[')?;
    let mut layers = Vec::new();
    let mut rest = after;
    loop {
        rest = rest.trim_start();
        if rest.starts_with(']') {
            break;
        }
        if rest.starts_with(',') {
            rest = rest[1..].trim_start();
            continue;
        }
        if !rest.starts_with('{') {
            return None;
        }
        let end = find_matching_brace(rest)?;
        let obj = &rest[..=end];
        let layer_id = json_string_value(obj, "layer_id")?.to_owned();
        let display_name = json_string_value(obj, "display_name")?.to_owned();
        let (start_num, start_den) = json_rational(obj, "start")?;
        let (duration_num, duration_den) = json_rational(obj, "duration")?;
        if start_den == 0 || duration_den == 0 {
            return None;
        }
        let position_keys = parse_position_keys(obj)?;
        // param_keys 欠落・壊れは空。layer 自体は落とさない。
        let param_keys = parse_param_keys(obj).unwrap_or_default();
        // effects 欠落は空。壊れ値は空へ fallback（layer 自体は落とさない）。
        let (effects, effects_truncated) = parse_layer_effects(obj).unwrap_or_default();
        let (source_params, source_params_truncated) =
            parse_layer_source_params(obj).unwrap_or_default();
        layers.push(HostTimelineLayer {
            layer_id,
            display_name,
            start_secs: start_num as f64 / start_den as f64,
            duration_secs: duration_num as f64 / duration_den as f64,
            position_keys,
            param_keys,
            effects,
            effects_truncated,
            source_params,
            source_params_truncated,
            visible: json_bool_value(obj, "visible").unwrap_or(true),
            solo: json_bool_value(obj, "solo").unwrap_or(false),
        });
        rest = &rest[end + 1..];
    }
    Some(layers)
}

fn parse_catalog(json: &str) -> Option<HostCatalogProjection> {
    let obj = find_key_object(json, "catalog")?;
    let effects = parse_catalog_entries(obj, "effects")?;
    // sources 壊れは sources だけ空へ。effects / catalog 全体は落とさない。
    let sources = match obj.find("\"sources\"") {
        None => Vec::new(),
        Some(_) => parse_catalog_entries(obj, "sources")
            .map(|entries| {
                entries
                    .into_iter()
                    .map(|entry| HostCatalogSource {
                        plugin_id: entry.plugin_id,
                        name: entry.name,
                        effect_version: entry.effect_version,
                    })
                    .collect()
            })
            .unwrap_or_default(),
    };
    Some(HostCatalogProjection { effects, sources })
}

fn parse_catalog_entries(obj: &str, key: &str) -> Option<Vec<HostCatalogEffect>> {
    let marker = format!("\"{key}\"");
    let at = obj.find(&marker)?;
    let after = obj[at + marker.len()..].trim_start().strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('[')?;
    let mut entries = Vec::new();
    let mut rest = after;
    loop {
        rest = rest.trim_start();
        if rest.starts_with(']') {
            break;
        }
        if rest.starts_with(',') {
            rest = rest[1..].trim_start();
            continue;
        }
        if !rest.starts_with('{') {
            return None;
        }
        let end = find_matching_brace(rest)?;
        let entry_obj = &rest[..=end];
        let plugin_id = json_string_value(entry_obj, "plugin_id")?.to_owned();
        let name = json_string_value(entry_obj, "name")?.to_owned();
        let effect_version = json_u32_value(entry_obj, "effect_version")?;
        entries.push(HostCatalogEffect {
            plugin_id,
            name,
            effect_version,
        });
        rest = &rest[end + 1..];
    }
    Some(entries)
}

fn parse_layer_effects(layer_obj: &str) -> Option<(Vec<HostTimelineEffect>, bool)> {
    let marker = "\"effects\"";
    let Some(at) = layer_obj.find(marker) else {
        return Some((Vec::new(), false));
    };
    let after = layer_obj[at + marker.len()..]
        .trim_start()
        .strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('[')?;
    let effects_truncated = json_bool_value(layer_obj, "effects_truncated").unwrap_or(false);
    let mut effects = Vec::new();
    let mut rest = after;
    loop {
        rest = rest.trim_start();
        if rest.starts_with(']') {
            break;
        }
        if rest.starts_with(',') {
            rest = rest[1..].trim_start();
            continue;
        }
        if !rest.starts_with('{') {
            return None;
        }
        let end = find_matching_brace(rest)?;
        let obj = &rest[..=end];
        let effect_use_id = json_string_value(obj, "effect_use_id")?.to_owned();
        let plugin_id = json_string_value(obj, "plugin_id")?.to_owned();
        let params = parse_effect_params(obj)?;
        effects.push(HostTimelineEffect {
            effect_use_id,
            plugin_id,
            params,
        });
        rest = &rest[end + 1..];
    }
    Some((effects, effects_truncated))
}

fn parse_layer_source_params(layer_obj: &str) -> Option<(Vec<HostTimelineSourceParam>, bool)> {
    let marker = "\"source_params\"";
    let Some(at) = layer_obj.find(marker) else {
        return Some((Vec::new(), false));
    };
    let after = layer_obj[at + marker.len()..]
        .trim_start()
        .strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('[')?;
    let source_params_truncated =
        json_bool_value(layer_obj, "source_params_truncated").unwrap_or(false);
    let mut source_params = Vec::new();
    let mut rest = after;
    loop {
        rest = rest.trim_start();
        if rest.starts_with(']') {
            break;
        }
        if rest.starts_with(',') {
            rest = rest[1..].trim_start();
            continue;
        }
        if !rest.starts_with('{') {
            return None;
        }
        let end = find_matching_brace(rest)?;
        let obj = &rest[..=end];
        let param_id = json_string_value(obj, "param_id")?.to_owned();
        let value = json_f64_value(obj, "value")?;
        if !value.is_finite() {
            return None;
        }
        source_params.push(HostTimelineSourceParam { param_id, value });
        rest = &rest[end + 1..];
    }
    Some((source_params, source_params_truncated))
}

fn parse_effect_params(effect_obj: &str) -> Option<Vec<HostTimelineEffectParam>> {
    let marker = "\"params\"";
    let at = effect_obj.find(marker)?;
    let after = effect_obj[at + marker.len()..]
        .trim_start()
        .strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('[')?;
    let mut params = Vec::new();
    let mut rest = after;
    loop {
        rest = rest.trim_start();
        if rest.starts_with(']') {
            break;
        }
        if rest.starts_with(',') {
            rest = rest[1..].trim_start();
            continue;
        }
        if !rest.starts_with('{') {
            return None;
        }
        let end = find_matching_brace(rest)?;
        let obj = &rest[..=end];
        let param_id = json_string_value(obj, "param_id")?.to_owned();
        let value = json_f64_value(obj, "value")?;
        if !value.is_finite() {
            return None;
        }
        params.push(HostTimelineEffectParam { param_id, value });
        rest = &rest[end + 1..];
    }
    Some(params)
}

fn json_u32_value(json: &str, key: &str) -> Option<u32> {
    let value = json_i64_value(json, key)?;
    u32::try_from(value).ok()
}

fn json_f64_value(json: &str, key: &str) -> Option<f64> {
    let needle = format!("\"{key}\"");
    let mut search = json;
    let mut abs = 0usize;
    while let Some(at) = search.find(&needle) {
        abs += at;
        let after = &json[abs + needle.len()..];
        let trimmed = after.trim_start();
        if let Some(rest) = trimmed.strip_prefix(':') {
            let (value, _) = parse_json_f64(rest.trim_start())?;
            return Some(value);
        }
        abs += needle.len();
        search = &json[abs..];
    }
    None
}

fn parse_stage_geometry(json: &str) -> Option<HostStageGeometry> {
    let obj = find_key_object(json, "stage_geometry")?;
    let layers_truncated = json_bool_value(obj, "layers_truncated")?;
    let marker = "\"layers\"";
    let at = obj.find(marker)?;
    let after = obj[at + marker.len()..].trim_start().strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('[')?;
    let mut layers = Vec::new();
    let mut rest = after;
    loop {
        rest = rest.trim_start();
        if rest.starts_with(']') {
            break;
        }
        if rest.starts_with(',') {
            rest = rest[1..].trim_start();
            continue;
        }
        if !rest.starts_with('{') {
            return None;
        }
        let end = find_matching_brace(rest)?;
        let layer_obj = &rest[..=end];
        let layer_id = json_string_value(layer_obj, "layer_id")?;
        let corners = parse_corners(layer_obj)?;
        let position = parse_vec2_field(layer_obj, "position").unwrap_or([0.0, 0.0]);
        let rotation = json_f64_value(layer_obj, "rotation").unwrap_or(0.0);
        let scale = parse_vec2_field(layer_obj, "scale").unwrap_or([1.0, 1.0]);
        layers.push(HostStageGeometryLayer {
            layer_id,
            corners,
            position,
            rotation,
            scale,
        });
        rest = &rest[end + 1..];
    }
    Some(HostStageGeometry {
        layers,
        layers_truncated,
    })
}

fn parse_corners(layer_obj: &str) -> Option<[[f64; 2]; 4]> {
    let marker = "\"corners\"";
    let at = layer_obj.find(marker)?;
    let after = layer_obj[at + marker.len()..]
        .trim_start()
        .strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('[')?;
    let mut points = Vec::with_capacity(4);
    let mut rest = after;
    loop {
        rest = rest.trim_start();
        if rest.starts_with(']') {
            break;
        }
        if rest.starts_with(',') {
            rest = rest[1..].trim_start();
            continue;
        }
        if !rest.starts_with('[') {
            return None;
        }
        let end = find_matching_bracket(rest)?;
        let pair = &rest[1..end];
        let (x, after_x) = parse_json_f64(pair)?;
        if !is_finite_f32_compatible(x) {
            return None;
        }
        let after_x = after_x.trim_start();
        if !after_x.starts_with(',') {
            return None;
        }
        let (y, after_y) = parse_json_f64(&after_x[1..])?;
        if !is_finite_f32_compatible(y) {
            return None;
        }
        if !after_y.trim_start().is_empty() {
            return None;
        }
        points.push([x, y]);
        rest = &rest[end + 1..];
    }
    if points.len() != 4 {
        return None;
    }
    Some([points[0], points[1], points[2], points[3]])
}

fn parse_vec2_field(layer_obj: &str, key: &str) -> Option<[f64; 2]> {
    let needle = format!("\"{key}\"");
    let at = layer_obj.find(&needle)?;
    let after = layer_obj[at + needle.len()..]
        .trim_start()
        .strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('[')?;
    let (x, rest) = parse_json_f64(after)?;
    if !is_finite_f32_compatible(x) {
        return None;
    }
    let rest = rest.trim_start().strip_prefix(',')?;
    let (y, _) = parse_json_f64(rest)?;
    if !is_finite_f32_compatible(y) {
        return None;
    }
    Some([x, y])
}

fn is_finite_f32_compatible(value: f64) -> bool {
    value.is_finite() && value.abs() <= f64::from(f32::MAX)
}

fn parse_json_f64(input: &str) -> Option<(f64, &str)> {
    let trimmed = input.trim_start();
    let end = trimmed
        .find(|ch: char| !(ch.is_ascii_digit() || matches!(ch, '-' | '+' | '.' | 'e' | 'E')))
        .unwrap_or(trimmed.len());
    if end == 0 {
        return None;
    }
    let value = trimmed[..end].parse().ok()?;
    Some((value, &trimmed[end..]))
}

fn json_bool_value(json: &str, key: &str) -> Option<bool> {
    let needle = format!("\"{key}\"");
    let mut search = json;
    let mut abs = 0usize;
    while let Some(at) = search.find(&needle) {
        abs += at;
        let after = &json[abs + needle.len()..];
        let trimmed = after.trim_start();
        if let Some(rest) = trimmed.strip_prefix(':') {
            let rest = rest.trim_start();
            if rest.starts_with("true") {
                return Some(true);
            }
            if rest.starts_with("false") {
                return Some(false);
            }
            return None;
        }
        abs += needle.len();
        search = &json[abs..];
    }
    None
}

fn find_matching_bracket(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;
    for (i, ch) in s.char_indices() {
        if in_string {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_position_keys(layer_obj: &str) -> Option<Vec<HostTimelineKey>> {
    parse_key_array(layer_obj, "\"position_keys\"")
}

fn parse_param_keys(layer_obj: &str) -> Option<Vec<HostTimelineKey>> {
    parse_key_array(layer_obj, "\"param_keys\"")
}

fn parse_key_array(layer_obj: &str, marker: &str) -> Option<Vec<HostTimelineKey>> {
    let at = layer_obj.find(marker)?;
    let after = layer_obj[at + marker.len()..]
        .trim_start()
        .strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('[')?;
    let mut keys = Vec::new();
    let mut rest = after;
    loop {
        rest = rest.trim_start();
        if rest.starts_with(']') {
            break;
        }
        if rest.starts_with(',') {
            rest = rest[1..].trim_start();
            continue;
        }
        if !rest.starts_with('{') {
            return None;
        }
        let end = find_matching_brace(rest)?;
        let obj = &rest[..=end];
        let key_id = json_string_value(obj, "key_id")?.parse::<u64>().ok()?;
        let (time_num, time_den) = json_rational(obj, "time")?;
        if time_den == 0 {
            return None;
        }
        let value = parse_optional_vec2(obj);
        keys.push(HostTimelineKey {
            key_id,
            time_secs: time_num as f64 / time_den as f64,
            value,
        });
        rest = &rest[end + 1..];
    }
    Some(keys)
}

#[cfg(target_os = "macos")]
fn snapshot_has_position_key(handle: u64, layer_id: &str, key_id: u64) -> bool {
    let mut out = [0u8; MAX_SNAPSHOT_JSON_BYTES];
    let written =
        unsafe { motolii_rn_host_read_snapshot_json(handle, out.as_mut_ptr(), out.len()) };
    if written <= 0 {
        return false;
    }
    let Some(json_bytes) = slice_from_written(&out, written) else {
        return false;
    };
    let Ok(json) = std::str::from_utf8(json_bytes) else {
        return false;
    };
    let Some(proj) = parse_timeline_projection(json) else {
        return false;
    };
    let Some(layers) = proj.timeline_layers.as_deref() else {
        return false;
    };
    layer_has_position_key(layers, layer_id, key_id)
}

fn layer_has_position_key(layers: &[HostTimelineLayer], layer_id: &str, key_id: u64) -> bool {
    layers.iter().any(|layer| {
        layer.layer_id == layer_id && layer.position_keys.iter().any(|key| key.key_id == key_id)
    })
}

fn parse_optional_vec2(obj: &str) -> Option<[f64; 2]> {
    let marker = "\"value\"";
    let at = obj.find(marker)?;
    let after = obj[at + marker.len()..].trim_start().strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('[')?;
    let (x, after_x) = parse_json_f64(after)?;
    let after_x = after_x.trim_start();
    if !after_x.starts_with(',') {
        return None;
    }
    let (y, after_y) = parse_json_f64(&after_x[1..])?;
    let after_y = after_y.trim_start();
    if !after_y.starts_with(']') {
        return None;
    }
    Some([x, y])
}

fn find_key_object<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{key}\"");
    let mut search = json;
    let mut abs = 0usize;
    while let Some(at) = search.find(&needle) {
        abs += at;
        let after = &json[abs + needle.len()..];
        let trimmed = after.trim_start();
        if let Some(rest) = trimmed.strip_prefix(':') {
            let rest = rest.trim_start();
            if rest.starts_with('{') {
                let end = find_matching_brace(rest)?;
                return Some(&rest[..=end]);
            }
        }
        abs += needle.len();
        search = &json[abs..];
    }
    None
}

fn find_root_key_array<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{key}\"");
    let bytes = json.as_bytes();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;
    let mut i = 0usize;
    while i < bytes.len() {
        if in_string {
            if escape {
                escape = false;
            } else if bytes[i] == b'\\' {
                escape = true;
            } else if bytes[i] == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            b'"' if depth == 1 && json[i..].starts_with(&needle) => {
                let after = json[i + needle.len()..].trim_start().strip_prefix(':')?;
                let array = after.trim_start();
                let end = find_matching_bracket(array)?;
                return Some(&array[..=end]);
            }
            b'"' => in_string = true,
            _ => {}
        }
        i += 1;
    }
    None
}

fn json_rational(json: &str, key: &str) -> Option<(i64, i64)> {
    let obj = find_key_object(json, key)?;
    let num = json_i64_value(obj, "num")?;
    let den = json_i64_value(obj, "den")?;
    Some((num, den))
}

fn json_i64_value(json: &str, key: &str) -> Option<i64> {
    let needle = format!("\"{key}\"");
    let mut search = json;
    let mut abs = 0usize;
    while let Some(at) = search.find(&needle) {
        abs += at;
        let after = &json[abs + needle.len()..];
        let trimmed = after.trim_start();
        if let Some(rest) = trimmed.strip_prefix(':') {
            let rest = rest.trim_start();
            let end = rest
                .find(|ch: char| !(ch.is_ascii_digit() || ch == '-' || ch == '+'))
                .unwrap_or(rest.len());
            if end == 0 {
                return None;
            }
            return rest[..end].parse().ok();
        }
        abs += needle.len();
        search = &json[abs..];
    }
    None
}

fn find_matching_brace(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;
    for (i, ch) in s.char_indices() {
        if in_string {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use crate::renderer_core::RendererCore;
    use crate::timeline_skia::TimelineScene;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Mutex, MutexGuard};

    static TEST_LOCK: Mutex<()> = Mutex::new(());
    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_lock() -> MutexGuard<'static, ()> {
        TEST_LOCK.lock().expect("host bridge test lock")
    }

    fn temp_project(tag: &str) -> std::path::PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("motolii-rnapp-host-{tag}-{id}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("project.json");
        assert!(ensure_project_document(&path));
        path
    }

    fn bounds_from_snapshot(
        snap: &motolii_ui::RnProductSnapshotForTest,
    ) -> Vec<crate::timeline_skia::SnapshotLayerInput> {
        use crate::timeline_skia::{SnapshotKeyInput, SnapshotLayerInput};
        snap.timeline
            .layers
            .iter()
            .map(|layer| SnapshotLayerInput {
                layer_id: layer.layer_id.clone(),
                display_name: layer.display_name.clone(),
                interval_secs: Some((
                    layer.start.as_seconds_f64(),
                    layer.duration.as_seconds_f64(),
                )),
                keys: layer
                    .position_keys
                    .iter()
                    .filter_map(|key| {
                        Some(SnapshotKeyInput {
                            key_id: key.key_id.parse().ok()?,
                            time_secs: key.time.as_seconds_f64(),
                        })
                    })
                    .collect(),
            })
            .collect()
    }

    fn dispatch_kind(host: u64, kind: &str, extra: &str) {
        let intent = format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"{kind}","host_handle":"{host}"{extra}}}"#
        );
        let mut out = [0u8; MAX_SNAPSHOT_JSON_BYTES];
        let written = unsafe {
            motolii_rn_host_dispatch_intent_json(
                host,
                intent.as_ptr(),
                intent.len(),
                out.as_mut_ptr(),
                out.len(),
            )
        };
        assert!(written > 0, "dispatch {kind} failed: {written}");
        let response_bytes =
            slice_from_written(&out, written).expect("dispatch response within buffer");
        let response = std::str::from_utf8(response_bytes).expect("utf8");
        assert!(
            response.contains(r#""accepted":true"#),
            "expected accepted: {response}"
        );
    }

    fn install_slot(host: u64) {
        let mut guard = host_slot().lock().expect("slot");
        *guard = Some(HostSlot {
            handle: host,
            project_path: PathBuf::new(),
            stage_handle: None,
            stage_pointer_active: false,
            stage_mounted: false,
            pointer_sequence: 0,
            stage_logical_width: 0.0,
            stage_logical_height: 0.0,
        });
    }

    fn clear_slot() {
        let mut guard = host_slot().lock().expect("slot");
        *guard = None;
    }

    #[test]
    fn place_then_undo_changes_from_snapshot_band_count() {
        let _lock = test_lock();
        let path = temp_project("place-undo");
        let host = motolii_ui::host_create_for_test(&path).expect("create host");
        let baseline = motolii_ui::host_read_snapshot_for_test(host).expect("baseline");
        let baseline_scene = TimelineScene::from_snapshot(
            &bounds_from_snapshot(&baseline),
            baseline.primary_layer_id.as_deref(),
        );
        assert_eq!(baseline_scene.band_count(), 0);
        assert!(baseline_scene.real);

        dispatch_kind(
            host,
            "place_rectangle",
            r#","position":[0.25,-0.125],"playhead":{"num":0,"den":1}"#,
        );
        let placed_snap = motolii_ui::host_read_snapshot_for_test(host).expect("placed");
        let placed_scene = TimelineScene::from_snapshot(
            &bounds_from_snapshot(&placed_snap),
            placed_snap.primary_layer_id.as_deref(),
        );
        assert_eq!(placed_scene.band_count(), 1);

        dispatch_kind(host, "undo", "");
        let undone_snap = motolii_ui::host_read_snapshot_for_test(host).expect("undone");
        let undone_scene = TimelineScene::from_snapshot(
            &bounds_from_snapshot(&undone_snap),
            undone_snap.primary_layer_id.as_deref(),
        );
        assert_eq!(undone_scene.band_count(), 0);

        motolii_ui::host_destroy_for_test(host).expect("destroy");
    }

    fn read_host_projection(host: u64) -> HostTimelineProjection {
        let mut out = [0u8; MAX_SNAPSHOT_JSON_BYTES];
        let written =
            unsafe { motolii_rn_host_read_snapshot_json(host, out.as_mut_ptr(), out.len()) };
        assert!(written > 0, "host snapshot json failed: {written}");
        let json_bytes =
            slice_from_written(&out, written).expect("snapshot response within buffer");
        let json = std::str::from_utf8(json_bytes).expect("snapshot json");
        parse_timeline_projection(json).expect("projection parse")
    }

    #[test]
    fn place_rectangle_snapshot_json_feeds_stage_geometry_identity() {
        let _lock = test_lock();
        let path = temp_project("place-rect-stage-geom");
        let host = motolii_ui::host_create_for_test(&path).expect("create host");
        let before = read_host_projection(host);
        assert!(before.primary_layer_id.is_none());
        assert!(
            before
                .stage_geometry
                .as_ref()
                .is_none_or(|geometry| geometry.layers.is_empty())
        );

        dispatch_kind(
            host,
            "place_rectangle",
            r#","position":[0.25,-0.125],"playhead":{"num":0,"den":1}"#,
        );
        let after = read_host_projection(host);
        let primary = after.primary_layer_id.expect("placed rectangle is primary");
        let geometry = after.stage_geometry.expect("stage_geometry after place");
        assert_eq!(geometry.layers.len(), 1);
        assert_eq!(geometry.layers[0].layer_id, primary);
        assert_eq!(geometry.layers[0].position, [0.25, -0.125]);
        assert_eq!(geometry.layers[0].rotation, 0.0);
        assert_eq!(geometry.layers[0].scale, [1.0, 1.0]);
        assert_eq!(after.bounds.len(), 1);
        assert_eq!(after.bounds[0].0, primary);
        let snap = motolii_ui::host_read_snapshot_for_test(host).expect("placed document layer");
        assert!(
            snap.layer_ids.iter().any(|id| id == &primary),
            "place_rectangle must write a Document layer, not only a snapshot field"
        );
        assert!(
            crate::rerun_stage::host_layer_fill_is_visible(
                geometry.layers[0].corners,
                false,
                false
            ),
            "placed rectangle must keep Stage fill until evaluated Image"
        );
        assert!(
            !crate::rerun_stage::host_layer_fill_is_visible(
                geometry.layers[0].corners,
                true,
                false
            ),
            "evaluated Image must hide the opaque fill"
        );
        assert!(
            crate::rerun_stage::host_layer_fill_is_visible(geometry.layers[0].corners, true, true),
            "gizmo preview must keep fill while the evaluated Image is stale"
        );

        install_slot(host);
        let revision = after.revision.parse::<u64>().expect("revision");
        let preview = try_preview_stage_transform(
            revision,
            &primary,
            AppStageTransformEdit::TranslateWorld([0.1, 0.0]),
        )
        .expect("host preview");
        assert_eq!(preview.layers.len(), 1);
        assert_ne!(
            preview.layers[0].corners, geometry.layers[0].corners,
            "gizmo preview must move Document path corners"
        );
        assert!((preview.layers[0].position[0] - 0.35).abs() < 1e-12);
        let unchanged = read_host_projection(host)
            .stage_geometry
            .expect("preview must not mutate live Document");
        assert_eq!(unchanged.layers[0].corners, geometry.layers[0].corners);
        let accepted_terminal = dispatch_commit_stage_transform(
            revision,
            &primary,
            AppStageTransformEdit::TranslateWorld([0.1, 0.0]),
        );
        assert!(accepted_terminal.accepted);
        assert!(accepted_terminal.projection.is_some());
        let committed = read_host_projection(host)
            .stage_geometry
            .expect("stage_geometry after gizmo commit");
        assert_eq!(committed.layers[0].corners, preview.layers[0].corners);
        assert!((committed.layers[0].position[0] - 0.35).abs() < 1e-12);
        assert_eq!(committed.layers[0].rotation, 0.0);

        let committed_rev = read_host_projection(host)
            .revision
            .parse::<u64>()
            .expect("revision after commit");
        let rejected_terminal = dispatch_commit_stage_transform(
            revision,
            &primary,
            AppStageTransformEdit::RotateZ(0.25),
        );
        assert!(!rejected_terminal.accepted);
        assert_eq!(rejected_terminal.diagnostics[0].reason, "stale_document");
        assert_eq!(
            rejected_terminal
                .projection
                .expect("authoritative snapshot")
                .stage_geometry
                .expect("authoritative geometry")
                .layers[0]
                .corners,
            committed.layers[0].corners
        );
        try_commit_stage_transform(
            committed_rev,
            &primary,
            AppStageTransformEdit::RotateZ(0.25),
        )
        .expect("gizmo rotate commit");
        let rotated = read_host_projection(host)
            .stage_geometry
            .expect("stage_geometry after rotate");
        assert!((rotated.layers[0].rotation - 0.25).abs() < 1e-12);
        assert!((rotated.layers[0].position[0] - 0.35).abs() < 1e-12);
        clear_slot();

        motolii_ui::host_destroy_for_test(host).expect("destroy");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn handle_free_rn_ellipse_intent_reaches_document_and_stage_projection() {
        let _lock = test_lock();
        clear_slot();
        let path = temp_project("handle-free-place-ellipse");
        let host = motolii_ui::host_create_for_test(&path).expect("create host");
        install_slot(host);
        let intent = br#"{"version":1,"direction":"rn-to-host","kind":"place_ellipse","position":[0.25,-0.125],"playhead":{"num":0,"den":1}}"#;
        let mut out = [0u8; MAX_SNAPSHOT_JSON_BYTES];

        let written = unsafe {
            motolii_rnapp_host_dispatch_json(
                intent.as_ptr(),
                intent.len(),
                out.as_mut_ptr(),
                out.len(),
            )
        };

        assert!(written > 0);
        let response = std::str::from_utf8(
            slice_from_written(&out, written).expect("dispatch response within buffer"),
        )
        .expect("utf8 response");
        assert!(response.contains(r#""accepted":true"#), "{response}");
        let after = read_host_projection(host);
        assert_eq!(after.revision, "1");
        assert_eq!(after.bounds.len(), 1);
        assert_eq!(after.bounds[0].1, "Ellipse");
        let primary = after.primary_layer_id.expect("placed ellipse primary");
        assert_eq!(after.bounds[0].0, primary);
        let geometry = after.stage_geometry.expect("stage geometry");
        assert_eq!(geometry.layers.len(), 1);
        assert_eq!(geometry.layers[0].layer_id, primary);
        assert_eq!(geometry.layers[0].position, [0.25, -0.125]);

        clear_slot();
        motolii_ui::host_destroy_for_test(host).expect("destroy");
    }

    #[test]
    fn place_vism_snapshot_json_feeds_stage_identity() {
        let _lock = test_lock();
        let path = temp_project("place-vism-stage-id");
        let host = motolii_ui::host_create_for_test(&path).expect("create host");

        dispatch_kind(
            host,
            "place_vism",
            r#","plugin_id":"core.layer_source.radial_repeater","position":[0.0,0.0],"playhead":{"num":0,"den":1}"#,
        );
        let after = read_host_projection(host);
        let primary = after.primary_layer_id.expect("placed vism is primary");
        assert_eq!(after.bounds.len(), 1);
        assert_eq!(after.bounds[0].0, primary);

        motolii_ui::host_destroy_for_test(host).expect("destroy");
    }

    #[test]
    fn inject_host_handle_replaces_empty_value() {
        let patched = inject_host_handle(
            r#"{"version":1,"direction":"rn-to-host","kind":"undo","host_handle":""}"#,
            42,
        )
        .expect("inject");
        assert!(patched.contains(r#""host_handle":"42""#));
    }

    #[test]
    fn inject_host_handle_adds_missing_top_level_field() {
        let patched = inject_host_handle(
            r#"{"version":1,"direction":"rn-to-host","kind":"undo"}"#,
            42,
        )
        .expect("inject");
        assert_eq!(
            patched,
            r#"{"version":1,"direction":"rn-to-host","kind":"undo","host_handle":"42"}"#
        );
    }

    #[test]
    fn inject_host_handle_does_not_rewrite_nested_only_field() {
        let patched = inject_host_handle(
            r#"{"version":1,"direction":"rn-to-host","kind":"undo","meta":{"host_handle":"nested"}}"#,
            42,
        )
        .expect("inject");
        assert!(patched.contains(r#""meta":{"host_handle":"nested"}"#));
        assert!(patched.ends_with(r#","host_handle":"42"}"#));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn dispatch_without_host_returns_the_typed_startup_reject() {
        let _lock = test_lock();
        clear_slot();
        let expected =
            r#"{"version":1,"accepted":false,"diagnostics":[{"reason":"project_already_open"}]}"#;
        *host_startup_reject().lock().expect("startup reject") = Some(expected.to_owned());
        let intent = br#"{"version":1,"direction":"rn-to-host","kind":"place_ellipse"}"#;
        let mut out = [0u8; 256];

        let written = unsafe {
            motolii_rnapp_host_dispatch_json(
                intent.as_ptr(),
                intent.len(),
                out.as_mut_ptr(),
                out.len(),
            )
        };

        assert!(written > 0);
        assert_eq!(&out[..written as usize], expected.as_bytes());
        *host_startup_reject().lock().expect("startup reject") = None;
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn host_lifecycle_identity_reuses_same_path_and_tombstones_shutdown_handle() {
        let _lock = test_lock();
        clear_slot();
        *host_startup_reject().lock().expect("startup reject") = None;
        let first_path = temp_project("lifecycle-first");
        let first_bytes = first_path.to_string_lossy();

        assert!(unsafe { motolii_rnapp_host_ensure(first_bytes.as_ptr(), first_bytes.len()) });
        let first_host = try_host_handle().expect("first host");
        assert!(unsafe { motolii_rnapp_host_ensure(first_bytes.as_ptr(), first_bytes.len()) });
        assert_eq!(try_host_handle(), Some(first_host));

        assert!(try_stage_mount(1600.0, 900.0, 1.0));
        let first_stage = host_slot()
            .lock()
            .expect("slot")
            .as_ref()
            .and_then(|slot| slot.stage_handle)
            .expect("stage");
        assert!(try_stage_unmount());
        assert!(try_stage_mount(1600.0, 900.0, 1.0));
        {
            let remounted = host_slot().lock().expect("slot");
            let remounted = remounted.as_ref().expect("remounted slot");
            assert_eq!(remounted.handle, first_host);
            assert_eq!(remounted.stage_handle, Some(first_stage));
        }

        let second_path = temp_project("lifecycle-second");
        let second_bytes = second_path.to_string_lossy();
        assert!(!unsafe { motolii_rnapp_host_ensure(second_bytes.as_ptr(), second_bytes.len()) });
        assert_eq!(try_host_handle(), Some(first_host));

        assert!(try_host_shutdown());
        assert_eq!(try_host_handle(), None);
        assert!(try_host_shutdown(), "shutdown without a slot is idempotent");

        let mut out = [0u8; 1024];
        let written = motolii_rn_host_read_snapshot_json(first_host, out.as_mut_ptr(), out.len());
        let rejected = std::str::from_utf8(
            slice_from_written(&out, written).expect("destroyed host response"),
        )
        .expect("destroyed host utf8");
        assert!(rejected.contains(r#""accepted":false"#));
        assert!(rejected.contains(r#""reason":"destroyed_host_handle""#));

        assert!(unsafe { motolii_rnapp_host_ensure(second_bytes.as_ptr(), second_bytes.len()) });
        let second_host = try_host_handle().expect("second host");
        assert_ne!(second_host, first_host, "host handles stay monotonic");
        assert!(try_host_shutdown());
    }

    #[test]
    fn inject_host_handle_rewrites_only_top_level_not_nested() {
        let patched = inject_host_handle(
            concat!(
                r#"{"version":1,"direction":"rn-to-host","kind":"set_effect_param","#,
                r#""host_handle":"","meta":{"host_handle":"nested"},"target":"1"}"#
            ),
            99,
        )
        .expect("inject");
        assert!(patched.contains(r#""host_handle":"99""#));
        assert!(patched.contains(r#""host_handle":"nested""#));
        // top-levelだけが99。nestedは維持。
        let top = patched.find(r#""host_handle":"99""#).expect("top");
        let nested = patched.find(r#""host_handle":"nested""#).expect("nested");
        assert!(top < nested);
    }

    #[test]
    fn parse_projection_reads_bounds_and_revision() {
        let json = r#"{
            "version":1,
            "direction":"host-to-rn",
            "role":"product",
            "host_handle":"1",
            "revision":"3",
            "projection_generation":"0",
            "current_time":{"num":0,"den":1},
            "primary_layer_id":"L1",
            "stage":{"selection":[],"bounds":[
                {"layer_id":"L1","display_name":"rect \"A\""},
                {"layer_id":"L2","display_name":"rect \n"}
            ]},
            "diagnostics":[]
        }"#;
        let proj = parse_timeline_projection(json).expect("parse");
        assert_eq!(proj.revision, "3");
        assert_eq!(proj.projection_generation, "0");
        assert_eq!(proj.current_time, (0, 1));
        assert!(proj.fps.is_none());
        assert_eq!(proj.primary_layer_id.as_deref(), Some("L1"));
        assert_eq!(
            proj.bounds,
            vec![
                ("L1".into(), r#"rect "A""#.into()),
                ("L2".into(), "rect \n".into())
            ]
        );
    }

    #[test]
    fn parse_projection_from_host_snapshot_json_matches_wire_projection() {
        let _lock = test_lock();
        let path = temp_project("projection-json-wire");
        let host = motolii_ui::host_create_for_test(&path).expect("create host");
        let baseline = motolii_ui::host_read_snapshot_for_test(host).expect("baseline");
        let mut out = [0u8; MAX_SNAPSHOT_JSON_BYTES];
        let written =
            unsafe { motolii_rn_host_read_snapshot_json(host, out.as_mut_ptr(), out.len()) };
        assert!(written > 0, "host snapshot json failed: {written}");
        let json_bytes =
            slice_from_written(&out, written).expect("snapshot response within buffer");
        let json = std::str::from_utf8(json_bytes).expect("snapshot json");
        let proj = parse_timeline_projection(json).expect("projection parse");
        assert_eq!(proj.revision, baseline.revision);
        assert_eq!(proj.primary_layer_id, baseline.primary_layer_id);
        assert_eq!(proj.bounds.len(), baseline.layer_ids.len());
        for (idx, layer_id) in baseline.layer_ids.iter().enumerate() {
            assert_eq!(proj.bounds[idx].0, *layer_id);
        }
        let timeline = proj.timeline_layers.expect("timeline from host");
        assert_eq!(timeline.len(), baseline.timeline.layers.len());
        if !timeline.is_empty() {
            assert_eq!(timeline[0].layer_id, baseline.timeline.layers[0].layer_id);
        }
        assert_eq!(
            proj.fps,
            Some((baseline.timeline.fps.num(), baseline.timeline.fps.den()))
        );
        assert_eq!(
            proj.current_time,
            (baseline.current_time.num(), baseline.current_time.den())
        );

        motolii_ui::host_destroy_for_test(host).expect("destroy");
    }

    #[test]
    fn parse_projection_parses_layer_ids_and_falls_back_without_timeline_on_bad_key() {
        let json = r#"{
            "version":1,
            "direction":"host-to-rn",
            "role":"product",
            "host_handle":"1",
            "revision":"9",
            "projection_generation":"0",
            "current_time":{"num":0,"den":1},
            "primary_layer_id":"L1",
            "stage":{"selection":[],"bounds":[
                {"layer_id":"L1","display_name":"rect"}
            ]},
            "timeline":{
                "fps":{"num":30,"den":1},
                "layers":[
                    {
                        "layer_id":"L1",
                        "display_name":"rect",
                        "start":{"num":0,"den":1},
                        "duration":{"num":10,"den":1},
                        "position_keys":[
                            {"key_id":"NaN","time":{"num":4,"den":1}}
                        ],
                        "keys_truncated":false
                    }
                ],
                "layers_truncated":false
            },
            "diagnostics":[]
        }"#;
        let proj = parse_timeline_projection(json).expect("parse");
        assert_eq!(proj.bounds, vec![("L1".into(), "rect".into())]);
        assert!(proj.timeline_layers.is_none());
    }

    #[test]
    fn timeline_json_maps_to_scene_bars_and_keeps_key_id() {
        let json = r#"{
            "version":1,
            "direction":"host-to-rn",
            "role":"product",
            "host_handle":"1",
            "revision":"7",
            "projection_generation":"0",
            "current_time":{"num":0,"den":1},
            "primary_layer_id":"11",
            "stage":{"selection":[],"bounds":[
                {"layer_id":"11","display_name":"rect"}
            ]},
            "timeline":{
                "fps":{"num":30,"den":1},
                "layers":[{
                    "layer_id":"11",
                    "display_name":"rect",
                    "start":{"num":0,"den":1},
                    "duration":{"num":10,"den":1},
                    "position_keys":[{"key_id":"42","time":{"num":4,"den":1}}],
                    "keys_truncated":false
                }],
                "layers_truncated":false
            },
            "diagnostics":[]
        }"#;
        let proj = parse_timeline_projection(json).expect("parse");
        let layers = snapshot_layers_from_projection(&proj);
        assert_eq!(crate::timeline_skia::SECONDS_PER_BAR, 1.0);
        let duration_secs = 10.0;
        let song_bars = duration_secs / crate::timeline_skia::SECONDS_PER_BAR;
        assert!((song_bars - duration_secs).abs() < 1e-12);
        let scene = TimelineScene::from_snapshot_with_song_bars(
            &layers,
            proj.primary_layer_id.as_deref(),
            song_bars as f32,
        );
        let (a, b, keys) = scene.clip0_span_and_keys(0).expect("clip0");
        assert!((a - 0.0).abs() < 1e-6);
        assert!((f64::from(b) - duration_secs).abs() < 1e-6);
        assert_eq!(keys.len(), 1);
        assert!((keys[0].0 - 4.0).abs() < 1e-6);
        assert_eq!(keys[0].1, 42);
        assert_eq!(scene.selected_flat, 0);
        assert!((f64::from(scene.song_bars) - duration_secs).abs() < 1e-6);
        assert!((scene.view_a - 0.0).abs() < 1e-6);
        assert!((f64::from(scene.view_b) - duration_secs).abs() < 1e-6);
        assert_eq!(proj.fps, Some((30, 1)));
        assert_eq!(frame_from_scrub_bar(duration_secs, 30, 1), 300);
        assert_eq!(frame_from_scrub_bar(f64::from(keys[0].0), 30, 1), 120);
    }

    #[test]
    fn position_keys_parse_optional_value_without_requiring_it() {
        let with_value = r#"{
            "version":1,
            "direction":"host-to-rn",
            "role":"product",
            "host_handle":"1",
            "revision":"7",
            "projection_generation":"0",
            "current_time":{"num":0,"den":1},
            "primary_layer_id":"11",
            "stage":{"selection":[],"bounds":[
                {"layer_id":"11","display_name":"rect"}
            ]},
            "timeline":{
                "fps":{"num":30,"den":1},
                "layers":[{
                    "layer_id":"11",
                    "display_name":"rect",
                    "start":{"num":0,"den":1},
                    "duration":{"num":10,"den":1},
                    "position_keys":[
                        {"key_id":"42","time":{"num":4,"den":1},"value":[0.25,-0.5]},
                        {"key_id":"43","time":{"num":5,"den":1}}
                    ],
                    "keys_truncated":false
                }],
                "layers_truncated":false
            },
            "diagnostics":[]
        }"#;
        let proj = parse_timeline_projection(with_value).expect("parse");
        let layers = proj.timeline_layers.expect("layers");
        assert_eq!(layers[0].position_keys.len(), 2);
        assert_eq!(layers[0].position_keys[0].value, Some([0.25, -0.5]));
        assert_eq!(layers[0].position_keys[1].value, None);
        assert_eq!(layers[0].position_keys[0].key_id, 42);
        assert_eq!(layers[0].position_keys[1].key_id, 43);
    }

    #[test]
    fn param_keys_union_into_scene_keys() {
        let json = r#"{
            "version":1,
            "direction":"host-to-rn",
            "role":"product",
            "host_handle":"1",
            "revision":"7",
            "projection_generation":"0",
            "current_time":{"num":0,"den":1},
            "primary_layer_id":"11",
            "stage":{"selection":[],"bounds":[
                {"layer_id":"11","display_name":"rect"}
            ]},
            "timeline":{
                "fps":{"num":30,"den":1},
                "layers":[{
                    "layer_id":"11",
                    "display_name":"rect",
                    "start":{"num":0,"den":1},
                    "duration":{"num":10,"den":1},
                    "position_keys":[{"key_id":"42","time":{"num":4,"den":1}}],
                    "param_keys":[
                        {"property":"scale","key_id":"99","time":{"num":2,"den":1},"vec":[1.0,1.0]},
                        {"property":"opacity","key_id":"100","time":{"num":6,"den":1},"value":0.5}
                    ],
                    "keys_truncated":false
                }],
                "layers_truncated":false
            },
            "diagnostics":[]
        }"#;
        let proj = parse_timeline_projection(json).expect("parse");
        let host_layers = proj.timeline_layers.as_ref().expect("layers");
        assert_eq!(host_layers[0].position_keys.len(), 1);
        assert_eq!(host_layers[0].param_keys.len(), 2);
        assert_eq!(host_layers[0].param_keys[0].key_id, 99);
        assert_eq!(host_layers[0].param_keys[1].key_id, 100);
        assert!(super::layer_has_position_key(host_layers, "11", 42));
        assert!(!super::layer_has_position_key(host_layers, "11", 99));
        assert!(!super::layer_has_position_key(host_layers, "11", 100));
        let layers = snapshot_layers_from_projection(&proj);
        let scene = TimelineScene::from_snapshot_with_song_bars(
            &layers,
            proj.primary_layer_id.as_deref(),
            10.0,
        );
        let (_, _, keys) = scene.clip0_span_and_keys(0).expect("clip0");
        assert_eq!(keys.len(), 3);
        assert!(
            keys.iter()
                .any(|key| key.1 == 42 && (key.0 - 4.0).abs() < 1e-6)
        );
        assert!(
            keys.iter()
                .any(|key| key.1 == 99 && (key.0 - 2.0).abs() < 1e-6)
        );
        assert!(
            keys.iter()
                .any(|key| key.1 == 100 && (key.0 - 6.0).abs() < 1e-6)
        );
    }

    #[test]
    fn param_key_id_does_not_dispatch_set_position_key_time() {
        let _lock = test_lock();
        clear_slot();
        let path = temp_project("param-key-drag-noop");
        let host = motolii_ui::host_create_for_test(&path).expect("create host");
        install_slot(host);
        dispatch_kind(
            host,
            "place_rectangle",
            r#","position":[0.25,-0.125],"playhead":{"num":0,"den":1}"#,
        );
        let placed = motolii_ui::host_read_snapshot_for_test(host).expect("placed");
        let layer_id = placed.primary_layer_id.expect("primary after place");
        dispatch_kind(
            host,
            "add_position_key",
            &format!(r#","target":"{layer_id}","time":{{"num":1,"den":1}}"#),
        );
        let keyed = motolii_ui::host_read_snapshot_for_test(host).expect("keyed");
        let position_key_id = keyed
            .timeline
            .layers
            .iter()
            .find(|layer| layer.layer_id == layer_id)
            .expect("keyed layer")
            .position_keys
            .last()
            .expect("added key")
            .key_id
            .parse::<u64>()
            .expect("key id");
        let param_key_id = position_key_id.wrapping_add(1_000_003);
        super::TEST_SET_POSITION_KEY_TIME_DISPATCH_COUNT.store(0, Ordering::SeqCst);
        assert!(
            try_dispatch_timeline_edit(
                &crate::timeline_skia::TimelineEditCommit::SetPositionKeyTime {
                    layer_id: layer_id.clone(),
                    key_id: param_key_id,
                    bar: 2.0,
                }
            )
            .is_none()
        );
        assert_eq!(
            super::TEST_SET_POSITION_KEY_TIME_DISPATCH_COUNT.load(Ordering::SeqCst),
            0
        );
        let _ = try_dispatch_timeline_edit(
            &crate::timeline_skia::TimelineEditCommit::SetPositionKeyTime {
                layer_id,
                key_id: position_key_id,
                bar: 2.0,
            },
        );
        assert_eq!(
            super::TEST_SET_POSITION_KEY_TIME_DISPATCH_COUNT.load(Ordering::SeqCst),
            1
        );

        clear_slot();
        motolii_ui::host_destroy_for_test(host).expect("destroy");
    }

    #[test]
    fn missing_timeline_falls_back_to_full_width_rows() {
        let json = r#"{
            "version":1,
            "direction":"host-to-rn",
            "role":"product",
            "host_handle":"1",
            "revision":"3",
            "projection_generation":"0",
            "current_time":{"num":0,"den":1},
            "primary_layer_id":"L1",
            "stage":{"selection":[],"bounds":[
                {"layer_id":"L1","display_name":"rect \"A\""},
                {"layer_id":"L2","display_name":"rect \n"}
            ]},
            "diagnostics":[]
        }"#;
        let proj = parse_timeline_projection(json).expect("parse");
        assert!(proj.timeline_layers.is_none());
        let layers = snapshot_layers_from_projection(&proj);
        let scene = TimelineScene::from_snapshot_with_song_bars(
            &layers,
            proj.primary_layer_id.as_deref(),
            (10.0 / crate::timeline_skia::SECONDS_PER_BAR) as f32,
        );
        let (a, b, keys) = scene.clip0_span_and_keys(0).expect("clip0");
        assert!((a - 0.0).abs() < 1e-6);
        assert!((b - 10.0).abs() < 1e-6);
        assert!(keys.is_empty());
        assert_eq!(scene.band_count(), 2);
    }

    #[test]
    fn frame_from_scrub_bar_rounds_at_fps_30_and_24_boundaries() {
        assert_eq!(crate::timeline_skia::SECONDS_PER_BAR, 1.0);
        // 1s → fps30 frame 30、fps24 frame 24。
        assert_eq!(frame_from_scrub_bar(1.0, 30, 1), 30);
        assert_eq!(frame_from_scrub_bar(1.0, 24, 1), 24);
        // 0.5s → fps30 frame 15、fps24 frame 12。
        assert_eq!(frame_from_scrub_bar(0.5, 30, 1), 15);
        assert_eq!(frame_from_scrub_bar(0.5, 24, 1), 12);
        // 0.5 frame は最近傍で 1（24fps も同じ。µs先丸めだと 0 になっていた）
        assert_eq!(frame_from_scrub_bar(0.5 / 30.0, 30, 1), 1);
        assert_eq!(frame_from_scrub_bar(0.5 / 24.0, 24, 1), 1);
        // 直前は0
        assert_eq!(frame_from_scrub_bar(0.49 / 30.0, 30, 1), 0);
        assert_eq!(frame_from_scrub_bar(0.49 / 24.0, 24, 1), 0);
        assert_eq!(frame_from_scrub_bar(0.5, 30, 0), 0);
        assert_eq!(frame_from_scrub_bar(0.5, 0, 1), 0);
    }

    #[test]
    fn rational_time_parts_from_bar_emits_seconds_not_ableton_bars() {
        assert_eq!(crate::timeline_skia::SECONDS_PER_BAR, 1.0);
        assert_eq!(rational_time_parts_from_bar(0.0), (0, 1_000_000));
        assert_eq!(rational_time_parts_from_bar(1.0), (1_000_000, 1_000_000));
        assert_eq!(rational_time_parts_from_bar(0.5), (500_000, 1_000_000));
        assert_eq!(rational_time_parts_from_bar(10.0), (10_000_000, 1_000_000));
    }

    #[test]
    fn parse_stage_geometry_reads_corners() {
        let json = r#"{
            "version":1,
            "direction":"host-to-rn",
            "role":"product",
            "host_handle":"1",
            "revision":"3",
            "projection_generation":"0",
            "current_time":{"num":0,"den":1},
            "stage":{"selection":[],"bounds":[{"layer_id":"1","display_name":"r"}]},
            "stage_geometry":{
                "layers":[{
                    "layer_id":"1",
                    "corners":[[-0.5,-0.5],[0.5,-0.5],[0.5,0.5],[-0.5,0.5]]
                }],
                "layers_truncated":false
            },
            "diagnostics":[]
        }"#;
        let proj = parse_timeline_projection(json).expect("parse");
        let geom = proj.stage_geometry.expect("stage_geometry");
        assert!(!geom.layers_truncated);
        assert_eq!(geom.layers.len(), 1);
        assert_eq!(geom.layers[0].layer_id, "1");
        assert_eq!(
            geom.layers[0].corners,
            [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]]
        );
    }

    #[test]
    fn parse_stage_geometry_falls_back_to_none_when_layers_truncated_missing() {
        let json = r#"{
            "version":1,
            "direction":"host-to-rn",
            "role":"product",
            "host_handle":"1",
            "revision":"3",
            "projection_generation":"0",
            "current_time":{"num":0,"den":1},
            "stage":{"selection":[],"bounds":[{"layer_id":"1","display_name":"r"}]},
            "stage_geometry":{
                "layers":[
                    {"layer_id":"1","corners":[[-0.5,-0.5],[0.5,-0.5],[0.5,0.5],[-0.5,0.5]]}
                ]
            },
            "diagnostics":[]
        }"#;
        let proj = parse_timeline_projection(json).expect("parse");
        assert!(proj.stage_geometry.is_none());
    }

    #[test]
    fn parse_stage_geometry_falls_back_to_none_when_layers_truncated_is_not_bool() {
        let json = r#"{
            "version":1,
            "direction":"host-to-rn",
            "role":"product",
            "host_handle":"1",
            "revision":"3",
            "projection_generation":"0",
            "current_time":{"num":0,"den":1},
            "stage":{"selection":[],"bounds":[{"layer_id":"1","display_name":"r"}]},
            "stage_geometry":{
                "layers":[
                    {"layer_id":"1","corners":[[-0.5,-0.5],[0.5,-0.5],[0.5,0.5],[-0.5,0.5]]}
                ],
                "layers_truncated":"false"
            },
            "diagnostics":[]
        }"#;
        let proj = parse_timeline_projection(json).expect("parse");
        assert!(proj.stage_geometry.is_none());
    }

    #[test]
    fn parse_stage_geometry_falls_back_to_none_on_three_point_corners() {
        let json = r#"{
            "version":1,
            "direction":"host-to-rn",
            "role":"product",
            "host_handle":"1",
            "revision":"3",
            "projection_generation":"0",
            "current_time":{"num":0,"den":1},
            "stage":{"selection":[],"bounds":[{"layer_id":"1","display_name":"r"}]},
            "stage_geometry":{
                "layers":[{
                    "layer_id":"1",
                    "corners":[[-0.5,-0.5],[0.5,-0.5],[0.5,0.5]]
                }],
                "layers_truncated":false
            },
            "diagnostics":[]
        }"#;
        let proj = parse_timeline_projection(json).expect("parse");
        assert!(proj.stage_geometry.is_none());
    }

    #[test]
    fn parse_stage_geometry_falls_back_to_none_with_infinite_corner() {
        let json = r#"{
            "version":1,
            "direction":"host-to-rn",
            "role":"product",
            "host_handle":"1",
            "revision":"3",
            "projection_generation":"0",
            "current_time":{"num":0,"den":1},
            "stage":{"selection":[],"bounds":[{"layer_id":"1","display_name":"r"}]},
            "stage_geometry":{
                "layers":[{
                    "layer_id":"1",
                    "corners":[[-0.5,-0.5],[0.5,-0.5],[0.5,"inf"],[-0.5,0.5]]
                }],
                "layers_truncated":false
            },
            "diagnostics":[]
        }"#;
        let proj = parse_timeline_projection(json).expect("parse");
        assert!(proj.stage_geometry.is_none());
    }

    #[test]
    fn parse_stage_geometry_falls_back_to_none_on_broken_corners() {
        let json = r#"{
            "version":1,
            "direction":"host-to-rn",
            "role":"product",
            "host_handle":"1",
            "revision":"3",
            "projection_generation":"0",
            "current_time":{"num":0,"den":1},
            "stage":{"selection":[],"bounds":[{"layer_id":"1","display_name":"r"}]},
            "stage_geometry":{
                "layers":[{
                    "layer_id":"1",
                    "corners":[[-0.5,-0.5],[0.5,-0.5]]
                }],
                "layers_truncated":false
            },
            "diagnostics":[]
        }"#;
        let proj = parse_timeline_projection(json).expect("parse");
        assert!(proj.stage_geometry.is_none());
        assert_eq!(proj.revision, "3");
    }

    #[test]
    fn set_time_dispatch_requires_accepted_response() {
        assert!(response_is_accepted(r#"{"accepted":true}"#));
        assert!(!response_is_accepted(r#"{"accepted":false}"#));
        assert!(!response_is_accepted(r#"{"foo":true}"#));
        assert!(!response_is_accepted(r#"not-json"#));
    }

    #[test]
    fn terminal_response_keeps_typed_diagnostic_and_authoritative_snapshot() {
        let response = r#"{
            "accepted":false,
            "snapshot":{
                "revision":"9",
                "projection_generation":"5",
                "current_time":{"num":3,"den":1},
                "stage":{"selection":[],"bounds":[]},
                "stage_geometry":{"layers":[],"layers_truncated":false},
                "timeline":{"duration":{"num":10,"den":1},"fps":{"num":30,"den":1},"layers":[],"layers_truncated":false},
                "diagnostics":[{"reason":"snapshot_only"}]
            },
            "diagnostics":[{
                "reason":"stale_projection_generation",
                "expected_projection_generation":"4",
                "actual_projection_generation":"5"
            }],
            "message":"stale terminal edit"
        }"#;
        let result = parse_terminal_result(response).expect("terminal response");
        assert!(!result.accepted);
        assert_eq!(result.feedback(), Some("stale terminal edit"));
        assert_eq!(result.stamp(), Some((9, 5)));
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0],
            HostTerminalDiagnostic {
                reason: "stale_projection_generation".into(),
                host_handle: None,
                stage_handle: None,
                timeline_handle: None,
                expected_projection_generation: Some("4".into()),
                actual_projection_generation: Some("5".into()),
            }
        );
        assert_eq!(result.projection.expect("snapshot").current_time, (3, 1));
    }

    #[test]
    fn playhead_from_current_time_uses_seconds_not_bars() {
        assert_eq!(crate::timeline_skia::SECONDS_PER_BAR, 1.0);
        // 2s / fixture曲長96秒 → 2/96。Ableton 1bar=2s なら 1/96 になる。
        let ph_fixture = playhead_from_current_time(2, 1, crate::timeline_skia::SONG_BARS);
        assert!((ph_fixture - 2.0 / 96.0).abs() < 1e-12);
        let ph_10s = playhead_from_current_time(2, 1, 10.0);
        assert!((ph_10s - 2.0 / 10.0).abs() < 1e-12);
        assert_eq!(playhead_from_current_time(0, 1, 10.0), 0.0);
    }

    #[test]
    fn parse_timeline_projection_parses_timeline_duration() {
        let json = r#"{
            "version":1,
            "direction":"host-to-rn",
            "role":"product",
            "host_handle":"1",
            "revision":"7",
            "projection_generation":"0",
            "current_time":{"num":0,"den":1},
            "primary_layer_id":"11",
            "stage":{"selection":[],"bounds":[{"layer_id":"11","display_name":"rect"}]},
            "timeline":{
                "duration":{"num":40,"den":1},
                "fps":{"num":30,"den":1},
                "layers":[{
                    "layer_id":"11",
                    "display_name":"rect",
                    "start":{"num":0,"den":1},
                    "duration":{"num":40,"den":1},
                    "position_keys":[],
                    "keys_truncated":false
                }],
                "layers_truncated":false
            },
            "diagnostics":[ ]
        }"#;
        let projection = parse_timeline_projection(json).expect("parse");
        assert_eq!(projection.timeline_duration, Some((40, 1)));
    }

    #[test]
    fn set_time_dispatch_moves_current_time_via_host_slot() {
        let _lock = test_lock();
        clear_slot();
        let path = temp_project("set-time-scrub");
        let host = motolii_ui::host_create_for_test(&path).expect("create host");
        install_slot(host);

        let baseline = motolii_ui::host_read_snapshot_for_test(host).expect("baseline");
        let fps_num = baseline.timeline.fps.num();
        let fps_den = baseline.timeline.fps.den();
        assert_eq!((fps_num, fps_den), (30, 1));

        // 1s → frame 30。既定fpsで往復一致。
        let frame = frame_from_scrub_bar(1.0, fps_num, fps_den);
        assert_eq!(frame, 30);
        let terminal = try_dispatch_set_time(frame).expect("terminal");
        assert!(terminal.accepted);
        assert!(terminal.diagnostics.is_empty());
        assert_eq!(
            terminal.projection.as_ref().expect("snapshot").current_time,
            (1, 1)
        );
        assert_eq!(terminal.stamp(), Some((0, 1)));
        let after = motolii_ui::host_read_snapshot_for_test(host).expect("after");
        assert_eq!(after.current_time.num(), 1);
        assert_eq!(after.current_time.den(), 1);
        let ph =
            playhead_from_current_time(after.current_time.num(), after.current_time.den(), 10.0);
        assert!((ph - 1.0 / 10.0).abs() < 1e-12);

        clear_slot();
        motolii_ui::host_destroy_for_test(host).expect("destroy");
    }

    #[test]
    fn shuttle_reverse_keymap_steps_playhead_via_host_slot() {
        let _lock = test_lock();
        clear_slot();
        let path = temp_project("shuttle-reverse-keymap");
        let host = motolii_ui::host_create_for_test(&path).expect("create host");
        install_slot(host);

        assert!(try_dispatch_set_time(45).is_some_and(|result| result.accepted));
        let chars = b"j";
        let consumed = unsafe {
            motolii_rnapp_host_key_event(38, 0, chars.as_ptr(), chars.len(), false, false)
        };
        assert_eq!(
            consumed, 1,
            "J must reach host_key_event → try_dispatch_keymap"
        );
        let after = motolii_ui::host_read_snapshot_for_test(host).expect("after");
        assert_eq!(after.current_time.num(), 22);
        assert_eq!(after.current_time.den(), 15);

        clear_slot();
        motolii_ui::host_destroy_for_test(host).expect("destroy");
    }

    #[test]
    fn set_time_is_not_dispatched_without_host_slot() {
        let _lock = test_lock();
        clear_slot();
        assert!(try_dispatch_set_time(60).is_none());
    }

    #[test]
    fn add_position_key_grows_wire_timeline_keys() {
        let _lock = test_lock();
        let path = temp_project("add-pos-key");
        let host = motolii_ui::host_create_for_test(&path).expect("create host");
        // placeでprimaryを載せ、add_position_keyはtarget+time必須(rn_product_host 718-747)。
        dispatch_kind(
            host,
            "place_rectangle",
            r#","position":[0.25,-0.125],"playhead":{"num":0,"den":1}"#,
        );
        let placed = motolii_ui::host_read_snapshot_for_test(host).expect("placed");
        let layer_id = placed.primary_layer_id.expect("primary after place");
        let before_keys = placed
            .timeline
            .layers
            .iter()
            .find(|layer| layer.layer_id == layer_id)
            .map(|layer| layer.position_keys.len())
            .unwrap_or(0);

        dispatch_kind(
            host,
            "add_position_key",
            &format!(r#","target":"{layer_id}","time":{{"num":1,"den":1}}"#),
        );
        let after = motolii_ui::host_read_snapshot_for_test(host).expect("after");
        let layer = after
            .timeline
            .layers
            .iter()
            .find(|layer| layer.layer_id == layer_id)
            .expect("keyed layer");
        assert_eq!(layer.position_keys.len(), before_keys + 1);
        assert_eq!(layer.position_keys.last().unwrap().time.num(), 1);
        assert_eq!(layer.position_keys.last().unwrap().time.den(), 1);

        motolii_ui::host_destroy_for_test(host).expect("destroy");
    }

    #[test]
    fn timeline_edit_commit_remove_position_key_clears_wire_keys() {
        let _lock = test_lock();
        clear_slot();
        let path = temp_project("remove-pos-key-commit");
        let host = motolii_ui::host_create_for_test(&path).expect("create host");
        install_slot(host);
        dispatch_kind(
            host,
            "place_rectangle",
            r#","position":[0.25,-0.125],"playhead":{"num":0,"den":1}"#,
        );
        let placed = motolii_ui::host_read_snapshot_for_test(host).expect("placed");
        let layer_id = placed.primary_layer_id.expect("primary after place");
        dispatch_kind(
            host,
            "add_position_key",
            &format!(r#","target":"{layer_id}","time":{{"num":1,"den":1}}"#),
        );
        let keyed = motolii_ui::host_read_snapshot_for_test(host).expect("keyed");
        let key_id = keyed
            .timeline
            .layers
            .iter()
            .find(|layer| layer.layer_id == layer_id)
            .expect("keyed layer")
            .position_keys
            .last()
            .expect("added key")
            .key_id
            .parse::<u64>()
            .expect("key id");
        let terminal = try_dispatch_timeline_edit(
            &crate::timeline_skia::TimelineEditCommit::RemovePositionKey {
                layer_id: layer_id.clone(),
                key_id,
            },
        )
        .expect("terminal");
        assert!(terminal.accepted);
        let terminal_layer = terminal
            .projection
            .as_ref()
            .and_then(|projection| projection.timeline_layers.as_ref())
            .and_then(|layers| layers.iter().find(|layer| layer.layer_id == layer_id))
            .expect("terminal layer");
        assert!(terminal_layer.position_keys.is_empty());
        let after = motolii_ui::host_read_snapshot_for_test(host).expect("after");
        let layer = after
            .timeline
            .layers
            .iter()
            .find(|layer| layer.layer_id == layer_id)
            .expect("layer");
        assert!(layer.position_keys.is_empty());

        clear_slot();
        motolii_ui::host_destroy_for_test(host).expect("destroy");
    }

    #[test]
    fn stage_seat_mount_pointer_selects_seed_layer_at_center() {
        let _lock = test_lock();
        clear_slot();
        let path = temp_project("stage-ptr-hit");
        let host = motolii_ui::host_create_for_test(&path).expect("create host");
        install_slot(host);
        let baseline = motolii_ui::host_read_snapshot_for_test(host).expect("baseline");
        assert!(baseline.primary_layer_id.is_none());
        assert!(baseline.layer_ids.is_empty());

        let (width, height) = (1600.0_f64, 900.0_f64);
        assert!(try_stage_mount(width, height, 1.0));
        // seed rect center=[0,0] → view-local (w/2, h/2)
        assert!(try_stage_pointer("down", width * 0.5, height * 0.5));
        let after = try_read_timeline_projection().expect("projection");
        assert!(after.primary_layer_id.is_none());

        clear_slot();
        motolii_ui::host_destroy_for_test(host).expect("destroy");
    }

    #[test]
    fn stage_pointer_rejects_when_unmounted_and_keeps_selection() {
        let _lock = test_lock();
        clear_slot();
        let path = temp_project("stage-ptr-unmounted");
        let host = motolii_ui::host_create_for_test(&path).expect("create host");
        install_slot(host);
        dispatch_kind(
            host,
            "place_rectangle",
            r#","position":[0.25,-0.125],"playhead":{"num":0,"den":1}"#,
        );
        assert!(try_stage_mount(1600.0, 900.0, 1.0));
        assert!(try_stage_pointer("down", 800.0, 450.0));
        let selected = try_read_timeline_projection()
            .expect("selected")
            .primary_layer_id;
        assert!(try_stage_unmount());
        assert!(!try_stage_pointer("down", 800.0, 450.0));
        let after = try_read_timeline_projection().expect("after");
        assert_eq!(after.primary_layer_id, selected);

        clear_slot();
        motolii_ui::host_destroy_for_test(host).expect("destroy");
    }

    #[test]
    fn stage_pointer_state_machine_blocks_invalid_transitions_and_reopens_after_cancel() {
        let _lock = test_lock();
        clear_slot();
        let path = temp_project("stage-ptr-seq");
        let host = motolii_ui::host_create_for_test(&path).expect("create host");
        install_slot(host);
        assert!(try_stage_mount(800.0, 600.0, 1.0));
        assert!(!try_stage_pointer("drag", 12.0, 14.0));
        assert!(!try_stage_pointer("up", 12.0, 14.0));
        assert!(try_stage_pointer("down", 10.0, 10.0));
        assert!(try_stage_pointer("drag", 12.0, 14.0));
        assert!(try_stage_pointer("cancel", 12.0, 14.0));
        assert!(!try_stage_pointer("drag", 12.0, 14.0));
        assert!(!try_stage_pointer("up", 12.0, 14.0));
        assert!(try_stage_pointer("down", 10.0, 10.0));
        assert!(try_stage_pointer("drag", 12.0, 14.0));
        assert!(try_stage_pointer("up", 12.0, 14.0));
        let seq = {
            let guard = host_slot().lock().expect("slot");
            guard.as_ref().expect("slot").pointer_sequence
        };
        assert_eq!(seq, 6);

        clear_slot();
        motolii_ui::host_destroy_for_test(host).expect("destroy");
    }

    #[test]
    fn stage_resize_registers_and_mounts_when_unregistered_or_unmounted() {
        let _lock = test_lock();
        clear_slot();
        let path = temp_project("stage-resize-retry");
        let host = motolii_ui::host_create_for_test(&path).expect("create host");
        install_slot(host);
        assert!(try_stage_resize(800.0, 600.0, 1.0));
        assert!(try_stage_unmount());
        assert!(try_stage_resize(1024.0, 768.0, 1.0));

        clear_slot();
        motolii_ui::host_destroy_for_test(host).expect("destroy");
    }

    #[test]
    fn timeline_selection_dispatch_selects_and_clears_via_host_slot() {
        let _lock = test_lock();
        clear_slot();
        let path = temp_project("tl-selection");
        let host = motolii_ui::host_create_for_test(&path).expect("create host");
        install_slot(host);
        dispatch_kind(
            host,
            "place_rectangle",
            r#","position":[0.25,-0.125],"playhead":{"num":0,"den":1}"#,
        );
        let placed = motolii_ui::host_read_snapshot_for_test(host).expect("placed");
        let layer_id = placed.primary_layer_id.expect("primary after place");

        let cleared_terminal = try_dispatch_timeline_selection(
            &crate::timeline_skia::TimelineSelectionCommit::ClearSelection,
        )
        .expect("clear terminal");
        assert!(cleared_terminal.accepted);
        assert!(
            cleared_terminal
                .projection
                .expect("clear snapshot")
                .primary_layer_id
                .is_none()
        );
        let cleared = try_read_timeline_projection().expect("cleared");
        assert!(cleared.primary_layer_id.is_none());

        let selected_terminal = try_dispatch_timeline_selection(
            &crate::timeline_skia::TimelineSelectionCommit::SelectLayer {
                layer_id: layer_id.clone(),
            },
        )
        .expect("select terminal");
        assert!(selected_terminal.accepted);
        assert_eq!(
            selected_terminal
                .projection
                .expect("select snapshot")
                .primary_layer_id
                .as_deref(),
            Some(layer_id.as_str())
        );
        let selected = try_read_timeline_projection().expect("selected");
        assert_eq!(
            selected.primary_layer_id.as_deref(),
            Some(layer_id.as_str())
        );

        clear_slot();
        motolii_ui::host_destroy_for_test(host).expect("destroy");
    }

    #[test]
    fn parse_catalog_and_layer_effects_from_wire_json() {
        let json = r#"{
            "version":1,
            "direction":"host-to-rn",
            "role":"product",
            "host_handle":"1",
            "revision":"3",
            "projection_generation":"0",
            "current_time":{"num":0,"den":1},
            "primary_layer_id":"11",
            "stage":{"selection":[],"bounds":[{"layer_id":"11","display_name":"L"}]},
            "catalog":{
                "effects":[
                    {"plugin_id":"core.filter.opacity","name":"Opacity","effect_version":1},
                    {"plugin_id":"core.param.sine","name":"Sine","effect_version":2}
                ],
                "sources":[
                    {"plugin_id":"core.layer_source.radial_repeater","name":"Radial Repeater","effect_version":1}
                ]
            },
            "timeline":{
                "fps":{"num":30,"den":1},
                "layers":[{
                    "layer_id":"11",
                    "display_name":"L",
                    "start":{"num":0,"den":1},
                    "duration":{"num":10,"den":1},
                    "position_keys":[],
                    "keys_truncated":false,
                    "effects":[{
                        "effect_use_id":"7",
                        "plugin_id":"core.filter.opacity",
                        "params":[{"param_id":"amount","value":0.5}]
                    }],
                    "effects_truncated":false,
                    "source_params":[{"param_id":"count","value":12.0}],
                    "source_params_truncated":false
                }],
                "layers_truncated":false
            },
            "diagnostics":[]
        }"#;
        let catalog = parse_catalog_projection(json).expect("catalog");
        assert_eq!(catalog.effects.len(), 2);
        assert_eq!(catalog.effects[0].plugin_id, "core.filter.opacity");
        assert_eq!(catalog.effects[0].name, "Opacity");
        assert_eq!(catalog.effects[0].effect_version, 1);
        assert_eq!(catalog.sources.len(), 1);
        assert_eq!(
            catalog.sources[0].plugin_id,
            "core.layer_source.radial_repeater"
        );
        assert_eq!(catalog.sources[0].name, "Radial Repeater");
        let proj = parse_timeline_projection(json).expect("parse");
        let layers = proj.timeline_layers.expect("layers");
        assert_eq!(layers[0].effects.len(), 1);
        assert_eq!(layers[0].effects[0].effect_use_id, "7");
        assert_eq!(layers[0].effects[0].params[0].value, 0.5);
        assert!(!layers[0].effects_truncated);
        assert_eq!(layers[0].source_params.len(), 1);
        assert_eq!(layers[0].source_params[0].param_id, "count");
        assert_eq!(layers[0].source_params[0].value, 12.0);
        assert!(!layers[0].source_params_truncated);
    }

    #[test]
    fn parse_timeline_effects_respects_truncated_flag_after_eight_entries() {
        let json = r#"{
            "version":1,
            "direction":"host-to-rn",
            "role":"product",
            "host_handle":"1",
            "revision":"3",
            "projection_generation":"0",
            "current_time":{"num":0,"den":1},
            "stage":{"selection":[],"bounds":[{"layer_id":"11","display_name":"L"}]},
            "timeline":{
                "fps":{"num":30,"den":1},
                "layers":[{
                    "layer_id":"11",
                    "display_name":"L",
                    "start":{"num":0,"den":1},
                    "duration":{"num":10,"den":1},
                    "position_keys":[],
                    "keys_truncated":false,
                    "effects":[
                        {"effect_use_id":"e0","plugin_id":"p0","params":[]},
                        {"effect_use_id":"e1","plugin_id":"p1","params":[]},
                        {"effect_use_id":"e2","plugin_id":"p2","params":[]},
                        {"effect_use_id":"e3","plugin_id":"p3","params":[]},
                        {"effect_use_id":"e4","plugin_id":"p4","params":[]},
                        {"effect_use_id":"e5","plugin_id":"p5","params":[]},
                        {"effect_use_id":"e6","plugin_id":"p6","params":[]},
                        {"effect_use_id":"e7","plugin_id":"p7","params":[]}
                    ],
                    "effects_truncated":false
                }],
                "layers_truncated":false
            },
            "diagnostics":[]
        }"#;
        let proj = parse_timeline_projection(json).expect("parse");
        let layers = proj.timeline_layers.expect("layers");
        assert_eq!(layers[0].effects.len(), 8);
        assert_eq!(layers[0].effects[7].effect_use_id, "e7");
        assert!(!layers[0].effects_truncated);

        let json = r#"{
            "version":1,
            "direction":"host-to-rn",
            "role":"product",
            "host_handle":"1",
            "revision":"3",
            "projection_generation":"0",
            "current_time":{"num":0,"den":1},
            "stage":{"selection":[],"bounds":[{"layer_id":"11","display_name":"L"}]},
            "timeline":{
                "fps":{"num":30,"den":1},
                "layers":[{
                    "layer_id":"11",
                    "display_name":"L",
                    "start":{"num":0,"den":1},
                    "duration":{"num":10,"den":1},
                    "position_keys":[],
                    "keys_truncated":false,
                    "effects":[
                        {"effect_use_id":"e0","plugin_id":"p0","params":[]},
                        {"effect_use_id":"e1","plugin_id":"p1","params":[]},
                        {"effect_use_id":"e2","plugin_id":"p2","params":[]},
                        {"effect_use_id":"e3","plugin_id":"p3","params":[]},
                        {"effect_use_id":"e4","plugin_id":"p4","params":[]},
                        {"effect_use_id":"e5","plugin_id":"p5","params":[]},
                        {"effect_use_id":"e6","plugin_id":"p6","params":[]},
                        {"effect_use_id":"e7","plugin_id":"p7","params":[]},
                        {"effect_use_id":"e8","plugin_id":"p8","params":[]}
                    ],
                    "effects_truncated":true
                }],
                "layers_truncated":false
            },
            "diagnostics":[]
        }"#;
        let proj = parse_timeline_projection(json).expect("parse");
        let layers = proj.timeline_layers.expect("layers");
        assert_eq!(layers[0].effects.len(), 9);
        assert_eq!(layers[0].effects[7].effect_use_id, "e7");
        assert_eq!(layers[0].effects[8].effect_use_id, "e8");
        assert!(layers[0].effects_truncated);
    }

    #[test]
    fn parse_catalog_and_effects_fall_back_on_broken_values() {
        let broken_catalog = r#"{
            "version":1,
            "direction":"host-to-rn",
            "role":"product",
            "host_handle":"1",
            "revision":"3",
            "projection_generation":"0",
            "current_time":{"num":0,"den":1},
            "stage":{"selection":[],"bounds":[]},
            "catalog":{"effects":[{"plugin_id":1}]},
            "timeline":{"fps":{"num":30,"den":1},"layers":[],"layers_truncated":false},
            "diagnostics":[]
        }"#;
        assert!(parse_catalog_projection(broken_catalog).is_none());
        assert!(parse_timeline_projection(broken_catalog).is_some());

        let broken_effects = r#"{
            "version":1,
            "direction":"host-to-rn",
            "role":"product",
            "host_handle":"1",
            "revision":"3",
            "projection_generation":"0",
            "current_time":{"num":0,"den":1},
            "stage":{"selection":[],"bounds":[]},
            "timeline":{
                "fps":{"num":30,"den":1},
                "layers":[{
                    "layer_id":"11",
                    "display_name":"L",
                    "start":{"num":0,"den":1},
                    "duration":{"num":10,"den":1},
                    "position_keys":[],
                    "keys_truncated":false,
                    "effects":[{"effect_use_id":"7","plugin_id":"x","params":"bad"}],
                    "effects_truncated":false
                }],
                "layers_truncated":false
            },
            "diagnostics":[]
        }"#;
        let proj = parse_timeline_projection(broken_effects).expect("parse");
        let layers = proj.timeline_layers.expect("layers kept");
        assert!(layers[0].effects.is_empty());
        assert!(!layers[0].effects_truncated);

        let broken_sources = r#"{
            "version":1,
            "direction":"host-to-rn",
            "role":"product",
            "host_handle":"1",
            "revision":"3",
            "projection_generation":"0",
            "current_time":{"num":0,"den":1},
            "stage":{"selection":[],"bounds":[]},
            "catalog":{
                "effects":[{"plugin_id":"core.filter.opacity","name":"Opacity","effect_version":1}],
                "sources":[{"plugin_id":1}]
            },
            "timeline":{"fps":{"num":30,"den":1},"layers":[],"layers_truncated":false},
            "diagnostics":[]
        }"#;
        let catalog = parse_catalog_projection(broken_sources).expect("catalog kept");
        assert_eq!(catalog.effects.len(), 1);
        assert_eq!(catalog.effects[0].plugin_id, "core.filter.opacity");
        assert!(catalog.sources.is_empty());

        let broken_source_params = r#"{
            "version":1,
            "direction":"host-to-rn",
            "role":"product",
            "host_handle":"1",
            "revision":"3",
            "projection_generation":"0",
            "current_time":{"num":0,"den":1},
            "stage":{"selection":[],"bounds":[]},
            "timeline":{
                "fps":{"num":30,"den":1},
                "layers":[{
                    "layer_id":"11",
                    "display_name":"L",
                    "start":{"num":0,"den":1},
                    "duration":{"num":10,"den":1},
                    "position_keys":[],
                    "keys_truncated":false,
                    "effects":[],
                    "effects_truncated":false,
                    "source_params":"bad",
                    "source_params_truncated":false
                }],
                "layers_truncated":false
            },
            "diagnostics":[]
        }"#;
        let proj = parse_timeline_projection(broken_source_params).expect("parse");
        let layers = proj.timeline_layers.expect("layers kept");
        assert!(layers[0].source_params.is_empty());
        assert!(!layers[0].source_params_truncated);
    }

    /// F9: stamp不変tickではsnapshot読みFFI(=try_read_timeline_projection)を呼ばない。
    #[test]
    fn unchanged_projection_stamp_skips_snapshot_json_read() {
        let _lock = test_lock();
        let path = temp_project("stamp-skip-read");
        let host = motolii_ui::host_create_for_test(&path).expect("create host");
        install_slot(host);
        test_reset_snapshot_read_count();

        let stamp = try_read_projection_stamp().expect("stamp");
        assert_eq!(stamp, (0, 0));
        let before = test_snapshot_read_count();

        // stamp一致ならgateはfull読み不要 → カウンタ不変
        assert!(!RendererCore::host_snapshot_read_needed(
            Some(stamp),
            Some(stamp),
            false
        ));
        assert_eq!(test_snapshot_read_count(), before);

        // 変化時だけ読む
        dispatch_kind(host, "set_time", r#","frame":12"#);
        let next = try_read_projection_stamp().expect("stamp after set_time");
        assert_ne!(next, stamp);
        assert!(RendererCore::host_snapshot_read_needed(
            Some(stamp),
            Some(next),
            false
        ));
        let _ = try_read_timeline_projection().expect("projection");
        assert_eq!(test_snapshot_read_count(), before + 1);

        clear_slot();
        motolii_ui::host_destroy_for_test(host).expect("destroy");
    }

    #[test]
    fn mac_key_table_maps_space_delete_undo_to_existing_kinds() {
        let space = super::resolve_mac_key_action(49, 0, " ");
        assert_eq!(
            space
                .as_ref()
                .and_then(motolii_ui::product_action_host_kind),
            Some("toggle_playback")
        );
        let delete = super::resolve_mac_key_action(117, 0, "");
        assert_eq!(
            delete
                .as_ref()
                .and_then(motolii_ui::product_action_host_kind),
            Some("delete_layer")
        );
        let backspace = super::resolve_mac_key_action(51, 0, "");
        assert_eq!(
            backspace
                .as_ref()
                .and_then(motolii_ui::product_action_host_kind),
            Some("delete_layer")
        );
        let undo = super::resolve_mac_key_action(6, super::MOD_META, "z");
        assert_eq!(
            undo.as_ref().and_then(motolii_ui::product_action_host_kind),
            Some("undo")
        );
        let redo = super::resolve_mac_key_action(6, super::MOD_META | super::MOD_SHIFT, "z");
        assert_eq!(
            redo.as_ref().and_then(motolii_ui::product_action_host_kind),
            Some("redo")
        );
        let duplicate = super::resolve_mac_key_action(2, super::MOD_META, "d");
        assert_eq!(
            duplicate
                .as_ref()
                .and_then(motolii_ui::product_action_host_kind),
            Some("duplicate")
        );
        let shuttle_forward = super::resolve_mac_key_action(37, 0, "l");
        assert_eq!(
            shuttle_forward
                .as_ref()
                .and_then(motolii_ui::product_action_host_kind),
            Some("shuttle_forward")
        );
        let shuttle_reverse = super::resolve_mac_key_action(38, 0, "j");
        assert_eq!(
            shuttle_reverse
                .as_ref()
                .and_then(motolii_ui::product_action_host_kind),
            Some("shuttle_reverse")
        );
        let shuttle_stop = super::resolve_mac_key_action(40, 0, "k");
        assert_eq!(
            shuttle_stop
                .as_ref()
                .and_then(motolii_ui::product_action_host_kind),
            Some("shuttle_stop")
        );
        let split = super::resolve_mac_key_action(40, super::MOD_META, "k");
        assert_eq!(
            split
                .as_ref()
                .and_then(motolii_ui::product_action_host_kind),
            Some("split")
        );
        let mark_in = super::resolve_mac_key_action(34, 0, "i");
        assert_eq!(
            mark_in
                .as_ref()
                .and_then(motolii_ui::product_action_host_kind),
            Some("trim_clip_in")
        );
        let mark_out = super::resolve_mac_key_action(31, 0, "o");
        assert_eq!(
            mark_out
                .as_ref()
                .and_then(motolii_ui::product_action_host_kind),
            Some("trim_clip_out")
        );
        let mute = super::resolve_mac_key_action(46, 0, "m");
        assert_eq!(
            mute.as_ref().and_then(motolii_ui::product_action_host_kind),
            Some(motolii_ui::PRODUCT_HOST_KIND_MUTE)
        );
        let solo = super::resolve_mac_key_action(1, 0, "s");
        assert_eq!(
            solo.as_ref().and_then(motolii_ui::product_action_host_kind),
            Some(motolii_ui::PRODUCT_HOST_KIND_SOLO)
        );
    }
}
