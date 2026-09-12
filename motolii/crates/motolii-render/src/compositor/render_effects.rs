use crate::render::compositor::*;

/// 層ごとの (効果後の絵, 余白 px, 溢れ) と、フレーム後に pool へ返す scratch。
/// 溢れ = coverage の外へ出た分の絵と、その混ぜ方(溢れの法、manifest の `SPILL`)。
pub(crate) type LayerSpill = Option<(LayerContent, BlendMode)>;
/// 層の持ち物: 焼いた効果。鍵(素材の texture・効果列の値・枠)が同じ間は焼き直さない。
/// Stage と Camera の 2 枚も、静止した層の次のコマも、同じ物を覗く(rerun の store と同じ持ち方)。
/// 2 render 続けて使われなかった物は scratch へ返す。
#[derive(Default)]
pub(crate) struct BakedEffects {
    entries: Vec<BakedEntry>,
    generation: u64,
}
struct BakedKey { source: GpuTexture2D, passes: Vec<EffectPass>, frame: Option<effects::vism::ImageFrame> }
impl PartialEq for BakedKey {
    fn eq(&self, other: &Self) -> bool { self.source.handle() == other.source.handle() && self.passes == other.passes && self.frame == other.frame }
}
struct BakedEntry { key: BakedKey, content: LayerContent, padding: u32, spill: LayerSpill, owned: Vec<(u32, u32, wgpu::TextureFormat, wgpu::Texture)>, used: u64 }
impl BakedEffects {
    fn hit(&mut self, key: &BakedKey) -> Option<(LayerContent, u32, LayerSpill)> {
        let generation = self.generation;
        let entry = self.entries.iter_mut().find(|e| e.key == *key)?;
        entry.used = generation;
        Some((entry.content.clone(), entry.padding, entry.spill.clone()))
    }
    fn keep(&mut self, key: BakedKey, content: LayerContent, padding: u32, spill: LayerSpill, owned: Vec<(u32, u32, wgpu::TextureFormat, wgpu::Texture)>) {
        let used = self.generation;
        self.entries.push(BakedEntry { key, content, padding, spill, owned, used });
    }
    fn sweep(&mut self, scratch: &mut effects::EffectScratch) {
        let generation = self.generation;
        self.generation += 1;
        let (kept, stale): (Vec<_>, Vec<_>) = self.entries.drain(..).partition(|e| e.used + 1 >= generation);
        self.entries = kept;
        for entry in stale {
            for (width, height, format, texture) in entry.owned { scratch.release(width, height, format, texture); }
        }
    }
    pub(crate) fn clear(&mut self, scratch: &mut effects::EffectScratch) {
        for entry in self.entries.drain(..) {
            for (width, height, format, texture) in entry.owned { scratch.release(width, height, format, texture); }
        }
    }
}

type EffectiveLayers = (Vec<LayerContent>, Vec<u32>, Vec<LayerSpill>, Vec<(u32,u32,wgpu::TextureFormat,wgpu::Texture)>);

impl Compositor {
    pub(crate) fn convert_image_encoding(&mut self, encoder: &mut wgpu::CommandEncoder, source: &wgpu::Texture, to_linear: bool) -> wgpu::Texture {
        const ID: &str = "motolii.material_encoding";
        if !self.effect_programs.contains_key(ID) {
            let definition = self.catalog.definitions.iter().find(|d| d.plugin_id() == ID).expect("material encoding shader");
            self.effect_programs.insert(ID.into(), effects::EffectProgram::compile_for(&self.ctx, definition, wgpu::TextureFormat::Rgba16Float));
        }
        let out = self.effect_scratch.acquire(&self.ctx.device,source.width(),source.height(),wgpu::TextureFormat::Rgba16Float);
        self.effect_programs[ID].record(&self.ctx, encoder, &mut self.effect_scratch, &[&source.create_view(&Default::default())], &out.create_view(&Default::default()), &[("to_linear".into(), if to_linear {1.0} else {0.0})], [source.width() as f32,source.height() as f32]);
        out
    }

    pub(crate) fn effective_layer_textures(&mut self, layers: &[LayerWithPasses]) -> Result<EffectiveLayers, CompositorError> {
        self.effective_layer_textures_in_frame(layers, None)
    }

