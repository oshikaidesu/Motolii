use std::collections::BTreeMap;
use std::collections::{HashMap, HashSet};

use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::doc::store::{
    LayerId, LayerSource, RationalTime, ResolvedLayer, ResolvedMask, ShapeNode, StoreView,
    TextDocument,
};
use crate::render::compositor::{
    BlendMode as CompositeBlendMode, EffectPass, Layer, LayerContent, LayerWithPasses, Window,
};

use crate::render::engine::translate::{
    translate_blend_mode, translate_clip, translate_effect_passes, translate_matte_mode, translate_point_displace,
};
use crate::render::engine::{Engine, EngineError};

impl Engine {
    pub fn render_with_camera_override(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
        include_background: bool,
        camera_override: Option<ResolvedCamera>,
    ) -> Result<Vec<u8>, EngineError> {
        let frame_start = std::time::Instant::now();
        self.compositor.measurement = Default::default();
        self.layer_failures.clear();
        self.purge_idle_video_players();
        let composition = view
            .composition()
            .map_err(|e| EngineError::Store(e.to_string()))?
            .ok_or(EngineError::NoComposition)?;
        let comp = composition.spec();
        let resolved = view
            .resolved_layers(t)
            .map_err(|e| EngineError::Store(e.to_string()))?;
        let camera = match camera_override {
            Some(camera) => camera,
            None => self.resolve_camera_in(view, &resolved, t)?,
        };

        let text_documents = collect_text_documents(view, &resolved, t)?;
        let shape_documents = collect_shape_documents(view, &resolved, t)?;
        self.compositor.measurement.resolve_us = frame_start.elapsed().as_micros() as u64;
        let layer_start = std::time::Instant::now();
        let layers = self.layers_from_resolved(
            view,
            comp,
            camera,
            self.resolve_camera_in(view, &resolved, t)?,
            t,
            &resolved,
            &text_documents,
            &shape_documents,
        )?;

        self.compositor.measurement.layer_build_us = layer_start.elapsed().as_micros() as u64;
        let background_color = if include_background {
            composition.background
        } else {
            crate::render::compositor::NO_BACKGROUND
        };
        let pixels = self.compositor.render_with_effects(comp, camera, &layers, background_color)?;
        let m = &mut self.compositor.measurement;
        m.total_us = frame_start.elapsed().as_micros() as u64;
        m.prepare_us = m.total_us.saturating_sub(m.submit_us + m.wait_us + m.readback_us);
        Ok(pixels)
    }

    /// 効果が宣言した時刻のずれごとに、**その時刻の層の絵**を用意する。
    ///
    /// 効果が自分で前フレームを覚えるのは恒久禁止(`docs/plugin-resources.md` §6) — 追跡できなくなり、
    /// 純関数契約・フレーム並列・スクラブが壊れるため。ここは逆で、ホストが時刻を決めて渡すので
    /// `render_frame(t)` は純関数のまま。
    ///
    /// 「時刻 t の層の姿」を作るのは Document の resolve 1 箇所だけ。ここでは引き直した姿を使う
    /// (`source_time` だけを手でずらすと、mask やキーフレームは t のままの継ぎ接ぎになる)。
    #[allow(clippy::too_many_arguments)]
    fn sources_at_other_times(
        &mut self,
        passes: &[EffectPass],
        layer: LayerId,
        others: &OtherTimes,
        now: (&[ResolvedLayer], &HashMap<LayerId, TextDocument>, &HashMap<LayerId, Vec<ShapeNode>>, RationalTime),
        comp: CompSpec,
        camera: ResolvedCamera,
        projection_camera: ResolvedCamera,
    ) -> Vec<Vec<crate::render::compositor::GpuTexture2D>> {
        if passes.iter().all(|pass| pass.image_time_offsets().is_empty() && pass.image_layers().is_empty()) {
            return Vec::new();
        }
        passes
            .iter()
            .map(|pass| {
                // 指した層の絵(同じ時刻)。自分自身と無い層は断る — 黙って今の絵で代用しない。
                let (resolved, texts, shapes, t) = now;
                let picked: Option<Vec<_>> = pass.image_layers().iter().map(|&target| {
                    let got = (|| {
                        if target == layer { return None; }
                        let mut then = resolved.iter().find(|l| l.id == target)?.clone();
                        then.id = lookbehind_layer_id(target, 0.0);
                        let (content, _, _) = self.texture_for_resolved(&then, texts, shapes, t, comp, camera, projection_camera).ok()?;
                        content.as_ref().and_then(|c| c.texture()).and_then(|t| self.compositor.snapshot_texture(t))
                    })();
                    if got.is_none() {
                        self.layer_failures.push(format!("指した層 {} の絵が無い(自分自身か、無い層)", target.0));
                    }
                    got
                }).collect();
                if !pass.image_layers().is_empty() {
                    return picked.unwrap_or_default();
                }
                pass.image_time_offsets()
                    .iter()
                    .map(|offset| {
                        let got = (|| {
                        let (at, resolved, texts, shapes) = others.get(&offset_key(*offset))?;
                        let mut then = resolved.iter().find(|l| l.id == layer)?.clone();
                        // 別の流れとして読む。復号器の texture は層ごとに 1 本なので、同じ層として
                        // 読むと今の時刻の絵まで巻き添えで上書きされる(差が 0 になる)。
                        then.id = lookbehind_layer_id(layer, *offset);
                        let (content, _, _) = self
                            .texture_for_resolved(&then, texts, shapes, *at, comp, camera, projection_camera)
                            .ok()?;
                        // 素材の texture は時刻ごとに同じ 1 枚へ上書きされるので、使う分を写しておく。
                        content.as_ref().and_then(|c| c.texture()).and_then(|t| self.compositor.snapshot_texture(t))
                        })();
                        if got.is_none() {
                            // 届かなかった物を黙って「今の絵」で代用しない — 同じ時刻が辿り方で
                            // 変わる原因になる(それは時間参照を入れた意味を消す)。
                            self.layer_failures.push(format!("{offset} 秒前の絵が間に合わなかった"));
                        }
                        got
                    })
                    .collect::<Option<Vec<_>>>()
                    .unwrap_or_default()
            })
            .collect()
    }

    /// 時計(TIME 系)は comp の時刻と fps から。壁時計は使わない — 同じ時刻は何度描いても同じ絵。
    fn stamp_clock(&mut self, view: &StoreView<'_>, t: RationalTime) {
        self.compositor.clock = view.composition().ok().flatten().map(|c| {
            let fps = c.fps;
            let frame = t.try_to_frame_round(fps).unwrap_or(0) as f32;
            [t.as_seconds_f64() as f32, fps.den() as f32 / fps.num() as f32, frame]
        });
    }

    /// feedback は「入点を初期条件とする漸化式」(`docs/plugin-resources.md` §6-3)。状態が t−1 を
    /// 表していなければ、直近の checkpoint(無ければ入点)から t の手前まで、その層だけを順に描く。
    /// 順再生と書き出しは 1 歩ずつなのでここは何もしない。スクラブは最大 checkpoint 間隔ぶんの歩数。
    #[allow(clippy::too_many_arguments)]
    fn replay_feedback(
        &mut self,
        view: &StoreView<'_>,
        comp: CompSpec,
        camera: ResolvedCamera,
        projection_camera: ResolvedCamera,
        t: RationalTime,
        resolved: &[ResolvedLayer],
    ) -> Result<(), EngineError> {
        let Some(composition) = view.composition().ok().flatten() else { return Ok(()) };
        let fps = composition.fps;
        let Ok(now) = t.try_to_frame_round(fps) else { return Ok(()) };
        // 履歴は書類の関数: 書類が変われば捨てて入点から。
        self.compositor.feedback_set_revision(view.revision_key());
        let mut start: Option<i64> = None;
        let mut ids: HashSet<LayerId> = HashSet::new();
        for layer in resolved {
            let mut effects = super::translate::translate_effect_passes(&layer.effects);
            super::translate::stamp_feedback(&mut effects, layer.id, layer.copy, 0);
            let mut after = super::translate::translate_plate_passes(&layer.after_effects);
            super::translate::stamp_feedback(&mut after, layer.id, layer.copy, 1);
            let keys: Vec<_> = effects.iter().chain(&after).filter_map(|p| p.feedback).collect();
            if keys.is_empty() { continue; }
            let in_point = view.meta(layer.id).ok().flatten().map_or(0, |m| m.timing.start);
            for key in keys {
                if matches!(self.compositor.feedback_frame(key), Some(have) if have == now || have + 1 == now) { continue; }
                let from = self.compositor.feedback_restore(key, now - 1).map_or(in_point, |c| c + 1);
                if from >= now { continue; }
                ids.insert(layer.id);
                start = Some(start.map_or(from, |s: i64| s.min(from)));
            }
        }
        let Some(start) = start else { return Ok(()) };
        self.feedback_replaying = true;
        let result = (|| {
            for frame in start..now {
                let at = RationalTime::try_from_frame(frame, fps).map_err(|e| EngineError::Store(e.to_string()))?;
                let then: Vec<ResolvedLayer> = view.resolved_layers(at).map_err(|e| EngineError::Store(e.to_string()))?
                    .into_iter().filter(|l| ids.contains(&l.id) || l.plate.is_some_and(|g| ids.contains(&g))).collect();
                let texts = collect_text_documents(view, &then, at)?;
                let shapes = collect_shape_documents(view, &then, at)?;
                let layers = self.layers_from_resolved(view, comp, camera, projection_camera, at, &then, &texts, &shapes)?;
                self.compositor.effective_layer_textures(&layers)?;
            }
            Ok(())
        })();
        self.feedback_replaying = false;
        result
    }

