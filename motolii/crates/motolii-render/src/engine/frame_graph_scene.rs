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
        // Members first, then the frame's one reflection capture (it sees the members of every
        // plate), then the plates' pictures lit by it.
        self.pending_plates.clear();
        self.reflectables.clear();
        self.reflectable_member_ids.clear();
        self.preparation_events.clear();
        self.plates_needed_early = false;
        self.known_frame_light = None;
        self.deferring_plates = true;
        let prepared = self.prepare_gpu_scene_with_solver_members(scene, solver, comp, projection_camera, time, fps);
        self.deferring_plates = false;
        let prepared = prepared?;
        if !self.plates_needed_early {
            self.frame_light = self.light_and_bake(comp, projection_camera, &prepared.layers)?;
            return Ok(prepared);
        }
        // Something in the frame (a matte, a clip group) reads a plate's picture while the frame is
        // prepared: the reflectable scene just collected lights the frame once, and the frame is
        // prepared again with its plates baked in place under that same light.
        self.frame_light = self.light_the_scene(comp, projection_camera, &prepared.layers)?;
        self.pending_plates.clear();
        // The draft pass's record is dropped with its work; only the capture it made stays.
        self.preparation_events.retain(|event| matches!(event, PreparationEvent::Capture(_)));
        self.known_frame_light = Some(self.frame_light.clone());
        let prepared = self.prepare_gpu_scene_with_solver_members(scene, solver, comp, projection_camera, time, fps);
        self.known_frame_light = None;
        prepared
    }

    /// A picture prepared as a frame of its own (another time's composite, an analysis picture):
    /// its plates wait for its one capture like a frame's. Returns what `f` made and its light.
    pub(super) fn as_own_frame<T>(&mut self, comp: CompSpec, camera: ResolvedCamera, f: impl FnOnce(&mut Self) -> Result<(T, Vec<LayerWithPasses>), EngineError>) -> Result<(T, crate::render::compositor::WorldLight), EngineError> {
        self.as_frame_part(comp, camera, true, f)
    }

    /// Like `as_own_frame`, but when `own_frame` is false the pictures belong to the frame being
    /// evaluated (a read for analysis): its plates are baked unlit by any scene reflection, and no
    /// capture is made for them — a frame has one.
    pub(super) fn as_frame_part<T>(&mut self, comp: CompSpec, camera: ResolvedCamera, own_frame: bool, f: impl FnOnce(&mut Self) -> Result<(T, Vec<LayerWithPasses>), EngineError>) -> Result<(T, crate::render::compositor::WorldLight), EngineError> {
        let pending = std::mem::take(&mut self.pending_plates);
        let reflectables = std::mem::take(&mut self.reflectables);
        let member_ids = std::mem::take(&mut self.reflectable_member_ids);
        let deferring = std::mem::replace(&mut self.deferring_plates, true);
        let known = self.known_frame_light.take();
        let early = std::mem::replace(&mut self.plates_needed_early, false);
        let made = f(self);
        self.deferring_plates = false;
        let lit = made.and_then(|(value, top)| {
            if own_frame { return Ok((value, self.light_and_bake(comp, camera, &top)?)); }
            let unlit = crate::render::compositor::WorldLight::default();
            self.bake_pending_plates(&unlit)?;
            Ok((value, unlit))
        });
        self.pending_plates = pending;
        self.reflectables = reflectables;
        self.reflectable_member_ids = member_ids;
        self.deferring_plates = deferring;
        self.known_frame_light = known;
        self.plates_needed_early = early;
        lit
    }

    /// The frame's light, captured once, and its waiting plates baked with it.
    fn light_and_bake(&mut self, comp: CompSpec, camera: ResolvedCamera, top: &[LayerWithPasses]) -> Result<crate::render::compositor::WorldLight, EngineError> {
        let light = self.light_the_scene(comp, camera, top)?;
        self.bake_pending_plates(&light)?;
        Ok(light)
    }

    /// The frame's world light, once, from its reflectable scene: the top-level layers (not the
    /// plates' pictures, which are not baked yet) and every plate's members, each where it is.
    fn light_the_scene(&mut self, comp: CompSpec, camera: ResolvedCamera, top: &[LayerWithPasses]) -> Result<crate::render::compositor::WorldLight, EngineError> {
        let plates: Vec<_> = self.pending_plates.iter().map(|plate| plate.target_texture().clone()).collect();
        let is_plate = |layer: &LayerWithPasses| layer.layer.content.texture().is_some_and(|t| {
            self.compositor.ctx.gpu_resources.textures.get_from_handle(t.handle()).ok().is_some_and(|g| plates.iter().any(|p| *p == g.texture))
        });
        let mut scene: Vec<LayerWithPasses> = top.iter().filter(|layer| !is_plate(layer)).cloned().collect();
        let same_as_frame = scene.len() == top.len() && self.reflectables.is_empty();
        // The plates' members this capture sees (the top level is the frame's own layer list).
        self.reflectable_ids = std::mem::take(&mut self.reflectable_member_ids);
        scene.extend(self.reflectables.drain(..));
        for layer in &mut scene { layer.layer.projection_camera = camera; }
        let (pictures, paddings, spills, _always_empty) = self.compositor.effective_layer_textures(&scene)?;
        let inputs = crate::render::compositor::sequential_inputs(&scene, &pictures, &paddings, &spills);
        let environment = self.compositor.world_environment.clone();
        let (reflection, light, meshes) = self.compositor.capture_world_light(comp, &inputs, environment.as_deref())?;
        drop(inputs);
        self.world_light_captures += 1;
        self.preparation_events.push(PreparationEvent::Capture(self.world_light_captures));
        Ok(crate::render::compositor::WorldLight { reflection, light, meshes: meshes.filter(|_| same_as_frame), serial: self.world_light_captures })
    }

    fn prepare_gpu_scene_with_solver_members(
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
            let mut prepared = self.prepare_gpu_scene_incremental(scene, comp, projection_camera)?;
            let started = std::time::Instant::now();
            self.prepare_frame_graph_blocks(scene, solver, comp, time, fps, &mut prepared)?;
            if self.blocks.object_count() > 0 { self.ledger.claim("blocks", "block solver", format!("{} objects", self.blocks.object_count()), started.elapsed()); }
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
        if scene.layers.iter().any(|layer| layer.reads_other_pictures()) {
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
        self.adopt_world_environment(&graph)?;
        self.execute_render_graph(&graph, comp, projection_camera)
    }

    /// The top-level graph's environment becomes the composition's light for every draw in the
    /// frame, including the nested draws of plates.
    pub(super) fn adopt_world_environment(&mut self, graph: &crate::render_graph::RenderGraph) -> Result<(), EngineError> {
        let path = graph.layers.iter().rev().find_map(|work| match &work.content {
            RasterSource::EnvironmentMap { path } => Some(path.clone()),
            _ => None,
        });
        self.compositor.world_environment = match path {
            Some(path) => match self.environment_content_for(&path)?.0 {
                Some(LayerContent::Environment(environment)) => Some(environment),
                _ => None,
            },
            None => None,
        };
        Ok(())
    }

    /// The scene's contributions as material-space pictures the host reads back.
    /// The scene's contributions as material-space pictures the host reads back. `own_frame`: the
    /// scene is an evaluation of its own (a frozen frame at its time) and is lit like a frame; else
    /// it belongs to the frame being evaluated (an analysis read) and makes no capture of its own.
    pub(super) fn prepare_gpu_pictures(&mut self, scene: &SceneValue, comp: CompSpec, projection_camera: ResolvedCamera, own_frame: bool) -> Result<GpuSceneValue, EngineError> {
        self.compositor.refresh_catalog_programs();
        let catalog = self.compositor.catalog.clone();
        let graph = crate::render_lowering::lower_scene_as_pictures(scene, &catalog)
            .map_err(|error| EngineError::Store(error.to_string()))?;
        self.adopt_world_environment(&graph)?;
        let (prepared, _light) = self.as_frame_part(comp, projection_camera, own_frame, |engine| {
            let prepared = engine.execute_render_graph(&graph, comp, projection_camera)?;
            let top = prepared.layers.clone();
            Ok((prepared, top))
        })?;
        Ok(prepared)
    }

    /// Cassette executor: reads only the backend-neutral graph.
    pub(super) fn execute_render_graph(&mut self, graph: &RenderGraph, comp: CompSpec, projection_camera: ResolvedCamera) -> Result<GpuSceneValue, EngineError> {
        let mut prepared = Vec::with_capacity(graph.layers.len());
        for work in &graph.layers {
            prepared.push(self.execute_layer(work, comp, projection_camera)?);
        }
        self.compose_prepared(graph, &prepared, comp, projection_camera)
    }

    /// Clip groups and mattes over prepared contributions, in scene order.
    fn compose_prepared(&mut self, graph: &RenderGraph, prepared: &[Option<LayerWithPasses>], comp: CompSpec, projection_camera: ResolvedCamera) -> Result<GpuSceneValue, EngineError> {
        // A matte or clip group reads its layers' pictures now: plates waiting for the frame's light
        // are baked first, each with its own capture.
        if !self.pending_plates.is_empty() && graph.output.iter().any(|composed| composed.mask.is_some() || !composed.atop.is_empty()) {
            self.plates_needed_early = true;
        }
        let mut groups = std::collections::HashMap::new();
        let mut layers = Vec::with_capacity(graph.output.len());
        let mut layer_ids = Vec::with_capacity(graph.output.len());
        for composed in &graph.output {
            let started = std::time::Instant::now();
            let realized = self.realize_composed(graph, prepared, composed, &mut groups, comp, projection_camera)?;
            if composed.mask.is_some() || !composed.atop.is_empty() {
                let why = if composed.mask.is_some() { "matte composed" } else { "clip group composed" };
                self.ledger.claim("compose", format!("layer {}", graph.layers[composed.base].id.0), why, started.elapsed());
            }
            if let Some(layer) = realized {
                layers.push(layer);
                layer_ids.push(graph.layers[composed.base].id);
            }
        }
        Ok(GpuSceneValue { layers, layer_ids })
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
                    let baked = if self.deferring_plates || self.known_frame_light.is_none() {
                        self.reflectable_member_ids.extend(prepared.layer_ids.iter().copied());
                        self.defer_isolated_layers(comp, camera, prepared.layers, placement, *average, self.picture_density)?
                    } else {
                        self.bake_isolated_layers(comp, camera, prepared.layers, CompositeBlendMode::Normal, placement, *average, self.picture_density)?
                    };
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
            // A plate baked at the views' density carries it too, so its effects keep their size.
            let frame = matches!(work.content, RasterSource::Vector { .. } | RasterSource::Isolate { .. })
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
            ).then_some([0, comp.width, comp.height]);
            self.stamp_feedback(&mut direct_passes, work.id, work.instance, 0, direct_screen);

            let mut plate_passes = work.after_passes.clone();
            let plate_screen = plate_passes.iter()
                .any(|pass| pass.reads_backdrop || pass.reads_composite())
                .then_some([0, comp.width, comp.height]);
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
                self.bake_isolated_layers(comp, camera, sources, CompositeBlendMode::Normal, placement, false, 1.0)?
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
                // Another time is a frame of its own: its plates wait for its one capture.
                let ((content, _), _light) = self.as_own_frame(comp, camera, |engine| {
                    let made = engine.execute_raster(*id, *id, source, None, comp, camera)?;
                    Ok((made, Vec::new()))
                })?;
                let Some(texture) = content.and_then(|content| content.texture().cloned()) else {
                    return Ok(None);
                };
                Ok(self.compositor.snapshot_texture(&texture))
            }
            ImageInput::Graph { graph, background, absent_when_empty, .. } => {
                let (prepared, light) = self.as_own_frame(comp, camera, |engine| {
                    let prepared = engine.execute_render_graph(graph, comp, camera)?;
                    let top = prepared.layers.clone();
                    Ok((prepared, top))
                })?;
                if prepared.layers.is_empty() && *absent_when_empty { return Ok(None); }
                let (texture, _) = self.compositor.bake_picture(comp, camera, &prepared.layers, *background, 1.0, Some(&light), None)?;
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
                        key.screen = Some([0, window.width, window.height]);
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


mod reuse;
pub(super) use reuse::ContributionCache;

#[cfg(test)]
mod tests;


/// What the frame's preparation did, in order: the invariants are read from this, not from pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::engine) enum PreparationEvent {
    /// A frame-level world light capture, by serial.
    Capture(u64),
    /// A plate baked, lit by the capture of this serial (0: unlit, a read within the frame).
    Bake(u64),
}
