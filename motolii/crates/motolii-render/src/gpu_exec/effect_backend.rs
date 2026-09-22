use super::resource_store::GpuResourceStore;
use super::types::{GpuResourceKey, GpuResourceVersion};

#[derive(Clone)]
pub(crate) struct ResidentEffectChain {
    pub passes: Vec<crate::render::compositor::EffectPass>,
    pub plate_passes: Vec<crate::render::compositor::EffectPass>,
}

pub(crate) fn resident_effect_chain(
    store: &mut GpuResourceStore<ResidentEffectChain>,
    key: GpuResourceKey,
    version: GpuResourceVersion,
    generation: u64,
    passes: Vec<crate::render::compositor::EffectPass>,
    plate_passes: Vec<crate::render::compositor::EffectPass>,
) -> ResidentEffectChain {
    if let Some(hit) = store.current(key, version).cloned() {
        store.touch(key, generation);
        return hit;
    }
    let value = ResidentEffectChain { passes, plate_passes };
    store.install(key, version, generation, value.clone());
    value
}
