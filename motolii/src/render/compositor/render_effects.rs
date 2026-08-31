
use crate::render::compositor::*;

impl Compositor {
    // `&mut EffectScratch` と Vism を同時に触るための分割借用。self をまとめて
    // 借りると衝突するので、必要な場所だけ field ごとに分ける。
    fn record_isf(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        src_view: &wgpu::TextureView,
        dst_view: &wgpu::TextureView,
        params: &[(String, f32)],
        render_size: [f32; 2],
    ) {
        let Self { ctx, isf_bloom, effect_scratch, .. } = self;
        isf_bloom.record(ctx, encoder, effect_scratch, src_view, dst_view, params, render_size);
    }

    fn record_gradient(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        dst_view: &wgpu::TextureView,
        render_size: [f32; 2],
    ) {
        let Self { ctx, wgsl_gradient, effect_scratch, .. } = self;
        wgsl_gradient.record(ctx, encoder, effect_scratch, dst_view, render_size);
    }

    fn record_tri_led(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        dst_view: &wgpu::TextureView,
        render_size: [f32; 2],
    ) {
        let Self { ctx, wgsl_tri_led, effect_scratch, .. } = self;
        wgsl_tri_led.record(ctx, encoder, effect_scratch, dst_view, render_size);
    }

    pub(crate) fn effective_layer_textures(
        &mut self,
        layers: &[LayerWithPasses],
    ) -> Result<
        (
            Vec<LayerContent>,
            Vec<u32>,
            Vec<(u32, u32, wgpu::TextureFormat, wgpu::Texture)>,
        ),
        CompositorError,
    > {
        let mut effective_textures: Vec<LayerContent> = Vec::with_capacity(layers.len());
        let mut effective_paddings: Vec<u32> = Vec::with_capacity(layers.len());
        let mut checked_out: Vec<(u32, u32, wgpu::TextureFormat, wgpu::Texture)> = Vec::new();
        let mut copy_encoder: Option<wgpu::CommandEncoder> = None;

        for lwp in layers {
            // 3D の素材は焼かない(裁定 2026-08-30)。エフェクトはテクスチャの上でしか
            // 動かないので、掛かっていても素通しする。
            let Some(layer_texture) = lwp.layer.content.texture().cloned() else {
                effective_textures.push(lwp.layer.content.clone());
                effective_paddings.push(0);
                continue;
            };
            if lwp.passes.is_empty() {
                effective_textures.push(LayerContent::Texture(layer_texture));
                effective_paddings.push(0);
                continue;
            }

            let [width, height] = layer_texture.width_height();
            let padding = lwp
                .passes
                .iter()
                .map(EffectPass::padding)
                .max()
                .unwrap_or(0);
            let padded_width = width + 2 * padding;
            let padded_height = height + 2 * padding;

            let format = lwp
                .passes
                .iter()
                .find_map(EffectPass::intermediate_format)
                .unwrap_or_else(|| layer_texture.format());

            let src_handle = layer_texture.handle();
            let src = self
                .ctx
                .gpu_resources
                .textures
                .get_from_handle(src_handle)
                .map_err(|e| CompositorError::Effect(e.to_string()))?;

            let scratch =
                self.effect_scratch
                    .acquire(&self.ctx.device, padded_width, padded_height, format);

            let encoder = copy_encoder.get_or_insert_with(|| {
                self.ctx
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("motolii-compositor-effective-layer-textures"),
                    })
            });

            for pass in &lwp.passes {
                match pass {
                    EffectPass::Identity => {
                        encoder.copy_texture_to_texture(
                            wgpu::TexelCopyTextureInfo {
                                texture: &src.texture,
                                mip_level: 0,
                                origin: wgpu::Origin3d::ZERO,
                                aspect: wgpu::TextureAspect::All,
                            },
                            wgpu::TexelCopyTextureInfo {
                                texture: &scratch,
                                mip_level: 0,
                                origin: wgpu::Origin3d {
                                    x: padding,
                                    y: padding,
                                    z: 0,
                                },
                                aspect: wgpu::TextureAspect::All,
                            },
                            wgpu::Extent3d {
                                width,
                                height,
                                depth_or_array_layers: 1,
                            },
                        );
                    }
                    EffectPass::Glow {
                        threshold,
                        intensity,
                        radius,
                    } => {
                        let padded_source = self.effect_scratch.acquire(
                            &self.ctx.device,
                            padded_width,
                            padded_height,
                            layer_texture.format(),
                        );
                        let padded_source_view = padded_source.create_view(&Default::default());
                        {
                            let _clear_pass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some(
                                        "motolii-compositor-glow-padded-source-clear",
                                    ),
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view: &padded_source_view,
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
                        }
                        encoder.copy_texture_to_texture(
                            wgpu::TexelCopyTextureInfo {
                                texture: &src.texture,
                                mip_level: 0,
                                origin: wgpu::Origin3d::ZERO,
                                aspect: wgpu::TextureAspect::All,
                            },
                            wgpu::TexelCopyTextureInfo {
                                texture: &padded_source,
                                mip_level: 0,
                                origin: wgpu::Origin3d {
                                    x: padding,
                                    y: padding,
                                    z: 0,
                                },
                                aspect: wgpu::TextureAspect::All,
                            },
                            wgpu::Extent3d {
                                width,
                                height,
                                depth_or_array_layers: 1,
                            },
                        );

                        let dst_view = scratch.create_view(&Default::default());
                        let Self { ctx, glow_vism, effect_scratch, .. } = self;
                        glow_vism.record_over(
                            ctx,
                            encoder,
                            effect_scratch,
                            &[&padded_source_view],
                            &dst_view,
                            &[
                                ("threshold".to_owned(), *threshold),
                                ("intensity".to_owned(), *intensity),
                                ("radius".to_owned(), *radius),
                            ],
                            [padded_width as f32, padded_height as f32],
                        );
                        effect_scratch.release(
                            padded_width,
                            padded_height,
                            layer_texture.format(),
                            padded_source,
                        );
                    }
                    EffectPass::Isf { params } => {
                        let src_view = src.texture.create_view(&Default::default());
                        let dst_view = scratch.create_view(&Default::default());
                        self.record_isf(
                            encoder,
                            &src_view,
                            &dst_view,
                            params,
                            [width as f32, height as f32],
                        );
                    }
                    EffectPass::Gradient => {
                        let dst_view = scratch.create_view(&Default::default());
                        self.record_gradient(encoder, &dst_view, [width as f32, height as f32]);
                    }
                    EffectPass::TriLed => {
                        let dst_view = scratch.create_view(&Default::default());
                        self.record_tri_led(encoder, &dst_view, [width as f32, height as f32]);
                    }
                }
            }