    pub(crate) fn effective_layer_textures_in_frame(&mut self, layers: &[LayerWithPasses], frame: Option<effects::vism::ImageFrame>) -> Result<EffectiveLayers, CompositorError> {
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
        let mut effective_spills: Vec<LayerSpill> = Vec::with_capacity(layers.len());
        let mut checked_out = Vec::new();
        let mut copy_encoder: Option<wgpu::CommandEncoder> = None;
        // 同じ素材に同じ効果列が続く(配置効果の複製)なら、鎖は 1 回だけ流して結果を配る。
        let mut previous: Option<(GpuTexture2D, &[EffectPass], LayerContent, u32, LayerSpill)> = None;

        let shared_frame = frame;
        for lwp in layers {
            // 層が自分の枠を持てば(密度 > 1 の素材)それで評価する。呼び手の枠が優先。
            let frame = shared_frame.or(lwp.layer.frame);
            let Some(layer_texture) = lwp.layer.content.texture().cloned() else {
                effective_textures.push(lwp.layer.content.clone());
                effective_paddings.push(0);
                effective_spills.push(None);
                continue;
            };
            if lwp.passes.is_empty() {
                effective_textures.push(lwp.layer.content.clone());
                effective_paddings.push(0);
                effective_spills.push(None);
                continue;
            }
            if let Some((source, passes, content, padding, spill)) = &previous {
                if shared_frame.is_none() && source.handle() == layer_texture.handle() && *passes == lwp.passes.as_slice() {
                    effective_textures.push(content.clone());
                    effective_paddings.push(*padding);
                    effective_spills.push(spill.clone());
                    continue;
                }
            }
            let baked_key = BakedKey { source: layer_texture.clone(), passes: lwp.passes.clone(), frame };
            if let Some((content, padding, spill)) = self.baked_effects.hit(&baked_key) {
                self.surface_work.baked_hits += 1;
                previous = Some((layer_texture, lwp.passes.as_slice(), content.clone(), padding, spill.clone()));
                effective_textures.push(content);
                effective_paddings.push(padding);
                effective_spills.push(spill);
                continue;
            }
            self.surface_work.bakes += 1;
            let mut owned = Vec::new();
            let [width, height] = layer_texture.width_height();
            let padding = lwp
                .passes
                .iter()
                .map(EffectPass::padding)
                .max()
                .unwrap_or(0);
            let padding = frame.map_or(padding, |f| (padding as f32 * f.density().into_iter().fold(0.0f32, f32::max)).ceil() as u32);
            // 上限を越える texture は wgpu が無効な物を返し、pool に入ると以後の全フレームを壊す。reach を削ってでも収める。
            let limit = self.ctx.device.limits().max_texture_dimension_2d;
            let padding = padding.min(limit.saturating_sub(width.max(height)) / 2);
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
            let mut current_linear = matches!(lwp.layer.content, LayerContent::LinearTexture(_)) || layer_texture.format().is_srgb();
            let mut current_is_scratch = false;
            let encoder = copy_encoder.get_or_insert_with(|| {
                self.ctx
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("motolii-compositor-effective-layer-textures"),
                    })
            });

            if padding > 0 {
                let padded = self.padded_copy(encoder, &current, [width, height], padding);
                current = padded;
                current_is_scratch = true;
            }

            let (next, next_linear, next_is_scratch) = self.record_pass_chain(
                encoder, current, current_linear, current_is_scratch,
                &lwp.passes, frame, [padded_width, padded_height], padding, [width, height],
            )?;
            current = next;
            current_linear = next_linear;
            current_is_scratch = next_is_scratch;

            // 溢れの法: SPILL を宣言した効果があれば、出力を素材の coverage の内と外に分ける。
            // 内は層の Blend、外(光・影)は宣言された混ぜ方で下へ。分け方は 1 箇所、効果は分岐しない。
            let spill_mode = lwp.passes.iter().find_map(|pass| pass.spill);
            let mut spill: LayerSpill = None;
            if let Some(mode) = spill_mode {
                let coverage = self.padded_copy(encoder, &src.texture, [width, height], padding);
                let format = current.format();
                let inside = self.matte_by_coverage(encoder, &current, &coverage, [padded_width, padded_height], format, 0.0)?;
                let outside = self.matte_by_coverage(encoder, &current, &coverage, [padded_width, padded_height], format, 1.0)?;
                self.effect_scratch.release(padded_width, padded_height, coverage.format(), coverage);
                if current_is_scratch { self.effect_scratch.release(padded_width, padded_height, format, current); }
                current = inside;
                current_is_scratch = true;
                self.next_effect_key += 1;
                let imported = self.ctx.texture_manager_2d.import_gpu_premultiplied(self.next_effect_key, &self.ctx, &outside)
                    .map_err(|error| CompositorError::Effect(error.to_string()))?;
                spill = Some((if current_linear { LayerContent::LinearTexture(imported) } else { LayerContent::Texture(imported) }, mode));
                owned.push((padded_width, padded_height, outside.format(), outside));
            }

