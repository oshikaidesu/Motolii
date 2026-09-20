//! 仕上げて外へ出す: 積み終えた 1 枚を、読み戻し・別の texture・渡された窓のどれかへ渡す。
//! 選ばれた層の outline mask もここで作る(縁は描かず、mask を `selection_bounds` が畳む)。

use super::*;

impl Compositor {
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
            format: crate::render::compositor::BLEND_TARGET_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        })
    }

    pub(crate) fn finalize_readback(
        &mut self,
        _comp: CompSpec,
        camera: ResolvedCamera,
        background: Option<(AccumulatorBacking, GpuTexture2D)>,
        background_color: [f32; 4],
    ) -> Result<Vec<u8>, CompositorError> {
        let _ = camera;
        let mut final_rects: Vec<TexturedRect> = Vec::with_capacity(1);
        if let Some((_, imported)) = &background {
            final_rects.push(screen_rect(self.window, imported.clone()));
        }

        let final_draw_data = RectangleDrawData::new(&self.ctx, &final_rects)
            .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

        let mut final_view_builder = ViewBuilder::new(
            &self.ctx,
            screen_target_config("motolii-comp-sequential-finalize", self.window),
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
        let gpu_measurement = self.measure_pending_gpu();
        // submit の時計は `flush_pending` が持つ(1 コマに何度も出るため)。
        self.flush_pending();
        let wait_start = std::time::Instant::now();
        self.ctx
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| CompositorError::Draw(e.to_string()))?;

        self.measurement.wait_us = wait_start.elapsed().as_micros() as u64;
        let readback_start = std::time::Instant::now();
        let measured = gpu_measurement.ok_or("disabled_or_unsupported")
            .and_then(|m| m.finish(self.ctx.queue.get_timestamp_period()));
        self.measurement.gpu_status = measured.as_ref().map_or_else(|e| *e, |_| "valid");
        self.measurement.final_submission_gpu_us = measured.ok();
        let mut out: Option<Vec<u8>> = None;
        ScreenshotProcessor::next_readback_result::<()>(
            &self.ctx,
            identifier,
            |data, _extent, ()| {
                out = Some(data.to_vec());
            },
        );

        self.measurement.readback_us = readback_start.elapsed().as_micros() as u64;
        out.ok_or(CompositorError::ReadbackMissing)
    }

    /// 選ばれた層だけをもう 1 度、同じカメラで透明の上に描き、re_renderer の outline の object-id mask を作る
    /// (`draw_phases/outlines.rs`)。縁は描かない — mask を `selection_bounds` が畳んで籠にする。誰も選ばれていなければ無し。
    pub(crate) fn outline_view(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        inputs: &[SequentialInput<'_>],
    ) -> Result<Option<(ViewBuilder, AccumulatorBacking)>, CompositorError> {
        if !inputs.iter().any(|input| input.outline != 0) {
            return Ok(None);
        }
        let projection = crate::doc::core::camera_projection(comp, camera);
        let view_from_world = macaw::IsoTransform::from_rotation_translation(
            projection.rotation,
            -(projection.rotation * projection.eye),
        );
        let mut config = sequential_target_config("motolii-comp-outline", comp, self.window, view_from_world, projection, None);
        config.outline_config = Some(re_renderer::OutlineConfig {
            outline_radius_pixel: 1.0,
            color_layer_a: Rgba::TRANSPARENT,
            color_layer_b: Rgba::TRANSPARENT,
        });
        let owned = self.create_blend_scratch_texture(self.window.width, self.window.height);
        let mut view_builder = ViewBuilder::new_with_external_resolved(
            &self.ctx,
            config,
            ViewBuilderId::new(self.next_readback),
            &owned,
        )
        .map_err(|e| CompositorError::View(e.to_string()))?;
        self.next_readback += 1;
        let draws = self.surface_scene_draws(comp, inputs, Vec::new(), false, &|index| inputs[index].outline == 0, None, 0)?;
        draws.queue(&self.ctx, &mut view_builder);
        let command_buffer = view_builder
            .draw(&self.ctx, Rgba::TRANSPARENT)
            .map_err(|e| CompositorError::Draw(e.to_string()))?;
        self.pending.push(command_buffer);
        if let (Some(mask), Some(bounds)) = (view_builder.outline_mask_texture(), self.selection_bounds.as_mut()) {
            let mut encoder = self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("motolii-selection-bounds") });
            bounds.record(&self.ctx.device, &mut encoder, &mask.default_view, self.window.size());
            self.pending.push(encoder.finish());
        }
        Ok(Some((view_builder, owned)))
    }

    pub(crate) fn finalize_into(
        &mut self,
        target: &wgpu::Texture,
        _comp: CompSpec,
        camera: ResolvedCamera,
        background: Option<(AccumulatorBacking, GpuTexture2D)>,
        background_color: [f32; 4],
        outline: Option<(ViewBuilder, AccumulatorBacking)>,
    ) -> Result<(), CompositorError> {
        let _ = camera;
        let mut final_rects: Vec<TexturedRect> = Vec::with_capacity(1);
        if let Some((_, imported)) = &background {
            final_rects.push(screen_rect(self.window, imported.clone()));
        }
        let draw_data = RectangleDrawData::new(&self.ctx, &final_rects)
            .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

        let mut view_builder = {
            let mut vb = ViewBuilder::new(
                &self.ctx,
                screen_target_config("motolii-comp-finalize-into", self.window),
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
        if let (true, Some(bounds)) = (outline.is_some(), self.selection_bounds.as_mut()) {
            bounds.schedule_map();
        }
        drop(outline);
        // ここで poll しない — 共有 device を毎フレーム止めると blitz/vello が壊れる。
        Ok(())
    }

    pub(crate) fn finalize_texture(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        background: Option<(AccumulatorBacking, GpuTexture2D)>,
        background_color: [f32; 4],
    ) -> Result<(wgpu::Texture, wgpu::TextureView), CompositorError> {
        // 呼び手が中身を使うので、ここで出す。
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
                let texture = self.create_blend_scratch_texture(self.window.width, self.window.height);
                let mut view_builder = ViewBuilder::new_with_external_resolved(
                    &self.ctx,
                    sequential_target_config("motolii-comp-zero-copy-empty", comp, self.window,
                        view_from_world,
                        projection,
                None,
            ),
                    ViewBuilderId::new(self.next_readback),
                    &texture,
                )
                .map_err(|e| CompositorError::View(e.to_string()))?;
                self.next_readback += 1;
                view_builder.queue_draw(&self.ctx, draw_data);
                let command_buffer = view_builder
                    .draw(
                        &self.ctx,
                        crate::render::compositor::clear_color(background_color),
                    )
                    .map_err(|e| CompositorError::Draw(e.to_string()))?;
                self.pending.push(command_buffer);
                self.flush_pending();

                let view = texture.create_view(&Default::default());
                Ok((texture, view))
            }
        }
    }
}
