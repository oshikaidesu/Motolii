use std::path::Path;
use std::sync::Arc;

use re_renderer::renderer::{GpuMeshInstance, MeshDrawData};

use crate::render::compositor::effects::surface_program::SurfaceShading;
use re_renderer::Color32;

use crate::render::compositor::{
    projected_spatial_placement, Compositor, CompositorError, GpuModelData,
};
use crate::render::media::SpatialBounds;

impl Compositor {
    pub(crate) fn import_model(&mut self, path: &str) -> Result<GpuModelData, CompositorError> {
        let bytes = std::fs::read(path).map_err(|error| {
            CompositorError::Draw(format!("3D素材を読めない: {path}: {error}"))
        })?;
        let extension = Path::new(path)
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase)
            .ok_or_else(|| CompositorError::Draw(format!("3D素材に拡張子が無い: {path}")))?;
        let cpu = match extension.as_str() {
            "glb" => {
                re_renderer::importer::gltf::load_gltf_from_buffer(path, &bytes, &self.ctx)
                    .map_err(|error| CompositorError::Draw(error.to_string()))?
            }
            "obj" => re_renderer::importer::obj::load_obj_from_buffer(&bytes, &self.ctx)
                .map_err(|error| CompositorError::Draw(error.to_string()))?,
            "stl" => re_renderer::importer::stl::load_stl_from_buffer(&bytes, &self.ctx)
                .map_err(|error| CompositorError::Draw(error.to_string()))?,
            _ => {
                return Err(CompositorError::Draw(format!(
                    "対応していない3D形式: {extension}"
                )))
            }
        };
        let bbox = cpu.bbox();
        let bounds = SpatialBounds::from_points([bbox.min.to_array(), bbox.max.to_array()])
            .map_err(|error| CompositorError::Draw(error.to_string()))?;
        let instances = cpu
            .into_gpu_meshes(&self.ctx)
            .map_err(|error| CompositorError::Draw(error.to_string()))?;
        Ok(GpuModelData {
            instances: Arc::new(instances),
            bounds,
        })
    }

    pub(crate) fn model_draw_data(
        &mut self,
        model: &GpuModelData,
        placement: crate::doc::core::LayerPlacement,
        opacity: f32,
        comp: crate::doc::core::CompSpec,
        camera: crate::doc::core::ResolvedCamera,
        projection: crate::doc::store::LayerProjection,
        shading: &SurfaceShading,
        clip: Option<super::ClipSpec>,
    ) -> Result<MeshDrawData, CompositorError> {
        let world_from_object = projected_spatial_placement(comp, camera, projection, placement, model.bounds);
        let centre = world_from_object.transform_point3((glam::Vec3::from(model.bounds.min) + glam::Vec3::from(model.bounds.max)) * 0.5);
        let clip = clip.map_or(re_renderer::ClipPlane::NONE, |c| c.world(centre, world_from_object));
        let alpha = (opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
        let tint = Color32::from_rgba_unmultiplied(0, 0, 0, alpha);
        let instances: Vec<GpuMeshInstance> = model
            .instances
            .iter()
            .cloned()
            .map(|mut instance| {
                instance.world_from_mesh = world_from_object * instance.world_from_mesh;
                instance.additive_tint = tint;
                instance.program = shading.program.clone();
                instance.params = shading.params;
                instance
            })
            .collect();
        MeshDrawData::new_clipped(&self.ctx, &instances, clip)
            .map_err(|error| CompositorError::Draw(error.to_string()))
    }
}
