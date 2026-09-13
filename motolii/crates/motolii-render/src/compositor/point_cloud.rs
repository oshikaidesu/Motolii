
use re_renderer::renderer::PointCloudBatchFlags;
use re_renderer::{Color32, PointCloudBuilder, Size};

use crate::render::compositor::{
    projected_spatial_placement, Compositor, CompositorError,
};
use crate::render::media::SpatialBounds;

impl Compositor {
    /// 層の変形を `world_from_obj` に載せた点群の draw data。**焼かない** —
    /// 呼び手が板と同じ view へ積む(裁定 2026-08-30「3D は既定で空間に居る」)。
    pub(crate) fn point_cloud_draw_data(
        &mut self,
        positions: &[[f32; 3]],
        colors: &[[u8; 4]],
        bounds: SpatialBounds,
        point_size: f32,
        placement: crate::doc::core::LayerPlacement,
        opacity: f32,
        comp: crate::doc::core::CompSpec,
        camera: crate::doc::core::ResolvedCamera,
        projection: crate::doc::store::LayerProjection,
        displace: PointDisplace,
        clip: Option<super::ClipSpec>,
        outline: re_renderer::OutlineMaskPreference,
    ) -> Result<re_renderer::renderer::PointCloudDrawData, CompositorError> {
        let world_from_obj = projected_spatial_placement(comp, camera, projection, placement, bounds);
        let centre = world_from_obj.transform_point3((glam::Vec3::from(bounds.min) + glam::Vec3::from(bounds.max)) * 0.5);
        let clip = clip.map_or(re_renderer::ClipPlane::NONE, |c| c.world(centre, world_from_obj));
        let points = displaced_points(positions, world_from_obj, displace);
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

        let radii = vec![Size::new_ui_points(point_size.max(1e-4)); points.len()];
        let picking_ids = vec![Default::default(); points.len()];
        let mut builder = PointCloudBuilder::new(&self.ctx);
        builder
            .batch("motolii-point-cloud")
            .world_from_obj(world_from_obj)
            .clip(clip)
            .outline_mask_ids(outline)
            .add_points_slow(&points, &radii, &colors, &picking_ids)
            .flags(PointCloudBatchFlags::FLAG_ENABLE_SHADING);
        builder
            .into_draw_data()
            .map_err(|e| CompositorError::Draw(e.to_string()))
    }
}

/// Turbulent Displace の CPU の写し。点群は fork の hook を通らないので、同じ欄をここで受ける
/// (最小コアの継ぎ目 D。網は `vism/turbulent_displace.wgsl`、点群はこれ)。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointDisplace {
    pub amount: f32,
    pub size: f32,
    pub complexity: u32,
    pub evolution: f32,
    pub offset: glam::Vec3,
    /// Direction の軸の伏せ(XY なら Z が 0)。Normal は点群に法線が無いので XYZ と同じ。
    pub mask: glam::Vec3,
}

impl Default for PointDisplace {
    fn default() -> Self {
        Self { amount: 0.0, size: 100.0, complexity: 3, evolution: 0.0, offset: glam::Vec3::ZERO, mask: glam::Vec3::ONE }
    }
}

/// 点群の点を、網の頂点と同じ場(上流 `noise`、同じ座標の取り方)で動かす。点群には法線が無いので
/// Along に関わらずベクトル場として動かす。amount 0 はそのまま。
pub(crate) fn displaced_points(
    positions: &[[f32; 3]],
    world_from_obj: glam::Affine3A,
    displace: PointDisplace,
) -> Vec<glam::Vec3> {
    if displace.amount == 0.0 {
        return positions.iter().copied().map(glam::Vec3::from).collect();
    }
    let frame = world_from_obj.matrix3;
    let obj_from_frame = frame.inverse();
    let size = displace.size.max(1e-3);
    let octaves = displace.complexity.clamp(1, 8);
    positions
        .iter()
        .map(|p| {
            let p = glam::Vec3::from(*p);
            let frame_position = frame * p;
            let q = (frame_position + displace.offset) / size + displace.evolution * glam::vec3(0.53, 0.71, 0.89);
            let field = glam::vec3(
                re_renderer::noise::fbm3(q, octaves),
                re_renderer::noise::fbm3(q + glam::vec3(31.7, 0.0, 0.0), octaves),
                re_renderer::noise::fbm3(q + glam::vec3(0.0, 47.3, 0.0), octaves),
            );
            p + obj_from_frame * (field * displace.mask * displace.amount)
        })
        .collect()
}

#[cfg(test)]
mod displace_tests {
    use super::*;

    #[test]
    fn points_move_by_at_most_amount_and_evolution_changes_them() {
        let positions: Vec<[f32; 3]> = (0..50).map(|i| [i as f32 * 3.0, (i * 7 % 11) as f32, -(i as f32)]).collect();
        let still = displaced_points(&positions, glam::Affine3A::IDENTITY, Default::default());
        assert!(still.iter().zip(&positions).all(|(a, b)| *a == glam::Vec3::from(*b)));
        let field = PointDisplace { amount: 5.0, size: 10.0, ..Default::default() };
        let moved = displaced_points(&positions, glam::Affine3A::IDENTITY, field);
        let mut any = false;
        for (a, b) in moved.iter().zip(&positions) {
            let d = (*a - glam::Vec3::from(*b)).abs();
            assert!(d.max_element() <= 5.0 + 1e-3, "{d:?}");
            any |= d.max_element() > 0.5;
        }
        assert!(any, "何も動かない");
        let later = displaced_points(&positions, glam::Affine3A::IDENTITY, PointDisplace { evolution: 1.0, ..field });
        assert!(moved.iter().zip(&later).any(|(a, b)| (*a - *b).length() > 0.1));
    }
}