            self.next_effect_key += 1;
            let key = self.next_effect_key;
            let imported = self
                .ctx
                .texture_manager_2d
                .import_gpu_premultiplied(key, &self.ctx, &scratch)
                .map_err(|e| CompositorError::Effect(e.to_string()))?;

            effective_textures.push(LayerContent::Texture(imported));
            effective_paddings.push(padding);
            checked_out.push((padded_width, padded_height, format, scratch));
        }

        if let Some(encoder) = copy_encoder.take() {
            self.pending.push(encoder.finish());
        }
        // 効果の出力を次段が読むので、ここで一度だけ出す(層ごとには止めない)。
        self.flush_pending();

        Ok((effective_textures, effective_paddings, checked_out))
    }

    pub fn render_with_effects(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layers: &[LayerWithPasses],
        background_color: [f32; 4],
    ) -> Result<Vec<u8>, CompositorError> {
        let (effective_textures, effective_paddings, checked_out) =
            self.effective_layer_textures(layers)?;

        let inputs = sequential_inputs(layers, &effective_textures, &effective_paddings);

        let background = self.accumulate_sequential(comp, camera, &inputs, background_color)?;
        let frame = self.finalize_readback(comp, camera, background, background_color)?;

        for (width, height, format, texture) in checked_out {
            self.effect_scratch.release(width, height, format, texture);
        }

        Ok(frame)
    }

    pub fn render_to_texture(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layers: &[LayerWithPasses],
        background_color: [f32; 4],
    ) -> Result<(wgpu::Texture, wgpu::TextureView), CompositorError> {
        let (effective_textures, effective_paddings, checked_out) =
            self.effective_layer_textures(layers)?;

        let inputs = sequential_inputs(layers, &effective_textures, &effective_paddings);

        let background = self.accumulate_sequential(comp, camera, &inputs, background_color)?;
        let (texture, view) = self.finalize_texture(comp, camera, background, background_color)?;

        for (width, height, format, scratch_texture) in checked_out {
            self.effect_scratch
                .release(width, height, format, scratch_texture);
        }

        Ok((texture, view))
    }

}

pub(crate) fn sequential_inputs<'a>(
    layers: &'a [LayerWithPasses],
    effective_textures: &'a [LayerContent],
    effective_paddings: &[u32],
) -> Vec<SequentialInput<'a>> {
    layers
        .iter()
        .zip(effective_textures.iter())
        .zip(effective_paddings.iter())
        .map(|((lwp, content), &padding)| {
            let layer = &lwp.layer;
            let pad = padding as f32;
            SequentialInput {
                content: match content {
                    LayerContent::Texture(t) => SequentialContent::Rect(t),
                    LayerContent::Cloud { positions, colors, point_size } => {
                        SequentialContent::Cloud { positions, colors, point_size: *point_size }
                    }
                    LayerContent::Mesh(mesh) => SequentialContent::Mesh(mesh),
                },
                local_min: glam::Vec2::new(-pad, -pad),
                local_size: glam::Vec2::new(layer.size[0] + 2.0 * pad, layer.size[1] + 2.0 * pad),
                transform: layer.placement.transform,
                z: layer.placement.z,
                rotation_x: layer.placement.rotation_x,
                rotation_y: layer.placement.rotation_y,
                pinned: layer.pinned,
                opacity: layer.placement.opacity,
                depth_offset: layer.placement.order,
                blend_mode: layer.blend_mode,
            }
        })
        .collect()
}
