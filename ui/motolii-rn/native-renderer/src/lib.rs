mod renderer_core;
mod rerun_stage;
// renderer_core と同じく skia-safe を前提にする(macOS / Windows のみ依存に入る)。
mod timeline_skia;

#[cfg(target_os = "macos")]
mod host_bridge;
#[cfg(target_os = "macos")]
mod platform;

use std::ffi::{CStr, c_char, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind};
#[cfg(target_os = "macos")]
use std::sync::atomic::{AtomicPtr, Ordering};

#[cfg(target_os = "macos")]
use motolii_ui::AppStageTransformEdit;
#[cfg(target_os = "macos")]
use platform::macos::MacOsSurfaceRenderer;
#[cfg(target_os = "macos")]
use renderer_core::RenderStats;
#[cfg(target_os = "macos")]
use renderer_core::{PointerPhase, StagePointerButton};

#[cfg(target_os = "macos")]
static ACTIVE_STAGE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

#[cfg(target_os = "macos")]
fn register_active_stage(handle: *mut c_void) {
    ACTIVE_STAGE.store(handle, Ordering::Release);
}

#[cfg(target_os = "macos")]
fn unregister_active_stage(handle: *mut c_void) {
    let _ = ACTIVE_STAGE.compare_exchange(
        handle,
        std::ptr::null_mut(),
        Ordering::AcqRel,
        Ordering::Acquire,
    );
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct MotoliiTimelineFeedback {
    pub object_index: i32,
    pub time: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct MotoliiStageTransform {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub rotation_x: f64,
    pub rotation_y: f64,
    pub rotation_z: f64,
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_macos_renderer_create_ca_layer(
    layer: *mut c_void,
    width: u32,
    height: u32,
) -> *mut c_void {
    catch_unwind(AssertUnwindSafe(|| {
        MacOsSurfaceRenderer::new_stage(layer, width, height)
    }))
    .ok()
    .and_then(Result::ok)
    .map_or(std::ptr::null_mut(), |renderer| {
        let handle = Box::into_raw(Box::new(renderer)).cast();
        register_active_stage(handle);
        // 製品マウントでhost snapshotをnativeへ届ける。テスト直呼びでは製品経路が空のまま。
        let _ = crate::host_bridge::try_read_timeline_projection();
        handle
    })
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_macos_timeline_renderer_create_ca_layer(
    layer: *mut c_void,
    width: u32,
    height: u32,
) -> *mut c_void {
    catch_unwind(AssertUnwindSafe(|| {
        MacOsSurfaceRenderer::new_timeline(layer, width, height)
    }))
    .ok()
    .and_then(Result::ok)
    .map_or(std::ptr::null_mut(), |renderer| {
        let handle = Box::into_raw(Box::new(renderer)).cast();
        // 製品マウントでhost snapshotをnativeへ届ける。テスト直呼びでは製品経路が空のまま。
        let _ = crate::host_bridge::try_read_timeline_projection();
        handle
    })
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_macos_renderer_resize(
    handle: *mut c_void,
    width: u32,
    height: u32,
) -> bool {
    if handle.is_null() {
        return false;
    }
    catch_unwind(AssertUnwindSafe(|| unsafe {
        (&mut *handle.cast::<MacOsSurfaceRenderer>()).resize(width, height);
    }))
    .is_ok()
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_macos_renderer_render(handle: *mut c_void) -> bool {
    if handle.is_null() {
        return false;
    }
    catch_unwind(AssertUnwindSafe(|| unsafe {
        (&mut *handle.cast::<MacOsSurfaceRenderer>()).render()
    }))
    .ok()
    .and_then(Result::ok)
    .is_some()
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_macos_stage_renderer_pointer(
    handle: *mut c_void,
    phase: u32,
    button: u32,
    modifiers: u32,
    x: f64,
    y: f64,
) -> bool {
    if handle.is_null() {
        return false;
    }
    let phase = match phase {
        0 => PointerPhase::Down,
        1 => PointerPhase::Move,
        2 => PointerPhase::Up,
        _ => PointerPhase::Cancel,
    };
    let Some(button) = StagePointerButton::from_raw(button) else {
        return false;
    };
    catch_unwind(AssertUnwindSafe(|| unsafe {
        (&mut *handle.cast::<MacOsSurfaceRenderer>()).stage_pointer(phase, button, modifiers, x, y);
    }))
    .is_ok()
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_macos_stage_renderer_scroll(
    handle: *mut c_void,
    delta_x: f64,
    delta_y: f64,
    magnification: f64,
    modifiers: u32,
    x: f64,
    y: f64,
) -> bool {
    if handle.is_null() {
        return false;
    }
    catch_unwind(AssertUnwindSafe(|| unsafe {
        (&mut *handle.cast::<MacOsSurfaceRenderer>()).stage_scroll(
            delta_x,
            delta_y,
            magnification,
            modifiers,
            x,
            y,
        )
    }))
    .unwrap_or(false)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
/// # Safety
/// `handle` must be a live renderer returned by this library and `item_id`
/// must point to a NUL-terminated UTF-8 string for the duration of this call.
pub unsafe extern "C" fn motolii_macos_stage_renderer_set_created_item(
    handle: *mut c_void,
    item_id: *const c_char,
) -> bool {
    if handle.is_null() || item_id.is_null() {
        return false;
    }
    let Ok(item_id) = (unsafe { CStr::from_ptr(item_id) }).to_str() else {
        return false;
    };
    catch_unwind(AssertUnwindSafe(|| unsafe {
        (&mut *handle.cast::<MacOsSurfaceRenderer>()).set_created_item(item_id)
    }))
    .unwrap_or(false)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_macos_active_stage_renderer() -> *mut c_void {
    ACTIVE_STAGE.load(Ordering::Acquire)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_macos_stage_renderer_fit_view(handle: *mut c_void) -> bool {
    if handle.is_null() {
        return false;
    }
    catch_unwind(AssertUnwindSafe(|| unsafe {
        (&mut *handle.cast::<MacOsSurfaceRenderer>()).fit_stage_view()
    }))
    .unwrap_or(false)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_macos_stage_renderer_one_to_one(handle: *mut c_void) -> bool {
    if handle.is_null() {
        return false;
    }
    catch_unwind(AssertUnwindSafe(|| unsafe {
        (&mut *handle.cast::<MacOsSurfaceRenderer>()).set_stage_one_to_one()
    }))
    .unwrap_or(false)
}

#[cfg(target_os = "macos")]
fn app_stage_transform_edit(kind: i32, a: f64, b: f64) -> Result<AppStageTransformEdit, String> {
    if !(a.is_finite() && b.is_finite()) {
        return Err("The transform result is not finite".to_owned());
    }
    match kind {
        0 => Ok(AppStageTransformEdit::TranslateWorld([a, b])),
        1 => Ok(AppStageTransformEdit::RotateZ(a)),
        2 => Ok(AppStageTransformEdit::Scale([a, b])),
        _ => Err("The transform kind is invalid".to_owned()),
    }
}

#[cfg(target_os = "macos")]
unsafe fn stage_transform_request_parts(
    target_utf8: *const u8,
    target_len: usize,
    revision_utf8: *const u8,
    revision_len: usize,
) -> Result<(String, u64), String> {
    if target_utf8.is_null() || revision_utf8.is_null() {
        return Err("The Stage transform request is invalid".to_owned());
    }
    let target =
        std::str::from_utf8(unsafe { std::slice::from_raw_parts(target_utf8, target_len) })
            .map_err(|_| "The selected layer identity is invalid".to_owned())?
            .to_owned();
    let revision =
        std::str::from_utf8(unsafe { std::slice::from_raw_parts(revision_utf8, revision_len) })
            .map_err(|_| "The live Document revision is invalid".to_owned())?
            .parse::<u64>()
            .map_err(|_| "The live Document revision is invalid".to_owned())?;
    Ok((target, revision))
}

#[cfg(target_os = "macos")]
unsafe fn write_stage_transform_result(
    result: Result<(), String>,
    out: *mut u8,
    out_cap: usize,
) -> i64 {
    let Err(error) = result else {
        return 0;
    };
    let bytes = error.as_bytes();
    if out.is_null() || bytes.is_empty() || bytes.len() > out_cap {
        return -1;
    }
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, bytes.len()) };
    bytes.len() as i64
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
/// # Safety
/// String inputs must be valid for their lengths and `out` must be writable for `out_cap` bytes.
pub unsafe extern "C" fn motolii_macos_active_stage_preview_transform(
    target_utf8: *const u8,
    target_len: usize,
    revision_utf8: *const u8,
    revision_len: usize,
    kind: i32,
    a: f64,
    b: f64,
    out: *mut u8,
    out_cap: usize,
) -> i64 {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let (target, revision) = unsafe {
            stage_transform_request_parts(target_utf8, target_len, revision_utf8, revision_len)
        }?;
        let edit = app_stage_transform_edit(kind, a, b)?;
        let handle = ACTIVE_STAGE.load(Ordering::Acquire);
        if handle.is_null() {
            return Err("Stage renderer is unavailable".to_owned());
        }
        unsafe { (&mut *handle.cast::<MacOsSurfaceRenderer>()) }
            .preview_stage_transform_from_app(revision, &target, edit)
    }))
    .unwrap_or_else(|_| Err("Stage preview failed".to_owned()));
    unsafe { write_stage_transform_result(result, out, out_cap) }
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
/// # Safety
/// String inputs must be valid for their lengths and `out` must be writable for `out_cap` bytes.
pub unsafe extern "C" fn motolii_macos_active_stage_commit_transform(
    target_utf8: *const u8,
    target_len: usize,
    revision_utf8: *const u8,
    revision_len: usize,
    kind: i32,
    a: f64,
    b: f64,
    out: *mut u8,
    out_cap: usize,
) -> i64 {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let (target, revision) = unsafe {
            stage_transform_request_parts(target_utf8, target_len, revision_utf8, revision_len)
        }?;
        let edit = app_stage_transform_edit(kind, a, b)?;
        let handle = ACTIVE_STAGE.load(Ordering::Acquire);
        if handle.is_null() {
            return Err("Stage renderer is unavailable".to_owned());
        }
        unsafe { (&mut *handle.cast::<MacOsSurfaceRenderer>()) }
            .commit_stage_transform_from_app(revision, &target, edit)
    }))
    .unwrap_or_else(|_| Err("Stage commit failed".to_owned()));
    unsafe { write_stage_transform_result(result, out, out_cap) }
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
/// # Safety
/// `out` must be writable for `out_cap` bytes.
pub unsafe extern "C" fn motolii_macos_active_stage_cancel_transform(
    out: *mut u8,
    out_cap: usize,
) -> i64 {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let handle = ACTIVE_STAGE.load(Ordering::Acquire);
        if handle.is_null() {
            return Err("Stage renderer is unavailable".to_owned());
        }
        unsafe { (&mut *handle.cast::<MacOsSurfaceRenderer>()) }.cancel_stage_transform_from_app()
    }))
    .unwrap_or_else(|_| Err("Stage cancel failed".to_owned()));
    unsafe { write_stage_transform_result(result, out, out_cap) }
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
/// # Safety
/// `handle` must be a live renderer returned by this library and `transform`
/// must point to writable storage for one `MotoliiStageTransform`.
pub unsafe extern "C" fn motolii_macos_stage_renderer_get_transform(
    handle: *mut c_void,
    transform: *mut MotoliiStageTransform,
) -> bool {
    if handle.is_null() || transform.is_null() {
        return false;
    }
    catch_unwind(AssertUnwindSafe(|| unsafe {
        let Some(projection) =
            (&*handle.cast::<MacOsSurfaceRenderer>()).stage_transform_projection()
        else {
            return false;
        };
        transform.write(MotoliiStageTransform {
            x: projection.x,
            y: projection.y,
            z: projection.z,
            rotation_x: projection.rotation_x,
            rotation_y: projection.rotation_y,
            rotation_z: projection.rotation_z,
        });
        true
    }))
    .unwrap_or(false)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_macos_stage_renderer_set_transform(
    handle: *mut c_void,
    transform: MotoliiStageTransform,
) -> bool {
    if handle.is_null() {
        return false;
    }
    catch_unwind(AssertUnwindSafe(|| unsafe {
        (&mut *handle.cast::<MacOsSurfaceRenderer>()).set_stage_transform_projection(
            rerun_stage::StageTransformProjection {
                x: transform.x,
                y: transform.y,
                z: transform.z,
                rotation_x: transform.rotation_x,
                rotation_y: transform.rotation_y,
                rotation_z: transform.rotation_z,
            },
        )
    }))
    .unwrap_or(false)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_macos_timeline_renderer_set_state(
    handle: *mut c_void,
    selected_object_index: i32,
    playhead: f64,
) -> bool {
    if handle.is_null() {
        return false;
    }
    catch_unwind(AssertUnwindSafe(|| unsafe {
        (&mut *handle.cast::<MacOsSurfaceRenderer>())
            .set_timeline_state(selected_object_index, playhead);
    }))
    .is_ok()
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
/// # Safety
/// `handle` must be a live renderer returned by this library and `feedback`
/// must point to writable storage for one `MotoliiTimelineFeedback`.
pub unsafe extern "C" fn motolii_macos_timeline_renderer_hit_test(
    handle: *mut c_void,
    x: f64,
    y: f64,
    feedback: *mut MotoliiTimelineFeedback,
) -> bool {
    if handle.is_null() || feedback.is_null() {
        return false;
    }
    catch_unwind(AssertUnwindSafe(|| unsafe {
        let renderer = &*handle.cast::<MacOsSurfaceRenderer>();
        let Some((object_index, time)) = renderer.timeline_hit_test(x, y) else {
            return false;
        };
        feedback.write(MotoliiTimelineFeedback { object_index, time });
        true
    }))
    .unwrap_or(false)
}

/// hover hit種に応じたcursor code。0=arrow 1=resizeLR 2=openHand 3=closedHand 4=pointingHand
#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
/// # Safety
/// `handle` must be a live renderer returned by this library.
pub unsafe extern "C" fn motolii_macos_timeline_renderer_hover_cursor(
    handle: *mut c_void,
    x: f64,
    y: f64,
) -> i32 {
    if handle.is_null() {
        return 0;
    }
    catch_unwind(AssertUnwindSafe(|| unsafe {
        (&*handle.cast::<MacOsSurfaceRenderer>()).timeline_hover_cursor(x, y)
    }))
    .unwrap_or(0)
}

/// Stage hover → cursor code。同上。
#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
/// # Safety
/// `handle` must be a live renderer returned by this library.
pub unsafe extern "C" fn motolii_macos_stage_renderer_hover_cursor(
    handle: *mut c_void,
    x: f64,
    y: f64,
) -> i32 {
    if handle.is_null() {
        return 0;
    }
    catch_unwind(AssertUnwindSafe(|| unsafe {
        (&*handle.cast::<MacOsSurfaceRenderer>()).stage_hover_cursor(x, y)
    }))
    .unwrap_or(0)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
/// # Safety
/// `handle` must be a live renderer returned by this library and `feedback`
/// must point to writable storage for one `MotoliiTimelineFeedback`.
pub unsafe extern "C" fn motolii_macos_timeline_renderer_pointer(
    handle: *mut c_void,
    phase: u32,
    x: f64,
    y: f64,
    modifiers: u32,
    feedback: *mut MotoliiTimelineFeedback,
) -> bool {
    if handle.is_null() || feedback.is_null() {
        return false;
    }
    let phase = match phase {
        0 => PointerPhase::Down,
        1 => PointerPhase::Move,
        2 => PointerPhase::Up,
        _ => PointerPhase::Cancel,
    };
    catch_unwind(AssertUnwindSafe(|| unsafe {
        let renderer = &mut *handle.cast::<MacOsSurfaceRenderer>();
        let Some((object_index, time)) = renderer.timeline_pointer(phase, x, y, modifiers) else {
            return false;
        };
        feedback.write(MotoliiTimelineFeedback { object_index, time });
        true
    }))
    .unwrap_or(false)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_macos_timeline_renderer_scroll(
    handle: *mut c_void,
    delta_x: f64,
    delta_y: f64,
    magnification: f64,
    modifiers: u32,
    x: f64,
    y: f64,
) -> bool {
    if handle.is_null() {
        return false;
    }
    catch_unwind(AssertUnwindSafe(|| unsafe {
        (&mut *handle.cast::<MacOsSurfaceRenderer>()).timeline_scroll(
            delta_x,
            delta_y,
            magnification,
            modifiers,
            x,
            y,
        )
    }))
    .unwrap_or(false)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
/// # Safety
/// `handle` must be a live renderer returned by this library and `stats`
/// must point to writable storage for one `RenderStats`.
pub unsafe extern "C" fn motolii_macos_renderer_get_stats(
    handle: *mut c_void,
    stats: *mut RenderStats,
) -> bool {
    if handle.is_null() || stats.is_null() {
        return false;
    }
    catch_unwind(AssertUnwindSafe(|| unsafe {
        stats.write((&*handle.cast::<MacOsSurfaceRenderer>()).stats());
    }))
    .is_ok()
}

/// keymap用の薄いFFI。実体はhost_bridge / renderer scene判定。
#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
/// # Safety
/// `handle` must be a live timeline renderer returned by this library.
pub unsafe extern "C" fn motolii_macos_timeline_renderer_keymap_delete(
    handle: *mut c_void,
) -> bool {
    if handle.is_null() {
        return false;
    }
    catch_unwind(AssertUnwindSafe(|| unsafe {
        (&*handle.cast::<MacOsSurfaceRenderer>()).timeline_keymap_delete()
    }))
    .unwrap_or(false)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_macos_renderer_destroy(handle: *mut c_void) {
    if handle.is_null() {
        return;
    }
    let _ = catch_unwind(AssertUnwindSafe(|| unsafe {
        unregister_active_stage(handle);
        drop(Box::from_raw(handle.cast::<MacOsSurfaceRenderer>()));
    }));
}
