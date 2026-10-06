//! CORE = Rerun. The Skin writes values at a frame; the Render Core asks what is true at a frame. Nothing else lives here.
use std::ffi::{c_char, CStr};
use std::sync::{Arc, Mutex, OnceLock};

use re_chunk::{Chunk, RowId};
use re_chunk_store::LatestAtQuery;
use re_entity_db::EntityDb;
use re_log_types::{EntityPath, StoreId, StoreKind, TimePoint, Timeline};
use re_sdk_types::archetypes::Scalars;
use re_sdk_types::components::Scalar;

fn db() -> &'static Mutex<EntityDb> {
    static DB: OnceLock<Mutex<EntityDb>> = OnceLock::new();
    DB.get_or_init(|| Mutex::new(EntityDb::new(StoreId::random(StoreKind::Recording, "motolii"))))
}

fn timeline() -> Timeline {
    Timeline::new_sequence("frame")
}

/// # Safety
/// `path` is a NUL-terminated entity path.
#[no_mangle]
pub unsafe extern "C" fn core_set(path: *const c_char, frame: i64, value: f64) {
    let path = unsafe { CStr::from_ptr(path) }.to_string_lossy();
    let chunk = Chunk::builder(EntityPath::from(path.as_ref()))
        .with_archetype(RowId::new(), TimePoint::default().with(timeline(), frame), &Scalars::single(value))
        .build()
        .expect("chunk");
    db().lock().unwrap().add_chunk(&Arc::new(chunk)).expect("add_chunk");
}

/// # Safety
/// `path` is a NUL-terminated entity path.
#[no_mangle]
pub unsafe extern "C" fn core_get(path: *const c_char, frame: i64, fallback: f64) -> f64 {
    let path = unsafe { CStr::from_ptr(path) }.to_string_lossy();
    let q = LatestAtQuery::new(*timeline().name(), frame);
    db().lock()
        .unwrap()
        .latest_at_component_quiet::<Scalar>(&EntityPath::from(path.as_ref()), &q, Scalars::descriptor_scalars().component)
        .map(|(_, v)| v.0 .0)
        .unwrap_or(fallback)
}
