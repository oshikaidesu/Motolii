use std::hash::{Hash, Hasher};
use crate::frame_graph::SceneLayerValue;

use super::resource_store::GpuResourceStore;
use super::types::{GpuResourceKey, GpuResourceVersion};

#[derive(Clone)]
pub(crate) struct ResidentEffectChain {
    pub passes: Vec<crate::render::compositor::EffectPass>,
    pub plate_passes: Vec<crate::render::compositor::EffectPass>,
}

/// Translation of semantic effect values is CPU work but belongs to the GPU
/// execution resource, not to whole-scene assembly.
pub(crate) fn resident_effect_chain(
    store: &mut GpuResourceStore<ResidentEffectChain>,
    key: GpuResourceKey,
    version: GpuResourceVersion,
    generation: u64,
    layer: &SceneLayerValue,
    screen: [u32; 2],
) -> ResidentEffectChain {
    if let Some(hit) = store.current(key, version).cloned() {
        store.touch(key, generation);
        return hit;
    }

    let passes = crate::render::engine::translate::translate_effect_passes(&layer.effects);
    let plate_passes = crate::render::engine::translate::translate_plate_passes(&layer.after_effects);

    let value = ResidentEffectChain { passes, plate_passes };
    store.install(key, version, generation, value.clone());
    value
}

pub(crate) fn effect_chain_key(layer: &SceneLayerValue) -> Option<(GpuResourceKey, GpuResourceVersion)> {
    if layer.effect_keys.is_empty() && layer.after_effect_keys.is_empty() {
        return None;
    }
    let mut identity_hasher = std::collections::hash_map::DefaultHasher::new();
    layer.layer.hash(&mut identity_hasher);
    layer.instance.hash(&mut identity_hasher);
    layer.effect_keys.hash(&mut identity_hasher);
    layer.after_effect_keys.hash(&mut identity_hasher);
    let identity = super::types::GpuResourceIdentity::synthetic(
        identity_hasher.finish(),
        super::types::GpuResourceClass::Effect,
        0,
    );
    let mut encoded = crate::frame_graph::CanonicalEncoder::new();
    let _ = encoded.string(&format!("{:?}{:?}", layer.effects, layer.after_effects));
    Some((identity.key(), GpuResourceVersion::from_canonical(&encoded)))
}
