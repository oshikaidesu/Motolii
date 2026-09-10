use std::collections::{HashMap, HashSet};

use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::doc::store::{
    LayerId, LayerSource, RationalTime, ResolvedLayer, ResolvedMask, ShapeNode, StoreView,
    TextDocument,
};
use crate::render::compositor::{
    BlendMode as CompositeBlendMode, EffectPass, Layer, LayerContent, LayerWithPasses,
};

use crate::render::engine::translate::{
    translate_blend_mode, translate_clip, translate_effect_passes, translate_matte_mode, translate_point_displace,
};
use crate::render::engine::{Engine, EngineError};

impl Engine {
    pub(crate) fn render_with_camera_override(
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
        let shape_documents = collect_shape_documents(view, &resolved)?;
        self.compositor.measurement.resolve_us = frame_start.elapsed().as_micros() as u64;
        let layer_start = std::time::Instant::now();
        let layers = self.layers_from_resolved(
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

    fn layers_from_resolved(
        &mut self,
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
        let mut layers: Vec<LayerWithPasses> = Vec::with_capacity(resolved.len() + 1);
        // 層 id → layers の添字(通り抜けの配置なら複製の数だけ)。クリップの下地探しに使う。
        let mut contributions: HashMap<LayerId, Vec<usize>> = HashMap::new();

        let by_id: HashMap<LayerId, &ResolvedLayer> =
            resolved.iter().map(|layer| (layer.id, layer)).collect();
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
        let mut entry_copy: Vec<u32> = Vec::with_capacity(resolved.len() + 1);
        let mut removed: HashSet<usize> = HashSet::new();
        for (index, layer) in resolved.iter().enumerate() {
            if index < skip_below
                || matte_sources.contains(&layer.id)
                || (layer.clip_to_below && layer.matte.is_none())
                || layer.placement.opacity <= 0.0
            {
                continue;
            }

            let blend_mode = translate_blend_mode(layer.blend_mode)?;
            let (built, passes) = if layer.after_effects.is_empty() {
                let Some(built) = self.build_layer_shared(&mut previous_build, layer, text_documents, shape_documents, t, comp, camera, projection_camera, blend_mode)? else {
                    continue;
                };
                let passes = translate_effect_passes(&layer.effects);
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
                        copies.push(LayerWithPasses { layer: built, passes: translate_effect_passes(&copy.effects) });
                    }
                }
                if copies.is_empty() {
                    continue;
                }
                let plate = self.bake_isolated_layers(comp, camera, copies, blend_mode, layer.placement)?;
                (plate, translate_effect_passes(&layer.after_effects))
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
                match self.clip_onto_base(LayerWithPasses { layer: union, passes: Vec::new() }, &built, &passes)? {
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
                    let (source_content, source_natural) = self.texture_for_resolved(
                        source,
                        text_documents,
                        shape_documents,
                        t,
                        comp,
                    )?;
                    let Some(source_content) = source_content else {
                        continue;
                    };
                    let source_blend = translate_blend_mode(source.blend_mode)?;
                    let source_layer = Layer {
                        content: source_content,
                        size: layer_size(source, source_natural),
                        placement: source.placement,
                        projection: source.projection,
                        projection_camera,
                        blend_mode: source_blend,
                        shading: Default::default(),
                        displace: Default::default(),
                        clip: None,
                    };
                    let source_layer =
                        self.flatten_if_asked(comp, camera, source_layer, source.flatten)?;
                    let source_layer = self.apply_masks_to_layer(source_layer, &source.masks)?;
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
            layers.push(LayerWithPasses {
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
        comp: CompSpec,
        background: [f32; 4],
        camera: ResolvedCamera,
        t: RationalTime,
        resolved: &[ResolvedLayer],
        text_documents: &HashMap<LayerId, TextDocument>,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), EngineError> {
        self.render_resolved_to_texture_with_shapes(
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
        comp: CompSpec,
        background: [f32; 4],
        camera: ResolvedCamera,
        t: RationalTime,
        resolved: &[ResolvedLayer],
        text_documents: &HashMap<LayerId, TextDocument>,
        shape_documents: &HashMap<LayerId, Vec<ShapeNode>>,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), EngineError> {
        let layers =
            self.layers_from_resolved(comp, camera, camera, t, resolved, text_documents, shape_documents)?;
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
        let shape_documents = collect_shape_documents(view, &resolved)?;
        self.render_resolved_to_texture_with_shapes(
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
        let shape_documents = collect_shape_documents(view, &resolved)?;
        let layers = self.layers_from_resolved(
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
    ) -> Result<(), EngineError> {
        let composition = view
            .composition()
            .map_err(|e| EngineError::Store(e.to_string()))?
            .ok_or(EngineError::NoComposition)?;
        let comp = composition.spec();
        let resolved = view
            .resolved_layers(t)
            .map_err(|e| EngineError::Store(e.to_string()))?;
        let text_documents = collect_text_documents(view, &resolved, t)?;
        let shape_documents = collect_shape_documents(view, &resolved)?;
        let layers = self.layers_from_resolved(
            comp,
            camera,
            self.resolve_camera_in(view, &resolved, t)?,
            t,
            &resolved,
            &text_documents,
            &shape_documents,
        )?;
        let background_color = if include_background {
            composition.background
        } else {
            crate::render::compositor::NO_BACKGROUND
        };
        Ok(self
            .compositor
            .render_into(target, comp, camera, &layers, background_color)?)
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
        self.bake_isolated_layers(comp, camera, vec![LayerWithPasses { layer, passes: passes.to_vec() }], blend, placement)
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
            if *id == layer.id && *frame == layer.source_frame && layer.copy > 0 && !layer.flatten {
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
        let (content, natural) =
            self.texture_for_resolved(layer, text_documents, shape_documents, t, comp)?;
        let Some(content) = content else {
            return Ok(None);
        };
        let shading = if matches!(content, crate::render::compositor::LayerContent::Model(_) | crate::render::compositor::LayerContent::Texture(_)) {
            match self.compositor.surface_shading(&layer.effects) {
                Ok(shading) => shading,
                Err(reason) => {
                    self.layer_failures.push(format!("layer {} の hook を組めない: {reason}", layer.id.0));
                    Default::default()
                }
            }
        } else {
            Default::default()
        };
        let built = self.flatten_if_asked(
            comp,
            camera,
            Layer {
                content,
                size: layer_size(layer, natural),
                placement: layer.placement,
                projection: layer.projection,
                projection_camera,
                blend_mode,
                shading,
                displace: translate_point_displace(&layer.effects),
                clip: translate_clip(&layer.effects),
            },
            layer.flatten,
        )?;
        Ok(Some(self.apply_masks_to_layer(built, &layer.masks)?))
    }

    fn apply_masks_to_layer(
        &mut self,
        mut layer: Layer,
        masks: &[ResolvedMask],
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
        let coverage = crate::render::engine::mask::fold_masks(masks, &canvas)?;
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
) -> Result<HashMap<LayerId, Vec<ShapeNode>>, EngineError> {
    let mut documents = HashMap::new();
    for layer in resolved {
        if layer.source == LayerSource::Shape {
            let shapes = view
                .shapes(layer.id)
                .map_err(|e| EngineError::Store(e.to_string()))?;
            documents.insert(layer.id, shapes);
        }
    }
    Ok(documents)
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
        placement, property, Composition, Document, EffectId, EffectInstance, Fps, Intent,
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
            Intent::SetConstant { layer: group, property: PropertyId::effect_param(EffectId(0), "subject").unwrap(), value: Value::F64(1.0) },
            Intent::SetConstant { layer: group, property: PropertyId::effect_param(EffectId(0), "position_each").unwrap(), value: Value::Vec2([10.0, 0.0]) },
        ]).unwrap();
        add_file_layer(&mut inside, 11, 0, &red, Some(group), false);
        add_file_layer(&mut inside, 12, 1, &green, Some(group), true);
        let pixels = engine.render_frame(&inside.view(), RationalTime::ZERO).unwrap();
        assert_eq!(covered(&pixels), 2 * (DOT * DOT) as usize, "each copy carries its own clipped lyric, nothing leaks outside the copies");
        assert!(red_floor(&pixels) >= 120, "inside a copy the clip is drawn once, got red {}", red_floor(&pixels));
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
