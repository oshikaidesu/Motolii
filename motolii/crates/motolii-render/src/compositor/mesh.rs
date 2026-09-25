use std::path::Path;
use std::sync::Arc;

use re_renderer::renderer::{GpuMeshInstance, MeshDrawData};

use crate::render::compositor::effects::surface_program::SurfaceShading;
use re_renderer::Color32;

use crate::render::compositor::{
    projected_spatial_placement, Compositor, CompositorError, GpuModelData,
};
use crate::render::media::SpatialBounds;

/// U79 spike: a stone at least this large on screen (radius, px) is a hero — shaded per sample (twin of
/// `JEWEL_HERO_PX` in `vism/material_filament.wgsl`).
const STONE_HERO_PX: f32 = 90.0;

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
        let vertices = crate::render::media::silhouette_points(cpu.instance_vertex_positions());
        let faceted = cpu.is_faceted();
        let instances = cpu
            .into_gpu_meshes(&self.ctx)
            .map_err(|error| CompositorError::Draw(error.to_string()))?;
        Ok(GpuModelData {
            planar_size: None,
            instances: Arc::new(instances),
            bounds,
            vertices: Arc::new(vertices),
            faceted,
            flat_parts: std::sync::Arc::from([]),
        })
    }

    pub(crate) fn model_instances(
        &mut self,
        model: &GpuModelData,
        size: [f32; 2],
        placement: crate::doc::core::LayerPlacement,
        opacity: f32,
        comp: crate::doc::core::CompSpec,
        camera: crate::doc::core::ResolvedCamera,
        projection: crate::doc::store::LayerProjection,
        shading: &SurfaceShading,
        clip: Option<super::ClipSpec>,
    ) -> (Vec<GpuMeshInstance>, re_renderer::ClipPlane) {
        let world_from_object = if let Some(natural) = model.planar_size {
            let (corner, u, v) = super::projected_placement_corners(comp, camera, projection, placement, glam::Vec2::ZERO, glam::Vec2::from(size));
            glam::Affine3A::from_cols((u / natural[0].max(1.0)).into(), (v / natural[1].max(1.0)).into(), u.cross(v).normalize_or_zero().into(), corner.into())
        } else { projected_spatial_placement(comp, camera, projection, placement, model.bounds) };
        let centre = world_from_object.transform_point3((glam::Vec3::from(model.bounds.min) + glam::Vec3::from(model.bounds.max)) * 0.5);
        let clip = clip.map_or(re_renderer::ClipPlane::NONE, |c| c.world(centre, world_from_object));
        let alpha = (opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
        let tint = Color32::from_rgba_unmultiplied(0, 0, 0, alpha);
        let mut program = shading.program.clone().or_else(|| self.standard_surface_program());
        // A solid's caps vary smoothly even when a field bends them on the GPU (a wave, a twist), so
        // they shade once per pixel too: MSAA still resolves the silhouette from coverage. Only the rim
        // (walls and bevels) keeps per-sample shading.
        let flat_program = shading.program_small.clone().or_else(|| program.clone());
        // U79 spike: a cut stone smaller than a hero on screen shades once per pixel.
        if model.faceted && shading.program_small.is_some() {
            let (scale, _, _) = world_from_object.to_scale_rotation_translation();
            let radius = ((glam::Vec3::from(model.bounds.max) - glam::Vec3::from(model.bounds.min)) * 0.5 * scale.abs()).max_element();
            let eye = crate::doc::core::camera_projection(comp, camera);
            let distance = (centre - eye.eye).length().max(1e-3);
            let pixels = radius / distance * comp.height as f32 * 0.5 / (eye.vertical_fov_radians * 0.5).tan();
            if pixels < STONE_HERO_PX {
                program = shading.program_small.clone();
            }
        }
        // U78 spike: a solid's surface gets its object frame (centre, radius, rotation) in spare slots.
        let mut params = shading.params;
        if params[super::effects::surface_program::SOLID_SLOT] > 0.0 {
            let (scale, rotation, _) = world_from_object.to_scale_rotation_translation();
            let half = (glam::Vec3::from(model.bounds.max) - glam::Vec3::from(model.bounds.min)) * 0.5 * scale.abs();
            params[12..15].copy_from_slice(&centre.to_array());
            params[15] = half.max_element();
            params[16..20].copy_from_slice(&rotation.to_array());
            params[21] = half.z;
        }
        let instances: Vec<GpuMeshInstance> = model
            .instances
            .iter()
            .cloned()
            .enumerate()
            .map(|(part, mut instance)| {
                instance.world_from_mesh = world_from_object * instance.world_from_mesh;
                instance.additive_tint = tint;
                instance.program = if model.flat_parts.get(part).copied().unwrap_or(false) { flat_program.clone() } else { program.clone() };
                instance.params = params;
                instance
            })
            .collect();
        (instances, clip)
    }

}
