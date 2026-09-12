use re_renderer::renderer::{RectangleDrawData, RectangleOptions, TexturedRect};
use re_renderer::view_builder::ViewBuilder;
use re_renderer::{GpuTexture, Rgba, ScreenshotProcessor, ViewBuilderId};

use crate::render::compositor::*;

pub(crate) struct BackdropResource {
    dimensions: [u32; 2],
    texture: wgpu::Texture,
    imported: GpuTexture2D,
}

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

        let mut shared_meshes = None;
        let reflection = self.cached_scene_reflection(comp, inputs, environment, &mut shared_meshes)?;
        // 光は環境から来る。遮る層があれば太陽から見た型紙を 1 枚描き、全ての run がそれを読む。
        let light = self.capture_light_cookie(comp, inputs, environment, shared_meshes.as_ref())?;
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
                matches!(i.content, crate::render::compositor::SequentialContent::Rect(_) | crate::render::compositor::SequentialContent::LinearRect(_))
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
                    colormapped_texture: input.content.image().expect("焼く経路へ来るのは矩形だけ"),
                    options: RectangleOptions {
                        multiplicative_tint: Rgba::from_rgba_premultiplied(
                            input.opacity,
                            input.opacity,
                            input.opacity,
                            input.opacity,
                        ),
                        depth_offset: 0,
                        clip: input.clip.map_or(re_renderer::ClipPlane::NONE, |c| c.world_for_rect(corner, extent_u, extent_v)),
                        surface: input.shading.program.clone(),
                        surface_params: input.shading.params,
                        ..Default::default()
                    },
                };
                let draw_data = RectangleDrawData::new(&self.ctx, &[solo_rect])
                    .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

                let solo_owned = spare
                    .pop()
                    .unwrap_or_else(|| self.create_blend_scratch_texture(comp.width, comp.height));
                let mut solo_config = sequential_target_config(
                    "motolii-comp-sequential-solo", comp, view_from_world, projection, environment,
                );
                solo_config.scene_reflection = reflection.clone();
                solo_config.light = light.clone();
                self.surface_work.main_runs += 1;
                if input.shading.reads_backdrop {
                    if let Some((backing, _)) = &background {
                        if let Some(encoder) = blend_encoder.take() {
                            batch.push(encoder.finish());
                        }
                        solo_config.backdrop = Some(self.backdrop_pyramid(comp, backing, &mut batch, input.shading.backdrop_roughness)?);
                    }
                }
                let mut solo_view_builder = ViewBuilder::new_with_external_resolved(
                    &self.ctx,
                    solo_config,
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

                background = Some(self.stack_over(comp, background.take(), solo_owned, mode_index, &mut spare, &mut blend_encoder)?);

                idx += 1;
                continue;
            }

            let run_start = idx;
            let mut run_has_rect = false;
            while idx < inputs.len() && !bakeable(&inputs[idx]) {
                // 表面プログラムは手前で run を切り、それまでの合成を背後として読む。
                if idx > run_start && (inputs[idx].shading.reads_backdrop || (run_has_rect && matches!(inputs[idx].content, SequentialContent::Model(_)))) {
                    break;
                }
                run_has_rect |= matches!(inputs[idx].content, SequentialContent::Rect(_) | SequentialContent::LinearRect(_));
                idx += 1;
            }
            let run = &inputs[run_start..idx];

            // 地は持ち込まない。run は自分の層だけを透明の上に描き、後で地の上へ over する(Blender の
            // BackgroundPipeline / AE と同じ: 地は最初の clear 色で、以後どの pass にも「背景」は入らない)。
            let rects: Vec<TexturedRect> = Vec::new();

            // 空は地。環境層の run に来たら、空だけを 1 枚描いて、それまでの累算の**下**へ敷く
            // (Blender の BackgroundPipeline: 何も描かれていない画素にだけ世界が見える)。
            let sky = run.iter().any(|input| matches!(input.content, SequentialContent::Environment(e) if environment.is_some_and(|top| std::ptr::eq(top, e))));
            if sky {
                let sky_owned = spare
                    .pop()
                    .unwrap_or_else(|| self.create_blend_scratch_texture(comp.width, comp.height));
                let mut sky_view = ViewBuilder::new_with_external_resolved(
                    &self.ctx,
                    sequential_target_config("motolii-comp-sequential-sky", comp, view_from_world, projection, environment),
                    ViewBuilderId::new(self.next_readback),
                    &sky_owned,
                )
                .map_err(|e| CompositorError::View(e.to_string()))?;
                self.next_readback += 1;
                sky_view.queue_draw(
                    &self.ctx,
                    re_renderer::renderer::GenericSkyboxDrawData::new(&self.ctx, re_renderer::renderer::GenericSkyboxType::Environment),
                );
                let command_buffer = sky_view.draw(&self.ctx, Rgba::TRANSPARENT).map_err(|e| CompositorError::Draw(e.to_string()))?;
                if let Some(encoder) = blend_encoder.take() {
                    batch.push(encoder.finish());
                }
                batch.push(command_buffer);
                const DEST_OVER: u32 = 4;
                background = Some(self.stack_over(comp, background.take(), sky_owned, DEST_OVER, &mut spare, &mut blend_encoder)?);
            }
            let draws = self.surface_scene_draws(comp, run, rects, false, &|_| false, shared_meshes.as_ref(), run_start)?;

            let needs_backdrop = run.iter().any(|i| i.shading.reads_backdrop);
            let backdrop = match (&background, needs_backdrop) {
                (Some((backing, _)), true) => {
                    // 背後の合成はまだ blend encoder の中かもしれない。先に流してから写す。
                    if let Some(encoder) = blend_encoder.take() {
                        batch.push(encoder.finish());
                    }
                    let roughness = run.iter().filter(|i| i.shading.reads_backdrop).map(|i| i.shading.backdrop_roughness).fold(0.0f32, f32::max);
                    Some(self.backdrop_pyramid(comp, backing, &mut batch, roughness)?)
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
            config.scene_reflection = reflection.clone();
            config.light = light.clone();
            self.surface_work.main_runs += 1;

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

            draws.queue(&self.ctx, &mut view_builder);
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

            const SRC_OVER: u32 = 3;
            background = Some(self.stack_over(comp, background.take(), run_owned, SRC_OVER, &mut spare, &mut blend_encoder)?);
        }

        if let Some(encoder) = blend_encoder.take() {
            batch.push(encoder.finish());
        }
        self.pending.append(&mut batch);
        drop(spare);

        Ok(background)
    }

    /// ここまでの合成(背後)を mip 付きで写す。ガラスの網が粗さで段を読む。
    /// 描いた 1 枚(層または run)を地の上へ積む。地が無ければそれが地になる。
    /// `mode` は `vello_blend_mode` の番号(Normal は `SRC_OVER`)。
    fn stack_over(
        &mut self,
        comp: CompSpec,
        background: Option<(AccumulatorBacking, GpuTexture2D)>,
        canvas: AccumulatorBacking,
        mode: u32,
        spare: &mut Vec<AccumulatorBacking>,
        blend_encoder: &mut Option<wgpu::CommandEncoder>,
    ) -> Result<(AccumulatorBacking, GpuTexture2D), CompositorError> {
        let stacked = match background {
            None => canvas,
            Some((backing, _)) => {
                let dst_view = backing.create_view(&Default::default());
                let src_view = canvas.create_view(&Default::default());
                let out_texture = spare.pop().unwrap_or_else(|| self.create_blend_scratch_texture(comp.width, comp.height));
                let out_view = out_texture.create_view(&Default::default());
                let encoder = blend_encoder.get_or_insert_with(|| {
                    self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("motolii-compositor-blend-pass-encoder"),
                    })
                });
                let Self { ctx, blend_vism, effect_scratch, .. } = self;
                blend_vism.record_over(
                    ctx,
                    encoder,
                    effect_scratch,
                    &[&dst_view, &src_view],
                    &out_view,
                    &[("mode".to_owned(), mode as f32)],
                    [comp.width as f32, comp.height as f32],
                );
                spare.push(backing);
                spare.push(canvas);
                out_texture
            }
        };
        self.next_effect_key += 1;
        let key = self.next_effect_key;
        let imported = self
            .ctx
            .texture_manager_2d
            .import_gpu_premultiplied(key, &self.ctx, &stacked)
            .map_err(|e| CompositorError::Effect(e.to_string()))?;
        Ok((stacked, imported))
    }

    fn backdrop_pyramid(
        &mut self,
        comp: CompSpec,
        backing: &wgpu::Texture,
        batch: &mut Vec<wgpu::CommandBuffer>,
        max_roughness: f32,
    ) -> Result<GpuTexture2D, CompositorError> {
        self.surface_work.backdrop_copies += 1;
        let size = wgpu::Extent3d { width: comp.width, height: comp.height, depth_or_array_layers: 1 };
        let resource = match self.backdrop_resource.take() {
            Some(resource) if resource.dimensions == [comp.width, comp.height] => resource,
            _ => {
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
                let imported = self.import_premultiplied(&texture)?;
                self.surface_work.backdrop_allocations += 1;
                BackdropResource { dimensions: [comp.width,comp.height], texture, imported }
            }
        };
        let texture = &resource.texture;
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
        encoder.copy_texture_to_texture(level0(backing), level0(texture), size);
        // 粗さが読む段までしか焼かない(Unity の opaque texture と同じ嘘)。粗さ 0 なら写しだけ。
        let levels = re_renderer::backdrop_levels_read(max_roughness, texture.mip_level_count());
        self.surface_work.backdrop_mip_levels += u64::from(levels);
        self.ctx.texture_manager_2d.generate_mipmap_levels(&self.ctx, &mut encoder, texture, levels);
        batch.push(encoder.finish());
        let result = resource.imported.clone();
        self.backdrop_resource = Some(resource);
        Ok(result)
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
        let _ = camera;
        let mut final_rects: Vec<TexturedRect> = Vec::with_capacity(1);
        if let Some((_, imported)) = &background {
            final_rects.push(screen_rect(comp, imported.clone()));
        }

        let final_draw_data = RectangleDrawData::new(&self.ctx, &final_rects)
            .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

        let mut final_view_builder = ViewBuilder::new(
            &self.ctx,
            screen_target_config("motolii-comp-sequential-finalize", comp),
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
        let submit_start = std::time::Instant::now();
        self.flush_pending();
        self.measurement.submit_us = submit_start.elapsed().as_micros() as u64;
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
        let mut config = sequential_target_config("motolii-comp-outline", comp, view_from_world, projection, None);
        config.outline_config = Some(re_renderer::OutlineConfig {
            outline_radius_pixel: 1.0,
            color_layer_a: Rgba::TRANSPARENT,
            color_layer_b: Rgba::TRANSPARENT,
        });
        let owned = self.create_blend_scratch_texture(comp.width, comp.height);
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
            bounds.record(&self.ctx.device, &mut encoder, &mask.default_view, [comp.width, comp.height]);
            self.pending.push(encoder.finish());
        }
        Ok(Some((view_builder, owned)))
    }

    pub(crate) fn finalize_into(
        &mut self,
        target: &wgpu::Texture,
        comp: CompSpec,
        camera: ResolvedCamera,
        background: Option<(AccumulatorBacking, GpuTexture2D)>,
        background_color: [f32; 4],
        outline: Option<(ViewBuilder, AccumulatorBacking)>,
    ) -> Result<(), CompositorError> {
        let _ = camera;
        let mut final_rects: Vec<TexturedRect> = Vec::with_capacity(1);
        if let Some((_, imported)) = &background {
            final_rects.push(screen_rect(comp, imported.clone()));
        }
        let draw_data = RectangleDrawData::new(&self.ctx, &final_rects)
            .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

        let mut view_builder = {
            let mut vb = ViewBuilder::new(
                &self.ctx,
                screen_target_config("motolii-comp-finalize-into", comp),
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
                    crate::render::compositor::LayerContent::LinearTexture(t) => crate::render::compositor::SequentialContent::LinearRect(t),
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
                blocks_light: layer.blocks_light,
                outline: layer.outline,
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
            blocks_light: layer.blocks_light,
            outline: layer.outline,
            frame: None,
        })
    }
}
