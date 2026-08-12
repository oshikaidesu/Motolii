//! RN app ↔ RnProductHost 接続の単一owner。
//!
//! processに最大1 host。ObjC/RNは薄いcarrierとしてここへ委譲する。

use std::path::Path;
use std::sync::{Mutex, OnceLock};

const MAX_JSON_BYTES: usize = 16_384;
const MAX_SNAPSHOT_JSON_BYTES: usize = 131_072;

#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(test)]
static TEST_SELECTION_DISPATCH_COUNT: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
static TEST_KEYMAP_REMOVE_POSITION_KEY_COUNT: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
static TEST_KEYMAP_DELETE_LAYER_COUNT: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
static TEST_MOVE_LAYER_BY_DISPATCH_COUNT: AtomicU64 = AtomicU64::new(0);

// extern importではなくRust経由で呼ぶ。externで宣言すると同一crate graph内でも
// motolii-uiの該当objectがarchiveから引かれず、appのlinkで未解決symbolになる(実測)。
#[cfg(target_os = "macos")]
use motolii_ui::{
    motolii_rn_host_create, motolii_rn_host_dispatch_intent_json,
    motolii_rn_host_read_snapshot_json, motolii_rn_stage_register,
};

struct HostSlot {
    handle: u64,
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

fn host_slot() -> &'static Mutex<Option<HostSlot>> {
    static SLOT: OnceLock<Mutex<Option<HostSlot>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

/// Host投影。revision変化時だけTimelineへ適用する。
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostTimelineProjection {
    pub revision: String,
    pub projection_generation: String,
    pub primary_layer_id: Option<String>,
    /// wire `current_time` の {num,den}。欠落時は0/1。
    pub current_time: (i64, i64),
    /// wire `timeline.fps`。timeline欠落時はNone。
    pub fps: Option<(i64, i64)>,
    pub bounds: Vec<(String, String)>,
    /// wire `timeline` がある時だけ。欠落時は旧host互換fallback。
    pub timeline_layers: Option<Vec<HostTimelineLayer>>,
    /// wire `stage_geometry`。欠落・壊れている時はNone（timeline投影は落とさない）。
    pub stage_geometry: Option<HostStageGeometry>,
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
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostTimelineLayer {
    pub layer_id: String,
    pub display_name: String,
    pub start_secs: f64,
    pub duration_secs: f64,
    pub position_keys: Vec<HostTimelineKey>,
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
    let Ok(mut guard) = host_slot().lock() else {
        return false;
    };
    if guard.is_some() {
        return true;
    }
    if !ensure_project_document(Path::new(path)) {
        return false;
    }
    let path_bytes = path.as_bytes();
    let mut host_handle = 0u64;
    let mut out = [0u8; MAX_JSON_BYTES];
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
        return false;
    }
    *guard = Some(HostSlot {
        handle: host_handle,
        stage_handle: None,
        stage_pointer_active: false,
        stage_mounted: false,
        pointer_sequence: 0,
        stage_logical_width: 0.0,
        stage_logical_height: 0.0,
    });
    true
}

/// Host投影のStage seatを1つだけregisterする。既登録はreuse。
#[cfg(target_os = "macos")]
fn ensure_stage_registered(slot: &mut HostSlot) -> bool {
    if slot.stage_handle.is_some() {
        return true;
    }
    let mut stage_handle = 0u64;
    let mut out = [0u8; MAX_JSON_BYTES];
    let written = unsafe {
        motolii_rn_stage_register(
            slot.handle,
            &mut stage_handle,
            out.as_mut_ptr(),
            out.len(),
        )
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
    let mut out = [0u8; MAX_JSON_BYTES];
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
/// `kind_utf8` must point to `kind_len` UTF-8 bytes naming undo/redo/delete_layer.
pub unsafe extern "C" fn motolii_rnapp_host_keymap(kind_utf8: *const u8, kind_len: usize) -> bool {
    if kind_utf8.is_null() || kind_len == 0 || kind_len > 64 {
        return false;
    }
    let Ok(kind) = std::str::from_utf8(unsafe { std::slice::from_raw_parts(kind_utf8, kind_len) })
    else {
        return false;
    };
    try_dispatch_keymap(kind)
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
        return -1;
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

/// Hostが在る時だけsnapshotを読む。不在・失敗はNone。
pub(crate) fn try_read_timeline_projection() -> Option<HostTimelineProjection> {
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

/// scrub bar → set_time frame。`frame = round(bar * 2 * fps.num / fps.den)`。
pub(crate) fn frame_from_scrub_bar(bar: f64, fps_num: i64, fps_den: i64) -> i64 {
    if fps_num <= 0 || fps_den <= 0 {
        return 0;
    }
    const SCALE: i128 = 1_000_000;
    let s_fixed = (bar * crate::timeline_skia::SECONDS_PER_BAR * (SCALE as f64)).round() as i128;
    let num = s_fixed * (fps_num as i128);
    let den = (fps_den as i128) * SCALE;
    let half = den / 2;
    let signed_half = if num.is_negative() { -half } else { half };
    ((num + signed_half) / den) as i64
}

/// bar → RationalTime wire `{num,den}`。1 bar = 2秒、SCALE固定小数でf64連鎖丸めを避ける。
pub(crate) fn rational_time_parts_from_bar(bar: f64) -> (i64, i64) {
    const SCALE: i128 = 1_000_000;
    let s_fixed = (bar * crate::timeline_skia::SECONDS_PER_BAR * (SCALE as f64)).round() as i128;
    (s_fixed as i64, SCALE as i64)
}

/// host `current_time` → playhead(0..1)。bar = secs/2、曲基準 / SONG_BARS。
pub(crate) fn playhead_from_current_time(num: i64, den: i64) -> f64 {
    if den == 0 {
        return 0.0;
    }
    let secs = num as f64 / den as f64;
    let bar = secs / crate::timeline_skia::SECONDS_PER_BAR;
    (bar / f64::from(crate::timeline_skia::SONG_BARS)).clamp(0.0, 1.0)
}

/// process host slot経由でset_timeを送る。host不在はfalse(呼ばない)。
pub(crate) fn try_dispatch_set_time(frame: i64) -> bool {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = frame;
        false
    }
    #[cfg(target_os = "macos")]
    {
        let Ok(guard) = host_slot().lock() else {
            return false;
        };
        let Some(slot) = guard.as_ref() else {
            return false;
        };
        let intent = format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"set_time","host_handle":"{}","frame":{}}}"#,
            slot.handle, frame
        );
        dispatch_intent_json_accepted(slot.handle, &intent)
    }
}

pub(crate) fn try_dispatch_timeline_edit(
    commit: &crate::timeline_skia::TimelineEditCommit,
) -> bool {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = commit;
        false
    }
    #[cfg(target_os = "macos")]
    {
        use crate::timeline_skia::TimelineEditCommit;
        let Ok(guard) = host_slot().lock() else {
            return false;
        };
        let Some(slot) = guard.as_ref() else {
            return false;
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
                let (num, den) = rational_time_parts_from_bar(f64::from(*bar));
                format!(
                    concat!(
                        r#"{{"version":1,"direction":"rn-to-host","kind":"set_position_key_time","#,
                        r#""host_handle":"{}","target":"{}","key_id":"{}","time":{{"num":{},"den":{}}}}}"#
                    ),
                    slot.handle, layer_id, key_id, num, den
                )
            }
        };
        dispatch_intent_json_accepted(slot.handle, &intent)
    }
}

