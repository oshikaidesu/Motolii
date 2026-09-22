use crate::doc::core::{CompSpec, LayerPlacement};
use crate::frame_graph::{SceneLayerValue, ScenePlateValue, SceneValue};
use crate::render::compositor::{BlendMode as CompositeBlendMode, LayerContent, LayerWithPasses};
use crate::render::engine::ResolvedCamera;

use super::resource_store::GpuResourceStore;
use super::types::{GpuIdentitySource, GpuResourceClass, GpuResourceIdentity, GpuResourceVersion};

#[derive(Clone)]
pub(crate) struct ResidentCompositeLayer {
    pub layer: LayerWithPasses,
}

fn identity(source: &SceneLayerValue, class: GpuResourceClass) -> GpuResourceIdentity {
    GpuResourceIdentity {
        source: GpuIdentitySource::Synthetic(source.layer.0),
        class,
        slot: source.instance,
    }
}

fn version(source: &SceneLayerValue, class: GpuResourceClass) -> GpuResourceVersion {
    let mut encoded = crate::frame_graph::CanonicalEncoder::new();
    encoded.u64(source.layer.0).u32(source.instance).u8(class as u8);
    let _ = encoded.string(&format!("{:?}{:?}", source.content, source.matte));
    GpuResourceVersion::from_canonical(&encoded)
}

impl crate::render::engine::Engine {
    pub(crate) fn gpu_plate(
        &mut self,
        store: &mut GpuResourceStore<ResidentCompositeLayer>,
        source: &SceneLayerValue,
        plate: &ScenePlateValue,
        comp: CompSpec,
        camera: ResolvedCamera,
        generation: u64,
    ) -> Result<Option<(LayerContent, [f32; 2])>, crate::render::engine::EngineError> {
        let id = identity(source, GpuResourceClass::Plate);
        let key = id.key();
        let ver = version(source, GpuResourceClass::Plate);
        if let Some(hit) = store.current(key, ver).cloned() {
            store.touch(key, generation);
            return Ok(Some((hit.layer.layer.content, hit.layer.layer.size)));
        }
        let nested = SceneValue { layers: plate.members.iter().filter_map(|member| member.layer.clone()).collect() };
        let prepared = self.prepare_gpu_scene(&nested, comp, camera)?;
        if prepared.layers.is_empty() { return Ok(None); }
        let placement = LayerPlacement {
            transform: glam::Affine2::IDENTITY,
            world_transform: None,
            opacity: 1.0,
            order: i32::from(source.order),
            z: 0.0,
            rotation_x: 0.0,
            rotation_y: 0.0,
            plane: None,
        };
        let baked = self.bake_isolated_layers(
            comp, camera, prepared.layers, CompositeBlendMode::Normal, placement, plate.average,
        )?;
        let wrapped = LayerWithPasses { layer: baked.clone(), passes: Vec::new(), padding: 0, pass_sources: Vec::new(), cut: Vec::new() };
        store.install(key, ver, generation, ResidentCompositeLayer { layer: wrapped });
        Ok(Some((baked.content, baked.size)))
    }

    pub(crate) fn gpu_matte(
        &mut self,
        store: &mut GpuResourceStore<ResidentCompositeLayer>,
        source: &SceneLayerValue,
        target: &LayerWithPasses,
        matte_source: &LayerWithPasses,
        comp: CompSpec,
        camera: ResolvedCamera,
        generation: u64,
    ) -> Result<LayerWithPasses, crate::render::engine::EngineError> {
        let id = identity(source, GpuResourceClass::Matte);
        let key = id.key();
        let ver = version(source, GpuResourceClass::Matte);
        if let Some(hit) = store.current(key, ver).cloned() {
            store.touch(key, generation);
            return Ok(hit.layer);
        }
        let target_layer = self.apply_effects_before_matte(comp, camera, target.layer.clone(), &target.passes)?;
        let source_layer = self.apply_effects_before_matte(comp, camera, matte_source.layer.clone(), &matte_source.passes)?;
        let matte = source.matte.expect("gpu_matte requires matte");
        let layer = self.apply_matte(comp, camera, &target_layer, &source_layer, matte.mode)?;
        let result = LayerWithPasses { layer, passes: Vec::new(), padding: 0, pass_sources: Vec::new(), cut: Vec::new() };
        store.install(key, ver, generation, ResidentCompositeLayer { layer: result.clone() });
        Ok(result)
    }
}
