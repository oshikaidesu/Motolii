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
struct BakedKey { source: GpuTexture2D, passes: Vec<EffectPass>, frame: Option<effects::vism::ImageFrame>, others: Vec<GpuTexture2D>, clock: Option<Clock>, feedback_frame: Option<i64> }
impl PartialEq for BakedKey {
    fn eq(&self, other: &Self) -> bool {
        self.source.handle() == other.source.handle() && self.passes == other.passes && self.frame == other.frame
            && self.clock == other.clock && self.feedback_frame == other.feedback_frame
            && self.others.len() == other.others.len()
            && self.others.iter().zip(&other.others).all(|(a, b)| a.handle() == b.handle())
    }
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
    /// 色の規約を写す。出口は乗算済み線形(`to_linear`)か乗算済み sRGB。入口の素性は
    /// `source_encoded`(sRGB 符号化か)と `source_premultiplied` で言う。
    pub(crate) fn convert_image_encoding(&mut self, encoder: &mut wgpu::CommandEncoder, source: &wgpu::Texture, to_linear: bool, source_encoded: bool, source_premultiplied: bool) -> wgpu::Texture {
        const ID: &str = "motolii.material_encoding";
        if !self.effect_programs.contains_key(ID) {
            let definition = self.catalog.definitions.iter().find(|d| d.plugin_id() == ID).expect("material encoding shader");
            self.effect_programs.insert(ID.into(), effects::EffectProgram::compile_for(&self.ctx, definition, wgpu::TextureFormat::Rgba16Float));
        }
        let out = self.effect_scratch.acquire(&self.ctx.device,source.width(),source.height(),wgpu::TextureFormat::Rgba16Float);
        let flag = |b: bool| if b { 1.0 } else { 0.0 };
        self.effect_programs[ID].record(&self.ctx, encoder, &mut self.effect_scratch, &[&source.create_view(&Default::default())], &out.create_view(&Default::default()),
            &[("to_linear".into(), flag(to_linear)), ("source_encoded".into(), flag(source_encoded)), ("source_premultiplied".into(), flag(source_premultiplied))],
            [source.width() as f32,source.height() as f32]);
        out
    }

    pub(crate) fn effective_layer_textures(&mut self, layers: &[LayerWithPasses]) -> Result<EffectiveLayers, CompositorError> {
        self.effective_layer_textures_in_frame(layers, None)
    }

