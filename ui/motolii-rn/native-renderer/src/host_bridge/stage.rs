#[cfg(target_os = "macos")]
use motolii_ui::{motolii_rn_host_dispatch_intent_json, motolii_rn_stage_register};

use super::slot::{host_slot, slice_from_written, HostSlot, MAX_JSON_BYTES, MAX_SNAPSHOT_JSON_BYTES};
use super::terminal::response_is_accepted;

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
