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

    pub fn render_context(&self) -> &re_renderer::RenderContext {
        &self.ctx
    }

    /// 共有面へ直接書く口(裁定256)。検査失敗では内部状態を変えない。
    ///
    /// **effect pass を適用してから描く**(2026-08-28 修理)——[`Self::render_with_effects`]/
    /// [`Self::render_to_texture`]と同じく `layers[i].passes` を
    /// `Self::effective_layer_textures` へ通し、その実効 texture(と、pass が出力を
    /// 拡張した分の padding)を rect へ使う。**この関数だけが `lwp.layer.texture` を
    /// そのまま描いていた**ため、共有面(Makepad の zero-copy Stage)は effect を
    /// 一切反映していなかった——export/CPU 読み戻し経路(上記2つ)は元から正しかった。
    pub fn render_into(
        &mut self,
        target: &wgpu::Texture,
        comp: CompSpec,
        camera: ResolvedCamera,
        layers: &[LayerWithPasses],
    ) -> Result<(), CompositorError> {
        check_presentable_target(target, comp)?;

        // layer ごとに「合成へ渡す実効 texture」を決める(`render_effects.rs` の
        // `Self::effective_layer_textures` が3経路で共有する核)。`checked_out` は
        // この関数の合成が終わってから(下の poll の後)プールへ返す——
        // `render_with_effects`/`render_to_texture` と同じ返却タイミング。
        let (effective_textures, effective_paddings, checked_out) =
            self.effective_layer_textures(layers)?;

        let projection = motolii_core::camera_projection(comp, camera);
        let pinned_cancel = motolii_core::camera_screen_from_world_z0(comp, camera).inverse();
        let view_from_world = macaw::IsoTransform::from_rotation_translation(
            projection.rotation,
            -(projection.rotation * projection.eye),
        );

        self.ctx.begin_frame();

        let mut rects: Vec<TexturedRect> = Vec::with_capacity(layers.len());
        for ((lwp, texture), &padding) in layers.iter().zip(&effective_textures).zip(&effective_paddings) {
            let layer = &lwp.layer;
            let (transform, z, rx, ry) = if layer.pinned {
                (pinned_cancel * layer.placement.transform, 0.0, 0.0, 0.0)
            } else {
                (
                    layer.placement.transform,
                    layer.placement.z,
                    layer.placement.rotation_x,
                    layer.placement.rotation_y,
                )
            };
            let a = match layer.blend_mode {
                crate::BlendMode::Add => 0.0,
                _ => layer.placement.opacity,
            };
            // pass が出力を拡張した分(texel、`EffectPass::padding`)だけ quad を
            // local 空間で広げる——`render_with_effects`/`render_to_texture` が
            // `SequentialInput::local_min`/`local_size` でやっているのと同じ計算
            // (padding=0 なら従来と完全に同じ幾何)。
            let pad = padding as f32;
            let (corner, extent_u, extent_v) = crate::tilted_corners(
                transform,
                glam::Vec2::new(-pad, -pad),
                glam::Vec2::new(layer.size[0] + 2.0 * pad, layer.size[1] + 2.0 * pad),
                z,
                rx,
                ry,
            );
            rects.push(TexturedRect {
                top_left_corner_position: corner,
                extent_u,
                extent_v,
                colormapped_texture: ColormappedTexture::from_unorm_rgba(texture.clone()),
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

        // scratch をプールへ返す——この合成(上の poll)が終わった後なので、
        // `render_with_effects`/`render_to_texture` と同じ返却タイミング
        // (`effective_layer_textures` の doc 参照)。
        for (width, height, format, scratch_texture) in checked_out {
            self.effect_scratch
                .release(width, height, format, scratch_texture);
        }
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