    fn layers_from_resolved(
        &mut self,
        view: &StoreView<'_>,
        comp: CompSpec,
        camera: ResolvedCamera,
        projection_camera: ResolvedCamera,
        t: RationalTime,
        resolved: &[ResolvedLayer],
        text_documents: &HashMap<LayerId, TextDocument>,
        shape_documents: &HashMap<LayerId, Vec<ShapeNode>>,
    ) -> Result<Vec<LayerWithPasses>, EngineError> {
        self.layer_failures.clear();
        self.drawn_layers = 0;
        self.compositor.refresh_catalog_programs();
        let needs_auxiliary_views = {
            let surface_ids: HashSet<_> = self.compositor.catalog.definitions.iter()
                .filter(|d| d.manifest.stage == crate::render::compositor::IsfStage::Surface)
                .map(|d| d.plugin_id()).collect();
            resolved.iter().flat_map(|l| l.effects.iter().chain(&l.after_effects))
                .any(|e| surface_ids.contains(e.plugin_id.as_str()))
        };
        self.stamp_clock(view, t);
        // feedback(前のフレームを保つ効果)の状態が t−1 に無ければ、入点か直近の checkpoint から t の手前まで辿り直す。
        if !self.feedback_replaying {
            self.replay_feedback(view, comp, camera, projection_camera, t, resolved)?;
            self.stamp_clock(view, t);
        }
        // 別の時刻を要求した効果があれば、その時刻の層の姿をここで 1 回だけ引き直す(同じずれは共有)。
        let mut other_times: OtherTimes = BTreeMap::new();
        for layer in resolved {
            for pass in super::translate::translate_effect_passes(&layer.effects) {
                for offset in pass.image_time_offsets() {
                    let key = offset_key(*offset);
                    if other_times.contains_key(&key) {
                        continue;
                    }
                    let at = shifted_by_seconds(t, *offset);
                    let Ok(then) = view.resolved_layers(at) else { continue };
                    let (Ok(texts), Ok(shapes)) = (collect_text_documents(view, &then, at), collect_shape_documents(view, &then, at)) else { continue };
                    other_times.insert(key, (at, then, texts, shapes));
                }
            }
        }
        let mut layers: Vec<LayerWithPasses> = Vec::with_capacity(resolved.len() + 1);
        // 層 id → layers の添字(通り抜けの配置なら複製の数だけ)。クリップの下地探しに使う。
        let mut contributions: HashMap<LayerId, Vec<usize>> = HashMap::new();

        let by_id: HashMap<LayerId, &ResolvedLayer> =
            resolved.iter().map(|layer| (layer.id, layer)).collect();
        if !self.feedback_replaying {
            self.materials.retain(|id, _| by_id.contains_key(id));
        }
        let matte_sources: HashSet<LayerId> = resolved
            .iter()
            .filter(|layer| !layer.clip_to_below)
            .filter_map(|layer| layer.matte.map(|matte| matte.layer))
            .collect();

        let mut skip_below = 0;
        let mut previous_build: Option<(LayerId, i64, Layer)> = None;
        // 配置効果の複製が何枚あるか。clip の相手を「同じ番号の複製」か「複製の和」かで選ぶのに使う。
        let mut copies_of: HashMap<LayerId, usize> = HashMap::new();
        for layer in resolved {
            *copies_of.entry(layer.id).or_default() += 1;
        }
        // 見えない層は復号も描画もしない: 画面の外か、上の不透明な層に丸ごと覆われた層。
        let unseen = self.unseen_layers(comp, camera, resolved, needs_auxiliary_views, &matte_sources);
        let mut entry_copy: Vec<u32> = Vec::with_capacity(resolved.len() + 1);
        let mut removed: HashSet<usize> = HashSet::new();
        // グループの板に焼き込み済みの層。
        let mut plated: HashSet<usize> = HashSet::new();
        for (index, layer) in resolved.iter().enumerate() {
            if index < skip_below
                || plated.contains(&index)
                || matte_sources.contains(&layer.id)
                || (layer.clip_to_below && layer.matte.is_none())
                || layer.placement.opacity <= 0.0
                || unseen.contains(&index)
            {
                continue;
            }

            let blend_mode = translate_blend_mode(layer.blend_mode)?;
            let (built, passes) = if let Some(group) = layer.plate {
                // Whole の効果を積んだグループ: 同じ板の子孫を全部 1 枚に焼いてから、板の効果を掛ける。
                // 板の不透明度と混ぜ方はグループの物。効果はどちらの道でも「1 枚に掛かる」だけ(裁定 2026-09-11)。
                let members: Vec<usize> = (index..resolved.len())
                    .filter(|&j| resolved[j].plate == Some(group) && resolved[j].ghost == layer.ghost && !unseen.contains(&j))
                    .filter(|&j| !matte_sources.contains(&resolved[j].id) && !resolved[j].clip_to_below && resolved[j].placement.opacity > 0.0)
                    .collect();
                plated.extend(&members);
                let mut copies = Vec::new();
                for &j in &members {
                    let member = &resolved[j];
                    if let Some(built) = self.build_layer_shared(&mut previous_build, member, text_documents, shape_documents, t, comp, camera, projection_camera, translate_blend_mode(member.blend_mode)?)? {
                        let mut passes = translate_effect_passes(&member.effects);
                        super::translate::stamp_feedback(&mut passes, member.id, member.copy, 0);
                        copies.push(LayerWithPasses { layer: built, passes, pass_sources: Vec::new() });
                    }
                }
                if copies.is_empty() {
                    continue;
                }
                let owner = by_id.get(&group).copied();
                let plate_blend = translate_blend_mode(owner.map_or(layer.blend_mode, |g| g.blend_mode))?;
                let mut plate = self.bake_isolated_layers(comp, camera, copies, plate_blend, layer.placement)?;
                plate.placement.opacity = owner.map_or(1.0, |g| g.placement.opacity);
                let mut after = super::translate::translate_plate_passes(&layer.after_effects);
                super::translate::stamp_feedback(&mut after, layer.id, layer.copy, 1);
                (plate, after)
            } else if layer.after_effects.is_empty() {
                let Some(built) = self.build_layer_shared(&mut previous_build, layer, text_documents, shape_documents, t, comp, camera, projection_camera, blend_mode)? else {
                    continue;
                };
                let mut passes = translate_effect_passes(&layer.effects);
                super::translate::stamp_feedback(&mut passes, layer.id, layer.copy, 0);
                // 補助viewが無いときだけ主カメラでカリングする。反射・matte・clipの入力は残す。
                if !needs_auxiliary_views && layer.matte.is_none() && !layer.clip_to_below && offscreen(comp, camera, &built, &passes) {
                    continue;
                }
                (built, passes)
            } else {
                // 配置効果の下に効果が積まれた層: 同じ層の配置を全部 1 枚に合わせてから残りを掛ける。
                let end = index + resolved[index..].iter().take_while(|copy| copy.id == layer.id).count();
                skip_below = end;
                let mut copies = Vec::new();
                for copy in &resolved[index..end] {
                    if let Some(built) = self.build_layer_shared(&mut previous_build, copy, text_documents, shape_documents, t, comp, camera, projection_camera, blend_mode)? {
                        let mut passes = translate_effect_passes(&copy.effects);
                        super::translate::stamp_feedback(&mut passes, copy.id, copy.copy, 0);
                        copies.push(LayerWithPasses { layer: built, passes, pass_sources: Vec::new() });
                    }
                }
                if copies.is_empty() {
                    continue;
                }
                let plate = self.bake_isolated_layers(comp, camera, copies, blend_mode, layer.placement)?;
                let mut after = super::translate::translate_plate_passes(&layer.after_effects);
                super::translate::stamp_feedback(&mut after, layer.id, layer.copy, 1);
                (plate, after)
            };

            if layer.clip_to_below {
                let Some(indices) = layer.matte.and_then(|matte| contributions.get(&matte.layer).cloned()) else {
                    continue;
                };
                let indices: Vec<usize> = indices.into_iter().filter(|i| !removed.contains(i)).collect();
                let no_texture = |engine: &mut Engine| engine.layer_failures.push(format!(
                    "layer {} clips to a base without a texture (point cloud / model bases are not clippable)",
                    layer.id.0
                ));
                if copies_of.get(&layer.id).copied().unwrap_or(1) > 1 || indices.len() <= 1 {
                    // 複製の中(同じグループが増やされた)か、下地が 1 枚: 同じ番号の複製にだけ切る。
                    for index in indices.into_iter().filter(|&i| entry_copy[i] == layer.copy) {
                        let base = layers[index].clone();
                        match self.clip_onto_base(base, &built, &passes)? {
                            Some(clipped) => layers[index] = clipped,
                            None => no_texture(self),
                        }
                    }
                    continue;
                }
                // 外から、増やされた下地に切る: 複製の和を 1 枚にして 1 回だけ切る(画面上の重なり、二重に描かない)。
                let first = indices[0];
                let (blend, placement) = (layers[first].layer.blend_mode, layers[first].layer.placement);
                let union = self.bake_isolated_layers(comp, camera, indices.iter().map(|&i| layers[i].clone()).collect(), blend, placement)?;
                match self.clip_onto_base(LayerWithPasses { layer: union, passes: Vec::new(), pass_sources: Vec::new() }, &built, &passes)? {
                    Some(clipped) => {
                        layers[first] = clipped;
                        removed.extend(indices.into_iter().skip(1));
                    }
                    None => no_texture(self),
                }
                continue;
            }

            let (final_layer, passes) = match layer.matte {
                None => (built, passes),
                Some(matte) => {
                    let target = self.apply_effects_before_matte(comp, camera, built, &passes)?;
                    let Some(source) = by_id.get(&matte.layer).copied() else {
                        continue;
                    };
                    let source_blend = translate_blend_mode(source.blend_mode)?;
                    let Some(mut source_layer) = self.build_layer(source, text_documents, shape_documents, t, comp, camera, projection_camera, source_blend)? else { continue; };
                    source_layer.outline = 0;
                    let source_passes = translate_effect_passes(&source.effects);
                    let source_layer = self.apply_effects_before_matte(
                        comp,
                        camera,
                        source_layer,
                        &source_passes,
                    )?;
                    (
                        self.apply_matte(comp, camera, &target, &source_layer, matte.mode)?,
                        Vec::new(),
                    )
                }
            };

            self.drawn_layers += 1;
            contributions.entry(layer.id).or_default().push(layers.len());
            entry_copy.push(layer.copy);
            // 別の時刻の絵を要求した効果へ、ホストがその時刻の層の絵を渡す(効果は覚えない)。
            // 自分の絵も先に写す — 別時刻の復号が同じ player texture を書き換えるので、
            // 写さないと「今」と「前」が同じ絵になる。
            let mut final_layer = final_layer;
            if passes.iter().any(|pass| !pass.image_time_offsets().is_empty() || !pass.image_layers().is_empty()) {
                final_layer.content = match &final_layer.content {
                    LayerContent::Texture(t) => self.compositor.snapshot_texture(t).map(LayerContent::Texture),
                    LayerContent::LinearTexture(t) => self.compositor.snapshot_texture(t).map(LayerContent::LinearTexture),
                    _ => None,
                }
                .unwrap_or(final_layer.content);
            }
            let pass_sources = self.sources_at_other_times(&passes, layer.id, &other_times, (resolved, text_documents, shape_documents, t), comp, camera, projection_camera);
            layers.push(LayerWithPasses {
                pass_sources,
                layer: final_layer,
                passes,
            });
        }

        if removed.is_empty() {
            return Ok(layers);
        }
        self.drawn_layers -= removed.len();
        Ok(layers.into_iter().enumerate().filter(|(i, _)| !removed.contains(i)).map(|(_, l)| l).collect())
    }

