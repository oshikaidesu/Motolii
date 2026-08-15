use std::path::Path;

// extern importではなくRust経由で呼ぶ。externで宣言すると同一crate graph内でも
// motolii-uiの該当objectがarchiveから引かれず、appのlinkで未解決symbolになる(実測)。
#[cfg(target_os = "macos")]
use motolii_ui::motolii_rn_host_create;

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn motolii_rn_host_destroy(host_handle: u64, out: *mut u8, out_cap: usize) -> i64;
}

use super::slot::{
    host_slot, host_startup_reject, slice_from_written, HostSlot, MAX_SNAPSHOT_JSON_BYTES,
};
use super::terminal::response_is_accepted;

/// 欠落documentを開ける最小projectでseedする。
/// `Document::new_current()` だけだと place_rectangle が process_next で落ちるため、
/// 空track 1本のseedを置く。
pub(super) fn ensure_project_document(path: &Path) -> bool {
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
pub(super) fn try_host_shutdown() -> bool {
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
