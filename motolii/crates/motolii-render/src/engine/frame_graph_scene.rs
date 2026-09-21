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
            let force_picture = clip_bases.contains(&source.layer);
            let frozen = self.frame_graph_frozen_content(source);
            let (content, natural, frozen_padding, frozen_frame, frozen_hit) = if let Some((content, natural, padding, frame)) = frozen {
                (Some(content), natural, padding, frame, true)
            } else {
                let (content, natural) = match &source.content {
                SceneContentValue::None => continue,
                SceneContentValue::Text(text) if force_picture => self.shape_texture_from_shapes(&text.shapes(), key, false, 0.05, comp, None, true)?,
                SceneContentValue::Text(text) => self.text_texture_from_shapes(&text.shapes(), key, comp)?,
                SceneContentValue::Shape(shapes) => self.shape_texture_from_shapes(shapes, key, !force_picture, 0.05, comp, None, true)?,
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
mod tests {
    use super::*;
    use crate::doc::store::{
        BlendMode, Composition, Fps, LayerAttrsPatch, LayerId, LayerMeta, LayerSource,
        LayerTiming, Matte, MatteMode, ShapeNode,
    };
    use crate::doc::vector::{Brush, Fill, PathSource, Point, Rgb, Shape};
    use crate::frame_graph::{
        CompiledGraph, EvaluationContext, FrameQuality, Generation, GraphNode, GraphRevision,
        GraphTopology, NodeExecutor, NodeInputs, NodeValue, SceneProgram, SceneProgramError,
    };
    use motolii_edit::{Document, Intent};

    struct Executor<'a>(&'a SceneProgram);
    impl NodeExecutor for Executor<'_> {
        type Error = SceneProgramError;
        fn execute(
            &mut self,
            node: &GraphNode,
            inputs: NodeInputs,
            context: EvaluationContext,
        ) -> Result<NodeValue, Self::Error> {
            self.0.execute(node, &inputs, &context)
        }
    }

    fn document() -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 30,
            background: [0.0; 4],
        })).unwrap();
        doc
    }

    fn rectangle(rgb: Rgb) -> Vec<ShapeNode> {
        vec![ShapeNode::Leaf(Shape {
            source: PathSource::Rectangle { size: Point { x: 32.0, y: 32.0 } },
            ops: Vec::new(),
            stroke: None,
            fill: Some(Fill { brush: Brush::Solid(rgb), ..Default::default() }),
        })]
    }

    fn add_shape(doc: &mut Document, id: u64, order: i16, rgb: Rgb) -> LayerId {
        let layer = LayerId(id);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Shape,
                    order,
                    timing: LayerTiming::place(0, None, 30),
                },
            },
            Intent::SetShapes { layer, shapes: rectangle(rgb) },
        ]).unwrap();
        layer
    }

    fn scene(doc: &Document) -> SceneValue {
        let program = SceneProgram::compile(&doc.view()).unwrap();
        let root = program.scene().scene;
        let topology = GraphTopology::try_new(program.nodes(), vec![root]).unwrap();
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(1), topology);
        let mut executor = Executor(&program);
        let frame = graph.evaluate(
            &mut executor,
            crate::doc::core::RationalTime::ZERO,
            FrameQuality::Export,
            Generation::new(1),
        ).unwrap();
        frame.value(root).and_then(|value| value.downcast_ref::<SceneValue>()).unwrap().clone()
    }

    #[test]
    fn track_matte_source_is_auxiliary_not_a_second_draw_layer() {
        let mut doc = document();
        let source = add_shape(&mut doc, 1, 0, Rgb { r: 1.0, g: 1.0, b: 1.0 });
        let target = add_shape(&mut doc, 2, 1, Rgb { r: 1.0, g: 0.0, b: 0.0 });
        doc.apply(Intent::SetAttrs {
            layer: target,
            patch: LayerAttrsPatch {
                matte: Some(Some(Matte { layer: source, mode: MatteMode::Alpha })),
                ..Default::default()
            },
        }).unwrap();

        let scene = scene(&doc);
        let mut engine = Engine::new().unwrap();
        let gpu = engine.prepare_gpu_scene(
            &scene,
            doc.view().composition().unwrap().unwrap().spec(),
            ResolvedCamera::default(),
        ).unwrap();
        assert_eq!(gpu.layers.len(), 1, "matte source must be consumed");
    }

    #[test]
    fn clipping_folds_the_upper_picture_into_its_base() {
        let mut doc = document();
        add_shape(&mut doc, 1, 0, Rgb { r: 0.0, g: 1.0, b: 0.0 });
        let upper = add_shape(&mut doc, 2, 1, Rgb { r: 1.0, g: 0.0, b: 0.0 });
        doc.apply(Intent::SetAttrs {
            layer: upper,
            patch: LayerAttrsPatch { clip_to_below: Some(true), ..Default::default() },
        }).unwrap();

        let scene = scene(&doc);
        let mut engine = Engine::new().unwrap();
        let gpu = engine.prepare_gpu_scene(
            &scene,
            doc.view().composition().unwrap().unwrap().spec(),
            ResolvedCamera::default(),
        ).unwrap();
        assert_eq!(gpu.layers.len(), 1, "clip upper is not a separate contribution");
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
    }

    #[test]
    fn stencil_is_built_as_a_matte_source_and_never_drawn_itself() {
        let mut doc = document();
        add_shape(&mut doc, 1, 0, Rgb { r: 1.0, g: 0.0, b: 0.0 });
        let stencil = add_shape(&mut doc, 2, 1, Rgb { r: 1.0, g: 1.0, b: 1.0 });
        doc.apply(Intent::SetAttrs {
            layer: stencil,
            patch: LayerAttrsPatch { blend_mode: Some(BlendMode::StencilAlpha), ..Default::default() },
        }).unwrap();

        let scene = scene(&doc);
        assert_eq!(
            scene.layers.iter().find(|layer| layer.layer == LayerId(1)).unwrap().matte,
            Some(Matte { layer: stencil, mode: MatteMode::Alpha }),
        );

        let mut engine = Engine::new().unwrap();
        let gpu = engine.prepare_gpu_scene(
            &scene,
            doc.view().composition().unwrap().unwrap().spec(),
            ResolvedCamera::default(),
        ).unwrap();
        assert_eq!(gpu.layers.len(), 1, "stencil itself is auxiliary");
    }
}
