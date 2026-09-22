use crate::doc::core::{CompSpec, LayerPlacement, ResolvedCamera};
use crate::doc::store::LayerId;
use crate::frame_graph::{SceneContentValue, SceneImageSourceValue, SceneLayerValue, SceneValue, SolverPlanValue};
use crate::render::compositor::{BlendMode as CompositeBlendMode, Layer, LayerWithPasses};
use crate::render::engine::{Engine, EngineError};

#[derive(Clone)]
pub(crate) struct PreparedScene {
    pub layers: Vec<LayerWithPasses>,
    pub layer_ids: Vec<LayerId>,
}

impl Engine {
    pub(super) fn prepare_execution_scene_with_solver(
        &mut self,
        scene: &SceneValue,
        solver: &SolverPlanValue,
        comp: CompSpec,
        projection_camera: ResolvedCamera,
        time: crate::doc::core::RationalTime,
        fps: crate::doc::store::Fps,
    ) -> Result<PreparedScene, EngineError> {
        let physics_overlays: std::collections::HashSet<LayerId> = self.overlay_frames.iter()
            .filter_map(|(layer, frame)| frame.physics.then_some(*layer))
            .collect();
        if physics_overlays.is_empty() {
            // Culling is execution planning only: semantic SceneValue remains
            // untouched and cacheable. The planner may omit only simple media
            // contributions that cannot feed analysis/matte/solver work.
            let planned = self.plan_frame_graph_scene(scene, solver, comp, projection_camera);
            let scene = planned.as_ref().unwrap_or(scene);
            let mut prepared = self.prepare_execution_scene(scene, comp, projection_camera)?;
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
        let mut base = self.prepare_execution_scene(&base_scene, comp, projection_camera)?;
        self.prepare_frame_graph_blocks(scene, solver, comp, time, fps, &mut base)?;

        let mut prepared = self.prepare_execution_scene(scene, comp, projection_camera)?;
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

    pub(crate) fn prepare_execution_scene(&mut self, scene: &SceneValue, comp: CompSpec, projection_camera: ResolvedCamera) -> Result<PreparedScene, EngineError> {
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
            } else if let Some(resident) = self.frame_graph_resident_content(source) {
                (Some(resident.content), resident.natural, 0, None, false)
            } else {
                let (content, natural) = match &source.content {
                SceneContentValue::None => continue,
                // Resident leaf content is the production owner for these
                // semantic values. Reaching this fallback means the new
                // backend could not materialize the resource; keep the bridge
                // temporarily for special force-picture/extrusion cases.
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
                SceneContentValue::Particles(_) => {
                    return Err(EngineError::Store("GPU resident particle content missing".into()));
                }
                SceneContentValue::Plate(plate) => {
                    self.frame_graph_plate(source, plate, comp, projection_camera)?
                        .map_or((None, [comp.width as f32, comp.height as f32]), |(content, size)| (Some(content), size))
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
            let placement = self.frame_graph_resident_placement(source)
                .unwrap_or_else(|| crate::gpu_exec::ResidentPlacement::from_scene(source));
            let effects = self.frame_graph_resident_effects(source).unwrap_or_else(|| {
                let mut passes = crate::render::engine::translate::translate_effect_passes(&source.effects);
                let mut plate_passes = crate::render::engine::translate::translate_plate_passes(&source.after_effects);
                crate::render::engine::translate::stamp_feedback(&mut passes, source.layer, source.instance, 0, None, 0);
                crate::render::engine::translate::stamp_feedback(&mut plate_passes, source.layer, source.instance, 1, None, 0);
                crate::gpu_exec::ResidentEffectChain { passes, plate_passes }
            });
            let pass_sources = if frozen_hit {
                Vec::new()
            } else {
                self.frame_graph_snapshot_rows(source, comp, projection_camera)?
            };
            let resident = crate::gpu_exec::ResidentContent { content, natural };
            let mut prepared = self.gpu_contribution_layer(
                source,
                resident,
                placement,
                effects,
                pass_sources,
                comp,
                projection_camera,
            )?;
            prepared.padding = frozen_padding;
            prepared.layer.frame = frozen_frame;
            layers.push(prepared);
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
            let semantic = &scene.layers[index];
            layers[index] = self.frame_graph_matte(
                semantic,
                &layers[index],
                &layers[source_index],
                comp,
                projection_camera,
            )?;
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

        Ok(PreparedScene { layers, layer_ids })
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

    pub(crate) fn set_frame_graph_source_clock(&mut self, time: crate::doc::core::RationalTime) {
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
                        self.gpu_history.observe(key);
                    }
                }
            }
        }
    }
}


#[cfg(test)]
mod tests;
