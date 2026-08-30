
use crate::*;

impl Compositor {
    pub(crate) fn effective_layer_textures(
        &mut self,
        layers: &[LayerWithPasses],
    ) -> Result<
        (
            Vec<GpuTexture2D>,
            Vec<u32>,
            Vec<(u32, u32, wgpu::TextureFormat, wgpu::Texture)>,
        ),
        CompositorError,
    > {
        let mut effective_textures: Vec<GpuTexture2D> = Vec::with_capacity(layers.len());
        let mut effective_paddings: Vec<u32> = Vec::with_capacity(layers.len());
        let mut checked_out: Vec<(u32, u32, wgpu::TextureFormat, wgpu::Texture)> = Vec::new();
        let mut copy_encoder: Option<wgpu::CommandEncoder> = None;

        for lwp in layers {
            if lwp.passes.is_empty() {
                effective_textures.push(lwp.layer.texture.clone());
                effective_paddings.push(0);
                continue;
            }

            let [width, height] = lwp.layer.texture.width_height();
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
                .unwrap_or_else(|| lwp.layer.texture.format());

            let src_handle = lwp.layer.texture.handle();
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
                            lwp.layer.texture.format(),
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

                        let bloom = self.effect_scratch.acquire(
                            &self.ctx.device,
                            padded_width,
                            padded_height,
                            effects::GLOW_INTERMEDIATE_FORMAT,
                        );
                        let blur_ping = self.effect_scratch.acquire(
                            &self.ctx.device,
                            padded_width,
                            padded_height,
                            effects::GLOW_INTERMEDIATE_FORMAT,
                        );
                        let bloom_view = bloom.create_view(&Default::default());
                        let blur_ping_view = blur_ping.create_view(&Default::default());
                        let dst_view = scratch.create_view(&Default::default());

                        self.glow_pipelines.record(
                            &self.ctx.device,
                            &self.ctx.queue,
                            encoder,
                            &padded_source_view,
                            &bloom_view,
                            &blur_ping_view,
                            &dst_view,
                            *threshold,
                            *intensity,
                            *radius,
                        );

                        checked_out.push((
                            padded_width,
                            padded_height,
                            lwp.layer.texture.format(),
                            padded_source,
                        ));
                        checked_out.push((
                            padded_width,
                            padded_height,
                            effects::GLOW_INTERMEDIATE_FORMAT,
                            bloom,
                        ));
                        checked_out.push((
                            padded_width,
                            padded_height,
                            effects::GLOW_INTERMEDIATE_FORMAT,
                            blur_ping,
                        ));
                    }
                    EffectPass::Isf { params } => {
                        let src_view = src.texture.create_view(&Default::default());
                        let dst_view = scratch.create_view(&Default::default());
                        self.isf_bloom.record(
                            &self.ctx,
                            encoder,
                            &src_view,
                            &dst_view,
                            params,
                            [width as f32, height as f32],
                        );
                    }
                    EffectPass::Gradient => {
                        let dst_view = scratch.create_view(&Default::default());
                        self.wgsl_gradient.record(&self.ctx, encoder, &dst_view);
                    }
                    EffectPass::TriLed => {
                        let dst_view = scratch.create_view(&Default::default());
                        self.wgsl_tri_led.record(&self.ctx, encoder, &dst_view);
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

            effective_textures.push(imported);
            effective_paddings.push(padding);
            checked_out.push((padded_width, padded_height, format, scratch));
        }

        if let Some(encoder) = copy_encoder.take() {
            self.ctx.before_submit();
            self.ctx.queue.submit([encoder.finish()]);
            self.ctx.begin_frame();
            self.ctx
                .device
                .poll(wgpu::PollType::wait_indefinitely())
                .map_err(|e| CompositorError::Draw(e.to_string()))?;
        }

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
    effective_textures: &'a [GpuTexture2D],
    effective_paddings: &[u32],
) -> Vec<SequentialInput<'a>> {
    layers
        .iter()
        .zip(effective_textures.iter())
        .zip(effective_paddings.iter())
        .map(|((lwp, texture), &padding)| {
            let layer = &lwp.layer;
            let pad = padding as f32;
            SequentialInput {
                texture,
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
