use re_renderer::renderer::{
    RectangleDrawData, RectangleOptions,
    TexturedRect,
};
use re_renderer::view_builder::ViewBuilder;
use re_renderer::{GpuTexture, Rgba, ScreenshotProcessor, ViewBuilderId};

use crate::render::compositor::*;

impl Compositor {
    pub fn effect_passes_created_textures(&self) -> u64 {
        self.effect_scratch.created_count()
    }

    pub fn sequential_submits(&self) -> u64 {
        self.sequential_submits
    }

    /// 貯めたパスを**一度に** submit する。層ごとに GPU を止めない。
    pub(crate) fn flush_pending(&mut self) {
        if self.pending.is_empty() {
            return;
        }
        self.ctx.before_submit();
        let batch: Vec<wgpu::CommandBuffer> = self.pending.drain(..).collect();
        self.ctx.queue.submit(batch);
        self.sequential_submits += 1;
        // staging buffer の回収は submit の**後**に一度だけ。
        self.ctx.begin_frame();
    }

    pub(crate) fn accumulate_sequential(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        inputs: &[SequentialInput<'_>],
        background_color: [f32; 4],
    ) -> Result<Option<(AccumulatorBacking, GpuTexture2D)>, CompositorError> {
        debug_assert!(
            inputs
                .windows(2)
                .all(|w| w[0].depth_offset <= w[1].depth_offset),
            "accumulate_sequential: inputs は depth_offset 非減少(=重ね順)で渡すこと"
        );
        let projection = crate::doc::core::camera_projection(comp, camera);
        let pinned_cancel = crate::doc::core::camera_screen_from_world_z0(comp, camera).inverse();
        let view_from_world = macaw::IsoTransform::from_rotation_translation(
            projection.rotation,
            -(projection.rotation * projection.eye),
        );

        let mut background: Option<(AccumulatorBacking, GpuTexture2D)> = None;

        // 合成の全パスを1つの束へ記録し、**最後に一度だけ** submit する。
        // 層ごとの submit + poll(wait_indefinitely) は、層が増えるほど GPU を直列に
        // 止めていた(55層すべて blend mode なら同期110回)。GPU は1回の submit の中で
        // 記録順に実行し、テクスチャの読み書きの間には自動で barrier が入る。
        let mut batch: Vec<wgpu::CommandBuffer> = Vec::new();
        let mut blend_encoder: Option<wgpu::CommandEncoder> = None;
        // 役目を終えた全面テクスチャは捨てずにここへ戻し、次のパスの出力に使い回す
        // (ping-pong)。**submit までは生かしておく必要がある**ので、置き場所も兼ねる。
        let mut spare: Vec<AccumulatorBacking> = Vec::new();

        let mut idx = 0;
        while idx < inputs.len() {
            let input = &inputs[idx];

            if let Some(mode_index) = two_texture_pass_mode_index(input.blend_mode) {
                let (transform, z, rx, ry) = if input.pinned {
                    (pinned_cancel * input.transform, 0.0, 0.0, 0.0)
                } else {
                    (input.transform, input.z, input.rotation_x, input.rotation_y)
                };
                let (corner, extent_u, extent_v) = crate::render::compositor::tilted_corners(
                    transform,
                    input.local_min,
                    input.local_size,
                    z,
                    rx,
                    ry,
                );

                let solo_rect = TexturedRect {
                    top_left_corner_position: corner,
                    extent_u,
                    extent_v,
                    colormapped_texture: crate::render::compositor::premultiplied_texture(input.texture.clone()),
                    options: RectangleOptions {
                        multiplicative_tint: Rgba::from_rgba_premultiplied(
                            input.opacity,
                            input.opacity,
                            input.opacity,
                            input.opacity,
                        ),
                        depth_offset: 0,
                        ..Default::default()
                    },
                };
                let draw_data = RectangleDrawData::new(&self.ctx, &[solo_rect])
                    .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

                let solo_owned = spare
                    .pop()
                    .unwrap_or_else(|| self.create_blend_scratch_texture(comp.width, comp.height));
                let mut solo_view_builder = ViewBuilder::new_with_external_resolved(
                    &self.ctx,
                    sequential_target_config(
                        "motolii-comp-sequential-solo",
                        comp,
                        view_from_world,
                        projection,
                    ),
                    ViewBuilderId::new(self.next_readback),
                    &solo_owned,
                )
                .map_err(|e| CompositorError::View(e.to_string()))?;
                self.next_readback += 1;

                solo_view_builder.queue_draw(&self.ctx, draw_data);
                let clear = if background.is_none() {
                    crate::render::compositor::clear_color(background_color)
                } else {
                    Rgba::TRANSPARENT
                };
                let command_buffer = solo_view_builder
                    .draw(&self.ctx, clear)
                    .map_err(|e| CompositorError::Draw(e.to_string()))?;

                if let Some(encoder) = blend_encoder.take() {
                    batch.push(encoder.finish());
                }
                batch.push(command_buffer);

                let layer_canvas = solo_owned;

                match background.take() {
                    None => {
                        self.next_effect_key += 1;
                        let key = self.next_effect_key;
                        let imported = self
                            .ctx
                            .texture_manager_2d
                            .import_gpu_premultiplied(key, &self.ctx, &layer_canvas)
                            .map_err(|e| CompositorError::Effect(e.to_string()))?;
                        background = Some((layer_canvas, imported));
                    }
                    Some((backing, _)) => {
                        let dst_view = backing.create_view(&Default::default());
                        let src_view = layer_canvas.create_view(&Default::default());
                        let out_texture = spare
                            .pop()
                            .unwrap_or_else(|| {
                                self.create_blend_scratch_texture(comp.width, comp.height)
                            });
                        let out_view = out_texture.create_view(&Default::default());

                        let encoder = blend_encoder.get_or_insert_with(|| {
                            self.ctx.device.create_command_encoder(
                                &wgpu::CommandEncoderDescriptor {
                                    label: Some("motolii-compositor-blend-pass-encoder"),
                                },
                            )
                        });
                        self.blend_pipelines.record(
                            &self.ctx.device,
                            &self.ctx.queue,
                            encoder,
                            &dst_view,
                            &src_view,
                            &out_view,
                            mode_index,
                        );

                        self.next_effect_key += 1;
                        let key = self.next_effect_key;
                        let imported = self
                            .ctx
                            .texture_manager_2d
                            .import_gpu_premultiplied(key, &self.ctx, &out_texture)
                            .map_err(|e| CompositorError::Effect(e.to_string()))?;
                        background = Some((out_texture, imported));
                        // この2枚はもう誰も読まない。次のパスの出力へ回す
                        spare.push(backing);
                        spare.push(layer_canvas);
                    }
                }

                idx += 1;
                continue;
            }

            let run_start = idx;
            while idx < inputs.len() && two_texture_pass_mode_index(inputs[idx].blend_mode).is_none() {
                idx += 1;
            }
            let run = &inputs[run_start..idx];

            let mut rects: Vec<TexturedRect> = Vec::with_capacity(run.len() + 1);
            if let Some((_, imported)) = &background {
                let plane_z = crate::render::compositor::accumulator_plane_z(
                    comp,
                    camera,
                    run.iter().map(|i| {
                        let (transform, z) = if i.pinned {
                            (pinned_cancel * i.transform, 0.0)
                        } else {
                            (i.transform, i.z)
                        };
                        let c = transform.transform_point2(i.local_min + i.local_size * 0.5);
                        glam::vec3(c.x, c.y, z)
                    }),
                );
                rects.push(background_rect(
                    comp,
                    camera,
                    imported.clone(),
                    run[0].depth_offset.saturating_sub(1),
                    plane_z,
                ));
            }

            for input in run {
                let (transform, z, rx, ry) = if input.pinned {
                    (pinned_cancel * input.transform, 0.0, 0.0, 0.0)
                } else {
                    (input.transform, input.z, input.rotation_x, input.rotation_y)
                };
                let (corner, extent_u, extent_v) = crate::render::compositor::tilted_corners(
                    transform,
                    input.local_min,
                    input.local_size,
                    z,
                    rx,
                    ry,
                );
                let a = match input.blend_mode {
                    BlendMode::Normal => input.opacity,
                    BlendMode::Add => 0.0,
                    _ => unreachable!(
                        "two_texture_pass_mode_index が None を返した blend_mode のみ run に入る"
                    ),
                };
                rects.push(TexturedRect {
                    top_left_corner_position: corner,
                    extent_u,
                    extent_v,
                    colormapped_texture: crate::render::compositor::premultiplied_texture(input.texture.clone()),
                    options: RectangleOptions {
                        multiplicative_tint: Rgba::from_rgba_premultiplied(
                            input.opacity,
                            input.opacity,
                            input.opacity,
                            a,
                        ),
                        depth_offset: input.depth_offset,
                        ..Default::default()
                    },
                });
            }

            let draw_data = RectangleDrawData::new(&self.ctx, &rects)
                .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

            let run_owned = spare
                .pop()
                .unwrap_or_else(|| self.create_blend_scratch_texture(comp.width, comp.height));
            let mut view_builder = ViewBuilder::new_with_external_resolved(
                &self.ctx,
                sequential_target_config(
                    "motolii-comp-sequential-run",
                    comp,
                    view_from_world,
                    projection,
                ),
                ViewBuilderId::new(self.next_readback),
                &run_owned,
            )
            .map_err(|e| CompositorError::View(e.to_string()))?;
            self.next_readback += 1;

            view_builder.queue_draw(&self.ctx, draw_data);
            let clear = if background.is_none() {
                crate::render::compositor::clear_color(background_color)
            } else {
                Rgba::TRANSPARENT
            };
            let command_buffer = view_builder
                .draw(&self.ctx, clear)
                .map_err(|e| CompositorError::Draw(e.to_string()))?;

            if let Some(encoder) = blend_encoder.take() {
                batch.push(encoder.finish());
            }
            batch.push(command_buffer);

            self.next_effect_key += 1;
            let key = self.next_effect_key;
            let imported = self
                .ctx
                .texture_manager_2d
                .import_gpu_premultiplied(key, &self.ctx, &run_owned)
                .map_err(|e| CompositorError::Effect(e.to_string()))?;
            if let Some((old, _)) = background.replace((run_owned, imported)) {
                spare.push(old);
            }
        }

        if let Some(encoder) = blend_encoder.take() {
            batch.push(encoder.finish());
        }
        self.pending.append(&mut batch);
        // spare はここで落ちるが、記録済みのコマンドが参照している間は wgpu が実体を保つ。
        drop(spare);

        Ok(background)
    }

    pub(crate) fn create_blend_scratch_texture(&self, width: u32, height: u32) -> wgpu::Texture {
        self.ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("motolii-compositor-blend-output"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: blend::SEPARABLE_BLEND_TARGET_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        })
    }