pub(crate) fn try_dispatch_remove_position_key(layer_id: &str, key_id: u64) -> bool {
    #[cfg(test)]
    TEST_KEYMAP_REMOVE_POSITION_KEY_COUNT.fetch_add(1, Ordering::SeqCst);
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (layer_id, key_id);
        false
    }
    #[cfg(target_os = "macos")]
    {
        let Ok(guard) = host_slot().lock() else {
            return false;
        };
        let Some(slot) = guard.as_ref() else {
            return false;
        };
        let intent = format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"remove_position_key","#,
                r#""host_handle":"{}","target":"{}","key_id":"{}"}}"#
            ),
            slot.handle, layer_id, key_id
        );
        dispatch_intent_json_accepted(slot.handle, &intent)
    }
}

pub(crate) fn try_dispatch_timeline_selection(
    commit: &crate::timeline_skia::TimelineSelectionCommit,
) -> bool {
    #[cfg(test)]
    TEST_SELECTION_DISPATCH_COUNT.fetch_add(1, Ordering::SeqCst);
    #[cfg(not(target_os = "macos"))]
    {
        let _ = commit;
        false
    }
    #[cfg(target_os = "macos")]
    {
        use crate::timeline_skia::TimelineSelectionCommit;
        let Ok(guard) = host_slot().lock() else {
            return false;
        };
        let Some(slot) = guard.as_ref() else {
            return false;
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
        dispatch_intent_json_accepted(slot.handle, &intent)
    }
}

pub(crate) fn try_dispatch_move_layer_by(target: &str, delta: [f64; 2]) -> bool {
    #[cfg(test)]
    TEST_MOVE_LAYER_BY_DISPATCH_COUNT.fetch_add(1, Ordering::SeqCst);
    if !delta.iter().all(|value| value.is_finite()) {
        return false;
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = target;
        false
    }
    #[cfg(target_os = "macos")]
    {
        let Ok(guard) = host_slot().lock() else {
            return false;
        };
        let Some(slot) = guard.as_ref() else {
            return false;
        };
        let intent = format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"move_layer_by","#,
                r#""host_handle":"{}","target":"{}","delta":[{},{}]}}"#
            ),
            slot.handle, target, delta[0], delta[1]
        );
        dispatch_intent_json_accepted(slot.handle, &intent)
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
pub(crate) fn try_timeline_keymap_delete(scene: &crate::timeline_skia::TimelineScene) -> bool {
    if let Some((layer_id, key_id)) = crate::timeline_skia::selected_real_key(scene) {
        return try_dispatch_remove_position_key(&layer_id, key_id);
    }
    #[cfg(test)]
    TEST_KEYMAP_DELETE_LAYER_COUNT.fetch_add(1, Ordering::SeqCst);
    try_dispatch_keymap("delete_layer")
}

/// keymap: undo / redo / delete_layer(現primary)。primaryなしのdeleteは何もしない。
pub(crate) fn try_dispatch_keymap(kind: &str) -> bool {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = kind;
        false
    }
    #[cfg(target_os = "macos")]
    {
        let Ok(guard) = host_slot().lock() else {
            return false;
        };
        let Some(slot) = guard.as_ref() else {
            return false;
        };
        let intent = match kind {
            "undo" | "redo" => format!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"{kind}","host_handle":"{}"}}"#,
                slot.handle
            ),
            "delete_layer" => {
                drop(guard);
                let Some(projection) = try_read_timeline_projection() else {
                    return false;
                };
                let Some(target) = projection.primary_layer_id else {
                    // primaryなしは何もしない(拒否でも失敗でもない)。
                    return true;
                };
                let Ok(guard) = host_slot().lock() else {
                    return false;
                };
                let Some(slot) = guard.as_ref() else {
                    return false;
                };
                let intent = format!(
                    concat!(
                        r#"{{"version":1,"direction":"rn-to-host","kind":"delete_layer","#,
                        r#""host_handle":"{}","target":"{}"}}"#
                    ),
                    slot.handle, target
                );
                return dispatch_intent_json_accepted(slot.handle, &intent);
            }
            _ => return false,
        };
        dispatch_intent_json_accepted(slot.handle, &intent)
    }
}

