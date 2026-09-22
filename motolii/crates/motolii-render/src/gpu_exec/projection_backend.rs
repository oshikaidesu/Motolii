use crate::doc::store::LayerProjection;

use super::resource_store::GpuResourceStore;
use super::types::{GpuResourceKey, GpuResourceVersion};

/// Projection is always an explicit authoring-time pin, never derived from
/// evaluated geometry/transform/camera state. This resident value carries
/// exactly the declared `LayerProjection` the Document authored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ResidentProjection {
    pub projection: LayerProjection,
}

pub(crate) fn resident_projection(
    store: &mut GpuResourceStore<ResidentProjection>,
    key: GpuResourceKey,
    version: GpuResourceVersion,
    generation: u64,
    projection: LayerProjection,
) -> ResidentProjection {
    if let Some(hit) = store.current(key, version).copied() {
        store.touch(key, generation);
        return hit;
    }
    let value = ResidentProjection { projection };
    store.install(key, version, generation, value);
    value
}
