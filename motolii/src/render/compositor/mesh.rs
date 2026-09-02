use std::path::Path;
use std::sync::Arc;

use re_renderer::renderer::{GpuMeshInstance, MeshDrawData};
use re_renderer::Color32;

use crate::render::compositor::{
    spatial_world_from_bounds, Compositor, CompositorError, GpuModelData,
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
        transform: glam::Affine2,
        z: f32,
        rotation_x: f32,
        rotation_y: f32,
        opacity: f32,
    ) -> Result<MeshDrawData, CompositorError> {
        let world_from_object = spatial_world_from_bounds(
            transform,
            z,
            rotation_x,
            rotation_y,
            model.bounds,
        );
        let alpha = (opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
        let tint = Color32::from_rgba_unmultiplied(0, 0, 0, alpha);
        let instances: Vec<GpuMeshInstance> = model
            .instances
            .iter()
            .cloned()
            .map(|mut instance| {
                instance.world_from_mesh = world_from_object * instance.world_from_mesh;
                instance.additive_tint = tint;
                instance
            })
            .collect();
        MeshDrawData::new(&self.ctx, &instances)
            .map_err(|error| CompositorError::Draw(error.to_string()))
    }
}