#[cfg(target_os = "macos")]
fn dispatch_intent_json_accepted(handle: u64, intent: &str) -> bool {
    if intent.len() > MAX_JSON_BYTES {
        return false;
    }
    let mut out = [0u8; MAX_JSON_BYTES];
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

fn response_is_accepted(response: &str) -> bool {
    let Some(pos) = response.find("\"accepted\"") else {
        return false;
    };
    let Some(colon_pos) = response[pos..].find(':') else {
        return false;
    };
    response[pos + colon_pos + 1..].trim_start().starts_with("true")
}

fn slice_from_written<'a>(out: &'a [u8], written: i64) -> Option<&'a [u8]> {
    let written = usize::try_from(written).ok()?;
    (written <= out.len()).then_some(&out[..written])
}

fn inject_host_handle(intent: &str, handle: u64) -> Result<String, ()> {
    const KEY: &str = "\"host_handle\"";
    let key_at = intent.find(KEY).ok_or(())?;
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
        fps,
        bounds,
        timeline_layers,
        stage_geometry,
    })
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
        layers.push(HostTimelineLayer {
            layer_id,
            display_name,
            start_secs: start_num as f64 / start_den as f64,
            duration_secs: duration_num as f64 / duration_den as f64,
            position_keys,
        });
        rest = &rest[end + 1..];
    }
    Some(layers)
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
        layers.push(HostStageGeometryLayer { layer_id, corners });
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
    let marker = "\"position_keys\"";
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

