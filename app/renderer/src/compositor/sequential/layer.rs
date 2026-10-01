//! 1 層を描く: 層の列を run の入口へ渡す `render_sequential`、1 層を窓へ描く `render_layer_to_canvas`、
//! 相手の層の α で切る `matte_layer`。

use super::*;

impl Compositor {
    pub fn render_sequential(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layers: &[Layer],
        background_color: [f32; 4],
    ) -> Result<Vec<u8>, CompositorError> {
        let inputs: Vec<SequentialInput<'_>> = layers
            .iter()
            .map(|layer| SequentialInput {
                content: match &layer.content {
                    crate::render::compositor::LayerContent::Texture(t) => {
                        crate::render::compositor::SequentialContent::Rect(t)
                    }
                    crate::render::compositor::LayerContent::LinearTexture(t) => crate::render::compositor::SequentialContent::LinearRect(t),
                    crate::render::compositor::LayerContent::Cloud {
                        positions,
                        colors,
                        bounds,
                        point_size,
                        sizes,
                        sprites,
                        links,
                    } => crate::render::compositor::SequentialContent::Cloud {
                        positions,
                        colors,
                        bounds: *bounds,
                        point_size: *point_size,
                        sizes: sizes.as_ref().map(|s| s.as_slice()),
                        sprites: *sprites,
                        links: links.as_deref(),
                    },
                    crate::render::compositor::LayerContent::Model(model) => {
                        crate::render::compositor::SequentialContent::Model(model)
                    }
                    crate::render::compositor::LayerContent::Environment(e) => {
                        crate::render::compositor::SequentialContent::Environment(e)
                    }
                },
                local_min: glam::Vec2::ZERO,
                local_size: glam::Vec2::new(layer.size[0], layer.size[1]),
                placement: layer.placement,
                projection: layer.projection,
                projection_camera: layer.projection_camera,
                opacity: layer.placement.opacity,
                depth_offset: layer.placement.order,
                blend_mode: layer.blend_mode,
                shading: layer.shading.clone(),
                displace: layer.displace,
                clip: layer.clip,
                shadow: layer.shadow,
                outline: layer.outline,
                screen_passes: &[],
                screen_sources: &[],
            })
            .collect();

        self.window = crate::render::compositor::Window::output(comp);
        let background = self.accumulate_sequential(comp, camera, &inputs, background_color)?;
        self.finalize_readback(comp, camera, background, background_color)
    }

    fn render_layer_to_canvas(
        &mut self,
        comp: CompSpec,
        projection: crate::doc::core::CameraProjection,
        view_from_world: macaw::IsoTransform,
        _camera: ResolvedCamera,
        layer: &Layer,
        label: &'static str,
    ) -> Result<GpuTexture, CompositorError> {
        let (corner, u, v) = projected_placement_corners(
            comp, layer.projection_camera, layer.projection, layer.placement,
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
            sequential_target_config(label, comp, self.window, view_from_world, projection, None),
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
        // 窓はコマの組み立ての後で決まる。まだ 1 度も描いていない engine では寸法が 0 なので、出力の窓で描く。
        if self.window.width == 0 || self.window.height == 0 {
            self.window = crate::render::compositor::Window::output(comp);
        }
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
        let out_texture = self.create_blend_scratch_texture(self.window.width, self.window.height);
        let out_view = out_texture.create_view(&Default::default());
        let window = self.window;

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
            projection_camera: camera,
            blend_mode: layer.blend_mode,
            shading: Default::default(),
            displace: Default::default(),
            clip: None,
            shadow: layer.shadow,
            outline: layer.outline,
            frame: None,
        })
    }
}
