use crate::doc::core::{CompSpec, LayerPlacement, ResolvedCamera};
use crate::doc::store::LayerId;
use crate::frame_graph::{SceneContentValue, SceneImageSourceValue, SceneLayerValue, SceneValue, SolverPlanValue};
use crate::render::compositor::{BlendMode as CompositeBlendMode, Layer, LayerWithPasses};
use crate::render::engine::{Engine, EngineError};

#[derive(Clone)]
pub(super) struct GpuSceneValue {
    pub layers: Vec<LayerWithPasses>,
    pub layer_ids: Vec<LayerId>,
}

impl Engine {
    pub(super) fn prepare_gpu_scene_with_solver(
        &mut self,
        scene: &SceneValue,
        solver: &SolverPlanValue,
        comp: CompSpec,
        projection_camera: ResolvedCamera,
        time: crate::doc::core::RationalTime,
        fps: crate::doc::store::Fps,
    ) -> Result<GpuSceneValue, EngineError> {
        let physics_overlays: std::collections::HashSet<LayerId> = self.overlay_frames.iter()
            .filter_map(|(layer, frame)| frame.physics.then_some(*layer))
            .collect();
        if physics_overlays.is_empty() {
            // Culling is execution planning only: semantic SceneValue remains
            // untouched and cacheable. The planner may omit only simple media
            // contributions that cannot feed analysis/matte/solver work.
            let planned = self.plan_frame_graph_scene(scene, solver, comp, projection_camera);
            let scene = planned.as_ref().unwrap_or(scene);
            let mut prepared = self.prepare_gpu_scene(scene, comp, projection_camera)?;
            self.prepare_frame_graph_blocks(scene, solver, comp, time, fps, &mut prepared)?;
            return Ok(prepared);
        }

        // Physics Trace reads the solver's current-frame state. Build the
        // ordinary scene first, solve motion/physics, then lower the complete
        // scene once the overlay can read those results.
        let base_scene = SceneValue {
            layers: scene.layers.iter()
                .filter(|layer| !physics_overlays.contains(&layer.layer))
                .cloned()
                .collect(),
        };
        let mut base = self.prepare_gpu_scene(&base_scene, comp, projection_camera)?;
        self.prepare_frame_graph_blocks(scene, solver, comp, time, fps, &mut base)?;

        let mut prepared = self.prepare_gpu_scene(scene, comp, projection_camera)?;
        for (id, layer) in prepared.layer_ids.iter().copied().zip(prepared.layers.iter_mut()) {
            self.attach_block_id(id, &mut layer.layer, comp);
        }
        Ok(prepared)
    }

