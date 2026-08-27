//! 共有面へ書く口(裁定256)。
//!
//! Makepad / OS handle を知らない。渡された `wgpu::Texture` が Host 仕様
//! (`Rgba8UnormSrgb`・comp サイズ・`RENDER_ATTACHMENT`)かを見て、
//! `ViewBuilder::new_with_external_resolved` でその面へ直接書く。
//! ここから blit しない。

use re_renderer::renderer::{ColormappedTexture, RectangleDrawData, RectangleOptions, TexturedRect};
use re_renderer::view_builder::ViewBuilder;
use re_renderer::{Rgba, ViewBuilderId};

use crate::{
    sequential_target_config, to_point3, to_vector3, CompSpec, Compositor, CompositorError,
    LayerWithPasses, ResolvedCamera,
};

impl Compositor {
    pub fn device(&self) -> &wgpu::Device {
        &self.ctx.device
    }

    /// 共有面へ直接書く口(裁定256)。検査失敗では内部状態を変えない。
    pub fn render_into(
        &mut self,
        target: &wgpu::Texture,
        comp: CompSpec,
        camera: ResolvedCamera,
        layers: &[LayerWithPasses],
    ) -> Result<(), CompositorError> {
        check_presentable_target(target, comp)?;

        let projection = motolii_core::camera_projection(comp, camera);
        let pinned_cancel = motolii_core::camera_screen_from_world_z0(comp, camera).inverse();
        let view_from_world = macaw::IsoTransform::from_rotation_translation(
            projection.rotation,
            -(projection.rotation * projection.eye),
        );

        self.ctx.begin_frame();

        let mut rects: Vec<TexturedRect> = Vec::with_capacity(layers.len());
        for lwp in layers {
            let layer = &lwp.layer;
            let (transform, z) = if layer.pinned {
                (pinned_cancel * layer.placement.transform, 0.0)
            } else {
                (layer.placement.transform, layer.placement.z)
            };
            let a = match layer.blend_mode {
                crate::BlendMode::Add => 0.0,
                _ => layer.placement.opacity,
            };
            rects.push(TexturedRect {
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
                colormapped_texture: ColormappedTexture::from_unorm_rgba(layer.texture.clone()),
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
            });
        }

        let draw_data = RectangleDrawData::new(&self.ctx, &rects)
            .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

        let mut view_builder = ViewBuilder::new_with_external_resolved(
            &self.ctx,
            sequential_target_config(
                "motolii-comp-presentable",
                comp,
                view_from_world,
                projection,
            ),
            ViewBuilderId::new(self.next_readback),
            target,
        )
        .map_err(|e| CompositorError::View(e.to_string()))?;
        self.next_readback += 1;

        view_builder.queue_draw(&self.ctx, draw_data);
        let command_buffer = view_builder
            .draw(&self.ctx, Rgba::TRANSPARENT)
            .map_err(|e| CompositorError::Draw(e.to_string()))?;
        self.ctx.before_submit();
        self.ctx.queue.submit([command_buffer]);
        self.ctx.begin_frame();
        self.ctx
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| CompositorError::Draw(e.to_string()))?;
        Ok(())
    }
}

/// 製品 Stage の共有面が満たす画素形式。
pub const PRESENTABLE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// 共有面として受けられるか。失敗しても Document / compositor 内部状態は変えない。
pub fn check_presentable_target(
    target: &wgpu::Texture,
    comp: CompSpec,
) -> Result<(), CompositorError> {
    if target.format() != PRESENTABLE_FORMAT {
        return Err(CompositorError::PresentableFormat {
            got: format!("{:?}", target.format()),
        });
    }
    if target.width() != comp.width || target.height() != comp.height {
        return Err(CompositorError::PresentableSize {
            got: [target.width(), target.height()],
            expected: [comp.width, comp.height],
        });
    }
    if !target
        .usage()
        .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
    {
        return Err(CompositorError::PresentableUsage);
    }
    Ok(())
}
