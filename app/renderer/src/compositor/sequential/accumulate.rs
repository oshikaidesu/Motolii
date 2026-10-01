//! 積む: 焼ける矩形はその場で混ぜ、焼けない物は run にまとめて描き、地の上へ重ねる。
//! ガラスが読む背後の mip(`backdrop_pyramid`)と、層の絵へ焼けなかった効果を画面で掛ける所(`apply_screen_passes`)もここ。

use super::*;

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
        blend_vism.get(ctx).record_over(
            ctx, &mut encoder, effect_scratch, &[&base_view, &upper_view], &output_view,
            &[("mode".to_owned(), mode as f32)], [width as f32, height as f32],
        );
        self.pending.push(encoder.finish());
        self.import_premultiplied(&output)
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
                        field_grid: input.shading.field_grid(),
                        ..Default::default()
                    },
                };
                let draw_data = RectangleDrawData::new(&self.ctx, &[solo_rect])
                    .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

                let solo_owned = spare
                    .pop()
                    .unwrap_or_else(|| self.create_blend_scratch_texture(self.window.width, self.window.height));
                let mut solo_config = sequential_target_config("motolii-comp-sequential-solo", comp, self.window, view_from_world, projection, environment,
                );
                if input.projection != crate::doc::store::LayerProjection::TwoD && self.window.projection_camera.is_none() {
                    solo_config.near_fade_distance = camera.near_fade;
                }
                solo_config.scene_reflection = reflection.clone();
                solo_config.motion = self.motion.clone();
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
            // 2D は世界に居ない(法 2026-09-12): 深度に参加せず、積み順だけで重なる。2D と非 2D を同じ run に
            // 入れないことで、2D の連続は 1 つの面、間の 3D 群は自分たちだけで深度を解き、run 同士は積み順で
            // over になる(AE の「2D 層は 3D 世界の仕切り」)。描くのはどちらも re_renderer の同じ view。
            let flat = |i: &SequentialInput<'_>| i.projection == crate::doc::store::LayerProjection::TwoD;
            while idx < inputs.len() && !bakeable(&inputs[idx]) {
                if idx > run_start && flat(&inputs[idx]) != flat(&inputs[run_start]) {
                    break;
                }
                // 表面プログラムは手前で run を切り、それまでの合成を背後として読む。
                if idx > run_start && (inputs[idx].shading.reads_backdrop || (run_has_rect && matches!(inputs[idx].content, SequentialContent::Model(_)))) {
                    break;
                }
                // 画面で効く効果列を持つ層は 1 つで 1 run。隣を巻き込むと隣にも効いてしまう。
                if idx > run_start && !inputs[idx].screen_passes.is_empty() {
                    break;
                }
                run_has_rect |= matches!(inputs[idx].content, SequentialContent::Rect(_) | SequentialContent::LinearRect(_));
                idx += 1;
                if !inputs[idx - 1].screen_passes.is_empty() {
                    break;
                }
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
                    .unwrap_or_else(|| self.create_blend_scratch_texture(self.window.width, self.window.height));
                let mut sky_view = ViewBuilder::new_with_external_resolved(
                    &self.ctx,
                    sequential_target_config("motolii-comp-sequential-sky", comp, self.window, view_from_world, projection, environment),
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
            let mut config = sequential_target_config("motolii-comp-sequential-run", comp, self.window,
                view_from_world,
                projection,
                environment,
            );
            config.backdrop = backdrop;
            // 近いと薄く: 作中カメラの絵の、世界に居る run だけ(2D は画面の物、Stage は作中カメラに従わない)。
            if !flat(&run[0]) && self.window.projection_camera.is_none() {
                config.near_fade_distance = camera.near_fade;
            }
            config.scene_reflection = reflection.clone();
            config.motion = self.motion.clone();
            config.light = light.clone();
            self.surface_work.main_runs += 1;

            let run_owned = spare
                .pop()
                .unwrap_or_else(|| self.create_blend_scratch_texture(self.window.width, self.window.height));
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

            // 焼く先の絵が無かった素材(網・点群・環境)の効果列は、描いた後のこの窓へ流す。
            let run_owned = match run {
                [only] if !only.screen_passes.is_empty() => {
                    let encoder = blend_encoder.get_or_insert_with(|| {
                        self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("motolii-compositor-screen-passes-encoder"),
                        })
                    });
                    let backdrop = background.as_ref().map(|(backing, _)| backing.clone());
                    self.apply_screen_passes(encoder, run_owned, only.screen_passes, only.screen_sources, backdrop, &mut spare)?
                }
                _ => run_owned,
            };

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
    /// 窓 1 枚に効果列を流し、同じ寸法・同じ形式の新しい窓を返す。
    ///
    /// 層の絵へ焼けなかった効果(網・点群・環境には焼く先が無い)の行き先。層の平面ではなく
    /// **画面**で効くので、その層はこの run の中で平らな 1 枚になる(AE のプリコンポと同じ代償)。
    fn apply_screen_passes(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        canvas: AccumulatorBacking,
        passes: &[EffectPass],
        sources: &[Vec<GpuTexture2D>],
        backdrop: Option<AccumulatorBacking>,
        spare: &mut Vec<AccumulatorBacking>,
    ) -> Result<AccumulatorBacking, CompositorError> {
        let (width, height) = (self.window.width, self.window.height);
        // 下の合成を読む効果には、ここまでの合成を 2 枚目の image として渡す(層の絵と同じ sRGB 符号化)。
        // まだ何も無ければ透明。
        let mut below: Option<wgpu::Texture> = None;
        if passes.iter().any(|p| p.reads_backdrop) {
            // 累算器は乗算済み線形(sRGB 形式は読む時に hardware が decode)。そのまま渡せる。
            below = Some(match &backdrop {
                Some(backing) => backing.clone(),
                None => self.ctx.texture_manager_2d.zeroed_texture_float().texture.clone(),
            });
        }
        // 2 枚目: 下の合成(今)か、別の時刻の合成(host が先に描いた写し)。
        let others: Vec<Vec<wgpu::Texture>> = passes.iter().enumerate().map(|(i, p)| match (&below, p.reads_backdrop) {
            (Some(b), true) => vec![b.clone()],
            _ => sources.get(i).map(|row| row.iter().filter_map(|t| self.ctx.gpu_resources.textures.get_from_handle(t.handle()).ok().map(|g| g.texture.clone())).collect()).unwrap_or_default(),
        }).collect();
        // 窓は線形。効果列は層の絵と同じ作法(Pass は sRGB 符号化で受ける)で流し、終わりで線形へ戻す。
        let (mut current, linear, premultiplied, mut is_scratch) = self.record_pass_chain(
            encoder, canvas.clone(), true, true, false, passes, &others, None, [width, height], 0, [width, height],
        )?;
        drop(below);
        if !linear {
            let back = self.convert_image_encoding(encoder, &current, true, true, premultiplied);
            if is_scratch {
                self.effect_scratch.release(width, height, current.format(), current);
            }
            current = back;
            is_scratch = true;
        }

        let out = spare.pop().unwrap_or_else(|| self.create_blend_scratch_texture(width, height));
        let dst_view = canvas.create_view(&Default::default());
        let src_view = current.create_view(&Default::default());
        let out_view = out.create_view(&Default::default());
        let window = self.window.size_f32();
        {
            // compose 1 = copy(vello の Compose: Clear=0, Copy=1, Dest=2, SrcOver=3)。
            // 下は読まない — 効果の結果で置き換える。
            const COPY: u32 = 1;
            let Self { ctx, blend_vism, effect_scratch, .. } = self;
            blend_vism.get(ctx).record_over(
                ctx, encoder, effect_scratch,
                &[&dst_view, &src_view], &out_view,
                &[("mode".to_owned(), COPY as f32)], window,
            );
        }
        if is_scratch {
            self.effect_scratch.release(width, height, current.format(), current);
        }
        spare.push(canvas);
        Ok(out)
    }

    fn stack_over(
        &mut self,
        _comp: CompSpec,
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
                let out_texture = spare.pop().unwrap_or_else(|| self.create_blend_scratch_texture(self.window.width, self.window.height));
                let out_view = out_texture.create_view(&Default::default());
                let window = self.window.size_f32();
                let encoder = blend_encoder.get_or_insert_with(|| {
                    self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("motolii-compositor-blend-pass-encoder"),
                    })
                });
                let Self { ctx, blend_vism, effect_scratch, .. } = self;
                blend_vism.get(ctx).record_over(
                    ctx,
                    encoder,
                    effect_scratch,
                    &[&dst_view, &src_view],
                    &out_view,
                    &[("mode".to_owned(), mode as f32)],
                    window,
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
        _comp: CompSpec,
        backing: &wgpu::Texture,
        batch: &mut Vec<wgpu::CommandBuffer>,
        max_roughness: f32,
    ) -> Result<GpuTexture2D, CompositorError> {
        self.surface_work.backdrop_copies += 1;
        let window = self.window;
        let size = wgpu::Extent3d { width: window.width, height: window.height, depth_or_array_layers: 1 };
        let resource = match self.backdrop_resource.take() {
            Some(resource) if resource.dimensions == window.size() => resource,
            _ => {
                let texture = self.ctx.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("motolii-backdrop-pyramid"),
                    size,
                    mip_level_count: re_renderer::resource_managers::MipmapGenerator::mip_level_count(window.width, window.height),
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
                BackdropResource { dimensions: window.size(), texture, imported }
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
}