    pub(crate) fn finalize_readback(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        background: Option<(AccumulatorBacking, GpuTexture2D)>,
        background_color: [f32; 4],
    ) -> Result<Vec<u8>, CompositorError> {
        let projection = crate::doc::core::camera_projection(comp, camera);
        let _pinned_cancel = crate::doc::core::camera_screen_from_world_z0(comp, camera).inverse();
        let view_from_world = macaw::IsoTransform::from_rotation_translation(
            projection.rotation,
            -(projection.rotation * projection.eye),
        );

        let mut final_rects: Vec<TexturedRect> = Vec::with_capacity(1);
        if let Some((_, imported)) = &background {
            final_rects.push(background_rect(comp, camera, imported.clone(), -1, 0.0));
        }

        let final_draw_data = RectangleDrawData::new(&self.ctx, &final_rects)
            .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

        let mut final_view_builder = ViewBuilder::new(
            &self.ctx,
            sequential_target_config(
                "motolii-comp-sequential-finalize",
                comp,
                view_from_world,
                projection,
            ),
            ViewBuilderId::new(self.next_readback),
        )
        .map_err(|e| CompositorError::View(e.to_string()))?;
        self.next_readback += 1;

        final_view_builder.queue_draw(&self.ctx, final_draw_data);

        let identifier = self.next_readback;
        self.next_readback += 1;
        final_view_builder
            .schedule_screenshot(&self.ctx, identifier, ())
            .map_err(|e| CompositorError::View(e.to_string()))?;

        let clear = if background.is_some() {
            Rgba::TRANSPARENT
        } else {
            crate::render::compositor::clear_color(background_color)
        };
        let command_buffer = final_view_builder
            .draw(&self.ctx, clear)
            .map_err(|e| CompositorError::Draw(e.to_string()))?;

        self.pending.push(command_buffer);
        self.flush_pending();
        self.ctx
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| CompositorError::Draw(e.to_string()))?;