    pub fn render_resolved_to_texture(
        &mut self,
        view: &StoreView<'_>,
        comp: CompSpec,
        background: [f32; 4],
        camera: ResolvedCamera,
        t: RationalTime,
        resolved: &[ResolvedLayer],
        text_documents: &HashMap<LayerId, TextDocument>,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), EngineError> {
        self.render_resolved_to_texture_with_shapes(
            view,
            comp,
            background,
            camera,
            t,
            resolved,
            text_documents,
            &HashMap::new(),
        )
    }

    pub fn render_resolved_to_texture_with_shapes(
        &mut self,
        view: &StoreView<'_>,
        comp: CompSpec,
        background: [f32; 4],
        camera: ResolvedCamera,
        t: RationalTime,
        resolved: &[ResolvedLayer],
        text_documents: &HashMap<LayerId, TextDocument>,
        shape_documents: &HashMap<LayerId, Vec<ShapeNode>>,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), EngineError> {
        let layers =
            self.layers_from_resolved(view, comp, camera, camera, t, resolved, text_documents, shape_documents)?;
        Ok(self
            .compositor
            .render_to_texture(comp, camera, &layers, background)?)
    }

    pub fn render_frame_to_texture(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), EngineError> {
        let composition = view
            .composition()
            .map_err(|e| EngineError::Store(e.to_string()))?
            .ok_or(EngineError::NoComposition)?;
        let comp = composition.spec();
        let resolved = view
            .resolved_layers(t)
            .map_err(|e| EngineError::Store(e.to_string()))?;
        let camera = self.resolve_camera_in(view, &resolved, t)?;
        let text_documents = collect_text_documents(view, &resolved, t)?;
        let shape_documents = collect_shape_documents(view, &resolved, t)?;
        self.render_resolved_to_texture_with_shapes(
            view,
            comp,
            composition.background,
            camera,
            t,
            &resolved,
            &text_documents,
            &shape_documents,
        )
    }

    pub fn render_frame_into(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
        target: &wgpu::Texture,
    ) -> Result<(), EngineError> {
        let composition = view
            .composition()
            .map_err(|e| EngineError::Store(e.to_string()))?
            .ok_or(EngineError::NoComposition)?;
        let comp = composition.spec();
        let resolved = view
            .resolved_layers(t)
            .map_err(|e| EngineError::Store(e.to_string()))?;
        let camera = self.resolve_camera_in(view, &resolved, t)?;
        let text_documents = collect_text_documents(view, &resolved, t)?;
        let shape_documents = collect_shape_documents(view, &resolved, t)?;
        let layers = self.layers_from_resolved(
            view,
            comp,
            camera,
            self.resolve_camera_in(view, &resolved, t)?,
            t,
            &resolved,
            &text_documents,
            &shape_documents,
        )?;
        Ok(self
            .compositor
            .render_into(target, comp, camera, &layers, composition.background)?)
    }

    /// Render from an observation camera while retaining authored layer projection.
    pub fn render_frame_into_with_camera(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
        target: &wgpu::Texture,
        camera: ResolvedCamera,
        include_background: bool,
        outline: &[LayerId],
    ) -> Result<(), EngineError> {
        let comp = view.composition().map_err(|e| EngineError::Store(e.to_string()))?.ok_or(EngineError::NoComposition)?.spec();
        self.render_frame_into_window(view, t, target, camera, include_background, outline, Window::output(comp))
    }

    /// 同じ世界を、出力寸法以外の窓へ(Stage のタブ: 寸法と関心域は窓が言う)。
    pub fn render_frame_into_window(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
        target: &wgpu::Texture,
        camera: ResolvedCamera,
        include_background: bool,
        outline: &[LayerId],
        window: Window,
    ) -> Result<(), EngineError> {
        self.outline_layers = outline.iter().copied().take(255).collect();
        self.outline_order = self.outline_layers.clone();
        let composition = view
            .composition()
            .map_err(|e| EngineError::Store(e.to_string()))?
            .ok_or(EngineError::NoComposition)?;
        let comp = composition.spec();
        let resolved = view
            .resolved_layers(t)
            .map_err(|e| EngineError::Store(e.to_string()))?;
        let text_documents = collect_text_documents(view, &resolved, t)?;
        let shape_documents = collect_shape_documents(view, &resolved, t)?;
        let document_camera = self.resolve_camera_in(view, &resolved, t)?;
        let projection_camera = window.projection_camera.unwrap_or(document_camera);
        let mut layers = self.layers_from_resolved(
            view,
            comp,
            camera,
            projection_camera,
            t,
            &resolved,
            &text_documents,
            &shape_documents,
        )?;
        // 2D は出力の画面の物: どの窓でも作中カメラの箱に貼り付き、箱と一緒に動く(Boxcam)。
        // 2.5D と 3D は世界に居るので、窓の投影基準(Stage は既定)のまま。
        for layer in &mut layers {
            if layer.layer.projection == crate::doc::store::LayerProjection::TwoD {
                layer.layer.projection_camera = document_camera;
            }
        }
        let background_color = if include_background {
            composition.background
        } else {
            crate::render::compositor::NO_BACKGROUND
        };
        let drawn = self.compositor.render_into_window(target, comp, camera, &layers, background_color, window);
        self.outline_layers.clear();
        Ok(drawn?)
    }

