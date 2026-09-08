use re_renderer::renderer::{RectangleDrawData, RectangleOptions, TexturedRect};
use re_renderer::view_builder::ViewBuilder;
use re_renderer::{GpuTexture, Rgba, ScreenshotProcessor, ViewBuilderId};

use crate::render::compositor::*;

impl Compositor {
    pub(crate) fn source_atop(
        &mut self,
        base: &GpuTexture2D,
        upper: &GpuTexture2D,
        blend: BlendMode,
    ) -> Result<GpuTexture2D, CompositorError> {
        let compose = 9u32;
        let mode = match blend {
            BlendMode::Normal => compose,
            BlendMode::Add => return Err(CompositorError::UnsupportedBlendMode(blend)),
            other => (vello_blend_mode(other).expect("mix mode") & !0xff) | compose,
        };
        let [width, height] = base.width_height();
        if upper.width_height() != [width, height] {
            return Err(CompositorError::Effect("Clipping inputs must share composition dimensions".into()));
        }
        let base_resource = self.ctx.gpu_resources.textures.get_from_handle(base.handle())
            .map_err(|error| CompositorError::Effect(error.to_string()))?;
        let upper_resource = self.ctx.gpu_resources.textures.get_from_handle(upper.handle())
            .map_err(|error| CompositorError::Effect(error.to_string()))?;
        let base_view = base_resource.texture.create_view(&Default::default());
        let upper_view = upper_resource.texture.create_view(&Default::default());
        let output = self.create_blend_scratch_texture(width, height);
        let output_view = output.create_view(&Default::default());
        let mut encoder = self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("motolii-clipping-source-atop"),
        });
        let Self { ctx, blend_vism, effect_scratch, .. } = self;
        blend_vism.record_over(
            ctx, &mut encoder, effect_scratch, &[&base_view, &upper_view], &output_view,
            &[("mode".to_owned(), mode as f32)], [width as f32, height as f32],
        );
        self.pending.push(encoder.finish());
        self.import_premultiplied(&output)
    }

    pub fn effect_passes_created_textures(&self) -> u64 {
        self.effect_scratch.created_count()
    }

    pub fn sequential_submits(&self) -> u64 {
        self.sequential_submits
    }

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
        let view_from_world = macaw::IsoTransform::from_rotation_translation(
            projection.rotation,
            -(projection.rotation * projection.eye),
        );

        // 照明は comp に 1 つ: 重ね順で一番上の環境層。空を敷くのはその層の run。
        let environment = inputs.iter().rev().find_map(|i| match i.content {
            SequentialContent::Environment(e) => Some(e),
            _ => None,
        });

        let mut background: Option<(AccumulatorBacking, GpuTexture2D)> = None;

        // 層ごとに submit しない — 同期の回数が層数に比例する。
        let mut batch: Vec<wgpu::CommandBuffer> = Vec::new();
        let mut blend_encoder: Option<wgpu::CommandEncoder> = None;
        // submit までは生かしておく必要がある(記録済みのコマンドが参照する)。
        let mut spare: Vec<AccumulatorBacking> = Vec::new();

        let mut idx = 0;
        while idx < inputs.len() {
            let input = &inputs[idx];

            // この入口で焼ける物 = 矩形かつ mix 系の blend。網・点群の mix は焼けないので run 側へ落とす
            // (落とした物を run の走査が拾わないと、idx が進まず空の run で panic か無限ループ)。
            let bakeable = |i: &crate::render::compositor::SequentialInput<'_>| {
                matches!(i.content, crate::render::compositor::SequentialContent::Rect(_))
                    && vello_blend_mode(i.blend_mode).is_some()
            };
            if let Some(mode_index) = vello_blend_mode(input.blend_mode).filter(|_| bakeable(input)) {
                let (corner, extent_u, extent_v) = crate::render::compositor::projected_placement_corners(
                    comp, input.projection_camera, input.projection,
                    input.placement,
                    input.local_min,
                    input.local_size,
                );

                let solo_rect = TexturedRect {
                    top_left_corner_position: corner,
                    extent_u,
                    extent_v,
                    colormapped_texture: crate::render::compositor::premultiplied_texture(
                        match input.content {
                            crate::render::compositor::SequentialContent::Rect(t) => t.clone(),
                            _ => unreachable!("焼く経路へ来るのは矩形だけ"),
                        },
                    ),
                    options: RectangleOptions {
                        multiplicative_tint: Rgba::from_rgba_premultiplied(
                            input.opacity,
                            input.opacity,
                            input.opacity,
                            input.opacity,
                        ),
                        depth_offset: 0,
                        clip: input.clip.map_or(re_renderer::ClipPlane::NONE, |c| c.world_for_rect(corner, extent_u, extent_v)),
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
                        environment,
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
                        let out_texture = spare.pop().unwrap_or_else(|| {
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
                        let Self {
                            ctx,
                            blend_vism,
                            effect_scratch,
                            ..
                        } = self;
                        blend_vism.record_over(
                            ctx,
                            encoder,
                            effect_scratch,
                            &[&dst_view, &src_view],
                            &out_view,
                            &[("mode".to_owned(), mode_index as f32)],
                            [comp.width as f32, comp.height as f32],
                        );

                        self.next_effect_key += 1;
                        let key = self.next_effect_key;
                        let imported = self
                            .ctx
                            .texture_manager_2d
                            .import_gpu_premultiplied(key, &self.ctx, &out_texture)
                            .map_err(|e| CompositorError::Effect(e.to_string()))?;
                        background = Some((out_texture, imported));
                        spare.push(backing);
                        spare.push(layer_canvas);
                    }
                }

                idx += 1;
                continue;
            }

            let run_start = idx;
            while idx < inputs.len() && !bakeable(&inputs[idx]) {
                // 網はその手前で run を切る: 下に描いた物を 1 枚(背後)にして渡し、ガラスが屈折して通す。
                if idx > run_start && matches!(inputs[idx].content, SequentialContent::Model(_)) {
                    break;
                }
                idx += 1;
            }
            let run = &inputs[run_start..idx];

            let mut rects: Vec<TexturedRect> = Vec::with_capacity(run.len() + 1);
            if let Some((_, imported)) = &background {
                let plane_z = crate::render::compositor::accumulator_plane_z(
                    comp,
                    camera,
                    run.iter().map(|i| {
                        let bounds = match i.content {
                            SequentialContent::Cloud { bounds, .. } => Some(bounds),
                            SequentialContent::Model(model) => Some(model.bounds),
                            SequentialContent::Rect(_) | SequentialContent::Environment(_) => None,
                        };
                        if let Some(bounds) = bounds {
                            projected_spatial_placement(comp, i.projection_camera, i.projection, i.placement, bounds)
                                .transform_point3((glam::Vec3::from(bounds.min) + glam::Vec3::from(bounds.max)) * 0.5)
                        } else {
                            let (corner, u, v) = projected_placement_corners(
                                comp, i.projection_camera, i.projection, i.placement, i.local_min, i.local_size,
                            );
                            corner + (u + v) * 0.5
                        }
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

            let mut clouds: Vec<re_renderer::renderer::PointCloudDrawData> = Vec::new();
            let mut meshes: Vec<re_renderer::renderer::MeshDrawData> = Vec::new();
            let mut sky = false;
            for input in run {
                if let SequentialContent::Environment(e) = input.content {
                    // 一番上の環境層だけが空を敷く。下に埋もれた物は板にも空にもならない。
                    sky |= environment.is_some_and(|top| std::ptr::eq(top, e));
                    continue;
                }
                if let crate::render::compositor::SequentialContent::Cloud {
                    positions,
                    colors,
                    bounds,
                    point_size,
                } = input.content
                {
                    clouds.push(self.point_cloud_draw_data(
                        positions,
                        colors,
                        bounds,
                        point_size,
                        input.placement,
                        input.opacity,
                        comp, input.projection_camera, input.projection,
                        input.displace,
                        input.clip,
                    )?);
                    continue;
                }
                if let SequentialContent::Model(model) = input.content {
                    meshes.push(
                        self.model_draw_data(
                            model,
                            input.placement,
                            input.opacity,
                            comp, input.projection_camera, input.projection,
                            &input.shading,
                            input.clip,
                        )?,
                    );
                    continue;
                }
                let (corner, extent_u, extent_v) = crate::render::compositor::projected_placement_corners(
                    comp, input.projection_camera, input.projection,
                    input.placement,
                    input.local_min,
                    input.local_size,
                );
                let a = match input.blend_mode {
                    BlendMode::Normal => input.opacity,
                    BlendMode::Add => 0.0,
                    _ => {
                        unreachable!("vello_blend_mode が None を返した blend_mode のみ run に入る")
                    }
                };
                rects.push(TexturedRect {
                    top_left_corner_position: corner,
                    extent_u,
                    extent_v,
                    colormapped_texture: crate::render::compositor::premultiplied_texture(
                        match input.content {
                            crate::render::compositor::SequentialContent::Rect(t) => t.clone(),
                            _ => unreachable!("点群は上で continue している"),
                        },
                    ),
                    options: RectangleOptions {
                        multiplicative_tint: Rgba::from_rgba_premultiplied(
                            input.opacity,
                            input.opacity,
                            input.opacity,
                            a,
                        ),
                        depth_offset: input.depth_offset,
                        clip: input.clip.map_or(re_renderer::ClipPlane::NONE, |c| c.world_for_rect(corner, extent_u, extent_v)),
                        ..Default::default()
                    },
                });
            }

            let draw_data = RectangleDrawData::new(&self.ctx, &rects)
                .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

            let has_model = run.iter().any(|i| matches!(i.content, SequentialContent::Model(_)));
            let backdrop = match (&background, has_model) {
                (Some((backing, _)), true) => {
                    // 背後の合成はまだ blend encoder の中かもしれない。先に流してから写す。
                    if let Some(encoder) = blend_encoder.take() {
                        batch.push(encoder.finish());
                    }
                    Some(self.backdrop_pyramid(comp, backing, &mut batch)?)
                }
                _ => None,
            };
            let mut config = sequential_target_config(
                "motolii-comp-sequential-run",
                comp,
                view_from_world,
                projection,
                environment,
            );
            config.backdrop = backdrop;

            let run_owned = spare
                .pop()
                .unwrap_or_else(|| self.create_blend_scratch_texture(comp.width, comp.height));
            let mut view_builder = ViewBuilder::new_with_external_resolved(
                &self.ctx,
                config,
                ViewBuilderId::new(self.next_readback),
                &run_owned,
            )
            .map_err(|e| CompositorError::View(e.to_string()))?;
            self.next_readback += 1;

            view_builder.queue_draw(&self.ctx, draw_data);
            for cloud in clouds {
                view_builder.queue_draw(&self.ctx, cloud);
            }
            for mesh in meshes {
                view_builder.queue_draw(&self.ctx, mesh);
            }
            if sky {
                view_builder.queue_draw(
                    &self.ctx,
                    re_renderer::renderer::GenericSkyboxDrawData::new(
                        &self.ctx,
                        re_renderer::renderer::GenericSkyboxType::Environment,
                    ),
                );
            }
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
        drop(spare);

        Ok(background)
    }

    /// ここまでの合成(背後)を mip 付きで写す。ガラスの網が粗さで段を読む。
    fn backdrop_pyramid(
        &mut self,
        comp: CompSpec,
        backing: &wgpu::Texture,
        batch: &mut Vec<wgpu::CommandBuffer>,
    ) -> Result<GpuTexture2D, CompositorError> {
        let size = wgpu::Extent3d { width: comp.width, height: comp.height, depth_or_array_layers: 1 };
        let texture = self.ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("motolii-backdrop-pyramid"),
            size,
            mip_level_count: re_renderer::resource_managers::MipmapGenerator::mip_level_count(comp.width, comp.height),
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: crate::render::compositor::BLEND_TARGET_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let mut encoder = self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("motolii-backdrop-pyramid"),
        });
        fn level0(texture: &wgpu::Texture) -> wgpu::TexelCopyTextureInfo<'_> {
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            }
        }
        encoder.copy_texture_to_texture(level0(backing), level0(&texture), size);
        self.ctx.texture_manager_2d.generate_mipmaps(&self.ctx, &mut encoder, &texture);
        batch.push(encoder.finish());
        self.import_premultiplied(&texture)
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
        comp: CompSpec,
        camera: ResolvedCamera,
        background: Option<(AccumulatorBacking, GpuTexture2D)>,
        background_color: [f32; 4],
    ) -> Result<Vec<u8>, CompositorError> {
        let projection = crate::doc::core::camera_projection(comp, camera);
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
                None,
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

        self.ctx.before_submit();
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
                None,
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
                let texture = self.create_blend_scratch_texture(comp.width, comp.height);
                let mut view_builder = ViewBuilder::new_with_external_resolved(
                    &self.ctx,
                    sequential_target_config(
                        "motolii-comp-zero-copy-empty",
                        comp,
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
                    crate::render::compositor::LayerContent::Cloud {
                        positions,
                        colors,
                        bounds,
                        point_size,
                    } => crate::render::compositor::SequentialContent::Cloud {
                        positions,
                        colors,
                        bounds: *bounds,
                        point_size: *point_size,
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
        camera: ResolvedCamera,
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
            sequential_target_config(label, comp, view_from_world, projection, None),
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
        let out_texture = self.create_blend_scratch_texture(comp.width, comp.height);
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
        matte_vism.record_over(
            ctx,
            &mut encoder,
            effect_scratch,
            &[&layer_view, &matte_view],
            &out_view,
            &[("mode".to_owned(), matte::matte_mode_index(mode) as f32)],
            [comp.width as f32, comp.height as f32],
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
            size: [comp.width as f32, comp.height as f32],
            placement: LayerPlacement {
                transform: glam::Affine2::IDENTITY,
                world_transform: None,
                opacity: 1.0,
                order: layer.placement.order,
                z: 0.0,
                rotation_x: 0.0,
                rotation_y: 0.0,
            },
            projection: crate::doc::store::LayerProjection::TwoD,
            projection_camera: camera,
            blend_mode: layer.blend_mode,
            shading: Default::default(),
            displace: Default::default(),
            clip: None,
        })
    }
}