    fn plan_frame_graph_scene(
        &self,
        scene: &SceneValue,
        solver: &SolverPlanValue,
        comp: CompSpec,
        camera: ResolvedCamera,
    ) -> Option<SceneValue> {
        // Temporal/named image dependencies already carry their own source
        // values, but conservatively keep the full scene while such auxiliary
        // views exist. The same rule applies to Block/Follow/physics work.
        if scene.layers.iter().any(|layer| !layer.image_sources.is_empty()) {
            return None;
        }
        let block_ids: std::collections::HashSet<&str> = self.compositor.catalog.definitions.iter()
            .filter(|definition| definition.manifest.stage == crate::render::compositor::effects::isf::IsfStage::Block)
            .map(|definition| definition.plugin_id())
            .collect();
        if scene.layers.iter().any(|layer| layer.effects.iter().any(|effect| block_ids.contains(effect.plugin_id.as_str()))) {
            return None;
        }
        if solver.layers.values().any(|value| value.relation != crate::frame_graph::RelationValue::default()) {
            return None;
        }

        let matte_sources: std::collections::HashSet<LayerId> = scene.layers.iter()
            .filter_map(|layer| layer.matte.map(|matte| matte.layer))
            .collect();
        let screen = [0.0f32, 0.0, comp.width as f32, comp.height as f32];
        let mut covers: Vec<[f32; 4]> = Vec::new();
        let mut omitted = std::collections::HashSet::new();

        for (index, layer) in scene.layers.iter().enumerate().rev() {
            let feeds_others = matte_sources.contains(&layer.layer) || layer.matte.is_some() || layer.clip_to_below;
            let SceneContentValue::Media { source, .. } = &layer.content else { continue };
            if crate::render::media::is_still_image_path(&source.path)
                || crate::render::media::is_audio_path(&source.path)
                || crate::render::media::is_mesh_path(&source.path)
                || crate::render::media::is_point_cloud_path(&source.path)
            {
                continue;
            }
            let Some(info) = self.probes.get(&source.path) else { continue };
            if info.rotation != 0 { continue; }
            let size = [info.width as f32, info.height as f32];
            let corners = crate::doc::core::projected_screen_corners(
                comp,
                camera,
                camera,
                layer.projection,
                layer.transform.spatial,
                [0.0, 0.0, 0.0],
                [size[0], size[1], 0.0],
            );
            if corners.iter().any(|corner| !corner.is_finite()) { continue; }
            let rect = corners.iter().fold([f32::MAX, f32::MAX, f32::MIN, f32::MIN], |rect, corner| {
                [rect[0].min(corner.x), rect[1].min(corner.y), rect[2].max(corner.x), rect[3].max(corner.y)]
            });
            let epsilon = 0.5;
            let axis_aligned = corners.iter().all(|corner| {
                ((corner.x - rect[0]).abs() < epsilon || (corner.x - rect[2]).abs() < epsilon)
                    && ((corner.y - rect[1]).abs() < epsilon || (corner.y - rect[3]).abs() < epsilon)
            });
            let plain = layer.effects.is_empty()
                && layer.after_effects.is_empty()
                && layer.masks.is_empty()
                && !layer.flatten;
            let offscreen = rect[2] < screen[0] || rect[3] < screen[1] || rect[0] > screen[2] || rect[1] > screen[3];
            let covered = covers.iter().any(|cover| {
                rect[0] >= cover[0] && rect[1] >= cover[1] && rect[2] <= cover[2] && rect[3] <= cover[3]
            });
            if !feeds_others && plain && (offscreen || covered) {
                omitted.insert(index);
                continue;
            }
            let opaque = axis_aligned
                && plain
                && layer.opacity >= 1.0
                && layer.blend == crate::doc::store::BlendMode::Normal
                && layer.matte.is_none()
                && !layer.clip_to_below
                && !layer.ghost;
            if opaque { covers.push(rect); }
        }

        (!omitted.is_empty()).then(|| SceneValue {
            layers: scene.layers.iter().enumerate()
                .filter(|(index, _)| !omitted.contains(index))
                .map(|(_, layer)| layer.clone())
                .collect(),
        })
    }