    /// 平面へ収める。3D の素材を comp の絵へ一度焼き、以後は板として扱う
    /// (裁定 2026-08-30「平面に収めるのは選択肢」)。焼いた層にも blend・matte・
    /// エフェクトは今まで通り効く。
    fn flatten_if_asked(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layer: Layer,
        flatten: bool,
    ) -> Result<Layer, EngineError> {
        if !flatten || layer.content.texture().is_some() {
            return Ok(layer);
        }
        let mut baked_placement = layer.placement;
        baked_placement.opacity = 1.0;
        let source = LayerWithPasses {
            pass_sources: Vec::new(),
            layer: Layer {
                placement: baked_placement,
                ..layer.clone()
            },
            passes: Vec::new(),
        };
        let (texture, _view) = self.compositor.render_to_texture(
            comp,
            camera,
            std::slice::from_ref(&source),
            crate::render::compositor::NO_BACKGROUND,
        )?;
        let imported = self.compositor.import_premultiplied(&texture)?;
        Ok(Layer {
            content: crate::render::compositor::LayerContent::Texture(imported),
            size: [comp.width as f32, comp.height as f32],
            // 焼いた絵は既に comp の座標に居るので、もう一度動かさない。
            placement: crate::doc::core::LayerPlacement {
                transform: glam::Affine2::IDENTITY,
                world_transform: None,
                z: 0.0,
                rotation_x: 0.0,
                rotation_y: 0.0,
                ..layer.placement
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

    /// Track Matte は、Effect後の層をsourceのcoverageで切り、その結果を他層へblendする。
    /// EffectをMatte後のcomp大textureへ掛けると、0-input Effectが透明域を再び塗るため、
    /// 既存のlocal-texture Effect経路をここで一度だけcomp座標へ収めてからMatteへ渡す。
    fn apply_effects_before_matte(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layer: Layer,
        passes: &[EffectPass],
    ) -> Result<Layer, EngineError> {
        if passes.is_empty() {
            return Ok(layer);
        }

        self.bake_isolated_layer(comp, camera, layer, passes)
    }

    fn bake_isolated_layer(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layer: Layer,
        passes: &[EffectPass],
    ) -> Result<Layer, EngineError> {
        let (blend, placement) = (layer.blend_mode, layer.placement);
        self.bake_isolated_layers(comp, camera, vec![LayerWithPasses { layer, passes: passes.to_vec(), pass_sources: Vec::new() }], blend, placement)
    }

    /// 層(または 1 つの層の配置たち)を comp 大の 1 枚へ焼く。
    fn bake_isolated_layers(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        mut sources: Vec<LayerWithPasses>,
        output_blend: CompositeBlendMode,
        placement: crate::doc::core::LayerPlacement,
    ) -> Result<Layer, EngineError> {
        // BlendはMatteでcoverageを得た後、作品の下層との間に一度だけ掛ける。
        for source in &mut sources {
            source.layer.blend_mode = CompositeBlendMode::Normal;
        }
        let (texture, _view) = self.compositor.render_to_texture(
            comp,
            camera,
            &sources,
            crate::render::compositor::NO_BACKGROUND,
        )?;
        let imported = self.compositor.import_premultiplied(&texture)?;
        Ok(Layer {
            content: LayerContent::Texture(imported),
            size: [comp.width as f32, comp.height as f32],
            placement: crate::doc::core::LayerPlacement {
                transform: glam::Affine2::IDENTITY,
                world_transform: None,
                opacity: 1.0,
                z: 0.0,
                rotation_x: 0.0,
                rotation_y: 0.0,
                ..placement
            },
            projection: crate::doc::store::LayerProjection::TwoD,
            projection_camera: camera,
            blend_mode: output_blend,
            shading: Default::default(),
            displace: Default::default(),
            clip: None,
            blocks_light: sources.iter().any(|s| s.layer.blocks_light),
            outline: sources.iter().map(|s| s.layer.outline).max().unwrap_or(0),
            frame: None,
        })
    }

    /// 配置効果の複製は素材と mask が同じなので、直前に組んだ 1 枚を置き直すだけにする。
    /// 平面化は置き場所で絵が変わるので共有しない。
    #[allow(clippy::too_many_arguments)]
    fn build_layer_shared(
        &mut self,
        previous: &mut Option<(LayerId, i64, Layer)>,
        layer: &ResolvedLayer,
        text_documents: &HashMap<LayerId, TextDocument>,
        shape_documents: &HashMap<LayerId, Vec<ShapeNode>>,
        t: RationalTime,
        comp: CompSpec,
        camera: ResolvedCamera,
        projection_camera: ResolvedCamera,
        blend_mode: CompositeBlendMode,
    ) -> Result<Option<Layer>, EngineError> {
        if let Some((id, frame, built)) = previous {
            if *id == layer.id && *frame == layer.source_frame && layer.copy > 0 && !layer.flatten && !matches!(layer.source, LayerSource::Text | LayerSource::Shape) {
                return Ok(Some(Layer { placement: layer.placement, blend_mode, ..built.clone() }));
            }
        }
        let built = self.build_layer(layer, text_documents, shape_documents, t, comp, camera, projection_camera, blend_mode)?;
        *previous = built.clone().map(|built| (layer.id, layer.source_frame, built));
        Ok(built)
    }

    /// 素材を取り、平面化と mask まで済ませた 1 枚。素材が無ければ None。
    #[allow(clippy::too_many_arguments)]
    fn build_layer(
        &mut self,
        layer: &ResolvedLayer,
        text_documents: &HashMap<LayerId, TextDocument>,
        shape_documents: &HashMap<LayerId, Vec<ShapeNode>>,
        t: RationalTime,
        comp: CompSpec,
        camera: ResolvedCamera,
        projection_camera: ResolvedCamera,
        blend_mode: CompositeBlendMode,
    ) -> Result<Option<Layer>, EngineError> {
        let (content, natural, frame) =
            self.texture_for_resolved(layer, text_documents, shape_documents, t, comp, camera, projection_camera)?;
        let Some(content) = content else {
            return Ok(None);
        };
        let mut built = Layer {
            content, size: layer_size(layer, natural), placement: layer.placement,
            projection: layer.projection, projection_camera, blend_mode,
            shading: Default::default(), displace: translate_point_displace(&layer.effects),
            clip: translate_clip(&layer.effects), blocks_light: layer.blocks_light, outline: self.outline_id(layer.id),
            frame,
        };
        let uses_material = self.compositor.catalog.descriptors.iter().any(|d| matches!(d.stage, crate::render::compositor::EffectStage::Warp | crate::render::compositor::EffectStage::Field) && layer.effects.iter().any(|e| e.plugin_id == d.plugin_id));
        let masks_applied = (uses_material || frame.is_some()) && built.content.texture().is_some();
        if masks_applied { built = self.apply_masks_to_layer(built, &layer.masks, natural, frame)?; }
        built = self.apply_material_domains(built, layer, natural, frame)?;
        if matches!(built.content, LayerContent::Model(_) | LayerContent::Texture(_) | LayerContent::LinearTexture(_)) {
            built.shading = self.compositor.surface_shading_for(&layer.effects, matches!(&built.content, LayerContent::Model(m) if m.planar_size.is_some()))
                .map_err(EngineError::Store)?;
        }
        // 場を通った板(material_plane)の後ろの絵の効果は、描いた物を 1 枚の絵として受ける
        // (Blender の Visual Effects の作法)。平らな素材そのものの絵の効果はここへ来ない —
        // texture_for_resolved が素材座標の絵にしている(広がりの法)。空間の物の枠を comp でなく
        // 物の投影範囲 + reach にするのは宿題。
        let picture_of_spatial = matches!(&built.content, LayerContent::Model(m) if m.planar_size.is_some()) && !translate_effect_passes(&layer.effects).is_empty();
        let built = self.flatten_if_asked(comp, camera, built, layer.flatten || picture_of_spatial)?;
        Ok(Some(if masks_applied { built } else { self.apply_masks_to_layer(built, &layer.masks, natural, frame)? }))
    }

    fn apply_masks_to_layer(
        &mut self,
        mut layer: Layer,
        masks: &[ResolvedMask],
        natural: [f32; 2],
        frame: Option<crate::render::compositor::effects::vism::ImageFrame>,
    ) -> Result<Layer, EngineError> {
        if masks.is_empty() {
            return Ok(layer);
        }
        let Some(texture) = layer.content.texture().cloned() else {
            self.layer_failures.push(
                "3D layer masks require the explicit flatten property before 2D coverage"
                    .to_owned(),
            );
            return Ok(layer);
        };
        let [width, height] = texture.width_height();
        let canvas = crate::doc::vector::Canvas {
            width,
            height,
            origin_x: 0,
            origin_y: 0,
        };
        let frame = frame.unwrap_or(crate::render::compositor::effects::vism::ImageFrame { size: natural, origin: [0.0;2], pixels: [width,height] });
        let sx = width as f64 / frame.size[0].max(1.0) as f64;
        let sy = height as f64 / frame.size[1].max(1.0) as f64;
        let masks: Vec<_> = masks.iter().cloned().map(|mut mask| {
            for vertex in &mut mask.shape.vertices {
                vertex.point[0] = (vertex.point[0] - frame.origin[0] as f64) * sx;
                vertex.point[1] = (vertex.point[1] - frame.origin[1] as f64) * sy;
                for point in [&mut vertex.in_tangent, &mut vertex.out_tangent] { point[0] *= sx; point[1] *= sy; }
            }
            mask.expansion *= (sx + sy) * 0.5;
            mask
        }).collect();
        let coverage = crate::render::engine::mask::fold_masks(&masks, &canvas)?;
        let rgba = coverage
            .bytes
            .into_iter()
            .flat_map(|alpha| [alpha, alpha, alpha, alpha])
            .collect::<Vec<_>>();

        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        "layer-mask-coverage".hash(&mut hasher);
        width.hash(&mut hasher);
        height.hash(&mut hasher);
        rgba.hash(&mut hasher);
        let key = hasher.finish();
        let mask_texture = self
            .compositor
            .cached_rgba(key, "layer-mask-coverage", || {
                Ok::<_, std::convert::Infallible>((rgba, width, height))
            })?;
        let masked = self
            .compositor
            .apply_local_alpha_mask(&texture, &mask_texture)?;
        layer.content = LayerContent::Texture(masked);
        Ok(layer)
    }

    pub fn apply_matte(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        target: &Layer,
        matte_source: &Layer,
        mode: crate::doc::store::MatteMode,
    ) -> Result<Layer, EngineError> {
        Ok(self.compositor.matte_layer(
            comp,
            camera,
            target,
            matte_source,
            translate_matte_mode(mode),
        )?)
    }
}

fn collect_text_documents(
    view: &StoreView<'_>,
    resolved: &[ResolvedLayer],
    t: RationalTime,
) -> Result<HashMap<LayerId, TextDocument>, EngineError> {
    let mut documents = HashMap::new();
    for layer in resolved {
        if layer.source == LayerSource::Text {
            if let Some(document) = view
                .resolved_text_document(layer.id, t)
                .map_err(|e| EngineError::Store(e.to_string()))?
            {
                documents.insert(layer.id, document);
            }
        }
    }
    Ok(documents)
}

fn collect_shape_documents(
    view: &StoreView<'_>,
    resolved: &[ResolvedLayer],
    t: RationalTime,
) -> Result<HashMap<LayerId, Vec<ShapeNode>>, EngineError> {
    let mut documents = HashMap::new();
    for layer in resolved {
        if layer.source == LayerSource::Shape {
            let shapes = view
                .shapes_at(layer.id, t)
                .map_err(|e| EngineError::Store(e.to_string()))?;
            documents.insert(layer.id, shown_shapes(&shapes, layer));
        }
    }
    Ok(documents)
}

/// パス効果を掛けた姿。配置の上下どちらに積んでも輪郭には同じに効く(輪郭は絵より先)。
pub(crate) fn shown_shapes(shapes: &[ShapeNode], layer: &ResolvedLayer) -> Vec<ShapeNode> {
    let effects: Vec<_> = layer.effects.iter().chain(&layer.after_effects).cloned().collect();
    crate::doc::store::pathop::with_effects(shapes, &effects)
}

/// 層の 4 隅を画面に映して、効果の余白込みで枠の外に丸ごと出ていれば true。
/// 3D の姿勢が無い層は判定しない(false)。
fn offscreen(comp: CompSpec, camera: ResolvedCamera, layer: &Layer, passes: &[EffectPass]) -> bool {
    let Some(world) = layer.placement.world_transform else { return false };
    if layer.content.texture().is_none() {
        return false;
    }
    let margin = passes.iter().map(EffectPass::padding).max().unwrap_or(0) as f32 + 2.0;
    let corners = crate::doc::core::projected_screen_corners(
        comp,
        camera,
        camera,
        layer.projection,
        world,
        [0.0, 0.0, 0.0],
        [layer.size[0], layer.size[1], 0.0],
    );
    if corners.iter().any(|c| !c.is_finite()) {
        return false;
    }
    let (w, h) = (comp.width as f32, comp.height as f32);
    corners.iter().all(|c| c.x < -margin)
        || corners.iter().all(|c| c.x > w + margin)
        || corners.iter().all(|c| c.y < -margin)
        || corners.iter().all(|c| c.y > h + margin)
}

pub(crate) fn layer_size(layer: &ResolvedLayer, natural: [f32; 2]) -> [f32; 2] {
    [
        if layer.declared_size[0] > 0.0 {
            layer.declared_size[0]
        } else {
            natural[0]
        },
        if layer.declared_size[1] > 0.0 {
            layer.declared_size[1]
        } else {
            natural[1]
        },
    ]
}

#[cfg(test)]
mod placement_contract {
    //! 配置効果(motolii.repeat)は他の効果と同じ口から入り、素材を N 個置く。
    //! 既定は通り抜け。配置効果の**下**に効果を積んだ時だけ、配置を 1 枚に合わせてから掛かる。
    use crate::doc::store::{
        placement, property, Composition, Document, EffectId, EffectInstance, EffectScope, Fps, Intent,
        LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value,
    };
    use crate::render::engine::{known_effects, Engine};

    const SIZE: u32 = 48;
    const DOT: u32 = 4;

    fn document(path: &std::path::Path, count: f64, below: &[&str]) -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: SIZE,
            height: SIZE,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 1,
            background: [0.0, 0.0, 0.0, 0.0],
        }))
        .unwrap();
        let layer = LayerId(1);
        let repeat = EffectId(0);
        let mut effects = vec![EffectInstance { id: repeat, plugin_id: placement::REPEAT.to_owned() }];
        effects.extend(below.iter().enumerate().map(|(i, id)| EffectInstance {
            id: EffectId(i as u32 + 1),
            plugin_id: (*id).to_owned(),
        }));
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::File { path: path.to_string_lossy().into_owned(), fingerprint: None },
                    order: 0,
                    timing: LayerTiming::place(0, None, 1),
                },
            },
            Intent::SetConstant {
                layer,
                property: PropertyId::new(property::POSITION).unwrap(),
                value: Value::Vec2([4.0, 4.0]),
            },
            Intent::SetEffects { layer, effects },
            Intent::SetConstant { layer, property: PropertyId::effect_param(repeat, "count").unwrap(), value: Value::F64(count) },
            Intent::SetConstant { layer, property: PropertyId::effect_param(repeat, "position_each").unwrap(), value: Value::Vec2([10.0, 0.0]) },
        ])
        .unwrap();
        doc
    }

    fn covered(pixels: &[u8]) -> usize {
        pixels.chunks(4).filter(|px| px[3] > 0).count()
    }

    fn png(dir: &std::path::Path, name: &str, rgba: [u8; 4]) -> std::path::PathBuf {
        let path = dir.join(name);
        let pixels = rgba.into_iter().cycle().take((DOT * DOT * 4) as usize).collect::<Vec<_>>();
        image::save_buffer(&path, &pixels, DOT, DOT, image::ColorType::Rgba8).unwrap();
        path
    }

    fn add_file_layer(doc: &mut Document, id: u64, order: i16, path: &std::path::Path, parent: Option<LayerId>, clip: bool) -> LayerId {
        use crate::doc::store::LayerAttrsPatch;
        let layer = LayerId(id);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::File { path: path.to_string_lossy().into_owned(), fingerprint: None },
                    order,
                    timing: LayerTiming::place(0, None, 1),
                },
            },
            Intent::SetAttrs { layer, patch: LayerAttrsPatch { parent: Some(parent), clip_to_below: Some(clip), ..Default::default() } },
            Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([4.0, 4.0]) },
        ])
        .unwrap();
        layer
    }

    /// 半透明の緑を、赤の複製に clip した時の画素。二重に描かれていれば赤が 64 まで落ちる。
    fn red_floor(pixels: &[u8]) -> u8 {
        pixels.chunks(4).filter(|px| px[3] > 0).map(|px| px[0]).min().unwrap_or(255)
    }

    #[test]
    fn clipping_onto_repeated_copies_is_screen_overlap_from_outside_and_per_copy_inside() {
        let dir = tempfile::tempdir().unwrap();
        let red = png(dir.path(), "red.png", [255, 0, 0, 255]);
        let green = png(dir.path(), "green.png", [0, 255, 0, 128]);
        let mut engine = Engine::new().unwrap();

        // 外から: 赤 3 枚(2 px ずつ重なる)の上に緑を clip。緑は和に 1 回だけ乗るので、重なりでも赤は半分より落ちない。
        let mut outside = document(&red, 3.0, &[]);
        outside.apply(Intent::SetConstant { layer: LayerId(1), property: PropertyId::effect_param(EffectId(0), "position_each").unwrap(), value: Value::Vec2([2.0, 0.0]) }).unwrap();
        add_file_layer(&mut outside, 2, 1, &green, None, true);
        let pixels = engine.render_frame(&outside.view(), RationalTime::ZERO).unwrap();
        assert_eq!(covered(&pixels), (DOT * (DOT + 4)) as usize, "the clip paints only where the union of copies is");
        assert!(red_floor(&pixels) >= 120, "a half-transparent clip is drawn once over overlapping copies, got red {}", red_floor(&pixels));

        // 中で: グループに赤と(赤へ clip した)緑を入れて丸ごと 2 枚に増やす。緑は自分の番号の赤にだけ切られる。
        let mut inside = Document::new();
        inside.apply(Intent::SetComposition(Composition { width: SIZE, height: SIZE, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [0.0; 4] })).unwrap();
        let group = LayerId(10);
        inside.apply_all([
            Intent::AddLayer(group),
            Intent::SetMeta { layer: group, meta: LayerMeta { source: LayerSource::Group, order: 0, timing: LayerTiming::place(0, None, 1) } },
            Intent::SetEffects { layer: group, effects: vec![EffectInstance { id: EffectId(0), plugin_id: placement::REPEAT.to_owned() }] },
            Intent::SetConstant { layer: group, property: PropertyId::effect_param(EffectId(0), "count").unwrap(), value: Value::F64(2.0) },
            Intent::SetConstant { layer: group, property: PropertyId::effect_scope(EffectId(0)), value: Value::Enum(EffectScope::Whole.enum_value()) },
            Intent::SetConstant { layer: group, property: PropertyId::effect_param(EffectId(0), "position_each").unwrap(), value: Value::Vec2([10.0, 0.0]) },
        ]).unwrap();
        add_file_layer(&mut inside, 11, 0, &red, Some(group), false);
        add_file_layer(&mut inside, 12, 1, &green, Some(group), true);
        let pixels = engine.render_frame(&inside.view(), RationalTime::ZERO).unwrap();
        assert_eq!(covered(&pixels), 2 * (DOT * DOT) as usize, "each copy carries its own clipped lyric, nothing leaks outside the copies");
        assert!(red_floor(&pixels) >= 120, "inside a copy the clip is drawn once, got red {}", red_floor(&pixels));
    }

    /// グループの効果は host が解く。Each なら子がそのまま重なり(重なりは 2 回描かれる)、
    /// Whole を 1 つ積むと子は 1 枚に焼かれてからグループの不透明度で 1 回だけ乗る。効果(gain=1)は何も知らない。
    #[test]
    fn a_whole_effect_bakes_the_group_into_one_plate() {
        let dir = tempfile::tempdir().unwrap();
        let red = png(dir.path(), "red.png", [255, 0, 0, 255]);
        let mut engine = Engine::new().unwrap();
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: SIZE, height: SIZE, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [0.0; 4] })).unwrap();
        let group = LayerId(10);
        doc.apply_all([
            Intent::AddLayer(group),
            Intent::SetMeta { layer: group, meta: LayerMeta { source: LayerSource::Group, order: 0, timing: LayerTiming::place(0, None, 1) } },
            Intent::SetConstant { layer: group, property: PropertyId::new(property::OPACITY).unwrap(), value: Value::F64(0.5) },
            Intent::SetEffects { layer: group, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.gain".to_owned() }] },
        ]).unwrap();
        add_file_layer(&mut doc, 11, 0, &red, Some(group), false);
        let second = add_file_layer(&mut doc, 12, 1, &red, Some(group), false);
        doc.apply(Intent::SetConstant { layer: second, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([6.0, 4.0]) }).unwrap();
        let alpha_max = |pixels: &[u8]| pixels.chunks(4).map(|px| px[3]).max().unwrap();

        // Each(既定): グループの不透明度は子に降りず、重なりも 1 枚ずつ。全部不透明。
        let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert_eq!(covered(&pixels), (DOT * (DOT + 2)) as usize);
        assert_eq!(alpha_max(&pixels), 255, "pass-through children keep their own opacity");

        // Whole: 2 枚を 1 枚に焼き、グループの 50 % で 1 回乗る。重なりも 50 % のまま。
        doc.apply(Intent::SetConstant { layer: group, property: PropertyId::effect_scope(EffectId(0)), value: Value::Enum(EffectScope::Whole.enum_value()) }).unwrap();
        let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert_eq!(covered(&pixels), (DOT * (DOT + 2)) as usize, "the plate covers the same pixels");
        let alphas: std::collections::BTreeSet<u8> = pixels.chunks(4).filter(|px| px[3] > 0).map(|px| px[3]).collect();
        assert!(alphas.iter().all(|a| (120..=136).contains(a)), "one plate at 50 %, overlap included, got {alphas:?}");
    }

    /// 背景は世界の板でなく出力の地。カメラを回しても書き出しの全画素が背景色で、素材の無い所に穴は開かない。
    #[test]
    fn the_background_fills_every_pixel_under_a_tilted_camera() {
        let dir = tempfile::tempdir().unwrap();
        let green = png(dir.path(), "green.png", [0, 255, 0, 255]);
        let mut engine = Engine::new().unwrap();
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: SIZE, height: SIZE, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [1.0, 0.0, 0.0, 1.0] })).unwrap();
        add_file_layer(&mut doc, 1, 0, &green, None, false);
        // 注視点(comp 中心)に置く。寄せたカメラでも枠の中に残る。
        doc.apply(Intent::SetConstant { layer: LayerId(1), property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([SIZE as f64 * 0.5 - DOT as f64 * 0.5; 2]) }).unwrap();
        let camera = LayerId(2);
        doc.apply_all([
            Intent::AddLayer(camera),
            Intent::SetMeta { layer: camera, meta: LayerMeta { source: LayerSource::Camera, order: 1, timing: LayerTiming::place(0, None, 1) } },
            Intent::SetConstant { layer: camera, property: PropertyId::new(property::CAMERA_ORBIT).unwrap(), value: Value::Vec2([-25.0, 60.0]) },
            Intent::SetConstant { layer: camera, property: PropertyId::new(property::CAMERA_DISTANCE).unwrap(), value: Value::F64(0.5) },
        ]).unwrap();
        let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert_eq!(covered(&pixels), (SIZE * SIZE) as usize, "no hole in the frame");
        let green_px = pixels.chunks(4).filter(|px| px[1] > 200 && px[0] < 50).count();
        assert!(green_px > 0, "the layer is still in view");
        // 縁の画素は赤と緑の混ざり。どの画素も背景か素材の色で、黒や灰(板の外・深度の穴)は無い。
        let stray = pixels.chunks(4).filter(|px| (u32::from(px[0]) + u32::from(px[1])) < 200 || px[2] > 50).count();
        assert_eq!(stray, 0, "every pixel is background or layer");
    }

    /// 生成器(gradient のように image 入力の無い効果)は素材の形の中に閉じ込められる。
    /// Repeat の上なら各複製に、下なら増えた後の全体に付くが、どちらも形は消えない。
    #[test]
    fn a_generator_stays_inside_the_source_shape_above_and_below_the_placement() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("disc.png");
        let mut pixels = Vec::new();
        for y in 0..DOT * 3 {
            for x in 0..DOT * 3 {
                let inside = (x as f32 - 5.5).powi(2) + (y as f32 - 5.5).powi(2) < 25.0;
                pixels.extend_from_slice(&[255, 255, 255, if inside { 255 } else { 0 }]);
            }
        }
        image::save_buffer(&source, &pixels, DOT * 3, DOT * 3, image::ColorType::Rgba8).unwrap();
        let disc = pixels.chunks(4).filter(|px| px[3] > 0).count();
        let mut engine = Engine::new().unwrap();
        let mut render = |doc: &Document| engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();

        let below = render(&document(&source, 3.0, &["motolii.gradient"]));
        assert_eq!(covered(&below), 3 * disc, "below the placement the gradient fills only the three discs");

        let mut above = document(&source, 3.0, &[]);
        above
            .apply(Intent::SetEffects {
                layer: LayerId(1),
                effects: vec![
                    EffectInstance { id: EffectId(1), plugin_id: "motolii.gradient".to_owned() },
                    EffectInstance { id: EffectId(0), plugin_id: placement::REPEAT.to_owned() },
                ],
            })
            .unwrap();
        let above = render(&above);
        assert_eq!(covered(&above), 3 * disc, "above the placement each copy is a gradient disc");
        assert!(above.chunks(4).filter(|px| px[3] > 0).any(|px| px[0] != px[1]), "the gradient is visibly painted");
    }

    #[test]
    fn the_repeat_effect_places_the_source_count_times_through_the_effect_stack() {
        assert!(
            known_effects().iter().any(|e| e.plugin_id == placement::REPEAT),
            "the placement effect must sit in the same catalog as the shader effects"
        );
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("dot.png");
        let pixels = [255u8, 0, 0, 255].into_iter().cycle().take((DOT * DOT * 4) as usize).collect::<Vec<_>>();
        image::save_buffer(&source, &pixels, DOT, DOT, image::ColorType::Rgba8).unwrap();
        let mut engine = Engine::new().unwrap();
        let mut render = |doc: &Document| engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();

        let one = render(&document(&source, 1.0, &[]));
        let three = render(&document(&source, 3.0, &[]));
        let area = (DOT * DOT) as usize;
        assert_eq!(covered(&one), area, "one copy is the plain layer");
        assert_eq!(covered(&three), 3 * area, "three copies at step 10 do not overlap and are all drawn");

        // 10 copies at step 10 px in a 48 px comp: the last ones fall outside and are not built.
        let far = render(&document(&source, 10.0, &[]));
        assert_eq!(covered(&far), 5 * area, "only the copies inside the frame paint (x = 4 … 44; 54 and beyond are outside)");
        let mut counting = Engine::new().unwrap();
        counting.render_frame(&document(&source, 10.0, &[]).view(), RationalTime::ZERO).unwrap();
        assert_eq!(counting.drawn_layers(), 5, "copies wholly outside the frame are culled before they are built");

        let mut blurred = document(&source, 3.0, &["motolii.blur"]);
        blurred
            .apply(Intent::SetConstant {
                layer: LayerId(1),
                property: PropertyId::effect_param(EffectId(1), "radius").unwrap(),
                value: Value::F64(2.0),
            })
            .unwrap();
        let three_then_blur = render(&blurred);
        assert!(
            covered(&three_then_blur) > 3 * area,
            "a blur below the placement runs on the assembled picture and spreads past every copy"
        );
        assert!(
            three.chunks(4).zip(three_then_blur.chunks(4)).all(|(sharp, soft)| sharp[3] == 0 || soft[3] > 0),
            "every copy is still present under the blur"
        );
    }
}