            self.next_effect_key += 1;
            let imported = self
                .ctx
                .texture_manager_2d
                .import_gpu_premultiplied(self.next_effect_key, &self.ctx, &current)
                .map_err(|error| CompositorError::Effect(error.to_string()))?;
            let content = if current_linear { LayerContent::LinearTexture(imported) } else { LayerContent::Texture(imported) };
            previous = Some((layer_texture.clone(), lwp.passes.as_slice(), content.clone(), padding, spill.clone()));
            effective_textures.push(content.clone());
            effective_paddings.push(padding);
            effective_spills.push(spill.clone());
            owned.push((padded_width, padded_height, current.format(), current));
            self.baked_effects.keep(baked_key, content, padding, spill, owned);
        }
        if let Some(encoder) = copy_encoder {
            self.pending.push(encoder.finish());
        }
        self.flush_pending();
        let Self { baked_effects, effect_scratch, .. } = self;
        baked_effects.sweep(effect_scratch);
        Ok((effective_textures, effective_paddings, effective_spills, checked_out))
    }

    /// 生成器(image 入力なし)は矩形全面を塗るので、直前の絵の alpha の中へ閉じ込める。
    /// 素材の形(切り抜き・文字・図形)を効果が消さないための、効果側に依らない 1 箇所。
    /// matte を生成器と同じ出力 format で組み直して使う(format を跨ぐと次の pass の decode が変わる)。
    fn confine_to_coverage(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        generated: wgpu::Texture,
        coverage: &wgpu::Texture,
        size: [u32; 2],
        format: wgpu::TextureFormat,
    ) -> Result<wgpu::Texture, CompositorError> {
        let confined = self.matte_by_coverage(encoder, &generated, coverage, size, format, 0.0)?;
        self.effect_scratch.release(size[0], size[1], format, generated);
        Ok(confined)
    }

    /// 元の絵を、別の絵の coverage で切る(mode 0 = alpha の内側、1 = 外側)。生成器の閉じ込めと溢れの分離が共有する 1 箇所。
    fn matte_by_coverage(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::Texture,
        coverage: &wgpu::Texture,
        [width, height]: [u32; 2],
        format: wgpu::TextureFormat,
        mode: f32,
    ) -> Result<wgpu::Texture, CompositorError> {
        let confined = self.effect_scratch.acquire(&self.ctx.device, width, height, format);
        let program = match self.coverage_programs.entry(format) {
            std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let matte = self
                    .catalog
                    .definitions
                    .iter()
                    .find(|d| d.source.name == "matte")
                    .ok_or_else(|| CompositorError::Effect("missing validated matte program".into()))?;
                entry.insert(effects::EffectProgram::compile_for(&self.ctx, matte, format))
            }
        };
        program.record_over(
            &self.ctx,
            encoder,
            &mut self.effect_scratch,
            &[&source.create_view(&Default::default()), &coverage.create_view(&Default::default())],
            &confined.create_view(&Default::default()),
            &[("mode".to_owned(), mode)],
            [width as f32, height as f32],
        );
        Ok(confined)
    }

    /// 効果列を 1 枚の texture へ順に流す。焼く経路(層の絵)と、run の経路(絵を持たない素材を
    /// 描いた後の窓)が同じ意味を通るように、ここ 1 箇所だけが効果を順に記録する。
    /// `size` は余白込みの寸法、`unpadded` は余白を除いた素材の寸法(warp の既定の枠が使う)。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_pass_chain(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        mut current: wgpu::Texture,
        mut current_linear: bool,
        mut current_is_scratch: bool,
        passes: &[EffectPass],
        frame: Option<effects::vism::ImageFrame>,
        size: [u32; 2],
        padding: u32,
        unpadded: [u32; 2],
    ) -> Result<(wgpu::Texture, bool, bool), CompositorError> {
        let [padded_width, padded_height] = size;
        let [width, height] = unpadded;
        for pass in passes {
            let is_warp = self.catalog.descriptors.iter().any(|d| d.plugin_id == pass.plugin_id && d.stage == EffectStage::Warp);
            if current_linear != is_warp {
                let converted = self.convert_image_encoding(encoder, &current, is_warp);
                if current_is_scratch { self.effect_scratch.release(padded_width,padded_height,current.format(),current); }
                current = converted;
                current_is_scratch = true;
            }
            current_linear = is_warp;
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
            if is_warp {
                let frame = frame.unwrap_or(effects::vism::ImageFrame { size: [width as f32,height as f32], origin: [0.0;2], pixels: [width,height] }).padded(padding);
                program.record_in_frame(&self.ctx, encoder, &mut self.effect_scratch, &sources, &destination_view, &pass.params, frame);
            } else {
                // pass は ISF の作法(render_size = 画素)。論理 px の欄だけ host が密度で画素へ写す。
                let density = frame.map_or(1.0, |f| f.density().into_iter().fold(1.0f32, f32::max));
                let params = program.params_at_density(&pass.params, density);
                program.record(&self.ctx, encoder, &mut self.effect_scratch, &sources, &destination_view, &params, [padded_width as f32,padded_height as f32]);
            }
            let destination = if program.image_input_count() == 0 {
                self.confine_to_coverage(encoder, destination, &current, [padded_width, padded_height], format)?
            } else {
                destination
            };
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
        Ok((current, current_linear, current_is_scratch))
    }

    /// 元の絵を余白ぶん広げた scratch の中央へ写す(周りは透明)。効果の入力と、溢れを分ける coverage が使う。
    fn padded_copy(&mut self, encoder: &mut wgpu::CommandEncoder, source: &wgpu::Texture, [width, height]: [u32; 2], padding: u32) -> wgpu::Texture {
        let padded = self.effect_scratch.acquire(&self.ctx.device, width + 2 * padding, height + 2 * padding, source.format());
        let padded_view = padded.create_view(&Default::default());
        {
            let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("motolii-compositor-vism-padded-source-clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &padded_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo { texture: source, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::TexelCopyTextureInfo { texture: &padded, mip_level: 0, origin: wgpu::Origin3d { x: padding, y: padding, z: 0 }, aspect: wgpu::TextureAspect::All },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        );
        padded
    }

    pub fn render_with_effects(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layers: &[LayerWithPasses],
        background_color: [f32; 4],
    ) -> Result<Vec<u8>, CompositorError> {
        let (effective_textures, effective_paddings, effective_spills, checked_out) =
            self.effective_layer_textures(layers)?;

        let inputs = sequential_inputs(layers, &effective_textures, &effective_paddings, &effective_spills);

        self.window = crate::render::compositor::Window::output(comp);
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
        let (effective_textures, effective_paddings, effective_spills, checked_out) =
            self.effective_layer_textures(layers)?;

        let inputs = sequential_inputs(layers, &effective_textures, &effective_paddings, &effective_spills);

        self.window = crate::render::compositor::Window::output(comp);
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
    effective_spills: &'a [LayerSpill],
) -> Vec<SequentialInput<'a>> {
    layers
        .iter()
        .zip(effective_textures.iter())
        .zip(effective_paddings.iter())
        .zip(effective_spills.iter())
        .flat_map(|(((lwp, content), &padding), spill)| {
            let layer = &lwp.layer;
            // 余白は絵の画素。置く時は論理 px(密度 > 1 の素材は密度で割る)。
            let density = layer.frame.map_or([1.0, 1.0], |f| f.density());
            let pad = [padding as f32 / density[0].max(1.0), padding as f32 / density[1].max(1.0)];
            let body = SequentialInput {
                content: match content {
                    LayerContent::Texture(t) => SequentialContent::Rect(t),
                    LayerContent::LinearTexture(t) => SequentialContent::LinearRect(t),
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
                    LayerContent::Environment(e) => SequentialContent::Environment(e),
                },
                local_min: glam::Vec2::new(-pad[0], -pad[1]),
                local_size: glam::Vec2::new(layer.size[0] + 2.0 * pad[0], layer.size[1] + 2.0 * pad[1]),
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
                // 焼く先の絵が無かった層(網・点群・環境)は、効果列をここから画面へ持って行く。
                screen_passes: if layer.content.texture().is_none() { lwp.passes.as_slice() } else { &[] },
            };
            // 溢れ: 同じ置き場に、coverage 外の絵だけを宣言された混ぜ方で重ねる(層の Blend と独立)。
            let spilled = spill.as_ref().and_then(|(content, mode)| {
                let texture = match content { LayerContent::Texture(t) => SequentialContent::Rect(t), LayerContent::LinearTexture(t) => SequentialContent::LinearRect(t), _ => return None };
                Some(SequentialInput { content: texture, blend_mode: *mode, shading: Default::default(), displace: Default::default(), blocks_light: false, outline: 0, screen_passes: &[], ..body })
            });
            std::iter::once(body).chain(spilled)
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

    /// Radiance: 明るい所が光源、形が遮蔽。光は空気中に見え(Air)、遮蔽の裏は暗い。
    #[test]
    fn radiance_lights_the_air_around_emitters_and_occluders_cast_shadows() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("scene.png");
        let size = 96u32;
        let mut pixels = Vec::with_capacity((size * size * 4) as usize);
        for y in 0..size {
            for x in 0..size {
                let emitter = (20..36).contains(&x) && (40..56).contains(&y);
                let wall = (50..54).contains(&x) && (20..76).contains(&y);
                pixels.extend_from_slice(&if emitter { [255, 255, 255, 255] } else if wall { [0, 0, 0, 255] } else { [0, 0, 0, 0] });
            }
        }
        image::save_buffer(&source, &pixels, size, size, image::ColorType::Rgba8).unwrap();
        let mut doc = document_with_effects(&source, &["motolii.radiance"]);
        doc.apply(Intent::SetComposition(Composition {
            width: size,
            height: size,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 1,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .unwrap();
        for (name, value) in [("air", 1.0), ("intensity", 4.0), ("radius", 16.0)] {
            doc.apply(Intent::SetConstant {
                layer: LayerId(1),
                property: PropertyId::effect_param(EffectId(0), name).unwrap(),
                value: Value::F64(value),
            })
            .unwrap();
        }
        let mut engine = Engine::new().unwrap();
        let scope = engine.gpu_device().push_error_scope(wgpu::ErrorFilter::Validation);
        let lit = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        let error = pollster::block_on(scope.pop());
        assert!(error.is_none(), "radiance passes must validate: {error:?}");
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        let at = |frame: &[u8], x: u32, y: u32| frame[((y * size + x) * 4) as usize];
        let beside = at(&lit, 42, 48);
        let behind_wall = at(&lit, 60, 48);
        let far_corner = at(&lit, 90, 6);
        assert!(beside > 20, "air next to the emitter must be lit: {beside}");
        assert!(behind_wall < beside / 2, "the wall must shadow the far side: beside {beside}, behind {behind_wall}");
        assert!(far_corner < beside, "light falls off with distance: corner {far_corner}, beside {beside}");

        doc.apply(Intent::SetConstant { layer: LayerId(1), property: PropertyId::effect_param(EffectId(0), "intensity").unwrap(), value: Value::F64(0.0) }).unwrap();
        let dark = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert!(doc.undo());
        let relit = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert_eq!(relit, lit, "GPU reduction must not retain a stale no-emitter result");
        doc.apply(Intent::SetEffects { layer: LayerId(1), effects: Vec::new() }).unwrap();
        let plain = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert_eq!(dark, plain, "without emission no light can be added");
        assert_eq!(at(&plain, 42, 48), 0, "without the effect the air is dark");
    }

    #[test]
    fn radiance_preaverage_preserves_reference_pixels() {
        use crate::render::compositor::effects::{self, VismSource};
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.png");
        let size = 96u32;
        let mut pixels = vec![0; (size * size * 4) as usize];
        for y in 32..48 { for x in 20..36 { let p = ((y*size+x)*4) as usize; pixels[p..p+4].copy_from_slice(&[255,255,255,255]); } }
        image::save_buffer(&source, &pixels, size, size, image::ColorType::Rgba8).unwrap();
        let doc = if let Ok(path) = std::env::var("MOTOLII_RADIANCE_BENCH_DOCUMENT") {
            let mut doc = Document::load(path).unwrap();
            let ids = doc.view().resolved_layers(RationalTime::ZERO).unwrap().iter().map(|l| l.id).collect::<Vec<_>>();
            for layer in ids { for effect in doc.view().effects(layer).unwrap() { doc.apply(Intent::SetConstant { layer, property: PropertyId::effect_enabled(effect.id), value: Value::Bool(true) }).unwrap(); } }
            doc
        } else {
            let mut doc = document_with_effects(&source, &["motolii.radiance"]);
            doc.apply(Intent::SetComposition(Composition { width:size, height:size, fps:Fps::try_new(30,1).unwrap(), duration_frames:150, background:[0.0,0.0,0.0,1.0] })).unwrap();
            doc
        };
        let fps = doc.view().composition().unwrap().unwrap().fps;
        let old = include_str!("../../../../reference/radiance-before-2026-09-12.wgsl");
        let mut reference = Engine::new().unwrap();
        let mut definition = reference.compositor.catalog.definitions.iter().find(|d| d.plugin_id() == "motolii.radiance").unwrap().clone();
        let (manifest, body) = effects::isf::parse_isf_source(old).unwrap();
        definition.source = VismSource { name:"radiance-reference".into(), extension:"wgsl".into(), source:old.into() };
        definition.manifest = manifest;
        definition.vertex_text = body.clone(); definition.fragment_text = body;
        definition.stage().unwrap();
        let program = effects::EffectProgram::compile(&reference.compositor.ctx, &definition);
        reference.compositor.effect_programs.insert("motolii.radiance".into(), program);
        let mut candidate = Engine::new().unwrap();
        let mut times = [0.0f64; 2];
        for frame in [0,30,60,90] {
            let t = RationalTime::try_from_frame(frame, fps).unwrap();
            let expected = reference.render_frame(&doc.view(), t).unwrap();
            let actual = candidate.render_frame(&doc.view(), t).unwrap();
            let bad = actual.iter().zip(&expected).filter(|(a,b)| a.abs_diff(**b)>3).count();
            assert_eq!(bad, 0, "pre-averaging must retain image values, frame {frame}");
            for (i, engine) in [&mut reference, &mut candidate].into_iter().enumerate() {
                let start = std::time::Instant::now();
                for _ in 0..3 { engine.render_frame(&doc.view(), t).unwrap(); }
                times[i] += start.elapsed().as_secs_f64()*1000.0;
            }
        }
        eprintln!("radiance reference {:.3} ms, optimized {:.3} ms (including readback)",times[0]/12.0,times[1]/12.0);
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

/// 絵を持たない素材にも効果列が届く(実 GPU)。
/// 届かない実装(素材の texture が無ければ効果を捨てる)だと、網の外はいつまでも 0 のまま。
#[cfg(test)]
mod passes_reach_every_material {
    use crate::doc::store::{property, Composition, Document, EffectId, EffectInstance, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value};
    use crate::render::engine::Engine;

    const SIZE: u32 = 64;

    fn document(path: &std::path::Path, radius: f64) -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: SIZE, height: SIZE, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
        let layer = LayerId(1);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: path.to_string_lossy().into_owned(), fingerprint: None }, order: 0, timing: LayerTiming::place(0, None, 1) } },
            Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([16.0, 16.0]) },
            Intent::SetConstant { layer, property: PropertyId::new(property::SCALE).unwrap(), value: Value::Vec2([16.0, 16.0]) },
            Intent::SetEffects { layer, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.blur".into() }] },
            Intent::SetConstant { layer, property: PropertyId::effect_param(EffectId(0), "radius").unwrap(), value: Value::F64(radius) },
        ]).unwrap();
        doc
    }

    fn drawn(frame: &[u8], x: u32, y: u32) -> u8 {
        let i = ((y * SIZE + x) * 4) as usize;
        frame[i..i + 3].iter().copied().max().unwrap()
    }

    #[test]
    fn a_blur_on_a_mesh_bleeds_past_its_edge() {
        let dir = tempfile::tempdir().unwrap();
        let obj = dir.path().join("quad.obj");
        // 32×32 を (16,16) に置くので、絵は x,y が 16..48。外側の 8 px は素の網なら真っ黒。
        std::fs::write(&obj, "v -1 -1 0\nv 1 -1 0\nv 1 1 0\nv -1 1 0\nvn 0 0 -1\nf 1//1 2//1 3//1\nf 1//1 3//1 4//1\n").unwrap();
        let mut engine = Engine::new().unwrap();

        let sharp = engine.render_frame(&document(&obj, 0.0).view(), RationalTime::ZERO).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        assert!(drawn(&sharp, 32, 32) > 0, "網は描かれている");
        assert_eq!(drawn(&sharp, 32, 10), 0, "ぼかさなければ縁の外は黒");

        let soft = engine.render_frame(&document(&obj, 12.0).view(), RationalTime::ZERO).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        assert!(drawn(&soft, 32, 10) > 0, "ブラーが網の縁の外へ滲む");
    }
}