fn parse_optional_vec2(obj: &str) -> Option<[f64; 2]> {
    let marker = "\"value\"";
    let at = obj.find(marker)?;
    let after = obj[at + marker.len()..]
        .trim_start()
        .strip_prefix(':')?;
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
        let mut out = [0u8; MAX_JSON_BYTES];
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
        let mut out = [0u8; MAX_JSON_BYTES];
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
            Some((
                baseline.timeline.fps.num(),
                baseline.timeline.fps.den()
            ))
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
        let scene = TimelineScene::from_snapshot(&layers, proj.primary_layer_id.as_deref());
        let (a, b, keys) = scene.clip0_span_and_keys(0).expect("clip0");
        assert!((a - 0.0).abs() < 1e-6);
        assert!((b - 5.0).abs() < 1e-6); // 10s / 2
        assert_eq!(keys.len(), 1);
        assert!((keys[0].0 - 2.0).abs() < 1e-6); // 4s / 2
        assert_eq!(keys[0].1, 42);
        assert_eq!(scene.selected_flat, 0);
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
        let scene = TimelineScene::from_snapshot(&layers, proj.primary_layer_id.as_deref());
        let (a, b, keys) = scene.clip0_span_and_keys(0).expect("clip0");
        assert!((a - 0.0).abs() < 1e-6);
        assert!((b - 96.0).abs() < 1e-6);
        assert!(keys.is_empty());
        assert_eq!(scene.band_count(), 2);
    }

    #[test]
    fn frame_from_scrub_bar_rounds_at_fps_30_and_24_boundaries() {
        // bar=1 → 2s。fps30 → frame 60、fps24 → frame 48。
        assert_eq!(frame_from_scrub_bar(1.0, 30, 1), 60);
        assert_eq!(frame_from_scrub_bar(1.0, 24, 1), 48);
        // 半端: bar=0.5 → 1s → fps30 frame 30、fps24 frame 24。
        assert_eq!(frame_from_scrub_bar(0.5, 30, 1), 30);
        assert_eq!(frame_from_scrub_bar(0.5, 24, 1), 24);
        // 丸め境界: bar * 2 * 30 = 0.5 → frame 1 (round half away from zero via f64::round)
        assert_eq!(frame_from_scrub_bar(0.5 / 60.0, 30, 1), 1);
        assert_eq!(frame_from_scrub_bar(0.5 / 48.0, 24, 1), 0);
        // 直前は0
        assert_eq!(frame_from_scrub_bar(0.49 / 60.0, 30, 1), 0);
        assert_eq!(frame_from_scrub_bar(0.49 / 48.0, 24, 1), 0);
        assert_eq!(frame_from_scrub_bar(0.5, 30, 0), 0);
        assert_eq!(frame_from_scrub_bar(0.5, 0, 1), 0);
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
    fn playhead_from_current_time_uses_two_seconds_per_bar() {
        // 2s = 1 bar → 1/96
        let ph = playhead_from_current_time(2, 1);
        assert!((ph - 1.0 / 96.0).abs() < 1e-12);
        assert_eq!(playhead_from_current_time(0, 1), 0.0);
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

        // bar=1 → 2s → frame 60。既定fpsで往復一致。
        let frame = frame_from_scrub_bar(1.0, fps_num, fps_den);
        assert_eq!(frame, 60);
        assert!(try_dispatch_set_time(frame));
        let after = motolii_ui::host_read_snapshot_for_test(host).expect("after");
        assert_eq!(after.current_time.num(), 2);
        assert_eq!(after.current_time.den(), 1);
        let ph = playhead_from_current_time(after.current_time.num(), after.current_time.den());
        assert!((ph - 1.0 / 96.0).abs() < 1e-12);

        clear_slot();
        motolii_ui::host_destroy_for_test(host).expect("destroy");
    }

    #[test]
    fn set_time_is_not_dispatched_without_host_slot() {
        let _lock = test_lock();
        clear_slot();
        assert!(!try_dispatch_set_time(60));
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

        assert!(try_dispatch_timeline_selection(
            &crate::timeline_skia::TimelineSelectionCommit::ClearSelection
        ));
        let cleared = try_read_timeline_projection().expect("cleared");
        assert!(cleared.primary_layer_id.is_none());

        assert!(try_dispatch_timeline_selection(
            &crate::timeline_skia::TimelineSelectionCommit::SelectLayer {
                layer_id: layer_id.clone(),
            }
        ));
        let selected = try_read_timeline_projection().expect("selected");
        assert_eq!(selected.primary_layer_id.as_deref(), Some(layer_id.as_str()));

        clear_slot();
        motolii_ui::host_destroy_for_test(host).expect("destroy");
    }

}
