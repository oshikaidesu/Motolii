use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::frame_graph::SceneLayerValue;
use crate::render::compositor::Layer;

use super::resource_store::GpuResourceStore;
use super::types::{GpuIdentitySource, GpuResourceClass, GpuResourceIdentity, GpuResourceKey, GpuResourceVersion};

#[derive(Clone)]
pub(crate) struct ResidentProcessedLayer {
    pub layer: Layer,
}

fn identity(source: &SceneLayerValue, class: GpuResourceClass) -> GpuResourceIdentity {
    let tag = source.content_key.map_or(source.layer.0, |key| key.as_u64());
    GpuResourceIdentity { source: GpuIdentitySource::Synthetic(tag), class, slot: source.instance }
}

fn version(source: &SceneLayerValue, class: GpuResourceClass) -> GpuResourceVersion {
    let mut encoded = crate::frame_graph::CanonicalEncoder::new();
    encoded.u64(source.layer.0).u32(source.instance).u8(class as u8);
    let _ = encoded.string(&format!("{:?}{:?}{}{:?}", source.masks, source.matte, source.flatten, source.content));
    GpuResourceVersion::from_canonical(&encoded)
}

impl crate::render::engine::Engine {
    pub(crate) fn gpu_process_mask_flatten(
        &mut self,
        store: &mut GpuResourceStore<ResidentProcessedLayer>,
        source: &SceneLayerValue,
        mut layer: Layer,
        natural: [f32; 2],
        frame: Option<crate::render::compositor::effects::vism::ImageFrame>,
        comp: CompSpec,
        camera: ResolvedCamera,
        generation: u64,
    ) -> Result<Layer, crate::render::engine::EngineError> {
        if !source.masks.is_empty() {
            let id = identity(source, GpuResourceClass::Mask);
            let key = id.key();
            let ver = version(source, GpuResourceClass::Mask);
            if let Some(hit) = store.current(key, ver).cloned() {
                layer = hit.layer;
                store.touch(key, generation);
            } else {
                layer = self.apply_masks_to_layer(layer, &source.masks, natural, frame)?;
                store.install(key, ver, generation, ResidentProcessedLayer { layer: layer.clone() });
            }
        }
        if source.flatten {
            let id = identity(source, GpuResourceClass::Flatten);
            let key = id.key();
            let ver = version(source, GpuResourceClass::Flatten);
            if let Some(hit) = store.current(key, ver).cloned() {
                layer = hit.layer;
                store.touch(key, generation);
            } else {
                layer = self.flatten_if_asked(comp, camera, layer, true)?;
                store.install(key, ver, generation, ResidentProcessedLayer { layer: layer.clone() });
            }
        }
        Ok(layer)
    }
}
