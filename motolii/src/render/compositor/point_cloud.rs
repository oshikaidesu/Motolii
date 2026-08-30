
use re_renderer::renderer::PointCloudBatchFlags;
use re_renderer::{Color32, PointCloudBuilder, Size};

use crate::render::compositor::{Compositor, CompositorError};

impl Compositor {
    /// 層の変形を `world_from_obj` に載せた点群の draw data。**焼かない** —
    /// 呼び手が板と同じ view へ積む(裁定 2026-08-30「3D は既定で空間に居る」)。
    pub(crate) fn point_cloud_draw_data(
        &mut self,
        positions: &[[f32; 3]],
        colors: &[[u8; 4]],
        point_size: f32,
        transform: glam::Affine2,
        z: f32,
        opacity: f32,
    ) -> Result<re_renderer::renderer::PointCloudDrawData, CompositorError> {
        let points: Vec<glam::Vec3> = positions.iter().copied().map(glam::Vec3::from).collect();
        let alpha = (opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
        let colors: Vec<Color32> = colors
            .iter()
            .map(|c| {
                Color32::from_rgba_unmultiplied(
                    c[0],
                    c[1],
                    c[2],
                    ((c[3] as u16 * alpha as u16) / 255) as u8,
                )
            })
            .collect();

        let m = transform.matrix2;
        let t = transform.translation;
        let world_from_obj = glam::Affine3A::from_cols(
            glam::Vec3A::new(m.x_axis.x, m.x_axis.y, 0.0),
            glam::Vec3A::new(m.y_axis.x, m.y_axis.y, 0.0),
            glam::Vec3A::new(0.0, 0.0, 1.0),
            glam::Vec3A::new(t.x, t.y, z),
        );

        let radii = vec![Size::new_scene_units(point_size.max(1e-4)); points.len()];
        let picking_ids = vec![Default::default(); points.len()];
        let mut builder = PointCloudBuilder::new(&self.ctx);
        builder
            .batch("motolii-point-cloud")
            .world_from_obj(world_from_obj)
            .add_points_slow(&points, &radii, &colors, &picking_ids)
            .flags(PointCloudBatchFlags::FLAG_ENABLE_SHADING);
        builder
            .into_draw_data()
            .map_err(|e| CompositorError::Draw(e.to_string()))
    }
}
