mod renderer_core;

#[cfg(target_os = "macos")]
mod platform;

use std::ffi::c_void;
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
