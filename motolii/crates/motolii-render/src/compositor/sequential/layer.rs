//! A matte: a layer cut by the α of another, both drawn through the picture's camera into the
//! composition's own window (`render_layer_to_canvas`, `matte_layer`).

use super::*;

impl Compositor {

    fn render_layer_to_canvas(
        &mut self,
        comp: CompSpec,
        projection: crate::doc::core::CameraProjection,
        view_from_world: macaw::IsoTransform,
        camera: ResolvedCamera,
        layer: &Layer,
        label: &'static str,
    ) -> Result<GpuTexture, CompositorError> {
        let (corner, u, v) = projected_placement_corners(
            comp, camera, layer.projection, layer.placement,
            glam::Vec2::ZERO, glam::Vec2::from(layer.size),
        );

        let rect = TexturedRect {
            top_left_corner_position: corner,
            extent_u: u,
            extent_v: v,
            colormapped_texture: crate::render::compositor::premultiplied_texture(
                layer
                    .content
                    .texture()
                    .ok_or_else(|| {
                        CompositorError::Draw(
                            "3D の素材は matte の対象にできない(平面に収めてから)".into(),
                        )
                    })?
                    .clone(),
            ),
            options: RectangleOptions {
                multiplicative_tint: Rgba::from_rgba_premultiplied(
                    layer.placement.opacity,
                    layer.placement.opacity,
                    layer.placement.opacity,
                    layer.placement.opacity,
                ),
                depth_offset: 0,
                ..Default::default()
            },
        };
        let draw_data = RectangleDrawData::new(&self.ctx, &[rect])
            .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

        let mut view_builder = ViewBuilder::new(
            &self.ctx,
            sequential_target_config(label, comp, Window::output(comp), view_from_world, projection, None),
            ViewBuilderId::new(self.next_readback),
        )
        .map_err(|e| CompositorError::View(e.to_string()))?;
        self.next_readback += 1;

        view_builder.queue_draw(&self.ctx, draw_data);
        let command_buffer = view_builder
            .draw(&self.ctx, Rgba::TRANSPARENT)
            .map_err(|e| CompositorError::Draw(e.to_string()))?;

        self.pending.push(command_buffer);

        Ok(view_builder.main_target().clone())
    }

    pub fn matte_layer(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layer: &Layer,
        matte_source: &Layer,
        mode: MatteMode,
    ) -> Result<Layer, CompositorError> {
        let projection = crate::doc::core::camera_projection(comp, camera);
        let view_from_world = macaw::IsoTransform::from_rotation_translation(
            projection.rotation,
            -(projection.rotation * projection.eye),
        );

        let layer_canvas = self.render_layer_to_canvas(
            comp,
            projection,
            view_from_world,
            camera,
            layer,
            "motolii-comp-matte-layer",
        )?;
        let matte_canvas = self.render_layer_to_canvas(
            comp,
            projection,
            view_from_world,
            camera,
            matte_source,
            "motolii-comp-matte-source",
        )?;

        let layer_view = layer_canvas.default_view.clone();
        let matte_view = matte_canvas.default_view.clone();
        let window = Window::output(comp);
        let out_texture = self.create_blend_scratch_texture(window.width, window.height);
        let out_view = out_texture.create_view(&Default::default());

        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("motolii-compositor-matte-pass-encoder"),
            });
        let Self {
            ctx,
            matte_vism,
            effect_scratch,
            ..
        } = self;
        matte_vism.get(ctx).record_over(
            ctx,
            &mut encoder,
            effect_scratch,
            &[&layer_view, &matte_view],
            &out_view,
            &[("mode".to_owned(), matte::matte_mode_index(mode) as f32)],
            window.size_f32(),
        );
        self.pending.push(encoder.finish());

        self.next_effect_key += 1;
        let key = self.next_effect_key;
        let imported = self
            .ctx
            .texture_manager_2d
            .import_gpu_premultiplied(key, &self.ctx, &out_texture)
            .map_err(|e| CompositorError::Effect(e.to_string()))?;

        Ok(Layer {
            content: crate::render::compositor::LayerContent::Texture(imported),
            // 描いた絵は窓の切り取り。世界へ戻すときは関心域の場所と大きさに置く。
            size: [window.roi[2], window.roi[3]],
            placement: LayerPlacement {
                transform: glam::Affine2::from_translation(glam::vec2(window.roi[0], window.roi[1])),
                world_transform: None,
                opacity: 1.0,
                order: layer.placement.order,
                z: 0.0,
                rotation_x: 0.0,
                rotation_y: 0.0,
                plane: None,
            },
            projection: crate::doc::store::LayerProjection::TwoD,
            blend_mode: layer.blend_mode,
            shading: Default::default(),
            displace: Default::default(),
            clip: None,
            shadow: layer.shadow,
            frame: None,
        })
    }
}
