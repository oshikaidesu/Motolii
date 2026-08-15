use motolii_ui::AppStageTransformEdit;
#[cfg(target_os = "macos")]
use motolii_ui::{motolii_rn_host_dispatch_intent_json, motolii_rn_host_read_snapshot_json};

use super::dispatch::try_commit_stage_transform;
use super::slot::{host_slot, host_startup_reject, MAX_JSON_BYTES};
use super::terminal::inject_host_handle;

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