/// 使われなくなった動画の再生機(ffmpeg 子プロセス)を掃く猶予。30fps で 5 秒。
const VIDEO_PLAYER_PURGE_EVERY: u32 = 150;

impl Engine {
    fn purge_idle_video_players(&mut self) {
        self.frame_cache_tick += 1;
        self.flush_pending_frame_copies();
        self.renders_since_video_purge += 1;
        if self.renders_since_video_purge < VIDEO_PLAYER_PURGE_EVERY {
            return;
        }
        self.renders_since_video_purge = 0;
        for (_, video) in self.videos.values() {
            video.begin_frame();
        }
    }
}

/// 画面座標の矩形(左・上・右・下)。
type ScreenRect = [f32; 4];

fn rect_inside(inner: ScreenRect, outer: ScreenRect) -> bool {
    inner[0] >= outer[0] && inner[1] >= outer[1] && inner[2] <= outer[2] && inner[3] <= outer[3]
}

impl Engine {
    /// 復号の前に分かる範囲で層の画面上の矩形を出す。寸が分かる動画の層だけ。
    /// 軸に沿った矩形なら `Some((rect, true))`、回っていれば外接矩形と `false`。
    fn media_screen_rect(&self, comp: CompSpec, camera: ResolvedCamera, layer: &ResolvedLayer) -> Option<(ScreenRect, bool)> {
        let LayerSource::File { path, .. } = &layer.source else { return None };
        if crate::render::media::is_still_image_path(path)
            || crate::render::media::is_mesh_path(path)
            || crate::render::media::is_point_cloud_path(path)
        {
            return None;
        }
        let info = self.probes.get(path)?;
        if info.rotation != 0 {
            return None;
        }
        let world = layer.placement.world_transform?;
        let size = layer_size(layer, [info.width as f32, info.height as f32]);
        let corners = crate::doc::core::projected_screen_corners(
            comp, camera, camera, layer.projection, world, [0.0, 0.0, 0.0], [size[0], size[1], 0.0],
        );
        if corners.iter().any(|c| !c.is_finite()) {
            return None;
        }
        let rect = corners.iter().fold([f32::MAX, f32::MAX, f32::MIN, f32::MIN], |r, c| {
            [r[0].min(c.x), r[1].min(c.y), r[2].max(c.x), r[3].max(c.y)]
        });
        // 4 隅が矩形の角に乗っていれば軸に沿っている(回転・傾き無し)。
        let eps = 0.5;
        let on_corner = |c: &glam::Vec2| {
            ((c.x - rect[0]).abs() < eps || (c.x - rect[2]).abs() < eps)
                && ((c.y - rect[1]).abs() < eps || (c.y - rect[3]).abs() < eps)
        };
        Some((rect, corners.iter().all(on_corner)))
    }