    pub(crate) fn effective_layer_textures_in_frame(&mut self, layers: &[LayerWithPasses], frame: Option<effects::vism::ImageFrame>) -> Result<EffectiveLayers, CompositorError> {
        self.refresh_catalog_programs();
        for pass in layers.iter().flat_map(|layer| &layer.passes) {
            if !self.effect_programs.contains_key(&pass.plugin_id) {
                let definition = self.catalog.definitions.iter().find(|definition| {
                    definition.plugin_id() == pass.plugin_id
                        && (definition.manifest.expose || definition.manifest.stage == effects::IsfStage::Warp)
                        && matches!(definition.manifest.stage, effects::IsfStage::Pass | effects::IsfStage::Warp)
                }).ok_or_else(|| CompositorError::Effect(format!("unknown Vism {}", pass.plugin_id)))?;
                self.effect_programs.insert(pass.plugin_id.clone(), effects::EffectProgram::compile(&self.ctx, definition));
            }
        }
        let mut effective_textures = Vec::with_capacity(layers.len());
        let mut effective_paddings = Vec::with_capacity(layers.len());
        let mut effective_spills: Vec<LayerSpill> = Vec::with_capacity(layers.len());
        let checked_out = Vec::new();
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
            // 下の合成を読む列は、合成の途中でしか値が決まらない。ここでは焼かず、run の窓で流す。
            if lwp.passes.is_empty() || lwp.passes.iter().any(|p| p.reads_backdrop || p.reads_composite()) {
                effective_textures.push(lwp.layer.content.clone());
                effective_paddings.push(lwp.padding);
                effective_spills.push(None);
                continue;
            }
            if let Some((source, passes, content, padding, spill)) = &previous {
                if shared_frame.is_none() && lwp.pass_sources.iter().all(Vec::is_empty) && !lwp.passes.iter().any(|p| p.uses_clock)
                    && source.handle() == layer_texture.handle() && *passes == lwp.passes.as_slice() {
                    effective_textures.push(content.clone());
                    effective_paddings.push(*padding);
                    effective_spills.push(spill.clone());
                    continue;
                }
            }
            // 時計を読む効果が 1 つでもあれば時刻が鍵に入る。読まない列は時刻で焼き直さない。
            let clock = lwp.passes.iter().any(|p| p.uses_clock).then_some(self.clock).flatten();
            // feedback を持つ列はフレーム番号が鍵(同じフレームの描き直しだけ当たる)。
            let feedback_frame = lwp.passes.iter().any(|p| p.persistent).then(|| self.frame_index()).flatten();
            let baked_key = BakedKey { source: layer_texture.clone(), passes: lwp.passes.clone(), frame, others: lwp.pass_sources.iter().flatten().cloned().collect(), clock, feedback_frame };
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
            // 素材の素性: 線形テクスチャと sRGB 形式は乗算済み線形、それ以外は非乗算 sRGB(層の法)。
            let mut current_linear = matches!(lwp.layer.content, LayerContent::LinearTexture(_)) || layer_texture.format().is_srgb();
            let mut current_premultiplied = current_linear;
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

            // 別の時刻の絵を、束ねられる形へ(pool の借りはここで解く)。
            let others: Vec<Vec<wgpu::Texture>> = lwp.pass_sources.iter().map(|row| {
                row.iter().filter_map(|t| self.ctx.gpu_resources.textures.get_from_handle(t.handle()).ok().map(|g| g.texture.clone())).collect()
            }).collect();
            let encoder = copy_encoder.as_mut().expect("直前に用意した");
            let (next, next_linear, next_premultiplied, next_is_scratch) = self.record_pass_chain(
                encoder, current, current_linear, current_premultiplied, current_is_scratch,
                &lwp.passes, &others, frame, [padded_width, padded_height], padding, [width, height],
            )?;
            current = next;
            current_linear = next_linear;
            current_premultiplied = next_premultiplied;
            current_is_scratch = next_is_scratch;
            // 置く時は乗算済み線形(rerun の AlreadyPremultiplied)。列の出口が乗算済み sRGB ならここで戻す。
            if !current_linear {
                let back = self.convert_image_encoding(encoder, &current, true, true, current_premultiplied);
                if current_is_scratch { self.effect_scratch.release(padded_width, padded_height, current.format(), current); }
                current = back;
                current_linear = true;
                current_premultiplied = true;
                current_is_scratch = true;
            }

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
                spill = Some((LayerContent::LinearTexture(imported), mode));
                owned.push((padded_width, padded_height, outside.format(), outside));
            }

