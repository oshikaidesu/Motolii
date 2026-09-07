use std::path::Path;
use std::sync::Arc;

use re_renderer::renderer::{GpuMeshInstance, MeshDrawData, MeshProgram};

use crate::render::compositor::effects::mesh_program::{self, MeshShading};
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

    /// 効果列の hook(field / surface)から網の描き方を組む。変種は catalog の世代ごとに覚える。
    pub(crate) fn mesh_shading(&mut self, effects: &[crate::doc::store::ResolvedEffect]) -> Result<MeshShading, String> {
        self.refresh_catalog_programs();
        let catalog = self.catalog.clone();
        let (field, surface) = mesh_program::hooks(effects, &catalog.definitions);
        if field.is_none() && surface.is_none() {
            return Ok(MeshShading::default());
        }
        let key = format!("{}|{}|{}", field.map_or("", |d| d.plugin_id()), surface.map_or("", |d| d.plugin_id()), catalog.generation);
        let program = match self.mesh_programs.get(&key) {
            Some(program) => program.clone(),
            None => {
                let desc = mesh_program::program_desc(field, surface)?;
                let program = Arc::new(MeshProgram::new(&self.ctx, desc).map_err(|e| e.to_string())?);
                self.mesh_programs.insert(key, program.clone());
                program
            }
        };
        Ok(MeshShading { program: Some(program), params: mesh_program::params(effects, field, surface) })
    }

    pub(crate) fn model_draw_data(
        &mut self,
        model: &GpuModelData,
        placement: crate::doc::core::LayerPlacement,
        opacity: f32,
        comp: crate::doc::core::CompSpec,
        camera: crate::doc::core::ResolvedCamera,
        projection: crate::doc::store::LayerProjection,
        shading: &MeshShading,
    ) -> Result<MeshDrawData, CompositorError> {
        let world_from_object = projected_spatial_placement(comp, camera, projection, placement, model.bounds);
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
        MeshDrawData::new(&self.ctx, &instances)
            .map_err(|error| CompositorError::Draw(error.to_string()))
    }
}

#[cfg(test)]
mod program_contract {
    use crate::doc::store::ResolvedEffect;

    fn compiled_without_validation_error(compositor: &mut crate::render::compositor::Compositor, effects: &[ResolvedEffect]) {
        let scope = compositor.ctx.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let shading = compositor.mesh_shading(effects).unwrap();
        let error = pollster::block_on(scope.pop());
        assert!(error.is_none(), "{}", error.unwrap());
        assert_eq!(shading.program.is_some(), !effects.is_empty());
    }

    /// wgpu の validation error は非同期なので、error scope で拾って契約にする:
    /// 既定の変種と、棚の hook(Glass・Turbulent Displace・両方)を差した変種が compile できる。
    #[test]
    fn default_and_shelf_hook_programs_compile() {
        let mut compositor = crate::render::compositor::Compositor::headless().unwrap();
        let scope = compositor.ctx.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let desc = re_renderer::renderer::MeshProgramDesc { label: "probe".into(), field: None, surface: None };
        re_renderer::renderer::MeshProgram::new(&compositor.ctx, desc).unwrap();
        let error = pollster::block_on(scope.pop());
        assert!(error.is_none(), "{}", error.unwrap());

        let glass = ResolvedEffect { plugin_id: "motolii.glass".into(), params: vec![] };
        let turbulence = ResolvedEffect { plugin_id: "motolii.turbulent_displace".into(), params: vec![] };
        compiled_without_validation_error(&mut compositor, &[]);
        compiled_without_validation_error(&mut compositor, std::slice::from_ref(&glass));
        compiled_without_validation_error(&mut compositor, std::slice::from_ref(&turbulence));
        compiled_without_validation_error(&mut compositor, &[turbulence.clone(), glass.clone()]);
        // 同じ組は同じ変種。
        let a = compositor.mesh_shading(&[turbulence.clone(), glass.clone()]).unwrap().program.unwrap();
        let b = compositor.mesh_shading(&[glass, turbulence]).unwrap().program.unwrap();
        assert!(std::sync::Arc::ptr_eq(&a, &b));
        assert_eq!(compositor.mesh_programs.len(), 3);
    }
}
