use crate::doc::store::LayerId;
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

    let mut passes = crate::render::engine::translate::translate_effect_passes(&layer.effects);
    let direct_screen = passes.iter()
        .any(|pass| pass.reads_backdrop || pass.reads_composite())
        .then_some(screen);
    crate::render::engine::translate::stamp_feedback(
        &mut passes,
        layer.layer,
        layer.instance,
        0,
        direct_screen,
        0,
    );

    let mut plate_passes = crate::render::engine::translate::translate_plate_passes(&layer.after_effects);
    let plate_screen = plate_passes.iter()
        .any(|pass| pass.reads_backdrop || pass.reads_composite())
        .then_some(screen);
    crate::render::engine::translate::stamp_feedback(
        &mut plate_passes,
        layer.layer,
        layer.instance,
        1,
        plate_screen,
        0,
    );

    let value = ResidentEffectChain { passes, plate_passes };
    store.install(key, version, generation, value.clone());
    value
}

pub(crate) fn effect_chain_key(layer: &SceneLayerValue) -> Option<(GpuResourceKey, GpuResourceVersion)> {
    let first = layer.content_key?;
    let identity = super::types::GpuResourceIdentity::semantic(
        first,
        super::types::GpuResourceClass::Effect,
        layer.instance,
    );
    let mut encoded = crate::frame_graph::CanonicalEncoder::new();
    let _ = encoded.string(&format!("{:?}{:?}", layer.effects, layer.after_effects));
    Some((identity.key(), GpuResourceVersion::from_canonical(&encoded)))
}