            self.next_effect_key += 1;
            let imported = self
                .ctx
                .texture_manager_2d
                .import_gpu_premultiplied(self.next_effect_key, &self.ctx, &current)
                .map_err(|error| CompositorError::Effect(error.to_string()))?;
            let content = LayerContent::LinearTexture(imported);
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
        mut current_premultiplied: bool,
        mut current_is_scratch: bool,
        passes: &[EffectPass],
        // `others` は効果ごとの「別の時刻の絵」(`passes` と同じ並び)。空なら層の絵 1 枚だけ。
        others: &[Vec<wgpu::Texture>],
        frame: Option<effects::vism::ImageFrame>,
        size: [u32; 2],
        padding: u32,
        unpadded: [u32; 2],
    ) -> Result<(wgpu::Texture, bool, bool, bool), CompositorError> {
        let [padded_width, padded_height] = size;
        let [width, height] = unpadded;
        for (index, pass) in passes.iter().enumerate() {
            let is_warp = self.catalog.descriptors.iter().any(|d| d.plugin_id == pass.plugin_id && d.stage == EffectStage::Warp);
            // 効果は**乗算済み線形**で受ける(warp も pass も。累算器と同じ空間)。sRGB のまま平均すると
            // 縁で over が成り立たず沈む。素材(非乗算 sRGB)は最初の効果の前で 1 度だけ写す。
            let _ = is_warp;
            if !current_linear || !current_premultiplied {
                let converted = self.convert_image_encoding(encoder, &current, true, !current_linear, current_premultiplied);
                if current_is_scratch { self.effect_scratch.release(padded_width,padded_height,current.format(),current); }
                current = converted;
                current_is_scratch = true;
            }
            current_linear = true;
            current_premultiplied = true;
            // 2 枚目以降も 1 枚目と同じ空間(乗算済み線形)で渡す。層の絵の写し(非乗算 sRGB)はここで写す。
            // 線形の texture(float)と sRGB 形式は既に乗算済み線形(層の法)。
            let mut converted_others: Vec<wgpu::Texture> = Vec::new();
            let mut other_textures: Vec<wgpu::Texture> = Vec::new();
            for t in others.get(index).map(|row| row.as_slice()).unwrap_or(&[]) {
                let linear = t.format().is_srgb() || matches!(t.format(), wgpu::TextureFormat::Rgba16Float | wgpu::TextureFormat::Rgba32Float);
                if linear { other_textures.push(t.clone()); continue; }
                let converted = self.convert_image_encoding(encoder, t, true, true, false);
                converted_others.push(converted.clone());
                other_textures.push(converted);
            }
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
            let other_views: Vec<wgpu::TextureView> = other_textures.iter().map(|t| t.create_view(&Default::default())).collect();
            let sources: Vec<_> = source_view.iter().chain(other_views.iter()).collect();
            let destination_view = destination.create_view(&Default::default());
            // feedback: 状態の持ち主は host。frame の並びから今フレームの扱いを決める。
            let frame_index = self.frame_index();
            let feedback = pass.feedback.map(|key| {
                let state = self.feedback.entry(key).or_default();
                let step = match (state.frame, frame_index) {
                    (Some(have), Some(now)) if have == now => effects::FeedbackStep::Reuse,
                    (Some(have), Some(now)) if have + 1 == now => effects::FeedbackStep::Advance,
                    _ => effects::FeedbackStep::Restart,
                };
                (key, step)
            });
            let clock_params = self.clock_params();
            let state = feedback.map(|(key, step)| (self.feedback.get_mut(&key).expect("entry"), step));
            if is_warp {
                let frame = frame.unwrap_or(effects::vism::ImageFrame { size: [width as f32,height as f32], origin: [0.0;2], pixels: [width,height] }).padded(padding);
                program.record_feedback_in_frame(&self.ctx, encoder, &mut self.effect_scratch, &sources, &destination_view, &pass.params, frame, state);
            } else {
                // pass は ISF の作法(render_size = 画素)。論理 px の欄だけ host が密度で画素へ写す。
                let density = frame.map_or(1.0, |f| f.density().into_iter().fold(1.0f32, f32::max));
                let mut params = program.params_at_density(&pass.params, density);
                if pass.uses_clock {
                    params.extend(clock_params);
                }
                let render_size = [padded_width as f32, padded_height as f32];
                program.record_feedback_in_frame(&self.ctx, encoder, &mut self.effect_scratch, &sources, &destination_view, &params, effects::vism::ImageFrame { size: render_size, origin: [0.0;2], pixels: [padded_width, padded_height] }, state);
            }
            if let (Some((key, step)), Some(now)) = (feedback, frame_index) {
                let state = self.feedback.get_mut(&key).expect("entry");
                state.frame = Some(now);
                match step { effects::FeedbackStep::Restart => state.fresh = true, effects::FeedbackStep::Advance => state.fresh = false, effects::FeedbackStep::Reuse => {} }
                // K フレームごとに写しを焼く(同じフレームの描き直しでは焼かない)。
                if step != effects::FeedbackStep::Reuse && now.rem_euclid(effects::FEEDBACK_CHECKPOINT_EVERY) == 0 && !state.checkpoints.iter().any(|(f, _)| *f == now) {
                    let copies = state.targets.iter().map(|(name, target)| {
                        let copy = self.ctx.device.create_texture(&wgpu::TextureDescriptor {
                            label: Some("motolii-feedback-checkpoint"), size: target.next.size(), mip_level_count: 1, sample_count: 1,
                            dimension: wgpu::TextureDimension::D2, format: target.next.format(),
                            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST, view_formats: &[],
                        });
                        encoder.copy_texture_to_texture(target.next.as_image_copy(), copy.as_image_copy(), target.next.size());
                        (name.clone(), copy)
                    }).collect();
                    state.checkpoints.push((now, copies));
                    state.checkpoints.sort_by_key(|(f, _)| *f);
                    while state.checkpoints.len() > effects::FEEDBACK_CHECKPOINTS_MAX { state.checkpoints.remove(0); }
                }
            }
            for converted in converted_others {
                self.effect_scratch.release(converted.width(), converted.height(), converted.format(), converted);
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
        Ok((current, current_linear, current_premultiplied, current_is_scratch))
    }

    /// 今のフレーム番号(engine が置いた時計から)。無ければ feedback は毎回初期条件。
    pub(crate) fn frame_index(&self) -> Option<i64> { self.clock.map(|c| c[2].round() as i64) }

    /// feedback の状態が今表しているフレーム。
    pub(crate) fn feedback_frame(&self, key: effects::FeedbackKey) -> Option<i64> { self.feedback.get(&key).and_then(|s| s.frame) }
    /// 今の絵が初期条件から描かれたか(辿り直しの要否)。
    pub(crate) fn feedback_is_fresh(&self, key: effects::FeedbackKey) -> bool { self.feedback.get(&key).is_some_and(|s| s.fresh) }

    /// `before` 以前で最も新しい checkpoint を今の状態へ戻し、そのフレームを返す。無ければ None(入点から)。
    pub(crate) fn feedback_restore(&mut self, key: effects::FeedbackKey, before: i64) -> Option<i64> {
        let state = self.feedback.get_mut(&key)?;
        let (at, copies) = state.checkpoints.iter().filter(|(f, _)| *f <= before).max_by_key(|(f, _)| *f)?;
        let mut encoder = self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("motolii-feedback-restore") });
        for (name, copy) in copies {
            let Some(target) = state.targets.get(name) else { return None };
            encoder.copy_texture_to_texture(copy.as_image_copy(), target.next.as_image_copy(), copy.size());
        }
        state.frame = Some(*at);
        let at = *at;
        self.pending.push(encoder.finish());
        Some(at)
    }

    /// 書類の指紋が変われば feedback の状態を全部捨てる(同じ時刻は同じ絵: 履歴は書類の関数)。
    pub(crate) fn feedback_set_revision(&mut self, revision: u64) {
        if self.feedback_revision != revision {
            self.feedback_revision = revision;
            self.feedback.clear();
        }
    }

    /// 欄の列に足す時計(TIME 秒・TIMEDELTA 秒・FRAMEINDEX)。engine が frame ごとに `clock` を置く。
    fn clock_params(&self) -> Vec<(String, f32)> {
        let [time, delta, frame] = self.clock.unwrap_or([0.0; 3]);
        effects::vism::CLOCK_KEYS.iter().zip([time, delta, frame]).map(|(k, v)| ((*k).to_owned(), v)).collect()
    }

    /// 別の時刻の絵を 1 枚だけ写し取る。
    ///
    /// 素材の texture は時刻ごとに**同じ 1 枚へ上書き**されるので(動画は path ごとに 1 枚)、
    /// 写さずに持つと、後の復号で中身が入れ替わる。写した物だけが「あの時刻の絵」でいられる。
    pub(crate) fn snapshot_texture(&mut self, source: &GpuTexture2D) -> Option<GpuTexture2D> {
        let size;
        let copy;
        {
            let src = self.ctx.gpu_resources.textures.get_from_handle(source.handle()).ok()?;
            if !src.texture.usage().contains(wgpu::TextureUsages::COPY_SRC) {
                return None;
            }
            size = src.texture.size();
            copy = self.ctx.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("motolii-other-time"),
                size,
                mip_level_count: src.texture.mip_level_count(),
                sample_count: src.texture.sample_count(),
                dimension: wgpu::TextureDimension::D2,
                format: src.texture.format(),
                usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let mut encoder = self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("motolii-other-time-copy"),
            });
            encoder.copy_texture_to_texture(src.texture.as_image_copy(), copy.as_image_copy(), size);
            // ここで submit しない。復号したコマの転送は frame 共通の encoder に積まれていて、
            // 流れるのは `before_submit` の中 — 先に打つと、まだ届いていない texture を写す
            // (冷えていれば零、暖まっていれば前のコマ。同じ時刻の絵が辿り方で変わる)。
            // pending に積めば `flush_pending` が `before_submit` → この写し、の順で流す。
            self.pending.push(encoder.finish());
        }
        self.next_effect_key += 1;
        self.ctx
            .texture_manager_2d
            .import_gpu_premultiplied(self.next_effect_key, &self.ctx, &copy)
            .ok()
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
                        sizes,
                        sprites,
                        links,
                    } => SequentialContent::Cloud {
                        positions,
                        colors,
                        bounds: *bounds,
                        point_size: *point_size,
                        sizes: sizes.as_ref().map(|s| s.as_slice()),
                        sprites: *sprites,
                        links: links.as_deref(),
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
                shadow: layer.shadow,
                outline: layer.outline,
                // 焼く先の絵が無かった層(網・点群・環境)は、効果列をここから画面へ持って行く。
                // 焼く先の絵が無い層(網・点群・環境)と、下の合成を読む効果列は、画面へ持って行く。
                screen_passes: if layer.content.texture().is_none() || lwp.passes.iter().any(|p| p.reads_backdrop || p.reads_composite()) { lwp.passes.as_slice() } else { &[] },
                screen_sources: lwp.pass_sources.as_slice(),
            };
            // 溢れ: 同じ置き場に、coverage 外の絵だけを宣言された混ぜ方で重ねる(層の Blend と独立)。
            let spilled = spill.as_ref().and_then(|(content, mode)| {
                let texture = match content { LayerContent::Texture(t) => SequentialContent::Rect(t), LayerContent::LinearTexture(t) => SequentialContent::LinearRect(t), _ => return None };
                Some(SequentialInput { content: texture, blend_mode: *mode, shading: Default::default(), displace: Default::default(), shadow: 0.0, outline: 0, screen_passes: &[], screen_sources: &[], ..body })
            });
            std::iter::once(body).chain(spilled)
        })
        .collect()
}

#[cfg(test)]
mod tests;

/// 絵を持たない素材にも効果列が届く(実 GPU)。
/// 届かない実装(素材の texture が無ければ効果を捨てる)だと、網の外はいつまでも 0 のまま。
#[cfg(test)]
mod passes_reach_every_material;

/// 下の合成を読む効果(BACKDROP_INPUT)。「背景のコピー」で自分の絵が下の合成になり、続く効果は
/// その絵に掛かる(実 GPU)。読めていなければ自分の絵(赤)が残るか、下(青)と同じままになる。
#[cfg(test)]
mod passes_can_read_what_is_beneath;

/// 層を指す欄(LAYER)。Set Matte が、利用者が選んだ層の α で自分を切る(実 GPU)。
#[cfg(test)]
mod passes_can_read_a_picked_layer;

/// 機械学習の代わりの静的な 3 本(Depth Map / Kuwahara / XDoG)が、それぞれの手掛かりどおりに振る舞う(実 GPU)。
#[cfg(test)]
mod stylize_without_machine_learning;
