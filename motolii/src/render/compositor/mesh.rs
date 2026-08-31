
use re_renderer::mesh::{CpuMesh, GpuMesh, Material};
use re_renderer::renderer::{GpuMeshInstance, MeshDrawData};
use re_renderer::{Color32, Rgba32Unmul};

use crate::render::compositor::{Compositor, CompositorError};

fn smallvec_one(material: Material) -> smallvec::SmallVec<[Material; 1]> {
    let mut out = smallvec::SmallVec::new();
    out.push(material);
    out
}

impl Compositor {
    /// 材質が要求する最小の albedo。作り直さないよう1枚だけ持つ。
    fn white_pixel(&mut self) -> Result<re_renderer::resource_managers::GpuTexture2D, CompositorError> {
        if let Some(texture) = self.white_pixel.clone() {
            return Ok(texture);
        }
        let texture = self.upload_rgba("motolii-white", &[255, 255, 255, 255], 1, 1)?;
        self.white_pixel = Some(texture.clone());
        Ok(texture)
    }

    /// 層の変形を `world_from_mesh` に載せた網の draw data。**焼かない** —
    /// 呼び手が板と同じ view へ積む(裁定 2026-08-30「3D は既定で空間に居る」)。
    pub(crate) fn mesh_draw_data(
        &mut self,
        positions: &[[f32; 3]],
        indices: &[[u32; 3]],
        normals: &[[f32; 3]],
        colors: &[[u8; 4]],
        transform: glam::Affine2,
        z: f32,
        opacity: f32,
    ) -> Result<MeshDrawData, CompositorError> {
        let alpha = (opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
        let cpu = CpuMesh {
            label: "motolii-mesh".into(),
            triangle_indices: indices.iter().map(|i| glam::UVec3::from(*i)).collect(),
            vertex_positions: positions.iter().copied().map(glam::Vec3::from).collect(),
            vertex_colors: colors
                .iter()
                .map(|c| {
                    Rgba32Unmul::from_rgba_unmul_array([
                        c[0],
                        c[1],
                        c[2],
                        ((c[3] as u16 * alpha as u16) / 255) as u8,
                    ])
                })
                .collect(),
            vertex_normals: normals.iter().copied().map(glam::Vec3::from).collect(),
            vertex_texcoords: vec![glam::Vec2::ZERO; positions.len()],
            bbox: macaw::BoundingBox::from_points(
                positions.iter().map(|p| glam::Vec3::from(*p)),
            ),
            // 材質が1つも無いと1回も描かれない(上流は材質ごとに描く)。
            // 色は頂点色に任せ、albedo は白の1画素。
            materials: smallvec_one(Material {
                label: "motolii-mesh".into(),
                index_range: 0..(indices.len() as u32 * 3),
                albedo: self.white_pixel()?,
                albedo_factor: re_renderer::Rgba::WHITE,
            }),
        };
        let gpu = GpuMesh::new(&self.ctx, &cpu)
            .map_err(|e| CompositorError::Draw(e.to_string()))?;

        let m = transform.matrix2;
        let t = transform.translation;
        let world_from_mesh = glam::Affine3A::from_cols(
            glam::Vec3A::new(m.x_axis.x, m.x_axis.y, 0.0),
            glam::Vec3A::new(m.y_axis.x, m.y_axis.y, 0.0),
            glam::Vec3A::new(0.0, 0.0, 1.0),
            glam::Vec3A::new(t.x, t.y, z),
        );

        let instance = GpuMeshInstance {
            gpu_mesh: std::sync::Arc::new(gpu),
            world_from_mesh,
            // 黒 = 足し色なし。透明にすると上流が「全部透明」と見て不透明の段に入らない。
            additive_tint: Color32::BLACK,
            outline_mask_ids: Default::default(),
            picking_layer_id: Default::default(),
            cull_mode: None,
        };
        MeshDrawData::new(&self.ctx, &[instance])
            .map_err(|e| CompositorError::Draw(e.to_string()))
    }
}
