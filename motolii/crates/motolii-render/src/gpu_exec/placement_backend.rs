use crate::doc::core::LayerPlacement;

use super::resource_store::GpuResourceStore;
use super::types::{GpuResourceKey, GpuResourceVersion};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ResidentPlacement {
    pub placement: LayerPlacement,
}

pub(crate) fn resident_placement(
    store: &mut GpuResourceStore<ResidentPlacement>,
    key: GpuResourceKey,
    version: GpuResourceVersion,
    generation: u64,
    placement: LayerPlacement,
) -> ResidentPlacement {
    if let Some(hit) = store.current(key, version).copied() {
        store.touch(key, generation);
        return hit;
    }
    let value = ResidentPlacement { placement };
    store.install(key, version, generation, value);
    value
}
