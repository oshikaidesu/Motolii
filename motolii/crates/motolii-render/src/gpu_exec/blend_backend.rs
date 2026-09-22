use crate::doc::store::BlendMode;

use super::resource_store::GpuResourceStore;
use super::types::{GpuResourceKey, GpuResourceVersion};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ResidentBlend {
    pub blend: BlendMode,
}

pub(crate) fn resident_blend(
    store: &mut GpuResourceStore<ResidentBlend>,
    key: GpuResourceKey,
    version: GpuResourceVersion,
    generation: u64,
    blend: BlendMode,
) -> ResidentBlend {
    if let Some(hit) = store.current(key, version).copied() {
        store.touch(key, generation);
        return hit;
    }
    let value = ResidentBlend { blend };
    store.install(key, version, generation, value);
    value
}
