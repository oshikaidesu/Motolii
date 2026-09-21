use crate::doc::core::{CompSpec, LayerPlacement, ResolvedCamera};
use crate::doc::store::LayerId;
use crate::frame_graph::{SceneContentValue, SceneValue};
use crate::render::compositor::{Layer, LayerWithPasses};
use crate::render::engine::{Engine, EngineError};

#[derive(Clone)]
pub(super) struct GpuSceneValue { pub layers: Vec<LayerWithPasses> }

impl Engine {
    pub(super) fn prepare_gpu_scene(&mut self, scene: &SceneValue, comp: CompSpec, projection_camera: ResolvedCamera) -> Result<GpuSceneValue, EngineError> {
        let mut layers = Vec::with_capacity(scene.layers.len());
        let mut identities = Vec::with_capacity(scene.layers.len());
        for source in &scene.layers {
            let key = LayerId(source.content_key.map_or(0, |key| key.as_u64()));
            let (content, natural) = match &source.content {
                SceneContentValue::None => continue,
                SceneContentValue::Text(text) => self.text_texture_from_shapes(&text.shapes(), key, comp)?,
                SceneContentValue::Shape(shapes) => self.shape_texture_from_shapes(shapes, key, true, 0.05, comp, None, true)?,
                SceneContentValue::Material(material) => self.mesh_content_for(&material.source.path, comp)?,
            };
            let Some(content) = content else { continue };
            let placement = LayerPlacement { transform: source.transform.affine, world_transform: Some(source.transform.spatial), order: i32::from(source.order), opacity: source.opacity, z: source.transform.spatial.translation.z, rotation_x: 0.0, rotation_y: 0.0, plane: None };
            let passes: Vec<_> = crate::render::engine::translate::translate_effect_passes(&source.effects).into_iter().chain(crate::render::engine::translate::translate_plate_passes(&source.after_effects)).collect();
            let layer = Layer { content, size: natural, placement, projection: source.projection, projection_camera, blend_mode: crate::render::engine::translate::translate_blend_mode(source.blend)?, shading: self.compositor.surface_shading_for(&source.effects, false).map_err(EngineError::Store)?, displace: crate::render::engine::translate::translate_point_displace(&source.effects), clip: crate::render::engine::translate::translate_clip(&source.effects), shadow: crate::render::engine::translate::translate_cast_shadow(&source.effects), outline: 0, frame: None };
            let layer = self.apply_masks_to_layer(layer, &source.masks, natural, None)?;
            layers.push(LayerWithPasses { layer, passes, padding: 0, pass_sources: Vec::new(), cut: Vec::new() });
            identities.push((source.layer, source.matte));
        }
        let by_id: std::collections::HashMap<_, _> = identities.iter().enumerate().map(|(index, (layer, _))| (*layer, index)).collect();
        let originals = layers.clone();
        for (index, (_, matte)) in identities.iter().enumerate() {
            let Some(matte) = matte else { continue };
            let Some(source_index) = by_id.get(&matte.layer).copied() else { continue };
            let target = self.apply_effects_before_matte(comp, projection_camera, originals[index].layer.clone(), &originals[index].passes)?;
            let source = self.apply_effects_before_matte(comp, projection_camera, originals[source_index].layer.clone(), &originals[source_index].passes)?;
            let layer = self.apply_matte(comp, projection_camera, &target, &source, matte.mode)?;
            layers[index] = LayerWithPasses { layer, passes: Vec::new(), padding: 0, pass_sources: Vec::new(), cut: Vec::new() };
        }
        Ok(GpuSceneValue { layers })
    }
}