    /// 上から順に不透明な矩形を積み、丸ごと覆われた層と画面の外の層を集める(Blender VSE の型)。
    fn unseen_layers(
        &self,
        comp: CompSpec,
        camera: ResolvedCamera,
        resolved: &[ResolvedLayer],
        needs_auxiliary_views: bool,
        matte_sources: &HashSet<LayerId>,
    ) -> HashSet<usize> {
        let mut unseen = HashSet::new();
        if needs_auxiliary_views {
            return unseen;
        }
        let screen: ScreenRect = [0.0, 0.0, comp.width as f32, comp.height as f32];
        let mut covers: Vec<ScreenRect> = Vec::new();
        for (index, layer) in resolved.iter().enumerate().rev() {
            // 他の層の入力になる物は消さない。
            let feeds_others = matte_sources.contains(&layer.id) || layer.matte.is_some() || layer.clip_to_below;
            let Some((rect, axis_aligned)) = self.media_screen_rect(comp, camera, layer) else { continue };
            let plain = layer.effects.is_empty() && layer.after_effects.is_empty() && layer.masks.is_empty();
            let offscreen = rect[2] < 0.0 || rect[3] < 0.0 || rect[0] > screen[2] || rect[1] > screen[3];
            if !feeds_others && plain && (offscreen || covers.iter().any(|cover| rect_inside(rect, *cover))) {
                unseen.insert(index);
                continue;
            }
            let opaque = axis_aligned
                && plain
                && layer.placement.opacity >= 1.0
                && layer.blend_mode == crate::doc::store::BlendMode::Normal
                && layer.matte.is_none()
                && !layer.clip_to_below
                && !layer.ghost
                // 面が画面に平行なら projection の種類は問わない(描画順は order のまま)。
                && layer.placement.rotation_x == 0.0
                && layer.placement.rotation_y == 0.0;
            if opaque {
                covers.push(rect);
            }
        }
        unseen
    }
}