    pub(super) fn prepare_gpu_scene(&mut self, scene: &SceneValue, comp: CompSpec, projection_camera: ResolvedCamera) -> Result<GpuSceneValue, EngineError> {
        #[derive(Clone, Copy)]
        struct Entry {
            layer: LayerId,
            matte: Option<crate::doc::store::Matte>,
            clip_to_below: bool,
            stencil: bool,
        }

        let clip_bases: std::collections::HashSet<_> = scene.layers.iter()
            .filter(|layer| layer.clip_to_below)
            .filter_map(|layer| layer.matte.map(|matte| matte.layer))
            .collect();

        let mut layers = Vec::with_capacity(scene.layers.len());
        let mut entries = Vec::with_capacity(scene.layers.len());
        for source in &scene.layers {
            let key = LayerId(source.content_key.map_or(0, |key| key.as_u64()));
            let solid = crate::render::engine::translate::translate_solid(&source.effects)
                .map(|solid| if solid.depth > 0.0 { solid } else { crate::render::compositor::extrude::Solid { depth: source.depth, ..solid } })
                .unwrap_or(crate::render::compositor::extrude::Solid { depth: source.depth, bevel: None });
            let flat = solid.extent() <= 0.0;
            let force_picture = clip_bases.contains(&source.layer) || !flat;
            let overlay_content = source.effects.iter().any(|effect| crate::extensions::overlay::is_track_overlay(&effect.plugin_id))
                .then(|| self.overlay_content(source.layer, comp))
                .transpose()?
                .flatten();
            let frozen = self.frame_graph_frozen_content(source);
            let (content, natural, frozen_padding, frozen_frame, frozen_hit) = if let Some((content, natural, padding, frame)) = frozen {
                (Some(content), natural, padding, frame, true)
            } else if let Some((content, natural)) = overlay_content {
                (Some(content), natural, 0, None, false)
            } else {
                let (content, natural) = match &source.content {
                SceneContentValue::None => continue,
                SceneContentValue::Text(text) if force_picture => self.shape_texture_from_shapes(&text.shapes(), key, false, 0.05, comp, None, true)?,
                SceneContentValue::Text(text) => self.text_texture_from_shapes(&text.shapes(), key, comp)?,
                SceneContentValue::Shape(shapes) => {
                    let stretched;
                    let shapes = if source.shape_stretch != [1.0, 1.0] {
                        stretched = crate::picture::shapes_ops::stretch_outline(shapes, source.shape_stretch);
                        stretched.as_slice()
                    } else {
                        shapes.as_slice()
                    };
                    self.shape_texture_from_shapes(shapes, key, !force_picture, 0.05, comp, None, source.shape_stretch == [1.0, 1.0])?
                },
                SceneContentValue::Material(material) => self.mesh_content_for(&material.source.path, comp)?,
                SceneContentValue::Media { source: media, time } => {
                    if source.environment && crate::render::media::is_still_image_path(&media.path) {
                        self.environment_content_for(&media.path)?
                    } else {
                        self.file_content_for(&media.path, *time, source.layer, comp)?
                    }
                }
                SceneContentValue::Particles(value) => {
                    let frame = super::ParticleFrame::from_particles(&value.particles, value.turbulence, value.links);
                    let natural = [frame.bounds.max[0].max(1.0), frame.bounds.max[1].max(1.0)];
                    (
                        Some(crate::render::compositor::LayerContent::Cloud {
                            positions: frame.positions,
                            colors: frame.colors,
                            bounds: frame.bounds,
                            point_size: 1.0,
                            sizes: Some(frame.sizes),
                            sprites: true,
                            links: frame.links,
                        }),
                        natural,
                    )
                }
                SceneContentValue::Plate(plate) => {
                    let nested = SceneValue {
                        layers: plate.members.iter().filter_map(|member| member.layer.clone()).collect(),
                    };
                    let prepared = self.prepare_gpu_scene(&nested, comp, projection_camera)?;
                    if prepared.layers.is_empty() {
                        (None, [comp.width as f32, comp.height as f32])
                    } else {
                        let placement = LayerPlacement {
                            transform: glam::Affine2::IDENTITY,
                            world_transform: None,
                            opacity: 1.0,
                            order: i32::from(source.order),
                            z: 0.0,
                            rotation_x: 0.0,
                            rotation_y: 0.0,
                            plane: None,
                        };
                        let baked = self.bake_isolated_layers(
                            comp,
                            projection_camera,
                            prepared.layers,
                            CompositeBlendMode::Normal,
                            placement,
                            plate.average,
                        )?;
                        (Some(baked.content), baked.size)
                    }
                }
            };
                (content, natural, 0, None, false)
            };
            let Some(mut content) = content else { continue };
            if let crate::render::compositor::LayerContent::Texture(texture) = &content {
                if !flat && source.projection != crate::doc::store::LayerProjection::TwoD && source.masks.is_empty() {
                    content = self.frame_graph_extruded_content(source, texture.clone(), natural, comp, solid)?;
                }
            }
            if !frozen_hit && !source.image_sources.is_empty() {
                content = match &content {
                    crate::render::compositor::LayerContent::Texture(texture) => self.compositor.snapshot_texture(texture)
                        .map(crate::render::compositor::LayerContent::Texture)
                        .unwrap_or_else(|| content.clone()),
                    crate::render::compositor::LayerContent::LinearTexture(texture) => self.compositor.snapshot_texture(texture)
                        .map(crate::render::compositor::LayerContent::LinearTexture)
                        .unwrap_or_else(|| content.clone()),
                    _ => content,
                };
            }
            let placement = LayerPlacement {
                transform: source.transform.affine,
                world_transform: Some(source.transform.spatial),
                order: i32::from(source.order),
                opacity: source.opacity,
                z: source.transform.spatial.translation.z,
                rotation_x: 0.0,
                rotation_y: 0.0,
                plane: None,
            };
            let (passes, pass_sources) = if frozen_hit {
                (Vec::new(), Vec::new())
            } else {
                let mut direct_passes = crate::render::engine::translate::translate_effect_passes(&source.effects);
                let direct_screen = (
                    content.texture().is_none()
                        || direct_passes.iter().any(|pass| pass.reads_backdrop || pass.reads_composite())
                ).then_some([comp.width, comp.height]);
                self.stamp_feedback(&mut direct_passes, source.layer, source.instance, 0, direct_screen);

                let mut plate_passes = crate::render::engine::translate::translate_plate_passes(&source.after_effects);
                let plate_screen = plate_passes.iter()
                    .any(|pass| pass.reads_backdrop || pass.reads_composite())
                    .then_some([comp.width, comp.height]);
                self.stamp_feedback(&mut plate_passes, source.layer, source.instance, 1, plate_screen);

                (
                    direct_passes.into_iter().chain(plate_passes).collect(),
                    self.frame_graph_image_sources(&source.image_sources, comp, projection_camera)?,
                )
            };
            // Stencil/Silhouette are matte sources, never compositor blend modes.
            let blend_mode = if source.blend.is_stencil() {
                CompositeBlendMode::Normal
            } else {
                crate::render::engine::translate::translate_blend_mode(source.blend)?
            };
            let layer = Layer {
                content,
                size: natural,
                placement,
                projection: source.projection,
                projection_camera,
                blend_mode,
                shading: self.compositor.surface_shading_for(&source.effects, false).map_err(EngineError::Store)?,
                displace: crate::render::engine::translate::translate_point_displace(&source.effects),
                clip: crate::render::engine::translate::translate_clip(&source.effects),
                shadow: crate::render::engine::translate::translate_cast_shadow(&source.effects),
                outline: self.outline_id(source.layer),
                frame: frozen_frame,
            };
            let layer = self.apply_masks_to_layer(layer, &source.masks, natural, frozen_frame)?;
            let source_tick = match &source.content {
                SceneContentValue::Media { time, .. } => (time.as_seconds_f64() * 1_000_000.0).round() as i64,
                _ => 0,
            };
            let layer = self.apply_material_domains_semantic(
                layer,
                source.layer,
                &source.effects,
                matches!(source.source, crate::doc::store::LayerSource::File { .. }),
                source_tick,
                natural,
                frozen_frame,
            )?;
            let layer = self.flatten_if_asked(comp, projection_camera, layer, source.flatten)?;
            layers.push(LayerWithPasses { layer, passes, padding: frozen_padding, pass_sources, cut: Vec::new() });
            entries.push(Entry {
                layer: source.layer,
                matte: source.matte,
                clip_to_below: source.clip_to_below,
                stencil: source.blend.is_stencil(),
            });
        }

        let by_id: std::collections::HashMap<_, _> = entries.iter().enumerate()
            .map(|(index, entry)| (entry.layer, index))
            .collect();
        let mut removed = vec![false; layers.len()];

        // Clipping is source-atop onto the visible base. The upper contribution
        // never reaches the final scene as a separate layer.
        for index in 0..layers.len() {
            let entry = entries[index];
            if !entry.clip_to_below || entry.stencil {
                continue;
            }
            let Some(base_id) = entry.matte.map(|matte| matte.layer) else {
                removed[index] = true;
                continue;
            };
            let Some(base_index) = by_id.get(&base_id).copied() else {
                removed[index] = true;
                continue;
            };
            match self.clip_onto_base(layers[base_index].clone(), &layers[index].layer, &layers[index].passes)? {
                Some(clipped) => layers[base_index] = clipped,
                None => self.layer_failures.push(format!(
                    "layer {} clips to a base without a texture (point cloud / model bases are not clippable)",
                    entry.layer.0
                )),
            }
            removed[index] = true;
        }

        // Track mattes and stencils consume their source. A missing source means
        // the target has no coverage and therefore contributes nothing.
        let matte_sources: std::collections::HashSet<_> = scene.layers.iter()
            .filter(|entry| !entry.clip_to_below)
            .filter_map(|entry| entry.matte.map(|matte| matte.layer))
            .collect();

        for index in 0..layers.len() {
            if removed[index] || entries[index].clip_to_below {
                continue;
            }
            let Some(matte) = entries[index].matte else { continue };
            let Some(source_index) = by_id.get(&matte.layer).copied() else {
                removed[index] = true;
                continue;
            };
            let target = self.apply_effects_before_matte(
                comp,
                projection_camera,
                layers[index].layer.clone(),
                &layers[index].passes,
            )?;
            let source = self.apply_effects_before_matte(
                comp,
                projection_camera,
                layers[source_index].layer.clone(),
                &layers[source_index].passes,
            )?;
            let layer = self.apply_matte(comp, projection_camera, &target, &source, matte.mode)?;
            layers[index] = LayerWithPasses {
                layer,
                passes: Vec::new(),
                padding: 0,
                pass_sources: Vec::new(),
                cut: Vec::new(),
            };
        }

        let kept: Vec<_> = layers.into_iter().enumerate()
            .filter(|(index, _)| {
                !removed[*index]
                    && !entries[*index].stencil
                    && !matte_sources.contains(&entries[*index].layer)
            })
            .map(|(index, layer)| (entries[index].layer, layer))
            .collect();
        let layer_ids = kept.iter().map(|(layer, _)| *layer).collect();
        let layers = kept.into_iter().map(|(_, layer)| layer).collect();

        Ok(GpuSceneValue { layers, layer_ids })
    }


