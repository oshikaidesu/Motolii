use crate::render::compositor::*;

impl Compositor {
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
        self.refresh_catalog_programs();
        for pass in layers.iter().flat_map(|layer| &layer.passes) {
            if !self.effect_programs.contains_key(&pass.plugin_id) {
                return Err(CompositorError::Effect(format!(
                    "unknown Vism {}",
                    pass.plugin_id
                )));
            }
        }
        let mut effective_textures = Vec::with_capacity(layers.len());
        let mut effective_paddings = Vec::with_capacity(layers.len());
        let mut checked_out = Vec::new();
        let mut copy_encoder: Option<wgpu::CommandEncoder> = None;

        for lwp in layers {
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
            let border = padding
                .checked_mul(2)
                .ok_or_else(|| CompositorError::Effect("effect padding overflow".into()))?;
            let padded_width = width
                .checked_add(border)
                .ok_or_else(|| CompositorError::Effect("effect width overflow".into()))?;
            let padded_height = height
                .checked_add(border)
                .ok_or_else(|| CompositorError::Effect("effect height overflow".into()))?;
            let src = self
                .ctx
                .gpu_resources
                .textures
                .get_from_handle(layer_texture.handle())
                .map_err(|error| CompositorError::Effect(error.to_string()))?;
            let mut current = src.texture.clone();
            let mut current_is_scratch = false;
            let encoder = copy_encoder.get_or_insert_with(|| {
                self.ctx
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("motolii-compositor-effective-layer-textures"),
                    })
            });

            if padding > 0 && self.effect_programs[&lwp.passes[0].plugin_id].image_input_count() > 0
            {
                let padded = self.effect_scratch.acquire(
                    &self.ctx.device,
                    padded_width,
                    padded_height,
                    current.format(),
                );
                let padded_view = padded.create_view(&Default::default());
                {
                    let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("motolii-compositor-vism-padded-source-clear"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &padded_view,
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
                        texture: &current,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyTextureInfo {
                        texture: &padded,
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
                current = padded;
                current_is_scratch = true;
            }

            for pass in &lwp.passes {
                let program = &self.effect_programs[&pass.plugin_id];
                let format = pass
                    .intermediate_format()
                    .unwrap_or_else(|| current.format());
                let destination = self.effect_scratch.acquire(
                    &self.ctx.device,
                    padded_width,
                    padded_height,
                    format,
                );
                let source_view = (program.image_input_count() > 0)
                    .then(|| current.create_view(&Default::default()));
                let sources: Vec<_> = source_view.iter().collect();
                let destination_view = destination.create_view(&Default::default());
                program.record(
                    &self.ctx,
                    encoder,
                    &mut self.effect_scratch,
                    &sources,
                    &destination_view,
                    &pass.params,
                    [padded_width as f32, padded_height as f32],
                );
                // The previous output stays checked out until its consuming pass is recorded.
                // Reuse thereafter is ordered by this command encoder, never within the same pass.
                if current_is_scratch {
                    self.effect_scratch.release(
                        padded_width,
                        padded_height,
                        current.format(),
                        current,
                    );
                }
                current = destination;
                current_is_scratch = true;
            }

            self.next_effect_key += 1;
            let imported = self
                .ctx
                .texture_manager_2d
                .import_gpu_premultiplied(self.next_effect_key, &self.ctx, &current)
                .map_err(|error| CompositorError::Effect(error.to_string()))?;
            effective_textures.push(LayerContent::Texture(imported));
            effective_paddings.push(padding);
            checked_out.push((padded_width, padded_height, current.format(), current));
        }
        if let Some(encoder) = copy_encoder {
            self.pending.push(encoder.finish());
        }
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
                    LayerContent::Cloud {
                        positions,
                        colors,
                        bounds,
                        point_size,
                    } => SequentialContent::Cloud {
                        positions,
                        colors,
                        bounds: *bounds,
                        point_size: *point_size,
                    },
                    LayerContent::Model(model) => SequentialContent::Model(model),
                },
                local_min: glam::Vec2::new(-pad, -pad),
                local_size: glam::Vec2::new(layer.size[0] + 2.0 * pad, layer.size[1] + 2.0 * pad),
                placement: layer.placement,
                projection: layer.projection,
                projection_camera: layer.projection_camera,
                opacity: layer.placement.opacity,
                depth_offset: layer.placement.order,
                blend_mode: layer.blend_mode,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::doc::store::{
        Composition, Document, EffectId, EffectInstance, Fps, Intent, LayerId, LayerMeta,
        LayerSource, LayerTiming, PropertyId, RationalTime, Value,
    };
    use crate::render::engine::Engine;

    fn document_with_effects(path: &std::path::Path, plugins: &[&str]) -> Document {
        let mut doc = Document::new();
        doc.apply_all([
            Intent::SetComposition(Composition {
                width: 24,
                height: 24,
                fps: Fps::try_new(30, 1).unwrap(),
                duration_frames: 1,
                background: [0.0; 4],
            }),
            Intent::AddLayer(LayerId(1)),
            Intent::SetMeta {
                layer: LayerId(1),
                meta: LayerMeta {
                    source: LayerSource::File {
                        path: path.to_string_lossy().into_owned(),
                        fingerprint: None,
                    },
                    order: 0,
                    timing: LayerTiming::place(0, None, 1),
                },
            },
            Intent::SetEffects {
                layer: LayerId(1),
                effects: plugins
                    .iter()
                    .enumerate()
                    .map(|(index, plugin)| EffectInstance {
                        id: EffectId(index as u32),
                        plugin_id: (*plugin).into(),
                    })
                    .collect(),
            },
        ])
        .unwrap();
        doc
    }

    #[test]
    fn mixed_format_effects_consume_previous_output_and_recover_after_undo() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.png");
        let pixels = [32u8, 64, 96, 255]
            .into_iter()
            .cycle()
            .take(24 * 24 * 4)
            .collect::<Vec<_>>();
        image::save_buffer(&source, &pixels, 24, 24, image::ColorType::Rgba8).unwrap();
        let mut doc = document_with_effects(&source, &["motolii.gradient"]);
        let mut engine = Engine::new().unwrap();
        let mut checked_render = |doc: &Document| {
            let scope = engine
                .gpu_device()
                .push_error_scope(wgpu::ErrorFilter::Validation);
            let pixels = engine
                .render_frame(&doc.view(), RationalTime::ZERO)
                .unwrap();
            let error = pollster::block_on(scope.pop());
            assert!(
                error.is_none(),
                "effect attachments must match pipeline formats: {error:?}"
            );
            pixels
        };
        let gradient = checked_render(&doc);
        assert!(
            gradient.chunks_exact(4).any(|pixel| pixel[0] != pixel[1]),
            "the reference is visibly nonuniform"
        );
        for plugins in [
            vec!["motolii.gradient", "motolii.blur"],
            vec!["motolii.blur", "motolii.gradient"],
            vec!["motolii.gradient", "motolii.blur", "motolii.blur"],
        ] {
            let mut edits = vec![Intent::SetEffects {
                layer: LayerId(1),
                effects: plugins
                    .iter()
                    .enumerate()
                    .map(|(index, plugin)| EffectInstance {
                        id: EffectId(index as u32),
                        plugin_id: (*plugin).into(),
                    })
                    .collect(),
            }];
            for (index, plugin) in plugins.iter().enumerate() {
                if *plugin == "motolii.blur" {
                    edits.push(Intent::SetConstant {
                        layer: LayerId(1),
                        property: PropertyId::effect_param(EffectId(index as u32), "radius")
                            .unwrap(),
                        value: Value::F64(0.0),
                    });
                }
            }
            doc.apply_all(edits).unwrap();
            let actual = checked_render(&doc);
            assert_eq!(actual.len(), gradient.len());
            assert!(actual.iter().zip(&gradient).all(|(actual, expected)| actual.abs_diff(*expected) <= 2),
            "a zero-radius blur must preserve the previous Vism output through 8-bit/float format transitions: {plugins:?}");
            assert!(doc.undo());
            assert_eq!(
                checked_render(&doc),
                gradient,
                "removing the chain restores rendering in the same Engine"
            );
        }
    }
}