/// 先に開いておく幅。30fps で半秒。
const WARM_AHEAD_FRAMES: i64 = 15;

impl Engine {
    /// 再生位置の少し先で現れる動画層の復号器を、今のうちに開く。timeline は未来を知っている。
    /// 待たず、失敗も記録しない(本番の描画で改めて分かる)。
    pub fn warm_upcoming(&mut self, view: &StoreView<'_>, t: RationalTime) -> Result<(), EngineError> {
        let composition = view
            .composition()
            .map_err(|e| EngineError::Store(e.to_string()))?
            .ok_or(EngineError::NoComposition)?;
        let fps = composition.fps;
        let now_frame = t.try_to_frame_floor(fps).map_err(|e| EngineError::Time(e.to_string()))?;
        let ahead = RationalTime::try_from_frame(now_frame + WARM_AHEAD_FRAMES, fps)
            .map_err(|e| EngineError::Time(e.to_string()))?;
        let now_ids: HashSet<LayerId> = view
            .resolved_layers(t)
            .map_err(|e| EngineError::Store(e.to_string()))?
            .iter()
            .map(|layer| layer.id)
            .collect();
        let upcoming = view
            .resolved_layers(ahead)
            .map_err(|e| EngineError::Store(e.to_string()))?;

        let failures = std::mem::take(&mut self.layer_failures);
        let was_realtime = self.realtime;
        self.realtime = true;
        for layer in upcoming.iter().filter(|layer| !now_ids.contains(&layer.id)) {
            let LayerSource::File { path, .. } = &layer.source else { continue };
            if crate::render::media::is_still_image_path(path)
                || crate::render::media::is_audio_path(path)
                || crate::render::media::is_mesh_path(path)
                || crate::render::media::is_point_cloud_path(path)
            {
                continue;
            }
            let _ = self.media_texture_for(path, layer.source_time, layer.id);
        }
        self.realtime = was_realtime;
        self.layer_failures = failures;
        Ok(())
    }
}

#[cfg(test)]
mod projection_contract {
    //! 3 つの札の法(2026-09-12): 2D は世界に居ない(深度に参加せず積み順だけ)、2.5D は z で並ぶ、
    //! 既定カメラでは 3 つの札が同じ場所に映る。
    use crate::doc::store::*;
    use crate::doc::vector::{Brush, Fill, PathSource, Point, Rgb, Shape};
    use crate::render::engine::Engine;

    fn square(color: Rgb, size: f64) -> ShapeNode {
        ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: size, y: size } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(color), ..Default::default() }) })
    }
    fn document() -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: 256, height: 256, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [1.0, 1.0, 1.0, 1.0] })).unwrap();
        doc
    }
    fn layer(doc: &mut Document, id: u64, order: i16, color: Rgb, projection: LayerProjection, position: [f64; 2], z: f64) {
        let layer = LayerId(id);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Shape, order, timing: LayerTiming::place(0, None, 1) } },
            Intent::SetShapes { layer, shapes: vec![square(color, 80.0)] },
            Intent::SetAttrs { layer, patch: LayerAttrsPatch { projection: Some(projection), ..Default::default() } },
            Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2(position) },
            Intent::SetConstant { layer, property: PropertyId::new(property::POSITION_Z).unwrap(), value: Value::F64(z) },
        ]).unwrap();
    }
    fn count(px: &[u8], red: bool) -> usize {
        px.chunks_exact(4).filter(|p| if red { p[0] > 128 && p[2] < 100 } else { p[2] > 128 && p[0] < 100 }).count()
    }

    /// 下の段に手前(z=-200)の 3D の板、上の段に 2D の板。2D は積み順で勝ち、3D に隠されない。
    /// 同じ配置で上の段が 2.5D(z=0)なら、深度で 3D の奥になる。
    #[test]
    fn a_two_d_layer_above_in_the_stack_is_never_hidden_by_closer_three_d() {
        let mut engine = Engine::new().unwrap();
        let mut doc = document();
        layer(&mut doc, 1, 0, Rgb { r: 1.0, g: 0.0, b: 0.0 }, LayerProjection::ThreeD, [128.0, 128.0], -200.0);
        layer(&mut doc, 2, 1, Rgb { r: 0.0, g: 0.2, b: 1.0 }, LayerProjection::TwoD, [148.0, 148.0], 0.0);
        let px = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert!(count(&px, false) > 6000, "the 2D layer on top must be fully visible: {} blue pixels", count(&px, false));
        assert!(count(&px, true) > 1000, "the 3D layer still shows around it");
        doc.apply(Intent::SetAttrs { layer: LayerId(2), patch: LayerAttrsPatch { projection: Some(LayerProjection::TwoPointFiveD), ..Default::default() } }).unwrap();
        let px = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert!(count(&px, false) < 100, "a 2.5D layer at z=0 sits behind a 3D layer at z=-200: {} blue pixels", count(&px, false));
    }

    /// 2D 同士は積み順。3D の間に挟まれても変わらない。
    #[test]
    fn two_d_layers_stack_in_timeline_order() {
        let mut engine = Engine::new().unwrap();
        let mut doc = document();
        layer(&mut doc, 1, 0, Rgb { r: 0.0, g: 0.2, b: 1.0 }, LayerProjection::TwoD, [128.0, 128.0], 0.0);
        layer(&mut doc, 2, 1, Rgb { r: 1.0, g: 0.0, b: 0.0 }, LayerProjection::TwoD, [128.0, 128.0], -500.0);
        let px = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert!(count(&px, false) < 50, "z does not lift a 2D layer above a later one: {} blue pixels", count(&px, false));
        assert!(count(&px, true) > 6000);
    }

    /// 既定カメラでは同じ Position の板が 2D / 2.5D / 3D で同じ場所に映る。
    #[test]
    fn the_three_projections_coincide_under_the_default_camera() {
        let mut engine = Engine::new().unwrap();
        let mut centroids = Vec::new();
        for projection in [LayerProjection::TwoD, LayerProjection::TwoPointFiveD, LayerProjection::ThreeD] {
            let mut doc = document();
            layer(&mut doc, 1, 0, Rgb { r: 0.0, g: 0.2, b: 1.0 }, projection, [60.0, 60.0], 0.0);
            let px = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
            let (mut sx, mut sy, mut n) = (0.0, 0.0, 0usize);
            for (i, p) in px.chunks_exact(4).enumerate() { if p[2] > 128 && p[0] < 100 { sx += (i % 256) as f64; sy += (i / 256) as f64; n += 1; } }
            centroids.push((sx / n as f64, sy / n as f64, n));
        }
        for c in &centroids[1..] {
            assert!((c.0 - centroids[0].0).abs() < 0.5 && (c.1 - centroids[0].1).abs() < 0.5 && c.2 == centroids[0].2, "{centroids:?}");
        }
    }
}

/// 時刻を秒だけずらす(ミリ秒の分母で厳密に足す)。クリップの前へは行かない。
fn shifted_by_seconds(t: RationalTime, offset: f32) -> RationalTime {
    const DEN: i64 = 1000;
    let num = (offset as f64 * DEN as f64).round() as i64;
    let shifted = t
        .num()
        .checked_mul(DEN)
        .and_then(|scaled| num.checked_mul(t.den()).map(|by| scaled + by))
        .zip(t.den().checked_mul(DEN))
        .and_then(|(num, den)| RationalTime::try_new(num, den).ok());
    match shifted {
        Some(at) if at.as_seconds_f64() >= 0.0 => at,
        _ => RationalTime::ZERO,
    }
}

/// 別の時刻ごとに引き直した「層の姿」。鍵はずれ(ミリ秒)。
type OtherTimes = BTreeMap<i64, (RationalTime, Vec<ResolvedLayer>, HashMap<LayerId, TextDocument>, HashMap<LayerId, Vec<ShapeNode>>)>;

/// 時刻のずれ(秒)を鍵にする — 同じずれは 1 回しか引かない。
fn offset_key(offset: f32) -> i64 {
    (offset as f64 * 1000.0).round() as i64
}

/// 別の時刻を読むための層の番号。復号器の流れを本体と分けるためだけの物で、Document には無い。
fn lookbehind_layer_id(layer: LayerId, offset: f32) -> LayerId {
    let mut hasher = std::hash::DefaultHasher::new();
    use std::hash::{Hash as _, Hasher as _};
    layer.0.hash(&mut hasher);
    offset_key(offset).hash(&mut hasher);
    LayerId(hasher.finish() | 1 << 63)
}

/// 別の時刻を読む効果は、**たどり着き方で絵が変わってはいけない**(実 GPU)。
///
/// 効果が自分で前フレームを覚える道を恒久禁止している理由がここ(`docs/plugin-resources.md` §6)。
/// ホストが時刻を渡す形なら、同じ時刻は何度描いても、どの順で描いても同じ絵になる。
#[cfg(test)]
mod time_reference_is_deterministic {
    use crate::doc::store::{property, Composition, Document, EffectId, EffectInstance, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value};
    use crate::render::engine::Engine;