    fn frame_graph_extruded_content(
        &mut self,
        source: &crate::frame_graph::SceneLayerValue,
        texture: crate::render::compositor::GpuTexture2D,
        natural: [f32; 2],
        comp: CompSpec,
        solid: crate::render::compositor::extrude::Solid,
    ) -> Result<crate::render::compositor::LayerContent, EngineError> {
        use std::hash::{Hash, Hasher};

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        texture.handle.hash(&mut hasher);
        solid.hash_key(&mut hasher);
        source.shape_stretch[0].to_bits().hash(&mut hasher);
        source.shape_stretch[1].to_bits().hash(&mut hasher);
        let key = hasher.finish();
        if let Some((cached, model)) = self.extrusions.get(&source.layer) {
            if *cached == key {
                return Ok(crate::render::compositor::LayerContent::Model(model.clone()));
            }
        }

        let rectangle = |size: [f32; 2]| {
            let vertex = |x: f32, y: f32| re_renderer::renderer::PathVertex {
                point: glam::vec2(x, y),
                in_tangent: glam::Vec2::ZERO,
                out_tangent: glam::Vec2::ZERO,
            };
            vec![(
                vec![re_renderer::renderer::PathContour {
                    closed: true,
                    vertices: vec![
                        vertex(0.0, 0.0),
                        vertex(size[0], 0.0),
                        vertex(size[0], size[1]),
                        vertex(0.0, size[1]),
                    ],
                }],
                re_renderer::renderer::PathFillRule::NonZero,
            )]
        };

        let outlines = match &source.content {
            crate::frame_graph::SceneContentValue::Text(text) => {
                let shapes = text.shapes();
                match crate::picture::shapes_ops::content_canvas(&shapes)? {
                    Some(canvas) => crate::render::compositor::paths::outlines(&shapes, &canvas)?,
                    None => Vec::new(),
                }
            }
            crate::frame_graph::SceneContentValue::Shape(shapes) => {
                let stretched;
                let shapes = if source.shape_stretch != [1.0, 1.0] {
                    stretched = crate::picture::shapes_ops::stretch_outline(shapes, source.shape_stretch);
                    stretched.as_slice()
                } else {
                    shapes.as_slice()
                };
                match crate::picture::shapes_ops::content_canvas(shapes)? {
                    Some(canvas) => crate::render::compositor::paths::outlines(shapes, &canvas)?,
                    None => Vec::new(),
                }
            }
            _ => rectangle(natural),
        };

        let Some(model) = self.compositor.extrude_model(&outlines, texture.clone(), natural, solid)? else {
            return Ok(crate::render::compositor::LayerContent::Texture(texture));
        };
        let model = std::sync::Arc::new(model);
        self.extrusions.insert(source.layer, (key, model.clone()));
        let _ = comp;
        Ok(crate::render::compositor::LayerContent::Model(model))
    }