        self.ctx.begin_frame();

        let mut out: Option<Vec<u8>> = None;
        ScreenshotProcessor::next_readback_result::<()>(
            &self.ctx,
            identifier,
            |data, _extent, ()| {
                out = Some(data.to_vec());
            },
        );

        out.ok_or(CompositorError::ReadbackMissing)
    }

    pub(crate) fn finalize_into(
        &mut self,
        target: &wgpu::Texture,
        comp: CompSpec,
        camera: ResolvedCamera,
        background: Option<(AccumulatorBacking, GpuTexture2D)>,
        background_color: [f32; 4],
    ) -> Result<(), CompositorError> {
        let projection = crate::doc::core::camera_projection(comp, camera);
        let _pinned_cancel = crate::doc::core::camera_screen_from_world_z0(comp, camera).inverse();
        let view_from_world = macaw::IsoTransform::from_rotation_translation(
            projection.rotation,
            -(projection.rotation * projection.eye),
        );

        let mut final_rects: Vec<TexturedRect> = Vec::with_capacity(1);
        if let Some((_, imported)) = &background {
            final_rects.push(background_rect(comp, camera, imported.clone(), -1, 0.0));
        }
        let draw_data = RectangleDrawData::new(&self.ctx, &final_rects)
            .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

        let mut view_builder = {
            let mut vb = ViewBuilder::new(
                &self.ctx,
                sequential_target_config(
                    "motolii-comp-finalize-into",
                    comp,
                    view_from_world,
                    projection,
                ),
                ViewBuilderId::new(self.next_readback),
            )
            .map_err(|e| CompositorError::View(e.to_string()))?;
            self.next_readback += 1;
            vb.queue_draw(&self.ctx, draw_data);
            vb
        };

        let clear = if background.is_some() {
            Rgba::TRANSPARENT
        } else {
            crate::render::compositor::clear_color(background_color)
        };
        let command_buffer = view_builder
            .draw(&self.ctx, clear)
            .map_err(|e| CompositorError::Draw(e.to_string()))?;
        self.pending.push(command_buffer);

        let target_view = target.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.ctx.output_format_color()),
            ..Default::default()
        });
        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("motolii-comp-composite-into"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("motolii-comp-composite-into-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            view_builder.composite(&self.ctx, &mut pass);
        }
        self.pending.push(encoder.finish());
        self.flush_pending();
        self.ctx
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| CompositorError::Draw(e.to_string()))?;
        Ok(())
    }

    pub(crate) fn finalize_texture(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        background: Option<(AccumulatorBacking, GpuTexture2D)>,
        background_color: [f32; 4],
    ) -> Result<(wgpu::Texture, wgpu::TextureView), CompositorError> {
        // 呼び手はこのテクスチャを直に使う。貯めたパスをここで出しておかないと
        // 中身がまだ書かれていない。
        self.flush_pending();
        match background {
            Some((backing, _imported)) => {
                let texture = backing.clone();
                let view = texture.create_view(&Default::default());
                Ok((texture, view))
            }
            None => {
                let projection = crate::doc::core::camera_projection(comp, camera);
                let view_from_world = macaw::IsoTransform::from_rotation_translation(
                    projection.rotation,
                    -(projection.rotation * projection.eye),
                );
                let draw_data = RectangleDrawData::new(&self.ctx, &[])
                    .map_err(|e| CompositorError::Rectangles(e.to_string()))?;
                let texture = self.create_blend_scratch_texture(comp.width, comp.height);
                let mut view_builder = ViewBuilder::new_with_external_resolved(
                    &self.ctx,
                    sequential_target_config(
                        "motolii-comp-zero-copy-empty",
                        comp,
                        view_from_world,
                        projection,
                    ),
                    ViewBuilderId::new(self.next_readback),
                    &texture,
                )
                .map_err(|e| CompositorError::View(e.to_string()))?;
                self.next_readback += 1;
                view_builder.queue_draw(&self.ctx, draw_data);
                let command_buffer = view_builder
                    .draw(&self.ctx, crate::render::compositor::clear_color(background_color))
                    .map_err(|e| CompositorError::Draw(e.to_string()))?;
                self.pending.push(command_buffer);
                self.flush_pending();

                let view = texture.create_view(&Default::default());
                Ok((texture, view))
            }
        }
    }

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
                texture: &layer.texture,
                local_min: glam::Vec2::ZERO,
                local_size: glam::Vec2::new(layer.size[0], layer.size[1]),
                transform: layer.placement.transform,
                z: layer.placement.z,
                rotation_x: layer.placement.rotation_x,
                rotation_y: layer.placement.rotation_y,
                pinned: layer.pinned,
                opacity: layer.placement.opacity,
                depth_offset: layer.placement.order,
                blend_mode: layer.blend_mode,
            })
            .collect();

        let background = self.accumulate_sequential(comp, camera, &inputs, background_color)?;
        self.finalize_readback(comp, camera, background, background_color)
    }

    fn render_layer_to_canvas(
        &mut self,
        comp: CompSpec,
        projection: crate::doc::core::CameraProjection,
        view_from_world: macaw::IsoTransform,
        pinned_cancel: glam::Affine2,
        layer: &Layer,
        label: &'static str,
    ) -> Result<GpuTexture, CompositorError> {
        let (transform, z) = if layer.pinned {
            (pinned_cancel * layer.placement.transform, 0.0)
        } else {
            (layer.placement.transform, layer.placement.z)
        };

        let tilt = crate::render::compositor::tilt(layer.placement.rotation_x, layer.placement.rotation_y);
        let u = tilt * to_vector3(transform.transform_vector2(glam::Vec2::new(layer.size[0], 0.0)));
        let v = tilt * to_vector3(transform.transform_vector2(glam::Vec2::new(0.0, layer.size[1])));
        let center = to_point3(
            transform.transform_point2(glam::Vec2::new(layer.size[0], layer.size[1]) * 0.5),
            z,
        );

        let rect = TexturedRect {
            top_left_corner_position: center - (u + v) * 0.5,
            extent_u: u,
            extent_v: v,
            colormapped_texture: crate::render::compositor::premultiplied_texture(layer.texture.clone()),
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
            sequential_target_config(label, comp, view_from_world, projection),
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
        let pinned_cancel = crate::doc::core::camera_screen_from_world_z0(comp, camera).inverse();
        let view_from_world = macaw::IsoTransform::from_rotation_translation(
            projection.rotation,
            -(projection.rotation * projection.eye),
        );

        let layer_canvas = self.render_layer_to_canvas(
            comp,
            projection,
            view_from_world,
            pinned_cancel,
            layer,
            "motolii-comp-matte-layer",
        )?;
        let matte_canvas = self.render_layer_to_canvas(
            comp,
            projection,
            view_from_world,
            pinned_cancel,
            matte_source,
            "motolii-comp-matte-source",
        )?;

        let layer_view = layer_canvas.default_view.clone();
        let matte_view = matte_canvas.default_view.clone();
        let out_texture = self.create_blend_scratch_texture(comp.width, comp.height);
        let out_view = out_texture.create_view(&Default::default());

        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("motolii-compositor-matte-pass-encoder"),
            });
        self.matte_pipelines.record(
            &self.ctx.device,
            &self.ctx.queue,
            &mut encoder,
            &layer_view,
            &matte_view,
            &out_view,
            matte::matte_mode_index(mode),
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
            texture: imported,
            size: [comp.width as f32, comp.height as f32],
            placement: LayerPlacement {
                transform: glam::Affine2::IDENTITY,
                opacity: 1.0,
                order: layer.placement.order,
                z: 0.0,
                rotation_x: 0.0,
                rotation_y: 0.0,
            },
            pinned: true,
            blend_mode: layer.blend_mode,
        })
    }
}