    const SIZE: u32 = 64;

    /// 中身が時刻で変わる素材(動画)が要る。1 コマごとに色が変わる小さな mp4 を作る。
    fn clip(dir: &std::path::Path) -> Option<std::path::PathBuf> {
        let out = dir.join("ramp.mp4");
        let status = std::process::Command::new("ffmpeg")
            .args(["-v", "error", "-f", "lavfi", "-i", "testsrc2=s=64x64:d=2:r=10",
                   "-pix_fmt", "yuv420p", "-c:v", "libx264", "-y"])
            .arg(&out)
            .status()
            .ok()?;
        status.success().then_some(out)
    }

    fn document(path: &std::path::Path) -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: SIZE, height: SIZE, fps: Fps::try_new(10, 1).unwrap(), duration_frames: 20, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
        let layer = LayerId(1);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: path.to_string_lossy().into_owned(), fingerprint: None }, order: 0, timing: LayerTiming::place(0, None, 20) } },
            Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) },
            Intent::SetEffects { layer, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.time_difference".into() }] },
        ]).unwrap();
        doc
    }

    /// 先に素の層(効果なし)で測る。ここが揺れるなら、揺れているのは復号であって時間参照ではない。
    #[test]
    fn a_plain_clip_is_already_the_same_however_you_got_there() {
        let dir = tempfile::tempdir().unwrap();
        let Some(path) = clip(dir.path()) else { eprintln!("ffmpeg が無いので飛ばす"); return };
        let mut doc = document(&path);
        doc.apply(Intent::SetEffects { layer: LayerId(1), effects: vec![] }).unwrap();
        let at = RationalTime::try_from_frame(12, Fps::try_new(10, 1).unwrap()).unwrap();
        let mut engine = Engine::new().unwrap();
        let jumped = engine.render_frame(&doc.view(), at).unwrap();
        for frame in 0..12 {
            let t = RationalTime::try_from_frame(frame, Fps::try_new(10, 1).unwrap()).unwrap();
            engine.render_frame(&doc.view(), t).unwrap();
        }
        let scrubbed = engine.render_frame(&doc.view(), at).unwrap();
        assert_eq!(jumped, scrubbed, "効果なしでも、たどり着き方で絵が変わっている");
    }

    #[test]
    fn the_same_time_gives_the_same_picture_however_you_got_there() {
        let dir = tempfile::tempdir().unwrap();
        let Some(path) = clip(dir.path()) else { eprintln!("ffmpeg が無いので飛ばす"); return };
        let doc = document(&path);
        let at = RationalTime::try_from_frame(12, Fps::try_new(10, 1).unwrap()).unwrap();
        let mut engine = Engine::new().unwrap();

        // いきなりその時刻へ飛ぶ。
        let jumped = engine.render_frame(&doc.view(), at).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());

        // 頭から順に辿ってから、同じ時刻へ。
        for frame in 0..12 {
            let t = RationalTime::try_from_frame(frame, Fps::try_new(10, 1).unwrap()).unwrap();
            engine.render_frame(&doc.view(), t).unwrap();
        }
        let scrubbed = engine.render_frame(&doc.view(), at).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());

        assert_eq!(jumped, scrubbed, "同じ時刻の絵が、たどり着き方で変わった");
        // 素材が時刻で変わる物であることの確認(変わらない素材なら試験になっていない)。
        let other = RationalTime::try_from_frame(4, Fps::try_new(10, 1).unwrap()).unwrap();
        assert_ne!(jumped, engine.render_frame(&doc.view(), other).unwrap(), "時刻で絵が変わる素材で測っている");
    }
}

/// feedback(前のフレームを保つ効果)は**入点を初期条件とする漸化式**(`docs/plugin-resources.md` §6-3)。
/// 効果は覚えない — host が状態を持ち、飛んで来ても入点(か checkpoint)から辿り直すので、
/// 同じ時刻は何度描いても、どの順で描いても同じ絵(実 GPU)。
#[cfg(test)]
mod feedback_is_a_recurrence_from_the_in_point {
    use crate::doc::store::{property, Composition, Document, EffectId, EffectInstance, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value};
    use crate::render::engine::Engine;

    const SIZE: u32 = 64;
    const FRAMES: i64 = 50;

    fn clip(dir: &std::path::Path) -> Option<std::path::PathBuf> {
        let out = dir.join("ramp.mp4");
        let status = std::process::Command::new("ffmpeg")
            .args(["-v", "error", "-f", "lavfi", "-i", "testsrc2=s=64x64:d=5:r=10", "-pix_fmt", "yuv420p", "-c:v", "libx264", "-y"])
            .arg(&out).status().ok()?;
        status.success().then_some(out)
    }

    fn fps() -> Fps { Fps::try_new(10, 1).unwrap() }
    fn at(frame: i64) -> RationalTime { RationalTime::try_from_frame(frame, fps()).unwrap() }

    fn document(path: &std::path::Path, with_trail: bool) -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: SIZE, height: SIZE, fps: fps(), duration_frames: FRAMES, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
        let layer = LayerId(1);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: path.to_string_lossy().into_owned(), fingerprint: None }, order: 0, timing: LayerTiming::place(0, None, FRAMES) } },
            Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) },
        ]).unwrap();
        if with_trail {
            doc.apply(Intent::SetEffects { layer, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "import.ceil_trail".into() }] }).unwrap();
        }
        doc
    }

    fn walked_to(doc: &Document, frame: i64) -> (Engine, Vec<u8>) {
        let mut engine = Engine::new().unwrap();
        let mut last = Vec::new();
        for f in 0..=frame { last = engine.render_frame(&doc.view(), at(f)).unwrap(); }
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        (engine, last)
    }

    #[test]
    fn the_same_frame_is_the_same_picture_however_you_got_there() {
        let dir = tempfile::tempdir().unwrap();
        let Some(path) = clip(dir.path()) else { eprintln!("ffmpeg が無いので飛ばす"); return };
        let doc = document(&path, true);
        let (mut engine, walked) = walked_to(&doc, 12);
        // 同じフレームをもう一度(2 つ目の窓が同じ時刻を描く形)。
        assert_eq!(engine.render_frame(&doc.view(), at(12)).unwrap(), walked, "同じフレームの描き直しで絵が変わった");
        // いきなり飛ぶ(別の engine = 状態が無い)。
        let mut fresh = Engine::new().unwrap();
        let jumped = fresh.render_frame(&doc.view(), at(12)).unwrap();
        assert!(fresh.layer_failures().is_empty(), "{:?}", fresh.layer_failures());
        assert_eq!(jumped, walked, "飛んで来た絵が、辿った絵と違う");
        // 効果が効いている(残像 ≠ 素の絵)、時刻で違う。
        let plain = Engine::new().unwrap().render_frame(&document(&path, false).view(), at(12)).unwrap();
        assert_ne!(walked, plain, "feedback が絵を変えていない");
        assert_ne!(walked, engine.render_frame(&doc.view(), at(4)).unwrap(), "時刻で絵が変わる素材で測っている");
    }

    #[test]
    fn going_back_and_forward_replays_from_the_nearest_checkpoint() {
        let dir = tempfile::tempdir().unwrap();
        let Some(path) = clip(dir.path()) else { eprintln!("ffmpeg が無いので飛ばす"); return };
        let doc = document(&path, true);
        let (_, straight) = walked_to(&doc, 45);
        // 40 まで辿ってから(checkpoint 30 が焼けている)45 へ飛ぶ。
        let (mut engine, _) = walked_to(&doc, 40);
        assert_eq!(engine.render_frame(&doc.view(), at(45)).unwrap(), straight, "checkpoint から辿り直した絵が違う");
        // 戻る(35 ← 45): checkpoint 30 から 5 歩。
        let (_, back) = walked_to(&doc, 35);
        assert_eq!(engine.render_frame(&doc.view(), at(35)).unwrap(), back, "戻った絵が違う");
        // 入点より前は状態が無く、入点(0)は初期条件。
        assert_eq!(engine.render_frame(&doc.view(), at(0)).unwrap(), walked_to(&doc, 0).1);
    }

    /// 静止した素材に残像を掛けると、history は 0.2 ずつ入力へ寄る(1 − 0.8ⁿ)。前のフレームを本当に
    /// 読んでいれば 12 フレーム目は 0 フレーム目より明るい。毎フレーム初期条件なら同じ明るさのまま。
    #[test]
    fn history_accumulates_over_frames() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("white.png");
        image::save_buffer(&path, &vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE, image::ColorType::Rgba8).unwrap();
        let doc = document(&path, true);
        let mean = |px: &[u8]| px.chunks_exact(4).map(|p| p[0] as f64).sum::<f64>() / (SIZE * SIZE) as f64;
        let (_, first) = walked_to(&doc, 0);
        let (_, later) = walked_to(&doc, 12);
        assert!(mean(&later) > mean(&first) * 2.0, "history が積もっていない: {} → {}", mean(&first), mean(&later));
    }

    #[test]
    fn editing_the_document_restarts_the_history() {
        let dir = tempfile::tempdir().unwrap();
        let Some(path) = clip(dir.path()) else { eprintln!("ffmpeg が無いので飛ばす"); return };
        let mut doc = document(&path, true);
        let (mut engine, before) = walked_to(&doc, 12);
        // 書類を変えると(位置を動かす)、履歴は新しい書類で入点から: 新しい engine の絵と一致する。
        doc.apply(Intent::SetConstant { layer: LayerId(1), property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([6.0, 0.0]) }).unwrap();
        let edited = engine.render_frame(&doc.view(), at(12)).unwrap();
        assert_ne!(edited, before);
        assert_eq!(edited, Engine::new().unwrap().render_frame(&doc.view(), at(12)).unwrap(), "編集後の絵が、入点からの絵と違う");
    }
}