    fn frame_graph_frozen_content(
        &mut self,
        source: &SceneLayerValue,
    ) -> Option<(
        crate::render::compositor::LayerContent,
        [f32; 2],
        u32,
        Option<crate::render::compositor::effects::vism::ImageFrame>,
    )> {
        if !source.freeze_eligible || source.ghost || source.instance != 0 || self.freezing == Some(source.layer) {
            return None;
        }
        let comp_frame = self.compositor.clock?.get(2).copied()?.round() as i64;
        let layer_frame = comp_frame - source.timing_start;
        let Self { frozen, compositor, .. } = self;
        let picture = frozen.load(
            source.layer,
            layer_frame,
            &mut |bytes, width, height| compositor.upload_rgba16f("motolii-frozen", bytes, width, height).ok(),
        )?;
        Some((
            crate::render::compositor::LayerContent::LinearTexture(picture.texture),
            picture.natural,
            picture.padding,
            picture.frame,
        ))
    }

    fn frame_graph_image_sources(
        &mut self,
        rows: &[Vec<SceneImageSourceValue>],
        comp: CompSpec,
        camera: ResolvedCamera,
    ) -> Result<Vec<Vec<crate::render::compositor::GpuTexture2D>>, EngineError> {
        rows.iter().map(|row| {
            let mut textures = Vec::with_capacity(row.len());
            for source in row {
                match self.frame_graph_image_source(source, comp, camera)? {
                    Some(texture) => textures.push(texture),
                    None => {
                        textures.clear();
                        break;
                    }
                }
            }
            Ok(textures)
        }).collect()
    }

