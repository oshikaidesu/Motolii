//! 1 コマ分の層を建てて焼く: 積み方・効果・マスク・Matte・板への焼き込み。

use super::*;

impl Engine {
    /// 今描いている窓の寸法(画面の道の feedback の鍵)。読み戻しの道は出力寸法。
    fn window_size(&self, comp: CompSpec) -> [u32; 2] {
        self.feedback_window.map_or([comp.width, comp.height], |w| [w.width, w.height])
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn build_layers(
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
        self.solve_particles(view, resolved, t);
        let needs_auxiliary_views = {
            let surface_ids: HashSet<_> = self.compositor.catalog.definitions.iter()
                .filter(|d| d.manifest.stage == crate::render::compositor::IsfStage::Surface)
                .map(|d| d.plugin_id()).collect();
            resolved.iter().flat_map(|l| l.effects.iter().chain(&l.after_effects))
                .any(|e| surface_ids.contains(e.plugin_id.as_str()))
        };
        self.stamp_clock(view, t);
        // 別の時刻を要求した効果があれば、その時刻の層の姿をここで 1 回だけ引き直す(同じずれは共有)。
        let mut other_times: OtherTimes = BTreeMap::new();
        for layer in resolved {
            let in_point = layer_in_point(view, layer.id);
            for pass in crate::render::engine::translate::translate_effect_passes(&layer.effects).into_iter().chain(crate::render::engine::translate::translate_plate_passes(&layer.after_effects)) {
                for (i, offset) in pass.image_time_offsets().iter().enumerate() {
                    let base = pass.image_time_base(i);
                    let key = time_key(*offset, base, layer.id);
                    if !other_times.contains_key(&key) {
                        let at = match base {
                            TimeBase::Offset => shifted_by_seconds(t, *offset),
                            TimeBase::At => shifted_by_seconds(in_point, *offset),
                            TimeBase::Frames => crate::render::engine::motion::shifted_by_frames(view, t, *offset),
                        };
                        let Ok(then) = crate::picture::resolve::resolved_layers(view, at) else { continue };
                        let (Ok(texts), Ok(shapes)) = (collect_text_documents(view, &then, at), collect_shape_documents(view, &then, at)) else { continue };
                        other_times.insert(key, (at, then, texts, shapes));
                    }
                    // 別の時刻は別の流れの id で描く(lookbehind_layer_id)。文字と形の中身は id で引くので、その id でも引けるようにする。
                    if let Some((_, _, texts, shapes)) = other_times.get_mut(&key) {
                        let stream = lookbehind_layer_id(layer.id, key);
                        if let Some(document) = texts.get(&layer.id).cloned() { texts.insert(stream, document); }
                        if let Some(nodes) = shapes.get(&layer.id).cloned() { shapes.insert(stream, nodes); }
                    }
                }
            }
        }
        // 合体後の別時刻(SOURCE below / group / comp)は、本番を組む前に描いて写す。
        let mut composites: Composites = HashMap::new();
        for layer in resolved {
            let passes = crate::render::engine::translate::translate_effect_passes(&layer.effects).into_iter()
                .chain(crate::render::engine::translate::translate_plate_passes(&layer.after_effects));
            for pass in passes {
                for (i, offset) in pass.image_time_offsets().iter().enumerate() {
                    let source = pass.image_time_sources().get(i).copied().unwrap_or_default();
                    if source == crate::render::compositor::TimeSource::Own { continue; }
                    let key = (time_key(*offset, pass.image_time_base(i), layer.id), source, layer.id);
                    if composites.contains_key(&key) { continue; }
                    let Some((at, then, texts, shapes)) = other_times.get(&key.0) else { continue };
                    if let Some(picture) = self.composite_at(view, *at, then, texts, shapes, comp, layer.id, source) {
                        composites.insert(key, picture);
                    } else {
                        self.layer_failures.push(format!("{offset} 秒の合成({source:?})が間に合わなかった"));
                    }
                }
            }
        }
        if !composites.is_empty() { self.stamp_clock(view, t); }
        if self.feedback_namespace == 0 && !self.feedback_replaying { self.feedback_saw_composites = !composites.is_empty(); }
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
        // クリップの下地は絵が要る(網の形は切れない): 誰かが clip している層は、形でも絵に描く。
        self.clip_bases = resolved
            .iter()
            .filter(|layer| layer.clip_to_below)
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
                // Stencil / Silhouette は自分では描かない(切る相手へ matte として配られる)。
                || layer.blend_mode.is_stencil()
                || (layer.clip_to_below && layer.matte.is_none())
                || layer.placement.opacity <= 0.0
                || unseen.contains(&index)
            {
                continue;
            }

            let blend_mode = translate_blend_mode(layer.blend_mode)?;
            let mut frozen_padding = 0u32;
            let mut cut = Vec::new();
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
                        let screen = built.content.texture().is_none().then_some([comp.width, comp.height]);
                        self.stamp_feedback(&mut passes, member.id, member.copy, 0, screen);
                        copies.push(LayerWithPasses { layer: built, passes, pass_sources: Vec::new(), padding: 0, cut: Vec::new() });
                    }
                }
                if copies.is_empty() {
                    continue;
                }
                let owner = by_id.get(&group).copied();
                let plate_blend = translate_blend_mode(owner.map_or(layer.blend_mode, |g| g.blend_mode))?;
                let mut plate = self.bake_isolated_layers(comp, camera, copies, plate_blend, layer.placement, false)?;
                plate.placement.opacity = owner.map_or(1.0, |g| g.placement.opacity);
                let mut after = crate::render::engine::translate::translate_plate_passes(&layer.after_effects);
                let screen = after.iter().any(|p| p.reads_backdrop || p.reads_composite()).then(|| self.window_size(comp));
                self.stamp_feedback(&mut after, layer.id, layer.copy, 1, screen);
                (plate, after)
            } else if layer.after_effects.is_empty() && layer.averaged == 0 {
              // 凍った層は cache の絵で差し替え、素材の復号も効果の列も走らない(docs/freeze-and-flatten.md §2)。
              if let Some((built, padding)) = self.frozen_layer(view, layer, t, projection_camera, blend_mode)? {
                frozen_padding = padding;
                (built, Vec::new())
              } else {
                let Some(built) = self.build_layer_shared(&mut previous_build, layer, text_documents, shape_documents, t, comp, camera, projection_camera, blend_mode)? else {
                    continue;
                };
                cut = self.box_cut_in_comp(layer, &built);
                let mut passes = translate_effect_passes(&layer.effects);
                let screen = (built.content.texture().is_none() || passes.iter().any(|p| p.reads_backdrop || p.reads_composite())).then(|| self.window_size(comp));
                self.stamp_feedback(&mut passes, layer.id, layer.copy, 0, screen);
                // 補助viewが無いときだけ主カメラでカリングする。反射・matte・clipの入力は残す。
                // 解き手が動かす物は、書類の位置で間引かない(画面の外から入って来る)。
                // 解析で 1 枚だけ組んでいる間も間引かない(絵がどこに居ても透過を読みたい)。
                if !needs_auxiliary_views && !self.analysing && layer.matte.is_none() && !layer.clip_to_below && !self.blocks.moves(layer.id) && offscreen(comp, camera, &built, &passes) {
                    continue;
                }
                (built, passes)
              }
            } else {
                // 配置効果の下に効果が積まれた層: 同じ層の配置を全部 1 枚に合わせてから残りを掛ける。
                let end = index + resolved[index..].iter().take_while(|copy| copy.id == layer.id).count();
                skip_below = end;
                let mut copies = Vec::new();
                if layer.averaged > 0 {
                    copies = self.motion_blur_copies(&mut previous_build, &resolved[index..end], text_documents, shape_documents, t, comp, camera, projection_camera)?;
                } else {
                    for copy in &resolved[index..end] {
                        if let Some(built) = self.build_layer_shared(&mut previous_build, copy, text_documents, shape_documents, t, comp, camera, projection_camera, blend_mode)? {
                            let mut passes = translate_effect_passes(&copy.effects);
                            let screen = built.content.texture().is_none().then_some([comp.width, comp.height]);
                            self.stamp_feedback(&mut passes, copy.id, copy.copy, 0, screen);
                            copies.push(LayerWithPasses { layer: built, passes, pass_sources: Vec::new(), padding: 0, cut: Vec::new() });
                        }
                    }
                }
                if copies.is_empty() {
                    continue;
                }
                let mut plate_placement = layer.placement;
                if layer.averaged > 0 { plate_placement.opacity = 1.0; }
                let plate = self.bake_isolated_layers(comp, camera, copies, blend_mode, plate_placement, layer.averaged > 0)?;
                let mut after = crate::render::engine::translate::translate_plate_passes(&layer.after_effects);
                let screen = after.iter().any(|p| p.reads_backdrop || p.reads_composite()).then(|| self.window_size(comp));
                self.stamp_feedback(&mut after, layer.id, layer.copy, 1, screen);
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
                let union = self.bake_isolated_layers(comp, camera, indices.iter().map(|&i| layers[i].clone()).collect(), blend, placement, false)?;
                match self.clip_onto_base(LayerWithPasses { layer: union, passes: Vec::new(), pass_sources: Vec::new(), padding: 0, cut: Vec::new() }, &built, &passes)? {
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
                    // マットは相手の層の最終の絵(AE の track matte と同じ): 配置の写しが複数あるか、中身が絵でない(形の輪郭)なら
                    // 写しを全部 comp 大の 1 枚に焼いてから使う。
                    let copies: Vec<&ResolvedLayer> = resolved.iter().filter(|l| l.id == matte.layer && !l.ghost).collect();
                    let source_layer = if copies.len() > 1 || source_layer.content.texture().is_none() {
                        let mut plate = Vec::new();
                        for copy in copies {
                            if let Some(built) = self.build_layer_shared(&mut previous_build, copy, text_documents, shape_documents, t, comp, camera, projection_camera, CompositeBlendMode::Normal)? {
                                let passes = translate_effect_passes(&copy.effects);
                                plate.push(LayerWithPasses { layer: built, passes, pass_sources: Vec::new(), padding: 0, cut: Vec::new() });
                            }
                        }
                        self.bake_isolated_layers(comp, camera, plate, CompositeBlendMode::Normal, source.placement, false)?
                    } else {
                        self.apply_effects_before_matte(comp, camera, source_layer, &source_passes)?
                    };
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
            let pass_sources = self.sources_at_other_times(&composites, &passes, layer.id, &other_times, (resolved, text_documents, shape_documents, t), comp, camera, projection_camera);
            layers.push(LayerWithPasses {
                pass_sources,
                layer: final_layer,
                passes,
                padding: frozen_padding,
                cut,
            });
        }

        if removed.is_empty() {
            return Ok(layers);
        }
        self.drawn_layers -= removed.len();
        Ok(layers.into_iter().enumerate().filter(|(i, _)| !removed.contains(i)).map(|(_, l)| l).collect())
    }

    /// 平面へ収める。3D の素材を comp の絵へ一度焼き、以後は板として扱う
    /// (裁定 2026-08-30「平面に収めるのは選択肢」)。焼いた層にも blend・matte・
    /// エフェクトは今まで通り効く。
    pub(in crate::engine) fn flatten_if_asked(
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
            padding: 0,
            cut: Vec::new(),
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
            shadow: layer.shadow,
            outline: layer.outline,
            frame: None,
        })
    }

    /// Track Matte は、Effect後の層をsourceのcoverageで切り、その結果を他層へblendする。
    /// EffectをMatte後のcomp大textureへ掛けると、0-input Effectが透明域を再び塗るため、
    /// 既存のlocal-texture Effect経路をここで一度だけcomp座標へ収めてからMatteへ渡す。
    pub(in crate::engine) fn apply_effects_before_matte(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layer: Layer,
        passes: &[EffectPass],
    ) -> Result<Layer, EngineError> {
        // 形・文字の輪郭のままの層(絵でない)は、切る前に 1 枚の絵に焼く(matte は絵同士で掛ける)。
        if passes.is_empty() && layer.content.texture().is_some() {
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
        self.bake_isolated_layers(comp, camera, vec![LayerWithPasses { layer, passes: passes.to_vec(), pass_sources: Vec::new(), padding: 0, cut: Vec::new() }], blend, placement, false)
    }

    /// 層(または 1 つの層の配置たち)を comp 大の 1 枚へ焼く。`average` なら写しを足す
    /// (Motion Blur: 各写しの不透明度は 1/枚数なので、足すと平均になる)。
    pub(super) fn bake_isolated_layers(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        mut sources: Vec<LayerWithPasses>,
        output_blend: CompositeBlendMode,
        placement: crate::doc::core::LayerPlacement,
        average: bool,
    ) -> Result<Layer, EngineError> {
        // BlendはMatteでcoverageを得た後、作品の下層との間に一度だけ掛ける。
        for source in &mut sources {
            source.layer.blend_mode = if average { CompositeBlendMode::Add } else { CompositeBlendMode::Normal };
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
            shadow: sources.iter().map(|s| s.layer.shadow).fold(0.0, f32::max),
            outline: sources.iter().map(|s| s.layer.outline).max().unwrap_or(0),
            frame: None,
        })
    }

    /// 配置効果の複製は素材と mask が同じなので、直前に組んだ 1 枚を置き直すだけにする。
    /// 平面化は置き場所で絵が変わるので共有しない。
    #[allow(clippy::too_many_arguments)]
    pub(in crate::engine) fn build_layer_shared(
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
                let mut shared = Layer { placement: layer.placement, blend_mode, ..built.clone() };
                self.attach_block(layer, &mut shared, comp);
                return Ok(Some(shared));
            }
        }
        let built = self.build_layer(layer, text_documents, shape_documents, t, comp, camera, projection_camera, blend_mode)?;
        *previous = built.clone().map(|built| (layer.id, layer.source_frame, built));
        Ok(built.map(|mut built| { self.attach_block(layer, &mut built, comp); built }))
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
        // Group の背景(と影)の形は、箱の左上が素材座標の 1 に来る前提で並ぶ子と揃う。影が箱の外へ出ると形の画布の原点がずれるので、その分だけ置き場所を戻す。
        let mut placement = layer.placement;
        if layer.source == LayerSource::Group {
            if let Some(Ok(Some(canvas))) = shape_documents.get(&layer.id).map(|shapes| crate::picture::shapes_ops::content_canvas(shapes)) {
                let shift = glam::vec2(canvas.origin_x as f32 - 1.0, canvas.origin_y as f32 - 1.0);
                if shift != glam::Vec2::ZERO {
                    placement.transform = placement.transform * glam::Affine2::from_translation(-shift);
                    placement.world_transform = placement.world_transform.map(|w| w * glam::Affine3A::from_translation((-shift).extend(0.0)));
                }
            }
        }
        let mut built = Layer {
            content, size: layer_size(layer, natural), placement,
            projection: layer.projection, projection_camera, blend_mode,
            shading: Default::default(), displace: translate_point_displace(&layer.effects),
            clip: translate_clip(&layer.effects), shadow: translate_cast_shadow(&layer.effects), outline: self.outline_id(layer.id),
            frame,
        };
        let uses_material = self.compositor.catalog.descriptors.iter().any(|d| matches!(d.stage, crate::render::compositor::EffectStage::Warp | crate::render::compositor::EffectStage::Field) && layer.effects.iter().any(|e| e.plugin_id == d.plugin_id));
        // 祖先の箱の切りは、ブロックが動かす層には素材の枠で掛けない — ずれの後に箱の枠で掛ける(`cut_moved_in_their_boxes`)。
        let own: Vec<ResolvedMask>;
        let masks: &[ResolvedMask] = if self.cut_after_motion(layer) {
            own = layer.masks.iter().filter(|m| m.frame == crate::doc::store::MaskFrame::Layer).cloned().collect();
            &own
        } else {
            &layer.masks
        };
        let masks_applied = (uses_material || frame.is_some()) && built.content.texture().is_some();
        if masks_applied { built = self.apply_masks_to_layer(built, masks, natural, frame)?; }
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
        Ok(Some(if masks_applied { built } else { self.apply_masks_to_layer(built, masks, natural, frame)? }))
    }

    pub(in crate::engine) fn apply_masks_to_layer(
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
        let canvas = crate::picture::shapes_ops::Canvas {
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

    /// ブロックのずれで動く層を祖先の箱で切る(`MaskFrame::Box`): ずれを書いた後に comp 大へ焼き、箱の枠の切りを掛ける。
    /// 切りは箱に留まり、ずれは中身だけを動かす(CSS の overflow: clip / clip-path は要素の箱に掛かり、中で transform した
    /// 子孫は箱で切れる)。
    pub(super) fn cut_moved_in_their_boxes(&mut self, comp: CompSpec, camera: ResolvedCamera, layers: &mut [LayerWithPasses]) -> Result<(), EngineError> {
        for entry in layers.iter_mut().filter(|entry| !entry.cut.is_empty()) {
            let cut = std::mem::take(&mut entry.cut);
            let source = entry.clone();
            let (blend, placement) = (source.layer.blend_mode, source.layer.placement);
            let baked = self.bake_isolated_layers(comp, camera, vec![source], blend, placement, false)?;
            let layer = self.apply_masks_to_layer(baked, &cut, [comp.width as f32, comp.height as f32], None)?;
            *entry = LayerWithPasses { layer, passes: Vec::new(), pass_sources: Vec::new(), padding: 0, cut: Vec::new() };
        }
        Ok(())
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
