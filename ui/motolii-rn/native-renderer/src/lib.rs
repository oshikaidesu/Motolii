mod renderer_core;
mod rerun_stage;
// renderer_core と同じく skia-safe を前提にする(macOS / Windows のみ依存に入る)。
mod timeline_skia;

#[cfg(target_os = "macos")]
mod platform;

use std::ffi::{CStr, c_char, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind};

#[cfg(target_os = "macos")]
use platform::macos::MacOsSurfaceRenderer;
#[cfg(target_os = "macos")]
use renderer_core::PointerPhase;
#[cfg(target_os = "macos")]
use renderer_core::RenderStats;

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
        Box::into_raw(Box::new(renderer)).cast()
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
        Box::into_raw(Box::new(renderer)).cast()
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
    catch_unwind(AssertUnwindSafe(|| unsafe {
        (&mut *handle.cast::<MacOsSurfaceRenderer>()).stage_pointer(phase, x, y);
    }))
    .is_ok()
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
            (&mut *handle.cast::<MacOsSurfaceRenderer>()).take_stage_transform_projection()
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

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub extern "C" fn motolii_macos_renderer_destroy(handle: *mut c_void) {
    if handle.is_null() {
        return;
    }
    let _ = catch_unwind(AssertUnwindSafe(|| unsafe {
        drop(Box::from_raw(handle.cast::<MacOsSurfaceRenderer>()));
    }));
}