    fn frame_graph_image_source(
        &mut self,
        source: &SceneImageSourceValue,
        comp: CompSpec,
        camera: ResolvedCamera,
    ) -> Result<Option<crate::render::compositor::GpuTexture2D>, EngineError> {
        match source {
            SceneImageSourceValue::Content { layer, content, time, namespace } => {
                let previous_clock = self.compositor.clock;
                let previous_namespace = self.feedback_namespace;
                self.feedback_namespace = *namespace;
                self.set_frame_graph_source_clock(*time);
                let result = (|| {
                let content = match content {
                    SceneContentValue::None => return Ok(None),
                    SceneContentValue::Text(text) => self.shape_texture_from_shapes(
                        &text.shapes(), *layer, false, 0.05, comp, None, false,
                    )?.0,
                    SceneContentValue::Shape(shapes) => self.shape_texture_from_shapes(
                        shapes, *layer, false, 0.05, comp, None, false,
                    )?.0,
                    SceneContentValue::Material(_) | SceneContentValue::Particles(_) => return Ok(None),
                    SceneContentValue::Media { source, time } => {
                        self.file_content_for(&source.path, *time, *layer, comp)?.0
                    }
                    SceneContentValue::Plate(plate) => {
                        let nested = SceneValue {
                            layers: plate.members.iter().filter_map(|member| member.layer.clone()).collect(),
                        };
                        let prepared = self.prepare_gpu_scene(&nested, comp, camera)?;
                        if prepared.layers.is_empty() { return Ok(None); }
                        let (texture, _) = self.compositor.render_to_texture(
                            comp,
                            camera,
                            &prepared.layers,
                            crate::render::compositor::NO_BACKGROUND,
                        )?;
                        return Ok(self.compositor.import_premultiplied(&texture).ok());
                    }
                };
                let Some(texture) = content.and_then(|content| content.texture().cloned()) else {
                    return Ok(None);
                };
                Ok(self.compositor.snapshot_texture(&texture))
                })();
                self.compositor.clock = previous_clock;
                self.feedback_namespace = previous_namespace;
                result
            }
            SceneImageSourceValue::Scene { scene, background, time, namespace } => {
                let previous_clock = self.compositor.clock;
                let previous_namespace = self.feedback_namespace;
                self.feedback_namespace = *namespace;
                self.set_frame_graph_source_clock(*time);
                let result = (|| {
                let prepared = self.prepare_gpu_scene(scene, comp, camera)?;
                let (texture, _) = self.compositor.render_to_texture(
                    comp,
                    camera,
                    &prepared.layers,
                    *background,
                )?;
                Ok(self.compositor.import_premultiplied(&texture).ok())
                })();
                self.compositor.clock = previous_clock;
                self.feedback_namespace = previous_namespace;
                result
            }
        }
    }

    fn set_frame_graph_source_clock(&mut self, time: crate::doc::core::RationalTime) {
        let delta = self.compositor.clock.map_or(1.0 / 30.0, |clock| clock[1].max(1.0e-9));
        let frame = (time.as_seconds_f64() as f32 / delta).round();
        self.compositor.clock = Some([time.as_seconds_f64() as f32, delta, frame]);
    }

    pub(super) fn stamp_frame_graph_window_feedback(
        &mut self,
        layers: &mut [LayerWithPasses],
        window: crate::render::compositor::Window,
    ) {
        for entry in layers {
            let screen_chain = entry.layer.content.texture().is_none()
                || entry.passes.iter().any(|pass| pass.reads_backdrop || pass.reads_composite());
            for pass in &mut entry.passes {
                if let Some(mut key) = pass.feedback {
                    if screen_chain || pass.reads_backdrop || pass.reads_composite() {
                        key.screen = Some(window.size());
                        pass.feedback = Some(key);
                    }
                    if self.feedback_namespace == 0 {
                        self.feedback_keys_seen.push(key);
                    }
                }
            }
        }
    }
}


#[cfg(test)]
mod tests;
