
use re_renderer::renderer::PointCloudBatchFlags;
use re_renderer::view_builder::{
    BlendWithBackground, Projection, RenderMode, TargetConfiguration, ViewBuilder,
};
use re_renderer::{Color32, PointCloudBuilder, Rgba, Size, ViewBuilderId};

use crate::{Compositor, CompositorError, GpuTexture2D};

const POINT_CLOUD_VERTICAL_FOV_DEGREES: f32 = motolii_core::CAMERA_BASE_VERTICAL_FOV_DEGREES;

impl Compositor {
    pub fn render_point_cloud_to_texture(
        &mut self,
        positions: &[[f32; 3]],
        colors: &[[u8; 4]],
        width: u32,
        height: u32,
    ) -> Result<GpuTexture2D, CompositorError> {
        let positions: Vec<glam::Vec3> = positions.iter().copied().map(glam::Vec3::from).collect();
        let colors: Vec<Color32> = colors
            .iter()
            .map(|c| Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]))
            .collect();

        let (center, radius) = bounding_sphere(&positions);
        let half_fov = (POINT_CLOUD_VERTICAL_FOV_DEGREES * 0.5).to_radians();
        let distance = (radius.max(1e-3) * 1.2) / half_fov.tan();
        let eye = center + glam::Vec3::new(0.0, 0.0, distance);
        let view_from_world = macaw::IsoTransform::look_at_rh(eye, center, glam::Vec3::Y)
            .ok_or_else(|| {
                CompositorError::View("点群カメラを組めない(eye/target が縮退)".into())
            })?;

        self.ctx.begin_frame();

        let radii = vec![Size::new_ui_points(1.5); positions.len()];
        let picking_ids = vec![Default::default(); positions.len()];
        let mut builder = PointCloudBuilder::new(&self.ctx);
        builder
            .batch("motolii-point-cloud")
            .world_from_obj(glam::Affine3A::IDENTITY)
            .add_points_slow(&positions, &radii, &colors, &picking_ids)
            .flags(PointCloudBatchFlags::FLAG_ENABLE_SHADING);
        let draw_data = builder
            .into_draw_data()
            .map_err(|e| CompositorError::Draw(e.to_string()))?;

        let owned = self.create_blend_scratch_texture(width, height);
        let mut view_builder = ViewBuilder::new_with_external_resolved(
            &self.ctx,
            TargetConfiguration {
                name: "motolii-point-cloud".into(),
                render_mode: RenderMode::Deterministic,
                resolution_in_pixel: [width, height],
                view_from_world,
                projection_from_view: Projection::Perspective {
                    vertical_fov: half_fov * 2.0,
                    near_plane_distance: motolii_core::NEAR_PLANE,
                    aspect_ratio: width as f32 / height as f32,
                },
                pixels_per_point: 1.0,
                blend_with_background: BlendWithBackground::Premultiplied,
                ..Default::default()
            },
            ViewBuilderId::new(self.next_readback),
            &owned,
        )
        .map_err(|e| CompositorError::View(e.to_string()))?;
        self.next_readback += 1;

        view_builder.queue_draw(&self.ctx, draw_data);
        let command_buffer = view_builder
            .draw(&self.ctx, Rgba::TRANSPARENT)
            .map_err(|e| CompositorError::Draw(e.to_string()))?;

        self.ctx.before_submit();
        self.ctx.queue.submit([command_buffer]);
        self.sequential_submits += 1;
        self.ctx.begin_frame();
        self.ctx
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| CompositorError::Draw(e.to_string()))?;

        self.next_effect_key += 1;
        let key = self.next_effect_key;
        self.ctx
            .texture_manager_2d
            .import_gpu_premultiplied(key, &self.ctx, &owned)
            .map_err(|e| CompositorError::Effect(e.to_string()))
    }
}

fn bounding_sphere(positions: &[glam::Vec3]) -> (glam::Vec3, f32) {
    if positions.is_empty() {
        return (glam::Vec3::ZERO, 1.0);
    }
    let mut min = positions[0];
    let mut max = positions[0];
    for &p in &positions[1..] {
        min = min.min(p);
        max = max.max(p);
    }
    let center = (min + max) * 0.5;
    let radius = positions
        .iter()
        .map(|&p| (p - center).length())
        .fold(0.0_f32, f32::max);
    (center, radius)
}
