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
        for source in &scene.layers {
            let key = LayerId(source.content_key.map_or(0, |key| key.as_u64()));
            let (content, natural) = match &source.content {
                SceneContentValue::None => continue,
                SceneContentValue::Text(text) => self.shape_texture_from_shapes(&text.shapes(), key, true, 0.05, comp, None, true)?,
                SceneContentValue::Shape(shapes) => self.shape_texture_from_shapes(shapes, key, true, 0.05, comp, None, true)?,
                SceneContentValue::Material(material) => self.mesh_content_for(&material.source.path, comp)?,
            };
            let Some(content) = content else { continue };
            let placement = LayerPlacement { transform: source.transform.affine, world_transform: Some(source.transform.spatial), order: i32::from(source.order), opacity: source.opacity, z: source.transform.spatial.translation.z, rotation_x: 0.0, rotation_y: 0.0, plane: None };
            layers.push(LayerWithPasses { layer: Layer { content, size: natural, placement, projection: source.projection, projection_camera, blend_mode: crate::render::engine::translate::translate_blend_mode(source.blend)?, shading: Default::default(), displace: Default::default(), clip: None, shadow: 0.0, outline: 0, frame: None }, passes: Vec::new(), padding: 0, pass_sources: Vec::new(), cut: Vec::new() });
        }
        Ok(GpuSceneValue { layers })
    }
}
