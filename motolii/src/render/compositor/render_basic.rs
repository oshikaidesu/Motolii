use re_renderer::renderer::{
    RectangleDrawData, RectangleOptions,
    TexturedRect,
};
use re_renderer::view_builder::{
    BlendWithBackground, Projection, RenderMode, TargetConfiguration, ViewBuilder,
};
use re_renderer::Rgba;

use crate::render::compositor::*;

impl Compositor {
    pub fn render(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layers: &[Layer],
    ) -> Result<Vec<u8>, CompositorError> {
        self.render_with_timing(comp, camera, layers)
            .map(|(frame, _)| frame)
    }

    pub fn render_with_timing(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layers: &[Layer],
    ) -> Result<(Vec<u8>, RenderTiming), CompositorError> {
        let mut timing = RenderTiming::default();
        let build_start = std::time::Instant::now();

        let projection = crate::doc::core::camera_projection(comp, camera);
        let pinned_cancel = crate::doc::core::camera_screen_from_world_z0(comp, camera).inverse();

        let rects: Vec<TexturedRect> = layers
            .iter()
            .map(|layer| {
                let (transform, z) = if layer.pinned {
                    (pinned_cancel * layer.placement.transform, 0.0)
                } else {
                    (layer.placement.transform, layer.placement.z)
                };
                let a = fixed_function_tint_alpha(layer.blend_mode, layer.placement.opacity)?;
                Ok(TexturedRect {
                    top_left_corner_position: to_point3(
                        transform.transform_point2(glam::Vec2::ZERO),
                        z,
                    ),
                    extent_u: to_vector3(
                        transform.transform_vector2(glam::Vec2::new(layer.size[0], 0.0)),
                    ),
                    extent_v: to_vector3(
                        transform.transform_vector2(glam::Vec2::new(0.0, layer.size[1])),
                    ),
                    colormapped_texture: crate::render::compositor::premultiplied_texture(
                        layer
                            .content
                            .texture()
                            .ok_or_else(|| {
                                CompositorError::Draw("3D の素材はこの経路では描かない".into())
                            })?
                            .clone(),
                    ),
                    options: RectangleOptions {
                        multiplicative_tint: Rgba::from_rgba_premultiplied(
                            layer.placement.opacity,
                            layer.placement.opacity,
                            layer.placement.opacity,
                            a,
                        ),
                        depth_offset: layer.placement.order,
                        ..Default::default()
                    },
                })
            })
            .collect::<Result<Vec<_>, CompositorError>>()?;

        self.ctx.begin_frame();

        let draw_data = RectangleDrawData::new(&self.ctx, &rects)
            .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

        let view_from_world = macaw::IsoTransform::from_rotation_translation(
            projection.rotation,
            -(projection.rotation * projection.eye),
        );

        let mut view_builder = ViewBuilder::new(
            &self.ctx,
            TargetConfiguration {
                name: "motolii-comp".into(),
                render_mode: RenderMode::Deterministic,
                resolution_in_pixel: [comp.width, comp.height],
                view_from_world,
                projection_from_view: Projection::Perspective {
                    vertical_fov: projection.vertical_fov_radians,
                    near_plane_distance: projection.near_plane_distance,
                    aspect_ratio: projection.aspect_ratio,
                },
                pixels_per_point: 1.0,
                blend_with_background: BlendWithBackground::Premultiplied,
                ..Default::default()
            },
            re_renderer::ViewBuilderId::new(self.next_readback),
        )
        .map_err(|e| CompositorError::View(e.to_string()))?;

        view_builder.queue_draw(&self.ctx, draw_data);

        let identifier = self.next_readback;
        self.next_readback += 1;
        view_builder
            .schedule_screenshot(&self.ctx, identifier, ())
            .map_err(|e| CompositorError::View(e.to_string()))?;

        let command_buffer = view_builder
            .draw(&self.ctx, Rgba::TRANSPARENT)
            .map_err(|e| CompositorError::Draw(e.to_string()))?;
        timing.build_us = build_start.elapsed().as_micros();

        let gpu_start = std::time::Instant::now();

        self.ctx.before_submit();
        self.ctx.queue.submit([command_buffer]);

        self.ctx.begin_frame();
        self.ctx
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| CompositorError::Draw(e.to_string()))?;
        timing.gpu_us = gpu_start.elapsed().as_micros();

        let readback_start = std::time::Instant::now();
        self.ctx.begin_frame();

        let mut out: Option<Vec<u8>> = None;
        re_renderer::ScreenshotProcessor::next_readback_result::<()>(
            &self.ctx,
            identifier,
            |data, _extent, ()| {
                out = Some(data.to_vec());
            },
        );

        let frame = out.ok_or(CompositorError::ReadbackMissing)?;
        timing.readback_us = readback_start.elapsed().as_micros();
        Ok((frame, timing))
    }

}
