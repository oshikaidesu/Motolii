use crate::doc::core::{CompSpec, LayerPlacement, ResolvedCamera};
use crate::doc::store::LayerId;
use crate::frame_graph::{SceneContentValue, SceneLayerValue, SceneValue, SolverPlanValue};
use crate::render::compositor::LayerContent;
use crate::render::compositor::effects::surface_program::SurfaceRecipe;
use crate::render_graph::{Composed, Extrusion, ImageInput, LayerWork, RasterSource, RenderGraph};
use crate::render::compositor::{BlendMode as CompositeBlendMode, Layer, LayerWithPasses};
use crate::render::engine::{Engine, EngineError};

#[derive(Clone)]
pub(super) struct GpuSceneValue {
    pub layers: Vec<LayerWithPasses>,
    pub layer_ids: Vec<LayerId>,
    /// For each output, the scene contribution it draws when it is that contribution alone
    /// (no clip group, no matte): only those can take a new placement without being prepared again.
    pub plain_sources: Vec<Option<usize>>,
}

/// The scene a prepared frame was built from, kept to recognise a frame that differs only in
/// where things are placed.
pub(super) struct PreparedFrame {
    scene: SceneValue,
    prepared: GpuSceneValue,
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
            if let Some(mut prepared) = self.moved_only(scene) {
                self.prepare_frame_graph_blocks(scene, solver, comp, time, fps, &mut prepared)?;
                self.drawn_layers = prepared.layers.len();
                return Ok(prepared);
            }
            // Culling is execution planning only: semantic SceneValue remains
            // untouched and cacheable. The planner may omit only simple media
            // contributions that cannot feed analysis/matte/solver work.
            let planned = self.plan_frame_graph_scene(scene, solver, comp, projection_camera);
            let scene_for_frame = scene;
            let scene = planned.as_ref().unwrap_or(scene);
            let mut prepared = self.prepare_gpu_scene(scene, comp, projection_camera)?;
            self.full_prepares += 1;
            self.remember_prepared(scene_for_frame, planned.is_none(), &prepared);
            self.prepare_frame_graph_blocks(scene, solver, comp, time, fps, &mut prepared)?;
            self.drawn_layers = prepared.layers.len();
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
        self.drawn_layers = prepared.layers.len();
        Ok(prepared)
    }

    /// The previous frame's prepared layers with this frame's placements, when nothing but
    /// placement changed. No lowering, no preparation: the cost is one pass over the layers.
    fn moved_only(&mut self, scene: &SceneValue) -> Option<GpuSceneValue> {
        let last = self.last_prepared.as_ref()?;
        if last.scene.layers.len() != scene.layers.len()
            || !last.scene.layers.iter().zip(&scene.layers).all(|(a, b)| same_but_placement(a, b))
        {
            return None;
        }
        let mut prepared = last.prepared.clone();
        for (layer, source) in prepared.layers.iter_mut().zip(&prepared.plain_sources) {
            let Some(index) = *source else { return None };
            let placed = &scene.layers[index];
            let placement = &mut layer.layer.placement;
            placement.transform = placed.transform.affine;
            placement.world_transform = Some(placed.transform.spatial);
            placement.z = placed.transform.spatial.translation.z;
            placement.opacity = placed.opacity;
            placement.order = i32::from(placed.order);
        }
        self.last_prepared = Some(PreparedFrame { scene: scene.clone(), prepared: prepared.clone() });
        Some(prepared)
    }

    /// Keeps a prepared frame whose layers depend only on their contributions' content, so a
    /// later frame that merely moves them can reuse it.
    fn remember_prepared(&mut self, scene: &SceneValue, uncut: bool, prepared: &GpuSceneValue) {
        // Preparation that moved a layer itself (a planar warp's frame) is not a placement to replace.
        let placed_as_authored = prepared.layers.iter().zip(&prepared.plain_sources).all(|(layer, source)| {
            source.is_some_and(|index| {
                let authored = &scene.layers[index];
                layer.layer.placement.transform == authored.transform.affine
                    && layer.layer.placement.world_transform == Some(authored.transform.spatial)
            })
        });
        let reusable = uncut && placed_as_authored && scene.layers.iter().all(placement_independent);
        self.last_prepared = reusable.then(|| PreparedFrame { scene: scene.clone(), prepared: prepared.clone() });
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
        self.compositor.refresh_catalog_programs();
        let catalog = self.compositor.catalog.clone();
        let graph = crate::render_lowering::lower_scene(scene, &catalog)
            .map_err(|error| EngineError::Store(error.to_string()))?;
        self.execute_render_graph(&graph, comp, projection_camera)
    }

    /// The scene's contributions as material-space pictures the host reads back.
    pub(super) fn prepare_gpu_pictures(&mut self, scene: &SceneValue, comp: CompSpec, projection_camera: ResolvedCamera) -> Result<GpuSceneValue, EngineError> {
        self.compositor.refresh_catalog_programs();
        let catalog = self.compositor.catalog.clone();
        let graph = crate::render_lowering::lower_scene_as_pictures(scene, &catalog)
            .map_err(|error| EngineError::Store(error.to_string()))?;
        self.execute_render_graph(&graph, comp, projection_camera)
    }

    /// Cassette executor: reads only the backend-neutral graph.
    pub(super) fn execute_render_graph(&mut self, graph: &RenderGraph, comp: CompSpec, projection_camera: ResolvedCamera) -> Result<GpuSceneValue, EngineError> {
        let mut prepared = Vec::with_capacity(graph.layers.len());
        for work in &graph.layers {
            prepared.push(self.execute_layer(work, comp, projection_camera)?);
        }
        let mut groups = std::collections::HashMap::new();
        let mut layers = Vec::with_capacity(graph.output.len());
        let mut layer_ids = Vec::with_capacity(graph.output.len());
        let mut plain_sources = Vec::with_capacity(graph.output.len());
        for composed in &graph.output {
            if let Some(layer) = self.realize_composed(graph, &prepared, composed, &mut groups, comp, projection_camera)? {
                layers.push(layer);
                layer_ids.push(graph.layers[composed.base].id);
                plain_sources.push((composed.atop.is_empty() && composed.mask.is_none()).then_some(composed.base));
            }
        }
        Ok(GpuSceneValue { layers, layer_ids, plain_sources })
    }

    /// How finely outlines are cut: one device pixel after projection. Vector
    /// output reuses power-of-two steps; pictures use the exact density so
    /// placing them does not resample the edge.
    fn outline_tolerance(&self, work: Option<&LayerWork>, natural: [f32; 2], vector: bool, comp: CompSpec, camera: ResolvedCamera) -> f32 {
        let Some(work) = work else { return 0.05 };
        let (origin, u, v) = crate::render::compositor::projected_placement_corners(comp, camera, work.projection, work.placement, glam::Vec2::ZERO, natural.into());
        let projection = crate::doc::core::camera_projection(comp, camera);
        let matrix = projection.projection_matrix() * projection.view_matrix();
        let project = |p: glam::Vec3| {
            let p = matrix * p.extend(1.0);
            glam::vec2(p.x / p.w, p.y / p.w) * glam::vec2(comp.width as f32, comp.height as f32) * 0.5
        };
        let density = [origin, origin + u, origin + v, origin + u + v, origin + (u + v) * 0.5].into_iter().flat_map(|p| {
            [(project(p + u / natural[0].max(1.0)) - project(p)).length(),
             (project(p + v / natural[1].max(1.0)) - project(p)).length()]
        }).filter(|v| v.is_finite()).fold(1.0f32, f32::max);
        let mut exact = ((density.max(1.0) * 1024.0).round() / 1024.0).max(1.0);
        if !work.passes.is_empty() {
            let reach = work.passes.iter().map(|p| p.padding() as f32).fold(0.0f32, f32::max);
            let limit = self.compositor.ctx.device.limits().max_texture_dimension_2d as f32;
            let extent = natural[0].max(natural[1]).max(1.0) + 2.0 * reach;
            exact = exact.min((limit / extent).max(1.0));
        }
        let stepped = (density * (1.0 - 1e-4)).log2().ceil().exp2().max(1.0);
        (0.05 / if vector { stepped } else { exact }).max(1e-6)
    }

    fn execute_raster(&mut self, id: LayerId, key: LayerId, source: &RasterSource, work: Option<&LayerWork>, comp: CompSpec, camera: ResolvedCamera) -> Result<(Option<LayerContent>, [f32; 2]), EngineError> {
        let order = work.map_or(0, |work| work.placement.order);
        Ok(match source {
            RasterSource::None => (None, [0.0, 0.0]),
            RasterSource::Vector { shapes, vector, remember, field_step } => {
                let step = field_step.then_some(crate::render::engine::texture::FIELD_STEP);
                // An unbent vector outline is painted exactly from its curves: no cut to choose.
                let tolerance = if *vector && step.is_none() {
                    0.05
                } else {
                    let natural = match self.cached_shape(key, false, shapes, *vector, step, None) {
                        Some(hit) => hit.natural,
                        None => crate::picture::shapes_ops::content_canvas(shapes)?
                            .map_or([1.0; 2], |canvas| [canvas.width as f32, canvas.height as f32]),
                    };
                    self.outline_tolerance(work, natural, *vector, comp, camera)
                };
                self.shape_texture_from_shapes(shapes, key, *vector, tolerance, comp, step, *remember)?
            }
            RasterSource::CanvasVector { shapes } => self.text_texture_from_shapes(shapes, key, comp)?,
            RasterSource::Mesh { path } => self.mesh_content_for(path, comp)?,
            RasterSource::EnvironmentMap { path } => self.environment_content_for(path)?,
            RasterSource::Image { path, time } => self.file_content_for(path, *time, id, comp)?,
            RasterSource::Points { positions, colors, sizes, bounds, links } => (
                Some(LayerContent::Cloud {
                    positions: positions.clone(),
                    colors: colors.clone(),
                    bounds: *bounds,
                    point_size: 1.0,
                    sizes: Some(sizes.clone()),
                    sprites: true,
                    links: links.clone(),
                }),
                [bounds.max[0].max(1.0), bounds.max[1].max(1.0)],
            ),
            RasterSource::Isolate { graph, average } => {
                let prepared = self.execute_render_graph(graph, comp, camera)?;
                if prepared.layers.is_empty() {
                    (None, [comp.width as f32, comp.height as f32])
                } else {
                    let placement = LayerPlacement {
                        transform: glam::Affine2::IDENTITY,
                        world_transform: None,
                        opacity: 1.0,
                        order,
                        z: 0.0,
                        rotation_x: 0.0,
                        rotation_y: 0.0,
                        plane: None,
                    };
                    let baked = self.bake_isolated_layers(comp, camera, prepared.layers, CompositeBlendMode::Normal, placement, *average)?;
                    (Some(baked.content), baked.size)
                }
            }
        })
    }

    fn execute_layer(&mut self, work: &LayerWork, comp: CompSpec, projection_camera: ResolvedCamera) -> Result<Option<LayerWithPasses>, EngineError> {
        let host = if work.host_picture { self.overlay_content(work.id, comp)? } else { None };
        let frozen = self.frame_graph_frozen_content(work);
        let (content, natural, frozen_padding, frozen_frame, frozen_hit) = if let Some((content, natural, padding, frame)) = frozen {
            (Some(content), natural, padding, frame, true)
        } else if let Some((content, natural)) = host {
            (Some(content), natural, 0, None, false)
        } else {
            let (content, natural) = self.execute_raster(work.id, work.content_key, &work.content, Some(work), comp, projection_camera)?;
            // A picture drawn above one pixel per unit carries its logical frame.
            let frame = matches!(work.content, RasterSource::Vector { .. })
                .then(|| content.as_ref().and_then(|content| content.texture()))
                .flatten()
                .map(|texture| crate::render::compositor::effects::vism::ImageFrame { size: natural, origin: [0.0; 2], pixels: texture.width_height() });
            (content, natural, 0, frame, false)
        };
        let Some(mut content) = content else { return Ok(None) };
        if let (LayerContent::Texture(texture), Some(extrusion)) = (&content, &work.extrude) {
            content = self.frame_graph_extruded_content(work, extrusion, texture.clone(), natural)?;
        }
        if !frozen_hit && !work.image_inputs.is_empty() {
            content = match &content {
                LayerContent::Texture(texture) => self.compositor.snapshot_texture(texture)
                    .map(LayerContent::Texture)
                    .unwrap_or_else(|| content.clone()),
                LayerContent::LinearTexture(texture) => self.compositor.snapshot_texture(texture)
                    .map(LayerContent::LinearTexture)
                    .unwrap_or_else(|| content.clone()),
                _ => content,
            };
        }
        let (passes, pass_sources) = if frozen_hit {
            (Vec::new(), Vec::new())
        } else {
            let mut direct_passes = work.passes.clone();
            let direct_screen = (
                content.texture().is_none()
                    || direct_passes.iter().any(|pass| pass.reads_backdrop || pass.reads_composite())
            ).then_some([comp.width, comp.height]);
            self.stamp_feedback(&mut direct_passes, work.id, work.instance, 0, direct_screen);

            let mut plate_passes = work.after_passes.clone();
            let plate_screen = plate_passes.iter()
                .any(|pass| pass.reads_backdrop || pass.reads_composite())
                .then_some([comp.width, comp.height]);
            self.stamp_feedback(&mut plate_passes, work.id, work.instance, 1, plate_screen);

            (
                direct_passes.into_iter().chain(plate_passes).collect(),
                self.frame_graph_image_sources(&work.image_inputs, comp, projection_camera)?,
            )
        };
        let layer = Layer {
            content,
            size: natural,
            placement: work.placement,
            projection: work.projection,
            projection_camera,
            blend_mode: work.blend,
            shading: Default::default(),
            displace: work.displace,
            clip: work.clip,
            shadow: work.shadow,
            outline: 0,
            frame: frozen_frame,
        };
        let layer = self.apply_masks_to_layer(layer, &work.masks, natural, frozen_frame)?;
        let mut layer = self.apply_material_recipe(layer, work.id, &work.material, work.source_is_file, work.source_tick, natural, frozen_frame)?;
        if matches!(layer.content, LayerContent::Model(_) | LayerContent::Texture(_) | LayerContent::LinearTexture(_)) {
            let recipe = SurfaceRecipe { unlit: matches!(&layer.content, LayerContent::Model(model) if model.planar_size.is_some()), ..work.surface.clone() };
            layer.shading = self.compositor.surface_shading_from(&recipe).map_err(EngineError::Store)?;
        }
        let layer = self.flatten_if_asked(comp, projection_camera, layer, work.isolate)?;
        Ok(Some(LayerWithPasses { layer, passes, padding: frozen_padding, pass_sources, cut: Vec::new() }))
    }

    /// An absent picture contributes no coverage, so a picture masked by an
    /// absent picture is absent too.
    fn realize_composed(
        &mut self,
        graph: &RenderGraph,
        prepared: &[Option<LayerWithPasses>],
        composed: &Composed,
        groups: &mut std::collections::HashMap<usize, Option<LayerWithPasses>>,
        comp: CompSpec,
        camera: ResolvedCamera,
    ) -> Result<Option<LayerWithPasses>, EngineError> {
        if !groups.contains_key(&composed.base) {
            let mut base = prepared[composed.base].clone();
            for &upper in &composed.atop {
                let Some(upper) = prepared[upper].as_ref() else { continue };
                let Some(current) = base.take() else { break };
                base = match self.clip_onto_base(current.clone(), &upper.layer, &upper.passes)? {
                    Some(clipped) => Some(clipped),
                    None => {
                        self.layer_failures.push(format!(
                            "layer {} clips to a base without a texture (point cloud / model bases are not clippable)",
                            graph.layers[composed.base].id.0
                        ));
                        Some(current)
                    }
                };
            }
            groups.insert(composed.base, base);
        }
        let Some(base) = groups[&composed.base].clone() else { return Ok(None) };
        let Some((masks, mode)) = &composed.mask else { return Ok(Some(base)) };
        let mut sources = Vec::with_capacity(masks.len());
        for mask in masks {
            if let Some(source) = self.realize_composed(graph, prepared, mask, groups, comp, camera)? { sources.push(source); }
        }
        let source = match sources.len() {
            0 => return Ok(None),
            1 => { let source = sources.remove(0); self.apply_effects_before_matte(comp, camera, source.layer, &source.passes)? }
            _ => {
                let placement = sources[0].layer.placement;
                self.bake_isolated_layers(comp, camera, sources, CompositeBlendMode::Normal, placement, false)?
            }
        };
        let target = self.apply_effects_before_matte(comp, camera, base.layer, &base.passes)?;
        let layer = self.compositor.matte_layer(comp, camera, &target, &source, *mode)?;
        Ok(Some(LayerWithPasses { layer, passes: Vec::new(), padding: 0, pass_sources: Vec::new(), cut: Vec::new() }))
    }

    fn frame_graph_extruded_content(
        &mut self,
        work: &LayerWork,
        extrusion: &Extrusion,
        texture: crate::render::compositor::GpuTexture2D,
        natural: [f32; 2],
    ) -> Result<crate::render::compositor::LayerContent, EngineError> {
        let solid = extrusion.solid;
        use std::hash::{Hash, Hasher};

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        texture.handle.hash(&mut hasher);
        solid.hash_key(&mut hasher);
        extrusion.stretch[0].to_bits().hash(&mut hasher);
        extrusion.stretch[1].to_bits().hash(&mut hasher);
        let key = hasher.finish();
        if let Some((cached, model)) = self.extrusions.get(&work.id) {
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

        let outlines = match &extrusion.outline {
            Some(shapes) => match crate::picture::shapes_ops::content_canvas(shapes)? {
                Some(canvas) => crate::render::compositor::paths::outlines(shapes, &canvas)?,
                None => Vec::new(),
            },
            None => rectangle(natural),
        };

        let Some(model) = self.compositor.extrude_model(&outlines, texture.clone(), natural, solid)? else {
            return Ok(crate::render::compositor::LayerContent::Texture(texture));
        };
        let model = std::sync::Arc::new(model);
        self.extrusions.insert(work.id, (key, model.clone()));
        Ok(crate::render::compositor::LayerContent::Model(model))
    }

    fn frame_graph_frozen_content(
        &mut self,
        work: &LayerWork,
    ) -> Option<(
        crate::render::compositor::LayerContent,
        [f32; 2],
        u32,
        Option<crate::render::compositor::effects::vism::ImageFrame>,
    )> {
        let timing_start = work.freeze?;
        if self.freezing == Some(work.id) {
            return None;
        }
        let comp_frame = self.compositor.clock?.get(2).copied()?.round() as i64;
        let layer_frame = comp_frame - timing_start;
        let Self { frozen, compositor, .. } = self;
        let picture = frozen.load(
            work.id,
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
        rows: &[Vec<ImageInput>],
        comp: CompSpec,
        camera: ResolvedCamera,
    ) -> Result<Vec<Vec<crate::render::compositor::GpuTexture2D>>, EngineError> {
        rows.iter().map(|row| {
            let mut textures = Vec::with_capacity(row.len());
            for input in row {
                match self.frame_graph_image_source(input, comp, camera)? {
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
        input: &ImageInput,
        comp: CompSpec,
        camera: ResolvedCamera,
    ) -> Result<Option<crate::render::compositor::GpuTexture2D>, EngineError> {
        let (time, namespace) = match input {
            ImageInput::Absent => return Ok(None),
            ImageInput::Refused { layer } => {
                self.layer_failures.push(format!("指した層 {} の絵が無い(自分自身か、無い層)", layer.0));
                return Ok(None);
            }
            ImageInput::Raster { time, namespace, .. } | ImageInput::Graph { time, namespace, .. } => (*time, *namespace),
        };
        let previous_clock = self.compositor.clock;
        let previous_namespace = self.feedback_namespace;
        self.feedback_namespace = namespace;
        self.set_frame_graph_source_clock(time);
        let result = (|| match input {
            ImageInput::Absent | ImageInput::Refused { .. } => Ok(None),
            ImageInput::Raster { id, source, .. } => {
                let (content, _) = self.execute_raster(*id, *id, source, None, comp, camera)?;
                let Some(texture) = content.and_then(|content| content.texture().cloned()) else {
                    return Ok(None);
                };
                Ok(self.compositor.snapshot_texture(&texture))
            }
            ImageInput::Graph { graph, background, absent_when_empty, .. } => {
                let prepared = self.execute_render_graph(graph, comp, camera)?;
                if prepared.layers.is_empty() && *absent_when_empty { return Ok(None); }
                let (texture, _) = self.compositor.render_to_texture(comp, camera, &prepared.layers, *background)?;
                Ok(self.compositor.import_premultiplied(&texture).ok())
            }
        })();
        self.compositor.clock = previous_clock;
        self.feedback_namespace = previous_namespace;
        result
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


/// A contribution whose prepared picture does not depend on where it is placed or on the frame.
fn placement_independent(layer: &SceneLayerValue) -> bool {
    let content = match &layer.content {
        SceneContentValue::None | SceneContentValue::Shape(_) | SceneContentValue::Text(_) | SceneContentValue::Material(_) => true,
        SceneContentValue::Media { source, .. } => crate::render::media::is_still_image_path(&source.path),
        SceneContentValue::Particles(_) | SceneContentValue::Plate(_) => false,
    };
    content
        && layer.image_sources.is_empty()
        && layer.matte.is_none()
        && !layer.clip_to_below
        && !layer.flatten
        && !layer.freeze_eligible
        && !layer.effects.iter().chain(&layer.after_effects).any(|effect| crate::extensions::overlay::is_track_overlay(&effect.plugin_id))
}

/// Everything but placement (transform, opacity, order) is the same; content is compared by
/// identity of the shared evaluated value, never by walking it.
fn same_but_placement(a: &SceneLayerValue, b: &SceneLayerValue) -> bool {
    let content = match (&a.content, &b.content) {
        (SceneContentValue::None, SceneContentValue::None) => true,
        (SceneContentValue::Shape(x), SceneContentValue::Shape(y)) => std::sync::Arc::ptr_eq(x, y),
        (SceneContentValue::Text(x), SceneContentValue::Text(y)) => std::sync::Arc::ptr_eq(x, y),
        (SceneContentValue::Material(x), SceneContentValue::Material(y)) => x == y,
        (SceneContentValue::Media { source: x, time: tx }, SceneContentValue::Media { source: y, time: ty }) => {
            x == y && (tx == ty || crate::render::media::is_still_image_path(&x.path))
        }
        _ => false,
    };
    content
        && placement_independent(b)
        && a.layer == b.layer
        && a.instance == b.instance
        && a.source == b.source
        && a.content_key == b.content_key
        && a.effects == b.effects
        && a.after_effects == b.after_effects
        && a.masks == b.masks
        && a.environment == b.environment
        && a.ghost == b.ghost
        && a.timing_start == b.timing_start
        && a.projection == b.projection
        && a.blend == b.blend
        && a.shape_stretch == b.shape_stretch
        && a.depth == b.depth
}
