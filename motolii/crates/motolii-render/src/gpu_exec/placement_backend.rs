use crate::doc::core::LayerPlacement;
use crate::frame_graph::SceneLayerValue;

use super::resource_store::GpuResourceStore;
use super::types::{GpuResourceKey, GpuResourceVersion};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ResidentPlacement {
    pub placement: LayerPlacement,
}

impl ResidentPlacement {
    pub fn from_scene(layer: &SceneLayerValue) -> Self {
        Self {
            placement: LayerPlacement {
                transform: layer.transform.affine,
                world_transform: Some(layer.transform.spatial),
                order: i32::from(layer.order),
                opacity: layer.opacity,
                z: layer.transform.spatial.translation.z,
                rotation_x: 0.0,
                rotation_y: 0.0,
                plane: None,
            },
        }
    }
}

pub(crate) fn resident_placement(
    store: &mut GpuResourceStore<ResidentPlacement>,
    key: GpuResourceKey,
    version: GpuResourceVersion,
    generation: u64,
    layer: &SceneLayerValue,
) -> ResidentPlacement {
    if let Some(hit) = store.current(key, version).copied() {
        store.touch(key, generation);
        return hit;
    }
    let value = ResidentPlacement::from_scene(layer);
    store.install(key, version, generation, value);
    value
}
